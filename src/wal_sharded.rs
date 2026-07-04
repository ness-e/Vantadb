#![allow(dead_code)]
use crate::error::Result;
use crate::wal::WalWriter;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
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
    shards: Vec<Arc<Mutex<Option<WalWriter>>>>,
    num_shards: usize,
    base_path: PathBuf,
    sync_mode: crate::config::SyncMode,
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
            shards.push(Arc::new(Mutex::new(Some(writer))));
        }

        Ok(Self {
            shards,
            num_shards,
            base_path: base_path.to_path_buf(),
            sync_mode,
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
    pub fn write(&self, key: &str, record: &crate::wal::WalRecord) -> Result<()> {
        let idx = self.shard_index(key);
        let mut guard = self.shards[idx].lock();
        if let Some(ref mut writer) = *guard {
            writer.append(record)?;
        }
        Ok(())
    }

    /// Flush (sync) all shards to disk.
    pub fn flush_all(&self) -> Result<()> {
        for shard in &self.shards {
            let mut guard = shard.lock();
            if let Some(ref mut writer) = *guard {
                writer.sync()?;
            }
        }
        Ok(())
    }

    /// Flush (sync) only the shard determined by the key.
    pub fn flush_shard(&self, key: &str) -> Result<()> {
        let idx = self.shard_index(key);
        let mut guard = self.shards[idx].lock();
        if let Some(ref mut writer) = *guard {
            writer.sync()?;
        }
        Ok(())
    }

    /// Return a reference to the shard list.
    pub fn shards(&self) -> &[Arc<Mutex<Option<WalWriter>>>] {
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
}
