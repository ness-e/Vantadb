//! Dedicated 3-layer auth tests (D19, MEM-05).
//!
//! Pattern AAA: arrange → act → assert. Same in-memory engine setup as
//! `src/entity/tests.rs`; seeds `user` entities through `EntityStore` the way
//! a producer would, then asserts the L1 (Bearer) / L2 (service-id) / L3
//! (user-key) chain end-to-end over real HTTP plus unit tests for identity
//! resolution.

use super::*;
use crate::config::VantaConfig;
use crate::entity::EntityStore;
use crate::node::FieldValue;
use crate::sdk::VantaEmbedded;
use crate::storage::{BackendKind, StorageEngine};
use std::collections::HashMap;
use std::net::SocketAddr;

fn fields(pairs: &[(&str, &str)]) -> HashMap<String, FieldValue> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), FieldValue::String(v.to_string())))
        .collect()
}

fn in_memory_storage(audit_log_path: Option<std::path::PathBuf>) -> Arc<StorageEngine> {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        audit_log_path,
        ..Default::default()
    };
    Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine"))
}

fn seed_user(store: &EntityStore<'_>, user_id: &str, user_key: &str, user_type: &str) {
    store
        .entity_set(
            "default",
            "user",
            user_id,
            fields(&[
                ("user_key", user_key),
                ("user_type", user_type),
                ("status", "active"),
            ]),
        )
        .expect("seed user");
}

fn auth_state(
    api_key: Option<String>,
    storage: Option<Arc<StorageEngine>>,
    audit: Option<Arc<AuditLogger>>,
) -> AuthState {
    AuthState::new(
        api_key,
        None, // alt_api_key
        RbacConfig::default(),
        Arc::new(Rbac::new()),
        &[],
        storage,
        audit,
    )
}

fn get_request(path: &str, headers: &[(&str, &str)]) -> axum::extract::Request {
    let mut builder = axum::http::Request::builder()
        .method("GET")
        .uri(path)
        .header(header::AUTHORIZATION, "Bearer sk-test");
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    builder
        .body(axum::body::Body::empty())
        .expect("build request")
}

// ── Unit: resolve_user_key (L3 resolution core) ──

#[test]
fn resolve_user_key_finds_user_by_key() {
    let storage = in_memory_storage(None);
    let store = EntityStore::new(storage.as_ref());
    seed_user(&store, "usr-1", "uk-abc", "user");

    let (user_id, is_admin) = resolve_user_key(&store, AUTH_ENTITY_NS, "uk-abc")
        .expect("resolve")
        .expect("found");
    assert_eq!(user_id, "usr-1");
    assert!(!is_admin);
}

#[test]
fn resolve_user_key_detects_system_admin() {
    let storage = in_memory_storage(None);
    let store = EntityStore::new(storage.as_ref());
    seed_user(&store, "usr-9", "uk-admin", "system_admin");

    let (_, is_admin) = resolve_user_key(&store, AUTH_ENTITY_NS, "uk-admin")
        .expect("resolve")
        .expect("found");
    assert!(is_admin);
}

#[test]
fn resolve_user_key_missing_or_wrong_returns_none() {
    let storage = in_memory_storage(None);
    let store = EntityStore::new(storage.as_ref());
    seed_user(&store, "usr-1", "uk-abc", "user");

    assert!(resolve_user_key(&store, AUTH_ENTITY_NS, "uk-xyz")
        .expect("resolve")
        .is_none());
    assert!(
        resolve_user_key(&store, AUTH_ENTITY_NS, "uk-ABC")
            .expect("resolve")
            .is_none(),
        "user_key match must be case-sensitive"
    );
}

// ── Unit: resolve_identity (L2/L3 header resolution) ──

#[test]
fn resolve_identity_l3_user_key_wins() {
    let storage = in_memory_storage(None);
    let store = EntityStore::new(storage.as_ref());
    seed_user(&store, "usr-7", "uk-7", "user");
    let auth = auth_state(Some("sk-test".into()), Some(storage), None);

    let identity = resolve_identity(
        &get_request(
            "/api/v2/health",
            &[(USER_KEY_HEADER, "uk-7"), (SERVICE_ID_HEADER, "svc-1")],
        ),
        &auth,
    )
    .expect("resolve");
    assert_eq!(
        identity,
        AuthIdentity::User {
            user_id: "usr-7".into(),
            is_system_admin: false
        }
    );
}

#[test]
fn resolve_identity_l2_service_id() {
    let auth = auth_state(Some("sk-test".into()), None, None);
    let identity = resolve_identity(
        &get_request("/api/v2/health", &[(SERVICE_ID_HEADER, "svc-1")]),
        &auth,
    )
    .expect("resolve");
    assert_eq!(
        identity,
        AuthIdentity::Service {
            service_id: "svc-1".into()
        }
    );
}

#[test]
fn resolve_identity_transport_when_no_headers() {
    let auth = auth_state(Some("sk-test".into()), None, None);
    let identity = resolve_identity(&get_request("/api/v2/health", &[]), &auth).expect("resolve");
    assert_eq!(identity, AuthIdentity::Transport);
}

#[test]
fn resolve_identity_l3_unknown_key_fails_closed() {
    let storage = in_memory_storage(None);
    let store = EntityStore::new(storage.as_ref());
    seed_user(&store, "usr-1", "uk-abc", "user");
    let auth = auth_state(Some("sk-test".into()), Some(storage), None);

    let err = resolve_identity(
        &get_request("/api/v2/health", &[(USER_KEY_HEADER, "uk-nope")]),
        &auth,
    )
    .unwrap_err();
    assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    assert_eq!(err.1, "invalid_user_key");
}

#[test]
fn resolve_identity_l3_without_storage_fails_closed() {
    let auth = auth_state(Some("sk-test".into()), None, None);
    let err = resolve_identity(
        &get_request("/api/v2/health", &[(USER_KEY_HEADER, "uk-abc")]),
        &auth,
    )
    .unwrap_err();
    assert_eq!(err.0, StatusCode::UNAUTHORIZED);
}

// ── Integration: 3-layer auth over HTTP ──

/// Spawn the full app on an ephemeral port; returns its address.
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

async fn http_get(addr: SocketAddr, path: &str, headers: &[(&str, &str)]) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n");
    for (k, v) in headers {
        request.push_str(&format!("{k}: {v}\r\n"));
    }
    request.push_str("Connection: close\r\n\r\n");

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();

    let status = response
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, response)
}

fn server_state(storage: Arc<StorageEngine>, api_key: Option<&str>) -> Arc<ServerState> {
    let db = VantaEmbedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: api_key.map(Arc::from),
        alt_api_key: None,
        rbac_config: RbacConfig::default(),
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    })
}

#[tokio::test]
async fn auth_l1_accepts_valid_token() {
    let state = server_state(in_memory_storage(None), Some("sk-test"));
    let addr = spawn(state).await;
    let (status, _) = http_get(
        addr,
        "/api/v2/health",
        &[(header::AUTHORIZATION.as_str(), "Bearer sk-test")],
    )
    .await;
    assert_eq!(status, 200, "valid Bearer must reach the protected route");
}

#[tokio::test]
async fn auth_l1_rejects_missing_token() {
    let state = server_state(in_memory_storage(None), Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(addr, "/api/v2/health", &[]).await;
    assert_eq!(status, 401, "missing Bearer must be rejected: {body}");
}

#[tokio::test]
async fn auth_l1_rejects_wrong_token() {
    let state = server_state(in_memory_storage(None), Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(
        addr,
        "/api/v2/health",
        &[(header::AUTHORIZATION.as_str(), "Bearer nope")],
    )
    .await;
    assert_eq!(status, 401, "wrong Bearer must be rejected: {body}");
}

#[tokio::test]
async fn auth_l2_service_id_grants_admin() {
    let state = server_state(in_memory_storage(None), Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(
        addr,
        "/api/v2/health",
        &[
            (header::AUTHORIZATION.as_str(), "Bearer sk-test"),
            (SERVICE_ID_HEADER, "svc-1"),
        ],
    )
    .await;
    assert_eq!(status, 200, "Bearer + service-id must pass: {body}");
}

#[tokio::test]
async fn auth_l3_user_key_accepts_known_user() {
    let storage = in_memory_storage(None);
    seed_user(
        &EntityStore::new(storage.as_ref()),
        "usr-1",
        "uk-abc",
        "user",
    );
    let state = server_state(storage, Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(
        addr,
        "/api/v2/health",
        &[
            (header::AUTHORIZATION.as_str(), "Bearer sk-test"),
            (USER_KEY_HEADER, "uk-abc"),
        ],
    )
    .await;
    assert_eq!(status, 200, "Bearer + known user-key must pass: {body}");
}

#[tokio::test]
async fn auth_l3_user_key_rejects_unknown_user() {
    let storage = in_memory_storage(None);
    seed_user(
        &EntityStore::new(storage.as_ref()),
        "usr-1",
        "uk-abc",
        "user",
    );
    let state = server_state(storage, Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(
        addr,
        "/api/v2/health",
        &[
            (header::AUTHORIZATION.as_str(), "Bearer sk-test"),
            (USER_KEY_HEADER, "uk-nope"),
        ],
    )
    .await;
    assert_eq!(status, 401, "unknown user-key must be rejected: {body}");
}

#[tokio::test]
async fn auth_l3_precedes_service_id() {
    let storage = in_memory_storage(None);
    seed_user(
        &EntityStore::new(storage.as_ref()),
        "usr-1",
        "uk-abc",
        "user",
    );
    let state = server_state(storage, Some("sk-test"));
    let addr = spawn(state).await;
    let (status, body) = http_get(
        addr,
        "/api/v2/health",
        &[
            (header::AUTHORIZATION.as_str(), "Bearer sk-test"),
            (USER_KEY_HEADER, "uk-nope"),
            (SERVICE_ID_HEADER, "svc-1"),
        ],
    )
    .await;
    assert_eq!(
        status, 401,
        "user-key resolution must win over service-id (fail closed): {body}"
    );
}

#[tokio::test]
async fn auth_events_recorded_in_audit_log() {
    let audit_path =
        std::env::temp_dir().join(format!("vanta-auth-test-{}.jsonl", std::process::id()));
    let _ = std::fs::remove_file(&audit_path);

    let storage = in_memory_storage(Some(audit_path.clone()));
    seed_user(
        &EntityStore::new(storage.as_ref()),
        "usr-1",
        "uk-abc",
        "user",
    );
    let state = server_state(storage, Some("sk-test"));
    let addr = spawn(state).await;

    // 1) L1 failure → auth_l1 err invalid_token
    let (status, _) = http_get(
        addr,
        "/api/v2/health",
        &[(header::AUTHORIZATION.as_str(), "Bearer nope")],
    )
    .await;
    assert_eq!(status, 401);

    // 2) L3 success → auth_l3 ok with user id
    let (status, _) = http_get(
        addr,
        "/api/v2/health",
        &[
            (header::AUTHORIZATION.as_str(), "Bearer sk-test"),
            (USER_KEY_HEADER, "uk-abc"),
        ],
    )
    .await;
    assert_eq!(status, 200);

    let log = std::fs::read_to_string(&audit_path).expect("audit log written");
    let _ = std::fs::remove_file(&audit_path);
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 auth events, got: {log}");
    assert!(
        log.contains(r#""op":"auth_l1""#) && log.contains(r#""outcome":"err""#),
        "auth_l1 failure event expected: {log}"
    );
    assert!(
        log.contains(r#""op":"auth_l3""#)
            && log.contains(r#""outcome":"ok""#)
            && log.contains("usr-1"),
        "auth_l3 success event expected: {log}"
    );
    assert!(
        !log.contains("sk-test") && !log.contains("uk-abc"),
        "secrets must never appear in the audit log: {log}"
    );
}
