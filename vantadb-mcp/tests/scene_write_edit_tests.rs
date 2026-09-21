// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// FIND-107 S1: `scene_write` / `scene_edit` — MCP exposure of the vanta-memory
// sandboxed scene write path (MEM-13 tool layer).
//
// Round-trips go through the public `handle_tools_call` API, mirroring how
// external agents consume the tool (same pattern as `scene_tests.rs`).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig};

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
        &McpConfig::default(),
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

// ── tools/list ───────────────────────────────────────────────────────────

#[test]
fn tools_list_registers_scene_write_edit_with_valid_schemas() {
    let (_dir, _storage) = setup_storage();
    let res = handle_tools_list(&McpConfig::default()).expect("tools/list");
    let tools = res["tools"].as_array().expect("tools array");
    for name in ["scene_write", "scene_edit"] {
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
            .find(|t| t["name"] == json!("scene_write"))
            .expect("scene_write")["annotations"]["readOnlyHint"],
        false
    );
}

// ── round-trip ───────────────────────────────────────────────────────────

#[test]
fn scene_write_creates_block_readable_via_scene_read() {
    let (_dir, storage) = setup_storage();
    let written = result_json(call(
        "scene_write",
        json!({
            "session_key": "agent-1",
            "scene_name": "deploy",
            "summary": "deployment notes",
            "content": "we deploy with cargo and docker",
        }),
        &storage,
    ));
    assert_eq!(written["scene"]["scene_name"], "deploy");

    let scene = result_json(call(
        "scene_read",
        json!({"session_key": "agent-1", "scene_name": "deploy"}),
        &storage,
    ));
    assert!(scene["scene"]["content"]
        .as_str()
        .unwrap_or_default()
        .contains("docker"));
}

#[test]
fn scene_edit_patches_summary_and_keeps_content() {
    let (_dir, storage) = setup_storage();
    result_json(call(
        "scene_write",
        json!({
            "session_key": "agent-1",
            "scene_name": "deploy",
            "summary": "old summary",
            "content": "we deploy with cargo",
        }),
        &storage,
    ));
    let edited = result_json(call(
        "scene_edit",
        json!({
            "session_key": "agent-1",
            "scene_name": "deploy",
            "summary": "new summary",
        }),
        &storage,
    ));
    assert_eq!(edited["scene"]["meta"]["summary"], "new summary");
    assert!(edited["scene"]["content"]
        .as_str()
        .unwrap_or_default()
        .contains("cargo"));
}

// ── error contract ───────────────────────────────────────────────────────

#[test]
fn scene_edit_missing_scene_is_error_content_not_protocol_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "scene_edit",
        json!({"session_key": "agent-1", "scene_name": "ghost", "summary": "x"}),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("not found"),
        "domain error as content: {text}"
    );
}

#[test]
fn scene_write_edit_reject_bad_params() {
    let (_dir, storage) = setup_storage();
    // Missing content → JSON-RPC invalid params.
    let err = call(
        "scene_write",
        json!({"session_key": "s", "scene_name": "n", "summary": "sum"}),
        &storage,
    )
    .expect_err("missing content");
    assert_eq!(err["code"], -32602, "JSON-RPC invalid params");

    // Whitespace-only content → domain error as content (TDAM guard).
    let res = call(
        "scene_write",
        json!({"session_key": "s", "scene_name": "n", "summary": "sum", "content": "   "}),
        &storage,
    );
    assert!(res.is_ok());
    assert!(msg(res).to_lowercase().contains("invalid"));

    // Edit without fields → domain error as content.
    let res = call(
        "scene_edit",
        json!({"session_key": "s", "scene_name": "n"}),
        &storage,
    );
    assert!(res.is_ok());
    assert!(msg(res).to_lowercase().contains("invalid"));
}
