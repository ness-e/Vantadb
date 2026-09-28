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

use crate::node::{FieldValue, UnifiedNode};
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

// ── RRF fusion operator (WIRE-08) ─────────────────────────────────────────

/// Volcano operator that fuses the ranked outputs of its arms with Reciprocal
/// Rank Fusion (WIRE-08): each arm contributes `1 / (rrf_k + rank + 1)` per
/// node, with `rank` starting at 0 in the arm's natural output order. Nodes
/// appearing in several arms accumulate contributions. Output order is
/// `score desc, node id asc` (deterministic).
///
/// Local to the planner on purpose: the extension registry
/// (`OperatorRegistry`) dispatches on `LogicalOperator` variants and stays
/// untouched — no new variant, no consumer `match` edits (WIRE-08 stop
/// condition: extend by operator, never rewrite the planner).
///
/// ponytail: arms are drained eagerly into memory at `open()` (rank fusion
/// needs complete ranked lists). Streaming fusion with bounded windows is a
/// later optimization if profiles ever show it as a bottleneck.
struct PhysicalRrfFusion<'a> {
    arms: Vec<Box<dyn crate::query::PhysicalOperator + 'a>>,
    rrf_k: f32,
    fused: Vec<UnifiedNode>,
    cursor: usize,
}

impl<'a> PhysicalRrfFusion<'a> {
    fn new(arms: Vec<Box<dyn crate::query::PhysicalOperator + 'a>>, rrf_k: f32) -> Self {
        Self {
            arms,
            rrf_k,
            fused: Vec::new(),
            cursor: 0,
        }
    }
}

impl crate::query::PhysicalOperator for PhysicalRrfFusion<'_> {
    fn open(&mut self) -> crate::error::Result<()> {
        use std::collections::BTreeMap;
        self.fused.clear();
        self.cursor = 0;
        let mut scores: BTreeMap<u128, f32> = BTreeMap::new();
        let mut nodes: BTreeMap<u128, UnifiedNode> = BTreeMap::new();
        for arm in &mut self.arms {
            arm.open()?;
            let mut rank = 0usize;
            while let Some(node) = arm.next()? {
                let contribution = 1.0 / (self.rrf_k + rank as f32 + 1.0);
                *scores.entry(node.id).or_insert(0.0) += contribution;
                nodes.entry(node.id).or_insert(node);
                rank += 1;
            }
            arm.close()?;
        }
        let mut ids: Vec<u128> = scores.keys().copied().collect();
        ids.sort_by(|a, b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(b))
        });
        self.fused = ids.into_iter().filter_map(|id| nodes.remove(&id)).collect();
        Ok(())
    }

    fn next(&mut self) -> crate::error::Result<Option<UnifiedNode>> {
        if self.cursor < self.fused.len() {
            let node = self.fused[self.cursor].clone();
            self.cursor += 1;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> crate::error::Result<()> {
        self.fused.clear();
        self.cursor = 0;
        Ok(())
    }
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
    // WIRE-08: en modo Hybrid con ambos brazos presentes, `rrf_k` del profile
    // SÍ afecta el path IQL — alimenta el operador `PhysicalRrfFusion` de abajo.
    // `candidate_k` sigue siendo exclusivo del path SDK (el CBO no deriva
    // presupuestos de candidatos por brazo).
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
    let mut rrf_fused = false;
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
        // WIRE-08: RRF fusion operator — opt-in via `search_profile`. With a
        // profile, the vector arm and the lexical arm are fused (union
        // semantics, same shape as the SDK hybrid path); without one, the
        // proven vector-then-filter path below is byte-identical.
        let fuse_rrf = plan.search_profile.is_some() && !text_matches.is_empty();
        if fuse_rrf {
            let vector_arm: Box<dyn crate::query::PhysicalOperator + 'a> = Box::new(
                crate::physical_plan::PhysicalVectorSearch::new(storage, query_text, min_score),
            );
            let mut lexical_arm: Box<dyn crate::query::PhysicalOperator + 'a> =
                Box::new(crate::physical_plan::PhysicalScan::new(storage, entity));
            for (field, query) in &text_matches {
                lexical_arm = Box::new(crate::physical_plan::PhysicalTextFilter::new(
                    lexical_arm,
                    field.clone(),
                    query.clone(),
                ));
            }
            let rrf_k = plan
                .search_profile
                .and_then(|profile| profile.rrf_k)
                .map(|k| k as f32)
                .unwrap_or(crate::search_profile::RRF_K);
            let mut fused: Box<dyn crate::query::PhysicalOperator + 'a> =
                Box::new(PhysicalRrfFusion::new(vec![vector_arm, lexical_arm], rrf_k));
            for (field, rel_op, value) in sorted_filters {
                fused = Box::new(crate::physical_plan::PhysicalFilter::new(
                    fused, field, rel_op, value,
                ));
            }
            rrf_fused = true;
            fused
        } else if joint_selectivity < HIGH_SELECTIVITY_THRESHOLD && !sorted_filters.is_empty() {
            // CBO: filter-before-vector vs vector-before-filter
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

    // Apply lexical text filters (phrase-aware) on top of the chain. With RRF
    // fusion the text condition is already an arm of the fused operator.
    if !rrf_fused {
        for (field, query) in text_matches {
            current_operator = Box::new(crate::physical_plan::PhysicalTextFilter::new(
                current_operator,
                field,
                query,
            ));
        }
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

#[cfg(test)]
mod wire08_rrf_tests {
    use super::*;
    use crate::query::PhysicalOperator as _;
    use crate::query::{LogicalOperator, LogicalPlan};

    // ── WIRE-08: RRF fusion operator ─────────────────────────────────────────

    /// Minimal in-memory arm for RRF operator tests (rank order = vec order).
    struct MockRrfArm {
        nodes: Vec<UnifiedNode>,
        cursor: usize,
    }

    impl crate::query::PhysicalOperator for MockRrfArm {
        fn open(&mut self) -> crate::error::Result<()> {
            self.cursor = 0;
            Ok(())
        }
        fn next(&mut self) -> crate::error::Result<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                return Ok(Some(node));
            }
            Ok(None)
        }
        fn close(&mut self) -> crate::error::Result<()> {
            self.cursor = 0;
            Ok(())
        }
    }

    fn rrf_node(id: u128) -> UnifiedNode {
        UnifiedNode::new(id)
    }

    fn drain(op: &mut dyn crate::query::PhysicalOperator) -> Vec<u128> {
        let mut ids = Vec::new();
        while let Some(node) = op.next().unwrap() {
            ids.push(node.id);
        }
        ids
    }

    #[test]
    fn rrf_fusion_scores_by_rank_and_dedups_overlap() {
        // arm1 = [1, 2], arm2 = [2, 3] with rrf_k = 60:
        // 2 → 1/61 + 1/61 = 0.03279; 1 → 1/61 = 0.01639; 3 → 1/62 = 0.01613.
        let arm1: Box<dyn crate::query::PhysicalOperator> = Box::new(MockRrfArm {
            nodes: vec![rrf_node(1), rrf_node(2)],
            cursor: 0,
        });
        let arm2: Box<dyn crate::query::PhysicalOperator> = Box::new(MockRrfArm {
            nodes: vec![rrf_node(2), rrf_node(3)],
            cursor: 0,
        });
        let mut fused = PhysicalRrfFusion::new(vec![arm1, arm2], 60.0);
        fused.open().unwrap();
        let ids = drain(&mut fused);
        fused.close().unwrap();
        assert_eq!(ids, vec![2, 1, 3], "overlap wins, ranks order the rest");
    }

    #[test]
    fn rrf_fusion_handles_empty_and_single_arms() {
        let empty: Box<dyn crate::query::PhysicalOperator> = Box::new(MockRrfArm {
            nodes: Vec::new(),
            cursor: 0,
        });
        let single: Box<dyn crate::query::PhysicalOperator> = Box::new(MockRrfArm {
            nodes: vec![rrf_node(7), rrf_node(8)],
            cursor: 0,
        });
        let mut fused = PhysicalRrfFusion::new(vec![empty, single], 60.0);
        fused.open().unwrap();
        let ids = drain(&mut fused);
        fused.close().unwrap();
        assert_eq!(ids, vec![7, 8], "empty arm contributes nothing");
    }

    #[test]
    fn rrf_cbo_fuses_arms_when_profile_is_present() {
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::search_profile::{SearchProfileConfig, SearchProfileMode};
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut alpha = UnifiedNode::new(1);
        alpha
            .relational
            .insert("content".into(), FieldValue::String("alpha beta".into()));
        storage.insert(&alpha).unwrap();
        let mut gamma = UnifiedNode::new(2);
        gamma
            .relational
            .insert("content".into(), FieldValue::String("gamma delta".into()));
        storage.insert(&gamma).unwrap();

        let operators = vec![
            LogicalOperator::Scan { entity: "*".into() },
            LogicalOperator::VectorSearch {
                field: "content".into(),
                query_vec: "alpha".into(),
                min_score: 0.0,
            },
            LogicalOperator::TextFilter {
                field: "content".into(),
                query: "alpha".into(),
            },
            LogicalOperator::Limit { top_k: 5 },
        ];

        // Without a profile: proven intersection path — the (empty, no embedding
        // feature) vector arm is post-filtered by text → zero rows.
        let plan_no_profile = LogicalPlan {
            operators: operators.clone(),
            temperature: 0.0,
            enforce_role: None,
            search_profile: None,
        };
        let mut op = optimize_and_compile(&plan_no_profile, &storage).unwrap();
        op.open().unwrap();
        assert!(
            op.next().unwrap().is_none(),
            "no profile keeps the pre-WIRE-08 intersection semantics"
        );
        op.close().unwrap();

        // With a Hybrid profile: RRF operator fuses the text arm (vector arm is
        // empty without embedding features) → the text-only match survives.
        let plan_profile = LogicalPlan {
            operators,
            temperature: 0.0,
            enforce_role: None,
            search_profile: Some(SearchProfileConfig {
                mode: SearchProfileMode::Hybrid,
                rrf_k: Some(60),
                candidate_k: None,
            }),
        };
        let mut op = optimize_and_compile(&plan_profile, &storage).unwrap();
        op.open().unwrap();
        let row = op.next().unwrap().expect("fused row");
        assert_eq!(row.id, 1, "lexical arm match is returned by the fusion");
        assert!(op.next().unwrap().is_none());
        op.close().unwrap();
    }

    #[test]
    fn rrf_cbo_applies_relational_filters_after_fusion() {
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::query::RelOp;
        use crate::search_profile::{SearchProfileConfig, SearchProfileMode};
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut keep = UnifiedNode::new(1);
        keep.relational
            .insert("content".into(), FieldValue::String("alpha beta".into()));
        keep.relational
            .insert("tier".into(), FieldValue::String("hot".into()));
        storage.insert(&keep).unwrap();
        let mut drop = UnifiedNode::new(2);
        drop.relational
            .insert("content".into(), FieldValue::String("alpha gamma".into()));
        drop.relational
            .insert("tier".into(), FieldValue::String("cold".into()));
        storage.insert(&drop).unwrap();

        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::VectorSearch {
                    field: "content".into(),
                    query_vec: "alpha".into(),
                    min_score: 0.0,
                },
                LogicalOperator::TextFilter {
                    field: "content".into(),
                    query: "alpha".into(),
                },
                LogicalOperator::FilterRelational {
                    field: "tier".into(),
                    op: RelOp::Eq,
                    value: FieldValue::String("hot".into()),
                },
                LogicalOperator::Limit { top_k: 5 },
            ],
            temperature: 0.0,
            enforce_role: None,
            search_profile: Some(SearchProfileConfig {
                mode: SearchProfileMode::Hybrid,
                rrf_k: None,
                candidate_k: None,
            }),
        };
        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        let rows = drain(op.as_mut());
        op.close().unwrap();
        assert_eq!(rows, vec![1], "relational filter applies over fused rows");
    }

    #[test]
    fn rrf_cbo_keyword_profile_keeps_text_only_path() {
        use crate::config::Config;
        use crate::node::{FieldValue, UnifiedNode};
        use crate::search_profile::{SearchProfileConfig, SearchProfileMode};
        use crate::storage::{BackendKind, StorageEngine};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");

        let mut alpha = UnifiedNode::new(1);
        alpha
            .relational
            .insert("content".into(), FieldValue::String("alpha beta".into()));
        storage.insert(&alpha).unwrap();

        // Keyword mode clears the vector arm → plain scan + text filter path.
        let plan = LogicalPlan {
            operators: vec![
                LogicalOperator::Scan { entity: "*".into() },
                LogicalOperator::VectorSearch {
                    field: "content".into(),
                    query_vec: "alpha".into(),
                    min_score: 0.0,
                },
                LogicalOperator::TextFilter {
                    field: "content".into(),
                    query: "alpha".into(),
                },
            ],
            temperature: 0.0,
            enforce_role: None,
            search_profile: Some(SearchProfileConfig {
                mode: SearchProfileMode::Keyword,
                rrf_k: None,
                candidate_k: None,
            }),
        };
        let mut op = optimize_and_compile(&plan, &storage).unwrap();
        op.open().unwrap();
        let rows = drain(op.as_mut());
        op.close().unwrap();
        assert_eq!(rows, vec![1], "keyword mode keeps the text filter path");
    }
}
