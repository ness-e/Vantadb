//! Transactional operations: snapshots, MVCC txn reads/writes, commit/abort.
//!
//! [`TxnManager`] owns the pure transaction bookkeeping extracted from
//! `StorageEngine` (C2S3, SRP): id allocation, the active set, per-txn write
//! buffers, write-write conflict checks and read-your-writes probes.
//! WAL durability, store application and lock orchestration stay on the
//! engine — the manager never touches wal/hnsw/cache/backend.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use web_time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{LockPolicy, PendingHnswOp, Snapshot, FLAG_TOMBSTONE};
use crate::storage::ops::NodeMetadata;

/// An operation buffered inside an uncommitted transaction.
/// Written to WAL + stores atomically at commit time.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)] // UnifiedNode is hot-path; boxing adds indirection per insert
pub(crate) enum BufferedWrite {
    Insert(UnifiedNode),
    Delete(u128),
}

/// Outcome of probing the sole active txn buffer for one node id.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // Insert carries the hit clone transiently on the stack; boxing adds indirection per read (same rationale as BufferedWrite)
pub(crate) enum BufferProbe {
    /// Zero or >1 active txns — caller falls through to shared paths.
    NoSoleTxn,
    /// Sole active txn holds no buffered op for this id.
    Miss,
    /// Newest buffered op is an insert (clone of the buffered node).
    Insert(UnifiedNode),
    /// Newest buffered op is a delete.
    Delete,
}

/// Pure transaction bookkeeping: id counter, active set, write buffers.
///
/// All methods are thin state operations with the same lock discipline the
/// engine used inline (`active` then `buffers`, never nested in reverse).
/// No I/O, no WAL, no store access — unit-testable without storage.
pub(crate) struct TxnManager {
    next_txn_id: AtomicU64,
    active: parking_lot::Mutex<HashSet<u64>>,
    buffers: parking_lot::Mutex<HashMap<u64, Vec<BufferedWrite>>>,
}

impl TxnManager {
    pub(crate) fn new() -> Self {
        Self {
            next_txn_id: AtomicU64::new(1),
            active: parking_lot::Mutex::new(HashSet::new()),
            buffers: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Allocate a fresh txn id and register it as active.
    pub(crate) fn begin(&self) -> u64 {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::Relaxed);
        self.active.lock().insert(txn_id);
        txn_id
    }

    /// Current id for snapshots and non-txn MVCC stamps (Relaxed — same
    /// ordering the engine used for `begin_snapshot`/insert stamps).
    pub(crate) fn snapshot_id(&self) -> u64 {
        self.next_txn_id.load(Ordering::Relaxed)
    }

    /// Current id for GC cutoffs (Acquire — preserves the `gc_mvcc_versions`
    /// ordering, which must observe committed deletes).
    pub(crate) fn stable_id(&self) -> u64 {
        self.next_txn_id.load(Ordering::Acquire)
    }

    /// Whether `txn_id` is currently active.
    pub(crate) fn is_active(&self, txn_id: u64) -> bool {
        self.active.lock().contains(&txn_id)
    }

    /// Whether any transaction is active (direct-path fast check).
    pub(crate) fn has_active(&self) -> bool {
        !self.active.lock().is_empty()
    }

    /// The active id when exactly one txn is active, else `None`.
    /// `None` covers both empty (direct path) and multi (explicit-path
    /// error) — the caller distinguishes via [`Self::has_active`].
    /// The impossible len==1-but-empty set degrades to `None` (safe
    /// fallthrough, same as the AUD-031 probes) instead of panicking.
    pub(crate) fn sole_id(&self) -> Option<u64> {
        let active = self.active.lock();
        if active.len() == 1 {
            active.iter().next().copied()
        } else {
            None
        }
    }

    /// Append `op` to `txn_id`'s buffer (caller must have checked active).
    pub(crate) fn push(&self, txn_id: u64, op: BufferedWrite) {
        self.buffers.lock().entry(txn_id).or_default().push(op);
    }

    /// Check whether another active txn holds a buffered write for `node_id`.
    // ponytail: O(N) linear scan over all buffered ops per txn (moved as-is
    // from the engine). Add a HashMap<u64, HashSet<u128>> hot-set indexed by
    // txn_id for O(1) conflict checks if contention becomes a bottleneck.
    pub(crate) fn check_conflict(&self, node_id: u128, my_txn_id: u64) -> Result<()> {
        let buffers = self.buffers.lock();
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
                    return Err(crate::error::Error::InvalidInput(format!(
                        "Write-write conflict: node {} is being modified by concurrent txn {}",
                        node_id, other_id
                    )));
                }
            }
        }
        Ok(())
    }

    /// End `txn_id` and drain its buffer. `None` = wasn't active (caller
    /// takes the Phase-1 fallback path); `Some` = drained ops (maybe empty).
    pub(crate) fn take(&self, txn_id: u64) -> Option<Vec<BufferedWrite>> {
        if !self.active.lock().remove(&txn_id) {
            return None;
        }
        Some(self.buffers.lock().remove(&txn_id).unwrap_or_default())
    }

    /// Drop `txn_id` without draining (abort path). Returns prior active state.
    pub(crate) fn discard(&self, txn_id: u64) -> bool {
        let was_active = self.active.lock().remove(&txn_id);
        self.buffers.lock().remove(&txn_id);
        was_active
    }

    /// Clone the sole active txn buffer (chunk-scan helper for batch paths).
    /// `None` = zero or >1 active txns. One lock pass per call — callers
    /// amortize across a chunk (ERR-037), never per id in a loop.
    /// The impossible len==1-but-empty set degrades to `None` (AUD-031).
    pub(crate) fn cloned_sole_buffer(&self) -> Option<Vec<BufferedWrite>> {
        let active = self.active.lock();
        if active.len() != 1 {
            return None;
        }
        let txn_id = active.iter().next().copied()?;
        drop(active);
        Some(
            self.buffers
                .lock()
                .get(&txn_id)
                .cloned()
                .unwrap_or_default(),
        )
    }

    /// Read-your-writes probe of the sole active txn buffer (newest first).
    /// `Err` only in the impossible active-set-corrupted case, preserving
    /// the `get()` behavior; multi/empty degrade to `NoSoleTxn`.
    pub(crate) fn probe(&self, id: u128) -> Result<BufferProbe> {
        let active = self.active.lock();
        if active.is_empty() || active.len() > 1 {
            return Ok(BufferProbe::NoSoleTxn);
        }
        let txn_id = active.iter().next().copied().ok_or_else(|| {
            crate::error::Error::generic_error(
                "active transaction set corrupted: len()==1 but no txn id".to_string(),
            )
        })?;
        drop(active);
        let buffers = self.buffers.lock();
        let Some(buffer) = buffers.get(&txn_id) else {
            return Ok(BufferProbe::Miss);
        };
        for op in buffer.iter().rev() {
            match op {
                BufferedWrite::Insert(n) if n.id == id => {
                    return Ok(BufferProbe::Insert(n.clone()));
                }
                BufferedWrite::Delete(d) if *d == id => return Ok(BufferProbe::Delete),
                _ => {}
            }
        }
        Ok(BufferProbe::Miss)
    }
}

impl StorageEngine {
    // ΓöÇΓöÇΓöÇ Transaction Support ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    /// Begin a write transaction.
    ///
    /// Registers this txn_id in the active set so subsequent insert/delete
    /// ops (via [`Self::insert_in_txn`] / [`Self::delete_in_txn`]) are buffered.
    ///
    /// Multiple concurrent transactions are supported. Plain `insert()` /
    /// `delete()` route to the sole active txn if exactly one exists, or
    /// error if >1 (use explicit `_in_txn` methods).
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn begin_transaction(&self) -> Result<u64> {
        Ok(self.txn.begin())
    }

    /// Create a read snapshot at the current transaction ID.
    ///
    /// The snapshot captures a point-in-time view of committed data.
    /// Reads via [`Self::get_with_snapshot`] see only data committed at or
    /// before this txn_id ΓÇö uncommitted and later-committed data is
    /// invisible.
    #[tracing::instrument(skip(self), level = "debug")]
    pub fn begin_snapshot(&self) -> Snapshot {
        Snapshot {
            txn_id: self.txn.snapshot_id(),
        }
    }

    /// Insert inside an explicit transaction (concurrent-safe).
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert_in_txn(&self, node: &UnifiedNode, txn_id: u64) -> Result<()> {
        if !self.txn.is_active(txn_id) {
            return Err(crate::error::Error::InvalidInput(format!(
                "Transaction {} is not active",
                txn_id
            )));
        }
        self.txn.check_conflict(node.id, txn_id)?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let mut buffered = node.clone();
        buffered.last_accessed = now_ms;
        self.txn.push(txn_id, BufferedWrite::Insert(buffered));
        Ok(())
    }

    /// Delete inside an explicit transaction (concurrent-safe).
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete_in_txn(&self, id: u128, reason: &str, txn_id: u64) -> Result<()> {
        if !self.txn.is_active(txn_id) {
            return Err(crate::error::Error::InvalidInput(format!(
                "Transaction {} is not active",
                txn_id
            )));
        }
        self.txn.check_conflict(id, txn_id)?;

        let _ = reason; // unused for now; reserved for audit log
        self.txn.push(txn_id, BufferedWrite::Delete(id));
        Ok(())
    }

    /// Commit a transaction: drain the buffered writes, stamp them with
    /// the txn_id for MVCC visibility, flush as an atomic WAL batch,
    /// then apply to stores.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn commit_transaction(&self, txn_id: u64) -> Result<()> {
        // 1. Verify and unregister, draining the buffer in one step.
        let Some(buffer) = self.txn.take(txn_id) else {
            // Phase 1 fallback: if no buffering, just append Commit marker
            if let Some(ref sharded) = self.wal {
                sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
            }
            return Ok(());
        };

        if buffer.is_empty() {
            if let Some(ref sharded) = self.wal {
                sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
            }
            return Ok(());
        }

        // 3. Build WAL phase-1 batch: Begin + ops + Prepare (WAL v2, RES-01)
        use crate::wal::WalRecord;
        let op_count = buffer.len() as u32;
        let mut wal_records = Vec::with_capacity(buffer.len() + 2);
        wal_records.push(WalRecord::Begin(txn_id));
        for op in &buffer {
            match op {
                BufferedWrite::Insert(node) => wal_records.push(WalRecord::Insert(node.clone())),
                BufferedWrite::Delete(id) => wal_records.push(WalRecord::Delete { id: *id }),
            }
        }
        wal_records.push(WalRecord::Prepare { txn_id, op_count });

        // 4. Write WAL phase-1 batch atomically (Begin+ops+Prepare are durable now).
        // ERR-010 (FIND-62): hold insert_lock across [WAL batch → apply → drain
        // → Commit], exactly like insert()/delete()/batch_insert(). flush()
        // counts WAL records under the same guard; without it a concurrent
        // flush could checkpoint between this batch_append and the HNSW drain
        // below, persisting a checkpoint_seq that covers records whose index
        // mutation is still queued → invisible record on recovery.
        let _guard =
            self.acquire_insert_lock("acquire insert_lock in commit_transaction (ERR-010)")?;
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(wal_records)?;
        }

        // 5. Apply buffered ops to stores with MVCC stamps.
        //    On any error we write an Abort marker so recovery discards the
        //    phase-1 ops (no matching Commit) and return the error to the caller
        //    (truthful error path, RES-01 / ACID Phase 4a).
        let apply_result: Result<()> = (|| {
            for op in &buffer {
                match op {
                    BufferedWrite::Insert(node) => {
                        // ERR-013: cardinality/index updates are deferred from the
                        // buffering stage to commit; applied here so they only
                        // count records that actually commit.
                        self.apply_insert_stats(node);
                        // Remove old from HNSW/cache so the new insert can take its place
                        {
                            let hnsw = self.hnsw.load();
                            hnsw.remove_node(node.id);
                        }
                        self.cache.volatile.write().remove(&node.id);
                        self.apply_insert_with_txn(node, txn_id)?;
                    }
                    BufferedWrite::Delete(id) => {
                        // ERR-013: cardinality/index decrement is deferred from the
                        // buffering delete path; apply it here on commit.
                        self.apply_delete_stats(*id);
                        // Stamp metadata as deleted_by this txn instead of removing
                        self.stamp_deleted_in_backend(*id, txn_id)?;
                        // Still tombstone vstore + remove from HNSW + cache.
                        // FIND-62: insert_lock is held here (ERR-010), so use the
                        // inner variant with AssumeHeld — apply_delete() would
                        // re-acquire the non-reentrant lock and time out.
                        self.apply_delete_inner(*id, LockPolicy::AssumeHeld)?;
                    }
                }
            }
            Ok(())
        })();

        if let Err(e) = apply_result {
            if let Some(ref sharded) = self.wal {
                // Best-effort: append Abort. Even if this fails, the missing
                // Commit marker still makes recovery discard the txn via the
                // slice-mask (no Apply phase 2 ran implies no Commit on disk).
                let _ = sharded.append(&crate::wal::WalRecord::Abort(txn_id));
            }
            return Err(e);
        }

        // FIND-62: drain the batch queued via try_push_pending_hnsw above under
        // the same guard (pattern from insert()), so the HNSW entries exist
        // before any concurrent flush can checkpoint past our WAL records.
        self.drain_hnsw_batch_locked()?;

        // 6. Phase 2: write the Commit marker. THIS is the durability commit point.
        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
        }

        Ok(())
    }
    /// Abort a transaction: clear the buffered writes for this txn and
    /// append an `Abort(txn_id)` marker to the WAL.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn abort_transaction(&self, txn_id: u64) -> Result<()> {
        self.txn.discard(txn_id);

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
                let val =
                    postcard::to_allocvec(&meta).map_err(crate::error::Error::serialization)?;
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
        .map_err(crate::error::Error::serialization)?;

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
            let mut guard = self.cache.volatile.write();
            guard.insert(node.id, node.clone());
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
        let Some((storage_offset, index_vec)) = hnsw.node_view(id) else {
            return Ok(None);
        };
        let (seg_id, local_off) = unpack_offset(storage_offset);

        let vstore = self
            .vector_store
            .get(seg_id as usize)
            .ok_or_else(|| {
                crate::error::Error::generic_error(format!(
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

        let kind = crate::node::NodeFlags::vector_kind(header.flags);
        let vector = if kind == 0 {
            if header.vector_len == 0 {
                VectorRepresentations::None
            } else {
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
                let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];
                let f32_vec: &[f32] = unsafe {
                    std::slice::from_raw_parts(
                        slice.as_ptr() as *const f32,
                        header.vector_len as usize,
                    )
                };
                VectorRepresentations::Full(f32_vec.to_vec())
            }
        } else {
            match kind {
                crate::node::NodeFlags::VECTOR_KIND_FULL => {
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
                    let f32_vec: &[f32] = unsafe {
                        std::slice::from_raw_parts(
                            vec_bytes.as_ptr() as *const f32,
                            header.vector_len as usize,
                        )
                    };
                    VectorRepresentations::Full(f32_vec.to_vec())
                }
                crate::node::NodeFlags::VECTOR_KIND_BINARY => {
                    let Some(vec_len_bytes) = (header.vector_len as u64).checked_mul(8) else {
                        return Ok(None);
                    };
                    let Some(vec_end) = header.vector_offset.checked_add(vec_len_bytes) else {
                        return Ok(None);
                    };
                    if vec_end > vstore.mmap_bytes().len() as u64 {
                        return Ok(None);
                    }
                    let vec_start = header.vector_offset as usize;
                    let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];
                    let (_, u64_slice, _) = unsafe { slice.align_to::<u64>() };
                    if u64_slice.len() != header.vector_len as usize {
                        return Ok(None);
                    }
                    VectorRepresentations::Binary(u64_slice.to_vec().into_boxed_slice())
                }
                crate::node::NodeFlags::VECTOR_KIND_TURBO => {
                    let Some(vec_end) = header.vector_offset.checked_add(header.vector_len as u64)
                    else {
                        return Ok(None);
                    };
                    if vec_end > vstore.mmap_bytes().len() as u64 {
                        return Ok(None);
                    }
                    let vec_start = header.vector_offset as usize;
                    let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];
                    VectorRepresentations::Turbo(slice.to_vec().into_boxed_slice())
                }
                crate::node::NodeFlags::VECTOR_KIND_SQ8 => {
                    let Some(payload_end) = (header.vector_len as u64)
                        .checked_add(4)
                        .and_then(|b| header.vector_offset.checked_add(b))
                        .filter(|&end| end <= vstore.mmap_bytes().len() as u64)
                    else {
                        return Ok(None);
                    };
                    let vec_start = header.vector_offset as usize;
                    let payload = &vstore.mmap_bytes()[vec_start..payload_end as usize];
                    let n = header.vector_len as usize;
                    let scale = f32::from_le_bytes(payload[n..n + 4].try_into().unwrap_or([0; 4]));
                    if !scale.is_finite() {
                        return Ok(None);
                    }
                    let data: Vec<i8> = payload[..n].iter().map(|&b| b as i8).collect();
                    VectorRepresentations::SQ8(data.into_boxed_slice(), scale)
                }
                crate::node::NodeFlags::VECTOR_KIND_NONE => VectorRepresentations::None,
                _ => VectorRepresentations::None,
            }
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = FilterBitset::from_u128(header.bitset);
        node.vector = vector;
        // ADR-032 legacy rescue for kind==0/none
        if kind == 0 || kind == crate::node::NodeFlags::VECTOR_KIND_NONE {
            if let crate::node::VectorRepresentations::SQ8(data, scale) = &index_vec {
                if matches!(node.vector, VectorRepresentations::None)
                    || node.vector.dimensions() == 0
                {
                    node.vector = crate::node::VectorRepresentations::SQ8(data.clone(), *scale);
                }
            }
            if let crate::node::VectorRepresentations::Binary(b) = &index_vec {
                if matches!(node.vector, VectorRepresentations::None)
                    || node.vector.dimensions() == 0
                {
                    node.vector = crate::node::VectorRepresentations::Binary(b.clone());
                }
            }
            if let crate::node::VectorRepresentations::Turbo(t) = &index_vec {
                if matches!(node.vector, VectorRepresentations::None)
                    || node.vector.dimensions() == 0
                {
                    node.vector = crate::node::VectorRepresentations::Turbo(t.clone());
                }
            }
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
        node.flags =
            crate::node::NodeFlags(header.flags & !crate::node::NodeFlags::VECTOR_KIND_MASK);

        Ok(Some(node))
    }
}

#[cfg(test)]
mod txn_manager_tests {
    use super::*;

    fn node(id: u128) -> UnifiedNode {
        UnifiedNode::new(id)
    }

    #[test]
    fn begin_allocates_monotonic_ids() {
        let m = TxnManager::new();
        let a = m.begin();
        let b = m.begin();
        let c = m.begin();
        assert!(a < b && b < c, "ids must be monotonic: {a} {b} {c}");
        assert!(m.is_active(a) && m.is_active(b) && m.is_active(c));
    }

    #[test]
    fn sole_id_tracks_routing_states() {
        let m = TxnManager::new();
        assert!(!m.has_active());
        assert_eq!(m.sole_id(), None);
        let a = m.begin();
        assert!(m.has_active());
        assert_eq!(m.sole_id(), Some(a));
        let _b = m.begin();
        assert!(m.has_active());
        assert_eq!(m.sole_id(), None, "multi-txn routes to explicit path");
    }

    #[test]
    fn conflict_detected_across_txns_only() {
        let m = TxnManager::new();
        let t1 = m.begin();
        let t2 = m.begin();
        m.push(t1, BufferedWrite::Insert(node(7)));
        // Same txn: no conflict with itself.
        assert!(m.check_conflict(7, t1).is_ok());
        // Other txn: write-write conflict.
        let err = m.check_conflict(7, t2).unwrap_err();
        assert!(err.to_string().contains("Write-write conflict"), "{err:?}");
        // Untouched id: no conflict.
        assert!(m.check_conflict(8, t2).is_ok());
    }

    #[test]
    fn take_drains_and_deactivates() {
        let m = TxnManager::new();
        let t = m.begin();
        m.push(t, BufferedWrite::Delete(1));
        let buf = m.take(t).expect("active txn drains");
        assert_eq!(buf.len(), 1);
        assert!(!m.is_active(t));
        assert!(m.take(t).is_none(), "second take = not active");
        assert!(m.take(999).is_none(), "unknown txn = not active");
    }

    #[test]
    fn discard_clears_without_draining() {
        let m = TxnManager::new();
        let t = m.begin();
        m.push(t, BufferedWrite::Delete(1));
        assert!(m.discard(t));
        assert!(!m.has_active());
        assert!(!m.discard(t), "second discard = already gone");
    }

    #[test]
    fn probe_hit_miss_and_multi() {
        let m = TxnManager::new();
        assert!(matches!(m.probe(1).unwrap(), BufferProbe::NoSoleTxn));
        let t = m.begin();
        assert!(matches!(m.probe(1).unwrap(), BufferProbe::Miss));
        m.push(t, BufferedWrite::Insert(node(1)));
        m.push(t, BufferedWrite::Delete(2));
        assert!(matches!(m.probe(1).unwrap(), BufferProbe::Insert(_)));
        assert!(matches!(m.probe(2).unwrap(), BufferProbe::Delete));
        assert!(matches!(m.probe(3).unwrap(), BufferProbe::Miss));
        // Newest op wins: overwrite insert over earlier delete.
        m.push(t, BufferedWrite::Insert(node(2)));
        assert!(matches!(m.probe(2).unwrap(), BufferProbe::Insert(_)));
        let _u = m.begin();
        assert!(matches!(m.probe(1).unwrap(), BufferProbe::NoSoleTxn));
    }

    #[test]
    fn snapshot_ids_track_allocation() {
        let m = TxnManager::new();
        let before = m.snapshot_id();
        let t = m.begin();
        assert!(m.snapshot_id() > before);
        assert!(m.stable_id() >= t);
    }
}
