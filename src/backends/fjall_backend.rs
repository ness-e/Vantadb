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
//!   API, so it does NOT implement the `Snapshotable` role
//!   (`as_snapshotable() == None`, `supports_checkpoint == false`).
//! - **No manual compaction**: Fjall manages compaction internally via its
//!   LSM background threads, so it does NOT implement the `Compactable`
//!   role (`as_compactable() == None`).

use crate::backend::{BackendPartition, BackendWriteOp, Scannable, StorageBackend};
use crate::config::Config;
use crate::error::{Error, Result};
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use tracing::info;

/// Fjall-backed implementation of `StorageBackend`.
///
/// Owns a `fjall::Database` and nine `Keyspace` handles corresponding to
/// the `BackendPartition` variants. Created through `FjallBackend::open`.
pub(crate) struct FjallBackend {
    db: Database,
    default: Keyspace,
    tombstone_storage: Keyspace,
    compressed_archive: Keyspace,
    tombstones: Keyspace,
    namespace_index: Keyspace,
    payload_index: Keyspace,
    text_index: Keyspace,
    sparse_index: Keyspace,
    internal_metadata: Keyspace,
    versions: Keyspace,
}

impl FjallBackend {
    /// Open a Fjall database at `path`.
    ///
    /// Creates the database directory if it does not exist.
    /// Opens (or creates) one keyspace per `BackendPartition` using the
    /// same names as the RocksDB column families for semantic continuity.
    pub(crate) fn open(path: &str, _config: &Config) -> Result<Self> {
        let db = Database::builder(path)
            .open()
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let default = db
            .keyspace("default", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let tombstone_storage = db
            .keyspace("tombstone_storage", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let compressed_archive = db
            .keyspace("compressed_archive", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let tombstones = db
            .keyspace("tombstones", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let namespace_index = db
            .keyspace("namespace_index", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let payload_index = db
            .keyspace("payload_index", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let text_index = db
            .keyspace("text_index", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let sparse_index = db
            .keyspace("sparse_index", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let internal_metadata = db
            .keyspace("internal_metadata", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        let versions = db
            .keyspace("versions", KeyspaceCreateOptions::default)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        info!("Fjall database opened at '{}'", path);

        Ok(Self {
            db,
            default,
            tombstone_storage,
            compressed_archive,
            tombstones,
            namespace_index,
            payload_index,
            text_index,
            sparse_index,
            internal_metadata,
            versions,
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
            BackendPartition::TextIndex => &self.text_index,
            BackendPartition::SparseIndex => &self.sparse_index,
            BackendPartition::InternalMetadata => &self.internal_metadata,
            BackendPartition::Versions => &self.versions,
        }
    }
}

impl StorageBackend for FjallBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .insert(key, value)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.keyspace(partition)
            .get(key)
            .map(|opt| opt.map(|slice| slice.to_vec()))
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn get_many(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let ks = self.keyspace(partition);
        keys.iter()
            .filter_map(|k| match ks.get(k) {
                Ok(Some(val)) => Some(Ok((k.to_vec(), val.to_vec()))),
                Ok(None) => None,
                Err(e) => Some(Err(Error::Io(std::io::Error::other(e.to_string())))),
            })
            .collect()
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .remove(key)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
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
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
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
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn capabilities(&self) -> crate::backend::BackendCapabilities {
        crate::backend::BackendCapabilities {
            supports_checkpoint: false,
            supports_manual_compaction: false,
            kind: crate::backend::BackendKind::Fjall,
        }
    }
}

/// Fjall serves the scan role. Snapshot and compaction roles are NOT
/// implemented (Fjall exposes no point-in-time snapshot or manual
/// compaction API): callers see `as_*() == None` + `capabilities()`.
impl Scannable for FjallBackend {
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let ks = self.keyspace(partition);
        let mut result = Vec::new();
        for item in ks.iter() {
            let kv = item
                .into_inner()
                .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
            result.push((kv.0.to_vec(), kv.1.to_vec()));
        }
        Ok(result)
    }

    fn scan_prefix_iter<'a>(
        &'a self,
        partition: BackendPartition,
        prefix: &'a [u8],
    ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>> {
        let ks = self.keyspace(partition);
        let prefix = prefix.to_vec();
        let mut range = ks.range(prefix.as_slice()..);
        Ok(Box::new(std::iter::from_fn(move || {
            let guard = range.next()?;
            let (key, value) = match guard.into_inner() {
                Ok(kv) => kv,
                Err(e) => {
                    return Some(Err(Error::Io(std::io::Error::other(e.to_string()))));
                }
            };
            if !key.starts_with(&prefix) {
                return None;
            }
            Some(Ok((key.to_vec(), value.to_vec())))
        })))
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backend::BackendWriteOp;
    use crate::config::Config;
    use tempfile::tempdir;

    fn open_fjall() -> (FjallBackend, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config::default();
        let backend = FjallBackend::open(dir.path().to_str().unwrap(), &config).unwrap();
        (backend, dir)
    }

    #[test]
    fn test_fjall_open() {
        let (_b, _dir) = open_fjall();
    }

    #[test]
    fn test_fjall_put_get_default() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"k1", b"v1").unwrap();
        let val = b
            .get(BackendPartition::Default, b"k1")
            .unwrap()
            .expect("k1");
        assert_eq!(val, b"v1");
    }

    #[test]
    fn test_fjall_get_missing() {
        let (b, _dir) = open_fjall();
        assert!(b
            .get(BackendPartition::Default, b"missing")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_fjall_put_get_multi_partition() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"d", b"default").unwrap();
        b.put(BackendPartition::Tombstones, b"t", b"tomb").unwrap();
        b.put(BackendPartition::TextIndex, b"x", b"text").unwrap();
        assert_eq!(
            b.get(BackendPartition::Default, b"d").unwrap().unwrap(),
            b"default"
        );
        assert_eq!(
            b.get(BackendPartition::Tombstones, b"t").unwrap().unwrap(),
            b"tomb"
        );
        assert_eq!(
            b.get(BackendPartition::TextIndex, b"x").unwrap().unwrap(),
            b"text"
        );
    }

    #[test]
    fn test_fjall_delete() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"k", b"v").unwrap();
        b.delete(BackendPartition::Default, b"k").unwrap();
        assert!(b.get(BackendPartition::Default, b"k").unwrap().is_none());
    }

    #[test]
    fn test_fjall_delete_missing() {
        let (b, _dir) = open_fjall();
        // Deleting a missing key should not error
        b.delete(BackendPartition::Default, b"missing").unwrap();
    }

    #[test]
    fn test_fjall_write_batch() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"del", b"val").unwrap();

        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"a".to_vec(),
                value: b"1".to_vec(),
            },
            BackendWriteOp::Put {
                partition: BackendPartition::NamespaceIndex,
                key: b"b".to_vec(),
                value: b"2".to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: b"del".to_vec(),
            },
        ];
        b.write_batch(ops).unwrap();

        assert_eq!(
            b.get(BackendPartition::Default, b"a").unwrap().unwrap(),
            b"1"
        );
        assert_eq!(
            b.get(BackendPartition::NamespaceIndex, b"b")
                .unwrap()
                .unwrap(),
            b"2"
        );
        assert!(b.get(BackendPartition::Default, b"del").unwrap().is_none());
    }

    #[test]
    fn test_fjall_scan() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"k1", b"v1").unwrap();
        b.put(BackendPartition::Default, b"k2", b"v2").unwrap();
        let entries = b.scan(BackendPartition::Default).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_fjall_scan_empty_partition() {
        let (b, _dir) = open_fjall();
        assert!(b
            .scan(BackendPartition::CompressedArchive)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_fjall_scan_prefix() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::PayloadIndex, b"abc", b"1").unwrap();
        b.put(BackendPartition::PayloadIndex, b"abd", b"2").unwrap();
        b.put(BackendPartition::PayloadIndex, b"zzz", b"3").unwrap();
        let entries = b
            .scan_prefix(BackendPartition::PayloadIndex, b"ab")
            .unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, b"abc");
        assert_eq!(entries[1].0, b"abd");
    }

    #[test]
    fn test_fjall_scan_prefix_no_match() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"abc", b"1").unwrap();
        assert!(b
            .scan_prefix(BackendPartition::Default, b"zz")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_fjall_flush() {
        let (b, _dir) = open_fjall();
        b.put(BackendPartition::Default, b"k", b"v").unwrap();
        b.flush().unwrap();
        // After flush the value should still be there
        assert_eq!(
            b.get(BackendPartition::Default, b"k").unwrap().unwrap(),
            b"v"
        );
    }

    #[test]
    fn test_fjall_snapshot_role_absent() {
        let (b, _dir) = open_fjall();
        // Fjall does NOT implement Snapshotable (no native snapshot API):
        // non-support is declared in the type, not discovered at runtime.
        assert!(b.as_snapshotable().is_none());
        assert!(!b.capabilities().supports_checkpoint);
    }

    #[test]
    fn test_fjall_compact_role_absent() {
        let (b, _dir) = open_fjall();
        // Fjall manages LSM compaction internally: no Compactable role.
        assert!(b.as_compactable().is_none());
        assert!(!b.capabilities().supports_manual_compaction);
    }

    #[test]
    fn test_fjall_capabilities() {
        let (b, _dir) = open_fjall();
        let caps = b.capabilities();
        assert!(!caps.supports_checkpoint);
        assert!(!caps.supports_manual_compaction);
        assert_eq!(caps.kind, crate::backend::BackendKind::Fjall);
    }
}
