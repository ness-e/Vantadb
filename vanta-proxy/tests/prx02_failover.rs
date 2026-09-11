// PRX-02 failover tests: 429/5xx on the primary upstream moves to the next
// upstream with exponential backoff; the last attempt surfaces verbatim.
// E2E against the real router with an in-memory engine + scripted mocks.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;

use vanta_proxy::config::{
    AuthConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
};
use vanta_proxy::server;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

fn seeded_engine() -> Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    let engine = vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
        .expect("engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .entity_set("default", "user", USER_ID, fields)
        .expect("seed user");
    for (collection, id) in [("team", "team-1"), ("agent", "agent-1"), ("task", "task-1")] {
        EntityStore::new(&engine)
            .entity_set("default", collection, id, HashMap::new())
            .expect("seed entity");
    }
    Arc::new(engine)
}

fn one(url: &str) -> UpstreamConfig {
    UpstreamConfig {
        url: url.to_string(),
        api_key: String::new(),
        forward_timeout_secs: 600,
        models: Vec::new(),
    }
}

fn state_for_failover(a: &str, b: &str) -> server::AppState {
    let cfg = ProxyConfig {
        server: ServerConfig::default(),
        upstream: one(a),
        upstreams: vec![one(a), one(b)],
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
        report: Default::default(),
        cost: Default::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
    };
    server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

fn plain_message() -> Value {
    json!({
        "model": "m",
        "max_tokens": 8,
        "messages": [{ "role": "user", "content": "hello" }]
    })
}

async fn post_messages(proxy_url: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{proxy_url}/agent/space/v1/messages"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "s-failover")
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// Scripted mock: always answers `status` with `marker` JSON, counting hits.
fn mock(status: u16, marker: &'static str, hits: Arc<AtomicUsize>) -> Router {
    Router::new().route(
        "/v1/messages",
        post(move |body: bytes::Bytes| async move {
            let _ = body;
            hits.fetch_add(1, Ordering::SeqCst);
            (
                axum::http::StatusCode::from_u16(status).unwrap(),
                Json(json!({ "from": marker })),
            )
        }),
    )
}

#[tokio::test]
async fn failover_429_primary_serves_from_secondary() {
    let a_hits = Arc::new(AtomicUsize::new(0));
    let b_hits = Arc::new(AtomicUsize::new(0));
    let a = spawn(mock(429, "a", a_hits.clone())).await;
    let b = spawn(mock(200, "b", b_hits.clone())).await;
    let proxy = spawn(server::router(state_for_failover(&a, &b))).await;

    let started = Instant::now();
    let resp = post_messages(&proxy, plain_message()).await;
    let elapsed = started.elapsed();

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap(), json!({ "from": "b" }));
    assert_eq!(a_hits.load(Ordering::SeqCst), 1, "primary tried once");
    assert_eq!(b_hits.load(Ordering::SeqCst), 1, "secondary served");
    assert!(
        elapsed >= Duration::from_millis(50),
        "backoff must delay the retry, took {elapsed:?}"
    );
}

#[tokio::test]
async fn failover_500_primary_serves_from_secondary() {
    let b_hits = Arc::new(AtomicUsize::new(0));
    let a = spawn(mock(500, "a", Arc::new(AtomicUsize::new(0)))).await;
    let b = spawn(mock(200, "b", b_hits.clone())).await;
    let proxy = spawn(server::router(state_for_failover(&a, &b))).await;

    let resp = post_messages(&proxy, plain_message()).await;
    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap(), json!({ "from": "b" }));
    assert_eq!(b_hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failover_all_down_surfaces_last_status_verbatim() {
    let a = spawn(mock(429, "a", Arc::new(AtomicUsize::new(0)))).await;
    let b = spawn(mock(503, "b", Arc::new(AtomicUsize::new(0)))).await;
    let proxy = spawn(server::router(state_for_failover(&a, &b))).await;

    let resp = post_messages(&proxy, plain_message()).await;
    // Last attempt (B=503) passes through — real upstream signal, no synthetic error.
    assert_eq!(resp.status().as_u16(), 503);
}

#[tokio::test]
async fn failover_client_error_does_not_retry() {
    // 400 is a client error: served from the primary, secondary untouched.
    let b_hits = Arc::new(AtomicUsize::new(0));
    let a = spawn(mock(400, "a", Arc::new(AtomicUsize::new(0)))).await;
    let b = spawn(mock(200, "b", b_hits.clone())).await;
    let proxy = spawn(server::router(state_for_failover(&a, &b))).await;

    let resp = post_messages(&proxy, plain_message()).await;
    assert_eq!(resp.status().as_u16(), 400);
    assert_eq!(b_hits.load(Ordering::SeqCst), 0, "no failover on 4xx");
}
