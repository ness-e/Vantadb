use ahash::RandomState;
use std::collections::BinaryHeap;

use super::distance::*;
use crate::index::graph::{self, CPIndex, NeighborVec, NodeSim, NodeSimMin};
use crate::index::IndexType;
use crate::node::{DistanceMetric, FilterBitset};
use crate::storage::engine::FLAG_TOMBSTONE;

// E2: Per-thread pool of `NeighborVec` for reuse in `search_layer`.
// Reduces SmallVec heap allocations when neighbor lists exceed 32 elements.
thread_local! {
    static NL_POOL: std::cell::RefCell<Vec<NeighborVec>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Return a `NeighborVec` to the pool for reuse. Clears contents first.
fn give_nl(mut v: NeighborVec) {
    v.clear();
    NL_POOL.with(|pool| pool.borrow_mut().push(v));
}

#[cfg(debug_assertions)]
pub(crate) struct SearchProfile {
    vfile_reads: u64,
    unique_pages: std::collections::HashSet<u64>,
    compute_ns: u64,
    candidates_seen: u64,
    start: std::time::Instant,
    compute_start: std::time::Instant,
}

#[cfg(debug_assertions)]
impl SearchProfile {
    pub(crate) fn new() -> Self {
        Self {
            vfile_reads: 0,
            unique_pages: std::collections::HashSet::new(),
            compute_ns: 0,
            candidates_seen: 0,
            start: std::time::Instant::now(),
            compute_start: std::time::Instant::now(),
        }
    }

    fn start_compute(&mut self) {
        self.compute_start = std::time::Instant::now();
    }

    fn end_compute(&mut self) {
        self.compute_ns += self.compute_start.elapsed().as_nanos() as u64;
    }

    fn record_vfile_candidate(&mut self, storage_offset: u64) {
        self.vfile_reads += 2;
        self.candidates_seen += 1;
        self.unique_pages.insert(storage_offset >> 12);
    }

    fn record_vfile_entry(&mut self, storage_offset: u64) {
        self.vfile_reads += 1;
        self.unique_pages.insert(storage_offset >> 12);
    }

    fn log(&self, ef_search: usize, top_k: usize) {
        let elapsed = self.start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let compute_ms = self.compute_ns as f64 / 1_000_000.0;
        let io_ms = (elapsed_ms - compute_ms).max(0.0);
        tracing::debug!(
            "search_profile: ef={} top_k={} {:.2}ms total ({:.2}ms compute, {:.2}ms io), \
             {} vfile_reads, {} unique_pages, {} candidates",
            ef_search,
            top_k,
            elapsed_ms,
            compute_ms,
            io_ms,
            self.vfile_reads,
            self.unique_pages.len(),
            self.candidates_seen,
        );
    }
}

#[cfg(not(debug_assertions))]
pub(crate) struct SearchProfile;

#[cfg(not(debug_assertions))]
impl SearchProfile {
    pub(crate) fn new() -> Self {
        Self
    }
    fn start_compute(&mut self) {}
    fn end_compute(&mut self) {}
    fn record_vfile_candidate(&mut self, _storage_offset: u64) {}
    fn record_vfile_entry(&mut self, _storage_offset: u64) {}
    fn log(&self, _ef_search: usize, _top_k: usize) {}
}

impl CPIndex {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn search_layer(
        &self,
        query_vec: &[f32],
        query_norm: Option<f32>,
        query_inv_norm: Option<f32>,
        entry_points: &[u128],
        ef: usize,
        layer: usize,
        query_mask: &FilterBitset,
        acorn_expansion: bool,
        vector_store: Option<&crate::storage::vfile::VantaFile>,
        metric: DistanceMetric,
        visited: &mut std::collections::HashSet<u128, RandomState>,
        profile: &mut SearchProfile,
    ) -> BinaryHeap<NodeSimMin> {
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                // ERR-042: read the disk header once per entry point and reuse
                // it for both the distance computation and the tombstone
                // eligibility check below (was 2x read_header per entry point).
                let node_header = vector_store.and_then(|vs| vs.read_header(node.storage_offset));
                let d = if let Some(vs) = vector_store {
                    profile.record_vfile_entry(node.storage_offset);
                    profile.start_compute();
                    let result = if let Some(header) = node_header {
                        if let Some(vec_end) = (header.vector_len as u64)
                            .checked_mul(4)
                            .and_then(|b| header.vector_offset.checked_add(b))
                            .filter(|&end| end <= vs.mmap_bytes().len() as u64)
                            .map(|end| end as usize)
                        {
                            let vec_start = header.vector_offset as usize;
                            let vec_data = &vs.mmap_bytes()[vec_start..vec_end];
                            // SAFETY: 1) bounds — the `vec_end > vs.mmap_bytes().len()`
                            // guard above ensures `vec_start + vector_len*4 <= mmap size`,
                            // so `vec_bytes` is an in-mapping byte slice of exactly
                            // `vector_len*4` bytes; 2) alignment — `read_header` rejects
                            // headers whose `vector_offset` is not a multiple of 4
                            // (INV-024 M-1 central guard), so `vec_bytes.as_ptr()` is
                            // 4-byte aligned, required for a valid `&[f32]`;
                            // 3) lifetime — the borrow keeps the mapping alive.
                            debug_assert_eq!(
                                vec_data.as_ptr().align_offset(4),
                                0,
                                "f32 vector must be 4-byte aligned"
                            );
                            let f32_vec: &[f32] = unsafe {
                                std::slice::from_raw_parts(
                                    vec_data.as_ptr() as *const f32,
                                    header.vector_len as usize,
                                )
                            };
                            match metric {
                                DistanceMetric::Cosine => {
                                    if let Some(q_inv_norm) = query_inv_norm {
                                        let node_inv_norm = node.inv_cached_norm;
                                        if node_inv_norm > f32::EPSILON {
                                            cosine_sim_cached_norms(
                                                query_vec,
                                                q_inv_norm,
                                                f32_vec,
                                                node_inv_norm,
                                            )
                                        } else {
                                            f32_slice_similarity(
                                                query_vec, query_norm, f32_vec, metric,
                                            )
                                        }
                                    } else {
                                        f32_slice_similarity(query_vec, query_norm, f32_vec, metric)
                                    }
                                }
                                DistanceMetric::Euclidean => {
                                    -euclidean_distance_squared_f32(query_vec, f32_vec)
                                }
                                // SparseDot has its own brute-force path.
                                DistanceMetric::SparseDot => 0.0,
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    profile.end_compute();
                    result
                } else {
                    self.fast_similarity(query_vec, query_norm, query_inv_norm, &node, metric)
                };

                let eligible = if vector_store.is_some() {
                    node_header
                        .map(|h| (h.flags & FLAG_TOMBSTONE) == 0)
                        .unwrap_or(false)
                } else {
                    (node.flags & FLAG_TOMBSTONE) == 0
                };
                if !eligible {
                    continue;
                }

                candidates.push(NodeSim(d, ep));
                if query_mask.is_all_set() || node.bitset.matches_mask(query_mask) {
                    results.push(NodeSimMin(d, ep));
                }
                visited.insert(ep);
            }
        }

        while let Some(NodeSim(d_cand, cand_id)) = candidates.pop() {
            if results.len() >= ef {
                if let Some(worst) = results.peek() {
                    if d_cand < worst.0 {
                        break;
                    }
                }
            }

            // Inline neighbor cache: try the node's own neighbor_lists first
            // (avoids a separate DashMap read on neighbor_index).
            // Only use inline cache if the list is non-empty — empty lists mean
            // the node hasn't had neighbors set yet (e.g. first node, or during
            // concurrent insert), so fall back to neighbor_index.
            // ERR-047: borrow the inline list instead of copying it into the
            // per-thread pool (one O(M) alloc saved per expanded candidate).
            // The nodes guard above stays alive for the whole expansion below,
            // so the Cow borrow is valid while the neighbor_index fallback maps
            // to an owned copy.
            let node_guard = self.nodes.get(&cand_id);
            let neighbors: Option<std::borrow::Cow<'_, NeighborVec>> = match &node_guard {
                Some(n) => n
                    .neighbor_lists
                    .get(layer)
                    .filter(|l| !l.is_empty())
                    .map(std::borrow::Cow::Borrowed),
                None => None,
            };
            let neighbors = match neighbors {
                Some(c) => Some(c),
                None => self
                    .neighbor_index
                    .get_neighbors(cand_id, layer)
                    .map(std::borrow::Cow::Owned),
            };

            if let Some(neighbors_list) = neighbors {
                if graph::should_prefetch() {
                    if let Some(vs) = vector_store {
                        let mmap_base = vs.mmap_bytes().as_ptr();
                        let mmap_len = vs.mmap_bytes().len();
                        for &pf_neighbor_id in neighbors_list.iter() {
                            if !visited.contains(&pf_neighbor_id) {
                                if let Some(pf_node) = self.nodes.get(&pf_neighbor_id) {
                                    if let Some(h) = vs.read_header(pf_node.storage_offset) {
                                        let Some(vec_len_bytes) =
                                            (h.vector_len as u64).checked_mul(4)
                                        else {
                                            continue;
                                        };
                                        let Some(vec_end) =
                                            h.vector_offset.checked_add(vec_len_bytes)
                                        else {
                                            continue;
                                        };
                                        if vec_end <= mmap_len as u64 && vec_len_bytes > 0 {
                                            graph::prefetch_mmap_vector(
                                                mmap_base,
                                                h.vector_offset as usize,
                                                vec_len_bytes as usize,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                for &neighbor_id in neighbors_list.iter() {
                    // ERR-048: single hash lookup — insert() returns true if the
                    // id was not already present, replacing contains + insert.
                    if visited.insert(neighbor_id) {
                        if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                            // ERR-042: read the disk header once per candidate
                            // and reuse it for both the distance computation and
                            // the tombstone eligibility check below (was 2x
                            // read_header per candidate in this hot loop).
                            let node_header =
                                vector_store.and_then(|vs| vs.read_header(neighbor.storage_offset));
                            let d = if let Some(vs) = vector_store {
                                profile.record_vfile_candidate(neighbor.storage_offset);
                                profile.start_compute();
                                let result = if let Some(h) = node_header {
                                    if let Some(vec_end) = (h.vector_len as u64)
                                        .checked_mul(4)
                                        .and_then(|b| h.vector_offset.checked_add(b))
                                        .filter(|&end| end <= vs.mmap_bytes().len() as u64)
                                        .map(|end| end as usize)
                                    {
                                        let vec_start = h.vector_offset as usize;
                                        let v_data = &vs.mmap_bytes()[vec_start..vec_end];
                                        // SAFETY: 1) bounds — the `vec_end > vs.mmap_bytes().len()`
                                        // guard above ensures `h.vector_len * 4` does not exceed
                                        // the mapping, so `v_data` is an in-mapping byte slice
                                        // of exactly `vector_len*4` bytes; 2) alignment —
                                        // `read_header` rejects non-4-multiple `vector_offset`
                                        // (INV-024 M-1 central guard), so `v_data.as_ptr()` is
                                        // 4-byte aligned; 3) lifetime — the borrow keeps the
                                        // mapping alive.
                                        debug_assert_eq!(
                                            v_data.as_ptr().align_offset(4),
                                            0,
                                            "f32 neighbor vector must be 4-byte aligned"
                                        );
                                        let f32_v: &[f32] = unsafe {
                                            std::slice::from_raw_parts(
                                                v_data.as_ptr() as *const f32,
                                                h.vector_len as usize,
                                            )
                                        };
                                        match metric {
                                            DistanceMetric::Cosine => {
                                                if let Some(q_inv_norm) = query_inv_norm {
                                                    let neighbor_inv_norm =
                                                        neighbor.inv_cached_norm;
                                                    if neighbor_inv_norm > f32::EPSILON {
                                                        cosine_sim_cached_norms(
                                                            query_vec,
                                                            q_inv_norm,
                                                            f32_v,
                                                            neighbor_inv_norm,
                                                        )
                                                    } else {
                                                        f32_slice_similarity(
                                                            query_vec, query_norm, f32_v, metric,
                                                        )
                                                    }
                                                } else {
                                                    f32_slice_similarity(
                                                        query_vec, query_norm, f32_v, metric,
                                                    )
                                                }
                                            }
                                            DistanceMetric::Euclidean => {
                                                -euclidean_distance_squared_f32(query_vec, f32_v)
                                            }
                                            DistanceMetric::SparseDot => 0.0,
                                        }
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                };
                                profile.end_compute();
                                result
                            } else {
                                self.fast_similarity(
                                    query_vec,
                                    query_norm,
                                    query_inv_norm,
                                    &neighbor,
                                    metric,
                                )
                            };

                            let eligible = if vector_store.is_some() {
                                node_header
                                    .map(|h| (h.flags & FLAG_TOMBSTONE) == 0)
                                    .unwrap_or(false)
                            } else {
                                (neighbor.flags & FLAG_TOMBSTONE) == 0
                            };
                            if !eligible {
                                continue;
                            }

                            if results.len() < ef || results.peek().is_some_and(|worst| d > worst.0)
                            {
                                candidates.push(NodeSim(d, neighbor_id));
                                if query_mask.is_all_set()
                                    || neighbor.bitset.matches_mask(query_mask)
                                {
                                    results.push(NodeSimMin(d, neighbor_id));
                                    if results.len() > ef {
                                        results.pop();
                                    }
                                }

                                // ── ACORN-1: second-hop expansion ──
                                // When a neighbor fails the filter, immediately expand to its
                                // neighbors to maintain filtered subgraph connectivity through
                                // sparse non-matching regions.
                                // Uses inline neighbor cache (neighbor is already loaded from
                                // nodes.get above — no extra DashMap access).
                                if acorn_expansion && !query_mask.is_all_set() {
                                    let passes_filter = query_mask.is_all_set()
                                        || neighbor.bitset.matches_mask(query_mask);
                                    if !passes_filter {
                                        let second_hop: Option<std::borrow::Cow<'_, NeighborVec>> =
                                            neighbor
                                                .neighbor_lists
                                                .get(layer)
                                                .filter(|l| !l.is_empty())
                                                .map(std::borrow::Cow::Borrowed)
                                                .or_else(|| {
                                                    self.neighbor_index
                                                        .get_neighbors(neighbor_id, layer)
                                                        .map(std::borrow::Cow::Owned)
                                                });
                                        if let Some(second_list) = second_hop {
                                            let budget = ef.saturating_sub(results.len()).max(16);
                                            for &second_id in second_list.iter().take(budget) {
                                                // ERR-048: single hash lookup via insert().
                                                if visited.insert(second_id) {
                                                    if let Some(second_node) =
                                                        self.nodes.get(&second_id)
                                                    {
                                                        let d2 = self.fast_similarity(
                                                            query_vec,
                                                            query_norm,
                                                            query_inv_norm,
                                                            &second_node,
                                                            metric,
                                                        );
                                                        let eligible2 = (second_node.flags
                                                            & FLAG_TOMBSTONE)
                                                            == 0;
                                                        if !eligible2 {
                                                            continue;
                                                        }
                                                        if results.len() < ef
                                                            || results
                                                                .peek()
                                                                .is_some_and(|worst| d2 > worst.0)
                                                        {
                                                            candidates.push(NodeSim(d2, second_id));
                                                            if second_node
                                                                .bitset
                                                                .matches_mask(query_mask)
                                                            {
                                                                results.push(NodeSimMin(
                                                                    d2, second_id,
                                                                ));
                                                                if results.len() > ef {
                                                                    results.pop();
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // E2: Return second_hop to pool (owned only —
                                            // borrowed inline cache must not be pooled).
                                            if let std::borrow::Cow::Owned(v) = second_list {
                                                give_nl(v);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // E2: Return the NeighborVec to the thread-local pool for reuse
                // (owned only — borrowed inline cache must not be pooled).
                if let std::borrow::Cow::Owned(v) = neighbors_list {
                    give_nl(v);
                }
            }
        }
        results
    }

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

    fn use_flat_search(&self) -> bool {
        self.config
            .flat_threshold
            .map(|t| self.nodes.len() <= t)
            .unwrap_or(false)
    }

    #[tracing::instrument(skip(self, query_vec, vector_store), level = "debug")]
    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        _q_1bit: Option<&[u64]>,
        _q_3bit: Option<(&[u8], f32)>,
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&crate::storage::vfile::VantaFile>,
    ) -> Vec<(u128, f32)> {
        // AUDREP-55: a zero-norm query is undefined under cosine similarity
        // (the cosine of a zero vector is 0/0). Historically this silently
        // fell back to Euclidean scoring, which swaps the score range
        // (cosine ∈ [-1, 1] vs euclidean ∈ (-∞, 0]) so the caller's score
        // thresholds become meaningless across calls. This function (and the
        // VecIndex::search trait) return a plain Vec, so an error cannot be
        // propagated cleanly at this boundary; instead we fail loudly and
        // return no results, keeping the cosine metric pure — consistent with
        // AUDREP-27, which rejects zero-norm inserts under cosine.
        if self.config.distance_metric == DistanceMetric::Cosine
            && f32_l2_norm(query_vec) < f32::EPSILON
        {
            tracing::warn!(
                "zero-norm cosine query is undefined; returning no results \
                 (AUDREP-55, was silently re-scored with euclidean)"
            );
            return Vec::new();
        }

        // IVF path: lazy-build on first search, then search. AUDREP-09:
        // rebuild whenever the node count changed since the last build, so
        // vectors added after a cached IVF was built become candidates.
        if self.config.index_type == IndexType::Ivf {
            return self.search_ivf(query_vec, query_mask, top_k);
        }

        // SCANN (SQ8) path: same lazy-build pattern as IVF. Without this the
        // configured `index_type = Scann` would silently fall through to the
        // HNSW graph, ignoring the selected backend entirely.
        if self.config.index_type == IndexType::Scann {
            return self.search_scann(query_vec, query_mask, top_k);
        }

        if self.use_flat_search() {
            return crate::index::flat::flat_search(
                &self.nodes,
                query_vec,
                query_mask,
                top_k,
                self.config.distance_metric,
            );
        }

        let ep = match self.get_entry_point() {
            Some(id) => id,
            None => return Vec::new(),
        };

        let static_ef = self.config.ef_search;
        let ef_search = if self.config.auto_tune {
            let tuned_ef = crate::index::auto_tune::AutoTune::current_ef();
            static_ef.max(tuned_ef).max(top_k)
        } else {
            static_ef.max(top_k)
        };
        let (effective_metric, query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                // AUDREP-55 guard at the top of search_nearest guarantees
                // norm >= f32::EPSILON here, so 1/norm is finite and the
                // metric stays Cosine — no silent Euclidean fallback remains.
                let norm = f32_l2_norm(query_vec);
                debug_assert!(
                    norm >= f32::EPSILON,
                    "zero-norm cosine query must be rejected up-front (AUDREP-55)"
                );
                (DistanceMetric::Cosine, Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(query_vec);
                (DistanceMetric::Euclidean, Some(norm), None)
            }
            DistanceMetric::SparseDot => {
                // Sparse has its own brute-force search path; never routed through
                // the dense HNSW query. Degenerate norm pair if reached.
                (DistanceMetric::SparseDot, None, None)
            }
        };
        let mut curr_entry_points = vec![ep];
        let mut visited: std::collections::HashSet<u128, RandomState> =
            std::collections::HashSet::with_capacity_and_hasher(
                ef_search.max(top_k).saturating_mul(3),
                RandomState::new(),
            );

        let mut profile = SearchProfile::new();
        let max_l = self.max_layer.load(std::sync::atomic::Ordering::Acquire);
        for layer in (1..=max_l).rev() {
            visited.clear();
            let mut w = self.search_layer(
                query_vec,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                &crate::node::ALL_BITSET,
                false, // no ACORN on coarse layers
                vector_store,
                effective_metric,
                &mut visited,
                &mut profile,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        visited.clear();
        let w = self.search_layer(
            query_vec,
            query_norm,
            query_inv_norm,
            &curr_entry_points,
            ef_search,
            0,
            query_mask,
            !query_mask.is_all_set(), // ACORN enabled for non-trivial masks
            vector_store,
            effective_metric,
            &mut visited,
            &mut profile,
        );
        profile.log(ef_search, top_k);

        let mut result: Vec<NodeSimMin> = w.into_sorted_vec();
        result.retain(|ns| !ns.0.is_nan());

        result.truncate(top_k);

        let mut final_results = Vec::with_capacity(result.len());
        for NodeSimMin(score, id) in result {
            let adjusted_score = match effective_metric {
                DistanceMetric::Euclidean => -(-score).max(0.0).sqrt(),
                DistanceMetric::Cosine => score,
                DistanceMetric::SparseDot => score,
            };
            final_results.push((id, adjusted_score));
        }
        final_results
    }

    /// Lazy-build and search the IVF index (AUDREP-09). Shared by
    /// `search_nearest` (config `index_type = Ivf`) and per-search overrides
    /// from bindings (`method = "ivf"`).
    pub(crate) fn search_ivf(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
    ) -> Vec<(u128, f32)> {
        let mut guard = self.ivf_index.lock();
        let node_count = self.nodes.len();
        if guard.as_ref().is_none_or(|_| {
            self.ivf_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed)
                != node_count
        }) {
            let ivf_config = crate::index::ivf::IvfConfig {
                nlist: (node_count as f64).sqrt() as usize + 1,
                nprobe: 10,
                distance_metric: self.config.distance_metric,
            };
            *guard = Some(crate::index::ivf::IvfIndex::build(&self.nodes, &ivf_config));
            self.ivf_built_at_node_count
                .store(node_count, std::sync::atomic::Ordering::Relaxed);
        }
        match guard.as_ref() {
            Some(ivf) => ivf.search(query_vec, top_k, query_mask),
            None => Vec::new(),
        }
    }

    /// Lazy-build and search the SCANN (SQ8) index. Mirrors the IVF lazy
    /// cache: rebuilt whenever the node count diverges from the last build.
    pub(crate) fn search_scann(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
    ) -> Vec<(u128, f32)> {
        let mut guard = self.scann_index.lock();
        let node_count = self.nodes.len();
        if guard.as_ref().is_none_or(|_| {
            self.scann_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed)
                != node_count
        }) {
            let scann = crate::index::scann::ScannIndex::new(self.config.distance_metric);
            for entry in self.nodes.iter() {
                let node = entry.value();
                if let crate::node::VectorRepresentations::Full(v) = &node.vec_data {
                    crate::index::VecIndex::add(
                        &scann,
                        node.id,
                        node.bitset.clone(),
                        crate::node::VectorRepresentations::Full(v.clone()),
                        node.storage_offset,
                    );
                }
            }
            *guard = Some(scann);
            self.scann_built_at_node_count
                .store(node_count, std::sync::atomic::Ordering::Relaxed);
        }
        match guard.as_ref() {
            Some(scann) => crate::index::VecIndex::search(
                scann,
                query_vec,
                query_mask,
                top_k,
                None,
                self.config.distance_metric,
            ),
            None => Vec::new(),
        }
    }

    /// Run a search through an explicit index backend, ignoring the engine's
    /// configured `index_type`. Used by per-search `method` overrides from
    /// bindings; the shared `config` is never mutated (thread-safe).
    pub(crate) fn search_with_method(
        &self,
        method: IndexType,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
    ) -> Vec<(u128, f32)> {
        match method {
            IndexType::Ivf => self.search_ivf(query_vec, query_mask, top_k),
            IndexType::Scann => self.search_scann(query_vec, query_mask, top_k),
            IndexType::Flat => crate::index::flat::flat_search(
                &self.nodes,
                query_vec,
                query_mask,
                top_k,
                self.config.distance_metric,
            ),
            IndexType::Hnsw | IndexType::DiskAnn => {
                self.search_nearest(query_vec, None, None, query_mask, top_k, None)
            }
        }
    }
}

impl crate::index::VecIndex for CPIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &crate::node::FilterBitset,
        top_k: usize,
        vector_store: Option<&crate::storage::vfile::VantaFile>,
        _distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)> {
        // CPIndex already knows its distance metric from config;
        // the _distance_metric argument is accepted for trait compatibility
        // with index types that don't carry their own config (e.g. FlatIndex).
        self.search_nearest(query_vec, None, None, query_mask, top_k, vector_store)
    }

    fn add(
        &self,
        id: u128,
        bitset: crate::node::FilterBitset,
        vec_data: crate::node::VectorRepresentations,
        storage_offset: u64,
    ) {
        // `VecIndex::add` returns `()`, so a rejected insert (e.g. zero-norm
        // vector under cosine, AUDREP-27) cannot be propagated here; surface
        // it loudly instead of dropping it silently.
        if let Err(e) = CPIndex::add(self, id, bitset, vec_data, storage_offset) {
            tracing::warn!(id, error = %e, "index add rejected at VecIndex boundary");
        }
    }

    fn estimate_memory_bytes(&self) -> usize {
        CPIndex::estimate_memory_bytes(self)
    }

    fn len(&self) -> usize {
        self.total_nodes.load(std::sync::atomic::Ordering::Relaxed) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::graph::HnswConfig;
    use crate::node::{VectorRepresentations, ALL_BITSET};

    fn make_index(metric: DistanceMetric) -> CPIndex {
        CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: metric,
            ..HnswConfig::default()
        })
    }

    fn add_node(index: &CPIndex, id: u128, vec: Vec<f32>) {
        index
            .add(id, FilterBitset::new(), VectorRepresentations::Full(vec), 0)
            .expect("test vectors are non-zero-norm");
    }

    // ── search_nearest ──────────────────────────────────────────────

    #[test]
    fn test_search_nearest_empty_index() {
        let index = make_index(DistanceMetric::Cosine);
        let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert!(results.is_empty(), "empty index should return no results");
    }

    #[test]
    fn test_search_nearest_single_node_cosine() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 42, vec![1.0, 0.0, 0.0]);
        let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 1, "single node should be found");
        assert_eq!(results[0].0, 42, "id should match");
        assert!(
            results[0].1 > 0.99,
            "self-similarity should be ~1.0, got {}",
            results[0].1
        );
    }

    #[test]
    fn test_search_nearest_single_node_euclidean() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 7, vec![3.0, 4.0]);
        let results = index.search_nearest(&[3.0, 4.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 1, "single node should be found");
        assert_eq!(results[0].0, 7, "id should match");
        assert!(
            results[0].1.abs() < 0.01,
            "self-distance should be ~0.0, got {}",
            results[0].1
        );
    }

    #[test]
    fn test_search_nearest_ordering_cosine() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.9, 0.1, 0.0]);
        add_node(&index, 2, vec![-1.0, 0.0, 0.0]);
        let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0, "identical vector should be first");
        for win in results.windows(2) {
            assert!(
                win[0].1 >= win[1].1 - 1e-6,
                "scores should be descending: {} < {}",
                win[0].1,
                win[1].1
            );
        }
    }

    #[test]
    fn test_search_nearest_top_k_limits() {
        let index = make_index(DistanceMetric::Cosine);
        for i in 0..10u128 {
            add_node(&index, i, vec![i as f32 + 1.0, 0.0, 0.0]);
        }
        let results = index.search_nearest(&[0.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
        assert!(
            results.len() <= 3,
            "should not exceed top_k, got {}",
            results.len()
        );
    }

    #[test]
    fn test_search_nearest_zero_top_k() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 1, vec![1.0, 0.0]);
        let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 0, None);
        assert!(
            results.is_empty(),
            "top_k=0 should return empty, got {}",
            results.len()
        );
    }

    #[test]
    fn test_search_nearest_scores_not_nan() {
        let index = make_index(DistanceMetric::Cosine);
        for i in 0..5u128 {
            add_node(&index, i, vec![(i as f32) * 0.5, 0.3, -0.1]);
        }
        let results = index.search_nearest(&[0.5, -0.2, 0.1], None, None, &ALL_BITSET, 5, None);
        for &(id, score) in &results {
            assert!(
                score.is_finite(),
                "score for id={} should be finite, got {}",
                id,
                score
            );
        }
    }

    #[test]
    fn test_search_nearest_euclidean_negative_scores() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![0.0, 0.0]);
        add_node(&index, 1, vec![10.0, 10.0]);
        let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 2, None);
        for &(_, score) in &results {
            assert!(
                score <= 0.0,
                "Euclidean scores should be <= 0, got {}",
                score
            );
        }
    }

    #[test]
    fn test_search_nearest_closer_first_euclidean() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![0.0, 0.0]);
        add_node(&index, 1, vec![1.0, 1.0]);
        add_node(&index, 2, vec![10.0, 10.0]);
        let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 3, None);
        assert_eq!(results[0].0, 0, "closest vector (id=0) should be first");
        assert_eq!(results[2].0, 2, "farthest (id=2) should be last");
    }

    // ── select_neighbors ────────────────────────────────────────────

    #[test]
    fn test_select_neighbors_empty_candidates() {
        let index = make_index(DistanceMetric::Cosine);
        let heap = BinaryHeap::new();
        let selected = index.select_neighbors(heap, 5, |_| false);
        assert!(
            selected.is_empty(),
            "empty candidates should produce empty selection"
        );
    }

    #[test]
    fn test_select_neighbors_returns_top_m() {
        let index = make_index(DistanceMetric::Cosine);
        for i in 0..6u128 {
            add_node(&index, i, vec![(i as f32 + 1.0) * 0.2, 0.0, 0.0]);
        }
        let mut heap = BinaryHeap::new();
        for i in 0..6u128 {
            // node compared with itself has sim=1.0
            if let Some(node) = index.nodes.get(&i) {
                if let Some(slice) = node.vec_data.as_f32_slice() {
                    let sim = cosine_sim_f32(slice, slice);
                    heap.push(NodeSimMin(sim, i));
                }
            }
        }
        let selected = index.select_neighbors(heap, 3, |_| false);
        assert_eq!(selected.len(), 3, "should select top 3 from 6 candidates");
    }

    #[test]
    fn test_select_neighbors_with_tombstone_skipped() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.0, 1.0, 0.0]);
        // Mark node 0 as tombstone
        if let Some(mut n) = index.nodes.get_mut(&0) {
            n.flags |= FLAG_TOMBSTONE;
        }
        let mut heap = BinaryHeap::new();
        heap.push(NodeSimMin(1.0, 0));
        heap.push(NodeSimMin(0.5, 1));
        // With top-M selection, tombstone filtering is skipped.
        // During rebuild, tombstones don't appear in the candidate set
        // (they are filtered out during vstore scan).
        let selected = index.select_neighbors(heap, 5, |_| false);
        assert_eq!(selected.len(), 2, "top-M selects both candidates by score");
        assert!(selected.contains(&0), "top-M does not filter tombstones");
    }

    #[test]
    fn test_select_neighbors_keeps_best_scores() {
        // Regression test: B2 (1379343b) inverted the select_nth_unstable_by
        // comparator, which left the m SMALLEST-score elements in vec[0..m] —
        // i.e. the m WORST neighbors — instead of the m best. Every insert and
        // shrink then built edges to the worst candidates, degrading topology.
        let index = make_index(DistanceMetric::Cosine);
        let mut heap = BinaryHeap::new();
        // Higher score = better neighbor.
        heap.push(NodeSimMin(0.1, 1));
        heap.push(NodeSimMin(0.9, 2));
        heap.push(NodeSimMin(0.5, 3));
        heap.push(NodeSimMin(0.7, 4));
        heap.push(NodeSimMin(0.3, 5));
        let selected = index.select_neighbors(heap, 2, |_| false);
        let mut ids: Vec<u128> = selected.iter().copied().collect();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec![2, 4],
            "must keep the 2 BEST candidates (scores 0.9, 0.7), got {ids:?}"
        );
    }

    // ── use_flat_search ──────────────────────────────────────────────────

    #[test]
    fn test_use_flat_search_none_threshold() {
        let index = CPIndex::new_with_config(HnswConfig {
            flat_threshold: None,
            ..HnswConfig::default()
        });
        assert!(
            !index.use_flat_search(),
            "None threshold should always return false"
        );
    }

    #[test]
    fn test_use_flat_search_some_zero_threshold() {
        let index = CPIndex::new_with_config(HnswConfig {
            flat_threshold: Some(0),
            ..HnswConfig::default()
        });
        // Empty index: 0 <= 0 → true
        assert!(
            index.use_flat_search(),
            "empty + 0 threshold should be true"
        );
        // Add node: 1 <= 0 → false
        add_node(&index, 1, vec![1.0, 0.0]);
        assert!(
            !index.use_flat_search(),
            "1 node + 0 threshold should be false"
        );
    }

    #[test]
    fn test_use_flat_search_default_threshold() {
        let index = make_index(DistanceMetric::Cosine);
        // Default threshold = Some(10000), small indexes are below it
        assert!(
            index.use_flat_search(),
            "small index should use flat search by default"
        );
    }

    // ── AUDREP-55: zero-norm cosine queries ────────────────────────────

    #[test]
    fn test_search_nearest_zero_norm_cosine_rejected() {
        // Zero-norm query under cosine is undefined (0/0). It must not be
        // silently re-scored with euclidean (score range would swap and
        // callers could no longer compare scores across calls); it must
        // return no results, deterministically, for any zero vector length.
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 1, vec![1.0, 0.0, 0.0]);

        for query in [
            vec![0.0_f32, 0.0],
            vec![0.0_f32, 0.0, 0.0],
            vec![0.0_f32, -0.0, 0.0, 0.0],
        ] {
            let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
            assert!(
                results.is_empty(),
                "zero-norm cosine query must return no results, got {results:?}"
            );
        }

        // Contrast: zero-norm queries remain valid for other metrics — the
        // guard must only fire under cosine.
        let euc = make_index(DistanceMetric::Euclidean);
        add_node(&euc, 1, vec![3.0, 4.0]);
        let results = euc.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 1, "zero-norm euclidean query is valid");
    }

    #[test]
    fn test_search_nearest_hnsw_zero_norm_cosine_rejected() {
        // Same contract via the HNSW path (flat_threshold = None) — the
        // guard lives at the search_nearest entry point, before any
        // flat/IVF/HNSW routing.
        let index = make_hnsw_index(DistanceMetric::Cosine);
        add_node(&index, 1, vec![1.0, 0.0, 0.0]);

        for query in [vec![0.0_f32, 0.0, 0.0], vec![0.0_f32, 0.0]] {
            let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
            assert!(
                results.is_empty(),
                "HNSW zero-norm cosine query must return no results, got {results:?}"
            );
        }

        let euc = make_hnsw_index(DistanceMetric::Euclidean);
        add_node(&euc, 1, vec![3.0, 4.0]);
        let results = euc.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 1, "HNSW zero-norm euclidean query is valid");
    }

    // ── IVF lazy-build invalidation (AUDREP-09) ────────────────────────

    fn make_ivf_index(metric: DistanceMetric) -> CPIndex {
        CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: metric,
            flat_threshold: None,
            index_type: crate::index::IndexType::Ivf,
            auto_tune: false,
        })
    }

    #[test]
    fn test_ivf_rebuilds_when_nodes_added_after_build() {
        let index = make_ivf_index(DistanceMetric::Euclidean);
        for id in 0..20_u128 {
            add_node(&index, id, vec![id as f32 * 0.5, id as f32 * 0.5]);
        }
        // Force the lazy IVF build on the first search.
        index.search_nearest(&[0.5, 0.5], None, None, &ALL_BITSET, 5, None);
        assert!(
            index.ivf_index.lock().is_some(),
            "IVF should be built after first search"
        );
        assert_eq!(
            index
                .ivf_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed),
            20,
            "IVF built over the initial 20 nodes"
        );

        // Add a new, far vector after the build, then search for it.
        index
            .add(
                999_u128,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![999.0, 999.0]),
                0,
            )
            .expect("test vector is non-zero-norm");
        let results = index.search_nearest(&[999.0, 999.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(
            index
                .ivf_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed),
            21,
            "cached IVF must be rebuilt after node count changed"
        );
        assert!(
            results.iter().any(|(id, _)| *id == 999_u128),
            "newly added vector must be a candidate after rebuild, got {results:?}"
        );
    }

    // ── search_nearest via HNSW path (flat_threshold = None) ────────────

    fn make_hnsw_index(metric: DistanceMetric) -> CPIndex {
        CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: metric,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        })
    }

    #[test]
    fn test_search_nearest_hnsw_empty_index() {
        let index = make_hnsw_index(DistanceMetric::Cosine);
        let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert!(
            results.is_empty(),
            "empty index via HNSW should return empty"
        );
    }

    #[test]
    fn test_search_nearest_hnsw_cosine() {
        let index = make_hnsw_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.9, 0.1, 0.0]);
        add_node(&index, 2, vec![-1.0, 0.0, 0.0]);
        let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0, "identical vector should be first via HNSW");
        for win in results.windows(2) {
            assert!(
                win[0].1 >= win[1].1 - 1e-6,
                "scores should be descending via HNSW: {} < {}",
                win[0].1,
                win[1].1
            );
        }
        for &(_, score) in &results {
            assert!(
                score.is_finite(),
                "HNSW score should be finite, got {}",
                score
            );
        }
    }

    #[test]
    fn test_search_nearest_hnsw_euclidean() {
        let index = make_hnsw_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![0.0, 0.0]);
        add_node(&index, 1, vec![3.0, 4.0]);
        add_node(&index, 2, vec![-5.0, -5.0]);
        let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 3, None);
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0].0, 0,
            "self-vector should be first via HNSW Euclidean"
        );
        for &(_, score) in &results {
            assert!(
                score <= 0.0,
                "Euclidean score should be <= 0, got {}",
                score
            );
        }
    }

    #[test]
    fn test_search_with_method_override_routes_backends() {
        // Engine configured for HNSW, but per-search `method` must route to
        // each explicit backend (FEAT-04 binding override path).
        let index = make_hnsw_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![-1.0, 0.0, 0.0]);
        add_node(&index, 2, vec![0.0, 1.0, 0.0]);
        let query = vec![1.0, 0.0, 0.0];
        for method in [
            crate::index::IndexType::Hnsw,
            crate::index::IndexType::Ivf,
            crate::index::IndexType::Scann,
            crate::index::IndexType::Flat,
        ] {
            let results = index.search_with_method(method, &query, &ALL_BITSET, 3);
            assert_eq!(results.len(), 3, "method {method:?} should return 3 nodes");
            assert_eq!(
                results[0].0, 0,
                "method {method:?} should rank the identical vector first, got {results:?}"
            );
        }
    }

    // ── search_layer direct tests ───────────────────────────────────────

    use std::collections::HashSet;

    #[test]
    fn test_search_layer_empty_entry_points() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Cosine,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(
            results.is_empty(),
            "empty entry points should return empty results"
        );
    }

    #[test]
    fn test_search_layer_cosine_ordered() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.8, 0.6, 0.0]);
        // For search_layer to traverse to node 1, node 0 must have it as neighbor.
        // insert_hnsw with ef_construction=50 should connect them.
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[0],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Cosine,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(!results.is_empty(), "search_layer should find results");
        let sorted = results.into_sorted_vec();
        assert!(
            sorted[0].1 == 0,
            "node 0 should be best match, got id={}",
            sorted[0].1
        );
    }

    #[test]
    fn test_search_layer_tombstone_not_returned() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.9, 0.1, 0.0]);
        // Mark node 0 as tombstone
        if let Some(mut n) = index.nodes.get_mut(&0) {
            n.flags |= FLAG_TOMBSTONE;
        }
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[0, 1],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Cosine,
            &mut visited,
            &mut SearchProfile::new(),
        );
        // The already-visited tombstone should be filtered from results
        let sorted = results.into_sorted_vec();
        for ns in &sorted {
            assert!(ns.1 != 0, "tombstone node 0 should not appear in results");
        }
    }

    #[test]
    fn test_search_layer_euclidean_metric() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![1.0, 0.0]);
        add_node(&index, 1, vec![10.0, 10.0]);
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0],
            Some(1.0),
            None,
            &[0],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Euclidean,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(
            !results.is_empty(),
            "search_layer Euclidean should find results"
        );
        let sorted = results.into_sorted_vec();
        // Euclidean scores should be negative
        for ns in &sorted {
            assert!(ns.0 <= 0.0, "Euclidean score should be <= 0");
        }
    }

    // ── select_neighbors: diversity pruning ─────────────────────────────

    #[test]
    fn test_select_neighbors_diversity_prunes_similar() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0]);
        add_node(&index, 1, vec![0.99, 0.01]);
        add_node(&index, 2, vec![-1.0, 0.0]);

        let mut heap = BinaryHeap::new();
        heap.push(NodeSimMin(0.9, 0));
        heap.push(NodeSimMin(0.85, 1));
        heap.push(NodeSimMin(0.1, 2));

        // select_neighbors now uses simple top-M (no diversity check):
        // sorted: [0 (0.9), 1 (0.85), 2 (0.1)] → top 2 = [0, 1]
        let selected = index.select_neighbors(heap, 2, |_| false);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&0), "best candidate should be selected");
        assert!(
            selected.contains(&1),
            "second-best should also be selected with top-M"
        );
    }

    #[test]
    fn test_select_neighbors_diversity_with_euclidean() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![1.0, 0.0]);
        add_node(&index, 1, vec![1.1, 0.0]); // close to 0
        add_node(&index, 2, vec![-1.0, 0.0]);

        let mut heap = BinaryHeap::new();
        // Euclidean scores are negative (similarity = -distance)
        // Distance(0, self) = 0, so similarity = 0
        // But we use arbitrary scores for selection
        heap.push(NodeSimMin(0.0, 0));
        heap.push(NodeSimMin(-0.01, 1)); // slightly worse
        heap.push(NodeSimMin(-2.0, 2)); // much worse

        let selected = index.select_neighbors(heap, 2, |_| false);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&0), "best should be selected");
        // node 1 is close to 0 so it may or may not be pruned depending on distances
        assert!(
            selected.contains(&1) || selected.contains(&2),
            "should pick at least one more candidate"
        );
    }

    #[test]
    fn test_select_neighbors_m_zero() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0]);

        let mut heap = BinaryHeap::new();
        heap.push(NodeSimMin(1.0, 0));
        let selected = index.select_neighbors(heap, 0, |_| false);
        assert!(selected.is_empty(), "m=0 should return empty selection");
    }

    #[test]
    fn test_select_neighbors_missing_node_skipped() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0]);

        let mut heap = BinaryHeap::new();
        // With top-M selection, the function doesn't validate node existence
        // (IDs come from search_layer which only yields live nodes during build)
        heap.push(NodeSimMin(1.0, 999));
        heap.push(NodeSimMin(0.5, 0));

        let selected = index.select_neighbors(heap, 5, |_| false);
        assert_eq!(selected.len(), 2, "top-M selects both available entries");
        assert!(
            selected.contains(&999),
            "top-M selects by score, not existence"
        );
    }

    #[test]
    fn test_select_neighbors_discarded_fills_remaining() {
        let index = make_index(DistanceMetric::Cosine);
        // 3 nodes where some may be pruned for diversity
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.98, 0.02, 0.0]);
        add_node(&index, 2, vec![-0.5, 0.5, 0.0]);

        let mut heap = BinaryHeap::new();
        // Node 0 and 1 are close, node 2 is different
        heap.push(NodeSimMin(0.9, 0));
        heap.push(NodeSimMin(0.8, 1));
        heap.push(NodeSimMin(0.3, 2));

        // select_neighbors with m=3 should try to return 3, even if pruning happens
        let selected = index.select_neighbors(heap, 3, |_| false);
        assert_eq!(
            selected.len(),
            3,
            "should fill remaining slots with discarded candidates"
        );
    }

    // ── Miri tests for unsafe patterns ──────────────────────────────
    //
    // search.rs has 4 unsafe blocks: `from_raw_parts` in search_layer
    // for mmap-backed vector access (2 blocks in the entry_point loop,
    // 2 in the neighbor evaluation loop). These require an actual
    // VantaFile with mmap data which Miri cannot provide.
    //
    // These Miri tests exercise search_layer and the HNSW search path
    // with `vector_store = None`, which routes through fast_similarity
    // → distance kernels (unsafe blocks in distance.rs, already tested
    // by the distance.rs and graph.rs Miri tests).

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_search_layer_empty_entry_points() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Cosine,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(results.is_empty(), "empty entry points → empty results");
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_search_layer_cosine_small() {
        let index = make_index(DistanceMetric::Cosine);
        add_node(&index, 0, vec![1.0, 0.0, 0.0]);
        add_node(&index, 1, vec![0.9, 0.1, 0.0]);
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[0],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Cosine,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(!results.is_empty(), "should find results");
        let sorted = results.into_sorted_vec();
        assert!(
            sorted[0].1 == 0 || sorted[0].1 == 1,
            "top result should be 0 or 1"
        );
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_search_layer_euclidean() {
        let index = make_index(DistanceMetric::Euclidean);
        add_node(&index, 0, vec![1.0, 0.0]);
        add_node(&index, 1, vec![10.0, 10.0]);
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[1.0, 0.0],
            Some(1.0),
            None,
            &[0],
            10,
            0,
            &ALL_BITSET,
            false, // no ACORN in existing tests
            None,
            DistanceMetric::Euclidean,
            &mut visited,
            &mut SearchProfile::new(),
        );
        assert!(!results.is_empty(), "should find results");
        let sorted = results.into_sorted_vec();
        for ns in &sorted {
            assert!(ns.0 <= 0.0, "Euclidean scores ≤ 0");
        }
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_search_nearest_hnsw_path() {
        // Force HNSW path (flat_threshold = None)
        let index = make_hnsw_index(DistanceMetric::Cosine);
        for i in 0u128..10 {
            add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
        }
        let results = index.search_nearest(&[0.0, 0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        // AUDREP-55: zero-norm cosine query is undefined; must return no
        // results (and a warning) instead of re-scoring with euclidean.
        assert_eq!(results.len(), 0);
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_search_nearest_hnsw_euclidean() {
        let index = make_hnsw_index(DistanceMetric::Euclidean);
        for i in 0u128..10 {
            add_node(&index, i, vec![(i as f32) * 0.5, 0.0]);
        }
        let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 5);
        for &(_, score) in &results {
            assert!(score.is_finite());
            assert!(score <= 0.0);
        }
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_select_neighbors_basic() {
        let index = make_index(DistanceMetric::Cosine);
        for i in 0u128..6 {
            add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
        }
        let mut heap = BinaryHeap::new();
        for i in 0u128..6 {
            if let Some(node) = index.nodes.get(&i) {
                if let Some(slice) = node.vec_data.as_f32_slice() {
                    let sim = cosine_sim_f32(slice, slice);
                    heap.push(NodeSimMin(sim, i));
                }
            }
        }
        let selected = index.select_neighbors(heap, 3, |_| false);
        assert_eq!(selected.len(), 3);
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_select_neighbors_euclidean() {
        let index = make_index(DistanceMetric::Euclidean);
        for i in 0u128..4 {
            add_node(&index, i, vec![(i as f32) * 2.0, 0.0]);
        }
        let mut heap = BinaryHeap::new();
        for i in 0u128..4 {
            if let Some(node) = index.nodes.get(&i) {
                if let Some(slice) = node.vec_data.as_f32_slice() {
                    // self-distance is 0 → score = 0
                    let sim = -euclidean_distance_squared_f32(slice, slice);
                    heap.push(NodeSimMin(sim, i));
                }
            }
        }
        let selected = index.select_neighbors(heap, 2, |_| false);
        assert_eq!(selected.len(), 2);
    }

    // ── ACORN-1: second-hop filtered search expansion ──────────────────

    /// Helper to add a node with a custom bitset.
    fn add_node_with_bitset(index: &CPIndex, id: u128, vec: Vec<f32>, bits: &[u32]) {
        let mut bs = FilterBitset::new();
        for &b in bits {
            bs.set_bit(b as usize);
        }
        index
            .add(id, bs, VectorRepresentations::Full(vec), 0)
            .expect("test vectors are non-zero-norm");
    }

    /// Force a node's neighbors in both neighbor_index AND inline cache.
    /// Direct `neighbor_index.set_neighbors()` bypasses the inline cache on
    /// HnswNode, causing ACORN expansion to read stale neighbors from the cache.
    fn set_test_neighbors(index: &CPIndex, id: u128, layer: usize, neighbors: NeighborVec) {
        let inline_cache = neighbors.clone();
        index.neighbor_index.set_neighbors(id, layer, neighbors);
        if let Some(mut node_ref) = index.nodes.get_mut(&id) {
            if node_ref.neighbor_lists.len() > layer {
                node_ref.neighbor_lists[layer] = inline_cache;
            }
        }
    }

    #[test]
    fn test_acorn_expands_through_non_matching() {
        // ACORN-1 scenario: a non-matching node N (blocks filter) is a neighbor of
        // entry A. Without ACORN, N is never popped from the candidates heap because
        // a competing matching node X (also a neighbor of A) pops first, discovers
        // a very close node R (raising the "worst" threshold), and N then fails the
        // break check (d_N < worst). ACORN pre-expands N's neighbors when N is first
        // discovered as A's neighbor, finding S before N reaches the top of the heap.
        //
        // Euclidean, ef=2, single entry A:
        //   A=[1,0,0], d=-1.00, matches {0}
        //   X=[0.8,0,0], d=-0.64, matches {0} (competes with N for heap slot)
        //   N=[0.95,0,0], d=-0.9025, FAILS {0} (has {1})
        //   R=[0.3,0,0], d=-0.09, matches {0} (neighbor of X, raises threshold)
        //   S=[0.2,0,0], d=-0.04, matches {0} (neighbor of N, only reachable via ACORN)
        //
        // Topology: A→{X,N}, X→{A,R}, N→{A,S}
        //
        // Without ACORN: A→X→R (raises worst)→N(d=-0.9025 < worst=-0.09) → break!
        //   S never discovered.
        // With ACORN: A→N ACORN-expands to S immediately, S enters results.

        let index = CPIndex::new_with_config(HnswConfig {
            m: 2,
            m_max0: 2,
            ef_construction: 10,
            ef_search: 50,
            ml: 1.0,
            distance_metric: DistanceMetric::Euclidean,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // Insert 5 nodes
        add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
        add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
        add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
        add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
        add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

        // Verify layer-0 neighbors exist
        for id in 0u128..5 {
            assert!(
                index
                    .neighbor_index
                    .get_neighbors(id, 0)
                    .map(|n| !n.is_empty())
                    .unwrap_or(false),
                "node {id} should have at least one neighbor after HNSW insert"
            );
        }

        // Force topology (use set_test_neighbors to update both neighbor_index and inline cache)
        // A → X, N
        set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]); // X, N
                                                                             // X → A, R
        set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]); // A, R
                                                                             // N → A, S
        set_test_neighbors(&index, 1, 0, smallvec::smallvec![0u128, 4u128]); // A, S
                                                                             // R → X (backlink)
        set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]);
        // S → N (backlink)
        set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);

        // ── WITHOUT ACORN ──
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let no_acorn = index.search_layer(
            &[0.0, 0.0, 0.0],
            Some(0.0),
            None,
            &[0], // entry A only
            2,    // ef=2
            0,
            &mask,
            false,
            None,
            DistanceMetric::Euclidean,
            &mut visited,
            &mut SearchProfile::new(),
        );
        let no_acorn_ids: Vec<u128> = no_acorn
            .into_sorted_vec()
            .into_iter()
            .map(|ns| ns.1)
            .collect();
        assert!(
            !no_acorn_ids.contains(&4),
            "without ACORN, node 4 (S) should NOT be found (N never popped), got {:?}",
            no_acorn_ids
        );

        // ── WITH ACORN ──
        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let with_acorn = index.search_layer(
            &[0.0, 0.0, 0.0],
            Some(0.0),
            None,
            &[0],
            2,
            0,
            &mask,
            true,
            None,
            DistanceMetric::Euclidean,
            &mut visited,
            &mut SearchProfile::new(),
        );
        let with_acorn_ids: Vec<u128> = with_acorn
            .into_sorted_vec()
            .into_iter()
            .map(|ns| ns.1)
            .collect();
        assert!(
            with_acorn_ids.contains(&4),
            "with ACORN, node 4 (S) should be found via ACORN expansion through N, got {:?}",
            with_acorn_ids
        );
    }

    #[test]
    fn test_acorn_no_regression_all_set() {
        // When query_mask.is_all_set(), acorn=true should behave identically
        // to acorn=false because the expansion is never triggered.
        let index = CPIndex::new_with_config(HnswConfig {
            m: 1,
            m_max0: 1,
            ef_construction: 10,
            ef_search: 50,
            ml: 1.0,
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]);
        add_node_with_bitset(&index, 1, vec![0.99, 0.01, 0.0], &[1]);

        // Node 0's neighbor is node 1 (force it)
        index
            .neighbor_index
            .set_neighbors(0, 0, smallvec::smallvec![1u128]);
        index
            .neighbor_index
            .set_neighbors(1, 0, smallvec::smallvec![0u128]);

        // With ALL_BITSET, acorn should never trigger because
        // !query_mask.is_all_set() is false
        let mut visited1: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let r1 = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[0],
            10,
            0,
            &ALL_BITSET,
            false,
            None,
            DistanceMetric::Cosine,
            &mut visited1,
            &mut SearchProfile::new(),
        );

        let mut visited2: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let r2 = index.search_layer(
            &[1.0, 0.0, 0.0],
            Some(1.0),
            Some(1.0),
            &[0],
            10,
            0,
            &ALL_BITSET,
            true,
            None,
            DistanceMetric::Cosine,
            &mut visited2,
            &mut SearchProfile::new(),
        );

        // Both should return the same results when mask is all_set
        let ids1: Vec<u128> = r1.into_sorted_vec().into_iter().map(|ns| ns.1).collect();
        let ids2: Vec<u128> = r2.into_sorted_vec().into_iter().map(|ns| ns.1).collect();
        assert_eq!(
            ids1, ids2,
            "ACORN should not change results when mask is ALL_BITSET"
        );
    }

    #[test]
    fn test_acorn_budget_respected() {
        // Verify the ACORN budget formula: ef.saturating_sub(results.len()).max(16).
        // Uses the same topology as test_acorn_expands_through_non_matching but
        // with ef=1 so budget = (1-0).max(16) = 16, enough to explore all second-hops.
        let index = make_hnsw_index(DistanceMetric::Euclidean);

        // Insert 5 nodes as before
        add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
        add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
        add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
        add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
        add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

        // Force same topology (using set_test_neighbors for both caches)
        set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]);
        set_test_neighbors(&index, 1, 0, smallvec::smallvec![0u128, 4u128]);
        set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]);
        set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]);
        set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);

        let results = index.search_layer(
            &[0.0, 0.0, 0.0],
            Some(0.0),
            None,
            &[0],
            2, // ef=2 → budget=(2-0).max(16)=16
            0,
            &mask,
            true, // acorn_expansion = true
            None,
            DistanceMetric::Euclidean,
            &mut HashSet::with_capacity_and_hasher(100, RandomState::new()),
            &mut SearchProfile::new(),
        );

        let ids: Vec<u128> = results
            .into_sorted_vec()
            .into_iter()
            .map(|ns| ns.1)
            .collect();
        // ACORN should find at least one second-hop node (S=4) through N
        assert!(
            ids.contains(&4),
            "ACORN-expanded node (id=4) should be found with ef=1, got {:?}",
            ids
        );
    }

    #[test]
    fn test_acorn_second_hop_after_repair_orphans() {
        // ERR-020: ACORN's second-hop expansion must read the POST-repair
        // adjacency, not a stale inline cache.
        //
        // repair_orphan_links repairs `neighbor_index` but (pre-fix) left each
        // HnswNode's inline `neighbor_lists` cache holding the removed orphan
        // ids. search_layer prefers the inline cache and only falls back to
        // `neighbor_index` when it is empty, so ACORN keeps walking dead
        // edges. This test inserts 16 orphan nodes into non-matching node N's
        // neighbor list, removes them (simulating engine deletes), repairs,
        // then runs the ACORN search with ef=2 so the second-hop budget is
        // 16: the stale orphans crowd out the live second-hop node S before
        // the fix.
        //
        // Topology (Euclidean, entry A matches {0}; N blocks the filter):
        //   A=[1,0,0] {0} → {X, N}
        //   N=[0.95,0,0] {1} → {D0..D15, A, S}   (D* are removed orphans)
        //   X=[0.8,0,0] {0} → {A, R}
        //   R=[0.3,0,0] {0} → {X}
        //   S=[0.2,0,0] {0} → {N}
        // S is only reachable via ACORN expansion through N. With the stale
        // inline cache, take(16) over {D0..D15, A, S} never reaches S; after
        // repair the list is {A, S} and S is found.
        let index = CPIndex::new_with_config(HnswConfig {
            m: 2,
            m_max0: 2,
            ef_construction: 10,
            ef_search: 50,
            ml: 1.0,
            distance_metric: DistanceMetric::Euclidean,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
        add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
        add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
        add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
        add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

        // 16 orphan nodes D0..D15 (ids 5..20), inserted so the delete flow
        // that orphans N's list is realistic, then raw-removed below.
        let orphans: Vec<u128> = (5u128..21).collect();
        for d in &orphans {
            add_node_with_bitset(&index, *d, vec![0.1, 0.0, 0.0], &[1]);
        }

        let mut n_neighbors: NeighborVec = orphans.iter().copied().collect();
        n_neighbors.push(0); // A
        n_neighbors.push(4); // S
        set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]); // A → X, N
        set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]); // X → A, R
        set_test_neighbors(&index, 1, 0, n_neighbors); // N → orphans + A + S
        set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]); // R → X
        set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]); // S → N

        // Provoke orphan repair: raw-remove the D nodes (the engine delete
        // path this mirrors); repair_orphan_links cleans the leftovers.
        for d in &orphans {
            assert!(
                index.nodes.remove(d).is_some(),
                "orphan node {d} should exist before removal"
            );
        }
        let report = index.repair_orphan_links();
        assert!(
            report.repaired_links >= orphans.len() as u64,
            "repair should drop at least the {}-node orphan list from N, got {}",
            orphans.len(),
            report.repaired_links
        );

        // Post-repair invariant: no inline neighbor cache references a
        // removed id (the direct causal check for the stale second hop).
        for d in &orphans {
            if let Some(node_ref) = index.nodes.get(&1) {
                for list in &node_ref.neighbor_lists {
                    assert!(
                        !list.contains(d),
                        "inline neighbor list of node 1 still references removed node {d}"
                    );
                }
            }
        }

        let mut mask = FilterBitset::new();
        mask.set_bit(0);

        let mut visited: HashSet<u128, RandomState> =
            HashSet::with_capacity_and_hasher(100, RandomState::new());
        let results = index.search_layer(
            &[0.0, 0.0, 0.0],
            Some(0.0),
            None,
            &[0], // entry A only
            2,    // ef=2 → ACORN budget = (2-1).max(16) = 16
            0,
            &mask,
            true, // acorn_expansion
            None,
            DistanceMetric::Euclidean,
            &mut visited,
            &mut SearchProfile::new(),
        );
        let ids: Vec<u128> = results
            .into_sorted_vec()
            .into_iter()
            .map(|ns| ns.1)
            .collect();
        assert!(
            ids.contains(&4),
            "ACORN second-hop after repair_orphan_links: live node S (4) must be found \
             (stale orphans must not crowd the expansion budget), got {:?}",
            ids
        );
    }
}
