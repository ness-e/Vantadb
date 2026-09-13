// ponytail: planner invariants on join_spec Some/None flow above the call site; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Search planner for VantaDB hybrid retrieval.
//!
//! This module owns the routing logic, RRF fusion constants, and candidate
//! budget derivation that drive `Embedded::search`. Extracting these
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

use crate::node::FieldValue;
use crate::query::RelOp;
use crate::search_profile::SearchProfileMode;

// ── Planner constants ─────────────────────────────────────────────────────
// RRF / candidate-budget consts live in the neutral `crate::search_profile`
// leaf (C2M3); fusion helpers live in `crate::sdk::search::fusion.

/// Selectivity threshold below which filters are considered highly selective.
///
/// When joint selectivity falls below this value, the optimizer prefers
/// applying filters before vector search (scan→filter→refine) over the
/// default vector-search→filter order. A filter with selectivity 0.1 means
/// it prunes ~90 % of rows.
pub const HIGH_SELECTIVITY_THRESHOLD: f32 = 0.1;

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
    let mut text_matches: Vec<(String, String)> = Vec::new();

    // JOIN and SubqueryFilter produce their own sub-plans that wrap the chain
    let mut has_join = false;
    let mut join_spec: Option<(
        crate::query::LogicalPlan,
        crate::query::LogicalPlan,
        String,
        String,
    )> = None;
    let mut subquery_filters: Vec<(String, RelOp, crate::query::LogicalPlan)> = Vec::new();
    // C2S6: extension operators (e.g. `Dedup`) never get a named arm here —
    // they collect below and compile through `OperatorRegistry` by name.
    let mut pending_extensions: Vec<crate::query::LogicalOperator> = Vec::new();

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
                subquery_filters.push((field.clone(), op.clone(), *subquery_plan.clone()));
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
            crate::query::LogicalOperator::TextFilter { field, query } => {
                text_matches.push((field.clone(), query.clone()));
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
            // C2S6 legacy: `Traverse` has no physical operator; keep the
            // historical ignore (governor still reads it). Turning this into
            // an error without a physical impl would be a breaking change —
            // explicitly out of scope (design §4: no `Traverse` physical).
            crate::query::LogicalOperator::Traverse { .. } => {}
            // C2S6: anything else (today: `Dedup`; tomorrow: new operators)
            // compiles via the registry — this match never names extensions.
            _ => {
                pending_extensions.push(op.clone());
            }
        }
    }

    // MEM-01: el perfil de búsqueda puede forzar el modo en el plan físico.
    // Keyword descarta el vector search (queda solo el filtro léxico);
    // Vector descarta los filtros de texto (queda solo el vector search).
    // ponytail: rrf_k/candidate_k del profile se propagan al LogicalPlan pero no
    // afectan el path IQL: el CBO no fusiona RRF (solo el path SDK lo usa).
    if let Some(profile) = plan.search_profile {
        match profile.mode {
            SearchProfileMode::Keyword => vector_search = None,
            SearchProfileMode::Vector => text_matches.clear(),
            SearchProfileMode::Hybrid => {}
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
    let mut current_operator: Box<dyn crate::query::PhysicalOperator + 'a> = if has_join {
        // INVARIANT (B2b): `has_join` is set true only in the `Join` arm above,
        // which always sets `join_spec` in the same statement — `None` here is
        // unreachable via the public API. `ok_or_else` keeps E1 green and turns
        // a hypothetical inconsistency into `Schema` instead of a panic.
        let (left_plan, right_plan, left_field, right_field) = join_spec
            .ok_or_else(|| crate::error::Error::Schema("JOIN operator without join spec".into()))?;
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
        current_operator = Box::new(crate::physical_plan::PhysicalSubqueryFilter::new(
            current_operator,
            subq_op,
            field,
            op,
        ));
    }

    // Apply lexical text filters (phrase-aware) on top of the chain
    for (field, query) in text_matches {
        current_operator = Box::new(crate::physical_plan::PhysicalTextFilter::new(
            current_operator,
            field,
            query,
        ));
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

    // C2S6: wrap extension operators last (dispatch by name — adding an
    // operator means a new variant + `register`, never an arm above).
    // Extensions apply post-chain in plan order; like `sort/project/limit`
    // they wrap whatever the base chain produced.
    if !pending_extensions.is_empty() {
        let registry = crate::operator_registry::OperatorRegistry::new();
        for ext in &pending_extensions {
            current_operator = registry.compile(ext, current_operator)?;
        }
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

    // ── Route labels ─────────────────────────────────────────────────────

    #[test]
    fn route_labels_match_debug_report_strings() {
        assert_eq!(SearchRoute::Hybrid.label(), "hybrid");
        assert_eq!(SearchRoute::TextOnly.label(), "text-only");
        assert_eq!(SearchRoute::VectorOnly.label(), "vector-only");
        assert_eq!(SearchRoute::Empty.label(), "empty");
    }

    // ── optimize_and_compile ─────────────────────────────────────────────

    use crate::query::{LogicalOperator, LogicalPlan};

    #[test]
    fn optimize_and_compile_scan_only_produces_working_operator() {
        use crate::config::Config;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let plan = LogicalPlan {
            operators: vec![LogicalOperator::Scan { entity: "*".into() }],
            temperature: 0.0,
            enforce_role: None,
            search_profile: None,
        };

        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        assert!(op.next().unwrap().is_none(), "empty storage yields no rows");
        op.close().unwrap();
    }

    #[test]
    fn optimize_and_compile_scan_with_filter() {
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
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
            search_profile: None,
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
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
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
            search_profile: None,
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
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
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
            search_profile: None,
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
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
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
            search_profile: None,
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
