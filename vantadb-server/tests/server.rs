//! API Server & Health Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../../tests/common/mod.rs"]
mod common;

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use std::net::SocketAddr;
use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tower::ServiceExt;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

struct TestContext {
    _temp_dir: tempfile::TempDir,
    state: Arc<ServerState>,
}

fn build_context(api_key: Option<&str>, concurrency: usize) -> TestContext {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap());
    let state = Arc::new(ServerState {
        storage,
        semaphore: Arc::new(tokio::sync::Semaphore::new(concurrency)),
        api_key: api_key.map(Arc::from),
    });
    TestContext {
        _temp_dir: temp_dir,
        state,
    }
}

fn add_addr(req: Request<Body>) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts.extensions.insert(ConnectInfo::<SocketAddr>(SocketAddr::from(([127, 0, 0, 1], 54321))));
    Request::from_parts(parts, body)
}

async fn get(app: &mut axum::Router, uri: &str, auth_token: Option<&str>) -> StatusCode {
    let mut req = Request::builder().uri(uri).method("GET");
    if let Some(token) = auth_token {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    app.oneshot(add_addr(req.body(Body::empty()).unwrap()))
        .await
        .unwrap()
        .status()
}

async fn post_query(app: &mut axum::Router, auth_token: Option<&str>) -> StatusCode {
    let mut req = Request::builder()
        .uri("/api/v2/query")
        .method("POST")
        .header("content-type", "application/json");
    if let Some(token) = auth_token {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    app.oneshot(add_addr(req.body(Body::from(r#"{"query":"test"}"#)).unwrap()))
        .await
        .unwrap()
        .status()
}

// ─── TSK-14: Authentication (Bearer Token) ────────────────────────────────

#[tokio::test]
async fn test_auth_no_auth_mode() {
    let ctx = build_context(None, 10);
    let mut router = app(ctx.state, 0);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_auth_valid_token() {
    let ctx = build_context(Some("valid-key"), 10);
    let mut router = app(ctx.state, 0);

    let status = post_query(&mut router, Some("valid-key")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_auth_invalid_token() {
    let ctx = build_context(Some("valid-key"), 10);
    let mut router = app(ctx.state, 0);

    let status = post_query(&mut router, Some("wrong-key")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_missing_header() {
    let ctx = build_context(Some("valid-key"), 10);
    let mut router = app(ctx.state, 0);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_wrong_scheme() {
    let ctx = build_context(Some("valid-key"), 10);
    let mut router = app(ctx.state, 0);

    let req = add_addr(
        Request::builder()
            .uri("/api/v2/query")
            .method("POST")
            .header("content-type", "application/json")
            .header("Authorization", "Basic dGVzdDp0ZXN0")
            .body(Body::from(r#"{"query":"test"}"#))
            .unwrap(),
    );
    let status = router.oneshot(req).await.unwrap().status();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_health_exempt() {
    let ctx = build_context(Some("valid-key"), 10);
    let mut router = app(ctx.state, 0);

    let status = get(&mut router, "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

// ─── TSK-15: Rate Limiting ────────────────────────────────────────────────

#[tokio::test]
async fn test_rate_limit_disabled_with_zero_rpm() {
    let ctx = build_context(None, 10);
    let mut router = app(ctx.state, 0);

    for i in 0..10 {
        let status = post_query(&mut router, None).await;
        assert_eq!(status, StatusCode::OK, "request {} should pass at RPM=0", i);
    }
}

#[tokio::test]
async fn test_rate_limit_enforces_after_burst() {
    let ctx = build_context(None, 10);
    let mut router = app(ctx.state, 5);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::OK);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_health_unaffected() {
    let ctx = build_context(None, 10);
    let mut router = app(ctx.state, 5);

    let status = get(&mut router, "/health", None).await;
    assert_eq!(status, StatusCode::OK);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::OK);

    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

    let status = get(&mut router, "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

// ─── Existing test ────────────────────────────────────────────────────────

#[tokio::test]
async fn api_server_certification() {
    let mut harness = VantaHarness::new("API LAYER (SERVER & HEALTH)");

    harness.execute("Health: Endpoint Availability & Router State", || {
        futures::executor::block_on(async {
            let temp_dir = tempfile::tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap());
            let state = Arc::new(ServerState {
                storage,
                semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
                api_key: None,
            });
            let app = app(state, 100);

            TerminalReporter::sub_step("Dispatching oneshot request to /health...");
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            TerminalReporter::success("API Health check passed.");
        });
    });
}
