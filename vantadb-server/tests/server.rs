// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! API Server & Health Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

// The shared core harness (tests/common/mod.rs) references `cfg(feature = "sysinfo")`
// which is a core-crate feature, not declared in vantadb-server — silence the lint.
#[allow(unexpected_cfgs)]
#[path = "../../tests/common/mod.rs"]
mod common;

#[path = "helpers/mod.rs"]
mod helpers;

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use common::{TerminalReporter, VantaHarness};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::config::RbacConfig;
use vantadb::connection_pool::ConnectionPool;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

struct TestContext {
    _temp_dir: tempfile::TempDir,
    state: Arc<ServerState>,
}

fn build_context(api_key: Option<&str>, concurrency: usize) -> TestContext {
    let (_temp_dir, state) = helpers::build_server_state(Path::new(""), api_key, concurrency);
    TestContext { _temp_dir, state }
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
    app.oneshot(add_addr(
        req.body(Body::from(r#"{"query":"SELECT * FROM Node"}"#))
            .unwrap(),
    ))
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
    let router = app(ctx.state, 0);

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

// ─── RBAC: token→role enforcement (AuthState.token_role_map + Rbac) ───────

/// Builds a state whose `api_key` doubles as the RBAC token for the given role.
/// The middleware (cli_server.rs `auth_middleware`) resolves the role from
/// `token_role_map` after the constant-time key check and denies with 403 when
/// the role lacks the permission required by the HTTP method.
fn build_rbac_context(api_key: &str, role: &str) -> TestContext {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(10, Duration::from_millis(5000))),
        api_key: Some(Arc::from(api_key)),
        alt_api_key: None,
        rbac_config: RbacConfig {
            token_role_map: HashMap::from([(api_key.to_string(), role.to_string())]),
        },
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    TestContext {
        _temp_dir: dir,
        state,
    }
}

#[tokio::test]
async fn test_rbac_reader_forbidden_on_write() {
    let ctx = build_rbac_context("reader-key", "reader");
    let mut router = app(ctx.state, 0);

    // POST is a write; reader role holds only Read → 403 before execution.
    let status = post_query(&mut router, Some("reader-key")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rbac_writer_allowed_on_write() {
    let ctx = build_rbac_context("writer-key", "writer");
    let mut router = app(ctx.state, 0);

    let status = post_query(&mut router, Some("writer-key")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_rbac_admin_allowed_on_write() {
    let ctx = build_rbac_context("admin-key", "admin");
    let mut router = app(ctx.state, 0);

    // Admin role bypasses the per-permission check (has_permission short-circuits).
    let status = post_query(&mut router, Some("admin-key")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_rbac_reader_allowed_on_read_route() {
    let ctx = build_rbac_context("reader-key", "reader");
    let mut router = app(ctx.state, 0);

    // GET is a read; reader role holds Read → allowed.
    let status = get(&mut router, "/metrics", Some("reader-key")).await;
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

    // REST-01: without an API key the governor allows the full rpm as burst
    // (burst = 5 here) and replenishes every 12s. Fire BURST+1 requests
    // rapidly so no token is refunded mid-burst: the first `BURST` pass (200)
    // and the next one is rejected (429).
    const BURST: usize = 5;
    for i in 0..BURST {
        let status = post_query(&mut router, None).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "request {} should pass within the burst window",
            i
        );
    }
    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_health_unaffected() {
    let ctx = build_context(None, 10);
    let mut router = app(ctx.state, 5);

    // /health is a public route (not behind the governor), so it stays 200
    // even while the protected query route is rate-limited. Exhaust the
    // no-auth burst (rpm = 5) and assert the next query trips the limiter,
    // while /health remains untouched before and after.
    const BURST: usize = 5;

    let status = get(&mut router, "/health", None).await;
    assert_eq!(status, StatusCode::OK);

    for i in 0..BURST {
        let status = post_query(&mut router, None).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "request {} should pass within the burst window",
            i
        );
    }
    let status = post_query(&mut router, None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

    let status = get(&mut router, "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

// ─── TSK-17: Concurrent Requests ──────────────────────────────────────────

/// Helper: sends a POST /api/v2/query on an owned router (one clone per call).
async fn post_query_owned(router: axum::Router) -> StatusCode {
    let (mut parts, body) = Request::builder()
        .uri("/api/v2/query")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"query":"SELECT * FROM Node"}"#))
        .unwrap()
        .into_parts();
    parts
        .extensions
        .insert(ConnectInfo::<SocketAddr>(SocketAddr::from((
            [127, 0, 0, 1],
            54321,
        ))));
    router
        .oneshot(Request::from_parts(parts, body))
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn test_concurrency_parallel_requests() {
    let ctx = build_context(None, 10);
    let router = app(ctx.state, 0);

    let mut handles = Vec::new();
    for _ in 0..20 {
        handles.push(tokio::spawn(post_query_owned(router.clone())));
    }
    for (i, h) in handles.into_iter().enumerate() {
        assert_eq!(h.await.unwrap(), StatusCode::OK, "request {}", i);
    }
}

#[tokio::test]
async fn test_concurrency_batch_with_small_semaphore() {
    let ctx = build_context(None, 2);
    let router = app(ctx.state, 0);

    let mut handles = Vec::new();
    for _ in 0..10 {
        handles.push(tokio::spawn(post_query_owned(router.clone())));
    }
    for (i, h) in handles.into_iter().enumerate() {
        assert_eq!(h.await.unwrap(), StatusCode::OK, "request {}", i);
    }
}

#[tokio::test]
async fn test_concurrency_with_auth() {
    let ctx = build_context(Some("shared-key"), 5);
    let router = app(ctx.state, 0);

    let mut handles = Vec::new();
    for _ in 0..10 {
        let r = router.clone();
        handles.push(tokio::spawn(async move {
            let (mut parts, body) = Request::builder()
                .uri("/api/v2/query")
                .method("POST")
                .header("content-type", "application/json")
                .header("Authorization", "Bearer shared-key")
                .body(Body::from(r#"{"query":"SELECT * FROM Node"}"#))
                .unwrap()
                .into_parts();
            parts
                .extensions
                .insert(ConnectInfo::<SocketAddr>(SocketAddr::from((
                    [127, 0, 0, 1],
                    54321,
                ))));
            r.oneshot(Request::from_parts(parts, body))
                .await
                .unwrap()
                .status()
        }));
    }
    for (i, h) in handles.into_iter().enumerate() {
        assert_eq!(h.await.unwrap(), StatusCode::OK, "request {}", i);
    }
}

// ─── ENT-04: Circuit Breaker + Connection Pool ───────────────────────────

/// Forces the breaker open (threshold 1 → one failure trips it), then verifies
/// `/api/v2/query` fast-fails with 503 + `Retry-After` while open.
#[tokio::test]
async fn test_circuit_breaker_open_returns_503_with_retry_after() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
    let breaker = Arc::new(CircuitBreaker::new(1, Duration::from_secs(30)));
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: breaker.clone(),
        pool: Arc::new(ConnectionPool::new(10, Duration::from_millis(5000))),
        api_key: None,
        alt_api_key: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    breaker.record_failure(); // opens (threshold == 1)
    assert_eq!(
        breaker.state(),
        vantadb::circuit_breaker::CircuitState::Open
    );
    let router = app(state, 0);

    let req = add_addr(
        Request::builder()
            .uri("/api/v2/query")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"query":"test"}"#))
            .unwrap(),
    );
    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let retry_after = res
        .headers()
        .get(axum::http::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert_eq!(
        retry_after, "30",
        "Retry-After must mirror the open timeout"
    );
}

/// A half-open breaker admits exactly one probe; a successful probe closes it.
#[tokio::test]
async fn test_circuit_breaker_half_open_probe_success_closes() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
    let breaker = Arc::new(CircuitBreaker::new(1, Duration::from_secs(0)));
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: breaker.clone(),
        pool: Arc::new(ConnectionPool::new(10, Duration::from_millis(5000))),
        api_key: None,
        alt_api_key: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    breaker.record_failure(); // open (0s timeout → immediately eligible for probe)
    let router = app(state, 0);

    // First request claims the half-open probe slot and executes successfully.
    let req = add_addr(
        Request::builder()
            .uri("/api/v2/query")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"query":"SELECT * FROM Node"}"#))
            .unwrap(),
    );
    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "probe request should pass");
    assert_eq!(
        breaker.state(),
        vantadb::circuit_breaker::CircuitState::Closed,
        "successful probe must close the breaker"
    );
}

// ─── TSK-16: TLS/HTTPS (requires --features tls) ─────────────────────────
#[cfg(feature = "tls")]
fn setup_tls() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[cfg(feature = "tls")]
fn generate_test_cert(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    use rcgen::{CertificateParams, KeyPair};

    let key_pair = KeyPair::generate().unwrap();
    let params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    let cert = params.self_signed(&key_pair).unwrap();

    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, key_pair.serialize_pem()).unwrap();
    (cert_path, key_path)
}

#[cfg(feature = "tls")]
#[tokio::test]
async fn test_tls_config_loading() {
    setup_tls();
    let dir = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = generate_test_cert(dir.path());

    let result = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path).await;
    assert!(
        result.is_ok(),
        "RustlsConfig should load from valid PEM files"
    );
}

#[cfg(feature = "tls")]
#[tokio::test]
async fn test_build_tls13_config_loading() {
    setup_tls();
    let dir = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = generate_test_cert(dir.path());

    let config = vantadb::cli_server::build_tls13_config(
        cert_path.to_str().unwrap(),
        key_path.to_str().unwrap(),
    )
    .await
    .expect("build_tls13_config should load valid PEM files");

    assert!(
        config.alpn_protocols.contains(&b"h2".to_vec()),
        "ALPN should include h2"
    );
    assert!(
        config.alpn_protocols.contains(&b"http/1.1".to_vec()),
        "ALPN should include http/1.1"
    );
}

#[cfg(feature = "tls")]
#[tokio::test]
async fn test_tls_server_health_and_query() {
    setup_tls();
    let (dir, state) = helpers::build_server_state(Path::new("db"), Some("tls-key"), 10);
    let (cert_path, key_path) = generate_test_cert(dir.path());

    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .unwrap();

    let router = app(state, 0);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    tokio::spawn(async move {
        if let Err(e) = axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
        {
            eprintln!("TLS server exited with error: {}", e);
        }
    });

    // Wait for the TLS server to actually accept connections (event-based, not fixed sleep)
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        if tokio::time::Instant::now() >= deadline {
            panic!("TLS server at {} did not start within 5s", addr);
        }
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            break;
        }
        tokio::task::yield_now().await;
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Health (no auth required)
    let resp = client
        .get(format!("https://{}/health", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Query with valid auth
    let resp = client
        .post(format!("https://{}/api/v2/query", addr))
        .header("Authorization", "Bearer tls-key")
        .header("content-type", "application/json")
        .body(r#"{"query":"SELECT * FROM Node"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Query without auth (rejected)
    let resp = client
        .post(format!("https://{}/api/v2/query", addr))
        .header("content-type", "application/json")
        .body(r#"{"query":"test"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Query with invalid auth (rejected)
    let resp = client
        .post(format!("https://{}/api/v2/query", addr))
        .header("Authorization", "Bearer wrong-key")
        .header("content-type", "application/json")
        .body(r#"{"query":"test"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ─── Existing test ────────────────────────────────────────────────────────

#[tokio::test]
async fn api_server_certification() {
    let mut harness = VantaHarness::new("API LAYER (SERVER & HEALTH)");

    harness.execute("Health: Endpoint Availability & Router State", || {
        futures::executor::block_on(async {
            let (_dir, state) = helpers::build_server_state(Path::new(""), None, 10);
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
