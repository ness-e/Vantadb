//! HNSW layer traversal — the core greedy beam-search loop. Hot path.

use ahash::RandomState;
use std::collections::BinaryHeap;

use super::pool::give_nl;
use super::profile::SearchProfile;
use crate::index::distance::{
    cosine_sim_cached_norms, euclidean_distance_squared_f32, f32_slice_similarity,
};
use crate::index::graph::{self, CPIndex, NeighborVec, NodeSim, NodeSimMin};
use crate::node::{DistanceMetric, FilterBitset, NodeFlags};
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

        // AUD-047: metric scoring for the disk path is identical for entry
        // points and neighbor candidates — single helper, zero-cost (Fn,
        // inlined in release).
        let metric_score = |vec: &[f32], inv_cached_norm: f32| -> f32 {
            match metric {
                DistanceMetric::Cosine => {
                    if let Some(q_inv_norm) = query_inv_norm {
                        if inv_cached_norm > f32::EPSILON {
                            cosine_sim_cached_norms(query_vec, q_inv_norm, vec, inv_cached_norm)
                        } else {
                            f32_slice_similarity(query_vec, query_norm, vec, metric)
                        }
                    } else {
                        f32_slice_similarity(query_vec, query_norm, vec, metric)
                    }
                }
                DistanceMetric::Euclidean => -euclidean_distance_squared_f32(query_vec, vec),
                // SparseDot has its own brute-force path.
                DistanceMetric::SparseDot => 0.0,
            }
        };

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                // ERR-042: read the disk header once per entry point and reuse
                // it for both the distance computation and the tombstone
                // eligibility check below (was 2x read_header per entry point).
                let node_header = vector_store.and_then(|vs| vs.read_header(node.storage_offset));
                let d = if let Some(vs) = vector_store {
                    // ADR-032: quantized vectors (Binary/Turbo/SQ8) are not scored from vstore
                    // f32 bytes — they delegate to fast_similarity which already knows
                    // how to handle their in-memory HNSW representation. Only FULL
                    // (and legacy kind==0 with len>0) is scored via the vstore f32 path.
                    let kind = node_header.map(|h| NodeFlags::vector_kind(h.flags));
                    let is_full = matches!(
                        kind,
                        Some(k) if k == NodeFlags::VECTOR_KIND_FULL
                            || (k == 0 && node_header.map(|h| h.vector_len > 0).unwrap_or(false))
                    );
                    if !is_full {
                        self.fast_similarity(query_vec, query_norm, query_inv_norm, &node, metric)
                    } else {
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
                                // SAFETY: same as before — bounds + align + copy
                                debug_assert_eq!(
                                    vec_data.as_ptr().align_offset(4),
                                    0,
                                    "f32 vector must be 4-byte aligned"
                                );
                                let (_, f32_vec, _) = unsafe { vec_data.align_to::<f32>() };
                                if f32_vec.len() != header.vector_len as usize {
                                    0.0
                                } else {
                                    metric_score(f32_vec, node.inv_cached_norm)
                                }
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        profile.end_compute();
                        result
                    }
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
                                        let kind = NodeFlags::vector_kind(h.flags);
                                        let payload_len: u64 = match kind {
                                            NodeFlags::VECTOR_KIND_FULL => {
                                                (h.vector_len as u64).checked_mul(4).unwrap_or(0)
                                            }
                                            NodeFlags::VECTOR_KIND_BINARY => {
                                                (h.vector_len as u64).checked_mul(8).unwrap_or(0)
                                            }
                                            NodeFlags::VECTOR_KIND_TURBO => h.vector_len as u64,
                                            NodeFlags::VECTOR_KIND_SQ8 => h.vector_len as u64 + 4,
                                            NodeFlags::VECTOR_KIND_NONE => 0,
                                            _ => {
                                                if h.vector_len == 0 {
                                                    0
                                                } else {
                                                    (h.vector_len as u64)
                                                        .checked_mul(4)
                                                        .unwrap_or(0)
                                                }
                                            }
                                        };
                                        let Some(vec_end) =
                                            h.vector_offset.checked_add(payload_len)
                                        else {
                                            continue;
                                        };
                                        if vec_end <= mmap_len as u64 && payload_len > 0 {
                                            graph::prefetch_mmap_vector(
                                                mmap_base,
                                                h.vector_offset as usize,
                                                payload_len as usize,
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
                            let kind = node_header.map(|h| NodeFlags::vector_kind(h.flags));
                            let is_full = matches!(
                                kind,
                                Some(k) if k == NodeFlags::VECTOR_KIND_FULL
                                    || (k == 0 && node_header.map(|h| h.vector_len > 0).unwrap_or(false))
                            );
                            let d = if let Some(vs) = vector_store {
                                if !is_full {
                                    self.fast_similarity(
                                        query_vec,
                                        query_norm,
                                        query_inv_norm,
                                        &neighbor,
                                        metric,
                                    )
                                } else {
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
                                            // SAFETY: same as entry point
                                            debug_assert_eq!(
                                                v_data.as_ptr().align_offset(4),
                                                0,
                                                "f32 neighbor vector must be 4-byte aligned"
                                            );
                                            let (_, f32_v, _) = unsafe { v_data.align_to::<f32>() };
                                            if f32_v.len() != h.vector_len as usize {
                                                0.0
                                            } else {
                                                metric_score(f32_v, neighbor.inv_cached_norm)
                                            }
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        0.0
                                    };
                                    profile.end_compute();
                                    result
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
