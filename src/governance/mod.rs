//! Governance primitives for StorageEngine.
//!
//! This module provides the data structures (`AdmissionFilter`, `ConsistencyBuffer`,
//! `ConflictResolver`) that are conditionally compiled into `StorageEngine` when
//! the `governance` feature is active. The full experimental logic (maintenance
//! workers, invalidation dispatch, etc.) lives in the `experimental-governance`
//! crate under `packages/`.

pub mod admission_filter;
pub mod conflict_resolver;
pub mod consistency;

pub use admission_filter::AdmissionFilter;
pub use conflict_resolver::{ConfidenceArbiter, ConflictResolver, ResolutionResult};
pub use consistency::{ConsistencyBuffer, ConsistencyRecord};

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
