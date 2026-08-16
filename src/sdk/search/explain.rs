use super::super::builder::VantaEmbedded;
use super::super::serialization::{validate_metadata, validate_namespace};
use super::super::types::*;
use super::debug;
use crate::error::Result;
use std::collections::BTreeMap;
use tracing;

impl VantaEmbedded {
    /// Explain the search plan for a memory search request without executing it.
    #[tracing::instrument(skip(self, request), err)]
    pub fn explain_memory_search(
        &self,
        request: VantaMemorySearchRequest,
    ) -> Result<VantaSearchExplanation> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = request
            .text_query
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let has_vector = !request.query_vector.is_empty();
        let has_sparse = request
            .query_sparse
            .as_ref()
            .is_some_and(|sparse| !sparse.is_empty());
        if request.top_k == 0 {
            return Ok(VantaSearchExplanation {
                route: "empty".to_string(),
                hits: Vec::new(),
                fusion_report: None,
            });
        }

        let engine = self.engine_handle()?;
        #[allow(clippy::type_complexity)]
        let (route, hits, text_ranks, vector_ranks, fusion_report): (
            String,
            Vec<VantaMemorySearchHit>,
            BTreeMap<(String, String), usize>,
            BTreeMap<(String, String), usize>,
            Option<VantaHybridFusionReport>,
        ) = match (text_query, has_vector, has_sparse) {
            (Some(text_query), true, _) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
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
                let text_ranks = debug::rank_map(&lexical_hits);
                let vector_ranks = debug::rank_map(&vector_hits);
                let mut hits = match request.query_sparse.as_ref() {
                    Some(query_sparse) if !query_sparse.is_empty() => {
                        let sparse_hits = self.sparse_memory_search(
                            &request.namespace,
                            query_sparse,
                            &request.filters,
                            budget,
                        )?;
                        crate::planner::fuse_rrf_many(vec![lexical_hits, vector_hits, sparse_hits])
                    }
                    _ => {
                        let (hits, _report) =
                            crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                        hits
                    }
                };
                hits.truncate(request.top_k);
                ("hybrid".to_string(), hits, text_ranks, vector_ranks, None)
            }
            (Some(text_query), false, true) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let sparse_hits = self.sparse_memory_search(
                    &request.namespace,
                    request.query_sparse.as_ref().unwrap(),
                    &request.filters,
                    budget,
                )?;
                let text_ranks = debug::rank_map(&lexical_hits);
                let mut hits = crate::planner::fuse_rrf_many(vec![lexical_hits, sparse_hits]);
                hits.truncate(request.top_k);
                (
                    "hybrid".to_string(),
                    hits,
                    text_ranks,
                    BTreeMap::new(),
                    None,
                )
            }
            (Some(text_query), false, _) => {
                let hits = self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )?;
                let text_ranks = debug::rank_map(&hits);
                (
                    "text-only".to_string(),
                    hits,
                    text_ranks,
                    BTreeMap::new(),
                    None,
                )
            }
            (None, true, true) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
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
                    request.query_sparse.as_ref().unwrap(),
                    &request.filters,
                    budget,
                )?;
                let vector_ranks = debug::rank_map(&vector_hits);
                let mut hits = crate::planner::fuse_rrf_many(vec![vector_hits, sparse_hits]);
                hits.truncate(request.top_k);
                (
                    "hybrid".to_string(),
                    hits,
                    BTreeMap::new(),
                    vector_ranks,
                    None,
                )
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
                let vector_ranks = debug::rank_map(&hits);
                (
                    "vector-only".to_string(),
                    hits,
                    BTreeMap::new(),
                    vector_ranks,
                    None,
                )
            }
            (None, false, true) => {
                let hits = self.sparse_memory_search(
                    &request.namespace,
                    request.query_sparse.as_ref().unwrap(),
                    &request.filters,
                    request.top_k,
                )?;
                (
                    "sparse-only".to_string(),
                    hits,
                    BTreeMap::new(),
                    BTreeMap::new(),
                    None,
                )
            }
            (None, false, _) => {
                return Ok(VantaSearchExplanation {
                    route: "empty".to_string(),
                    hits: Vec::new(),
                    fusion_report: None,
                });
            }
        };

        let explained_hits = hits
            .into_iter()
            .map(|hit| debug::explain_hit(&engine, hit, text_query, &text_ranks, &vector_ranks))
            .collect::<Result<Vec<_>>>()?;

        Ok(VantaSearchExplanation {
            route,
            hits: explained_hits,
            fusion_report,
        })
    }
}
