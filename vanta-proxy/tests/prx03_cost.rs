// PRX-03 cost tracking + virtual keys: request-side accounting lands in the
// ledger (per key × session × model) and budget enforcement answers 429
// with the `budget_exceeded` shape — without ever forwarding upstream.
// E2E against the real router with an in-memory engine + scripted mock.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;

use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vantadb::entity::{EntityStore, EntityWrite};
use vantadb::node::FieldValue;

use vanta_proxy::config::{
    AuthConfig, CostConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
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
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: USER_ID,
            fields,
        })
        .expect("seed user");
    for (collection, id) in [("team", "team-1"), ("agent", "agent-1"), ("task", "task-1")] {
        EntityStore::new(&engine)
            .set(EntityWrite {
                namespace: "default",
                collection,
                id,
                fields: HashMap::new(),
            })
            .expect("seed entity");
    }
    Arc::new(engine)
}

fn state_with_cost(upstream: &str, cost: CostConfig) -> server::AppState {
    let cfg = ProxyConfig {
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream.to_string(),
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
        report: Default::default(),
        cost,
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection: Default::default(),
    };
    server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

fn priced_message() -> Value {
    json!({
        "model": "gpt-4o",
        "max_tokens": 8,
        "messages": [{ "role": "user", "content": "hello world, bill this turn" }]
    })
}

async fn post_messages(proxy_url: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{proxy_url}/agent/space/v1/messages"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "s-cost")
        .json(&body)
        .send()
        .await
        .unwrap()
}

fn mock_ok() -> Router {
    Router::new().route(
        "/v1/messages",
        post(move |body: bytes::Bytes| async move {
            let _ = body;
            (
                axum::http::StatusCode::OK,
                Json(json!({ "usage": { "input_tokens": 10, "output_tokens": 3 } })),
            )
        }),
    )
}

#[tokio::test]
async fn successful_turn_is_recorded_by_key_session_model() {
    let upstream = spawn(mock_ok()).await;
    let state = state_with_cost(&upstream, CostConfig::default());
    let proxy = spawn(server::router(state.clone())).await;

    let resp = post_messages(&proxy, priced_message()).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Ledger splits the turn across all three axes (contract: accounting).
    assert!(
        state.cost.spent_by_key(USER_ID) > 0.0,
        "key must accrue spend"
    );
    assert!(
        state.cost.spent_by_session("s-cost") > 0.0,
        "session must accrue spend"
    );
    assert!(
        state.cost.spent_by_model("gpt-4o") > 0.0,
        "model must accrue spend"
    );
    assert_eq!(state.cost.spent_by_key("nobody"), 0.0);
    let snap = state.cost.snapshot();
    assert!(snap.entries >= 1, "snapshot must hold the turn");
    assert!(snap.total_usd > 0.0);
}

#[tokio::test]
async fn over_budget_with_enforce_answers_429_without_forwarding() {
    // Budget 0.0 + enforce: already over before the first byte — deterministic.
    let upstream = spawn(mock_ok()).await;
    let state = state_with_cost(
        &upstream,
        CostConfig {
            enabled: true,
            default_budget_usd: Some(0.0),
            enforce: true,
            prices: Default::default(),
        },
    );
    // Direct gate check (no HTTP): Limited without any recorded spend.
    let key = vanta_proxy::cost::VirtualKey {
        id: USER_ID.to_string(),
        budget_usd: Some(0.0),
        enforce: true,
    };
    assert!(matches!(
        state.cost.check_budget(&key),
        vanta_proxy::cost::BudgetDecision::Limited { .. }
    ));

    let proxy = spawn(server::router(state.clone())).await;
    let resp = post_messages(&proxy, priced_message()).await;
    assert_eq!(resp.status().as_u16(), 429);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["type"], "budget_exceeded");
}

#[tokio::test]
async fn over_budget_without_enforce_allows_log_first() {
    // Pre-mortem default: over budget, enforce=false → warn + allow (200).
    let upstream = spawn(mock_ok()).await;
    let state = state_with_cost(
        &upstream,
        CostConfig {
            enabled: true,
            default_budget_usd: Some(0.0),
            enforce: false,
            prices: Default::default(),
        },
    );
    let proxy = spawn(server::router(state.clone())).await;

    let resp = post_messages(&proxy, priced_message()).await;
    assert_eq!(resp.status().as_u16(), 200, "log-first must not block");
}

// ── WIRE-01 Step 2 RED: cost real en el path productivo ─────────────────────
// El mock devuelve `usage{input_tokens:10, output_tokens:3}`; el path con
// buffer (exact cache on) debe registrar el lado output → output_tokens ≠ 0.
// Hoy FAIL: `record_response_usage` solo se invoca en unit tests.
#[tokio::test]
async fn buffered_response_records_output_tokens_nonzero() {
    use vanta_proxy::config::CacheConfig;

    let upstream = spawn(mock_ok()).await;
    let cfg = ProxyConfig {
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream.clone(),
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: CacheConfig {
            enabled: true,
            ..Default::default()
        },
        report: Default::default(),
        cost: CostConfig::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection: Default::default(),
    };
    let state = server::AppState::from_engine(cfg, seeded_engine()).unwrap();
    let proxy = spawn(server::router(state.clone())).await;

    let resp = post_messages(&proxy, priced_message()).await;
    assert_eq!(resp.status().as_u16(), 200);

    assert!(
        state.cost.output_tokens_by_session("s-cost") != 0,
        "buffered usage must land output_tokens in the ledger"
    );
}
