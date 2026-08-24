//! MCP-30: `scene_read` / `scene_list` / `scene_query` — MCP exposure of the
//! vanta-memory gateway knowledge handlers (structured scene navigation).
//!
//! Round-trips go through the public `handle_tools_call` API with scenes
//! seeded via vanta-memory's own `upsert_scene` (the same fixture its gateway
//! unit tests use — no L0 pipeline needed), mirroring how external agents
//! consume the tool.

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig};

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
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

/// Text of a tool result (`text_content` payload) or JSON-RPC error message.
fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

/// Parse a successful tool result's text content as JSON.
fn result_json(res: Result<Value, Value>) -> Value {
    let text = msg(res);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("result is not JSON: {e}\n{text}"))
}

/// Seed one live scene block for `session` through the public vanta-memory API.
fn seed_scene(
    storage: &Arc<StorageEngine>,
    session: &str,
    name: &str,
    summary: &str,
    content: &str,
) {
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    vanta_memory::core::scene::scene_index::upsert_scene(&db, session, name, summary, content)
        .expect("seed scene");
}

// ── tools/list ───────────────────────────────────────────────────────────

#[test]
fn tools_list_registers_scene_tools_with_valid_schemas() {
    let (_dir, _storage) = setup_storage();
    let res = handle_tools_list().expect("tools/list");
    let tools = res["tools"].as_array().expect("tools array");
    for name in ["scene_read", "scene_list", "scene_query"] {
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
    }
}

// ── round-trip: seeded session ───────────────────────────────────────────

#[test]
fn scene_list_roundtrip_lists_seeded_scenes_heat_desc() {
    let (_dir, storage) = setup_storage();
    seed_scene(
        &storage,
        "agent-1",
        "research",
        "pricing research",
        "user researched pricing tiers",
    );
    // Second write bumps deploy's heat to 2 → it must sort first.
    seed_scene(
        &storage,
        "agent-1",
        "deploy",
        "deployment notes",
        "we deploy with cargo",
    );
    seed_scene(
        &storage,
        "agent-1",
        "deploy",
        "deployment notes",
        "we deploy with cargo",
    );

    let scenes = result_json(call(
        "scene_list",
        json!({"session_key": "agent-1"}),
        &storage,
    ))["scenes"]
        .as_array()
        .expect("scenes array")
        .clone();
    assert!(!scenes.is_empty(), "seeded session must list > 0 scenes");
    assert_eq!(scenes[0]["filename"], "deploy", "hottest scene first");
    assert_eq!(scenes[0]["heat"], 2);
    assert_eq!(scenes.len(), 2);
}

#[test]
fn scene_read_roundtrip_returns_block_content() {
    let (_dir, storage) = setup_storage();
    seed_scene(
        &storage,
        "agent-1",
        "deploy",
        "deployment notes",
        "we deploy with cargo and docker",
    );

    let listed = result_json(call(
        "scene_list",
        json!({"session_key": "agent-1"}),
        &storage,
    ));
    let name = listed["scenes"][0]["filename"]
        .as_str()
        .expect("filename id");

    let scene = result_json(call(
        "scene_read",
        json!({"session_key": "agent-1", "scene_name": name}),
        &storage,
    ))["scene"]
        .clone();
    assert_eq!(scene["scene_name"], "deploy");
    assert_eq!(scene["meta"]["summary"], "deployment notes");
    assert!(
        scene["content"]
            .as_str()
            .unwrap_or_default()
            .contains("cargo"),
        "block content must travel back to the agent"
    );
}

#[test]
fn scene_query_finds_scene_by_keyword_and_reads_it() {
    let (_dir, storage) = setup_storage();
    seed_scene(
        &storage,
        "agent-1",
        "deploy",
        "deployment notes",
        "we deploy with cargo and docker",
    );
    seed_scene(
        &storage,
        "agent-1",
        "offtopic",
        "cooking",
        "pasta recipe with tomato",
    );

    let hits = result_json(call(
        "scene_query",
        json!({"session_key": "agent-1", "keyword": "docker"}),
        &storage,
    ))["hits"]
        .as_array()
        .expect("hits array")
        .clone();
    assert_eq!(hits.len(), 1, "only the matching scene surfaces");
    assert_eq!(hits[0]["scene_name"], "deploy");

    // Navigation loop closes: hit id feeds scene_read.
    let scene = result_json(call(
        "scene_read",
        json!({"session_key": "agent-1", "scene_name": hits[0]["scene_name"]}),
        &storage,
    ));
    assert!(
        scene["scene"]["content"]
            .as_str()
            .unwrap_or_default()
            .contains("docker"),
        "hit id must resolve to the block that matched"
    );
}

// ── error contract ───────────────────────────────────────────────────────

#[test]
fn scene_read_missing_scene_is_error_content_not_protocol_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "scene_read",
        json!({"session_key": "agent-1", "scene_name": "ghost"}),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("not found"),
        "domain error as content: {text}"
    );
}

#[test]
fn scene_tools_reject_missing_params_as_invalid_params() {
    let (_dir, storage) = setup_storage();
    let err = call("scene_read", json!({}), &storage).expect_err("missing session_key");
    assert_eq!(err["code"], -32602, "JSON-RPC invalid params");
    let err =
        call("scene_query", json!({"session_key": "s"}), &storage).expect_err("missing keyword");
    assert_eq!(err["code"], -32602);
}

#[test]
fn scene_tools_reject_empty_session_and_keyword_via_domain_errors() {
    let (_dir, storage) = setup_storage();
    let res = call("scene_list", json!({"session_key": "   "}), &storage);
    assert!(
        res.is_ok(),
        "empty session_key is a domain error, not protocol"
    );
    assert!(msg(res).to_lowercase().contains("invalid"));

    let res = call(
        "scene_query",
        json!({"session_key": "s", "keyword": "   "}),
        &storage,
    );
    assert!(res.is_ok());
    assert!(msg(res).to_lowercase().contains("invalid"));
}
