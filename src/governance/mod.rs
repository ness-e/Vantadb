pub mod invalidations;
pub mod maintenance_worker;
pub mod admission_filter;
pub mod consistency;
pub mod conflict_resolver;

pub use conflict_resolver::{ConflictResolver, ConfidenceArbiter};

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A permanent record of a node that has been logically deleted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditableTombstone {
    pub id: u64,
    pub timestamp_deleted: u64,
    pub reason: String,
    pub original_hash: u64,
}

impl AuditableTombstone {
    pub fn new(id: u64, reason: impl Into<String>, original_hash: u64) -> Self {
        let timestamp_deleted = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            timestamp_deleted,
            reason: reason.into(),
            original_hash,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Accept,
    Reject(String),
    Superposition(crate::governance::consistency::ConsistencyRecord),
    Merge { new_confidence: f32 },
}
