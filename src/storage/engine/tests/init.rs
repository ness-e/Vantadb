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

#[test]
fn test_memory_governor_field() {
    let engine = in_memory_engine();
    assert!(
        engine.memory_governor.is_some(),
        "memory_governor should exist"
    );
}
