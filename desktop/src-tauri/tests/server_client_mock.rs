//! Integration tests: typed `ServerClient` against a mock axum HTTP server
//! that mirrors the real VantaDB API contract (`src/cli_server.rs`):
//!   - GET  /health        (no auth)
//!   - GET  /metrics       (Bearer)
//!   - POST /api/v2/query  (Bearer, `{"query": "..."}`, envelope `success`)
//!
//! Validates: statement mapping for put/get/delete/list/search, auth header,
//! and domain-error handling for `success:false` (HTTP 200).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use vantadb_desktop_lib::connections::{ServerClient, ServerClientConfig};

const TEST_TOKEN: &str = "test-api-key-123";

/// What the mock server recorded for one request.
#[derive(Debug, Default, Clone)]
struct Recorded {
    queries: Vec<String>,
    saw_bearer: Vec<bool>,
}

type Shared = Arc<Mutex<Recorded>>;

fn mock_router(shared: Shared) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/api/v2/query", post(query))
        .with_state(shared)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": "OK"}))
}

async fn metrics(State(shared): State<Shared>, headers: HeaderMap) -> Result<String, StatusCode> {
    let auth_ok =
        headers.get("authorization") == Some(&format!("Bearer {TEST_TOKEN}").parse().unwrap());
    {
        let mut rec = shared.lock().unwrap();
        rec.saw_bearer.push(auth_ok);
    }
    if !auth_ok {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok("# TYPE vanta_queries_total counter".to_string())
}

async fn query(
    State(shared): State<Shared>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let auth_ok =
        headers.get("authorization") == Some(&format!("Bearer {TEST_TOKEN}").parse().unwrap());
    let statement = payload["query"].as_str().unwrap_or_default().to_string();

    {
        let mut rec = shared.lock().unwrap();
        rec.queries.push(statement.clone());
        rec.saw_bearer.push(auth_ok);
    }

    if !auth_ok {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"success": false, "error": "Unauthorized"})),
        );
    }

    // Domain failure: server returns 200 with success:false.
    if statement.contains("FAIL") {
        return (
            StatusCode::OK,
            Json(json!({"success": false, "data": "Execution Error: node not found"})),
        );
    }

    // Write-ish statements (INSERT/DELETE) return node_id.
    if statement.contains("INSERT") || statement.contains("DELETE") {
        return (
            StatusCode::OK,
            Json(json!({"success": true, "data": "Mutated 1 nodes: inserted", "node_id": 42})),
        );
    }

    // Read-ish statements return nodes.
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": "Read 1 nodes.",
            "node_id": null,
            "nodes": [{
                "id": 7,
                "semantic_cluster": 0,
                "relational": {"key": "k1", "namespace": "agent/main"},
                "hits": 3,
                "confidence_score": 0.95
            }]
        })),
    )
}

/// Spawn the mock server on an ephemeral port; return (client, shared, port).
async fn spawn() -> (ServerClient, Shared, u16) {
    let shared: Shared = Arc::new(Mutex::new(Recorded::default()));
    let app = mock_router(shared.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let port = addr.port();
    let cfg = ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port,
        token: Some(TEST_TOKEN.to_string()),
        timeout: std::time::Duration::from_secs(5),
    };
    let client = ServerClient::new(cfg).unwrap();
    (client, shared, port)
}

fn recorded(shared: &Shared) -> Recorded {
    shared.lock().unwrap().clone()
}

#[tokio::test]
async fn health_no_auth_ok() {
    let (client, _shared, _port) = spawn().await;
    let report = client.health().await.unwrap();
    assert!(report.ok);
    assert_eq!(report.data, "OK");
}

#[tokio::test]
async fn metrics_sends_bearer() {
    let (client, shared, _port) = spawn().await;
    let text = client.metrics().await.unwrap();
    assert!(text.contains("vanta_queries_total"));
    let rec = recorded(&shared);
    assert!(rec.saw_bearer.iter().any(|b| *b));
}

#[tokio::test]
async fn put_maps_insert_statement_with_auth() {
    let (client, shared, _port) = spawn().await;
    let resp = client
        .put(42, "memory", &[("key", "k1"), ("payload", "hello")])
        .await
        .unwrap();
    assert!(resp.success);
    assert_eq!(resp.node_id, Some(42));

    let rec = recorded(&shared);
    assert_eq!(rec.queries.len(), 1);
    assert!(rec.saw_bearer[0]);
    let stmt = &rec.queries[0];
    assert!(stmt.starts_with("INSERT NODE#42 TYPE memory"));
    assert!(stmt.contains("key: \"k1\""));
    assert!(stmt.contains("payload: \"hello\""));
}

#[tokio::test]
async fn put_escapes_quotes() {
    let (client, shared, _port) = spawn().await;
    let resp = client
        .put(1, "memory", &[("payload", "say \"hi\"")])
        .await
        .unwrap();
    assert!(resp.success);
    let rec = recorded(&shared);
    assert!(rec.queries[0].contains("payload: \"say \\\"hi\\\"\""));
}

#[tokio::test]
async fn get_maps_match_statement() {
    let (client, shared, _port) = spawn().await;
    let resp = client.get(7).await.unwrap();
    assert!(resp.success);
    let nodes = resp.nodes.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, 7);
    assert_eq!(nodes[0].relational["key"], "k1");

    let rec = recorded(&shared);
    assert_eq!(rec.queries[0], "MATCH NODE#7");
    assert!(rec.saw_bearer[0]);
}

#[tokio::test]
async fn delete_maps_delete_statement() {
    let (client, shared, _port) = spawn().await;
    let resp = client.delete(99).await.unwrap();
    assert!(resp.success);
    let rec = recorded(&shared);
    assert_eq!(rec.queries[0], "DELETE NODE#99");
    assert!(rec.saw_bearer[0]);
}

#[tokio::test]
async fn list_maps_from_statement() {
    let (client, shared, _port) = spawn().await;
    let resp = client.list("memory").await.unwrap();
    assert!(resp.success);
    let rec = recorded(&shared);
    assert_eq!(rec.queries[0], "FROM memory");
    assert!(rec.saw_bearer[0]);
}

#[tokio::test]
async fn search_maps_vector_statement_with_auth() {
    let (client, shared, _port) = spawn().await;
    let resp = client
        .search("memory", "content", "neural memory", 0.5)
        .await
        .unwrap();
    assert!(resp.success);

    let rec = recorded(&shared);
    assert_eq!(rec.queries.len(), 1);
    assert!(rec.saw_bearer[0]);
    let stmt = &rec.queries[0];
    assert!(stmt.starts_with("FROM memory WHERE content ~"));
    assert!(stmt.contains("\"neural memory\""));
    assert!(stmt.contains("min = 0.5"));
}

#[tokio::test]
async fn domain_failure_success_false_maps_to_http_domain_error() {
    let (client, _shared, _port) = spawn().await;
    let err = client.query("MATCH NODE#999 FAIL").await.unwrap_err();

    match err {
        vantadb_desktop_lib::VantaError::Http {
            kind,
            message,
            status,
        } => {
            assert_eq!(
                kind,
                vantadb_desktop_lib::error::HttpErrorKind::Domain,
                "success:false must be a domain error, got: {message}"
            );
            assert_eq!(status, Some(200), "server returns 200 for domain failures");
            assert!(message.contains("Execution Error"));
        }
        other => panic!("expected Http domain error, got {other:?}"),
    }
}

#[tokio::test]
async fn wrong_token_gets_unauthorized() {
    let (_client, _shared, port) = spawn().await;
    let cfg = ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port,
        token: Some("wrong-token".to_string()),
        timeout: std::time::Duration::from_secs(5),
    };
    let client = ServerClient::new(cfg).unwrap();
    let err = client
        .query("MATCH NODE#1")
        .await
        .expect_err("wrong token must fail");

    match err {
        vantadb_desktop_lib::VantaError::Http { kind, status, .. } => {
            assert_eq!(
                kind,
                vantadb_desktop_lib::error::HttpErrorKind::Unauthorized
            );
            assert_eq!(status, Some(401));
        }
        other => panic!("expected Http unauthorized, got {other:?}"),
    }
}

#[tokio::test]
async fn timeout_yields_http_error() {
    // Connect to a closed port → connection refused surfaces as Http::Other.
    let cfg = ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port: 1, // unlikely to be listening
        token: None,
        timeout: std::time::Duration::from_secs(1),
    };
    let client = ServerClient::new(cfg).unwrap();
    let err = client.health().await.expect_err("must fail");
    assert!(matches!(err, vantadb_desktop_lib::VantaError::Http { .. }));
}
