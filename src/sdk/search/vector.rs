use super::super::builder::Embedded;
use super::super::serialization::{
    matches_memory_filters, record_from_node, validate_vector_dimension,
};
use super::super::types::*;
use crate::cost_estimator::{CostEstimator, FilterStrategy};
use crate::error::Result;
use crate::index::cosine_sim_f32;
use crate::node::UnifiedNode;

impl Embedded {
    /// Exact distance scan over a pre-filtered set of records (TEC-11 / Q11).
    ///
    /// Evaluates vector distance directly without traversing the HNSW graph,
    /// guaranteeing recall@k = 1.0 when candidate selectivity is very high (< 0.5%)
    /// or when candidate set is small (<= top_k.max(32)).
    fn exact_scan(
        &self,
        records: Vec<MemoryRecord>,
        query_vector: &[f32],
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
    ) -> Result<Vec<MemorySearchHit>> {
        let mut hits = Vec::with_capacity(top_k.min(records.len()));
        for record in records {
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
            hits.push(MemorySearchHit {
                score,
                record,
                explanation: None,
            });
        }
        super::fusion::sort_hits(&mut hits);
        hits.truncate(top_k);

        // Feed search results into cache warmer co-access tracking
        if hits.len() >= 2 {
            let node_ids: Vec<u128> = hits.iter().map(|h| h.record.node_id).collect();
            if let Ok(cw_engine) = self.engine_handle() {
                cw_engine.cache.warmer.record_co_access(&node_ids);
            }
        }

        if distance_metric == crate::node::DistanceMetric::Euclidean {
            for hit in hits.iter_mut() {
                hit.score = -(-hit.score).max(0.0).sqrt();
            }
        }
        Ok(hits)
    }

    /// Pre-filter path: use `records_for_namespace` to fetch only records
    /// matching the filters, then brute-force vector similarity on the
    /// (typically small) result set.
    fn vector_memory_search_prefilter(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &MemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
    ) -> Result<Vec<MemorySearchHit>> {
        let records = self.records_for_namespace(namespace, filters)?;
        self.exact_scan(records, query_vector, top_k, distance_metric)
    }
    pub(super) fn vector_memory_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &MemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
        method: Option<crate::index::IndexType>,
    ) -> Result<Vec<MemorySearchHit>> {
        if query_vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let engine = self.engine_handle()?;

        // FIND-411: the query vector must match the dimension of the vectors
        // held by the index. A mismatch used to be scored as a *perfect*
        // match (the distance kernels returned 0.0 for two vectors of
        // different lengths), so a malformed query silently returned the
        // worst candidates as the best ones.
        validate_vector_dimension(self.index_vector_dimension(&engine), query_vector.len())?;

        if !filters.is_empty() {
            crate::metrics::record_filtered_vector_query();
        }

        // ---- Selectivity-based strategy ----
        let estimator = CostEstimator::new(&engine);
        let strategy = estimator.select_filter_strategy(filters);

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

        // Active filters: inspect precalculated matching candidates and real selectivity.
        // If matching_ids.len() / total_population < 0.005 (0.5%) or matching_ids.len() <= top_k.max(32),
        // commute automatically to exact_scan without walking the HNSW graph (TEC-11 / Q11 Opción B+C).
        let (matching_records, real_selectivity) = if !filters.is_empty() {
            let records = self.records_for_namespace(namespace, filters)?;
            if records.is_empty() {
                return Ok(Vec::new());
            }

            let total_population = engine.hnsw.load().node_count();
            let real_sel = if total_population > 0 {
                records.len() as f32 / total_population as f32
            } else {
                0.0
            };

            if real_sel < crate::cost_estimator::EXACT_SCAN_SELECTIVITY_THRESHOLD
                || records.len() <= top_k.max(32)
            {
                return self.exact_scan(records, query_vector, top_k, distance_metric);
            }

            (Some(records), real_sel)
        } else {
            (None, 1.0)
        };

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

        // InFilter uses adaptive ef_search tuned by selectivity; PostFilter uses standard budget.
        let budget = if strategy == FilterStrategy::InFilter {
            let base_ef = (top_k.saturating_mul(15)).min(750).max(top_k);
            let est_sel = estimator.joint_selectivity(filters);
            let selectivity = if real_selectivity < 1.0 {
                real_selectivity
            } else {
                est_sel
            };
            CostEstimator::adaptive_ef_search(base_ef, top_k, selectivity)
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
                Some(m) => index.search_with_method(
                    m,
                    query_vector,
                    &query_mask,
                    budget,
                    distance_metric,
                )?,
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
                    if let Some(record) = record_from_node(node) {
                        // For both InFilter and PostFilter, check namespace and exact filters.
                        // InFilter's Bloom bitset prunes non-matching nodes during graph search,
                        // while matches_memory_filters eliminates Bloom false positives.
                        let passes = record.namespace == *namespace
                            && matches_memory_filters(&record, filters);
                        if passes {
                            let score = raw_score;
                            hits.push(MemorySearchHit {
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
        // When index pruning or strict query mask filtering yielded zero candidates,
        // ensure high recall by scanning the namespace directly.
        // Fall through to records_for_namespace + brute-force vector scan to guarantee recall.
        if hits.is_empty() && strategy != FilterStrategy::PreFilter && !query_vector.is_empty() {
            if !filters.is_empty() {
                crate::metrics::record_brute_fallback_query();
            }
            crate::index::auto_tune::AutoTune::report_brute_fallback();
            let records = match matching_records {
                Some(r) => r,
                None => self.records_for_namespace(namespace, filters)?,
            };
            hits = self.exact_scan(records, query_vector, top_k, distance_metric)?;
        } else if !hits.is_empty() {
            crate::index::auto_tune::AutoTune::report_success();
        }

        // Feed search results into cache warmer co-access tracking
        if hits.len() >= 2 {
            let node_ids: Vec<u128> = hits.iter().map(|h| h.record.node_id).collect();
            engine.cache.warmer.record_co_access(&node_ids);
        }

        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::DistanceMetric;
    use crate::sdk::connect::connect;

    #[test]
    fn test_filtered_vector_search_exact_scan_low_selectivity() {
        let db = connect(":memory:").expect("in-memory db open");
        let namespace = "test_exact_scan";

        // Insert 500 records. Exactly 2 records have target metadata (`tag = "rare"`),
        // giving real selectivity 2 / 500 = 0.004 (0.4% < 0.5% threshold).
        let query_vec = vec![1.0, 0.0, 0.0, 0.0];

        // Target record 1: very close to query_vec
        let mut target_meta_1 = MemoryMetadata::new();
        target_meta_1.insert("tag".into(), Value::String("rare".into()));
        db.put(MemoryInput {
            namespace: namespace.into(),
            key: "target_1".into(),
            payload: "target document 1".into(),
            metadata: target_meta_1,
            vector: Some(vec![0.99, 0.01, 0.0, 0.0]),
            ..Default::default()
        })
        .expect("put target 1");

        // Target record 2: also close to query_vec
        let mut target_meta_2 = MemoryMetadata::new();
        target_meta_2.insert("tag".into(), Value::String("rare".into()));
        db.put(MemoryInput {
            namespace: namespace.into(),
            key: "target_2".into(),
            payload: "target document 2".into(),
            metadata: target_meta_2,
            vector: Some(vec![0.95, 0.05, 0.0, 0.0]),
            ..Default::default()
        })
        .expect("put target 2");

        // 498 distractor records with `tag = "common"` and orthogonal/different vectors
        for i in 0..498 {
            let mut common_meta = MemoryMetadata::new();
            common_meta.insert("tag".into(), Value::String("common".into()));
            let angle = (i as f32) * 0.01;
            db.put(MemoryInput {
                namespace: namespace.into(),
                key: format!("distractor_{i}"),
                payload: format!("distractor {i}"),
                metadata: common_meta,
                vector: Some(vec![0.0, 1.0, angle.sin(), angle.cos()]),
                ..Default::default()
            })
            .expect("put distractor");
        }

        // Verify total population in index is 500
        let engine = db.engine_handle().expect("engine handle");
        assert_eq!(engine.hnsw.load().node_count(), 500);

        // Search with filter for `tag = "rare"`
        let mut filters = MemoryMetadata::new();
        filters.insert("tag".into(), Value::String("rare".into()));

        let hits = db
            .vector_memory_search(
                namespace,
                &query_vec,
                &filters,
                5,
                DistanceMetric::Cosine,
                None,
            )
            .expect("vector search should succeed");

        // Selectivity is 2/500 = 0.4% (< 0.5% EXACT_SCAN_SELECTIVITY_THRESHOLD),
        // and matching candidate count (2) <= top_k.max(32) (32).
        // It must commute to exact_scan and return all matching records (recall@5 = 1.0).
        assert_eq!(hits.len(), 2, "must find exactly the 2 target records");
        let found_keys: std::collections::HashSet<String> =
            hits.iter().map(|h| h.record.key.clone()).collect();
        assert!(found_keys.contains("target_1"));
        assert!(found_keys.contains("target_2"));

        // All returned records must pass the filter
        for hit in &hits {
            assert_eq!(
                hit.record.metadata.get("tag"),
                Some(&Value::String("rare".into()))
            );
        }

        // Check ranking: target_1 is closer to query_vec than target_2
        assert_eq!(hits[0].record.key, "target_1");
        assert_eq!(hits[1].record.key, "target_2");
        assert!(hits[0].score > hits[1].score);
    }
}
