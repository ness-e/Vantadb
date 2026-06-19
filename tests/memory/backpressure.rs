use vantadb::config::VantaConfig;
use vantadb::node::{NodeTier, UnifiedNode};
use vantadb::storage::StorageEngine;

#[test]
fn test_backpressure_disabled_with_zero_threshold() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string())
        .with_rss_threshold(0.0);
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    assert!(engine.check_memory_pressure().is_ok());
}

#[test]
fn test_backpressure_allows_normal_operation() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string())
        .with_rss_threshold(0.95);
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let mut node = UnifiedNode::with_vector(1, vec![0.1, 0.2, 0.3, 0.4]);
    node.tier = NodeTier::Hot;
    let result = engine.insert(&node);
    assert!(result.is_ok(), "Insert should succeed: {:?}", result);
}
