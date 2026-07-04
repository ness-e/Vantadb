---
title: "ADR 008: WASM Support Strategy and Browser Deployment"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 008: WASM Support Strategy and Browser Deployment

## Status

Status: Approved

## Context

Vector databases are increasingly deployed in browser-based and edge-computing contexts: client-side semantic search, in-browser RAG pipelines, and offline-first applications. VantaDB's embedded architecture makes it a candidate for WASM compilation and browser execution, but several constraints must be addressed:

1. **I/O Model:** Browser WASM lacks direct filesystem access. Persistent storage requires the Origin Private File System (OPFS) API, which is asynchronous and operates through JavaScript bridges.
2. **Threading:** WASM does not support OS threads in the standard browser runtime. `std::thread` and `std::sync::Mutex` from the Rust standard library are unavailable.
3. **SIMD:** WASM SIMD (128-bit) is conditionally available and can accelerate distance computations (cosine, dot-product) by 2-4x.
4. **Bundle Size:** WASM binary size directly impacts page load time and must be aggressively optimized.
5. **Execution Environment:** Two distinct targets exist: web browsers (no `std::fs`, no threads) and server-side WASM runtimes (WasmEdge, Wasmtime with WASI support, partial `std::fs`).

## Decision

1. **Current: InMemory-Only WASM Build (Phase 1):** The initial WASM target (`wasm32-unknown-unknown`) uses the `InMemory` storage backend exclusively. All data is kept in `Vec<f32>` and `HashMap`-backed collections within the WASM linear memory. This supports:
   - Client-side indexing of small datasets (up to ~500K embeddings depending on dimensionality and available memory).
   - Full HNSW graph construction and query in the browser.
   - Stateless ephemeral use cases: demos, interactive notebooks, and benchmarks.

   The `InMemory` backend is compiled via:
   ```rust
   #[cfg(target_family = "wasm")]
   type Backend = vantadb::storage::InMemory;
   ```

2. **Future: OPFS Persistence (Phase 2):** When the Web Platform stabilizes the OPFS API with synchronous access handles (`FileSystemSyncAccessHandle`), VantaDB will introduce an `opfs` storage backend that maps LSM segments to OPFS files. This backend:
   - Operates through a single `SharedArrayBuffer`-mediated bridge to avoid async overhead.
   - Supports persistence across page reloads.
   - Requires `cross-origin-isolated` HTTP headers for `SharedArrayBuffer` access.

3. **Future: WASM SIMD Support (Phase 2):** The HNSW distance function will provide a SIMD-accelerated path gated on `#[cfg(target_feature = "simd128")]`. The WASM binary will be distributed in two variants:
   - `vantadb.wasm` (baseline, no SIMD)
   - `vantadb-simd.wasm` (with SIMD, requires browser support detection at load time)

4. **Bundle Size Targets:**

   | Configuration | Target Size | Current Status |
   |--------------|-------------|----------------|
   | Minimal (InMemory, LTO, `-Oz`)| `< 500 KB` gzip | Achieved |
   | Full (InMemory + OPFS) | `< 800 KB` gzip | Phase 2 |
   | Full + SIMD variant | `< 900 KB` gzip | Phase 2 |

5. **Threading Strategy:** VantaDB on WASM runs single-threaded. The `std::sync::Mutex` and `RwLock` usages are compiled out for the WASM target using a `#![cfg(not(target_family = "wasm"))]` gate on all concurrent wrappers. Query and insert operations execute on the calling JavaScript microtask without internal parallelism.

## Consequences

### Benefits

- **Browser-Native Semantic Search:** Applications can run ANN search entirely client-side, eliminating server costs and latency for vector queries.
- **Offline-First Capability:** Phase 2 OPFS persistence enables fully offline embedding databases that survive browser restarts, suitable for progressive web applications.
- **Minimal Payload:** A 500 KB gzip WASM binary is competitive with pure-JavaScript vector libraries while providing HNSW-level recall quality.

### Technical Debt / Costs

- **Memory Ceiling:** WASM linear memory is limited to 4 GB (practical ceiling ~2 GB). Datasets exceeding ~500K 768-dim vectors require server-side deployment.
- **Single-Threaded Bottleneck:** Insert performance under load is limited to a single core. Concurrent inserts are serialized, and large batch operations block the UI thread unless yielded via `requestIdleFrame` / `setTimeout` scheduling.
- **OPFS API Immaturity:** `FileSystemSyncAccessHandle` is currently available only in dedicated Web Workers (not the main thread) and requires `cross-origin-isolated` headers that many hosting platforms do not set by default.
- **Testing Complexity:** WASM-specific test infrastructure (headless Chrome, Node WASM runner) must be maintained alongside the native test suite in CI.
