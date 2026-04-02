use iadbms::gc::GcWorker;
use iadbms::storage::StorageEngine;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_sweep_logic() {
    let storage = StorageEngine::new();
    let mut worker = GcWorker::new(&storage);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Past Node (should be sweeped)
    worker.register_ttl(1, now - 10);
    // Future Node (should be preserved)
    worker.register_ttl(2, now + 100);

    let purged = worker.sweep();
    
    assert_eq!(purged, 1); // Only Node 1 cleared
}
