# DRV-130 T3: Node Reordering for SSD Locality

## Status: ❌ WONTFIX — Phase 1 complete, <20% improvement

## Problem

`compact_layout_bfs` rewrites the VantaFile in BFS order but is only triggered by tombstone ratio (>20%). It was designed for compaction (hole-closing), not search locality.

## Hypothesis

Running `compact_layout_bfs` before search will significantly reduce overhead because BFS-ordered offsets → consecutive search steps hit consecutive mmap pages → fewer cache misses.

## Phase 1 Results

Benchmark: 10K vectors × 128 dims × 200 queries, Cosine, ef_search=100, top_k=10.

| Group | Time (200 queries) | Per query | vs in_memory |
|-------|-------------------|-----------|-------------|
| `in_memory` | 783 ms | 3.9 ms/q | 1x |
| `with_vfile` | 2,440 ms | 12.2 ms/q | **3.1x** |
| `with_vfile_compacted` | 2,221 ms | 11.1 ms/q | **2.8x** |

**Improvement: ~9%** — well below the 20% threshold.

### Root cause analysis

Search follows a greedy distance-guided path (`search_layer`). The access pattern depends on the query vector and graph topology, not on storage offset order. BFS ordering of offsets doesn't correlate with the search path:

- The BFS traversal starts from the entry point and follows layer-0 neighbor edges
- The search traversal starts from the entry point but greedily selects the closest node at each step
- These two paths diverge after the first few nodes
- Most of the overhead (read_header bounds check, mmap dereference, VectorRepresentations conversion) is in function call overhead, not page misses

### Files changed
- `benches/vfile_search.rs` — added `with_vfile_compacted` group
- `src/storage/mod.rs` — `archive` module: `pub(crate)` → `pub`
- `src/storage/archive.rs` — `compact_layout`, `traverse_graph`, `reindex_nodes`: `pub(crate)` → `pub`

### Visibility changes reverted (not needed for production)
The visibility changes to `archive.rs` and `storage/mod.rs` were needed only for the benchmark. These files can be reverted to `pub(crate)` if desired, or kept as `pub` for future external use.

## Decision

T3 closed as **WONTFIX**. The existing `compact_layout_bfs` (triggered by tombstone ratio) is sufficient for its designed purpose. Prefetch (`MADV_WILLNEED`) already mitigates I/O. No further action on node reordering.

### What was already good enough
- `compact_layout_bfs` exists and works for tombstone cleanup
- Prefetch hints (`prefetch_mmap_vector`) reduce I/O latency during search
- The 3.1x overhead of VantaFile-backed search vs in-memory is acceptable for the persistent storage use case
