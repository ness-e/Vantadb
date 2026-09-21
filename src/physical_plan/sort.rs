//! Physical sort operator.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

// ─── Physical Sort Operator ──────────────────────────────────

/// Physical sort operator that sorts nodes by a relational field.
pub struct PhysicalSort<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Sort field.
    field: String,
    /// Sort descending.
    desc: bool,
    /// Buffered nodes to sort.
    nodes: Vec<UnifiedNode>,
    /// Current position in sorted nodes.
    cursor: usize,
}

impl<'a> PhysicalSort<'a> {
    /// Create a new sort operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, field: String, desc: bool) -> Self {
        Self {
            child,
            field,
            desc,
            nodes: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalSort<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.nodes.clear();
        self.cursor = 0;

        while let Some(node) = self.child.next()? {
            self.nodes.push(node);
        }

        let field = &self.field;
        let desc = self.desc;
        self.nodes.sort_by(|a, b| {
            let a_val = a.relational.get(field);
            let b_val = b.relational.get(field);
            let cmp = match (a_val, b_val) {
                (
                    Some(crate::node::FieldValue::String(av)),
                    Some(crate::node::FieldValue::String(bv)),
                ) => av.cmp(bv),
                (
                    Some(crate::node::FieldValue::Int(av)),
                    Some(crate::node::FieldValue::Int(bv)),
                ) => av.cmp(bv),
                (
                    Some(crate::node::FieldValue::Float(av)),
                    Some(crate::node::FieldValue::Float(bv)),
                ) => av.partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal),
                (
                    Some(crate::node::FieldValue::Bool(av)),
                    Some(crate::node::FieldValue::Bool(bv)),
                ) => av.cmp(bv),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Equal,
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });

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
        self.nodes.clear();
        self.child.close()
    }
}
