//! Physical projection and limit operators.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

// ─── Physical Project Operator ───────────────────────────────

/// Physical project operator that narrows fields on each node.
pub struct PhysicalProject<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Fields to retain.
    fields: Vec<String>,
}

impl<'a> PhysicalProject<'a> {
    /// Create a new project operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, fields: Vec<String>) -> Self {
        Self { child, fields }
    }
}

impl PhysicalOperator for PhysicalProject<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if let Some(mut node) = self.child.next()? {
            let mut projected = std::collections::BTreeMap::new();
            for field in &self.fields {
                if let Some(val) = node.relational.remove(field) {
                    projected.insert(field.clone(), val);
                }
            }
            node.relational = projected;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

// ─── Physical Limit Operator ─────────────────────────────────

/// Physical limit operator that caps the number of returned nodes.
pub struct PhysicalLimit<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Maximum number of rows.
    limit: usize,
    /// Number of rows emitted so far.
    count: usize,
}

impl<'a> PhysicalLimit<'a> {
    /// Create a new limit operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, limit: usize) -> Self {
        Self {
            child,
            limit,
            count: 0,
        }
    }
}

impl PhysicalOperator for PhysicalLimit<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.count = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if self.count >= self.limit {
            return Ok(None);
        }
        if let Some(node) = self.child.next()? {
            self.count += 1;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}
