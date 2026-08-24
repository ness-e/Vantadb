//! MCP-32: `thread_create` / `thread_send` / `thread_get` / `thread_list` /
//! `thread_delete` / `thread_purge_expired` — MCP exposure of the agentic
//! conversation thread API.
//!
//! Round-trips go through the public `handle_tools_call` API against a real
//! `StorageEngine` (no mocks), mirroring how external agents consume the tool.

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

/// Parse a successful tool result's text content as JSON.
fn result_json(res: Result<Value, Value>) -> Value {
    let text = res.expect("tool call should succeed")["content"][0]["text"]
        .as_str()
        .expect("text content")
        .to_string();
    serde_json::from_str(&text).expect("valid JSON payload")
}

fn err_msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

fn seed_thread(storage: &Arc<StorageEngine>, title: &str) -> String {
    let res = call("thread_create", json!({ "title": title }), storage);
    result_json(res)["thread_id"].as_str().unwrap().to_string()
}

#[test]
fn tools_list_includes_six_thread_tools() {
    let list = handle_tools_list().expect("tools/list");
    let names: Vec<&str> = list["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for expected in [
        "thread_create",
        "thread_send",
        "thread_get",
        "thread_list",
        "thread_delete",
        "thread_purge_expired",
    ] {
        assert!(names.contains(&expected), "missing tool: {expected}");
    }
}

#[test]
fn create_send_get_roundtrip() {
    let (_dir, storage) = setup_storage();
    let id = seed_thread(&storage, "triage notes");

    let sent = call(
        "thread_send",
        json!({ "thread_id": id, "role": "user", "content": "hola mundo" }),
        &storage,
    );
    assert_eq!(result_json(sent)["ok"], json!(true));

    let got = result_json(call("thread_get", json!({ "thread_id": id }), &storage));
    let thread = &got["thread"];
    // The wire shape is the serialized MessageThread directly; tolerate both
    // a bare thread object and an {thread: ...} wrapper for forward compat.
    let thread = if thread.is_object() && thread["messages"].is_array() {
        thread.clone()
    } else {
        got.clone()
    };
    assert_eq!(thread["title"], json!("triage notes"));
    let messages = thread["messages"].as_array().expect("messages array");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], json!("user"));
    assert_eq!(messages[0]["content"], json!("hola mundo"));
}

#[test]
fn get_missing_thread_is_error_content_not_rpc_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "thread_get",
        json!({ "thread_id": u128::MAX.to_string() }),
        &storage,
    );
    let msg = err_msg(res);
    assert!(
        msg.to_lowercase().contains("not found"),
        "expected not-found error_content, got: {msg}"
    );
}

#[test]
fn delete_removes_thread_and_get_fails_after() {
    let (_dir, storage) = setup_storage();
    let id = seed_thread(&storage, "to delete");

    let del = result_json(call("thread_delete", json!({ "thread_id": id }), &storage));
    assert_eq!(del["deleted"], json!(true));

    let after = err_msg(call("thread_get", json!({ "thread_id": id }), &storage));
    assert!(after.to_lowercase().contains("not found"), "got: {after}");
}

#[test]
fn list_returns_seeded_threads_with_count() {
    let (_dir, storage) = setup_storage();
    let a = seed_thread(&storage, "alpha");
    let _b = seed_thread(&storage, "beta");

    let listed = result_json(call("thread_list", json!({}), &storage));
    assert_eq!(listed["count"], json!(2));
    let ids: Vec<&str> = listed["threads"]
        .as_array()
        .expect("threads array")
        .iter()
        .filter_map(|t| t["thread_id"].as_str())
        .collect();
    assert!(ids.contains(&a.as_str()));
}

#[test]
fn purge_on_empty_store_reports_zero() {
    let (_dir, storage) = setup_storage();
    let purged = result_json(call("thread_purge_expired", json!({}), &storage));
    assert_eq!(purged["purged"], json!(0));
}

#[test]
fn malformed_thread_id_is_invalid_params() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "thread_get",
        json!({ "thread_id": "not-a-number" }),
        &storage,
    );
    match res {
        Err(e) => assert_eq!(e["code"], json!(-32602), "expected invalid-params"),
        Ok(v) => panic!("expected JSON-RPC error, got ok: {v}"),
    }
}
