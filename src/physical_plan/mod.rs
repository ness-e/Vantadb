//! Physical query plan operators executed against storage.
//!
//! [`PhysicalScan`] and related operators translate logical plan nodes
//! into concrete storage reads, filtering, and projection.
//!
//! Split into per-operator submodules (REVIEW-05): `scan`, `filter`,
//! `vector`, `project`, `sort`, `join`, `dedup` (C2S6 extension exemplar),
//! `offset` (WIRE-12 registry extension), `aggregate` (WIRE-13 registry
//! extension).

mod aggregate;
mod dedup;
mod filter;
mod join;
mod offset;
mod project;
mod scan;
mod sort;
mod vector;

pub use aggregate::PhysicalAggregate;
pub use dedup::PhysicalDedup;
pub use filter::{PhysicalFilter, PhysicalTextFilter};
pub use join::{PhysicalNestedLoopJoin, PhysicalSubqueryFilter};
pub use offset::PhysicalOffset;
pub use project::{PhysicalLimit, PhysicalProject};

/// System-metadata field names shared between the projection and the
/// post-plan filters (FIND-552) — `pub(crate)`: internal contract, so the
/// public API surface (`tests/api/public-api.txt`) does not change.
/// `SYSTEM_METADATA_FIELDS` (the allowlist itself) stays private to
/// `project`, its only consumer.
pub(crate) use project::{FIELD_OWNER_ROLE, FIELD_TYPE};
pub use scan::PhysicalScan;
pub use sort::PhysicalSort;
pub use vector::{PhysicalVectorRefine, PhysicalVectorSearch};

/// Query-embedder seam (FIND-413) — `pub(crate)`: test-only injection point.
pub(crate) use vector::QueryEmbedder;

#[cfg(test)]
pub(crate) use filter::evaluate_condition;

// ─── Shared field-reference / numeric helpers (FIND-556, FIND-157) ──────

/// Strip the alias qualifier from a field reference (`"p.name"` → `"name"`).
///
/// The lexer admits `.` in idents, so parser-produced field references may
/// carry the table alias while stored relational keys never do. Callers
/// must try the **exact** key first — a real dotted key (`meta.env`) can
/// legitimately exist and must win — and only then fall back to the
/// stripped form (FIND-556 risk register).
pub(crate) fn strip_alias(field_ref: &str) -> &str {
    field_ref.rsplit('.').next().unwrap_or(field_ref)
}

/// Exact `i64` vs `f64` ordering without precision loss (FIND-157 DoD).
///
/// `i64 as f64` rounds above 2^53, which would silently declare
/// `Int(2^53 + 1)` equal to `Float(2^53)` — the silent-wrong-result class
/// this helper exists to remove. Returns `None` for non-finite `f` (NaN
/// has no total order); callers map `None` to `false` for every operator,
/// including `Neq`.
pub(crate) fn cmp_int_float(i: i64, f: f64) -> Option<std::cmp::Ordering> {
    if !f.is_finite() {
        return None;
    }
    // First `f64` above `i64::MAX` (2^63); `-I64_RANGE` == `i64::MIN` exactly.
    const I64_RANGE: f64 = 9_223_372_036_854_775_808.0;
    if f >= I64_RANGE {
        return Some(std::cmp::Ordering::Less); // every i64 < f
    }
    if f < -I64_RANGE {
        return Some(std::cmp::Ordering::Greater); // every i64 > f
    }
    // f ∈ [-2^63, 2^63): `f.trunc()` fits i64 and `as i64` is exact in range
    // (|trunc| < 2^63 is always representable; no rounding can occur).
    let f_trunc = f.trunc() as i64;
    match i.cmp(&f_trunc) {
        std::cmp::Ordering::Equal => {
            // Same integer part — the fraction decides (f = trunc + fract).
            if f > f_trunc as f64 {
                Some(std::cmp::Ordering::Less) // i = trunc < f
            } else if f < f_trunc as f64 {
                Some(std::cmp::Ordering::Greater) // i = trunc > f
            } else {
                Some(std::cmp::Ordering::Equal)
            }
        }
        ord => Some(ord),
    }
}

/// Apply a [`crate::query::RelOp`] to an [`Ordering`](std::cmp::Ordering)
/// (FIND-157): shared by
/// `filter` and `join` so `Gte`/`Lte` semantics can't drift between the two
/// comparison paths.
pub(crate) fn rel_op_matches(ord: std::cmp::Ordering, op: &crate::query::RelOp) -> bool {
    use std::cmp::Ordering::*;
    match op {
        crate::query::RelOp::Eq => ord == Equal,
        crate::query::RelOp::Neq => ord != Equal,
        crate::query::RelOp::Gt => ord == Greater,
        crate::query::RelOp::Gte => ord != Less,
        crate::query::RelOp::Lt => ord == Less,
        crate::query::RelOp::Lte => ord != Greater,
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::node::{FieldValue, UnifiedNode};
    use crate::query::{PhysicalOperator, RelOp};

    fn node_with_field(key: &str, val: FieldValue) -> UnifiedNode {
        let mut node = UnifiedNode::new(1);
        node.relational.insert(key.into(), val);
        node
    }

    // ── evaluate_condition ──

    #[test]
    fn test_evaluate_condition_eq_string() {
        let node = node_with_field("name", FieldValue::String("alice".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Eq,
            &FieldValue::String("alice".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Eq,
            &FieldValue::String("bob".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_neq_string() {
        let node = node_with_field("name", FieldValue::String("alice".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Neq,
            &FieldValue::String("bob".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Neq,
            &FieldValue::String("alice".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_gt_int() {
        let node = node_with_field("age", FieldValue::Int(30));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(20)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(30)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(40)
        ));
    }

    #[test]
    fn test_evaluate_condition_gte_int() {
        let node = node_with_field("age", FieldValue::Int(30));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(30)
        ));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(20)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(40)
        ));
    }

    #[test]
    fn test_evaluate_condition_lt_float() {
        let node = node_with_field("price", FieldValue::Float(10.5));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lt,
            &FieldValue::Float(20.0)
        ));
        assert!(!evaluate_condition(
            &node,
            "price",
            &RelOp::Lt,
            &FieldValue::Float(5.0)
        ));
    }

    #[test]
    fn test_evaluate_condition_lte_float() {
        let node = node_with_field("price", FieldValue::Float(10.0));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lte,
            &FieldValue::Float(10.0)
        ));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lte,
            &FieldValue::Float(15.0)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_eq() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(evaluate_condition(
            &node,
            "active",
            &RelOp::Eq,
            &FieldValue::Bool(true)
        ));
        assert!(!evaluate_condition(
            &node,
            "active",
            &RelOp::Eq,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_neq() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(evaluate_condition(
            &node,
            "active",
            &RelOp::Neq,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_non_relational() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(!evaluate_condition(
            &node,
            "active",
            &RelOp::Gt,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_null_eq() {
        let node = node_with_field("empty", FieldValue::Null);
        assert!(evaluate_condition(
            &node,
            "empty",
            &RelOp::Eq,
            &FieldValue::Null
        ));
        assert!(!evaluate_condition(
            &node,
            "empty",
            &RelOp::Neq,
            &FieldValue::Null
        ));
    }

    #[test]
    fn test_evaluate_condition_missing_field_neq() {
        let node = UnifiedNode::new(1);
        assert!(evaluate_condition(
            &node,
            "missing",
            &RelOp::Neq,
            &FieldValue::String("x".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_missing_field_eq() {
        let node = UnifiedNode::new(1);
        assert!(!evaluate_condition(
            &node,
            "missing",
            &RelOp::Eq,
            &FieldValue::String("x".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_type_mismatch() {
        let node = node_with_field("val", FieldValue::Int(42));
        assert!(!evaluate_condition(
            &node,
            "val",
            &RelOp::Eq,
            &FieldValue::String("42".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_string_gte() {
        let node = node_with_field("name", FieldValue::String("banana".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("apple".into())
        ));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("banana".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("cherry".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_negative_float() {
        let node = node_with_field("balance", FieldValue::Float(-5.0));
        assert!(evaluate_condition(
            &node,
            "balance",
            &RelOp::Lt,
            &FieldValue::Float(0.0)
        ));
        assert!(evaluate_condition(
            &node,
            "balance",
            &RelOp::Eq,
            &FieldValue::Float(-5.0)
        ));
    }

    #[test]
    fn test_relop_ordering_consistent() {
        let a = FieldValue::Int(50);
        let test_values = [0i64, 50, 100];
        for &v in &test_values {
            let node = node_with_field("x", FieldValue::Int(v));
            let gt = evaluate_condition(&node, "x", &RelOp::Gt, &a);
            let lt = evaluate_condition(&node, "x", &RelOp::Lt, &a);
            if v == 50 {
                assert!(
                    !gt && !lt,
                    "Gt and Lt should both be false for equal v={}",
                    v
                );
            } else {
                assert_ne!(gt, lt, "Gt and Lt should be opposites for v={}", v);
            }
        }
    }

    // ── PhysicalOperator trait object safety ──

    #[test]
    fn test_physical_operator_trait_satisfied() {
        fn _is_send_sync<T: Send + Sync>() {}
        _is_send_sync::<PhysicalFilter>();
        _is_send_sync::<PhysicalProject>();
        _is_send_sync::<PhysicalLimit>();
        _is_send_sync::<PhysicalOffset>();
        _is_send_sync::<PhysicalSort>();
    }

    // ── MockScan helper — controlled child operator ──────────────────────

    /// Mock operator that yields a fixed set of nodes for testing parent operators.
    struct MockScan {
        nodes: Vec<UnifiedNode>,
        saved: Vec<UnifiedNode>,
        cursor: usize,
    }

    impl MockScan {
        fn new(nodes: Vec<UnifiedNode>) -> Self {
            let saved = nodes.clone();
            Self {
                nodes,
                saved,
                cursor: 0,
            }
        }
    }

    impl PhysicalOperator for MockScan {
        fn open(&mut self) -> Result<()> {
            self.nodes = self.saved.clone();
            self.cursor = 0;
            Ok(())
        }

        fn next(&mut self) -> Result<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                Ok(Some(node))
            } else {
                Ok(None)
            }
        }

        fn close(&mut self) -> Result<()> {
            self.nodes.clear();
            Ok(())
        }
    }

    fn bool_node(id: u128, active: bool) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("active".into(), FieldValue::Bool(active));
        node
    }

    fn int_node(id: u128, val: i64) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational.insert("val".into(), FieldValue::Int(val));
        node
    }

    fn string_node(id: u128, name: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("name".into(), FieldValue::String(name.into()));
        node
    }

    // ── PhysicalFilter ──────────────────────────────────────────────────

    #[test]
    fn test_physical_filter_matches() {
        let child = MockScan::new(vec![
            bool_node(1, true),
            bool_node(2, false),
            bool_node(3, true),
        ]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        let r1 = filter.next().unwrap().expect("node 1 matches");
        assert_eq!(r1.id, 1);
        let r2 = filter.next().unwrap().expect("node 3 matches");
        assert_eq!(r2.id, 3);
        assert!(filter.next().unwrap().is_none(), "no more matches");
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_no_match() {
        let child = MockScan::new(vec![bool_node(1, false), bool_node(2, false)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_none(), "no nodes match");
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_empty_child() {
        let child = MockScan::new(vec![]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_neq() {
        let child = MockScan::new(vec![string_node(1, "alice"), string_node(2, "bob")]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "name".into(),
            RelOp::Neq,
            FieldValue::String("bob".into()),
        );
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 1, "alice != bob");
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_int_gt() {
        let child = MockScan::new(vec![int_node(1, 10), int_node(2, 20), int_node(3, 30)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "val".into(),
            RelOp::Gt,
            FieldValue::Int(15),
        );
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 2);
        assert_eq!(filter.next().unwrap().unwrap().id, 3);
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_open_close_cycle() {
        let child = MockScan::new(vec![bool_node(1, true)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_some());
        filter.close().unwrap();
        // Can re-open
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_some());
        filter.close().unwrap();
    }

    // ── PhysicalProject ─────────────────────────────────────────────────

    #[test]
    fn test_physical_project_narrows_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("a".into(), FieldValue::Int(1));
        node.relational
            .insert("b".into(), FieldValue::String("x".into()));
        node.relational.insert("c".into(), FieldValue::Float(3.0));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["a".into(), "c".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got projected node");
        assert_eq!(result.relational.len(), 2, "only 2 fields retained");
        assert!(result.relational.contains_key("a"));
        assert!(result.relational.contains_key("c"));
        assert!(!result.relational.contains_key("b"), "b removed");
        assert!(project.next().unwrap().is_none());
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_empty_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("a".into(), FieldValue::Int(1));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec![]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got node with no fields");
        assert!(result.relational.is_empty(), "all fields removed");
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_preserves_only_requested_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("keep".into(), FieldValue::Int(42));
        node.relational
            .insert("drop".into(), FieldValue::String("gone".into()));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["keep".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().unwrap();
        assert_eq!(result.relational.get("keep"), Some(&FieldValue::Int(42)));
        assert!(!result.relational.contains_key("drop"));
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_empty_child() {
        let child = MockScan::new(vec![]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["a".into()]);
        project.open().unwrap();
        assert!(project.next().unwrap().is_none());
        project.close().unwrap();
    }

    /// FIND-552: the projection must keep the **system metadata** that the
    /// post-plan filters read *after* the plan has run (valid-time `AS OF`,
    /// the low-confidence `SemanticSummary` filter and the `ROLE` prune) —
    /// while still dropping the user fields the query did not ask for.
    #[test]
    fn test_physical_project_keeps_system_metadata_but_drops_user_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational
            .insert("name".into(), FieldValue::String("n".into()));
        node.relational
            .insert("type".into(), FieldValue::String("Doc".into()));
        node.relational
            .insert("secret".into(), FieldValue::String("s".into()));
        node.relational.insert(
            crate::sdk::serialization::FIELD_VALID_AT_MS.into(),
            FieldValue::Int(1000),
        );
        node.relational.insert(
            crate::sdk::serialization::FIELD_INVALID_AT_MS.into(),
            FieldValue::Int(2000),
        );
        node.relational.insert(
            crate::sdk::serialization::FIELD_CREATED_AT_MS.into(),
            FieldValue::Int(500),
        );
        node.relational
            .insert("_owner_role".into(), FieldValue::String("admin".into()));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["name".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got projected node");

        // Requested field kept, unrequested user field dropped.
        assert_eq!(
            result.relational.get("name"),
            Some(&FieldValue::String("n".into()))
        );
        assert!(
            !result.relational.contains_key("secret"),
            "user fields the query did not ask for must be dropped"
        );
        // System metadata the post-plan filters depend on must survive.
        for field in [
            "type",
            "_owner_role",
            crate::sdk::serialization::FIELD_VALID_AT_MS,
            crate::sdk::serialization::FIELD_INVALID_AT_MS,
            crate::sdk::serialization::FIELD_CREATED_AT_MS,
        ] {
            assert!(
                result.relational.contains_key(field),
                "system metadata `{field}` must survive the projection"
            );
        }
        project.close().unwrap();
    }

    // ── PhysicalLimit ───────────────────────────────────────────────────

    #[test]
    fn test_physical_limit_caps_results() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2), int_node(3, 3)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 2);
        limit.open().unwrap();
        assert_eq!(limit.next().unwrap().unwrap().id, 1);
        assert_eq!(limit.next().unwrap().unwrap().id, 2);
        assert!(limit.next().unwrap().is_none(), "limit stops at 2");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_zero() {
        let child = MockScan::new(vec![int_node(1, 1)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 0);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_none(), "zero limit yields nothing");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_exact_count() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 2);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_some());
        assert!(limit.next().unwrap().is_some());
        assert!(limit.next().unwrap().is_none());
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_more_than_available() {
        let child = MockScan::new(vec![int_node(1, 1)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 10);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_some(), "one node available");
        assert!(limit.next().unwrap().is_none(), "no more despite limit=10");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_empty_child() {
        let child = MockScan::new(vec![]);
        let mut limit = PhysicalLimit::new(Box::new(child), 5);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_none());
        limit.close().unwrap();
    }

    // ── PhysicalOffset (WIRE-12) ────────────────────────────────────────

    #[test]
    fn test_physical_offset_skips_leading_rows() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2), int_node(3, 3)]);
        let mut offset = PhysicalOffset::new(Box::new(child), 2);
        offset.open().unwrap();
        assert_eq!(offset.next().unwrap().unwrap().id, 3, "first two skipped");
        assert!(offset.next().unwrap().is_none());
        offset.close().unwrap();
    }

    #[test]
    fn test_physical_offset_zero_passes_all() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2)]);
        let mut offset = PhysicalOffset::new(Box::new(child), 0);
        offset.open().unwrap();
        assert_eq!(offset.next().unwrap().unwrap().id, 1);
        assert_eq!(offset.next().unwrap().unwrap().id, 2);
        assert!(offset.next().unwrap().is_none());
        offset.close().unwrap();
    }

    #[test]
    fn test_physical_offset_more_than_available() {
        let child = MockScan::new(vec![int_node(1, 1)]);
        let mut offset = PhysicalOffset::new(Box::new(child), 10);
        offset.open().unwrap();
        assert!(
            offset.next().unwrap().is_none(),
            "skip beyond stream is empty"
        );
        offset.close().unwrap();
    }

    #[test]
    fn test_physical_offset_empty_child() {
        let child = MockScan::new(vec![]);
        let mut offset = PhysicalOffset::new(Box::new(child), 3);
        offset.open().unwrap();
        assert!(offset.next().unwrap().is_none());
        offset.close().unwrap();
    }

    #[test]
    fn test_physical_offset_open_close_cycle() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2)]);
        let mut offset = PhysicalOffset::new(Box::new(child), 1);
        offset.open().unwrap();
        assert_eq!(offset.next().unwrap().unwrap().id, 2);
        offset.close().unwrap();
        // Re-open resets the skip counter.
        offset.open().unwrap();
        assert_eq!(offset.next().unwrap().unwrap().id, 2);
        offset.close().unwrap();
    }

    // ── PhysicalSort ────────────────────────────────────────────────────

    #[test]
    fn test_physical_sort_ascending() {
        let child = MockScan::new(vec![int_node(3, 30), int_node(1, 10), int_node(2, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "lowest first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "highest last");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_descending() {
        let child = MockScan::new(vec![int_node(3, 30), int_node(1, 10), int_node(2, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), true);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "highest first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "lowest last");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_by_string() {
        let child = MockScan::new(vec![
            string_node(2, "banana"),
            string_node(1, "apple"),
            string_node(3, "cherry"),
        ]);
        let mut sort = PhysicalSort::new(Box::new(child), "name".into(), false);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "apple first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2, "banana second");
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "cherry third");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_missing_field_none_first_asc() {
        let mut n1 = UnifiedNode::new(1);
        n1.relational.insert("val".into(), FieldValue::Int(10));
        let mut n2 = UnifiedNode::new(2);
        n2.relational.insert("x".into(), FieldValue::Int(99));
        let child = MockScan::new(vec![n1, n2]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        // Nodes without the field sort first (None < Some)
        assert_eq!(
            sort.next().unwrap().unwrap().id,
            2,
            "node2 lacks 'val' → first"
        );
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "node1 has val=10");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_empty_child() {
        let child = MockScan::new(vec![]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_open_close_cycle() {
        let child = MockScan::new(vec![int_node(2, 20), int_node(1, 10)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), true);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 1);
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_deterministic_within_equal_keys() {
        let child = MockScan::new(vec![int_node(2, 30), int_node(1, 10), int_node(3, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        let ids: Vec<u128> = std::iter::from_fn(|| sort.next().unwrap().map(|n| n.id)).collect();
        assert_eq!(ids, vec![1, 3, 2], "sorted by val ascending → id order");
        sort.close().unwrap();
    }

    // ── PhysicalVectorRefine (passthrough when no embedding) ────────────

    /// Embedding seam standing in for "this build has no embedding capability"
    /// (neither `remote-inference` nor `embed-local`): the vector arm stays
    /// empty, so the refine passes the child rows through unfiltered.
    ///
    /// Injecting it keeps the premise of these tests true in EVERY feature
    /// combination — without it, a build with `remote-inference` would embed
    /// the query (and fail, or filter) instead of leaving the arm empty.
    fn no_embedding_capability() -> QueryEmbedder {
        std::sync::Arc::new(|_text: &str| -> Result<Option<Vec<f32>>> { Ok(None) })
    }

    #[test]
    fn test_physical_vector_refine_passthrough_no_embedding() {
        // Without embedding capability, query_vector stays None
        // so next() passes through all child nodes unfiltered.
        let child = MockScan::new(vec![bool_node(1, true), bool_node(2, false)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5)
            .with_embedder(no_embedding_capability());
        refine.open().unwrap();
        let r1 = refine.next().unwrap().expect("passthrough node 1");
        assert_eq!(r1.id, 1);
        let r2 = refine.next().unwrap().expect("passthrough node 2");
        assert_eq!(r2.id, 2);
        assert!(refine.next().unwrap().is_none());
        refine.close().unwrap();
    }

    #[test]
    fn test_physical_vector_refine_empty_child() {
        let child = MockScan::new(vec![]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5)
            .with_embedder(no_embedding_capability());
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_none());
        refine.close().unwrap();
    }

    #[test]
    fn test_physical_vector_refine_open_close_cycle() {
        let child = MockScan::new(vec![bool_node(1, true)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5)
            .with_embedder(no_embedding_capability());
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
        // Re-open
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
    }

    // ── FIND-556: alias-qualified field references ──────────────────────

    /// `p.name` is a valid ident (the lexer admits `.`), the parser keeps
    /// the qualifier, and stored relational keys are never qualified — the
    /// lookup must strip the alias like the JOIN/subquery paths already do.
    #[test]
    fn test_evaluate_condition_alias_qualified_field() {
        let node = node_with_field("name", FieldValue::String("alice".into()));
        assert!(evaluate_condition(
            &node,
            "p.name",
            &RelOp::Eq,
            &FieldValue::String("alice".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "p.name",
            &RelOp::Eq,
            &FieldValue::String("bob".into())
        ));
    }

    #[test]
    fn test_physical_filter_alias_qualified_field() {
        let child = MockScan::new(vec![string_node(1, "alice"), string_node(2, "bob")]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "p.name".into(),
            RelOp::Eq,
            FieldValue::String("alice".into()),
        );
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 1, "alice matches");
        assert!(filter.next().unwrap().is_none(), "bob does not");
        filter.close().unwrap();
    }

    /// FIND-556 collateral: `SELECT p.name`/`FETCH p.name` reached the
    /// projection qualified — `remove("p.name")` found nothing and the row
    /// went out with zero fields.
    #[test]
    fn test_physical_project_alias_qualified_field() {
        let mut node = UnifiedNode::new(1);
        node.relational
            .insert("name".into(), FieldValue::String("n".into()));
        node.relational.insert("price".into(), FieldValue::Int(5));

        let child = MockScan::new(vec![node]);
        let mut project =
            PhysicalProject::new(Box::new(child), vec!["p.name".into(), "sq.price".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got projected node");
        assert_eq!(
            result.relational.get("name"),
            Some(&FieldValue::String("n".into())),
            "qualified projection resolves the stored unqualified key"
        );
        assert_eq!(result.relational.get("price"), Some(&FieldValue::Int(5)));
        project.close().unwrap();
    }

    // ── FIND-157: mixed Int/Float comparison ────────────────────────────

    /// Stored `Float` vs literal `Int` (and vice versa): both variants are
    /// numeric, so they compare in the real-number domain — the literal
    /// `28` parses as `Int` while the SDK stores `Float`, and both must
    /// match instead of hitting the "different variant ⇒ false" catch-all.
    #[test]
    fn test_evaluate_condition_float_field_int_literal() {
        let node = node_with_field("price", FieldValue::Float(28.0));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Eq,
            &FieldValue::Int(28)
        ));
        assert!(!evaluate_condition(
            &node,
            "price",
            &RelOp::Neq,
            &FieldValue::Int(28)
        ));
        assert!(!evaluate_condition(
            &node,
            "price",
            &RelOp::Eq,
            &FieldValue::Int(29)
        ));
    }

    #[test]
    fn test_evaluate_condition_int_field_float_literal() {
        let node = node_with_field("price", FieldValue::Int(28));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Eq,
            &FieldValue::Float(28.0)
        ));
        assert!(!evaluate_condition(
            &node,
            "price",
            &RelOp::Eq,
            &FieldValue::Float(28.5)
        ));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Neq,
            &FieldValue::Float(28.5)
        ));
    }

    #[test]
    fn test_evaluate_condition_cross_type_ordering() {
        let int_node = node_with_field("v", FieldValue::Int(10));
        assert!(evaluate_condition(
            &int_node,
            "v",
            &RelOp::Gt,
            &FieldValue::Float(9.5)
        ));
        assert!(evaluate_condition(
            &int_node,
            "v",
            &RelOp::Lt,
            &FieldValue::Float(10.5)
        ));

        let float_node = node_with_field("v", FieldValue::Float(10.5));
        assert!(evaluate_condition(
            &float_node,
            "v",
            &RelOp::Gte,
            &FieldValue::Int(10)
        ));
        assert!(evaluate_condition(
            &float_node,
            "v",
            &RelOp::Gt,
            &FieldValue::Int(10)
        ));
        assert!(!evaluate_condition(
            &float_node,
            "v",
            &RelOp::Lte,
            &FieldValue::Int(10)
        ));
    }

    /// `i64 as f64` rounds above 2^53: an exact comparison must NOT declare
    /// `Int(2^53+1)` equal to `Float(2^53)` — that would reintroduce the
    /// silent-wrong-result class this fix removes.
    #[test]
    fn test_evaluate_condition_cross_type_i64_precision() {
        let node = node_with_field("v", FieldValue::Int(9_007_199_254_740_993)); // 2^53 + 1
        assert!(!evaluate_condition(
            &node,
            "v",
            &RelOp::Eq,
            &FieldValue::Float(9_007_199_254_740_992.0) // 2^53
        ));
        assert!(evaluate_condition(
            &node,
            "v",
            &RelOp::Gt,
            &FieldValue::Float(9_007_199_254_740_992.0)
        ));
        assert!(evaluate_condition(
            &node,
            "v",
            &RelOp::Neq,
            &FieldValue::Float(9_007_199_254_740_992.0)
        ));
    }

    // ── FIND-557: empty scalar subquery yields zero rows ────────────────

    /// `x = (SELECT ...)` over an empty subquery is a comparison against
    /// NULL: false for every row, for every operator — the predicate must
    /// not silently disappear (0 rows, not "all rows pass through").
    #[test]
    fn test_subquery_filter_empty_subquery_yields_no_rows() {
        let child = MockScan::new(vec![int_node(1, 5), int_node(2, 10)]);
        let subq = MockScan::new(vec![]);
        let mut filter =
            PhysicalSubqueryFilter::new(Box::new(child), Box::new(subq), "val".into(), RelOp::Eq);
        filter.open().unwrap();
        assert!(
            filter.next().unwrap().is_none(),
            "empty scalar subquery ⇒ no row matches (FIND-557)"
        );
        filter.close().unwrap();
    }

    #[test]
    fn test_subquery_filter_empty_subquery_neq_yields_no_rows() {
        let child = MockScan::new(vec![int_node(1, 5), int_node(2, 10)]);
        let subq = MockScan::new(vec![]);
        let mut filter =
            PhysicalSubqueryFilter::new(Box::new(child), Box::new(subq), "val".into(), RelOp::Neq);
        filter.open().unwrap();
        assert!(
            filter.next().unwrap().is_none(),
            "`!= NULL` is still not-true ⇒ 0 rows (SQL NULL semantics)"
        );
        filter.close().unwrap();
    }

    /// Cross-type scalar: a `Float` subquery scalar compared against an
    /// `Int` field (FIND-157 reaching `compare_field_values` via the
    /// subquery path).
    #[test]
    fn test_subquery_filter_float_scalar_vs_int_field() {
        let mut subq_node = UnifiedNode::new(100);
        subq_node
            .relational
            .insert("score".into(), FieldValue::Float(10.0));
        let subq = MockScan::new(vec![subq_node]);
        let child = MockScan::new(vec![int_node(1, 5), int_node(2, 10), int_node(3, 20)]);
        let mut filter =
            PhysicalSubqueryFilter::new(Box::new(child), Box::new(subq), "val".into(), RelOp::Gte);
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 2);
        assert_eq!(filter.next().unwrap().unwrap().id, 3);
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    // ── Open/Close lifecycle on every operator ──────────────────────────

    #[test]
    fn test_all_operators_support_close_without_open() {
        // Verify calling close() on a fresh operator does not panic.
        let mut f = PhysicalFilter::new(
            Box::new(MockScan::new(vec![])),
            "x".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        f.close().unwrap();

        let mut p = PhysicalProject::new(Box::new(MockScan::new(vec![])), vec![]);
        p.close().unwrap();

        let mut l = PhysicalLimit::new(Box::new(MockScan::new(vec![])), 0);
        l.close().unwrap();

        let mut o = PhysicalOffset::new(Box::new(MockScan::new(vec![])), 3);
        o.close().unwrap();

        let mut s = PhysicalSort::new(Box::new(MockScan::new(vec![])), "x".into(), false);
        s.close().unwrap();

        let mut r = PhysicalVectorRefine::new(Box::new(MockScan::new(vec![])), "q".into(), 0.0);
        r.close().unwrap();
    }
}
