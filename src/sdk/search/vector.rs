use super::super::builder::VantaEmbedded;
use super::super::serialization::{matches_memory_filters, memory_record_from_node};
use super::super::types::*;
use crate::cost_estimator::{CostEstimator, FilterStrategy};
use crate::error::Result;
use crate::index::cosine_sim_f32;
use crate::index::VecIndex;
use crate::node::UnifiedNode;

impl VantaEmbedded {
    /// Pre-filter path: use `records_for_namespace` to fetch only records
    /// matching the filters, then brute-force vector similarity on the
    /// (typically small) result set.
    fn vector_memory_search_prefilter(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let mut hits = Vec::with_capacity(top_k);
        for record in self.records_for_namespace(namespace, filters)? {
            let Some(vector) = record.vector.as_ref() else {
                continue;
            };
            if vector.len() != query_vector.len() {
                continue;
            }
            let score = match distance_metric {
                crate::node::DistanceMetric::Cosine => cosine_sim_f32(query_vector, vector),
                crate::node::DistanceMetric::Euclidean => {
                    -crate::index::euclidean_distance_squared_f32(query_vector, vector)
                }
                // Sparse search has its own brute-force path over sparse_vectors.
                crate::node::DistanceMetric::SparseDot => 0.0,
            };
            hits.push(VantaMemorySearchHit {
                score,
                record,
                explanation: None,
            });
        }
        crate::planner::sort_hits(&mut hits);
        hits.truncate(top_k);

        // Feed search results into cache warmer co-access tracking
        if hits.len() >= 2 {
            let node_ids: Vec<u128> = hits.iter().map(|h| h.record.node_id).collect();
            if let Ok(cw_engine) = self.engine_handle() {
                cw_engine.cache_warmer.record_co_access(&node_ids);
            }
        }

        if distance_metric == crate::node::DistanceMetric::Euclidean {
            for hit in hits.iter_mut() {
                hit.score = -(-hit.score).max(0.0).sqrt();
            }
        }
        Ok(hits)
    }
    pub(super) fn vector_memory_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
        method: Option<crate::index::IndexType>,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        if query_vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let engine = self.engine_handle()?;

        // ---- Selectivity-based strategy ----
        let strategy = CostEstimator::new(&engine).select_filter_strategy(filters);

        // PreFilter: skip HNSW entirely, brute-force on the filtered subset.
        if strategy == FilterStrategy::PreFilter {
            return self.vector_memory_search_prefilter(
                namespace,
                query_vector,
                filters,
                top_k,
                distance_metric,
            );
        }

        // InFilter: build bitset from matching records → pass as query_mask.
        // PostFilter: keep ALL_BITSET (no pre-filter).
        let query_mask = match strategy {
            FilterStrategy::InFilter => self.bitset_from_filters(namespace, filters)?,
            _ => crate::node::ALL_BITSET.clone(),
        };

        // Short-circuit: no records match the filter → empty result.
        if query_mask.is_empty() {
            return Ok(Vec::new());
        }

        // InFilter uses a slightly larger budget because we know post-filtering
        // will discard some candidates; PostFilter uses the standard budget.
        let budget = if strategy == FilterStrategy::InFilter {
            (top_k.saturating_mul(15)).min(750).max(top_k)
        } else {
            (top_k.saturating_mul(10)).min(500).max(top_k)
        };

        let candidates = {
            let index = engine.vec_index();
            // OLD-21: decide which index backend this query should route through
            // (Flat / IVF / HNSW). `select_index_strategy` returns the single
            // authority for the decision; the metric lets operators observe it.
            // A per-search `method` override (FEAT-04) takes precedence; without
            // one, `search_nearest` executes the matching backend internally via
            // the index's `flat_threshold` + `index_type`.
            let routing =
                method.unwrap_or_else(|| CostEstimator::new(&engine).select_index_strategy());
            crate::metrics::record_vector_index_routing(routing);
            // After LSM compaction, nodes may reside on any level (L0..L3).
            // Pass `None` to force HNSW to use inline vec_data for distance
            // computation — this is correct for all levels since `get()` later
            // resolves the packed offset to the right segment.
            match method {
                Some(m) => index.search_with_method(m, query_vector, &query_mask, budget),
                None => index.search(query_vector, &query_mask, budget, None, distance_metric),
            }
        };

        let mut hits = Vec::with_capacity(top_k);
        {
            let candidate_ids: Vec<u128> = candidates.iter().map(|(id, _)| *id).collect();
            let mut node_map: std::collections::HashMap<u128, UnifiedNode> =
                std::collections::HashMap::with_capacity(candidate_ids.len());
            for n in engine.get_many(&candidate_ids)? {
                node_map.insert(n.id, n);
            }
            for (node_id, raw_score) in candidates {
                if hits.len() >= top_k {
                    break;
                }
                if let Some(node) = node_map.get(&node_id) {
                    if let Some(record) = memory_record_from_node(node) {
                        // For InFilter, the bitset already guarantees the
                        // record matches filters (we built it from
                        // records_for_namespace). Only namespace check needed.
                        // For PostFilter, still need matches_memory_filters.
                        let passes = if strategy == FilterStrategy::PostFilter {
                            record.namespace == *namespace
                                && matches_memory_filters(&record, filters)
                        } else {
                            record.namespace == *namespace
                        };
                        if passes {
                            let score = raw_score;
                            hits.push(VantaMemorySearchHit {
                                score,
                                record,
                                explanation: None,
                            });
                        }
                    }
                }
            }
        }

        // Brute-force fallback for PostFilter and InFilter.
        // PreFilter already handled above (no HNSW call).
        // InFilter's query_mask approach fails because node bitsets are
        // never populated on insert, so HNSW rejects all candidates.
        // Fall through to records_for_namespace + brute-force vector scan.
        if hits.is_empty() && strategy != FilterStrategy::PreFilter && !query_vector.is_empty() {
            crate::index::auto_tune::AutoTune::report_brute_fallback();
            for record in self.records_for_namespace(namespace, filters)? {
                let Some(vector) = record.vector.as_ref() else {
                    continue;
                };
                if vector.len() != query_vector.len() {
                    continue;
                }
                let score = match distance_metric {
                    crate::node::DistanceMetric::Cosine => cosine_sim_f32(query_vector, vector),
                    crate::node::DistanceMetric::Euclidean => {
                        -crate::index::euclidean_distance_squared_f32(query_vector, vector)
                    }
                    // Sparse search has its own brute-force path over sparse_vectors.
                    crate::node::DistanceMetric::SparseDot => 0.0,
                };
                hits.push(VantaMemorySearchHit {
                    score,
                    record,
                    explanation: None,
                });
            }
            crate::planner::sort_hits(&mut hits);
            hits.truncate(top_k);
            if distance_metric == crate::node::DistanceMetric::Euclidean {
                for hit in hits.iter_mut() {
                    hit.score = -(-hit.score).max(0.0).sqrt();
                }
            }
        } else if !hits.is_empty() {
            crate::index::auto_tune::AutoTune::report_success();
        }

        // Feed search results into cache warmer co-access tracking
        if hits.len() >= 2 {
            let node_ids: Vec<u128> = hits.iter().map(|h| h.record.node_id).collect();
            engine.cache_warmer.record_co_access(&node_ids);
        }

        Ok(hits)
    }
}
