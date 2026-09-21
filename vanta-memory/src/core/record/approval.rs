//! MEM-68: optional capture-approval gate (Cursor pattern).
//!
//! Default off: [`should_gate`] returns `false` and hosts call
//! [`apply_dedup_batch`](super::l1_writer::apply_dedup_batch) directly —
//! byte-identical to pre-MEM-68 (never-block, gap #6 research). Gate on:
//! extracted memories wait in [`CaptureApprovalQueue`] until [`approve`]
//! (persists with default `Store` decisions) or [`reject`] (drops them).
//!
//! Scope: one in-memory queue (no threads, no persistence — pre-mortem #2).

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use vantadb::sdk::Embedded;

use crate::core::abstractions::{ExtractedMemory, MemoryRecord};
use crate::core::record::l1_writer::{apply_dedup_batch, EmbedFn, L1Error};

/// Gate switch. Default off so existing captures never block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureApprovalConfig {
    /// When `true`, captures wait in [`CaptureApprovalQueue`] for approve/reject.
    pub enabled: bool,
}

/// Whether captures must wait for approval instead of persisting directly.
pub fn should_gate(config: &CaptureApprovalConfig) -> bool {
    config.enabled
}

/// One gated capture awaiting an approve/reject decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PendingCapture {
    /// Queue id (`p_{created_at_ms}_{seq}`).
    pub id: String,
    /// L1 session key the memories belong to.
    pub session_key: String,
    /// Source conversation instance id.
    pub session_id: String,
    /// Extracted memories held back from the store.
    pub memories: Vec<ExtractedMemory>,
    /// Queue arrival time (unix ms, caller clock).
    pub created_at_ms: u64,
}

/// Errors surfaced by the approval queue.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApprovalError {
    /// No pending capture with that id (already decided or never queued).
    #[error("pending capture not found: {0}")]
    NotFound(String),
    /// The approve-time store write failed.
    #[error("approval store: {0}")]
    Store(#[from] L1Error),
}

/// In-memory pending queue. Cheap to share: wrap in `Arc` if several owners
/// need it.
#[derive(Debug, Default)]
pub struct CaptureApprovalQueue {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    pending: Vec<PendingCapture>,
    next_seq: u64,
}

impl CaptureApprovalQueue {
    /// Empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // Same poison policy as `LocalStateBackend`: plain data, recover.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Hold back extracted memories for approval. Returns the pending id.
    pub fn submit(
        &self,
        session_key: &str,
        session_id: &str,
        memories: Vec<ExtractedMemory>,
        now_ms: u64,
    ) -> String {
        let mut inner = self.lock();
        let id = format!("p_{now_ms}_{}", inner.next_seq);
        inner.next_seq += 1;
        inner.pending.push(PendingCapture {
            id: id.clone(),
            session_key: session_key.to_string(),
            session_id: session_id.to_string(),
            memories,
            created_at_ms: now_ms,
        });
        id
    }

    /// Snapshot of captures still awaiting a decision (submission order).
    pub fn list_pending(&self) -> Vec<PendingCapture> {
        self.lock().pending.clone()
    }

    /// Captures awaiting a decision.
    pub fn pending_count(&self) -> usize {
        self.lock().pending.len()
    }

    /// Approve a pending capture: persist its memories with default `Store`
    /// decisions and drop it from the queue. `now_ms` seeds record ids and
    /// timestamps; `embed` is the optional MEM-46 embedding hook.
    pub fn approve(
        &self,
        db: &Embedded,
        id: &str,
        now_ms: u64,
        embed: Option<&EmbedFn>,
    ) -> Result<Vec<MemoryRecord>, ApprovalError> {
        let capture = self
            .lock()
            .pending
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| ApprovalError::NotFound(id.to_string()))?;
        // Empty decisions → `apply_dedup_batch` stores every memory (its
        // documented default), so approve never merges or skips silently.
        let written = apply_dedup_batch(
            db,
            &capture.session_key,
            &capture.session_id,
            &capture.memories,
            &[],
            now_ms,
            embed,
        )?;
        self.lock().pending.retain(|p| p.id != id);
        Ok(written)
    }

    /// Reject a pending capture: drop it without persisting. Returns `false`
    /// when the id was unknown (idempotent — a second reject is a no-op).
    pub fn reject(&self, id: &str) -> bool {
        let mut inner = self.lock();
        let before = inner.pending.len();
        inner.pending.retain(|p| p.id != id);
        inner.pending.len() != before
    }
}
