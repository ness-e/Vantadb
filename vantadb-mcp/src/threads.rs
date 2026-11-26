//! MCP-32: MCP exposure of agentic conversation threads (MessageThread CRUD).
//!
//! Six tools wrapping `Embedded`'s thread API (`src/agentic/thread.rs`,
//! reached through the builder): create / send / get / list / delete / purge.
//! Thread ids are `u128` and travel as JSON strings (the MEM-32 wire
//! convention used by every other u128-facing tool). Domain errors surface as
//! `error_content` results the LLM can self-correct; malformed params are
//! JSON-RPC invalid-params (-32602).
//!
//! MCP-44: `thread_send` also feeds the memory pipeline — the turn becomes an
//! L0 record and ONE L1 extraction task is enqueued on the process-local
//! queue. Storage alone is not memory: without the enqueue a turn sent through
//! MCP is never recallable.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_payload};
use serde_json::{json, Value};
use std::sync::{Arc, OnceLock};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::ingest::observability::{self, QueueSnapshot, RunnerState};
use vanta_memory::ingest::runner_config::IngestRunnerCfg;
use vanta_memory::utils::{LocalStateBackend, SystemClock};
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
            "description": "Appends one message (role, content) to a thread AND feeds the memory pipeline: 'user'/'assistant' turns are captured as L0 records and one L1 extraction task is queued for the writer's scheduler. Roles are free-form strings ('user', 'assistant', ...); other roles are stored but not captured. Returns {ok:true, memory:{captured,queued}} — `queued` means a scheduler will consolidate it (see capabilities.ingest when it is `none`).",
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

fn db_from(storage: &Arc<StorageEngine>) -> vantadb::Embedded {
    vantadb::Embedded::from_engine(storage.clone())
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

    let db = db_from(storage);
    match db.send_message(thread_id, &role, content) {
        Ok(()) => {
            // MCP-44: storage is not memory — feed the pipeline (L0 + L1).
            let feed = capture_and_enqueue(&db, storage, thread_id, &role, content);
            Ok(text_content(serialize_content(&json!({
                "ok": true,
                "memory": { "captured": feed.captured, "queued": feed.queued },
            }))))
        }
        Err(e) => Ok(error_content(e.to_string())),
    }
}

// ── MCP-44: memory pipeline feed (L0 capture + L1 enqueue) ─────────────────

/// Process-local L1 queue.
///
/// ADR-0054: the queue, timers and leases live in the process that owns the DB
/// (the writer). One instance per process on purpose — MCP-43's scheduler must
/// drain THIS queue, not a second one. In proxy mode this module never runs
/// here: the proxy forwards the whole `tools/call` to the writer (MCP-48), so
/// the enqueue happens in the process that can actually drain it.
pub(crate) fn memory_queue() -> &'static Arc<LocalStateBackend<SystemClock>> {
    static QUEUE: OnceLock<Arc<LocalStateBackend<SystemClock>>> = OnceLock::new();
    QUEUE.get_or_init(|| Arc::new(LocalStateBackend::new(SystemClock)))
}

/// Live queue occupancy for the `capabilities.ingest` surface (ADR-0065 D1):
/// read at answer time, in the process that owns the queue. Never a photograph
/// of the last pass — `last_pass.queue` keeps that history.
pub(crate) fn queue_snapshot() -> QueueSnapshot {
    let (depth, claimed) = memory_queue().queue_occupancy();
    QueueSnapshot { depth, claimed }
}

/// What `thread_send` did with the memory pipeline (additive wire fields).
#[derive(Debug, Clone, Copy)]
struct MemoryFeed {
    /// The turn became a new L0 record (durable, LLM-free).
    captured: bool,
    /// One L1 extraction task was enqueued for the writer's scheduler.
    queued: bool,
}

/// Capture one conversation turn into L0 and enqueue its L1 extraction — the
/// same shape the HTTP host uses (`HttpCaptureBridge::trigger`, MEM-55).
///
/// One enqueue per NEW turn: the L0 cursor filters replays within the captured
/// window, so a retry inside that window enqueues nothing. A later retry is a
/// new turn by construction (the timestamp moves the cursor); the L1 dedup
/// absorbs the repeat. P4: an L0 failure is logged, never propagated — the
/// thread message is already persisted and the turn is not lost.
fn capture_and_enqueue(
    db: &vantadb::Embedded,
    storage: &Arc<StorageEngine>,
    thread_id: u128,
    role: &str,
    content: &str,
) -> MemoryFeed {
    let skipped = MemoryFeed {
        captured: false,
        queued: false,
    };
    // Thread roles are free-form (MCP-32): only conversation roles reach L0.
    let Ok(l0_role) = role.parse::<L0Role>() else {
        return skipped;
    };
    let queue = memory_queue();
    let session_id = thread_id.to_string();
    let recorded = L0Recorder::new(db.clone()).record_turn(
        &L0Capture {
            session_id: session_id.clone(),
            messages: vec![L0Message {
                id: None, // stable derived key → idempotent upsert
                role: l0_role,
                content: content.to_string(),
                timestamp_ms: queue.now_ms(),
            }],
        },
        None,
    );
    let recorded_count = match recorded {
        Ok(result) => result.recorded_count,
        Err(e) => {
            tracing::warn!(thread_id = %session_id, error = %e, "thread_send: L0 capture failed");
            return skipped;
        }
    };
    if recorded_count == 0 {
        return skipped; // cursor replay: nothing new to extract
    }
    note_runner_state_once(storage);
    queue.enqueue_task(TaskPayload {
        id: String::new(), // assigned by the queue on enqueue
        kind: TaskKind::L1,
        session_id,
        priority: 1,
        created_at_ms: queue.now_ms(),
        attempts: 0,
    });
    publish_backlog_if_degraded(queue);
    MemoryFeed {
        captured: true,
        queued: true,
    }
}

/// Teach this process which ingest runner it has, the first time a turn is
/// enqueued.
///
/// `observability` only learns the runner from `build_runner` (the single
/// provider→runner mapping, MEM-72). A `--mcp` session that only uses threads
/// would otherwise report `runner:"unknown"` + `depth:0` with work piling up —
/// the silent idle MEM-72 exists to delete. A runner a host already declared
/// wins: this fills the gap, it does not re-elect.
fn note_runner_state_once(storage: &Arc<StorageEngine>) {
    if observability::status().runner != RunnerState::Unknown {
        return;
    }
    // Side effect is the point (`build_runner` notes the runner); the value is
    // not used — the scheduler builds its own per pass (MCP-43).
    let _ = ingest_runner_cfg(storage).build_runner();
}

/// Server-side ingest config, same sources as the wiki path
/// (`wiki.rs::ingest_toml_path`): `VANTADB_INGEST_CONFIG` wins, else
/// `<data_dir>/vanta-ingest.toml`. Duplicated because that helper is private;
/// both must stay in sync (same host config, one meaning). `pub(crate)` since
/// MCP-43: the scheduler's per-pass runner factory resolves it here.
pub(crate) fn ingest_runner_cfg(storage: &Arc<StorageEngine>) -> IngestRunnerCfg {
    let path = std::env::var("VANTADB_INGEST_CONFIG")
        .ok()
        .filter(|p| !p.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| storage.data_dir().join("vanta-ingest.toml"));
    IngestRunnerCfg::from_env_toml(&path)
}

/// Feed the MEM-72 surface when this process already knows there is no runner:
/// "degraded with backlog" is a broken pipeline and must read as such instead
/// of `idle`. `note_backlog` owns the once-per-episode warn; this adds no
/// second diagnostic signal, and — ADR-0065 D7 — an enqueue never writes
/// `last_pass` (only a pass does).
fn publish_backlog_if_degraded(queue: &LocalStateBackend<SystemClock>) {
    if !observability::status().degraded {
        return;
    }
    let (depth, claimed) = queue.queue_occupancy();
    observability::note_backlog(QueueSnapshot { depth, claimed });
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

/// MCP-44 — the contract this file exists for: a turn sent through the tool
/// becomes recallable memory after one pass, and a runner-less pipeline that
/// has queued work reads as broken (MEM-72), never as idle.
///
/// One `#[test]` on purpose: the queue and the ingest status are process-wide,
/// so phases must run in order (same rule MEM-72 followed).
#[cfg(test)]
mod memory_feed_tests {
    use super::*;
    use crate::handlers::tools::handle_tools_call;
    use crate::{McpConfig, McpProfile};
    use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
    use vanta_memory::core::record::read_session_records;
    use vanta_memory::ingest::runner_config::IngestRunnerCfg;
    use vanta_memory::services::scheduler::run_pass;
    use vantadb::executor::Executor;
    use vantadb::storage::StorageEngine;

    /// L1 extraction response shape per `vanta-memory/tests/l1_extractor.rs`.
    const EXTRACTION_JSON: &str = r#"[
      {"scene_name": "Storage", "message_ids": [], "memories": [
        {"content": "The user prefers the fjall storage backend", "type": "preference", "priority": 80, "source_message_ids": []}
      ]}
    ]"#;

    /// Covers the two LLM calls an L1 pass makes (pattern: `vanta-memory/tests/scheduler.rs`).
    struct ScriptedRunner;

    impl LlmRunner for ScriptedRunner {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            match params.task_id.as_str() {
                "l1-extraction" => Ok(EXTRACTION_JSON.to_string()),
                "l1-conflict-detection" => {
                    let judged = params
                        .prompt
                        .split("NEW MEMORIES TO JUDGE")
                        .nth(1)
                        .unwrap_or(&params.prompt);
                    let record_id = judged
                        .split("\"record_id\": \"")
                        .nth(1)
                        .and_then(|rest| rest.split('"').next())
                        .unwrap_or("m_0");
                    Ok(format!(
                        r#"[{{"record_id": "{record_id}", "action": "store"}}]"#
                    ))
                }
                _ => Ok("[]".to_string()),
            }
        }
    }

    #[test]
    fn thread_send_captures_l0_enqueues_one_l1_task_and_recall_returns_the_turn() {
        // Phase 1 — the process already knows there is no ingest runner: the
        // default `local` provider maps to `ConcreteRunner::None` (MEM-72).
        IngestRunnerCfg::defaults()
            .build_runner()
            .expect("local provider always builds a runner");
        assert!(
            observability::status().degraded,
            "local provider is runner=none"
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let storage =
            Arc::new(StorageEngine::open(dir.path().to_str().expect("utf8")).expect("open"));
        let executor = Executor::new(&storage);
        let config = McpConfig {
            profile: McpProfile::Full,
            ..Default::default()
        };
        let call = |name: &str, args: Value| -> Value {
            let res = handle_tools_call(
                &Some(json!({ "name": name, "arguments": args })),
                &executor,
                &storage,
                &config,
            )
            .expect("tool call must succeed");
            serde_json::from_str(res["content"][0]["text"].as_str().expect("text content"))
                .expect("JSON payload")
        };

        let thread_id = call("thread_create", json!({ "title": "storage chat" }))["thread_id"]
            .as_str()
            .expect("thread id")
            .to_string();

        // One turn → one L0 record + exactly ONE L1 task.
        let sent = call(
            "thread_send",
            json!({
                "thread_id": thread_id,
                "role": "user",
                "content": "I prefer the fjall storage backend for this project",
            }),
        );
        assert_eq!(sent["ok"], json!(true));
        assert_eq!(
            sent["memory"],
            json!({ "captured": true, "queued": true }),
            "the turn must feed the pipeline: {sent}"
        );

        let queue = memory_queue();
        let tasks = queue.list_queued_tasks();
        assert_eq!(tasks.len(), 1, "one turn = one L1 task (no double enqueue)");
        assert_eq!(
            tasks[0].session_id, thread_id,
            "L1 runs on the thread's session"
        );
        assert_eq!(tasks[0].kind, TaskKind::L1);

        // Honest degraded half: queued work + runner `none` = broken pipeline.
        let caps = call("capabilities", json!({}));
        assert_eq!(caps["ingest"]["runner"], json!("none"), "caps: {caps}");
        assert_eq!(caps["ingest"]["degraded"], json!(true));
        assert_eq!(caps["ingest"]["queue"]["depth"], json!(1), "caps: {caps}");
        assert!(
            caps["ingest"]["notice"]
                .as_str()
                .unwrap_or_default()
                .contains("1 queued task(s)"),
            "the backlog must be legible: {caps}"
        );

        // A second turn queues a second task — never two per turn.
        call(
            "thread_send",
            json!({
                "thread_id": thread_id,
                "role": "assistant",
                "content": "Understood - fjall keeps the storage local and fast",
            }),
        );
        assert_eq!(queue.list_queued_tasks().len(), 2, "one task per turn");
        assert_eq!(
            observability::degraded_warnings(),
            1,
            "the backlog warns once per degraded episode, not per turn"
        );

        // A role outside the conversation set is stored, not captured.
        let other = call(
            "thread_send",
            json!({
                "thread_id": thread_id,
                "role": "system",
                "content": "you are a helpful assistant for this project",
            }),
        );
        assert_eq!(
            other["memory"],
            json!({ "captured": false, "queued": false }),
            "non-conversation roles are not extracted: {other}"
        );
        assert_eq!(
            queue.list_queued_tasks().len(),
            2,
            "non-conversation roles are not extracted"
        );
        let got = call("thread_get", json!({ "thread_id": thread_id }));
        assert_eq!(
            got["messages"].as_array().expect("messages").len(),
            3,
            "every message is still persisted: {got}"
        );

        // Phase 2 — one pass (MCP-43's job in production) extracts L1 records.
        let db = vantadb::Embedded::from_engine(storage.clone());
        let stats = run_pass(&**queue, db.clone(), &ScriptedRunner);
        assert_eq!(stats.worker.processed, 2, "both turns consolidated");
        assert_eq!(stats.worker.failed, 0);
        assert_eq!(
            queue.list_queued_tasks().len(),
            0,
            "the pass drained the queue"
        );
        let records = read_session_records(&db, &thread_id).expect("read l1 records");
        assert!(
            records.iter().any(|r| r.content.contains("fjall")),
            "expected extracted memory, got: {records:?}"
        );

        // Phase 3 — the turn is recallable WITHOUT ever calling `memory_put`.
        let recalled = call(
            "memory_recall",
            json!({ "query": "fjall storage backend", "scope": "agent" }),
        );
        let hits = recalled["recalled"].as_array().expect("recalled array");
        assert!(
            hits.iter()
                .any(|h| h["content"].as_str().unwrap_or_default().contains("fjall")),
            "recall must return the previous turn: {recalled}"
        );
    }
}
