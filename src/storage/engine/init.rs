//! StorageEngine initialization: opening, backend setup, index loading, WAL recovery.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use web_time::Instant;

use crate::backend::{BackendPartition, StorageBackend};
#[cfg(feature = "fjall")]
use crate::backends::fjall_backend::FjallBackend;
use crate::backends::in_memory::InMemoryBackend;
#[cfg(feature = "rocksdb")]
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BackendKind, FLAG_TOMBSTONE, GIB, MIB};
use crate::storage::ops;
#[cfg(unix)]
use crate::storage::vfile::install_sigbus_handler;
use crate::storage::vfile::VantaFile;

impl StorageEngine {
    /// Open with default configuration (backward-compatible).
    pub fn open(path: &str) -> Result<Self> {
        Self::open_with_config(path, None)
    }

    /// Open with explicit configuration for memory budgets and mode overrides.
    pub fn open_with_config(path: &str, config: Option<VantaConfig>) -> Result<Self> {
        let startup_started = Instant::now();
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();
        let effective_memory = config.memory_limit.unwrap_or(caps.total_memory);

        let (lock_file, backend, data_dir) = Self::init_storage(path, &config)?;

        let (hnsw, vector_store, wal_writer, wal_replay_ms, wal_records_replayed) =
            if matches!(config.backend_kind, BackendKind::InMemory) {
                let hnsw = CPIndex::new();
                let vector_store = VantaFile::create_in_memory(64 * MIB);
                let wal_writer = None;
                (hnsw, vector_store, wal_writer, 0u64, 0u64)
            } else {
                let (mut hnsw, mut vector_store) =
                    Self::init_indexes(&data_dir, &config, caps, effective_memory)?;
                let (wal_replay_ms, wal_records_replayed) = Self::recover_state(
                    &data_dir,
                    &config,
                    backend.as_ref(),
                    &mut hnsw,
                    &mut vector_store,
                )?;
                let wal_writer = crate::storage::wal::init_wal(&data_dir, &config)?;
                (
                    hnsw,
                    vector_store,
                    wal_writer,
                    wal_replay_ms,
                    wal_records_replayed,
                )
            };

        crate::metrics::record_startup(
            startup_started.elapsed().as_millis() as u64,
            wal_replay_ms,
            wal_records_replayed,
        );

        let estimated_hnsw_bytes = hnsw.estimate_memory_bytes() as u64;
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            estimated_hnsw_bytes,
            crate::storage::vfile::engine_mmap_resident_bytes(&hnsw, &vector_store),
            0,
            0,
        );

        if hnsw.nodes.len() > 10_000 && estimated_hnsw_bytes > effective_memory / 2 {
            tracing::warn!(
                hnsw_nodes = hnsw.nodes.len(),
                estimated_mb = estimated_hnsw_bytes / MIB,
                effective_mb = effective_memory / MIB,
                "HNSW index exceeds 50% of memory budget",
            );
        }

        let cardinality_stats = Self::initialize_cardinality_stats(backend.as_ref());

        let engine = Self {
            config: config.clone(),
            read_only: config.read_only,
            hnsw: arc_swap::ArcSwap::from_pointee(hnsw),
            insert_lock: parking_lot::Mutex::new(()),
            volatile_cache: parking_lot::RwLock::new(std::collections::HashMap::new()),
            last_query_timestamp: std::sync::atomic::AtomicU64::new(0),
            emergency_maintenance_trigger: std::sync::atomic::AtomicBool::new(false),
            data_dir,
            vector_store: parking_lot::RwLock::new(vector_store),
            wal: wal_writer.map(std::sync::Arc::new),
            _lock_file: lock_file,
            text_stats_cache: parking_lot::RwLock::new(std::collections::HashMap::new()),
            text_ns_cache: parking_lot::RwLock::new(std::collections::HashMap::new()),
            cardinality_stats: parking_lot::RwLock::new(cardinality_stats),
            backend,
            memory_governor: Some(std::sync::Arc::new(
                crate::memory_governor::MemoryGovernor::new(&config),
            )),
            quantization_governor: std::sync::Arc::new(
                crate::vector::governor::QuantizationGovernor::new(
                    crate::vector::governor::QuantizationConfig::default(),
                ),
            ),
            edge_index: Some(std::sync::Arc::new(crate::edge_index::EdgeIndex::new())),
            scalar_index: Some(std::sync::Arc::new(crate::scalar_index::ScalarIndex::new())),
        };
        Ok(engine)
    }

    fn init_storage(
        path: &str,
        config: &VantaConfig,
    ) -> Result<(Option<File>, Arc<dyn StorageBackend>, PathBuf)> {
        ops::prevent_path_traversal(path)?;
        let base_path = PathBuf::from(path);

        if matches!(config.backend_kind, BackendKind::InMemory) {
            let backend: Arc<dyn StorageBackend> = Arc::new(InMemoryBackend::new());
            return Ok((None, backend, PathBuf::new()));
        }

        if config.read_only && !base_path.exists() {
            return Err(VantaError::NotFound {
                kind: "database_path".into(),
                id: base_path.display().to_string(),
            });
        }
        let lock_file = {
            let lock_path = base_path.join(".vanta.lock");
            if !config.read_only {
                std::fs::create_dir_all(&base_path).map_err(VantaError::IoError)?;
            }

            let file_result = OpenOptions::new()
                .read(true)
                .write(!config.read_only)
                .create(!config.read_only)
                .open(&lock_path);

            let file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    if config.read_only {
                        return Err(VantaError::NotFound {
                            kind: "lock_file".into(),
                            id: base_path.join(".vanta.lock").display().to_string(),
                        });
                    } else {
                        return Err(VantaError::IoError(e));
                    }
                }
            };

            let mut delay = std::time::Duration::from_millis(5);
            let total_limit = std::time::Duration::from_millis(config.file_lock_timeout_ms);
            let start_time = std::time::Instant::now();
            let mut acquired = false;

            while start_time.elapsed() < total_limit {
                let lock_res: std::io::Result<()> = if config.read_only {
                    #[cfg(feature = "fs2")]
                    {
                        fs2::FileExt::try_lock_shared(&file)
                    }
                    #[cfg(not(feature = "fs2"))]
                    {
                        Ok(())
                    }
                } else {
                    #[cfg(feature = "fs2")]
                    {
                        fs2::FileExt::try_lock_exclusive(&file)
                    }
                    #[cfg(not(feature = "fs2"))]
                    {
                        Ok(())
                    }
                };

                if lock_res.is_ok() {
                    acquired = true;
                    break;
                }

                std::thread::sleep(delay);
                delay = std::cmp::min(delay * 2, std::time::Duration::from_millis(100));
            }

            if !acquired {
                let msg = if config.read_only {
                    format!(
                        "Database at '{}' is locked exclusively by another process (writer). \
                         Cannot acquire shared read-only lock within timeout.",
                        base_path.display()
                    )
                } else {
                    format!(
                        "Database at '{}' is locked by another process. \
                         Cannot acquire exclusive writer lock within timeout.",
                        base_path.display()
                    )
                };
                return Err(VantaError::DatabaseBusy(msg));
            }

            Some(file)
        };

        #[cfg(unix)]
        {
            if let Err(e) = install_sigbus_handler() {
                tracing::warn!("Failed to install SIGBUS handler: {}", e);
            }
        }

        if !config.read_only {
            crate::schema::load_or_create_schema(&base_path)?;
            info!(
                "Storage schema: version={} flags={}",
                crate::schema::CURRENT_SCHEMA_VERSION,
                0
            );
        } else {
            crate::schema::check_schema_compatibility(&base_path)?;
        }

        let backend: Arc<dyn StorageBackend> = match config.backend_kind {
            #[cfg(feature = "rocksdb")]
            BackendKind::RocksDb => Arc::new(RocksDbBackend::open(path, config)?),
            #[cfg(not(feature = "rocksdb"))]
            BackendKind::RocksDb => {
                return Err(VantaError::ValidationError {
                    field: "backend_feature".into(),
                    reason: "RocksDB backend requires the 'rocksdb' feature".into(),
                })
            }
            #[cfg(feature = "fjall")]
            BackendKind::Fjall => Arc::new(FjallBackend::open(path, config)?),
            #[cfg(not(feature = "fjall"))]
            BackendKind::Fjall => {
                return Err(VantaError::ValidationError {
                    field: "backend_feature".into(),
                    reason: "Fjall backend requires the 'fjall' feature".into(),
                })
            }
            BackendKind::InMemory => Arc::new(InMemoryBackend::new()),
        };

        let data_dir = base_path.join("data");
        if config.read_only && !data_dir.exists() {
            return Err(VantaError::NotFound {
                kind: "data_directory".into(),
                id: data_dir.display().to_string(),
            });
        }
        if !config.read_only {
            std::fs::create_dir_all(&data_dir).map_err(VantaError::IoError)?;
        }

        Ok((lock_file, backend, data_dir))
    }

    fn init_indexes(
        data_dir: &Path,
        config: &VantaConfig,
        caps: &crate::hardware::HardwareCapabilities,
        effective_memory: u64,
    ) -> Result<(CPIndex, VantaFile)> {
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.mmap_hnsw
            && (config.force_mmap
                || caps.profile == crate::hardware::HardwareProfile::LowResource
                || effective_memory < 16 * GIB);

        let mut hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path, use_mmap) {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Resource Governance: MMap backend activated (cold-start)"
                );
            }
            loaded
        } else {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Resource Governance: MMap backend activated (fresh)"
                );
                CPIndex::with_backend(IndexBackend::new_mmap(index_path.clone()))
            } else {
                info!(
                    backend = "in-memory",
                    "HNSW Performance Mode: InMemory backend"
                );
                CPIndex::new()
            }
        };

        let vector_store_path = data_dir.join("vector_store.vanta");
        let vector_store = if config.read_only {
            VantaFile::open_read_only(vector_store_path)?
        } else {
            VantaFile::open(vector_store_path, 64 * MIB)?
        };

        if let Some(threshold) = config.flat_threshold {
            hnsw.config.flat_threshold = Some(threshold);
        }

        Ok((hnsw, vector_store))
    }

    fn recover_state(
        data_dir: &Path,
        config: &VantaConfig,
        backend: &dyn StorageBackend,
        hnsw: &mut CPIndex,
        vector_store: &mut VantaFile,
    ) -> Result<(u64, u64)> {
        let index_path = data_dir.join("vector_index.bin");

        if hnsw.nodes.is_empty() {
            let report =
                crate::storage::archive::rebuild_hnsw_from_vstore(hnsw, vector_store, index_path)?;
            crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);
            if report.scanned_nodes > 0 {
                info!(
                    scanned_nodes = report.scanned_nodes,
                    indexed_vectors = report.indexed_vectors,
                    skipped_tombstones = report.skipped_tombstones,
                    duration_ms = report.duration_ms,
                    "Index reconstructed from VantaFile"
                );
            }
        }

        let wal_path = data_dir.join("vanta.wal");
        let mut wal_replay_ms = 0u64;
        let mut wal_records_replayed = 0u64;
        let checkpoint_seq: u64 = backend
            .get(BackendPartition::InternalMetadata, b"checkpoint_seq")?
            .and_then(|bytes| postcard::from_bytes::<u64>(&bytes).ok())
            .unwrap_or(0);

        if !config.read_only && config.wal_shards > 0 {
            let num_shards = config.wal_shards.max(1);

            // Build shard path for a given index
            let shard_path_for = |idx: usize| -> std::path::PathBuf {
                if num_shards > 1 {
                    let dir = wal_path.parent().unwrap_or(Path::new("."));
                    let stem = wal_path.file_stem().unwrap_or_default().to_string_lossy();
                    let ext = wal_path
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default();
                    let shard_name = format!("{}.shard{}{}", stem, idx, ext);
                    dir.join(shard_name)
                } else {
                    wal_path.clone()
                }
            };

            // With multi-shard WAL the base vanta.wal never exists; check shard0 instead.
            let guard_path = shard_path_for(0);
            if guard_path.exists() {
                let wal_replay_started = Instant::now();

                // Compute per-shard skip from global checkpoint based on round-robin distribution.
                // With N shards, a record at local position `p` (0-indexed) in shard `s` has
                // global seq: global_seq = s + N * p.
                // The first `checkpoint_seq` records are already checkpointed and must be skipped.
                // Each shard `s` has either `floor(checkpoint_seq/N)` or `ceil(checkpoint_seq/N)`
                // pre-checkpoint records: shards 0..remainder-1 have one extra.
                let full_rounds = checkpoint_seq / num_shards as u64;
                let remainder = checkpoint_seq % num_shards as u64;

                // Read all records from all shards, compute their global seq, and sort.
                struct TimedRecord {
                    global_seq: u64,
                    record: crate::wal::WalRecord,
                }
                let mut pending: Vec<TimedRecord> = Vec::new();
                for shard_idx in 0..num_shards {
                    let shard_path = shard_path_for(shard_idx);
                    if !shard_path.exists() {
                        continue;
                    }
                    let mut reader = match crate::wal::WalReader::open(&shard_path) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    let skip = full_rounds + if (shard_idx as u64) < remainder { 1 } else { 0 };
                    let mut local_pos = 0u64;
                    while let Some(record) = reader.next_record()? {
                        if local_pos >= skip {
                            let global_seq = shard_idx as u64 + num_shards as u64 * local_pos;
                            pending.push(TimedRecord { global_seq, record });
                        }
                        local_pos += 1;
                    }
                }
                pending.sort_by_key(|tr| tr.global_seq);
                for tr in pending {
                    wal_records_replayed += 1;
                    match tr.record {
                        crate::wal::WalRecord::Insert(node) => {
                            StorageEngine::replay_write_node(
                                vector_store,
                                hnsw,
                                backend,
                                node.id,
                                &node,
                            )?;
                        }
                        crate::wal::WalRecord::Update { id, node } => {
                            StorageEngine::replay_write_node(
                                vector_store,
                                hnsw,
                                backend,
                                id,
                                &node,
                            )?;
                        }
                        crate::wal::WalRecord::Delete { id } => {
                            if let Some(index_node) = hnsw.nodes.get(&id) {
                                let offset = index_node.storage_offset;
                                if let Some(h) = vector_store.read_header(offset) {
                                    let mut tombstoned = h;
                                    tombstoned.flags |= FLAG_TOMBSTONE;
                                    vector_store.write_header(offset, &tombstoned)?;
                                }
                            }
                            // PERF-23/28: Remove from HNSW graph to prevent zombie nodes
                            hnsw.nodes.remove(&id);
                            // If this was the entry point, promote a replacement
                            if hnsw.entry_point.load(std::sync::atomic::Ordering::Relaxed) == id {
                                let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
                                hnsw.entry_point
                                    .store(new_ep, std::sync::atomic::Ordering::Relaxed);
                            }
                            let _ = backend.delete(BackendPartition::Default, &id.to_le_bytes());
                        }
                        crate::wal::WalRecord::Checkpoint { .. } => {}
                    }
                }
                wal_replay_ms = wal_replay_started.elapsed().as_millis() as u64;
                if wal_records_replayed > 0 {
                    info!(
                        replayed = wal_records_replayed,
                        duration_ms = wal_replay_ms,
                        checkpoint_seq,
                        "WAL replay: recovered un-flushed mutations"
                    );
                }
            }
        }
        Ok((wal_replay_ms, wal_records_replayed))
    }
}
