//! MCP-31: `context_assemble` — MCP exposure of the vanta-memory context
//! engine (assemble + session recall under a token budget).
//!
//! Round-trips go through the public `handle_tools_call` API with a seeded
//! session (persona via vanta-seed, L1 records via the SDK), mirroring how
//! external agents will consume the tool.

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

/// Seed one persona document for `session` through vanta-seed.
fn seed_persona(storage: &Arc<StorageEngine>, session: &str, content: &str) {
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    let counts = vanta_memory::seed::import_seed(
        &db,
        &vanta_memory::seed::SeedInput {
            scope: "seed".into(),
            skills: vec![],
            persona: Some(vanta_memory::seed::SeedPersona {
                session_key: session.into(),
                content: content.into(),
            }),
        },
    )
    .expect("seed persona");
    assert_eq!(counts.created, 1, "persona seeded");
}

/// Persist one L1 memory record for `session` (same shape the pipeline writes).
fn seed_l1(storage: &Arc<StorageEngine>, session: &str, id: &str, content: &str) {
    let record = vanta_memory::core::abstractions::MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: vanta_memory::core::abstractions::MemoryType::WorkFact,
        priority: 50,
        scene_name: String::new(),
        source_message_ids: vec![],
        metadata: Value::Null,
        timestamps: vec![],
        created_at: "2026-08-23T00:00:00Z".into(),
        updated_at: "2026-08-23T00:00:00Z".into(),
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
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    db.put(VantaMemoryInput {
        namespace: format!("l1/{session}"),
        key: id.into(),
        payload: serde_json::to_string(&record).expect("serialize MemoryRecord"),
        vector: None,
        sparse_vector: None,
        metadata: vantadb::sdk::VantaMemoryMetadata::new(),
        ttl_ms: None,
    })
    .expect("seed l1 record");
}

/// Re-estimate tokens of the returned messages with the engine's estimator.
fn estimated_tokens(ctx: &Value) -> u64 {
    let estimator = vanta_memory::context_engine::TokenEstimator::default();
    let messages: Vec<vanta_memory::context_engine::ChatMessage> = ctx["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .map(|m| {
            vanta_memory::context_engine::ChatMessage::new(
                serde_json::from_value(m["role"].clone()).expect("role"),
                m["content"].as_str().unwrap_or_default(),
            )
        })
        .collect();
    estimator.estimate_messages(&messages)
}

// ── tools/list ───────────────────────────────────────────────────────────

#[test]
fn test_tools_list_registers_context_assemble() {
    let res = handle_tools_list(&McpConfig::default());
    let tools = res.expect("tools/list")["tools"]
        .as_array()
        .expect("tools array")
        .to_vec();
    let tool = tools
        .iter()
        .find(|t| t["name"] == "context_assemble")
        .expect("context_assemble listed");
    let schema = &tool["inputSchema"];
    assert_eq!(schema["type"], "object");
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(required.contains(&"session_key"), "required: {required:?}");
    assert!(required.contains(&"token_budget"), "required: {required:?}");
    // Optional params advertised so agents can discover them.
    assert!(
        schema["properties"]["query"].is_object(),
        "query optional param documented"
    );
    assert!(
        schema["properties"]["messages"].is_object(),
        "messages optional param documented"
    );
}

// ── round-trip con sesión seedada ────────────────────────────────────────

#[test]
fn test_context_assemble_seeded_persona_roundtrip_within_budget() {
    let (_dir, storage) = setup_storage();
    seed_persona(
        &storage,
        "sess-a",
        "# Profile\nThe user prefers concise answers.",
    );

    let ctx = result_json(call(
        "context_assemble",
        json!({ "session_key": "sess-a", "token_budget": 4000 }),
        &storage,
    ));

    assert_eq!(
        ctx["recall_injected"], true,
        "persona recall injected: {ctx}"
    );
    let joined = ctx["messages"]
        .as_array()
        .expect("messages")
        .iter()
        .map(|m| m["content"].as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("<user-persona>"),
        "persona block present: {joined}"
    );
    assert!(
        estimated_tokens(&ctx) <= 4000,
        "assembled context within budget"
    );
}

#[test]
fn test_context_assemble_query_surfaces_l1_memories() {
    let (_dir, storage) = setup_storage();
    seed_l1(
        &storage,
        "sess-b",
        "m1",
        "User prefers dark mode in every tool",
    );

    let ctx = result_json(call(
        "context_assemble",
        json!({
            "session_key": "sess-b",
            "token_budget": 4000,
            "query": "which theme does the user prefer?"
        }),
        &storage,
    ));

    assert_eq!(ctx["recall_injected"], true, "{ctx}");
    let joined = ctx["messages"]
        .as_array()
        .expect("messages")
        .iter()
        .map(|m| m["content"].as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("<relevant-memories>"),
        "L1 block present: {joined}"
    );
    assert!(
        joined.contains("dark mode"),
        "the matching memory content surfaced: {joined}"
    );
}

#[test]
fn test_context_assemble_compacts_large_history_under_budget() {
    let (_dir, storage) = setup_storage();
    // 30 turns of ~200 words each ≈ far above the budget. The LAST TWO
    // messages are tiny because the engine protects them (min_keep = 2,
    // TDAM parity): when the protected tail alone exceeds the budget the
    // engine deliberately returns over budget (engine.rs cursor guarantee),
    // so a fitting tail is what makes "≤ budget" the correct expectation.
    let mut messages: Vec<Value> = (0..28)
        .map(|i| {
            let filler = "lorem ipsum dolor sit amet ".repeat(40);
            json!({ "role": if i % 2 == 0 { "user" } else { "assistant" }, "content": format!("turn {i}: {filler}") })
        })
        .collect();
    messages.push(json!({ "role": "assistant", "content": "Understood." }));
    messages.push(json!({ "role": "user", "content": "Final question?" }));

    let ctx = result_json(call(
        "context_assemble",
        json!({ "session_key": "hist-sess", "token_budget": 300, "messages": messages }),
        &storage,
    ));

    assert!(
        estimated_tokens(&ctx) <= 300,
        "compacted output within budget: {}",
        estimated_tokens(&ctx)
    );
    let report = &ctx["report"];
    assert!(
        report["tokens_after"].as_u64().unwrap() <= report["tokens_before"].as_u64().unwrap(),
        "history shrank: {report}"
    );
}

// ── errores claros, sin panic ────────────────────────────────────────────

#[test]
fn test_context_assemble_zero_budget_is_clear_error() {
    let (_dir, storage) = setup_storage();
    let res = call(
        "context_assemble",
        json!({ "session_key": "s", "token_budget": 0 }),
        &storage,
    );
    let text = msg(res);
    assert!(
        text.to_lowercase().contains("budget"),
        "clear error: {text}"
    );
    assert!(!text.is_empty());
}

#[test]
fn test_context_assemble_invalid_params_rejected() {
    let (_dir, storage) = setup_storage();

    // Missing token_budget → protocol-level invalid params.
    let res = call("context_assemble", json!({ "session_key": "s" }), &storage);
    assert!(res.is_err(), "missing token_budget must be an error");

    // Unknown role string → invalid params, not a panic / silent drop.
    let res = call(
        "context_assemble",
        json!({
            "session_key": "s",
            "token_budget": 1000,
            "messages": [{ "role": "wizard", "content": "hi" }]
        }),
        &storage,
    );
    assert!(res.is_err(), "invalid role must be rejected");
}

#[test]
fn test_context_assemble_unknown_session_still_assembles_history() {
    let (_dir, storage) = setup_storage();

    let ctx = result_json(call(
        "context_assemble",
        json!({
            "session_key": "never-seeded",
            "token_budget": 10_000,
            "messages": [{ "role": "user", "content": "hello" }]
        }),
        &storage,
    ));

    assert_eq!(ctx["recall_injected"], false, "nothing to inject: {ctx}");
    let msgs = ctx["messages"].as_array().expect("messages");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["content"], "hello");
}
