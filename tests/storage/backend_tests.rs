//! Backend abstraction integration test suite.
//!
//! Validates `StorageEngine` with both `RocksDbBackend` and `InMemoryBackend`
//! through the public API.
//!
//! Direct `StorageBackend` trait tests live inside the crate as unit tests
//! (see `src/backends/in_memory.rs`) because the trait is `pub(crate)`.

#[path = "../common/mod.rs"]
mod common;

use common::TerminalReporter;
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── StorageEngine + InMemoryBackend Integration ────────────

#[test]
fn test_storage_engine_with_inmemory_backend_insert_get_delete() {
    let dir = tempdir().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let storage =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();

    // Insert
    let mut node = UnifiedNode::new(42);
    node.vector = vantadb::VectorRepresentations::Full(vec![1.0, 2.0, 3.0]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    storage.insert(&node).unwrap();

    // Get
    let retrieved = storage.get(42).unwrap().expect("Node 42 should exist");
    assert_eq!(retrieved.id, 42);

    // Delete
    storage.delete(42, "test deletion").unwrap();
    assert!(storage.get(42).unwrap().is_none());

    TerminalReporter::success("StorageEngine + InMemoryBackend roundtrip verified.");
}

// ─── StorageEngine + RocksDbBackend Smoke Test ──────────────

#[test]
fn test_storage_engine_rocksdb_backend_still_works() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine with RocksDB");

    let node = UnifiedNode::new(99);
    storage.insert(&node).unwrap();

    let retrieved = storage.get(99).unwrap().expect("Node 99 should exist");
    assert_eq!(retrieved.id, 99);

    TerminalReporter::success("StorageEngine + RocksDbBackend smoke test passed.");
}

// ─── Purge Permanent via Backend ────────────────────────────

#[test]
fn test_purge_permanent_via_backend() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(db_path).unwrap();

    // Insert a node
    let node = UnifiedNode::new(77);
    storage.insert(&node).unwrap();

    // Verify it exists
    assert!(storage.get(77).unwrap().is_some());

    // Purge should delete from all partitions without error
    storage.purge_permanent(77).unwrap();

    TerminalReporter::success("purge_permanent via backend abstraction verified.");
}
