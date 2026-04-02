use iadbms::gc::GcWorker;
use iadbms::storage::StorageEngine;
use iadbms::node::UnifiedNode;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[test]
fn test_sweep_logic() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).unwrap();
    
    // Insert mock nodes
    let node1 = UnifiedNode::new(1);
    let node2 = UnifiedNode::new(2);
    storage.insert(&node1).unwrap();
    storage.insert(&node2).unwrap();

    let mut worker = GcWorker::new(&storage);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Past Node (should be swept and deleted)
    worker.register_ttl(1, now - 10);
    // Future Node (should be preserved)
    worker.register_ttl(2, now + 100);

    let purged = worker.sweep();
    
    assert_eq!(purged, 1); 
    
    // Assert Node 1 was deleted physically
    assert!(storage.get(1).unwrap().is_none());
    // Assert Node 2 remains
    assert!(storage.get(2).unwrap().is_some());
}
