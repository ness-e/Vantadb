use tempfile::TempDir;
use vantadb::config::VantaConfig;
use vantadb::storage::StorageEngine;

#[test]
fn test_mmap_hnsw_config_respected() {
    let dir = TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string())
        .with_mmap_hnsw(true);
    let engine =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let stats = engine.get_memory_stats();
    // Engine opens and reports stats with mmap_hnsw=true
    assert!(stats.node_count == 0, "fresh engine has zero nodes");
}

#[test]
fn test_mmap_hnsw_disabled() {
    let dir = TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string())
        .with_mmap_hnsw(false);
    let engine =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let stats = engine.get_memory_stats();
    // Engine opens and reports stats with mmap_hnsw=false
    assert!(stats.node_count == 0, "fresh engine has zero nodes");
}
