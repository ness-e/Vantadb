//! Physical query plan operators executed against storage.
//!
//! [`PhysicalScan`] and related operators translate logical plan nodes
//! into concrete storage reads, filtering, and projection.
//!
//! Split into per-operator submodules (REVIEW-05): [`scan`], [`filter`],
//! [`vector`], [`project`], [`sort`], [`join`].

mod filter;
mod join;
mod project;
mod scan;
mod sort;
mod vector;

pub use filter::{PhysicalFilter, PhysicalTextFilter};
pub use join::{PhysicalNestedLoopJoin, PhysicalSubqueryFilter};
pub use project::{PhysicalLimit, PhysicalProject};
pub use scan::PhysicalScan;
pub use sort::PhysicalSort;
pub use vector::{PhysicalVectorRefine, PhysicalVectorSearch};

#[cfg(test)]
pub(crate) use filter::evaluate_condition;

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

    #[test]
    fn test_physical_vector_refine_passthrough_no_embedding() {
        // Without `remote-inference` feature, query_vector stays None
        // so next() passes through all child nodes unfiltered.
        let child = MockScan::new(vec![bool_node(1, true), bool_node(2, false)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
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
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_none());
        refine.close().unwrap();
    }

    #[test]
    fn test_physical_vector_refine_open_close_cycle() {
        let child = MockScan::new(vec![bool_node(1, true)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
        // Re-open
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
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

        let mut s = PhysicalSort::new(Box::new(MockScan::new(vec![])), "x".into(), false);
        s.close().unwrap();

        let mut r = PhysicalVectorRefine::new(Box::new(MockScan::new(vec![])), "q".into(), 0.0);
        r.close().unwrap();
    }
}
