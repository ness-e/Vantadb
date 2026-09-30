// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! VER-04: injection governance, end-to-end through the proxy wire —
//! budget enforced per request (0 = no block, low = truncated ≤ budget,
//! roomy = persona present), namespace ACL deny outside scope, and the
//! injection audit JSONL (metadata only: ns/key/budget/truncated/acl).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vanta_proxy::config::{
    AuthConfig, InjectionConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
    WritebackConfig,
};
use vanta_proxy::server::{router, AppState};
use vantadb::entity::{EntityStore, EntityWrite};
use vantadb::node::FieldValue;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::storage::StorageEngine;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

#[derive(Default)]
struct Captured {
    body: Vec<u8>,
    #[allow(dead_code)]
    headers: HeaderMap,
}

type Shared = Arc<Mutex<Option<Captured>>>;

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

/// In-memory engine seeded with the test user entity (D34).
fn seeded_engine() -> Arc<StorageEngine> {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: USER_ID,
            fields,
        })
        .expect("seed user");
    Arc::new(engine)
}

/// Seed a long persona + two scenes for `session_key` (filler is compressed
/// prose so the block is large under a roomy budget).
fn seed_memory(engine: &Arc<StorageEngine>, session_key: &str) {
    use vanta_memory::core::abstractions::PersonaMode;
    use vanta_memory::core::persona::persona_generator::{
        persona_namespace, PersonaRecord, PERSONA_KEY,
    };
    use vanta_memory::core::scene::scene_index::upsert_scene;

    let db = Embedded::from_engine(engine.clone());
    let record = PersonaRecord {
        content: format!("PERSONA-MARKER concise answers. {}", "p ".repeat(2000)),
        mode: PersonaMode::First,
        generated_at_ms: 0,
        generated_at: "2026-09-29T00:00:00+00:00".into(),
    };
    db.put(MemoryInput {
        namespace: persona_namespace(session_key),
        key: PERSONA_KEY.into(),
        payload: serde_json::to_string(&record).expect("persona json"),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed persona");
    upsert_scene(
        &db,
        session_key,
        "runbook",
        "deploys",
        "how to deploy the service",
    )
    .expect("seed scene");
}

struct TestEnv {
    proxy_url: String,
    upstream_captured: Shared,
}

async fn setup_with(injection: InjectionConfig) -> TestEnv {
    let captured: Shared = Arc::new(Mutex::new(None));
    let c = captured.clone();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(move |headers: HeaderMap, body: bytes::Bytes| {
            let c = c.clone();
            async move {
                *c.lock().unwrap() = Some(Captured {
                    body: body.to_vec(),
                    headers,
                });
                Json(json!({ "id": "chatcmpl-1" }))
            }
        }),
    );
    let upstream_url = spawn(upstream).await;
    let engine = seeded_engine();
    seed_memory(&engine, "sess-1");

    let cfg = ProxyConfig {
        report: Default::default(),
        cost: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: WritebackConfig::default(),
        cache: Default::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection,
        envelope: Default::default(),
    };
    let state = AppState::from_engine(cfg, engine).expect("proxy state");
    let proxy_url = spawn(router(state)).await;
    TestEnv {
        proxy_url,
        upstream_captured: captured,
    }
}

async fn post_chat(env: &TestEnv) -> reqwest::Response {
    let payload = json!({
        "model": "m",
        "messages": [{ "role": "user", "content": "hi" }],
    });
    reqwest::Client::new()
        .post(format!("{}/agent/space/v1/chat/completions", env.proxy_url))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "sess-1")
        .json(&payload)
        .send()
        .await
        .unwrap()
}

/// The system-prompt prefix the upstream actually received (OpenAI path
/// prepends the block to `messages[0].content`).
async fn forwarded_prefix(env: &TestEnv) -> String {
    let cap = env.upstream_captured.lock().unwrap();
    let cap = cap.as_ref().expect("upstream received request");
    let body: Value = serde_json::from_slice(&cap.body).unwrap();
    body["messages"][0]["content"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn audit_rows(path: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("jsonl line"))
        .collect()
}

// ── c1: budget enforced per request (e2e) ───────────────────────────────────
#[tokio::test]
async fn budget_zero_disables_block_and_low_budget_truncates() {
    // Budget 0 → injection off: the forward is byte-identical (no block).
    let off = setup_with(InjectionConfig {
        max_tokens: 0,
        ..Default::default()
    })
    .await;
    assert_eq!(post_chat(&off).await.status(), 200);
    let prefix = forwarded_prefix(&off).await;
    assert!(
        !prefix.contains("<vanta-memory>"),
        "budget 0 must disable injection: {prefix}"
    );

    // Low budget → persona hard-truncated inside the cap (hard cap wins).
    let low = setup_with(InjectionConfig {
        max_tokens: 60,
        ..Default::default()
    })
    .await;
    assert_eq!(post_chat(&low).await.status(), 200);
    let prefix = forwarded_prefix(&low).await;
    assert!(prefix.starts_with("<vanta-memory>"), "block injected");
    let end = prefix.find("</vanta-memory>").expect("block closed") + "</vanta-memory>".len();
    let block = &prefix[..end];
    assert!(
        vanta_proxy::cost::estimate_text_tokens(block.len()) <= 60,
        "block over budget: {} tokens",
        vanta_proxy::cost::estimate_text_tokens(block.len())
    );
    assert!(
        block.contains("…[truncated]"),
        "low budget must truncate visibly: {block}"
    );

    // Roomy budget → persona text present.
    let roomy = setup_with(InjectionConfig {
        max_tokens: 10_000,
        ..Default::default()
    })
    .await;
    assert_eq!(post_chat(&roomy).await.status(), 200);
    let prefix = forwarded_prefix(&roomy).await;
    assert!(prefix.contains("PERSONA-MARKER"), "roomy keeps persona");
}

// ── c2: ACL deny outside scope ──────────────────────────────────────────────
#[tokio::test]
async fn acl_denies_sources_outside_the_allowlist() {
    let env = setup_with(InjectionConfig {
        max_tokens: 10_000,
        namespace_allow_prefixes: vec!["other/".into()],
        audit_log_path: String::new(),
    })
    .await;
    assert_eq!(post_chat(&env).await.status(), 200);
    let prefix = forwarded_prefix(&env).await;
    assert!(
        !prefix.contains("<vanta-memory>"),
        "denied sources must not inject: {prefix}"
    );
}

// ── c3: audit log consultable, metadata only ────────────────────────────────
#[tokio::test]
async fn injection_audit_logs_sources_budget_and_acl_without_payload() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("injection-audit.jsonl");
    let env = setup_with(InjectionConfig {
        max_tokens: 10_000,
        namespace_allow_prefixes: Vec::new(),
        audit_log_path: audit_path.to_string_lossy().to_string(),
    })
    .await;
    assert_eq!(post_chat(&env).await.status(), 200);

    let rows = audit_rows(&audit_path);
    assert!(
        rows.len() >= 2,
        "expected persona+scene audit rows, got {rows:?}"
    );
    let persona_row = rows
        .iter()
        .find(|r| r["namespace"] == "persona/sess-1")
        .expect("persona injection audited");
    assert_eq!(persona_row["op"], "injection");
    assert_eq!(persona_row["outcome"], "ok");
    assert_eq!(persona_row["key"], "persona.md");
    let reason = persona_row["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("surface=proxy"), "reason: {reason}");
    assert!(reason.contains("budget="), "reason: {reason}");
    assert!(reason.contains("acl=allow"), "reason: {reason}");

    // Metadata only: no persona content in the audit file.
    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    assert!(
        !raw.contains("PERSONA-MARKER"),
        "payload leaked into the audit: {raw}"
    );
}

// ── c2/c3: denied namespaces are audited (never silent) ─────────────────────
#[tokio::test]
async fn acl_denials_are_audited_with_deny_outcome() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("injection-audit.jsonl");
    let env = setup_with(InjectionConfig {
        max_tokens: 10_000,
        namespace_allow_prefixes: vec!["other/".into()],
        audit_log_path: audit_path.to_string_lossy().to_string(),
    })
    .await;
    assert_eq!(post_chat(&env).await.status(), 200);

    let rows = audit_rows(&audit_path);
    let denied: Vec<&Value> = rows.iter().filter(|r| r["outcome"] == "denied").collect();
    assert!(
        denied.iter().any(|r| r["namespace"] == "persona/sess-1"),
        "persona denial missing: {rows:?}"
    );
    assert!(
        denied.iter().any(|r| r["namespace"] == "scene/sess-1"),
        "scene denial missing: {rows:?}"
    );
    for row in denied {
        assert!(
            row["reason"]
                .as_str()
                .unwrap_or_default()
                .contains("acl=deny"),
            "deny reason: {row:?}"
        );
    }
}

// ── EXE-01 demo producer (CI: `injection-governance-demo.yml`) ──────────────

/// Demo directory: `GOVERNANCE_DEMO_DIR` or
/// `<workspace>/target/injection-governance-demo`.
fn demo_dir() -> std::path::PathBuf {
    std::env::var("GOVERNANCE_DEMO_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace root")
                .join("target/injection-governance-demo")
        })
}

/// Seeds the fixtures `scripts/demo-governance-e2e.ps1` replays through the
/// injection-audit JSONL. One request (low budget + ACL allowing only
/// `persona/`) produces every governance row the demo contract names:
/// `outcome=ok` with `truncated=true` (block under budget, visible cut) and
/// `outcome=denied` with `acl=deny` (scene outside scope). Metadata-only is
/// asserted (no payload marker in the audit file). Writes `<demo>/handoff.json`.
#[tokio::test]
#[ignore = "demo producer: run via scripts/demo-governance-e2e.ps1 (--run-ignored ignored-only)"]
async fn governance_demo_producer() {
    let demo = demo_dir();
    if demo.exists() {
        std::fs::remove_dir_all(&demo).expect("clean demo dir");
    }
    std::fs::create_dir_all(&demo).expect("demo dir");
    let audit_path = demo.join("injection-audit.jsonl");

    // Low budget → persona injected hard-truncated (≤ budget, visible marker);
    // ACL allows persona/ only → the scene is denied by policy.
    let env = setup_with(InjectionConfig {
        max_tokens: 60,
        namespace_allow_prefixes: vec!["persona/".into()],
        audit_log_path: audit_path.to_string_lossy().to_string(),
    })
    .await;
    assert_eq!(post_chat(&env).await.status(), 200);

    let prefix = forwarded_prefix(&env).await;
    assert!(
        prefix.starts_with("<vanta-memory>"),
        "block injected: {prefix}"
    );
    let end = prefix.find("</vanta-memory>").expect("block closed") + "</vanta-memory>".len();
    let block = &prefix[..end];
    assert!(
        vanta_proxy::cost::estimate_text_tokens(block.len()) <= 60,
        "block over budget: {} tokens",
        vanta_proxy::cost::estimate_text_tokens(block.len())
    );
    assert!(
        block.contains("…[truncated]"),
        "low budget must truncate visibly: {block}"
    );
    // Self-contained ACL check (the workflow runs only this test): the denied
    // scene's content must never reach the forwarded prompt.
    assert!(
        !prefix.contains("how to deploy the service"),
        "ACL-denied scene must never enter the prompt: {prefix}"
    );

    let rows = audit_rows(&audit_path);
    let persona = rows
        .iter()
        .find(|r| r["namespace"] == "persona/sess-1" && r["outcome"] == "ok")
        .expect("persona injection audited");
    let reason = persona["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("truncated=true"), "reason: {reason}");
    assert!(reason.contains("budget="), "reason: {reason}");
    assert!(reason.contains("acl=allow"), "reason: {reason}");
    let denied = rows
        .iter()
        .find(|r| r["namespace"] == "scene/sess-1" && r["outcome"] == "denied")
        .expect("scene ACL denial audited");
    assert!(
        denied["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("acl=deny"),
        "deny reason: {:?}",
        denied["reason"]
    );
    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    assert!(
        !raw.contains("PERSONA-MARKER"),
        "payload leaked into the audit: {raw}"
    );

    let handoff = serde_json::json!({
        "generated_by": "ver04_governance demo producer",
        "audit": audit_path.display().to_string(),
        "session": "sess-1",
    });
    std::fs::write(
        demo.join("handoff.json"),
        serde_json::to_string_pretty(&handoff).expect("handoff json"),
    )
    .expect("write handoff");
    println!(
        "injection governance demo fixtures OK — audit: {} ({} rows, metadata-only)",
        audit_path.display(),
        rows.len()
    );
}
