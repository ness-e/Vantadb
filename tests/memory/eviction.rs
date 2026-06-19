use vantadb::config::VantaConfig;
use vantadb::node::{NodeTier, UnifiedNode};
use vantadb::storage::StorageEngine;

#[test]
fn test_evict_cold_nodes_empty_engine() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string());
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let report = engine.evict_cold_nodes(0.5).unwrap();
    assert_eq!(report.evicted, 0);
    assert_eq!(report.scanned, 0);
}

#[test]
fn test_evict_cold_nodes_with_hot_nodes() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string());
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();

    for i in 0..5 {
        let mut node = UnifiedNode::with_vector(100 + i, vec![0.1; 4]);
        node.tier = NodeTier::Hot;
        node.importance = i as f32 / 10.0;
        node.hits = (5 - i) as u32;
        engine.insert(&node).unwrap();
    }

    let report = engine.evict_cold_nodes(0.5).unwrap();
    assert!(report.scanned > 0, "Should have scanned nodes");
    assert_eq!(report.evicted, 2);
}

#[test]
fn test_evict_cold_nodes_zero_ratio() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string());
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let report = engine.evict_cold_nodes(0.0).unwrap();
    assert_eq!(report.evicted, 0);
}
