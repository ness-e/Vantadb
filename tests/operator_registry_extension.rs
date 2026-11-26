#![allow(clippy::expect_used, clippy::unwrap_used)]
//! C2S6 extension contract: a new operator ships without touching the proven
//! `planner` / `executor` matches.
//!
//! - `dedup_flows_through_planner_untouched`: plan `[Scan, Dedup]` compiles
//!   through `optimize_and_compile` (the exact path `execute_plan` uses) and
//!   deduplicates over a real in-memory engine. S6-2 added zero `Dedup` arms
//!   to `planner.rs` / `executor.rs` (only comments; `git diff` evidence).
//! - `locally_registered_operator_compiles_without_planner_or_executor`:
//!   a test-local compiler registered at runtime dispatches by name.
//! - `duplicate_registration_is_schema_error_not_silent_override`.
//! - `unknown_operator_is_schema_error_not_panic` (+ cost passthrough).

use vantadb::config::Config;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::operator_registry::{
    OperatorCompiler, OperatorCostModel, OperatorRegistry, DUPLICATE_OPERATOR_MARKER,
    UNKNOWN_OPERATOR_MARKER,
};
use vantadb::planner::optimize_and_compile;
use vantadb::query::{LogicalOperator, LogicalPlan, PhysicalOperator};
use vantadb::storage::{BackendKind, StorageEngine};

fn test_engine() -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        ..Config::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

fn named_node(id: u128, name: &str) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.relational
        .insert("name".into(), FieldValue::String(name.into()));
    node
}

fn scan_dedup_plan() -> LogicalPlan {
    LogicalPlan {
        operators: vec![
            LogicalOperator::Scan { entity: "*".into() },
            LogicalOperator::Dedup {
                field: "name".into(),
            },
        ],
        temperature: 0.0,
        enforce_role: None,
        search_profile: None,
    }
}

#[test]
fn dedup_flows_through_planner_untouched() {
    let engine = test_engine();
    engine.insert(&named_node(1, "alice")).expect("insert 1");
    engine.insert(&named_node(2, "bob")).expect("insert 2");
    engine.insert(&named_node(3, "alice")).expect("insert 3");

    // Same entry point `execute_plan` uses — planner source names no `Dedup`
    // arm (catch-all routes it to the registry).
    let mut op = optimize_and_compile(&scan_dedup_plan(), &engine).expect("compile");

    op.open().expect("open");
    let mut names: Vec<String> = Vec::new();
    while let Some(node) = op.next().expect("next") {
        match node.relational.get("name") {
            Some(FieldValue::String(s)) => names.push(s.clone()),
            other => panic!("expected name string, got {other:?}"),
        }
    }
    op.close().expect("close");

    names.sort();
    assert_eq!(names, vec!["alice".to_string(), "bob".to_string()]);
}

/// Test-local Volcano passthrough used to prove runtime registration.
struct PassChild<'a> {
    child: Option<Box<dyn PhysicalOperator + 'a>>,
}

impl PhysicalOperator for PassChild<'_> {
    fn open(&mut self) -> vantadb::error::Result<()> {
        if let Some(child) = self.child.as_mut() {
            child.open()?;
        }
        Ok(())
    }
    fn next(&mut self) -> vantadb::error::Result<Option<UnifiedNode>> {
        match self.child.as_mut() {
            Some(child) => child.next(),
            None => Ok(None),
        }
    }
    fn close(&mut self) -> vantadb::error::Result<()> {
        if let Some(child) = self.child.as_mut() {
            child.close()?;
        }
        Ok(())
    }
}

struct LocalCompiler;
struct LocalCost;

impl OperatorCompiler for LocalCompiler {
    fn operator_name(&self) -> &'static str {
        "traverse"
    }
    fn compile<'a>(
        &self,
        _op: &LogicalOperator,
        child: Box<dyn PhysicalOperator + 'a>,
    ) -> vantadb::error::Result<Box<dyn PhysicalOperator + 'a>> {
        Ok(Box::new(PassChild { child: Some(child) }))
    }
}

impl OperatorCostModel for LocalCost {
    fn operator_name(&self) -> &'static str {
        "traverse"
    }
    fn estimate(&self, _op: &LogicalOperator, in_rows: f64) -> (f64, usize) {
        (in_rows, (in_rows * 1024.0) as usize)
    }
}

fn traverse_op() -> LogicalOperator {
    LogicalOperator::Traverse {
        min_depth: 1,
        max_depth: 2,
        edge_label: "knows".into(),
    }
}

#[test]
fn locally_registered_operator_compiles_without_planner_or_executor() {
    // `traverse` has NO compiler in the default registry — registering one
    // locally proves dispatch-by-name works with zero planner/executor edits.
    let mut registry = OperatorRegistry::new();
    assert!(!registry.contains("traverse"));
    registry
        .register("traverse", LocalCompiler, LocalCost)
        .expect("register local operator");

    let engine = test_engine();
    let scan: Box<dyn PhysicalOperator> = Box::new(vantadb::physical_plan::PhysicalScan::new(
        &engine,
        "*".into(),
    ));
    let mut op = registry
        .compile(&traverse_op(), scan)
        .expect("local compiler runs");
    op.open().expect("open");
    op.close().expect("close");

    let (rows, _) = registry.estimate(&traverse_op(), 100.0);
    assert_eq!(rows, 100.0);
}

#[test]
fn duplicate_registration_is_schema_error_not_silent_override() {
    let mut registry = OperatorRegistry::new();
    let err = match registry.register("dedup", LocalCompiler, LocalCost) {
        Ok(()) => panic!("re-registering pre-registered 'dedup' must fail"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains(DUPLICATE_OPERATOR_MARKER),
        "unexpected: {err}"
    );
}

#[test]
fn unknown_operator_is_schema_error_not_panic() {
    let registry = OperatorRegistry::new();
    let engine = test_engine();
    let scan: Box<dyn PhysicalOperator> = Box::new(vantadb::physical_plan::PhysicalScan::new(
        &engine,
        "*".into(),
    ));
    let err = match registry.compile(&traverse_op(), scan) {
        Ok(_) => panic!("unregistered operator must fail"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains(UNKNOWN_OPERATOR_MARKER),
        "unexpected: {err}"
    );

    // Cost never fails: passthrough, rows preserved, bytes non-negative.
    let (rows, bytes) = registry.estimate(&traverse_op(), 100.0);
    assert!(rows <= 100.0);
    assert_eq!(rows, 100.0);
    assert_eq!(bytes, 100 * 1024);
}
