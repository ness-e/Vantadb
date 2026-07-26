# P4 Engineering Health — DRV-130: SIFT 1M High-Recall 127s Bottleneck

**Goal:** Add profiling instrumentation and identify SSD locality bottleneck in `search_nearest`.

**Result:** Task 1 ✅ implemented, Task 2 ✅ WONTFIX (prefetch exists). Only Task 3 remains (deferred).

---

### ✅ Task 1: Profile the access pattern in search_nearest

**Done.** `SearchProfile` struct with counters for `vfile_reads`, `unique_pages` (4K-aligned), `compute_ns`, `candidates_seen`. Instrumentation added to both hot paths (entry points loop and neighbor evaluation loop) in `search_layer`. Profile logged via `tracing::debug!` at end of each `search_nearest`.

Files: `src/index/search.rs` (+SearchProfile struct, instrumented search_layer, search_nearest), `src/index/graph.rs` (+SearchProfile import, dummy profile in insert_hnsw)

### ✅ Task 2: Prefetch batching — WONTFIX (already exists)

**Already implemented.** `graph::prefetch_mmap_vector()` calls `madvise(MADV_WILLNEED)` on Unix, `PrefetchVirtualMemory` on Windows. Controlled by `PrefetchMode` enum (Auto/Enabled/Disabled) in `src/config.rs:79`. The prefetch loop runs before the neighbor evaluation loop in `search_layer` (graph.rs:124). No work needed.

### 🔵 Task 3: Node reordering for SSD locality — DEFERRED

Requires a benchmark that exercises the real VantaFile I/O path (`vector_store: Some(&vs)`). Current benchmarks all pass `None` (in-memory only). Without profiling data, cannot confirm I/O is the 127s bottleneck.

**Prerequisite:** VantaFile-backed benchmark created at `benches/vfile_search.rs`.

**Benchmark results** (10K vectors, 128d, 200 queries, ef=100, top_k=10):
- `in_memory`: **265 ms** (1.3 ms/q) — in-memory node data
- `with_vfile`: **1.12 s** (5.6 ms/q) — VantaFile-backed, same data
- **Overhead: ~4.2x** from VantaFile header read + byte slicing, even without real disk I/O (in-memory VantaFile)
- `populate_vfile`: **2.6 ms** — vector store population
- T3 remains deferred: node reordering is a large change. Prefetch already mitigates some I/O.

---

## Routing

| Task | Sub-agent | Status |
|------|-----------|--------|
| Task 1 (Profile) | `vanta-tuner` | ✅ Done (self-implemented) |
| Task 2 (Prefetch) | `vanta-tuner` | ✅ WONTFIX |
| Task 3 (Reordering) | `vanta-engine` | 🔵 DEFER |
