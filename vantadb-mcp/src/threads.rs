//! MCP-32: MCP exposure of agentic conversation threads (MessageThread CRUD).
//!
//! Six tools wrapping `VantaEmbedded`'s thread API (`src/agentic/thread.rs`,
//! reached through the builder): create / send / get / list / delete / purge.
//! Thread ids are `u128` and travel as JSON strings (the MEM-32 wire
//! convention used by every other u128-facing tool). Domain errors surface as
//! `error_content` results the LLM can self-correct; malformed params are
//! JSON-RPC invalid-params (-32602).

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_payload};
use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::storage::StorageEngine;

/// Tool definitions for `tools/list` (MEM-33 pattern).
pub(crate) fn thread_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "thread_create",
            "description": "Creates a conversation thread. Returns {thread_id} (u128 as string). Pass ttl_secs for auto-expiry (optional; purged later via thread_purge_expired).",
            "annotations": {
                "title": "Thread Create",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short human-readable thread title" },
                    "ttl_secs": { "type": "number", "description": "Optional time-to-live in seconds" }
                },
                "required": ["title"]
            }
        }),
        json!({
            "name": "thread_send",
            "description": "Appends one message (role, content) to a thread. Roles are free-form strings ('user', 'assistant', ...). Returns {ok:true}.",
            "annotations": {
                "title": "Thread Send",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "thread_id": { "type": "string", "description": "Thread id (u128 as returned by thread_create)" },
                    "role": { "type": "string", "description": "Sender role label" },
                    "content": { "type": "string", "description": "Message body" }
                },
                "required": ["thread_id", "role", "content"]
            }
        }),
        json!({
            "name": "thread_get",
            "description": "Reads one thread with its full message history: {thread:{thread_id,title,messages:[{role,content,timestamp}],created_at,updated_at}}. Missing threads answer an error_content 'not found'.",
            "annotations": {
                "title": "Thread Get",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "thread_id": { "type": "string", "description": "Thread id" }
                },
                "required": ["thread_id"]
            }
        }),
        json!({
            "name": "thread_list",
            "description": "Lists threads (insertion order) with pagination: {threads:[...],count}. Use offset+limit to page. Read-only.",
            "annotations": {
                "title": "Thread List",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "number", "description": "Optional page size (default 20)" },
                    "offset": { "type": "number", "description": "Optional skip count (default 0)" }
                }
            }
        }),
        json!({
            "name": "thread_delete",
            "description": "Deletes a thread and its message history permanently. Returns {deleted:true}. There is no undo.",
            "annotations": {
                "title": "Thread Delete",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "thread_id": { "type": "string", "description": "Thread id to delete" }
                },
                "required": ["thread_id"]
            }
        }),
        json!({
            "name": "thread_purge_expired",
            "description": "Removes all threads whose TTL has expired. Returns {purged:n}. No-op when no TTLs were set.",
            "annotations": {
                "title": "Thread Purge Expired",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": { "type": "object", "properties": {} }
        }),
    ]
}

/// Dispatch a `tools/call` for the thread tools.
pub(crate) fn handle_thread_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    match name {
        "thread_create" => thread_create_tool(args, storage, config),
        "thread_send" => thread_send_tool(args, storage, config),
        "thread_get" => thread_get_tool(args, storage, config),
        "thread_list" => thread_list_tool(args, storage, config),
        "thread_delete" => thread_delete_tool(args, storage, config),
        "thread_purge_expired" => thread_purge_tool(storage),
        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

fn db_from(storage: &Arc<StorageEngine>) -> vantadb::VantaEmbedded {
    vantadb::VantaEmbedded::from_engine(storage.clone())
}

/// `MessageThread` carries a `u128` id, which serde_json cannot serialize
/// natively — transport it as a string (MEM-32 wire convention for u128).
fn thread_to_json(t: vantadb::agentic::MessageThread) -> Value {
    let messages: Vec<Value> = t
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": m.role,
                "content": m.content,
                "timestamp": m.timestamp,
                "metadata": m.metadata,
            })
        })
        .collect();
    json!({
        "thread_id": t.thread_id.to_string(),
        "title": t.title,
        "messages": messages,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
        "metadata": t.metadata,
    })
}

/// Extract + validate a `thread_id` (u128 transported as string).
fn thread_id_arg(args: &Value) -> Result<u128, Value> {
    let raw = args["thread_id"].as_str().ok_or_else(|| {
        McpError::invalid_params("Missing or invalid 'thread_id' (expected u128 string)").to_json()
    })?;
    raw.parse::<u128>().map_err(|_| {
        McpError::invalid_params(format!("'thread_id' is not a valid u128: {raw}")).to_json()
    })
}

fn short_string_arg(args: &Value, key: &str, config: &McpConfig) -> Result<String, Value> {
    let v = args[key]
        .as_str()
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid '{key}'")).to_json())?;
    validate_payload(v, config.max_query_length).map_err(|e| e.to_json())?;
    Ok(v.to_string())
}

fn thread_create_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let title = short_string_arg(args, "title", config)?;
    let ttl_secs = args["ttl_secs"].as_u64();
    match db_from(storage).create_thread(&title, ttl_secs) {
        Ok(id) => Ok(text_content(serialize_content(
            &json!({ "thread_id": id.to_string() }),
        ))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

fn thread_send_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let thread_id = thread_id_arg(args)?;
    let role = short_string_arg(args, "role", config)?;
    let content = args["content"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'content'").to_json())?;
    // Message bodies run longer than titles; still capped at the request-size
    // guard so a runaway agent cannot stream unbounded payloads over stdio.
    validate_payload(content, config.max_query_length * 4).map_err(|e| e.to_json())?;

    match db_from(storage).send_message(thread_id, &role, content) {
        Ok(()) => Ok(text_content(serialize_content(&json!({ "ok": true })))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

fn thread_get_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    _config: &McpConfig,
) -> Result<Value, Value> {
    let thread_id = thread_id_arg(args)?;
    match db_from(storage).get_thread(thread_id) {
        Ok(Some(t)) => Ok(text_content(serialize_content(&thread_to_json(t)))),
        Ok(None) => Ok(error_content(format!("Thread {thread_id} not found"))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

fn thread_list_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let limit = args["limit"]
        .as_u64()
        .map(|l| (l as usize).min(config.max_top_k));
    let limit = limit.unwrap_or(20);
    let offset = args["offset"].as_u64().unwrap_or(0) as usize;
    match db_from(storage).list_threads(limit, offset) {
        Ok(threads) => {
            let count = threads.len();
            let threads: Vec<Value> = threads.into_iter().map(thread_to_json).collect();
            Ok(text_content(serialize_content(
                &json!({ "threads": threads, "count": count }),
            )))
        }
        Err(e) => Ok(error_content(e.to_string())),
    }
}

fn thread_delete_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    _config: &McpConfig,
) -> Result<Value, Value> {
    let thread_id = thread_id_arg(args)?;
    match db_from(storage).delete_thread(thread_id) {
        Ok(()) => Ok(text_content(serialize_content(&json!({ "deleted": true })))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

fn thread_purge_tool(storage: &Arc<StorageEngine>) -> Result<Value, Value> {
    match db_from(storage).purge_expired_threads() {
        Ok(purged) => Ok(text_content(serialize_content(
            &json!({ "purged": purged }),
        ))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}
