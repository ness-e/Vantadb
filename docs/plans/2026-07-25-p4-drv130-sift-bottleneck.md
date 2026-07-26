# P4 Engineering Health — DRV-130: SIFT 1M High-Recall 127s Bottleneck

**Goal:** Add profiling instrumentation and identify SSD locality bottleneck in `search_nearest`.

**Result:** Task 1 ✅ implemented + fix, Task 2 ✅ WONTFIX (prefetch exists), Task 3 ❌ WONTFIX (benchmarked, ~9% below 20% threshold).

---

### ✅ Task 1: Profile the access pattern in search_nearest

**Done.** `SearchProfile` struct with counters for `vfile_reads`, `unique_pages` (4K-aligned), `compute_ns`, `candidates_seen`. Instrumentation added to both hot paths (entry points loop and neighbor evaluation loop) in `search_layer`. Profile logged via `tracing::debug!` at end of each `search_nearest`.

**Fix aplicado (2026-07-26):** SearchProfile original causaba 23% overhead en `pass_none` por ThinLTO que no podía especializar entre `None` vs `Some(vfile)`. Solución: struct gated tras `#[cfg(debug_assertions)]` con ZST no-op en release. Métodos extraídos (`record_vfile_entry`, `record_vfile_candidate`, `start_compute/end_compute`) para encapsular profiling. `if vector_store.is_some()` eliminado del hot path. Benchmarks: `pass_none` 506ms→282ms (-44%), `in_memory` 412ms→306ms (-26%), `with_vfile` 1.546s→1.171s (-24%). Commit `8e747a18`.

Files: `src/index/search.rs` (+SearchProfile struct with cfg gate, ZST no-op, instrumented search_layer, search_nearest), `src/index/graph.rs` (+SearchProfile import, inline SearchProfile::new in insert_hnsw)

### ✅ Task 2: Prefetch batching — WONTFIX (already exists)

**Already implemented.** `graph::prefetch_mmap_vector()` calls `madvise(MADV_WILLNEED)` on Unix, `PrefetchVirtualMemory` on Windows. Controlled by `PrefetchMode` enum (Auto/Enabled/Disabled) in `src/config.rs:79`. The prefetch loop runs before the neighbor evaluation loop in `search_layer` (graph.rs:124). No work needed.

### ❌ Task 3: Node reordering for SSD locality — WONTFIX

**Benchmark results** (10K vectors × 128d × 200 queries, Cosine, ef=100, top_k=10):

| Group | Time | Per query | vs in_memory |
|-------|------|-----------|-------------|
| `in_memory` | 783 ms | 3.9 ms/q | 1x |
| `with_vfile` | 2,440 ms | 12.2 ms/q | 3.1x |
| `with_vfile_compacted` | 2,221 ms | 11.1 ms/q | 2.8x |

**Improvement: ~9%** — below 20% threshold.

**Root cause:** Search follows greedy distance-guided path, not BFS order. Access pattern depends on query vector and graph topology, not storage offset. Overhead is function call overhead (read_header, mmap dereference), not page misses. Prefetch already mitigates I/O.

See `docs/plans/2026-07-25-p4-drv130-t3-node-reordering.md` for full analysis.

---

## Routing

| Task | Sub-agent | Status |
|------|-----------|--------|
| Task 1 (Profile) | `vanta-tuner` | ✅ Done (self-implemented) |
| Task 2 (Prefetch) | `vanta-tuner` | ✅ WONTFIX |
| Task 3 (Reordering) | `vanta-engine` | ❌ WONTFIX |
