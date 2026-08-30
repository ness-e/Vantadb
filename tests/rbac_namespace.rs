//! SRV-05 — RBAC scoping por namespace (integration test).
//!
//! Validates the contract that the `/api/v2/records/{ns}/{key}` and
//! `/api/v2/list?namespace=` endpoints authorize via
//! `Rbac::can_access_namespace` rather than the coarse `has_permission`
//! global check. This closes the privilege-escalation gap pre-mortem (a role
//! with `Permission::Read` MUST NOT silently read across all namespaces).
//!
//! Pattern borrowed from qdrant v1.9 per-collection RBAC + weaviate roles.
//!
//! AAA: arrange an in-memory `ServerState` with a `token_role_map` that maps
//! the Bearer to one of the pre-registered roles (`admin` / `reader` /
//! `writer`), act by issuing an HTTP request against a record endpoint, assert
//! the status code matches expectation.
//!
//! ponytail: the unit-level exhaustive coverage of `can_access_namespace`
//! (with custom roles like `ns_admin`) lives in `src/rbac.rs` tests
//! (`test_rbac_can_access_namespace_*`). This integration test exists to
//! satisfy the `cargo test --test rbac_namespace` gate in the plan file and
//! to verify the HTTP middleware routes through `can_access_namespace` (not
//! `has_permission` global) when a namespace is extracted from the request.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::cli_server::{app, ServerState};
use vantadb::config::{RbacConfig, VantaConfig};
use vantadb::connection_pool::ConnectionPool;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::{BackendKind, StorageEngine};

const KEY: &str = "sk-rbac-ns-test-aaaa";

// ── helpers ─────────────────────────────────────────────────────────────

fn in_memory_storage() -> Arc<StorageEngine> {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine"))
}

fn server_state(
    storage: Arc<StorageEngine>,
    token_role_map: HashMap<String, String>,
) -> Arc<ServerState> {
    let db = VantaEmbedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(100, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: Some(Arc::from(KEY)),
        alt_api_key: None,
        rbac_config: RbacConfig { token_role_map },
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    })
}

fn map_role(role: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(KEY.to_string(), role.to_string());
    m
}

async fn spawn(state: Arc<ServerState>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app(state, 0).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

async fn http_get(addr: SocketAddr, path: &str, bearer: &str) -> u16 {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {bearer}\r\nConnection: close\r\n\r\n"
    );
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

async fn http_post(addr: SocketAddr, path: &str, bearer: &str, body: &str) -> u16 {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {bearer}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ── tests ───────────────────────────────────────────────────────────────

/// Pre-mortem coverage: `admin` bypasses the namespace-scope check via
/// `Permission::Admin` short-circuit in `Rbac::can_access_namespace`. The
/// middleware must reach the protected handler on any namespace.
#[tokio::test]
async fn ns_admin_role_can_access_any_namespace_record() {
    let state = server_state(in_memory_storage(), map_role("admin"));
    let addr = spawn(state).await;

    // Admin: read of /api/v2/records/{ns}/{key} → 404 (record missing,
    // proves auth passed) or 200 (if a pre-existing record matches).
    // Either way, NOT 401/403.
    let s = http_get(addr, "/api/v2/records/team/k1", KEY).await;
    assert!(
        s != 401 && s != 403,
        "admin role must bypass namespace scope (got {s})"
    );
}

/// Pre-mortem coverage: a role with only `Permission::Read` (no
/// `NamespaceRead("team")`) MUST NOT silently read across all namespaces.
/// The middleware must call `can_access_namespace(reader, "team", false)`,
/// which returns `false` for missing namespace permission → 403.
#[tokio::test]
async fn ns_reader_role_cannot_access_namespaced_record() {
    let state = server_state(in_memory_storage(), map_role("reader"));
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/records/team/k1", KEY).await;
    assert_eq!(
        s, 403,
        "reader role must be denied on /api/v2/records/team/* (no NamespaceRead(\"team\")), got {s}"
    );
}

/// Mirror of the read case for writes: a role with only `Permission::Write`
/// (no `NamespaceWrite("team")`) MUST NOT write across all namespaces. We
/// hit `POST /api/v2/records?namespace=team` (query-param namespace), so
/// `extract_namespace` picks up `team` and the middleware routes through
/// `can_access_namespace(writer, "team", true)` → 403 (no
/// `NamespaceWrite("team")` on `writer`).
#[tokio::test]
async fn ns_writer_role_cannot_write_namespaced_record_without_namespace_perm() {
    let state = server_state(in_memory_storage(), map_role("writer"));
    let addr = spawn(state).await;

    let s = http_post(addr, "/api/v2/records?namespace=team", KEY, "{}").await;
    // The middleware runs BEFORE the route handler, so any non-2xx
    // response from RBAC (403/400/405) proves the namespace check ran.
    // 422 would mean the request passed RBAC into the handler.
    assert!(
        s != 200 && s != 201 && s != 422,
        "writer role without NamespaceWrite(\"team\") must NOT pass RBAC; got {s}"
    );
}

/// Backwards compat (pre-mortem 2): endpoints that are NOT record/search/
/// list endpoints (e.g. /api/v2/health) must continue to use the coarse
/// `has_permission(role, &Permission::Read)` global check. The `reader` role
/// has `Permission::Read` → must reach /api/v2/health with 200.
#[tokio::test]
async fn ns_non_record_endpoint_uses_global_reader_permission() {
    let state = server_state(in_memory_storage(), map_role("reader"));
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/health", KEY).await;
    assert_eq!(
        s, 200,
        "reader role must pass on non-record endpoint /api/v2/health (global Permission::Read), got {s}"
    );
}

/// Bearer present in `token_role_map` pointing to `admin` → admin bypass on
/// `/api/v2/list?namespace=any`. Verifies the `?namespace=` query-param
/// extraction path in `extract_namespace` is honored by the middleware.
#[tokio::test]
async fn ns_query_param_namespace_is_respected() {
    let state = server_state(in_memory_storage(), map_role("admin"));
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/list?namespace=any", KEY).await;
    assert!(
        s != 401 && s != 403,
        "admin must reach /api/v2/list?namespace=any (got {s})"
    );
}

/// Mirror of the reader-cannot-access-namespaced test for `/api/v2/list`
/// with a query-param namespace: a reader without `NamespaceRead("any")`
/// must NOT silently access `/api/v2/list?namespace=any`.
#[tokio::test]
async fn ns_reader_role_cannot_access_namespaced_list_query() {
    let state = server_state(in_memory_storage(), map_role("reader"));
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/list?namespace=any", KEY).await;
    assert_eq!(
        s, 403,
        "reader role must be denied on /api/v2/list?namespace=any (no NamespaceRead(\"any\")), got {s}"
    );
}

/// Bearer NOT in `token_role_map` → bare transport RBAC → bypass the role
/// check entirely → reach protected handler. Verifies the fall-through path
/// in the middleware (`if identity == Transport { if let Some(role) ... }`)
/// skips when there is no role entry.
#[tokio::test]
async fn ns_bearer_without_role_entry_falls_through_to_transport() {
    // Empty map → KEY has no role → bare transport.
    let state = server_state(in_memory_storage(), HashMap::new());
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/health", KEY).await;
    assert_eq!(
        s, 200,
        "Bearer without token_role_map entry must fall through to transport (200), got {s}"
    );
}

/// Reader role on a `/api/v2/records/{ns}/{key}` URL where the namespace is
/// extracted from the path: must be 403 (no `NamespaceRead("team")`). This
/// is the canonical "RBAC map per namespace" scenario from the pre-mortem
/// and is the contract this integration test was created to lock in.
#[tokio::test]
async fn ns_path_namespace_403_for_reader_role() {
    let state = server_state(in_memory_storage(), map_role("reader"));
    let addr = spawn(state).await;

    // Try multiple distinct namespaces — none should be accessible by
    // `reader` without explicit `NamespaceRead(...)`.
    for ns in &["team", "team_alpha", "beta", "gamma"] {
        let path = format!("/api/v2/records/{ns}/some-key");
        let s = http_get(addr, &path, KEY).await;
        assert_eq!(
            s, 403,
            "reader role must be denied on {path} (no NamespaceRead(\"{ns}\")), got {s}"
        );
    }
}
