//! FIND-107 S2+S3: MCP exposure of the vanta-memory dream module.
//!
//! Read tools (`dream_list`/`dream_load`) plus the scoped delete
//! (`dream_discard`, S2) wrap `vanta_memory::core::dream`'s pure store
//! functions over `&Embedded` (`list_dream_runs` / `load_dream_run` /
//! `discard_dream_run` over `dream/<session>/<run_id>`). Write tools
//! (`dream_consolidate`/`dream_promote`, S3 + VER-07) wrap
//! `consolidate_session` (LLM-free: no runner injected, same degraded mode
//! the crate documents) and the promotion surface: `dream_promote` returns
//! the record-level dry-run plan by default and **applies it to
//! `l1/<session>` only with `dry_run:false`** (idempotent; fail-closed
//! quality gate on supersedes; DELETE scoped to the run's scanned inputs).
//! Consolidation never touches L1; promotion is the module's only mutating
//! entry point and defaults to preview. Domain errors surface as
//! `error_content` results the LLM can self-correct (MEM-32) while param
//! errors are JSON-RPC invalid-params (same contract as `scenes.rs`).

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_identifier};
use serde_json::{json, Value};
use std::sync::Arc;
use vanta_memory::core::dream::{
    consolidate_session, discard_dream_run, list_dream_runs, load_dream_run, plan_promotion,
    promote_dream_run, DreamConfig,
};
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
        json!({
            "name": "dream_consolidate",
            "description": "Runs one LLM-free consolidation pass over a session's L1 records (dedupe + contradiction provenance + relative-date normalization) and persists the view to dream/<session>/<run_id>. The original L1 store is never mutated. Fails as an error-content message when the session is not idle. Write path.",
            "annotations": {
                "title": "Dream Consolidate",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session to consolidate" },
                    "now_ms": { "type": "number", "description": "Wall-clock epoch ms (caller clock; anchors the run and the idle check)" },
                    "last_active_at_ms": { "type": "number", "description": "Epoch ms of the last agent activity; now_ms - last_active_at_ms must reach idle_threshold_ms" },
                    "idle_threshold_ms": { "type": "number", "description": "Optional idle threshold in ms (default 600000 = 10min)" },
                    "run_id_salt": { "type": "string", "description": "Optional salt for deterministic run ids (tests)" }
                },
                "required": ["session_key", "now_ms", "last_active_at_ms"]
            }
        }),
        json!({
            "name": "dream_promote",
            "description": "Promotes a dream run into l1/<session>: reports the record-level diff {action: ADD|UPDATE|DELETE|NOOP, key, reason} and, with dry_run:false, applies it (ADD/UPDATE upsert the record; DELETE removes a record the run scanned and dropped). Defaults to dry_run:true — the plan is reported and nothing mutates. Idempotent: re-promoting an applied run returns an all-NOOP plan and writes nothing. Fail-closed quality gate on supersedes; post-run additions to L1 are never part of the plan.",
            "annotations": {
                "title": "Dream Promote",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_key": { "type": "string", "description": "Session that owns the run" },
                    "run_id": { "type": "string", "description": "Run id returned by dream_list" },
                    "dry_run": { "type": "boolean", "description": "true (default) = report the promotion plan only; false = apply it to l1/<session>" }
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
        "dream_consolidate" => dream_consolidate_tool(args, storage, config),
        "dream_promote" => dream_promote_tool(args, storage, config),
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

/// `dream_discard(session_key, run_id)`: delete the run key (`run.json`); an empty
/// namespace may persist but `dream_list` skips it. L1 intact.
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

/// Extract a required `u64` epoch-ms param at the trust boundary.
fn epoch_ms_arg(args: &Value, field: &str) -> Result<u64, Value> {
    args[field].as_u64().ok_or_else(|| {
        McpError::invalid_params(format!("Missing or invalid '{field}' (epoch ms)")).to_json()
    })
}

/// `dream_consolidate(session_key, now_ms, last_active_at_ms, ...)`: one
/// LLM-free pass (no runner injected — the crate's documented degraded mode).
/// Not-idle is a domain error (content), not a protocol error.
fn dream_consolidate_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let now_ms = epoch_ms_arg(args, "now_ms")?;
    let last_active_at_ms = epoch_ms_arg(args, "last_active_at_ms")?;
    let mut dream_config = DreamConfig::default();
    if let Some(threshold) = args["idle_threshold_ms"].as_u64() {
        dream_config = dream_config.with_idle_threshold_ms(threshold);
    }
    if let Some(salt) = args["run_id_salt"].as_str() {
        dream_config = dream_config.with_run_id_salt(salt);
    }
    match consolidate_session(
        &db_from(storage),
        &session_key,
        now_ms,
        last_active_at_ms,
        &dream_config,
    ) {
        Ok(run) => Ok(text_content(serialize_content(&json!({ "run": run })))),
        Err(e) => Ok(error_content(e.to_string())),
    }
}

/// `dream_promote(session_key, run_id, dry_run=true)`: returns the
/// record-level promotion plan (`{dry_run, mutated, counts, ops}`). With the
/// default `dry_run:true` nothing mutates; `dry_run:false` applies the plan
/// to `l1/<session>` (idempotent — a second apply is all-NOOP). `mutated` is
/// `true` only when an apply actually wrote at least one record. Domain
/// errors (missing run, quality gate) surface as error-content messages.
fn dream_promote_tool(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let session_key = session_key_arg(args, config)?;
    let run_id = run_id_arg(args, config)?;
    // A missing OR non-boolean `dry_run` falls back to the safe default
    // (plan only, never an implicit apply).
    let dry_run = args["dry_run"].as_bool().unwrap_or(true);
    let db = db_from(storage);
    let result = if dry_run {
        plan_promotion(&db, &session_key, &run_id)
    } else {
        promote_dream_run(&db, &session_key, &run_id)
    };
    match result {
        Ok(plan) => {
            let counts = plan.counts();
            let mutated = !dry_run && counts.writes() > 0;
            Ok(text_content(serialize_content(&json!({
                "dry_run": dry_run,
                "mutated": mutated,
                "counts": counts,
                "ops": plan.ops,
            }))))
        }
        Err(e) => Ok(error_content(e.to_string())),
    }
}
