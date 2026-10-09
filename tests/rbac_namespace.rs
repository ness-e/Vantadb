// ponytail: integration test unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

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
use vantadb::config::{Config, RbacConfig, RbacRoleCfg};
use vantadb::connection_pool::ConnectionPool;
use vantadb::sdk::{Embedded, MemoryInput};
use vantadb::storage::{BackendKind, StorageEngine};

const KEY: &str = "sk-rbac-ns-test-aaaa";

// ── helpers ─────────────────────────────────────────────────────────────

fn in_memory_storage() -> Arc<StorageEngine> {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine"))
}

/// Storage whose `Config.audit_log_path` feeds the server's `AuthState` audit
/// sink (MGR-04/MEMG-10 RBAC-denial audit tests).
fn in_memory_storage_with_audit(path: &std::path::Path) -> Arc<StorageEngine> {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        audit_log_path: Some(path.to_path_buf()),
        ..Default::default()
    };
    Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine"))
}

/// Append-only JSONL rows written by the audit logger.
fn audit_rows(path: &std::path::Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("audit jsonl line"))
        .collect()
}

fn server_state(
    storage: Arc<StorageEngine>,
    token_role_map: HashMap<String, String>,
) -> Arc<ServerState> {
    let db = Embedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(100, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: Some(Arc::from(KEY)),
        alt_api_key: None,
        jwt_secret: None,
        rbac_config: RbacConfig {
            token_role_map,
            ..Default::default()
        },
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

async fn http_delete(addr: SocketAddr, path: &str, bearer: &str) -> u16 {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let request = format!(
        "DELETE {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {bearer}\r\nConnection: close\r\n\r\n"
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
/// The middleware must call `can_access_namespace(reader, "team", AccessMode::Read)`,
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
/// `can_access_namespace(writer, "team", AccessMode::Write)` → 403 (no
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

// ── MGR-04/MEMG-10: RBAC denials are audited (per action) ──────────────────

/// A denied write is audited with action/namespace/role — metadata only
/// (the Bearer token never reaches the audit log).
#[tokio::test]
async fn rbac_write_denial_is_audited_with_action_namespace_and_role() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("audit.jsonl");
    let state = server_state(
        in_memory_storage_with_audit(&audit_path),
        map_role("writer"),
    );
    let addr = spawn(state).await;

    let s = http_post(addr, "/api/v2/records?namespace=team", KEY, "{}").await;
    assert_eq!(s, 403, "writer without NamespaceWrite(team) must be denied");

    let rows = audit_rows(&audit_path);
    let denied = rows
        .iter()
        .find(|r| r["op"] == "auth_rbac")
        .expect("RBAC denial must be audited (was silent before MEMG-10)");
    assert_eq!(denied["outcome"], "denied");
    assert_eq!(denied["namespace"], "team");
    assert_eq!(denied["key"], "writer");
    let reason = denied["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("action=write"), "reason: {reason}");
    assert!(reason.contains("scope=namespace"), "reason: {reason}");

    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    assert!(!raw.contains(KEY), "the Bearer token must never be audited");
}

/// A denied read is audited with `action=read` (per-action labels).
#[tokio::test]
async fn rbac_read_denial_is_audited_with_action_read() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("audit.jsonl");
    let state = server_state(
        in_memory_storage_with_audit(&audit_path),
        map_role("reader"),
    );
    let addr = spawn(state).await;

    let s = http_get(addr, "/api/v2/records/team/k1", KEY).await;
    assert_eq!(s, 403);

    let denied = audit_rows(&audit_path)
        .into_iter()
        .find(|r| r["op"] == "auth_rbac")
        .expect("read denial must be audited");
    assert_eq!(denied["namespace"], "team");
    let reason = denied["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("action=read"), "reason: {reason}");
}

/// A denied delete is audited with `action=delete` and the enforcement mode
/// it actually went through (`enforced=write` — today's write-covers-delete
/// mapping; the strict separation is FIND-301).
#[tokio::test]
async fn rbac_delete_denial_is_audited_with_action_delete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("audit.jsonl");
    let state = server_state(
        in_memory_storage_with_audit(&audit_path),
        map_role("writer"),
    );
    let addr = spawn(state).await;

    let s = http_delete(addr, "/api/v2/records/team/k1", KEY).await;
    assert_eq!(s, 403);

    let denied = audit_rows(&audit_path)
        .into_iter()
        .find(|r| r["op"] == "auth_rbac")
        .expect("delete denial must be audited");
    let reason = denied["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("action=delete"), "reason: {reason}");
    assert!(reason.contains("enforced=write"), "reason: {reason}");
}

// ── MEMG-04: aislamiento multi-tenant (2 credenciales namespace-scoped) ────

/// Alt Bearer for the second tenant (SRV-04 key pair = the two credentials a
/// server can hold today; N>2 provisioning is FIND-304).
const KEY_BETA: &str = "sk-tenant-beta-bbbb";

/// Two real tenant credentials: primary → `tenant_alpha` (ns `alpha`), alt →
/// `tenant_beta` (ns `beta`). Neither role has a global read/write/admin
/// permission, so cross-tenant access must be denied on every enabled surface
/// and any namespace the request fails to declare falls back to the coarse
/// check — which denies scoped roles (fail closed).
fn two_tenant_state_with_audit(audit: Option<&std::path::Path>) -> Arc<ServerState> {
    let mut token_role_map = HashMap::new();
    token_role_map.insert(KEY.to_string(), "tenant_alpha".to_string());
    token_role_map.insert(KEY_BETA.to_string(), "tenant_beta".to_string());
    let mut roles = HashMap::new();
    roles.insert(
        "tenant_alpha".to_string(),
        RbacRoleCfg {
            // `agent/main` covers the percent-encoded-path regression (R3).
            namespace_read: vec!["alpha".to_string(), "agent/main".to_string()],
            namespace_write: vec!["alpha".to_string(), "agent/main".to_string()],
        },
    );
    roles.insert(
        "tenant_beta".to_string(),
        RbacRoleCfg {
            namespace_read: vec!["beta".to_string()],
            namespace_write: vec!["beta".to_string()],
        },
    );
    let storage = match audit {
        Some(path) => in_memory_storage_with_audit(path),
        None => in_memory_storage(),
    };
    let db = Embedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(100, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: Some(Arc::from(KEY)),
        alt_api_key: Some(Arc::from(KEY_BETA)),
        jwt_secret: None,
        rbac_config: RbacConfig {
            token_role_map,
            roles,
        },
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    })
}

fn two_tenant_state() -> Arc<ServerState> {
    two_tenant_state_with_audit(None)
}

/// Read surface: own namespace passes RBAC (404 = missing record proves the
/// handler ran), sibling namespace is 403 in both directions.
#[tokio::test]
async fn tenant_reads_own_namespace_and_cannot_read_sibling() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own = http_get(addr, "/api/v2/records/alpha/k1", KEY).await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant must read its own namespace (got {own})"
    );

    let cross = http_get(addr, "/api/v2/records/beta/k1", KEY).await;
    assert_eq!(cross, 403, "alpha tenant must not read beta records");

    let cross_back = http_get(addr, "/api/v2/records/alpha/k1", KEY_BETA).await;
    assert_eq!(cross_back, 403, "beta tenant must not read alpha records");
}

/// Write surface with the namespace declared in the JSON **body** (the
/// pre-MEMG-04 hole: no `?namespace=` → no check).
#[tokio::test]
async fn tenant_writes_own_body_namespace_and_cannot_write_sibling() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own = http_post(
        addr,
        "/api/v2/records",
        KEY,
        r#"{"namespace":"alpha","key":"k1","payload":"hello","metadata":{},"vector":null,"ttl_ms":null}"#,
    )
    .await;
    assert_eq!(
        own, 201,
        "alpha tenant must write its own namespace (got {own})"
    );

    let cross = http_post(
        addr,
        "/api/v2/records",
        KEY,
        r#"{"namespace":"beta","key":"k1","payload":"x","metadata":{}}"#,
    )
    .await;
    assert_eq!(cross, 403, "alpha tenant must not write into beta");

    let cross_back = http_post(
        addr,
        "/api/v2/records",
        KEY_BETA,
        r#"{"namespace":"alpha","key":"k2","payload":"x","metadata":{}}"#,
    )
    .await;
    assert_eq!(cross_back, 403, "beta tenant must not write into alpha");
}

/// A matching `?namespace=` query param must not mask a foreign namespace
/// declared in the body — the check covers the union (query ∪ body).
#[tokio::test]
async fn tenant_query_namespace_cannot_mask_foreign_body_namespace() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let s = http_post(
        addr,
        "/api/v2/records?namespace=alpha",
        KEY,
        r#"{"namespace":"beta","key":"k1","payload":"x","metadata":{}}"#,
    )
    .await;
    assert_eq!(
        s, 403,
        "query namespace must not mask a foreign body namespace"
    );
}

/// `POST /records/batch`: any record in a foreign namespace denies the whole
/// batch (all-or-nothing, no partial writes).
#[tokio::test]
async fn tenant_batch_with_any_foreign_namespace_is_denied() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let mixed = http_post(
        addr,
        "/api/v2/records/batch",
        KEY,
        r#"[{"namespace":"alpha","key":"a1","payload":"x","metadata":{}},{"namespace":"beta","key":"b1","payload":"x","metadata":{}}]"#,
    )
    .await;
    assert_eq!(
        mixed, 403,
        "a batch touching a foreign namespace must be denied whole"
    );

    let own = http_post(
        addr,
        "/api/v2/records/batch",
        KEY,
        r#"[{"namespace":"alpha","key":"a1","payload":"x","metadata":{}},{"namespace":"alpha","key":"a2","payload":"y","metadata":{}}]"#,
    )
    .await;
    assert_eq!(
        own, 201,
        "alpha batch on its own namespace must pass (got {own})"
    );
}

/// Search surface: namespace declared in the body.
#[tokio::test]
async fn tenant_search_is_namespace_scoped() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own = http_post(
        addr,
        "/api/v2/search",
        KEY,
        r#"{"namespace":"alpha","query_vector":[0.1,0.2],"filters":{},"text_query":null,"top_k":1,"distance_metric":"Cosine","explain":false}"#,
    )
    .await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant must search its own namespace (got {own})"
    );

    let cross = http_post(
        addr,
        "/api/v2/search",
        KEY,
        r#"{"namespace":"beta","query_vector":[0.1,0.2],"filters":{},"text_query":null,"top_k":1,"distance_metric":"Cosine","explain":false}"#,
    )
    .await;
    assert_eq!(cross, 403, "alpha tenant must not search beta");
}

/// List surface: own namespace passes, sibling denied, all-namespaces fan-out
/// denied (fail closed — no global read permission).
#[tokio::test]
async fn tenant_list_is_namespace_scoped_and_list_all_is_denied() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own = http_get(addr, "/api/v2/list?namespace=alpha", KEY).await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant must list its own namespace (got {own})"
    );

    let cross = http_get(addr, "/api/v2/list?namespace=beta", KEY).await;
    assert_eq!(cross, 403, "alpha tenant must not list beta");

    let all = http_get(addr, "/api/v2/list", KEY).await;
    assert_eq!(all, 403, "scoped tenant must not list all namespaces");
}

/// Delete surface (path namespace).
#[tokio::test]
async fn tenant_delete_is_namespace_scoped() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let cross = http_delete(addr, "/api/v2/records/beta/k1", KEY).await;
    assert_eq!(cross, 403, "alpha tenant must not delete beta records");

    let own = http_delete(addr, "/api/v2/records/alpha/k1", KEY).await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant may delete its own records (got {own})"
    );
}

/// Export surface: own namespace passes, sibling denied, all-namespaces
/// export denied (fail closed).
#[tokio::test]
async fn tenant_export_is_namespace_scoped_and_all_export_denied() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("out.jsonl");
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own_body =
        serde_json::json!({ "path": out, "namespace": "alpha", "filter": null }).to_string();
    let own = http_post(addr, "/api/v2/export", KEY, &own_body).await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant must export its own namespace (got {own})"
    );

    let cross_body =
        serde_json::json!({ "path": out, "namespace": "beta", "filter": null }).to_string();
    let cross = http_post(addr, "/api/v2/export", KEY, &cross_body).await;
    assert_eq!(cross, 403, "alpha tenant must not export beta");

    let all_body = serde_json::json!({ "path": out, "filter": null }).to_string();
    let all = http_post(addr, "/api/v2/export", KEY, &all_body).await;
    assert_eq!(
        all, 403,
        "scoped tenant must not export all namespaces (fail closed)"
    );
}

/// Import surface: inline records are checked per record; path imports declare
/// no namespace → fail closed for scoped tenants.
#[tokio::test]
async fn tenant_import_is_namespace_scoped_and_path_import_denied() {
    let state = two_tenant_state();
    // Seed one record per namespace directly against the shared engine so the
    // import payloads carry valid node ids (import re-validates the hash).
    let seeder = Embedded::from_engine(state.storage.clone());
    let rec_alpha = seeder
        .put(MemoryInput::new("alpha", "seed-a", "payload"))
        .expect("seed alpha");
    let rec_beta = seeder
        .put(MemoryInput::new("beta", "seed-b", "payload"))
        .expect("seed beta");
    let addr = spawn(state).await;

    let own_body = serde_json::json!({ "records": [rec_alpha] }).to_string();
    let own = http_post(addr, "/api/v2/import", KEY, &own_body).await;
    assert!(
        own != 401 && own != 403,
        "alpha tenant must import into its own namespace (got {own})"
    );

    let cross_body = serde_json::json!({ "records": [rec_beta] }).to_string();
    let cross = http_post(addr, "/api/v2/import", KEY, &cross_body).await;
    assert_eq!(cross, 403, "alpha tenant must not import into beta");

    let path_body = serde_json::json!({ "path": "whatever.jsonl" }).to_string();
    let by_path = http_post(addr, "/api/v2/import", KEY, &path_body).await;
    assert_eq!(
        by_path, 403,
        "path import declares no namespace → fail closed for scoped tenants"
    );
}

/// Fail-closed: a scoped credential whose request does not declare any
/// namespace falls back to the coarse check, which it cannot pass.
#[tokio::test]
async fn scoped_tenant_missing_namespace_is_denied_fail_closed() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let s = http_post(addr, "/api/v2/records", KEY, "{}").await;
    assert_eq!(
        s, 403,
        "body without namespace must fall back to the coarse check → deny"
    );
}

/// A body-namespace denial is audited like the path/query denials (MEMG-10
/// contract): namespace + role + action, never the token.
#[tokio::test]
async fn tenant_body_write_denial_is_audited() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("audit.jsonl");
    let state = two_tenant_state_with_audit(Some(&audit_path));
    let addr = spawn(state).await;

    let s = http_post(
        addr,
        "/api/v2/records",
        KEY,
        r#"{"namespace":"beta","key":"k1","payload":"x"}"#,
    )
    .await;
    assert_eq!(s, 403);

    let rows = audit_rows(&audit_path);
    let denied = rows
        .iter()
        .find(|r| r["op"] == "auth_rbac")
        .expect("body-namespace denial must be audited");
    assert_eq!(denied["namespace"], "beta");
    assert_eq!(denied["key"], "tenant_alpha");
    let reason = denied["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("action=write"), "reason: {reason}");

    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    assert!(!raw.contains(KEY), "the Bearer token must never be audited");
}

// ── MEMG-04 post-review P2-01: regresiones (C1/R2/R3/R4) ───────────────────

/// C1 regression: `POST /search` with a matching `?namespace=` must NOT mask
/// an **empty** body namespace — the handler treats it as "all namespaces"
/// (`search_all`), so the scoped credential could read cross-tenant. Empty
/// body declarations discard the query namespace and fall back to the coarse
/// check (fail closed).
#[tokio::test]
async fn tenant_search_query_namespace_cannot_mask_blank_body() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let s = http_post(
        addr,
        "/api/v2/search?namespace=alpha",
        KEY,
        r#"{"namespace":"","query_vector":[0.1,0.2],"filters":{},"text_query":null,"top_k":1,"distance_metric":"Cosine","explain":false}"#,
    )
    .await;
    assert_eq!(
        s, 403,
        "blank body namespace must discard the query namespace (search_all leak)"
    );
}

/// R2: global `writer`/`reader` roles hold no `Namespace*` grants — a
/// namespaced body write is denied, the same semantics SRV-05 already applies
/// to path namespaces. This locks the tightening intentionally (pre-MEMG-04
/// the body-only namespace fell through to the coarse global check).
#[tokio::test]
async fn global_writer_role_denied_on_body_namespace() {
    let state = server_state(in_memory_storage(), map_role("writer"));
    let addr = spawn(state).await;

    let s = http_post(
        addr,
        "/api/v2/records",
        KEY,
        r#"{"namespace":"team","key":"k1","payload":"x","metadata":{}}"#,
    )
    .await;
    assert_eq!(
        s, 403,
        "global writer without NamespaceWrite must be denied on a namespaced body write"
    );
}

/// R3: a namespace containing `/` is percent-encoded in the path
/// (`agent%2Fmain`) — the RBAC compare must decode the segment before matching
/// the grant.
#[tokio::test]
async fn tenant_reads_own_namespace_with_encoded_slash() {
    let state = two_tenant_state();
    let addr = spawn(state).await;

    let own = http_get(addr, "/api/v2/records/agent%2Fmain/k1", KEY).await;
    assert!(
        own != 401 && own != 403,
        "grant `agent/main` must match the encoded path segment (got {own})"
    );

    let cross = http_get(addr, "/api/v2/records/agent%2Fmain/k1", KEY_BETA).await;
    assert_eq!(cross, 403, "beta tenant must not read `agent/main`");
}

/// R4: a least-privilege scoped role with only `namespace_read` must be able
/// to SEARCH its own namespace (`POST /search` is a read operation).
#[tokio::test]
async fn read_only_scoped_role_can_search_own_namespace() {
    let mut token_role_map = HashMap::new();
    token_role_map.insert(KEY.to_string(), "tenant_reader".to_string());
    let mut roles = HashMap::new();
    roles.insert(
        "tenant_reader".to_string(),
        RbacRoleCfg {
            namespace_read: vec!["alpha".to_string()],
            namespace_write: Vec::new(),
        },
    );
    let storage = in_memory_storage();
    let db = Embedded::from_engine(storage.clone());
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(100, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: Some(Arc::from(KEY)),
        alt_api_key: None,
        jwt_secret: None,
        rbac_config: RbacConfig {
            token_role_map,
            roles,
        },
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    });
    let addr = spawn(state).await;

    let own = http_post(
        addr,
        "/api/v2/search",
        KEY,
        r#"{"namespace":"alpha","query_vector":[0.1,0.2],"filters":{},"text_query":null,"top_k":1,"distance_metric":"Cosine","explain":false}"#,
    )
    .await;
    assert!(
        own != 401 && own != 403,
        "read-only scoped role must search its own namespace (got {own})"
    );

    let cross = http_post(
        addr,
        "/api/v2/search",
        KEY,
        r#"{"namespace":"beta","query_vector":[0.1,0.2],"filters":{},"text_query":null,"top_k":1,"distance_metric":"Cosine","explain":false}"#,
    )
    .await;
    assert_eq!(cross, 403, "read-only role must not search beta");
}
