//! Specialized subsystem coordinators / facades for
//! [`StorageEngine`](crate::storage::engine::StorageEngine).
//!
//! Provides zero-cost coordinator views over key engine subsystems:
//! - [`IndexCoordinator`]: HNSW indexing and micro-batching operations.
//! - [`VectorStoreService`]: Multi-level vector store file I/O and segment management.
//! - [`WalCoordinator`]: Write-ahead log appends, recovery queries, and checkpoint helpers.

#![allow(dead_code)]

use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock, RwLockWriteGuard};

use crate::backend::{BackendPartition, StorageBackend};
use crate::error::Result;
use crate::index_port::{IndexPort, IndexType, VectorStoreRef};
use crate::lsm::{pack_offset, unpack_offset, SegmentRegistry};
use crate::node::{DistanceMetric, FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::PendingHnswOp;
use crate::storage::vfile::File;
use crate::wal::WalRecord;
use crate::wal_sharded::ShardedWal;

/// Read-only view of the live vector index (FIND-655).
///
/// [`StorageEngine::vec_index`](crate::storage::StorageEngine::vec_index)
/// hands out this facade instead of the raw `dyn IndexPort` handle so that
/// no external caller can reach the graph mutators (`add_node`,
/// `remove_node`, `set_entry_point`, `repair_orphan_links`, …). Every
/// mutation of the index must go through an engine wrapper that holds
/// `insert_lock` — the FIND-651 quiesce barrier that `create_snapshot`
/// relies on while mirroring `vector_index.bin`.
///
/// The wrapped `arc_swap` guard pins the current `Arc` snapshot of the
/// index for the lifetime of this value (same lease semantics as
/// [`arc_swap::Guard`]); dropping it releases the snapshot.
pub struct IndexReader {
    guard: arc_swap::Guard<Arc<Box<dyn IndexPort>>>,
}

impl IndexReader {
    /// Wrap a loaded index guard. Crate-internal: only
    /// [`StorageEngine::vec_index`](crate::storage::StorageEngine::vec_index)
    /// constructs this facade.
    #[inline]
    pub(crate) fn new(guard: arc_swap::Guard<Arc<Box<dyn IndexPort>>>) -> Self {
        Self { guard }
    }

    /// Mirrors [`IndexPort::search_nearest`].
    #[inline]
    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        q_1bit: Option<&[u64]>,
        q_3bit: Option<(&[u8], f32)>,
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
    ) -> Vec<(u128, f32)> {
        self.guard
            .search_nearest(query_vec, q_1bit, q_3bit, query_mask, top_k, vector_store)
    }

    /// Mirrors [`IndexPort::search_with_method`].
    #[inline]
    pub fn search_with_method(
        &self,
        method: IndexType,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<(u128, f32)>> {
        self.guard
            .search_with_method(method, query_vec, query_mask, top_k, metric)
    }

    /// Mirrors [`IndexPort::search`].
    #[inline]
    pub fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
        distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        self.guard
            .search(query_vec, query_mask, top_k, vector_store, distance_metric)
    }

    /// Mirrors [`IndexPort::storage_offset_of`].
    #[inline]
    pub fn storage_offset_of(&self, id: u128) -> Option<u64> {
        self.guard.storage_offset_of(id)
    }

    /// Mirrors [`IndexPort::stored_vector`].
    #[inline]
    pub fn stored_vector(&self, id: u128) -> Option<VectorRepresentations> {
        self.guard.stored_vector(id)
    }

    /// Mirrors [`IndexPort::node_view`].
    #[inline]
    pub fn node_view(&self, id: u128) -> Option<(u64, VectorRepresentations)> {
        self.guard.node_view(id)
    }

    /// Mirrors [`IndexPort::is_sq8_vector`].
    #[inline]
    pub fn is_sq8_vector(&self, id: u128) -> Option<bool> {
        self.guard.is_sq8_vector(id)
    }

    /// Mirrors [`IndexPort::contains_node`].
    #[inline]
    pub fn contains_node(&self, id: u128) -> bool {
        self.guard.contains_node(id)
    }

    /// Mirrors [`IndexPort::vector_dimension`].
    #[inline]
    pub fn vector_dimension(&self) -> Option<usize> {
        self.guard.vector_dimension()
    }

    /// Mirrors [`IndexPort::node_count`].
    #[inline]
    pub fn node_count(&self) -> usize {
        self.guard.node_count()
    }

    /// Mirrors [`IndexPort::is_empty`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.guard.is_empty()
    }

    /// Mirrors [`IndexPort::total_nodes`].
    #[inline]
    pub fn total_nodes(&self) -> u64 {
        self.guard.total_nodes()
    }

    /// Mirrors [`IndexPort::entry_point`].
    #[inline]
    pub fn entry_point(&self) -> Option<u128> {
        self.guard.entry_point()
    }

    /// Mirrors [`IndexPort::max_layer`].
    #[inline]
    pub fn max_layer(&self) -> usize {
        self.guard.max_layer()
    }

    /// Mirrors [`IndexPort::all_node_ids`].
    #[inline]
    pub fn all_node_ids(&self) -> Vec<u128> {
        self.guard.all_node_ids()
    }

    /// Mirrors [`IndexPort::layer_neighbors`].
    #[inline]
    pub fn layer_neighbors(&self, id: u128, layer: usize) -> Vec<u128> {
        self.guard.layer_neighbors(id, layer)
    }

    /// Mirrors [`IndexPort::inline_neighbors`].
    #[inline]
    pub fn inline_neighbors(&self, id: u128, layer: usize) -> Vec<u128> {
        self.guard.inline_neighbors(id, layer)
    }

    /// Mirrors [`IndexPort::scan_entries`].
    #[inline]
    pub fn scan_entries(&self) -> Vec<(u128, u64, u32)> {
        self.guard.scan_entries()
    }

    /// Mirrors [`IndexPort::validate_index`].
    #[inline]
    pub fn validate_index(&self) -> std::result::Result<(), Vec<String>> {
        self.guard.validate_index()
    }

    /// Mirrors [`IndexPort::node_layers`].
    #[inline]
    pub fn node_layers(&self, id: u128) -> usize {
        self.guard.node_layers(id)
    }

    /// Mirrors [`IndexPort::estimate_memory_bytes`].
    #[inline]
    pub fn estimate_memory_bytes(&self) -> usize {
        self.guard.estimate_memory_bytes()
    }

    /// Mirrors [`IndexPort::flat_threshold`].
    #[inline]
    pub fn flat_threshold(&self) -> Option<usize> {
        self.guard.flat_threshold()
    }

    /// Mirrors [`IndexPort::distance_metric`].
    #[inline]
    pub fn distance_metric(&self) -> DistanceMetric {
        self.guard.distance_metric()
    }

    /// Mirrors [`IndexPort::index_kind`].
    #[inline]
    pub fn index_kind(&self) -> IndexType {
        self.guard.index_kind()
    }

    /// Mirrors [`MmapBackend::is_mmap`](crate::index_port::MmapBackend::is_mmap).
    #[inline]
    pub fn is_mmap(&self) -> bool {
        self.guard.is_mmap()
    }

    /// Mirrors
    /// [`MmapBackend::mmap_path`](crate::index_port::MmapBackend::mmap_path).
    #[inline]
    pub fn mmap_path(&self) -> Option<&std::path::Path> {
        self.guard.mmap_path()
    }

    /// Mirrors
    /// [`MmapBackend::mmap_resident_bytes`](crate::index_port::MmapBackend::mmap_resident_bytes).
    #[inline]
    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        self.guard.mmap_resident_bytes()
    }
}

/// Coordinator facade for the HNSW index and pending mutation batch.
///
/// Crate-internal: obtained via `StorageEngine::index_coordinator()`,
/// which is `pub(crate)`. Its mutating methods (`add_node`, `remove_node`,
/// `set_entry_point`) bypass `insert_lock` on purpose — callers run inside
/// the engine's lock-holding sections (FIND-651/FIND-655); exposing them
/// would reopen the quiesce-barrier bypass.
#[derive(Clone, Copy)]
pub(crate) struct IndexCoordinator<'a> {
    hnsw: &'a ArcSwap<Box<dyn IndexPort>>,
    pending_hnsw_batch: &'a Mutex<Vec<PendingHnswOp>>,
}

impl<'a> IndexCoordinator<'a> {
    /// Create a new coordinator wrapping the index handle and pending batch mutex.
    #[inline]
    pub(crate) fn new(
        hnsw: &'a ArcSwap<Box<dyn IndexPort>>,
        pending_hnsw_batch: &'a Mutex<Vec<PendingHnswOp>>,
    ) -> Self {
        Self {
            hnsw,
            pending_hnsw_batch,
        }
    }

    /// Load the current HNSW index handle.
    #[inline]
    pub(crate) fn load_index(&self) -> arc_swap::Guard<Arc<Box<dyn IndexPort>>> {
        self.hnsw.load()
    }

    /// Return the storage offset of a node if indexed.
    #[inline]
    pub(crate) fn storage_offset_of(&self, id: u128) -> Option<u64> {
        self.hnsw.load().storage_offset_of(id)
    }

    /// Remove a node from the index. Returns `true` if it was present.
    #[inline]
    pub(crate) fn remove_node(&self, id: u128) -> bool {
        self.hnsw.load().remove_node(id)
    }

    /// Return the number of indexed nodes.
    #[inline]
    pub(crate) fn node_count(&self) -> usize {
        self.hnsw.load().node_count()
    }

    /// Check if a node is present in the index.
    #[inline]
    pub(crate) fn contains_node(&self, id: u128) -> bool {
        self.hnsw.load().contains_node(id)
    }

    /// Return the current entry point node ID, if any.
    #[inline]
    pub(crate) fn entry_point(&self) -> Option<u128> {
        self.hnsw.load().entry_point()
    }

    /// Update the index entry point.
    #[inline]
    pub(crate) fn set_entry_point(&self, id: u128) {
        self.hnsw.load().set_entry_point(id);
    }

    /// Find a replacement entry point if available.
    #[inline]
    pub(crate) fn find_new_entry_point(&self) -> Option<u128> {
        self.hnsw.load().find_new_entry_point()
    }

    /// Add a node to the HNSW index.
    #[inline]
    pub(crate) fn add_node(
        &self,
        id: u128,
        bitset: FilterBitset,
        vector: VectorRepresentations,
        storage_offset: u64,
    ) -> Result<()> {
        self.hnsw
            .load()
            .add_node(id, bitset, vector, storage_offset)
    }

    /// Push an operation to the pending micro-batch and return the new batch length.
    pub(crate) fn push_pending_op(&self, op: PendingHnswOp) -> usize {
        let mut batch = self.pending_hnsw_batch.lock();
        batch.push(op);
        batch.len()
    }

    /// Drain all queued HNSW mutation operations and apply them to the index.
    ///
    /// Precondition: Caller MUST already hold `insert_lock` to prevent races.
    pub(crate) fn drain_batch_locked(&self) -> Result<bool> {
        let ops = {
            let mut pending = self.pending_hnsw_batch.lock();
            if pending.is_empty() {
                return Ok(false);
            }
            std::mem::take(&mut *pending)
        };

        let hnsw = self.hnsw.load();
        for op in ops {
            if op.is_delete {
                hnsw.remove_node(op.id);
            } else {
                hnsw.add_node(op.id, op.bitset, op.vector, op.storage_offset)?;
            }
        }
        Ok(true)
    }
}

/// Service coordinator for multi-level vector store (`vstore_L*.vanta`) files and segment registry.
#[derive(Clone, Copy)]
pub(crate) struct VectorStoreService<'a> {
    vector_store: &'a [RwLock<File>],
    segment_registry: &'a SegmentRegistry,
}

impl<'a> VectorStoreService<'a> {
    /// Create a new service coordinator.
    #[inline]
    pub(crate) fn new(
        vector_store: &'a [RwLock<File>],
        segment_registry: &'a SegmentRegistry,
    ) -> Self {
        Self {
            vector_store,
            segment_registry,
        }
    }

    /// Number of active vector store segments.
    #[inline]
    pub(crate) fn segment_count(&self) -> usize {
        self.vector_store.len()
    }

    /// Reference to the underlying multi-level segment registry.
    #[inline]
    pub(crate) fn segment_registry(&self) -> &'a SegmentRegistry {
        self.segment_registry
    }

    /// Acquire write guard on segment 0 (L0 hot tier).
    #[inline]
    pub(crate) fn vstore0(&self) -> Result<RwLockWriteGuard<'_, File>> {
        self.vector_store.first().map(|v| v.write()).ok_or_else(|| {
            crate::error::Error::generic_error(
                "corrupt storage: vector_store has no segment 0".to_string(),
            )
        })
    }

    /// Write a node to L0 segment and return `(local_offset, packed_offset)`.
    pub(crate) fn write_node_l0(&self, node: &UnifiedNode) -> Result<(u64, u64)> {
        Self::write_node_l0_slice(self.vector_store, node)
    }

    /// Static helper to write a node to L0 segment directly from a slice of files.
    pub(crate) fn write_node_l0_slice(
        vector_store: &[RwLock<File>],
        node: &UnifiedNode,
    ) -> Result<(u64, u64)> {
        let mut l0 = vector_store.first().map(|v| v.write()).ok_or_else(|| {
            crate::error::Error::generic_error(
                "corrupt storage: vector_store has no segment 0".to_string(),
            )
        })?;
        let local_off = crate::storage::ops::write_node_to_vstore(&mut l0, node)?;
        let packed = pack_offset(0, local_off);
        Ok((local_off, packed))
    }

    /// Set or clear tombstone flag at the given packed offset.
    pub(crate) fn set_tombstone_at_packed_offset(
        &self,
        packed_offset: u64,
        tombstoned: bool,
    ) -> Result<bool> {
        let (seg_id, local_off) = unpack_offset(packed_offset);
        if let Some(vs) = self.vector_store.get(seg_id as usize) {
            let mut vstore = vs.write();
            vstore.set_tombstone(local_off, tombstoned)
        } else {
            Ok(false)
        }
    }

    /// Flush all vector store segments to disk.
    pub(crate) fn flush_all_segments(&self) -> Result<()> {
        for vs in self.vector_store {
            vs.read().flush()?;
        }
        Ok(())
    }
}

/// Coordinator facade for the Sharded Write-Ahead Log.
#[derive(Clone, Copy)]
pub(crate) struct WalCoordinator<'a> {
    wal: Option<&'a Arc<ShardedWal>>,
}

impl<'a> WalCoordinator<'a> {
    /// Create a new coordinator wrapping the optional ShardedWal reference.
    #[inline]
    pub(crate) fn new(wal: Option<&'a Arc<ShardedWal>>) -> Self {
        Self { wal }
    }

    /// Whether WAL durability is enabled on this engine.
    #[inline]
    pub(crate) fn is_enabled(&self) -> bool {
        self.wal.is_some()
    }

    /// Append a single record to the WAL. No-op if WAL is disabled.
    #[inline]
    pub(crate) fn append(&self, record: &WalRecord) -> Result<()> {
        if let Some(wal) = self.wal {
            wal.append(record)?;
        }
        Ok(())
    }

    /// Append a batch of records to the WAL. No-op if WAL is disabled.
    #[inline]
    pub(crate) fn batch_append(&self, records: Vec<WalRecord>) -> Result<()> {
        if let Some(wal) = self.wal {
            wal.batch_append(records)?;
        }
        Ok(())
    }

    /// Return total records written across all WAL shards.
    #[inline]
    pub(crate) fn total_records_written(&self) -> u64 {
        self.wal.map(|w| w.total_record_count()).unwrap_or(0)
    }

    /// Flush (fsync) all WAL shards.
    #[inline]
    pub(crate) fn flush_all(&self) -> Result<()> {
        if let Some(wal) = self.wal {
            wal.flush_all()?;
        }
        Ok(())
    }

    /// Rotate all WAL shards (archive current and reopen fresh).
    #[inline]
    pub(crate) fn rotate_all(&self) -> Result<()> {
        if let Some(wal) = self.wal {
            wal.rotate_all()?;
        }
        Ok(())
    }

    /// Record a checkpoint sequence number in backend metadata.
    pub(crate) fn record_checkpoint_seq(
        &self,
        backend: &dyn StorageBackend,
        seq: u64,
    ) -> Result<()> {
        if seq > 0 {
            let seq_bytes =
                postcard::to_allocvec(&seq).map_err(crate::error::Error::serialization)?;
            backend.put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq_bytes,
            )?;
            backend.flush()?;
        }
        Ok(())
    }

    /// Reset checkpoint sequence number to 0.
    pub(crate) fn reset_checkpoint_seq(&self, backend: &dyn StorageBackend) -> Result<()> {
        let zero: [u8; 8] = 0u64.to_le_bytes();
        backend.put(BackendPartition::InternalMetadata, b"checkpoint_seq", &zero)?;
        backend.flush()?;
        Ok(())
    }

    /// Read the latest persisted checkpoint sequence number.
    pub(crate) fn read_checkpoint_seq(&self, backend: &dyn StorageBackend) -> Result<u64> {
        let seq = backend
            .get(BackendPartition::InternalMetadata, b"checkpoint_seq")?
            .and_then(|bytes| postcard::from_bytes::<u64>(&bytes).ok())
            .unwrap_or(0);
        Ok(seq)
    }
}
