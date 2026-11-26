//! Physical filter operators: relational conditions and lexical text filters.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

// ─── Physical Filter Operator ────────────────────────────────

/// Physical filter operator that evaluates relational conditions.
pub struct PhysicalFilter<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Field to filter on.
    field: String,
    /// Comparison operator.
    op: crate::query::RelOp,
    /// Expected value.
    value: crate::node::FieldValue,
}

impl<'a> PhysicalFilter<'a> {
    /// Create a new filter operator wrapping a child operator.
    pub fn new(
        child: Box<dyn PhysicalOperator + 'a>,
        field: String,
        op: crate::query::RelOp,
        value: crate::node::FieldValue,
    ) -> Self {
        Self {
            child,
            field,
            op,
            value,
        }
    }
}

impl PhysicalOperator for PhysicalFilter<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while let Some(node) = self.child.next()? {
            if evaluate_condition(&node, &self.field, &self.op, &self.value) {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

pub(crate) fn evaluate_condition(
    node: &UnifiedNode,
    field: &str,
    op: &crate::query::RelOp,
    expected: &crate::node::FieldValue,
) -> bool {
    // FIND-556: parser-produced field refs may carry the table alias
    // (`p.name`) while stored keys never do — exact key first (a real
    // dotted key like `meta.env` wins), then the stripped form. The JOIN
    // and subquery paths already stripped unconditionally.
    let actual = node
        .relational
        .get(field)
        .or_else(|| node.relational.get(super::strip_alias(field)));

    if let Some(actual) = actual {
        match (actual, expected) {
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
            // FIND-157: both variants are numeric — compare in the real
            // domain via the exact `i64`↔`f64` ordering (no `as f64`
            // precision loss above 2^53). `None` (non-finite float) ⇒
            // false for every operator, including `Neq`.
            (crate::node::FieldValue::Int(a), crate::node::FieldValue::Float(e)) => {
                super::cmp_int_float(*a, *e)
                    .map(|ord| super::rel_op_matches(ord, op))
                    .unwrap_or(false)
            }
            (crate::node::FieldValue::Float(a), crate::node::FieldValue::Int(e)) => {
                super::cmp_int_float(*e, *a)
                    .map(|ord| super::rel_op_matches(ord.reverse(), op))
                    .unwrap_or(false)
            }
            _ => false,
        }
    } else {
        matches!(op, crate::query::RelOp::Neq)
    }
}

// ─── Physical Text Filter Operator ────────────────────────────

/// Physical text filter operator that evaluates lexical text conditions
/// (phrase-aware) against a node's string field.
pub struct PhysicalTextFilter<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Field to filter on.
    field: String,
    /// Text query (quoted phrases preserved).
    query: String,
}

impl<'a> PhysicalTextFilter<'a> {
    /// Create a new text filter operator wrapping a child operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, field: String, query: String) -> Self {
        Self {
            child,
            field,
            query,
        }
    }
}

impl PhysicalOperator for PhysicalTextFilter<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while let Some(node) = self.child.next()? {
            if let Some(crate::node::FieldValue::String(value)) = node.relational.get(&self.field) {
                if crate::text_index::text_contains_query(value, &self.query) {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}
