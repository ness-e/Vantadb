//! HNSW layer traversal — the core greedy beam-search loop. Hot path.

use ahash::RandomState;
use std::collections::BinaryHeap;

use super::pool::give_nl;
use super::profile::SearchProfile;
use crate::index::distance::{
    cosine_sim_cached_norms, euclidean_distance_squared_f32, f32_slice_similarity,
};
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
}
