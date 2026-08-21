//! Gateway-facing handlers (MCP scene/knowledge tools).
//!
//! Typed request/response layer a future MCP server wraps; no transport here.

pub mod knowledge_handlers;

pub use knowledge_handlers::{
    scene_list, scene_query, scene_read, KnowledgeError, SceneListRequest, SceneListResponse,
    SceneQueryHit, SceneQueryRequest, SceneQueryResponse, SceneReadRequest, SceneReadResponse,
    DEFAULT_QUERY_TOP_K,
};
