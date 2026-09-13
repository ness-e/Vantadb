//! Insert and batch-insert operations (single node and bulk paths).

use rand::SeedableRng;
use std::sync::atomic::Ordering;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::node::{Edge, FieldValue, FilterBitset, RelFields, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferedWrite, EvictionReason, PendingHnswOp, FLAG_TOMBSTONE};
use crate::storage::ops::NodeMetadata;
use crate::wal::WalRecord;

use super::{BatchInsertOptions, InsertMode};

/// Field → value → count cardinality map (alias of `cardinality_stats`).
pub(crate) type CardStats =
    std::collections::HashMap<String, std::collections::HashMap<String, usize>>;

#[derive(Clone)]
pub(crate) struct ExistingMeta {
    relational: RelFields,
    edges: Vec<Edge>,
}

// ─── D1a Slice 4: batch-phase free helpers (SLAP for `batch_insert_with_opts`) ───
//
// Mechanical split: same statements, same order, no optimization.

/// HNSW insertion policy for a batch (pure; P1/P3 flags untouched).
pub(crate) fn batch_should_insert_hnsw(opts: &BatchInsertOptions, batch_len: usize) -> bool {
    match opts.insert_mode {
        InsertMode::Incremental => true,
        InsertMode::Rebuild => false,
        InsertMode::Auto => batch_len < opts.incremental_threshold.unwrap_or(1000),
    }
}

/// Cap cardinality pairs (was tripled inline: bump + both batch branches).
pub(crate) fn cap_cardinality(stats: &mut CardStats) {
    let total: usize = stats.values().map(|m| m.len()).sum();
    if total <= crate::config::MAX_CARDINALITY_PAIRS {
        return;
    }
    if let Some(min_field) = stats
        .iter()
        .min_by_key(|(_, m)| m.len())
        .map(|(k, _)| k.clone())
    {
        stats.remove(&min_field);
    }
}

/// Decrement cardinality counts for an overwritten node's old fields.
pub(crate) fn decrement_existing_stats(stats: &mut CardStats, existing: &ExistingMeta) {
    for (field, value) in &existing.relational {
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
}

/// Staged persist buffers for one batch (travel together serialize→commit).
pub(crate) struct BatchStageBufs {
    pub(crate) kv_ops: Vec<BackendWriteOp>,
    pub(crate) hnsw_entries: Vec<(u128, FilterBitset, VectorRepresentations, u64)>,
    pub(crate) vstore_offsets: Vec<u64>,
}

impl BatchStageBufs {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            kv_ops: Vec::with_capacity(n),
            hnsw_entries: Vec::with_capacity(n),
            vstore_offsets: Vec::with_capacity(n),
        }
    }

    /// Stage one HNSW entry (id/bitset/vector snapshot + packed offset).
    pub(crate) fn push_entry(
        &mut self,
        id: u128,
        bitset: &FilterBitset,
        vector: &VectorRepresentations,
        offset: u64,
    ) {
        self.hnsw_entries
            .push((id, bitset.clone(), vector.clone(), offset));
    }
}

/// Pre-allocate vstore batch space (P4: avoid per-node grow_to syscalls).
pub(crate) fn prealloc_vstore_batch(
    vstore: &mut crate::storage::vfile::File,
    batch_len: usize,
) -> Result<()> {
    // ponytail: fixed estimate; tune if fragmentation appears
    let batch_estimate = batch_len as u64 * 1280;
    let needed = vstore.write_cursor + batch_estimate;
    if needed > vstore.size {
        vstore.grow_to(std::cmp::max(vstore.size * 2, needed + 4096))?;
    }
    Ok(())
}

/// Bump one cardinality pair, capped at 100 values per field.
pub(crate) fn bump_val_map(
    val_map: &mut std::collections::HashMap<String, usize>,
    val_key: String,
) {
    if val_map.len() < 100 || val_map.contains_key(&val_key) {
        *val_map.entry(val_key).or_default() += 1;
    }
}

/// Flag one staged offset as tombstoned after a KV batch failure (P4).
pub(crate) fn tombstone_offset(
    vstore: &mut crate::storage::vfile::File,
    packed: u64,
    batch_error: &crate::error::Error,
) {
    let (_seg_id, local_off) = crate::lsm::unpack_offset(packed);
    let Some(mut hdr) = vstore.read_header(local_off) else {
        return;
    };
    hdr.flags |= FLAG_TOMBSTONE;
    if let Err(te) = vstore.write_header(local_off, &hdr) {
        tracing::error!(
            offset = local_off,
            batch_error = %batch_error,
            header_error = %te,
            "failed to write tombstone header after KV batch write failure"
        );
    }
}

impl StorageEngine {
    /// Insert or overwrite a node: persist to WAL, vector store, KV backend, and HNSW index.
    ///
    /// # ACID note
    /// WAL is appended first, then File, then KV backend. If KV write fails after
    /// File succeeds, the entry is tombstoned (P4). WAL replay post-crash covers all
    /// other mid-operation failure gaps. In-process errors (non-crash) between WAL commit
    /// and store I/O may leave partial state ΓÇö caller should retry at the operation level.
    /// Full saga/2PC is deferred to ACID Phase 0.
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.check_pressure()?;

        // Inside transaction ΓåÆ buffer in the txn's write set; stats, indexes
        // and store writes are applied only at commit (ERR-013). Applying
        // cardinality/index updates eagerly here would leave them inflated
        // when the transaction aborts.
        {
            let active = self.active_txns.lock();
            if !active.is_empty() {
                if active.len() == 1 {
                    let txn_id = active.iter().next().copied().ok_or_else(|| {
                        crate::error::Error::generic_error(
                            "active transaction set corrupted: len()==1 but no txn id".to_string(),
                        )
                    })?;
                    drop(active);
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let mut buffers = self.txn_buffers.lock();
                    let mut buffered = node.clone();
                    buffered.last_accessed = now_ms;
                    buffers
                        .entry(txn_id)
                        .or_default()
                        .push(BufferedWrite::Insert(buffered));
                    return Ok(());
                }
                return Err(crate::error::Error::InvalidInput(
                    "Multiple active transactions; use insert_in_txn() instead".into(),
                ));
            }
        }

        // Non-transaction path: stats minus eager cardinality/index updates
        // (shared with commit_transaction), then WAL + apply inside a single
        // insert_lock
        self.apply_insert_stats(node);

        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::Error::Io(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Non-transaction path: WAL + apply inside a single insert_lock
        // critical section (ERR-010). flush() takes the same guard around
        // [drain ΓåÆ serialize ΓåÆ checkpoint_seq write], so a concurrent flush
        // can never count this WAL record before its HNSW mutation has been
        // drained into the serialized snapshot ΓÇö no invisible records and no
        // duplicates on replay. `apply_insert` queues to the pending batch
        // (try_push_pending_hnsw never blocks on the guard we hold here).
        {
            let _guard = self.acquire_insert_lock("acquire insert_lock in insert (WAL + queue)")?;
            if let Some(ref sharded) = self.wal {
                let mut wal_node = node.clone();
                wal_node.last_accessed = now_ms;
                // moved (not cloned again) ΓÇö eliminates the 2nd clone
                sharded.append(&crate::wal::WalRecord::Insert(wal_node))?;
            }
            // Pass &node directly ΓÇö no active_node intermediate needed
            self.apply_insert(node)?;
            // We hold the guard, so try_push_pending_hnsw (called inside
            // apply_insert) could never opportunistically drain ΓÇö drain the
            // batch NOW under the same latch so the HNSW entry exists for
            // immediate reads (get()/search), exactly like the pre-ERR-010
            // eager path did.
            self.drain_hnsw_batch_locked()?;
            Ok(())
        }
    }

    /// Apply cardinality stats + edge/scalar index updates for an insert.
    ///
    /// Called from the non-transactional `insert()` path and from
    /// `commit_transaction` when a buffered insert is actually committed.
    /// It is intentionally NOT called while an insert is being buffered into a
    /// transaction ΓÇö an abort would otherwise leave these counters inflated
    /// for records that never committed (ERR-013).
    pub(crate) fn apply_insert_stats(&self, node: &UnifiedNode) {
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

            // increment new-node cardinality stats (same lock)
            Self::bump_cardinality(&mut stats, node);

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
            // insert-only: increment cardinality stats
            let mut stats = self.cardinality_stats.write();
            Self::bump_cardinality(&mut stats, node);
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
    }

    /// Increment cardinality stats for a node's relational fields, capped at
    /// `MAX_CARDINALITY_PAIRS` total pairs (drops the field with fewest entries
    /// on overflow). Shared by the overwrite and insert-only branches of
    /// `apply_insert_stats` (MOD-06 dedup).
    fn bump_cardinality(stats: &mut CardStats, node: &UnifiedNode) {
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
        cap_cardinality(stats);
    }

    // ─── D1a Slice 4: batch-phase method helpers ───
    //
    // Mechanical split of `batch_insert_with_opts` phases (validate → stats →
    // persist → metrics). Same statements, same order, no optimization.

    /// Remove an overwritten node's old edge/scalar index entries.
    pub(crate) fn remove_existing_indexes(&self, existing: &ExistingMeta, id: u128) {
        if let Some(ref ei) = self.edge_index {
            for edge in &existing.edges {
                ei.remove_edge(id, edge.target);
            }
        }
        if let Some(ref si) = self.scalar_index {
            for (field, value) in &existing.relational {
                si.remove(field, value, id);
            }
        }
    }

    /// Remove an overwritten node's old stats + index entries (batch path).
    pub(crate) fn remove_existing_from_stats(
        &self,
        stats: &mut CardStats,
        existing: &ExistingMeta,
        id: u128,
    ) {
        decrement_existing_stats(stats, existing);
        self.remove_existing_indexes(existing, id);
    }

    /// Add a batch node's new stats/edges/scalars (batch bookkeeping).
    pub(crate) fn add_node_to_stats(&self, stats: &mut CardStats, node: &UnifiedNode) {
        for (field, value) in &node.relational {
            let val_keys = value.to_cardinality_keys();
            let val_map = stats.entry(field.clone()).or_default();
            for val_key in val_keys {
                bump_val_map(val_map, val_key);
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

    /// Phase 3: WAL batch append (P3: skipped when `skip_wal`).
    pub(crate) fn append_batch_wal(
        &self,
        nodes: &[UnifiedNode],
        opts: &BatchInsertOptions,
    ) -> Result<()> {
        if opts.skip_wal {
            return Ok(());
        }
        if let Some(ref sharded) = self.wal {
            let records: Vec<WalRecord> =
                nodes.iter().map(|n| WalRecord::Insert(n.clone())).collect();
            sharded.batch_append(records)?;
        }
        Ok(())
    }

    /// Cache the batch's Hot nodes; returns whether eviction is needed.
    pub(crate) fn cache_batch_hot_nodes(&self, nodes: &[UnifiedNode]) -> bool {
        let mut cache = self.volatile_cache.write();
        for node in nodes {
            if node.tier == crate::node::NodeTier::Hot {
                cache.insert(node.id, node.clone());
            }
        }
        let caps = crate::hardware::HardwareCapabilities::global();
        let max_nodes = (caps.total_memory / 4 / 1536) as usize;
        cache.len() > max_nodes
    }

    /// FND-02: eviction after the cache guard drops (locked variant, since
    /// insert_lock is held on the batch path).
    pub(crate) fn run_batch_eviction_if_needed(&self, needs_eviction: bool) {
        if !needs_eviction {
            return;
        }
        self.emergency_maintenance_trigger
            .store(true, Ordering::Release);
        if let Err(e) = self.evict_cold_nodes_with_reason_locked(
            self.config.eviction_ratio,
            EvictionReason::Watermark,
        ) {
            tracing::warn!("eviction failed: {e}");
        }
    }

    /// PERF-30: auto-flush past threshold (only when insert_lock is free).
    pub(crate) fn maybe_auto_flush(&self) {
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold && self.insert_lock.try_lock().is_some() {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }
    }

    // ─── D1a Slice 4-resto: prelude + persist-phase helpers ───

    /// Validate + timestamp a batch. `None` = empty (caller returns `Ok(())`).
    pub(crate) fn batch_prelude(&self, nodes: &[UnifiedNode]) -> Result<Option<u64>> {
        if nodes.is_empty() {
            return Ok(None);
        }
        self.check_pressure()?;
        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::Error::Io(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });
        self.touch_activity();
        // Reuse the shared clock read (same call the inline code made).
        Ok(Some(super::get::now_ms_epoch_millis()))
    }

    /// Build the KV put op for one node (metadata payload, LE-id key).
    pub(crate) fn build_kv_put_op(&self, node: &UnifiedNode) -> Result<BackendWriteOp> {
        let created_by = self.next_txn_id.load(std::sync::atomic::Ordering::Relaxed);
        let metadata = NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: created_by,
            deleted_by_txn: None,
        };
        let metadata_val =
            postcard::to_allocvec(&metadata).map_err(crate::error::Error::serialization)?;
        Ok(BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: node.id.to_le_bytes().to_vec(),
            value: metadata_val,
        })
    }

    /// Stage one node: vstore write + offset pack + KV op (HNSW entry inline).
    pub(crate) fn stage_one_node(
        &self,
        vstore: &mut crate::storage::vfile::File,
        node: &UnifiedNode,
        now_ms: u64,
    ) -> Result<(u64, BackendWriteOp)> {
        let mut active_node = node.clone();
        active_node.last_accessed = now_ms;
        let local_off = crate::storage::ops::write_node_to_vstore(vstore, &active_node)?;
        let storage_offset = crate::lsm::pack_offset(0, local_off);
        let op = self.build_kv_put_op(&active_node)?;
        Ok((storage_offset, op))
    }

    /// Phase 2: vstore writes + KV/HNSW staging for the whole batch.
    pub(crate) fn serialize_batch_nodes(
        &self,
        nodes: &[UnifiedNode],
        now_ms: u64,
        should_insert_hnsw: bool,
        bufs: &mut BatchStageBufs,
    ) -> Result<()> {
        let mut vstore = self.vstore0()?;
        prealloc_vstore_batch(&mut vstore, nodes.len())?;
        for node in nodes {
            let (storage_offset, op) = self.stage_one_node(&mut vstore, node, now_ms)?;
            bufs.vstore_offsets.push(storage_offset);
            if should_insert_hnsw {
                bufs.push_entry(node.id, &node.bitset, &node.vector, storage_offset);
            }
            bufs.kv_ops.push(op);
        }
        Ok(())
    }

    /// Bulk HNSW insert, highest level first (deterministic seed 42).
    pub(crate) fn insert_hnsw_leveled(
        &self,
        entries: &[(u128, FilterBitset, VectorRepresentations, u64)],
    ) -> Result<()> {
        let hnsw = self.hnsw.load();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut leveled: Vec<(usize, u128, FilterBitset, VectorRepresentations, u64)> =
            Vec::with_capacity(entries.len());
        for (id, bitset, vector, offset) in entries {
            // ponytail: deterministic seed — reproducible HNSW topology
            let level = crate::index::random_layer_from_config(&hnsw.config, &mut rng);
            leveled.push((level, *id, bitset.clone(), vector.clone(), *offset));
        }
        // Higher level first — better entry point placement
        leveled.sort_by_key(|k| std::cmp::Reverse(k.0));
        for (level, id, bitset, vector, offset) in &leveled {
            hnsw.add_with_level(*id, bitset.clone(), vector.clone(), *offset, *level)?;
        }
        Ok(())
    }

    /// Phase 4: KV batch write; on failure tombstone the staged offsets (P4).
    pub(crate) fn write_batch_kv_or_tombstone(
        &self,
        kv_ops: Vec<BackendWriteOp>,
        vstore_offsets: &[u64],
    ) -> Result<()> {
        if let Err(e) = self.backend.write_batch(kv_ops) {
            let mut vstore = self.vstore0()?;
            for &packed in vstore_offsets {
                tombstone_offset(&mut vstore, packed, &e);
            }
            return Err(e);
        }
        Ok(())
    }

    /// Serial stats path (non-rayon): existing-probe + bookkeeping + cap.
    #[cfg(not(feature = "rayon"))]
    pub(crate) fn apply_batch_stats_serial(
        &self,
        nodes: &[UnifiedNode],
        opts: &BatchInsertOptions,
    ) {
        let mut stats = self.cardinality_stats.write();
        for node in nodes {
            if !opts.skip_existing_check {
                if let Some(existing_node) = self.existing_for_batch(node.id) {
                    self.remove_existing_from_stats(&mut stats, &existing_node, node.id);
                }
            }
            self.add_node_to_stats(&mut stats, node);
        }
        cap_cardinality(&mut stats);
    }

    /// Amortized existence probe, one lock pass per 256-chunk (ERR-037).
    #[cfg(feature = "rayon")]
    pub(crate) fn probe_existing_for_batch(
        &self,
        nodes: &[UnifiedNode],
    ) -> Vec<Option<ExistingMeta>> {
        use rayon::prelude::*;
        nodes
            .par_chunks(256)
            .flat_map_iter(|chunk| {
                let ids: Vec<u128> = chunk.iter().map(|n| n.id).collect();
                self.existing_for_batch_many(&ids).into_iter()
            })
            .collect()
    }

    /// Rayon stats path: amortized probe + bookkeeping + cap.
    #[cfg(feature = "rayon")]
    pub(crate) fn apply_batch_stats_rayon(&self, nodes: &[UnifiedNode], opts: &BatchInsertOptions) {
        let existing: Vec<Option<ExistingMeta>> = if opts.skip_existing_check {
            vec![None; nodes.len()]
        } else {
            self.probe_existing_for_batch(nodes)
        };
        let mut stats = self.cardinality_stats.write();
        for (i, node) in nodes.iter().enumerate() {
            if let Some(ref existing_node) = existing[i] {
                self.remove_existing_from_stats(&mut stats, existing_node, node.id);
            }
            self.add_node_to_stats(&mut stats, node);
        }
        cap_cardinality(&mut stats);
    }

    /// Commit under one insert_lock guard (ERR-010): WAL → KV → HNSW →
    /// cache/evict → auto-flush. Same order as the inline code.
    pub(crate) fn commit_batch_locked(
        &self,
        nodes: &[UnifiedNode],
        opts: &BatchInsertOptions,
        mut bufs: BatchStageBufs,
        should_insert_hnsw: bool,
    ) -> Result<()> {
        let _guard = self.acquire_insert_lock("acquire insert_lock in batch_insert")?;
        self.append_batch_wal(nodes, opts)?;
        let kv_ops = std::mem::take(&mut bufs.kv_ops);
        self.write_batch_kv_or_tombstone(kv_ops, &bufs.vstore_offsets)?;
        if should_insert_hnsw {
            self.insert_hnsw_leveled(&bufs.hnsw_entries)?;
        }
        let needs_eviction = self.cache_batch_hot_nodes(nodes);
        self.run_batch_eviction_if_needed(needs_eviction);
        self.maybe_auto_flush();
        Ok(())
    }

    /// Apply an insert to the stores (vstore, KV backend, HNSW, cache).
    ///
    /// Does NOT write to WAL ΓÇö the caller is responsible for WAL logging.
    /// Does NOT check active_txns ΓÇö only called outside the buffering path.
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub(crate) fn apply_insert(&self, node: &UnifiedNode) -> Result<()> {
        // Serialize the KV payload first (pure input-derived work) so the
        // vector_store write lock is held only for the actual mmap append ΓÇö
        // ERR-035: the writes were serializing backend.put (WAL/I-O) under the
        // write lock, blocking every read-side search for its full duration.
        let key = node.id.to_le_bytes();
        // non-txn insert; use next_txn_id as pseudo-txn
        let created_by = self.next_txn_id.load(std::sync::atomic::Ordering::Relaxed);
        let metadata_val = postcard::to_allocvec(&NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: created_by,
            deleted_by_txn: None,
        })
        .map_err(crate::error::Error::serialization)?;

        let (local_off, storage_offset) = {
            let mut vstore = self.vstore0()?;
            let local_off = crate::storage::ops::write_node_to_vstore(&mut vstore, node)?;
            (local_off, crate::lsm::pack_offset(0, local_off))
        }; // vstore guard dropped here ΓÇö readers can proceed

        // ERR-014: register the HNSW mutation BEFORE publishing the KV metadata.
        // try_push_pending_hnsw queues the op ΓÇö insert() holds insert_lock, so
        // the opportunistic drain is skipped ΓÇö and the drain below applies it to
        // the index synchronously. A concurrent get() therefore can never observe
        // metadata (backend.put) whose HNSW entry does not yet exist, which was
        // the insertΓåÆget staleness window: the KV record became visible (and the
        // WAL record was appended earlier in insert()) before the index drain.
        self.try_push_pending_hnsw(PendingHnswOp {
            id: node.id,
            bitset: node.bitset.clone(),
            vector: node.vector.clone(),
            storage_offset,
            is_delete: false,
        })?;
        self.drain_hnsw_batch_locked()?;

        // P4: if KV backend write fails after the vstore/index mutations, remove
        // the HNSW entry added above (get() reads via index+metadata, but search
        // would otherwise surface a ghost) and tombstone the vstore entry to
        // prevent orphan vectors. Only the error fix-up re-acquires the lock.
        if let Err(e) = self
            .backend
            .put(BackendPartition::Default, &key, &metadata_val)
        {
            self.hnsw.load().nodes.remove(&node.id);
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

        if node.tier == crate::node::NodeTier::Hot {
            // FND-02: eviction must run AFTER dropping the cache write guard —
            // evict_cold_nodes_with_reason_locked reads/mutates volatile_cache
            // itself and parking_lot's RwLock is not reentrant (a write guard
            // held here would deadlock the eviction's own cache lock).
            let needs_eviction = {
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

                cache.len() > max_nodes
            };

            if needs_eviction {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
                // FND-02: insert_lock is held here (ERR-010), so use the
                // locked variant — eviction → consolidate must not re-acquire
                // the non-reentrant insert_lock.
                if let Err(e) = self.evict_cold_nodes_with_reason_locked(
                    self.config.eviction_ratio,
                    EvictionReason::Watermark,
                ) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold.
        // insert() may hold insert_lock here (ERR-010, non-reentrant), so
        // only auto-flush when the lock is free ΓÇö otherwise the checkpoint
        // would block on the very guard we hold. Skipped flushes happen at the
        // next uncontended call or user-initiated flush().
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold && self.insert_lock.try_lock().is_some() {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }

        Ok(())
    }

    /// Cheap existence probe for `batch_insert` index/cardinality bookkeeping.
    ///
    /// ERR-037: the previous per-node `self.get()` did a full read-path per node
    /// (active-txn lock, cache write lock, KV metadata read, HNSW lookup, vstore
    /// mmap vector read + clone) ΓÇö the vector payload was cloned and discarded.
    /// This probe returns only `relational` + `edges` (all bookkeeping needs),
    /// from a shared read-only cache peek or the KV metadata blob, skipping the
    /// HNSW/vstore vector I/O entirely.
    ///
    /// Ordering parity with `get()`: active txn buffer first (read-your-writes),
    /// then cache, then backend. A disk-tombstoned-but-KV-present node surfaces
    /// as Some; the resulting index removes are no-ops and converge with the
    /// re-insert below (same final state as the prior get()-based path).
    ///
    /// Used by the non-rayon fallback path; the rayon path uses the amortized
    /// [`Self::existing_for_batch_many`].
    #[cfg(not(feature = "rayon"))]
    fn existing_for_batch(&self, id: u128) -> Option<ExistingMeta> {
        // Read-your-writes: check active txn buffer first (parity with get())
        {
            let active = self.active_txns.lock();
            if active.len() == 1 {
                // Impossible branch guard: a HashSet with len()==1 always yields
                // one element, so the skip below is dead code kept for defense in
                // depth (panic ΓåÆ no host kill). If it ever fired, falling through
                // to cache/backend is the safe degradation for this best-effort
                // metadata probe (AUD-031).
                if let Some(&txn_id) = active.iter().next() {
                    drop(active);
                    let buffers = self.txn_buffers.lock();
                    if let Some(buffer) = buffers.get(&txn_id) {
                        for op in buffer.iter().rev() {
                            match op {
                                BufferedWrite::Insert(node) if node.id == id => {
                                    return Some(ExistingMeta {
                                        relational: node.relational.clone(),
                                        edges: node.edges.clone(),
                                    });
                                }
                                BufferedWrite::Delete(del_id) if *del_id == id => {
                                    return None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Shared read() only ΓÇö no write lock, no hits bookkeeping (probe, not a read).
        if let Some(node) = self.volatile_cache.read().get(&id) {
            if !node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                return Some(ExistingMeta {
                    relational: node.relational.clone(),
                    edges: node.edges.clone(),
                });
            }
            return None;
        }

        self.backend_existing_meta(id)
    }

    /// Batch variant of [`Self::existing_for_batch`] that amortizes lock
    /// acquisition across a chunk of ids.
    ///
    /// ERR-037 follow-up: the per-node probe acquired `active_txns` + `cache`
    /// locks per node. On cache-hit-heavy batches (overwrite re-inserts) 12
    /// rayon threads ├ù N tiny critical sections became an SRWLOCK acquisition
    /// storm that cost more than the avoided vector clone (overwrite_10000:
    /// +146% ΓåÆ +86%). One read-lock pass per chunk keeps the same check order
    /// (txn buffer ΓåÆ cache ΓåÆ backend), so semantics are unchanged.
    #[cfg(feature = "rayon")]
    fn existing_for_batch_many(&self, ids: &[u128]) -> Vec<Option<ExistingMeta>> {
        let mut result: Vec<Option<ExistingMeta>> = vec![None; ids.len()];
        let mut resolved = vec![false; ids.len()];
        let mut missing: Vec<usize> = Vec::with_capacity(ids.len());

        // Read-your-writes: scan the active txn buffer once (parity with get()).
        {
            let active = self.active_txns.lock();
            if active.len() == 1 {
                // Impossible branch guard: a HashSet with len()==1 always yields
                // one element, so the skip below is dead code kept for defense in
                // depth (panic ΓåÆ no host kill). If it ever fired, the probe falls
                // through to cache/backend ΓÇö safe degradation for this best-effort
                // read-your-writes scan (AUD-031).
                if let Some(&txn_id) = active.iter().next() {
                    drop(active);
                    let buffers = self.txn_buffers.lock();
                    if let Some(buffer) = buffers.get(&txn_id) {
                        for (i, id) in ids.iter().enumerate() {
                            for op in buffer.iter().rev() {
                                match op {
                                    BufferedWrite::Insert(node) if node.id == *id => {
                                        result[i] = Some(ExistingMeta {
                                            relational: node.relational.clone(),
                                            edges: node.edges.clone(),
                                        });
                                        resolved[i] = true;
                                        break;
                                    }
                                    BufferedWrite::Delete(del_id) if *del_id == *id => {
                                        // Definitive: not present from the reader's view.
                                        resolved[i] = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        for (i, r) in resolved.iter().enumerate() {
            if !r {
                missing.push(i);
            }
        }

        // One shared read() over the cache for all ids in the chunk.
        if !missing.is_empty() {
            let cache = self.volatile_cache.read();
            missing.retain(|&i| {
                if let Some(node) = cache.get(&ids[i]) {
                    if !node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                        result[i] = Some(ExistingMeta {
                            relational: node.relational.clone(),
                            edges: node.edges.clone(),
                        });
                    }
                    false // resolved (present or tombstone-None)
                } else {
                    true // still missing ΓåÆ backend
                }
            });
        }

        // Backend metadata reads for the remaining misses. Serial per chunk:
        // the chunk is Γëñ256 ids, so a nested par_iter would cost more in
        // rayon spawn/steal overhead than it saves (measured: nested 10000-id
        // backend misses were ~2.5x slower than the same work flat-parallel).
        for &i in &missing {
            result[i] = self.backend_existing_meta(ids[i]);
        }

        result
    }

    /// Backend fallback for the existence probes: read only the KV metadata
    /// blob (relational + edges), never the vstore vector payload.
    fn backend_existing_meta(&self, id: u128) -> Option<ExistingMeta> {
        let key = id.to_le_bytes();
        let metadata_res = self.backend.get(BackendPartition::Default, &key).ok()??;
        let metadata: NodeMetadata =
            crate::storage::ops::deserialize_node_payload(&metadata_res, "node metadata").ok()?;
        Some(ExistingMeta {
            relational: metadata.relational,
            edges: metadata.edges,
        })
    }

    /// Insert multiple nodes in a single batch operation.
    ///
    /// Reduces I/O and lock contention by batching WAL records, KV backend writes,
    /// and acquiring the HNSW insert lock once for all nodes.
    ///
    /// HNSW behaviour follows `BatchInsertOptions::default()`, which uses
    /// `InsertMode::Auto` ΓÇö small batches insert incrementally, large batches
    /// require a separate `rebuild_vector_index()` call.
    pub fn batch_insert(&self, nodes: &[UnifiedNode]) -> Result<()> {
        self.batch_insert_with_opts(nodes, BatchInsertOptions::default())
    }

    /// Insert multiple nodes with caller-supplied options.
    ///
    /// **P1** ΓÇö `skip_existing_check` avoids per-node existence check.
    /// **P3** ΓÇö `skip_wal` skips WAL record generation + `batch_append`.
    ///
    /// HNSW behavior is controlled by `InsertMode`:
    /// - `InsertMode::Incremental` ΓÇö inserts each node into the HNSW index
    ///   as it is written. Good for small batches.
    /// - `InsertMode::Rebuild` ΓÇö skips HNSW insertion; caller must call
    ///   `rebuild_vector_index()` afterward. Good for large batches.
    /// - `InsertMode::Auto` (default) ΓÇö chooses between the two above
    ///   based on batch size vs `incremental_threshold` (default: 1000).
    #[tracing::instrument(skip(self, nodes), level = "debug", err)]
    pub fn batch_insert_with_opts(
        &self,
        nodes: &[UnifiedNode],
        opts: BatchInsertOptions,
    ) -> Result<()> {
        let Some(now_ms) = self.batch_prelude(nodes)? else {
            return Ok(());
        };
        let mut bufs = BatchStageBufs::with_capacity(nodes.len());
        #[cfg(feature = "rayon")]
        self.apply_batch_stats_rayon(nodes, &opts);
        #[cfg(not(feature = "rayon"))]
        self.apply_batch_stats_serial(nodes, &opts);
        let should_insert_hnsw = batch_should_insert_hnsw(&opts, nodes.len());
        self.serialize_batch_nodes(nodes, now_ms, should_insert_hnsw, &mut bufs)?;
        self.commit_batch_locked(nodes, &opts, bufs, should_insert_hnsw)?;
        Ok(())
    }

    /// Insert multiple SDK records in a single batch operation.
    ///
    /// Converts `NodeInput` records to internal `UnifiedNode` nodes,
    /// then delegates to `batch_insert` for batched persistence.
    /// Returns the IDs of all inserted records.
    #[tracing::instrument(skip(self, records), level = "debug", err)]
    pub fn insert_batch(&self, records: &[crate::NodeInput]) -> Result<Vec<u128>> {
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
}
