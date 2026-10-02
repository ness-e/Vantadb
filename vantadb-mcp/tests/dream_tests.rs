// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// FIND-107 S2: `dream_list` / `dream_load` / `dream_discard` — MCP exposure of
// the vanta-memory dream store layer (MEM-61: list/load/discard over
// `dream/<session>/<run_id>`, L1 never touched).
//
// Round-trips go through the public `handle_tools_call` API, mirroring how
// external agents consume the tool (same pattern as `scene_tests.rs`). Dream
// runs are seeded via vanta-memory's own `consolidate_session` (the same
// fixture its `dreaming.rs` integration test uses — no pipeline worker needed).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig, McpProfile};

/// WIRE-02: these tests exercise the extended surface; pin `full` explicitly
/// (the production default is now `agent`).
fn full_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Full,
        ..Default::default()
    }
}

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_str().expect("db path utf8");
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

fn call(name: &str, args: Value, storage: &Arc<StorageEngine>) -> Result<Value, Value> {
    let executor = Executor::new(storage);
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": args })),
        &executor,
        storage,
        &full_config(),
    )
}

fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

fn result_json(res: Result<Value, Value>) -> Value {
    let text = msg(res);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("result is not JSON: {e}\n{text}"))
}

/// Seed one L1 record for `session` (no dream run yet).
fn seed_l1(storage: &Arc<StorageEngine>, session: &str) {
    seed_l1_with_metadata(storage, session, json!(null));
}

/// Seed one L1 record carrying `metadata` (e.g. a relative date so a
/// consolidation pass produces a `normalize` UPDATE in the promotion plan).
fn seed_l1_with_metadata(storage: &Arc<StorageEngine>, session: &str, metadata: Value) {
    use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};

    let db = vantadb::Embedded::from_engine(storage.clone());
    let rec = MemoryRecord {
        id: "m1".into(),
        content: "user prefers dark mode".into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "ui".into(),
        source_message_ids: vec![],
        metadata,
        timestamps: vec![],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
        version: 1,
        session_key: session.into(),
        session_id: "".into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    };
    let ns = format!("l1/{session}");
    db.put(vantadb::sdk::MemoryInput {
        namespace: ns,
        key: rec.id.clone(),
        payload: serde_json::to_string(&rec).expect("serialize"),
        metadata: vantadb::sdk::MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put l1");
}

/// Seed one L1 record, then run an LLM-free consolidation pass directly.
/// Returns the run_id of the persisted `dream/<session>/<run_id>` run.
fn seed_dream_run(storage: &Arc<StorageEngine>, session: &str, salt: &str) -> String {
    let db = vantadb::Embedded::from_engine(storage.clone());
    seed_l1(storage, session);
    consolidate(&db, session, salt)
}

/// Seed one L1 record carrying a relative date (`activity_start_time`), then
/// consolidate — the run view normalizes it, so promotion shows an UPDATE.
fn seed_dated_dream_run(storage: &Arc<StorageEngine>, session: &str, salt: &str) -> String {
    let db = vantadb::Embedded::from_engine(storage.clone());
    seed_l1_with_metadata(storage, session, json!({ "activity_start_time": "ayer" }));
    consolidate(&db, session, salt)
}

/// Run one consolidation pass over `session` and return the run id.
fn consolidate(db: &vantadb::Embedded, session: &str, salt: &str) -> String {
    let now_ms = 1_700_000_000_000u64;
    let config = vanta_memory::core::dream::DreamConfig::default().with_run_id_salt(salt);
    let run = vanta_memory::core::dream::consolidate_session(
        db,
        session,
        now_ms,
        now_ms - 3_600_000, // 1h idle (>= 10min default)
        &config,
    )
    .expect("consolidate");
    run.run_id
}

fn l1_count(storage: &Arc<StorageEngine>, session: &str) -> usize {
    let db = vantadb::Embedded::from_engine(storage.clone());
    vanta_memory::core::record::read_session_records(&db, session)
        .expect("read l1")
        .len()
}

// ── tools/list ───────────────────────────────────────────────────────────

#[test]
fn tools_list_registers_dream_tools_with_valid_schemas() {
    let (_dir, _storage) = setup_storage();
    let res = handle_tools_list(&full_config()).expect("tools/list");
    let tools = res["tools"].as_array().expect("tools array");
    for name in ["dream_list", "dream_load", "dream_discard"] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == json!(name))
            .unwrap_or_else(|| panic!("{name} missing from tools/list"));
        assert!(!tool["description"].as_str().unwrap_or_default().is_empty());
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert!(tool["inputSchema"]["properties"].is_object());
        assert!(
            !tool["inputSchema"]["required"]
                .as_array()
                .expect("required")
                .is_empty(),
            "{name} must declare required params"
        );
        assert_eq!(tool["annotations"]["openWorldHint"], false);
    }
    assert_eq!(
        tools
            .iter()
            .find(|t| t["name"] == json!("dream_list"))
            .expect("dream_list")["annotations"]["readOnlyHint"],
        true
    );
    assert_eq!(
        tools
            .iter()
            .find(|t| t["name"] == json!("dream_discard"))
            .expect("dream_discard")["annotations"]["destructiveHint"],
        true
    );
}

// ── round-trip ───────────────────────────────────────────────────────────

#[test]
fn dream_list_load_discard_roundtrip_keeps_l1_intact() {
    let (_dir, storage) = setup_storage();
    let run_id = seed_dream_run(&storage, "dream-agent-1", "test-roundtrip");
    assert_eq!(l1_count(&storage, "dream-agent-1"), 1);

    // list shows the run (metadata only).
    let listed = result_json(call(
        "dream_list",
        json!({"session_key": "dream-agent-1"}),
        &storage,
    ));
    let runs = listed["runs"].as_array().expect("runs array");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["run_id"], run_id);
    assert_eq!(runs[0]["inputs_scanned"], 1);

    // load returns the full run.
    let loaded = result_json(call(
        "dream_load",
        json!({"session_key": "dream-agent-1", "run_id": run_id}),
        &storage,
    ));
    assert_eq!(loaded["run"]["run_id"], run_id);
    assert_eq!(loaded["run"]["session_id"], "dream-agent-1");

    // discard removes the run; L1 stays byte-identical.
    let discarded = result_json(call(
        "dream_discard",
        json!({"session_key": "dream-agent-1", "run_id": run_id}),
        &storage,
    ));
    assert_eq!(discarded["discarded"], true);
    let listed = result_json(call(
        "dream_list",
        json!({"session_key": "dream-agent-1"}),
        &storage,
    ));
    assert_eq!(listed["runs"].as_array().expect("runs").len(), 0);
    assert_eq!(
        l1_count(&storage, "dream-agent-1"),
        1,
        "discard must never touch l1/<session>"
    );
}

// ── error contract ───────────────────────────────────────────────────────

#[test]
fn tools_list_registers_dream_consolidate_promote_with_valid_schemas() {
    let (_dir, _storage) = setup_storage();
    let res = handle_tools_list(&full_config()).expect("tools/list");
    let tools = res["tools"].as_array().expect("tools array");
    for name in ["dream_consolidate", "dream_promote"] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == json!(name))
            .unwrap_or_else(|| panic!("{name} missing from tools/list"));
        assert!(!tool["description"].as_str().unwrap_or_default().is_empty());
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert!(tool["inputSchema"]["properties"].is_object());
        assert!(
            !tool["inputSchema"]["required"]
                .as_array()
                .expect("required")
                .is_empty(),
            "{name} must declare required params"
        );
        assert_eq!(tool["annotations"]["openWorldHint"], false);
    }
    assert_eq!(
        tools
            .iter()
            .find(|t| t["name"] == json!("dream_promote"))
            .expect("dream_promote")["annotations"]["readOnlyHint"],
        false
    );
    assert_eq!(
        tools
            .iter()
            .find(|t| t["name"] == json!("dream_promote"))
            .expect("dream_promote")["annotations"]["destructiveHint"],
        true
    );
    assert_eq!(
        tools
            .iter()
            .find(|t| t["name"] == json!("dream_consolidate"))
            .expect("dream_consolidate")["annotations"]["readOnlyHint"],
        false
    );
}

#[test]
fn dream_promote_schema_exposes_dry_run_as_optional_boolean() {
    let (_dir, _storage) = setup_storage();
    let res = handle_tools_list(&full_config()).expect("tools/list");
    let tools = res["tools"].as_array().expect("tools array");
    let promote = tools
        .iter()
        .find(|t| t["name"] == json!("dream_promote"))
        .expect("dream_promote");
    assert_eq!(
        promote["inputSchema"]["properties"]["dry_run"]["type"],
        "boolean"
    );
    let required = promote["inputSchema"]["required"]
        .as_array()
        .expect("required");
    assert!(
        !required.iter().any(|r| r == "dry_run"),
        "dry_run is optional (safe default true): {required:?}"
    );
}

#[test]
fn dream_consolidate_creates_run_listed_and_l1_intact() {
    let (_dir, storage) = setup_storage();
    // Seed L1 directly (no dream run yet).
    seed_l1(&storage, "dream-agent-2");
    assert_eq!(l1_count(&storage, "dream-agent-2"), 1);

    let now_ms = 1_700_000_000_000u64;
    let run = result_json(call(
        "dream_consolidate",
        json!({
            "session_key": "dream-agent-2",
            "now_ms": now_ms,
            "last_active_at_ms": now_ms - 3_600_000,
            "run_id_salt": "test-consolidate",
        }),
        &storage,
    ));
    assert_eq!(run["run"]["session_id"], "dream-agent-2");
    assert_eq!(run["run"]["inputs_scanned"], 1);
    assert_eq!(run["run"]["runner_label"], "none");

    // Visible via dream_list; L1 untouched.
    let listed = result_json(call(
        "dream_list",
        json!({"session_key": "dream-agent-2"}),
        &storage,
    ));
    assert_eq!(listed["runs"].as_array().expect("runs").len(), 1);
    assert_eq!(l1_count(&storage, "dream-agent-2"), 1);
}

#[test]
fn dream_consolidate_not_idle_is_error_content_not_protocol_error() {
    let (_dir, storage) = setup_storage();
    seed_l1(&storage, "dream-agent-3");
    let now_ms = 1_700_000_000_000u64;
    let res = call(
        "dream_consolidate",
        json!({
            "session_key": "dream-agent-3",
            "now_ms": now_ms,
            "last_active_at_ms": now_ms - 1_000, // 1s idle < 10min default
        }),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("not idle"),
        "domain error as content: {text}"
    );
}

#[test]
fn dream_promote_defaults_to_dry_run_and_reports_diff_without_mutating_l1() {
    let (_dir, storage) = setup_storage();
    let run_id = seed_dated_dream_run(&storage, "dream-agent-5", "test-promote-dry");

    // No `dry_run` argument → safe default: plan only.
    let plan = result_json(call(
        "dream_promote",
        json!({"session_key": "dream-agent-5", "run_id": run_id}),
        &storage,
    ));
    assert_eq!(plan["dry_run"], true);
    assert_eq!(plan["mutated"], false);
    let ops = plan["ops"].as_array().expect("ops array");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["action"], "UPDATE");
    assert_eq!(ops[0]["reason"], "normalize");
    assert_eq!(ops[0]["key"], "m1");
    assert_eq!(ops[0]["namespace"], "l1/dream-agent-5");
    assert_eq!(plan["counts"]["update"], 1);
    assert_eq!(plan["counts"]["delete"], 0);

    // Nothing mutated: run still listed, L1 still has the raw relative date.
    let listed = result_json(call(
        "dream_list",
        json!({"session_key": "dream-agent-5"}),
        &storage,
    ));
    assert_eq!(listed["runs"].as_array().expect("runs").len(), 1);
    let db = vantadb::Embedded::from_engine(storage.clone());
    let records =
        vanta_memory::core::record::read_session_records(&db, "dream-agent-5").expect("read l1");
    assert_eq!(
        records[0]
            .metadata
            .as_object()
            .unwrap()
            .get("activity_start_time")
            .unwrap()
            .as_str()
            .unwrap(),
        "ayer",
        "dry-run must not touch l1"
    );
}

#[test]
fn dream_promote_apply_mutates_l1_and_reapply_is_all_noop() {
    let (_dir, storage) = setup_storage();
    let run_id = seed_dated_dream_run(&storage, "dream-agent-6", "test-promote-apply");
    let db = vantadb::Embedded::from_engine(storage.clone());

    let applied = result_json(call(
        "dream_promote",
        json!({"session_key": "dream-agent-6", "run_id": run_id, "dry_run": false}),
        &storage,
    ));
    assert_eq!(applied["dry_run"], false);
    assert_eq!(applied["mutated"], true);
    assert_eq!(applied["counts"]["update"], 1);

    let records =
        vanta_memory::core::record::read_session_records(&db, "dream-agent-6").expect("read l1");
    assert_ne!(
        records[0]
            .metadata
            .as_object()
            .unwrap()
            .get("activity_start_time")
            .unwrap()
            .as_str()
            .unwrap(),
        "ayer",
        "apply must write the normalized date into l1"
    );

    // Idempotent: second apply writes nothing and reports all-NOOP.
    let reapplied = result_json(call(
        "dream_promote",
        json!({"session_key": "dream-agent-6", "run_id": run_id, "dry_run": false}),
        &storage,
    ));
    assert_eq!(reapplied["mutated"], false);
    let ops = reapplied["ops"].as_array().expect("ops");
    assert!(
        ops.iter().all(|o| o["action"] == "NOOP"),
        "re-apply must be all NOOP: {ops:?}"
    );
}

#[test]
fn dream_promote_missing_run_is_error_content_not_protocol_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "dream_promote",
        json!({"session_key": "dream-agent-4", "run_id": "ghost-run-id"}),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("not found"),
        "domain error as content: {text}"
    );
}

#[test]
fn dream_consolidate_promote_reject_missing_params_as_invalid_params() {
    let (_dir, storage) = setup_storage();
    let err = call("dream_consolidate", json!({"session_key": "s"}), &storage)
        .expect_err("missing now_ms");
    assert_eq!(err["code"], -32602, "JSON-RPC invalid params");
    let err = call("dream_promote", json!({}), &storage).expect_err("missing session_key");
    assert_eq!(err["code"], -32602);
}

#[test]
fn dream_load_missing_run_is_error_content_not_protocol_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "dream_load",
        json!({"session_key": "dream-agent-1", "run_id": "ghost-run-id"}),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("not found"),
        "domain error as content: {text}"
    );
}

#[test]
fn dream_tools_reject_missing_params_as_invalid_params() {
    let (_dir, storage) = setup_storage();
    let err = call("dream_list", json!({}), &storage).expect_err("missing session_key");
    assert_eq!(err["code"], -32602, "JSON-RPC invalid params");
    let err =
        call("dream_load", json!({"session_key": "s"}), &storage).expect_err("missing run_id");
    assert_eq!(err["code"], -32602);
    let err =
        call("dream_discard", json!({"session_key": "s"}), &storage).expect_err("missing run_id");
    assert_eq!(err["code"], -32602);
}
