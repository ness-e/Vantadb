use super::builder::Embedded;
use super::serialization::{validate_metadata, validate_namespace};
use super::types::*;
use crate::error::{Error, Result};
use std::collections::BTreeMap;
use tracing;

pub(crate) mod debug;
pub(crate) mod phrase;
pub(crate) mod snippet;
pub(crate) mod text_index;

pub(crate) mod audit;
#[cfg(debug_assertions)]
pub(crate) mod debug_ops;
pub(crate) mod explain;
pub(crate) mod fusion;
pub(crate) mod hybrid;
pub(crate) mod lexical;
pub(crate) mod mmr;
pub(crate) mod multi;
pub(crate) mod page;
pub(crate) mod sparse;
pub(crate) mod vector;

impl Embedded {
    /// Hybrid search across memory records combining text (BM25) and vector (HNSW) retrieval.
    /// Route selection (text-only, vector-only, hybrid) is automatic based on the request payload.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::Config;
    /// use vantadb::{
    ///     BackendKind, Embedded, MemoryInput, MemorySearchRequest,
    /// };
    ///
    /// let db = Embedded::open_with_config(Config {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(MemoryInput::new(
    ///     "docs",
    ///     "fox",
    ///     "The quick brown fox jumps over the lazy dog",
    /// ))
    /// .expect("put first record");
    /// db.put(MemoryInput::new(
    ///     "docs",
    ///     "sleepy",
    ///     "The lazy dog sleeps all day",
    /// ))
    /// .expect("put second record");
    ///
    /// let hits = db
    ///     .search(MemorySearchRequest {
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
    pub fn search(&self, request: MemorySearchRequest) -> Result<Vec<MemorySearchHit>> {
        Ok(self.search_page(request)?.hits)
    }

    /// Search with cursor-based pagination (WIRE-08).
    ///
    /// Behaves like [`search`](Self::search) but returns a [`MemorySearchPage`]
    /// carrying an opaque `next_cursor` when the page is full. Pass that token
    /// back in `MemorySearchRequest::cursor` to resume: the next page returns
    /// hits after the last returned hit's identity in the current ranking.
    ///
    /// Resume is **best-effort, not a snapshot**: a hit returned in a previous
    /// page can be returned again (or skipped) when interleaved writes reorder
    /// its rank across the anchor — BM25/IDF are recalculated corpus-wide on
    /// every write. Guaranteed: writes that rank *before* the anchor are never
    /// duplicated by the resumed page. A page with fewer than `top_k` hits is
    /// the last page (`next_cursor == None`); `top_k == 0` returns an empty
    /// page with no cursor (ERR-033: limit 0 means no records). A stronger
    /// cursor (snapshot / server-side session) is tracked as FIND-183.
    ///
    /// The token is bound to the request's plan fingerprint (namespace, query,
    /// filters, metric, profile, range) and to the current process; a mismatch
    /// fails with the stable `SEARCH_CURSOR_INVALID` marker. Pagination is
    /// rejected for `mmr`/`group_by` requests (set-dependent selection) and is
    /// not available through `search_multi`/`search_all` (single-namespace
    /// plans only).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::{connect, MemorySearchRequest};
    ///
    /// let db = connect(":memory:").expect("open in-memory database");
    /// db.put(vantadb::MemoryInput::new("docs", "a", "alpha beta")).unwrap();
    /// db.put(vantadb::MemoryInput::new("docs", "b", "alpha gamma")).unwrap();
    /// db.put(vantadb::MemoryInput::new("docs", "c", "alpha delta")).unwrap();
    ///
    /// let first = db
    ///     .search_page(MemorySearchRequest {
    ///         namespace: "docs".into(),
    ///         text_query: Some("alpha".into()),
    ///         top_k: 2,
    ///         ..Default::default()
    ///     })
    ///     .expect("first page");
    /// assert_eq!(first.hits.len(), 2);
    /// assert!(first.next_cursor.is_some(), "full page yields a cursor");
    ///
    /// let second = db
    ///     .search_page(MemorySearchRequest {
    ///         namespace: "docs".into(),
    ///         text_query: Some("alpha".into()),
    ///         top_k: 2,
    ///         cursor: first.next_cursor,
    ///         ..Default::default()
    ///     })
    ///     .expect("second page");
    /// assert_eq!(second.hits.len(), 1);
    /// assert!(
    ///     !first.hits.iter().any(|h| h.record.key == second.hits[0].record.key),
    ///     "resumed page never repeats a hit"
    /// );
    /// assert!(second.next_cursor.is_none(), "short page is the last page");
    /// ```
    pub fn search_page(&self, request: MemorySearchRequest) -> Result<MemorySearchPage> {
        let (page, _boost_report) = page::run_search_page(self, request, None, None)?;
        Ok(page)
    }

    /// Same as [`search`](Self::search) with an explicit index backend override
    /// for the dense-vector portion of the query.
    ///
    /// `method` accepts `Ivf`, `Scann`, `Flat` or `Hnsw`. `None` (default)
    /// keeps the automatic engine routing completely untouched.
    pub fn search_with_method(
        &self,
        request: MemorySearchRequest,
        method: Option<crate::index::IndexType>,
    ) -> Result<Vec<MemorySearchHit>> {
        let (page, _boost_report) = page::run_search_page(self, request, method, None)?;
        Ok(page.hits)
    }

    /// Same as [`search_with_method`](Self::search_with_method) but returns the
    /// full [`MemorySearchPage`] (cursor + abstention signal), mirroring
    /// [`search_page`](Self::search_page). SCH-07: page-shaped transports
    /// (MCP search tools) need `abstained`/`abstention_reason` even when the
    /// dense-index backend is overridden; `search_with_method` alone would drop
    /// the signal silently.
    pub fn search_page_with_method(
        &self,
        request: MemorySearchRequest,
        method: Option<crate::index::IndexType>,
    ) -> Result<MemorySearchPage> {
        let (page, _boost_report) = page::run_search_page(self, request, method, None)?;
        Ok(page)
    }

    /// Search with an opt-in deterministic entity-cluster boost (WIRE-05).
    ///
    /// Behaves like [`search`](Self::search) but additionally applies the
    /// caller-supplied [`EntityBoost`] to every RRF-fused candidate list: hits
    /// that share an entity cluster with other fused candidates receive an
    /// additive score delta before the final ranking. The returned
    /// [`EntityBoostedSearch::boost_report`] carries per-hit provenance
    /// (cluster, peers, `base_score`, `delta`), so the boost is auditable and
    /// reversible — no stored data is mutated and an empty `boost` is
    /// byte-identical to [`search`](Self::search).
    ///
    /// The boost applies to RRF-fused routes (hybrid, text+sparse,
    /// vector+sparse); single-channel routes (text-only, vector-only,
    /// sparse-only) have no fusion score to boost and are returned unchanged.
    pub fn search_with_entity_boost(
        &self,
        request: MemorySearchRequest,
        boost: &EntityBoost,
    ) -> Result<EntityBoostedSearch> {
        let effective = (!boost.is_empty()).then_some(boost);
        let (page, boost_report) = page::run_search_page(self, request, None, effective)?;
        Ok(EntityBoostedSearch {
            hits: page.hits,
            boost_report,
        })
    }

    #[tracing::instrument(skip(self, request), err)]
    fn search_impl(
        &self,
        request: MemorySearchRequest,
        method: Option<crate::index::IndexType>,
        boost: Option<&EntityBoost>,
    ) -> Result<(Vec<MemorySearchHit>, EntityBoostReport)> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let (rrf_k, candidate_k) = fusion::resolve_search_profile(&request);
        let mode = fusion::search_mode(&request);
        let mut text_query = fusion::trimmed_text_query(&request);
        let mut has_vector = !request.query_vector.is_empty();
        let mut query_sparse = request
            .query_sparse
            .as_ref()
            .filter(|sparse| !sparse.is_empty());

        // MEM-01: el profile puede forzar el modo de búsqueda. Keyword ignora el
        // canal vectorial (denso + sparse); Vector ignora el texto.
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
            return Ok((Vec::new(), EntityBoostReport::default()));
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
            return Err(Error::InvalidInput(
                "zero-norm cosine query vector is undefined; use a non-zero vector \
                 or the euclidean distance metric (AUDREP-55, ERR-028)"
                    .into(),
            ));
        }

        if request.explain {
            let engine = self.engine_handle()?;
            let (hits, text_ranks, vector_ranks, boost_report) =
                match (text_query, has_vector, query_sparse) {
                    (Some(text_query), true, _) => {
                        let budget = fusion::hybrid_candidate_budget(request.top_k, candidate_k);
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
                        let (mut hits, _fusion_report, boost_report) = match query_sparse {
                            Some(query_sparse) => {
                                let sparse_hits = self.sparse_memory_search(
                                    &request.namespace,
                                    query_sparse,
                                    &request.filters,
                                    budget,
                                )?;
                                let (hits, boost_report) = fusion::fuse_rrf_many_with_entity_boost(
                                    vec![lexical_hits, vector_hits, sparse_hits],
                                    rrf_k,
                                    boost,
                                );
                                (hits, None, boost_report)
                            }
                            _ => {
                                let (hits, report, boost_report) =
                                    fusion::fuse_rrf_impl(lexical_hits, vector_hits, rrf_k, boost);
                                (hits, Some(report), boost_report)
                            }
                        };
                        hits.truncate(request.top_k);
                        (hits, text_ranks, vector_ranks, boost_report)
                    }
                    (Some(text_query), false, Some(query_sparse)) => {
                        let budget = fusion::hybrid_candidate_budget(request.top_k, candidate_k);
                        let lexical_hits = self.lexical_search(
                            &request.namespace,
                            text_query,
                            &request.filters,
                            budget,
                        )?;
                        let sparse_hits = self.sparse_memory_search(
                            &request.namespace,
                            query_sparse,
                            &request.filters,
                            budget,
                        )?;
                        let text_ranks = debug::rank_map(&lexical_hits);
                        let (mut hits, boost_report) = fusion::fuse_rrf_many_with_entity_boost(
                            vec![lexical_hits, sparse_hits],
                            rrf_k,
                            boost,
                        );
                        hits.truncate(request.top_k);
                        (hits, text_ranks, BTreeMap::new(), boost_report)
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
                            hits,
                            text_ranks,
                            BTreeMap::new(),
                            EntityBoostReport::default(),
                        )
                    }
                    (None, true, Some(query_sparse)) => {
                        let budget = fusion::hybrid_candidate_budget(request.top_k, candidate_k);
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
                            query_sparse,
                            &request.filters,
                            budget,
                        )?;
                        let vector_ranks = debug::rank_map(&vector_hits);
                        let (mut hits, boost_report) = fusion::fuse_rrf_many_with_entity_boost(
                            vec![vector_hits, sparse_hits],
                            rrf_k,
                            boost,
                        );
                        hits.truncate(request.top_k);
                        (hits, BTreeMap::new(), vector_ranks, boost_report)
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
                        (
                            hits,
                            BTreeMap::new(),
                            vector_ranks,
                            EntityBoostReport::default(),
                        )
                    }
                    (None, false, Some(query_sparse)) => {
                        let hits = self.sparse_memory_search(
                            &request.namespace,
                            query_sparse,
                            &request.filters,
                            request.top_k,
                        )?;
                        (
                            hits,
                            BTreeMap::new(),
                            BTreeMap::new(),
                            EntityBoostReport::default(),
                        )
                    }
                    (None, false, _) => (
                        Vec::new(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                        EntityBoostReport::default(),
                    ),
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

            return Ok((explained_hits, boost_report));
        }

        match (text_query, has_vector, query_sparse) {
            (Some(text_query), true, _) => {
                crate::metrics::record_planner_hybrid_query();
                self.hybrid_search(
                    &request.namespace,
                    &request.query_vector,
                    text_query,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                    query_sparse,
                    method,
                    rrf_k,
                    candidate_k,
                    boost,
                )
            }
            (Some(text_query), false, Some(query_sparse)) => {
                crate::metrics::record_planner_hybrid_query();
                let budget = fusion::hybrid_candidate_budget(request.top_k, candidate_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let sparse_hits = self.sparse_memory_search(
                    &request.namespace,
                    query_sparse,
                    &request.filters,
                    budget,
                )?;
                let (mut hits, boost_report) = fusion::fuse_rrf_many_with_entity_boost(
                    vec![lexical_hits, sparse_hits],
                    rrf_k,
                    boost,
                );
                hits.truncate(request.top_k);
                Ok((hits, boost_report))
            }
            (Some(text_query), false, _) => {
                crate::metrics::record_planner_text_only_query();
                let hits = self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )?;
                Ok((hits, EntityBoostReport::default()))
            }
            (None, true, Some(query_sparse)) => {
                crate::metrics::record_planner_hybrid_query();
                let budget = fusion::hybrid_candidate_budget(request.top_k, candidate_k);
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
                let (mut hits, boost_report) = fusion::fuse_rrf_many_with_entity_boost(
                    vec![vector_hits, sparse_hits],
                    rrf_k,
                    boost,
                );
                hits.truncate(request.top_k);
                Ok((hits, boost_report))
            }
            (None, true, _) => {
                crate::metrics::record_planner_vector_only_query();
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                    method,
                )?;
                Ok((hits, EntityBoostReport::default()))
            }
            (None, false, Some(query_sparse)) => {
                crate::metrics::record_planner_sparse_only_query();
                let hits = self.sparse_memory_search(
                    &request.namespace,
                    query_sparse,
                    &request.filters,
                    request.top_k,
                )?;
                Ok((hits, EntityBoostReport::default()))
            }
            (None, false, _) => Ok((Vec::new(), EntityBoostReport::default())),
        }
    }
}

#[cfg(test)]
mod tests;
