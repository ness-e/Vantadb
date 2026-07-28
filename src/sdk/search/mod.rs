use super::builder::VantaEmbedded;
use super::serialization::{
    matches_memory_filters, memory_record_from_node, validate_metadata, validate_namespace,
};
#[cfg(debug_assertions)]
use super::serialization::{DERIVED_INDEX_STATE_KEY, TEXT_INDEX_STATE_KEY};
use super::types::*;
use crate::backend::BackendPartition;
#[cfg(debug_assertions)]
use crate::backend::BackendWriteOp;
use crate::error::{ChainedError, Result, VantaError};
use crate::index::cosine_sim_f32;
use crate::index::VecIndex;
use crate::node::{FilterBitset, UnifiedNode};
use crate::planner::HIGH_SELECTIVITY_THRESHOLD;
use crate::query::RelOp;
use crate::storage::StorageEngine;
pub(crate) mod debug;
pub(crate) mod phrase;
pub(crate) mod snippet;
pub(crate) mod text_index;

use std::collections::BTreeMap;
use tracing;
use web_time::Instant;

/// Selectivity threshold below which **PreFilter** is chosen:
/// scan metadata → build bitset → brute-force vector search on the small subset.
/// Filters with selectivity below this value match < 1 % of rows.
const PREFILTER_THRESHOLD: f32 = 0.01;

/// 3 filtering strategies ordered by increasing selectivity.
///
/// The optimizer picks one based on the estimated joint selectivity of all
/// query filters — how many rows survive the filter before vector search.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum FilterStrategy {
    /// Highly selective (joint_sel < 1 %): pre-filter metadata first, then
    /// vector-search only the matching records (brute-force on a tiny set).
    PreFilter,
    /// Moderately selective (1 % ≤ joint_sel < 10 %): build a bitset of
    /// matching node IDs and pass it as `query_mask` during HNSW traversal
    /// so the graph walk only visits candidates that survive the filter.
    InFilter,
    /// Low selectivity (joint_sel ≥ 10 %): let HNSW see everything, then
    /// post-filter results with `matches_memory_filters`.  Current default.
    PostFilter,
}

/// Convert `VantaValue` (SDK metadata type) → `FieldValue` (engine stats type).
///
/// Delegates to the existing `From<VantaValue>` impl in `conversions.rs`.
fn vanta_value_to_field_value(v: &VantaValue) -> crate::node::FieldValue {
    crate::node::FieldValue::from(v.clone())
}

/// Estimate the joint selectivity of all query filters against the engine's
/// cardinality statistics, then pick the best filtering strategy.
fn select_filter_strategy(engine: &StorageEngine, filters: &VantaMemoryMetadata) -> FilterStrategy {
    if filters.is_empty() {
        return FilterStrategy::PostFilter;
    }

    let mut joint_selectivity = 1.0f32;
    for (field, value) in filters.iter() {
        let fv = vanta_value_to_field_value(value);
        let sel = engine.get_estimated_selectivity(field, &RelOp::Eq, &fv);
        joint_selectivity *= sel;
    }

    if joint_selectivity < PREFILTER_THRESHOLD {
        FilterStrategy::PreFilter
    } else if joint_selectivity < HIGH_SELECTIVITY_THRESHOLD {
        FilterStrategy::InFilter
    } else {
        FilterStrategy::PostFilter
    }
}

impl VantaEmbedded {
    /// Hybrid search across memory records combining text (BM25) and vector (HNSW) retrieval.
    /// Route selection (text-only, vector-only, hybrid) is automatic based on the request payload.
    #[tracing::instrument(skip(self, request), err)]
    pub fn search(&self, request: VantaMemorySearchRequest) -> Result<Vec<VantaMemorySearchHit>> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = crate::planner::trimmed_text_query(&request);
        let has_vector = !request.query_vector.is_empty();

        if request.top_k == 0 {
            return Ok(Vec::new());
        }

        if request.explain {
            let engine = self.engine_handle()?;
            let (hits, text_ranks, vector_ranks) = match (text_query, has_vector) {
                (Some(text_query), true) => {
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
                    )?;
                    let text_ranks = debug::rank_map(&lexical_hits);
                    let vector_ranks = debug::rank_map(&vector_hits);
                    let (mut hits, _report) =
                        crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                    hits.truncate(request.top_k);
                    (hits, text_ranks, vector_ranks)
                }
                (Some(text_query), false) => {
                    let hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        request.top_k,
                    )?;
                    let text_ranks = debug::rank_map(&hits);
                    (hits, text_ranks, BTreeMap::new())
                }
                (None, true) => {
                    let hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        request.top_k,
                        request.distance_metric,
                    )?;
                    let vector_ranks = debug::rank_map(&hits);
                    (hits, BTreeMap::new(), vector_ranks)
                }
                (None, false) => (Vec::new(), BTreeMap::new(), BTreeMap::new()),
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

        match (text_query, has_vector) {
            (Some(text_query), true) => {
                crate::metrics::record_planner_hybrid_query();
                self.hybrid_search(
                    &request.namespace,
                    &request.query_vector,
                    text_query,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )
            }
            (Some(text_query), false) => {
                crate::metrics::record_planner_text_only_query();
                self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )
            }
            (None, true) => {
                crate::metrics::record_planner_vector_only_query();
                self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )
            }
            (None, false) => Ok(Vec::new()),
        }
    }

    fn lexical_search(
        &self,
        namespace: &str,
        query_text: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let started = Instant::now();
        let engine = self.engine_handle()?;
        text_index::ensure_text_index_query_ready(&engine)?;

        if top_k == 0 {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let query_plan = crate::text_index::query_plan(query_text);
        if query_plan.terms.is_empty() {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let Some(namespace_stats) = Self::load_text_namespace_stats(&engine, namespace)? else {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        };
        if namespace_stats.doc_count == 0 {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        }

        let doc_count = namespace_stats.doc_count as f32;
        let avg_doc_len = if namespace_stats.total_doc_len == 0 {
            1.0
        } else {
            namespace_stats.total_doc_len as f32 / doc_count
        };
        let mut scores: BTreeMap<u128, f32> = BTreeMap::new();
        let mut candidate_positions: BTreeMap<u128, BTreeMap<String, Vec<u32>>> = BTreeMap::new();
        let mut doc_stats_cache: BTreeMap<String, crate::text_index::TextDocStats> =
            BTreeMap::new();
        let mut candidates_scored = 0u64;

        for token in query_plan.terms {
            let Some(term_stats) = Self::load_text_term_stats(&engine, namespace, &token)? else {
                continue;
            };
            if term_stats.df == 0 {
                continue;
            }

            let df = term_stats.df as f32;
            let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
            let prefix = crate::text_index::posting_prefix(namespace, &token);
            for entry in engine.scan_partition_prefix_iter(BackendPartition::TextIndex, &prefix)? {
                let (posting_key, posting_value) = entry?;
                if crate::text_index::is_internal_key(&posting_key) {
                    continue;
                }
                let posting = crate::text_index::decode_posting(&posting_value).map_err(|err| {
                    VantaError::SearchError(ChainedError::msg(format!(
                        "text_query found an unreadable posting; run rebuild_index: {err}"
                    )))
                })?;
                let Some(record_key) =
                    crate::text_index::posting_record_key(namespace, &token, &posting_key)
                else {
                    continue;
                };
                let doc_stats = if let Some(stats) = doc_stats_cache.get(&record_key) {
                    stats.clone()
                } else {
                    let Some(stats) = Self::load_text_doc_stats(&engine, namespace, &record_key)?
                    else {
                        return Err(VantaError::NotFound {
                            kind: "document_stats".into(),
                            id: "unknown".into(),
                        });
                    };
                    doc_stats_cache.insert(record_key.clone(), stats.clone());
                    stats
                };
                if doc_stats.node_id != posting.node_id {
                    return Err(VantaError::SearchError(ChainedError::msg(
                        "text_query found posting/doc stats mismatch; run rebuild_index",
                    )));
                }

                let tf = posting.tf as f32;
                let doc_len = doc_stats.doc_len as f32;
                let denominator = tf
                    + crate::text_index::BM25_K1
                        * (1.0 - crate::text_index::BM25_B
                            + crate::text_index::BM25_B * (doc_len / avg_doc_len));
                let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
                scores
                    .entry(posting.node_id)
                    .and_modify(|score| *score += contribution)
                    .or_insert(contribution);
                candidate_positions
                    .entry(posting.node_id)
                    .or_default()
                    .insert(token.clone(), posting.positions);
                candidates_scored += 1;
            }
        }

        let mut hits = Vec::new();
        let node_ids: Vec<u128> = scores.keys().copied().collect();
        let mut node_map: std::collections::HashMap<u128, UnifiedNode> =
            std::collections::HashMap::with_capacity(node_ids.len());
        for n in engine.get_many(&node_ids)? {
            node_map.insert(n.id, n);
        }
        for (node_id, score) in scores {
            let positions_match = candidate_positions
                .get(&node_id)
                .map(|positions| {
                    phrase::text_positions_match_phrases(positions, &query_plan.phrases)
                })
                .unwrap_or(query_plan.phrases.is_empty());
            if !positions_match {
                continue;
            }
            if let Some(node) = node_map.get(&node_id) {
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        hits.push(VantaMemorySearchHit {
                            record,
                            score,
                            explanation: None,
                        });
                    }
                }
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.record.key.cmp(&b.record.key))
                .then(a.record.node_id.cmp(&b.record.node_id))
        });
        hits.truncate(top_k);
        crate::metrics::record_text_lexical_query(
            started.elapsed().as_millis() as u64,
            candidates_scored,
        );
        Ok(hits)
    }

    /// Build a `FilterBitset` from the node IDs of all records matching
    /// the given filters within a namespace.
    ///
    /// When shredded data is available for the filter fields this uses direct
    /// typed comparisons against the column store instead of loading full
    /// records — an order of magnitude faster for selective queries.
    fn bitset_from_filters(
        &self,
        namespace: &str,
        filters: &VantaMemoryMetadata,
    ) -> Result<FilterBitset> {
        // ── Shredded fast path ────────────────────────────────
        // Try resolving every filter field from the shredded column store.
        // If all fields are present in shredded data we skip loading full
        // records entirely, reducing I/O for selective queries.
        if let Ok(engine) = self.engine_handle() {
            let (ids, _has_index) = self.indexed_ids_by_namespace(&engine, namespace)?;
            let mut bitset = FilterBitset::with_capacity(ids.len());
            let mut all_resolved = true;

            for &node_id in &ids {
                let shredded = match crate::shred::ShreddedRowStore::get(node_id, &*engine.backend)
                {
                    Ok(Some(s)) => s,
                    _ => {
                        all_resolved = false;
                        break; // not all nodes have shredded data → fall back
                    }
                };

                let matched = filters.iter().all(|(field, expected)| {
                    shredded
                        .get(field)
                        .is_some_and(|s| crate::shred::matches_shredded(s, &RelOp::Eq, expected))
                });

                if matched {
                    bitset.set_bit(node_id as usize);
                }
            }

            if all_resolved {
                return Ok(bitset);
            }
        }

        // ── Fallback: load full records and use existing filtering ─
        let records = self.records_for_namespace(namespace, filters)?;
        let mut bitset = FilterBitset::with_capacity(records.len());
        for record in &records {
            bitset.set_bit(record.node_id as usize);
        }
        Ok(bitset)
    }

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
        Ok(hits)
    }

    fn vector_memory_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        if query_vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let engine = self.engine_handle()?;

        // ---- Selectivity-based strategy ----
        let strategy = select_filter_strategy(&engine, filters);

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
            // After LSM compaction, nodes may reside on any level (L0..L3).
            // Pass `None` to force HNSW to use inline vec_data for distance
            // computation — this is correct for all levels since `get()` later
            // resolves the packed offset to the right segment.
            index.search(query_vector, &query_mask, budget, None, distance_metric)
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

        Ok(hits)
    }

    fn hybrid_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        text_query: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let started = Instant::now();
        if top_k == 0 {
            crate::metrics::record_hybrid_query(0, 0);
            return Ok(Vec::new());
        }

        let budget = crate::planner::hybrid_candidate_budget(top_k);
        let lexical_hits = self.lexical_search(namespace, text_query, filters, budget)?;
        let vector_hits =
            self.vector_memory_search(namespace, query_vector, filters, budget, distance_metric)?;
        let mut hits = crate::planner::fuse_rrf(lexical_hits, vector_hits);
        let candidates_fused = hits.len() as u64;
        hits.truncate(top_k);
        crate::metrics::record_hybrid_query(started.elapsed().as_millis() as u64, candidates_fused);
        Ok(hits)
    }

    /// Run a read-only structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index(&self, namespace: Option<&str>) -> Result<VantaTextIndexAuditReport> {
        let engine = self.engine_handle()?;
        text_index::run_audit(&engine, namespace)
    }

    /// Run a deep structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index_deep(
        &self,
        namespace: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let engine = self.engine_handle()?;
        text_index::run_audit_deep(&engine, namespace)
    }

    /// Public repair primitive for the text index.
    #[tracing::instrument(skip(self), err)]
    pub fn repair_text_index(&self) -> Result<VantaTextIndexRepairReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "repair_text_index is not available when VantaDB is opened read-only"
                    .into(),
            });
        }
        crate::metrics::record_text_index_repair();
        let report = self.rebuild_text_index_with_report()?;
        Ok(text_index::run_repair(report))
    }

    /// Generate a text snippet with optional highlighting of matched terms.
    #[tracing::instrument(skip(self, payload))]
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        snippet::generate_snippet_with_highlighting(payload, text_query, with_highlighting)
    }

    // highlight_terms, generate_snippet_with_highlighting moved to snippet.rs

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
        ) = match (text_query, has_vector) {
            (Some(text_query), true) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                )?;
                let text_ranks = debug::rank_map(&lexical_hits);
                let vector_ranks = debug::rank_map(&vector_hits);
                let (mut hits, report) =
                    crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                hits.truncate(request.top_k);
                (
                    "hybrid".to_string(),
                    hits,
                    text_ranks,
                    vector_ranks,
                    Some(report),
                )
            }
            (Some(text_query), false) => {
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
            (None, true) => {
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
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
            (None, false) => {
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

        let text_query = crate::planner::trimmed_text_query(&request);
        let has_vector = !request.query_vector.is_empty();
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

        match (text_query, has_vector) {
            (Some(text_query), true) => {
                let budget = crate::planner::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                )?;
                let text_candidates = lexical_hits.len();
                let vector_candidates = vector_hits.len();
                let mut fused_hits = crate::planner::fuse_rrf(lexical_hits, vector_hits);
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
            (Some(text_query), false) => {
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
            (None, true) => {
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
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
            (None, false) => Ok(VantaMemorySearchDebugReport {
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::DistanceMetric;
    use crate::sdk::connect::connect;

    /// Open an in-memory VantaDB for testing.
    fn setup() -> VantaEmbedded {
        connect(":memory:").expect("in-memory db open")
    }

    /// Insert a single record with optional vector and metadata.
    fn insert(
        db: &VantaEmbedded,
        namespace: &str,
        key: &str,
        payload: &str,
        vector: Option<Vec<f32>>,
        metadata: VantaMemoryMetadata,
    ) -> VantaMemoryRecord {
        let input = VantaMemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: payload.into(),
            metadata,
            vector,
            ttl_ms: None,
        };
        db.put(input).expect("put should succeed")
    }

    // ── empty / edge cases ─────────────────────────────────────

    #[test]
    fn test_search_empty_no_text_no_vector() {
        let db = setup();
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            ..Default::default()
        };
        let results = db.search(req).expect("search should succeed");
        assert!(results.is_empty(), "expected empty results");
    }

    #[test]
    fn test_search_top_k_zero() {
        let db = setup();
        // Even with matching data, top_k=0 short-circuits
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );

        // Text-only with top_k=0
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            top_k: 0,
            ..Default::default()
        };
        assert!(db.search(req).unwrap().is_empty());

        // Vector-only with top_k=0
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 0,
            ..Default::default()
        };
        assert!(db.search(req).unwrap().is_empty());

        // Hybrid with top_k=0
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 0,
            ..Default::default()
        };
        assert!(db.search(req).unwrap().is_empty());
    }

    #[test]
    fn test_search_invalid_namespace() {
        let db = setup();
        let req = VantaMemorySearchRequest {
            namespace: "".into(),
            text_query: Some("hello".into()),
            ..Default::default()
        };
        let err = db.search(req).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("namespace"),
            "expected namespace error, got: {msg}"
        );
    }

    // ── text-only lexical search ───────────────────────────────

    #[test]
    fn test_search_text_only_matching() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world welcome",
            None,
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "k2",
            "hello earth",
            None,
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("text search");
        assert!(!results.is_empty(), "expected hits for 'hello'");
        // Both records contain "hello"
        assert_eq!(results.len(), 2, "both records match 'hello'");
        // BM25 scores should be positive
        for hit in &results {
            assert!(
                hit.score > 0.0,
                "expected positive BM25 score, got {}",
                hit.score
            );
        }
    }

    #[test]
    fn test_search_text_only_no_matches() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            None,
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("goodbye".into()),
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("text search");
        assert!(results.is_empty(), "expected no hits for 'goodbye'");
    }

    #[test]
    fn test_search_text_only_with_filters() {
        let db = setup();
        let mut meta_a = VantaMemoryMetadata::new();
        meta_a.insert("lang".into(), VantaValue::String("en".into()));
        insert(&db, "test", "k1", "hello world", None, meta_a);

        let mut meta_b = VantaMemoryMetadata::new();
        meta_b.insert("lang".into(), VantaValue::String("es".into()));
        insert(&db, "test", "k2", "hola mundo", None, meta_b);

        // Search with filter for lang=en
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("lang".into(), VantaValue::String("en".into()));
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            filters,
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("text search with filter");
        assert_eq!(results.len(), 1, "expected one hit matching lang=en");
        assert_eq!(results[0].record.key, "k1");
    }

    #[test]
    fn test_search_text_only_filter_no_match() {
        let db = setup();
        let mut meta = VantaMemoryMetadata::new();
        meta.insert("lang".into(), VantaValue::String("en".into()));
        insert(&db, "test", "k1", "hello world", None, meta);

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("lang".into(), VantaValue::String("de".into()));
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            filters,
            top_k: 10,
            ..Default::default()
        };
        let results = db
            .search(req)
            .expect("text search with non-matching filter");
        assert!(
            results.is_empty(),
            "expected no hits with non-matching filter"
        );
    }

    // ── vector-only HNSW search ────────────────────────────────

    #[test]
    fn test_search_vector_only_hnsw() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "some text",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );

        // Search with exact same vector → cosine similarity = 1.0
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 10,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let results = db.search(req).expect("vector search");
        assert_eq!(results.len(), 1, "expected one hit");
        assert!(
            results[0].score > 0.99,
            "expected near-perfect cosine score, got {}",
            results[0].score
        );
    }

    #[test]
    fn test_search_vector_only_different_ns_no_match() {
        let db = setup();
        insert(
            &db,
            "ns1",
            "k1",
            "some text",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );

        // Search in a different namespace → no matches
        let req = VantaMemorySearchRequest {
            namespace: "other".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("vector search different ns");
        assert!(
            results.is_empty(),
            "expected no hits in different namespace"
        );
    }

    #[test]
    fn test_search_vector_only_with_filters() {
        let db = setup();
        let mut meta_a = VantaMemoryMetadata::new();
        meta_a.insert("type".into(), VantaValue::String("doc".into()));
        insert(
            &db,
            "test",
            "k1",
            "text a",
            Some(vec![0.1, 0.2, 0.3]),
            meta_a,
        );

        let mut meta_b = VantaMemoryMetadata::new();
        meta_b.insert("type".into(), VantaValue::String("image".into()));
        insert(
            &db,
            "test",
            "k2",
            "text b",
            Some(vec![0.1, 0.2, 0.3]),
            meta_b,
        );

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("type".into(), VantaValue::String("doc".into()));
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            filters,
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("vector search with filter");
        assert_eq!(results.len(), 1, "expected one hit matching type=doc");
        assert_eq!(results[0].record.key, "k1");
    }

    #[test]
    fn test_search_vector_only_no_matches() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "text",
            Some(vec![0.9, 0.8, 0.7]),
            VantaMemoryMetadata::new(),
        );

        // Search with a very different vector in an empty namespace
        let req = VantaMemorySearchRequest {
            namespace: "empty_ns".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("vector search no matches");
        assert!(results.is_empty(), "expected no hits in empty namespace");
    }

    // ── hybrid search ──────────────────────────────────────────

    #[test]
    fn test_search_hybrid_both_text_and_vector() {
        let db = setup();
        // Two records, both containing "hello" and having similar vectors
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "k2",
            "hello there",
            Some(vec![0.11, 0.21, 0.31]),
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "k3",
            "goodbye world",
            Some(vec![0.9, 0.8, 0.7]),
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 5,
            ..Default::default()
        };
        let results = db.search(req).expect("hybrid search");
        assert!(!results.is_empty(), "expected hybrid results");
        // k1 and k2 match "hello" AND similar vector; k3 only has similar-ish vector
        assert!(results.len() >= 2, "expected at least 2 hits");
        // Scores should be positive (RRF and BM25 combine)
        for hit in &results {
            assert!(
                hit.score > 0.0,
                "expected positive score, got {}",
                hit.score
            );
        }
        // Top result should be k1 (exact vector match + "hello")
        assert_eq!(results[0].record.key, "k1");
    }

    // ── explain mode ───────────────────────────────────────────

    #[test]
    fn test_search_explain_mode() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 5,
            explain: true,
            ..Default::default()
        };
        let results = db.search(req).expect("explain search");
        assert_eq!(results.len(), 1, "expected one hit");
        let hit = &results[0];
        assert!(
            hit.explanation.is_some(),
            "expected explanation field in explain mode"
        );
        if let Some(explanation) = &hit.explanation {
            assert_eq!(explanation.identity, "test\0k1");
            assert!(!explanation.matched_tokens.is_empty());
        }
    }

    // ── BM25 scoring correctness ───────────────────────────────

    /// BM25 scoring follows the standard formula:
    ///   IDF = ln(1 + (N - df + 0.5) / (df + 0.5))
    ///   score = IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * doc_len / avg_doc_len))
    #[test]
    fn test_search_bm25_scoring_correctness() {
        let db = setup();
        // Insert two records in the same namespace to get N=2
        insert(
            &db,
            "test",
            "k1",
            "hello hello world", // "hello" appears twice in k1
            None,
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "k2",
            "hello foo bar", // "hello" appears once in k2
            None,
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("bm25 search");
        assert_eq!(results.len(), 2, "expected both records");

        // Both hits have positive BM25 scores
        for hit in &results {
            assert!(
                hit.score > 0.0,
                "expected positive BM25 score, got {}",
                hit.score
            );
        }

        // k1 has "hello" twice and "world" once (3 tokens), k2 has "hello" once and "foo","bar" (3 tokens)
        // "hello" appears in both documents → df=2 → IDF contributes equally
        // k1 has tf=2, k2 has tf=1 → k1 should score higher
        assert_eq!(
            results[0].record.key, "k1",
            "k1 has higher tf=2, should rank first"
        );
        assert!(
            results[0].score > results[1].score,
            "k1 (tf=2) should score higher than k2 (tf=1): {} vs {}",
            results[0].score,
            results[1].score
        );
    }

    // ── corrupt text index (debug only) ────────────────────────

    #[cfg(debug_assertions)]
    #[test]
    fn test_search_corrupt_text_index_state() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            None,
            VantaMemoryMetadata::new(),
        );

        // Corrupt the text index state so ensure_text_index_query_ready fails
        db.debug_corrupt_text_index_state_for_tests()
            .expect("corrupt state");

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            top_k: 10,
            ..Default::default()
        };
        let err = db.search(req).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("text_index") || msg.contains("rebuild_index") || msg.contains("search"),
            "expected error from corrupt text index, got: {msg}"
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_search_cleared_text_index_returns_empty() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            None,
            VantaMemoryMetadata::new(),
        );

        // Verify text search works before clearing
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            top_k: 10,
            ..Default::default()
        };
        let before = db.search(req.clone()).expect("search before clear");
        assert!(
            !before.is_empty(),
            "search should work before clearing index"
        );

        // Clear all text index entries (postings, stats)
        db.debug_clear_text_index_for_tests()
            .expect("clear text index");

        // After clearing, lexical search should return empty (no namespace stats)
        let after = db.search(req).expect("search after clear");
        assert!(after.is_empty(), "expected empty after clearing text index");
    }

    // ── empty query_vector (vector path, but empty) ────────────

    #[test]
    fn test_search_empty_query_vector_with_text() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "hello world",
            None,
            VantaMemoryMetadata::new(),
        );

        // text_query + empty query_vector → text-only path
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            text_query: Some("hello".into()),
            query_vector: vec![], // explicitly empty
            top_k: 10,
            ..Default::default()
        };
        let results = db.search(req).expect("text-only with empty query vector");
        assert!(!results.is_empty(), "text-only should still work");
    }

    // ── euclidean distance ─────────────────────────────────────

    #[test]
    fn test_search_vector_only_euclidean() {
        let db = setup();
        insert(
            &db,
            "test",
            "k1",
            "text",
            Some(vec![0.1, 0.2, 0.3]),
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "k2",
            "text",
            Some(vec![0.9, 0.8, 0.7]),
            VantaMemoryMetadata::new(),
        );

        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 5,
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        };
        let results = db.search(req).expect("euclidean search");
        // HNSW internally uses Cosine; Euclidean metric conversion only
        // applies in the brute-force fallback path. At minimum verify that
        // results are returned and ordered correctly.
        assert!(!results.is_empty(), "expected hits for euclidean");
        // k1 vector [0.1,0.2,0.3] is identical to query, k2 is further
        assert_eq!(
            results[0].record.key, "k1",
            "k1 has identical vector to query"
        );
    }

    // ── FilterStrategy ─────────────────────────────────────────

    #[test]
    fn test_select_filter_strategy_empty() {
        let db = setup();
        let engine = db.engine_handle().unwrap();
        let filters = VantaMemoryMetadata::new();
        let strategy = select_filter_strategy(&engine, &filters);
        assert_eq!(
            strategy,
            FilterStrategy::PostFilter,
            "empty filters → PostFilter"
        );
    }

    #[test]
    fn test_select_filter_strategy_highly_selective() {
        let db = setup();
        // Insert two records with different "color" metadata.
        insert(
            &db,
            "test",
            "red_one",
            "text",
            Some(vec![0.1, 0.2]),
            VantaMemoryMetadata::from([("color".into(), VantaValue::String("red".into()))]),
        );
        insert(
            &db,
            "test",
            "blue_one",
            "text",
            Some(vec![0.3, 0.4]),
            VantaMemoryMetadata::from([("color".into(), VantaValue::String("blue".into()))]),
        );

        let engine = db.engine_handle().unwrap();
        let mut filters = VantaMemoryMetadata::new();
        // "red" → 1 of 2 = selectivity 0.5.  That's above PREFILTER_THRESHOLD
        // but below HIGH_SELECTIVITY_THRESHOLD (0.1 < 0.5 < 0.1? no).
        // 0.5 is >= HIGH_SELECTIVITY_THRESHOLD (0.1) → PostFilter.
        // For a more selective test let's query a very rare value.
        // With only 2 records, "red" has freq 1 and total_nodes = 2, so sel = 0.5.
        // That's > 0.1 → PostFilter + 0.01.  Let's use a value that doesn't exist.
        // Non-existent value → selectivity 0.0 → PreFilter.
        filters.insert("nonexistent".into(), VantaValue::String("nope".into()));
        let strategy = select_filter_strategy(&engine, &filters);
        assert_eq!(
            strategy,
            FilterStrategy::PreFilter,
            "non-existent value → sel 0 → PreFilter"
        );
    }

    #[test]
    fn test_select_filter_strategy_moderate() {
        let db = setup();
        // Insert enough records so that a single "color:red" has selectivity
        // in the InFilter range: 1 / N < 0.1 but >= 0.01.
        // N = 20 → sel = 0.05 → InFilter.
        for i in 0..20 {
            let color = if i == 0 { "red" } else { "blue" };
            insert(
                &db,
                "test",
                &format!("k{i}"),
                "text",
                Some(vec![0.1, 0.2]),
                VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
            );
        }

        let engine = db.engine_handle().unwrap();
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("red".into()));
        let strategy = select_filter_strategy(&engine, &filters);
        // "red" has freq 1 / 20 = 0.05 → InFilter
        assert_eq!(
            strategy,
            FilterStrategy::InFilter,
            "1 red out of 20 → sel 0.05 → InFilter"
        );
    }

    #[test]
    fn test_vector_memory_search_with_pre_filter() {
        let db = setup();
        // Insert several records; only one has the target metadata.
        for i in 0..10 {
            let color = if i == 0 { "teal" } else { "gray" };
            insert(
                &db,
                "test",
                &format!("k{i}"),
                "text",
                Some(vec![i as f32 * 0.1, (i + 1) as f32 * 0.1]),
                VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
            );
        }

        let engine = db.engine_handle().unwrap();
        // Force PreFilter by choosing a highly selective value.
        // "teal" → 1 of 10 → sel = 0.1 (= HIGH_SELECTIVITY_THRESHOLD, not < PREFILTER_THRESHOLD 0.01)
        // To get PreFilter, we need sel < 0.01.  With 10 records, use a nonexistent value → sel 0.0.
        let mut filters = VantaMemoryMetadata::new();
        filters.insert(
            "color".into(),
            VantaValue::String("nonexistent_stuff".into()),
        );

        let strategy = select_filter_strategy(&engine, &filters);
        assert_eq!(
            strategy,
            FilterStrategy::PreFilter,
            "nonexistent → PreFilter"
        );

        let hits = db
            .vector_memory_search("test", &[0.1, 0.2], &filters, 5, DistanceMetric::Cosine)
            .expect("pre-filter search");
        assert!(hits.is_empty(), "no records match 'nonexistent_stuff'");
    }

    #[test]
    fn test_vector_memory_search_with_in_filter() {
        let db = setup();
        // 20 records, only "color:red" (1 record) → selectivity 0.05 → InFilter
        for i in 0..20 {
            let color = if i == 0 { "red" } else { "blue" };
            insert(
                &db,
                "test",
                &format!("k{i}"),
                "text",
                Some(vec![i as f32 * 0.1, (i + 1) as f32 * 0.1]),
                VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
            );
        }

        let engine = db.engine_handle().unwrap();
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("red".into()));

        let strategy = select_filter_strategy(&engine, &filters);
        assert_eq!(strategy, FilterStrategy::InFilter, "1/20 → InFilter");

        // Query close to [0.0, 0.1] (k0's vector) so k0 "red" ranks first.
        let hits = db
            .vector_memory_search("test", &[0.0, 0.1], &filters, 5, DistanceMetric::Cosine)
            .expect("in-filter search");
        assert!(!hits.is_empty(), "should find k0 (red)");
        assert_eq!(
            hits[0].record.key, "k0",
            "k0 has vector [0.0, 0.1] closest to query"
        );
        for hit in &hits {
            assert_eq!(
                hit.record.metadata.get("color"),
                Some(&VantaValue::String("red".into())),
                "only red records should appear"
            );
        }
    }

    #[test]
    fn test_bitset_from_filters() {
        let db = setup();
        insert(
            &db,
            "test",
            "a",
            "text",
            None,
            VantaMemoryMetadata::from([("group".into(), VantaValue::String("alpha".into()))]),
        );
        insert(
            &db,
            "test",
            "b",
            "text",
            None,
            VantaMemoryMetadata::from([("group".into(), VantaValue::String("beta".into()))]),
        );
        insert(
            &db,
            "test",
            "c",
            "text",
            None,
            VantaMemoryMetadata::from([("group".into(), VantaValue::String("alpha".into()))]),
        );

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("group".into(), VantaValue::String("alpha".into()));
        let bitset = db
            .bitset_from_filters("test", &filters)
            .expect("bitset from filters");
        assert!(!bitset.is_empty(), "bitset should contain alpha records");
        // "a" and "c" have alpha; verify via records
        let records = db.records_for_namespace("test", &filters).unwrap();
        assert_eq!(records.len(), 2, "two alpha records");
        assert!(
            bitset.has_bit(records[0].node_id as usize),
            "bitset has first alpha node_id"
        );
        assert!(
            bitset.has_bit(records[1].node_id as usize),
            "bitset has second alpha node_id"
        );
    }

    #[test]
    fn test_vector_memory_search_with_metadata_filter() {
        let db = setup();
        // Insert two records with different metadata, same vector namespace.
        insert(
            &db,
            "test",
            "doc1",
            "payload1",
            Some(vec![0.5, 0.5]),
            VantaMemoryMetadata::from([(
                "department".into(),
                VantaValue::String("engineering".into()),
            )]),
        );
        insert(
            &db,
            "test",
            "doc2",
            "payload2",
            Some(vec![0.5, 0.5]),
            VantaMemoryMetadata::from([(
                "department".into(),
                VantaValue::String("marketing".into()),
            )]),
        );

        let query = vec![0.5, 0.5];
        let mut filters = VantaMemoryMetadata::new();
        filters.insert(
            "department".into(),
            VantaValue::String("engineering".into()),
        );

        let hits = db
            .vector_memory_search("test", &query, &filters, 10, DistanceMetric::Cosine)
            .expect("search with metadata filter");
        assert_eq!(hits.len(), 1, "only engineering doc should match");
        assert_eq!(hits[0].record.key, "doc1");
    }

    #[test]
    fn test_vector_memory_search_no_filters() {
        let db = setup();
        insert(
            &db,
            "test",
            "a",
            "text",
            Some(vec![0.1, 0.2]),
            VantaMemoryMetadata::new(),
        );
        insert(
            &db,
            "test",
            "b",
            "text",
            Some(vec![0.9, 0.8]),
            VantaMemoryMetadata::new(),
        );

        // No filters → PostFilter (current behavior).
        let hits = db
            .vector_memory_search(
                "test",
                &[0.1, 0.2],
                &VantaMemoryMetadata::new(),
                5,
                DistanceMetric::Cosine,
            )
            .expect("search without filters");
        assert!(!hits.is_empty(), "should find both records");
        assert_eq!(hits[0].record.key, "a", "closest vector is a");
    }
}
