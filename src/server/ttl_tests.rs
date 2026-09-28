//! WIRE-04 — TTL surface end-to-end over the real axum router (in-crate).
//!
//! Covers the contract's three clauses with independent evidence:
//! 1. explicit `ttl_ms` → expires → `get` 404 → `purge_expired` ≥ 1 (physical purge);
//! 2. namespace default TTL (`Config::memory_default_ttl_ms`) applied to new writes only;
//! 3. background sweeper physically purges memory + derived indexes (node absent from
//!    the engine, not merely hidden by lazy read filtering) and stops on `shutdown()`.
//!
//! Uses `oneshot` on the real router (no TCP port); storage is a temp dir so the
//! physical-absence assertions (`engine.get`) run against the real engine.

// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::connection_pool::ConnectionPool;
use crate::sdk::Embedded;
use crate::server::router::app;
use crate::server::state::ServerState;
use crate::storage::StorageEngine;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

struct TtlContext {
    _temp_dir: tempfile::TempDir,
    state: Arc<ServerState>,
}

/// Build production-shaped state over a temp dir. `namespace_defaults`
/// mirrors the `VANTADB_MEMORY_DEFAULT_TTL_MS` config surface (namespace → ms).
fn ttl_state(namespace_defaults: &[(&str, u64)]) -> TtlContext {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    let config = Config {
        storage_path: path.to_string(),
        memory_default_ttl_ms: namespace_defaults
            .iter()
            .map(|(ns, ms)| (ns.to_string(), *ms))
            .collect(),
        ..Default::default()
    };
    let storage = Arc::new(StorageEngine::open_with_config(path, Some(config)).expect("open"));
    let db = Embedded::from_engine(storage.clone());
    db.ensure_indexes_current().expect("ensure indexes");
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(500))),
        api_key: None,
        alt_api_key: None,
        jwt_secret: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    TtlContext {
        _temp_dir: dir,
        state,
    }
}

fn add_addr(req: Request<Body>) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts
        .extensions
        .insert(ConnectInfo::<SocketAddr>(SocketAddr::from((
            [127, 0, 0, 1],
            54321,
        ))));
    Request::from_parts(parts, body)
}

async fn request(
    router: &mut axum::Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri).method(method);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let req = builder
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap();
    let response = router.oneshot(add_addr(req)).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// Poll `check` until true or the deadline elapses. Deadline-driven (not a fixed
/// sleep): expiry is a wall-clock event, so a bounded wait is the honest
/// synchronization primitive here.
async fn wait_until(timeout: Duration, mut check: impl FnMut() -> bool) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if check() {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn node_id_of(body: &str) -> u128 {
    let record: serde_json::Value = serde_json::from_str(body).expect("record JSON");
    record["node_id"]
        .as_str()
        .expect("node_id serialized as string (u128_serde)")
        .parse()
        .expect("node_id is a u128")
}

// ── 1. Explicit `ttl_ms` over HTTP: put → expire → 404 → purge ≥ 1 ──

#[tokio::test]
async fn ttl_http_explicit_ttl_expires_then_purges_physically() {
    let ctx = ttl_state(&[]);
    let mut router = app(ctx.state.clone(), 0);

    // Arrange: PUT a record whose TTL lapses quickly.
    let input = r#"{"namespace":"ttl-explicit","key":"k1","payload":"hello ttl","metadata":{},"ttl_ms":40}"#;
    let (status, body) = request(&mut router, "POST", "/api/v2/records", Some(input)).await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let record: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        record["expires_at_ms"].as_u64().is_some(),
        "ttl_ms must be resolved to expires_at_ms: {body}"
    );
    let node_id = node_id_of(&body);

    // Act: wait for the deadline (wall-clock event).
    let expired = wait_until(Duration::from_secs(2), || {
        ctx.state
            .db
            .get("ttl-explicit", "k1")
            .expect("get")
            .is_none()
    })
    .await;
    assert!(expired, "record must be read-hidden after its TTL lapses");

    // Assert (lazy read path): HTTP GET reports 404 while the node is still stored.
    let (status, body) = request(&mut router, "GET", "/api/v2/records/ttl-explicit/k1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");

    // Assert (physical presence): the node is still in the engine — the 404 above
    // was lazy read filtering, not deletion.
    assert!(
        ctx.state.storage.get(node_id).unwrap().is_some(),
        "expired-but-unpurged node must still be physically present"
    );

    // Assert (contract): purge_expired physically removes it.
    let purged = ctx.state.db.purge_expired().unwrap();
    assert!(purged >= 1, "purge_expired must remove the expired record");
    assert!(
        ctx.state.storage.get(node_id).unwrap().is_none(),
        "node must be physically gone from storage after purge"
    );
}

// ── 2. Namespace default TTL over HTTP (new writes only) ──

#[tokio::test]
async fn ttl_http_namespace_default_applies_to_new_writes() {
    let ctx = ttl_state(&[("ttl-default", 40)]);
    let mut router = app(ctx.state.clone(), 0);

    // New write without `ttl_ms` inherits the namespace default.
    let (status, body) = request(
        &mut router,
        "POST",
        "/api/v2/records",
        Some(r#"{"namespace":"ttl-default","key":"d1","payload":"inherits","metadata":{}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let record: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        record["expires_at_ms"].as_u64().is_some(),
        "namespace default must resolve expires_at_ms: {body}"
    );

    // A namespace without configuration keeps the never-expires semantics.
    let (status, body) = request(
        &mut router,
        "POST",
        "/api/v2/records",
        Some(r#"{"namespace":"ttl-other","key":"d2","payload":"no ttl","metadata":{}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let other: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        other["expires_at_ms"].is_null(),
        "unconfigured namespace must not expire: {body}"
    );

    // Expiry is enforced on the read path.
    let lapsed = wait_until(Duration::from_secs(2), || {
        ctx.state
            .db
            .get("ttl-default", "d1")
            .expect("get")
            .is_none()
    })
    .await;
    assert!(lapsed, "defaulted TTL must lapse");
    let (status, _) = request(&mut router, "GET", "/api/v2/records/ttl-default/d1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        ctx.state.db.purge_expired().unwrap() >= 1,
        "defaulted record must flow through the purge path"
    );
}

// ── 3. Background sweeper: physical purge with no manual `purge_expired` ──

#[tokio::test]
async fn ttl_http_sweeper_physically_purges_without_manual_call() {
    let ctx = ttl_state(&[("ttl-sweep", 30)]);
    let mut router = app(ctx.state.clone(), 0);

    // The same production entry point the server bootstrap spawns.
    let sweeper =
        crate::gc::spawn_memory_ttl_sweeper(ctx.state.db.clone(), Duration::from_millis(20));

    let (status, body) = request(
        &mut router,
        "POST",
        "/api/v2/records",
        Some(r#"{"namespace":"ttl-sweep","key":"s1","payload":"sweep me","metadata":{}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let node_id = node_id_of(&body);

    // No manual purge anywhere: the sweeper alone must physically remove the node.
    let purged = wait_until(Duration::from_secs(3), || {
        ctx.state
            .storage
            .get(node_id)
            .expect("engine get")
            .is_none()
    })
    .await;
    assert!(purged, "sweeper must physically purge the expired node");

    // Read path agrees with the physical state.
    let (status, _) = request(&mut router, "GET", "/api/v2/records/ttl-sweep/s1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Nothing left to purge → proof the sweeper (not the read filter) deleted it.
    assert_eq!(
        ctx.state.db.purge_expired().unwrap(),
        0,
        "sweeper must have already purged everything"
    );

    sweeper.shutdown().await;
}
