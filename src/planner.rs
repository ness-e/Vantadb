//! Search planner for VantaDB hybrid retrieval.
//!
//! This module owns the routing logic, RRF fusion constants, and candidate
//! budget derivation that drive `VantaEmbedded::search`. Extracting these
//! here keeps `sdk.rs` focused on orchestration while making the planner
//! independently testable.
//!
//! # Route classification
//!
//! Given a `(text_query, has_vector)` pair the planner selects one of:
//! - `hybrid`      — text + vector; candidates fused with Reciprocal Rank Fusion
//! - `text-only`   — BM25 lexical search only
//! - `vector-only` — HNSW approximate nearest neighbour only
//! - `empty`       — neither input provided; returns zero results

use std::collections::BTreeMap;

use crate::sdk::{VantaHybridFusionReport, VantaMemorySearchHit, VantaMemorySearchRequest};

// ── RRF constants ─────────────────────────────────────────────────────────

/// Reciprocal Rank Fusion smoothing constant (standard literature value: 60).
pub const RRF_K: f32 = 60.0;

/// Multiplier applied to `top_k` to derive the per-arm candidate budget.
pub const CANDIDATE_MULTIPLIER: usize = 4;

/// Minimum candidates fetched per arm in hybrid mode.
pub const MIN_CANDIDATE_BUDGET: usize = 32;

/// Maximum candidates fetched per arm in hybrid mode (guards against
/// unbounded lexical scan at large `top_k`).
pub const MAX_CANDIDATE_BUDGET: usize = 256;

// ── Route enum ────────────────────────────────────────────────────────────

/// The retrieval strategy selected by the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchRoute {
    /// BM25 and vector rankings executed independently, fused with RRF.
    Hybrid,
    /// BM25 lexical retrieval only.
    TextOnly,
    /// HNSW approximate nearest-neighbour only.
    VectorOnly,
    /// No usable input; zero results will be returned immediately.
    Empty,
}

impl SearchRoute {
    /// Human-readable label used in debug reports.
    pub fn label(self) -> &'static str {
        match self {
            SearchRoute::Hybrid => "hybrid",
            SearchRoute::TextOnly => "text-only",
            SearchRoute::VectorOnly => "vector-only",
            SearchRoute::Empty => "empty",
        }
    }
}

// ── Routing ───────────────────────────────────────────────────────────────

/// Classify the retrieval strategy for a search request.
///
/// The `text_query` parameter should be the pre-trimmed, non-empty query
/// string (or `None`).
pub fn classify(text_query: Option<&str>, has_vector: bool) -> SearchRoute {
    let route = match (text_query, has_vector) {
        (Some(_), true) => SearchRoute::Hybrid,
        (Some(_), false) => SearchRoute::TextOnly,
        (None, true) => SearchRoute::VectorOnly,
        (None, false) => SearchRoute::Empty,
    };
    tracing::debug!("Classified search route: {:?}", route);
    route
}

/// Derive the per-arm candidate budget for hybrid retrieval.
///
/// The budget is clamped to `[MIN_CANDIDATE_BUDGET, MAX_CANDIDATE_BUDGET]`
/// and never falls below `top_k` so that `fuse_rrf` always has enough
/// candidates to fill the requested result set.
pub fn hybrid_candidate_budget(top_k: usize) -> usize {
    top_k
        .saturating_mul(CANDIDATE_MULTIPLIER)
        .clamp(MIN_CANDIDATE_BUDGET, MAX_CANDIDATE_BUDGET)
        .max(top_k)
}

// ── Normalised request fields ─────────────────────────────────────────────

/// Extract the trimmed, non-empty text query from a search request.
pub fn trimmed_text_query(request: &VantaMemorySearchRequest) -> Option<&str> {
    request
        .text_query
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

// ── RRF fusion ────────────────────────────────────────────────────────────

/// Fuse lexical and vector hit lists using Reciprocal Rank Fusion.
///
/// Each ranked hit contributes `1 / (RRF_K + rank + 1)` to its score in
/// the merged result. Hits appearing in both lists receive contributions
/// from both rankings. The returned list is sorted descending by score,
/// with ties broken by `key` then `node_id` for determinism.
pub fn fuse_rrf(
    lexical_hits: Vec<VantaMemorySearchHit>,
    vector_hits: Vec<VantaMemorySearchHit>,
) -> Vec<VantaMemorySearchHit> {
    tracing::debug!(
        "Fusing lexical candidates ({}) and vector candidates ({}) with RRF_K = {}",
        lexical_hits.len(),
        vector_hits.len(),
        RRF_K
    );
    let mut fused: BTreeMap<(String, String), VantaMemorySearchHit> = BTreeMap::new();
    apply_rrf_contributions(&mut fused, lexical_hits);
    apply_rrf_contributions(&mut fused, vector_hits);

    let mut hits: Vec<_> = fused.into_values().collect();
    sort_hits(&mut hits);
    tracing::debug!("Fused candidates count: {}", hits.len());
    hits
}

/// Fuse lexical and vector hit lists and produce a fusion report.
pub fn fuse_rrf_with_report(
    lexical_hits: Vec<VantaMemorySearchHit>,
    vector_hits: Vec<VantaMemorySearchHit>,
) -> (Vec<VantaMemorySearchHit>, VantaHybridFusionReport) {
    let text_candidates = lexical_hits.len();
    let vector_candidates = vector_hits.len();
    let fused_hits = fuse_rrf(lexical_hits, vector_hits);
    let report = VantaHybridFusionReport {
        text_candidates,
        vector_candidates,
        fused_candidates: fused_hits.len(),
        rrf_k: RRF_K as usize,
    };
    (fused_hits, report)
}

fn apply_rrf_contributions(
    fused: &mut BTreeMap<(String, String), VantaMemorySearchHit>,
    hits: Vec<VantaMemorySearchHit>,
) {
    for (rank, hit) in hits.into_iter().enumerate() {
        let contribution = 1.0 / (RRF_K + rank as f32 + 1.0);
        let identity = (hit.record.namespace.clone(), hit.record.key.clone());
        fused
            .entry(identity)
            .and_modify(|existing| existing.score += contribution)
            .or_insert_with(|| VantaMemorySearchHit {
                record: hit.record,
                score: contribution,
                explanation: None,
            });
    }
}

// ── Sorting ───────────────────────────────────────────────────────────────

/// Sort hits descending by score; ties broken by `key` then `node_id`.
pub fn sort_hits(hits: &mut [VantaMemorySearchHit]) {
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.record.key.cmp(&b.record.key))
            .then(a.record.node_id.cmp(&b.record.node_id))
    });
}

// ── Cost-Based Optimizer (CBO) & Volcano Compiler ─────────────────────────

/// Optimise a logical plan and compile it into a physical operator.
pub fn optimize_and_compile<'a>(
    plan: &crate::query::LogicalPlan,
    storage: &'a crate::storage::StorageEngine,
) -> crate::error::Result<Box<dyn crate::query::PhysicalOperator + 'a>> {
    let mut entity = "*".to_string();
    for op in &plan.operators {
        if let crate::query::LogicalOperator::Scan { entity: ent } = op {
            entity = ent.clone();
        }
    }

    let mut relational_filters = Vec::new();
    let mut vector_search = None;
    let mut limit = None;
    let mut project = None;
    let mut sort = None;

    for op in &plan.operators {
        match op {
            crate::query::LogicalOperator::FilterRelational {
                field,
                op: rel_op,
                value,
            } => {
                relational_filters.push((field.clone(), rel_op.clone(), value.clone()));
            }
            crate::query::LogicalOperator::VectorSearch {
                field,
                query_vec,
                min_score,
            } => {
                vector_search = Some((field.clone(), query_vec.clone(), *min_score));
            }
            crate::query::LogicalOperator::Limit { top_k } => {
                limit = Some(*top_k);
            }
            crate::query::LogicalOperator::Project { fields } => {
                project = Some(fields.clone());
            }
            crate::query::LogicalOperator::Sort { field, desc } => {
                sort = Some((field.clone(), *desc));
            }
            _ => {}
        }
    }

    let mut joint_selectivity = 1.0f32;
    for (field, rel_op, value) in &relational_filters {
        let sel = storage.get_estimated_selectivity(field, rel_op, value);
        joint_selectivity *= sel;
    }

    let mut current_operator: Box<dyn crate::query::PhysicalOperator + 'a> =
        if let Some((_field, query_text, min_score)) = vector_search {
            if joint_selectivity < 0.1 && !relational_filters.is_empty() {
                let mut scan_op: Box<dyn crate::query::PhysicalOperator + 'a> =
                    Box::new(crate::physical_plan::PhysicalScan::new(storage, entity));
                for (field, rel_op, value) in relational_filters {
                    scan_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                        scan_op, field, rel_op, value,
                    ));
                }
                Box::new(crate::physical_plan::PhysicalVectorRefine::new(
                    scan_op, query_text, min_score,
                ))
            } else {
                let mut vs_op: Box<dyn crate::query::PhysicalOperator + 'a> = Box::new(
                    crate::physical_plan::PhysicalVectorSearch::new(storage, query_text, min_score),
                );
                for (field, rel_op, value) in relational_filters {
                    vs_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                        vs_op, field, rel_op, value,
                    ));
                }
                vs_op
            }
        } else {
            let mut scan_op: Box<dyn crate::query::PhysicalOperator + 'a> =
                Box::new(crate::physical_plan::PhysicalScan::new(storage, entity));
            for (field, rel_op, value) in relational_filters {
                scan_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                    scan_op, field, rel_op, value,
                ));
            }
            scan_op
        };

    if let Some((field, desc)) = sort {
        current_operator = Box::new(crate::physical_plan::PhysicalSort::new(
            current_operator,
            field,
            desc,
        ));
    }

    if let Some(fields) = project {
        current_operator = Box::new(crate::physical_plan::PhysicalProject::new(
            current_operator,
            fields,
        ));
    }

    if let Some(lim) = limit {
        current_operator = Box::new(crate::physical_plan::PhysicalLimit::new(
            current_operator,
            lim,
        ));
    }

    Ok(current_operator)
}

// ── Unit tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Route classification ──────────────────────────────────────────────

    #[test]
    fn classify_hybrid_when_both_inputs_present() {
        assert_eq!(classify(Some("query"), true), SearchRoute::Hybrid);
    }

    #[test]
    fn classify_text_only_when_no_vector() {
        assert_eq!(classify(Some("query"), false), SearchRoute::TextOnly);
    }

    #[test]
    fn classify_vector_only_when_no_text() {
        assert_eq!(classify(None, true), SearchRoute::VectorOnly);
    }

    #[test]
    fn classify_empty_when_no_inputs() {
        assert_eq!(classify(None, false), SearchRoute::Empty);
    }

    // ── Candidate budget ─────────────────────────────────────────────────

    #[test]
    fn budget_is_clamped_at_min() {
        assert_eq!(hybrid_candidate_budget(1), MIN_CANDIDATE_BUDGET);
    }

    #[test]
    fn budget_is_clamped_at_max_for_mid_range_top_k() {
        // top_k=64 → 64*4=256 = MAX_CANDIDATE_BUDGET; max(256, 64)=256
        let budget = hybrid_candidate_budget(64);
        assert_eq!(budget, MAX_CANDIDATE_BUDGET);
    }

    #[test]
    fn budget_returns_top_k_when_top_k_exceeds_max() {
        // top_k=10_000 → 10_000*4 clamped to 256; but max(256, 10_000)=10_000
        // The guardrail ensures we always fetch at least top_k candidates.
        let budget = hybrid_candidate_budget(10_000);
        assert!(budget >= 10_000);
    }

    #[test]
    fn budget_is_at_least_top_k() {
        // top_k=50 → 50*4=200 which is within [32,256]
        let budget = hybrid_candidate_budget(50);
        assert!(budget >= 50);
        assert_eq!(budget, 200);
    }

    #[test]
    fn budget_never_below_top_k_for_large_top_k() {
        // top_k=200 → 200*4=800 clamped to 256; but max(256, 200)=256 ≥ top_k
        let budget = hybrid_candidate_budget(200);
        assert!(budget >= 200);
    }

    // ── RRF fusion ───────────────────────────────────────────────────────

    fn make_hit(ns: &str, key: &str, score: f32, node_id: u64) -> VantaMemorySearchHit {
        use crate::sdk::{VantaMemoryMetadata, VantaMemoryRecord};
        VantaMemorySearchHit {
            record: VantaMemoryRecord {
                namespace: ns.to_string(),
                key: key.to_string(),
                payload: String::new(),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
                expires_at_ms: Some(0),
                version: 0,
                node_id,
                vector: None,
            },
            score,
            explanation: None,
        }
    }

    #[test]
    fn fuse_rrf_returns_deterministic_order() {
        let lex = vec![make_hit("ns", "a", 0.9, 1), make_hit("ns", "b", 0.8, 2)];
        let vec = vec![make_hit("ns", "b", 0.95, 2), make_hit("ns", "c", 0.7, 3)];
        let result = fuse_rrf(lex, vec);
        // "b" appears in both lists → highest combined RRF score
        assert_eq!(result[0].record.key, "b");
    }

    #[test]
    fn fuse_rrf_scores_are_positive() {
        let lex = vec![make_hit("ns", "x", 0.5, 10)];
        let vec = vec![make_hit("ns", "x", 0.5, 10)];
        let result = fuse_rrf(lex, vec);
        assert_eq!(result.len(), 1);
        assert!(result[0].score > 0.0);
    }

    #[test]
    fn fuse_rrf_deduplicates_same_namespace_key() {
        let lex = vec![make_hit("ns", "dup", 0.9, 99)];
        let vec = vec![make_hit("ns", "dup", 0.9, 99)];
        let result = fuse_rrf(lex, vec);
        assert_eq!(result.len(), 1, "same (namespace, key) must be merged");
    }

    #[test]
    fn sort_hits_is_deterministic_on_equal_scores() {
        let mut hits = vec![make_hit("ns", "z", 0.5, 20), make_hit("ns", "a", 0.5, 10)];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "a", "ties broken alphabetically by key");
    }

    // ── Route labels ─────────────────────────────────────────────────────

    #[test]
    fn route_labels_match_debug_report_strings() {
        assert_eq!(SearchRoute::Hybrid.label(), "hybrid");
        assert_eq!(SearchRoute::TextOnly.label(), "text-only");
        assert_eq!(SearchRoute::VectorOnly.label(), "vector-only");
        assert_eq!(SearchRoute::Empty.label(), "empty");
    }
}
