//! Read operations: get, get_many, warm_hnsw_top_layer.
//!
//! # StorageEngine get ↔ prefetch_related — intentional 2-node SCC (FIND-35, MCP-15, OLD-20)
//!
//! `get()` (cache miss) → `prefetch_related(id)` → `get(warm_id)` forms a deliberate
//! 2-node cycle to implement co-access prefetch (OLD-20). Without a bound, a mutual
//! co-access pair A↔B where both nodes are uncached recurses forever
//! (`get(A)→prefetch(A)→get(B)→prefetch(B)→get(A)…`) — `get()` never inserts the node
//! it materializes (only `prefetch_related`'s tail does post-recursion), so the
//! stack overflows (MCP-15). The cycle is bounded to **single-level** by
//! `PrefetchGuard` (`thread_local! Cell<bool>` + RAII `Drop` that clears on
//! unwind/panic). The outer `get()`'s prefetch runs; any nested
//! `get() → prefetch_related` inside the warm fetch is a no-op. CodeGraph therefore
//! reports a syntactic SCC, but operationally it is a DAG once the guard is
//! considered. `test_get_prefetch_does_not_recurse_forever` (cold-tier A↔B, 3
//! co-access records) is the regression guard. Invariant: `get` and
//! `prefetch_related` are synchronous and same-thread; `thread_local` is
//! sufficient. If prefetch ever becomes `async`/cross-task, migrate the guard to
//! `tokio::task_local!`. `warm_hnsw_top_layer` also re-enters `get` but is likewise
//! bounded by the same guard.
//! // ponytail: doc justifies intentional SCC single-level; flatten if prefetch becomes async (thread_local → task_local)

use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::BackendPartition;
use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FilterBitset, NodeTier, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferProbe, FLAG_TOMBSTONE};
use crate::storage::ops::NodeMetadata;

// MCP-15: re-entrancy guard for `prefetch_related`.
//
// `get()` (cache miss) calls `prefetch_related(id)`, which fetches each warm
// id via a recursive `self.get(warm_id)`. Without a guard, a co-access pair
// (A↔B) where BOTH nodes are cache misses recurses
// `get(A)→prefetch(A)→get(B)→prefetch(B)→get(A)→…` forever: `get()` never
// inserts the node it materializes (only `prefetch_related`'s tail does, after
// the recursion unwinds), so A and B remain mutually uncached through the
// whole chain and the stack overflows on the server worker thread. The guard
// makes prefetch single-level — the OLD-20 contract (prefetch the co-accessed
// nodes of the accessed node, not transitively).
thread_local! {
    static PREFETCH_IN_PROGRESS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII guard that acquires the prefetch re-entrancy flag and releases it on
/// drop (including unwind), so a panic mid-prefetch cannot leave the flag set
/// and silently disable future prefetches on the same worker thread.
struct PrefetchGuard;
impl PrefetchGuard {
    fn acquire() -> Option<Self> {
        if PREFETCH_IN_PROGRESS.with(|f| f.replace(true)) {
            None
        } else {
            Some(Self)
        }
    }
}
impl Drop for PrefetchGuard {
    fn drop(&mut self) {
        PREFETCH_IN_PROGRESS.with(|f| f.set(false));
    }
}

// ─── D1a Slice 1: pure vector decoders (SLAP helpers for `get`) ───
//
// Each decoder takes only primitives + the raw mmap bytes, so a corrupt
// header maps to `None` (the caller converts it to `Ok(None)` / `continue`,
// exactly like the inline code did). No I/O, no locks, no effects.

/// Decode a FULL (f32) vector payload. `None` = corrupt header/bounds.
pub(crate) fn decode_full_bytes(
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    let vec_len_bytes = (vector_len as u64).checked_mul(4)?;
    let vec_end = vector_offset.checked_add(vec_len_bytes)?;
    if vec_end > bytes.len() as u64 {
        return None;
    }
    let slice = &bytes[vector_offset as usize..vec_end as usize];
    debug_assert_eq!(slice.as_ptr().align_offset(4), 0);
    // SAFETY: slice is bounds-checked above with the exact f32 count and
    // alignment is debug-asserted — same guarantees as the inline code.
    let f32_vec: &[f32] =
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const f32, vector_len as usize) };
    Some(VectorRepresentations::Full(f32_vec.to_vec()))
}

/// Decode a BINARY (u64 words) payload. `None` = corrupt header/bounds.
pub(crate) fn decode_binary_bytes(
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    let vec_len_bytes = (vector_len as u64).checked_mul(8)?;
    let vec_end = vector_offset.checked_add(vec_len_bytes)?;
    if vec_end > bytes.len() as u64 {
        return None;
    }
    let slice = &bytes[vector_offset as usize..vec_end as usize];
    debug_assert_eq!(slice.as_ptr().align_offset(8), 0);
    // SAFETY: bounds-checked above; `align_to` yields only aligned words.
    let (_, u64_slice, _) = unsafe { slice.align_to::<u64>() };
    if u64_slice.len() != vector_len as usize {
        return None;
    }
    let words = u64_slice.to_vec().into_boxed_slice();
    Some(VectorRepresentations::Binary(words))
}

/// Decode a TURBO (raw bytes) payload. `None` = corrupt header/bounds.
pub(crate) fn decode_turbo_bytes(
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    let vec_end = vector_offset.checked_add(vector_len as u64)?;
    if vec_end > bytes.len() as u64 {
        return None;
    }
    let slice = &bytes[vector_offset as usize..vec_end as usize];
    let raw = slice.to_vec().into_boxed_slice();
    Some(VectorRepresentations::Turbo(raw))
}

/// Decode an SQ8 (i8 data + f32 scale tail) payload. `None` = corrupt.
pub(crate) fn decode_sq8_bytes(
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    let payload_end = (vector_len as u64)
        .checked_add(4)
        .and_then(|b| vector_offset.checked_add(b))
        .filter(|&end| end <= bytes.len() as u64)?;
    let payload = &bytes[vector_offset as usize..payload_end as usize];
    let n = vector_len as usize;
    let scale = f32::from_le_bytes(payload[n..n + 4].try_into().ok()?);
    if !scale.is_finite() {
        return None;
    }
    let data: Vec<i8> = payload[..n].iter().map(|&b| b as i8).collect();
    Some(VectorRepresentations::SQ8(data.into_boxed_slice(), scale))
}

/// Legacy pre-ADR-032 branch: kind 0 with len>0 is FULL, len==0 is NONE.
pub(crate) fn decode_legacy_kind0(
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    if vector_len == 0 {
        return Some(VectorRepresentations::None);
    }
    decode_full_bytes(vector_len, vector_offset, bytes)
}

/// Dispatch decode by vector kind. `None` = corrupt (caller skips the
/// node); `Some(None)` = valid node with no vector (incl. legacy input).
pub(crate) fn decode_vector_by_kind(
    kind: u32,
    vector_len: u32,
    vector_offset: u64,
    bytes: &[u8],
) -> Option<VectorRepresentations> {
    use crate::node::NodeFlags;
    if kind == 0 {
        return decode_legacy_kind0(vector_len, vector_offset, bytes);
    }
    match kind {
        NodeFlags::VECTOR_KIND_FULL => decode_full_bytes(vector_len, vector_offset, bytes),
        NodeFlags::VECTOR_KIND_BINARY => decode_binary_bytes(vector_len, vector_offset, bytes),
        NodeFlags::VECTOR_KIND_TURBO => decode_turbo_bytes(vector_len, vector_offset, bytes),
        NodeFlags::VECTOR_KIND_SQ8 => decode_sq8_bytes(vector_len, vector_offset, bytes),
        _ => Some(VectorRepresentations::None),
    }
}

// ─── D1a Slice 2: lookup-phase helpers (SLAP for `get`) ───
//
// Mechanical split of `get()`'s fetch phases. No semantic change: same lock
// order (active_txns → txn_buffers → cache → backend → hnsw → vstore) and
// same tombstone/None mapping. One level per helper, body ≤20 lines.

/// Millis since epoch (single clock read for access bookkeeping).
pub(crate) fn now_ms_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl StorageEngine {
    /// Read-your-writes probe of the active txn buffer. `None` = miss
    /// (caller continues); `Some(Some)` = buffered insert; `Some(None)` =
    /// buffered delete. Delegates to [`TxnManager::probe`](super::txn::TxnManager::probe):
    /// same lock order and same tombstone/None mapping, no semantic change.
    pub(crate) fn lookup_txn_buffer(&self, id: u128) -> Result<Option<Option<UnifiedNode>>> {
        match self.txn.probe(id)? {
            BufferProbe::NoSoleTxn | BufferProbe::Miss => Ok(None),
            BufferProbe::Insert(n) => Ok(Some(Some(n))),
            BufferProbe::Delete => Ok(Some(None)),
        }
    }

    /// Volatile-cache probe (ERR-036: never block the read hot path).
    /// `None` = miss; `Some(Some)` = hit; `Some(None)` = tombstoned hit.
    /// Delegates to [`CacheLayer::lookup_volatile`](super::cache::CacheLayer::lookup_volatile):
    /// same lock order and same tombstone/None mapping, no semantic change.
    pub(crate) fn lookup_volatile(&self, id: u128) -> Option<Option<UnifiedNode>> {
        self.cache.lookup_volatile(id)
    }

    /// KV metadata fetch. `None` = absent (caller returns `Ok(None)`).
    pub(crate) fn fetch_backend_metadata(&self, id: u128) -> Result<Option<NodeMetadata>> {
        let key = id.to_le_bytes();
        let Some(raw) = self.backend.get(BackendPartition::Default, &key)? else {
            return Ok(None);
        };
        Ok(Some(crate::storage::ops::deserialize_node_payload(
            &raw,
            "node metadata",
        )?))
    }

    /// HNSW index → packed storage offset. `None` = not indexed.
    pub(crate) fn lookup_index_offset(&self, id: u128) -> Option<u64> {
        let hnsw = self.hnsw.load();
        hnsw.storage_offset_of(id)
    }

    /// vstore header at a packed offset. `None` = torn write; `Err` = bad segment.
    pub(crate) fn read_header_at(
        &self,
        id: u128,
        storage_offset: u64,
    ) -> Result<Option<crate::node::DiskNodeHeader>> {
        let (seg_id, local_off) = unpack_offset(storage_offset);
        let vstore = self.vstore_segment_reader(seg_id, id)?;
        Ok(vstore.read_header(local_off))
    }

    /// Shared read guard over one vstore segment. `Err` = bad segment.
    pub(crate) fn vstore_segment_reader(
        &self,
        seg_id: u8,
        id: u128,
    ) -> Result<parking_lot::RwLockReadGuard<'_, crate::storage::vfile::File>> {
        Ok(self
            .vector_store
            .get(seg_id as usize)
            .ok_or_else(|| {
                crate::error::Error::generic_error(format!(
                    "corrupt storage: segment {seg_id} out of range for node {id}"
                ))
            })?
            .read())
    }

    /// Decode the vector payload at a packed offset. `Ok(None)` = corrupt
    /// bounds (caller skips the node); `Err` = bad segment.
    pub(crate) fn decode_vector_at(
        &self,
        storage_offset: u64,
        header: &crate::node::DiskNodeHeader,
    ) -> Result<Option<VectorRepresentations>> {
        let (seg_id, _) = unpack_offset(storage_offset);
        let vstore = self.vstore_segment_reader(seg_id, header.id)?;
        let kind = crate::node::NodeFlags::vector_kind(header.flags);
        Ok(decode_vector_by_kind(
            kind,
            header.vector_len,
            header.vector_offset,
            vstore.mmap_bytes(),
        ))
    }

    /// ADR-032 legacy rescue: kind 0/NONE with an empty decode falls back to
    /// the HNSW-resident quantized payload. Non-legacy kinds pass through.
    pub(crate) fn rescue_legacy_vector(
        &self,
        id: u128,
        kind: u32,
        decoded: VectorRepresentations,
    ) -> VectorRepresentations {
        let legacy = kind == 0 || kind == crate::node::NodeFlags::VECTOR_KIND_NONE;
        let empty = matches!(decoded, VectorRepresentations::None) || decoded.dimensions() == 0;
        if !legacy || !empty {
            return decoded;
        }
        let hnsw = self.hnsw.load();
        let Some(vec_data) = hnsw.stored_vector(id) else {
            return decoded;
        };
        pick_rescue_payload(&vec_data, decoded)
    }

    /// Materialize a node past the txn/cache fast paths: backend metadata →
    /// index offset → header → tombstone → decode → rescue → assemble.
    /// `Ok(None)` = absent at any stage (same mapping as the inline code).
    pub(crate) fn materialize_uncached(&self, id: u128) -> Result<Option<UnifiedNode>> {
        let Some(metadata) = self.fetch_backend_metadata(id)? else {
            return Ok(None);
        };
        let Some(storage_offset) = self.lookup_index_offset(id) else {
            return Ok(None);
        };
        let Some(header) = self.read_header_at(id, storage_offset)? else {
            return Ok(None);
        };
        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(None);
        }
        let Some(vector) = self.decode_vector_at(storage_offset, &header)? else {
            return Ok(None);
        };
        let kind = crate::node::NodeFlags::vector_kind(header.flags);
        let vector = self.rescue_legacy_vector(id, kind, vector);
        Ok(Some(assemble_node(id, &header, vector, metadata)))
    }
}

/// Prefer a legacy HNSW-resident quantized payload over an empty decode.
/// Pure: no I/O, no locks. Fallback is the decoded vector itself.
pub(crate) fn pick_rescue_payload(
    index_vec: &VectorRepresentations,
    decoded: VectorRepresentations,
) -> VectorRepresentations {
    use crate::node::VectorRepresentations as V;
    match index_vec {
        V::SQ8(d, s) => V::SQ8(d.clone(), *s),
        V::Binary(w) => V::Binary(w.clone()),
        V::Turbo(t) => V::Turbo(t.clone()),
        _ => decoded,
    }
}
/// Map the persisted tier byte (1 = Hot, anything else = Cold).
pub(crate) fn hot_or_cold(tier: u8) -> NodeTier {
    if tier == 1 {
        NodeTier::Hot
    } else {
        NodeTier::Cold
    }
}

/// Assemble the return node from header + decoded vector + KV metadata.
/// Pure (no I/O, no locks): header scores/flags thread straight through.
pub(crate) fn assemble_node(
    id: u128,
    header: &crate::node::DiskNodeHeader,
    vector: VectorRepresentations,
    metadata: NodeMetadata,
) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.bitset = FilterBitset::from_u128(header.bitset);
    node.vector = vector;
    node.relational = metadata.relational;
    node.edges = metadata.edges;
    node.confidence_score = header.confidence_score;
    node.importance = header.importance;
    node.tier = hot_or_cold(header.tier);
    node.flags = crate::node::NodeFlags(header.flags);
    node
}

impl StorageEngine {
    /// Retrieve a node by its numeric ID, checking the volatile cache first.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get(&self, id: u128) -> Result<Option<UnifiedNode>> {
        self.touch_activity();
        self.quantization_governor.record_access(id);

        // D1a: thin orchestrator — fast paths, one materialization, prefetch tail.
        if let Some(hit) = self.lookup_txn_buffer(id)? {
            return Ok(hit);
        }
        if let Some(hit) = self.lookup_volatile(id) {
            return Ok(hit);
        }
        let Some(node) = self.materialize_uncached(id)? else {
            return Ok(None);
        };
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
            // &***: Guard → Arc → Box → dyn (one deref per wrapper).
            crate::cache_warmer::CacheWarmer::hnsw_top_layer_ids(&***hnsw)
        };
        if top_ids.is_empty() {
            return;
        }
        // Use get() for each ΓÇö it checks cache first and fetches from stores.
        for &id in &top_ids {
            let _ = self.get(id);
        }
    }

    /// Prefetch nodes that are frequently co-accessed with the given ID.
    #[inline]
    fn prefetch_related(&self, id: u128) {
        // MCP-15: single-level prefetch. The recursive self.get() below would
        // otherwise re-enter prefetch_related for the warm id; when the
        // co-access pair is mutually uncached this cycles forever (stack
        // overflow). The guard lets the outer get()'s prefetch run but makes
        // any nested prefetch a no-op.
        let Some(_guard) = PrefetchGuard::acquire() else {
            return;
        };
        let to_fetch = {
            let guard = self.cache.volatile.read();
            self.cache
                .warmer
                .suggest_warm_ids(id, |i| guard.contains_key(&i))
        };
        if to_fetch.is_empty() {
            return;
        }
        for warm_id in to_fetch {
            // Recursive call: the PrefetchGuard above stops this from cycling
            // (a nested get() → prefetch_related is a no-op). No locks are
            // held at the call site.
            if let Ok(Some(node)) = self.get(warm_id) {
                let mut guard = self.cache.volatile.write();
                if let std::collections::hash_map::Entry::Vacant(e) = guard.entry(warm_id) {
                    e.insert(node);
                    self.cache.warmer.record_prefetch_hit();
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
        // ERR-036 (FND-02): never take a blocking write lock on the read path.
        // try_write bumps hits/last_accessed when uncontended; when a writer is
        // active we degrade to a read-only lookup (stats not bumped) so batch
        // reads never serialize behind a writer — same contract as get().
        match self.cache.volatile.try_write() {
            Some(mut guard) => {
                for (i, &id) in ids.iter().enumerate() {
                    self.quantization_governor.record_access(id);
                    if let Some(node) = guard.get_mut(&id) {
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
            None => {
                let guard = self.cache.volatile.read();
                for (i, &id) in ids.iter().enumerate() {
                    self.quantization_governor.record_access(id);
                    if let Some(node) = guard.get(&id) {
                        if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                            continue;
                        }
                        results.push(node.clone());
                    } else {
                        remaining_indices.push(i);
                    }
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
                crate::error::Error::backend_error(format!(
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

            let metadata: NodeMetadata = match crate::storage::ops::deserialize_node_payload(
                metadata_bytes,
                "node metadata",
            ) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let Some((storage_offset, index_vec)) = hnsw.node_view(id) else {
                continue;
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
            let Some(header) = vstore.read_header(local_off) else {
                continue;
            };

            if (header.flags & FLAG_TOMBSTONE) != 0 {
                continue;
            }

            let kind = crate::node::NodeFlags::vector_kind(header.flags);
            let vector = if kind == 0 {
                // legacy pre-ADR-032: kind 0 with len>0 is FULL, with len==0 is NONE (or rescue)
                if header.vector_len == 0 {
                    if let crate::node::VectorRepresentations::Binary(b) = &index_vec {
                        VectorRepresentations::Binary(b.clone())
                    } else if let crate::node::VectorRepresentations::SQ8(d, s) = &index_vec {
                        VectorRepresentations::SQ8(d.clone(), *s)
                    } else if let crate::node::VectorRepresentations::Turbo(t) = &index_vec {
                        VectorRepresentations::Turbo(t.clone())
                    } else {
                        VectorRepresentations::None
                    }
                } else {
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
                            continue;
                        };
                        let Some(vec_end) = header.vector_offset.checked_add(vec_len_bytes) else {
                            continue;
                        };
                        if vec_end > vstore.mmap_bytes().len() as u64 {
                            continue;
                        }
                        let vec_start = header.vector_offset as usize;
                        let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];
                        debug_assert_eq!(
                            slice.as_ptr().align_offset(4),
                            0,
                            "f32 must be 4-aligned"
                        );
                        let f32_vec: &[f32] = unsafe {
                            std::slice::from_raw_parts(
                                slice.as_ptr() as *const f32,
                                header.vector_len as usize,
                            )
                        };
                        VectorRepresentations::Full(f32_vec.to_vec())
                    }
                    crate::node::NodeFlags::VECTOR_KIND_BINARY => {
                        let Some(vec_len_bytes) = (header.vector_len as u64).checked_mul(8) else {
                            continue;
                        };
                        let Some(vec_end) = header.vector_offset.checked_add(vec_len_bytes) else {
                            continue;
                        };
                        if vec_end > vstore.mmap_bytes().len() as u64 {
                            continue;
                        }
                        let vec_start = header.vector_offset as usize;
                        let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];
                        debug_assert_eq!(
                            slice.as_ptr().align_offset(8),
                            0,
                            "u64 must be 8-aligned"
                        );
                        let (_, u64_slice, _) = unsafe { slice.align_to::<u64>() };
                        if u64_slice.len() != header.vector_len as usize {
                            continue;
                        }
                        VectorRepresentations::Binary(u64_slice.to_vec().into_boxed_slice())
                    }
                    crate::node::NodeFlags::VECTOR_KIND_TURBO => {
                        let Some(vec_end) =
                            header.vector_offset.checked_add(header.vector_len as u64)
                        else {
                            continue;
                        };
                        if vec_end > vstore.mmap_bytes().len() as u64 {
                            continue;
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
                            continue;
                        };
                        let vec_start = header.vector_offset as usize;
                        let payload = &vstore.mmap_bytes()[vec_start..payload_end as usize];
                        let n = header.vector_len as usize;
                        let scale =
                            f32::from_le_bytes(payload[n..n + 4].try_into().unwrap_or([0; 4]));
                        if !scale.is_finite() {
                            continue;
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

            results.push(node);
        }

        // OLD-20: Record co-access patterns for all IDs fetched together.
        if results.len() >= 2 {
            let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
            self.cache.warmer.record_co_access(&ids);
        }

        Ok(results)
    }
}
