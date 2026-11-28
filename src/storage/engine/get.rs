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

// ─── FIND-591: alignment-agnostic word decoders (no `unsafe`) ───
//
// The on-disk vector format is **little-endian**: the encrypted writer
// emits `word.to_le_bytes()` explicitly (`storage/ops.rs`,
// `materialize_payload`) and the plaintext writer's native byte cast is
// equivalent because VantaDB only compiles on little-endian targets
// (`const _` asserts in `index_port.rs` and `storage/vfile.rs`).
//
// Decoding byte-by-byte instead of `from_raw_parts`/`align_to` removes the
// alignment precondition: in an encrypted segment `node_bytes` is a
// decrypted heap blob whose base alignment is an implementation detail
// (today `AlignedBlob`, a `Vec<u64>`), so a typed cast over it was latent
// UB and `align_to` silently dropped the vector on a misaligned base.

/// Decode `bytes` into `f32` words, little-endian, no alignment needed.
/// A trailing <4-byte remainder is dropped (`chunks_exact`).
pub(crate) fn f32s_from_le_bytes(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Decode `bytes` into `u64` words, little-endian, no alignment needed.
/// A trailing <8-byte remainder is dropped (`chunks_exact`).
pub(crate) fn u64s_from_le_bytes(bytes: &[u8]) -> Vec<u64> {
    bytes
        .chunks_exact(8)
        .map(|c| u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
        .collect()
}

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
    Some(VectorRepresentations::Full(f32s_from_le_bytes(slice)))
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
    let words = u64s_from_le_bytes(slice).into_boxed_slice();
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

    /// KV metadata row (raw bytes). `None` = the backend holds no row for the
    /// id — the **only** positive proof of absence (`FIND-812`).
    pub(crate) fn fetch_backend_row(&self, id: u128) -> Result<Option<Vec<u8>>> {
        let key = id.to_le_bytes();
        self.backend.get(BackendPartition::Default, &key)
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
        let (seg_id, local_off) = unpack_offset(storage_offset);
        let vstore = self.vstore_segment_reader(seg_id, header.id)?;
        let kind = crate::node::NodeFlags::vector_kind(header.flags);
        // ADR-0057 W3: in an encrypted segment the payload lives inside the
        // decrypted node blob, not in the mapping.
        let bytes = vstore.node_bytes(local_off)?;
        Ok(decode_vector_by_kind(
            kind,
            header.vector_len,
            header.vector_offset,
            bytes.as_slice(),
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
    ///
    /// `FIND-812`: the outcome is **not** a bare `Option`. "The backend holds
    /// no row" and "the deserialization failed" used to collapse into the same
    /// `Ok(None)`, so a torn write was indistinguishable from a deletion — and
    /// the index rebuild, the other half of the operation, dropped the id for
    /// good. See [`Materialized`].
    pub(crate) fn materialize_uncached(&self, id: u128) -> Result<Materialized> {
        let raw = self.fetch_backend_row(id)?;
        let Some(raw) = raw else {
            return Ok(Materialized::Gap(RowGap::Absent));
        };
        self.materialize_row(id, &raw)
    }

    /// The same materialization over a backend row already in hand.
    ///
    /// `FIND-812`: a rebuild scans the partition, so re-reading every row it
    /// just read would double the I/O of the whole index.
    pub(crate) fn materialize_row(&self, id: u128, raw: &[u8]) -> Result<Materialized> {
        let metadata = match crate::storage::ops::deserialize_node_payload::<NodeMetadata>(
            raw,
            "node metadata",
        ) {
            Ok(metadata) => metadata,
            // The row EXISTS; only its bytes are unreadable. Reporting it as
            // absent is what let the rebuild delete a live row's index — but
            // the read path must keep failing loudly, so the error travels.
            Err(error) => return Ok(Materialized::Unreadable { id, error }),
        };
        // A transactional delete keeps the metadata row on purpose
        // (`deleted_by_txn`) and drops the index offset: without this check the
        // row would come back as "not indexed" and be re-indexed, resurrecting
        // a deleted record in the text/derived/sparse indexes.
        if metadata.deleted_by_txn.is_some() {
            return Ok(Materialized::Gap(RowGap::Tombstoned));
        }
        let Some(storage_offset) = self.lookup_index_offset(id) else {
            return Ok(Materialized::Degraded {
                node: node_from_metadata(id, metadata),
                gap: RowGap::NotIndexed,
            });
        };
        let Some(header) = self.read_header_at(id, storage_offset)? else {
            return Ok(Materialized::Degraded {
                node: node_from_metadata(id, metadata),
                gap: RowGap::HeaderCorrupt,
            });
        };
        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(Materialized::Gap(RowGap::Tombstoned));
        }
        let Some(vector) = self.decode_vector_at(storage_offset, &header)? else {
            return Ok(Materialized::Degraded {
                node: node_from_metadata(id, metadata),
                gap: RowGap::VectorCorrupt,
            });
        };
        let kind = crate::node::NodeFlags::vector_kind(header.flags);
        let vector = self.rescue_legacy_vector(id, kind, vector);
        Ok(Materialized::Node(assemble_node(
            id, &header, vector, metadata,
        )))
    }

    /// Enumerate the **row set** for a derived-index rebuild (`FIND-812`).
    ///
    /// The rebuild used to walk `scan_nodes()`, which silently drops every row
    /// whose header or vector does not decode (and every non-`Full` vector
    /// kind). Deleting all index entries and re-adding only what `scan_nodes`
    /// surfaces removed live rows from every read, and — because the backend
    /// row still exists — no later purge could ever see them again.
    ///
    /// Here the backend row set is the ground truth: a row whose side storage
    /// is unreadable is kept (degraded, without a vector) and counted, and a
    /// row whose metadata does not decode is reported by its id — the rebuild
    /// cannot reconstruct it (its payload lives in that same row), but the id
    /// stops being dropped in silence.
    pub(crate) fn scan_rows_for_rebuild(&self) -> Result<RebuildRows> {
        let mut rows = RebuildRows::default();
        for (key, value) in self.backend.scan(BackendPartition::Default)? {
            let Ok(id_bytes) = <[u8; 16]>::try_from(key.as_slice()) else {
                tracing::warn!(
                    key_len = key.len(),
                    "rebuild: backend row key is not a 16-byte node id; skipped"
                );
                continue;
            };
            let id = u128::from_le_bytes(id_bytes);
            match self.materialize_row(id, &value)? {
                Materialized::Node(node) => rows.nodes.push(node),
                Materialized::Degraded { node, gap } => {
                    rows.degraded.push((id, gap));
                    rows.nodes.push(node);
                }
                Materialized::Unreadable { id, .. } => rows.unreadable.push(id),
                Materialized::Gap(gap) => {
                    // A tombstoned row is the normal outcome of a delete.
                    // Anything else here means the backend holds a row the
                    // index cannot use at all — worth a trace, never a delete.
                    if gap != RowGap::Tombstoned {
                        tracing::debug!(
                            node_id = %id,
                            reason = gap.label(),
                            "rebuild: row has nothing to index"
                        );
                    }
                }
            }
        }
        Ok(rows)
    }
}

/// Why a row could not become a full node (`FIND-812`).
///
/// Only [`RowGap::Absent`] is a proof of absence. Everything else means the row
/// is there and cannot be read — which is *not* the same thing, and must never
/// be treated as one by anything that deletes index entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowGap {
    /// The backend holds no row for the id — the only positive proof of absence.
    Absent,
    /// The row carries a durable tombstone (or a transactional delete):
    /// deleted, by design.
    Tombstoned,
    /// The row exists but the vector index has no offset for it.
    NotIndexed,
    /// The row exists but its vstore header cannot be read (torn write).
    HeaderCorrupt,
    /// The row exists but its vector payload does not decode.
    VectorCorrupt,
}

impl RowGap {
    /// Stable label for logs / reports.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Tombstoned => "tombstoned",
            Self::NotIndexed => "not_indexed",
            Self::HeaderCorrupt => "header_corrupt",
            Self::VectorCorrupt => "vector_corrupt",
        }
    }
}

/// Outcome of [`StorageEngine::materialize_uncached`].
#[derive(Debug)]
pub(crate) enum Materialized {
    /// The row materialized in full.
    Node(UnifiedNode),
    /// The metadata row is intact but the side storage is not: the record can
    /// still be indexed (it just has no vector), so the id is kept.
    Degraded { node: UnifiedNode, gap: RowGap },
    /// Nothing to materialize, and *why*.
    Gap(RowGap),
    /// The row exists but its metadata does not decode. The decode error is
    /// carried along so the read path keeps failing **loudly**, exactly as it
    /// did before this change (`FIND-812`).
    Unreadable {
        id: u128,
        error: crate::error::Error,
    },
}

/// The backend row set a derived-index rebuild must cover (`FIND-812`).
#[derive(Debug, Default)]
pub(crate) struct RebuildRows {
    /// Rows that can be indexed (fully or degraded).
    pub(crate) nodes: Vec<UnifiedNode>,
    /// Rows kept with a degraded payload — counted and reported, never dropped.
    pub(crate) degraded: Vec<(u128, RowGap)>,
    /// Rows whose metadata does not decode: nothing can be rebuilt for them, so
    /// the caller reports the id (the row is still in the backend, visible to a
    /// later purge or repair).
    pub(crate) unreadable: Vec<u128>,
}

/// Build a node from its metadata row alone (no vector).
///
/// `FIND-812`: a corrupt vstore header or vector does not change what the
/// record *is* — its fields live in the backend row. Rebuilding from the
/// metadata keeps the id visible to every later repair.
pub(crate) fn node_from_metadata(id: u128, metadata: NodeMetadata) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = VectorRepresentations::None;
    node.relational = metadata.relational;
    node.edges = metadata.edges;
    node
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
        // `FIND-812`: only a full materialization is a node here. A degraded
        // row (corrupt header/vector) is deliberately NOT surfaced as a
        // vector-less ghost on the read path — the rebuild keeps its id so a
        // repair can see it, and `get()` keeps answering "not readable".
        // An unreadable metadata row still fails loudly, as before.
        let node = match self.materialize_uncached(id)? {
            Materialized::Node(node) => node,
            Materialized::Unreadable { error, .. } => return Err(error),
            Materialized::Degraded { .. } | Materialized::Gap(_) => return Ok(None),
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

            // ADR-0057 W3: in an encrypted segment the payload lives inside the
            // decrypted node blob, not in the mapping.
            let node_bytes = vstore.node_bytes(local_off)?;

            let kind = crate::node::NodeFlags::vector_kind(header.flags);
            let vector = if kind == 0 && header.vector_len == 0 {
                // legacy pre-ADR-032 rescue: kind 0 with len==0 takes the
                // index-resident quantized payload when there is one.
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
                // FIND-591: shared, alignment-agnostic decoders — `None`
                // means corrupt bounds and the node is skipped, exactly
                // like every inline branch did.
                let Some(decoded) = decode_vector_by_kind(
                    kind,
                    header.vector_len,
                    header.vector_offset,
                    node_bytes.as_slice(),
                ) else {
                    continue;
                };
                decoded
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

#[cfg(test)]
mod vector_decode_tests {
    //! FIND-591: the decoders must be alignment-agnostic — in an encrypted
    //! segment `node_bytes` is a decrypted heap blob, not a page-aligned
    //! mapping. Tests feed every decoder a deliberately **misaligned**
    //! buffer (1-byte prefix + `&buf[1..]`); the `miri_` prefix makes the
    //! CI `-- miri` filter pick them up.
    use super::*;

    /// A `Vec<u8>` whose `[1..]` view is NOT 4/8-byte aligned.
    fn unaligned_buf(payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(payload.len() + 1);
        buf.push(0xAA);
        buf.extend_from_slice(payload);
        buf
    }

    #[test]
    fn miri_vector_decode_full_unaligned() {
        let values = [1.5f32, -2.25, 0.0, 42.75];
        let payload: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let buf = unaligned_buf(&payload);
        let decoded = decode_full_bytes(4, 1, &buf).expect("decodes");
        match decoded {
            VectorRepresentations::Full(v) => assert_eq!(v, values),
            other => panic!("expected Full, got {other:?}"),
        }
    }

    #[test]
    fn miri_vector_decode_binary_unaligned() {
        let words = [0xDEADBEEFu64, 0x0123456789ABCDEF];
        let payload: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        let buf = unaligned_buf(&payload);
        let decoded = decode_binary_bytes(2, 1, &buf).expect("decodes");
        match decoded {
            VectorRepresentations::Binary(w) => assert_eq!(w.as_ref(), words),
            other => panic!("expected Binary, got {other:?}"),
        }
    }

    #[test]
    fn miri_vector_decode_full_via_dispatch_unaligned() {
        use crate::node::NodeFlags;
        let values = [3.25f32, -0.5];
        let payload: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let buf = unaligned_buf(&payload);
        let decoded =
            decode_vector_by_kind(NodeFlags::VECTOR_KIND_FULL, 2, 1, &buf).expect("decodes");
        match decoded {
            VectorRepresentations::Full(v) => assert_eq!(v, values),
            other => panic!("expected Full, got {other:?}"),
        }
    }

    #[test]
    fn miri_vector_decode_invalid_bounds_return_none() {
        let buf = [0u8; 32];
        // `end > len`.
        assert!(decode_full_bytes(4, 20, &buf).is_none());
        assert!(decode_binary_bytes(2, 20, &buf).is_none());
        // `checked_add` overflow on `vector_offset`.
        assert!(decode_full_bytes(1, u64::MAX, &buf).is_none());
        assert!(decode_binary_bytes(1, u64::MAX, &buf).is_none());
        // A huge `vector_len` that cannot fit the buffer.
        assert!(decode_full_bytes(u32::MAX, 0, &buf).is_none());
        assert!(decode_binary_bytes(u32::MAX, 0, &buf).is_none());
        // Dispatch must map the same failures to `None`.
        use crate::node::NodeFlags;
        assert!(decode_vector_by_kind(NodeFlags::VECTOR_KIND_FULL, 4, 20, &buf).is_none());
        assert!(decode_vector_by_kind(NodeFlags::VECTOR_KIND_BINARY, 2, 20, &buf).is_none());
    }

    #[test]
    fn miri_vector_decode_zero_len_keeps_contract() {
        // FULL with len 0 decodes to an empty vector (not `None`).
        match decode_full_bytes(0, 0, &[]).expect("decodes") {
            VectorRepresentations::Full(v) => assert!(v.is_empty()),
            other => panic!("expected Full, got {other:?}"),
        }
        match decode_binary_bytes(0, 0, &[]).expect("decodes") {
            VectorRepresentations::Binary(w) => assert!(w.is_empty()),
            other => panic!("expected Binary, got {other:?}"),
        }
        // Legacy kind 0 with len 0 is `None`-valued, not corrupt.
        assert!(matches!(
            decode_vector_by_kind(0, 0, 0, &[]),
            Some(VectorRepresentations::None)
        ));
    }
}
