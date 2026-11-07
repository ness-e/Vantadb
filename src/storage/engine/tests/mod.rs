//! Unit tests for the storage engine, organized by source module.
//!
//! Split from the original 4076-line `tests.rs` into per-feature files.

use super::*; // StorageEngine, MemoryStats, EvictionReason, EvictionReport, constants, etc.
use crate::backend::BackendKind;
use crate::config::Config;
use crate::node::UnifiedNode;

// ─── Sub-modules (one per source module) ──────────────────────

mod engine;
mod incremental;
mod init;
mod maintenance;
mod ops;
mod scalar_index;
mod stats;
mod types;

/// ADR-0057 W6: the activation of the at-rest encryption.
#[cfg(all(feature = "encryption", any(feature = "fjall", feature = "rocksdb")))]
mod encryption;

// ─── Shared helpers used by all sub-modules ───────────────────

pub(super) fn in_memory_engine() -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config))
        .expect("Failed to open in-memory engine")
}

pub(super) fn in_memory_read_only() -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: true,
        ..Config::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config))
        .expect("Failed to open read-only in-memory engine")
}

/// In-memory engine pre-wired with all four LSM tiers (L0..L3).
///
/// The plain `in_memory_engine()` helper creates a single L0 segment, which is
/// enough for most tests but not for tier promotion (L1/L2/L3 targets). This
/// mirrors the persistent layout produced by `SegmentRegistry::open_or_create`.
pub(super) fn in_memory_tiered_engine() -> StorageEngine {
    let mut engine = in_memory_engine();
    for _ in 1..4 {
        let vs = crate::storage::vfile::File::create_in_memory(64 * MIB);
        engine.vector_store.push(parking_lot::RwLock::new(vs));
    }
    engine
}

pub(super) fn sample_node(id: u128) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    node
}

// ─── Frente 4-A: Coordinators & Crash Recovery Tests ───────────

#[test]
fn test_storage_engine_coordinators_facade() {
    let engine = in_memory_engine();

    // 1. IndexCoordinator
    let index_coord = engine.index_coordinator();
    assert_eq!(index_coord.node_count(), 0);
    assert!(!index_coord.contains_node(100));

    // 2. VectorStoreService
    let vss = engine.vector_store_service();
    assert!(vss.segment_count() >= 1);
    let sample = sample_node(100);
    let (local_off, packed) = vss.write_node_l0(&sample).expect("write node l0");
    assert_eq!(local_off, 64);
    assert_eq!(crate::lsm::unpack_offset(packed), (0, 64));

    // Tombstone toggle via VectorStoreService
    assert!(vss
        .set_tombstone_at_packed_offset(packed, true)
        .expect("tombstone"));
    assert!(vss
        .set_tombstone_at_packed_offset(packed, false)
        .expect("untombstone"));
    assert!(vss.flush_all_segments().is_ok());

    // 3. WalCoordinator
    let wal_coord = engine.wal_coordinator();
    // InMemory engine has no WAL enabled
    assert!(!wal_coord.is_enabled());
    assert_eq!(wal_coord.total_records_written(), 0);
    assert!(wal_coord.append(&crate::wal::WalRecord::Begin(1)).is_ok());

    // Checkpoint helpers via WalCoordinator
    assert!(wal_coord
        .record_checkpoint_seq(&*engine.backend, 10)
        .is_ok());
    let seq = wal_coord
        .read_checkpoint_seq(&*engine.backend)
        .expect("read seq");
    assert_eq!(seq, 10);
    assert!(wal_coord.reset_checkpoint_seq(&*engine.backend).is_ok());
    let reset_seq = wal_coord
        .read_checkpoint_seq(&*engine.backend)
        .expect("read reset");
    assert_eq!(reset_seq, 0);

    // Index coordinator batch drain
    index_coord.push_pending_op(PendingHnswOp {
        id: 100,
        bitset: sample.bitset.clone(),
        vector: sample.vector.clone(),
        storage_offset: packed,
        is_delete: false,
    });
    let _guard = engine.acquire_insert_lock("test_drain").expect("lock");
    assert!(index_coord.drain_batch_locked().expect("drain"));
    drop(_guard);
    assert_eq!(index_coord.node_count(), 1);
    assert!(index_coord.contains_node(100));
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_txn_crash_physical_undo_pass_on_recovery() {
    use crate::backend::BackendPartition;
    use crate::storage::ops::NodeMetadata;
    use crate::wal::WalRecord;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();

    let existing_id = 42u128;
    let dirty_id = 99u128;
    let tid = 777u64;

    {
        let engine = StorageEngine::open(path).expect("open disk engine");
        // Step 0: Insert an existing node and flush so it is committed and durable
        engine
            .insert(&sample_node(existing_id))
            .expect("insert existing");
        engine.flush().expect("flush existing");

        // Simulate crash after Step 5 (mutations written to KV + vstore) but before Step 6 (no Commit)
        // WAL has [Begin(tid), Insert(dirty_node), Delete { id: existing_id }, Prepare]
        let dirty_node = sample_node(dirty_id);
        let wal = engine.wal.as_ref().expect("disk engine has WAL");
        wal.append(&WalRecord::Begin(tid)).expect("append begin");
        wal.append(&WalRecord::Insert(dirty_node.clone()))
            .expect("append insert");
        wal.append(&WalRecord::Delete { id: existing_id })
            .expect("append delete");
        wal.append(&WalRecord::Prepare {
            txn_id: tid,
            op_count: 2,
        })
        .expect("append prepare");
        wal.flush_all().expect("flush WAL shards");

        // Mutate KV and stores as step 5 would have done before crash:
        // 1. Write dirty_node to KV with created_by_txn = tid
        let dirty_key = dirty_id.to_le_bytes();
        let dirty_meta = NodeMetadata {
            relational: dirty_node.relational.clone(),
            edges: dirty_node.edges.clone(),
            created_by_txn: tid,
            deleted_by_txn: None,
        };
        let dirty_bytes = postcard::to_allocvec(&dirty_meta).expect("serialize dirty meta");
        engine
            .backend
            .put(BackendPartition::Default, &dirty_key, &dirty_bytes)
            .expect("put dirty KV");

        // Write dirty_node to vstore and add to HNSW
        let (_local_off, packed) = engine
            .vector_store_service()
            .write_node_l0(&dirty_node)
            .expect("write dirty node vstore");
        engine
            .index_coordinator()
            .add_node(
                dirty_id,
                dirty_node.bitset.clone(),
                dirty_node.vector.clone(),
                packed,
            )
            .expect("add dirty node to hnsw");

        // 2. Existing node stamped with deleted_by_txn = Some(tid)
        let exist_key = existing_id.to_le_bytes();
        let mut exist_meta: NodeMetadata = postcard::from_bytes(
            &engine
                .backend
                .get(BackendPartition::Default, &exist_key)
                .expect("get exist")
                .expect("exist meta bytes"),
        )
        .expect("deserialize exist meta");
        exist_meta.deleted_by_txn = Some(tid);
        let exist_bytes = postcard::to_allocvec(&exist_meta).expect("serialize exist meta");
        engine
            .backend
            .put(BackendPartition::Default, &exist_key, &exist_bytes)
            .expect("stamp exist deleted");

        // Tombstone existing node in vstore and remove from HNSW
        if let Some(packed_exist) = engine.index_coordinator().storage_offset_of(existing_id) {
            let _ = engine
                .vector_store_service()
                .set_tombstone_at_packed_offset(packed_exist, true);
        }
        engine.index_coordinator().remove_node(existing_id);

        // Flush backend KV to disk to simulate dirty state survived in KV
        engine.backend.flush().expect("flush backend");

        // Drop engine without Commit record: crash occurs here
    }

    // Reopen engine: init.rs recover_state executes the Physical Crash Recovery Undo Pass
    let engine = StorageEngine::open(path).expect("reopen disk engine");

    // Verification 1: dirty_node must be physically purged from KV
    let dirty_key = dirty_id.to_le_bytes();
    assert!(
        engine
            .backend
            .get(BackendPartition::Default, &dirty_key)
            .expect("get dirty")
            .is_none(),
        "dirty_node must be physically removed from KV backend"
    );

    // Verification 2: dirty_node must not be in HNSW
    assert!(
        !engine.index_coordinator().contains_node(dirty_id),
        "dirty_node must not be present in HNSW"
    );
    assert!(
        engine.get(dirty_id).expect("get dirty").is_none(),
        "get(dirty_id) must return None"
    );

    // Verification 3: existing_id was restored in KV with deleted_by_txn == None
    let exist_key = existing_id.to_le_bytes();
    let restored_bytes = engine
        .backend
        .get(BackendPartition::Default, &exist_key)
        .expect("get exist")
        .expect("existing node must exist in KV");
    let restored_meta: NodeMetadata =
        postcard::from_bytes(&restored_bytes).expect("deserialize restored meta");
    assert_eq!(
        restored_meta.deleted_by_txn, None,
        "existing_id must have deleted_by_txn == None after undo pass"
    );

    // Verification 4: existing_id must be readable
    let existing_node = engine.get(existing_id).expect("get existing");
    assert!(
        existing_node.is_some(),
        "existing_id must be fully readable and restored"
    );
}
