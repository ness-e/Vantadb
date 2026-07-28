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
use parking_lot::{FairMutex, RwLock};

pub use crate::backend::BackendPartition;
use crate::backend::StorageBackend;
use crate::config::VantaConfig;
use crate::error::Result;
use crate::index::CPIndex;
pub use crate::index::FreshHnswReport;
use crate::lsm::pack_offset;
pub(crate) use crate::lsm::SegmentRegistry;
use crate::node::{FilterBitset, LabelIntern, UnifiedNode, VectorRepresentations};
use crate::storage::vfile::VantaFile;

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

/// An operation buffered inside an uncommitted transaction.
/// Written to WAL + stores atomically at commit time.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)] // UnifiedNode is hot-path; boxing adds indirection per insert
pub(crate) enum BufferedWrite {
    Insert(UnifiedNode),
    Delete(u128),
}

/// A read snapshot capturing a consistent view of committed data.
///
/// Created via [`StorageEngine::begin_snapshot`]. All reads using this
/// snapshot see only data committed at or before `txn_id`.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    /// The transaction ID at which this snapshot was taken.
    pub txn_id: u64,
}

/// A filesystem-level snapshot created via POSIX hard links (or copy on Windows).
///
/// Unlike the MVCC [`Snapshot`], this is a point-in-time copy of all data files
/// in the storage directory — instant O(1) on Unix via hard links, O(n) on Windows
/// via fallback copy.
#[derive(Debug, Clone)]
pub struct FsSnapshot {
    /// Path to the snapshot directory.
    pub path: PathBuf,
    /// When the snapshot was created.
    pub created_at: std::time::Instant,
}

/// A pending HNSW mutation awaiting batch flush.
#[derive(Clone)]
pub(crate) struct PendingHnswOp {
    pub id: u128,
    pub bitset: FilterBitset,
    pub vector: VectorRepresentations,
    pub storage_offset: u64,
    pub is_delete: bool,
}

/// Default batch size for HNSW micro-batching.
pub(crate) const HNSW_BATCH_SIZE: usize = 64;

/// Pipeline mode for the segment optimizer: chooses which maintenance
/// operations to run in [`StorageEngine::run_pipeline`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PipelineMode {
    /// Full pipeline: Vacuum → FreshHNSW → CompactL0 → CompactL1 → CompactL2 → Merge → Reindex.
    #[default]
    Full,
    /// Only purge tombstones from the HNSW index.
    VacuumOnly,
    /// Only compact fragmented segments via layout BFS.
    MergeOnly,
    /// Only rebuild the HNSW vector index from scratch.
    IndexOnly,
    /// Only repair orphan links in the HNSW graph.
    FreshHnswOnly,
    /// Pipeline: Vacuum → Compact all levels → FreshHNSW → Merge → Reindex.
    CompactOnly,
    /// Pipeline: Vacuum → CompactL0 only → FreshHNSW → Merge → Reindex.
    CompactL0Only,
}

/// Report from a single vacuum pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct VacuumReport {
    /// Number of HNSW nodes scanned.
    pub scanned_nodes: u64,
    /// Number of tombstoned nodes removed from the HNSW index.
    pub removed_nodes: u64,
    /// Estimated bytes reclaimed by removing tombstoned nodes.
    pub reclaimed_bytes: u64,
    /// Duration of the vacuum pass in milliseconds.
    pub duration_ms: u64,
    /// Whether the pass completed successfully.
    pub success: bool,
}

/// Report from a single merge (compaction) pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct MergeReport {
    /// Number of segments before compaction (always 1 for single VantaFile).
    pub segments_before: u64,
    /// Number of segments after compaction (always 1 for single VantaFile).
    pub segments_after: u64,
    /// Estimated bytes saved by compaction.
    pub saved_bytes: u64,
    /// Duration of the merge pass in milliseconds.
    pub duration_ms: u64,
    /// Whether the pass completed successfully.
    pub success: bool,
}

/// Report from a single LSM level compaction pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct LsmReport {
    /// Which level was compacted (0 = L0, 1 = L1, 2 = L2).
    pub level: u8,
    /// Number of nodes promoted to the next level.
    pub nodes_promoted: u64,
    /// Bytes freed from the source level.
    pub reclaimed_bytes: u64,
    /// Duration of the compaction pass in milliseconds.
    pub duration_ms: u64,
    /// Whether the pass completed successfully.
    pub success: bool,
}

/// Report from a complete [`PipelineMode`] run.
#[derive(Debug, Clone, Default)]
pub struct PipelineReport {
    /// Vacuum report, if that phase was executed.
    pub vacuum: Option<VacuumReport>,
    /// Merge report, if that phase was executed.
    pub merge: Option<MergeReport>,
    /// LSM compaction reports, one per compacted level.
    pub lsm: Option<Vec<LsmReport>>,
    /// Index rebuild report, if that phase was executed.
    pub index: Option<IndexRebuildReport>,
    /// FreshHNSW report, if that phase was executed.
    pub fresh_hnsw: Option<FreshHnswReport>,
    /// Total wall-clock duration of the pipeline.
    pub total_duration_ms: u64,
    /// Whether all executed phases succeeded.
    pub success: bool,
}

/// Configuration for the segment optimizer pipeline.
///
/// Controls automatic vacuum, merge, and reindex behaviour.
#[derive(Debug, Clone, Copy)]
pub struct SegmentOptimizerConfig {
    /// Master switch for the optimizer (default: true).
    pub enabled: bool,
    /// Tombstone fraction (as a percentage) that triggers vacuum (default: 15.0).
    pub vacuum_threshold_pct: f32,
    /// How often to auto-run the pipeline in seconds (default: 3600).
    pub auto_run_interval_secs: u64,
    /// Maximum wall-clock duration for one pipeline run (default: 300).
    pub max_pipeline_duration_secs: u64,
    /// Per-level LSM tree compaction and sizing configuration.
    pub lsm: crate::lsm::LsmConfig,
}

impl Default for SegmentOptimizerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vacuum_threshold_pct: 15.0,
            auto_run_interval_secs: 3600,
            max_pipeline_duration_secs: 300,
            lsm: crate::lsm::LsmConfig::default(),
        }
    }
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
    pub(crate) insert_lock: FairMutex<()>,
    /// Pending HNSW mutations awaiting batch flush under a single
    /// `insert_lock` acquisition (Rayon micro-batching, P1).
    pub(crate) pending_hnsw_batch: parking_lot::Mutex<Vec<PendingHnswOp>>,
    /// Volatile LRU cache for hot (frequently accessed) nodes.
    pub volatile_cache: RwLock<std::collections::HashMap<u128, UnifiedNode>>,
    /// Monotonic timestamp (ms since epoch) of the last query activity.
    pub last_query_timestamp: AtomicU64,
    /// Monotonic transaction ID counter (P3 Phase 1).
    pub(crate) next_txn_id: AtomicU64,
    /// Active transaction IDs (concurrent: multiple active txns allowed).
    /// Empty = no active transaction → insert/delete go direct.
    /// Used for: snapshot visibility, write-write conflict detection.
    pub(crate) active_txns: parking_lot::Mutex<std::collections::HashSet<u64>>,
    /// Per-transaction write buffer. Keyed by txn_id.
    /// Only the active txn's buffer is meaningful.
    pub(crate) txn_buffers: parking_lot::Mutex<std::collections::HashMap<u64, Vec<BufferedWrite>>>,
    /// Flag signalling emergency maintenance (e.g. cache pressure).
    pub emergency_maintenance_trigger: AtomicBool,
    /// Path to the data directory.
    pub data_dir: PathBuf,
    /// Vector store files for persistent node vector data — one per LSM level.
    /// Index 0 = L0 (hot), 1 = L1 (warm), 2 = L2 (cold).
    /// All new writes go to index 0. Reads use unpack_offset() to select the correct file.
    pub vector_store: Vec<RwLock<VantaFile>>,
    /// Multi-level LSM segment registry tracking level metadata.
    #[allow(dead_code)]
    pub(crate) segment_registry: SegmentRegistry,
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
    /// Predictive cache warmer for co-access tracking and prefetch (OLD-20).
    pub(crate) cache_warmer: crate::cache_warmer::CacheWarmer,
    /// Bidirectional edge label interner: String ↔ u32.
    /// Reduces per-edge label overhead from ~24-32 bytes to 4 bytes.
    pub(crate) label_intern: parking_lot::Mutex<LabelIntern>,
}

// ─── Internal helpers used across sub-modules ──────────────────

impl StorageEngine {
    /// Replay a single write operation during WAL recovery.
    /// Writes to L0 (always) and packs the segment_id into the offset.
    fn replay_write_node(
        vector_store: &[RwLock<VantaFile>],
        hnsw: &CPIndex,
        backend: &dyn StorageBackend,
        node_id: u128,
        node: &UnifiedNode,
    ) -> Result<()> {
        use crate::backend::BackendPartition;

        use crate::storage::ops::NodeMetadata;
        let mut l0 = vector_store[0].write();
        let local_off = crate::storage::ops::write_node_to_vstore(&mut l0, node)?;
        let packed = pack_offset(0, local_off);
        hnsw.add(node_id, node.bitset.clone(), node.vector.clone(), packed);
        let key = node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: 0, // recovery is pre-MVCC
            deleted_by_txn: None,
        };
        let metadata_val =
            postcard::to_allocvec(&metadata).map_err(crate::error::VantaError::serialization)?;
        backend.put(BackendPartition::Default, &key, &metadata_val)?;
        Ok(())
    }
}

// ─── Label Interning ───────────────────────────────────────

impl StorageEngine {
    /// Intern a label string, returning a stable u32 ID.
    /// Creates a new entry if the label hasn't been seen before.
    pub fn intern_label(&self, label: &str) -> u32 {
        self.label_intern.lock().intern(label)
    }

    /// Resolve a label_id back to its string, if known.
    pub fn resolve_label(&self, id: u32) -> Option<String> {
        self.label_intern.lock().resolve(id).map(|s| s.to_string())
    }

    /// Convert a `UnifiedNode` to an SDK `VantaNodeRecord`, resolving edge labels.
    pub fn node_to_record(&self, node: crate::node::UnifiedNode) -> crate::sdk::VantaNodeRecord {
        crate::sdk::serialization::graph_types::unified_to_record(node, &self.label_intern.lock())
    }
}

// ─── VecIndex accessor ─────────────────────────────────────

impl StorageEngine {
    /// Return a handle to the vector index (HNSW / IVF / flat) as a
    /// [`VecIndex`](crate::index::VecIndex) trait object.
    ///
    /// The returned [`arc_swap::Guard`] auto-derefs to [`CPIndex`], which
    /// implements [`VecIndex`](crate::index::VecIndex).  Callers invoke
    /// trait methods (`.search()`, `.len()`, …) without binding to the
    /// concrete index type.
    pub fn vec_index(&self) -> arc_swap::Guard<Arc<CPIndex>> {
        self.hnsw.load()
    }
}

// ─── Filesystem Snapshots ──────────────────────────────────

impl StorageEngine {
    /// Create an instant filesystem snapshot by hard-linking all data files.
    ///
    /// On Unix, this is O(1) per file — the kernel creates directory entries
    /// pointing at the same inode, so no data is copied. On Windows, falls
    /// back to [`std::fs::copy`] (O(n) per file) since `CreateHardLinkA`
    /// requires NTFS and may need admin rights.
    #[cfg(unix)]
    pub fn create_snapshot(&self, name: &str) -> crate::error::Result<FsSnapshot> {
        let snap_dir = self.data_dir.join("snapshots").join(name);
        std::fs::create_dir_all(&snap_dir)?;

        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("snapshot_create_fail", |_| {
                Err(crate::error::VantaError::IoError(std::io::Error::other(
                    "Simulated snapshot create I/O failure",
                )))
            });
        }

        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest = snap_dir.join(entry.file_name());
                std::os::unix::fs::link(&path, &dest)?;
            }
        }
        Ok(FsSnapshot {
            path: snap_dir,
            created_at: std::time::Instant::now(),
        })
    }

    /// Create a filesystem snapshot (Windows/WASM fallback using copy).
    #[cfg(any(windows, target_arch = "wasm32"))]
    pub fn create_snapshot(&self, name: &str) -> crate::error::Result<FsSnapshot> {
        let snap_dir = self.data_dir.join("snapshots").join(name);
        std::fs::create_dir_all(&snap_dir)?;

        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("snapshot_create_fail", |_| {
                Err(crate::error::VantaError::IoError(std::io::Error::other(
                    "Simulated snapshot create I/O failure",
                )))
            });
        }

        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let dest = snap_dir.join(entry.file_name());
                std::fs::copy(&path, &dest)?;
            }
        }
        Ok(FsSnapshot {
            path: snap_dir,
            created_at: std::time::Instant::now(),
        })
    }

    /// List existing snapshot names.
    pub fn list_snapshots(&self) -> crate::error::Result<Vec<String>> {
        let snap_dir = self.data_dir.join("snapshots");
        if !snap_dir.exists() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&snap_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort();
        Ok(names)
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
