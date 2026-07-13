//! Storage engine: persistent vector store, WAL, HNSW index coordination.
//!
//! [`StorageEngine`] is the central persistence façade—it owns the backend
//! (in-memory, Fjall, or RocksDB), manages column-family partitions, and
//! drives node archival / recovery.

mod init;
mod maintenance;
mod ops;
mod partition;
mod stats;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::RwLock;

pub use crate::backend::BackendPartition;
use crate::backend::StorageBackend;
use crate::config::VantaConfig;
use crate::error::Result;
use crate::index::CPIndex;
use crate::node::UnifiedNode;
use crate::storage::vfile::{engine_mmap_resident_bytes, VantaFile};

// ─── Constants ──────────────────────────────────────────────────

pub(crate) const FLAG_TOMBSTONE: u32 = 0x8;
pub(crate) const STORAGE_ALIGNMENT: u64 = 64;
pub(crate) const MIB: u64 = 1024 * 1024;
pub(crate) const GIB: u64 = 1024 * 1024 * 1024;

// ─── Backend Kind ──────────────────────────────────────────

/// Selects which KV backend `StorageEngine` uses.
pub use crate::backend::BackendKind;

/// Memory usage statistics for a `StorageEngine` instance.
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Estimated logical memory footprint in bytes.
    pub logical_bytes: u64,
    /// Approximate resident set size (pages actually in RAM), if available.
    pub physical_rss: Option<u64>,
    /// Number of nodes currently indexed in the HNSW graph.
    pub node_count: u64,
    /// Number of entries in the volatile hot-node cache.
    pub cache_entries: usize,
    /// Total nodes evicted since startup.
    pub eviction_count: u64,
    /// Total bytes freed by eviction since startup.
    pub eviction_bytes: u64,
    /// Configured memory limit in bytes, or 0 if unlimited.
    pub memory_limit: u64,
    /// Number of SQ8-quantized nodes currently in the index.
    pub quantized_nodes: u64,
}

impl MemoryStats {
    /// Returns the physical RSS if available, otherwise falls back to logical estimate.
    #[inline]
    pub fn effective_bytes(&self) -> u64 {
        self.physical_rss.unwrap_or(self.logical_bytes)
    }

    /// Returns the ratio of effective usage to the memory limit (0.0–1.0).
    /// Returns 0.0 if the limit is 0 (unlimited).
    #[inline]
    pub fn pressure_ratio(&self) -> f64 {
        if self.memory_limit == 0 {
            return 0.0;
        }
        self.effective_bytes() as f64 / self.memory_limit as f64
    }
}

/// Why eviction was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvictionReason {
    /// High watermark exceeded.
    Watermark,
    /// OOM condition detected.
    Oom,
    /// Periodic maintenance cycle.
    #[default]
    Periodic,
    /// Manual trigger from the CLI or API.
    Manual,
}

/// Report returned by eviction operations.
#[derive(Debug, Clone, Copy)]
pub struct EvictionReport {
    /// Number of nodes successfully evicted from the volatile cache.
    pub evicted: usize,
    /// Number of candidate nodes scanned during eviction.
    pub scanned: usize,
    /// Why the eviction was triggered.
    pub reason: EvictionReason,
}

/// Report returned by quantization maintenance (PERF-09).
#[derive(Debug, Clone, Copy, Default)]
pub struct QuantizationMaintenanceReport {
    /// Number of nodes scanned for quantization decisions.
    pub scanned: u64,
    /// Number of nodes quantized from f32 → SQ8.
    pub quantized: u64,
    /// Number of nodes promoted from SQ8 → f32.
    pub promoted: u64,
}

/// Report returned by explicit ANN index rebuild operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRebuildReport {
    /// Total number of nodes scanned during rebuild.
    pub scanned_nodes: u64,
    /// Number of nodes with valid vectors added to the new index.
    pub indexed_vectors: u64,
    /// Number of tombstone (deleted) nodes skipped.
    pub skipped_tombstones: u64,
    /// Total rebuild duration in milliseconds.
    pub duration_ms: u64,
    /// File path where the rebuilt index was persisted.
    pub index_path: PathBuf,
    /// Whether the rebuild completed successfully.
    pub success: bool,
}

/// Central storage facade coordinating the KV backend, HNSW index, vector store, and WAL.
pub struct StorageEngine {
    /// Abstract KV backend. No RocksDB types leak through this field.
    pub(crate) backend: Arc<dyn StorageBackend>,
    /// Engine configuration including backend kind, memory limits, and sync mode.
    pub config: VantaConfig,
    /// If true, all mutating operations must be rejected.
    pub read_only: bool,
    /// Thread-safe HNSW index (swappable via RCU).
    pub hnsw: ArcSwap<CPIndex>,
    /// Serializes insert/refresh operations to avoid bidirectional
    /// neighbor update races. Searches acquire hnsw.read() freely.
    pub(crate) insert_lock: parking_lot::Mutex<()>,
    /// Volatile LRU cache for hot (frequently accessed) nodes.
    pub volatile_cache: RwLock<std::collections::HashMap<u128, UnifiedNode>>,
    /// Monotonic timestamp (ms since epoch) of the last query activity.
    pub last_query_timestamp: AtomicU64,
    /// Flag signalling emergency maintenance (e.g. cache pressure).
    pub emergency_maintenance_trigger: AtomicBool,
    /// Path to the data directory.
    pub data_dir: PathBuf,
    /// Vector store file for persistent node vector data.
    pub vector_store: RwLock<VantaFile>,
    /// Sharded write-ahead log for crash durability with reduced mutex contention.
    pub(crate) wal: Option<std::sync::Arc<crate::wal_sharded::ShardedWal>>,
    /// Memory governor for adaptive eviction
    pub(crate) memory_governor: Option<std::sync::Arc<crate::memory_governor::MemoryGovernor>>,
    /// Quantization governor for auto-transition f32 ↔ SQ8 (PERF-09)
    pub(crate) quantization_governor: std::sync::Arc<crate::vector::governor::QuantizationGovernor>,
    /// Global edge index for referential integrity.
    ///
    /// Tracks every directed edge `(source → target)` so that cascade delete
    /// (PERF-07) can find incoming edges when a node is removed.
    pub(crate) edge_index: Option<std::sync::Arc<crate::edge_index::EdgeIndex>>,
    /// Secondary scalar indexes.
    ///
    /// `field → value → [node_id]` hash map that turns
    /// [`filter_field`](StorageEngine::filter_field) from a full table scan
    /// into an O(1) lookup (PERF-08).
    pub(crate) scalar_index: Option<std::sync::Arc<crate::scalar_index::ScalarIndex>>,
    /// File handle for multi-process isolation lock
    pub(crate) _lock_file: Option<File>,
    /// In-memory cache for BM25 term stats to avoid redundant I/O during ingestion.
    pub(crate) text_stats_cache:
        RwLock<std::collections::HashMap<(String, String), crate::text_index::TextTermStats>>,
    /// In-memory cache for BM25 namespace stats.
    pub(crate) text_ns_cache:
        RwLock<std::collections::HashMap<String, crate::text_index::TextNamespaceStats>>,
    /// Lightweight cardinality statistics for query optimization.
    pub(crate) cardinality_stats:
        RwLock<std::collections::HashMap<String, std::collections::HashMap<String, usize>>>,
}

// ─── Internal helpers used across sub-modules ──────────────────

impl StorageEngine {
    /// Write a node to the vector store and return its storage offset.
    fn write_node_to_vstore(vstore: &mut VantaFile, node: &UnifiedNode) -> Result<u64> {
        crate::storage::ops::write_node_to_vstore(vstore, node)
    }

    /// Replay a single write operation during WAL recovery.
    fn replay_write_node(
        vstore: &mut VantaFile,
        hnsw: &CPIndex,
        backend: &dyn StorageBackend,
        node_id: u128,
        node: &UnifiedNode,
    ) -> Result<()> {
        use crate::backend::BackendPartition;
        use crate::storage::ops::NodeMetadata;
        let offset = crate::storage::ops::write_node_to_vstore(vstore, node)?;
        hnsw.add(node_id, node.bitset.clone(), node.vector.clone(), offset);
        let key = node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
        };
        let metadata_val = postcard::to_allocvec(&metadata)
            .map_err(|e| crate::error::VantaError::serialization(e))?;
        backend.put(BackendPartition::Default, &key, &metadata_val)?;
        Ok(())
    }
}

impl Drop for StorageEngine {
    /// Release the file lock when the engine is dropped.
    fn drop(&mut self) {
        #[cfg(feature = "fs2")]
        {
            if let Some(file) = &self._lock_file {
                let _ = fs2::FileExt::unlock(file);
            }
        }
    }
}
