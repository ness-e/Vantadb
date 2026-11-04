// ponytail: blanket allow - unwraps with documented invariants.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for ADRs / Conventions & Bug->Fix MCP tools (PI-2/PI-3, MGR-23/MGR-24).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn full_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Full,
        ..Default::default()
    }
}

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

fn tool_call(
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

fn tool_text(res: Result<Value, Value>) -> Value {
    let v = res.expect("tool call should succeed");
    assert_ne!(v["isError"], true, "tool call returned an error: {:?}", v);
    let text = v["content"][0]["text"]
        .as_str()
        .expect("text content present");
    serde_json::from_str(text).expect("text content is serialized JSON")
}

#[test]
fn test_adr_mcp_put_and_recall_filtering() {
    let (_dir, storage) = setup_storage();
    let config = full_config();

    // 1. Put active ADR via memory_put
    let _ = tool_text(tool_call(
        "memory_put",
        json!({
            "namespace": "l1/mcp",
            "key": "adr-046",
            "payload": "[ADR:ADR-046] Schema v2 single migration\nContext: Clean cut without dual schema debt",
            "category": "adr",
            "decision_id": "ADR-046",
            "status": "accepted",
            "decision": "Schema v2 single migration",
            "context": "Clean cut without dual schema debt",
            "modules": ["wire", "schema"]
        }),
        &storage,
        &config,
    ));

    // 2. Put superseded ADR via memory_put
    let _ = tool_text(tool_call(
        "memory_put",
        json!({
            "namespace": "l1/mcp",
            "key": "adr-045",
            "payload": "[ADR:ADR-045] Dual schema v1/v2 support\nContext: Legacy transition period",
            "category": "adr",
            "decision_id": "ADR-045",
            "status": "superseded",
            "superseded_by": "ADR-046",
            "decision": "Dual schema v1/v2 support",
            "context": "Legacy transition period",
            "modules": ["wire"]
        }),
        &storage,
        &config,
    ));

    // 3. Recall with query touching the "wire" module
    let json_recalled = tool_text(tool_call(
        "memory_recall",
        json!({
            "query": "wire schema migration decision",
            "scope": "agent"
        }),
        &storage,
        &config,
    ));

    let memories = json_recalled["recalled"]
        .as_array()
        .expect("recalled array");

    // Must recall active ADR-046
    assert!(!memories.is_empty(), "Must recall memories");
    let has_active = memories
        .iter()
        .any(|m| m["content"].as_str().unwrap_or("").contains("ADR-046"));
    assert!(has_active, "Active ADR-046 must be recalled");

    // Must NOT recall superseded ADR-045
    let has_superseded = memories
        .iter()
        .any(|m| m["content"].as_str().unwrap_or("").contains("ADR-045"));
    assert!(
        !has_superseded,
        "Superseded ADR-045 must be excluded from recall results"
    );
}

#[test]
fn test_bugfix_mcp_record_and_query() {
    let (_dir, storage) = setup_storage();
    let config = full_config();

    // 1. Record bugfix via bugfix_record tool
    let val = tool_text(tool_call(
        "bugfix_record",
        json!({
            "id": "bf_oom_batch",
            "symptom": "OOM crash on batch import when streaming large files",
            "root_cause": "Unbounded channel buffer in worker sink",
            "fix_summary": "Add bounded backpressure with bounded crossbeam channel",
            "files_touched": ["src/ingest/worker.rs", "src/ingest/sink.rs"],
            "regression_test": "test_batch_backpressure_memory_bound"
        }),
        &storage,
        &config,
    ));
    assert_eq!(val["status"], "recorded");

    // 2. Query bugfix by symptom substring
    let query_val = tool_text(tool_call(
        "bugfix_query",
        json!({
            "query": "OOM crash batch import"
        }),
        &storage,
        &config,
    ));
    let matches = query_val["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["id"], "bf_oom_batch");
    assert_eq!(
        matches[0]["root_cause"],
        "Unbounded channel buffer in worker sink"
    );

    // 3. Query bugfix by affected file
    let file_val = tool_text(tool_call(
        "bugfix_query",
        json!({
            "file": "src/ingest/worker.rs"
        }),
        &storage,
        &config,
    ));
    let file_matches = file_val["matches"].as_array().unwrap();
    assert_eq!(file_matches.len(), 1);
    assert_eq!(file_matches[0]["id"], "bf_oom_batch");
}
