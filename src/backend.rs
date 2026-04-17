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
pub(crate) enum BackendPartition {
    /// Primary metadata store (node metadata, relational fields).
    Default,
    /// Auditable tombstone archive for conflict resolution losers.
    TombstoneStorage,
    /// Compressed semantic summaries (data compression output).
    CompressedArchive,
    /// Lightweight tombstone markers for `is_deleted` checks.
    Tombstones,
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
        }
    }
}

// ─── Batch Write Operations ─────────────────────────────────

/// A single write operation within an atomic batch.
pub(crate) enum BackendWriteOp {
    #[allow(dead_code)]
    Put {
        partition: BackendPartition,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        partition: BackendPartition,
        key: Vec<u8>,
    },
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

    /// Delete a key from the given partition.
    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()>;

    /// Execute a batch of write operations atomically (where supported).
    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()>;

    /// Return all key-value pairs in the given partition.
    ///
    /// Returns a materialized `Vec` to avoid iterator lifetime issues
    /// behind `dyn Trait`. Not intended for hot-path use.
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Flush all pending writes to durable storage.
    fn flush(&self) -> Result<()>;

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
}
