use crate::error::{Result, VantaError};
use crate::wal::{WalReader, WalRecord, WalWriter};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A sharded write-ahead log that distributes writes across multiple WAL files.
#[allow(dead_code)]
pub(crate) struct ShardedWal {
    shards: Vec<Arc<Mutex<WalWriter>>>,
    num_shards: usize,
    base_path: PathBuf,
    sync_mode: crate::config::SyncMode,
    next_shard: AtomicUsize,
    wal_buffer_size: usize,
    flush_threshold: Option<usize>,
}

impl ShardedWal {
    /// Create a new `ShardedWal` with the given base path, shard count, and sync mode.
    pub fn new(
        base_path: &Path,
        num_shards: usize,
        sync_mode: crate::config::SyncMode,
    ) -> Result<Self> {
        Self::new_with_buffer(base_path, num_shards, sync_mode, 64 * 1024, None)
    }

    /// Create a new `ShardedWal` with configurable buffer size and flush threshold.
    pub fn new_with_buffer(
        base_path: &Path,
        num_shards: usize,
        sync_mode: crate::config::SyncMode,
        wal_buffer_size: usize,
        flush_threshold: Option<usize>,
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
            let writer = WalWriter::open_with_buffer(
                &shard_path,
                sync_mode,
                wal_buffer_size,
                flush_threshold,
            )?;
            shards.push(Arc::new(Mutex::new(writer)));
        }

        Ok(Self {
            shards,
            num_shards,
            base_path: base_path.to_path_buf(),
            sync_mode,
            next_shard: AtomicUsize::new(0),
            wal_buffer_size,
            flush_threshold,
        })
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
                VantaError::wal_error(format!("Failed to open shard {} for recovery: {}", i, e))
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
