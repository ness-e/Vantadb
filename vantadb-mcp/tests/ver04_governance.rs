// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! VER-04: injection governance on the MCP surfaces —
//! `inject_context` budget (fail-closed) + `memory_recall`/`context_assemble`
//! envelopes (`byte_count`/`truncated`), namespace ACL denials and the
//! metadata-only injection audit from `VANTADB_MCP_AUDIT_LOG`-equivalent
//! config.

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, McpConfig, McpProfile};

fn base_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Full,
        ..Default::default()
    }
}

fn audit_config(path: &std::path::Path, extra: McpConfig) -> McpConfig {
    let logger =
        vantadb::audit::AuditLogger::with_rotation(path, 1024 * 1024, 2).expect("audit logger");
    McpConfig {
        audit: Some(Arc::new(logger)),
        ..extra
    }
}

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

fn call_cfg(
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

fn result_json(res: Result<Value, Value>) -> Value {
    let v = res.expect("tool call ok");
    assert!(
        v["isError"].is_null() || v["isError"] == false,
        "tool reported error: {v}"
    );
    v["content"][0]["text"]
        .as_str()
        .map(|t| serde_json::from_str(t).expect("result text is JSON"))
        .unwrap_or_else(|| v["structuredContent"].clone())
}

fn audit_rows(path: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("jsonl line"))
        .collect()
}

/// Persist one L1 record for `session` (same shape the pipeline writes).
fn seed_l1(storage: &Arc<StorageEngine>, session: &str, id: &str, content: &str) {
    use vantadb::sdk::MemoryInput;
    let record = vanta_memory::core::abstractions::MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: vanta_memory::core::abstractions::MemoryType::WorkFact,
        priority: 50,
        scene_name: String::new(),
        source_message_ids: vec![],
        metadata: Value::Null,
        timestamps: vec![],
        created_at: "2026-09-29T00:00:00Z".into(),
        updated_at: "2026-09-29T00:00:00Z".into(),
        version: 1,
        session_key: session.into(),
        session_id: String::new(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    };
    let db = vantadb::Embedded::from_engine(storage.clone());
    db.put(MemoryInput {
        namespace: vanta_memory::core::record::l1_reader::l1_namespace(session),
        key: id.into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: vantadb::sdk::MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed l1");
}

// ── c1: inject_context budget (fail-closed, declared) ───────────────────────
#[test]
fn inject_context_rejects_content_over_the_byte_budget() {
    let (_dir, storage) = setup_storage();
    let config = McpConfig {
        byte_budget: 1024,
        ..base_config()
    };
    let res = call_cfg(
        "inject_context",
        json!({ "content": "x".repeat(2048), "thread_id": "1" }),
        &storage,
        &config,
    );
    let err = res.expect_err("over-budget injection must fail closed");
    let message = err["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("injection budget"),
        "error must name the budget: {err}"
    );
    assert!(
        message.contains("VANTADB_MCP_BYTE_BUDGET"),
        "error must name the knob: {err}"
    );
}

#[test]
fn inject_context_within_budget_reports_byte_count_and_audits_metadata_only() {
    let (_dir, storage) = setup_storage();
    let audit_dir = tempdir().unwrap();
    let audit_path = audit_dir.path().join("audit.jsonl");
    let config = audit_config(
        &audit_path,
        McpConfig {
            byte_budget: 4096,
            ..base_config()
        },
    );
    let content = "thread context body";
    let v = result_json(call_cfg(
        "inject_context",
        json!({ "content": content, "thread_id": "7" }),
        &storage,
        &config,
    ));
    assert_eq!(v["byte_count"], content.len());
    assert_eq!(v["truncated"], false);
    assert_eq!(v["status"], "Context Anchored");

    let rows = audit_rows(&audit_path);
    assert_eq!(rows.len(), 1, "one event per injection: {rows:?}");
    assert_eq!(rows[0]["op"], "injection");
    assert_eq!(rows[0]["namespace"], "threads");
    assert_eq!(rows[0]["key"], "7");
    let reason = rows[0]["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("tool=inject_context"), "{reason}");
    assert!(reason.contains("budget=19/4096"), "{reason}");
    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    assert!(
        !raw.contains(content),
        "injected content leaked into the audit: {raw}"
    );
}

// ── c1/c2: memory_recall envelope + ACL + audit ─────────────────────────────
#[test]
fn memory_recall_envelope_carries_byte_count_and_source_identity() {
    let (_dir, storage) = setup_storage();
    seed_l1(&storage, "mcp", "m1", "user prefers dark mode");
    let v = result_json(call_cfg(
        "memory_recall",
        json!({ "query": "dark mode" }),
        &storage,
        &base_config(),
    ));
    assert_eq!(v["truncated"], false);
    assert!(v["byte_count"].as_u64().unwrap_or(0) > 0);
    assert_eq!(v["recalled"][0]["source_namespace"], "l1/mcp");
    assert_eq!(v["recalled"][0]["source_key"], "m1");
}

#[test]
fn memory_recall_truncates_when_the_response_exceeds_the_byte_budget() {
    let (_dir, storage) = setup_storage();
    // One long hit: the char cap keeps ~budget chars, the envelope overhead
    // pushes it past the byte budget → truncated, never a silent oversize.
    seed_l1(&storage, "mcp", "m1", &"dark mode ".repeat(400));
    let config = McpConfig {
        byte_budget: 1024,
        ..base_config()
    };
    let v = result_json(call_cfg(
        "memory_recall",
        json!({ "query": "dark mode" }),
        &storage,
        &config,
    ));
    assert_eq!(v["truncated"], true, "oversize recall must flag truncated");
}

#[test]
fn memory_recall_acl_denies_out_of_scope_namespaces_and_audits_the_denial() {
    let (_dir, storage) = setup_storage();
    seed_l1(&storage, "mcp", "m1", "user prefers dark mode");
    seed_l1(&storage, "other", "m2", "user prefers dark mode too");
    let audit_dir = tempdir().unwrap();
    let audit_path = audit_dir.path().join("audit.jsonl");
    let config = audit_config(
        &audit_path,
        McpConfig {
            injection_namespaces: vec!["l1/mcp".into()],
            ..base_config()
        },
    );
    let v = result_json(call_cfg(
        "memory_recall",
        json!({ "query": "dark mode" }),
        &storage,
        &config,
    ));
    let recalled = v["recalled"].as_array().cloned().unwrap_or_default();
    assert!(
        recalled.iter().all(|r| r["source_namespace"] == "l1/mcp"),
        "out-of-scope namespaces must not be injected: {v}"
    );
    let rows = audit_rows(&audit_path);
    let denied = rows
        .iter()
        .find(|r| r["outcome"] == "denied")
        .expect("ACL denial must be audited (never silent)");
    assert_eq!(denied["namespace"], "l1/other");
    assert!(denied["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("acl=deny"));
}

// ── c1: context_assemble envelope coherence ─────────────────────────────────
#[test]
fn context_assemble_envelope_carries_byte_count_and_truncated() {
    let (_dir, storage) = setup_storage();
    let v = result_json(call_cfg(
        "context_assemble",
        json!({
            "session_key": "s1",
            "token_budget": 512,
            "messages": [{ "role": "user", "content": "hello" }]
        }),
        &storage,
        &base_config(),
    ));
    assert!(v["messages"].is_array(), "context object intact: {v}");
    assert!(v["report"].is_object(), "report intact: {v}");
    assert!(v["byte_count"].as_u64().unwrap_or(0) > 0);
    assert_eq!(v["truncated"], false);

    // A history that must be compacted under a tiny budget → truncated=true.
    let messages: Vec<Value> = (0..20)
        .map(|i| json!({ "role": "user", "content": format!("message {i} {}", "x".repeat(200)) }))
        .collect();
    let v = result_json(call_cfg(
        "context_assemble",
        json!({ "session_key": "s1", "token_budget": 200, "messages": messages }),
        &storage,
        &base_config(),
    ));
    assert_eq!(
        v["truncated"], true,
        "compacted history must flag truncated: {v}"
    );
}

// ── Review F1: deny-all recall still audits the ACL denials ─────────────────
#[test]
fn memory_recall_deny_all_still_audits_denials() {
    let (_dir, storage) = setup_storage();
    seed_l1(&storage, "mcp", "m1", "user prefers dark mode");
    let audit_dir = tempdir().unwrap();
    let audit_path = audit_dir.path().join("audit.jsonl");
    // ACL allows only a foreign prefix → every recall source is denied.
    let config = audit_config(
        &audit_path,
        McpConfig {
            injection_namespaces: vec!["other/".into()],
            ..base_config()
        },
    );
    let v = result_json(call_cfg(
        "memory_recall",
        json!({ "query": "dark mode" }),
        &storage,
        &config,
    ));
    assert_eq!(v["prepend_context"], Value::Null, "nothing may inject: {v}");
    assert!(
        v["recalled"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true),
        "nothing may inject: {v}"
    );

    // The deny-all pass MUST still produce audit rows (review F1: without
    // them the ACL is silent whenever it blocks everything).
    let rows = audit_rows(&audit_path);
    let denied: Vec<&Value> = rows.iter().filter(|r| r["outcome"] == "denied").collect();
    assert!(
        !denied.is_empty(),
        "deny-all must still audit its denials: {rows:?}"
    );
    assert!(
        denied.iter().any(|r| r["namespace"] == "l1/mcp"),
        "L1 denial must be present: {rows:?}"
    );
    assert!(
        denied.iter().all(|r| r["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("acl=deny")),
        "{rows:?}"
    );
}
