//! Fjall-backed implementation of `StorageBackend`.
//!
//! This adapter maps the `StorageBackend` trait onto `fjall` v3.1.x.
//!
//! ## Fjall API model (v3.1.4)
//!
//! - **`fjall::Database`**: Top-level container. Owns the journal and all
//!   keyspaces. One per engine path. Equivalent to a RocksDB `DB` instance.
//! - **`fjall::Keyspace`**: A named LSM-tree within the Database. Each
//!   `BackendPartition` maps 1:1 to a Keyspace using the same string names
//!   as the RocksDB column families.
//! - **`fjall::OwnedWriteBatch`** (aliased as `WriteBatch`): Atomic batch
//!   that can span multiple Keyspaces. Equivalent to RocksDB `WriteBatch`.
//! - **`fjall::PersistMode`**: Controls durability on `Database::persist()`.
//!   `SyncAll` = fsync(data + metadata), strongest guarantee.
//!
//! ## Limitations vs RocksDB
//!
//! - **No checkpoint**: Fjall does not expose a point-in-time snapshot-to-disk
//!   API. `checkpoint()` returns an explicit error.
//! - **No manual compaction**: Fjall manages compaction internally via its
//!   LSM background threads. `compact()` is a no-op.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use std::path::Path;
use tracing::info;

/// Fjall adapter implementing `StorageBackend`.
///
/// Owns a `fjall::Database` and four `Keyspace` handles corresponding to
/// the `BackendPartition` variants. Created through `FjallBackend::open`.
pub(crate) struct FjallBackend {
    db: Database,
    default: Keyspace,
    tombstone_storage: Keyspace,
    compressed_archive: Keyspace,
    tombstones: Keyspace,
    namespace_index: Keyspace,
    payload_index: Keyspace,
}

impl FjallBackend {
    /// Open a Fjall database at `path`.
    ///
    /// Creates the database directory if it does not exist.
    /// Opens (or creates) one keyspace per `BackendPartition` using the
    /// same names as the RocksDB column families for semantic continuity.
    pub(crate) fn open(path: &str) -> Result<Self> {
        let db = Database::builder(path)
            .open()
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let default = db
            .keyspace("default", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let tombstone_storage = db
            .keyspace("tombstone_storage", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let compressed_archive = db
            .keyspace("compressed_archive", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let tombstones = db
            .keyspace("tombstones", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let namespace_index = db
            .keyspace("namespace_index", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let payload_index = db
            .keyspace("payload_index", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        info!("Fjall database opened at '{}'", path);

        Ok(Self {
            db,
            default,
            tombstone_storage,
            compressed_archive,
            tombstones,
            namespace_index,
            payload_index,
        })
    }

    /// Resolve a `BackendPartition` to the corresponding `Keyspace` handle.
    fn keyspace(&self, partition: BackendPartition) -> &Keyspace {
        match partition {
            BackendPartition::Default => &self.default,
            BackendPartition::TombstoneStorage => &self.tombstone_storage,
            BackendPartition::CompressedArchive => &self.compressed_archive,
            BackendPartition::Tombstones => &self.tombstones,
            BackendPartition::NamespaceIndex => &self.namespace_index,
            BackendPartition::PayloadIndex => &self.payload_index,
        }
    }
}

impl StorageBackend for FjallBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .insert(key, value)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.keyspace(partition)
            .get(key)
            .map(|opt| opt.map(|slice| slice.to_vec()))
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .remove(key)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        // OwnedWriteBatch (type-aliased as WriteBatch) provides native atomic
        // writes across multiple Keyspaces within the same Database.
        // All operations are committed atomically via the shared journal.
        let mut batch = self.db.batch();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    batch.insert(self.keyspace(partition), key, value);
                }
                BackendWriteOp::Delete { partition, key } => {
                    batch.remove(self.keyspace(partition), key);
                }
            }
        }
        batch
            .commit()
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let ks = self.keyspace(partition);
        let mut result = Vec::new();
        for item in ks.iter() {
            let kv = item
                .into_inner()
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;
            result.push((kv.0.to_vec(), kv.1.to_vec()));
        }
        Ok(result)
    }

    /// Flush pending writes to durable storage.
    ///
    /// Uses `PersistMode::SyncAll` which calls `fsync` on both data and
    /// metadata, providing the strongest durability guarantee Fjall offers.
    ///
    /// Per Fjall docs: "Persisting only affects durability, NOT consistency.
    /// Even without flushing data is crash-safe." The journal architecture
    /// provides crash consistency regardless; this call ensures data survives
    /// power loss.
    fn flush(&self) -> Result<()> {
        self.db
            .persist(PersistMode::SyncAll)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    /// Checkpoint is not supported by Fjall.
    ///
    /// Fjall does not expose a point-in-time consistent snapshot-to-disk API
    /// equivalent to RocksDB's `Checkpoint::create_checkpoint`. Returning an
    /// honest error rather than simulating with unsafe file copies.
    fn checkpoint(&self, _path: &Path) -> Result<()> {
        Err(VantaError::Execution(
            "Checkpoint not supported by FjallBackend: Fjall does not expose a \
             point-in-time snapshot-to-disk API equivalent to RocksDB checkpoints"
                .to_string(),
        ))
    }

    /// No-op: Fjall manages LSM compaction automatically via internal
    /// background threads. No manual compaction trigger is needed or
    /// exposed for this use case.
    fn compact(&self) {
        // Fjall's LSM engine (lsm-tree crate) runs automatic background
        // compaction. There is no public manual compaction API to call here.
    }

    fn capabilities(&self) -> crate::backend::BackendCapabilities {
        crate::backend::BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: crate::backend::BackendKind::Fjall,
        }
    }
}
