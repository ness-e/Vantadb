use super::super::builder::VantaEmbedded;
use super::super::serialization::{validate_metadata, validate_namespace};
#[cfg(debug_assertions)]
use super::super::serialization::{DERIVED_INDEX_STATE_KEY, TEXT_INDEX_STATE_KEY};
use super::super::types::*;
use super::debug;
use crate::backend::BackendPartition;
#[cfg(debug_assertions)]
use crate::backend::BackendWriteOp;
use crate::error::{Result, VantaError};

impl VantaEmbedded {
    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_memory_breakdown(&self) -> serde_json::Value {
        let metrics = self.operational_metrics();
        serde_json::json!({
            "process_rss_bytes": metrics.process_rss_bytes,
            "process_virtual_bytes": metrics.process_virtual_bytes,
            "hnsw_nodes_count": metrics.hnsw_nodes_count,
            "hnsw_logical_bytes": metrics.hnsw_logical_bytes,
            "mmap_resident_bytes": metrics.mmap_resident_bytes,
            "volatile_cache_entries": metrics.volatile_cache_entries,
            "volatile_cache_cap_bytes": metrics.volatile_cache_cap_bytes,
        })
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_derived_index_state_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            DERIVED_INDEX_STATE_KEY,
            b"corrupt-derived-index-state",
        )
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_clear_derived_indexes_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        for (key, _value) in engine.scan_partition(BackendPartition::NamespaceIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::NamespaceIndex,
                key,
            });
        }
        for (key, _value) in engine.scan_partition(BackendPartition::PayloadIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::PayloadIndex,
                key,
            });
        }
        engine.write_backend_batch(ops)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_state_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            TEXT_INDEX_STATE_KEY,
            b"corrupt-text-index-state",
        )
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_clear_text_index_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::TextIndex,
                key,
            });
        }
        engine.write_backend_batch(ops)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_posting_tf_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
        new_tf: u32,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let pkey = crate::text_index::posting_key(namespace, token, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &pkey)? else {
            return Err(VantaError::NotFound {
                kind: "posting".into(),
                id: "unknown".into(),
            });
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        let val = crate::text_index::posting_value(posting.node_id, new_tf, &posting.positions)?;
        engine.put_to_partition(BackendPartition::TextIndex, &pkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_posting_positions_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
        new_positions: Vec<u32>,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let pkey = crate::text_index::posting_key(namespace, token, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &pkey)? else {
            return Err(VantaError::NotFound {
                kind: "posting".into(),
                id: "unknown".into(),
            });
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        let val = crate::text_index::posting_value(posting.node_id, posting.tf, &new_positions)?;
        engine.put_to_partition(BackendPartition::TextIndex, &pkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_term_stats_for_tests(
        &self,
        namespace: &str,
        token: &str,
        new_df: u64,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let skey = crate::text_index::term_stats_key(namespace, token);
        let val = crate::text_index::term_stats_value(new_df)?;
        engine.put_to_partition(BackendPartition::TextIndex, &skey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_doc_stats_for_tests(
        &self,
        namespace: &str,
        key: &str,
        new_doc_len: u32,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let dkey = crate::text_index::doc_stats_key(namespace, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &dkey)? else {
            return Err(VantaError::NotFound {
                kind: "doc_stats".into(),
                id: "unknown".into(),
            });
        };
        let stats = crate::text_index::decode_doc_stats(&bytes)?;
        let val = crate::text_index::doc_stats_value(stats.node_id, new_doc_len)?;
        engine.put_to_partition(BackendPartition::TextIndex, &dkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_posting_keys_for_tests(&self) -> Result<Vec<Vec<u8>>> {
        let engine = self.engine_handle()?;
        let mut keys: Vec<Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .map(|(key, _value)| key)
            .filter(|key| !crate::text_index::is_internal_key(key))
            .collect();
        keys.sort();
        Ok(keys)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_posting_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
    ) -> Result<Option<(u128, u32)>> {
        let engine = self.engine_handle()?;
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::TextIndex,
            &crate::text_index::posting_key(namespace, token, key),
        )?
        else {
            return Ok(None);
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        Ok(Some((posting.node_id, posting.tf)))
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_audit_for_tests(&self) -> Result<VantaTextIndexAuditReport> {
        self.audit_text_index_deep(None)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_memory_search_plan_for_tests(
        &self,
        request: VantaMemorySearchRequest,
    ) -> Result<VantaMemorySearchDebugReport> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let (rrf_k, candidate_k) = crate::planner::resolve_search_profile(&request);
        let mode = crate::planner::search_mode(&request);
        let mut text_query = crate::planner::trimmed_text_query(&request);
        let mut has_vector = !request.query_vector.is_empty();
        let mut query_sparse = request
            .query_sparse
            .as_ref()
            .filter(|sparse| !sparse.is_empty());
        match mode {
            SearchProfileMode::Keyword => {
                has_vector = false;
                query_sparse = None;
            }
            SearchProfileMode::Vector => {
                text_query = None;
            }
            SearchProfileMode::Hybrid => {}
        }
        if request.top_k == 0 {
            return Ok(VantaMemorySearchDebugReport {
                route: "empty".to_string(),
                budget: 0,
                text_candidates: 0,
                vector_candidates: 0,
                fused_candidates: 0,
                top_identities: Vec::new(),
            });
        }

        match (text_query, has_vector, query_sparse) {
            (Some(text_query), true, _) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k, candidate_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                    None,
                )?;
                let text_candidates = lexical_hits.len();
                let vector_candidates = vector_hits.len();
                let mut fused_hits = match query_sparse {
                    Some(query_sparse) => {
                        let sparse_hits = self.sparse_memory_search(
                            &request.namespace,
                            query_sparse,
                            &request.filters,
                            budget,
                        )?;
                        crate::planner::fuse_rrf_many(
                            vec![lexical_hits, vector_hits, sparse_hits],
                            rrf_k,
                        )
                    }
                    _ => crate::planner::fuse_rrf(lexical_hits, vector_hits, rrf_k),
                };
                let fused_candidates = fused_hits.len();
                fused_hits.truncate(request.top_k);
                Ok(VantaMemorySearchDebugReport {
                    route: "hybrid".to_string(),
                    budget,
                    text_candidates,
                    vector_candidates,
                    fused_candidates,
                    top_identities: debug::hit_identities(&fused_hits),
                })
            }
            (Some(text_query), false, Some(query_sparse)) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k, candidate_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let sparse_hits = self.sparse_memory_search(
                    &request.namespace,
                    query_sparse,
                    &request.filters,
                    budget,
                )?;
                let text_candidates = lexical_hits.len();
                let vector_candidates = sparse_hits.len();
                let mut fused_hits =
                    crate::planner::fuse_rrf_many(vec![lexical_hits, sparse_hits], rrf_k);
                let fused_candidates = fused_hits.len();
                fused_hits.truncate(request.top_k);
                Ok(VantaMemorySearchDebugReport {
                    route: "hybrid".to_string(),
                    budget,
                    text_candidates,
                    vector_candidates,
                    fused_candidates,
                    top_identities: debug::hit_identities(&fused_hits),
                })
            }
            (Some(text_query), false, _) => {
                let hits = self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )?;
                Ok(VantaMemorySearchDebugReport {
                    route: "text-only".to_string(),
                    budget: request.top_k,
                    text_candidates: hits.len(),
                    vector_candidates: 0,
                    fused_candidates: hits.len(),
                    top_identities: debug::hit_identities(&hits),
                })
            }
            (None, true, Some(query_sparse)) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k, candidate_k);
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                    None,
                )?;
                let sparse_hits = self.sparse_memory_search(
                    &request.namespace,
                    query_sparse,
                    &request.filters,
                    budget,
                )?;
                let vector_candidates = vector_hits.len();
                let mut fused_hits =
                    crate::planner::fuse_rrf_many(vec![vector_hits, sparse_hits], rrf_k);
                let fused_candidates = fused_hits.len();
                fused_hits.truncate(request.top_k);
                Ok(VantaMemorySearchDebugReport {
                    route: "hybrid".to_string(),
                    budget,
                    text_candidates: 0,
                    vector_candidates,
                    fused_candidates,
                    top_identities: debug::hit_identities(&fused_hits),
                })
            }
            (None, true, _) => {
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                    None,
                )?;
                Ok(VantaMemorySearchDebugReport {
                    route: "vector-only".to_string(),
                    budget: request.top_k,
                    text_candidates: 0,
                    vector_candidates: hits.len(),
                    fused_candidates: hits.len(),
                    top_identities: debug::hit_identities(&hits),
                })
            }
            (None, false, Some(query_sparse)) => {
                let hits = self.sparse_memory_search(
                    &request.namespace,
                    query_sparse,
                    &request.filters,
                    request.top_k,
                )?;
                Ok(VantaMemorySearchDebugReport {
                    route: "sparse-only".to_string(),
                    budget: request.top_k,
                    text_candidates: 0,
                    vector_candidates: hits.len(),
                    fused_candidates: hits.len(),
                    top_identities: debug::hit_identities(&hits),
                })
            }
            (None, false, _) => Ok(VantaMemorySearchDebugReport {
                route: "empty".to_string(),
                budget: 0,
                text_candidates: 0,
                vector_candidates: 0,
                fused_candidates: 0,
                top_identities: Vec::new(),
            }),
        }
    }
}
