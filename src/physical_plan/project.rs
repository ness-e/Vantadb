//! Physical projection and limit operators.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;

/// Field holding the node type marker (`"Doc"`, `"SemanticSummary"`, ...).
pub(crate) const FIELD_TYPE: &str = "type";
/// Field holding the owner role (read by the `ROLE` prune).
pub(crate) const FIELD_OWNER_ROLE: &str = "_owner_role";

/// System metadata that survives the projection (FIND-552).
///
/// The post-plan filters — valid-time (`AS OF`, SCH-03 / ADR-046 §D3), the
/// low-confidence `SemanticSummary` filter and the `ROLE` prune — read these
/// fields **after** the physical plan has run, i.e. after `PhysicalProject`
/// narrowed the row. Without this allowlist the projection would delete the
/// very fields those filters depend on and they would silently degrade:
/// "no metadata ⇒ always valid" / "no `type` ⇒ not a summary" / "no
/// `_owner_role` ⇒ pruned".
///
/// Any new post-plan filter that reads a node field must add that field here.
///
/// `pub(crate)`: this is an internal contract between the projection and the
/// post-plan filters, not public API (`tests/api/public-api.txt` unchanged).
pub(crate) const SYSTEM_METADATA_FIELDS: &[&str] = &[
    crate::sdk::serialization::FIELD_VALID_AT_MS,
    crate::sdk::serialization::FIELD_INVALID_AT_MS,
    crate::sdk::serialization::FIELD_CREATED_AT_MS,
    FIELD_TYPE,
    FIELD_OWNER_ROLE,
];

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
            // FIND-552: the post-plan filters run *after* this operator, so
            // the system metadata they read must survive the projection.
            for field in SYSTEM_METADATA_FIELDS {
                if let Some(val) = node.relational.remove(*field) {
                    projected.insert((*field).to_string(), val);
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
