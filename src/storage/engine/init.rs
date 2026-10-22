//! StorageEngine initialization: opening, backend setup, index loading, WAL recovery.

use std::fs::{File as StdFile, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use web_time::Instant;

use crate::backend::{BackendPartition, StorageBackend};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::index_port::IndexPort;
use crate::lsm::SegmentLevel;
use crate::node::LabelIntern;
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BackendKind, GIB, MIB};
use crate::storage::ops;
#[cfg(unix)]
use crate::storage::vfile::install_sigbus_handler;
use crate::storage::vfile::File;

impl StorageEngine {
    /// Open with default configuration (backward-compatible).
    pub fn open(path: &str) -> Result<Self> {
        Self::open_with_config(path, None)
    }

    /// Open with explicit configuration for memory budgets and mode overrides.
    pub fn open_with_config(path: &str, config: Option<Config>) -> Result<Self> {
        let startup_started = Instant::now();
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();
        let effective_memory = config.memory_limit.unwrap_or(caps.total_memory);
        // FIND-598 (Paso 0): the decrypted-node cache of every segment is
        // sized from the **effective** memory limit (the budget the operator
        // declared, or the total RAM). Rule 4 of `memory-budget.md`: a
        // resident structure declares its cap.
        crate::storage::vfile::set_decrypt_cache_memory_limit(effective_memory);

        // ADR-0057 W6: resolve **one** encryption context for this process,
        // from the `Config`, and decide the database's mode from the
        // artefacts themselves. The context is handed down to the four
        // artefacts (WAL, VantaFile, backend KV, HNSW image): the DEK and the
        // nonce space belong to the **database**, not to the artefact's file.
        #[cfg(feature = "encryption")]
        let mut encryption = DbEncryption::resolve(path, &config)?;

        let (lock_file, backend, data_dir) = Self::init_storage(path, &config)?;

        // The backend may have created the registry (a brand-new database
        // with a key is encrypted from its first byte), so the mode is
        // decided **again**, now that the registry exists.
        #[cfg(feature = "encryption")]
        encryption.after_open(&data_dir)?;

        // ADR-0057 W3: the encryption context of the `vfile` artefact. It is
        // resolved **once** here, at open, and handed down to every segment:
        // the DEK and the nonce space belong to the artefact, not to the file.
        // Without a key the context is `None` and every segment stays
        // plaintext (byte-identical to the layout before this change).
        //
        // The in-memory backend has no `data_dir` (and nothing at rest): the
        // context is not resolved for it — the registry must never be written
        // outside a real storage root.
        let vfile_crypto: crate::storage::vfile_crypto::VFileCryptoSlot = {
            #[cfg(feature = "encryption")]
            {
                use crate::crypto::registry::Artifact;
                if data_dir.as_os_str().is_empty()
                    || encryption.mode != EncryptionMode::Encrypted
                    || !encryption.encrypted.contains(&Artifact::VantaFile)
                {
                    // A plaintext database (or one whose segments were never
                    // encrypted): the segments stay plaintext, byte-identical
                    // to the layout before the at-rest wiring.
                    None
                } else {
                    crate::storage::vfile_crypto::vfile_crypto_from_config(&data_dir, &config)?
                }
            }
            #[cfg(not(feature = "encryption"))]
            {
                None
            }
        };

        let (hnsw, vector_store, segment_registry, wal_writer, wal_replay_ms, wal_records_replayed) =
            if matches!(config.backend_kind, BackendKind::InMemory) {
                let hnsw = crate::index::port_impl::new_in_memory_port();
                let vs = File::create_in_memory(64 * MIB);
                let vstore = vec![parking_lot::RwLock::new(vs)];
                let reg = crate::lsm::SegmentRegistry::new();
                let wal_writer = None;
                (hnsw, vstore, reg, wal_writer, 0u64, 0u64)
            } else {
                // ADR-0057 W6: the write context of the index image, resolved
                // at the open (outside every engine lock — ADR-0057 D5).
                #[cfg(feature = "encryption")]
                let image_crypto = encryption.hnsw_context(&data_dir)?;
                #[cfg(not(feature = "encryption"))]
                let image_crypto: Option<
                    crate::index::serialize::crypto::HnswImageCryptoContext,
                > = None;
                let (mut hnsw, vector_store, segment_registry) = Self::init_indexes(
                    &data_dir,
                    &config,
                    caps,
                    effective_memory,
                    vfile_crypto,
                    image_crypto,
                )?;
                // The read side of the WAL: the engine resolved the key
                // once, so the reader uses the same DEK as the writer.
                #[cfg(feature = "encryption")]
                let wal_key_material = encryption.wal_key()?;
                #[cfg(not(feature = "encryption"))]
                let wal_key_material: Option<&[u8]> = None;
                let (wal_replay_ms, wal_records_replayed) = Self::recover_state(
                    &data_dir,
                    &config,
                    backend.as_ref(),
                    &mut *hnsw,
                    &vector_store,
                    wal_key_material,
                )?;
                // ADR-0057 W2: the WAL context comes from the **same** one
                // the engine resolved: one DEK, one nonce space, one
                // registry for the whole database.
                let wal_writer = Self::init_wal(
                    &data_dir,
                    &config,
                    #[cfg(feature = "encryption")]
                    encryption.wal_context(&data_dir)?,
                )?;
                (
                    hnsw,
                    vector_store,
                    segment_registry,
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
        let resident_bytes: Option<u64> = {
            let mut total: Option<u64> = None;
            for vs in &vector_store {
                let guard = vs.read();
                if let Some(rb) = guard.mmap_resident_bytes() {
                    total = Some(total.unwrap_or(0) + rb);
                }
            }
            if let Some(rb) = hnsw.mmap_resident_bytes() {
                total = Some(total.unwrap_or(0) + rb);
            }
            total
        };
        crate::metrics::record_memory_breakdown(
            hnsw.node_count() as u64,
            estimated_hnsw_bytes,
            resident_bytes,
            0,
            0,
        );

        if hnsw.node_count() > 10_000 && estimated_hnsw_bytes > effective_memory / 2 {
            tracing::warn!(
                hnsw_nodes = hnsw.node_count(),
                estimated_mb = estimated_hnsw_bytes / MIB,
                effective_mb = effective_memory / MIB,
                "HNSW index exceeds 50% of memory budget",
            );
        }

        let cardinality_stats = Self::initialize_cardinality_stats(backend.as_ref());

        let engine = Self {
            config: config.clone(),
            read_only: config.read_only,
            hnsw: arc_swap::ArcSwap::new(Arc::new(hnsw)),
            insert_lock: parking_lot::FairMutex::new(()),
            pending_hnsw_batch: parking_lot::Mutex::new(Vec::new()),
            cache: super::cache::CacheLayer::new(cardinality_stats),
            last_query_timestamp: std::sync::atomic::AtomicU64::new(0),
            txn: super::txn::TxnManager::new(),
            emergency_maintenance_trigger: std::sync::atomic::AtomicBool::new(false),
            data_dir,
            vector_store,
            segment_registry,
            wal: wal_writer.map(std::sync::Arc::new),
            _lock_file: lock_file,
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
            label_intern: parking_lot::Mutex::new(LabelIntern::new()),
        };

        // OLD-20: Warm HNSW top-layer nodes on startup so the first search
        // doesn't pay a cold-start penalty reading entry-point nodes from disk.
        engine.warm_hnsw_top_layer();

        // MOD-04: rebuild the scalar index from backend metadata on open —
        // `recover_state` writes directly (replay_write_node) and never
        // maintains the index, so a reopen would otherwise start with an
        // empty index and TTL purge would miss every pre-existing expired
        // record. `edge_index` shares the same one-time rebuild pattern.
        engine.rebuild_scalar_index()?;

        Ok(engine)
    }

    fn init_storage(
        path: &str,
        config: &Config,
    ) -> Result<(Option<StdFile>, Arc<dyn StorageBackend>, PathBuf)> {
        ops::prevent_path_traversal(path)?;
        let base_path = PathBuf::from(path);

        if matches!(config.backend_kind, BackendKind::InMemory) {
            let backend: Arc<dyn StorageBackend> =
                Arc::new(crate::backends::in_memory::InMemoryBackend::new());
            return Ok((None, backend, PathBuf::new()));
        }

        if config.read_only && !base_path.exists() {
            return Err(Error::NotFound {
                kind: "database_path".into(),
                id: base_path.display().to_string(),
            });
        }
        let lock_file = {
            let lock_path = base_path.join(".vanta.lock");
            if !config.read_only {
                std::fs::create_dir_all(&base_path).map_err(Error::Io)?;
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
                        return Err(Error::NotFound {
                            kind: "lock_file".into(),
                            id: base_path.join(".vanta.lock").display().to_string(),
                        });
                    } else {
                        return Err(Error::Io(e));
                    }
                }
            };

            let mut delay = std::time::Duration::from_millis(5);
            let total_limit = std::time::Duration::from_millis(config.file_lock_timeout_ms);
            let start_time = Instant::now();
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
                    if !config.read_only {
                        let _ = crate::storage::lock::write_lock_metadata(
                            &file,
                            &crate::storage::lock::LockMetadata::current(),
                        );
                    }
                    break;
                }

                // wasm32: single-threaded, no other process can release the
                // lock between retries and std::thread::sleep panics there
                // (condvar::no_threads) — one pass suffices; a miss surfaces
                // as DatabaseBusy after the loop, no hot spin.
                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(delay);
                delay = std::cmp::min(delay * 2, std::time::Duration::from_millis(100));
            }

            if !acquired {
                let holder_detail = crate::storage::lock::read_lock_metadata(&lock_path)
                    .map(|meta| {
                        format!(
                            " (held by '{}', PID {} on host '{}')",
                            meta.process_name, meta.pid, meta.hostname
                        )
                    })
                    .unwrap_or_default();

                let msg = if config.read_only {
                    format!(
                        "Database at '{}' is locked exclusively by another process (writer){holder_detail}. \
                         Cannot acquire shared read-only lock within timeout.",
                        base_path.display()
                    )
                } else {
                    format!(
                        "Database at '{}' is locked by another process{holder_detail}. \
                         Cannot acquire exclusive writer lock within timeout.",
                        base_path.display()
                    )
                };
                return Err(Error::DatabaseBusy(msg));
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

        // C2S2 (OCP): construction goes through `BackendRegistry` — a new
        // backend registers its factory there; this function never grows
        // another `match` arm.
        let backend: Arc<dyn StorageBackend> =
            crate::backends::registry::BackendRegistry::default_registry().create(
                config.backend_kind,
                path,
                config,
            )?;

        let data_dir = base_path.join("data");
        if config.read_only && !data_dir.exists() {
            return Err(Error::NotFound {
                kind: "data_directory".into(),
                id: data_dir.display().to_string(),
            });
        }
        if !config.read_only {
            std::fs::create_dir_all(&data_dir).map_err(Error::Io)?;
        }

        Ok((lock_file, backend, data_dir))
    }

    /// Open the write-ahead log, encrypted when the database is encrypted.
    ///
    /// ADR-0057 W2: the context is the engine's **one** resolved context, so
    /// the WAL shares the database's DEK, nonce space and registry with the
    /// other three artefacts. A plaintext database keeps the plaintext WAL,
    /// byte-identical to the layout before the at-rest wiring.
    fn init_wal(
        data_dir: &Path,
        config: &Config,
        #[cfg(feature = "encryption")] crypto: Option<crate::wal::WalCrypto>,
    ) -> Result<Option<crate::wal_sharded::ShardedWal>> {
        if config.read_only {
            return Ok(None);
        }
        #[cfg(feature = "encryption")]
        if let Some(crypto) = crypto {
            let wal_path = data_dir.join("vanta.wal");
            return Ok(Some(crate::wal_sharded::ShardedWal::new_encrypted(
                &wal_path,
                config.wal_shards,
                config.sync_mode,
                config.wal_buffer_size.unwrap_or(64 * 1024),
                config.flush_threshold,
                crypto,
            )?));
        }
        crate::storage::wal::init_wal(data_dir, config)
    }

    fn init_indexes(
        data_dir: &Path,
        config: &Config,
        caps: &crate::hardware::HardwareCapabilities,
        effective_memory: u64,
        crypto: crate::storage::vfile_crypto::VFileCryptoSlot,
        // ADR-0057 W6 / D5: the write context of the index image, resolved
        // at the open of the engine (outside every engine lock).
        image_crypto: Option<crate::index::serialize::crypto::HnswImageCryptoContext>,
    ) -> Result<(
        Box<dyn IndexPort>,
        Vec<parking_lot::RwLock<File>>,
        crate::lsm::SegmentRegistry,
    )> {
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.mmap_hnsw
            && (config.force_mmap
                || caps.profile == crate::hardware::HardwareProfile::LowResource
                || effective_memory < 16 * GIB);

        let mut hnsw: Box<dyn IndexPort> =
            match crate::index::port_impl::open_index_port(&index_path, use_mmap, image_crypto) {
                Ok(loaded) => {
                    if use_mmap {
                        info!(
                            backend = "mmap",
                            "HNSW Resource Governance: MMap backend activated (cold-start)"
                        );
                    }
                    loaded
                }
                Err(_) => {
                    if use_mmap {
                        info!(
                            backend = "mmap",
                            "HNSW Resource Governance: MMap backend activated (fresh)"
                        );
                        crate::index::port_impl::new_mmap_port(index_path.clone())
                    } else {
                        info!(
                            backend = "in-memory",
                            "HNSW Performance Mode: InMemory backend"
                        );
                        crate::index::port_impl::new_in_memory_port()
                    }
                }
            };

        if let Some(threshold) = config.flat_threshold {
            hnsw.set_flat_threshold(Some(threshold));
        }

        // Multi-level: open or create L0..L3 VantaFiles via SegmentRegistry
        let (segment_registry, vfiles) = if config.read_only {
            // Read-only: open each level if it exists
            let mut reg = crate::lsm::SegmentRegistry::new();
            let mut vfs = Vec::new();
            for level in &[
                SegmentLevel::L0,
                SegmentLevel::L1,
                SegmentLevel::L2,
                SegmentLevel::L3,
            ] {
                let path = data_dir.join(level.file_name());
                if path.exists() {
                    let vf = File::open_with_crypto(path.clone(), 0, true, crypto.clone())?;
                    reg.register(level.as_u8(), level.as_u8(), path);
                    vfs.push(parking_lot::RwLock::new(vf));
                }
            }
            // Fallback: if no levels exist, open legacy path (may fail)
            if vfs.is_empty() {
                let legacy = data_dir.join("vector_store.vanta");
                let vf = File::open_with_crypto(legacy, 0, true, crypto.clone())?;
                let path = data_dir.join(SegmentLevel::L0.file_name());
                reg.register(0, 0, path);
                vfs.push(parking_lot::RwLock::new(vf));
            }
            (reg, vfs)
        } else {
            crate::lsm::SegmentRegistry::open_or_create(
                data_dir,
                &config.segment_optimizer,
                crypto,
            )?
        };

        Ok((hnsw, vfiles, segment_registry))
    }

    fn recover_state(
        data_dir: &Path,
        config: &Config,
        backend: &dyn StorageBackend,
        hnsw: &mut dyn IndexPort,
        vector_store: &[parking_lot::RwLock<File>],
        // ADR-0057 W2: the key of the WAL artefact — an encrypted WAL needs
        // it to replay; a plaintext WAL ignores it.
        #[cfg_attr(not(feature = "encryption"), allow(unused_variables))] wal_key: Option<&[u8]>,
    ) -> Result<(u64, u64)> {
        let index_path = data_dir.join("vector_index.bin");

        // The read side of the WAL: the engine resolved the key once, so the
        // reader uses the same DEK as the writer.
        #[cfg(feature = "encryption")]
        let wal_key = match wal_key {
            Some(material) => Some(crate::wal::WalKey::open(data_dir, material)?),
            None => None,
        };

        if hnsw.is_empty() {
            // Rebuild from L0 (and eventually all levels) — for now L0 is primary
            let report = {
                let l0_vf = vector_store[0].write();
                crate::storage::archive::rebuild_hnsw_from_vstore(hnsw, &l0_vf, index_path)?
            };
            crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);
            if report.scanned_nodes > 0 {
                info!(
                    scanned_nodes = report.scanned_nodes,
                    indexed_vectors = report.indexed_vectors,
                    skipped_tombstones = report.skipped_tombstones,
                    duration_ms = report.duration_ms,
                    "Index reconstructed from File"
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
            // AUDREP-16: reconcile to the on-disk layout instead of trusting the
            // config. A WAL written with a different shard count would otherwise
            // be replayed with mismatched shard files and lose data silently.
            let num_shards = crate::wal_sharded::detect_shard_count(&wal_path)
                .or_else(|| crate::wal_sharded::read_shard_meta(&wal_path))
                .unwrap_or(config.wal_shards.max(1));

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
                // ERR-011: track per-shard record counts so a shard whose tail was
                // truncated (or that failed to open) is detected instead of replaying
                // short and reporting a checkpoint that silently skipped records.
                let mut shard_counts = vec![0u64; num_shards];
                for (shard_idx, shard_count) in shard_counts.iter_mut().enumerate() {
                    let shard_path = shard_path_for(shard_idx);
                    if !shard_path.exists() {
                        continue;
                    }
                    #[cfg(feature = "encryption")]
                    let mut reader = match &wal_key {
                        // An encrypted (v4) WAL needs the key; a plaintext
                        // (v3) one ignores it.
                        Some(key) => crate::wal::WalReader::open_with_key(&shard_path, key.clone()),
                        None => crate::wal::WalReader::open(&shard_path),
                    }
                    .map_err(|e| {
                        Error::wal_error(format!(
                            "Failed to open WAL shard {shard_idx} during recovery: {e}"
                        ))
                    })?;
                    #[cfg(not(feature = "encryption"))]
                    let mut reader = crate::wal::WalReader::open(&shard_path).map_err(|e| {
                        Error::wal_error(format!(
                            "Failed to open WAL shard {shard_idx} during recovery: {e}"
                        ))
                    })?;
                    let skip = full_rounds + if (shard_idx as u64) < remainder { 1 } else { 0 };
                    let mut local_pos = 0u64;
                    while let Some(record) = reader.next_record()? {
                        if local_pos >= skip {
                            let global_seq = shard_idx as u64 + num_shards as u64 * local_pos;
                            pending.push(TimedRecord { global_seq, record });
                        }
                        local_pos += 1;
                    }
                    *shard_count = local_pos;
                }
                // ERR-011: round-robin only produces a coherent dataset when every
                // shard matches the sibling local positions; surface the gap instead
                // of silently replaying a truncated shard short. Single-shard legacy
                // WALs are exempt (there is no round-robin layout to corrupt).
                if num_shards > 1 {
                    if let Some(msg) = crate::wal_sharded::verify_shard_counts(&shard_counts) {
                        return Err(Error::wal_error(msg));
                    }
                }
                pending.sort_by_key(|tr| tr.global_seq);
                let mut skip_mask = vec![false; pending.len()];
                // MOD-02: track the currently-open txn batch by (txn_id, start
                // position). A txn's batch is [Begin, ops…, Commit] written by a
                // single `batch_append`, so its records occupy contiguous
                // global-seq slots. A crash mid-append leaves a durable prefix of
                // the batch with the Commit lost; recovery must discard that
                // prefix (no Commit → ops never applied) WITHOUT dropping records
                // of later, complete batches — a new `Begin` marks that boundary.
                let mut open_txn: Option<(u64, usize)> = None;
                for (i, tr) in pending.iter().enumerate() {
                    match &tr.record {
                        crate::wal::WalRecord::Begin(txn_id) => {
                            // A new batch starts at `i`; any batch still open here
                            // never got its Commit (contiguous slots mean its
                            // Commit would have appeared before this Begin).
                            // Discard the incomplete batch's own extent.
                            if let Some((_, start)) = open_txn {
                                skip_mask[start..i].fill(true);
                            }
                            open_txn = Some((*txn_id, i));
                        }
                        crate::wal::WalRecord::Commit(txn_id) => {
                            // Only the matching txn's Commit closes its batch; a
                            // bare Commit from another txn must not resurrect
                            // ops that never committed.
                            if let Some((open_id, _)) = open_txn {
                                if open_id == *txn_id {
                                    open_txn = None;
                                }
                            }
                        }
                        crate::wal::WalRecord::Abort(txn_id) => {
                            if let Some((open_id, start)) = open_txn {
                                if open_id == *txn_id {
                                    skip_mask[start..=i].fill(true);
                                    open_txn = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Some((_, start)) = open_txn {
                    // Trailing incomplete batch at EOF: nothing after it can be
                    // attributed to a different writer, so discard it fail-safe.
                    skip_mask[start..].fill(true);
                }

                for (i, tr) in pending.into_iter().enumerate() {
                    if skip_mask[i] {
                        continue;
                    }
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
                            if let Some(packed_offset) = hnsw.storage_offset_of(id) {
                                let (seg_id, local_off) = crate::lsm::unpack_offset(packed_offset);
                                if let Some(vs) = vector_store.get(seg_id as usize) {
                                    let mut vstore = vs.write();
                                    // ADR-0057 W3: in an encrypted segment the
                                    // header lives inside the ciphertext, so
                                    // the node is re-sealed instead of being
                                    // patched in place.
                                    vstore.set_tombstone(local_off, true)?;
                                }
                            }
                            // PERF-23/28: Remove from HNSW graph to prevent zombie nodes
                            hnsw.remove_node(id);
                            // If this was the entry point, promote a replacement
                            if hnsw.entry_point() == Some(id) {
                                let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
                                hnsw.set_entry_point(new_ep);
                            }
                            let _ = backend.delete(BackendPartition::Default, &id.to_le_bytes());
                        }
                        crate::wal::WalRecord::Checkpoint { .. } => {}
                        crate::wal::WalRecord::Begin(_)
                        | crate::wal::WalRecord::Commit(_)
                        | crate::wal::WalRecord::Abort(_)
                        // WAL v2 (RES-01): Prepare is a two-phase marker. Replay
                        // semantics are unchanged (slice-mask still drops ops whose
                        // Commit never became durable).
                        | crate::wal::WalRecord::Prepare { .. } => {}
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

// ─── ADR-0057 W6: the activation ─────────────────────────────────
//
// The engine resolves **one** encryption context per process, from the
// `Config`, and hands it to the four artefacts. The mode of the database is
// decided from the artefacts themselves (ADR-0057 D1 rule 2: the reader
// decides by the bytes on disk, never by the configuration).

/// The encryption mode of a database.
#[cfg(feature = "encryption")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EncryptionMode {
    /// No artefact of this database is encrypted: the bytes on disk are the
    /// plaintext layout.
    Plaintext,
    /// At least one artefact is encrypted: the database needs the key.
    Encrypted,
}

/// ADR-0057 W6: the encryption context of one database.
///
/// The engine resolves it **once**, at the open, and hands it to the four
/// artefacts: the DEK and the nonce space belong to the **database**, not to
/// the artefact's file. The registry behind it is **shared** by the four
/// artefacts — one in-memory state, one `persist` path (FIND-514).
#[cfg(feature = "encryption")]
pub(crate) struct DbEncryption {
    /// The master key material (zeroizing) — `None` when no key resolved.
    material: Option<std::sync::Arc<crate::crypto::Zeroizing<Vec<u8>>>>,
    /// The mode of the database.
    mode: EncryptionMode,
    /// The artefacts the registry records as encrypted — the **mode record**
    /// of the database (ADR-0057 D1 rule 3: no mixed mode).
    encrypted: std::collections::BTreeSet<crate::crypto::registry::Artifact>,
    /// The storage root of this database.
    root: std::path::PathBuf,
    /// Whether the database is brand new (no artefact on disk yet).
    /// Whether the open is read-only: an epoch is never allocated then, and
    /// the registry is never written.
    read_only: bool,
    /// Whether the backend is in-memory: there is nothing at rest, so the
    /// key is not used and nothing is said about it.
    in_memory: bool,
}

#[cfg(feature = "encryption")]
impl DbEncryption {
    /// Resolve the context of the database at `path`.
    ///
    /// The master key comes from the `Config` first — a programmatic
    /// `Config::encryption_key` is invisible to the environment, and the
    /// artefacts that resolve their own context must see the **same** key
    /// (ADR-0057 D2 lists the environment first, but the engine's decision
    /// is the authoritative one for the at-rest path).
    fn resolve(path: &str, config: &Config) -> Result<Self> {
        let root = Path::new(path);
        let data_dir = root.join("data");
        let mode = detect_encryption_mode(&data_dir);
        let new_database = is_new_database(root, &data_dir);

        let env_key = config
            .encryption_key
            .clone()
            .or_else(|| std::env::var("VANTADB_ENCRYPTION_KEY").ok());
        let material = match crate::crypto::resolve_master_key_material_strict(
            env_key.as_deref(),
            config.encryption_key_file.as_deref(),
            config.encryption_key_cmd.as_deref(),
        ) {
            Ok(material) => Some(std::sync::Arc::new(material)),
            Err(crate::crypto::CryptoError::KeyNotSet) => None,
            Err(err) => return Err(crypto_error(err)),
        };

        // An encrypted database opened without a key: refuse, loudly. The
        // backend must never be opened in that state — it would hand the
        // ciphertext back as if it were plaintext (FIND-516).
        if mode == EncryptionMode::Encrypted && material.is_none() {
            return Err(Error::EncryptionKeyRequired {
                artifact: "database".to_string(),
            });
        }

        // The decision, registered process-wide: the artefacts that resolve
        // their own context (the backend, the HNSW image) must use the
        // engine's key — and must use **no** key when the database stays
        // plaintext (a plaintext database is never migrated implicitly —
        // ADR-0057 D7). An in-memory backend has nothing at rest: the key
        // is not used at all.
        let in_memory = matches!(config.backend_kind, BackendKind::InMemory);
        let encrypt = match mode {
            EncryptionMode::Encrypted => true,
            EncryptionMode::Plaintext => new_database && material.is_some() && !in_memory,
        };
        crate::crypto::register_process_key(root, if encrypt { material.clone() } else { None });

        Ok(Self {
            material,
            mode,
            encrypted: Default::default(),
            root: root.to_path_buf(),
            read_only: config.read_only,
            in_memory,
        })
    }

    /// Re-decide the mode now that the backend is open (it may have created
    /// the registry of a brand-new database) and record which artefacts are
    /// encrypted.
    fn after_open(&mut self, data_dir: &Path) -> Result<()> {
        self.mode = detect_encryption_mode(data_dir);
        if self.mode != EncryptionMode::Encrypted {
            // A plaintext database with a key configured: say it out loud.
            if self.material.is_some() && !self.in_memory {
                warn_encryption_not_applied();
            }
            return Ok(());
        }
        let Some(material) = self.material.as_ref() else {
            return Err(Error::EncryptionKeyRequired {
                artifact: "database".to_string(),
            });
        };
        // The registry is the **mode record**: it says which artefacts are
        // encrypted. A brand-new database gets the four entries, so every
        // artefact is encrypted from its first byte (ADR-0057 D7). The
        // entries are created **here**, at open — outside every engine lock
        // (ADR-0057 D5).
        let mut registry =
            crate::crypto::registry::KeyRegistry::open_or_create(data_dir, &material[..])
                .map_err(crypto_error)?;
        // The four DEKs are ensured **whenever the mode is encrypted**, not
        // only for a brand-new database. A crash between the backend opening
        // (which creates the registry with a single `BackendKv` entry) and this
        // point left the registry saying "encrypted" while the WAL, the
        // segments and the HNSW image were still in plaintext — a silent
        // mixed mode (ADR-0057 D1 rule 3 forbids it) with no warning.
        //
        // `ensure_dek` is a no-op for an artefact that already has an entry
        // (it returns the existing DEK, it never re-derives one), so this
        // cannot invalidate a single existing ciphertext.
        for artifact in [
            crate::crypto::registry::Artifact::Wal,
            crate::crypto::registry::Artifact::VantaFile,
            crate::crypto::registry::Artifact::HnswImage,
            crate::crypto::registry::Artifact::BackendKv,
        ] {
            registry.ensure_dek(artifact).map_err(crypto_error)?;
        }
        self.encrypted = registry.artifacts();
        crate::crypto::register_process_key(&self.root, Some(material.clone()));
        Ok(())
    }

    /// The write-side context of the `wal` artefact — `None` for a plaintext
    /// database.
    fn wal_context(&self, data_dir: &Path) -> Result<Option<crate::wal::WalCrypto>> {
        use crate::crypto::registry::Artifact;
        if self.mode != EncryptionMode::Encrypted || !self.encrypted.contains(&Artifact::Wal) {
            return Ok(None);
        }
        let Some(material) = self.material.as_ref() else {
            return Ok(None);
        };
        Ok(Some(crate::wal::WalCrypto::open(data_dir, &material[..])?))
    }

    /// The read-side key of the `wal` artefact (the recovery path).
    fn wal_key(&self) -> Result<Option<&[u8]>> {
        use crate::crypto::registry::Artifact;
        if self.mode != EncryptionMode::Encrypted || !self.encrypted.contains(&Artifact::Wal) {
            return Ok(None);
        }
        Ok(self.material.as_ref().map(|material| &material[..]))
    }

    /// The **write** context of the `hnsw` artefact — `None` for a plaintext
    /// database.
    ///
    /// It is resolved **here**, at the open of the engine, so the nonce epoch
    /// of the image is allocated (and the registry written) **outside every
    /// engine lock** (ADR-0057 D5). Without it, the image would resolve its
    /// context at its first write — which happens under the `insert_lock`.
    fn hnsw_context(
        &self,
        data_dir: &Path,
    ) -> Result<Option<crate::index::serialize::crypto::HnswImageCryptoContext>> {
        use crate::crypto::registry::Artifact;
        if self.mode != EncryptionMode::Encrypted || !self.encrypted.contains(&Artifact::HnswImage)
        {
            return Ok(None);
        }
        let Some(material) = self.material.as_ref() else {
            return Ok(None);
        };
        let image_path = data_dir.join("vector_index.bin");
        let identifier = crate::index::serialize::crypto::image_identifier(data_dir, &image_path);
        let crypto = if self.read_only {
            crate::index::serialize::crypto::HnswImageCrypto::open_read(
                data_dir,
                &material[..],
                identifier,
            )?
        } else {
            crate::index::serialize::crypto::HnswImageCrypto::open(
                data_dir,
                &material[..],
                identifier,
            )?
        };
        Ok(Some(std::sync::Arc::new(crypto)))
    }
}

/// The format version of an on-disk artefact header, if the file exists.
#[cfg(feature = "encryption")]
fn header_version(path: &Path) -> Option<u16> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut head = [0u8; crate::binary_header::Header::SIZE];
    file.read_exact(&mut head).ok()?;
    let header = crate::binary_header::Header::deserialize(&head).ok()?;
    Some(header.format_version)
}

/// The WAL files of a database (the single-shard one and the shards).
#[cfg(feature = "encryption")]
fn wal_paths(data_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = vec![data_dir.join("vanta.wal")];
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("vanta.shard") && name.ends_with(".wal") {
                paths.push(entry.path());
            }
        }
    }
    paths
}

/// The VantaFile segments of a database (`L0..L3` plus the legacy name).
#[cfg(feature = "encryption")]
fn vfile_paths(data_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<std::path::PathBuf> = (0..4)
        .map(|level| data_dir.join(format!("vstore_L{level}.vanta")))
        .collect();
    paths.push(data_dir.join("vector_store.vanta"));
    paths
}

/// Decide the encryption mode of the database from the artefacts themselves
/// (ADR-0057 D1 rule 2: the reader decides by the bytes on disk, never by
/// the configuration).
#[cfg(feature = "encryption")]
fn detect_encryption_mode(data_dir: &Path) -> EncryptionMode {
    // The registry only exists for an encrypted database: it holds the
    // wrapped DEKs, and it is created only when a key resolved.
    if crate::crypto::registry::KeyRegistry::path_in(data_dir).exists() {
        return EncryptionMode::Encrypted;
    }
    // The WAL: format v4 is the encrypted one.
    for path in wal_paths(data_dir) {
        if header_version(&path).is_some_and(|v| v >= crate::wal::WAL_ENCRYPTED_FORMAT_VERSION) {
            return EncryptionMode::Encrypted;
        }
    }
    // The VantaFile segments: format v3 is the encrypted one.
    for path in vfile_paths(data_dir) {
        if header_version(&path)
            .is_some_and(|v| v >= crate::storage::vfile::VFILE_ENCRYPTED_VERSION)
        {
            return EncryptionMode::Encrypted;
        }
    }
    // The HNSW image: format v9 is the encrypted one.
    let image = data_dir.join("vector_index.bin");
    if header_version(&image)
        .is_some_and(|v| v >= crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION)
    {
        return EncryptionMode::Encrypted;
    }
    EncryptionMode::Plaintext
}

/// Whether the database is **brand new**: it holds no data at all.
///
/// A brand-new database with a key is encrypted from its first byte
/// (ADR-0057 D7); an existing database — even one that was only *opened* —
/// stays plaintext: a key never migrates it (no implicit migration, no
/// mixed mode).
#[cfg(feature = "encryption")]
fn is_new_database(root: &Path, data_dir: &Path) -> bool {
    if crate::crypto::registry::KeyRegistry::path_in(data_dir).exists() {
        return false;
    }
    if wal_paths(data_dir).iter().any(|path| path.exists()) {
        return false;
    }
    if vfile_paths(data_dir).iter().any(|path| path.exists()) {
        return false;
    }
    if data_dir.join("vector_index.bin").exists() {
        return false;
    }
    // The storage root: any file besides the process-local lock means the
    // database has been opened before (the schema, the backend files).
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == ".vanta.lock" || entry.path().is_dir() {
                continue;
            }
            return false;
        }
    }
    true
}

/// ADR-0057 D7: a plaintext database opened with a key configured stays
/// plaintext — the key is ignored for it, and the mismatch is said out loud.
#[cfg(feature = "encryption")]
fn warn_encryption_not_applied() {
    tracing::warn!(
        "encryption key configured but this database is PLAINTEXT: the WAL, the \
         VantaFile segments, the HNSW image and the backend key-value store stay \
         in cleartext. VantaDB never migrates a database implicitly (ADR-0057 D7) \
         — to encrypt an existing database, export it and import it into a new \
         database created with the key configured. The key is ignored for this \
         database."
    );
}

/// Map a crypto failure onto the crate error surface (ADR-0057 D6).
#[cfg(feature = "encryption")]
fn crypto_error(err: crate::crypto::CryptoError) -> Error {
    match err {
        crate::crypto::CryptoError::DecryptionFailed => Error::EncryptionKeyRejected {
            artifact: "database".to_string(),
        },
        crate::crypto::CryptoError::RegistryInvalid(detail) => Error::EncryptionCorrupt {
            artifact: "database".to_string(),
            detail,
        },
        other => Error::generic_error(format!("Encryption error: {other}")),
    }
}
