//! Core CRUD operations: insert, get, get_many, delete, purge, scan.

use crate::backend::BackendPartition;
use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{PendingHnswOp, FLAG_TOMBSTONE, HNSW_BATCH_SIZE};
use crate::storage::ops::NodeMetadata;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InsertMode {
    /// Insert each node into the HNSW index as it is written (no rebuild).
    Incremental,
    /// Skip HNSW insertion during the batch.
    /// The caller MUST call `rebuild_vector_index()` afterward.
    Rebuild,
    /// Automatically choose based on batch size vs `incremental_threshold`.
    /// Batches smaller than the threshold use `Incremental`;
    /// batches at or above the threshold use `Rebuild`.
    #[default]
    Auto,
}

/// Options that control batch-insert behaviour.
///
/// Use [`BatchInsertOptions::default()`] for standard behaviour
/// (existing-node check enabled, WAL enabled, HNSW index updated
/// via `InsertMode::Auto` with a default threshold of 1000 nodes).
#[derive(Clone, Debug, Default)]
pub struct BatchInsertOptions {
    /// When `true`, skip the per-node `self.get()` existence check.
    /// Safe when the caller guarantees all IDs are fresh.
    pub skip_existing_check: bool,
    /// When `true`, skip WAL record generation and `batch_append`.
    /// Use for bulk load where the source data can be re-inserted on crash.
    pub skip_wal: bool,
    /// Controls how the HNSW index is updated during this batch.
    /// Replaces the old `skip_hnsw: bool` field.
    /// Default: `InsertMode::Auto` with `incremental_threshold = Some(1000)`.
    pub insert_mode: InsertMode,
    /// Batches smaller than this threshold use `Incremental` mode
    /// when `insert_mode` is `Auto`. Default: `Some(1000)`.
    pub incremental_threshold: Option<usize>,
}

impl BatchInsertOptions {
    /// Returns `true` if this configuration implies a full rebuild is needed
    /// for a batch of the given size (i.e. HNSW was NOT updated incrementally).
    pub fn needs_rebuild(&self, batch_size: usize) -> bool {
        match self.insert_mode {
            InsertMode::Incremental => false,
            InsertMode::Rebuild => true,
            InsertMode::Auto => {
                let threshold = self.incremental_threshold.unwrap_or(1000);
                batch_size >= threshold
            }
        }
    }
}

impl StorageEngine {
    /// Bounds-checked write guard for the segment-0 vector store.
    ///
    /// A corrupt or empty `vector_store` (zero segments) must surface as a
    /// `VantaError` instead of a raw indexing panic (ERR-003).
    pub(crate) fn vstore0(
        &self,
    ) -> Result<parking_lot::RwLockWriteGuard<'_, crate::storage::vfile::VantaFile>> {
        self.vector_store.first().map(|v| v.write()).ok_or_else(|| {
            crate::error::VantaError::generic_error(
                "corrupt storage: vector_store has no segment 0".to_string(),
            )
        })
    }

    /// Scan the backend and physically remove entries whose MVCC delete
    /// stamp is older than `safe_cutoff`. A version is reclaimable when
    /// `deleted_by_txn < safe_cutoff` because any snapshot created at or
    /// after `safe_cutoff` will have `txn_id >= safe_cutoff` and cannot
    /// see the deleted version.
    ///
    /// When `safe_cutoff` is `None`, the current `next_txn_id` is used.
    /// Returns the number of entries physically removed.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn gc_mvcc_versions(&self, safe_cutoff: Option<u64>) -> Result<u64> {
        let cutoff = safe_cutoff
            .unwrap_or_else(|| self.next_txn_id.load(std::sync::atomic::Ordering::Acquire));

        let backend = &*self.backend;
        let entries = backend.scan(BackendPartition::Default)?;
        let mut keys: Vec<Vec<u8>> = Vec::new();

        for (key, val) in &entries {
            if let Ok(meta) =
                crate::storage::ops::deserialize_node_payload::<NodeMetadata>(val, "node metadata")
            {
                if let Some(deleted_by) = meta.deleted_by_txn {
                    if deleted_by < cutoff {
                        keys.push(key.clone());
                    }
                }
            }
        }

        if keys.is_empty() {
            return Ok(0);
        }

        let len = keys.len() as u64;
        for key in &keys {
            backend.delete(BackendPartition::Default, key)?;
        }

        tracing::info!(removed = len, cutoff, "MVCC GC completed");
        Ok(len)
    }

    /// Drain the pending HNSW mutation batch and apply all operations
    /// under a single `insert_lock` acquisition (P1 ΓÇö Rayon micro-batching).
    ///
    /// Returns `Ok(true)` if any ops were flushed.
    #[tracing::instrument(skip(self), level = "trace", err)]
    pub fn flush_pending_hnsw(&self) -> Result<bool> {
        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| crate::error::VantaError::Timeout {
                operation: "acquire insert_lock in flush_pending_hnsw".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;
        self.drain_hnsw_batch_locked()
    }

    /// Apply the pending HNSW mutation batch to the index.
    ///
    /// Caller MUST already hold `insert_lock` (see [`Self::flush_pending_hnsw`]
    /// and `flush()`'s ERR-010 checkpoint critical section). Never acquires the
    /// lock itself ΓÇö doing so from a context that holds the guard would
    /// deadlock, since `insert_lock` is not reentrant.
    pub(crate) fn drain_hnsw_batch_locked(&self) -> Result<bool> {
        let ops = {
            let mut pending = self.pending_hnsw_batch.lock();
            if pending.is_empty() {
                return Ok(false);
            }
            std::mem::take(&mut *pending)
        };

        let hnsw = self.hnsw.load();
        // Consume the taken batch by value: the Vec was already moved out of
        // the mutex via `mem::take`, so iterating by value moves each op into
        // `add` instead of cloning bitset+vector per insert (AUD-024).
        for op in ops {
            if op.is_delete {
                hnsw.nodes.remove(&op.id);
            } else {
                hnsw.add(op.id, op.bitset, op.vector, op.storage_offset)?;
            }
        }
        Ok(true)
    }

    /// Push a single HNSW mutation to the pending batch and attempt a
    /// non-blocking drain. Under high concurrency, ops from multiple
    /// `insert()` / `delete()` calls accumulate in the batch and are
    /// flushed atomically under one lock acquisition.
    pub(crate) fn try_push_pending_hnsw(&self, op: PendingHnswOp) -> Result<()> {
        let batch_len = {
            let mut pending = self.pending_hnsw_batch.lock();
            pending.push(op);
            pending.len()
        };
        if batch_len >= HNSW_BATCH_SIZE {
            tracing::trace!(batch_len, "HNSW pending batch reached HNSW_BATCH_SIZE");
        }

        // Opportunistic drain (P1 micro-batching): take the batch whenever the
        // insert lock is free. When the lock is busy ΓÇö another writer, or the
        // current thread's own outer guard already held by insert()/delete()/
        // flush() (ERR-010) ΓÇö the op stays queued and is drained by the next
        // holder: flush() drains the batch at the start of its checkpoint
        // critical section, so WAL-vs-index ordering is preserved either way.
        //
        // Never BLOCK on the lock here: the old threshold path called
        // flush_pending_hnsw() unconditionally, which deadlocks/timeouts
        // (insert_lock is not reentrant) when called under a held guard.
        if let Some(_guard) = self.insert_lock.try_lock() {
            let ops = {
                let mut pending = self.pending_hnsw_batch.lock();
                // could have been drained by another thread ΓÇö double-check
                if pending.is_empty() {
                    return Ok(());
                }
                std::mem::take(&mut *pending)
            };
            let hnsw = self.hnsw.load();
            // Same ownership refactor as drain_hnsw_batch_locked (AUD-024):
            // consume the taken batch by value to avoid 2 heap clones/insert.
            for op in ops {
                if op.is_delete {
                    hnsw.nodes.remove(&op.id);
                } else {
                    hnsw.add(op.id, op.bitset, op.vector, op.storage_offset)?;
                }
            }
            // guard dropped here
        }
        Ok(())
    }
    /// Insert a node into a specific backend column family and update the HNSW index.
    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let partition = crate::storage::ops::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val = postcard::to_allocvec(node).map_err(crate::error::VantaError::serialization)?;
        self.backend.put(partition, &key, &val)?;

        let mut vstore = self.vstore0()?;
        let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
        let storage_offset = crate::lsm::pack_offset(0, local_off);
        drop(vstore); // release vstore guard before refresh_index (acquires insert_lock) — Regla 8 lock order
        self.refresh_index(node, storage_offset)?;
        Ok(())
    }

    /// Return all currently readable nodes from the primary backend partition.
    pub fn scan_nodes(&self) -> Result<Vec<UnifiedNode>> {
        let (nodes, _) = self.scan_nodes_page("", usize::MAX)?;
        Ok(nodes)
    }

    /// Paginated scan: returns a page of nodes and the next cursor.
    pub fn scan_nodes_page(
        &self,
        cursor: &str,
        limit: usize,
    ) -> Result<(Vec<UnifiedNode>, String)> {
        // An empty cursor means "first page, no filter" ΓÇö parsing it as 0
        // would exclude node id 0 from every scan (ERR-010 collateral).
        let cursor_id: Option<u128> = if cursor.is_empty() {
            None
        } else {
            Some(cursor.parse().unwrap_or(0))
        };
        let entries = self.backend.scan(BackendPartition::Default)?;

        let raw_nodes = {
            let hnsw = self.hnsw.load();

            let mut collected = Vec::with_capacity(entries.len().min(limit));
            for (key, value) in entries {
                if collected.len() >= limit {
                    break;
                }
                let Ok(key_arr) = <[u8; 16]>::try_from(key.as_slice()) else {
                    continue;
                };
                let id = u128::from_le_bytes(key_arr);
                if let Some(cursor_id) = cursor_id {
                    if id <= cursor_id {
                        continue;
                    }
                }

                let metadata: NodeMetadata =
                    match crate::storage::ops::deserialize_node_payload(&value, "node metadata") {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                let index_node = match hnsw.nodes.get(&id) {
                    Some(n) => n,
                    None => continue,
                };
                let storage_offset = index_node.storage_offset;
                let (seg_id, local_off) = unpack_offset(storage_offset);
                let vstore = self
                    .vector_store
                    .get(seg_id as usize)
                    .ok_or_else(|| {
                        crate::error::VantaError::generic_error(format!(
                            "corrupt storage: segment {seg_id} out of range for node {id}"
                        ))
                    })?
                    .read();

                let header = match vstore.read_header(local_off) {
                    Some(h) => h,
                    None => continue,
                };

                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    continue;
                }

                let Some(vec_len_bytes) = (header.vector_len as u64).checked_mul(4) else {
                    continue;
                };
                let Some(vec_end) = header.vector_offset.checked_add(vec_len_bytes) else {
                    continue;
                };
                if vec_end > vstore.mmap_bytes().len() as u64 {
                    continue;
                }
                let vec_start = header.vector_offset as usize;
                let vec_end = vec_end as usize;

                let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
                debug_assert_eq!(
                    vec_bytes.as_ptr().align_offset(4),
                    0,
                    "f32 vector must be 4-byte aligned"
                );
                // SAFETY: 1) bounds ΓÇö guarded above, `vec_bytes` is an in-mapping byte
                // slice of exactly `vector_len*4` bytes; 2) alignment ΓÇö `read_header`
                // rejects non-4-multiple `vector_offset` (INV-024 M-1 central guard),
                // so `vec_bytes.as_ptr()` is 4-byte aligned even in release; 3) the
                // immediate .to_vec() eliminates aliasing concerns.
                let f32_vec: Vec<f32> = unsafe {
                    std::slice::from_raw_parts(
                        vec_bytes.as_ptr() as *const f32,
                        header.vector_len as usize,
                    )
                }
                .to_vec();

                collected.push((id, metadata, header, f32_vec));
            }
            collected
        };

        let mut nodes = Vec::with_capacity(raw_nodes.len());
        let mut last_id = 0u128;
        for (id, metadata, header, f32_vec) in raw_nodes {
            last_id = id;
            let mut node = UnifiedNode::new(id);
            node.bitset = FilterBitset::from_u128(header.bitset);
            node.vector = VectorRepresentations::Full(f32_vec);
            node.relational = metadata.relational;
            node.edges = metadata.edges;
            node.confidence_score = header.confidence_score;
            node.importance = header.importance;
            node.tier = if header.tier == 1 {
                crate::node::NodeTier::Hot
            } else {
                crate::node::NodeTier::Cold
            };
            node.flags = crate::node::NodeFlags(header.flags);
            nodes.push(node);
        }

        let next_cursor = if nodes.len() == limit && limit > 0 {
            last_id.to_string()
        } else {
            String::new()
        };

        Ok((nodes, next_cursor))
    }
}
