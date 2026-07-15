use std::path::Path;
use std::sync::Arc;
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
    let state = Arc::new(ServerState {
        storage,
        semaphore: Arc::new(tokio::sync::Semaphore::new(concurrency)),
        api_key: api_key.map(Arc::from),
        rbac_config: Default::default(),
    });
    (dir, state)
}
