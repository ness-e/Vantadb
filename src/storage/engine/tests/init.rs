//! INIT module tests: engine open/close paths, config variants, error handling.

use super::super::*;
use super::{in_memory_engine, sample_node};
use crate::backend::BackendKind;
use crate::config::VantaConfig;

// ─── open_with_config paths ────────────────────────────────────

#[test]
fn test_open_path_traversal_rejected() {
    let result = StorageEngine::open_with_config(
        "../etc/passwd",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    );
    let err = result.err().expect("path traversal should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("traversal") || msg.contains("Validation"),
        "expected traversal error, got: {msg}"
    );
}

#[test]
fn test_open_in_memory_empty_path() {
    let engine = StorageEngine::open_with_config(
        "",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("empty path with InMemory should work");
    assert!(!engine.read_only);
}

#[test]
fn test_open_with_config_custom_memory_limit() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        memory_limit: Some(2 * 1024 * 1024),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config.clone()))
        .expect("open with custom memory limit");
    assert_eq!(engine.config.memory_limit, Some(2 * 1024 * 1024));
}

#[test]
fn test_open_with_config_read_only_in_memory() {
    let engine = StorageEngine::open_with_config(
        ":memory:",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..VantaConfig::default()
        }),
    )
    .expect("read-only in-memory");
    assert!(engine.read_only);
}

#[test]
fn test_open_with_config_force_mmap() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        force_mmap: true,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config))
        .expect("force_mmap should not break in-memory open");
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
}

// ─── open() backward-compatible path ─────────────────────────

#[test]
fn test_open_default_in_memory() {
    let engine = StorageEngine::open_with_config(
        ":memory:",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("open() convenience");
    assert!(!engine.read_only);
}

// ─── Error paths ────────────────────────────────────────────────

#[cfg(feature = "fjall")]
#[test]
fn test_open_read_only_nonexistent_path() {
    let result = StorageEngine::open_with_config(
        "/nonexistent_vantadb_ro_test",
        Some(VantaConfig {
            read_only: true,
            ..VantaConfig::default()
        }),
    );
    let err = result
        .err()
        .expect("read-only on nonexistent path should error");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("not found") || msg.contains("database_path") || msg.contains("not exist"),
        "expected not-found error, got: {msg}"
    );
}

#[test]
fn test_open_read_only_without_lock_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = StorageEngine::open_with_config(
        dir.path().to_str().unwrap(),
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..VantaConfig::default()
        }),
    )
    .expect("InMemory read-only with minimal path should succeed");
    assert!(engine.read_only);
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
}

#[cfg(feature = "fjall")]
#[test]
fn test_open_read_only_without_lock_file_fjall() {
    let dir = tempfile::tempdir().expect("tempdir");
    let result = StorageEngine::open_with_config(
        dir.path().to_str().unwrap(),
        Some(VantaConfig {
            read_only: true,
            ..VantaConfig::default()
        }),
    );
    let err = result
        .err()
        .expect("read-only Fjall without lock should error");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("lock") || msg.contains("not found"),
        "expected lock error, got: {msg}"
    );
}

#[test]
fn test_open_then_reopen_read_only() {
    use tempfile::tempdir;
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap().to_string();

    let config_rw = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(&path, Some(config_rw)).expect("open writable");
    drop(engine);

    let config_ro = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: true,
        ..VantaConfig::default()
    };
    let engine_ro =
        StorageEngine::open_with_config(&path, Some(config_ro)).expect("reopen read-only");
    assert!(engine_ro.read_only);
    let result = engine_ro.insert(&sample_node(1));
    assert!(result.is_err(), "read-only engine should reject writes");
}

#[test]
#[cfg(not(feature = "rocksdb"))]
fn test_open_rocksdb_without_feature() {
    let dir = tempfile::tempdir().expect("tempdir");
    let result = StorageEngine::open_with_config(
        dir.path().to_str().unwrap(),
        Some(VantaConfig {
            backend_kind: BackendKind::RocksDb,
            ..VantaConfig::default()
        }),
    );
    let err = result.err().expect("RocksDb without feature should error");
    let msg = err.to_string();
    assert!(
        msg.contains("RocksDB") || msg.contains("Validation") || msg.contains("feature"),
        "error should mention RocksDB/feature, got: {msg}"
    );
}

#[test]
fn test_open_with_empty_backend_kind_in_memory() {
    let engine = StorageEngine::open_with_config(
        "",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("InMemory with empty path");
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
}

#[test]
fn test_open_with_none_config() {
    let engine = StorageEngine::open_with_config(
        ":memory:",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("open with explicit config");
    assert!(!engine.read_only);
}

#[test]
fn test_open_with_none_config_in_memory() {
    let engine = StorageEngine::open_with_config(
        ":memory:",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("open with None config");
    assert!(!engine.read_only);
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_init_indexes_mmap_fresh_start() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = VantaConfig {
        force_mmap: true,
        mmap_hnsw: true,
        memory_limit: Some(2 * 1024 * 1024 * 1024),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
        .expect("open with force_mmap and fresh directory");
    assert!(!engine.read_only);
    engine
        .insert(&sample_node(1))
        .expect("insert after mmap init");
    let retrieved = engine.get(1).expect("get").unwrap();
    assert_eq!(retrieved.id, 1);
}

#[test]
fn test_open_lock_file_io_error() {
    let bad_path = std::path::Path::new("/nonexistent_vantadb_lock_test_xyz");
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(bad_path.to_str().unwrap(), Some(config))
        .expect("InMemory should not care about lock path");
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
}

#[test]
fn test_open_with_read_only_wal_disabled() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: true,
        wal_shards: 0,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config))
        .expect("read-only with WAL disabled");
    assert!(engine.read_only);
    assert!(engine.wal.is_none());
}

#[test]
fn test_open_in_memory_with_relative_path() {
    let engine = StorageEngine::open_with_config(
        "test_in_memory_dir",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("InMemory with relative path");
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
    engine.insert(&sample_node(1)).expect("insert");
}

#[test]
fn test_open_in_memory_with_empty_string() {
    let engine = StorageEngine::open_with_config(
        "",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("InMemory with empty string");
    engine.insert(&sample_node(1)).expect("insert");
}

#[test]
fn test_init_storage_read_only_missing_data_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("empty_subdir");
    std::fs::create_dir_all(&path).expect("create subdir");

    let engine = StorageEngine::open_with_config(
        path.to_str().unwrap(),
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..VantaConfig::default()
        }),
    )
    .expect("InMemory read-only with explicit path");
    assert!(engine.read_only);
}

#[test]
fn test_open_in_memory_with_name() {
    let engine = StorageEngine::open_with_config(
        "named_in_memory_db",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("open named in-memory");
    engine.insert(&sample_node(1)).expect("insert");
    assert_eq!(engine.get(1).expect("get").unwrap().id, 1);
}

#[test]
fn test_in_memory_backend_capabilities_in_memory() {
    let engine = in_memory_engine();
    let caps = engine.backend_capabilities();
    assert_eq!(caps.kind, BackendKind::InMemory);
    assert!(!caps.supports_checkpoint);
    assert!(!caps.supports_manual_compaction);
}

// ─── Insert + flush + reopen (persistence) ────────────────────

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_insert_flush_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    {
        let engine = StorageEngine::open(path).expect("open");
        engine.insert(&sample_node(1)).expect("insert");
        engine.flush().expect("flush");
    }
    {
        let engine = StorageEngine::open(path).expect("reopen");
        let node = engine.get(1).expect("get");
        assert!(node.is_some(), "node should persist after reopen");
    }
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_delete_and_flush() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    {
        let engine = StorageEngine::open(path).expect("open");
        engine.insert(&sample_node(1)).expect("insert");
        engine.insert(&sample_node(2)).expect("insert");
        engine.delete(1, "test").expect("delete");
        engine.flush().expect("flush");
    }
    {
        let engine = StorageEngine::open(path).expect("reopen");
        assert!(engine.get(1).unwrap().is_none());
        assert!(engine.get(2).unwrap().is_some());
    }
}

// ─── Struct field access tests (engine introspection) ────────

#[test]
fn test_engine_drop_no_panic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = StorageEngine::open_with_config(
        dir.path().to_str().unwrap(),
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("open engine for drop test");
    engine.insert(&sample_node(42)).expect("insert");
    drop(engine);
    let engine2 = StorageEngine::open_with_config(
        dir.path().to_str().unwrap(),
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        }),
    )
    .expect("reopen after drop");
    assert_eq!(engine2.backend_kind(), BackendKind::InMemory);
}

#[test]
fn test_emergency_maintenance_trigger_field() {
    let engine = in_memory_engine();
    assert!(
        !engine
            .emergency_maintenance_trigger
            .load(std::sync::atomic::Ordering::Relaxed),
        "should start false"
    );
    engine
        .emergency_maintenance_trigger
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(
        engine
            .emergency_maintenance_trigger
            .load(std::sync::atomic::Ordering::Relaxed),
        "should reflect stored value"
    );
}

#[test]
fn test_data_dir_field_in_memory() {
    let engine = in_memory_engine();
    let dir = &engine.data_dir;
    let _ = dir.as_os_str().len();
}

#[test]
fn test_edge_and_scalar_index_fields() {
    let engine = in_memory_engine();
    assert!(engine.edge_index.is_some(), "edge_index should exist");
    assert!(engine.scalar_index.is_some(), "scalar_index should exist");
}

// ─── MOD-04: scalar index rebuild on reopen ───────────────────
//
// The scalar index is maintained incrementally on writes, but a reopen
// (recover_state → replay_write_node) never touches it — init must rebuild it
// from backend metadata. Without this, TTL purge (which selects candidates via
// `scalar_lookup_int_le`) would miss every pre-existing expired record.

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_scalar_index_rebuilt_on_reopen() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();

    let mut node = sample_node(1);
    node.set_field(
        "__vanta_expires_at_ms",
        crate::node::FieldValue::Int(1_700_000_000_000),
    );
    {
        let engine = open_disk_engine_with_wal(path);
        engine.insert(&node).expect("insert node");
        engine.flush().expect("flush");
    }

    let engine = open_disk_engine_with_wal(path);
    let ids = engine.scalar_lookup_int_le("__vanta_expires_at_ms", 1_800_000_000_000);
    assert_eq!(ids, vec![1], "scalar index must find node after reopen");
    assert!(
        engine
            .scalar_lookup_int_le("__vanta_expires_at_ms", 1_600_000_000_000)
            .is_empty(),
        "expiry above the lookup bound must not match"
    );
}

#[test]
fn test_memory_governor_field() {
    let engine = in_memory_engine();
    assert!(
        engine.memory_governor.is_some(),
        "memory_governor should start Some"
    );
}

// ─── MOD-02 (H-2): crash-atomic transactions ───────────────────
//
// A crash mid-`batch_append` leaves `[Begin, ops...]` durable across some
// shards while `Commit` never made it to disk. Recovery must discard the
// incomplete txn instead of applying it as if committed — without dropping
// unrelated durable records appended after it.

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn open_disk_engine_with_wal(path: &str) -> StorageEngine {
    // Default config → persistent backend + 4-shard WAL; recovery (recover_state)
    // only runs on the disk-backed path, never the InMemory branch.
    StorageEngine::open(path).expect("open disk-backed engine with WAL")
}

/// Simulate a kill-between-shards: write txn records through the engine's OWN
/// WAL handle (the real data-dir WAL), sync, then drop the engine scope — the
/// "crash". The txn's `Commit` is simply never appended, modeling a batch whose
/// tail never became durable across shards.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn simulate_crash_after(engine: &StorageEngine, records: &[crate::wal::WalRecord]) {
    let wal = engine.wal.as_ref().expect("engine opened with a WAL");
    for r in records {
        wal.append(r).expect("append crash-sim record");
    }
    wal.flush_all().expect("sync shards");
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_crash_partial_txn_without_commit_not_recovered() {
    use crate::wal::WalRecord;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();

    {
        let engine = open_disk_engine_with_wal(path);
        engine.insert(&sample_node(7)).expect("insert committed");
        engine.flush().expect("flush pre-crash data");

        // Crash mid-commit: Begin + two ops hit the shards, Commit never does.
        simulate_crash_after(
            &engine,
            &[
                WalRecord::Begin(100),
                WalRecord::Insert(sample_node(8)),
                WalRecord::Insert(sample_node(9)),
            ],
        );
    }

    let engine = open_disk_engine_with_wal(path);
    assert!(
        engine.get(8).expect("get").is_none(),
        "partial-txn insert must NOT survive recovery (crash atomicity)"
    );
    assert!(
        engine.get(9).expect("get").is_none(),
        "second partial-txn insert must NOT survive recovery"
    );
    assert!(
        engine.get(7).expect("get").is_some(),
        "pre-crash committed node must survive"
    );
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_crash_full_txn_with_commit_recovered() {
    use crate::wal::WalRecord;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    {
        let engine = open_disk_engine_with_wal(path);

        // Complete batch: Begin + ops + Commit all durable.
        simulate_crash_after(
            &engine,
            &[
                WalRecord::Begin(200),
                WalRecord::Insert(sample_node(10)),
                WalRecord::Insert(sample_node(11)),
                WalRecord::Commit(200),
            ],
        );
    }

    let engine = open_disk_engine_with_wal(path);
    assert!(
        engine.get(10).expect("get").is_some(),
        "committed txn insert must be recovered"
    );
    assert!(
        engine.get(11).expect("get").is_some(),
        "committed txn insert must be recovered"
    );
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_records_after_partial_txn_survive_recovery() {
    use crate::wal::WalRecord;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    {
        let engine = open_disk_engine_with_wal(path);

        // Partial txn [Begin(300), op, op] whose Commit never became durable,
        // followed by a COMPLETE later batch from an independent writer
        // [Begin(301), op, Commit(301)] (the batch's slots are contiguous, so
        // the later Begin bounds the incomplete txn's extent). Recovery must
        // discard only the incomplete txn's records — the committed later batch
        // survives.
        simulate_crash_after(
            &engine,
            &[
                WalRecord::Begin(300),
                WalRecord::Insert(sample_node(12)),
                WalRecord::Insert(sample_node(13)),
                WalRecord::Begin(301),
                WalRecord::Insert(sample_node(14)),
                WalRecord::Commit(301),
            ],
        );
    }

    let engine = open_disk_engine_with_wal(path);
    assert!(
        engine.get(12).expect("get").is_none(),
        "uncommitted txn op must be discarded"
    );
    assert!(
        engine.get(13).expect("get").is_none(),
        "uncommitted txn op must be discarded"
    );
    assert!(
        engine.get(14).expect("get").is_some(),
        "durable committed record after an incomplete txn must survive"
    );
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_persistence_binary_vector_roundtrip_vstore() {
    // ADR-032: Binary vector must survive flush + reopen via vstore, not just HNSW file.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    let bin_data: Box<[u64]> =
        vec![0xDEADBEEFu64, 0xCAFEu64, 0x1234567890ABCDEFu64].into_boxed_slice();
    {
        let engine = open_disk_engine_with_wal(path);
        let mut node = sample_node(9001);
        node.vector = crate::node::VectorRepresentations::Binary(bin_data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert binary");
        engine.flush().expect("flush");
        // immediate get before reopen should already use vstore+hnsw rescue correctly
        let got = engine.get(9001).expect("get").expect("exists");
        match got.vector {
            crate::node::VectorRepresentations::Binary(ref b) => {
                assert_eq!(b.as_ref(), bin_data.as_ref())
            }
            ref other => panic!("expected Binary before reopen, got {:?}", other),
        }
    }
    // Reopen — HNSW may be loaded from file, but also test rebuild path by removing index file
    {
        let engine = open_disk_engine_with_wal(path);
        let got = engine
            .get(9001)
            .expect("get after reopen")
            .expect("exists after reopen");
        match got.vector {
            crate::node::VectorRepresentations::Binary(ref b) => {
                assert_eq!(b.as_ref(), bin_data.as_ref())
            }
            ref other => panic!("expected Binary after reopen, got {:?}", other),
        }
        // Force rebuild from vstore alone: remove HNSW file and reopen with empty HNSW
        // (simulates index file loss). The vstore rebuild must recover Binary.
        drop(engine);
        let index_path = std::path::Path::new(path)
            .join("data")
            .join("vector_index.bin");
        let _ = std::fs::remove_file(&index_path);
        let engine2 = open_disk_engine_with_wal(path);
        let got2 = engine2
            .get(9001)
            .expect("get after rebuild")
            .expect("exists after rebuild");
        match got2.vector {
            crate::node::VectorRepresentations::Binary(ref b) => {
                assert_eq!(b.as_ref(), bin_data.as_ref())
            }
            ref other => panic!("expected Binary after rebuild, got {:?}", other),
        }
        // also verify rebuild report indexed it
        let report = engine2.rebuild_vector_index().expect("rebuild");
        assert!(report.scanned_nodes >= 1);
        assert!(report.indexed_vectors >= 1);
    }
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_persistence_turbo_vector_roundtrip_vstore() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    let turbo_data: Box<[u8]> = vec![0xAB, 0xCD, 0xEF, 0x01, 0x23].into_boxed_slice();
    {
        let engine = open_disk_engine_with_wal(path);
        let mut node = sample_node(9002);
        node.vector = crate::node::VectorRepresentations::Turbo(turbo_data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert turbo");
        engine.flush().expect("flush");
    }
    let engine = open_disk_engine_with_wal(path);
    let got = engine.get(9002).expect("get").expect("exists");
    match got.vector {
        crate::node::VectorRepresentations::Turbo(ref t) => {
            assert_eq!(t.as_ref(), turbo_data.as_ref())
        }
        ref other => panic!("expected Turbo, got {:?}", other),
    }
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_persistence_sq8_vector_roundtrip_vstore() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    let sq8_data: Box<[i8]> = vec![10, -20, 30, -40, 50].into_boxed_slice();
    #[allow(clippy::approx_constant)]
    let scale: f32 = 3.14;
    {
        let engine = open_disk_engine_with_wal(path);
        let mut node = sample_node(9003);
        node.vector = crate::node::VectorRepresentations::SQ8(sq8_data.clone(), scale);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert sq8");
        engine.flush().expect("flush");
    }
    let engine = open_disk_engine_with_wal(path);
    let got = engine.get(9003).expect("get").expect("exists");
    match got.vector {
        crate::node::VectorRepresentations::SQ8(ref d, s) => {
            assert_eq!(d.as_ref(), sq8_data.as_ref());
            assert!((s - scale).abs() < f32::EPSILON);
        }
        ref other => panic!("expected SQ8, got {:?}", other),
    }
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_persistence_full_vector_still_roundtrips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().unwrap();
    let full = vec![1.0f32, 2.0, 3.0, 4.0];
    {
        let engine = open_disk_engine_with_wal(path);
        let mut node = sample_node(9004);
        node.vector = crate::node::VectorRepresentations::Full(full.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert full");
        engine.flush().expect("flush");
    }
    let engine = open_disk_engine_with_wal(path);
    let got = engine.get(9004).expect("get").expect("exists");
    match got.vector {
        crate::node::VectorRepresentations::Full(ref v) => assert_eq!(v, &full),
        ref other => panic!("expected Full, got {:?}", other),
    }
}
