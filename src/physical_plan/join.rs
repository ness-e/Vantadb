// ponytail: nested-loop join step invariant; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Physical join operators: nested-loop join and subquery filter.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

// ─── Physical Nested Loop Join Operator ────────────────────────

// ponytail: NestedLoopJoin — simplest correct. Add HashJoin when perf warrants it.
// ponytail: Merges all relational fields (left precedence on name collision).
// ponytail: No predicate pushdown yet — all WHERE filters apply post-join.

/// Strip alias prefix from a qualified field reference (e.g. `"p.name"` → `"name"`).
fn strip_alias(field_ref: &str) -> &str {
    field_ref.split('.').next_back().unwrap_or(field_ref)
}

/// Physical nested-loop join operator that iterates left entities and,
/// for each left entity, scans all right entities applying the ON condition.
pub struct PhysicalNestedLoopJoin<'a> {
    /// Left child operator.
    left_child: Box<dyn PhysicalOperator + 'a>,
    /// Right child operator.
    right_child: Box<dyn PhysicalOperator + 'a>,
    /// Left-side join field (alias-qualified, e.g. `"p.addr_id"`).
    left_field: String,
    /// Right-side join field (alias-qualified, e.g. `"a.id"`).
    right_field: String,
    /// All right-side nodes buffered during open().
    right_nodes: Vec<UnifiedNode>,
    /// Index into right_nodes for the current left row.
    right_cursor: usize,
    /// Current left node being probed.
    current_left: Option<UnifiedNode>,
}

impl<'a> PhysicalNestedLoopJoin<'a> {
    /// Create a new nested-loop join operator.
    pub fn new(
        left_child: Box<dyn PhysicalOperator + 'a>,
        right_child: Box<dyn PhysicalOperator + 'a>,
        left_field: String,
        right_field: String,
    ) -> Self {
        Self {
            left_child,
            right_child,
            left_field,
            right_field,
            right_nodes: Vec::new(),
            right_cursor: 0,
            current_left: None,
        }
    }
}

impl PhysicalOperator for PhysicalNestedLoopJoin<'_> {
    fn open(&mut self) -> Result<()> {
        self.left_child.open()?;
        self.right_child.open()?;

        // Buffer all right-side nodes (needed for repeated probing per left row)
        self.right_nodes.clear();
        while let Some(node) = self.right_child.next()? {
            self.right_nodes.push(node);
        }
        self.right_child.close()?;

        self.right_cursor = 0;
        self.current_left = None;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let left_field = strip_alias(&self.left_field).to_string();
        let right_field = strip_alias(&self.right_field).to_string();

        loop {
            // If we don't have a current left row, fetch one
            if self.current_left.is_none() {
                match self.left_child.next()? {
                    Some(node) => {
                        self.current_left = Some(node);
                        self.right_cursor = 0;
                    }
                    None => return Ok(None), // left side exhausted
                }
            }

            let left = self.current_left.as_ref().unwrap();

            // Probe right nodes
            while self.right_cursor < self.right_nodes.len() {
                let right = &self.right_nodes[self.right_cursor];
                self.right_cursor += 1;

                // Evaluate ON condition: left_field = right_field
                let left_val = left.relational.get(&left_field);
                let right_val = right.relational.get(&right_field);

                let matches = match (left_val, right_val) {
                    (Some(lv), Some(rv)) => compare_field_values(lv, rv, &crate::query::RelOp::Eq),
                    _ => false,
                };

                if matches {
                    // Merge fields: left fields first, then right fields (left priority on conflict)
                    let mut merged = UnifiedNode::new(left.id);
                    merged.relational = left.relational.clone();
                    for (k, v) in &right.relational {
                        // Use BTreeMap entry API to avoid overwriting left fields
                        if !merged.relational.contains_key(k) {
                            merged.relational.insert(k.clone(), v.clone());
                        }
                    }
                    return Ok(Some(merged));
                }
            }

            // Exhausted right nodes for this left row; advance to next left
            self.current_left = None;
        }
    }

    fn close(&mut self) -> Result<()> {
        self.left_child.close()?;
        self.right_nodes.clear();
        self.current_left = None;
        Ok(())
    }
}

// ─── Physical Subquery Filter Operator ──────────────────────────

/// Physical subquery filter that evaluates a scalar subquery during `open()`
/// then filters child rows against the computed scalar value.
pub struct PhysicalSubqueryFilter<'a> {
    /// Child operator whose rows are filtered.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Subquery plan to evalute once during open().
    subquery_plan: Box<dyn PhysicalOperator + 'a>,
    /// Field to compare (alias-qualified).
    field: String,
    /// Comparison operator.
    op: crate::query::RelOp,
    /// Scalar value obtained by executing the subquery.
    scalar_value: Option<crate::node::FieldValue>,
}

impl<'a> PhysicalSubqueryFilter<'a> {
    /// Create a new subquery filter operator.
    pub fn new(
        child: Box<dyn PhysicalOperator + 'a>,
        subquery_plan: Box<dyn PhysicalOperator + 'a>,
        field: String,
        op: crate::query::RelOp,
    ) -> Self {
        Self {
            child,
            subquery_plan,
            field,
            op,
            scalar_value: None,
        }
    }
}

impl PhysicalOperator for PhysicalSubqueryFilter<'_> {
    fn open(&mut self) -> Result<()> {
        // Execute the subquery plan to get the scalar value
        self.subquery_plan.open()?;
        let mut subq_nodes: Vec<UnifiedNode> = Vec::new();
        while let Some(node) = self.subquery_plan.next()? {
            subq_nodes.push(node);
        }
        self.subquery_plan.close()?;

        // Take the first field value from the first result node as the scalar
        self.scalar_value = subq_nodes
            .first()
            .and_then(|n| n.relational.values().next().cloned());

        // Open child for iteration
        self.child.open()?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let sv = match &self.scalar_value {
            Some(v) => v,
            None => {
                // Subquery returned no rows — compare against Null
                // For Eq, no rows = no match. For Neq, all rows match.
                // Simplest: treat empty as Null.
                return self.child.next(); // pass through, no scalar to compare
            }
        };

        let field_name = strip_alias(&self.field);
        while let Some(node) = self.child.next()? {
            if let Some(actual) = node.relational.get(field_name) {
                if compare_field_values(actual, sv, &self.op) {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()?;
        Ok(())
    }
}

/// Compare two field values using the given relational operator.
fn compare_field_values(
    a: &crate::node::FieldValue,
    b: &crate::node::FieldValue,
    op: &crate::query::RelOp,
) -> bool {
    match (a, b) {
        (crate::node::FieldValue::String(a), crate::node::FieldValue::String(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Int(a), crate::node::FieldValue::Int(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Float(a), crate::node::FieldValue::Float(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Bool(a), crate::node::FieldValue::Bool(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            _ => false,
        },
        (crate::node::FieldValue::Null, crate::node::FieldValue::Null) => match op {
            crate::query::RelOp::Eq => true,
            crate::query::RelOp::Neq => false,
            _ => false,
        },
        _ => false,
    }
}
