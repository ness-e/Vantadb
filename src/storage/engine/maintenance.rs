//! Maintenance operations: refresh, consolidate, evict, rebuild, compact, flush, WAL.

use std::fs::OpenOptions;
use std::sync::Arc;
use web_time::Instant;

use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{NodeTier, UnifiedNode, VectorRepresentations};
use crate::storage::engine::{
    EvictionReason, EvictionReport, FreshHnswReport, IndexRebuildReport, LsmReport, MergeReport,
    PipelineMode, PipelineReport, QuantizationMaintenanceReport, StorageEngine, VacuumReport,
    FLAG_TOMBSTONE, STORAGE_ALIGNMENT,
};
use crate::storage::ops::NodeMetadata;
use crate::storage::vfile::MmapMut;
use crate::vector::governor::QuantizationAction;

impl StorageEngine {
    /// Check tombstone fragmentation and log a warning if it exceeds 20%.
    pub fn trigger_compaction(&self) -> Result<()> {
        let vstore = self.vector_store[0].write();
        let hnsw = self.hnsw.load();

        let tombstone_count = hnsw
            .nodes
            .iter()
            .filter(|r| {
                let n = r.value();
                if let Some(h) = vstore.read_header(n.storage_offset) {
                    (h.flags & FLAG_TOMBSTONE) != 0
                } else {
                    false
                }
            })
            .count();

        let total_nodes = hnsw.nodes.len();
        if total_nodes > 0 && (tombstone_count as f32 / total_nodes as f32) > 0.20 {
            tracing::warn!(
                tombstone_pct = (tombstone_count as f32 / total_nodes as f32 * 100.0) as u32,
                "Fragmentation >20% — offline compaction triggered"
            );
        }

        Ok(())
    }

    /// Flush all pending writes: backend, vector store, WAL checkpoint, and vector index.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn flush(&self) -> Result<()> {
        // Drain pending HNSW mutations before checkpointing
        self.flush_pending_hnsw()?;
        self.ensure_writable()?;
        self.backend.flush()?;
        for vs in &self.vector_store {
            vs.read().flush()?;
        }

        let current_wal_seq = self
            .wal
            .as_ref()
            .map(|s| s.total_record_count())
            .unwrap_or(0);

        if current_wal_seq > 0 {
            let seq_bytes =
                postcard::to_allocvec(&current_wal_seq).map_err(VantaError::serialization)?;
            self.backend.put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq_bytes,
            )?;
            self.backend.flush()?;
        }

        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("snapshot_serialize_fail", |_| {
                Err(crate::error::VantaError::IoError(std::io::Error::other(
                    "Simulated snapshot serialize I/O failure",
                )))
            });
        }
        self.save_vector_index()?;

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
        if let Some(rb) = hnsw.backend.mmap_resident_bytes() {
            resident_bytes = Some(resident_bytes.unwrap_or(0) + rb);
        }
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            hnsw.estimate_memory_bytes() as u64,
            resident_bytes,
            self.volatile_cache.read().len() as u64,
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

        if current.backend.is_mmap() {
            let data = current.serialize_to_bytes();
            let temp_path = index_path.with_extension("bin.tmp");

            let result = (|| -> std::io::Result<Arc<CPIndex>> {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&temp_path)?;
                file.set_len(data.len() as u64)?;

                // SAFETY: `file` is a newly created/truncated handle at `data.len()` bytes.
                // `MmapMut::map_mut` from memmap2 creates a writable mapping of matching size.
                // The mapped memory is immediately initialized via `copy_from_slice` below.
                let mut mapped = unsafe { MmapMut::map_mut(&file)? };
                mapped.copy_from_slice(&data);
                mapped.flush()?;

                let mut new_index =
                    CPIndex::deserialize_from_bytes(&mapped, false).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })?;

                new_index.backend = IndexBackend::MMapFile {
                    path: index_path.clone(),
                    mmap: Some(mapped),
                };

                drop(file);
                std::fs::rename(&temp_path, &index_path)?;
                Ok(Arc::new(new_index))
            })();

            match result {
                Ok(new_hnsw) => {
                    self.hnsw.store(new_hnsw);
                }
                Err(e) => {
                    return Err(VantaError::IoError(e));
                }
            }
        } else {
            current.persist_to_file(&index_path)?;
        }
        Ok(())
    }

    /// Update the HNSW index entry for a node with its current vector and storage offset.
    pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) -> Result<()> {
        if !storage_offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Ok(());
        }
        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let VectorRepresentations::Full(vec) = &node.vector {
                let _guard = self
                    .insert_lock
                    .try_lock_for(std::time::Duration::from_millis(
                        self.config.insert_lock_timeout_ms,
                    ))
                    .ok_or_else(|| VantaError::Timeout {
                        operation: "acquire insert_lock in refresh_index".into(),
                        duration_ms: self.config.insert_lock_timeout_ms,
                    })?;
                let index = self.hnsw.load();
                index.add(
                    node.id,
                    node.bitset.clone(),
                    VectorRepresentations::Full(vec.clone()),
                    storage_offset,
                );
                return Ok(());
            }
        }
        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| VantaError::Timeout {
                operation: "acquire insert_lock in refresh_index".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;
        let index = self.hnsw.load();
        index.add(
            node.id,
            node.bitset.clone(),
            VectorRepresentations::None,
            storage_offset,
        );
        Ok(())
    }

    /// Move a hot node to cold tier, persist metadata, and release mmap pages.
    pub fn consolidate_node(&self, node: &UnifiedNode) -> Result<()> {
        self.ensure_writable()?;
        let mut persisted = node.clone();
        persisted.tier = NodeTier::Cold;

        let key = persisted.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: persisted.relational.clone(),
            edges: persisted.edges.clone(),
            created_by_txn: 0, // consolidation is pre-MVCC
            deleted_by_txn: None,
        };
        let metadata_val = postcard::to_allocvec(&metadata).map_err(VantaError::serialization)?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        let offset = {
            let hnsw = self.hnsw.load();
            hnsw.nodes
                .get(&node.id)
                .map(|n| n.storage_offset)
                .unwrap_or(0)
        };
        self.refresh_index(&persisted, offset)?;

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
                    crate::index::release_mmap_vector(
                        mmap.as_ptr(),
                        offset_usize,
                        vector_size_aligned,
                    );
                }
            }
        }

        {
            let mut cache = self.volatile_cache.write();
            cache.remove(&node.id);
        }

        Ok(())
    }

    /// Evict a fraction of hot nodes from the volatile cache by lowest eviction score.
    pub fn evict_cold_nodes(&self, ratio: f64) -> Result<EvictionReport> {
        self.evict_cold_nodes_with_reason(ratio, EvictionReason::Periodic)
    }

    /// Evict a fraction of hot nodes with a specific reason for metrics.
    pub fn evict_cold_nodes_with_reason(
        &self,
        ratio: f64,
        reason: EvictionReason,
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
            let cache = self.volatile_cache.read();
            cache
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
            if self.consolidate_node(node).is_ok() {
                bytes_freed += node.memory_size() as u64;
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

    /// Rebuild the HNSW vector index from scratch by scanning all nodes in the VantaFile.
    pub fn rebuild_vector_index(&self) -> Result<IndexRebuildReport> {
        self.ensure_writable()?;

        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| VantaError::Timeout {
                operation: "acquire insert_lock in rebuild_vector_index".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;

        self.flush()?;

        let index_path = self.data_dir.join("vector_index.bin");
        let mut rebuilt = {
            let hnsw = self.hnsw.load();
            crate::storage::archive::fresh_index_like(&hnsw, index_path.clone())
        };

        let report = {
            // Rebuild from L0 (primary segment). For L1+ data, the offsets
            // will be updated when the HNSW is populated from all levels.
            let vstore = self.vector_store[0].read();
            crate::storage::archive::rebuild_hnsw_from_vstore_with_segment(
                &mut rebuilt,
                &vstore,
                index_path.clone(),
                Some(0),
            )?
        };

        if rebuilt.backend.is_mmap() {
            rebuilt.sync_to_mmap().map_err(VantaError::IoError)?;
        } else {
            rebuilt
                .persist_to_file(
                    rebuilt
                        .backend
                        .mmap_path()
                        .unwrap_or(&self.data_dir.join("vector_index.bin")),
                )
                .map_err(VantaError::IoError)?;
        }

        self.hnsw.store(Arc::new(rebuilt));

        crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);

        Ok(report)
    }

    /// Compacts the VantaFile by rewriting nodes in BFS order of the HNSW graph.
    pub fn compact_layout_bfs(&self) -> Result<u64> {
        self.ensure_writable()?;

        let _guard_insert = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| VantaError::Timeout {
                operation: "acquire insert_lock in compact_layout_bfs".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;

        self.flush()?;

        let started = Instant::now();

        // ponytail: compact_layout_bfs compacts L0 only. Multi-level BFS
        // compaction is handled by compact_level().
        let mut vstore = self.vector_store[0].write();
        let hnsw = self.hnsw.load();

        let entry_point_id = match hnsw.get_entry_point() {
            Some(ep) => ep,
            None => {
                tracing::info!("compact_layout_bfs: empty index, skipping");
                return Ok(0);
            }
        };

        let header_size = std::mem::size_of::<crate::node::DiskNodeHeader>() as u64;

        let bfs_order = crate::storage::archive::traverse_graph(&hnsw, entry_point_id);

        let (new_offset_map, new_file_size) =
            crate::storage::archive::compact_layout(&mut vstore, &hnsw, &bfs_order, header_size)?;
        let nodes_compacted = new_offset_map.len() as u64;

        crate::storage::archive::reindex_nodes(&hnsw, &new_offset_map);

        drop(hnsw);

        let elapsed_ms = started.elapsed().as_millis() as u64;
        tracing::info!(
            nodes_compacted = nodes_compacted,
            new_file_size = new_file_size,
            elapsed_ms = elapsed_ms,
            "compact_layout_bfs: VantaFile compactado en orden BFS"
        );

        drop(vstore);
        self.save_vector_index()?;

        Ok(nodes_compacted)
    }

    /// Create a checkpoint (live snapshot) of the backend for backup purposes.
    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        self.ensure_writable()?;
        if !self.supports_checkpoint() {
            return Err(VantaError::backend_error(format!(
                "Checkpoint (live snapshot) is not supported by the {:?} backend. \
                Live backups are not available natively. Please use filesystem-level snapshots (e.g., EBS, ZFS, LVM) \
                or perform a cold backup by safely shutting down the database process and copying the data directory.",
                self.backend_kind()
            )));
        }

        let mut save_path = std::path::PathBuf::from("./vantadb_snapshots");
        if let Ok(override_dir) = std::env::var("VANTA_BACKUP_DIR") {
            save_path = std::path::PathBuf::from(override_dir);
        }
        save_path.push(timestamp_name);

        self.backend.checkpoint(&save_path)
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
            self.quantization_governor.collect_actions(|node_id| {
                hnsw.nodes
                    .get(&node_id)
                    .map(|n| matches!(n.value().vec_data, VectorRepresentations::SQ8(..)))
            })
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
                            let offset = {
                                let hnsw = self.hnsw.load();
                                hnsw.nodes
                                    .get(&node_id)
                                    .map(|n| n.storage_offset)
                                    .unwrap_or(0)
                            };
                            let _guard = self
                                .insert_lock
                                .try_lock_for(std::time::Duration::from_millis(
                                    self.config.insert_lock_timeout_ms,
                                ))
                                .ok_or_else(|| VantaError::Timeout {
                                    operation: "acquire insert_lock in quantization maintenance"
                                        .into(),
                                    duration_ms: self.config.insert_lock_timeout_ms,
                                })?;
                            let hnsw = self.hnsw.load();
                            hnsw.add(node_id, node.bitset.clone(), node.vector.clone(), offset);
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
                            let offset = {
                                let hnsw = self.hnsw.load();
                                hnsw.nodes
                                    .get(&node_id)
                                    .map(|n| n.storage_offset)
                                    .unwrap_or(0)
                            };
                            let _guard = self
                                .insert_lock
                                .try_lock_for(std::time::Duration::from_millis(
                                    self.config.insert_lock_timeout_ms,
                                ))
                                .ok_or_else(|| VantaError::Timeout {
                                    operation: "acquire insert_lock in quantization maintenance"
                                        .into(),
                                    duration_ms: self.config.insert_lock_timeout_ms,
                                })?;
                            let hnsw = self.hnsw.load();
                            hnsw.add(node_id, node.bitset.clone(), node.vector.clone(), offset);
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
    /// Scans all HNSW nodes, reads each node's VantaFile header, and removes
    /// any node whose header has `FLAG_TOMBSTONE` set from the graph index.
    /// The VantaFile layout is **not** rewritten — call `merge_segments()`
    /// afterwards if you need to reclaim storage bytes.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn vacuum(&self) -> Result<VacuumReport> {
        self.ensure_writable()?;

        let started = Instant::now();
        // ponytail: vacuum scans L0 headers. Multi-level vacuum is handled
        // by the compact_level path.
        let vstore = self.vector_store[0].read();
        let hnsw = self.hnsw.load();

        let tombstone_ids: Vec<u128> = hnsw
            .nodes
            .iter()
            .filter_map(|entry| {
                let node = entry.value();
                let id = *entry.key();
                let header = vstore.read_header(node.storage_offset)?;
                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        let scanned = hnsw.nodes.len() as u64;
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

        for id in &tombstone_ids {
            hnsw.nodes.remove(id);
            hnsw.total_nodes
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            if let Some(ref scalar) = self.scalar_index {
                scalar.remove_node(*id);
            }
        }

        // ponytail: estimate reclaimed bytes — HnswNode overhead + vec + neighbor edges.
        // This is an upper bound; actual mmap pages won't be freed until compaction.
        let reclaimed_bytes = tombstone_ids.len() as u64 * 512;

        // If the entry point was removed, find a replacement
        if let Some(ep) = hnsw.get_entry_point() {
            if !hnsw.nodes.contains_key(&ep) {
                if let Some(new_ep) = hnsw.find_new_entry_point() {
                    hnsw.set_entry_point(new_ep);
                }
            }
        }

        drop(vstore);
        drop(hnsw);

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

    /// Merge (compact) the VantaFile if tombstone fragmentation exceeds the
    /// configured threshold.
    ///
    /// Delegates to [`compact_layout_bfs`] which rewrites the VantaFile in
    /// BFS order of the HNSW graph, skipping tombstoned nodes.
    #[tracing::instrument(skip(self), level = "info", err)]
    pub fn merge_segments(&self) -> Result<MergeReport> {
        self.ensure_writable()?;

        let started = Instant::now();
        let hnsw = self.hnsw.load();

        let total_nodes = hnsw.nodes.len();
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
        let tombstone_count = hnsw
            .nodes
            .iter()
            .filter(|r| {
                let n = r.value();
                let packed = n.storage_offset;
                let (seg_id, local_off) = crate::lsm::unpack_offset(packed);
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

        drop(hnsw);

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
            "merge_segments: VantaFile compacted"
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
        let hnsw = self.hnsw.load();
        let mut tombstone_count = 0u64;
        let mut total_count = 0u64;
        for entry in hnsw.nodes.iter() {
            let n = entry.value();
            let (seg_id, _local_off) = crate::lsm::unpack_offset(n.storage_offset);
            if seg_id as u8 == level {
                total_count += 1;
                if (n.flags & FLAG_TOMBSTONE) != 0 {
                    tombstone_count += 1;
                }
            }
        }
        drop(hnsw);
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
    /// HNSW offset references, then truncates the source level's VantaFile.
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

        let hnsw = self.hnsw.load();

        // Collect live node IDs in this level
        let live_ids: Vec<u128> = hnsw
            .nodes
            .iter()
            .filter_map(|entry| {
                let n = entry.value();
                let (seg_id, _) = crate::lsm::unpack_offset(n.storage_offset);
                if seg_id as u8 == level && (n.flags & FLAG_TOMBSTONE) == 0 {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        if live_ids.is_empty() {
            drop(hnsw);
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
        for (node_id, new_off) in &new_offsets {
            if let Some(mut node_ref) = hnsw.nodes.get_mut(node_id) {
                node_ref.storage_offset = *new_off;
            }
        }
        drop(hnsw);

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
        let mut vacuum_report: Option<VacuumReport> = None;
        let mut fresh_hnsw_report: Option<FreshHnswReport> = None;
        let mut merge_report: Option<MergeReport> = None;
        let mut index_report: Option<IndexRebuildReport> = None;
        let mut all_ok = true;

        tracing::info!(?mode, "pipeline: starting");

        // Phase 1: Vacuum
        let run_vacuum = matches!(mode, PipelineMode::Full | PipelineMode::VacuumOnly);
        if run_vacuum {
            match self.vacuum() {
                Ok(r) => {
                    tracing::info!(removed = r.removed_nodes, "pipeline: vacuum ok");
                    vacuum_report = Some(r);
                }
                Err(e) => {
                    tracing::error!(error = %e, "pipeline: vacuum failed");
                    all_ok = false;
                }
            }
        }

        // Phase 1.5: FreshHNSW (after Vacuum, before Merge)
        // Vacuum has already purged tombstoned nodes from the DashMap,
        // so FreshHNSW finds fewer orphan links.
        let run_fresh = matches!(mode, PipelineMode::Full | PipelineMode::FreshHnswOnly);
        if run_fresh {
            match self.fresh_hnsw() {
                Ok(r) => {
                    tracing::info!(repaired = r.repaired_links, "pipeline: fresh_hnsw ok");
                    fresh_hnsw_report = Some(r);
                }
                Err(e) => {
                    tracing::error!(error = %e, "pipeline: fresh_hnsw failed");
                    all_ok = false;
                }
            }
        }

        // Phase 1.75: LSM compaction (between FreshHNSW and Merge)
        // CompactOnly runs all levels; CompactL0Only runs L0 only.
        // Guarded by should_compact_level so we don't compact unnecessarily.
        let run_lsm = matches!(
            mode,
            PipelineMode::Full | PipelineMode::CompactOnly | PipelineMode::CompactL0Only
        );
        let mut lsm_reports: Vec<LsmReport> = Vec::new();
        if run_lsm {
            // Sources are L0..L2: compacting L2 promotes into the L3 archive
            // tier. L3 itself is never a source (terminal). The archive gate
            // lives in should_compact_level so disabling it stops at cold (L2).
            let max_level = match mode {
                PipelineMode::CompactL0Only => 0u8,
                _ => 2u8.min(self.vector_store.len().saturating_sub(1) as u8),
            };
            for level in 0..=max_level {
                if !self.should_compact_level(level) {
                    tracing::info!(level, "pipeline: compact skip (below threshold)");
                    continue;
                }
                match self.compact_level(level) {
                    Ok(r) => {
                        tracing::info!(level, promoted = r.nodes_promoted, "pipeline: compact ok");
                        lsm_reports.push(r);
                    }
                    Err(e) => {
                        tracing::error!(level, error = %e, "pipeline: compact failed");
                        all_ok = false;
                    }
                }
            }
        }

        // Phase 2: Merge
        let run_merge = matches!(mode, PipelineMode::Full | PipelineMode::MergeOnly);
        if run_merge {
            match self.merge_segments() {
                Ok(r) => {
                    tracing::info!(saved = r.saved_bytes, "pipeline: merge ok");
                    merge_report = Some(r);
                }
                Err(e) => {
                    tracing::error!(error = %e, "pipeline: merge failed");
                    all_ok = false;
                }
            }
        }

        // Phase 3: Reindex
        let run_index = matches!(mode, PipelineMode::Full | PipelineMode::IndexOnly);
        if run_index {
            match self.rebuild_vector_index() {
                Ok(r) => {
                    tracing::info!(indexed = r.indexed_vectors, "pipeline: reindex ok");
                    index_report = Some(r);
                }
                Err(e) => {
                    tracing::error!(error = %e, "pipeline: reindex failed");
                    all_ok = false;
                }
            }
        }

        let total_duration_ms = pipeline_start.elapsed().as_millis() as u64;
        tracing::info!(
            ?mode,
            total_duration_ms,
            success = all_ok,
            "pipeline: finished"
        );

        Ok(PipelineReport {
            vacuum: vacuum_report,
            fresh_hnsw: fresh_hnsw_report,
            merge: merge_report,
            index: index_report,
            lsm: if lsm_reports.is_empty() {
                None
            } else {
                Some(lsm_reports)
            },
            total_duration_ms,
            success: all_ok,
        })
    }

    /// Recover archived nodes from TombstoneStorage that belonged to the given summary node.
    pub fn recover_archived_nodes(&self, summary_id: u128) -> Result<Vec<UnifiedNode>> {
        self.ensure_writable()?;
        let entries = self.backend.scan(BackendPartition::TombstoneStorage)?;

        let mut recovered = Vec::new();
        let belonged_to_id = self.intern_label("belonged_to");
        for (_k, v) in &entries {
            if let Ok(mut node) = postcard::from_bytes::<UnifiedNode>(v) {
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
