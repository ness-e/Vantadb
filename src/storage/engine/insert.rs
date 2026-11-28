//! Insert and batch-insert operations (single node and bulk paths).

use std::sync::atomic::Ordering;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::node::{Edge, FieldValue, FilterBitset, RelFields, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferedWrite, EvictionReason, PendingHnswOp};
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
    // `ensure_capacity` grows to min(cap, max(needed, size*2)) — a raw
    // `grow_to(size*2)` overshoots `mem_growth_cap` and fails at >50% of
    // `Config::memory_limit` on `:memory:` stores (FIND-597 residual).
    // required = the batch contract (must fit under the cap);
    // desired = headroom hint, clamped to mem_growth_cap by ensure_capacity.
    vstore.ensure_capacity(needed, needed + 4096)?;
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
    // ADR-0057 W3: in an encrypted segment the header lives inside the
    // ciphertext, so the node is re-sealed instead of being patched in place.
    if let Err(te) = vstore.set_tombstone(local_off, true) {
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
            if self.txn.has_active() {
                if let Some(txn_id) = self.txn.sole_id() {
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let mut buffered = node.clone();
                    buffered.last_accessed = now_ms;
                    self.txn.push(txn_id, BufferedWrite::Insert(buffered));
                    return Ok(());
                }
                return Err(crate::error::Error::InvalidInput(
                    "Multiple active transactions; use insert_in_txn() instead".into(),
                ));
            }
        }

        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::Error::Io(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        // Probe for an existing node BEFORE applying the insert so old stats
        // and index entries can be decremented/removed during overwrite.
        let pre_existing = self.existing_meta(node.id);

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Non-transaction path: WAL + apply inside a single insert_lock
        // critical section (ERR-010). flush() takes the same guard around
        // [drain → serialize → checkpoint_seq write], so a concurrent flush
        // can never count this WAL record before its HNSW mutation has been
        // drained into the serialized snapshot — no invisible records and no
        // duplicates on replay. `apply_insert` queues to the pending batch
        // (try_push_pending_hnsw never blocks on the guard we hold here).
        {
            let _guard = self.acquire_insert_lock("acquire insert_lock in insert (WAL + queue)")?;
            if let Some(ref sharded) = self.wal {
                let mut wal_node = node.clone();
                wal_node.last_accessed = now_ms;
                // moved (not cloned again) — eliminates the 2nd clone
                sharded.append(&crate::wal::WalRecord::Insert(wal_node))?;
            }
            // Pass &node directly — no active_node intermediate needed
            self.apply_insert(node)?;
            // We hold the guard, so try_push_pending_hnsw (called inside
            // apply_insert) could never opportunistically drain — drain the
            // batch NOW under the same latch so the HNSW entry exists for
            // immediate reads (get()/search), exactly like the pre-ERR-010
            // eager path did.
            self.drain_hnsw_batch_locked()?;
        }

        // FIND-363 / FIND-345: Apply cardinality stats and secondary index
        // updates AFTER WAL + store commit and outside `insert_lock` so
        // read-only/WAL failures never leave phantom secondary index entries.
        self.apply_insert_stats(node, pre_existing.as_ref());
        Ok(())
    }

    /// Apply cardinality stats + edge/scalar index updates for an insert.
    ///
    /// Called from the non-transactional `insert()` path and from
    /// `commit_transaction` when a buffered insert is actually committed.
    /// It is intentionally NOT called while an insert is being buffered into a
    /// transaction — an abort would otherwise leave these counters inflated
    /// for records that never committed (ERR-013).
    pub(crate) fn apply_insert_stats(
        &self,
        node: &UnifiedNode,
        pre_existing: Option<&ExistingMeta>,
    ) {
        {
            let mut stats = self.cache.cardinality_stats.write();
            if let Some(existing) = pre_existing {
                decrement_existing_stats(&mut stats, existing);
            }
            Self::bump_cardinality(&mut stats, node);
        }

        if let Some(existing) = pre_existing {
            self.remove_existing_indexes(existing, node.id);
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
    ///
    /// `now_ms` stamps `last_accessed` on every `WalRecord::Insert` — the same
    /// stamp `serialize_batch_nodes` writes into the vstore/KV copies and the
    /// same stamp `insert()` puts on its WAL record, so a replayed batch
    /// restores the recency heuristic the live path wrote instead of the
    /// caller-side construction timestamp.
    ///
    /// FIND-640: when the batch runs inside an implicit WAL txn frame this
    /// helper is called during the WAL phase (all chunks durable before any
    /// store is touched); without a frame (`skip_wal` or WAL disabled) it is
    /// called interleaved inside the apply loop, as before.
    pub(crate) fn append_batch_wal(
        &self,
        nodes: &[UnifiedNode],
        opts: &BatchInsertOptions,
        now_ms: u64,
    ) -> Result<()> {
        if opts.skip_wal {
            return Ok(());
        }
        if let Some(ref sharded) = self.wal {
            let records: Vec<WalRecord> = nodes
                .iter()
                .map(|n| {
                    let mut wal_node = n.clone();
                    wal_node.last_accessed = now_ms;
                    WalRecord::Insert(wal_node)
                })
                .collect();
            sharded.batch_append(records)?;
        }
        Ok(())
    }

    /// Cache the batch's Hot nodes and invalidate overwritten Cold entries;
    /// returns whether eviction is needed.
    pub(crate) fn cache_batch_hot_nodes(&self, nodes: &[UnifiedNode]) -> bool {
        let mut guard = self.cache.volatile.write();
        for node in nodes {
            if node.tier == crate::node::NodeTier::Hot {
                guard.insert(node.id, node.clone());
            } else {
                // FIND-190: same stale-entry invalidation as apply_insert —
                // prefetch may have cached this Cold node's previous payload.
                // FIND-222: bump under the guard so the (bump, remove) pair is
                // one critical section for a concurrent prefetch.
                self.cache.note_invalidation();
                guard.remove(&node.id);
            }
        }
        let caps = crate::hardware::HardwareCapabilities::global();
        let max_nodes = (caps.total_memory / 4 / 1536) as usize;
        guard.len() > max_nodes
    }

    /// FND-02: eviction after the cache guard drops (locked variant, since
    /// insert_lock is held on the batch path).
    ///
    /// Precondition: the caller MUST hold `insert_lock` — the `_locked`
    /// eviction path it drives (`consolidate_node_locked`) assumes the FIND-651
    /// quiesce barrier is already held and never re-acquires the non-reentrant
    /// lock. `batch_insert_with_opts` re-acquires it around this call after
    /// the FIND-640 whole-batch guard drops.
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
            if hnsw.node_count() >= threshold && self.insert_lock.try_lock().is_some() {
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
    ///
    /// `stamp_txn` becomes `created_by_txn` of the metadata. FIND-640: under
    /// the implicit batch WAL frame this is the dedicated `batch_txn` id —
    /// the recovery undo pass only purges entries whose `created_by_txn`
    /// matches the uncommitted `record_txn_id`, so stamping the shared
    /// `txn.snapshot_id()` would make batch entries indistinguishable from
    /// foreign inserts and silently defeat the undo.
    pub(crate) fn build_kv_put_op(
        &self,
        node: &UnifiedNode,
        stamp_txn: u64,
    ) -> Result<BackendWriteOp> {
        let metadata = NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: stamp_txn,
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
    ///
    /// `stamp_txn` is the `created_by_txn` written into the KV metadata (see
    /// [`Self::build_kv_put_op`]); `now_ms` stamps `last_accessed` on the
    /// vstore copy.
    pub(crate) fn stage_one_node(
        &self,
        vstore: &mut crate::storage::vfile::File,
        node: &UnifiedNode,
        now_ms: u64,
        stamp_txn: u64,
    ) -> Result<(u64, BackendWriteOp)> {
        let mut active_node = node.clone();
        active_node.last_accessed = now_ms;
        let local_off = crate::storage::ops::write_node_to_vstore(vstore, &active_node)?;
        let storage_offset = crate::lsm::pack_offset(0, local_off);
        let op = self.build_kv_put_op(&active_node, stamp_txn)?;
        Ok((storage_offset, op))
    }

    /// Phase 2: vstore writes + KV/HNSW staging for the whole batch.
    ///
    /// `stamp_txn` threads into every staged KV op's `created_by_txn` — the
    /// dedicated `batch_txn` under FIND-640's implicit txn frame.
    pub(crate) fn serialize_batch_nodes(
        &self,
        nodes: &[UnifiedNode],
        now_ms: u64,
        should_insert_hnsw: bool,
        bufs: &mut BatchStageBufs,
        stamp_txn: u64,
    ) -> Result<()> {
        let mut vstore = self.vstore0()?;
        prealloc_vstore_batch(&mut vstore, nodes.len())?;
        for node in nodes {
            let (storage_offset, op) = self.stage_one_node(&mut vstore, node, now_ms, stamp_txn)?;
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
        // Single RNG stream (seed 42) shared by all entries — same draws as the
        // pre-split loop; the sort below is only meaningful with real spread.
        let levels = hnsw.bulk_levels_seeded(42, entries.len());
        let mut leveled: Vec<(usize, u128, FilterBitset, VectorRepresentations, u64)> =
            Vec::with_capacity(entries.len());
        for ((id, bitset, vector, offset), level) in entries.iter().zip(levels) {
            leveled.push((level, *id, bitset.clone(), vector.clone(), *offset));
        }
        // Higher level first — better entry point placement
        leveled.sort_by_key(|k| std::cmp::Reverse(k.0));
        for (level, id, bitset, vector, offset) in &leveled {
            hnsw.add_node_with_level(*id, bitset.clone(), vector.clone(), *offset, *level)?;
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
        let mut stats = self.cache.cardinality_stats.write();
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
        let mut stats = self.cache.cardinality_stats.write();
        for (i, node) in nodes.iter().enumerate() {
            if let Some(ref existing_node) = existing[i] {
                self.remove_existing_from_stats(&mut stats, existing_node, node.id);
            }
            self.add_node_to_stats(&mut stats, node);
        }
        cap_cardinality(&mut stats);
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
        let created_by = self.txn.snapshot_id();
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
            self.hnsw.load().remove_node(node.id);
            let mut vstore = self.vstore0()?;
            if let Err(te) = vstore.set_tombstone(local_off, true) {
                tracing::error!(
                    node_id = %node.id,
                    offset = local_off,
                    put_error = %e,
                    header_error = %te,
                    "failed to write tombstone header after KV put failure"
                );
            }
            return Err(e);
        }

        if node.tier == crate::node::NodeTier::Hot {
            // FND-02: eviction must run AFTER dropping the cache write guard —
            // evict_cold_nodes_with_reason_locked reads/mutates volatile
            // itself and parking_lot's RwLock is not reentrant (a write guard
            // held here would deadlock the eviction's own cache lock).
            let needs_eviction = {
                let mut guard = self.cache.volatile.write();
                guard.insert(node.id, node.clone());
                // ponytail: cache clone is ~3KB/insert with 768d f32 vec.
                // Switching volatile to HashMap<u128, Arc<UnifiedNode>>
                // would share allocations across get() reads and avoid the
                // per-insert clone. Deferred until cache write throughput is
                // a measured bottleneck.

                let caps = crate::hardware::HardwareCapabilities::global();
                let cache_cap_bytes = caps.total_memory / 4;
                let approx_node_size = 1536;
                let max_nodes = (cache_cap_bytes / approx_node_size) as usize;

                guard.len() > max_nodes
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
        } else {
            // FIND-190: `prefetch_related` caches co-accessed nodes of ANY
            // tier, so a Cold overwrite must invalidate the stale entry —
            // otherwise get()/get_many() (SDK list) serve the pre-overwrite
            // payload. Same invalidation the txn commit path performs before
            // apply_insert_with_txn (txn.rs).
            // FIND-222: runs after the KV put above, so it is the writer's last
            // step — a prefetch that read the old payload earlier is cancelled
            // by the bump or loses the race and gets its entry removed here.
            self.cache.invalidate(node.id);
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold.
        // insert() may hold insert_lock here (ERR-010, non-reentrant), so
        // only auto-flush when the lock is free ΓÇö otherwise the checkpoint
        // would block on the very guard we hold. Skipped flushes happen at the
        // next uncontended call or user-initiated flush().
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.node_count() >= threshold && self.insert_lock.try_lock().is_some() {
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
    pub(crate) fn existing_meta(&self, id: u128) -> Option<ExistingMeta> {
        // Read-your-writes: check active txn buffer first (parity with get()).
        // Impossible-branch guard degrades to fallthrough (AUD-031): probe()
        // only errors on active-set corruption, which this best-effort probe
        // treats as a miss and continues to cache/backend.
        use crate::storage::engine::BufferProbe;
        match self.txn.probe(id) {
            Ok(BufferProbe::Insert(node)) => {
                return Some(ExistingMeta {
                    relational: node.relational.clone(),
                    edges: node.edges.clone(),
                });
            }
            Ok(BufferProbe::Delete) => {
                return None;
            }
            _ => {}
        }

        // Shared read() only — no write lock, no hits bookkeeping (probe, not a read).
        if let Some(node) = self.cache.volatile.read().get(&id) {
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

    #[cfg(not(feature = "rayon"))]
    fn existing_for_batch(&self, id: u128) -> Option<ExistingMeta> {
        self.existing_meta(id)
    }

    /// Batch variant of `existing_for_batch` (the `#[cfg(not(feature = "rayon"))]`
    /// sibling, which does not exist under this cfg) that amortizes lock
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
        // One lock pass per chunk via cloned_sole_buffer (ERR-037 amortization
        // preserved); impossible-branch degrades to fallthrough (AUD-031).
        if let Some(buffer) = self.txn.cloned_sole_buffer() {
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
        for (i, r) in resolved.iter().enumerate() {
            if !r {
                missing.push(i);
            }
        }

        // One shared read() over the cache for all ids in the chunk.
        if !missing.is_empty() {
            let guard = self.cache.volatile.read();
            missing.retain(|&i| {
                if let Some(node) = guard.get(&ids[i]) {
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
    /// **Batch-wide atomicity (FIND-640):** with WAL durability enabled
    /// (`!opts.skip_wal && wal.is_some()`) the whole batch is framed in an
    /// implicit WAL transaction: `Begin(batch_txn)` → per-chunk `Insert`
    /// `batch_append`s → `Prepare{txn_id, op_count}` → per-chunk store apply →
    /// `Commit(batch_txn)`, all under a single `insert_lock` hold. The durable
    /// `Commit` is the commit point: a crash before it leaves zero records of
    /// the batch after reopen (recovery discards the frame and the undo pass
    /// purges applied entries via `created_by_txn`); a crash after it replays
    /// the `Insert` records idempotently. An in-process error rolls back the
    /// applied prefix and appends a best-effort `Abort`. With `skip_wal`
    /// there is no frame — the legacy per-chunk durable units remain (each
    /// chunk stays independently durable; opt-in for regenerable bulk loads).
    ///
    /// HNSW behavior is controlled by `InsertMode`:
    /// - `InsertMode::Incremental` ΓÇö inserts each node into the HNSW index
    ///   as it is written. Good for small batches.
    /// - `InsertMode::Rebuild` ΓÇö skips HNSW insertion; caller must call
    ///   `rebuild_vector_index()` afterward. Good for large batches.
    /// - `InsertMode::Auto` (default) ΓÇö chooses between the two above
    ///   based on batch size vs `incremental_threshold` (default: 1000).
    ///
    /// MUST NOT be called while holding `insert_lock` (non-reentrant): the
    /// rollback path re-acquires it — with a blocking `lock()` on threaded
    /// targets (deadlock) and with `try_lock()`, under the caller's guard, on
    /// wasm32 (single-threaded: it degrades to no lock, never panics).
    #[tracing::instrument(skip(self, nodes), level = "debug", err)]
    pub fn batch_insert_with_opts(
        &self,
        nodes: &[UnifiedNode],
        opts: BatchInsertOptions,
    ) -> Result<()> {
        let Some(now_ms) = self.batch_prelude(nodes)? else {
            return Ok(());
        };

        // ─── FASE 1: Pre-validación total en memoria (All-or-Nothing upfront) ───
        // Valida el 100% de los elementos antes de tocar WAL, B-Tree (KV), VStore o HNSW (FIND-221).
        self.validate_batch_elements(nodes)?;

        // ─── FASE 2: Ejecución atómica en chunks con Rollback integral ───
        const BATCH_CHUNK_SIZE: usize = 1000;

        let should_insert_hnsw = batch_should_insert_hnsw(&opts, nodes.len());

        struct AppliedEntry {
            id: u128,
            pre_existing_raw: Option<Vec<u8>>,
            vstore_offset: u64,
            inserted_hnsw: bool,
        }

        let mut applied_records: Vec<AppliedEntry> = Vec::with_capacity(nodes.len());
        let mut all_staged_offsets: Vec<u64> = Vec::with_capacity(nodes.len());

        // FIND-640: when a WAL is active the whole batch runs inside an
        // implicit transaction frame ([Begin → Insert records → Prepare] →
        // apply → Commit). The commit point is the durable `Commit` marker;
        // recovery discards every frame without one (open_txn EOF/Abort rule)
        // and the TXN-01 undo pass purges applied KV/HNSW/vstore entries via
        // `created_by_txn == batch_txn`.
        //
        // `batch_txn` is a dedicated id NOT registered in `txn.active` —
        // registering it would make `insert()`/`delete()` observe
        // `has_active()==true` and buffer foreign writes into a buffer this
        // frame never drains (they would silently die with it). The stamp is
        // still applied on every path (it is also this batch's `created_by_txn`
        // pseudo-txn id, like `snapshot_id()` was before).
        //
        // `opts.skip_wal` opts out of the frame entirely (no WAL → nothing to
        // discard; documented opt-in for regenerable bulk loads).
        let wal_framed = !opts.skip_wal && self.wal.is_some();
        let batch_txn = self.txn.alloc_implicit_id();

        let execution_result: Result<()> = (|| {
            // FIND-640 / FIND-651 (correctness, not optional): ONE
            // `insert_lock` guard held for the WHOLE batch. The recovery
            // `open_txn` tracking assumes the records of a txn occupy
            // *contiguous* global_seq slots. Every WAL mutation takes
            // `insert_lock`, so holding it across the frame guarantees no
            // foreign write interleaves between our per-chunk `batch_append`s
            // — an interleaved `Insert` would be misattributed to `batch_txn`
            // by `record_txn_id` and discarded with the prefix on recovery
            // (silent loss of an independently committed write). The old
            // per-chunk guard was only sound because each chunk was itself a
            // self-contained durable unit.
            let _guard = self.acquire_insert_lock(
                "acquire insert_lock in batch_insert_with_opts (FIND-640 txn frame)",
            )?;

            if wal_framed {
                // ── WAL phase: all durable truth before touching stores ──
                // Exact mirror of `commit_transaction` phase-1 (RES-01):
                // [Begin + ops + Prepare] becomes durable first, so a crash
                // during the WAL phase leaves zero applied work (undo pass
                // trivially empty) and a crash during apply finds every op
                // durable, discriminated by `created_by_txn`.
                self.wal_coordinator()
                    .append(&WalRecord::Begin(batch_txn))?;
                for (chunk_idx, chunk) in nodes.chunks(BATCH_CHUNK_SIZE).enumerate() {
                    // FIND-640 test hook: fails the `batch_append` of the
                    // second-and-later WAL chunks (chunk_idx > 0 → never the
                    // first), so tests can crash mid-WAL-phase deterministically
                    // (`wal_append_fail` cannot target a specific chunk under
                    // sharded WALs).
                    #[cfg(feature = "failpoints")]
                    fail::fail_point!("put_batch_wal_chunk_fail", chunk_idx > 0, |_| {
                        Err(crate::error::Error::Io(std::io::Error::other(
                            "Simulated batch WAL-phase chunk append failure",
                        )))
                    });
                    let _ = chunk_idx; // consumed only under `failpoints`
                    self.append_batch_wal(chunk, &opts, now_ms)?;
                }
                // RES-01 uniformity: every multi-op txn emits Prepare;
                // `op_count` stays a replay-time integrity cross-check.
                self.wal_coordinator().append(&WalRecord::Prepare {
                    txn_id: batch_txn,
                    op_count: nodes.len() as u32,
                })?;
            }

            // ── Apply phase: per-chunk staging under the same guard ──
            for (chunk_idx, chunk) in nodes.chunks(BATCH_CHUNK_SIZE).enumerate() {
                // FIND-640 test hook: fires at chunk boundaries inside the
                // apply loop (chunk_idx > 0 → the 2nd chunk onward) so tests
                // can fail or panic mid-apply after a committed prefix.
                #[cfg(feature = "failpoints")]
                fail::fail_point!("put_batch_chunk_commit_fail", chunk_idx > 0, |_| {
                    Err(crate::error::Error::Io(std::io::Error::other(
                        "Simulated batch apply-phase chunk failure",
                    )))
                });
                let _ = chunk_idx; // consumed only under `failpoints`

                let mut bufs = BatchStageBufs::with_capacity(chunk.len());

                let chunk_pre_existing: Vec<Option<Vec<u8>>> = if opts.skip_existing_check {
                    vec![None; chunk.len()]
                } else {
                    chunk
                        .iter()
                        .map(|n| {
                            self.backend
                                .get(BackendPartition::Default, &n.id.to_le_bytes())
                                .ok()
                                .flatten()
                        })
                        .collect()
                };

                // FIND-651: staging writes each node into the vstore mmap —
                // it must run inside the guard, not before it, so a
                // concurrent create_snapshot can't copy `vstore_L0.vanta`
                // mid-append while the batch's WAL/KV commit is pending.
                self.serialize_batch_nodes(
                    chunk,
                    now_ms,
                    should_insert_hnsw,
                    &mut bufs,
                    batch_txn,
                )?;
                all_staged_offsets.extend_from_slice(&bufs.vstore_offsets);
                if !wal_framed {
                    // No txn frame (`skip_wal` or WAL disabled): keep the
                    // legacy interleaved WAL append per chunk.
                    self.append_batch_wal(chunk, &opts, now_ms)?;
                }
                let kv_ops = std::mem::take(&mut bufs.kv_ops);
                self.write_batch_kv_or_tombstone(kv_ops, &bufs.vstore_offsets)?;

                if should_insert_hnsw {
                    self.insert_hnsw_leveled(&bufs.hnsw_entries)?;
                }

                for (i, node) in chunk.iter().enumerate() {
                    applied_records.push(AppliedEntry {
                        id: node.id,
                        pre_existing_raw: chunk_pre_existing[i].clone(),
                        vstore_offset: bufs.vstore_offsets[i],
                        inserted_hnsw: should_insert_hnsw,
                    });
                }
            }

            if wal_framed {
                // COMMIT POINT (FIND-640): the durable `Commit` marker —
                // everything before it is preparation, everything after is
                // epilogue. After this append is durable the batch is
                // committed; before it, recovery discards the whole frame.
                self.wal_coordinator()
                    .append(&WalRecord::Commit(batch_txn))?;
            }
            Ok(())
        })();

        if let Err(err) = execution_result {
            // ─── ROLLBACK COMPLETO ANTE FALLO ───
            // FIND-651: rollback writes vstore tombstones + backend entries, so
            // it needs `insert_lock` like every other mutation. Block until it
            // is free — the old code ignored the `Result` and raced on anyway
            // after a timeout (the rollback must run regardless for in-memory
            // consistency; waiting is strictly safer than proceeding unlocked).
            #[cfg(not(target_arch = "wasm32"))]
            let _rollback_guard = self.insert_lock.lock();
            #[cfg(target_arch = "wasm32")]
            // Single-threaded: a free lock is the only reachable state. The
            // `Option` keeps the guard alive when taken, and there is no
            // `expect` (denied crate-wide) in a rollback path that must run
            // regardless.
            let rollback_guard = self.insert_lock.try_lock();
            #[cfg(target_arch = "wasm32")]
            debug_assert!(
                rollback_guard.is_some(),
                "insert_lock free on a single-threaded target"
            );
            #[cfg(target_arch = "wasm32")]
            let _rollback_guard = rollback_guard;
            let hnsw = self.hnsw.load();
            let mut vstore_opt = self.vstore0().ok();

            for applied in applied_records.into_iter().rev() {
                if applied.inserted_hnsw {
                    hnsw.remove_node(applied.id);
                }
                if let Some(ref mut vstore) = vstore_opt {
                    tombstone_offset(vstore, applied.vstore_offset, &err);
                }
                let key = applied.id.to_le_bytes();
                if let Some(prev_bytes) = applied.pre_existing_raw {
                    let _ = self
                        .backend
                        .put(BackendPartition::Default, &key, &prev_bytes);
                } else {
                    let _ = self.backend.delete(BackendPartition::Default, &key);
                }
                self.cache.invalidate(applied.id);
            }

            if let Some(ref mut vstore) = vstore_opt {
                for &offset in &all_staged_offsets {
                    tombstone_offset(vstore, offset, &err);
                }
            }

            // FIND-640: best-effort `Abort` closes the open WAL frame so
            // post-mortem recovery discards [Begin..Prepare] by the Abort
            // rule instead of EOF — symmetrical with `commit_transaction`'s
            // error path. `_rollback_guard` is still held here: the marker
            // mutates the WAL (FIND-651). Best-effort: if the append itself
            // fails the frame stays EOF-open and recovery discards it
            // fail-safe anyway (R6).
            if wal_framed {
                let _ = self.wal_coordinator().append(&WalRecord::Abort(batch_txn));
            }

            return Err(err);
        }

        // ─── POST-COMMIT EXITOSO: Estadísticas, caché y auto-flush ───
        #[cfg(feature = "rayon")]
        self.apply_batch_stats_rayon(nodes, &opts);
        #[cfg(not(feature = "rayon"))]
        self.apply_batch_stats_serial(nodes, &opts);

        let needs_eviction = self.cache_batch_hot_nodes(nodes);
        // FIND-651: `run_batch_eviction_if_needed` drives the `_locked`
        // eviction variant, which requires `insert_lock` to be held — but the
        // FIND-640 whole-batch guard above is already dropped (it ended with
        // the execution closure, right after the Commit append), so it must
        // be re-acquired here (non-reentrant; callers never hold it, per this
        // method's doc contract). A lock timeout skips this best-effort
        // eviction instead of failing an already-committed batch — the same
        // skip-when-contended policy `maybe_auto_flush` uses; the trigger is
        // still set so a later maintenance pass retries the eviction.
        if needs_eviction {
            match self.acquire_insert_lock("acquire insert_lock in batch_insert_with_opts eviction")
            {
                Ok(_guard) => self.run_batch_eviction_if_needed(needs_eviction),
                Err(e) => {
                    self.emergency_maintenance_trigger
                        .store(true, Ordering::Release);
                    tracing::warn!("batch post-commit eviction skipped: {e}");
                }
            }
        }
        self.maybe_auto_flush();
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod t32_tests {
    use super::*;
    use crate::config::Config;
    use crate::storage::{BackendKind, StorageEngine};

    #[test]
    fn test_prealloc_vstore_batch_fits_above_half_memory_limit() {
        // T32: when vstore.size is > 50% of mem_growth_cap and needed <= cap,
        // prealloc_vstore_batch must clamp to cap via ensure_capacity rather
        // than failing prematurely on `vstore.size * 2 > cap`.
        let mut vstore =
            crate::storage::vfile::File::create_in_memory(3072).with_mem_growth_cap(4096);
        vstore.write_cursor = 2048;
        // batch_len = 1 -> batch_estimate = 1280 -> needed = 3328 <= 4096.
        prealloc_vstore_batch(&mut vstore, 1)
            .expect("needed (3328) <= cap (4096) must not fail prematurely on 2x doubling");
        assert!(vstore.size >= 3328 && vstore.size <= 4096);
    }

    #[test]
    fn test_prealloc_vstore_batch_in_memory_engine_above_half_limit() {
        // Integration check in :memory: StorageEngine with declared memory_limit.
        let cap = 256 * 1024u64;
        let engine = StorageEngine::open_with_config(
            ":memory:",
            Some(Config {
                backend_kind: BackendKind::InMemory,
                memory_limit: Some(cap),
                ..Config::default()
            }),
        )
        .expect("open :memory: engine");

        let mut vstore = engine.vstore0().expect("vstore0");
        // Grow to 60% of cap, then prealloc another batch that keeps total <= cap.
        vstore.grow_to(160 * 1024).expect("grow to 160 KiB");
        vstore.write_cursor = 160 * 1024;
        prealloc_vstore_batch(&mut vstore, 10)
            .expect("needed (160 KiB + 12.8 KiB) <= 256 KiB must succeed");
        assert!(vstore.size <= cap);
    }
}
