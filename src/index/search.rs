use std::collections::BinaryHeap;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;

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
    ) -> BinaryHeap<NodeSimMin> {
        let mut visited: std::collections::HashSet<u128, _> =
            std::collections::HashSet::with_capacity_and_hasher(
                ef * 2,
                BuildHasherDefault::<XxHash64>::default(),
            );
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

        let max_l = self.max_layer.load(std::sync::atomic::Ordering::Acquire);
        for layer in (1..=max_l).rev() {
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
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

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
