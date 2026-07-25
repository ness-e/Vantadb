use ahash::RandomState;
use std::collections::BinaryHeap;

use super::distance::*;
use crate::index::graph::{self, CPIndex, NeighborVec, NodeSim, NodeSimMin};
use crate::node::{DistanceMetric, FilterBitset};
use crate::storage::engine::FLAG_TOMBSTONE;

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
        vector_store: Option<&crate::storage::vfile::VantaFile>,
        metric: DistanceMetric,
        visited: &mut std::collections::HashSet<u128, RandomState>,
    ) -> BinaryHeap<NodeSimMin> {
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                let d = if let Some(vs) = vector_store {
                    if let Some(header) = vs.read_header(node.storage_offset) {
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
                    }
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

            let neighbors = if let Some(node) = self.nodes.get(&cand_id) {
                if layer < node.neighbors.len() {
                    Some(node.neighbors[layer].clone())
                } else {
                    None
                }
            } else {
                None
            };

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
                                if let Some(h) = vs.read_header(neighbor.storage_offset) {
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
                                }
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
                            }
                        }
                    }
                }
            }
        }
        results
    }

    pub(crate) fn select_neighbors(
        &self,
        candidates: BinaryHeap<NodeSimMin>,
        m: usize,
    ) -> NeighborVec {
        let sorted = candidates.into_sorted_vec();

        struct SelectedInfo {
            id: u128,
            vec: Option<Vec<f32>>,
            inv_norm: f32,
        }

        let mut selected: Vec<SelectedInfo> = Vec::with_capacity(m);
        let mut discarded: Vec<u128> = Vec::new();

        for ns in sorted.into_iter() {
            if selected.len() >= m {
                break;
            }

            let cand_id = ns.1;
            let sim_q_cand = ns.0;

            let cand_node = match self.nodes.get(&cand_id) {
                Some(n) => n,
                None => continue,
            };
            if (cand_node.flags & FLAG_TOMBSTONE) != 0 {
                continue;
            }
            let cand_slice = cand_node.vec_data.as_f32_slice();
            let cand_inv_norm = cand_node.inv_cached_norm;

            let mut is_diverse = true;
            for sel in &selected {
                let sim_cand_sel = match self.config.distance_metric {
                    DistanceMetric::Cosine => {
                        if let (Some(c_slice), Some(s_slice)) = (cand_slice, &sel.vec) {
                            cosine_sim_cached_norms(c_slice, cand_inv_norm, s_slice, sel.inv_norm)
                        } else {
                            if let Some(sel_node) = self.nodes.get(&sel.id) {
                                let cand_norm = if cand_inv_norm > f32::EPSILON {
                                    Some(1.0 / cand_inv_norm)
                                } else {
                                    None
                                };
                                calculate_similarity(
                                    cand_slice.unwrap_or(&[]),
                                    cand_norm,
                                    None,
                                    None,
                                    &sel_node.vec_data,
                                    self.config.distance_metric,
                                )
                            } else {
                                0.0
                            }
                        }
                    }
                    DistanceMetric::Euclidean => {
                        if let (Some(c_slice), Some(s_slice)) = (cand_slice, &sel.vec) {
                            -euclidean_distance_squared_f32(c_slice, s_slice)
                        } else {
                            if let Some(sel_node) = self.nodes.get(&sel.id) {
                                calculate_similarity(
                                    cand_slice.unwrap_or(&[]),
                                    None,
                                    None,
                                    None,
                                    &sel_node.vec_data,
                                    self.config.distance_metric,
                                )
                            } else {
                                0.0
                            }
                        }
                    }
                };

                if sim_cand_sel > sim_q_cand {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                selected.push(SelectedInfo {
                    id: cand_id,
                    vec: cand_slice.map(|s| s.to_vec()),
                    inv_norm: cand_inv_norm,
                });
            } else {
                discarded.push(cand_id);
            }
        }

        let mut final_selected: NeighborVec = selected.into_iter().map(|s| s.id).collect();
        for &disc_id in discarded.iter() {
            if final_selected.len() >= m {
                break;
            }
            final_selected.push(disc_id);
        }

        final_selected
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
        let tuned_ef = crate::index::auto_tune::AutoTune::current_ef();
        let ef_search = static_ef.max(tuned_ef).max(top_k);
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
                ef_search.max(top_k) * 2,
                RandomState::new(),
            );

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
                vector_store,
                effective_metric,
                &mut visited,
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
            vector_store,
            effective_metric,
            &mut visited,
        );

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
        let selected = index.select_neighbors(heap, 5);
        assert!(
            !selected.contains(&0),
            "tombstone node should not be selected"
        );
        assert!(
            selected.contains(&1),
            "non-tombstone node should be selected"
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
            None,
            DistanceMetric::Cosine,
            &mut visited,
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
            None,
            DistanceMetric::Cosine,
            &mut visited,
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
            None,
            DistanceMetric::Cosine,
            &mut visited,
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
            None,
            DistanceMetric::Euclidean,
            &mut visited,
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
        // Node 0, 1 are close to each other; node 2 is in opposite direction.
        add_node(&index, 0, vec![1.0, 0.0]);
        add_node(&index, 1, vec![0.99, 0.01]);
        add_node(&index, 2, vec![-1.0, 0.0]);

        let mut heap = BinaryHeap::new();
        // Lower scores so diversity check (sim_cand_sel > sim_q_cand) triggers
        // Node 0: score 0.9  Node 1: score 0.85  Node 2: score 0.1
        heap.push(NodeSimMin(0.9, 0));
        heap.push(NodeSimMin(0.85, 1));
        heap.push(NodeSimMin(0.1, 2));

        // select_neighbors sorts descending: [0 (0.9), 1 (0.85), 2 (0.1)]
        // Selects 0 first. Then checks 1: sim(1,0) ≈ 0.99 > 0.85 → not diverse → discard.
        // Then checks 2: sim(2,0) ≈ -1.0 < 0.1 → diverse → select.
        let selected = index.select_neighbors(heap, 2);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&0), "best candidate should be selected");
        assert!(
            !selected.contains(&1),
            "redundant candidate should be pruned"
        );
        assert!(
            selected.contains(&2),
            "diverse candidate should be selected"
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
        // Node 999 doesn't exist
        heap.push(NodeSimMin(1.0, 999));
        heap.push(NodeSimMin(0.5, 0));

        let selected = index.select_neighbors(heap, 5);
        assert!(
            !selected.contains(&999),
            "non-existent node should be skipped"
        );
        assert!(selected.contains(&0), "existing node should be selected");
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
            None,
            DistanceMetric::Cosine,
            &mut visited,
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
            None,
            DistanceMetric::Cosine,
            &mut visited,
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
            None,
            DistanceMetric::Euclidean,
            &mut visited,
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
}
