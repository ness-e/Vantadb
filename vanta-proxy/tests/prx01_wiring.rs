// PRX-01 wiring tests: advance trigger (header + route), classifier routing,
// real mem-command pipeline, upstream degraded tracker. E2E against the real
// router with an in-memory engine + mocked upstream.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;

use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

fn seeded_engine() -> std::sync::Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
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
    std::sync::Arc::new(engine)
}

fn state_for(upstream_url: &str) -> vanta_proxy::server::AppState {
    let cfg = vanta_proxy::config::ProxyConfig {
        report: Default::default(),
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

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

struct TestEnv {
    proxy_url: String,
}

async fn setup() -> TestEnv {
    let upstream = Router::new().route(
        "/v1/messages",
        post(|body: bytes::Bytes| async move {
            let _ = body;
            Json(json!({ "id": "msg-1", "content": [] }))
        }),
    );
    let upstream_url = spawn(upstream).await;
    let proxy_url = spawn(vanta_proxy::server::router(state_for(&upstream_url))).await;
    TestEnv { proxy_url }
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ── S2: classifier routing ─────────────────────────────────────────────

async fn post_messages(env: &TestEnv, session: &str, body: Value) -> reqwest::Response {
    client()
        .post(format!("{}/agent/space/v1/messages", env.proxy_url))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", session)
        .json(&body)
        .send()
        .await
        .unwrap()
}

fn turns_for(state: &vanta_proxy::server::AppState, session: &str) -> Vec<String> {
    vanta_proxy::capture::list_turns(&state.memory)
        .into_iter()
        .filter(|r| r.payload.contains(session))
        .map(|r| r.payload)
        .collect()
}

#[tokio::test]
async fn sidequery_with_session_skips_capture_but_main_captures() {
    let upstream = Router::new().route(
        "/v1/messages",
        post(|body: bytes::Bytes| async move {
            let _ = body;
            Json(json!({ "id": "msg-1", "content": [] }))
        }),
    );
    let upstream_url = spawn(upstream).await;
    let state = state_for(&upstream_url);
    let probe = state.clone();
    let proxy_url = spawn(vanta_proxy::server::router(state)).await;
    let env = TestEnv { proxy_url };

    // Main turn WITH session → full pipeline → turn captured.
    let mut marked = json!({"role": "user", "content": [{"type": "text", "text": "real work"}]});
    marked["content"][0]["cache_control"] = json!({"type": "ephemeral"});
    let resp = post_messages(
        &env,
        "sess-main-1",
        json!({"model": "claude-x", "messages": [marked]}),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Sidequery WITH session → verbatim bypass → nothing captured.
    let resp = post_messages(
        &env,
        "sess-side-1",
        json!({
            "model": "claude-x",
            "messages": [{"role": "user", "content": "title this"}],
            "thinking": {"type": "disabled"},
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Poll until the main turn lands (async fire-and-forget write path).
    let mut main_landed = false;
    for _ in 0..100 {
        if !turns_for(&probe, "sess-main-1").is_empty() {
            main_landed = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(main_landed, "main turn must be captured");
    assert!(
        turns_for(&probe, "sess-side-1").is_empty(),
        "sidequery must bypass capture"
    );
}

// ── S1: advance trigger ────────────────────────────────────────────────

#[tokio::test]
async fn session_advance_route_moves_team_to_agent() {
    let env = setup().await;
    let resp = client()
        .post(format!("{}/session/advance", env.proxy_url))
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "sess-route-1")
        .json(&json!({ "target": "agent", "entity_id": "agent-1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["stage"], "agent");
}

#[tokio::test]
async fn session_advance_route_rejects_unknown_entity() {
    let env = setup().await;
    let resp = client()
        .post(format!("{}/session/advance", env.proxy_url))
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "sess-route-2")
        .json(&json!({ "target": "agent", "entity_id": "agent-missing" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn session_advance_route_requires_auth_and_key_and_target() {
    let env = setup().await;
    // No user key → 401.
    let resp = client()
        .post(format!("{}/session/advance", env.proxy_url))
        .header("x-vanta-session", "sess-route-3")
        .json(&json!({ "target": "agent", "entity_id": "agent-1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    // No session key → 400.
    let resp = client()
        .post(format!("{}/session/advance", env.proxy_url))
        .header("x-vanta-user-key", USER_KEY)
        .json(&json!({ "target": "agent", "entity_id": "agent-1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    // Bad target → 400.
    let resp = client()
        .post(format!("{}/session/advance", env.proxy_url))
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "sess-route-3")
        .json(&json!({ "target": "bogus", "entity_id": "agent-1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// S4: degraded tracker — 3×upstream 503 → degraded, 5×200 → recover.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

async fn snapshot_degraded(proxy_url: &str) -> bool {
    client()
        .get(format!("{proxy_url}/snapshot"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["rate_limit"]["degraded"]
        .as_bool()
        .unwrap()
}

#[tokio::test]
async fn upstream_503_trips_degraded_and_200s_recover() {
    // Upstream serves 503 for the first 3 forwards, 200 after that.
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_up = calls.clone();
    let upstream = Router::new().route(
        "/v1/messages",
        post(move || {
            let calls_up = calls_up.clone();
            async move {
                let n = calls_up.fetch_add(1, Ordering::SeqCst);
                if n < 3 {
                    axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response()
                } else {
                    Json(json!({ "id": "msg-1", "content": [] })).into_response()
                }
            }
        }),
    );
    let upstream_url = spawn(upstream).await;
    let proxy_url = spawn(vanta_proxy::server::router(state_for(&upstream_url))).await;
    let env = TestEnv { proxy_url };

    let body = || {
        json!({
            "model": "claude-x",
            "messages": [{"role": "user", "content": "health probe"}],
        })
    };
    for _ in 0..3 {
        let resp = post_messages(&env, "sess-deg-1", body()).await;
        assert_eq!(resp.status(), 503);
    }
    assert!(
        snapshot_degraded(&env.proxy_url).await,
        "3×upstream 503 must flip degraded"
    );

    for _ in 0..5 {
        let resp = post_messages(&env, "sess-deg-1", body()).await;
        assert_eq!(resp.status(), 200);
    }
    assert!(
        !snapshot_degraded(&env.proxy_url).await,
        "5×éxito must recover out of degraded"
    );
}
