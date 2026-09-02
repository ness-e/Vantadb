// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::connection_pool::ConnectionPool;
use vantadb::storage::StorageEngine;
use vantadb_server::server::ServerState;

/// Canonical `ServerState` constructor for integration tests (MOD-15).
///
/// Mirrors production startup (`cli_server::run`, MOD-12): wraps the raw engine
/// and calls `ensure_indexes_current` so lexical/hybrid searches work on fresh
/// DBs. Variants that need custom fields construct `ServerState` literally in
/// the test instead: RBAC `token_role_map` (server.rs `build_rbac_context`),
/// breaker threshold/timeout (server.rs circuit-breaker tests) and reopening an
/// existing storage dir (e2e.rs persistence test).
pub fn build_server_state(
    path: &Path,
    api_key: Option<&str>,
    concurrency: usize,
) -> (tempfile::TempDir, Arc<ServerState>) {
    let dir = tempfile::tempdir().unwrap();
    let storage_path = dir.path().join(path);
    let storage = Arc::new(StorageEngine::open(storage_path.to_str().unwrap()).unwrap());
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    // MOD-12: mirror production startup (`cli_server::run`) which ensures
    // indexes are current after wrapping the raw engine — without this,
    // lexical/hybrid searches fail on fresh DBs ("text_index not found").
    db.ensure_indexes_current()
        .expect("ensure_indexes_current must succeed");
    let state = Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(
            concurrency,
            Duration::from_millis(5000),
        )),
        api_key: api_key.map(Arc::from),
        alt_api_key: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    (dir, state)
}
