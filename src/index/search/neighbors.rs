//! Neighbor selection (top-M pruning) and the flat-search threshold check.

use std::collections::BinaryHeap;

use crate::index::graph::{CPIndex, NeighborVec, NodeSimMin};

impl CPIndex {
    /// Top-M neighbor selection — the single source of truth for HNSW pruning
    /// (AUD-014). Simple top-M, no diversity heuristic (Malkov & Yashunin 2016 §4).
    ///
    /// Ordering is the canonical `NodeSimMin::Ord`: similarity DESCENDING, ties
    /// broken by node id ASCENDING — deterministic across every call path
    /// (insert vs shrink prune). Callers that previously re-implemented this
    /// selection inline must delegate here instead.
    ///
    /// `should_keep` is the post-filter evaluated only for candidates ranked
    /// beyond the top-M: when it returns true the candidate is kept anyway
    /// (over-capacity list). The default `|_| false` reproduces the historical
    /// exactly-M truncation. INV-024 uses it to never evict a node's last
    /// remaining inbound edge.
    ///
    /// Over-capacity is capped at `2 * m` (standard HNSW convention). Without
    /// the cap, `should_keep` may be true for a large fraction of candidates
    /// (e.g. `inbound_count <= 1` during a cold build), letting neighbor lists
    /// grow to O(N) and turning the next build/search into O(N²). With the
    /// cap, extra candidates are bounded by `m` regardless of the filter.
    pub(crate) fn select_neighbors<F>(
        &self,
        candidates: BinaryHeap<NodeSimMin>,
        m: usize,
        should_keep: F,
    ) -> NeighborVec
    where
        F: Fn(u128) -> bool,
    {
        if m == 0 {
            return NeighborVec::new();
        }
        // Full deterministic sort instead of the old select_nth_unstable_by +
        // truncate: the partial sort left top-M keys in arbitrary order and
        // could not express the `should_keep` post-filter. n is bounded by
        // ef_construction (≤ a few hundred) and the candidates come from a
        // heap, so O(n log n) comparisons are negligible next to the distance
        // computations that produced the heap.
        let mut vec = candidates.into_vec();
        vec.sort_unstable();
        let cap = m.saturating_mul(2);
        let mut pruned: NeighborVec = NeighborVec::new();
        for cand in vec {
            if pruned.len() < m || (should_keep(cand.1) && pruned.len() < cap) {
                pruned.push(cand.1);
            }
        }
        pruned
    }

    pub(super) fn use_flat_search(&self) -> bool {
        self.config
            .flat_threshold
            .map(|t| self.nodes.len() <= t)
            .unwrap_or(false)
    }
}
