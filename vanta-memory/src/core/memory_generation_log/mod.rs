//! Generation-log provenance for the memory pipeline (MEM-41).
//!
//! Every successful or failed generation at layers L1/L2/L3 appends one
//! [`GenerationLogEntry`] under the `genlog/<session>` namespace so provenance
//! is consultable per session/layer ordered by timestamp.
//!
//! Principio 4 (never blocks): all writes go through
//! [`record_best_effort`], which swallows store errors with a
//! `tracing::warn!` — a logging failure can never fail a memory generation.
//! Growth is capped per session (keep-recent) in [`store::try_record`].
//!
//! Design source: TDAM `core/memory-generation-log/{types,store,best-effort}.ts`
//! (277L), reduced to the fields the contract requires
//! ({layer, status, anchor_id, session, ts} + optional error).

pub mod store;

pub use store::{
    genlog_namespace, query_session, record_best_effort, try_record, MAX_ENTRIES_PER_SESSION,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::conversation::now_ms;

/// Pipeline layer that produced (or failed to produce) a memory artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationLayer {
    /// L1 extracted memories.
    L1,
    /// L2 scene blocks.
    L2,
    /// L3 persona.
    L3,
}

/// Outcome of a generation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStatus {
    Succeeded,
    Failed,
}

/// One provenance entry: what was generated, where, when, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationLogEntry {
    pub layer: GenerationLayer,
    pub status: GenerationStatus,
    /// Primary artifact id when there is one (L1: record id); `None` when the
    /// artifact is session-unique (L2 scenes / L3 persona are keyed by session).
    pub anchor_id: Option<String>,
    pub session_key: String,
    pub ts_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl GenerationLogEntry {
    /// Build an entry stamped with the current wall clock.
    pub fn new(
        layer: GenerationLayer,
        status: GenerationStatus,
        session_key: &str,
        anchor_id: Option<&str>,
        error: Option<String>,
    ) -> Self {
        Self {
            layer,
            status,
            anchor_id: anchor_id.map(str::to_string),
            session_key: session_key.to_string(),
            ts_ms: now_ms(),
            error,
        }
    }
}

/// Errors surfaced by the generation-log query surface. Best-effort writes
/// never propagate these (see [`record_best_effort`]).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GenLogError {
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
    #[error("malformed generation log payload: {0}")]
    Serde(#[from] serde_json::Error),
}
