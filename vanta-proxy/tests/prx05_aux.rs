// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-05 contract tests: model discovery + token counting + beta headers.

use std::sync::Mutex;

use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vanta_proxy::config::{ProxyConfig, ServerConfig, UpstreamConfig};
use vanta_proxy::server;

const USER_KEY: &str = "sk-prx05-test";

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
        .entity_set("default", "user", "usr-prx05", fields)
        .unwrap();
    std::sync::Arc::new(engine)
}

#[derive(Default)]
struct Captured {
    body: Vec<u8>,
    headers: HeaderMap,
}

type Shared = std::sync::Arc<Mutex<Option<Captured>>>;

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

/// Proxy wired to a mock upstream that captures headers+body (beta-header tests).
async fn setup_with_capture() -> (String, Shared) {
    let captured: Shared = std::sync::Arc::new(Mutex::new(None));
    let c1 = captured.clone();
    let c2 = captured.clone();
    let upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(|headers: HeaderMap, body: bytes::Bytes| async move {
                *c1.lock().unwrap() = Some(Captured {
                    body: body.to_vec(),
                    headers,
                });
                Json(json!({ "id": "chatcmpl-1" }))
            }),
        )
        .route(
            "/v1/messages",
            post(|headers: HeaderMap, body: bytes::Bytes| async move {
                *c2.lock().unwrap() = Some(Captured {
                    body: body.to_vec(),
                    headers,
                });
                Json(json!({ "id": "msg_1" }))
            }),
        );
    let upstream_url = spawn(upstream).await;
    let cfg = ProxyConfig {
        report: Default::default(),
        cost: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: vec!["claude-opus-4-6".to_string(), "gpt-test".to_string()],
        },
        upstreams: Vec::new(),
        auth: vanta_proxy::config::AuthConfig::default(),
        mem_command: vanta_proxy::config::MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
    };
    let proxy_url = spawn(server::router(
        server::AppState::from_engine(cfg, seeded_engine()).unwrap(),
    ))
    .await;
    (proxy_url, captured)
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

fn authed(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    req.header("x-vanta-user-key", USER_KEY)
}

// ── GET /v1/models (plain + prefixed) ───────────────────────────────────────
#[tokio::test]
async fn models_plain_returns_configured_list() {
    let (proxy_url, _) = setup_with_capture().await;
    let resp = authed(client().get(format!("{proxy_url}/v1/models")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "list");
    let ids: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["claude-opus-4-6", "gpt-test"]);
}

#[tokio::test]
async fn models_prefixed_shape_matches_and_requires_auth() {
    let (proxy_url, _) = setup_with_capture().await;
    // No key → 401 (D34, no open mode).
    let denied = client()
        .get(format!("{proxy_url}/cc/sp1/v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 401);
    // With key → 200, same OpenAI shape.
    let resp = authed(client().get(format!("{proxy_url}/cc/sp1/v1/models")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "list");
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"][0]["object"], "model");
}

// ── POST count_tokens ───────────────────────────────────────────────────────
#[tokio::test]
async fn count_tokens_returns_input_tokens_and_requires_auth() {
    let (proxy_url, _) = setup_with_capture().await;
    let payload =
        json!({"model": "c", "messages": [{"role": "user", "content": "hello world, count me"}]});
    let denied = client()
        .post(format!("{proxy_url}/v1/messages/count_tokens"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 401);
    for path in [
        "/v1/messages/count_tokens",
        "/cc/sp1/v1/messages/count_tokens",
    ] {
        let resp = authed(client().post(format!("{proxy_url}{path}")))
            .json(&payload)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "path {path}");
        let body: Value = resp.json().await.unwrap();
        assert!(
            body.get("input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                >= 1,
            "path {path}: {body}"
        );
    }
}

#[tokio::test]
async fn count_tokens_rejects_non_json_with_400() {
    let (proxy_url, _) = setup_with_capture().await;
    let resp = authed(client().post(format!("{proxy_url}/v1/messages/count_tokens")))
        .header("content-type", "application/json")
        .body("not-json{{{")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// ── Beta headers: verbatim passthrough + PRX-04 idempotency ─────────────────
#[tokio::test]
async fn beta_headers_forwarded_verbatim_and_prefix_stable() {
    let (proxy_url, captured) = setup_with_capture().await;
    let payload =
        json!({"model": "c", "max_tokens": 8, "messages": [{"role": "user", "content": "hi"}]});
    let send = |payload: Value| {
        let url = proxy_url.clone();
        async move {
            authed(client().post(format!("{url}/cc/sp1/v1/messages")))
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "prompt-caching-2024-07-31")
                .json(&payload)
                .send()
                .await
                .unwrap()
        }
    };
    assert_eq!(send(payload.clone()).await.status(), 200);
    let first = captured.lock().unwrap().take().expect("captured");
    assert_eq!(first.headers["anthropic-version"], "2023-06-01");
    assert_eq!(first.headers["anthropic-beta"], "prompt-caching-2024-07-31");
    // Same request again → byte-identical upstream body (PRX-04 prefix stability,
    // beta headers must not perturb the inject prefix).
    assert_eq!(send(payload.clone()).await.status(), 200);
    let second = captured.lock().unwrap().take().expect("captured");
    assert_eq!(first.body, second.body, "re-inject must be byte-stable");
    assert_eq!(
        second.headers["anthropic-beta"],
        "prompt-caching-2024-07-31"
    );
}
