//! MCP-30: MCP exposure of the vanta-memory gateway scene handlers.
//!
//! Three read-only tools wrapping [`vanta_memory::gateway`]'s pure functions
//! over `&VantaEmbedded`: structured scene navigation for external agents.
//! The gateway request/response serde types ARE the wire shape (zero new
//! wire types); domain errors surface as `error_content` results the LLM can
//! self-correct (MEM-32) while param errors are JSON-RPC invalid-params.
//!
//! `scene_query` runs keyword-only ranking (`embed = None`) — the same
//! degraded mode the crate documents for hook-less callers (MEM-47/D38).

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{
    error_content, serialize_content, text_content, validate_identifier, validate_payload,
};
use serde_json::{json, Value};
use std::sync::Arc;
use vanta_memory::gateway::{
    scene_list, scene_query, scene_read, SceneListRequest, SceneQueryRequest, SceneReadRequest,
};
use vantadb::storage::StorageEngine;

/// Tool definitions for `tools/list` (MEM-33 pattern).
pub(crate) fn scene_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "scene_read",
            "description": "Reads one live memory scene block by name from a session's structured scene store. Returns {scene:{scene_name, meta{created,updated,summary,heat}, content}}. Missing or soft-deleted scenes answer a 'not found' message (indistinguishable by design). Read-only.",
            "annotations": {
                "title": "Scene Read",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session whose scene store is queried" },
                    "scene_name": { "type": "string", "description": "Scene name to read" }
                },
                "required": ["session_key", "scene_name"]
            }
        }),
        json!({
            "name": "scene_list",
            "description": "Lists the scene index of a session (heat descending, soft-deleted excluded). Returns {scenes:[{filename,summary,heat,created,updated}]} where filename is the scene id for scene_read. Read-only.",
            "annotations": {
                "title": "Scene List",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session whose scene index is listed" }
                },
                "required": ["session_key"]
            }
        }),
        json!({
            "name": "scene_query",
            "description": "Keyword search over the live scene blocks of a session: ranks scenes by term overlap between the keyword and summary+content (ties by heat). Returns {hits:[{scene_name,summary,heat,updated,score}]}; empty when nothing matches. Load hits via scene_read. Read-only.",
            "annotations": {
                "title": "Scene Query",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session whose scene store is searched" },
                    "keyword": { "type": "string", "description": "Free-text keyword query" },
                    "top_k": { "type": "number", "description": "Optional maximum hits (default 5)" }
                },
                "required": ["session_key", "keyword"]
            }
        }),
    ]
}

/// Dispatch a `tools/call` for the scene tools.
pub(crate) fn handle_scene_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    match name {
        "scene_read" => scene_read_tool(args, storage, config),
        "scene_list" => scene_list_tool(args, storage, config),
        "scene_query" => scene_query_tool(args, storage, config),
        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

/// Extract + validate `session_key` at the trust boundary.
fn session_key_arg(args: &Value, config: &McpConfig) -> Result<String, Value> {
    let key = args["session_key"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'session_key'").to_json())?;
    validate_identifier(key, "session_key", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    Ok(key.to_string())
}

fn db_from(storage: &Arc<StorageEngine>) -> vantadb::VantaEmbedded {
    vantadb::VantaEmbedded::from_engine(storage.clone())
}

/// `scene_read(session_key, scene_name)`: one live block by name.
fn scene_read_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let scene_name = args["scene_name"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'scene_name'").to_json())?;
    validate_identifier(scene_name, "scene_name", config.max_key_length)
        .map_err(|e| e.to_json())?;

    match scene_read(
        &db_from(storage),
        &SceneReadRequest {
            session_key,
            scene_name: scene_name.to_string(),
        },
    ) {
        Ok(resp) => Ok(text_content(serialize_content(&resp))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

/// `scene_list(session_key)`: the session's scene index, heat desc.
fn scene_list_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    match scene_list(
        &db_from(storage),
        &SceneListRequest {
            session_key: session_key_arg(args, config)?,
        },
    ) {
        Ok(resp) => Ok(text_content(serialize_content(&resp))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

/// `scene_query(session_key, keyword, top_k?)`: keyword search over blocks.
fn scene_query_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let keyword = args["keyword"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'keyword'").to_json())?;
    validate_payload(keyword, config.max_query_length).map_err(|e| e.to_json())?;
    let top_k = args["top_k"]
        .as_u64()
        .map(|k| (k as usize).min(config.max_top_k));

    match scene_query(
        &db_from(storage),
        &SceneQueryRequest {
            session_key,
            keyword: keyword.to_string(),
            top_k,
        },
        None,
    ) {
        Ok(resp) => Ok(text_content(serialize_content(&resp))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}
