//! RocksDB-backed implementation of `StorageBackend`.
//!
//! This adapter encapsulates all direct interaction with the `rocksdb` crate.
//! No RocksDB types (DB, ColumnFamily handles, iterators, options) should leak
//! outside this module.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use crate::storage::EngineConfig;
use rocksdb::checkpoint::Checkpoint;
use rocksdb::{Direction, FlushOptions, IteratorMode, Options, WriteBatch, DB};
use std::path::Path;
use tracing::{info, warn};

/// RocksDB adapter implementing `StorageBackend`.
///
/// Owns the `rocksdb::DB` instance and all column family configuration.
/// Created exclusively through `RocksDbBackend::open`.
pub(crate) struct RocksDbBackend {
    db: DB,
}

impl RocksDbBackend {
    /// Open a RocksDB database at `path` with the given configuration.
    ///
    /// Preserves the original tuning: bloom filters, LRU cache sizing,
    /// memtable budgets, LZ4 compression, mmap access for low-RAM profiles,
    /// and per-CF block-based table options.
    pub(crate) fn open(path: &str, config: &EngineConfig) -> Result<Self> {
        let caps = crate::hardware::HardwareCapabilities::global();

        // Memory limit resolution priority:
        // 1. Explicit config.memory_limit (from Python SDK constructor)
        // 2. Hardware detection (from HardwareCapabilities)
        let effective_memory = config.memory_limit.unwrap_or(caps.total_memory);

        let mut opts = Options::default();
        opts.create_if_missing(!config.read_only);
        opts.create_missing_column_families(true);
        opts.set_max_background_jobs(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Adaptive Mode: Dynamic RocksDB tuning based on effective RAM
        let mut bopts = rocksdb::BlockBasedOptions::default();
        bopts.set_bloom_filter(10.0, false);
        // Performance Booster: Force retention of L0 indexes and bloom filters permanently
        bopts.set_cache_index_and_filter_blocks(true);
        bopts.set_pin_l0_filter_and_index_blocks_in_cache(true);

        // Standard Bopts for cold layers (no L0 pinning)
        let mut cold_bopts = rocksdb::BlockBasedOptions::default();
        cold_bopts.set_bloom_filter(10.0, false);

        // OOM Guard: Cap LRU Cache and WriteBuffer to ~60% of effective capacity
        let rocksdb_budget = (effective_memory as f64 * 0.60) as usize;
        let cache_size = (rocksdb_budget as f64 * 0.75) as usize; // 75% focus on block cache
        let write_buffer_total = rocksdb_budget - cache_size; // 25% for memtables

        let write_buffer_size = (write_buffer_total / 2).clamp(8 * 1024 * 1024, 128 * 1024 * 1024);

        opts.set_write_buffer_size(write_buffer_size);
        opts.set_max_write_buffer_number(2);

        let cache = rocksdb::Cache::new_lru_cache(cache_size);
        bopts.set_block_cache(&cache);
        cold_bopts.set_block_cache(&cache);

        info!(
            rocksdb_budget_mb = rocksdb_budget / 1024 / 1024,
            cache_mb = cache_size / 1024 / 1024,
            memtable_mb = write_buffer_size / 1024 / 1024,
            "RocksDB memory configured"
        );

        opts.set_block_based_table_factory(&bopts);

        if caps.profile == crate::hardware::HardwareProfile::LowResource
            || effective_memory < 16 * 1024 * 1024 * 1024
        {
            opts.set_allow_mmap_reads(true);
            opts.set_allow_mmap_writes(true);
            warn!(
                effective_memory_gb = effective_memory / 1024 / 1024 / 1024,
                "RAM < 16GB — MMap access forced (Resource Governance)"
            );
        }

        // Fast layers (LZ4)
        let mut default_opts = opts.clone();
        default_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        default_opts.set_block_based_table_factory(&bopts);

        // tombstone_storage: Unpinned bloom for efficiency
        let mut shadow_opts = rocksdb::Options::default();
        shadow_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        shadow_opts.set_block_based_table_factory(&cold_bopts);

        let mut archive_opts = rocksdb::Options::default();
        archive_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        archive_opts.set_block_based_table_factory(&bopts);

        let mut tombstone_opts = default_opts.clone();
        tombstone_opts.set_block_based_table_factory(&cold_bopts);

        let mut namespace_index_opts = default_opts.clone();
        namespace_index_opts.set_block_based_table_factory(&bopts);

        let mut payload_index_opts = default_opts.clone();
        payload_index_opts.set_block_based_table_factory(&bopts);

        let mut text_index_opts = default_opts.clone();
        text_index_opts.set_block_based_table_factory(&bopts);

        let mut internal_metadata_opts = default_opts.clone();
        internal_metadata_opts.set_block_based_table_factory(&cold_bopts);

        let cf_descriptors = vec![
            rocksdb::ColumnFamilyDescriptor::new("default", default_opts),
            rocksdb::ColumnFamilyDescriptor::new("tombstone_storage", shadow_opts),
            rocksdb::ColumnFamilyDescriptor::new("compressed_archive", archive_opts),
            rocksdb::ColumnFamilyDescriptor::new("tombstones", tombstone_opts),
            rocksdb::ColumnFamilyDescriptor::new("namespace_index", namespace_index_opts),
            rocksdb::ColumnFamilyDescriptor::new("payload_index", payload_index_opts),
            rocksdb::ColumnFamilyDescriptor::new("text_index", text_index_opts),
            rocksdb::ColumnFamilyDescriptor::new("internal_metadata", internal_metadata_opts),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(Self { db })
    }

    /// Helper: resolve a `BackendPartition` to its RocksDB column family handle.
    fn cf_handle(&self, partition: BackendPartition) -> Result<&rocksdb::ColumnFamily> {
        self.db.cf_handle(partition.cf_name()).ok_or_else(|| {
            VantaError::Execution(format!("Column family '{}' not found", partition.cf_name()))
        })
    }
}

impl StorageBackend for RocksDbBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        if partition == BackendPartition::Default {
            self.db
                .put(key, value)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .put_cf(&cf, key, value)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if partition == BackendPartition::Default {
            self.db
                .get(key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .get_cf(&cf, key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        if partition == BackendPartition::Default {
            self.db
                .delete(key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .delete_cf(&cf, key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        let mut batch = WriteBatch::default();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    if partition == BackendPartition::Default {
                        batch.put(&key, &value);
                    } else {
                        let cf = self.cf_handle(partition)?;
                        batch.put_cf(&cf, &key, &value);
                    }
                }
                BackendWriteOp::Delete { partition, key } => {
                    if partition == BackendPartition::Default {
                        batch.delete(&key);
                    } else {
                        let cf = self.cf_handle(partition)?;
                        batch.delete_cf(&cf, &key);
                    }
                }
            }
        }
        self.db
            .write(batch)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let cf = self.cf_handle(partition)?;
        let mut result = Vec::new();
        for item in self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start) {
            let (k, v) =
                item.map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;
            result.push((k.to_vec(), v.to_vec()));
        }
        Ok(result)
    }

    fn scan_prefix(
        &self,
        partition: BackendPartition,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let cf = self.cf_handle(partition)?;
        let mut result = Vec::new();
        for item in self
            .db
            .iterator_cf(&cf, IteratorMode::From(prefix, Direction::Forward))
        {
            let (k, v) =
                item.map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;
            if !k.starts_with(prefix) {
                break;
            }
            result.push((k.to_vec(), v.to_vec()));
        }
        Ok(result)
    }

    fn flush(&self) -> Result<()> {
        let mut flush_opt = FlushOptions::default();
        flush_opt.set_wait(true);
        self.db
            .flush_opt(&flush_opt)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn checkpoint(&self, path: &Path) -> Result<()> {
        let cp = Checkpoint::new(&self.db).map_err(|e| {
            VantaError::IoError(std::io::Error::other(format!(
                "Error creating Checkpoint initializer: {}",
                e
            )))
        })?;

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        cp.create_checkpoint(path).map_err(|e| {
            VantaError::IoError(std::io::Error::other(format!(
                "Error writing checkpoint: {}",
                e
            )))
        })
    }

    fn compact(&self) {
        let mut c_opts = rocksdb::CompactOptions::default();
        c_opts.set_exclusive_manual_compaction(false);
        self.db
            .compact_range_opt(None::<&[u8]>, None::<&[u8]>, &c_opts);
    }

    fn capabilities(&self) -> crate::backend::BackendCapabilities {
        crate::backend::BackendCapabilities {
            supports_checkpoint: true,
            supports_manual_compaction: true,
            kind: crate::backend::BackendKind::RocksDb,
        }
    }
}
