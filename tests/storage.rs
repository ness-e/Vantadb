use iadbms::storage::StorageEngine;
use iadbms::node::UnifiedNode;
use tempfile::tempdir;

#[test]
fn test_rocksdb_integration() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).unwrap();

    let node = UnifiedNode::new(42);
    storage.insert(&node).unwrap();

    let retrieved = storage.get(42).unwrap().unwrap();
    assert_eq!(retrieved.id, 42);
}
