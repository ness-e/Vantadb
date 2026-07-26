//! Core CRUD operations: insert, get, get_many, delete, purge, scan.

use std::sync::atomic::Ordering;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::node::{FieldValue, FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{
    BufferedWrite, EvictionReason, PendingHnswOp, Snapshot, FLAG_TOMBSTONE, HNSW_BATCH_SIZE,
};
use crate::storage::ops::NodeMetadata;
use crate::wal::WalRecord;

impl StorageEngine {
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
            let mut vstore = self.vector_store.write();
            let offset = Self::write_node_to_vstore(&mut vstore, node)?;

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
                // P4: tombstone on KV failure
                if let Some(mut hdr) = vstore.read_header(offset) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    let _ = vstore.write_header(offset, &hdr);
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

        let vstore = self.vector_store.read();
        let header = match vstore.read_header(storage_offset) {
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
            let mut vstore = self.vector_store.write();
            let offset = Self::write_node_to_vstore(&mut vstore, node)?;

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
                if let Some(mut hdr) = vstore.read_header(offset) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    let _ = vstore.write_header(offset, &hdr);
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
    pub fn batch_insert(&self, nodes: &[UnifiedNode]) -> Result<()> {
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

        let mut wal_records: Vec<WalRecord> = Vec::with_capacity(nodes.len());
        let mut kv_ops: Vec<BackendWriteOp> = Vec::with_capacity(nodes.len());
        let mut hnsw_entries: Vec<(u128, FilterBitset, VectorRepresentations, u64)> =
            Vec::with_capacity(nodes.len());
        let mut vstore_offsets: Vec<u64> = Vec::with_capacity(nodes.len());

        let mut vstore = self.vector_store.write();
        {
            let mut stats = self.cardinality_stats.write();
            for node in nodes {
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

        for node in nodes {
            let mut active_node = node.clone();
            active_node.last_accessed = now_ms;
            let storage_offset = Self::write_node_to_vstore(&mut vstore, &active_node)?;
            vstore_offsets.push(storage_offset);
            hnsw_entries.push((
                active_node.id,
                active_node.bitset.clone(),
                active_node.vector.clone(),
                storage_offset,
            ));
            wal_records.push(WalRecord::Insert(active_node.clone()));

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

        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }

        // P4: if KV batch write fails after VantaFile writes, tombstone all
        // entries to prevent orphan vectors in the vector store.
        if let Err(e) = self.backend.write_batch(kv_ops) {
            let mut vstore = self.vector_store.write();
            for &offset in &vstore_offsets {
                if let Some(mut hdr) = vstore.read_header(offset) {
                    hdr.flags |= FLAG_TOMBSTONE;
                    let _ = vstore.write_header(offset, &hdr);
                }
            }
            return Err(e);
        }

        {
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
            for (id, bitset, vector, offset) in &hnsw_entries {
                hnsw.add(*id, bitset.clone(), vector.clone(), *offset);
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

        let vstore = self.vector_store.read();
        let header = match vstore.read_header(storage_offset) {
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
        // SAFETY: vec_bytes is a slice of a memory-mapped region (page-aligned,
        // guaranteeing f32 alignment). The debug_assert_eq above verifies the
        // 4-byte alignment invariant. The lifetime is bounded by the caller's
        // read lock on the storage engine. The to_vec() copy below eliminates
        // the borrow, so no aliasing concern.
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

        Ok(Some(node))
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
        let vstore = self.vector_store.read();

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

            let Some(header) = vstore.read_header(storage_offset) else {
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
            // SAFETY: vec_bytes is page-aligned via mmap, guaranteeing f32
            // alignment. The debug_assert_eq above confirms the invariant.
            // The to_vec() copy clears the borrow, preventing aliasing.
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

        // Non-transaction path: WAL + apply immediately
        self.ensure_writable()?;
        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Delete { id })?;
        }
        self.apply_delete(id)
    }

    /// Apply a delete to the stores (vstore tombstone, HNSW, cache, backend).
    ///
    /// Does NOT write to WAL — the caller is responsible for WAL logging.
    /// Does NOT check active_txns or ensure_writable.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub(crate) fn apply_delete(&self, id: u128) -> Result<()> {
        let offset = {
            let hnsw = self.hnsw.load();
            hnsw.nodes.get(&id).map(|n| n.storage_offset)
        };

        // PERF-23: vector store tombstone
        if let Some(offset) = offset {
            let mut vstore = self.vector_store.write();
            if let Some(mut header) = vstore.read_header(offset) {
                header.flags |= FLAG_TOMBSTONE;
                vstore.write_header(offset, &header)?;
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

        let key = id.to_le_bytes();
        self.backend.delete(BackendPartition::Default, &key)?;

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
            {
                let mut vstore = self.vector_store.write();
                for &id in ids {
                    if let Some(offset) = hnsw.nodes.get(&id).map(|n| n.storage_offset) {
                        if let Some(mut header) = vstore.read_header(offset) {
                            header.flags |= FLAG_TOMBSTONE;
                            vstore.write_header(offset, &header)?;
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

        let mut vstore = self.vector_store.write();
        let storage_offset = Self::write_node_to_vstore(&mut vstore, node)?;
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
            let vstore = self.vector_store.read();

            let mut collected = Vec::with_capacity(entries.len().min(limit));
            for (key, value) in entries {
                if collected.len() >= limit {
                    break;
                }
                if key.len() != std::mem::size_of::<u128>() {
                    continue;
                }

                let id = u128::from_le_bytes(
                    key.as_slice().try_into().expect("key slice fits [u8; 16]"),
                );
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

                let header = match vstore.read_header(storage_offset) {
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
                // SAFETY: vec_bytes slice from page-aligned mmap, guaranteeing
                // f32 alignment. The debug_assert_eq above verifies it.
                // Immediate .to_vec() eliminates aliasing concerns.
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
