---
title: "ADR 008: WASM Support Strategy and Browser Deployment"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-22
aliases: []
---

# ADR 008: WASM Support Strategy and Browser Deployment

## Status

Approved.

## Context

Vector databases are increasingly deployed in browser-based and edge-computing contexts: client-side semantic search, in-browser RAG pipelines, and offline-first applications. VantaDB's embedded architecture makes it a candidate for WASM compilation and browser execution, but several constraints must be addressed:

1. **I/O Model:** Browser WASM lacks direct filesystem access. Persistent storage requires the Origin Private File System (OPFS) API or IndexedDB, both asynchronous and operating through JavaScript bridges.
2. **Threading:** WASM does not support OS threads in the standard browser runtime. `std::thread` and `std::sync::Mutex` from the Rust standard library are unavailable.
3. **SIMD:** WASM SIMD (128-bit) is conditionally available and can accelerate distance computations (cosine, dot-product) by 2-4x.
4. **Bundle Size:** WASM binary size directly impacts page load time and must be aggressively optimized.
5. **Execution Environment:** Two distinct targets exist: web browsers (no `std::fs`, no threads) and server-side WASM runtimes (WasmEdge, Wasmtime with WASI support, partial `std::fs`).

## Decision

### 1. WASM Build — InMemory and Persistent Backends (Implemented)

The primary WASM target (`wasm32-unknown-unknown`) uses `BackendKind::InMemory` as the storage engine. All data is kept in `Vec<f32>` and `BTreeMap`-backed collections within the WASM linear memory. This supports:

- Client-side indexing of small datasets (up to ~500K embeddings depending on dimensionality and available memory).
- Full HNSW graph construction and query in the browser.
- Stateless ephemeral use cases: demos, interactive notebooks, and benchmarks.
- **Persistent storage** via explicit `save()` / `load()` to OPFS (Origin Private File System) or IndexedDB backends.

The `InMemory` backend is compiled via:

```rust
#[cfg(target_family = "wasm")]
type Backend = vantadb::storage::InMemory;
```

Three persistent storage backends are implemented and exposed through the SDK:

- **`connect_persistent()`** — OPFS-based persistence (`OpfsStorage` in `vantadb-wasm/src/opfs.rs`). Uses async `FileSystemFileHandle`, `createWritable`, and `getFile` APIs. Supports read, write, append, and delete operations.
- **`connect_idb()`** — IndexedDB persistence (`IdbStorage` in `vantadb-wasm/src/idb.rs`). Inline JS bridge registered at wasm-bindgen time. Includes a `BroadcastChannel("vantadb-sync")` for cross-tab change notifications and a `subscribe()` API. Serves as fallback when OPFS is unavailable.
- **`connect_worker()`** — Web Worker bridge (`OpfsWorkerProxy` in `vantadb-wasm/src/worker.rs`, feature-gated behind `#[cfg(feature = "opfs")]`). Offloads all OPFS I/O to a dedicated thread via `MessageChannel` with typed requests, 5-second timeout, and exponential backoff retry (1s, 2s, max 2 retries).

**Persistence model:** All three backends use a full-dump strategy. `save()` / `save_idb()` serialize every in-memory record into a single `db_state.json` blob. `load()` / `load_idb()` read it back. There is no incremental persistence, no WAL, and no fsync guarantee.

### 2. Future: WASM SIMD Support (Phase 2)

The HNSW distance function will provide a SIMD-accelerated path gated on `#[cfg(target_feature = "simd128")]`. The WASM binary will be distributed in two variants:

- `vantadb.wasm` (baseline, no SIMD)
- `vantadb-simd.wasm` (with SIMD, requires browser support detection at load time)

### 3. Bundle Size Targets

| Configuration | Target Size | Current Status |
|--------------|-------------|----------------|
| Minimal (InMemory, LTO, `-Oz`) | `< 500 KB` gzip | Achieved |
| Full (InMemory + persistent backends) | `< 800 KB` gzip | Achieved (with OPFS + IDB) |
| Full + SIMD variant | `< 900 KB` gzip | Phase 2 — pending SIMD implementation |

### 4. Threading Strategy

VantaDB on WASM runs single-threaded. The `std::sync::Mutex` and `RwLock` usages are compiled out for the WASM target using a `#![cfg(not(target_family = "wasm"))]` gate on all concurrent wrappers. Query and insert operations execute on the calling JavaScript microtask without internal parallelism.

## Consequences

### Benefits

- **Browser-Native Semantic Search:** Applications can run ANN search entirely client-side, eliminating server costs and latency for vector queries.
- **Offline-First Capability:** OPFS and IndexedDB persistence enables fully offline embedding databases that survive browser restarts, suitable for progressive web applications.
- **Multi-Tab Coordination:** The IDB backend's `BroadcastChannel("vantadb-sync")` broadcasts change notifications across tabs via `subscribe()`, enabling basic cross-tab awareness.
- **Minimal Payload:** A 500 KB gzip WASM binary is competitive with pure-JavaScript vector libraries while providing HNSW-level recall quality.

### Technical Debt / Costs

- **Memory Ceiling:** WASM linear memory is limited to 4 GB (practical ceiling ~2 GB). Datasets exceeding ~500K 768-dim vectors require server-side deployment.
- **Single-Threaded Bottleneck:** Insert performance under load is limited to a single core. Concurrent inserts are serialized, and large batch operations block the UI thread unless yielded via `requestIdleFrame` / `setTimeout` scheduling.
- **Full-Dump Persistence:** Every `save()` rewrites the entire dataset. For 1M records, deleting one record still serializes 1M. No incremental persistence, no WAL, no crash recovery.
- **No Atomic Writes:** OPFS `createWritable` + `write` + `close` is not atomic. A browser crash during `save()` leaves `db_state.json` in an indeterminate state. No checksum, no atomic rename.
- **No Web Locks:** Neither OPFS nor IDB backends use `navigator.locks` to coordinate concurrent writes across tabs. Two tabs writing to the same storage will silently corrupt data.
- **Testing Complexity:** WASM-specific test infrastructure (headless Chrome via `wasm-pack test`) must be maintained alongside the native test suite in CI. IDB and Worker backends currently lack dedicated tests.
