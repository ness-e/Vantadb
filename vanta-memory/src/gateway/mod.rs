//! Gateway-facing handlers (MCP scene/knowledge tools).
//!
//! Typed request/response layer a future MCP server wraps; no transport here.

pub mod knowledge_handlers;

/// MEM-68: gateway handlers for the capture-approval queue.
pub mod approval_handlers;

pub use approval_handlers::{
    capture_approve, capture_list_pending, capture_reject, CaptureApprovalGatewayError,
    CaptureApproveRequest, CaptureApproveResponse, CaptureListPendingResponse,
    CaptureRejectRequest, CaptureRejectResponse,
};
pub use knowledge_handlers::{
    scene_list, scene_query, scene_read, KnowledgeError, SceneListRequest, SceneListResponse,
    SceneQueryHit, SceneQueryRequest, SceneQueryResponse, SceneReadRequest, SceneReadResponse,
    DEFAULT_QUERY_TOP_K,
};
