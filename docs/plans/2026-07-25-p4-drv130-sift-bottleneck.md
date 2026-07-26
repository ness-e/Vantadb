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

**Prerequisite:** Create VantaFile-backed SIFT 1M benchmark. Then run with `RUST_LOG=debug` to collect profiling data.

---

## Routing

| Task | Sub-agent | Status |
|------|-----------|--------|
| Task 1 (Profile) | `vanta-tuner` | ✅ Done (self-implemented) |
| Task 2 (Prefetch) | `vanta-tuner` | ✅ WONTFIX |
| Task 3 (Reordering) | `vanta-engine` | 🔵 DEFER |
