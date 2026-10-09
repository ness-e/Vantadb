//! Physical offset operator: skips the first `skip` rows of its child stream.
//!
//! Split into its own file following the C2S6 extension pattern (WIRE-12):
//! `LogicalOperator::Offset` compiles through `OperatorRegistry` to this
//! operator, which the planner wraps post-chain (after `Limit`). The
//! `Limit`/`Offset` composition that yields skip-then-take lives in
//! `query.rs::push_pagination`.

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

/// Physical offset operator: consumes and discards the first `skip` rows,
/// then streams the rest of the child unchanged.
pub struct PhysicalOffset<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Number of leading rows to skip.
    skip: usize,
    /// Number of rows skipped so far.
    skipped: usize,
}

impl<'a> PhysicalOffset<'a> {
    /// Create a new offset operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, skip: usize) -> Self {
        Self {
            child,
            skip,
            skipped: 0,
        }
    }
}

impl PhysicalOperator for PhysicalOffset<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.skipped = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.skipped < self.skip {
            match self.child.next()? {
                Some(_) => self.skipped += 1,
                None => return Ok(None),
            }
        }
        self.child.next()
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}
