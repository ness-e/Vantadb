//! Maintenance operations: refresh, consolidate, evict, rebuild, compact, flush, WAL.

use std::sync::Arc;
use web_time::Instant;

use crate::backend::BackendPartition;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::index_port::FreshHnswReport;
use crate::node::{NodeTier, UnifiedNode, VectorRepresentations};
use crate::storage::engine::{
    EvictionReason, EvictionReport, IndexRebuildReport, LockPolicy, LsmReport, MergeReport,
    PipelineMode, PipelineReport, QuantizationMaintenanceReport, StorageEngine, VacuumReport,
    FLAG_TOMBSTONE, STORAGE_ALIGNMENT,
};
use crate::storage::ops::NodeMetadata;
use crate::vector::governor::QuantizationAction;

impl StorageEngine {
    /// Check tombstone fragmentation and run an offline compaction pass when
    /// it exceeds the configured threshold.
    ///
    /// Delegates to [`Self::merge_segments`], which measures tombstone
    /// fragmentation against `segment_optimizer.vacuum_threshold_pct` and
    /// rewrites the File via [`Self::compact_layout_bfs`], actually
    /// reclaiming the space held by tombstoned nodes (MOD-03: this used to be
    /// a stub that only logged).
    pub fn trigger_compaction(&self) -> Result<()> {
        self.merge_segments()?;
        Ok(())
    }

    /// Flush all pending writes: backend, vector store, WAL checkpoint, and vector index.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn flush(&self) -> Result<()> {
        self.ensure_writable()?;

        // ERR-010: hold insert_lock across [drain → serialize → count → write].
        // Every mutating path (insert/delete/batch) appends its WAL record and
        // queues its HNSW mutation under the same guard, so while this critical
        // section runs no record can be counted by total_record_count() whose
        // mutation is not already drained into the index. The snapshot is the
        // exact, quiescent set of WAL records <= checkpoint_seq: replay skips
        // nothing it needs (no invisible records) and re-applies nothing it
        // already has (no duplicates).
        let _guard =
            self.acquire_insert_lock("acquire insert_lock in flush (ERR-010 checkpoint)")?;

        // Drain pending HNSW mutations before checkpointing (lock already held).
        self.drain_hnsw_batch_locked()?;
        self.backend.flush()?;
        for vs in &self.vector_store {
            vs.read().flush()?;
        }

        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("snapshot_serialize_fail", |_| {
                Err(crate::error::Error::Io(std::io::Error::other(
                    "Simulated snapshot serialize I/O failure",
                )))
            });
        }
        self.save_vector_index()?;

        // ERR-010: checkpoint_seq MUST be recorded AFTER the snapshot is
        // serialized. WAL records are written before their HNSW mutation, so
        // any op captured in the snapshot has its WAL record durable by the
        // time the count is read here — checkpoint_seq >= snapshot contents,
        // and replay on reopen never re-applies an already-snapshotted op
        // (no duplication). If save_vector_index fails above, the checkpoint
        // is NOT advanced and the WAL replay covers the gap instead. The
        // insert_lock above makes this count simultaneous with the drain, so
        // it can never observe a record whose mutation is still queued.
        let current_wal_seq = self
            .wal
            .as_ref()
            .map(|s| s.total_record_count())
            .unwrap_or(0);

        if current_wal_seq > 0 {
            let seq_bytes =
                postcard::to_allocvec(&current_wal_seq).map_err(Error::serialization)?;
            self.backend.put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq_bytes,
            )?;
            self.backend.flush()?;
        }
        // _guard drops here — the checkpoint is durable before any new
        // mutation can be counted.

        // PERF-09: Run quantization auto-transition during flush
        if let Ok(report) = self.run_quantization_maintenance() {
            if report.quantized > 0 || report.promoted > 0 {
                tracing::debug!(
                    quantized = report.quantized,
                    promoted = report.promoted,
                    "Quantization maintenance completed"
                );
            }
        }

        let hnsw = self.hnsw.load();
        let mut resident_bytes: Option<u64> = None;
        for vs in &self.vector_store {
            let guard = vs.read();
            if let Some(rb) = guard.mmap_resident_bytes() {
                resident_bytes = Some(resident_bytes.unwrap_or(0) + rb);
            }
        }
        if let Some(rb) = hnsw.mmap_resident_bytes() {
            resident_bytes = Some(resident_bytes.unwrap_or(0) + rb);
        }
        crate::metrics::record_memory_breakdown(
            hnsw.node_count() as u64,
            hnsw.estimate_memory_bytes() as u64,
            resident_bytes,
            self.cache.volatile.read().len() as u64,
            0,
        );
        Ok(())
    }

    /// Compact the WAL: flush all data, archive the current WAL file
    /// and start a fresh WAL.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn compact_wal(&self) -> Result<()> {
        self.flush()?;

        if let Some(ref sharded) = self.wal {
            sharded.rotate_all()?;
        }

        let zero: [u8; 8] = 0u64.to_le_bytes();
        self.backend
            .put(BackendPartition::InternalMetadata, b"checkpoint_seq", &zero)?;
        self.backend.flush()?;

        Ok(())
    }

    fn save_vector_index(&self) -> Result<()> {
        if self.wal.is_none() {
            return Ok(()); // ponytail: ephemeral in-memory mode, nothing to persist
        }
        let index_path = self.data_dir.join("vector_index.bin");
        let current = self.hnsw.load();

        if current.is_mmap() {
            // Persist dance moved to `IndexPort::persist_mmap` (byte-identical).
            let new_hnsw = current.persist_mmap(&index_path)?;
            self.hnsw.store(Arc::new(new_hnsw));
        } else {
            current.persist_to_file(&index_path)?;
        }
        Ok(())
    }

    /// Apply the HNSW index entry for a node WITHOUT acquiring `insert_lock`.
    ///
    /// Callers MUST already hold `insert_lock` (e.g. the `*_locked` variants of
    /// consolidation/eviction invoked from inside an insert section), or be on a
    /// path where no writer can concurrently mutate the HNSW index. The
    /// `MmapFull`→`Full` copy below is load-bearing: `consolidate_node_inner`
    /// releases the mmap pages right after this runs, so the entry must hold an
    /// owned vector before the release happens.
    fn apply_index_entry_unlocked(&self, node: &UnifiedNode, storage_offset: u64) -> Result<()> {
        if !storage_offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Ok(());
        }
        let index = self.hnsw.load();
        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let VectorRepresentations::Full(vec) = &node.vector {
                index.add_node(
                    node.id,
                    node.bitset.clone(),
                    VectorRepresentations::Full(vec.clone()),
                    storage_offset,
                )?;
                return Ok(());
            }
        }
        index.add_node(
            node.id,
            node.bitset.clone(),
            VectorRepresentations::None,
            storage_offset,
        )?;
        Ok(())
    }

    /// Update the HNSW index entry for a node with its current vector and storage offset.
    pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) -> Result<()> {
        if !storage_offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Ok(());
        }
        let _guard = self.acquire_insert_lock("acquire insert_lock in refresh_index")?;
        self.apply_index_entry_unlocked(node, storage_offset)
    }

    /// Move a hot node to cold tier, persist metadata, and release mmap pages.
    ///
    /// `policy` indicates whether the caller already holds `insert_lock`
    /// (insert paths use [`LockPolicy::AssumeHeld`]). In that case the HNSW
    /// entry is applied via `apply_index_entry_unlocked` instead of
    /// re-acquiring the non-reentrant lock, which would otherwise time out
    /// after `insert_lock_timeout_ms`.
    fn consolidate_node_inner(&self, node: &UnifiedNode, policy: LockPolicy) -> Result<()> {
        self.ensure_writable()?;

        // FND-02-M3: when the caller does not already hold insert_lock
        // (eviction/consolidation public path), acquire it for the WHOLE
        // critical section instead of only around the HNSW re-add.
        // delete() removes the node from HNSW and backend under insert_lock;
        // holding it here from the start closes the delete↔consolidate race
        // where a delete between backend.put and the HNSW re-add left a zombie
        // index entry (A), or a completed delete was silently resurrected (B).
        let _guard = match policy {
            LockPolicy::AssumeHeld => None,
            LockPolicy::Acquire => {
                Some(self.acquire_insert_lock("acquire insert_lock in consolidate_node")?)
            }
        };

        // FND-02-M3: version check under insert_lock — if a concurrent
        // delete() already removed the node, skip consolidation entirely:
        // re-persisting metadata / re-adding the entry would resurrect a node
        // the caller deleted. delete() removes the node from volatile
        // under insert_lock, so its presence here is the liveness signal.
        // (Checked against volatile, not HNSW: batch_insert with
        // InsertMode::Rebuild puts Hot nodes in the cache without indexing
        // them, so an HNSW check would strand them in cache forever.)
        if !self.cache.volatile.read().contains_key(&node.id) {
            return Ok(());
        }

        let mut persisted = node.clone();
        persisted.tier = NodeTier::Cold;

        let key = persisted.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: persisted.relational.clone(),
            edges: persisted.edges.clone(),
            created_by_txn: 0, // consolidation is pre-MVCC
            deleted_by_txn: None,
        };
        let metadata_val = postcard::to_allocvec(&metadata).map_err(Error::serialization)?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        let offset = self.hnsw.load().storage_offset_of(node.id).unwrap_or(0);
        // insert_lock is held in both branches now (caller's or ours), so the
        // HNSW entry is always applied without re-acquiring the non-reentrant
        // lock (FND-02-M3).
        self.apply_index_entry_unlocked(&persisted, offset)?;

        if offset > 0 {
            let (seg_id, local_off) = crate::lsm::unpack_offset(offset);
            let vstore = self.vector_store[seg_id as usize].read();
            let mmap = vstore.mmap_bytes();
            let vector_size = match &persisted.vector {
                VectorRepresentations::Full(v) => v.len() * 4,
                VectorRepresentations::MmapFull(mmap_opt) => {
                    mmap_opt.as_ref().map_or(0, |m| m.len() / 4)
                }
                VectorRepresentations::Binary(b) => b.len() * 8,
                VectorRepresentations::Turbo(t) => t.len(),
                VectorRepresentations::SQ8(d, _) => d.len() + 4,
                VectorRepresentations::None => 0,
            };
            let vector_size_aligned = (vector_size + 63) & !63;
            let offset_usize = local_off as usize;
            if offset_usize + vector_size_aligned <= mmap.len() && vector_size_aligned > 0 {
                // SAFETY: the bounds check above guarantees the range is within
                // the mmap region. `release_mmap_vector` expects the caller to
                // ensure this (per its own `# Safety` doc).
                unsafe {
                    crate::storage::vfile::release_mmap_vector(
                        mmap.as_ptr(),
                        offset_usize,
                        vector_size_aligned,
                    );
                }
            }
        }

        {
            let mut guard = self.cache.volatile.write();
            guard.remove(&node.id);
        }

        Ok(())
    }

    /// Move a hot node to cold tier, persist metadata, and release mmap pages.
    /// Acquires `insert_lock` internally; for callers that already hold it, use
    /// `Self::consolidate_node_locked`.
    pub fn consolidate_node(&self, node: &UnifiedNode) -> Result<()> {
        self.consolidate_node_inner(node, LockPolicy::Acquire)
    }

    /// Same as [`Self::consolidate_node`] but assumes the caller already holds
    /// `insert_lock` (insert paths). Applies the HNSW entry without
    /// re-acquiring the non-reentrant lock. Callers must NOT hold
    /// `volatile`'s write guard — this takes it itself.
    pub(crate) fn consolidate_node_locked(&self, node: &UnifiedNode) -> Result<()> {
        self.consolidate_node_inner(node, LockPolicy::AssumeHeld)
    }

    /// Evict a fraction of hot nodes from the volatile cache by lowest eviction score.
    pub fn evict_cold_nodes(&self, ratio: f64) -> Result<EvictionReport> {
        self.evict_cold_nodes_with_reason(ratio, EvictionReason::Periodic)
    }

    /// Evict a fraction of hot nodes with a specific reason for metrics.
    ///
    /// `policy` propagates to [`Self::consolidate_node_inner`]: when the
    /// caller already holds `insert_lock` (insert paths), the eviction runs the
    /// locked variants so the non-reentrant lock is never re-acquired.
    fn evict_cold_nodes_inner(
        &self,
        ratio: f64,
        reason: EvictionReason,
        policy: LockPolicy,
    ) -> Result<EvictionReport> {
        self.ensure_writable()?;
        let ratio = ratio.clamp(0.0, 1.0);
        if ratio <= 0.0 {
            return Ok(EvictionReport {
                evicted: 0,
                scanned: 0,
                reason,
            });
        }

        let candidates: Vec<UnifiedNode> = {
            let guard = self.cache.volatile.read();
            guard
                .values()
                .filter(|n| n.tier == NodeTier::Hot)
                .cloned()
                .collect()
        };

        if candidates.is_empty() {
            return Ok(EvictionReport {
                evicted: 0,
                scanned: 0,
                reason,
            });
        }

        // PERF-10: if MemoryGovernor is installed, record OOM preemptively
        if reason == EvictionReason::Oom {
            if let Some(ref gov) = self.memory_governor {
                gov.record_oom();
            }
        }

        let target = (candidates.len() as f64 * ratio).max(1.0) as usize;
        let scanned = candidates.len();
        let weights = self.config.eviction_weights();

        let mut scored: Vec<(f64, UnifiedNode)> = candidates
            .into_iter()
            .map(|n| {
                let score = n.eviction_score(&weights);
                (score, n)
            })
            .collect();
        scored.retain(|(score, _)| !score.is_nan());
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut bytes_freed: u64 = 0;
        let mut evicted = 0;
        for (_score, node) in scored.iter().take(target) {
            let result = match policy {
                LockPolicy::AssumeHeld => self.consolidate_node_locked(node),
                LockPolicy::Acquire => self.consolidate_node(node),
            };
            if result.is_ok() {
                bytes_freed += node.size() as u64;
                evicted += 1;
            }
        }

        crate::metrics::record_eviction(evicted as u64, scanned as u64, bytes_freed);

        Ok(EvictionReport {
            evicted,
            scanned,
            reason,
        })
    }

    /// Evict a fraction of hot nodes with a specific reason for metrics.
    /// Acquires `insert_lock` internally; for callers that already hold it, use
    /// `Self::evict_cold_nodes_with_reason_locked`.
    pub fn evict_cold_nodes_with_reason(
        &self,
        ratio: f64,
        reason: EvictionReason,
    ) -> Result<EvictionReport> {
        self.evict_cold_nodes_inner(ratio, reason, LockPolicy::Acquire)
    }

    /// Same as [`Self::evict_cold_nodes_with_reason`] but assumes the caller
    /// already holds `insert_lock` (insert paths). Callers must also NOT hold
    /// `volatile`'s write guard — the eviction reads and mutates the
    /// cache itself, and the RwLock is not reentrant.
    pub(crate) fn evict_cold_nodes_with_reason_locked(
        &self,
        ratio: f64,
        reason: EvictionReason,
    ) -> Result<EvictionReport> {
        self.evict_cold_nodes_inner(ratio, reason, LockPolicy::AssumeHeld)
    }

    /// Rebuild the HNSW vector index from scratch by scanning all nodes in the File.
    pub fn rebuild_vector_index(&self) -> Result<IndexRebuildReport> {
        self.ensure_writable()?;

        // flush() acquires insert_lock itself (ERR-010 checkpoint). It must run
        // BEFORE we take insert_lock: the lock is non-reentrant, so flushing
        // while holding the guard deadlocks native builds (try_lock_for times
        // out) and panics on wasm (parking_lot's timeout computes Instant,
        // which is not implemented on this platform).
        self.flush()?;

        let _guard = self.acquire_insert_lock("acquire insert_lock in rebuild_vector_index")?;

        let index_path = self.data_dir.join("vector_index.bin");
        let mut rebuilt = {
            let hnsw = self.hnsw.load();
            hnsw.fresh_box(index_path.clone())?
        };

        let report = {
            // Rebuild from L0 (primary segment). For L1+ data, the offsets
            // will be updated when the HNSW is populated from all levels.
            let vstore = self.vector_store[0].read();
            crate::storage::archive::rebuild_hnsw_from_vstore_with_segment(
                &mut *rebuilt,
                &vstore,
                index_path.clone(),
                Some(0),
            )?
        };

        if rebuilt.is_mmap() {
            rebuilt.sync_to_mmap().map_err(Error::Io)?;
        } else {
            rebuilt
                .persist_to_file(
                    rebuilt
                        .mmap_path()
                        .unwrap_or(&self.data_dir.join("vector_index.bin")),
                )
                .map_err(Error::Io)?;
        }

        self.hnsw.store(Arc::new(rebuilt));

        crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);

        Ok(report)
    }

    /// Compacts the File by rewriting nodes in BFS order of the HNSW graph.
    pub fn compact_layout_bfs(&self) -> Result<u64> {
        self.ensure_writable()?;

        // flush() acquires insert_lock itself (ERR-010 checkpoint). It must run
        // BEFORE we take insert_lock: the lock is non-reentrant, so flushing
        // while holding the guard deadlocks native builds (try_lock_for times
        // out) and panics on wasm (parking_lot's timeout computes Instant,
        // which is not implemented on this platform).
        self.flush()?;

        let _guard_insert =
            self.acquire_insert_lock("acquire insert_lock in compact_layout_bfs")?;

        let started = Instant::now();

        // ponytail: compact_layout_bfs compacts L0 only. Multi-level BFS
        // compaction is handled by compact_level().
        let mut vstore = self.vector_store[0].write();
        let hnsw = self.hnsw.load();

        let entry_point_id = match hnsw.entry_point() {
            Some(ep) => ep,
            None => {
                tracing::info!("compact_layout_bfs: empty index, skipping");
                return Ok(0);
            }
        };

        let header_size = std::mem::size_of::<crate::node::DiskNodeHeader>() as u64;

        // &***: Guard → Arc → Box → dyn (one deref per wrapper).
        let bfs_order = crate::storage::archive::traverse_graph(&***hnsw, entry_point_id);

        let (new_offset_map, new_file_size) = crate::storage::archive::compact_layout(
            &mut vstore,
            // &***: Guard → Arc → Box → dyn (one deref per wrapper).
            &***hnsw,
            &bfs_order,
            header_size,
        )?;
        let nodes_compacted = new_offset_map.len() as u64;

        // &***: Guard → Arc → Box → dyn (one deref per wrapper).
        crate::storage::archive::reindex_nodes(&***hnsw, &new_offset_map);

        drop(hnsw);

        let elapsed_ms = started.elapsed().as_millis() as u64;
        tracing::info!(
            nodes_compacted = nodes_compacted,
            new_file_size = new_file_size,
            elapsed_ms = elapsed_ms,
            "compact_layout_bfs: File compactado en orden BFS"
        );

        drop(vstore);
        self.save_vector_index()?;

        Ok(nodes_compacted)
    }

    /// Create a checkpoint (live snapshot) of the backend for backup purposes.
    ///
    /// Only backends implementing the [`Snapshotable`](crate::backend::Snapshotable) role support this.
    /// Others get the same honest `backend_error` as before — non-support
    /// is declared in the type (`as_snapshotable() == None`), not
    /// discovered at runtime.
    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let Some(snapshotable) = self.backend.as_snapshotable() else {
            return Err(Error::backend_error(format!(
                "Checkpoint (live snapshot) is not supported by the {:?} backend. \
                Live backups are not available natively. Please use filesystem-level snapshots (e.g., EBS, ZFS, LVM) \
                or perform a cold backup by safely shutting down the database process and copying the data directory.",
                self.backend_kind()
            )));
        };

        let cfg = Config::default();
        let mut save_path = std::path::PathBuf::from("./vantadb_snapshots");
        if let Some(ref backup_dir) = cfg.storage_cfg().backup_dir {
            save_path = backup_dir.clone();
        }
        save_path.push(timestamp_name);

        snapshotable.checkpoint(&save_path)
    }

    /// Run periodic quantization maintenance (PERF-09).
    ///
    /// Scans tracked nodes and auto-transitions cold f32 → SQ8 and hot SQ8 → f32.
    pub fn run_quantization_maintenance(&self) -> Result<QuantizationMaintenanceReport> {
        let mut quantized = 0u64;
        let mut promoted = 0u64;
        let mut scanned = 0u64;

        // Tick the governor
        self.quantization_governor.tick();

        let actions = {
            let hnsw = self.hnsw.load();
            self.quantization_governor
                .collect_actions(|node_id| hnsw.is_sq8_vector(node_id))
        };

        if actions.is_empty() {
            return Ok(QuantizationMaintenanceReport {
                scanned: 0,
                quantized: 0,
                promoted: 0,
            });
        }

        for (node_id, action) in actions {
            scanned += 1;
            match action {
                QuantizationAction::Quantize => {
                    // Read the node, quantize its vector, update in HNSW
                    if let Ok(Some(mut node)) = self.get(node_id) {
                        if let VectorRepresentations::Full(vec) = &node.vector {
                            let (packed, scale) =
                                crate::vector::governor::QuantizationGovernor::quantize_vector(vec);
                            node.vector = VectorRepresentations::SQ8(packed, scale);
                            let offset = self.hnsw.load().storage_offset_of(node_id).unwrap_or(0);
                            let _guard = self.acquire_insert_lock(
                                "acquire insert_lock in quantization maintenance",
                            )?;
                            let hnsw = self.hnsw.load();
                            hnsw.add_node(
                                node_id,
                                node.bitset.clone(),
                                node.vector.clone(),
                                offset,
                            )?;
                            crate::metrics::record_quantization();
                            self.quantization_governor.reset(node_id);
                            quantized += 1;
                        }
                    }
                }
                QuantizationAction::Promote => {
                    // Read the node, promote its vector, update in HNSW
                    if let Ok(Some(mut node)) = self.get(node_id) {
                        if let VectorRepresentations::SQ8(data, scale) = &node.vector {
                            let vec = crate::vector::governor::QuantizationGovernor::promote_vector(
                                data, *scale,
                            );
                            node.vector = VectorRepresentations::Full(vec);
                            let offset = self.hnsw.load().storage_offset_of(node_id).unwrap_or(0);
                            let _guard = self.acquire_insert_lock(
                                "acquire insert_lock in quantization maintenance",
                            )?;
                            let hnsw = self.hnsw.load();
                            hnsw.add_node(
                                node_id,
                                node.bitset.clone(),
                                node.vector.clone(),
                                offset,
                            )?;
                            crate::metrics::record_promotion();
                            self.quantization_governor.reset(node_id);
                            promoted += 1;
                        }
                    }
                }
                QuantizationAction::None => {}
            }
        }

        Ok(QuantizationMaintenanceReport {
            scanned,
            quantized,
            promoted,
        })
    }

    /// Purge tombstoned nodes from the HNSW index.
    ///
    /// Scans all HNSW nodes, reads each node's File header, and removes
    /// any node whose header has `FLAG_TOMBSTONE` set from the graph index.
    /// The File layout is **not** rewritten — call `merge_segments()`
    /// afterwards if you need to reclaim storage bytes.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn vacuum(&self) -> Result<VacuumReport> {
        self.ensure_writable()?;

        let started = Instant::now();
        // ponytail: vacuum scans L0 headers. Multi-level vacuum is handled
        // by the compact_level path.
        let vstore = self.vector_store[0].read();
        let hnsw = self.hnsw.load();

        let entries = hnsw.scan_entries();
        let tombstone_ids: Vec<u128> = entries
            .iter()
            .filter_map(|(id, storage_offset, _)| {
                let header = vstore.read_header(*storage_offset)?;
                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        let scanned = entries.len() as u64;
        let removed_count = tombstone_ids.len() as u64;

        if removed_count == 0 {
            let elapsed_ms = started.elapsed().as_millis() as u64;
            tracing::info!(scanned, "vacuum: no tombstones found");
            return Ok(VacuumReport {
                scanned_nodes: scanned,
                removed_nodes: 0,
                reclaimed_bytes: 0,
                duration_ms: elapsed_ms,
                success: true,
            });
        }

        {
            for id in &tombstone_ids {
                hnsw.remove_node(*id);
                hnsw.decrement_total_nodes(1);
                if let Some(ref scalar) = self.scalar_index {
                    scalar.remove_node(*id);
                }
            }

            // If the entry point was removed, find a replacement
            if let Some(ep) = hnsw.entry_point() {
                if !hnsw.contains_node(ep) {
                    if let Some(new_ep) = hnsw.find_new_entry_point() {
                        hnsw.set_entry_point(new_ep);
                    }
                }
            }
        }

        // ponytail: estimate reclaimed bytes — HnswNode overhead + vec + neighbor edges.
        // This is an upper bound; actual mmap pages won't be freed until compaction.
        let reclaimed_bytes = tombstone_ids.len() as u64 * 512;

        drop(vstore);

        let elapsed_ms = started.elapsed().as_millis() as u64;
        tracing::info!(
            scanned,
            removed = removed_count,
            reclaimed_bytes,
            elapsed_ms,
            "vacuum: tombstones purged from HNSW index"
        );

        Ok(VacuumReport {
            scanned_nodes: scanned,
            removed_nodes: removed_count,
            reclaimed_bytes,
            duration_ms: elapsed_ms,
            success: true,
        })
    }

    /// Merge (compact) the File if tombstone fragmentation exceeds the
    /// configured threshold.
    ///
    /// Delegates to [`compact_layout_bfs`](crate::storage::StorageEngine::compact_layout_bfs) which rewrites the File in
    /// BFS order of the HNSW graph, skipping tombstoned nodes.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn merge_segments(&self) -> Result<MergeReport> {
        self.ensure_writable()?;

        let started = Instant::now();
        let entries = self.hnsw.load().scan_entries();
        let total_nodes = entries.len();
        if total_nodes == 0 {
            let elapsed_ms = started.elapsed().as_millis() as u64;
            tracing::info!("merge_segments: empty index, skipping");
            return Ok(MergeReport {
                segments_before: 1,
                segments_after: 1,
                saved_bytes: 0,
                duration_ms: elapsed_ms,
                success: true,
            });
        }

        // ponytail: merge_segments reads L0 headers to count tombstones.
        let tombstone_count = entries
            .iter()
            .filter(|(_, storage_offset, _)| {
                let (seg_id, local_off) = crate::lsm::unpack_offset(*storage_offset);
                if let Some(vs) = self.vector_store.get(seg_id as usize) {
                    let guard = vs.read();
                    if let Some(h) = guard.read_header(local_off) {
                        (h.flags & FLAG_TOMBSTONE) != 0
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .count();

        let frag_pct = tombstone_count as f32 / total_nodes as f32 * 100.0;
        let threshold = self.config.segment_optimizer.vacuum_threshold_pct;

        if frag_pct < threshold {
            let elapsed_ms = started.elapsed().as_millis() as u64;
            tracing::info!(
                frag_pct = frag_pct as u32,
                threshold = threshold,
                "merge_segments: fragmentation below threshold, skipping"
            );
            return Ok(MergeReport {
                segments_before: 1,
                segments_after: 1,
                saved_bytes: 0,
                duration_ms: elapsed_ms,
                success: true,
            });
        }

        let file_size_before = {
            // ponytail: compact L0 only
            let vs = self.vector_store[0].read();
            vs.mmap_bytes().len() as u64
        };

        // compact_layout_bfs returns nodes_compacted count
        let nodes_compacted = self.compact_layout_bfs().unwrap_or(0);

        let file_size_after = {
            // ponytail: compact L0 only
            let vs = self.vector_store[0].read();
            vs.mmap_bytes().len() as u64
        };

        let saved_bytes = file_size_before.saturating_sub(file_size_after);
        let elapsed_ms = started.elapsed().as_millis() as u64;

        tracing::info!(
            frag_pct = frag_pct as u32,
            nodes_compacted,
            saved_bytes,
            elapsed_ms,
            "merge_segments: File compacted"
        );

        Ok(MergeReport {
            segments_before: 1,
            segments_after: 1,
            saved_bytes,
            duration_ms: elapsed_ms,
            success: true,
        })
    }

    /// Scans all HNSW nodes and removes orphan links — neighbor IDs that
    /// point to nodes that no longer exist in the index.
    ///
    /// This repairs the graph after delete operations, which remove a node
    /// from `self.nodes` but leave references in the neighbor lists of
    /// surviving nodes. Over time, orphan links degrade search quality.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn fresh_hnsw(&self) -> Result<FreshHnswReport> {
        self.ensure_writable()?;

        let hnsw = self.hnsw.load();
        let report = hnsw.repair_orphan_links();

        tracing::info!(
            scanned = report.scanned_nodes,
            layers = report.total_layers,
            repaired = report.repaired_links,
            elapsed_ms = report.duration_ms,
            "fresh_hnsw: orphan links repaired"
        );

        Ok(report)
    }

    /// Check whether a given LSM level should be compacted.
    ///
    /// Returns `true` if the segment at `level` exceeds its configured max size
    /// or its tombstone ratio exceeds the threshold.
    fn should_compact_level(&self, level: u8) -> bool {
        if level as usize >= self.vector_store.len() {
            return false;
        }
        let guard = self.vector_store[level as usize].read();
        let seg_size = guard.write_cursor;
        let config = &self.config.segment_optimizer.lsm;
        let (max_size, tombstone_threshold) = match level {
            0 => (config.l0_max_size, config.l0_tombstone_threshold),
            1 => (config.l1_max_size, config.l1_tombstone_threshold),
            // L2 (cold) promotes to L3 (archive) only when the archive tier is
            // enabled; otherwise L2 is the deepest tier.
            2 => {
                if !config.tier.archive {
                    return false;
                }
                (config.l2_max_size, config.l2_tombstone_threshold)
            }
            // L3 is the terminal tier: nothing above it to promote into.
            _ => return false,
        };
        if seg_size >= max_size {
            return true;
        }
        // Count tombstones from HNSW for this level
        let mut tombstone_count = 0u64;
        let mut total_count = 0u64;
        for (_, storage_offset, flags) in self.hnsw.load().scan_entries() {
            let (seg_id, _local_off) = crate::lsm::unpack_offset(storage_offset);
            if seg_id as u8 == level {
                total_count += 1;
                if (flags & FLAG_TOMBSTONE) != 0 {
                    tombstone_count += 1;
                }
            }
        }
        if total_count == 0 {
            return false;
        }
        let ratio = tombstone_count as f32 / total_count as f32;
        // ponytail: compact_level uses size and tombstone ratio to decide
        ratio >= tombstone_threshold
    }

    /// Compact a single LSM level by promoting live nodes to the next tier.
    ///
    /// Reads live (non-tombstone) nodes from `level` using `self.get()`,
    /// rewrites them to `level+1` using `write_node_to_vstore()`, updates
    /// HNSW offset references, then truncates the source level's File.
    ///
    /// Chain: L0(hot) -> L1(warm) -> L2(cold) -> L3(archive). L3 participates
    /// only when `LsmConfig::tier.archive` is enabled (see STORAGE-TIERS.md).
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn compact_level(&self, level: u8) -> Result<LsmReport> {
        let started = Instant::now();
        let target_level = level + 1;

        // L3 is the terminal tier — there is no L4 to promote into. A direct
        // compact_level(3) call (or a single-segment store) is a no-op, never
        // an out-of-bounds access.
        if target_level as usize >= self.vector_store.len() {
            return Ok(LsmReport {
                level,
                nodes_promoted: 0,
                reclaimed_bytes: 0,
                duration_ms: started.elapsed().as_millis() as u64,
                success: true,
            });
        }

        // All 4 LSM levels are pre-allocated at init (SegmentRegistry::open_or_create),
        // so vector_store has entries for L0..L3 — no unsafe growth needed.

        // Collect live node IDs in this level
        let live_ids: Vec<u128> = self
            .hnsw
            .load()
            .scan_entries()
            .into_iter()
            .filter_map(|(id, storage_offset, flags)| {
                let (seg_id, _) = crate::lsm::unpack_offset(storage_offset);
                if seg_id as u8 == level && (flags & FLAG_TOMBSTONE) == 0 {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        if live_ids.is_empty() {
            return Ok(LsmReport {
                level,
                nodes_promoted: 0,
                reclaimed_bytes: 0,
                duration_ms: started.elapsed().as_millis() as u64,
                success: true,
            });
        }

        let mut nodes_promoted = 0u64;
        let mut new_offsets: std::collections::HashMap<u128, u64> =
            std::collections::HashMap::with_capacity(live_ids.len());

        // Read each node via self.get() (segment-aware), write to target
        for node_id in &live_ids {
            let node = match self.get(*node_id) {
                Ok(Some(n)) => n,
                _ => continue,
            };
            let mut tgt = self.vector_store[target_level as usize].write();
            let new_raw_off = crate::storage::ops::write_node_to_vstore(&mut tgt, &node)?;
            let new_packed_off = crate::lsm::pack_offset(target_level, new_raw_off);
            new_offsets.insert(*node_id, new_packed_off);
            nodes_promoted += 1;
        }

        // Update HNSW offsets
        {
            let hnsw = self.hnsw.load();
            for (node_id, new_off) in &new_offsets {
                hnsw.set_storage_offset(*node_id, *new_off);
            }
        }

        // Truncate source segment: reset write cursor past alignment header
        let reclaimed_bytes = {
            let mut src = self.vector_store[level as usize].write();
            let size_before = src.write_cursor;
            src.write_cursor = STORAGE_ALIGNMENT;
            src.flush()?;
            size_before
        };

        let elapsed_ms = started.elapsed().as_millis() as u64;
        tracing::info!(
            level,
            target_level,
            nodes_promoted,
            reclaimed_bytes,
            elapsed_ms,
            "compact_level: done"
        );
        Ok(LsmReport {
            level,
            nodes_promoted,
            reclaimed_bytes,
            duration_ms: elapsed_ms,
            success: true,
        })
    }

    pub(super) fn pipeline_vacuum_step(
        &self,
        mode: PipelineMode,
        all_ok: &mut bool,
    ) -> Option<VacuumReport> {
        if !matches!(mode, PipelineMode::Full | PipelineMode::VacuumOnly) {
            return None;
        }
        match self.vacuum() {
            Ok(r) => {
                tracing::info!(removed = r.removed_nodes, "pipeline: vacuum ok");
                Some(r)
            }
            Err(e) => {
                tracing::error!(error = %e, "pipeline: vacuum failed");
                *all_ok = false;
                None
            }
        }
    }
    pub(super) fn pipeline_fresh_hnsw_step(
        &self,
        mode: PipelineMode,
        all_ok: &mut bool,
    ) -> Option<FreshHnswReport> {
        if !matches!(mode, PipelineMode::Full | PipelineMode::FreshHnswOnly) {
            return None;
        }
        match self.fresh_hnsw() {
            Ok(r) => {
                tracing::info!(repaired = r.repaired_links, "pipeline: fresh_hnsw ok");
                Some(r)
            }
            Err(e) => {
                tracing::error!(error = %e, "pipeline: fresh_hnsw failed");
                *all_ok = false;
                None
            }
        }
    }
    pub(super) fn pipeline_compact_one_level(
        &self,
        level: u8,
        all_ok: &mut bool,
    ) -> Option<LsmReport> {
        if !self.should_compact_level(level) {
            tracing::info!(level, "pipeline: compact skip (below threshold)");
            return None;
        }
        match self.compact_level(level) {
            Ok(r) => {
                tracing::info!(level, promoted = r.nodes_promoted, "pipeline: compact ok");
                Some(r)
            }
            Err(e) => {
                tracing::error!(level, error = %e, "pipeline: compact failed");
                *all_ok = false;
                None
            }
        }
    }
    pub(super) fn pipeline_lsm_steps(
        &self,
        mode: PipelineMode,
        all_ok: &mut bool,
    ) -> Vec<LsmReport> {
        if !matches!(
            mode,
            PipelineMode::Full | PipelineMode::CompactOnly | PipelineMode::CompactL0Only
        ) {
            return Vec::new();
        }
        let max_level = match mode {
            PipelineMode::CompactL0Only => 0u8,
            _ => 2u8.min(self.vector_store.len().saturating_sub(1) as u8),
        };
        let mut out = Vec::new();
        for level in 0..=max_level {
            if let Some(r) = self.pipeline_compact_one_level(level, all_ok) {
                out.push(r);
            }
        }
        out
    }
    pub(super) fn pipeline_merge_step(
        &self,
        mode: PipelineMode,
        all_ok: &mut bool,
    ) -> Option<MergeReport> {
        if !matches!(mode, PipelineMode::Full | PipelineMode::MergeOnly) {
            return None;
        }
        match self.merge_segments() {
            Ok(r) => {
                tracing::info!(saved = r.saved_bytes, "pipeline: merge ok");
                Some(r)
            }
            Err(e) => {
                tracing::error!(error = %e, "pipeline: merge failed");
                *all_ok = false;
                None
            }
        }
    }
    pub(super) fn pipeline_index_step(
        &self,
        mode: PipelineMode,
        all_ok: &mut bool,
    ) -> Option<IndexRebuildReport> {
        if !matches!(mode, PipelineMode::Full | PipelineMode::IndexOnly) {
            return None;
        }
        match self.rebuild_vector_index() {
            Ok(r) => {
                tracing::info!(indexed = r.indexed_vectors, "pipeline: reindex ok");
                Some(r)
            }
            Err(e) => {
                tracing::error!(error = %e, "pipeline: reindex failed");
                *all_ok = false;
                None
            }
        }
    }
    pub(super) fn finish_pipeline_report(
        mode: PipelineMode,
        pipeline_start: Instant,
        vacuum: Option<VacuumReport>,
        fresh_hnsw: Option<FreshHnswReport>,
        merge: Option<MergeReport>,
        index: Option<IndexRebuildReport>,
        lsm_reports: Vec<LsmReport>,
        all_ok: bool,
    ) -> PipelineReport {
        let total_duration_ms = pipeline_start.elapsed().as_millis() as u64;
        tracing::info!(
            ?mode,
            total_duration_ms,
            success = all_ok,
            "pipeline: finished"
        );
        PipelineReport {
            vacuum,
            fresh_hnsw,
            merge,
            index,
            lsm: (!lsm_reports.is_empty()).then_some(lsm_reports),
            total_duration_ms,
            success: all_ok,
        }
    }

    /// Run the segment optimizer pipeline according to `mode`.
    ///
    /// Phases execute in order: Vacuum → FreshHNSW → Merge → Reindex.
    /// A phase failure is logged but does **not** abort later phases. The
    /// FreshHNSW phase places after Vacuum so that tombstoned nodes are
    /// already purged — fewer orphan links to scan.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn run_pipeline(&self, mode: PipelineMode) -> Result<PipelineReport> {
        self.ensure_writable()?;

        let pipeline_start = Instant::now();
        let mut all_ok = true;
        tracing::info!(?mode, "pipeline: starting");
        let vacuum_report = self.pipeline_vacuum_step(mode, &mut all_ok);

        let fresh_hnsw_report = self.pipeline_fresh_hnsw_step(mode, &mut all_ok);

        let lsm_reports = self.pipeline_lsm_steps(mode, &mut all_ok);

        let merge_report = self.pipeline_merge_step(mode, &mut all_ok);

        let index_report = self.pipeline_index_step(mode, &mut all_ok);
        Ok(Self::finish_pipeline_report(
            mode,
            pipeline_start,
            vacuum_report,
            fresh_hnsw_report,
            merge_report,
            index_report,
            lsm_reports,
            all_ok,
        ))
    }

    /// Recover archived nodes from TombstoneStorage that belonged to the given summary node.
    pub fn recover_archived_nodes(&self, summary_id: u128) -> Result<Vec<UnifiedNode>> {
        self.ensure_writable()?;
        let entries = self.backend.scan(BackendPartition::TombstoneStorage)?;

        let mut recovered = Vec::new();
        let belonged_to_id = self.intern_label("belonged_to");
        for (_k, v) in &entries {
            let decode_res =
                crate::storage::ops::deserialize_node_payload::<UnifiedNode>(v, "archived node");
            if let Ok(mut node) = decode_res {
                if node
                    .edges
                    .iter()
                    .any(|e| e.target == summary_id && e.label_id == belonged_to_id)
                {
                    node.flags.set(crate::node::NodeFlags::ACTIVE);
                    node.flags.set(crate::node::NodeFlags::RECOVERED);
                    node.tier = NodeTier::Hot;
                    self.insert(&node)?;
                    recovered.push(node);
                }
            }
        }
        Ok(recovered)
    }
}
