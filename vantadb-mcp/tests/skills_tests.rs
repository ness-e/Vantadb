// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 — dedicated integration tests for the MEM-07 `skill_*` MCP tools.
//!
//! Coverage: tools/list registration, CRUD roundtrip, owner scope + owner
//! check hiding existence (404 without leaking), optimistic-lock
//! expected_version, substring patch semantics, resource manifest + size
//! limits + path validation, and parity with the native core `SkillStore`
//! (D13 — MCP is a thin wrapper, both channels observe the same state).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::sdk::SkillUpdateInput;
use vantadb::skills::SkillStore;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

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

/// Parse the JSON text of a successful tool result.
fn tool_text(res: Result<Value, Value>) -> Value {
    let v = res.expect("tool call should succeed");
    let text = v["content"][0]["text"]
        .as_str()
        .expect("text content present");
    serde_json::from_str(text).expect("text content is serialized JSON")
}

/// Extract the message of a tool error (error_content or JSON-RPC error).
fn tool_error(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

fn create_skill(
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
    owner: &str,
    name: &str,
    content: &str,
) -> String {
    let res = tool_call(
        "skill_create",
        json!({
            "name": name,
            "owner_agent": owner,
            "content": content,
        }),
        storage,
        config,
    );
    let body = tool_text(res);
    assert_eq!(body["ok"], true, "create {name} should succeed");
    assert_eq!(body["version"], 1);
    body["skill_id"].as_str().unwrap().to_string()
}

// ── Registration (tools/list) ───────────────────────────────────────────────

#[test]
fn test_tools_list_includes_skill_tools() {
    let res = handle_tools_list(&McpConfig::default()).expect("tools/list should succeed");
    let tools = res["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "skill_list",
        "skill_view",
        "skill_create",
        "skill_update",
        "skill_patch",
        "skill_files_write",
    ] {
        assert!(
            names.contains(&expected),
            "tools/list must include {expected}"
        );
    }
    // Writes require owner_agent identity and expected_version in the schema.
    let update = tools
        .iter()
        .find(|t| t["name"] == "skill_update")
        .expect("skill_update registered");
    let required: Vec<&str> = update["inputSchema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"owner_agent"));
    assert!(required.contains(&"expected_version"));
}

// ── CRUD roundtrip ──────────────────────────────────────────────────────────

#[test]
fn test_skill_create_view_roundtrip() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let skill_id = create_skill(
        &storage,
        &cfg,
        "agent-alpha",
        "code-review",
        "# Code Review\n\nChecklist.",
    );
    assert!(skill_id.starts_with("skl-"), "generated skill_id prefix");

    let res = tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["skill_id"], skill_id);
    assert_eq!(body["version"], 1);
    assert_eq!(body["name"], "code-review");
    assert_eq!(body["content"], "# Code Review\n\nChecklist.");
    assert_eq!(body["files"].as_array().unwrap().len(), 0);

    // View a specific version (head == v1 here).
    let res = tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha", "version": 1 }),
        &storage,
        &cfg,
    );
    assert_eq!(tool_text(res)["version"], 1);
}

#[test]
fn test_skill_list_scoped_by_owner() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    create_skill(&storage, &cfg, "agent-alpha", "a-skill", "alpha content");
    create_skill(&storage, &cfg, "agent-beta", "b-skill", "beta content");

    let res = tool_call(
        "skill_list",
        json!({ "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    let items = body["items"].as_array().unwrap();
    assert_eq!(body["total"], 1, "only agent-alpha's skill");
    assert_eq!(items[0]["name"], "a-skill");
    assert_eq!(items[0]["skill_id"], body["items"][0]["skill_id"]);

    let res = tool_call(
        "skill_list",
        json!({ "owner_agent": "agent-beta", "name_prefix": "b-" }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], "b-skill");

    // Prefix that matches nothing.
    let res = tool_call(
        "skill_list",
        json!({ "owner_agent": "agent-beta", "name_prefix": "zzz" }),
        &storage,
        &cfg,
    );
    assert_eq!(tool_text(res)["total"], 0);
}

// ── Writes: optimistic lock + idempotency ───────────────────────────────────

#[test]
fn test_skill_update_optimistic_lock_and_idempotent() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "editable", "# v1");

    // v1 → v2.
    let res = tool_call(
        "skill_update",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "content": "# v2",
            "description": "updated desc",
        }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["ok"], true);
    assert_eq!(body["version"], 2);
    assert_eq!(body["idempotent"], false);

    // Stale expected_version → conflict surfaced as a Skill Error.
    let res = tool_call(
        "skill_update",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "content": "# conflict",
        }),
        &storage,
        &cfg,
    );
    let msg = tool_error(res);
    assert!(
        msg.contains("Skill Error"),
        "stale version must surface a Skill Error, got: {msg}"
    );

    // Same content → idempotent, no new version.
    let res = tool_call(
        "skill_update",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 2,
            "content": "# v2",
            "description": "updated desc",
        }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["version"], 2, "no new version for identical write");
    assert_eq!(body["idempotent"], true);
}

#[test]
fn test_skill_create_idempotent_on_same_content() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let first = tool_text(tool_call(
        "skill_create",
        json!({
            "name": "same",
            "owner_agent": "agent-alpha",
            "content": "# identical",
        }),
        &storage,
        &cfg,
    ));
    let second = tool_text(tool_call(
        "skill_create",
        json!({
            "name": "same",
            "owner_agent": "agent-alpha",
            "content": "# identical",
        }),
        &storage,
        &cfg,
    ));
    assert_eq!(first["skill_id"], second["skill_id"]);
    assert_eq!(first["version"], 1);
    assert_eq!(second["version"], 1, "idempotent create must not bump");
    assert_eq!(second["idempotent"], true);

    // Same name, different content → conflict (unique (owner, name)).
    let conflict = tool_error(tool_call(
        "skill_create",
        json!({
            "name": "same",
            "owner_agent": "agent-alpha",
            "content": "# different",
        }),
        &storage,
        &cfg,
    ));
    assert!(
        conflict.contains("Skill Error"),
        "duplicate name must surface a Skill Error, got: {conflict}"
    );
}

// ── skill_patch (substring semantics) ───────────────────────────────────────

#[test]
fn test_skill_patch_substring_semantics() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "patchme", "aaa bbb aaa");

    // Ambiguous without replace_all.
    let res = tool_call(
        "skill_patch",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "old_string": "aaa",
            "new_string": "zzz",
        }),
        &storage,
        &cfg,
    );
    let msg = tool_error(res);
    assert!(
        msg.contains("occurs 2 times; pass replace_all=true"),
        "ambiguity must be explicit, got: {msg}"
    );

    // replace_all → both occurrences replaced.
    let res = tool_call(
        "skill_patch",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "old_string": "aaa",
            "new_string": "zzz",
            "replace_all": true,
        }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["version"], 2);
    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    assert_eq!(view["content"], "zzz bbb zzz");

    // Single occurrence without replace_all works.
    let res = tool_call(
        "skill_patch",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 2,
            "old_string": "bbb",
            "new_string": "ccc",
        }),
        &storage,
        &cfg,
    );
    assert_eq!(tool_text(res)["version"], 3);
    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    assert_eq!(view["content"], "zzz ccc zzz");

    // Missing substring.
    let res = tool_call(
        "skill_patch",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 3,
            "old_string": "nope",
            "new_string": "x",
        }),
        &storage,
        &cfg,
    );
    assert!(
        tool_error(res).contains("not found"),
        "missing substring must be reported"
    );

    // Empty old_string rejected.
    let res = tool_call(
        "skill_patch",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 3,
            "old_string": "",
            "new_string": "x",
        }),
        &storage,
        &cfg,
    );
    assert!(
        tool_error(res).contains("must not be empty"),
        "empty old_string must be rejected"
    );
}

// ── skill_files_write: manifest, limits, path validation ────────────────────

#[test]
fn test_skill_files_write_manifest() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "file-skill", "# main");

    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "path": "scripts/run.py",
            "content": "print('hi')",
            "mime_type": "text/x-python",
            "is_executable": true,
        }),
        &storage,
        &cfg,
    );
    let body = tool_text(res);
    assert_eq!(body["ok"], true);
    assert_eq!(body["version"], 2, "file write appends a version");

    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    let files = view["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "scripts/run.py");
    assert_eq!(files[0]["content"], "print('hi')");
    assert_eq!(files[0]["encoding"], "utf-8");
    assert_eq!(files[0]["mime_type"], "text/x-python");
    assert_eq!(files[0]["is_executable"], true);

    // Replacing the same path keeps a single manifest entry (version bump).
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 2,
            "path": "scripts/run.py",
            "content": "print('hi v2')",
        }),
        &storage,
        &cfg,
    );
    assert_eq!(tool_text(res)["version"], 3);
    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    let files = view["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["content"], "print('hi v2')");
    assert_eq!(
        files[0]["mime_type"], "",
        "replacement without mime clears it"
    );
}

#[test]
fn test_skill_files_write_limits_and_path_validation() {
    let (_dir, storage) = setup_storage();
    // Small knobs exercise the same enforcement paths as the 5MB/50MB defaults.
    let cfg = McpConfig {
        max_skill_resource_bytes: 60,
        max_skill_total_bytes: 100,
        ..Default::default()
    };
    assert_eq!(McpConfig::default().max_skill_resource_bytes, 5_000_000);
    assert_eq!(McpConfig::default().max_skill_total_bytes, 50_000_000);

    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "limited", &"a".repeat(80));

    // Per-resource limit: 61 bytes > 60.
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "path": "big.bin",
            "content": "x".repeat(61),
        }),
        &storage,
        &cfg,
    );
    let msg = tool_error(res);
    assert!(
        msg.contains("exceeds maximum size of 60 bytes"),
        "per-resource limit must be enforced, got: {msg}"
    );

    // Aggregate limit: content 80 + file 40 = 120 > 100.
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "path": "ok.bin",
            "content": "y".repeat(40),
        }),
        &storage,
        &cfg,
    );
    let msg = tool_error(res);
    assert!(
        msg.contains("exceeds maximum total size of 100 bytes"),
        "aggregate limit must be enforced, got: {msg}"
    );

    // Path traversal / absolute paths rejected.
    for bad_path in ["../evil.py", "/abs.py", "C:/abs.py", "a\\..\\b.txt"] {
        let res = tool_call(
            "skill_files_write",
            json!({
                "skill_id": skill_id,
                "owner_agent": "agent-alpha",
                "expected_version": 1,
                "path": bad_path,
                "content": "data",
            }),
            &storage,
            &cfg,
        );
        let msg = tool_error(res);
        assert!(
            msg.contains("Skill file path"),
            "path '{bad_path}' must be rejected, got: {msg}"
        );
    }

    // Invalid base64 encoding rejected.
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "path": "bad.b64",
            "content": "not valid !!! base64",
            "encoding": "base64",
        }),
        &storage,
        &cfg,
    );
    assert!(
        tool_error(res).contains("not valid base64"),
        "invalid base64 must be rejected"
    );

    // Unsupported encoding rejected.
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 1,
            "path": "x.bin",
            "content": "data",
            "encoding": "utf-16",
        }),
        &storage,
        &cfg,
    );
    assert!(
        tool_error(res).contains("Unsupported skill file encoding"),
        "unsupported encoding must be rejected"
    );
}

// ── Owner check: 404 without leaking existence ──────────────────────────────

#[test]
fn test_skill_owner_check_hides_existence() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "private", "# secret");

    let missing = tool_error(tool_call(
        "skill_view",
        json!({ "skill_id": "skl-doesnotexist", "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    assert_eq!(missing, "Skill not found");

    let not_owned = tool_error(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-beta" }),
        &storage,
        &cfg,
    ));
    assert_eq!(
        missing, not_owned,
        "owner mismatch must be indistinguishable from a missing skill"
    );

    let not_owned_update = tool_error(tool_call(
        "skill_update",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-beta",
            "expected_version": 1,
            "content": "# hijack",
        }),
        &storage,
        &cfg,
    ));
    assert_eq!(not_owned, not_owned_update);

    // The owner still sees it untouched.
    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    assert_eq!(view["content"], "# secret");
}

// ── Parity with the native core SkillStore (D13) ────────────────────────────

#[test]
fn test_skill_mcp_parity_with_native_skillstore() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    // Create via MCP…
    let skill_id = create_skill(&storage, &cfg, "agent-alpha", "parity", "# mcp v1");

    // …observe the same record through the core SkillStore.
    let store = SkillStore::new(&storage);
    let head = store
        .get_head(&skill_id)
        .expect("core read ok")
        .expect("head exists");
    assert_eq!(head.owner_agent, "agent-alpha");
    assert_eq!(head.name, "parity");
    assert_eq!(head.version, 1);
    assert_eq!(head.content, "# mcp v1");

    // Update via core…
    store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: "core-updated".into(),
                content: "# core v2".into(),
                metadata: None,
            },
        )
        .expect("core update ok");

    // …view the new head through MCP.
    let view = tool_text(tool_call(
        "skill_view",
        json!({ "skill_id": skill_id, "owner_agent": "agent-alpha" }),
        &storage,
        &cfg,
    ));
    assert_eq!(view["version"], 2);
    assert_eq!(view["content"], "# core v2");
    assert_eq!(view["description"], "core-updated");

    // File written via MCP is visible through the core metadata (manifest).
    let res = tool_call(
        "skill_files_write",
        json!({
            "skill_id": skill_id,
            "owner_agent": "agent-alpha",
            "expected_version": 2,
            "path": "notes.txt",
            "content": "hello",
        }),
        &storage,
        &cfg,
    );
    assert_eq!(tool_text(res)["version"], 3);
    let head = store
        .get_head(&skill_id)
        .expect("core read ok")
        .expect("head exists");
    assert_eq!(head.metadata.len(), 1);
    assert!(head.metadata.contains_key("file:notes.txt"));
    let record: Value = serde_json::from_str(&head.metadata["file:notes.txt"]).unwrap();
    assert_eq!(record["content"], "hello");
    assert_eq!(record["size_bytes"], 5);
}
