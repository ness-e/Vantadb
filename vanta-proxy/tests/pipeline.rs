//! MEM-26 contract tests: auth→session→inject pipeline against a mocked
//! upstream + in-memory local DB (D19 a-f).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};
use vantadb::storage::StorageEngine;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

/// Captured upstream request for assertions.
#[derive(Default)]
struct Captured {
    body: Vec<u8>,
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
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .entity_set("default", "user", USER_ID, fields)
        .expect("seed user");
    Arc::new(engine)
}

fn state_for(upstream_url: &str) -> vanta_proxy::server::AppState {
    let cfg = vanta_proxy::config::ProxyConfig {
        server: vanta_proxy::config::ServerConfig::default(),
        upstream: vanta_proxy::config::UpstreamConfig {
            url: upstream_url.to_string(),
            api_key: String::new(),
            forward_timeout_secs: 600,
        },
        auth: vanta_proxy::config::AuthConfig::default(),
        mem_command: vanta_proxy::config::MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
    };
    vanta_proxy::server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

struct TestEnv {
    proxy_url: String,
    upstream_captured: Shared,
}

/// Spawn mock upstream + proxy wired to it (user-key seeded).
async fn setup() -> TestEnv {
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
    let proxy_url = spawn(vanta_proxy::server::router(state_for(&upstream_url))).await;
    TestEnv {
        proxy_url,
        upstream_captured: captured,
    }
}

async fn post_chat(
    env: &TestEnv,
    extra_headers: &[(&str, &str)],
    body: Value,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}/agent/space/v1/chat/completions", env.proxy_url))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY);
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }
    req.json(&body).send().await.unwrap()
}

async fn captured_body(env: &TestEnv) -> Value {
    let cap = env.upstream_captured.lock().unwrap();
    let cap = cap.as_ref().expect("upstream received request");
    serde_json::from_slice(&cap.body).unwrap()
}

/// Seed persona + scene records for `session_key` via public SDK/lib APIs.
fn seed_memory(db: &VantaEmbedded, session_key: &str) {
    use vanta_memory::core::abstractions::PersonaMode;
    use vanta_memory::core::persona::persona_generator::{
        persona_namespace, PersonaRecord, PERSONA_KEY,
    };
    use vanta_memory::core::scene::scene_index::upsert_scene;

    let record = PersonaRecord {
        content: "# User Narrative Profile\nPrefers concise answers.".into(),
        mode: PersonaMode::First,
        generated_at_ms: 0,
        generated_at: "2026-08-21T00:00:00+00:00".into(),
    };
    db.put(VantaMemoryInput {
        namespace: persona_namespace(session_key),
        key: PERSONA_KEY.into(),
        payload: serde_json::to_string(&record).expect("persona json"),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("seed persona");

    upsert_scene(
        db,
        session_key,
        "deploy-runbook",
        "deploys",
        "how to deploy the service",
    )
    .expect("seed scene");
}

// ── (a) D34: missing/invalid → 401; valid key forwards ──────────────────────
#[tokio::test]
async fn a_auth_valid_key_forwards_invalid_and_missing_rejected() {
    let env = setup().await;
    let payload = json!({ "model": "m", "messages": [{ "role": "user", "content": "hi" }] });
    let client = reqwest::Client::new();

    // Invalid key → 401, nothing forwarded.
    let resp = client
        .post(format!("{}/agent/s/v1/chat/completions", env.proxy_url))
        .header("x-vanta-user-key", "sk-wrong")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Missing key → 401 (D34: no open mode).
    let resp = client
        .post(format!("{}/agent/s/v1/chat/completions", env.proxy_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert!(env.upstream_captured.lock().unwrap().is_none());

    // Valid key → 200 and forwarded.
    let resp = post_chat(&env, &[], payload).await;
    assert_eq!(resp.status(), 200);
    assert!(captured_body(&env).await["messages"].is_array());
}

// ── (b) sessionKey resolved from every alias header (integration slice; full ─
// ── priority matrix covered by unit tests in session.rs) ────────────────────
#[tokio::test]
async fn b_session_key_from_each_alias_header() {
    for alias in [
        "x-conversation-id",
        "x-session-id",
        "x-claude-code-session-id",
        "x-chat-id",
        "x-thread-id",
    ] {
        let env = setup().await;
        let payload = json!({ "model": "m", "messages": [{ "role": "user", "content": "hi" }] });
        let resp = post_chat(&env, &[(alias, "sess-b")], payload).await;
        assert_eq!(resp.status(), 200, "alias {alias} must resolve a session");
        // Session exists → L0/L1 tools injected even without memory content.
        let body = captured_body(&env).await;
        assert!(
            body["tools"].as_array().expect("tools").len() == 2,
            "alias {alias}: session active, tools exposed"
        );
    }
}

// ── (c)+(f) init limpio: no session header → verbatim forward ────────────────
#[tokio::test]
async fn f_no_session_header_forwards_verbatim() {
    let env = setup().await;
    let payload = json!({
        "model": "m",
        "messages": [{ "role": "system", "content": "S" }, { "role": "user", "content": "u1" }]
    });
    let resp = post_chat(&env, &[], payload.clone()).await;
    assert_eq!(resp.status(), 200);
    let body = captured_body(&env).await;
    assert_eq!(body, payload, "no session → no injection at all");

    // Internal credential never leaks upstream.
    let cap = env.upstream_captured.lock().unwrap();
    assert!(
        cap.as_ref()
            .unwrap()
            .headers
            .get("x-vanta-user-key")
            .is_none(),
        "user key leaked to upstream"
    );
}

// ── (d)+(e) injection lands ONLY on system prompt; tools present ────────────
#[tokio::test]
async fn d_injection_only_system_prompt_with_persona_and_scenes() {
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
                Json(json!({}))
            }
        }),
    );
    let upstream_url = spawn(upstream).await;
    let state = state_for(&upstream_url);
    seed_memory(&state.memory, "sess-inj");
    let env = TestEnv {
        proxy_url: spawn(vanta_proxy::server::router(state)).await,
        upstream_captured: captured,
    };

    let payload = json!({
        "model": "m",
        "messages": [
            { "role": "system", "content": "BASE PROMPT" },
            { "role": "user", "content": "u1" },
            { "role": "assistant", "content": "a1" }
        ]
    });
    let resp = post_chat(&env, &[("x-conversation-id", "sess-inj")], payload).await;
    assert_eq!(resp.status(), 200);

    let body = captured_body(&env).await;
    let msgs = body["messages"].as_array().unwrap();

    // System position carries the memory block; base prompt preserved after it.
    assert_eq!(msgs[0]["role"], "system");
    let sys = msgs[0]["content"].as_str().unwrap();
    assert!(sys.starts_with("<vanta-memory>"), "block prepended first");
    assert!(sys.contains("# User Narrative Profile"), "persona injected");
    assert!(sys.contains("deploy-runbook"), "current scene injected");
    assert!(
        sys.ends_with("BASE PROMPT"),
        "original system prompt intact"
    );

    // History untouched (D29/KV-cache).
    assert_eq!(msgs.len(), 3, "no messages added or removed");
    assert_eq!(msgs[1], json!({ "role": "user", "content": "u1" }));
    assert_eq!(msgs[2], json!({ "role": "assistant", "content": "a1" }));

    // (e) L0/L1 tools present in the same body.
    let names: Vec<&str> = body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t.pointer("/function/name").and_then(Value::as_str))
        .collect();
    assert!(names.contains(&"vanta_memory_capture"), "L0 tool exposed");
    assert!(names.contains(&"vanta_memory_search"), "L1 tool exposed");
}

// ── (g) MEM-50/D47: completed request captures the L0 turn via WriteBack ────
#[tokio::test]
async fn g_completed_request_tracks_l0_turn() {
    use std::time::Duration;
    use vanta_proxy::capture::list_turns;

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
                Json(json!({}))
            }
        }),
    );
    let upstream_url = spawn(upstream).await;
    // Keep a memory handle before the state is moved into the router.
    let state = state_for(&upstream_url);
    let memory = state.memory.clone();
    let env = TestEnv {
        proxy_url: spawn(vanta_proxy::server::router(state)).await,
        upstream_captured: captured,
    };

    // With session: 200 + turn persisted asynchronously (fire-and-forget).
    let payload = json!({
        "model": "m",
        "messages": [
            { "role": "system", "content": "S" },
            { "role": "user", "content": "capture me please" }
        ]
    });
    let resp = post_chat(&env, &[("x-conversation-id", "sess-capture")], payload).await;
    assert_eq!(resp.status(), 200);

    let mut turns = Vec::new();
    for _ in 0..40 {
        turns = list_turns(&memory);
        if !turns.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(turns.len(), 1, "exactly one L0 turn captured");
    assert!(turns[0].payload.contains("capture me please"));
    assert!(turns[0].payload.contains("sess-capture"));
    assert!(turns[0].payload.contains("openai"));

    // Without session header: verbatim path → no capture.
    let payload = json!({ "model": "m", "messages": [{ "role": "user", "content": "anon" }] });
    let resp = post_chat(&env, &[], payload).await;
    assert_eq!(resp.status(), 200);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(list_turns(&memory).len(), 1, "no session → no capture");
}
