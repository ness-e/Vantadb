//! Maintenance operations: refresh, consolidate, evict, rebuild, compact, flush, WAL.

use std::fs::OpenOptions;
use std::sync::Arc;
use web_time::Instant;

use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{NodeTier, UnifiedNode, VectorRepresentations};
use crate::storage::engine::{
    engine_mmap_resident_bytes, EvictionReason, EvictionReport, IndexRebuildReport,
    QuantizationMaintenanceReport, StorageEngine, FLAG_TOMBSTONE, STORAGE_ALIGNMENT,
};
use crate::storage::ops::NodeMetadata;
use crate::storage::vfile::MmapMut;
use crate::vector::governor::QuantizationAction;

impl StorageEngine {
    /// Check tombstone fragmentation and log a warning if it exceeds 20%.
    pub fn trigger_compaction(&self) -> Result<()> {
        let vstore = self.vector_store.write();
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
        self.ensure_writable()?;
        self.backend.flush()?;
        self.vector_store.read().flush()?;

        let current_wal_seq = self
            .wal
            .as_ref()
            .map(|s| s.total_record_count())
            .unwrap_or(0);

        if current_wal_seq > 0 {
            let seq_bytes = postcard::to_allocvec(&current_wal_seq)
                .map_err(|e| VantaError::SerializationError(Box::new(e)))?;
            self.backend.put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq_bytes,
            )?;
            self.backend.flush()?;
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
        let vector_store = self.vector_store.read();
        crate::metrics::record_memory_breakdown(
            hnsw.nodes.len() as u64,
            hnsw.estimate_memory_bytes() as u64,
            engine_mmap_resident_bytes(&hnsw, &vector_store),
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
        };
        let metadata_val = postcard::to_allocvec(&metadata)
            .map_err(|e| VantaError::SerializationError(Box::new(e)))?;
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
            let vstore = self.vector_store.read();
            let mmap = vstore.mmap_bytes();
            let vector_size = match &persisted.vector {
                VectorRepresentations::Full(v) => v.len() * 4,
                VectorRepresentations::MmapFull(_, len) => *len,
                VectorRepresentations::Binary(b) => b.len() * 8,
                VectorRepresentations::Turbo(t) => t.len(),
                VectorRepresentations::SQ8(d, _) => d.len() + 4,
                VectorRepresentations::None => 0,
            };
            let vector_size_aligned = (vector_size + 63) & !63;
            let offset_usize = offset as usize;
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
            let vstore = self.vector_store.read();
            crate::storage::archive::rebuild_hnsw_from_vstore(&mut rebuilt, &vstore, index_path)?
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

        let mut vstore = self.vector_store.write();
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
            return Err(VantaError::BackendError(format!(
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

    /// Recover archived nodes from TombstoneStorage that belonged to the given summary node.
    pub fn recover_archived_nodes(&self, summary_id: u128) -> Result<Vec<UnifiedNode>> {
        self.ensure_writable()?;
        let entries = self.backend.scan(BackendPartition::TombstoneStorage)?;

        let mut recovered = Vec::new();
        for (_k, v) in &entries {
            if let Ok(mut node) = postcard::from_bytes::<UnifiedNode>(v) {
                if node
                    .edges
                    .iter()
                    .any(|e| e.target == summary_id && e.label == "belonged_to")
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
