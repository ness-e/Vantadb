//! Extensible operator registry (C2S6): dispatch-by-name extension point
//! for `LogicalOperator`.
//!
//! OCP-grave fix: adding an operator must NOT edit the proven
//! `planner` / `executor` / `cost_estimator` matches. Built-in operators keep
//! their existing match arms (zero semantic change); NEW operators register
//! here by name and the planner's catch-all routes them through
//! [`OperatorRegistry::compile`] instead of silently dropping them.
//!
//! Leaf discipline (M3 precedent `search_profile.rs`): this module depends
//! only on `query` + `error` (downward). It never imports `planner`,
//! `executor`, `governor`, `storage` or `cost_estimator`, so no cycle is
//! possible (`planner` / `cost_estimator` import from here, never the reverse).
//!
//! Error contract: unknown / duplicate operators surface as
//! [`Error::Schema`] whose `Display` carries a stable marker
//! (`SCHEMA_UNKNOWN_OPERATOR` / `SCHEMA_DUPLICATE_OPERATOR`). `code()` stays
//! the canonical `VANTADB_CORRUPT` per ERR-CORE-01 (adding a new `Error`
//! variant would force snapshot + cross-binding changes — out of scope for
//! S6). Clients match on the `Display` marker substring, never on prose.

use std::collections::BTreeMap;

use crate::error::{Error, Result};
use crate::physical_plan::PhysicalDedup;
use crate::query::{LogicalOperator, PhysicalOperator};

/// Stable `Display` marker for compiling an unregistered operator.
/// Never `panic!`s — unknown operators are `Err`, not crashes (ends the
/// silent half-operator precedent H1 of `planner.rs:161`).
pub const UNKNOWN_OPERATOR_MARKER: &str = "SCHEMA_UNKNOWN_OPERATOR";

/// Stable `Display` marker for registering a name twice.
/// Additive registry: re-registration is an error, never a silent override.
pub const DUPLICATE_OPERATOR_MARKER: &str = "SCHEMA_DUPLICATE_OPERATOR";

/// Canonical name of a logical operator. The registry dispatches on this
/// name, NOT on the enum variant — that is what makes new operators
/// additive (new variant + `register`, zero `match` edits in consumers).
pub fn operator_name(op: &LogicalOperator) -> &'static str {
    match op {
        LogicalOperator::Scan { .. } => "scan",
        LogicalOperator::Traverse { .. } => "traverse",
        LogicalOperator::FilterRelational { .. } => "filter_relational",
        LogicalOperator::VectorSearch { .. } => "vector_search",
        LogicalOperator::TextFilter { .. } => "text_filter",
        LogicalOperator::Project { .. } => "project",
        LogicalOperator::Sort { .. } => "sort",
        LogicalOperator::Limit { .. } => "limit",
        LogicalOperator::Join { .. } => "join",
        LogicalOperator::SubqueryFilter { .. } => "subquery_filter",
        LogicalOperator::Dedup { .. } => "dedup",
    }
}

/// Names handled by the long-proven `planner` / `cost_estimator` matches.
/// Anything NOT in this list is an extension and must go through the registry.
pub const BUILTIN_OPERATOR_NAMES: [&str; 10] = [
    "scan",
    "traverse",
    "filter_relational",
    "vector_search",
    "text_filter",
    "project",
    "sort",
    "limit",
    "join",
    "subquery_filter",
];

/// Whether `name` is a built-in operator (handled by the classic matches).
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_OPERATOR_NAMES.contains(&name)
}

/// How to compile ONE extension operator into its Volcano fragment.
/// Receives the operator plus the already-compiled child chain and returns
/// the wrapped operator (same wrapping style as `Sort`/`Project`/`Limit`).
/// Extensions are pure child-wrappers: no `storage` access, so the registry
/// stays a storage-free leaf. Errors use `Error::Schema` for malformed ops.
pub trait OperatorCompiler: Send + Sync {
    /// Registry key this compiler handles (`"dedup"`, …).
    fn operator_name(&self) -> &'static str;
    /// Wrap `child` into the physical operator for `op`.
    fn compile<'a>(
        &self,
        op: &LogicalOperator,
        child: Box<dyn PhysicalOperator + 'a>,
    ) -> Result<Box<dyn PhysicalOperator + 'a>>;
}

/// How to cost ONE extension operator given the incoming row count.
/// Pure function of `(op, in_rows)` — same top-down rows semantics as
/// `CostEstimator::estimate_operator`. Unknown operators fall back to
/// passthrough (`in_rows`, never `panic!`).
pub trait OperatorCostModel: Send + Sync {
    /// Registry key this model handles.
    fn operator_name(&self) -> &'static str;
    /// `(estimated_rows, estimated_bytes)` for `op` given `in_rows`.
    fn estimate(&self, op: &LogicalOperator, in_rows: f64) -> (f64, usize);
}

struct ExtensionEntry {
    compiler: Box<dyn OperatorCompiler>,
    cost: Box<dyn OperatorCostModel>,
}

/// Byte floor for extension cost passthrough.
// ponytail: mirrors `cost_estimator::AVG_NODE_BYTES` (cold path only —
// estimates never scan). Kept local so this leaf never imports the estimator.
const EXTENSION_AVG_NODE_BYTES: usize = 1024;

/// Dispatch-by-name registry for extension operators.
///
/// `new()` starts EMPTY on purpose: built-ins live in the proven matches,
/// extensions arrive via [`register`](Self::register). One registry, one
/// live version (One-Version Rule — extend traits with defaulted methods,
/// never `OperatorRegistryV2`).
pub struct OperatorRegistry {
    entries: BTreeMap<&'static str, ExtensionEntry>,
}

impl OperatorRegistry {
    /// Empty registry (built-ins are NOT preloaded — they stay in the
    /// proven `planner` / `cost_estimator` matches; see module docs).
    /// The `dedup` extension exemplar IS pre-registered so the planner's
    /// catch-all resolves it with zero planner edits per operator.
    pub fn new() -> Self {
        let mut registry = Self {
            entries: BTreeMap::new(),
        };
        // Pre-registration is infallible (fresh map, unique name); a failure
        // here would be a programming error, so `debug_assert` + ignore.
        let pre = registry.register("dedup", DedupCompiler, DedupCost);
        debug_assert!(pre.is_ok());
        let _ = pre;
        registry
    }

    /// Number of registered extensions (diagnostics / tests).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry holds an entry (built-in or extension).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether `name` has a registered extension compiler.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Register an extension compiler + cost model under `name`.
    /// Duplicate names are `Err(Schema)` with [`DUPLICATE_OPERATOR_MARKER`]
    /// — additive only, never a silent override.
    pub fn register<C, M>(&mut self, name: &'static str, compiler: C, cost: M) -> Result<()>
    where
        C: OperatorCompiler + 'static,
        M: OperatorCostModel + 'static,
    {
        if self.entries.contains_key(name) {
            return Err(Error::Schema(format!(
                "{DUPLICATE_OPERATOR_MARKER}: '{name}' is already registered"
            )));
        }
        self.entries.insert(
            name,
            ExtensionEntry {
                compiler: Box::new(compiler),
                cost: Box::new(cost),
            },
        );
        Ok(())
    }

    /// Compile `op` by dispatching on [`operator_name`] to the registered
    /// compiler. Unregistered names are `Err(Schema)` with
    /// [`UNKNOWN_OPERATOR_MARKER`] — never `panic!`.
    pub fn compile<'a>(
        &self,
        op: &LogicalOperator,
        child: Box<dyn PhysicalOperator + 'a>,
    ) -> Result<Box<dyn PhysicalOperator + 'a>> {
        let name = operator_name(op);
        match self.entries.get(name) {
            Some(entry) => entry.compiler.compile(op, child),
            None => Err(Error::Schema(format!(
                "{UNKNOWN_OPERATOR_MARKER}: '{name}' has no registered compiler"
            ))),
        }
    }

    /// Cost `op` via the registered model, or passthrough `in_rows` when
    /// unregistered (same convention as `Sort`/`Project`/`Join` today).
    /// Never panics.
    pub fn estimate(&self, op: &LogicalOperator, in_rows: f64) -> (f64, usize) {
        let name = operator_name(op);
        match self.entries.get(name) {
            Some(entry) => entry.cost.estimate(op, in_rows),
            None => (
                in_rows,
                (in_rows * EXTENSION_AVG_NODE_BYTES as f64) as usize,
            ),
        }
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Extension exemplar wiring ───────────────────────────────
// The `dedup` compiler + cost live HERE (not in `physical_plan/dedup.rs`)
// so the dependency runs one way only: `registry → physical_plan`, never
// back. `physical_plan` stays a pure Volcano leaf (`query`/`node` only),
// which keeps `cargo modules --acyclic` green for these modules.
// Future storage-free extensions follow the same pattern: physical struct
// in `physical_plan/`, compiler + cost appended below, one `register` line
// in `new()`. `planner` / `executor` / `cost_estimator` never change.

/// Compiles `LogicalOperator::Dedup` by wrapping the child chain
/// (same style as `Sort` / `Project` / `Limit` in the planner).
pub struct DedupCompiler;

/// Costs `Dedup` as passthrough (same convention as `Sort` / `Project` /
/// `Join`): the dedup ratio is data-dependent and the estimator holds no
/// per-value stats, so 1.0 is the honest estimate — zero invention.
pub struct DedupCost;

impl OperatorCompiler for DedupCompiler {
    fn operator_name(&self) -> &'static str {
        "dedup"
    }

    fn compile<'a>(
        &self,
        op: &LogicalOperator,
        child: Box<dyn PhysicalOperator + 'a>,
    ) -> Result<Box<dyn PhysicalOperator + 'a>> {
        match op {
            LogicalOperator::Dedup { field } => {
                Ok(Box::new(PhysicalDedup::new(child, field.clone())))
            }
            other => Err(Error::Schema(format!(
                "dedup compiler cannot compile '{}'",
                operator_name(other)
            ))),
        }
    }
}

impl OperatorCostModel for DedupCost {
    fn operator_name(&self) -> &'static str {
        "dedup"
    }

    fn estimate(&self, _op: &LogicalOperator, in_rows: f64) -> (f64, usize) {
        (
            in_rows,
            (in_rows * EXTENSION_AVG_NODE_BYTES as f64) as usize,
        )
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::{FieldValue, UnifiedNode};

    struct MockScan {
        nodes: Vec<UnifiedNode>,
        cursor: usize,
    }

    impl MockScan {
        fn new(nodes: Vec<UnifiedNode>) -> Self {
            Self { nodes, cursor: 0 }
        }
    }

    impl PhysicalOperator for MockScan {
        fn open(&mut self) -> Result<()> {
            self.cursor = 0;
            Ok(())
        }
        fn next(&mut self) -> Result<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                return Ok(Some(node));
            }
            Ok(None)
        }
        fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    struct PassthroughCompiler;
    struct PassthroughCost;

    impl OperatorCompiler for PassthroughCompiler {
        fn operator_name(&self) -> &'static str {
            "traverse"
        }
        fn compile<'a>(
            &self,
            _op: &LogicalOperator,
            child: Box<dyn PhysicalOperator + 'a>,
        ) -> Result<Box<dyn PhysicalOperator + 'a>> {
            Ok(child)
        }
    }

    impl OperatorCostModel for PassthroughCost {
        fn operator_name(&self) -> &'static str {
            "traverse"
        }
        fn estimate(&self, _op: &LogicalOperator, in_rows: f64) -> (f64, usize) {
            (
                in_rows,
                (in_rows * EXTENSION_AVG_NODE_BYTES as f64) as usize,
            )
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
    fn operator_names_cover_all_builtins() {
        use crate::query::LogicalPlan;
        let scan_plan = || LogicalPlan {
            operators: vec![LogicalOperator::Scan { entity: "A".into() }],
            temperature: 0.0,
            enforce_role: None,
            search_profile: None,
        };
        let cases: Vec<(LogicalOperator, &str)> = vec![
            (LogicalOperator::Scan { entity: "E".into() }, "scan"),
            (traverse_op(), "traverse"),
            (
                LogicalOperator::FilterRelational {
                    field: "f".into(),
                    op: crate::query::RelOp::Eq,
                    value: FieldValue::Int(1),
                },
                "filter_relational",
            ),
            (
                LogicalOperator::VectorSearch {
                    field: "f".into(),
                    query_vec: "q".into(),
                    min_score: 0.5,
                },
                "vector_search",
            ),
            (
                LogicalOperator::TextFilter {
                    field: "f".into(),
                    query: "q".into(),
                },
                "text_filter",
            ),
            (
                LogicalOperator::Project {
                    fields: vec!["a".into()],
                },
                "project",
            ),
            (
                LogicalOperator::Sort {
                    field: "f".into(),
                    desc: false,
                },
                "sort",
            ),
            (LogicalOperator::Limit { top_k: 3 }, "limit"),
            (
                LogicalOperator::Dedup {
                    field: "name".into(),
                },
                "dedup",
            ),
            (
                LogicalOperator::Join {
                    left_plan: Box::new(scan_plan()),
                    right_plan: Box::new(scan_plan()),
                    left_field: "l".into(),
                    right_field: "r".into(),
                },
                "join",
            ),
            (
                LogicalOperator::SubqueryFilter {
                    field: "f".into(),
                    op: crate::query::RelOp::Gt,
                    subquery_plan: Box::new(scan_plan()),
                },
                "subquery_filter",
            ),
        ];
        assert_eq!(cases.len(), 11, "one case per operator");
        for (op, want) in &cases {
            assert_eq!(operator_name(op), *want);
        }
        // Built-ins stay in the proven matches; dedup is the registry exemplar.
        assert_eq!(BUILTIN_OPERATOR_NAMES.len(), 10);
        for name in BUILTIN_OPERATOR_NAMES {
            assert!(is_builtin(name), "{name} must be builtin");
        }
        assert!(!is_builtin("dedup"));
    }

    #[test]
    fn custom_operator_registers_and_compiles() {
        let mut registry = OperatorRegistry::new();
        assert!(registry.contains("dedup"), "dedup pre-registered");
        registry
            .register("traverse", PassthroughCompiler, PassthroughCost)
            .expect("first registration works");
        assert!(registry.contains("traverse"));
        assert_eq!(registry.len(), 2);
        let child: Box<dyn PhysicalOperator> = Box::new(MockScan::new(vec![]));
        let compiled = registry
            .compile(&traverse_op(), child)
            .expect("registered compiler runs");
        let mut compiled = compiled;
        compiled.open().expect("open");
        assert!(compiled.next().expect("next").is_none());
        compiled.close().expect("close");
        let (rows, bytes) = registry.estimate(&traverse_op(), 100.0);
        assert_eq!(rows, 100.0);
        assert_eq!(bytes, 100 * EXTENSION_AVG_NODE_BYTES);
    }

    #[test]
    fn duplicate_registration_is_schema_error_not_silent_override() {
        let mut registry = OperatorRegistry::new();
        registry
            .register("traverse", PassthroughCompiler, PassthroughCost)
            .expect("first registration works");
        let err = match registry.register("traverse", PassthroughCompiler, PassthroughCost) {
            Ok(()) => panic!("duplicate must fail"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains(DUPLICATE_OPERATOR_MARKER),
            "unexpected: {err}"
        );
        assert!(registry.contains("dedup"), "pre-registration intact");
        assert!(registry.contains("traverse"), "first registration intact");
    }

    #[test]
    fn unknown_operator_compile_is_schema_error_not_panic() {
        let registry = OperatorRegistry::new();
        let child: Box<dyn PhysicalOperator> = Box::new(MockScan::new(vec![]));
        let err = match registry.compile(&traverse_op(), child) {
            Ok(_) => panic!("unregistered must fail"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains(UNKNOWN_OPERATOR_MARKER),
            "unexpected: {err}"
        );
    }

    #[test]
    fn unknown_operator_estimate_is_passthrough_not_panic() {
        let registry = OperatorRegistry::new();
        let (rows, bytes) = registry.estimate(&traverse_op(), 100.0);
        assert_eq!(rows, 100.0);
        assert_eq!(bytes, 100 * EXTENSION_AVG_NODE_BYTES);
    }

    #[test]
    fn dedup_wrong_type_is_schema_error() {
        let compiler = DedupCompiler;
        let child: Box<dyn PhysicalOperator> = Box::new(MockScan::new(vec![]));
        let other = LogicalOperator::Scan { entity: "E".into() };
        let err = match compiler.compile(&other, child) {
            Ok(_) => panic!("non-dedup must fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("dedup compiler cannot compile"));
    }
}
