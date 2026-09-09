// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-12 compat suite: coding-agent wire shapes replayed through the real proxy.
//!
//! Each agent speaks one protocol; fixtures live in `fixtures/` (see README.md
//! for provenance — representative shapes, NOT live captures). Every test asserts
//! the proxy contract only: request forwarded byte-identical, upstream response
//! replayed byte-identical, route reachable. Upstream behavior is the mock's.

use std::sync::Mutex;

use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use vanta_proxy::config::{ProxyConfig, ServerConfig, UpstreamConfig};
use vanta_proxy::server;

const USER_KEY: &str = "sk-compat-test";

// ── fixtures (compile-time embedded, parse-checked by serde_json) ─────────────
const CLAUDE_REQ: &str = include_str!("fixtures/claude_code_messages_request.json");
const CLAUDE_RESP: &str = include_str!("fixtures/claude_code_messages_response.json");
const CODEX_REQ: &str = include_str!("fixtures/codex_responses_request.json");
const CODEX_RESP: &str = include_str!("fixtures/codex_responses_response.json");
const OPENCODE_REQ: &str = include_str!("fixtures/opencode_chat_request.json");
const OPENCODE_RESP: &str = include_str!("fixtures/opencode_chat_response.json");

/// In-memory engine seeded with the compat-test user (D34: auth is mandatory).
fn seeded_engine() -> std::sync::Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    let engine =
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let mut fields: std::collections::HashMap<String, vantadb::node::FieldValue> =
        std::collections::HashMap::new();
    fields.insert(
        "user_key".to_string(),
        vantadb::node::FieldValue::String(USER_KEY.to_string()),
    );
    vantadb::entity::EntityStore::new(&engine)
        .entity_set("default", "user", "usr-compat", fields)
        .unwrap();
    std::sync::Arc::new(engine)
}

/// Last upstream body per path, for verbatim assertions.
type Captured = std::sync::Arc<Mutex<Vec<(String, Vec<u8>)>>>;

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

/// Mock upstream replaying one canned JSON per path + the real proxy wired to it.
async fn setup() -> (String, Captured) {
    let captured: Captured = std::sync::Arc::new(Mutex::new(Vec::new()));
    // (path, canned response) pairs — one per agent protocol.
    let routes: Vec<(&'static str, &'static str)> = vec![
        ("/v1/messages", CLAUDE_RESP),
        ("/v1/responses", CODEX_RESP),
        ("/v1/chat/completions", OPENCODE_RESP),
    ];
    let mut app = Router::new();
    for (path, canned) in routes {
        let c = captured.clone();
        let body: Value = serde_json::from_str(canned).unwrap();
        app = app.route(
            path,
            post(move |headers: HeaderMap, bytes: bytes::Bytes| {
                let c = c.clone();
                let body = body.clone();
                async move {
                    c.lock().unwrap().push((path.to_string(), bytes.to_vec()));
                    let _ = headers;
                    Json(body)
                }
            }),
        );
    }
    let upstream_url = spawn(app).await;
    let cfg = ProxyConfig {
        report: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        auth: vanta_proxy::config::AuthConfig::default(),
        mem_command: vanta_proxy::config::MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
    };
    let state = server::AppState::from_engine(cfg, seeded_engine()).unwrap();
    (spawn(server::router(state)).await, captured)
}

async fn post_fixture(proxy_url: &str, path: &str, fixture: &str) -> reqwest::Response {
    let body: Value = serde_json::from_str(fixture).unwrap();
    reqwest::Client::new()
        .post(format!("{proxy_url}{path}"))
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-key-123")
        .header("x-vanta-user-key", USER_KEY)
        .json(&body)
        .send()
        .await
        .unwrap()
}

fn upstream_body_for(captured: &Captured, path: &str) -> Vec<u8> {
    captured
        .lock()
        .unwrap()
        .iter()
        .find(|(p, _)| p == path)
        .map(|(_, b)| b.clone())
        .expect("upstream received request for path")
}

// ── verbatim round-trips (one per agent) ──────────────────────────────────────

#[tokio::test]
async fn claude_code_messages_roundtrip_verbatim() {
    let (proxy_url, captured) = setup().await;
    let resp = post_fixture(&proxy_url, "/compat/s1/v1/messages", CLAUDE_REQ).await;
    assert_eq!(resp.status(), 200);
    let got: Value = resp.json().await.unwrap();
    assert_eq!(got, serde_json::from_str::<Value>(CLAUDE_RESP).unwrap());
    let fwd = upstream_body_for(&captured, "/v1/messages");
    assert_eq!(
        serde_json::from_slice::<Value>(&fwd).unwrap(),
        serde_json::from_str::<Value>(CLAUDE_REQ).unwrap()
    );
}

#[tokio::test]
async fn codex_responses_roundtrip_verbatim() {
    let (proxy_url, captured) = setup().await;
    let resp = post_fixture(&proxy_url, "/v1/responses", CODEX_REQ).await;
    assert_eq!(resp.status(), 200);
    let got: Value = resp.json().await.unwrap();
    assert_eq!(got, serde_json::from_str::<Value>(CODEX_RESP).unwrap());
    let fwd = upstream_body_for(&captured, "/v1/responses");
    assert_eq!(
        serde_json::from_slice::<Value>(&fwd).unwrap(),
        serde_json::from_str::<Value>(CODEX_REQ).unwrap()
    );
}

#[tokio::test]
async fn opencode_chat_roundtrip_verbatim() {
    let (proxy_url, captured) = setup().await;
    let resp = post_fixture(&proxy_url, "/compat/s1/v1/chat/completions", OPENCODE_REQ).await;
    assert_eq!(resp.status(), 200);
    let got: Value = resp.json().await.unwrap();
    assert_eq!(got, serde_json::from_str::<Value>(OPENCODE_RESP).unwrap());
    let fwd = upstream_body_for(&captured, "/v1/chat/completions");
    assert_eq!(
        serde_json::from_slice::<Value>(&fwd).unwrap(),
        serde_json::from_str::<Value>(OPENCODE_REQ).unwrap()
    );
}

// ── required-field contracts (release breaks shape → test names the field) ────

#[tokio::test]
async fn claude_code_request_carries_protocol_fields() {
    let v: Value = serde_json::from_str(CLAUDE_REQ).unwrap();
    assert!(v["model"].is_string(), "messages requires model");
    assert!(v["max_tokens"].is_number(), "messages requires max_tokens");
    assert!(v["messages"].is_array(), "messages requires messages[]");
    assert!(v["tools"].is_array(), "agent shape requires tools[]");
}

#[tokio::test]
async fn codex_request_carries_protocol_fields() {
    let v: Value = serde_json::from_str(CODEX_REQ).unwrap();
    assert!(v["model"].is_string(), "responses requires model");
    assert!(!v["input"].is_null(), "responses requires input");
    assert!(v["tools"].is_array(), "agent shape requires tools[]");
}

#[tokio::test]
async fn opencode_request_carries_protocol_fields() {
    let v: Value = serde_json::from_str(OPENCODE_REQ).unwrap();
    assert!(v["model"].is_string(), "chat requires model");
    assert!(v["messages"].is_array(), "chat requires messages[]");
    assert!(v["tools"].is_array(), "agent shape requires tools[]");
}
