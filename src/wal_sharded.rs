use crate::error::{Result, VantaError};
use crate::wal::{WalReader, WalRecord, WalWriter};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A write-ahead log operation for a key-value pair.
#[derive(Debug, Clone)]
pub(crate) enum WalOp {
    /// Insert or update a record.
    Put {
        /// Namespace of the record.
        namespace: String,
        /// Key of the record.
        key: String,
        /// Payload of the record.
        payload: String,
    },
    /// Delete a record by namespace and key.
    Delete {
        /// Namespace of the record.
        namespace: String,
        /// Key of the record.
        key: String,
    },
}

/// A sharded write-ahead log that distributes writes across multiple WAL files.
pub(crate) struct ShardedWal {
    shards: Vec<Arc<Mutex<WalWriter>>>,
    num_shards: usize,
    base_path: PathBuf,
    sync_mode: crate::config::SyncMode,
    next_shard: AtomicUsize,
}

impl ShardedWal {
    /// Create a new `ShardedWal` with the given base path, shard count, and sync mode.
    pub fn new(
        base_path: &Path,
        num_shards: usize,
        sync_mode: crate::config::SyncMode,
    ) -> Result<Self> {
        let num_shards = num_shards.max(1);
        let mut shards = Vec::with_capacity(num_shards);

        for i in 0..num_shards {
            let shard_path = if num_shards > 1 {
                let dir = base_path.parent().unwrap_or(Path::new("."));
                let stem = base_path.file_stem().unwrap_or_default().to_string_lossy();
                let ext = base_path
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let shard_name = format!("{}.shard{}{}", stem, i, ext);
                dir.join(shard_name)
            } else {
                base_path.to_path_buf()
            };
            let writer = WalWriter::open(&shard_path, sync_mode)?;
            shards.push(Arc::new(Mutex::new(writer)));
        }

        Ok(Self {
            shards,
            num_shards,
            base_path: base_path.to_path_buf(),
            sync_mode,
            next_shard: AtomicUsize::new(0),
        })
    }

    /// Compute the shard index for a given key using a hash function.
    fn shard_index(&self, key: &str) -> usize {
        if self.num_shards <= 1 {
            return 0;
        }
        let hash: u64 = key
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        (hash as usize) % self.num_shards
    }

    /// Write a record to the shard determined by the key.
    pub fn write(&self, key: &str, record: &WalRecord) -> Result<()> {
        let idx = self.shard_index(key);
        self.shards[idx].lock().append(record)
    }

    /// Append a record using round-robin shard distribution.
    /// Used when no specific key is available for shard routing.
    pub fn append(&self, record: &WalRecord) -> Result<()> {
        let idx = self.next_shard.fetch_add(1, Ordering::Relaxed) % self.num_shards;
        self.shards[idx].lock().append(record)
    }

    /// Append multiple records across shards, batching per shard to reduce I/O.
    pub fn batch_append(&self, records: &[WalRecord]) -> Result<()> {
        if records.is_empty() || self.num_shards == 0 {
            return Ok(());
        }
        let mut batches: Vec<Vec<WalRecord>> = (0..self.num_shards).map(|_| Vec::new()).collect();
        for record in records {
            let idx = self.next_shard.fetch_add(1, Ordering::Relaxed) % self.num_shards;
            batches[idx].push(record.clone());
        }
        for (idx, batch) in batches.iter().enumerate() {
            if !batch.is_empty() {
                self.shards[idx].lock().batch_append(batch)?;
            }
        }
        Ok(())
    }

    /// Replay all records across all shards, skipping those at or below
    /// `checkpoint_seq` per shard.
    pub fn recover(
        &self,
        checkpoint_seq: u64,
        mut f: impl FnMut(WalRecord) -> Result<()>,
    ) -> Result<()> {
        for (i, shard) in self.shards.iter().enumerate() {
            let path = {
                let guard = shard.lock();
                guard.path().to_path_buf()
            };
            if !path.exists() {
                continue;
            }
            let mut reader = WalReader::open(&path).map_err(|e| {
                VantaError::WalError(format!("Failed to open shard {} for recovery: {}", i, e))
            })?;
            let mut current_seq = 0u64;
            while let Some(record) = reader.next_record()? {
                current_seq += 1;
                if current_seq <= checkpoint_seq {
                    continue;
                }
                f(record)?;
            }
        }
        Ok(())
    }

    /// Flush (sync) all shards to disk.
    pub fn flush_all(&self) -> Result<()> {
        for shard in &self.shards {
            shard.lock().sync()?;
        }
        Ok(())
    }

    /// Flush (sync) only the shard determined by the key.
    pub fn flush_shard(&self, key: &str) -> Result<()> {
        let idx = self.shard_index(key);
        self.shards[idx].lock().sync()
    }

    /// Rotate all shards (flush, archive, and start fresh WAL files).
    pub fn rotate_all(&self) -> Result<()> {
        for shard in &self.shards {
            let replacement = {
                let mut guard = shard.lock();
                let path = guard.path().to_path_buf();
                guard.sync()?;
                WalWriter::open(&path, self.sync_mode)?
            };
            *shard.lock() = replacement;
        }
        Ok(())
    }

    /// Return a reference to the shard list.
    pub fn shards(&self) -> &[Arc<Mutex<WalWriter>>] {
        &self.shards
    }

    /// Return the number of configured shards.
    pub fn num_shards(&self) -> usize {
        self.num_shards
    }

    /// Return the number of shard entries.
    pub fn len(&self) -> usize {
        self.shards.len()
    }

    /// Return the total number of records across all shards.
    pub fn total_record_count(&self) -> u64 {
        self.shards.iter().map(|s| s.lock().record_count()).sum()
    }
}

impl std::fmt::Debug for ShardedWal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShardedWal")
            .field("num_shards", &self.num_shards)
            .field("base_path", &self.base_path)
            .finish()
    }
}
