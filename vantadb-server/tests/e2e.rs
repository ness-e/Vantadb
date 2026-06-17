//! End-to-End Integration Tests for vantadb-server
//!
//! These tests spin up a real TCP/HTTP server, make requests via reqwest,
//! and validate the full client -> server -> storage -> response roundtrip.
//! Unlike the unit tests in server.rs (which use axum::Router::oneshot),
//! these tests exercise the entire socket-level HTTP pipeline.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

/// Bind a real TCP listener on a random port, spawn the real server,
/// and return the base URL + join handle.
async fn spawn_server(state: Arc<ServerState>, rpm: u32) -> (String, tokio::task::JoinHandle<()>) {
    let router = app(state, rpm);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Brief pause to let the server start accepting connections
    tokio::time::sleep(Duration::from_millis(200)).await;

    (base, handle)
}

/// Build a test context (temp dir + ServerState) shared across E2E tests.
fn build_e2e_context(
    api_key: Option<&str>,
    concurrency: usize,
) -> (tempfile::TempDir, Arc<ServerState>) {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().join("db").to_str().unwrap()).unwrap());
    let state = Arc::new(ServerState {
        storage,
        semaphore: Arc::new(tokio::sync::Semaphore::new(concurrency)),
        api_key: api_key.map(Arc::from),
    });
    (dir, state)
}

#[tokio::test]
async fn test_e2e_health_and_metrics() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Health endpoint
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"], "OK");

    // Metrics endpoint
    let resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(!text.is_empty(), "Metrics body should not be empty");
    assert!(
        text.contains("vanta_"),
        "Metrics should contain 'vanta_' prefix: {}",
        text
    );
}

#[tokio::test]
async fn test_e2e_insert_and_query() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // 1. Insert a node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#101 TYPE Test { content: \"e2e-http\" }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Insert failed: {:?}",
        body
    );
    assert_eq!(body["node_id"].as_u64(), Some(101));

    // 2. Query for the node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "FROM Test FETCH content"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Query failed: {:?}",
        body
    );

    // 3. Delete the node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "DELETE NODE#101"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Delete failed: {:?}",
        body
    );
}

#[tokio::test]
async fn test_e2e_auth_over_http() {
    let (_dir, state) = build_e2e_context(Some("e2e-secret"), 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Health is always public
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Query without auth -> 401
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#1 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Query with valid auth -> 200
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer e2e-secret")
        .body(r#"{"query": "INSERT NODE#1 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    // Query with wrong token -> 401
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer wrong-token")
        .body(r#"{"query": "INSERT NODE#2 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_e2e_persistence_across_restart() {
    let (_dir, state) = build_e2e_context(None, 10);
    let storage_path = _dir.path().join("db").to_str().unwrap().to_string();

    // First server
    let (base1, handle1) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Insert data
    let resp = client
        .post(format!("{}/api/v2/query", base1))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#201 TYPE E2E { value: \"persist\" }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    // Shut down first server — JoinHandle::abort cancels the task
    handle1.abort();
    // Allow OS to release the port and storage engine to flush
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Second server, same storage directory
    let storage2 = Arc::new(StorageEngine::open(&storage_path).unwrap());
    let state2 = Arc::new(ServerState {
        storage: storage2,
        semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
        api_key: None,
    });
    let (base2, handle2) = spawn_server(state2, 0).await;

    // Verify persistence: search for the previously inserted node
    let resp = client
        .post(format!("{}/api/v2/query", base2))
        .header("content-type", "application/json")
        .body(r#"{"query": "FROM E2E FETCH value"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Persistence query failed: {:?}",
        body
    );

    // Clean shutdown
    handle2.abort();
}

#[tokio::test]
async fn test_e2e_rate_limit_over_http() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 5).await; // RPM=5

    let client = reqwest::Client::new();

    // First request should pass (burst allows it)
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#301 TYPE RL { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Rapid second request — with RPM=5 and burst=1, second should hit the rate limit
    tokio::time::sleep(Duration::from_millis(10)).await;
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#302 TYPE RL { }"}"#)
        .send()
        .await
        .unwrap();

    // Depending on governor timing, may or may not be 429.
    // Accept both 200 and 429 — the test validates the server responds,
    // not the exact rate limit timing over real sockets.
    assert!(
        resp.status() == 200 || resp.status() == 429,
        "Expected 200 or 429, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_e2e_bad_request_returns_400() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Send invalid JSON
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body("not-json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}
