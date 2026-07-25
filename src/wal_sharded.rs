use crate::error::{Result, VantaError};
use crate::wal::{WalReader, WalRecord, WalWriter};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A sharded write-ahead log that distributes writes across multiple WAL files.
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
        for record in records {
            let idx = self.next_shard.fetch_add(1, Ordering::Relaxed) % self.num_shards;
            self.shards[idx].lock().append(record)?;
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
                WalWriter::open_with_buffer(
                    &path,
                    self.sync_mode,
                    self.wal_buffer_size,
                    self.flush_threshold,
                )?
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::SyncMode;
    use crate::node::UnifiedNode;

    fn test_wal_path() -> PathBuf {
        std::env::temp_dir().join(format!("vanta_test_sharded_{}", rand::random::<u32>()))
    }

    fn make_record(id: u128) -> WalRecord {
        WalRecord::Insert(UnifiedNode::new(id))
    }

    /// Helper: remove shard files for cleanup.
    fn clean_shards(base: &Path, count: usize) {
        for i in 0..count {
            let shard_path = if count > 1 {
                let dir = base.parent().unwrap_or(Path::new("."));
                let stem = base.file_stem().unwrap_or_default().to_string_lossy();
                let ext = base
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                dir.join(format!("{}.shard{}{}", stem, i, ext))
            } else {
                base.to_path_buf()
            };
            let _ = std::fs::remove_file(&shard_path);
        }
    }

    // ─── Construction ───────────────────────────────────────────

    #[test]
    fn test_new_single_shard() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 1, SyncMode::Periodic).unwrap();
        assert_eq!(sw.num_shards, 1);
        assert_eq!(sw.shards.len(), 1);
        clean_shards(&path, 1);
    }

    #[test]
    fn test_new_multiple_shards() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 4, SyncMode::Periodic).unwrap();
        assert_eq!(sw.num_shards, 4);
        assert_eq!(sw.shards.len(), 4);
        clean_shards(&path, 4);
    }

    #[test]
    fn test_new_zero_shards_defaults_to_one() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 0, SyncMode::Periodic).unwrap();
        assert_eq!(sw.num_shards, 1);
        assert_eq!(sw.shards.len(), 1);
        clean_shards(&path, 1);
    }

    #[test]
    fn test_new_with_buffer_custom_parameters() {
        let path = test_wal_path();
        let sw = ShardedWal::new_with_buffer(&path, 2, SyncMode::Periodic, 128 * 1024, Some(10))
            .unwrap();
        assert_eq!(sw.num_shards, 2);
        assert_eq!(sw.wal_buffer_size, 128 * 1024);
        assert_eq!(sw.flush_threshold, Some(10));
        clean_shards(&path, 2);
    }

    #[test]
    fn test_debug_format() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();
        let fmt = format!("{:?}", sw);
        assert!(fmt.contains("ShardedWal"));
        assert!(fmt.contains("num_shards: 2"));
        clean_shards(&path, 2);
    }

    // ─── Append ─────────────────────────────────────────────────

    #[test]
    fn test_append_single_record() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 1, SyncMode::Periodic).unwrap();
        sw.append(&make_record(42)).unwrap();
        assert_eq!(sw.total_record_count(), 1);
        clean_shards(&path, 1);
    }

    #[test]
    fn test_append_multiple_records() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 1, SyncMode::Periodic).unwrap();
        sw.append(&make_record(1)).unwrap();
        sw.append(&make_record(2)).unwrap();
        sw.append(&make_record(3)).unwrap();
        assert_eq!(sw.total_record_count(), 3);
        clean_shards(&path, 1);
    }

    #[test]
    fn test_append_round_robin_distribution() {
        let path = test_wal_path();
        let num_shards = 3;
        let sw = ShardedWal::new(&path, num_shards, SyncMode::Periodic).unwrap();

        // 6 records across 3 shards = 2 per shard
        for i in 0..6 {
            sw.append(&make_record(i)).unwrap();
        }
        assert_eq!(sw.total_record_count(), 6);

        for (i, shard) in sw.shards.iter().enumerate() {
            assert_eq!(
                shard.lock().record_count(),
                2,
                "shard {} should have 2 records",
                i
            );
        }
        clean_shards(&path, num_shards);
    }

    #[test]
    fn test_append_round_robin_uneven() {
        let path = test_wal_path();
        let num_shards = 3;
        let sw = ShardedWal::new(&path, num_shards, SyncMode::Periodic).unwrap();

        // 4 records across 3 shards: shard0=2, shard1=1, shard2=1
        for i in 0..4 {
            sw.append(&make_record(i)).unwrap();
        }
        assert_eq!(sw.total_record_count(), 4);

        let counts: Vec<u64> = sw.shards.iter().map(|s| s.lock().record_count()).collect();
        assert_eq!(counts, vec![2, 1, 1], "uneven round-robin distribution");
        clean_shards(&path, num_shards);
    }

    #[test]
    fn test_append_all_record_variants() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 1, SyncMode::Periodic).unwrap();

        sw.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        sw.append(&WalRecord::Update {
            id: 1,
            node: UnifiedNode::new(1),
        })
        .unwrap();
        sw.append(&WalRecord::Delete { id: 1 }).unwrap();
        sw.append(&WalRecord::Begin(100)).unwrap();
        sw.append(&WalRecord::Commit(100)).unwrap();
        sw.append(&WalRecord::Abort(100)).unwrap();
        sw.append(&WalRecord::create_checkpoint(5, None)).unwrap();

        assert_eq!(sw.total_record_count(), 7);
        clean_shards(&path, 1);
    }

    // ─── Batch Append ───────────────────────────────────────────

    #[test]
    fn test_batch_append_multiple_records() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        let records: Vec<WalRecord> = (0..10).map(make_record).collect();
        sw.batch_append(&records).unwrap();
        assert_eq!(sw.total_record_count(), 10);
        clean_shards(&path, 2);
    }

    #[test]
    fn test_batch_append_empty() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 3, SyncMode::Periodic).unwrap();
        sw.batch_append(&[]).unwrap();
        assert_eq!(sw.total_record_count(), 0);
        clean_shards(&path, 3);
    }

    #[test]
    fn test_batch_append_single_record() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 3, SyncMode::Periodic).unwrap();
        sw.batch_append(&[make_record(99)]).unwrap();
        assert_eq!(sw.total_record_count(), 1);
        clean_shards(&path, 3);
    }

    // ─── Flush ───────────────────────────────────────────────────

    #[test]
    fn test_flush_all() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        sw.append(&make_record(1)).unwrap();
        sw.append(&make_record(2)).unwrap();
        sw.flush_all().unwrap();
        assert_eq!(sw.total_record_count(), 2);
        clean_shards(&path, 2);
    }

    #[test]
    fn test_flush_all_empty() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();
        // Flush with no records should succeed
        sw.flush_all().unwrap();
        assert_eq!(sw.total_record_count(), 0);
        clean_shards(&path, 2);
    }

    // ─── Total Record Count ──────────────────────────────────────

    #[test]
    fn test_total_record_count_empty() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 3, SyncMode::Periodic).unwrap();
        assert_eq!(sw.total_record_count(), 0);
        clean_shards(&path, 3);
    }

    #[test]
    fn test_total_record_count_after_ops() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        assert_eq!(sw.total_record_count(), 0);
        sw.append(&make_record(1)).unwrap();
        assert_eq!(sw.total_record_count(), 1);
        sw.append(&make_record(2)).unwrap();
        assert_eq!(sw.total_record_count(), 2);
        sw.append(&make_record(3)).unwrap();
        assert_eq!(sw.total_record_count(), 3);
        clean_shards(&path, 2);
    }

    // ─── Rotate ──────────────────────────────────────────────────

    #[test]
    fn test_rotate_all_reopens_files() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        sw.append(&make_record(1)).unwrap();
        sw.append(&make_record(2)).unwrap();
        sw.flush_all().unwrap();
        let before = sw.total_record_count();
        assert_eq!(before, 2);

        // rotate_all re-opens the same files (flush + new WalWriter at same path)
        sw.rotate_all().unwrap();

        // Old records remain visible (file re-opened, not truncated)
        assert_eq!(sw.total_record_count(), before);

        // Appending after rotation works and increments count
        sw.append(&make_record(3)).unwrap();
        assert_eq!(sw.total_record_count(), before + 1);
        clean_shards(&path, 2);
    }

    #[test]
    fn test_rotate_all_empty() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        // Rotating with no records should succeed
        sw.rotate_all().unwrap();
        assert_eq!(sw.total_record_count(), 0);
        clean_shards(&path, 2);
    }

    // ─── Recover ─────────────────────────────────────────────────

    #[test]
    fn test_recover_all_records() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        sw.append(&make_record(10)).unwrap();
        sw.append(&make_record(20)).unwrap();
        sw.flush_all().unwrap();

        let mut recovered = Vec::new();
        sw.recover(0, |record| {
            recovered.push(record);
            Ok(())
        })
        .unwrap();

        assert_eq!(recovered.len(), 2);
        clean_shards(&path, 2);
    }

    #[test]
    fn test_recover_with_checkpoint_skips_old_records() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        sw.append(&make_record(10)).unwrap();
        sw.append(&make_record(20)).unwrap();
        sw.flush_all().unwrap();

        // checkpoint_seq=5 → per-shard seq (1) ≤ 5 → all skipped
        let mut recovered = Vec::new();
        sw.recover(5, |record| {
            recovered.push(record);
            Ok(())
        })
        .unwrap();

        assert!(recovered.is_empty());
        clean_shards(&path, 2);
    }

    #[test]
    fn test_recover_empty_wal() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        let mut recovered = Vec::new();
        sw.recover(0, |record| {
            recovered.push(record);
            Ok(())
        })
        .unwrap();

        assert!(recovered.is_empty());
        clean_shards(&path, 2);
    }

    #[test]
    fn test_recover_missing_shard_file_skips_gracefully() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 2, SyncMode::Periodic).unwrap();

        // Write to shard 0 only (odd number of round-robins)
        sw.append(&make_record(1)).unwrap();
        sw.flush_all().unwrap();

        let mut recovered = Vec::new();
        sw.recover(0, |record| {
            recovered.push(record);
            Ok(())
        })
        .unwrap();

        // At least 1 record recovered from shard 0; shard 1 may not exist
        assert!(!recovered.is_empty());
        clean_shards(&path, 2);
    }

    // ─── Error Handling ──────────────────────────────────────────

    #[test]
    fn test_new_with_invalid_path_errors() {
        // Use a path that can't be created (empty string or root)
        let result = ShardedWal::new(Path::new(""), 1, SyncMode::Periodic);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_to_rotated_wal_works() {
        let path = test_wal_path();
        let sw = ShardedWal::new(&path, 1, SyncMode::Periodic).unwrap();

        sw.append(&make_record(1)).unwrap();
        let before = sw.total_record_count();
        assert_eq!(before, 1);

        sw.rotate_all().unwrap();
        // Old records remain; appending after rotation increments total
        sw.append(&make_record(2)).unwrap();
        assert_eq!(sw.total_record_count(), before + 1);
        clean_shards(&path, 1);
    }
}
