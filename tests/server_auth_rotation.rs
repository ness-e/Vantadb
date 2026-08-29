//! SRV-04 — Zero-downtime API key rotation (integration test).
//!
//! Validates the contract that two `Bearer` tokens (the primary and the
//! alternative API key) are accepted simultaneously while both are configured,
//! and that after promoting `alt_api_key` to `api_key` the old token is
//! rejected. Pattern from Qdrant v1.17 `alt_api_key`.
//!
//! AAA: arrange an in-memory `ServerState` with the relevant `api_key` /
//! `alt_api_key` combination, act by issuing an HTTP GET against
//! `/api/v2/health` with a Bearer header, assert the status code matches
//! expectation.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::cli_server::{app, ServerState};
use vantadb::config::{RbacConfig, VantaConfig};
use vantadb::connection_pool::ConnectionPool;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::{BackendKind, StorageEngine};

const OLD_KEY: &str = "sk-old-primary-aaaaaaaa";
const NEW_KEY: &str = "sk-new-primary-bbbbbbbb";

fn in_memory_storage() -> Arc<StorageEngine> {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    Arc::new(StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine"))
}

fn server_state(
    storage: Arc<StorageEngine>,
    api_key: Option<&str>,
    alt_api_key: Option<&str>,
) -> Arc<ServerState> {
    let db = VantaEmbedded::from_engine(storage.clone());
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(100, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
        api_key: api_key.map(Arc::from),
        alt_api_key: alt_api_key.map(Arc::from),
        rbac_config: RbacConfig::default(),
        trusted_proxies: Vec::new(),
        conversation_trigger: None,
    })
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

#[tokio::test]
async fn rotation_old_and_new_active_simultaneously() {
    // Rotation window: both keys configured → both Bearers must reach the
    // protected route. This is the heart of zero-downtime key rotation.
    let state = server_state(in_memory_storage(), Some(OLD_KEY), Some(NEW_KEY));
    let addr = spawn(state).await;

    let s_old = http_get(addr, "/api/v2/health", OLD_KEY).await;
    let s_new = http_get(addr, "/api/v2/health", NEW_KEY).await;

    assert_eq!(
        s_old, 200,
        "primary (old) key must still pass while alt is set"
    );
    assert_eq!(
        s_new, 200,
        "alt (new) key must pass during the rotation window"
    );
}

#[tokio::test]
async fn rotation_promote_alt_to_primary_revokes_old() {
    // After rotation: alt_api_key is removed, only api_key (new value) configured
    // → old token must be rejected with 401, new token must pass.
    let state = server_state(in_memory_storage(), Some(NEW_KEY), None);
    let addr = spawn(state).await;

    let s_old = http_get(addr, "/api/v2/health", OLD_KEY).await;
    let s_new = http_get(addr, "/api/v2/health", NEW_KEY).await;

    assert_eq!(
        s_old, 401,
        "old key must be rejected after rotation completes"
    );
    assert_eq!(
        s_new, 200,
        "new (now-primary) key must pass after promotion"
    );
}
