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
    fn scan_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

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
}
