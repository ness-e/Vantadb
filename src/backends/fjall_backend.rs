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
//! - **Compaction via keyspace rewrite**: Fjall manages LSM compaction
//!   internally but its WAL/journal (`0.jnl`) retains all key-value history
//!   (including tombstoned entries) until Fjall's own GC runs. For physical
//!   erasure (FIND-343 / EGO-16), `FjallBackend` implements `Compactable`
//!   via an **atomic keyspace rewrite**: it copies only live key-value pairs
//!   to a fresh `Database` at a temp path, drops all file handles (required
//!   on Windows), replaces the data directory, and reopens — producing a
//!   journal that contains no trace of previously deleted data.
//!
//! ## Thread safety
//!
//! All mutable state (the `Database` handle and all `Keyspace` handles) lives
//! inside a `parking_lot::RwLock<FjallInner>`. Normal read-path operations
//! take a read lock; `compact()` takes an exclusive write lock, performs the
//! full rewrite, and replaces `inner` with the fresh handles before releasing.

use std::path::PathBuf;

use crate::backend::{BackendPartition, BackendWriteOp, Compactable, Scannable, StorageBackend};
use crate::config::Config;
use crate::error::{Error, Result};
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use parking_lot::RwLock;
use tracing::info;

// ─── Inner state ──────────────────────────────────────────────────────────────

/// All mutable Fjall handles owned by `FjallBackend`.
///
/// Bundled together so they can be atomically replaced during compaction
/// (`compact()` takes `RwLock::write()`, swaps the struct, and releases).
struct FjallInner {
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

impl FjallInner {
    /// Open (or create) all keyspaces in `db`. Takes ownership of `db`.
    fn open_keyspaces(db: Database) -> Result<Self> {
        macro_rules! ks {
            ($db:expr, $name:expr) => {
                $db.keyspace($name, KeyspaceCreateOptions::default)
                    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
            };
        }

        let default = ks!(db, "default");
        let tombstone_storage = ks!(db, "tombstone_storage");
        let compressed_archive = ks!(db, "compressed_archive");
        let tombstones = ks!(db, "tombstones");
        let namespace_index = ks!(db, "namespace_index");
        let payload_index = ks!(db, "payload_index");
        let text_index = ks!(db, "text_index");
        let sparse_index = ks!(db, "sparse_index");
        let internal_metadata = ks!(db, "internal_metadata");
        let versions = ks!(db, "versions");

        Ok(FjallInner {
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

    /// Collect all live (non-tombstoned) KV pairs for every partition.
    fn collect_live_pairs(&self) -> Result<Vec<(BackendPartition, Vec<u8>, Vec<u8>)>> {
        let all_partitions = [
            BackendPartition::Default,
            BackendPartition::TombstoneStorage,
            BackendPartition::CompressedArchive,
            BackendPartition::Tombstones,
            BackendPartition::NamespaceIndex,
            BackendPartition::PayloadIndex,
            BackendPartition::TextIndex,
            BackendPartition::SparseIndex,
            BackendPartition::InternalMetadata,
            BackendPartition::Versions,
        ];

        let mut pairs: Vec<(BackendPartition, Vec<u8>, Vec<u8>)> = Vec::new();
        for partition in all_partitions {
            let ks = self.keyspace(partition);
            for item in ks.iter() {
                let kv = item
                    .into_inner()
                    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
                pairs.push((partition, kv.0.to_vec(), kv.1.to_vec()));
            }
        }
        Ok(pairs)
    }
}

// ─── FjallBackend ─────────────────────────────────────────────────────────────

/// Fjall-backed implementation of `StorageBackend`.
///
/// Owns a `fjall::Database` and nine `Keyspace` handles corresponding to
/// the `BackendPartition` variants. Created through `FjallBackend::open`.
///
/// All mutable state is guarded by a `parking_lot::RwLock` so that
/// `compact()` can safely replace the inner `Database` and `Keyspace`
/// handles in-place without exposing a data race.
pub(crate) struct FjallBackend {
    /// Path to the data directory (kept for reopening after compaction).
    path: PathBuf,
    /// Guarded handles. Every read-path operation takes `RwLock::read()`.
    /// `compact()` takes `RwLock::write()` for the full atomic rewrite.
    inner: RwLock<FjallInner>,
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

        let inner = FjallInner::open_keyspaces(db)?;

        info!("Fjall database opened at '{}'", path);

        Ok(Self {
            path: PathBuf::from(path),
            inner: RwLock::new(inner),
        })
    }
}

// ─── StorageBackend ───────────────────────────────────────────────────────────

impl StorageBackend for FjallBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        self.inner
            .read()
            .keyspace(partition)
            .insert(key, value)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.inner
            .read()
            .keyspace(partition)
            .get(key)
            .map(|opt| opt.map(|slice| slice.to_vec()))
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn get_many(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let guard = self.inner.read();
        let ks = guard.keyspace(partition);
        keys.iter()
            .filter_map(|k| match ks.get(k) {
                Ok(Some(val)) => Some(Ok((k.to_vec(), val.to_vec()))),
                Ok(None) => None,
                Err(e) => Some(Err(Error::Io(std::io::Error::other(e.to_string())))),
            })
            .collect()
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        self.inner
            .read()
            .keyspace(partition)
            .remove(key)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        // OwnedWriteBatch (type-aliased as WriteBatch) provides native atomic
        // writes across multiple Keyspaces within the same Database.
        // All operations are committed atomically via the shared journal.
        let guard = self.inner.read();
        let mut batch = guard.db.batch();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    batch.insert(guard.keyspace(partition), key, value);
                }
                BackendWriteOp::Delete { partition, key } => {
                    batch.remove(guard.keyspace(partition), key);
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
        self.inner
            .read()
            .db
            .persist(PersistMode::SyncAll)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    }

    fn capabilities(&self) -> crate::backend::BackendCapabilities {
        crate::backend::BackendCapabilities {
            supports_checkpoint: false,
            // FIND-343: physical erasure via atomic keyspace rewrite.
            supports_manual_compaction: true,
            kind: crate::backend::BackendKind::Fjall,
        }
    }

    /// Whether the store holds no data at all.
    ///
    /// Cheap: every keyspace answers "is it empty?" without materialising
    /// its entries. Used by the at-rest encryption wiring to tell a
    /// brand-new database from an existing plaintext one.
    #[cfg(feature = "encryption")]
    fn is_empty(&self) -> Result<bool> {
        let guard = self.inner.read();
        for keyspace in [
            &guard.default,
            &guard.tombstone_storage,
            &guard.compressed_archive,
            &guard.tombstones,
            &guard.namespace_index,
            &guard.payload_index,
            &guard.text_index,
            &guard.sparse_index,
            &guard.internal_metadata,
            &guard.versions,
        ] {
            let empty = keyspace
                .is_empty()
                .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
            if !empty {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Expose the `Compactable` role.
    ///
    /// `FjallBackend` implements physical erasure via atomic keyspace rewrite
    /// (FIND-343): see [`FjallBackend`] module-level docs and
    /// [`Compactable::compact`] below.
    fn as_compactable(&self) -> Option<&dyn Compactable> {
        Some(self)
    }
}

// ─── Scannable ────────────────────────────────────────────────────────────────

/// Fjall serves the scan role. The snapshot role is NOT implemented
/// (Fjall exposes no point-in-time snapshot API): callers see
/// `as_snapshotable() == None` + `capabilities().supports_checkpoint == false`.
impl Scannable for FjallBackend {
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let guard = self.inner.read();
        let ks = guard.keyspace(partition);
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
        // Eagerly collect all matching KV pairs under the read lock, then
        // return an infallible iterator over the owned Vec. This sidesteps
        // the lifetime challenge of holding a `parking_lot::RwLockReadGuard`
        // across the returned iterator boundary, and is equivalent in
        // correctness to the original lazy implementation because:
        //   1. `scan_prefix` callers iterate fully in the same logical step.
        //   2. `compact()` holds a *write* lock, so no scan is in progress
        //      concurrently — the data is not mutated mid-scan.
        let guard = self.inner.read();
        let ks = guard.keyspace(partition);
        let mut results: Vec<Result<(Vec<u8>, Vec<u8>)>> = Vec::new();
        for item in ks.range(prefix..) {
            let (key, value) = match item.into_inner() {
                Ok(kv) => kv,
                Err(e) => {
                    results.push(Err(Error::Io(std::io::Error::other(e.to_string()))));
                    break;
                }
            };
            if !key.starts_with(prefix) {
                break;
            }
            results.push(Ok((key.to_vec(), value.to_vec())));
        }
        Ok(Box::new(results.into_iter()))
    }
}

// ─── Compactable (FIND-343: physical erasure) ─────────────────────────────────

/// Physical erasure via **atomic keyspace rewrite**.
///
/// # Why this is necessary
///
/// Fjall's `0.jnl` journal is pre-allocated to 64 MiB and is only rotated /
/// purged when Fjall's own GC decides to (based on `max_journaling_size_in_bytes`,
/// default 512 MiB, AND all memtables flushed to SST). VantaDB has no way to
/// force-rotate the journal via Fjall's public API. After a `delete()` followed
/// by `flush()` + WAL rotation, the payload bytes still reside in `0.jnl`.
///
/// # Strategy
///
/// 1. Acquire exclusive write lock — no reads or writes proceed during compaction.
/// 2. Flush current state to durable storage (`SyncAll`).
/// 3. Collect all live (non-tombstoned) KV pairs from every partition.
/// 4. Open a brand-new `Database` at `.fjall_compaction_tmp` (sibling of data dir).
/// 5. Write all live KV pairs to the new `Database`, flush, then drop it.
/// 6. Drop the old `Database` handle (releases all file locks — critical on Windows).
/// 7. Delete old data directory, rename temp dir into its place.
/// 8. Reopen `Database` at the original path and update `inner`.
///
/// # Windows constraints
///
/// On Windows, `std::fs::rename` fails with `os error 5` (Access Denied) if
/// **any** process has an open handle to a file inside the source or target
/// directory. Steps 5 and 6 ensure all Fjall file handles are closed before
/// the filesystem operations.
impl Compactable for FjallBackend {
    fn compact(&self) -> Result<bool> {
        // ── 1. Acquire exclusive write lock ──────────────────────────────────
        let mut guard = self.inner.write();

        // ── 2. Flush current state to SST segments ───────────────────────────
        guard
            .db
            .persist(PersistMode::SyncAll)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;

        // ── 3. Collect all live KV pairs ─────────────────────────────────────
        let live_pairs = guard.collect_live_pairs()?;
        tracing::info!(
            live_kv_pairs = live_pairs.len(),
            "compact(Fjall): collected live pairs; starting atomic keyspace rewrite"
        );

        // ── 4. Compute temporary path ─────────────────────────────────────────
        let tmp_path: PathBuf = {
            let parent = self
                .path
                .parent()
                .ok_or_else(|| {
                    Error::Io(std::io::Error::other(
                        "FjallBackend::compact: data path has no parent directory",
                    ))
                })?
                .to_path_buf();
            parent.join(format!(".fjall_compaction_tmp_{}", std::process::id()))
        };

        // Remove stale temp dir if present (e.g. previous failed compaction).
        if tmp_path.exists() {
            std::fs::remove_dir_all(&tmp_path).map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "compact: remove stale tmp: {e}"
                )))
            })?;
        }

        // ── 5. Write live data to new Database and flush it ───────────────────
        // All handles to tmp_path are dropped at the end of this block.
        {
            let tmp_db = Database::builder(&tmp_path).open().map_err(|e| {
                Error::Io(std::io::Error::other(format!("compact: open tmp db: {e}")))
            })?;

            let tmp_inner = FjallInner::open_keyspaces(tmp_db)?;

            for (partition, key, value) in &live_pairs {
                tmp_inner
                    .keyspace(*partition)
                    .insert(key, value)
                    .map_err(|e| {
                        Error::Io(std::io::Error::other(format!(
                            "compact: insert to tmp: {e}"
                        )))
                    })?;
            }

            tmp_inner.db.persist(PersistMode::SyncAll).map_err(|e| {
                Error::Io(std::io::Error::other(format!("compact: flush tmp db: {e}")))
            })?;

            // tmp_inner (db + all keyspace handles) dropped here →
            // ALL file locks on tmp_path are released.
        }

        tracing::info!("compact(Fjall): temp database written and closed");

        // ── 6–8. Atomic swap under the write lock (Windows-safe) ─────────────
        //
        // Windows constraint: rename(src, dst) returns os error 5 (Access
        // Denied) if any process has open handles inside src OR dst.
        //
        // We need to:
        //   (a) close all handles on self.path  (old FjallInner)
        //   (b) close all handles on tmp_path   (done: step 5 closed them)
        //   (c) rename tmp_path → self.path
        //   (d) open new handles at self.path   (final FjallInner)
        //
        // To achieve (a)+(c)+(d) while holding the exclusive write lock
        // (so no reader ever observes an invalid state), we use
        // `ptr::read` / `ptr::write` to move the old FjallInner out,
        // drop it, perform the rename, and then write the new FjallInner
        // back — all with the lock held and with no intermediate "empty"
        // state visible to any other thread.
        //
        // SAFETY invariants:
        //   1. We hold `guard` (exclusive write lock) throughout this block.
        //      No other thread can access `*guard`.
        //   2. `ptr::read` bitwise-copies the value out, leaving the storage
        //      logically uninitialized. We MUST call `ptr::write` before
        //      `guard` is dropped, otherwise the lock storage is freed with
        //      an invalid value (double-free or similar UB on drop).
        //   3. Every error arm calls `ptr::write` to restore a valid value
        //      before returning, ensuring invariant 2 is upheld.
        let result: Result<()> = unsafe {
            // (a) Move the old FjallInner out — releases all handles on self.path.
            let old_inner = std::ptr::read(&*guard as *const FjallInner);
            drop(old_inner);

            // Helper: write a recovery FjallInner to restore lock validity
            // after an error. Tries tmp_path (may still exist after failure).
            macro_rules! restore_or_panic {
                ($path:expr) => {
                    match Database::builder($path).open().and_then(|db| {
                        // Re-use open_keyspaces by converting its Result.
                        FjallInner::open_keyspaces(db)
                            .map_err(|e| fjall::Error::Io(std::io::Error::other(e.to_string())))
                    }) {
                        Ok(recovery) => {
                            std::ptr::write(&mut *guard as *mut FjallInner, recovery);
                        }
                        Err(_) => {
                            // Cannot restore — process is in an unrecoverable state.
                            // Abort to avoid UB from the uninitialized storage being
                            // dropped by the guard.
                            std::process::abort();
                        }
                    }
                };
            }

            // (c) Surgical filesystem swap — only touch Fjall-owned entries.
            //
            // Rather than hard-coding Fjall's internal filenames (which differ
            // across minor versions: v3.1.x puts *.jnl directly in root while
            // older versions used a journals/ subdirectory), we discover the
            // Fjall-owned entries by reading the compacted tmp database's root.
            // Every entry that exists in tmp_path was created by Fjall and must
            // replace the corresponding stale entry in self.path.
            // Entries owned by StorageEngine (data/, .vanta.lock, etc.) live
            // only in self.path and are NEVER touched by this loop.
            let tmp_entries: Vec<std::ffi::OsString> = match std::fs::read_dir(&tmp_path) {
                Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.file_name())).collect(),
                Err(e) => {
                    restore_or_panic!(&self.path);
                    return Err(Error::Io(std::io::Error::other(format!(
                        "compact: read tmp dir: {e}"
                    ))));
                }
            };

            // Remove stale Fjall entries from self.path (those that the fresh
            // compact database also has, so we know they're Fjall-owned).
            for entry_name in &tmp_entries {
                let old_entry = self.path.join(entry_name);
                if !old_entry.exists() {
                    continue;
                }
                let r = if old_entry.is_dir() {
                    std::fs::remove_dir_all(&old_entry)
                } else {
                    std::fs::remove_file(&old_entry)
                };
                if let Err(e) = r {
                    restore_or_panic!(&self.path);
                    return Err(Error::Io(std::io::Error::other(format!(
                        "compact: remove old entry {:?}: {e}",
                        entry_name
                    ))));
                }
            }

            // Move compacted entries from tmp_path into self.path.
            for entry_name in &tmp_entries {
                let src = tmp_path.join(entry_name);
                let dst = self.path.join(entry_name);
                if let Err(e) = std::fs::rename(&src, &dst) {
                    restore_or_panic!(&self.path);
                    return Err(Error::Io(std::io::Error::other(format!(
                        "compact: move entry {:?}: {e}",
                        entry_name
                    ))));
                }
            }

            // Clean up the (now-empty) temp directory.
            let _ = std::fs::remove_dir_all(&tmp_path);

            // (d) Open final handles at the canonical path and restore lock.
            match Database::builder(&self.path)
                .open()
                .map_err(|e| {
                    Error::Io(std::io::Error::other(format!(
                        "compact: reopen final db: {e}"
                    )))
                })
                .and_then(FjallInner::open_keyspaces)
            {
                Ok(final_inner) => {
                    std::ptr::write(&mut *guard as *mut FjallInner, final_inner);
                    Ok(())
                }
                Err(e) => {
                    // self.path exists (rename succeeded) — try to restore from it.
                    restore_or_panic!(&self.path);
                    Err(e)
                }
            }
        };

        result?;

        tracing::info!(
            live_kv_pairs = live_pairs.len(),
            "compact(Fjall): atomic keyspace rewrite complete — journal purged"
        );

        Ok(true)
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backend::BackendWriteOp;
    use crate::config::Config;
    use std::path::Path;
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
    fn test_fjall_compact_role_present() {
        let (b, _dir) = open_fjall();
        // FIND-343: FjallBackend now implements Compactable via keyspace rewrite.
        assert!(b.as_compactable().is_some());
        assert!(b.capabilities().supports_manual_compaction);
    }

    #[test]
    fn test_fjall_capabilities() {
        let (b, _dir) = open_fjall();
        let caps = b.capabilities();
        assert!(!caps.supports_checkpoint);
        assert!(caps.supports_manual_compaction);
        assert_eq!(caps.kind, crate::backend::BackendKind::Fjall);
    }

    /// FIND-343: verify that compact() physically purges deleted data from
    /// the Fjall journal. After put → delete → compact, the marker bytes
    /// must not appear in any file under the data directory.
    #[test]
    fn test_fjall_compact_purges_deleted_bytes() {
        const MARKER: &[u8] = b"FIND343-COMPACT-PURGE-MARKER-deadbeef";

        let dir = tempdir().unwrap();
        let config = Config::default();
        let path = dir.path().to_str().unwrap();
        let backend = FjallBackend::open(path, &config).unwrap();

        // Write a value containing the marker, then delete it.
        backend
            .put(BackendPartition::Default, b"probe_key", MARKER)
            .unwrap();
        backend
            .delete(BackendPartition::Default, b"probe_key")
            .unwrap();
        backend.flush().unwrap();

        // Compact: atomic keyspace rewrite should eliminate the marker from disk.
        let ran = backend
            .as_compactable()
            .unwrap()
            .compact()
            .expect("compact must succeed");
        assert!(ran, "compact() must return true");

        // Verify the marker is absent from every byte of every file.
        fn scan_for_marker(root: &Path, needle: &[u8]) -> Vec<String> {
            let mut hits = Vec::new();
            let Ok(entries) = std::fs::read_dir(root) else {
                return hits;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    hits.extend(scan_for_marker(&p, needle));
                } else if let Ok(bytes) = std::fs::read(&p) {
                    if bytes.windows(needle.len()).any(|w| w == needle) {
                        hits.push(p.to_string_lossy().into_owned());
                    }
                }
            }
            hits
        }

        let hits = scan_for_marker(dir.path(), MARKER);
        assert!(
            hits.is_empty(),
            "compact() failed to purge marker from disk: {hits:?}"
        );

        // Live data written after the delete must survive compaction.
        backend
            .put(BackendPartition::Default, b"survivor", b"live_value")
            .unwrap();
        let compactable = backend.as_compactable().unwrap();
        compactable.compact().expect("second compact must succeed");
        let val = backend.get(BackendPartition::Default, b"survivor").unwrap();
        assert_eq!(val.as_deref(), Some(b"live_value" as &[u8]));
    }
}
