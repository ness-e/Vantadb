//! MEM-68: gateway handlers for the capture-approval queue.
//!
//! Typed request/response layer the MCP `capture_approve` / `capture_reject` /
//! `capture_list_pending` tools wrap; no transport here (same pattern as
//! `knowledge_handlers.rs`: boundary validation → queue call → typed envelope).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use vantadb::sdk::VantaEmbedded;

use crate::core::abstractions::MemoryRecord;
use crate::core::record::approval::{ApprovalError, CaptureApprovalQueue, PendingCapture};
use crate::core::record::l1_writer::EmbedFn;

/// Errors surfaced by the capture-approval gateway handlers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CaptureApprovalGatewayError {
    /// No pending capture with that id.
    #[error("pending capture not found: {0}")]
    NotFound(String),
    /// The approve-time store write failed.
    #[error("capture approve failed: {0}")]
    Store(String),
    /// Input rejected at the gateway boundary (empty id).
    #[error("invalid capture approval request: {0}")]
    Invalid(String),
}

impl From<ApprovalError> for CaptureApprovalGatewayError {
    fn from(err: ApprovalError) -> Self {
        match err {
            ApprovalError::NotFound(id) => Self::NotFound(id),
            ApprovalError::Store(inner) => Self::Store(inner.to_string()),
        }
    }
}

/// Response of [`capture_list_pending`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureListPendingResponse {
    /// Captures still awaiting a decision (submission order).
    pub pending: Vec<PendingCapture>,
}

/// Request of [`capture_approve`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureApproveRequest {
    /// Pending id returned by queue `submit` (gateway never invents ids).
    pub id: String,
}

/// Response of [`capture_approve`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureApproveResponse {
    /// Records persisted by the approval.
    pub written: Vec<MemoryRecord>,
}

/// Request of [`capture_reject`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureRejectRequest {
    /// Pending id returned by queue `submit`.
    pub id: String,
}

/// Response of [`capture_reject`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaptureRejectResponse {
    /// `false` when the id was already decided or never queued (idempotent).
    pub rejected: bool,
}

fn require_id(id: &str) -> Result<&str, CaptureApprovalGatewayError> {
    if id.trim().is_empty() {
        return Err(CaptureApprovalGatewayError::Invalid(
            "id must not be empty".to_string(),
        ));
    }
    Ok(id)
}

/// List captures awaiting an approve/reject decision.
pub fn capture_list_pending(queue: &CaptureApprovalQueue) -> CaptureListPendingResponse {
    CaptureListPendingResponse {
        pending: queue.list_pending(),
    }
}

/// Approve a pending capture, persisting its memories.
pub fn capture_approve(
    queue: &CaptureApprovalQueue,
    db: &VantaEmbedded,
    request: &CaptureApproveRequest,
    now_ms: u64,
    embed: Option<&EmbedFn>,
) -> Result<CaptureApproveResponse, CaptureApprovalGatewayError> {
    let id = require_id(&request.id)?;
    let written = queue.approve(db, id, now_ms, embed)?;
    Ok(CaptureApproveResponse { written })
}

/// Reject a pending capture, dropping it without persisting.
pub fn capture_reject(
    queue: &CaptureApprovalQueue,
    request: &CaptureRejectRequest,
) -> Result<CaptureRejectResponse, CaptureApprovalGatewayError> {
    let id = require_id(&request.id)?;
    Ok(CaptureRejectResponse {
        rejected: queue.reject(id),
    })
}
