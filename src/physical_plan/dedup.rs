//! Physical dedup operator (C2S6 extension exemplar).
//!
//! Unary in-memory Volcano wrapper: drops rows whose `field` value was
//! already emitted. Proves a new operator ships as new files + wiring in
//! `OperatorRegistry`, with zero edits to the proven `planner` / `executor`
//! matches (dispatch by name in the registry).
//!
//! Leaf note: this file depends only on `query` / `node` / `error` — the
//! registry compiler + cost model live in `operator_registry.rs`, so the
//! dependency runs one way (`registry → physical_plan`, never back).

use std::collections::HashSet;

use crate::error::Result;
use crate::node::{FieldValue, UnifiedNode};
use crate::query::PhysicalOperator;

// ─── Physical Dedup Operator ─────────────────────────────────────

/// Physical dedup operator: first-seen value of `field` wins.
///
/// Missing-field rows share a single `None` key (they collapse to one row).
/// `FieldValue` is `Hash + Eq` (`node/field.rs`), so the key set is exact —
/// no string serialization, no manual `Hash` impl.
pub struct PhysicalDedup<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Field whose first-seen value wins.
    field: String,
    /// Values already emitted (`None` = row missing the field).
    seen: HashSet<Option<FieldValue>>,
}

impl<'a> PhysicalDedup<'a> {
    /// Create a new dedup operator over `child` keyed by `field`.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, field: String) -> Self {
        Self {
            child,
            field,
            seen: HashSet::new(),
        }
    }
}

impl PhysicalOperator for PhysicalDedup<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.seen.clear();
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while let Some(node) = self.child.next()? {
            let key: Option<FieldValue> = node.relational.get(&self.field).cloned();
            if self.seen.insert(key) {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.seen.clear();
        self.child.close()
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::error::Result as CrateResult;

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
        fn open(&mut self) -> CrateResult<()> {
            self.nodes = self.saved.clone();
            self.cursor = 0;
            Ok(())
        }
        fn next(&mut self) -> CrateResult<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                Ok(Some(node))
            } else {
                Ok(None)
            }
        }
        fn close(&mut self) -> CrateResult<()> {
            self.nodes.clear();
            Ok(())
        }
    }

    fn named_node(id: u128, name: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("name".into(), FieldValue::String(name.into()));
        node
    }

    #[test]
    fn dedup_collapses_duplicate_field_values() {
        let child = MockScan::new(vec![
            named_node(1, "alice"),
            named_node(2, "bob"),
            named_node(3, "alice"),
        ]);
        let mut dedup = PhysicalDedup::new(Box::new(child), "name".into());
        dedup.open().unwrap();
        assert_eq!(dedup.next().unwrap().unwrap().id, 1);
        assert_eq!(dedup.next().unwrap().unwrap().id, 2);
        assert!(dedup.next().unwrap().is_none(), "second alice dropped");
        dedup.close().unwrap();
    }

    #[test]
    fn dedup_missing_field_collapses_to_one_row() {
        let mut stray = UnifiedNode::new(9);
        stray.relational.insert("other".into(), FieldValue::Int(1));
        let child = MockScan::new(vec![named_node(1, "alice"), stray]);
        let mut dedup = PhysicalDedup::new(Box::new(child), "name".into());
        dedup.open().unwrap();
        assert_eq!(dedup.next().unwrap().unwrap().id, 1);
        assert_eq!(
            dedup.next().unwrap().unwrap().id,
            9,
            "first missing-field row passes"
        );
        assert!(dedup.next().unwrap().is_none());
        dedup.close().unwrap();
    }

    #[test]
    fn dedup_reopen_resets_seen() {
        let child = MockScan::new(vec![named_node(1, "alice"), named_node(2, "alice")]);
        let mut dedup = PhysicalDedup::new(Box::new(child), "name".into());
        dedup.open().unwrap();
        assert!(dedup.next().unwrap().is_some());
        assert!(dedup.next().unwrap().is_none());
        dedup.close().unwrap();
        dedup.open().unwrap();
        assert_eq!(
            dedup.next().unwrap().unwrap().id,
            1,
            "seen resets on reopen"
        );
        dedup.close().unwrap();
    }
}
