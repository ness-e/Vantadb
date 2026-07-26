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

use crate::node::FieldValue;
use crate::query::RelOp;
use crate::sdk::{VantaHybridFusionReport, VantaMemorySearchHit, VantaMemorySearchRequest};

// ── RRF constants ─────────────────────────────────────────────────────────

/// Reciprocal Rank Fusion smoothing constant (standard literature value: 60).
pub const RRF_K: f32 = 60.0;

/// Selectivity threshold below which filters are considered highly selective.
///
/// When joint selectivity falls below this value, the optimizer prefers
/// applying filters before vector search (scan→filter→refine) over the
/// default vector-search→filter order. A filter with selectivity 0.1 means
/// it prunes ~90 % of rows.
pub const HIGH_SELECTIVITY_THRESHOLD: f32 = 0.1;

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
///
/// Handles two plan shapes:
/// 1. **Traditional** (FROM/MATCH): Scan → filters → vector → sort → project → limit
/// 2. **SELECT/JOIN**: Join(recursive) | Scan → post-join filters → subquery → project
pub fn optimize_and_compile<'a>(
    plan: &crate::query::LogicalPlan,
    storage: &'a crate::storage::StorageEngine,
) -> crate::error::Result<Box<dyn crate::query::PhysicalOperator + 'a>> {
    // ---- First pass: collect metadata and detect plan shape ----
    let mut entity = "*".to_string();
    let mut relational_filters = Vec::new();
    let mut vector_search = None;
    let mut limit = None;
    let mut project = None;
    let mut sort = None;

    // JOIN and SubqueryFilter produce their own sub-plans that wrap the chain
    let mut has_join = false;
    let mut join_spec: Option<(
        crate::query::LogicalPlan,
        crate::query::LogicalPlan,
        String,
        String,
    )> = None;
    let mut subquery_filters: Vec<(String, RelOp, crate::query::LogicalPlan)> = Vec::new();

    for op in &plan.operators {
        match op {
            crate::query::LogicalOperator::Scan { entity: ent } => {
                entity = ent.clone();
            }
            crate::query::LogicalOperator::Join {
                left_plan,
                right_plan,
                left_field,
                right_field,
            } => {
                has_join = true;
                join_spec = Some((
                    *left_plan.clone(),
                    *right_plan.clone(),
                    left_field.clone(),
                    right_field.clone(),
                ));
            }
            crate::query::LogicalOperator::SubqueryFilter {
                field,
                op,
                subquery_plan,
            } => {
                subquery_filters.push((
                    field.clone(),
                    op.clone(),
                    *subquery_plan.clone(),
                ));
            }
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
            _ => {} // Traverse and other operators are handled by the executor cycle
        }
    }

    // ---- CBO filter reordering and elimination ----
    let mut with_sel: Vec<(f32, String, RelOp, FieldValue)> = relational_filters
        .drain(..)
        .map(|(f, op, v)| {
            let sel = storage.get_estimated_selectivity(&f, &op, &v);
            (sel, f, op, v)
        })
        .collect();
    with_sel.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut joint_selectivity = 1.0f32;
    let mut sorted_filters = Vec::with_capacity(with_sel.len());
    for (sel, field, rel_op, value) in with_sel {
        if sel >= 1.0 {
            tracing::debug!("CBO: eliminated identity filter on {}", field);
            continue;
        }
        joint_selectivity *= sel;
        sorted_filters.push((field, rel_op, value));
    }

    // ---- Build the physical operator chain ----
    // ponytail: predicate pushdown across joins not yet implemented.
    // All WHERE filters apply post-join. Push filters into join children
    // when alias is resolvable for better performance.

    // Determine the base operator (scan or join) and apply sorted_filters
    let mut current_operator: Box<dyn crate::query::PhysicalOperator + 'a> =
        if has_join {
            let (left_plan, right_plan, left_field, right_field) = join_spec.unwrap();
            let left_op = optimize_and_compile(&left_plan, storage)?;
            let right_op = optimize_and_compile(&right_plan, storage)?;
            let mut join_op: Box<dyn crate::query::PhysicalOperator + 'a> =
                Box::new(crate::physical_plan::PhysicalNestedLoopJoin::new(
                    left_op,
                    right_op,
                    left_field,
                    right_field,
                ));
            // Apply post-join relational filters
            for (field, rel_op, value) in sorted_filters {
                join_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                    join_op, field, rel_op, value,
                ));
            }
            join_op
        } else if let Some((_field, query_text, min_score)) = vector_search {
            // CBO: filter-before-vector vs vector-before-filter
            if joint_selectivity < HIGH_SELECTIVITY_THRESHOLD && !sorted_filters.is_empty() {
                let mut scan_op: Box<dyn crate::query::PhysicalOperator + 'a> =
                    Box::new(crate::physical_plan::PhysicalScan::new(storage, entity));
                for (field, rel_op, value) in sorted_filters {
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
                for (field, rel_op, value) in sorted_filters {
                    vs_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                        vs_op, field, rel_op, value,
                    ));
                }
                vs_op
            }
        } else {
            let mut scan_op: Box<dyn crate::query::PhysicalOperator + 'a> =
                Box::new(crate::physical_plan::PhysicalScan::new(storage, entity));
            for (field, rel_op, value) in sorted_filters {
                scan_op = Box::new(crate::physical_plan::PhysicalFilter::new(
                    scan_op, field, rel_op, value,
                ));
            }
            scan_op
        };

    // Apply SubqueryFilter operators on top of the chain
    for (field, op, subq_plan) in subquery_filters {
        let subq_op = optimize_and_compile(&subq_plan, storage)?;
        current_operator = Box::new(
            crate::physical_plan::PhysicalSubqueryFilter::new(
                current_operator,
                subq_op,
                field,
                op,
            ),
        );
    }

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

    fn make_hit(ns: &str, key: &str, score: f32, node_id: u128) -> VantaMemorySearchHit {
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

    // ── trimmed_text_query ───────────────────────────────────────────────

    #[test]
    fn trimmed_text_query_none() {
        let req = VantaMemorySearchRequest {
            text_query: None,
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_empty() {
        let req = VantaMemorySearchRequest {
            text_query: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_whitespace() {
        let req = VantaMemorySearchRequest {
            text_query: Some("   ".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_valid() {
        let req = VantaMemorySearchRequest {
            text_query: Some("hello world".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), Some("hello world"));
    }

    #[test]
    fn trimmed_text_query_trims_input() {
        let req = VantaMemorySearchRequest {
            text_query: Some("  query  ".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), Some("query"));
    }

    // ── fuse_rrf_with_report ────────────────────────────────────────────

    #[test]
    fn fuse_rrf_with_report_counts() {
        let lex = vec![make_hit("ns", "a", 0.9, 1)];
        let vec = vec![make_hit("ns", "b", 0.8, 2)];
        let (_hits, report) = fuse_rrf_with_report(lex.clone(), vec.clone());
        assert_eq!(report.text_candidates, 1);
        assert_eq!(report.vector_candidates, 1);
        assert_eq!(report.rrf_k, RRF_K as usize);
    }

    #[test]
    fn fuse_rrf_with_report_fused_results() {
        let lex = vec![make_hit("ns", "a", 0.9, 1)];
        let vec = vec![make_hit("ns", "a", 0.8, 1)];
        let (hits, _report) = fuse_rrf_with_report(lex, vec);
        assert_eq!(hits.len(), 1, "same key merged into one");
        let expected = 2.0 / (RRF_K + 1.0);
        assert!((hits[0].score - expected).abs() < 1e-6);
    }

    // ── sort_hits edge cases ────────────────────────────────────────────

    #[test]
    fn sort_hits_descending_order() {
        let mut hits = vec![
            make_hit("ns", "a", 0.3, 1),
            make_hit("ns", "b", 0.9, 2),
            make_hit("ns", "c", 0.5, 3),
        ];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "b", "highest score first");
        assert_eq!(hits[1].record.key, "c", "middle score second");
        assert_eq!(hits[2].record.key, "a", "lowest score last");
    }

    #[test]
    fn sort_hits_ties_broken_by_key_then_node_id() {
        let mut hits = vec![
            make_hit("ns", "c", 0.5, 3),
            make_hit("ns", "a", 0.5, 1),
            make_hit("ns", "b", 0.5, 2),
        ];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "a", "ties broken alphabetically");
        assert_eq!(hits[1].record.key, "b");
        assert_eq!(hits[2].record.key, "c");
    }

    #[test]
    fn sort_hits_ties_same_key_different_node_id() {
        let mut hits = vec![make_hit("ns", "x", 0.5, 2), make_hit("ns", "x", 0.5, 1)];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.node_id, 1, "lower node_id first on tie");
        assert_eq!(hits[1].record.node_id, 2);
    }

    #[test]
    fn sort_hits_empty_list() {
        let mut hits: Vec<VantaMemorySearchHit> = vec![];
        sort_hits(&mut hits);
        assert!(hits.is_empty());
    }

    // ── optimize_and_compile ─────────────────────────────────────────────

    use crate::query::{LogicalOperator, LogicalPlan};

    #[test]
    fn optimize_and_compile_scan_only_produces_working_operator() {
        use crate::config::VantaConfig;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let plan = LogicalPlan {
            operators: vec![LogicalOperator::Scan { entity: "*".into() }],
            temperature: 0.0,
            enforce_role: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        assert!(op.next().unwrap().is_none(), "empty storage yields no rows");
        op.close().unwrap();
    }

    #[test]
    fn optimize_and_compile_scan_with_filter() {
        use crate::config::VantaConfig;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut node = UnifiedNode::new(1);
        node.relational
            .insert("type".into(), FieldValue::String("doc".into()));
        storage.insert(&node).unwrap();

        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::FilterRelational {
                    field: "type".into(),
                    op: RelOp::Eq,
                    value: FieldValue::String("doc".into()),
                },
            ],
            temperature: 0.0,
            enforce_role: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        let result = op.next().unwrap();
        assert!(result.is_some(), "filter matching node should be returned");
        assert_eq!(result.unwrap().id, 1);
        assert!(op.next().unwrap().is_none(), "no more rows");
        op.close().unwrap();
    }

    #[test]
    fn optimize_and_compile_scan_filter_no_match() {
        use crate::config::VantaConfig;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut node = UnifiedNode::new(1);
        node.relational
            .insert("type".into(), FieldValue::String("doc".into()));
        storage.insert(&node).unwrap();

        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::FilterRelational {
                    field: "type".into(),
                    op: RelOp::Eq,
                    value: FieldValue::String("note".into()),
                },
            ],
            temperature: 0.0,
            enforce_role: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        assert!(
            op.next().unwrap().is_none(),
            "filter excludes the only node"
        );
        op.close().unwrap();
    }

    #[test]
    fn optimize_and_compile_eliminates_identity_filter() {
        // CBO Rule 2: a filter with selectivity ≈ 1.0 should be skipped.
        // Insert one node; a filter on `type = doc` matches all rows
        // (selectivity = 1/1 = 1.0) → the optimizer should eliminate it.
        use crate::config::VantaConfig;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut node = UnifiedNode::new(42);
        node.relational
            .insert("type".into(), FieldValue::String("doc".into()));
        storage.insert(&node).unwrap();

        // Plan: scan + filter on `type = doc` (identity — matches 100 % rows)
        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::FilterRelational {
                    field: "type".into(),
                    op: RelOp::Eq,
                    value: FieldValue::String("doc".into()),
                },
            ],
            temperature: 0.0,
            enforce_role: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        let result = op.next().unwrap();
        assert!(
            result.is_some(),
            "identity filter eliminated: node should still be returned"
        );
        assert_eq!(result.unwrap().id, 42);
        // No more rows
        assert!(op.next().unwrap().is_none());
        op.close().unwrap();
    }

    #[test]
    fn optimize_and_compile_with_sort_limit_project() {
        use crate::config::VantaConfig;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        for i in 0..5 {
            let mut node = UnifiedNode::new(i);
            node.relational
                .insert("val".into(), FieldValue::Int(i as i64));
            node.relational
                .insert("name".into(), FieldValue::String(format!("n_{}", i)));
            storage.insert(&node).unwrap();
        }

        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::Sort {
                    field: "val".into(),
                    desc: true,
                },
                LogicalOperator::Project {
                    fields: vec!["val".into()],
                },
                LogicalOperator::Limit { top_k: 3 },
            ],
            temperature: 0.0,
            enforce_role: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        let mut values = Vec::new();
        while let Some(node) = op.next().unwrap() {
            values.push(node.relational.get("val").cloned());
            // Project should remove the "name" field
            assert!(
                !node.relational.contains_key("name"),
                "project should remove non-projected fields"
            );
        }
        assert_eq!(values.len(), 3, "limit caps to 3");
        assert_eq!(values[0], Some(FieldValue::Int(4)), "highest first (desc)");
        assert_eq!(values[1], Some(FieldValue::Int(3)));
        assert_eq!(values[2], Some(FieldValue::Int(2)));
        op.close().unwrap();
    }
}
