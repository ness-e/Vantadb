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
    build_server_state_with(path, api_key, concurrency, None)
}

/// Same as [`build_server_state`] but lets the caller set
/// `Config::export_base_dir` on the engine; `Embedded::from_engine` snapshots
/// it into `Embedded.config`, which is what the FIND-347 HTTP sandbox checks.
/// Required by tests that exercise absolute import/export paths over HTTP —
/// without a configured base dir those paths are rejected by design.
pub fn build_server_state_with(
    path: &Path,
    api_key: Option<&str>,
    concurrency: usize,
    export_base_dir: Option<std::path::PathBuf>,
) -> (tempfile::TempDir, Arc<ServerState>) {
    let dir = tempfile::tempdir().unwrap();
    let storage_path = dir.path().join(path);
    let storage = Arc::new(
        StorageEngine::open_with_config(
            storage_path.to_str().unwrap(),
            Some(vantadb::config::Config {
                export_base_dir,
                ..Default::default()
            }),
        )
        .unwrap(),
    );
    let db = vantadb::Embedded::from_engine(storage.clone());
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
        jwt_secret: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
        activity_tracker: Arc::new(vantadb_server::server::ActivityTracker::new()),
    });
    (dir, state)
}
