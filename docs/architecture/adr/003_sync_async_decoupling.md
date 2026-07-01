---
title: "ADR 003: Sync/Async Decoupling and Concurrent Execution Isolation"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-01
---

# ADR 003: Sync/Async Decoupling and Concurrent Execution Isolation

## Status

Status: Approved

## Context

The original design of VantaDB coupled the core library (`vantadb`) with the Tokio async runtime. This forced database consumers (e.g., command-line applications, native synchronous services, or dynamic language wrappers like Python/PyO3) to bootstrap and coordinate a heavy async runtime solely to interact with the local embedded database.
Additionally, mixing intensive disk-blocking and CPU-heavy indexing operations (such as HNSW graph traversal) directly within the Tokio thread pool for network traffic caused starvation of network tasks and severe degradation of P99 server latency.

## Decision

To resolve this architectural bottleneck and ensure a highly portable, industrial-grade embedded engine with stable latencies, a strict two-level execution thread decoupling was implemented:

1. **Synchronous Core Purification (`vantadb`):**
   * Completely remove all Tokio dependencies and abstractions, `async/await`, async communication channels, and futures from the `vantadb` crate.
   * The core compiles as 100% pure synchronous code via the `--no-default-features` flag.
   * All concurrency at the storage and index level is managed internally using highly efficient standard synchronous synchronization primitives (`std::sync::{Arc, RwLock, Mutex}`), enabling clean integration with RocksDB and Fjall.

2. **Isolation at the Server Boundary (`vantadb-server`):**
   * The server (`vantadb-server`) continues to use Tokio for managing network infrastructure, TCP connection dispatching, and the MCP protocol server.
   * However, every call to the underlying synchronous engine is dispatched at the boundary via dedicated blocking threads using the `tokio::task::spawn_blocking` primitive.
   * To prevent uncontrolled system thread exhaustion from bursts of intensive queries, a strict pool and semaphore governed by the `VantaConfig::max_blocking_threads` parameter was implemented:

     ```rust
     let permit = self.blocking_semaphore.acquire().await?;
     let db = self.db.clone();
     let result = tokio::task::spawn_blocking(move || {
         db.execute_hybrid(query)
     }).await?;
     ```

## Consequences

### Benefits

* **Full Integration Portability (Python SDK):** The PyO3 wrapper `vantadb-python` directly interacts with the VantaDB embedded engine in a native synchronous manner, without needing to bootstrap Tokio threads or couple with Python async event loops (asyncio), enabling clean and fast calls.
* **P99 Latency Control Under Stress:** By limiting and isolating the maximum number of concurrent threads that can block CPU or storage, the Tokio network reactor remains free at all times to route TCP connections and respond to MCP requests immediately.
* **Maintainability:** The storage core code is much cleaner, easier to reason about, and debug without unnecessary async abstractions or complex future lifetimes.

### Technical Debt / Costs

* **Context Switch Overhead:** The separation at the Sync/Async boundary introduces a minimal thread context-switch cost when dispatching tasks to `spawn_blocking`. However, the gains in network reactor stability far outweigh this cost.
* **Semaphore Tuning:** The `max_blocking_threads` parameter must be carefully calibrated according to the available physical cores in the deployment hardware to avoid excessive CPU contention or prolonged wait queues on the server.
