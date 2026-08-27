//! MCP-31: MCP exposure of the vanta-memory context engine.
//!
//! One tool, `context_assemble`: wraps [`assemble_with_recall`] (history
//! compaction + MMD + recall blocks under ONE shared token budget, MEM-37)
//! plus the session auto-recall hook (`perform_auto_recall`) that desktop
//! consumes over IPC (MEM-58). Compressor internals stay internal — v1
//! exposes assembly only; the compaction report travels as stable serde
//! metadata (`IntegratedContext` IS the wire type).
//!
//! Error contract follows the MEM-32 learning: domain errors surface as
//! `error_content` results the LLM can self-correct; param errors surface as
//! JSON-RPC invalid-params. An unknown session is NOT an error — assembly
//! still runs on the provided history.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{
    error_content, serialize_content, text_content, validate_identifier, validate_payload,
};
use serde_json::{json, Value};
use std::sync::Arc;
use vanta_memory::context_engine::{
    assemble_with_recall, AssembleConfig, ChatMessage, ChatRole, TokenEstimator,
};
use vanta_memory::core::hooks::{perform_auto_recall, AutoRecallParams, RecallConfig};
use vantadb::storage::StorageEngine;

/// Tool definitions for `tools/list` (MEM-33 pattern).
pub(crate) fn context_tool_definitions() -> Vec<Value> {
    vec![json!({
        "name": "context_assemble",
        "description": "Assembles a ready-to-send context window under a token budget with the memory OS context engine: compacts the provided chat history and injects session recall (relevant L1 memories for the query, user persona, scene navigation) when session_key matches a known session. Returns {messages, report{mode,msgs_conserved,msgs_before,tokens_before,tokens_after}, mmd_injected, recall_injected}. Read-only.",
            "annotations": {
                "title": "Context Assemble",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
        "inputSchema": {
            "type": "object",
            "properties": {
                "session_key": { "type": "string", "description": "Session whose memories/persona/scenes feed the recall blocks" },
                "token_budget": { "type": "number", "description": "Token budget for the assembled context (must be > 0)" },
                "query": { "type": "string", "description": "Optional user text that drives L1 memory search" },
                "messages": {
                    "type": "array",
                    "description": "Optional chat history to compact; each item is {role: system|user|assistant|tool_call|tool_result, content: string, id?: string}. Omit to get recall blocks only.",
                    "items": { "type": "object" }
                }
            },
            "required": ["session_key", "token_budget"]
        }
    })]
}

/// Dispatch a `tools/call` for the context tools.
pub(crate) fn handle_context_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    match name {
        "context_assemble" => context_assemble(args, storage, config),
        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

/// `context_assemble`: session recall + history compaction under one budget.
fn context_assemble(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    // ── trust boundary: validate every external input before it reaches the engine ──
    let session_key = args["session_key"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing or invalid 'session_key'").to_json())?;
    validate_identifier(session_key, "session_key", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let budget = args["token_budget"].as_u64().ok_or_else(|| {
        McpError::invalid_params("Missing or invalid 'token_budget' (unsigned integer)").to_json()
    })?;
    if budget == 0 {
        return Ok(error_content("'token_budget' must be greater than zero"));
    }
    let query = args["query"].as_str().unwrap_or("");
    let messages = parse_messages(args, config)?;

    let db = vantadb::VantaEmbedded::from_engine(storage.clone());

    // Session recall (L1 memories + persona + scene navigation). Empty query
    // still injects persona/navigation — documented hook behavior. No
    // embedding hook here: search degrades to keyword (crate contract D38).
    // An unknown/empty session yields Ok(None) — never an error.
    let (prepend, append) = match perform_auto_recall(
        &db,
        AutoRecallParams {
            user_text: query,
            session_key,
            isolation: None,
            config: RecallConfig::default(),
        },
        None,
    ) {
        Ok(Some(r)) => (r.prepend_context, r.append_system_context),
        Ok(None) => (None, None),
        Err(e) => return Ok(error_content(format!("recall failed: {e}"))),
    };

    // The engine owns correctness from here: compaction, MMD and recall all
    // spend the same mutable budget (budget > 0 already enforced above).
    match assemble_with_recall(
        messages,
        budget,
        &TokenEstimator::default(),
        0,
        &AssembleConfig::default(),
        None,
        prepend.as_deref(),
        append.as_deref(),
        None,
        None,
    ) {
        Ok(ctx) => Ok(text_content(serialize_content(&ctx))),
        Err(e) => Ok(error_content(format!("context assemble failed: {e}"))),
    }
}

/// Parse the optional `messages` history at the trust boundary: each entry
/// needs a known serde role and string content capped like any other payload.
fn parse_messages(args: &Value, config: &McpConfig) -> Result<Vec<ChatMessage>, Value> {
    let Some(arr) = args.get("messages").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    arr.iter()
        .map(|m| {
            let role_json = m
                .get("role")
                .ok_or_else(|| McpError::invalid_params("Each message needs a 'role'").to_json())?;
            let role: ChatRole = serde_json::from_value(role_json.clone()).map_err(|_| {
                McpError::invalid_params(
                    "Invalid message 'role' (expected system|user|assistant|tool_call|tool_result)",
                )
                .to_json()
            })?;
            let content = m["content"].as_str().ok_or_else(|| {
                McpError::invalid_params("Each message needs a string 'content'").to_json()
            })?;
            validate_payload(content, config.max_payload_length).map_err(|e| e.to_json())?;
            let mut msg = ChatMessage::new(role, content);
            if let Some(id) = m["id"].as_str() {
                msg = msg.with_id(id);
            }
            Ok(msg)
        })
        .collect()
}
