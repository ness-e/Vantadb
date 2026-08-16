//! Transactional operations: snapshots, MVCC txn reads/writes, commit/abort.

use web_time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferedWrite, PendingHnswOp, Snapshot, FLAG_TOMBSTONE};
use crate::storage::ops::NodeMetadata;

impl StorageEngine {
    // ΓöÇΓöÇΓöÇ Transaction Support ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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
    /// before this txn_id ΓÇö uncommitted and later-committed data is
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
                    // ERR-013: cardinality/index updates are deferred from the
                    // buffering stage to commit ΓÇö applied here so they only
                    // count records that actually commit.
                    self.apply_insert_stats(node);
                    // Remove old from HNSW/cache so the new insert can take its place
                    {
                        let hnsw = self.hnsw.load();
                        hnsw.nodes.remove(&node.id);
                    }
                    self.volatile_cache.write().remove(&node.id);
                    self.apply_insert_with_txn(node, txn_id)?;
                }
                BufferedWrite::Delete(id) => {
                    // ERR-013: cardinality/index decrement is deferred from the
                    // buffering delete path; apply it here on commit.
                    self.apply_delete_stats(*id);
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
            let meta_result = crate::storage::ops::deserialize_node_payload::<NodeMetadata>(
                &existing,
                "node metadata",
            );
            if let Ok(mut meta) = meta_result {
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
        // ERR-035: serialize KV payload outside the write lock (see apply_insert).
        let key = node.id.to_le_bytes();
        let metadata_val = postcard::to_allocvec(&NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: txn_id,
            deleted_by_txn: None,
        })
        .map_err(crate::error::VantaError::serialization)?;

        let (local_off, storage_offset) = {
            let mut vstore = self.vstore0()?;
            let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
            (local_off, crate::lsm::pack_offset(0, local_off))
        }; // vstore guard dropped here ΓÇö readers can proceed

        if let Err(e) = self.backend.put(
            crate::backend::BackendPartition::Default,
            &key,
            &metadata_val,
        ) {
            // P4: tombstone on KV failure ΓÇö re-acquire the guard only for this fix-up
            let mut vstore = self.vstore0()?;
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
        let metadata: NodeMetadata =
            match crate::storage::ops::deserialize_node_payload(&metadata_res, "node metadata") {
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
            None => return Ok(None),
        };

        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(None);
        }

        let Some(vec_len_bytes) = (header.vector_len as u64).checked_mul(4) else {
            return Ok(None);
        };
        let Some(vec_end) = header.vector_offset.checked_add(vec_len_bytes) else {
            return Ok(None);
        };
        if vec_end > vstore.mmap_bytes().len() as u64 {
            return Ok(None);
        }
        let vec_start = header.vector_offset as usize;
        let vec_end = vec_end as usize;

        let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
        // SAFETY: 1) bounds ΓÇö the `vec_end > vstore.size` guard above ensures
        // `vec_bytes` is a byte slice inside the mapping of exactly
        // `vector_len*4` bytes; 2) alignment ΓÇö `read_header` rejects headers
        // whose `vector_offset` is not a multiple of 4 (INV-024 M-1 central
        // guard), so `vec_bytes.as_ptr()` is 4-byte aligned, required for a
        // valid `&[f32]`; 3) lifetime ΓÇö the borrow of `vec_bytes` keeps the
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
}
