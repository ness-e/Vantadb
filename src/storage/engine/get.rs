//! Read operations: get, get_many, warm_hnsw_top_layer.

use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::BackendPartition;
use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::node::{FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferedWrite, FLAG_TOMBSTONE};
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

impl StorageEngine {
    /// Retrieve a node by its numeric ID, checking the volatile cache first.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get(&self, id: u128) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        self.quantization_governor.record_access(id);

        // Read-your-writes: check active txn buffer first
        {
            let active = self.active_txns.lock();
            if active.len() == 1 {
                let txn_id = active.iter().next().copied().ok_or_else(|| {
                    crate::error::VantaError::generic_error(
                        "active transaction set corrupted: len()==1 but no txn id".to_string(),
                    )
                })?;
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
            // ERR-036: never take a blocking write lock on the read hot path.
            // try_write bumps hits/last_accessed when uncontended; when a
            // writer is active we degrade to a read-only (shared) lookup so
            // concurrent readers never serialize behind a writer.
            match self.volatile_cache.try_write() {
                Some(mut cache) => {
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
                None => {
                    if let Some(node) = self.volatile_cache.read().get(&id) {
                        if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                            return Ok(None);
                        }
                        return Ok(Some(node.clone()));
                    }
                }
            }
        }

        let key = id.to_le_bytes();
        let metadata_res = match self.backend.get(BackendPartition::Default, &key)? {
            Some(res) => res,
            None => return Ok(None),
        };

        let metadata: NodeMetadata =
            crate::storage::ops::deserialize_node_payload(&metadata_res, "node metadata")?;

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
        debug_assert_eq!(
            vec_bytes.as_ptr().align_offset(4),
            0,
            "f32 vector must be 4-byte aligned"
        );
        // SAFETY: 1) bounds ΓÇö `vec_end` is guarded against exceeding the mapping
        // size, so `vec_bytes` is an in-mapping byte slice of exactly
        // `vector_len*4` bytes; 2) alignment ΓÇö `read_header` rejects non-4-multiple
        // `vector_offset` (INV-024 M-1 central guard), so `vec_bytes.as_ptr()` is
        // 4-byte aligned even in release (where debug_assert is compiled out);
        // 3) lifetime ΓÇö bounded by the caller's read lock on the storage engine;
        // the to_vec() copy below clears the borrow, so no aliasing concern.
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = FilterBitset::from_u128(header.bitset);
        node.vector = VectorRepresentations::Full(f32_vec.to_vec());
        // Preserve quantization format from HNSW ΓÇö get() reads f32 from vstore
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
            let cache = self.volatile_cache.read();
            self.cache_warmer
                .suggest_warm_ids(id, |i| cache.contains_key(&i))
        };
        if to_fetch.is_empty() {
            return;
        }
        for warm_id in to_fetch {
            // Recursive call: the PrefetchGuard above stops this from cycling
            // (a nested get() → prefetch_related is a no-op). No locks are
            // held at the call site.
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
        // ERR-036 (FND-02): never take a blocking write lock on the read path.
        // try_write bumps hits/last_accessed when uncontended; when a writer is
        // active we degrade to a read-only lookup (stats not bumped) so batch
        // reads never serialize behind a writer — same contract as get().
        match self.volatile_cache.try_write() {
            Some(mut cache) => {
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
            None => {
                let cache = self.volatile_cache.read();
                for (i, &id) in ids.iter().enumerate() {
                    self.quantization_governor.record_access(id);
                    if let Some(node) = cache.get(&id) {
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

            let metadata: NodeMetadata = match crate::storage::ops::deserialize_node_payload(
                metadata_bytes,
                "node metadata",
            ) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let Some(index_node) = hnsw.nodes.get(&id) else {
                continue;
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
            let Some(header) = vstore.read_header(local_off) else {
                continue;
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
}
