//! FIND-107 S2: MCP exposure of the vanta-memory dream store layer.
//!
//! Read tools (`dream_list`/`dream_load`) plus the scoped delete
//! (`dream_discard`) wrap `vanta_memory::core::dream`'s pure store functions
//! over `&Embedded` (`list_dream_runs` / `load_dream_run` /
//! `discard_dream_run` over `dream/<session>/<run_id>`). The L1 store is never
//! touched — `discard` deletes only the dream namespace (MEM-61 invariant).
//! Domain errors surface as `error_content` results the LLM can self-correct
//! (MEM-32) while param errors are JSON-RPC invalid-params (same contract as
//! `scenes.rs`).

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_identifier};
use serde_json::{json, Value};
use std::sync::Arc;
use vanta_memory::core::dream::{discard_dream_run, list_dream_runs, load_dream_run};
use vantadb::storage::StorageEngine;

/// Tool definitions for `tools/list` (MEM-33 pattern).
pub(crate) fn dream_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "dream_list",
            "description": "Lists every dream consolidation run for a session (metadata only: run_id, started_at_ms, ended_at_ms, inputs_scanned, runner_label). Load a run via dream_load. Read-only.",
            "annotations": {
                "title": "Dream List",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session whose dream runs are listed" }
                },
                "required": ["session_key"]
            }
        }),
        json!({
            "name": "dream_load",
            "description": "Loads the full persisted dream run (consolidated view living in dream/<session>/<run_id>; the original L1 store is never replaced). Missing runs answer a 'not found' message. Read-only.",
            "annotations": {
                "title": "Dream Load",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session that owns the run" },
                    "run_id": { "type": "string", "description": "Run id returned by dream_list" }
                },
                "required": ["session_key", "run_id"]
            }
        }),
        json!({
            "name": "dream_discard",
            "description": "Discards one dream run (deletes dream/<session>/<run_id>) after review. The original L1 store remains untouched. Idempotent: discarding twice is safe. Scoped destructive (dream namespace only).",
            "annotations": {
                "title": "Dream Discard",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session that owns the run" },
                    "run_id": { "type": "string", "description": "Run id returned by dream_list" }
                },
                "required": ["session_key", "run_id"]
            }
        }),
    ]
}

/// Dispatch a `tools/call` for the dream tools.
pub(crate) fn handle_dream_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    match name {
        "dream_list" => dream_list_tool(args, storage, config),
        "dream_load" => dream_load_tool(args, storage, config),
        "dream_discard" => dream_discard_tool(args, storage, config),
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

/// Extract + validate `run_id` at the trust boundary.
fn run_id_arg(args: &Value, config: &McpConfig) -> Result<String, Value> {
    let id = args["run_id"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'run_id'").to_json())?;
    validate_identifier(id, "run_id", config.max_key_length).map_err(|e| e.to_json())?;
    Ok(id.to_string())
}

fn db_from(storage: &Arc<StorageEngine>) -> vantadb::Embedded {
    vantadb::Embedded::from_engine(storage.clone())
}

/// `dream_list(session_key)`: metadata of every run for the session.
fn dream_list_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    match list_dream_runs(&db_from(storage), &session_key) {
        Ok(runs) => Ok(text_content(serialize_content(&json!({ "runs": runs })))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

/// `dream_load(session_key, run_id)`: full persisted run, or not-found content.
fn dream_load_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let run_id = run_id_arg(args, config)?;
    match load_dream_run(&db_from(storage), &session_key, &run_id) {
        Ok(Some(run)) => Ok(text_content(serialize_content(&json!({ "run": run })))),
        Ok(None) => Ok(error_content(format!("dream run {run_id} not found"))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

/// `dream_discard(session_key, run_id)`: delete the run namespace; L1 intact.
fn dream_discard_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let run_id = run_id_arg(args, config)?;
    match discard_dream_run(&db_from(storage), &session_key, &run_id) {
        Ok(()) => Ok(text_content(serialize_content(
            &json!({ "discarded": true }),
        ))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}
