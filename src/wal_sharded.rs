#![allow(dead_code)]
use crate::error::Result;
use crate::wal::WalWriter;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) enum WalOp {
    Put {
        namespace: String,
        key: String,
        payload: String,
    },
    Delete {
        namespace: String,
        key: String,
    },
}

pub(crate) struct ShardedWal {
    shards: Vec<Arc<Mutex<Option<WalWriter>>>>,
    num_shards: usize,
    base_path: PathBuf,
    sync_mode: crate::config::SyncMode,
}

impl ShardedWal {
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

    fn shard_index(&self, key: &str) -> usize {
        if self.num_shards <= 1 {
            return 0;
        }
        let hash: u64 = key
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        (hash as usize) % self.num_shards
    }

    pub fn write(&self, key: &str, record: &crate::wal::WalRecord) -> Result<()> {
        let idx = self.shard_index(key);
        let mut guard = self.shards[idx].lock();
        if let Some(ref mut writer) = *guard {
            writer.append(record)?;
        }
        Ok(())
    }

    pub fn flush_all(&self) -> Result<()> {
        for shard in &self.shards {
            let mut guard = shard.lock();
            if let Some(ref mut writer) = *guard {
                writer.sync()?;
            }
        }
        Ok(())
    }

    pub fn flush_shard(&self, key: &str) -> Result<()> {
        let idx = self.shard_index(key);
        let mut guard = self.shards[idx].lock();
        if let Some(ref mut writer) = *guard {
            writer.sync()?;
        }
        Ok(())
    }

    pub fn shards(&self) -> &[Arc<Mutex<Option<WalWriter>>>] {
        &self.shards
    }

    pub fn num_shards(&self) -> usize {
        self.num_shards
    }

    pub fn len(&self) -> usize {
        self.shards.len()
    }
}
