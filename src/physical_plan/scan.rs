//! Physical scan operator: iterates all nodes of a given entity type.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;

// ─── Physical Scan Operator ──────────────────────────────────

/// Physical scan operator that iterates all nodes of a given entity type.
pub struct PhysicalScan<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Entity type to scan.
    entity: String,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
}

impl<'a> PhysicalScan<'a> {
    /// Create a new scan operator for the given entity.
    pub fn new(storage: &'a StorageEngine, entity: String) -> Self {
        Self {
            storage,
            entity,
            prefetched: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalScan<'_> {
    fn open(&mut self) -> Result<()> {
        self.prefetched.clear();
        self.cursor = 0;

        let parts: Vec<&str> = self.entity.split('#').collect();
        if parts.len() == 2 {
            if let Ok(id) = parts[1].parse::<u128>() {
                self.prefetched = self.storage.get_many(&[id])?;
                return Ok(());
            }
        }

        let records = self
            .storage
            .backend
            .scan(crate::backend::BackendPartition::Default)?;
        let ids: Vec<u128> = records
            .iter()
            .filter_map(|(key_bytes, _)| {
                let arr: [u8; 16] = key_bytes.as_slice().try_into().ok()?;
                Some(u128::from_le_bytes(arr))
            })
            .collect();
        self.prefetched = self.storage.get_many(&ids)?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.cursor < self.prefetched.len() {
            let node = &self.prefetched[self.cursor];
            self.cursor += 1;

            if self.storage.is_deleted(node.id)? {
                continue;
            }

            if self.entity.contains('#') || self.entity == "*" {
                return Ok(Some(node.clone()));
            }
            if let Some(crate::node::FieldValue::String(t)) = node.relational.get("type") {
                if t == &self.entity {
                    return Ok(Some(node.clone()));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.prefetched.clear();
        Ok(())
    }
}
