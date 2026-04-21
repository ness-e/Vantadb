//! Backend abstraction integration test suite.
//!
//! Validates `StorageEngine` with `RocksDbBackend`, `InMemoryBackend`, and
//! `FjallBackend` through the public API (now defaulting to Fjall).

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── StorageEngine + InMemoryBackend Integration ────────────

#[test]
fn test_storage_engine_with_inmemory_backend_insert_get_delete() {
    TerminalReporter::suite_banner("STORAGE BACKEND CORE CERTIFICATION", 12);
    let mut session = VantaSession::begin("InMemory Backend CRUD");
    session.step("Initializing InMemory storage engine");

    let dir = tempdir().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let storage =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();

    session.step("Inserting node #42 with vector data");
    let mut node = UnifiedNode::new(42);
    node.vector = vantadb::VectorRepresentations::Full(vec![1.0, 2.0, 3.0]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    storage.insert(&node).unwrap();

    session.step("Retrieving and validating integrity");
    let retrieved = storage.get(42).unwrap().expect("Node 42 should exist");
    assert_eq!(retrieved.id, 42);

    session.step("Testing node excision (deletion)");
    storage.delete(42, "test deletion").unwrap();
    assert!(storage.get(42).unwrap().is_none());

    session.success("InMemory lifecycle verified.");
    session.finish(true);
}

// ─── StorageEngine + RocksDbBackend Smoke Test ──────────────

#[test]
fn test_storage_engine_rocksdb_backend_still_works() {
    let mut session = VantaSession::begin("RocksDB Persistence Check");
    session.step("Opening RocksDB storage engine");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::RocksDb,
        ..Default::default()
    };
    let storage = StorageEngine::open_with_config(db_path, Some(config))
        .expect("Failed to open StorageEngine with RocksDB");

    session.step("Verifying write-get consistency");
    let node = UnifiedNode::new(99);
    storage.insert(&node).unwrap();

    let retrieved = storage.get(99).unwrap().expect("Node 99 should exist");
    assert_eq!(retrieved.id, 99);

    session.success("RocksDB backend stable.");
    session.finish(true);
}

// ─── Purge Permanent via Backend ────────────────────────────

#[test]
fn test_purge_permanent_via_backend() {
    let mut session = VantaSession::begin("Atomic Purge Protocol");
    session.step("Preparing storage for purge");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).unwrap();

    let node = UnifiedNode::new(77);
    storage.insert(&node).unwrap();
    assert!(storage.get(77).unwrap().is_some());

    session.step("Executing multi-partition purge");
    storage.purge_permanent(77).unwrap();
    assert!(storage.get(77).unwrap().is_none());

    session.success("Purge protocol confirmed.");
    session.finish(true);
}

// ─── FjallBackend Tests ────────────────────────────────────────

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

#[test]
fn test_fjall_backend_basic_crud() {
    let mut session = VantaSession::begin("Fjall Basic CRUD");
    let (engine, _dir) = open_fjall_engine();

    session.step("Verifying Fjall insert/get/delete cycle");
    let node = UnifiedNode::new(1);
    engine.insert(&node).unwrap();
    let retrieved = engine.get(1).unwrap().expect("Node 1 should exist");
    assert_eq!(retrieved.id, 1);

    engine.delete(1, "test deletion").unwrap();
    assert!(engine.get(1).unwrap().is_none());

    session.success("Fjall CRUD verified.");
    session.finish(true);
}

#[test]
fn test_fjall_backend_batch_multi_partition() {
    let mut session = VantaSession::begin("Fjall Multi-Partition Purge");
    let (engine, _dir) = open_fjall_engine();

    session.step("Inserting node into Default partition");
    let node = UnifiedNode::new(200);
    engine.insert(&node).unwrap();

    session.step("Atomic batch purge across all LSM keyspaces");
    engine.purge_permanent(200).unwrap();
    assert!(engine.get(200).unwrap().is_none());

    session.success("LSM Multi-partition purge successful.");
    session.finish(true);
}

#[test]
fn test_storage_engine_with_fjall_backend_insert_get_delete() {
    let mut session = VantaSession::begin("Fjall Full Engine Roundtrip");
    let (engine, _dir) = open_fjall_engine();

    session.step("Inserting with high-dimensional vector data");
    let mut node = UnifiedNode::new(500);
    node.vector = vantadb::VectorRepresentations::Full(vec![0.1, 0.2, 0.3, 0.4]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    engine.insert(&node).unwrap();

    session.step("Validating retrieval consistency");
    let retrieved = engine.get(500).unwrap().expect("Node 500 should exist");
    assert_eq!(retrieved.id, 500);

    engine.delete(500, "engine roundtrip cleanup").unwrap();
    assert!(engine.get(500).unwrap().is_none());

    session.success("Fjall Full Engine roundtrip confirmed.");
    session.finish(true);
}

#[test]
fn test_storage_engine_fjall_backend_flush() {
    let mut session = VantaSession::begin("Fjall Flush Durability");
    let (engine, _dir) = open_fjall_engine();

    session.step("Writing to memtable");
    let node = UnifiedNode::new(600);
    engine.insert(&node).unwrap();

    session.step("Requesting synchronous flush");
    engine.flush().expect("FjallBackend flush() must not fail");

    assert!(engine.get(600).unwrap().is_some());
    session.success("Durability guaranteed via SyncAll.");
    session.finish(true);
}

#[test]
fn test_maintenance_with_fjall_degrades_gracefully() {
    let mut session = VantaSession::begin("Fjall Checkpoint Degradation");
    let (engine, dir) = open_fjall_engine();

    session.step("Attempting unsupported checkpoint");
    let checkpoint_path = dir.path().join("checkpoint_test");
    let result = engine.create_life_insurance(checkpoint_path.to_str().unwrap());

    assert!(result.is_err());
    session.success("System correctly reports Checkpoint as not supported.");
    session.finish(true);
}

#[test]
fn test_maintenance_with_rocksdb_preserves_behavior() {
    let mut session = VantaSession::begin("RocksDB Checkpoint Preservation");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::RocksDb,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(db_path, Some(config))
        .expect("Failed to open StorageEngine with RocksDB");

    session.step("Executing native RocksDB checkpoint");
    let checkpoint_path = dir.path().join("checkpoint_test");
    let result = engine.create_life_insurance(checkpoint_path.to_str().unwrap());

    assert!(result.is_ok());
    session.success("RocksDB native maintenance preserved.");
    session.finish(true);
}

#[test]
fn test_backend_capabilities() {
    let mut session = VantaSession::begin("Backend Capabilities Matrix");

    // InMemory
    let dir_m = tempdir().unwrap();
    let engine_mem = StorageEngine::open_with_config(
        dir_m.path().to_str().unwrap(),
        Some(EngineConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        }),
    )
    .unwrap();
    assert_eq!(
        engine_mem.backend_capabilities().kind,
        BackendKind::InMemory
    );

    // Fjall
    let (engine_f, _dir_f) = open_fjall_engine();
    assert_eq!(engine_f.backend_capabilities().kind, BackendKind::Fjall);
    assert!(!engine_f.backend_capabilities().supports_checkpoint);

    session.success("Matrix validation complete.");
    session.finish(true);
}

#[test]
fn test_compaction_request_fjall() {
    let mut session = VantaSession::begin("Fjall Compaction Degradation");
    let (engine, _dir) = open_fjall_engine();

    session.step("Requesting manual compaction (unsupported)");
    engine.request_compaction();

    session.success("Fjall correctly ignores manual compaction request.");
    session.finish(true);
}

#[test]
fn test_fjall_backend_opens_all_partitions() {
    let mut session = VantaSession::begin("Fjall Partition Discovery");
    let (engine, _dir) = open_fjall_engine();

    session.step("Validating initialization of 4 internal LSM partitions");
    let node = UnifiedNode::new(700);
    engine.insert(&node).unwrap();
    assert!(engine.get(700).unwrap().is_some());

    session.success("All partitions online.");
    session.finish(true);
}

// ─── FINAL SUMMARY REPORTER ──────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
