//! Storage backend abstraction layer.
//!
//! Defines the `StorageBackend` trait and supporting types that decouple
//! `StorageEngine` from any specific persistent KV store (RocksDB, Fjall, etc.).
//!
//! ## Design notes
//!
//! - `scan()` returns a materialized `Vec<(Vec<u8>, Vec<u8>)>` instead of an
//!   iterator. This avoids `dyn Trait` lifetime complexity and is acceptable
//!   because `scan` is only used in `recover_archived_nodes`, which collects
//!   all entries anyway. It is not intended as a hot-path abstraction.
//!
//! - `compact()` has a default no-op implementation. Backends that lack native
//!   compaction (e.g. `InMemoryBackend`) simply inherit the no-op.
//!
//! - This trait is **crate-internal** (`pub(crate)`). It is not part of the
//!   public API surface and should not be implemented outside this crate.

use crate::error::Result;
use std::path::Path;

// ─── Partition Vocabulary ───────────────────────────────────

/// Logical partitions that replace stringly-typed column family names.
///
/// Every KV operation targets exactly one partition. The backend
/// implementation decides how to map these to physical storage
/// (e.g. RocksDB column families, separate BTreeMaps, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendPartition {
    /// Primary metadata store (node metadata, relational fields).
    Default,
    /// Auditable tombstone archive for conflict resolution losers.
    TombstoneStorage,
    /// Compressed semantic summaries (data compression output).
    CompressedArchive,
    /// Lightweight tombstone markers for `is_deleted` checks.
    Tombstones,
    /// Derived namespace/key index for persistent memory APIs.
    NamespaceIndex,
    /// Derived metadata equality index for persistent memory filters.
    PayloadIndex,
    /// Derived inverted index for persistent memory payload tokens.
    TextIndex,
    /// Internal metadata used for derived-state health markers.
    InternalMetadata,
}

impl BackendPartition {
    /// Returns the RocksDB column family name for this partition.
    /// Used only by `RocksDbBackend` internally.
    #[cfg(feature = "rocksdb")]
    pub(crate) fn cf_name(&self) -> &'static str {
        match self {
            BackendPartition::Default => "default",
            BackendPartition::TombstoneStorage => "tombstone_storage",
            BackendPartition::CompressedArchive => "compressed_archive",
            BackendPartition::Tombstones => "tombstones",
            BackendPartition::NamespaceIndex => "namespace_index",
            BackendPartition::PayloadIndex => "payload_index",
            BackendPartition::TextIndex => "text_index",
            BackendPartition::InternalMetadata => "internal_metadata",
        }
    }
}

// ─── Batch Write Operations ─────────────────────────────────

/// A single write operation within an atomic batch.
#[derive(Clone)]
pub(crate) enum BackendWriteOp {
    /// Insert or update a key-value pair.
    Put {
        /// Target partition.
        partition: BackendPartition,
        /// Key bytes.
        key: Vec<u8>,
        /// Value bytes.
        value: Vec<u8>,
    },
    /// Delete a key.
    Delete {
        /// Target partition.
        partition: BackendPartition,
        /// Key bytes.
        key: Vec<u8>,
    },
}

// ─── Backend Capabilities ───────────────────────────────────

/// Indicates which KV backend is being used.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BackendKind {
    /// RocksDB storage backend.
    RocksDb,
    /// Fjall storage backend (default).
    #[default]
    Fjall,
    /// In-memory storage backend (no persistence).
    InMemory,
}

/// Introspection of a backend's supported features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// Whether the backend supports consistent snapshots.
    pub supports_checkpoint: bool,
    /// Whether the backend supports manual compaction.
    pub supports_manual_compaction: bool,
    /// Which backend implementation is in use.
    pub kind: BackendKind,
}

// ─── Backend Trait ──────────────────────────────────────────

/// Abstraction over the persistent KV store used by `StorageEngine`.
///
/// Covers only the operations that `StorageEngine` actually needs.
/// Does **not** include HNSW, VantaFile, WAL, or any higher-level
/// engine logic — those remain in `StorageEngine` directly.
///
/// This trait is crate-internal and should not be exposed publicly.
pub(crate) trait StorageBackend: Send + Sync {
    /// Write a key-value pair to the given partition.
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()>;

    /// Read a value by key from the given partition.
    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Retrieve multiple values by their keys in a single batch operation.
    ///
    /// Returns a `Vec` of `(key, value)` pairs for every key that was found.
    /// Keys that do not exist are silently omitted from the result.
    ///
    /// The default implementation calls `get()` for each key sequentially.
    /// Backends with native multi-get support should override this for
    /// better performance.
    fn get_many(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        keys.iter()
            .filter_map(|k| match self.get(partition, k) {
                Ok(Some(val)) => Some(Ok((k.to_vec(), val))),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }

    /// Delete a key from the given partition.
    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()>;

    /// Execute a batch of write operations atomically (where supported).
    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()>;

    /// Return all key-value pairs in the given partition.
    ///
    /// Returns a materialized `Vec` to avoid iterator lifetime issues
    /// behind `dyn Trait`. Not intended for hot-path use.
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Return key-value pairs whose keys start with `prefix`.
    ///
    /// This is intended for derived indexes and should avoid materializing
    /// unrelated entries from the same partition.
    fn scan_prefix_iter<'a>(
        &'a self,
        partition: BackendPartition,
        prefix: &'a [u8],
    ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>>;

    /// Materialized version of [`scan_prefix_iter`].
    ///
    /// The default implementation collects from the streaming iterator.
    /// Backends may override if a more efficient materialization exists.
    #[allow(dead_code)]
    fn scan_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.scan_prefix_iter(partition, prefix)?
            .collect::<Result<Vec<_>>>()
    }

    /// Flush all pending writes to durable storage.
    /// Default implementation is a no-op for backends without persistence.
    fn flush(&self) -> Result<()> {
        Ok(())
    }

    /// Create a consistent snapshot at the given filesystem path.
    ///
    /// Backends that do not support checkpointing should return an
    /// explicit error.
    fn checkpoint(&self, path: &Path) -> Result<()>;

    /// Request background compaction. Default implementation is a no-op
    /// for backends that do not support or need compaction.
    fn compact(&self) {
        // no-op by default
    }

    /// Introspect the capabilities of this backend instance.
    fn capabilities(&self) -> BackendCapabilities;
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ── BackendPartition ──

    #[test]
    fn test_backend_partition_variants() {
        assert_ne!(BackendPartition::Default, BackendPartition::Tombstones);
        assert_eq!(BackendPartition::Default, BackendPartition::Default);
    }

    #[test]
    fn test_backend_partition_count() {
        // All 8 partition variants must exist
        let all = [
            BackendPartition::Default,
            BackendPartition::TombstoneStorage,
            BackendPartition::CompressedArchive,
            BackendPartition::Tombstones,
            BackendPartition::NamespaceIndex,
            BackendPartition::PayloadIndex,
            BackendPartition::TextIndex,
            BackendPartition::InternalMetadata,
        ];
        assert_eq!(all.len(), 8);
        // Each variant is distinct
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                assert_ne!(all[i], all[j], "variant {:?} == {:?}", all[i], all[j]);
            }
        }
    }

    #[test]
    fn test_backend_partition_clone_copy() {
        let p = BackendPartition::TextIndex;
        let cloned = p;
        assert_eq!(p, cloned);
    }

    #[test]
    fn test_backend_partition_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        BackendPartition::Default.hash(&mut h1);
        let mut h2 = DefaultHasher::new();
        BackendPartition::Tombstones.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[cfg(feature = "rocksdb")]
    #[test]
    fn test_backend_partition_cf_names() {
        assert_eq!(BackendPartition::Default.cf_name(), "default");
        assert_eq!(
            BackendPartition::TombstoneStorage.cf_name(),
            "tombstone_storage"
        );
        assert_eq!(
            BackendPartition::CompressedArchive.cf_name(),
            "compressed_archive"
        );
        assert_eq!(BackendPartition::Tombstones.cf_name(), "tombstones");
        assert_eq!(
            BackendPartition::NamespaceIndex.cf_name(),
            "namespace_index"
        );
        assert_eq!(BackendPartition::PayloadIndex.cf_name(), "payload_index");
        assert_eq!(BackendPartition::TextIndex.cf_name(), "text_index");
        assert_eq!(
            BackendPartition::InternalMetadata.cf_name(),
            "internal_metadata"
        );
    }

    #[cfg(feature = "rocksdb")]
    #[test]
    fn test_backend_partition_all_unique() {
        let names: std::collections::HashSet<&str> = [
            BackendPartition::Default,
            BackendPartition::TombstoneStorage,
            BackendPartition::CompressedArchive,
            BackendPartition::Tombstones,
            BackendPartition::NamespaceIndex,
            BackendPartition::PayloadIndex,
            BackendPartition::TextIndex,
            BackendPartition::InternalMetadata,
        ]
        .iter()
        .map(|p| p.cf_name())
        .collect();
        assert_eq!(names.len(), 8);
    }

    // ── BackendKind ──

    #[test]
    fn test_backend_kind_default() {
        assert_eq!(BackendKind::default(), BackendKind::Fjall);
    }

    #[test]
    fn test_backend_kind_variants() {
        assert_ne!(BackendKind::RocksDb, BackendKind::Fjall);
        assert_ne!(BackendKind::InMemory, BackendKind::RocksDb);
    }

    #[test]
    fn test_backend_kind_all_variants() {
        // All three variants are distinct
        let kinds = [
            BackendKind::RocksDb,
            BackendKind::Fjall,
            BackendKind::InMemory,
        ];
        for i in 0..kinds.len() {
            for j in i + 1..kinds.len() {
                assert_ne!(kinds[i], kinds[j]);
            }
        }
    }

    #[test]
    fn test_backend_kind_debug() {
        let r = format!("{:?}", BackendKind::RocksDb);
        assert!(r.contains("RocksDb"));
        let f = format!("{:?}", BackendKind::Fjall);
        assert!(f.contains("Fjall"));
        let m = format!("{:?}", BackendKind::InMemory);
        assert!(m.contains("InMemory"));
    }

    // ── BackendCapabilities ──

    #[test]
    fn test_backend_capabilities_defaults() {
        let caps = BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: BackendKind::InMemory,
        };
        assert!(!caps.supports_checkpoint);
        assert!(!caps.supports_manual_compaction);
        assert_eq!(caps.kind, BackendKind::InMemory);
    }

    #[test]
    fn test_backend_capabilities_rocksdb() {
        let caps = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: true,
            kind: BackendKind::RocksDb,
        };
        assert!(caps.supports_checkpoint);
        assert!(caps.supports_manual_compaction);
    }

    #[test]
    fn test_backend_capabilities_fjall() {
        let caps = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::Fjall,
        };
        assert!(caps.supports_checkpoint);
        assert!(!caps.supports_manual_compaction);
        assert_eq!(caps.kind, BackendKind::Fjall);
    }

    #[test]
    fn test_backend_capabilities_in_memory_full() {
        let caps = BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: BackendKind::InMemory,
        };
        assert_eq!(caps.kind, BackendKind::InMemory);
        assert!(!caps.supports_checkpoint && !caps.supports_manual_compaction);
    }

    #[test]
    fn test_backend_capabilities_debug() {
        let caps = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::Fjall,
        };
        let dbg = format!("{:?}", caps);
        assert!(
            dbg.contains("BackendCapabilities"),
            "debug shows struct name"
        );
        assert!(dbg.contains("Fjall"), "debug shows kind");
    }

    #[test]
    fn test_backend_capabilities_partial_eq() {
        let a = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::RocksDb,
        };
        let b = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::RocksDb,
        };
        let c = BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: BackendKind::InMemory,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ── BackendWriteOp ──

    #[test]
    fn test_backend_write_op_put() {
        let op = BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        };
        match op {
            BackendWriteOp::Put {
                partition,
                key,
                value,
            } => {
                assert_eq!(partition, BackendPartition::Default);
                assert_eq!(key, b"k");
                assert_eq!(value, b"v");
            }
            _ => panic!("expected Put"),
        }
    }

    #[test]
    fn test_backend_write_op_delete() {
        let op = BackendWriteOp::Delete {
            partition: BackendPartition::Tombstones,
            key: b"del".to_vec(),
        };
        match op {
            BackendWriteOp::Delete { partition, key } => {
                assert_eq!(partition, BackendPartition::Tombstones);
                assert_eq!(key, b"del");
            }
            _ => panic!("expected Delete"),
        }
    }

    #[test]
    fn test_backend_write_op_edge_cases() {
        // Empty key/value
        let put_empty = BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"".to_vec(),
            value: b"".to_vec(),
        };
        match put_empty {
            BackendWriteOp::Put { key, value, .. } => {
                assert!(key.is_empty());
                assert!(value.is_empty());
            }
            _ => panic!("expected Put"),
        }
        // Large value
        let large_val = vec![0xABu8; 1024 * 64];
        let put_large = BackendWriteOp::Put {
            partition: BackendPartition::CompressedArchive,
            key: b"big".to_vec(),
            value: large_val.clone(),
        };
        match put_large {
            BackendWriteOp::Put { value, .. } => assert_eq!(value.len(), 1024 * 64),
            _ => panic!("expected Put"),
        }
    }

    #[test]
    fn test_backend_write_op_clone() {
        let op = BackendWriteOp::Put {
            partition: BackendPartition::TextIndex,
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        };
        let cloned = op.clone();
        match (&op, &cloned) {
            (
                BackendWriteOp::Put {
                    partition: p1,
                    key: k1,
                    value: v1,
                },
                BackendWriteOp::Put {
                    partition: p2,
                    key: k2,
                    value: v2,
                },
            ) => {
                assert_eq!(p1, p2);
                assert_eq!(k1, k2);
                assert_eq!(v1, v2);
            }
            _ => panic!("both should be Put"),
        }
    }

    // ── StorageBackend trait (via InMemoryBackend) ──

    #[test]
    fn test_backend_trait_default_flush() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        // flush() returns Ok(()) by default — no panic
        assert!(backend.flush().is_ok());
    }

    #[test]
    fn test_backend_trait_default_compact() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        // compact() is a no-op by default — must not panic
        backend.compact();
    }

    #[test]
    fn test_backend_trait_checkpoint_unsupported() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        let tmp = std::env::temp_dir().join("vantadb_test_checkpoint");
        let err = backend.checkpoint(&tmp).unwrap_err();
        assert!(
            err.to_string().contains("not supported"),
            "checkpoint should be unsupported: {}",
            err
        );
    }

    #[test]
    fn test_backend_trait_capabilities_in_memory() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        let caps = backend.capabilities();
        assert!(!caps.supports_checkpoint);
        assert!(!caps.supports_manual_compaction);
        assert_eq!(caps.kind, BackendKind::InMemory);
    }

    #[test]
    fn test_backend_trait_get_many_edge_cases() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        // Empty keys list
        let result = backend.get_many(BackendPartition::Default, &[]).unwrap();
        assert!(result.is_empty(), "empty keys should return empty result");

        // Seed data
        backend.put(BackendPartition::Default, b"a", b"1").unwrap();
        backend.put(BackendPartition::Default, b"b", b"2").unwrap();

        // Mixed found and missing
        let result = backend
            .get_many(BackendPartition::Default, &[b"a", b"missing", b"b"])
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(&result[0].1, b"1");
        assert_eq!(&result[1].1, b"2");
    }

    #[test]
    fn test_backend_trait_scans() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        // Scan empty partition
        let empty = backend.scan(BackendPartition::PayloadIndex).unwrap();
        assert!(empty.is_empty());

        // Scan with data
        backend
            .put(BackendPartition::Default, b"alpha", b"1")
            .unwrap();
        backend
            .put(BackendPartition::Default, b"beta", b"2")
            .unwrap();
        backend
            .put(BackendPartition::Default, b"gamma", b"3")
            .unwrap();
        let all = backend.scan(BackendPartition::Default).unwrap();
        assert_eq!(all.len(), 3);

        // scan_prefix on non-empty
        let prefixed = backend
            .scan_prefix(BackendPartition::Default, b"alpha")
            .unwrap();
        assert_eq!(prefixed.len(), 1);
    }

    #[test]
    fn test_backend_trait_write_batch_rollback_appearance() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        // Write batch: put multiple, delete one
        backend
            .put(BackendPartition::Default, b"survivor", b"val")
            .unwrap();
        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"keep".to_vec(),
                value: b"me".to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: b"survivor".to_vec(),
            },
        ];
        backend.write_batch(ops).unwrap();

        assert!(backend
            .get(BackendPartition::Default, b"keep")
            .unwrap()
            .is_some());
        assert!(backend
            .get(BackendPartition::Default, b"survivor")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_backend_trait_write_batch_empty() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        assert!(backend.write_batch(vec![]).is_ok());
    }

    // ── BackendCapabilities: all combos ──

    #[test]
    fn test_backend_capabilities_all_checkpoint_compaction_combos() {
        // All 4 boolean combos × 3 BackendKinds = 12 combos, spot-check key ones
        let all_false = BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: BackendKind::InMemory,
        };
        let all_true = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: true,
            kind: BackendKind::RocksDb,
        };
        let checkpoint_only = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::Fjall,
        };
        let compaction_only = BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: true,
            kind: BackendKind::RocksDb,
        };
        assert!(!all_false.supports_checkpoint);
        assert!(!all_false.supports_manual_compaction);
        assert!(all_true.supports_checkpoint);
        assert!(all_true.supports_manual_compaction);
        assert!(checkpoint_only.supports_checkpoint);
        assert!(!checkpoint_only.supports_manual_compaction);
        assert!(!compaction_only.supports_checkpoint);
        assert!(compaction_only.supports_manual_compaction);
    }

    #[test]
    fn test_backend_capabilities_clone_and_copy() {
        let a = BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: false,
            kind: BackendKind::Fjall,
        };
        let b = a; // Copy, not move
        assert_eq!(a, b); // Both still usable
        let c = a;
        assert_eq!(a, c);
    }

    #[test]
    fn test_backend_capabilities_kind_combinations() {
        for &kind in &[
            BackendKind::RocksDb,
            BackendKind::Fjall,
            BackendKind::InMemory,
        ] {
            let caps = BackendCapabilities {
                supports_checkpoint: true,
                supports_manual_compaction: false,
                kind,
            };
            assert_eq!(caps.kind, kind);
        }
    }

    // ── BackendPartition Debug ──

    #[test]
    fn test_backend_partition_debug() {
        let d = format!("{:?}", BackendPartition::Default);
        assert_eq!(d, "Default");
        let ts = format!("{:?}", BackendPartition::TombstoneStorage);
        assert_eq!(ts, "TombstoneStorage");
    }

    // ── BackendKind: clone, copy ──

    #[test]
    fn test_backend_kind_clone_copy() {
        let a = BackendKind::RocksDb;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a.clone(), BackendKind::RocksDb);
    }

    // ── BackendWriteOp: Debug format ──

    #[test]
    fn test_backend_write_op_debug() {
        // BackendWriteOp only derives Clone, not Debug, so we can't test Debug format.
        // Verify display-related behavior via manual match.
        let op = BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        };
        match op {
            BackendWriteOp::Put {
                partition, ref key, ..
            } => {
                assert_eq!(partition, BackendPartition::Default);
                assert_eq!(key, b"k");
            }
            _ => panic!("expected Put"),
        }
    }

    // ── Default trait implementation tests via wrapper ──
    //
    // Create a minimal backend that delegates to InMemoryBackend but
    // does NOT override get_many() or scan_prefix(), so we exercise
    // the trait defaults.

    struct DefaultGetManyWrapper {
        inner: crate::backends::in_memory::InMemoryBackend,
    }

    impl DefaultGetManyWrapper {
        fn new() -> Self {
            Self {
                inner: crate::backends::in_memory::InMemoryBackend::new(),
            }
        }
    }

    impl StorageBackend for DefaultGetManyWrapper {
        fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
            self.inner.put(partition, key, value)
        }
        fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
            self.inner.get(partition, key)
        }
        fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
            self.inner.delete(partition, key)
        }
        fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
            self.inner.write_batch(ops)
        }
        fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
            self.inner.scan(partition)
        }
        fn scan_prefix_iter<'a>(
            &'a self,
            partition: BackendPartition,
            prefix: &'a [u8],
        ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>> {
            self.inner.scan_prefix_iter(partition, prefix)
        }
        fn checkpoint(&self, path: &Path) -> Result<()> {
            self.inner.checkpoint(path)
        }
        fn capabilities(&self) -> BackendCapabilities {
            self.inner.capabilities()
        }
        // Intentionally NOT overriding get_many() — testing the trait default
        // Intentionally NOT overriding scan_prefix() — testing the trait default
    }

    #[test]
    fn test_default_get_many_empty_keys() {
        let backend = DefaultGetManyWrapper::new();
        let result = backend.get_many(BackendPartition::Default, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_default_get_many_with_data() {
        let backend = DefaultGetManyWrapper::new();
        backend.put(BackendPartition::Default, b"x", b"1").unwrap();
        backend.put(BackendPartition::Default, b"y", b"2").unwrap();
        // Default get_many: iterates calling get() for each key
        let result = backend
            .get_many(BackendPartition::Default, &[b"x", b"missing", b"y"])
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(&result[0].1, b"1");
        assert_eq!(&result[1].1, b"2");
    }

    #[test]
    fn test_default_get_many_partition_empty() {
        let backend = DefaultGetManyWrapper::new();
        // Partition that has no entries
        let result = backend
            .get_many(BackendPartition::PayloadIndex, &[b"a"])
            .unwrap();
        assert!(result.is_empty());
    }

    // ── Default scan_prefix via wrapper ──

    #[test]
    fn test_default_scan_prefix_basic() {
        let backend = DefaultGetManyWrapper::new();
        backend
            .put(BackendPartition::Default, b"alpha_x", b"1")
            .unwrap();
        backend
            .put(BackendPartition::Default, b"alpha_y", b"2")
            .unwrap();
        backend
            .put(BackendPartition::Default, b"beta_z", b"3")
            .unwrap();

        let prefix_results = backend
            .scan_prefix(BackendPartition::Default, b"alpha")
            .unwrap();
        assert_eq!(prefix_results.len(), 2);
        assert_eq!(&prefix_results[0].1, b"1");
    }

    #[test]
    fn test_default_scan_prefix_empty_partition() {
        let backend = DefaultGetManyWrapper::new();
        let result = backend
            .scan_prefix(BackendPartition::TextIndex, b"x")
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_default_scan_prefix_no_match() {
        let backend = DefaultGetManyWrapper::new();
        backend
            .put(BackendPartition::Default, b"aaa", b"v")
            .unwrap();
        let result = backend
            .scan_prefix(BackendPartition::Default, b"bbb")
            .unwrap();
        assert!(result.is_empty());
    }

    // ── Trait edge cases ──

    #[test]
    fn test_trait_delete_nonexistent_key() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        // Deleting a non-existent key should succeed
        assert!(backend
            .delete(BackendPartition::Default, b"nonexistent")
            .is_ok());
    }

    #[test]
    fn test_trait_partition_isolation() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        backend
            .put(BackendPartition::Default, b"shared", b"default_val")
            .unwrap();
        backend
            .put(BackendPartition::PayloadIndex, b"shared", b"payload_val")
            .unwrap();

        // Same key in different partitions should return different values
        let from_default = backend
            .get(BackendPartition::Default, b"shared")
            .unwrap()
            .unwrap();
        let from_payload = backend
            .get(BackendPartition::PayloadIndex, b"shared")
            .unwrap()
            .unwrap();
        assert_eq!(from_default, b"default_val");
        assert_eq!(from_payload, b"payload_val");
    }

    #[test]
    fn test_trait_put_get_roundtrip_all_partitions() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        let partitions = [
            BackendPartition::Default,
            BackendPartition::TombstoneStorage,
            BackendPartition::CompressedArchive,
            BackendPartition::Tombstones,
            BackendPartition::NamespaceIndex,
            BackendPartition::PayloadIndex,
            BackendPartition::TextIndex,
            BackendPartition::InternalMetadata,
        ];

        for (i, part) in partitions.iter().enumerate() {
            let key = format!("k{i}");
            let val = format!("v{i}");
            backend.put(*part, key.as_bytes(), val.as_bytes()).unwrap();
            let got = backend.get(*part, key.as_bytes()).unwrap().unwrap();
            assert_eq!(got, val.as_bytes(), "round-trip failed for {part:?}");
        }
    }

    #[test]
    fn test_trait_get_many_duplicate_keys() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();
        backend
            .put(BackendPartition::Default, b"dup", b"val")
            .unwrap();

        // Requesting same key twice returns it twice
        let result = backend
            .get_many(BackendPartition::Default, &[b"dup", b"dup"])
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(&result[0].1, b"val");
        assert_eq!(&result[1].1, b"val");
    }

    #[test]
    fn test_trait_scan_prefix_on_wrapper() {
        // Explicitly test scan_prefix on DefaultGetManyWrapper which does NOT
        // override scan_prefix, exercising the trait default body.
        let backend = DefaultGetManyWrapper::new();
        backend
            .put(BackendPartition::Default, b"prefix_key", b"v")
            .unwrap();
        let result = backend
            .scan_prefix(BackendPartition::Default, b"prefix")
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_write_batch_all_partitions() {
        use crate::backends::in_memory::InMemoryBackend;
        let backend = InMemoryBackend::new();

        let mut ops = Vec::new();
        for part in &[
            BackendPartition::Default,
            BackendPartition::Tombstones,
            BackendPartition::TextIndex,
        ] {
            ops.push(BackendWriteOp::Put {
                partition: *part,
                key: b"k".to_vec(),
                value: b"v".to_vec(),
            });
        }
        backend.write_batch(ops).unwrap();

        assert_eq!(
            backend
                .get(BackendPartition::Default, b"k")
                .unwrap()
                .unwrap(),
            b"v"
        );
        assert_eq!(
            backend
                .get(BackendPartition::Tombstones, b"k")
                .unwrap()
                .unwrap(),
            b"v"
        );
    }
}
