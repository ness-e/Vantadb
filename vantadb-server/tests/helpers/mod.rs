use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::connection_pool::ConnectionPool;
use vantadb::storage::StorageEngine;
use vantadb_server::server::ServerState;

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
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    (dir, state)
}
