//! Wiki lifecycle states (TDAM `wiki-service.ts:5-7` parity).
//!
//! State machine: `pending → processing → ready | failed`. A build failure
//! stores a truncated (`sync_error ≤500 chars`) reason. Re-ingest requests
//! are rejected while the wiki is `pending` or `processing` — the 409-busy
//! semantics of TDAM (`wiki-service.ts:272-288`) map to
//! [`VantaError::ExecutionConflict`] here.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::VantaError;

/// Lifecycle state of a wiki space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum WikiState {
    /// Created (or rebuild requested); the initial build is queued.
    Pending,
    /// A build is running (scanning / ingesting).
    Processing,
    /// Last build succeeded; the index is queryable.
    Ready,
    /// Last build failed; see `Wiki::sync_error`.
    Failed,
}

impl fmt::Display for WikiState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WikiState::Pending => "pending",
            WikiState::Processing => "processing",
            WikiState::Ready => "ready",
            WikiState::Failed => "failed",
        };
        f.write_str(s)
    }
}

impl WikiState {
    /// Whether a build is queued or running — further ingest requests must
    /// be rejected (TDAM busy semantics, wiki-service.ts:272-288).
    pub fn is_busy(self) -> bool {
        matches!(self, WikiState::Pending | WikiState::Processing)
    }

    /// Error for ingest attempts on a busy wiki (409-equivalent).
    pub fn busy_error(self, namespace: &str, slug: &str) -> VantaError {
        VantaError::ExecutionConflict {
            resource: format!("wiki:{namespace}:{slug}"),
            detail: format!(
                "ingest rejected while wiki is `{self}`; wait for the current build to finish"
            ),
        }
    }
}
