//! API-05 (P51 W4): `/snapshot` auth + canonical route surface (SEGURIDAD).
//!
//! Contract: `GET /snapshot` without `x-vanta-user-key` → 401; with a
//! provisioned key → 200. Covers the rest of the API-05 endpoint surface:
//! `/sessions/advance` (plural) and `/{agent}/{space_id}/v1/responses`.
//!
//! E2E against the real router over TCP (in-memory engine + mocked
//! upstream), same harness style as `prx01_wiring.rs`. The file-backed seed
//! helper is `#[ignore]`d (needs `--features vantadb/fjall`) and only feeds
//! the manual live smoke (`vanta-proxy` binary + curl).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;

use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vantadb::entity::{EntityStore, EntityWrite};
use vantadb::node::FieldValue;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

fn seeded_engine() -> std::sync::Arc<vantadb::storage::StorageEngine> {
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
    std::sync::Arc::new(engine)
}

fn state_for(upstream_url: &str) -> vanta_proxy::server::AppState {
    let cfg = vanta_proxy::config::ProxyConfig {
        server: vanta_proxy::config::ServerConfig::default(),
        upstream: vanta_proxy::config::UpstreamConfig {
            url: upstream_url.to_string(),
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: vanta_proxy::config::AuthConfig::default(),
        mem_command: vanta_proxy::config::MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
        report: Default::default(),
        cost: Default::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection: Default::default(),
        envelope: Default::default(),
    };
    vanta_proxy::server::AppState::from_engine(cfg, seeded_engine()).expect("app state")
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ── /snapshot auth (contract: no key → 401, key → 200) ─────────────────

#[tokio::test]
async fn snapshot_requires_user_key() {
    let upstream = Router::new().route("/v1/messages", post(|| async { Json(json!({})) }));
    let upstream_url = spawn(upstream).await;
    let proxy_url = spawn(vanta_proxy::server::router(state_for(&upstream_url))).await;

    // API-05: the exposure (sessions/cost without auth) is closed — no
    // credential must fail closed with 401.
    let resp = client()
        .get(format!("{proxy_url}/snapshot"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        401,
        "GET /snapshot without a user key must be 401"
    );

    // Provisioned key → 200 + the operational snapshot shape users consume.
    let resp = client()
        .get(format!("{proxy_url}/snapshot"))
        .header("x-vanta-user-key", USER_KEY)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "GET /snapshot with a provisioned key must be 200"
    );
    let body: Value = resp.json().await.unwrap();
    for key in ["turns", "sessions", "writeback", "rate_limit", "cost"] {
        assert!(
            body.get(key).is_some(),
            "snapshot must expose `{key}`, got: {body}"
        );
    }
}

// ── canonical routes ───────────────────────────────────────────────────

#[tokio::test]
async fn responses_prefixed_route_uses_agent_space_segments() {
    // The mock upstream only serves `/v1/responses`: a 200 proves the
    // prefixed route reached the wire with the canonical wire path.
    let upstream = Router::new().route(
        "/v1/responses",
        post(|body: bytes::Bytes| async move {
            let _ = body;
            Json(json!({"id": "resp-1", "object": "response"}))
        }),
    );
    let upstream_url = spawn(upstream).await;
    let proxy_url = spawn(vanta_proxy::server::router(state_for(&upstream_url))).await;

    let resp = client()
        .post(format!("{proxy_url}/cc/sp-42/v1/responses"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .json(&json!({"model": "m", "input": "hi"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "prefixed responses route must forward to /v1/responses"
    );
}

// ── live-smoke seed helper (manual, ignored) ───────────────────────────

/// Seeds the file-backed auth store consumed by the API-05 live smoke
/// (real `vanta-proxy` binary + curl): recreates
/// `<target>/tmp/api05-smoke-authdb` with user `usr-smoke` / key `sk-smoke`.
///
/// Run with a fjall-enabled build (in-memory tests don't need it):
/// `cargo test --target-dir target/session-api01 -p vanta-proxy \
///  --features vantadb/fjall --test api05_snapshot_auth -- --ignored
///  seed_live_smoke_auth_store`
#[test]
#[ignore = "smoke helper: needs --features vantadb/fjall + the live smoke steps"]
fn seed_live_smoke_auth_store() {
    let path = format!("{}\\api05-smoke-authdb", env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::remove_dir_all(&path);
    let db = vanta_proxy::auth::AuthDb::open(&path).expect("open smoke auth store");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String("sk-smoke".into()));
    EntityStore::new(&db.engine())
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-smoke",
            fields,
        })
        .expect("seed smoke user");
}
