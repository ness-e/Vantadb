use super::builder::VantaEmbedded;
use super::serialization::{validate_metadata, validate_namespace};
use super::types::*;
use crate::error::{Result, VantaError};
use std::collections::BTreeMap;
use tracing;

pub(crate) mod debug;
pub(crate) mod phrase;
pub(crate) mod snippet;
pub(crate) mod text_index;

pub(crate) mod audit;
pub(crate) mod debug_ops;
pub(crate) mod explain;
pub(crate) mod hybrid;
pub(crate) mod lexical;
pub(crate) mod multi;
pub(crate) mod sparse;
pub(crate) mod vector;

impl VantaEmbedded {
    /// Hybrid search across memory records combining text (BM25) and vector (HNSW) retrieval.
    /// Route selection (text-only, vector-only, hybrid) is automatic based on the request payload.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::VantaConfig;
    /// use vantadb::{
    ///     BackendKind, VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest,
    /// };
    ///
    /// let db = VantaEmbedded::open_with_config(VantaConfig {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(VantaMemoryInput::new(
    ///     "docs",
    ///     "fox",
    ///     "The quick brown fox jumps over the lazy dog",
    /// ))
    /// .expect("put first record");
    /// db.put(VantaMemoryInput::new(
    ///     "docs",
    ///     "sleepy",
    ///     "The lazy dog sleeps all day",
    /// ))
    /// .expect("put second record");
    ///
    /// let hits = db
    ///     .search(VantaMemorySearchRequest {
    ///         namespace: "docs".into(),
    ///         text_query: Some("fox".into()),
    ///         top_k: 10,
    ///         ..Default::default()
    ///     })
    ///     .expect("text search");
    ///
    /// // Only the first record contains the word "fox".
    /// assert_eq!(hits.len(), 1);
    /// assert_eq!(hits[0].record.payload, "The quick brown fox jumps over the lazy dog");
    ///
    /// db.close().expect("close database");
    /// ```
    pub fn search(&self, request: VantaMemorySearchRequest) -> Result<Vec<VantaMemorySearchHit>> {
        let exclude_superseded = request.exclude_superseded;
        let mut hits = self.search_impl(request, None)?;
        if exclude_superseded {
            // ADR-028: drop superseded records at final assembly — no index change.
            hits.retain(|hit| hit.record.superseded_by.is_none());
        }
        Ok(hits)
    }

    /// Same as [`search`](Self::search) with an explicit index backend override
    /// for the dense-vector portion of the query.
    ///
    /// `method` accepts `Ivf`, `Scann`, `Flat` or `Hnsw`. `None` (default)
    /// keeps the automatic engine routing completely untouched.
    pub fn search_with_method(
        &self,
        request: VantaMemorySearchRequest,
        method: Option<crate::index::IndexType>,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let exclude_superseded = request.exclude_superseded;
        let mut hits = self.search_impl(request, method)?;
        if exclude_superseded {
            // ADR-028: drop superseded records at final assembly — no index change.
            hits.retain(|hit| hit.record.superseded_by.is_none());
        }
        Ok(hits)
    }

    #[tracing::instrument(skip(self, request), err)]
    fn search_impl(
        &self,
        request: VantaMemorySearchRequest,
        method: Option<crate::index::IndexType>,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = crate::planner::trimmed_text_query(&request);
        let has_vector = !request.query_vector.is_empty();
        let has_sparse = request
            .query_sparse
            .as_ref()
            .is_some_and(|sparse| !sparse.is_empty());

        if request.top_k == 0 {
            return Ok(Vec::new());
        }

        // ERR-028: a zero-norm cosine query is undefined (cosine = 0/0).
        // `search_nearest` cannot surface an error (Vec-returning trait, see
        // AUDREP-55), so without this guard every binding would show a silent
        // empty result — indistinguishable from "no matches" — instead of an
        // error. Reject here so Python/MCP/WASM all report InvalidInput.
        if request.distance_metric == crate::node::DistanceMetric::Cosine
            && !request.query_vector.is_empty()
            && crate::index::f32_l2_norm(&request.query_vector) < f32::EPSILON
        {
            return Err(VantaError::InvalidInput(
                "zero-norm cosine query vector is undefined; use a non-zero vector \
                 or the euclidean distance metric (AUDREP-55, ERR-028)"
                    .into(),
            ));
        }

        if request.explain {
            let engine = self.engine_handle()?;
            let (hits, text_ranks, vector_ranks) = match (text_query, has_vector, has_sparse) {
                (Some(text_query), true, _) => {
                    let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                    let lexical_hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        budget,
                    )?;
                    let vector_hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        budget,
                        request.distance_metric,
                        method,
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
                            crate::planner::fuse_rrf_many(vec![
                                lexical_hits,
                                vector_hits,
                                sparse_hits,
                            ])
                        }
                        _ => {
                            let (hits, _report) =
                                crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                            hits
                        }
                    };
                    hits.truncate(request.top_k);
                    (hits, text_ranks, vector_ranks)
                }
                (Some(text_query), false, true) => {
                    let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                    let lexical_hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        budget,
                    )?;
                    let sparse_hits = self.sparse_memory_search(
                        &request.namespace,
                        request.query_sparse.as_ref().unwrap(),
                        &request.filters,
                        budget,
                    )?;
                    let text_ranks = debug::rank_map(&lexical_hits);
                    let mut hits = crate::planner::fuse_rrf_many(vec![lexical_hits, sparse_hits]);
                    hits.truncate(request.top_k);
                    (hits, text_ranks, BTreeMap::new())
                }
                (Some(text_query), false, _) => {
                    let hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        request.top_k,
                    )?;
                    let text_ranks = debug::rank_map(&hits);
                    (hits, text_ranks, BTreeMap::new())
                }
                (None, true, true) => {
                    let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                    let vector_hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        budget,
                        request.distance_metric,
                        method,
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
                    (hits, BTreeMap::new(), vector_ranks)
                }
                (None, true, _) => {
                    let hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        request.top_k,
                        request.distance_metric,
                        method,
                    )?;
                    let vector_ranks = debug::rank_map(&hits);
                    (hits, BTreeMap::new(), vector_ranks)
                }
                (None, false, true) => {
                    let hits = self.sparse_memory_search(
                        &request.namespace,
                        request.query_sparse.as_ref().unwrap(),
                        &request.filters,
                        request.top_k,
                    )?;
                    (hits, BTreeMap::new(), BTreeMap::new())
                }
                (None, false, _) => (Vec::new(), BTreeMap::new(), BTreeMap::new()),
            };

            let explained_hits = hits
                .into_iter()
                .map(|mut hit| {
                    let explanation = debug::explain_hit(
                        &engine,
                        hit.clone(),
                        text_query,
                        &text_ranks,
                        &vector_ranks,
                    )?;
                    hit.explanation = Some(explanation);
                    Ok(hit)
                })
                .collect::<Result<Vec<_>>>()?;

            return Ok(explained_hits);
        }

        match (text_query, has_vector, has_sparse) {
            (Some(text_query), true, _) => {
                crate::metrics::record_planner_hybrid_query();
                self.hybrid_search(
                    &request.namespace,
                    &request.query_vector,
                    text_query,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                    request.query_sparse.as_ref(),
                    method,
                )
            }
            (Some(text_query), false, true) => {
                crate::metrics::record_planner_hybrid_query();
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let sparse_hits = self.sparse_memory_search(
                    &request.namespace,
                    request.query_sparse.as_ref().unwrap(),
                    &request.filters,
                    budget,
                )?;
                let mut hits = crate::planner::fuse_rrf_many(vec![lexical_hits, sparse_hits]);
                hits.truncate(request.top_k);
                Ok(hits)
            }
            (Some(text_query), false, _) => {
                crate::metrics::record_planner_text_only_query();
                self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )
            }
            (None, true, true) => {
                crate::metrics::record_planner_hybrid_query();
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
                let mut hits = crate::planner::fuse_rrf_many(vec![vector_hits, sparse_hits]);
                hits.truncate(request.top_k);
                Ok(hits)
            }
            (None, true, _) => {
                crate::metrics::record_planner_vector_only_query();
                self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                    method,
                )
            }
            (None, false, true) => {
                crate::metrics::record_planner_sparse_only_query();
                self.sparse_memory_search(
                    &request.namespace,
                    request.query_sparse.as_ref().unwrap(),
                    &request.filters,
                    request.top_k,
                )
            }
            (None, false, _) => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests;
