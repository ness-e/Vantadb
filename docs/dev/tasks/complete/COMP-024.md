# COMP-024: ACORN-1 Algorithm (Second-Hop Filtered Search)

**Estado:** ✅ COMPLETED — 2026-07-28
**Resultado:** `acorn_expansion: bool` en `search_layer()` (src/index/search.rs:115), 2-hop expansion block con budget `ef.saturating_sub(results.len()).max(16)`, activado solo cuando `!query_mask.is_all_set()`. 3 tests (expands_through_non_matching, no_regression_all_set, budget_respected).

> **Goal:** Implement ACORN-1 (second-hop neighbor expansion during HNSW filtered traversal) to improve recall when filters are moderately selective (InFilter strategy).

## Current State

- COMP-023 (3 filtering strategies) is ✅ complete: PreFilter/InFilter/PostFilter with `FilterStrategy` + `select_filter_strategy()`
- InFilter uses `query_mask: &FilterBitset` in `search_layer` — nodes that don't match are skipped for results but still pushed as candidates
- **Problem:** When many nodes fail the filter, the HNSW greedy walk enters sparse regions where matching nodes are isolated by 2+ hops of non-matching nodes. The visited set prevents re-exploration, so matching clusters are missed.

## ACORN-1 Concept

ACORN-1 (search-time only, no index change):
- When a neighbor fails the filter, immediately expand to THAT neighbor's neighbors (2-hop)
- This maintains connectivity through non-matching regions without reindexing
- Proportional budget: limit second-hop expansion to `ef.saturating_sub(results.len()).max(16)` per non-matching neighbor

## Files to Modify

1. **`src/index/search.rs`** — main change:
   - `search_layer()`: add `acorn_expansion: bool` parameter
   - Add 2-hop expansion block after the existing filter check (line ~328)
   - `search_nearest()`: pass `acorn_expansion = !query_mask.is_all_set()` to layer 0 call
   - Update all existing `search_layer()` callers in the file (upper-layer calls keep `false`)
   - Add tests

## Implementation Detail

### search_layer signature change (line 89):

```rust
pub(crate) fn search_layer(
    &self,
    query_vec: &[f32],
    query_norm: Option<f32>,
    query_inv_norm: Option<f32>,
    entry_points: &[u128],
    ef: usize,
    layer: usize,
    query_mask: &FilterBitset,
    acorn_expansion: bool,  // <-- NEW
    vector_store: Option<&crate::storage::vfile::VantaFile>,
    metric: DistanceMetric,
    visited: &mut std::collections::HashSet<u128, RandomState>,
    profile: &mut SearchProfile,
) -> BinaryHeap<NodeSimMin> {
```

### search_nearest wiring (line 540-552):

The layer 0 search call passes `acorn_expansion` based on query_mask:
```rust
let w = self.search_layer(
    query_vec,
    query_norm,
    query_inv_norm,
    &curr_entry_points,
    ef_search,
    0,
    query_mask,
    !query_mask.is_all_set(),  // <-- ACORN enabled for non-trivial masks
    vector_store,
    effective_metric,
    &mut visited,
    &mut profile,
);
```

Upper-layer calls (lines ~519-537) keep `false`:
```rust
let mut w = self.search_layer(
    query_vec, query_norm, query_inv_norm,
    &curr_entry_points, 1, layer,
    &crate::node::ALL_BITSET,
    false,  // <-- no ACORN on coarse layers
    vector_store, effective_metric,
    &mut visited, &mut profile,
);
```

**NOTE:** All callers of `search_layer` in tests also need the new parameter.

### 2-hop expansion block (inside the neighbor loop, after line ~328):

After computing `d`, checking eligibility, and adding to candidates/results:

```rust
// HACK: To avoid duplicating the 60-line distance computation block,
// we need a different approach. Instead of duplicating, we restructure:
// 1. Keep the distance computation as-is
// 2. After the filter check, if acorn_expansion && !passes_filter,
//    fetch this node's neighbors and evaluate them through the same path

// After line ~330: `if results.len() > ef { results.pop(); }`
//
// Insert the 2-hop expansion:

// ── ACORN-1: second-hop expansion ─────────────────────────────
// When a neighbor fails the filter, expand to its neighbors
// immediately (instead of waiting for it to be popped from the
// heap). This prevents sparse-filter subgraph dead ends.
if acorn_expansion && !query_mask.is_all_set() {
    let passes_filter = query_mask.is_all_set()
        || neighbor.bitset.matches_mask(query_mask);
    if !passes_filter {
        // Fetch the second-hop neighbor list
        let second_hop = self.nodes.get(&neighbor_id)
            .and_then(|n| {
                if layer < n.neighbors.len() {
                    Some(n.neighbors[layer].clone())
                } else {
                    None
                }
            });
        if let Some(second_list) = second_hop {
            // Budget: proportional to remaining result capacity
            let budget = ef.saturating_sub(results.len()).max(16);
            for &second_id in second_list.iter().take(budget) {
                if !visited.contains(&second_id) {
                    visited.insert(second_id);
                    if let Some(second_node) = self.nodes.get(&second_id) {
                        // Distance computation (same pattern as primary loop)
                        let d2 = if let Some(vs) = vector_store {
                            // ... copy the distance computation block
                            // Use the same pattern as lines 238-305
                        } else {
                            self.fast_similarity(
                                query_vec, query_norm, query_inv_norm,
                                &second_node, metric,
                            )
                        };
                        let eligible2 = /* same as lines 307-316 */;
                        if !eligible2 { continue; }

                        if results.len() < ef
                            || results.peek().is_some_and(|worst| d2 > worst.0)
                        {
                            candidates.push(NodeSim(d2, second_id));
                            if second_node.bitset.matches_mask(query_mask) {
                                results.push(NodeSimMin(d2, second_id));
                                if results.len() > ef { results.pop(); }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**CRITICAL:** The distance computation code for the second hop duplicates the 60-line block from lines 238-305. This is necessary because:
- The borrow checker prevents a closure capturing `self` + `candidates` + `results` + `visited`
- A helper function would need all the local variables passed explicitly
- Duplication is the pragmatic ACORN-1 pattern used by Weaviate/Qdrant

### Helper to DRY distance computation

Alternatively, extract a **local closure** that takes `(node_id, &HnswNode, &[f32])` and returns `(f32, bool)` where the bool indicates eligibility. The closure only needs:
- `query_vec`, `query_norm`, `query_inv_norm` (already available)
- `vector_store` (already available)
- `metric` (already available)

```rust
let mut compute_dist_and_eligibility =
    |node_id: u128,
     node: &HnswNode,
     node_data: &[f32]|
     -> Option<(f32, bool)> {
        // ... distance and eligibility check ...
        // Returns None if tombstoned, Some((distance, eligible))
    };
```

## Testing Strategy

### New tests in `src/index/search.rs`:

1. **`test_acorn_expands_through_non_matching`** — Create a 3-node chain: A(0)→B(1)→C(2), where only A and C match the filter. Search with ACORN should find C; without ACORN, should miss C because B is filtered out and B's neighbors are never explored (B is never popped from the heap since ef is small).

2. **`test_acorn_no_regression_all_set`** — When `query_mask.is_all_set()`, acorn=false by default, search should behave identically to before.

3. **`test_acorn_budget_respected`** — Verify that the expansion budget limits second-hop nodes to `ef.saturating_sub(results.len()).max(16)`.

### Helper to add nodes with bitsets in tests:

```rust
fn add_node_with_bitset(
    index: &CPIndex,
    id: u128,
    vec: Vec<f32>,
    bits: &[u32],
) {
    let mut bs = FilterBitset::new();
    for &b in bits {
        bs.set_bit(b as usize);
    }
    index.add(
        id,
        bs,
        VectorRepresentations::Full(vec),
        0,
    );
}
```

## Verification

```bash
cargo test -p vantadb --lib -- search::tests::test_acorn_ 2>&1
cargo check -p vantadb 2>&1
```

Expected: 3 new tests pass, cargo check clean, all 1589+ existing tests still pass.

## Completion Criteria

- [ ] `cargo check -p vantadb` passes
- [ ] ACORN test(s) pass: verify 2-hop expansion finds nodes that pure InFilter misses
- [ ] No regression in filtered search tests
- [ ] ACORN is opt-in via the `acorn_expansion` parameter — default `false` preserves existing behavior
