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

/// Borrow a `NeighborVec` from the thread-local pool, or create a new one.
fn take_nl() -> NeighborVec {
    NL_POOL.with(|pool| pool.borrow_mut().pop().unwrap_or_default())
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
                let d = if let Some(vs) = vector_store {
                    profile.record_vfile_entry(node.storage_offset);
                    profile.start_compute();
                    let result = if let Some(header) = vs.read_header(node.storage_offset) {
                        let vec_start = header.vector_offset as usize;
                        let vec_end = vec_start + (header.vector_len as usize * 4);
                        if vec_end > vs.mmap_bytes().len() {
                            0.0
                        } else {
                            let vec_data = &vs.mmap_bytes()[vec_start..vec_end];
                            // SAFETY: `vec_end > vs.mmap_bytes().len()` guard above ensures
                            // `vec_start + header.vector_len * 4 <= mmap size` — the byte range
                            // is valid and the alignment cast to `f32` is safe (mmap pages are
                            // aligned, and HNSW stores vectors with 4-byte alignment in the
                            // memory-mapped file).
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
                            }
                        }
                    } else {
                        0.0
                    };
                    profile.end_compute();
                    result
                } else {
                    self.fast_similarity(query_vec, query_norm, query_inv_norm, &node, metric)
                };

                let eligible = if let Some(vs) = vector_store {
                    vs.read_header(node.storage_offset)
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
            // E2: Use per-thread pool to reuse NeighborVec allocations.
            let neighbors = self
                .nodes
                .get(&cand_id)
                .and_then(|n| {
                    n.neighbor_lists
                        .get(layer)
                        .filter(|l| !l.is_empty())
                        .map(|l| {
                            let mut v = take_nl();
                            v.extend_from_slice(l);
                            v
                        })
                })
                .or_else(|| self.neighbor_index.get_neighbors(cand_id, layer));

            if let Some(neighbors_list) = neighbors {
                if graph::should_prefetch() {
                    if let Some(vs) = vector_store {
                        let mmap_base = vs.mmap_bytes().as_ptr();
                        let mmap_len = vs.mmap_bytes().len();
                        for &pf_neighbor_id in &neighbors_list {
                            if !visited.contains(&pf_neighbor_id) {
                                if let Some(pf_node) = self.nodes.get(&pf_neighbor_id) {
                                    if let Some(h) = vs.read_header(pf_node.storage_offset) {
                                        let vec_start = h.vector_offset as usize;
                                        let vec_len_bytes = h.vector_len as usize * 4;
                                        if vec_start + vec_len_bytes <= mmap_len
                                            && vec_len_bytes > 0
                                        {
                                            graph::prefetch_mmap_vector(
                                                mmap_base,
                                                vec_start,
                                                vec_len_bytes,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                for &neighbor_id in &neighbors_list {
                    if !visited.contains(&neighbor_id) {
                        visited.insert(neighbor_id);

                        if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                            let d = if let Some(vs) = vector_store {
                                profile.record_vfile_candidate(neighbor.storage_offset);
                                profile.start_compute();
                                let result = if let Some(h) =
                                    vs.read_header(neighbor.storage_offset)
                                {
                                    let vec_start = h.vector_offset as usize;
                                    let vec_end = vec_start + (h.vector_len as usize * 4);
                                    if vec_end > vs.mmap_bytes().len() {
                                        0.0
                                    } else {
                                        let v_data = &vs.mmap_bytes()[vec_start..vec_end];
                                        // SAFETY: `vec_end > vs.mmap_bytes().len()` guard above
                                        // ensures `h.vector_len * 4` does not exceed the mmap
                                        // region. Pointer is derived from the mmap byte slice.
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
                                        }
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

                            let eligible = if let Some(vs) = vector_store {
                                vs.read_header(neighbor.storage_offset)
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
                                        let second_hop = neighbor
                                            .neighbor_lists
                                            .get(layer)
                                            .filter(|l| !l.is_empty())
                                            .map(|l| {
                                                let mut v = take_nl();
                                                v.extend_from_slice(l);
                                                v
                                            })
                                            .or_else(|| {
                                                self.neighbor_index
                                                    .get_neighbors(neighbor_id, layer)
                                            });
                                        if let Some(second_list) = second_hop {
                                            let budget = ef.saturating_sub(results.len()).max(16);
                                            for &second_id in second_list.iter().take(budget) {
                                                if !visited.contains(&second_id) {
                                                    visited.insert(second_id);
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
                                            // E2: Return second_hop to pool.
                                            give_nl(second_list);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // E2: Return the NeighborVec to the thread-local pool for reuse.
                give_nl(neighbors_list);
            }
        }
        results
    }

    pub(crate) fn select_neighbors(
        &self,
        candidates: BinaryHeap<NodeSimMin>,
        m: usize,
    ) -> NeighborVec {
        // Simple top-M selection — no diversity heuristic (Malkov & Yashunin 2016 §4).
        // The diversity check (comparing each candidate against every selected neighbor)
        // is O(m · candidates) extra distance computations — roughly 2-3× slower — with
        // negligible recall benefit when ef_construction >= 100 (< 0.5% recall drop).
        //
        // ponytail: if ef_construction < 50 or recall at low ef_search matters,
        // re-enable the check with a config flag.
        //
        // O(n) partial sort via select_nth_unstable_by instead of O(n log n)
        // into_sorted_vec. For ef_construction=200 and m=32: ~200 comparisons
        // vs ~200*log(200) ≈ 1500.
        let mut vec = candidates.into_vec();
        if vec.len() > m {
            // Comparator flipped vs. the natural ordering: select_nth_unstable_by
            // partitions with elements "less than" the pivot in [0..nth]. With
            // `b.0 < a.0` (descending), vec[0..m] holds the m HIGHEST scores —
            // the best neighbors. (Regression: B2 inverted this to ascending,
            // keeping the m WORST candidates.)
            vec.select_nth_unstable_by(m, |a, b| {
                b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            });
            vec.truncate(m);
        }
        vec.into_iter().map(|ns| ns.1).collect::<NeighborVec>()
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
        // IVF path: lazy-build on first search, then search
        if self.config.index_type == IndexType::Ivf {
            let mut guard = self.ivf_index.lock();
            if guard.is_none() {
                let ivf_config = crate::index::ivf::IvfConfig {
                    nlist: (self.nodes.len() as f64).sqrt() as usize + 1,
                    nprobe: 10,
                    distance_metric: self.config.distance_metric,
                };
                *guard = Some(crate::index::ivf::IvfIndex::build(&self.nodes, &ivf_config));
            }
            let ivf = guard.as_ref().unwrap();
            return ivf.search(query_vec, top_k, query_mask);
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
                let norm = f32_l2_norm(query_vec);
                if norm < f32::EPSILON {
                    (DistanceMetric::Euclidean, None, None)
                } else {
                    (DistanceMetric::Cosine, Some(norm), Some(1.0 / norm))
                }
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(query_vec);
                (DistanceMetric::Euclidean, Some(norm), None)
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
            };
            final_results.push((id, adjusted_score));
        }
        final_results
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
        CPIndex::add(self, id, bitset, vec_data, storage_offset);
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
        index.add(id, FilterBitset::new(), VectorRepresentations::Full(vec), 0);
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
            add_node(&index, i, vec![i as f32, 0.0, 0.0]);
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
        let selected = index.select_neighbors(heap, 5);
        assert!(
            selected.is_empty(),
            "empty candidates should produce empty selection"
        );
    }

    #[test]
    fn test_select_neighbors_returns_top_m() {
        let index = make_index(DistanceMetric::Cosine);
        for i in 0..6u128 {
            add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
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
        let selected = index.select_neighbors(heap, 3);
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
        let selected = index.select_neighbors(heap, 5);
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
        let selected = index.select_neighbors(heap, 2);
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
        let selected = index.select_neighbors(heap, 2);
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

        let selected = index.select_neighbors(heap, 2);
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
        let selected = index.select_neighbors(heap, 0);
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

        let selected = index.select_neighbors(heap, 5);
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
        let selected = index.select_neighbors(heap, 3);
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
    fn miri_search_nearest_hnsw_path() {
        // Force HNSW path (flat_threshold = None)
        let index = make_hnsw_index(DistanceMetric::Cosine);
        for i in 0u128..10 {
            add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
        }
        let results = index.search_nearest(&[0.0, 0.0, 0.0], None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 5);
        for &(_, score) in &results {
            assert!(score.is_finite());
        }
    }

    #[cfg(miri)]
    #[test]
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
        let selected = index.select_neighbors(heap, 3);
        assert_eq!(selected.len(), 3);
    }

    #[cfg(miri)]
    #[test]
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
        let selected = index.select_neighbors(heap, 2);
        assert_eq!(selected.len(), 2);
    }

    // ── ACORN-1: second-hop filtered search expansion ──────────────────

    /// Helper to add a node with a custom bitset.
    fn add_node_with_bitset(index: &CPIndex, id: u128, vec: Vec<f32>, bits: &[u32]) {
        let mut bs = FilterBitset::new();
        for &b in bits {
            bs.set_bit(b as usize);
        }
        index.add(id, bs, VectorRepresentations::Full(vec), 0);
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
}
