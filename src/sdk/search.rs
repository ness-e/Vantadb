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
use crate::error::{Result, VantaError};
use crate::index::cosine_sim_f32;
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, BTreeSet};
use tracing;
use web_time::Instant;

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
                    let budget = Self::hybrid_candidate_budget(request.top_k);
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
                    let text_ranks = Self::debug_rank_map(&lexical_hits);
                    let vector_ranks = Self::debug_rank_map(&vector_hits);
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
                    let text_ranks = Self::debug_rank_map(&hits);
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
                    let vector_ranks = Self::debug_rank_map(&hits);
                    (hits, BTreeMap::new(), vector_ranks)
                }
                (None, false) => (Vec::new(), BTreeMap::new(), BTreeMap::new()),
            };

            let explained_hits = hits
                .into_iter()
                .map(|mut hit| {
                    let explanation = Self::debug_explain_hit(
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
        Self::ensure_text_index_query_ready(&engine)?;

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
        let mut scores: BTreeMap<u64, f32> = BTreeMap::new();
        let mut candidate_positions: BTreeMap<u64, BTreeMap<String, Vec<u32>>> = BTreeMap::new();
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
            for (posting_key, posting_value) in
                engine.scan_partition_prefix(BackendPartition::TextIndex, &prefix)?
            {
                if crate::text_index::is_internal_key(&posting_key) {
                    continue;
                }
                let posting = crate::text_index::decode_posting(&posting_value).map_err(|err| {
                    VantaError::SearchError(format!(
                        "text_query found an unreadable posting; run rebuild_index: {err}"
                    ))
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
                    return Err(VantaError::SearchError(
                        "text_query found posting/doc stats mismatch; run rebuild_index"
                            .to_string(),
                    ));
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
        let node_ids: Vec<u64> = scores.keys().copied().collect();
        let node_map: std::collections::HashMap<u64, UnifiedNode> = engine
            .get_many(&node_ids)?
            .into_iter()
            .map(|n| (n.id, n))
            .collect();
        for (node_id, score) in scores {
            let positions_match = candidate_positions
                .get(&node_id)
                .map(|positions| Self::text_positions_match_phrases(positions, &query_plan.phrases))
                .unwrap_or(query_plan.phrases.is_empty());
            if !positions_match {
                continue;
            }
            if let Some(node) = node_map.get(&node_id) {
                if let Some(record) = memory_record_from_node(node.clone()) {
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

        let budget = (top_k.saturating_mul(10)).min(500).max(top_k);
        let candidates = {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            hnsw.search_nearest(query_vector, None, None, &crate::node::ALL_BITSET, budget, Some(&*vs))
        };

        let mut hits = Vec::with_capacity(top_k);
        {
            let candidate_ids: Vec<u64> = candidates.iter().map(|(id, _)| *id).collect();
            let node_map: std::collections::HashMap<u64, UnifiedNode> = engine
                .get_many(&candidate_ids)?
                .into_iter()
                .map(|n| (n.id, n))
                .collect();
            for (node_id, raw_score) in candidates {
                if hits.len() >= top_k {
                    break;
                }
                if let Some(node) = node_map.get(&node_id) {
                    if let Some(record) = memory_record_from_node(node.clone()) {
                        if record.namespace == namespace && matches_memory_filters(&record, filters)
                        {
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

        if hits.is_empty() && !query_vector.is_empty() {
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
            Self::sort_memory_hits(&mut hits);
            hits.truncate(top_k);
            if distance_metric == crate::node::DistanceMetric::Euclidean {
                for hit in hits.iter_mut() {
                    hit.score = -(-hit.score).max(0.0).sqrt();
                }
            }
        }

        Ok(hits)
    }

    fn sort_memory_hits(hits: &mut [VantaMemorySearchHit]) {
        crate::planner::sort_hits(hits);
    }

    fn hybrid_candidate_budget(top_k: usize) -> usize {
        crate::planner::hybrid_candidate_budget(top_k)
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

        let budget = Self::hybrid_candidate_budget(top_k);
        let lexical_hits = self.lexical_search(namespace, text_query, filters, budget)?;
        let vector_hits =
            self.vector_memory_search(namespace, query_vector, filters, budget, distance_metric)?;
        let mut hits = Self::fuse_rrf(lexical_hits, vector_hits);
        let candidates_fused = hits.len() as u64;
        hits.truncate(top_k);
        crate::metrics::record_hybrid_query(started.elapsed().as_millis() as u64, candidates_fused);
        Ok(hits)
    }

    fn fuse_rrf(
        lexical_hits: Vec<VantaMemorySearchHit>,
        vector_hits: Vec<VantaMemorySearchHit>,
    ) -> Vec<VantaMemorySearchHit> {
        crate::planner::fuse_rrf(lexical_hits, vector_hits)
    }

    fn ensure_text_index_query_ready(engine: &StorageEngine) -> Result<TextIndexState> {
        let state = Self::load_text_index_state(engine).map_err(|_| VantaError::NotFound {
            kind: "text_index_state".into(),
            id: "bm25".into(),
        })?;
        let Some(state) = state else {
            return Err(VantaError::NotFound {
                kind: "text_index".into(),
                id: "bm25".into(),
            });
        };
        if !Self::text_index_state_matches_spec(&state) {
            return Err(VantaError::ValidationError {
                field: "text_index_schema".into(),
                reason:
                    "text_query requires text_index schema v3; reopen writable or run rebuild_index"
                        .into(),
            });
        }
        Ok(state)
    }

    fn text_positions_match_phrases(
        term_positions: &BTreeMap<String, Vec<u32>>,
        phrases: &[Vec<String>],
    ) -> bool {
        phrases
            .iter()
            .all(|phrase| Self::text_positions_match_phrase(term_positions, phrase))
    }

    fn text_positions_match_phrase(
        term_positions: &BTreeMap<String, Vec<u32>>,
        phrase: &[String],
    ) -> bool {
        let Some(first_token) = phrase.first() else {
            return true;
        };
        let Some(first_positions) = term_positions.get(first_token) else {
            return false;
        };
        if phrase.len() == 1 {
            return !first_positions.is_empty();
        }

        first_positions.iter().any(|start| {
            phrase.iter().enumerate().skip(1).all(|(offset, token)| {
                let Some(positions) = term_positions.get(token) else {
                    return false;
                };
                positions.contains(&start.saturating_add(offset as u32))
            })
        })
    }

    /// Run a read-only structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index(&self, namespace: Option<&str>) -> Result<VantaTextIndexAuditReport> {
        if let Some(namespace) = namespace {
            validate_namespace(namespace)?;
        }
        let engine = self.engine_handle()?;
        Self::build_text_index_audit_report_shallow(&engine, namespace)
    }

    /// Run a deep structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index_deep(
        &self,
        namespace: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        if let Some(namespace) = namespace {
            validate_namespace(namespace)?;
        }
        let engine = self.engine_handle()?;
        Self::build_text_index_audit_report_deep(&engine, namespace)
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
        Ok(VantaTextIndexRepairReport {
            record_count: report.record_count,
            posting_entries: report.posting_entries,
            doc_stats_entries: report.doc_stats_entries,
            term_stats_entries: report.term_stats_entries,
            namespace_stats_entries: report.namespace_stats_entries,
            duration_ms: report.duration_ms,
            success: true,
        })
    }

    /// Generate a text snippet with optional highlighting of matched terms.
    #[tracing::instrument(skip(self, payload))]
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        Self::generate_snippet_with_highlighting(payload, text_query, with_highlighting)
    }

    fn generate_snippet_with_highlighting(
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        let query_plan = crate::text_index::query_plan(text_query);
        let first_token = query_plan.terms.iter().next()?;

        if payload.len() <= 120 {
            if with_highlighting {
                return Some(Self::highlight_terms(payload, &query_plan.terms));
            }
            return Some(payload.to_string());
        }

        let lower_payload = payload.to_ascii_lowercase();
        let match_at = lower_payload.find(first_token).unwrap_or(0);
        let mut start = match_at.saturating_sub(48);
        let mut end = match_at
            .saturating_add(first_token.len())
            .saturating_add(72)
            .min(payload.len());
        while start > 0 && !payload.is_char_boundary(start) {
            start -= 1;
        }
        while end < payload.len() && !payload.is_char_boundary(end) {
            end += 1;
        }

        let snippet_text = payload[start..end].trim();

        if with_highlighting {
            let highlighted = Self::highlight_terms(snippet_text, &query_plan.terms);
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&highlighted);
            if end < payload.len() {
                snippet.push_str("...");
            }
            Some(snippet)
        } else {
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(snippet_text);
            if end < payload.len() {
                snippet.push_str("...");
            }
            Some(snippet)
        }
    }

    fn highlight_terms(text: &str, terms: &BTreeSet<String>) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            let mut matched = false;

            for term in terms {
                let term_chars: Vec<char> = term.chars().collect();
                if i + term_chars.len() <= chars.len() {
                    let slice: String = chars[i..i + term_chars.len()].iter().collect();
                    if slice.eq_ignore_ascii_case(term) {
                        result.push_str("<strong>");
                        result.push_str(&slice);
                        result.push_str("</strong>");
                        i += term_chars.len();
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

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
                let budget = Self::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                )?;
                let text_ranks = Self::debug_rank_map(&lexical_hits);
                let vector_ranks = Self::debug_rank_map(&vector_hits);
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
                let text_ranks = Self::debug_rank_map(&hits);
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
                let vector_ranks = Self::debug_rank_map(&hits);
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
            .map(|hit| {
                Self::debug_explain_hit(&engine, hit, text_query, &text_ranks, &vector_ranks)
            })
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
    ) -> Result<Option<(u64, u32)>> {
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
                let budget = Self::hybrid_candidate_budget(request.top_k);
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
                let mut fused_hits = Self::fuse_rrf(lexical_hits, vector_hits);
                let fused_candidates = fused_hits.len();
                fused_hits.truncate(request.top_k);
                Ok(VantaMemorySearchDebugReport {
                    route: "hybrid".to_string(),
                    budget,
                    text_candidates,
                    vector_candidates,
                    fused_candidates,
                    top_identities: Self::debug_hit_identities(&fused_hits),
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
                    top_identities: Self::debug_hit_identities(&hits),
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
                    top_identities: Self::debug_hit_identities(&hits),
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

    #[cfg(debug_assertions)]
    fn debug_hit_identities(hits: &[VantaMemorySearchHit]) -> Vec<String> {
        hits.iter()
            .map(|hit| format!("{}\0{}", hit.record.namespace, hit.record.key))
            .collect()
    }

    fn debug_rank_map(hits: &[VantaMemorySearchHit]) -> BTreeMap<(String, String), usize> {
        hits.iter()
            .enumerate()
            .map(|(index, hit)| {
                (
                    (hit.record.namespace.clone(), hit.record.key.clone()),
                    index + 1,
                )
            })
            .collect()
    }

    fn debug_explain_hit(
        engine: &StorageEngine,
        hit: VantaMemorySearchHit,
        text_query: Option<&str>,
        text_ranks: &BTreeMap<(String, String), usize>,
        vector_ranks: &BTreeMap<(String, String), usize>,
    ) -> Result<VantaSearchExplanationHit> {
        let identity_tuple = (hit.record.namespace.clone(), hit.record.key.clone());
        let identity = format!("{}\0{}", hit.record.namespace, hit.record.key);
        let bm25_terms = if let Some(text_query) = text_query {
            Self::debug_bm25_terms_for_record(engine, &hit.record, text_query)?
        } else {
            Vec::new()
        };
        let matched_tokens = bm25_terms
            .iter()
            .map(|term| term.token.clone())
            .collect::<Vec<_>>();
        let matched_phrases = if let Some(text_query) = text_query {
            Self::debug_matched_phrases_for_record(engine, &hit.record, text_query)?
        } else {
            Vec::new()
        };
        let snippet = text_query.and_then(|query| Self::debug_snippet(&hit.record.payload, query));

        Ok(VantaSearchExplanationHit {
            identity,
            score: hit.score,
            snippet,
            matched_tokens,
            matched_phrases,
            bm25_terms,
            rrf_text_rank: text_ranks.get(&identity_tuple).copied(),
            rrf_vector_rank: vector_ranks.get(&identity_tuple).copied(),
        })
    }

    fn debug_bm25_terms_for_record(
        engine: &StorageEngine,
        record: &VantaMemoryRecord,
        text_query: &str,
    ) -> Result<Vec<VantaBm25TermContribution>> {
        let query_plan = crate::text_index::query_plan(text_query);
        if query_plan.terms.is_empty() {
            return Ok(Vec::new());
        }
        let Some(namespace_stats) = Self::load_text_namespace_stats(engine, &record.namespace)?
        else {
            return Ok(Vec::new());
        };
        let Some(doc_stats) = Self::load_text_doc_stats(engine, &record.namespace, &record.key)?
        else {
            return Ok(Vec::new());
        };
        if namespace_stats.doc_count == 0 {
            return Ok(Vec::new());
        }

        let doc_count = namespace_stats.doc_count as f32;
        let avg_doc_len = if namespace_stats.total_doc_len == 0 {
            1.0
        } else {
            namespace_stats.total_doc_len as f32 / doc_count
        };
        let doc_len = doc_stats.doc_len as f32;
        let mut terms = Vec::new();

        for token in query_plan.terms {
            let Some(term_stats) = Self::load_text_term_stats(engine, &record.namespace, &token)?
            else {
                continue;
            };
            let Some(posting_value) = engine.get_from_partition(
                BackendPartition::TextIndex,
                &crate::text_index::posting_key(&record.namespace, &token, &record.key),
            )?
            else {
                continue;
            };
            let posting = crate::text_index::decode_posting(&posting_value)?;
            let df = term_stats.df as f32;
            let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
            let tf = posting.tf as f32;
            let denominator = tf
                + crate::text_index::BM25_K1
                    * (1.0 - crate::text_index::BM25_B
                        + crate::text_index::BM25_B * (doc_len / avg_doc_len));
            let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
            terms.push(VantaBm25TermContribution {
                token,
                tf: posting.tf,
                df: term_stats.df,
                doc_len: doc_stats.doc_len,
                contribution,
            });
        }

        Ok(terms)
    }

    fn debug_matched_phrases_for_record(
        engine: &StorageEngine,
        record: &VantaMemoryRecord,
        text_query: &str,
    ) -> Result<Vec<String>> {
        let query_plan = crate::text_index::query_plan(text_query);
        if query_plan.phrases.is_empty() {
            return Ok(Vec::new());
        }

        let mut term_positions = BTreeMap::new();
        for token in query_plan.terms {
            if let Some(value) = engine.get_from_partition(
                BackendPartition::TextIndex,
                &crate::text_index::posting_key(&record.namespace, &token, &record.key),
            )? {
                let posting = crate::text_index::decode_posting(&value)?;
                term_positions.insert(token, posting.positions);
            }
        }

        Ok(query_plan
            .phrases
            .into_iter()
            .filter(|phrase| Self::text_positions_match_phrase(&term_positions, phrase))
            .map(|phrase| phrase.join(" "))
            .collect())
    }

    fn debug_snippet(payload: &str, text_query: &str) -> Option<String> {
        Self::generate_snippet_with_highlighting(payload, text_query, false)
    }
}
