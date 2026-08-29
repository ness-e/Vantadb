//! Integration tests for SRV-02 — tracing-id propagation (x-request-id /
//! x-tracing-id / traceparent → audit log JSONL).
//!
//! Pattern borrowed from qdrant v1.18: the caller's tracing id (first match of
//! the three headers, max 256 chars) is captured by
//! `request_metrics_middleware` and carried into every `AuditEvent` produced
//! by that request.
//!
//! ponytail: these tests do NOT duplicate the unit-level coverage of
//! `AuditEvent::with_request_id` roundtrip (lives in `src/audit.rs`) nor the
//! e2e coverage already present in `src/cli_server.rs::audit_event_carries_request_id`.
//! They exist to satisfy the `cargo test --test request_id` gate in the plan
//! file AND to give external contributors a single, discoverable place where
//! the contract is documented.

use std::sync::Arc;
use std::time::Duration;

use vantadb::audit::AuditEvent;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::cli_server::{app, ServerState};
use vantadb::config::VantaConfig;
use vantadb::connection_pool::ConnectionPool;
use vantadb::storage::StorageEngine;
use vantadb::{BackendKind, VantaEmbedded};

// ── helpers ─────────────────────────────────────────────────────────────

/// Spawn the app router on an ephemeral port, returning its address.
async fn spawn_app(router: axum::Router) -> std::net::SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

/// Send a raw HTTP/1.1 request and return the full response text.
async fn raw_request(addr: std::net::SocketAddr, request: String) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

/// Build the minimal `ServerState` needed to exercise the auth + metrics
/// middlewares (the two layers that emit audit events on auth failures).
fn build_state(audit_path: &std::path::Path) -> Arc<ServerState> {
    let cfg = VantaConfig {
        backend_kind: BackendKind::InMemory,
        audit_log_path: Some(audit_path.to_path_buf()),
        ..Default::default()
    };
    let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
    let db = VantaEmbedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: Some("test-key".into()),
        alt_api_key: None,
        rbac_config: Default::default(),
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    })
}

/// Drive one request through the server and return the tail of the audit
/// JSONL — the only line produced (auth_l1 err on missing token).
async fn drive_and_read_last_audit_line(
    addr: std::net::SocketAddr,
    audit_path: &std::path::Path,
    extra_headers: &[&str],
) -> String {
    let body = r#"{"namespace":"mem","key":"k1","payload":"x"}"#;
    let mut headers = format!(
        "Host: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for h in extra_headers {
        headers.push_str(h);
        headers.push_str("\r\n");
    }
    let request = format!("POST /api/v2/records HTTP/1.1\r\n{headers}\r\n{body}");

    let raw = raw_request(addr, request).await;
    // Wrong/missing Bearer → 401, but the auth_l1 event is still recorded.
    assert!(
        raw.starts_with("HTTP/1.1 401"),
        "auth must reject unauthenticated request, got: {raw}"
    );

    let content = std::fs::read_to_string(audit_path).unwrap();
    content.lines().last().unwrap().to_string()
}

// ── tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn x_request_id_propagates_into_audit_event() {
    // The three accepted headers, first-match-wins: x-request-id wins.
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let state = build_state(&audit_path);
    let addr = spawn_app(app(state, 0)).await;

    let last = drive_and_read_last_audit_line(addr, &audit_path, &["X-Request-Id: abc-123"]).await;
    assert!(
        last.contains("\"request_id\":\"abc-123\""),
        "audit event must carry the x-request-id, got: {last}"
    );
    // Roundtrip parse confirms the field shape is canonical.
    let event: AuditEvent = serde_json::from_str(&last).unwrap();
    assert_eq!(event.request_id.as_deref(), Some("abc-123"));
}

#[tokio::test]
async fn x_tracing_id_and_traceparent_are_alternate_aliases() {
    // When x-request-id is absent, the next two accepted aliases must each
    // surface in the audit log. Proves the precedence list is wired end-to-end.
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let state = build_state(&audit_path);
    let addr = spawn_app(app(state, 0)).await;

    // x-tracing-id alone.
    let last =
        drive_and_read_last_audit_line(addr, &audit_path, &["X-Tracing-Id: trace-alias"]).await;
    assert!(
        last.contains("\"request_id\":\"trace-alias\""),
        "x-tracing-id must populate request_id, got: {last}"
    );

    // traceparent alone (W3C trace-context shape — opaque to us).
    let last = drive_and_read_last_audit_line(
        addr,
        &audit_path,
        &["Traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"],
    )
    .await;
    assert!(
        last.contains("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"),
        "traceparent must be captured verbatim, got: {last}"
    );
}

#[tokio::test]
async fn missing_tracing_header_omits_request_id_field() {
    // Backwards-compat: a request without any of the three headers MUST NOT
    // serialize a `request_id` field (keeps the JSONL line shape compatible
    // with logs produced before SRV-02 landed).
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let state = build_state(&audit_path);
    let addr = spawn_app(app(state, 0)).await;

    let last = drive_and_read_last_audit_line(addr, &audit_path, &[]).await;
    assert!(
        !last.contains("request_id"),
        "no tracing header → no request_id field, got: {last}"
    );
    let event: AuditEvent = serde_json::from_str(&last).unwrap();
    assert_eq!(event.request_id, None);
}
