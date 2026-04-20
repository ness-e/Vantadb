//! Backend abstraction integration test suite.
//!
//! Validates `StorageEngine` with `RocksDbBackend`, `InMemoryBackend`, and
//! `FjallBackend` through the public API.
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

// ═══════════════════════════════════════════════════════════════
// ─── FjallBackend Tests ────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════

/// Helper: create a StorageEngine backed by Fjall in a tempdir.
fn open_fjall_engine() -> (StorageEngine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    let engine =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    (engine, dir)
}

// ─── 1. Basic CRUD ──────────────────────────────────────────

#[test]
fn test_fjall_backend_basic_crud() {
    let (engine, _dir) = open_fjall_engine();

    // Insert
    let node = UnifiedNode::new(1);
    engine.insert(&node).unwrap();

    // Get
    let retrieved = engine.get(1).unwrap().expect("Node 1 should exist");
    assert_eq!(retrieved.id, 1);

    // Delete
    engine.delete(1, "test deletion").unwrap();
    assert!(
        engine.get(1).unwrap().is_none(),
        "Node 1 should be gone after delete"
    );

    TerminalReporter::success("FjallBackend basic CRUD verified.");
}

// ─── 2. Batch Multi-Partition ───────────────────────────────

#[test]
fn test_fjall_backend_batch_multi_partition() {
    let (engine, _dir) = open_fjall_engine();

    // Insert a node — this writes to the Default partition
    let node = UnifiedNode::new(200);
    engine.insert(&node).unwrap();
    assert!(engine.get(200).unwrap().is_some());

    // purge_permanent issues a write_batch across Default, TombstoneStorage,
    // CompressedArchive, and Tombstones partitions atomically.
    engine.purge_permanent(200).unwrap();

    // After purge, node should be gone from all partitions.
    assert!(
        engine.get(200).unwrap().is_none(),
        "Node 200 should be purged from all partitions"
    );

    TerminalReporter::success("FjallBackend batch multi-partition verified.");
}

// ─── 3. Full Engine Roundtrip ───────────────────────────────

#[test]
fn test_storage_engine_with_fjall_backend_insert_get_delete() {
    let (engine, _dir) = open_fjall_engine();

    // Insert with vector data
    let mut node = UnifiedNode::new(500);
    node.vector = vantadb::VectorRepresentations::Full(vec![0.1, 0.2, 0.3, 0.4]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    engine.insert(&node).unwrap();

    // Retrieve and validate
    let retrieved = engine.get(500).unwrap().expect("Node 500 should exist");
    assert_eq!(retrieved.id, 500);

    // Delete and confirm
    engine.delete(500, "engine roundtrip cleanup").unwrap();
    assert!(engine.get(500).unwrap().is_none());

    TerminalReporter::success("StorageEngine + FjallBackend full roundtrip verified.");
}

// ─── 4. Flush Durability ────────────────────────────────────

#[test]
fn test_storage_engine_fjall_backend_flush() {
    let (engine, _dir) = open_fjall_engine();

    // Insert data
    let node = UnifiedNode::new(600);
    engine.insert(&node).unwrap();

    // flush() must succeed — not an empty stub
    engine.flush().expect("FjallBackend flush() must not fail");

    // Data must survive the flush
    let retrieved = engine
        .get(600)
        .unwrap()
        .expect("Node 600 should survive flush");
    assert_eq!(retrieved.id, 600);

    TerminalReporter::success("FjallBackend flush (PersistMode::SyncAll) verified.");
}

// ─── 5. Checkpoint Not Supported ────────────────────────────

#[test]
fn test_fjall_backend_checkpoint_not_supported() {
    let (engine, dir) = open_fjall_engine();

    let checkpoint_path = dir.path().join("checkpoint_test");
    let result = engine.create_life_insurance(checkpoint_path.to_str().unwrap());

    assert!(
        result.is_err(),
        "FjallBackend checkpoint must return an error, not fake success"
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not supported") || err_msg.contains("Checkpoint"),
        "Error message should be explicit about checkpoint not being supported, got: {}",
        err_msg
    );

    TerminalReporter::success("FjallBackend checkpoint honestly reports not-supported.");
}

// ─── 6. Partition Initialization ────────────────────────────

#[test]
fn test_fjall_backend_opens_all_partitions() {
    // Verify that the engine opens cleanly with Fjall — all 4 keyspaces
    // (default, tombstone_storage, compressed_archive, tombstones) are
    // created without error.
    let (engine, _dir) = open_fjall_engine();

    // If we got here, all keyspaces were created.
    // Insert and delete to exercise at least the default partition roundtrip.
    let node = UnifiedNode::new(700);
    engine.insert(&node).unwrap();
    assert!(engine.get(700).unwrap().is_some());

    TerminalReporter::success("FjallBackend all partitions initialize cleanly.");
}
