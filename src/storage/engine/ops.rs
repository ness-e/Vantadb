//! Core CRUD operations: insert, get, get_many, delete, purge, scan.

use rand::SeedableRng;
use std::sync::atomic::Ordering;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FieldValue, FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{
    BufferedWrite, EvictionReason, PendingHnswOp, Snapshot, FLAG_TOMBSTONE, HNSW_BATCH_SIZE,
};
use crate::storage::ops::NodeMetadata;
use crate::wal::WalRecord;

/// Controls how a batch operation updates the HNSW vector index.
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
            if let Ok(meta) = postcard::from_bytes::<NodeMetadata>(val) {
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
    /// under a single `insert_lock` acquisition (P1 — Rayon micro-batching).
    ///
    /// Returns `Ok(true)` if any ops were flushed.
    #[tracing::instrument(skip(self), level = "trace", err)]
    pub fn flush_pending_hnsw(&self) -> Result<bool> {
        let ops = {
            let mut pending = self.pending_hnsw_batch.lock();
            if pending.is_empty() {
                return Ok(false);
            }
            std::mem::take(&mut *pending)
        };

        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| crate::error::VantaError::Timeout {
                operation: "acquire insert_lock in flush_pending_hnsw".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;
        let hnsw = self.hnsw.load();
        for op in &ops {
            if op.is_delete {
                hnsw.nodes.remove(&op.id);
            } else {
                hnsw.add(
                    op.id,
                    op.bitset.clone(),
                    op.vector.clone(),
                    op.storage_offset,
                );
            }
        }
        Ok(true)
    }

    /// Push a single HNSW mutation to the pending batch and attempt a
    /// non-blocking drain. Under high concurrency, ops from multiple
    /// `insert()` / `delete()` calls accumulate in the batch and are
    /// flushed atomically under one lock acquisition.
    fn try_push_pending_hnsw(&self, op: PendingHnswOp) -> Result<()> {
        let needs_flush = {
            let mut pending = self.pending_hnsw_batch.lock();
            pending.push(op);
            pending.len() >= HNSW_BATCH_SIZE
        };
        if needs_flush {
            self.flush_pending_hnsw()?;
        } else {
            // non-blocking drain: if the lock is free, flush eagerly;
            // otherwise the next caller that hits the threshold will do it.
            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = self.insert_lock.try_lock() {
                let ops = {
                    let mut pending = self.pending_hnsw_batch.lock();
                    // could have been drained by another thread — double-check
                    if pending.is_empty() {
                        return Ok(());
                    }
                    std::mem::take(&mut *pending)
                };
                let hnsw = self.hnsw.load();
                for op in &ops {
                    if op.is_delete {
                        hnsw.nodes.remove(&op.id);
                    } else {
                        hnsw.add(
                            op.id,
                            op.bitset.clone(),
                            op.vector.clone(),
                            op.storage_offset,
                        );
                    }
                }
                // guard dropped here
            }
        }
        Ok(())
    }

    // ─── Transaction Support ──────────────────────────────────

    /// Begin a write transaction.
    ///
    /// Registers this txn_id in the active set so subsequent insert/delete
    /// ops (via [`insert_in_txn`] / [`delete_in_txn`]) are buffered.
    ///
    /// Multiple concurrent transactions are supported. Plain `insert()` /
    /// `delete()` route to the sole active txn if exactly one exists, or
    /// error if >1 (use explicit `_in_txn` methods).
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn begin_transaction(&self) -> Result<u64> {
        let txn_id = self
            .next_txn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.active_txns.lock().insert(txn_id);
        Ok(txn_id)
    }

    /// Create a read snapshot at the current transaction ID.
    ///
    /// The snapshot captures a point-in-time view of committed data.
    /// Reads via [`get_with_snapshot`] see only data committed at or
    /// before this txn_id — uncommitted and later-committed data is
    /// invisible.
    #[tracing::instrument(skip(self), level = "debug")]
    pub fn begin_snapshot(&self) -> Snapshot {
        let txn_id = self.next_txn_id.load(std::sync::atomic::Ordering::Relaxed);
        Snapshot { txn_id }
    }

    /// Check if another active txn holds a buffered write for `node_id`.
    // ponytail: O(N) linear scan over all buffered ops per txn.
    // Add a HashMap<u64, HashSet<u128>> hot-set indexed by txn_id
    // for O(1) conflict checks if contention becomes a bottleneck.
    fn check_write_conflict(&self, node_id: u128, my_txn_id: u64) -> Result<()> {
        let buffers = self.txn_buffers.lock();
        for (&other_id, ops) in buffers.iter() {
            if other_id == my_txn_id {
                continue;
            }
            for op in ops {
                let conflicted = match op {
                    BufferedWrite::Insert(n) => n.id == node_id,
                    BufferedWrite::Delete(id) => *id == node_id,
                };
                if conflicted {
                    return Err(crate::error::VantaError::InvalidInput(format!(
                        "Write-write conflict: node {} is being modified by concurrent txn {}",
                        node_id, other_id
                    )));
                }
            }
        }
        Ok(())
    }

    /// Insert inside an explicit transaction (concurrent-safe).
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert_in_txn(&self, node: &UnifiedNode, txn_id: u64) -> Result<()> {
        if !self.active_txns.lock().contains(&txn_id) {
            return Err(crate::error::VantaError::InvalidInput(format!(
                "Transaction {} is not active",
                txn_id
            )));
        }
        self.check_write_conflict(node.id, txn_id)?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let mut buffered = node.clone();
        buffered.last_accessed = now_ms;
        let mut buffers = self.txn_buffers.lock();
        buffers
            .entry(txn_id)
            .or_default()
            .push(BufferedWrite::Insert(buffered));
        Ok(())
    }

    /// Delete inside an explicit transaction (concurrent-safe).
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete_in_txn(&self, id: u128, reason: &str, txn_id: u64) -> Result<()> {
        if !self.active_txns.lock().contains(&txn_id) {
            return Err(crate::error::VantaError::InvalidInput(format!(
                "Transaction {} is not active",
                txn_id
            )));
        }
        self.check_write_conflict(id, txn_id)?;

        let _ = reason; // unused for now; reserved for audit log
        let mut buffers = self.txn_buffers.lock();
        buffers
            .entry(txn_id)
            .or_default()
            .push(BufferedWrite::Delete(id));
        Ok(())
    }

    /// Commit a transaction: drain the buffered writes, stamp them with
    /// the txn_id for MVCC visibility, flush as an atomic WAL batch,
    /// then apply to stores.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn commit_transaction(&self, txn_id: u64) -> Result<()> {
        // 1. Verify and unregister
        {
            let mut active = self.active_txns.lock();
            if !active.remove(&txn_id) {
                // Phase 1 fallback: if no buffering, just append Commit marker
                if let Some(ref sharded) = self.wal {
                    sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
                }
                return Ok(());
            }
        }

        // 2. Drain buffer for this txn
        let buffer = {
            let mut buffers = self.txn_buffers.lock();
            buffers.remove(&txn_id).unwrap_or_default()
        };

        if buffer.is_empty() {
            if let Some(ref sharded) = self.wal {
                sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
            }
            return Ok(());
        }

        // 3. Build WAL batch: Begin + all ops + Commit
        use crate::wal::WalRecord;
        let mut wal_records = Vec::with_capacity(buffer.len() + 2);
        wal_records.push(WalRecord::Begin(txn_id));
        for op in &buffer {
            match op {
                BufferedWrite::Insert(node) => wal_records.push(WalRecord::Insert(node.clone())),
                BufferedWrite::Delete(id) => wal_records.push(WalRecord::Delete { id: *id }),
            }
        }
        wal_records.push(WalRecord::Commit(txn_id));

        // 4. Write WAL batch atomically
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }

        // 5. Apply buffered ops to stores with MVCC stamps
        for op in &buffer {
            match op {
                BufferedWrite::Insert(node) => {
                    // Remove old from HNSW/cache so the new insert can take its place
                    {
                        let hnsw = self.hnsw.load();
                        hnsw.nodes.remove(&node.id);
                    }
                    self.volatile_cache.write().remove(&node.id);
                    self.apply_insert_with_txn(node, txn_id)?;
                }
                BufferedWrite::Delete(id) => {
                    // Stamp metadata as deleted_by this txn instead of removing
                    self.stamp_deleted_in_backend(*id, txn_id)?;
                    // Still tombstone vstore + remove from HNSW + cache
                    self.apply_delete(*id)?;
                }
            }
        }

        Ok(())
    }

    /// Abort a transaction: clear the buffered writes for this txn and
    /// append an `Abort(txn_id)` marker to the WAL.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn abort_transaction(&self, txn_id: u64) -> Result<()> {
        {
            let mut active = self.active_txns.lock();
            active.remove(&txn_id);
        }
        self.txn_buffers.lock().remove(&txn_id);

        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Abort(txn_id))?;
        }
        Ok(())
    }

    /// Stamp the backend metadata for `node_id` with `deleted_by_txn`.
    fn stamp_deleted_in_backend(&self, node_id: u128, txn_id: u64) -> Result<()> {
        use crate::storage::ops::NodeMetadata;
        let key = node_id.to_le_bytes();
        if let Some(existing) = self
            .backend
            .get(crate::backend::BackendPartition::Default, &key)?
        {
            if let Ok(mut meta) = postcard::from_bytes::<NodeMetadata>(&existing) {
                meta.deleted_by_txn = Some(txn_id);
                let val = postcard::to_allocvec(&meta)
                    .map_err(crate::error::VantaError::serialization)?;
                self.backend
                    .put(crate::backend::BackendPartition::Default, &key, &val)?;
            }
        }
        Ok(())
    }

    /// Apply an insert with explicit MVCC stamp.
    fn apply_insert_with_txn(&self, node: &UnifiedNode, txn_id: u64) -> Result<()> {
        let storage_offset = {
            let mut vstore = self.vector_store[0].write();
            let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
            let offset = crate::lsm::pack_offset(0, local_off);

            let key = node.id.to_le_bytes();
            let metadata = crate::storage::ops::NodeMetadata {
                relational: node.relational.clone(),
                edges: node.edges.clone(),
                created_by_txn: txn_id,
                deleted_by_txn: None,
            };
            let metadata_val = postcard::to_allocvec(&metadata)
                .map_err(crate::error::VantaError::serialization)?;
            if let Err(e) = self.backend.put(
                crate::backend::BackendPartition::Default,
                &key,
                &metadata_val,
            ) {
                // P4: tombstone on KV failure — local_off works because segment 0
                if let Some(mut hdr) = vstore.read_header(local_off) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    if let Err(te) = vstore.write_header(local_off, &hdr) {
                        tracing::error!(
                            node_id = %node.id,
                            offset = local_off,
                            put_error = %e,
                            header_error = %te,
                            "failed to write tombstone header after KV put failure"
                        );
                    }
                }
                return Err(e);
            }
            offset
        };

        self.try_push_pending_hnsw(PendingHnswOp {
            id: node.id,
            bitset: node.bitset.clone(),
            vector: node.vector.clone(),
            storage_offset,
            is_delete: false,
        })?;

        if node.tier == crate::node::NodeTier::Hot {
            let mut cache = self.volatile_cache.write();
            cache.insert(node.id, node.clone());
        }
        Ok(())
    }

    /// Retrieve a node using snapshot isolation.
    ///
    /// Only data committed before the snapshot's `txn_id` is visible.
    /// Uncommitted and later-committed versions are filtered out.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get_with_snapshot(&self, id: u128, snapshot: &Snapshot) -> Result<Option<UnifiedNode>> {
        self.touch_activity();
        self.quantization_governor.record_access(id);

        // Read from committed store with MVCC filter
        let key = id.to_le_bytes();
        let metadata_res = match self
            .backend
            .get(crate::backend::BackendPartition::Default, &key)?
        {
            Some(res) => res,
            None => return Ok(None),
        };

        use crate::storage::ops::NodeMetadata;
        let metadata: NodeMetadata = match postcard::from_bytes(&metadata_res) {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };

        // MVCC visibility: created_by_txn <= snapshot_id
        // AND (deleted_by_txn IS NULL OR deleted_by_txn > snapshot_id)
        if metadata.created_by_txn > snapshot.txn_id {
            return Ok(None);
        }
        if let Some(deleted) = metadata.deleted_by_txn {
            if deleted <= snapshot.txn_id {
                return Ok(None);
            }
        }

        let hnsw = self.hnsw.load();
        let index_node = match hnsw.nodes.get(&id) {
            Some(n) => n,
            None => return Ok(None),
        };
        let storage_offset = index_node.storage_offset;
        let (seg_id, local_off) = unpack_offset(storage_offset);

        let vstore = self.vector_store[seg_id as usize].read();
        let header = match vstore.read_header(local_off) {
            Some(h) => h,
            None => return Ok(None),
        };

        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(None);
        }

        let vec_start = header.vector_offset as usize;
        let vec_end = vec_start + (header.vector_len as usize * 4);
        if vec_end > vstore.size as usize {
            return Ok(None);
        }

        let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
        // SAFETY: 1) bounds — the `vec_end > vstore.size` guard above ensures
        // `vec_bytes` is a byte slice inside the mapping of exactly
        // `vector_len*4` bytes; 2) alignment — `read_header` rejects headers
        // whose `vector_offset` is not a multiple of 4 (INV-024 M-1 central
        // guard), so `vec_bytes.as_ptr()` is 4-byte aligned, required for a
        // valid `&[f32]`; 3) lifetime — the borrow of `vec_bytes` keeps the
        // mapping alive; the `.to_vec()` copy below clears the borrow.
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = FilterBitset::from_u128(header.bitset);
        node.vector = VectorRepresentations::Full(f32_vec.to_vec());
        if let crate::node::VectorRepresentations::SQ8(data, scale) = &index_node.value().vec_data {
            node.vector = crate::node::VectorRepresentations::SQ8(data.clone(), *scale);
        }
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

        Ok(Some(node))
    }

    /// Insert or overwrite a node: persist to WAL, vector store, KV backend, and HNSW index.
    ///
    /// # ACID note
    /// WAL is appended first, then VantaFile, then KV backend. If KV write fails after
    /// VantaFile succeeds, the entry is tombstoned (P4). WAL replay post-crash covers all
    /// other mid-operation failure gaps. In-process errors (non-crash) between WAL commit
    /// and store I/O may leave partial state — caller should retry at the operation level.
    /// Full saga/2PC is deferred to ACID Phase 0.
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.check_memory_pressure()?;
        if let Ok(Some(existing_node)) = self.get(node.id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &existing_node.relational {
                let val_keys = value.to_cardinality_keys();
                if let Some(val_map) = stats.get_mut(field.as_str()) {
                    for val_key in val_keys {
                        if let Some(count) = val_map.get_mut(&val_key) {
                            if *count > 0 {
                                *count -= 1;
                            }
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }

            // ── increment new-node cardinality stats (same lock) ──
            for (field, value) in &node.relational {
                let val_keys = value.to_cardinality_keys();
                let val_map = stats.entry(field.clone()).or_default();
                for val_key in val_keys {
                    if val_map.len() < 100 || val_map.contains_key(&val_key) {
                        *val_map.entry(val_key).or_default() += 1;
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }

            // PERF-07: remove old edges from global index
            if let Some(ref ei) = self.edge_index {
                for edge in &existing_node.edges {
                    ei.remove_edge(node.id, edge.target);
                }
            }
            // PERF-08: remove old scalar entries
            if let Some(ref si) = self.scalar_index {
                for (field, value) in &existing_node.relational {
                    si.remove(field, value, node.id);
                }
            }
        } else {
            // ── insert-only: increment cardinality stats ──
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &node.relational {
                let val_keys = value.to_cardinality_keys();
                let val_map = stats.entry(field.clone()).or_default();
                for val_key in val_keys {
                    if val_map.len() < 100 || val_map.contains_key(&val_key) {
                        *val_map.entry(val_key).or_default() += 1;
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }
        }

        // PERF-07: add new edges to global index
        if let Some(ref ei) = self.edge_index {
            for edge in &node.edges {
                ei.insert(node.id, edge.target);
            }
        }
        // PERF-08: add new scalar entries
        if let Some(ref si) = self.scalar_index {
            for (field, value) in &node.relational {
                si.insert(field, value, node.id);
            }
        }

        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Inside transaction → buffer instead of writing to stores
        {
            let active = self.active_txns.lock();
            if !active.is_empty() {
                if active.len() == 1 {
                    let txn_id = *active.iter().next().unwrap();
                    drop(active);
                    let mut buffers = self.txn_buffers.lock();
                    let mut buffered = node.clone();
                    buffered.last_accessed = now_ms;
                    buffers
                        .entry(txn_id)
                        .or_default()
                        .push(BufferedWrite::Insert(buffered));
                    return Ok(());
                }
                return Err(crate::error::VantaError::InvalidInput(
                    "Multiple active transactions; use insert_in_txn() instead".into(),
                ));
            }
        }

        // Non-transaction path: WAL + apply immediately
        if let Some(ref sharded) = self.wal {
            let mut wal_node = node.clone();
            wal_node.last_accessed = now_ms;
            // moved (not cloned again) — eliminates the 2nd clone
            sharded.append(&crate::wal::WalRecord::Insert(wal_node))?;
        }
        // Pass &node directly — no active_node intermediate needed
        self.apply_insert(node)
    }

    /// Apply an insert to the stores (vstore, KV backend, HNSW, cache).
    ///
    /// Does NOT write to WAL — the caller is responsible for WAL logging.
    /// Does NOT check active_txns — only called outside the buffering path.
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub(crate) fn apply_insert(&self, node: &UnifiedNode) -> Result<()> {
        let storage_offset = {
            let mut vstore = self.vector_store[0].write();
            let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
            let offset = crate::lsm::pack_offset(0, local_off);

            let key = node.id.to_le_bytes();
            // non-txn insert: use next_txn_id as pseudo-txn
            let created_by = self.next_txn_id.load(std::sync::atomic::Ordering::Relaxed);
            let metadata = NodeMetadata {
                relational: node.relational.clone(),
                edges: node.edges.clone(),
                created_by_txn: created_by,
                deleted_by_txn: None,
            };
            let metadata_val = postcard::to_allocvec(&metadata)
                .map_err(crate::error::VantaError::serialization)?;
            // P4: if KV backend write fails after VantaFile write, tombstone the entry
            // to prevent orphan vectors in the vector store.
            if let Err(e) = self
                .backend
                .put(BackendPartition::Default, &key, &metadata_val)
            {
                if let Some(mut hdr) = vstore.read_header(local_off) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    if let Err(te) = vstore.write_header(local_off, &hdr) {
                        tracing::error!(
                            node_id = %node.id,
                            offset = local_off,
                            put_error = %e,
                            header_error = %te,
                            "failed to write tombstone header after KV put failure"
                        );
                    }
                }
                return Err(e);
            }

            offset
        }; // vstore guard dropped here — readers can proceed

        self.try_push_pending_hnsw(PendingHnswOp {
            id: node.id,
            bitset: node.bitset.clone(),
            vector: node.vector.clone(),
            storage_offset,
            is_delete: false,
        })?;

        if node.tier == crate::node::NodeTier::Hot {
            let mut cache = self.volatile_cache.write();
            cache.insert(node.id, node.clone());
            // ponytail: cache clone is ~3KB/insert with 768d f32 vec.
            // Switching volatile_cache to HashMap<u128, Arc<UnifiedNode>>
            // would share allocations across get() reads and avoid the
            // per-insert clone. Deferred until cache write throughput is
            // a measured bottleneck.

            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;

            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
                if let Err(e) = self.evict_cold_nodes_with_reason(
                    self.config.eviction_ratio,
                    EvictionReason::Watermark,
                ) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }

        Ok(())
    }

    /// Insert multiple nodes in a single batch operation.
    ///
    /// Reduces I/O and lock contention by batching WAL records, KV backend writes,
    /// and acquiring the HNSW insert lock once for all nodes.
    ///
    /// HNSW behaviour follows `BatchInsertOptions::default()`, which uses
    /// `InsertMode::Auto` — small batches insert incrementally, large batches
    /// require a separate `rebuild_vector_index()` call.
    pub fn batch_insert(&self, nodes: &[UnifiedNode]) -> Result<()> {
        self.batch_insert_with_opts(nodes, BatchInsertOptions::default())
    }

    /// Insert multiple nodes with caller-supplied options.
    ///
    /// **P1** — `skip_existing_check` avoids per-node existence check.
    /// **P3** — `skip_wal` skips WAL record generation + `batch_append`.
    ///
    /// HNSW behavior is controlled by `InsertMode`:
    /// - `InsertMode::Incremental` — inserts each node into the HNSW index
    ///   as it is written. Good for small batches.
    /// - `InsertMode::Rebuild` — skips HNSW insertion; caller must call
    ///   `rebuild_vector_index()` afterward. Good for large batches.
    /// - `InsertMode::Auto` (default) — chooses between the two above
    ///   based on batch size vs `incremental_threshold` (default: 1000).
    #[tracing::instrument(skip(self, nodes), level = "debug", err)]
    pub fn batch_insert_with_opts(
        &self,
        nodes: &[UnifiedNode],
        opts: BatchInsertOptions,
    ) -> Result<()> {
        if nodes.is_empty() {
            return Ok(());
        }

        self.check_memory_pressure()?;
        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut kv_ops: Vec<BackendWriteOp> = Vec::with_capacity(nodes.len());
        let mut hnsw_entries: Vec<(u128, FilterBitset, VectorRepresentations, u64)> =
            Vec::with_capacity(nodes.len());
        let mut vstore_offsets: Vec<u64> = Vec::with_capacity(nodes.len());

        #[cfg(feature = "rayon")]
        {
            use rayon::prelude::*;
            let existing: Vec<Option<UnifiedNode>> = if opts.skip_existing_check {
                vec![None; nodes.len()]
            } else {
                nodes
                    .par_iter()
                    .map(|n| self.get(n.id).unwrap_or(None))
                    .collect()
            };
            let mut stats = self.cardinality_stats.write();
            for (i, node) in nodes.iter().enumerate() {
                if let Some(ref existing_node) = existing[i] {
                    for (field, value) in &existing_node.relational {
                        let val_keys = value.to_cardinality_keys();
                        if let Some(val_map) = stats.get_mut(field.as_str()) {
                            for val_key in val_keys {
                                if let Some(count) = val_map.get_mut(&val_key) {
                                    if *count > 0 {
                                        *count -= 1;
                                    }
                                }
                            }
                            val_map.retain(|_, &mut v| v > 0);
                        }
                    }
                    if let Some(ref ei) = self.edge_index {
                        for edge in &existing_node.edges {
                            ei.remove_edge(node.id, edge.target);
                        }
                    }
                    if let Some(ref si) = self.scalar_index {
                        for (field, value) in &existing_node.relational {
                            si.remove(field, value, node.id);
                        }
                    }
                }
                for (field, value) in &node.relational {
                    let val_keys = value.to_cardinality_keys();
                    let val_map = stats.entry(field.clone()).or_default();
                    for val_key in val_keys {
                        if val_map.len() < 100 || val_map.contains_key(&val_key) {
                            *val_map.entry(val_key).or_default() += 1;
                        }
                    }
                }
                if let Some(ref ei) = self.edge_index {
                    for edge in &node.edges {
                        ei.insert(node.id, edge.target);
                    }
                }
                if let Some(ref si) = self.scalar_index {
                    for (field, value) in &node.relational {
                        si.insert(field, value, node.id);
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            let mut stats = self.cardinality_stats.write();
            for node in nodes {
                if !opts.skip_existing_check {
                    if let Ok(Some(existing_node)) = self.get(node.id) {
                        for (field, value) in &existing_node.relational {
                            let val_keys = value.to_cardinality_keys();
                            if let Some(val_map) = stats.get_mut(field.as_str()) {
                                for val_key in val_keys {
                                    if let Some(count) = val_map.get_mut(&val_key) {
                                        if *count > 0 {
                                            *count -= 1;
                                        }
                                    }
                                }
                                val_map.retain(|_, &mut v| v > 0);
                            }
                        }
                        if let Some(ref ei) = self.edge_index {
                            for edge in &existing_node.edges {
                                ei.remove_edge(node.id, edge.target);
                            }
                        }
                        if let Some(ref si) = self.scalar_index {
                            for (field, value) in &existing_node.relational {
                                si.remove(field, value, node.id);
                            }
                        }
                    }
                }

                for (field, value) in &node.relational {
                    let val_keys = value.to_cardinality_keys();
                    let val_map = stats.entry(field.clone()).or_default();
                    for val_key in val_keys {
                        if val_map.len() < 100 || val_map.contains_key(&val_key) {
                            *val_map.entry(val_key).or_default() += 1;
                        }
                    }
                }

                if let Some(ref ei) = self.edge_index {
                    for edge in &node.edges {
                        ei.insert(node.id, edge.target);
                    }
                }
                if let Some(ref si) = self.scalar_index {
                    for (field, value) in &node.relational {
                        si.insert(field, value, node.id);
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }
        }

        // Determine whether to insert nodes into the HNSW index incrementally
        let should_insert_hnsw = match opts.insert_mode {
            InsertMode::Incremental => true,
            InsertMode::Rebuild => false,
            InsertMode::Auto => {
                let threshold = opts.incremental_threshold.unwrap_or(1000);
                nodes.len() < threshold
            }
        };

        // ── Phase 2: vstore writes + KV/HNSW entry prep ──────────
        let mut vstore = self.vector_store[0].write();

        // P4: pre-allocate batch space to avoid per-node grow_to syscalls
        let approx_per_node: u64 = 1280; // ponytail: fixed estimate; tune if fragmentation appears
        let batch_estimate = nodes.len() as u64 * approx_per_node;
        let current_size = vstore.size;
        let needed = vstore.write_cursor + batch_estimate;
        if needed > current_size {
            let new_size = std::cmp::max(current_size * 2, needed + 4096);
            vstore.grow_to(new_size)?;
        }

        for node in nodes {
            let mut active_node = node.clone();
            active_node.last_accessed = now_ms;
            let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, &active_node)?;
            let storage_offset = crate::lsm::pack_offset(0, local_off);
            vstore_offsets.push(storage_offset);
            if should_insert_hnsw {
                hnsw_entries.push((
                    active_node.id,
                    active_node.bitset.clone(),
                    active_node.vector.clone(),
                    storage_offset,
                ));
            }
            let key = active_node.id.to_le_bytes();
            let created_by = self.next_txn_id.load(std::sync::atomic::Ordering::Relaxed);
            let metadata = NodeMetadata {
                relational: active_node.relational.clone(),
                edges: active_node.edges.clone(),
                created_by_txn: created_by,
                deleted_by_txn: None,
            };
            let metadata_val = postcard::to_allocvec(&metadata)
                .map_err(crate::error::VantaError::serialization)?;
            kv_ops.push(BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: key.to_vec(),
                value: metadata_val,
            });
        }
        drop(vstore);

        // ── Phase 3: WAL (P3 — skip_wal flag) ────────────────────
        if !opts.skip_wal {
            if let Some(ref sharded) = self.wal {
                let wal_records: Vec<WalRecord> =
                    nodes.iter().map(|n| WalRecord::Insert(n.clone())).collect();
                sharded.batch_append(&wal_records)?;
            }
        }

        // ── Phase 4: KV batch write + tombstone on failure ────────
        if let Err(e) = self.backend.write_batch(kv_ops) {
            let mut vstore = self.vector_store[0].write();
            for &packed in &vstore_offsets {
                let (_seg_id, local_off) = crate::lsm::unpack_offset(packed);
                if let Some(mut hdr) = vstore.read_header(local_off) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    if let Err(te) = vstore.write_header(local_off, &hdr) {
                        tracing::error!(
                            offset = local_off,
                            batch_error = %e,
                            header_error = %te,
                            "failed to write tombstone header after KV batch write failure"
                        );
                    }
                }
            }
            return Err(e);
        }

        if should_insert_hnsw {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in batch_insert".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            let hnsw = self.hnsw.load();

            // P3 — Layer-wise bulk insert: pre-compute levels with local RNG
            // (avoids shared rng mutex) and sort descending so higher-level
            // nodes are inserted first — creating better entry points for
            // lower-level nodes and reducing search-layer descent cost.
            let config = &hnsw.config;
            let mut rng = rand::rngs::StdRng::seed_from_u64(42);
            let mut level_entries: Vec<(usize, u128, FilterBitset, VectorRepresentations, u64)> =
                Vec::with_capacity(hnsw_entries.len());

            for (id, bitset, vector, offset) in &hnsw_entries {
                // ponytail: deterministic seed — reproducible HNSW topology
                let level = crate::index::random_layer_from_config(config, &mut rng);
                level_entries.push((level, *id, bitset.clone(), vector.clone(), *offset));
            }

            // Higher level first → better entry point placement
            level_entries.sort_by_key(|k| std::cmp::Reverse(k.0));

            for (level, id, bitset, vector, offset) in &level_entries {
                hnsw.add_with_level(*id, bitset.clone(), vector.clone(), *offset, *level);
            }
        }

        {
            let mut cache = self.volatile_cache.write();
            for node in nodes {
                if node.tier == crate::node::NodeTier::Hot {
                    cache.insert(node.id, node.clone());
                }
            }
            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;
            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
                if let Err(e) = self.evict_cold_nodes_with_reason(
                    self.config.eviction_ratio,
                    EvictionReason::Watermark,
                ) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }
        Ok(())
    }

    /// Insert multiple SDK records in a single batch operation.
    ///
    /// Converts `VantaNodeInput` records to internal `UnifiedNode` nodes,
    /// then delegates to `batch_insert` for batched persistence.
    /// Returns the IDs of all inserted records.
    #[tracing::instrument(skip(self, records), level = "debug", err)]
    pub fn insert_batch(&self, records: &[crate::VantaNodeInput]) -> Result<Vec<u128>> {
        let nodes: Vec<UnifiedNode> = records
            .iter()
            .map(|input| {
                let mut node = UnifiedNode::new(input.id);
                if let Some(content) = &input.content {
                    node.set_field("content", FieldValue::String(content.clone()));
                }
                for (key, value) in &input.fields {
                    node.set_field(key.clone(), FieldValue::from(value.clone()));
                }
                if let Some(vector) = &input.vector {
                    if !vector.is_empty() {
                        node.vector = VectorRepresentations::Full(vector.clone());
                        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                    }
                }
                node
            })
            .collect();

        self.batch_insert(&nodes)?;

        Ok(records.iter().map(|r| r.id).collect())
    }

    /// Retrieve a node by its numeric ID, checking the volatile cache first.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get(&self, id: u128) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        self.quantization_governor.record_access(id);

        // Read-your-writes: check active txn buffer first
        {
            let active = self.active_txns.lock();
            if active.len() == 1 {
                let txn_id = *active.iter().next().unwrap();
                drop(active);
                let buffers = self.txn_buffers.lock();
                if let Some(buffer) = buffers.get(&txn_id) {
                    for op in buffer.iter().rev() {
                        match op {
                            BufferedWrite::Insert(node) if node.id == id => {
                                return Ok(Some(node.clone()));
                            }
                            BufferedWrite::Delete(del_id) if *del_id == id => {
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        {
            let mut cache = self.volatile_cache.write();
            if let Some(node) = cache.get_mut(&id) {
                if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                    return Ok(None);
                }
                node.hits += 1;
                node.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                return Ok(Some(node.clone()));
            }
        }

        let key = id.to_le_bytes();
        let metadata_res = match self.backend.get(BackendPartition::Default, &key)? {
            Some(res) => res,
            None => return Ok(None),
        };

        let metadata: NodeMetadata =
            postcard::from_bytes(&metadata_res).map_err(crate::error::VantaError::serialization)?;

        let hnsw = self.hnsw.load();
        let index_node = match hnsw.nodes.get(&id) {
            Some(n) => n,
            None => return Ok(None),
        };
        let storage_offset = index_node.storage_offset;
        let (seg_id, local_off) = unpack_offset(storage_offset);

        let vstore = self.vector_store[seg_id as usize].read();
        let header = match vstore.read_header(local_off) {
            Some(h) => h,
            None => return Ok(None),
        };

        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(None);
        }

        let vec_start = header.vector_offset as usize;
        let vec_end = vec_start + (header.vector_len as usize * 4);
        if vec_end > vstore.size as usize {
            return Ok(None);
        }

        let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
        debug_assert_eq!(
            vec_bytes.as_ptr().align_offset(4),
            0,
            "f32 vector must be 4-byte aligned"
        );
        // SAFETY: 1) bounds — `vec_end` is guarded against exceeding the mapping
        // size, so `vec_bytes` is an in-mapping byte slice of exactly
        // `vector_len*4` bytes; 2) alignment — `read_header` rejects non-4-multiple
        // `vector_offset` (INV-024 M-1 central guard), so `vec_bytes.as_ptr()` is
        // 4-byte aligned even in release (where debug_assert is compiled out);
        // 3) lifetime — bounded by the caller's read lock on the storage engine;
        // the to_vec() copy below clears the borrow, so no aliasing concern.
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = FilterBitset::from_u128(header.bitset);
        node.vector = VectorRepresentations::Full(f32_vec.to_vec());
        // Preserve quantization format from HNSW — get() reads f32 from vstore
        // but the HNSW may track a quantized (SQ8) representation instead.
        // Without this, run_quantization_maintenance Promote can never match
        // the SQ8 arm because self.get() always returns Full.
        if let crate::node::VectorRepresentations::SQ8(data, scale) = &index_node.value().vec_data {
            node.vector = crate::node::VectorRepresentations::SQ8(data.clone(), *scale);
        }
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

        // OLD-20: After a cache miss, proactively prefetch co-accessed nodes.
        // No locks are held at this point, so get() can be called recursively.
        self.prefetch_related(id);

        Ok(Some(node))
    }

    /// Warm HNSW top-layer nodes (entry point + highest-layer neighbors) into
    /// the volatile cache so search queries don't cold-read them from disk.
    pub(crate) fn warm_hnsw_top_layer(&self) {
        let top_ids = {
            let hnsw = self.hnsw.load();
            crate::cache_warmer::CacheWarmer::hnsw_top_layer_ids(&hnsw)
        };
        if top_ids.is_empty() {
            return;
        }
        // Use get() for each — it checks cache first and fetches from stores.
        for &id in &top_ids {
            let _ = self.get(id);
        }
    }

    /// Prefetch nodes that are frequently co-accessed with the given ID.
    #[inline]
    fn prefetch_related(&self, id: u128) {
        let to_fetch = {
            let cache = self.volatile_cache.read();
            self.cache_warmer
                .suggest_warm_ids(id, |i| cache.contains_key(&i))
        };
        if to_fetch.is_empty() {
            return;
        }
        for warm_id in to_fetch {
            // Recursive call: checks cache first so no infinite loop.
            // This is safe because no locks are held at the call site.
            if let Ok(Some(node)) = self.get(warm_id) {
                let mut cache = self.volatile_cache.write();
                if let std::collections::hash_map::Entry::Vacant(e) = cache.entry(warm_id) {
                    e.insert(node);
                    self.cache_warmer.record_prefetch_hit();
                }
            }
        }
    }

    /// Retrieve multiple nodes by ID in a single batch operation.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get_many(&self, ids: &[u128]) -> Result<Vec<UnifiedNode>> {
        self.touch_activity();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<UnifiedNode> = Vec::with_capacity(ids.len());

        let ids_with_keys: Vec<(u128, Vec<u8>)> = ids
            .iter()
            .map(|id| (*id, id.to_le_bytes().to_vec()))
            .collect();

        let mut remaining_indices: Vec<usize> = Vec::new();
        {
            let mut cache = self.volatile_cache.write();
            for (i, &id) in ids.iter().enumerate() {
                self.quantization_governor.record_access(id);
                if let Some(node) = cache.get_mut(&id) {
                    if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                        continue;
                    }
                    node.hits += 1;
                    node.last_accessed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    results.push(node.clone());
                } else {
                    remaining_indices.push(i);
                }
            }
        }

        if remaining_indices.is_empty() {
            return Ok(results);
        }

        let remaining_keys: Vec<&[u8]> = remaining_indices
            .iter()
            .map(|&i| ids_with_keys[i].1.as_slice())
            .collect();

        let backend_results = self
            .backend
            .get_many(BackendPartition::Default, &remaining_keys)?;

        let mut backend_map: std::collections::HashMap<u128, Vec<u8>> =
            std::collections::HashMap::with_capacity(backend_results.len());
        for (k, v) in backend_results {
            let key_slice: [u8; 16] = k.as_slice().try_into().map_err(|_| {
                crate::error::VantaError::backend_error(format!(
                    "corrupt backend: key length {} != 16",
                    k.len()
                ))
            })?;
            backend_map.insert(u128::from_le_bytes(key_slice), v);
        }

        let hnsw = self.hnsw.load();

        for &i in &remaining_indices {
            let id = ids[i];
            let Some(metadata_bytes) = backend_map.get(&id) else {
                continue;
            };

            let metadata: NodeMetadata = match postcard::from_bytes(metadata_bytes) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let Some(index_node) = hnsw.nodes.get(&id) else {
                continue;
            };
            let storage_offset = index_node.storage_offset;
            let (seg_id, local_off) = unpack_offset(storage_offset);

            let vstore = self.vector_store[seg_id as usize].read();
            let Some(header) = vstore.read_header(local_off) else {
                continue;
            };

            if (header.flags & FLAG_TOMBSTONE) != 0 {
                continue;
            }

            let vec_start = header.vector_offset as usize;
            let vec_end = vec_start + (header.vector_len as usize * 4);
            if vec_end > vstore.size as usize {
                continue;
            }

            let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
            debug_assert_eq!(
                vec_bytes.as_ptr().align_offset(4),
                0,
                "f32 vector must be 4-byte aligned"
            );
            // SAFETY: 1) bounds — guarded above, `vec_bytes` is an in-mapping byte
            // slice of exactly `vector_len*4` bytes; 2) alignment — `read_header`
            // rejects non-4-multiple `vector_offset` (INV-024 M-1 central guard),
            // so `vec_bytes.as_ptr()` is 4-byte aligned even in release; 3) the
            // to_vec() copy clears the borrow, preventing aliasing.
            let f32_vec: &[f32] = unsafe {
                std::slice::from_raw_parts(
                    vec_bytes.as_ptr() as *const f32,
                    header.vector_len as usize,
                )
            };

            let mut node = UnifiedNode::new(id);
            node.bitset = FilterBitset::from_u128(header.bitset);
            node.vector = VectorRepresentations::Full(f32_vec.to_vec());
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

            results.push(node);
        }

        // OLD-20: Record co-access patterns for all IDs fetched together.
        if results.len() >= 2 {
            let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
            self.cache_warmer.record_co_access(&ids);
        }

        Ok(results)
    }

    /// Mark a node as deleted: write tombstone, remove from cache and backend.
    ///
    /// # ACID note
    /// WAL is appended before any store I/O. If vector-store tombstone or backend delete
    /// fails mid-operation, the WAL record creates a phantom on recovery replay.
    /// Unlike insert(), there is no compensation action because WAL is the commit point.
    /// Full saga/2PC is deferred to ACID Phase 0.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete(&self, id: u128, _reason: &str) -> Result<()> {
        self.check_memory_pressure()?;
        if let Ok(Some(node)) = self.get(id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in node.relational {
                let val_keys = value.to_cardinality_keys();
                if let Some(val_map) = stats.get_mut(&field) {
                    for val_key in val_keys {
                        if let Some(count) = val_map.get_mut(&val_key) {
                            if *count > 0 {
                                *count -= 1;
                            }
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }

            // PERF-07: cascade — remove all edges referencing this node
            if let Some(ref ei) = self.edge_index {
                ei.remove_all_for_node(id);
            }
            // PERF-08: remove node from scalar index
            if let Some(ref si) = self.scalar_index {
                si.remove_node(id);
            }
        }

        // Inside transaction → buffer instead of writing to stores
        {
            let active = self.active_txns.lock();
            if !active.is_empty() {
                if active.len() == 1 {
                    let txn_id = *active.iter().next().unwrap();
                    drop(active);
                    let mut buffers = self.txn_buffers.lock();
                    buffers
                        .entry(txn_id)
                        .or_default()
                        .push(BufferedWrite::Delete(id));
                    return Ok(());
                }
                return Err(crate::error::VantaError::InvalidInput(
                    "Multiple active transactions; use delete_in_txn() instead".into(),
                ));
            }
        }

        // Non-transaction path: WAL + apply immediately + physically remove metadata
        self.ensure_writable()?;
        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Delete { id })?;
        }
        self.apply_delete(id)?;
        self.backend
            .delete(BackendPartition::Default, &id.to_le_bytes())
    }

    /// Apply a delete to the stores (vstore tombstone, HNSW, cache).
    ///
    /// Does NOT remove the backend metadata entry — callers that need
    /// physical removal (non-transactional deletes) must call
    /// `backend.delete()` separately alongside `stamp_deleted_in_backend`.
    /// Transactional deletes leave the metadata stamp so MVCC snapshots
    /// can still read the tombstone, and GC can later reclaim it.
    ///
    /// Does NOT write to WAL — the caller is responsible for WAL logging.
    /// Does NOT check active_txns or ensure_writable.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub(crate) fn apply_delete(&self, id: u128) -> Result<()> {
        let packed = {
            let hnsw = self.hnsw.load();
            hnsw.nodes.get(&id).map(|n| n.storage_offset)
        };

        // PERF-23: vector store tombstone
        if let Some(packed) = packed {
            let (seg_id, local_off) = unpack_offset(packed);
            if let Some(vs) = self.vector_store.get(seg_id as usize) {
                let mut vstore = vs.write();
                if let Some(mut header) = vstore.read_header(local_off) {
                    header.flags |= FLAG_TOMBSTONE;
                    vstore.write_header(local_off, &header)?;
                }
            }
        }

        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in apply_delete".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            let hnsw = self.hnsw.load();
            hnsw.nodes.remove(&id);

            // PERF-23: If we just removed the entry point, promote a replacement
            if hnsw.entry_point.load(Ordering::Relaxed) == id {
                let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
                hnsw.entry_point.store(new_ep, Ordering::Relaxed);
            }
        }

        self.volatile_cache.write().remove(&id);

        Ok(())
    }

    /// Delete multiple nodes in a single batch operation.
    ///
    /// Reduces I/O and lock contention by batching WAL records, KV backend writes,
    /// and processing HNSW removal for all nodes under one guard.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete_batch(&self, ids: &[u128]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        self.check_memory_pressure()?;
        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        // Phase 1: cardinality stats update, edge / scalar index removal
        {
            let mut stats = self.cardinality_stats.write();
            for &id in ids {
                if let Ok(Some(node)) = self.get(id) {
                    for (field, value) in &node.relational {
                        let val_keys = value.to_cardinality_keys();
                        if let Some(val_map) = stats.get_mut(field.as_str()) {
                            for val_key in val_keys {
                                if let Some(count) = val_map.get_mut(&val_key) {
                                    if *count > 0 {
                                        *count -= 1;
                                    }
                                }
                            }
                            val_map.retain(|_, &mut v| v > 0);
                        }
                    }
                    if let Some(ref ei) = self.edge_index {
                        ei.remove_all_for_node(id);
                    }
                    if let Some(ref si) = self.scalar_index {
                        si.remove_node(id);
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }
        }

        // Phase 2: WAL batch append
        let wal_records: Vec<WalRecord> = ids.iter().map(|&id| WalRecord::Delete { id }).collect();
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }

        // Phase 3: HNSW node removal + vector store tombstone marking
        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in delete_batch".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            let hnsw = self.hnsw.load();
            for &id in ids {
                if let Some(packed) = hnsw.nodes.get(&id).map(|n| n.storage_offset) {
                    let (seg_id, local_off) = unpack_offset(packed);
                    if let Some(vs) = self.vector_store.get(seg_id as usize) {
                        let mut vstore = vs.write();
                        if let Some(mut header) = vstore.read_header(local_off) {
                            header.flags |= FLAG_TOMBSTONE;
                            vstore.write_header(local_off, &header)?;
                        }
                    }
                }
            }
            for &id in ids {
                hnsw.nodes.remove(&id);
                if hnsw.entry_point.load(Ordering::Relaxed) == id {
                    let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
                    hnsw.entry_point.store(new_ep, Ordering::Relaxed);
                }
            }
        }

        // Phase 4: backend batch delete
        {
            let mut kv_ops: Vec<BackendWriteOp> = Vec::with_capacity(ids.len());
            for &id in ids {
                let key = id.to_le_bytes();
                kv_ops.push(BackendWriteOp::Delete {
                    partition: BackendPartition::Default,
                    key: key.to_vec(),
                });
            }
            self.backend.write_batch(kv_ops)?;
        }

        // Phase 5: volatile cache removal
        {
            let mut cache = self.volatile_cache.write();
            for &id in ids {
                cache.remove(&id);
            }
        }

        Ok(())
    }

    /// Permanently remove all traces of a node from all backend partitions.
    pub fn purge_permanent(&self, id: u128) -> Result<()> {
        self.ensure_writable()?;
        let key = id.to_le_bytes();
        self.backend.write_batch(vec![
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::TombstoneStorage,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Tombstones,
                key: key.to_vec(),
            },
        ])
    }

    /// Check whether a node has been marked as deleted in the tombstones partition.
    pub fn is_deleted(&self, id: u128) -> Result<bool> {
        let key = id.to_le_bytes();
        match self.backend.get(BackendPartition::Tombstones, &key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Insert a node into a specific backend column family and update the HNSW index.
    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let partition = crate::storage::ops::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val = postcard::to_allocvec(node).map_err(crate::error::VantaError::serialization)?;
        self.backend.put(partition, &key, &val)?;

        let mut vstore = self.vector_store[0].write();
        let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
        let storage_offset = crate::lsm::pack_offset(0, local_off);
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
        let cursor_id: u128 = cursor.parse().unwrap_or(0);
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
                if id <= cursor_id {
                    continue;
                }

                let metadata: NodeMetadata = match postcard::from_bytes(&value) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let index_node = match hnsw.nodes.get(&id) {
                    Some(n) => n,
                    None => continue,
                };
                let storage_offset = index_node.storage_offset;
                let (seg_id, local_off) = unpack_offset(storage_offset);
                let vstore = self.vector_store[seg_id as usize].read();

                let header = match vstore.read_header(local_off) {
                    Some(h) => h,
                    None => continue,
                };

                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    continue;
                }

                let vec_start = header.vector_offset as usize;
                let vec_end = vec_start + (header.vector_len as usize * 4);
                if vec_end > vstore.size as usize {
                    continue;
                }

                let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
                debug_assert_eq!(
                    vec_bytes.as_ptr().align_offset(4),
                    0,
                    "f32 vector must be 4-byte aligned"
                );
                // SAFETY: 1) bounds — guarded above, `vec_bytes` is an in-mapping byte
                // slice of exactly `vector_len*4` bytes; 2) alignment — `read_header`
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
