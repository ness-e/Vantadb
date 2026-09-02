// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-32 — D19 tests for the `code_*` MCP tools.
//!
//! Contract: 8 tools (`code_search/explore/callers/callees/impact/node/status/files`)
//! respond over a seeded graph, respect edge direction, are read-only (no
//! mutation), and an unknown tool yields a clear error.

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::sdk::VantaMemoryInput;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig};

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

fn call(
    name: &str,
    args: Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let executor = Executor::new(storage);
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": args })),
        &executor,
        storage,
        config,
    )
}

/// Message of a tool result (error_content text or JSON-RPC error message).
fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

/// Seed a diamond graph in namespace "code": root→left, root→right,
/// left→sink, right→sink ("uses"/"enables" labels).
/// Returns node ids [root, left, right, sink].
fn seed_graph(storage: &Arc<StorageEngine>) -> [u128; 4] {
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    // MCP-01/AUD-044 pattern: StorageEngine::open alone leaves the text_index
    // registry missing, and graphrag's BM25 seeding fails with
    // "text_index not found: bm25" unless indexes are ensured first.
    embedded
        .ensure_indexes_current()
        .expect("startup index ensure should succeed");
    let mut ids = Vec::new();
    for i in 0..4 {
        let input = VantaMemoryInput::new(
            "code",
            format!("n{i}"),
            format!("node {i} about vector database"),
        );
        ids.push(embedded.put(input).expect("put").node_id);
    }
    embedded
        .add_edge(ids[0], ids[1], "uses", Some(1.0), None)
        .unwrap();
    embedded
        .add_edge(ids[0], ids[2], "uses", Some(1.0), None)
        .unwrap();
    embedded
        .add_edge(ids[1], ids[3], "enables", Some(1.0), None)
        .unwrap();
    embedded
        .add_edge(ids[2], ids[3], "enables", Some(1.0), None)
        .unwrap();
    [ids[0], ids[1], ids[2], ids[3]]
}

#[test]
fn test_all_eight_code_tools_listed() {
    let list = handle_tools_list(&McpConfig::default()).expect("tools/list");
    let names: Vec<&str> = list["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for tool in [
        "code_search",
        "code_explore",
        "code_callers",
        "code_callees",
        "code_impact",
        "code_node",
        "code_status",
        "code_files",
    ] {
        assert!(names.contains(&tool), "{tool} should be listed");
    }
}

#[test]
fn test_code_callees_respect_direction() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let [root, left, right, sink] = seed_graph(&storage);

    let text = msg(call(
        "code_callees",
        json!({"node_id": root.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        text.contains(&left.to_string()),
        "callee left missing: {text}"
    );
    assert!(
        text.contains(&right.to_string()),
        "callee right missing: {text}"
    );
    assert!(!text.contains("\"count\":0"), "root must have callees");

    // Direction check: the sink has no outgoing edges.
    let leaf = msg(call(
        "code_callees",
        json!({"node_id": sink.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        leaf.contains("\"count\":0"),
        "sink has no outgoing edges: {leaf}"
    );
}

#[test]
fn test_code_callers_reverse_direction() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let [root, left, right, sink] = seed_graph(&storage);

    // Callers of the sink are left/right (incoming edges only).
    let text = msg(call(
        "code_callers",
        json!({"node_id": sink.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        text.contains(&left.to_string()),
        "caller left missing: {text}"
    );
    assert!(
        text.contains(&right.to_string()),
        "caller right missing: {text}"
    );

    // The root has no incoming edges → direction respected.
    let root_callers = msg(call(
        "code_callers",
        json!({"node_id": root.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        root_callers.contains("\"count\":0"),
        "root must have no callers: {root_callers}"
    );
}

#[test]
fn test_code_impact_forward_reachable_subgraph() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let [root, _left, _right, sink] = seed_graph(&storage);

    let text = msg(call(
        "code_impact",
        json!({"node_id": root.to_string(), "max_depth": 5}),
        &storage,
        &cfg,
    ));
    assert!(
        text.contains("\"direction\":\"Forward\""),
        "default direction: {text}"
    );
    assert!(
        text.contains(&sink.to_string()),
        "sink reachable from root: {text}"
    );

    // Reverse impact from the sink reaches back to the root.
    let rev = msg(call(
        "code_impact",
        json!({"node_id": sink.to_string(), "max_depth": 5, "direction": "Reverse"}),
        &storage,
        &cfg,
    ));
    assert!(rev.contains("\"direction\":\"Reverse\""), "{rev}");
    assert!(
        rev.contains(&root.to_string()),
        "root reachable backwards: {rev}"
    );

    // Invalid direction → clear error, no panic.
    let bad = msg(call(
        "code_impact",
        json!({"node_id": root.to_string(), "direction": "Sideways"}),
        &storage,
        &cfg,
    ));
    assert!(bad.contains("Invalid direction"), "clear error: {bad}");
}

#[test]
fn test_code_node_and_explore() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let [root, left, _right, _sink] = seed_graph(&storage);

    let text = msg(call(
        "code_node",
        json!({"node_id": root.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        text.contains(&root.to_string()),
        "record id present: {text}"
    );
    assert!(text.contains("vector database"), "payload content present");

    let explore = msg(call(
        "code_explore",
        json!({"node_id": root.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        explore.contains("outgoing") && explore.contains("incoming"),
        "explore shape: {explore}"
    );
    assert!(explore.contains(&left.to_string()), "neighbor listed");

    // Missing node → clear domain error, not panic.
    let missing = msg(call(
        "code_node",
        json!({"node_id": "999999"}),
        &storage,
        &cfg,
    ));
    assert!(missing.contains("Node not found"), "{missing}");
}

#[test]
fn test_code_search_over_seeded_graph() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    seed_graph(&storage);

    let text = msg(call(
        "code_search",
        json!({"namespace": "code", "query": "vector database"}),
        &storage,
        &cfg,
    ));
    assert!(!text.contains("Error"), "no error on seeded graph: {text}");
    assert!(
        text.contains("nodes") && text.contains("stats"),
        "shape: {text}"
    );
}

#[test]
fn test_code_status_and_files_stub() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let status = msg(call("code_status", json!({}), &storage, &cfg));
    assert!(status.contains("metrics"), "status shape: {status}");
    assert!(status.contains("read_only"), "capabilities reported");

    let files = msg(call("code_files", json!({}), &storage, &cfg));
    assert!(
        files.to_lowercase().contains("not supported"),
        "documented stub: {files}"
    );
}

#[test]
fn test_code_tools_are_read_only() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let ids = seed_graph(&storage);

    // Structural fingerprint of every graph view the tools expose.
    // NOTE: node `hits`/`last_accessed` are bumped by ANY read channel
    // (core AccessTracker, same as memory_get) — that's telemetry, not
    // mutation, so the fingerprint covers ids/edges/counts only.
    let fingerprint = || -> String {
        let mut out = String::new();
        for id in ids {
            let id_str = id.to_string();
            for tool in ["code_node", "code_explore", "code_callers", "code_callees"] {
                let res = call(tool, json!({"node_id": id_str}), &storage, &cfg);
                let text = msg(res);
                let v: Value =
                    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{tool}: {e} in {text}"));
                out.push_str(&format!(
                    "{tool}:{id_str}:{}",
                    v.to_string()
                        .replace(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'], "")
                ));
            }
        }
        out
    };

    let first = fingerprint();
    let second = fingerprint();
    assert_eq!(first, second, "graph structure stable across reads");

    // Impact reachability is also stable.
    for dir in ["Forward", "Reverse"] {
        let root = if dir == "Forward" { ids[0] } else { ids[3] };
        let args = json!({"node_id": root.to_string(), "max_depth": 5, "direction": dir});
        let t1 = msg(call("code_impact", args.clone(), &storage, &cfg));
        let t2 = msg(call("code_impact", args, &storage, &cfg));
        assert_eq!(
            t1.split("\"count\"").nth(1),
            t2.split("\"count\"").nth(1),
            "impact {dir} stable"
        );
        assert!(!t1.contains("\"count\":0"), "{dir} impact non-empty");
    }

    // Node count unchanged after all reads.
    let status = msg(call("code_status", json!({}), &storage, &cfg));
    assert!(status.contains("\"hnsw_nodes_count\":4"), "{status}");
}

#[test]
fn test_unknown_code_tool_clear_error() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    let err = msg(call("code_nonexistent", json!({}), &storage, &cfg));
    assert!(err.contains("Tool not found"), "clear error: {err}");
}
