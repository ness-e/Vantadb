# COMP-016: Supernode Mitigation (Indexed Relationships)

**Estado:** ✅ COMPLETED — 2026-07-28
**Resultado:** `label_index: HashMap<u32, Vec<u128>>` en `UnifiedNode`. `bfs_traverse_filtered`/`dfs_traverse_filtered` en `GraphTraverser` (src/graph.rs:103). SDK `graph_bfs_filtered`/`graph_dfs_filtered` + WASM + Python. 6 tests.

**Effort:** 🟢 3-5d
**Dependencies:** COMP-006 (Edge Label Interning) ✅
**Type:** Rust core (feature-add)
**Workflow:** feature-add (spec → implement → verify → review → accept → close)

---

## Problem

When a node has 10K+ edges (supernode), graph traversal scans ALL edges linearly
even when only a subset is needed. With COMP-006, each edge has a `label_id` but
no index exists to skip non-matching edges.

**Current O(n) bottleneck** in `src/graph.rs`:
- `bfs_traverse()` iterates `node.edges` fully at each level
- `dfs_traverse()` / `discover_edges()` iterates `node.edges` fully
- `InMemoryEngine::traverse()` (engine.rs:350) filters by label but still O(n)

---

## Solution

Add a label-based secondary index to `UnifiedNode` for O(1) per-label edge lookups.

### Part 1 — `UnifiedNode` changes (`src/node.rs`)

```rust
// New field on UnifiedNode:
pub label_index: HashMap<u32, Vec<u128>>,
// ^ serialized with #[serde(default)] — backward compatible
```

**Methods to add/modify:**

1. `fn rebuild_label_index(&mut self)` — builds `label_index` from `self.edges`
2. `fn ensure_label_index(&mut self)` — builds if empty
3. `fn targets_by_label(&self, label_id: u32) -> &[u128]` — slice of targets for a label
4. `add_edge()` — also insert into `label_index` (maintain both)
5. `add_weighted_edge()` — same

### Part 2 — GraphTraverser changes (`src/graph.rs`)

Add filtered traversal methods:

1. `fn bfs_traverse_filtered(&self, roots: &[u128], max_depth: usize, labels: &[u32]) -> Result<Vec<u128>>`
   - For each node, instead of `for edge in &node.edges`, iterate only edges matching `labels`
   - If node has `label_index` populated, use `targets_by_label()` for O(1) per-label
   - Fallback: scan `node.edges` and filter by `label_id` (for nodes without index)

2. `fn dfs_traverse_filtered(&self, roots: &[u128], max_depth: usize, labels: &[u32]) -> Result<Vec<u128>>`
   - Same approach via `discover_edges_filtered()`

3. `fn discover_edges_filtered(...)` — variant that only caches matching edges

### Part 3 — SDK exposure (`src/sdk/graph.rs`)

Add methods to `VantaEmbedded`:

1. `graph_bfs_filtered(roots, max_depth, labels)`
2. `graph_dfs_filtered(roots, max_depth, labels)`

### Part 4 — Python/WASM bindings

Minimal — if `add_edge` Python binding already accepts label, just ensure it calls
the Rust path that maintains the index. No new Python API methods needed for now.

### Part 5 — Tests

1. **Node index tests** (`src/node.rs` or `tests/core/`):
   - `test_label_index_build_from_edges` — rebuild_label_index() produces correct map
   - `test_label_index_after_add_edge` — add_edge maintains both structures
   - `test_label_index_empty` — empty node returns empty slices

2. **Graph traversal tests** (`src/graph.rs` or `tests/core/graph.rs`):
   - `test_bfs_filtered_basic` — BFS with label filter skips non-matching edges
   - `test_bfs_filtered_supernode` — supernode with mixed labels returns only matching
   - `test_dfs_filtered_basic` — DFS with label filter

3. **Serialization test** (`tests/storage/`):
   - `test_label_index_backward_compat` — old data (no label_index field) loads fine

---

## Key Constraints

- **Backward compat**: `#[serde(default)]` on `label_index` — old data works
- **No O(n) regression**: filtered traversal should be O(label_subset), not O(total_edges)
- **No breaking public API changes**: existing `bfs_traverse()` and `dfs_traverse()` unchanged
- **Keep ponytail**: YAGNI — don't add a full query planner, just the index + filtered methods

## Files to Modify

| File | Changes |
|------|---------|
| `src/node.rs` | Add `label_index` field, `rebuild_label_index()`, `targets_by_label()`, update `add_edge()`/`add_weighted_edge()` |
| `src/graph.rs` | Add `bfs_traverse_filtered()`, `dfs_traverse_filtered()`, `discover_edges_filtered()` |
| `src/sdk/graph.rs` | Add `graph_bfs_filtered()`, `graph_dfs_filtered()` |
| `vantadb-python/src/lib.rs` | Minor: ensure `add_edge` path maintains label_index |

## Verification

```bash
cargo check -p vantadb
cargo test -p vantadb -- graph 2>/dev/null | grep -E "(PASS|FAIL|test.*bfs|test.*dfs|test.*label_index)"
cargo fmt --check
```

## References

- `src/node.rs:805-900` — UnifiedNode struct + edge methods
- `src/graph.rs:10-217` — GraphTraverser + BFS/DFS/discover_edges
- `src/engine.rs:350-394` — existing InMemoryEngine::traverse() with label filtering
- `src/sdk/graph.rs:1-40` — public SDK graph API
- `src/edge_index.rs` — existing global EdgeIndex (different purpose, but pattern reference)
