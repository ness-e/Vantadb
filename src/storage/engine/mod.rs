//! Storage engine: persistent vector store, WAL, HNSW index coordination.
//!
//! [`StorageEngine`] is the central persistence façade—it owns the backend
//! (in-memory, Fjall, or RocksDB), manages column-family partitions, and
//! drives node archival / recovery.

mod cache;
pub mod coordinators;
mod delete;
// FIND-591: `pub(crate)` so `storage::archive` reaches the shared,
// alignment-agnostic vector decoders (`decode_vector_by_kind` et al.).
pub(crate) mod get;
mod init;
pub(crate) mod insert;
mod maintenance;
mod ops;
mod partition;
mod stats;
mod txn;

pub use self::coordinators::{IndexCoordinator, VectorStoreService, WalCoordinator};

#[cfg(test)]
mod tests;

use std::fs::File as StdFile;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::{FairMutex, FairMutexGuard, RwLock};
use web_time::{Instant, SystemTime, UNIX_EPOCH};

pub use crate::backend::BackendPartition;
use crate::backend::StorageBackend;
use crate::config::Config;
use crate::error::Result;
pub use crate::index_port::FreshHnswReport;
use crate::index_port::IndexPort;
pub(crate) use crate::lsm::SegmentRegistry;
use crate::node::{FilterBitset, LabelIntern, UnifiedNode, VectorRepresentations};
use crate::storage::vfile::File;

// ─── Constants ──────────────────────────────────────────────────

/// Whether the caller already holds `insert_lock` (D2: replaces `lock_held` /
/// `acquire: bool` flags in `consolidate_node_inner`, `evict_cold_nodes_inner`,
/// `apply_delete_inner`). The lock is non-reentrant: re-acquiring it times out
/// after `insert_lock_timeout_ms`, so lock-holding callers use `AssumeHeld`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LockPolicy {
    /// Acquire `insert_lock` for the critical section.
    Acquire,
    /// Caller already holds `insert_lock`; do not re-acquire.
    AssumeHeld,
}

/// Tombstone flag — single source of truth is `crate::node::NodeFlags::TOMBSTONE`
/// (F3X-H2: identical value `0x8`; index-side uses migrate straight to the kernel).
pub(crate) const FLAG_TOMBSTONE: u32 = crate::node::NodeFlags::TOMBSTONE;
pub(crate) const STORAGE_ALIGNMENT: u64 = 64;
pub(crate) const MIB: u64 = 1024 * 1024;
pub(crate) const GIB: u64 = 1024 * 1024 * 1024;

// ─── Backend Kind ──────────────────────────────────────────

/// Selects which KV backend `StorageEngine` uses.
pub use crate::backend::BackendKind;

/// Options passed to [`StorageEngine::batch_insert_with_opts`](crate::storage::StorageEngine::batch_insert_with_opts).
pub use self::ops::{BatchInsertOptions, InsertMode};

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
/// Defined in [`txn::TxnManager`]'s module (C2S3, SRP); re-exported here so
/// existing `crate::storage::engine::BufferedWrite` paths keep resolving.
pub(crate) use self::txn::{BufferProbe, BufferedWrite, TxnManager};

/// A read snapshot capturing a consistent view of committed data.
///
/// Created via [`StorageEngine::begin_snapshot`]. All reads using this
/// snapshot see only data committed at or before `txn_id`.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    /// The transaction ID at which this snapshot was taken.
    pub txn_id: u64,
}

/// A filesystem-level snapshot that copies mutable files and may link immutable
/// files on Unix (or copies everything on Windows).
///
/// Unlike the MVCC `Snapshot`, this is a point-in-time copy of all data files
/// in the storage directory — hard links (O(1)) for the immutable bulk on
/// Unix, copies for files the engine keeps writing to (WAL shards, HNSW
/// image) so post-snapshot writes cannot mutate the snapshot; O(n) on Windows
/// via fallback copy.
#[derive(Debug, Clone)]
pub struct FsSnapshot {
    /// Path to the snapshot directory.
    pub path: PathBuf,
    /// When the snapshot was created.
    pub created_at: Instant,
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
    /// Number of segments before compaction (always 1 for single File).
    pub segments_before: u64,
    /// Number of segments after compaction (always 1 for single File).
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
    /// Which level was compacted (0 = L0 hot, 1 = L1 warm, 2 = L2 cold, 3 = L3 archive).
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
    pub config: Config,
    /// If true, all mutating operations must be rejected.
    pub read_only: bool,
    /// Thread-safe HNSW index behind the cycle-breaking handle (F3X: ADR-042).
    /// Boxed because `Arc` requires `Sized` for arc-swap's `RefCnt`.
    pub hnsw: ArcSwap<Box<dyn IndexPort>>,
    /// Serializes insert/refresh operations to avoid bidirectional
    /// neighbor update races. Searches acquire hnsw.read() freely.
    ///
    /// FIND-651: this is also THE quiesce barrier — `create_snapshot` holds it
    /// across its whole data+backend mirror, so every path that mutates a
    /// mirrored file must run under it. Lock order (outermost → innermost):
    /// `insert_lock` → {`pending_hnsw_batch`, `vector_store[i]` write,
    /// WAL shard locks, backend-internal locks, `cache.volatile`,
    /// scalar/edge index internals}. `label_intern` → `insert_lock`
    /// (`intern_label` persists under `label_intern`; nothing acquires
    /// `label_intern` while holding `insert_lock`). Leaf locks
    /// (`cardinality_stats`, scalar/edge index, `volatile`) are sometimes
    /// taken inside `insert_lock` sections but are never held while acquiring
    /// `insert_lock` — that is the invariant that keeps the order acyclic.
    pub(crate) insert_lock: FairMutex<()>,
    /// Pending HNSW mutations awaiting batch flush under a single
    /// `insert_lock` acquisition (Rayon micro-batching, P1).
    pub(crate) pending_hnsw_batch: parking_lot::Mutex<Vec<PendingHnswOp>>,
    /// Cache layer: volatile hot-node map, BM25 text caches,
    /// cardinality stats and predictive warmer (C2S3b, SRP).
    /// Pure state lives in [`cache::CacheLayer`]; the engine only
    /// orchestrates around it. Call sites use `self.cache.*`.
    pub(crate) cache: cache::CacheLayer,
    /// Monotonic timestamp (ms since epoch) of the last query activity.
    pub last_query_timestamp: AtomicU64,
    /// Transaction bookkeeping: id counter, active set, write buffers.
    /// Pure state lives in [`TxnManager`]; the engine only orchestrates
    /// WAL + store application around it (C2S3, SRP).
    pub(crate) txn: TxnManager,
    /// Flag signalling emergency maintenance (e.g. cache pressure).
    pub emergency_maintenance_trigger: AtomicBool,
    /// Path to the data directory.
    pub data_dir: PathBuf,
    /// Vector store files for persistent node vector data — one per LSM level.
    /// Index 0 = L0 (hot), 1 = L1 (warm), 2 = L2 (cold).
    /// All new writes go to index 0. Reads use unpack_offset() to select the correct file.
    pub vector_store: Vec<RwLock<File>>,
    /// Multi-level LSM segment registry tracking level metadata.
    // Reason: accessed by compaction and LSM tiering routines; kept on engine struct for lifecycle ownership.
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
    pub(crate) _lock_file: Option<StdFile>,
    /// Bidirectional edge label interner: String ↔ u32.
    /// Reduces per-edge label overhead from ~24-32 bytes to 4 bytes.
    pub(crate) label_intern: parking_lot::Mutex<LabelIntern>,
}

// ─── Internal helpers used across sub-modules ──────────────────

impl StorageEngine {
    /// Return the IndexCoordinator facade for HNSW index and pending batch operations.
    pub fn index_coordinator(&self) -> IndexCoordinator<'_> {
        IndexCoordinator::new(&self.hnsw, &self.pending_hnsw_batch)
    }

    /// Return the VectorStoreService facade for multi-level vector store and segment registry.
    pub fn vector_store_service(&self) -> VectorStoreService<'_> {
        VectorStoreService::new(&self.vector_store, &self.segment_registry)
    }

    /// Return the WalCoordinator facade for write-ahead log operations.
    pub fn wal_coordinator(&self) -> WalCoordinator<'_> {
        WalCoordinator::new(self.wal.as_ref())
    }

    /// Replay a single write operation during WAL recovery.
    /// Writes to L0 (always) and packs the segment_id into the offset.
    fn replay_write_node(
        vector_store: &[RwLock<File>],
        hnsw: &dyn IndexPort,
        backend: &dyn StorageBackend,
        node_id: u128,
        node: &UnifiedNode,
    ) -> Result<()> {
        use crate::backend::BackendPartition;
        use crate::storage::ops::NodeMetadata;

        let (_local_off, packed) = VectorStoreService::write_node_l0_slice(vector_store, node)?;
        hnsw.add_node(node_id, node.bitset.clone(), node.vector.clone(), packed)?;
        let key = node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: node.relational.clone(),
            edges: node.edges.clone(),
            created_by_txn: 0, // recovery is pre-MVCC
            deleted_by_txn: None,
        };
        let metadata_val =
            postcard::to_allocvec(&metadata).map_err(crate::error::Error::serialization)?;
        backend.put(BackendPartition::Default, &key, &metadata_val)?;
        Ok(())
    }
}

// ─── Label Interning ───────────────────────────────────────

/// InternalMetadata key for the persisted edge label interner (NOT-01 / FIND-609).
pub const LABEL_INTERN_KEY: &[u8] = b"__vanta_label_intern";

impl StorageEngine {
    /// Intern a label string, returning a stable u32 ID.
    /// Creates a new entry if the label hasn't been seen before.
    /// Persists updated state to `BackendPartition::InternalMetadata` (NOT-01 / FIND-609).
    pub fn intern_label(&self, label: &str) -> u32 {
        let mut intern = self.label_intern.lock();
        let prev_len = intern.len();
        let id = intern.intern(label);
        if intern.len() > prev_len && !self.read_only {
            if let Ok(bytes) = intern.to_bytes() {
                if let Err(e) = self.put_to_partition(
                    crate::backend::BackendPartition::InternalMetadata,
                    LABEL_INTERN_KEY,
                    &bytes,
                ) {
                    tracing::warn!(error = %e, "failed to persist label intern state");
                }
            }
        }
        id
    }

    /// Resolve a label_id back to its string, if known.
    pub fn resolve_label(&self, id: u32) -> Option<String> {
        self.label_intern.lock().resolve(id).map(|s| s.to_string())
    }

    /// Convert a `UnifiedNode` to an SDK `NodeRecord`, resolving edge labels.
    pub fn node_to_record(&self, node: crate::node::UnifiedNode) -> crate::sdk::NodeRecord {
        crate::sdk::serialization::graph_types::unified_to_record(node, &self.label_intern.lock())
    }
}

// ─── VecIndex accessor ─────────────────────────────────────

impl StorageEngine {
    /// Return a handle to the vector index behind the cycle-breaking port.
    ///
    /// The returned [`arc_swap::Guard`] auto-derefs to `dyn IndexPort`, which
    /// exposes the search/maintenance surface. Callers invoke trait methods
    /// without binding to the concrete index type (F3X: ADR-042).
    pub fn vec_index(&self) -> arc_swap::Guard<Arc<Box<dyn IndexPort>>> {
        self.hnsw.load()
    }
}

// ─── Scalar index (MOD-04: TTL purge candidates) ───────────

impl StorageEngine {
    /// Look up node IDs whose integer value for `field` is `<= max`.
    ///
    /// Selects TTL-expired candidates (`expires_at_ms <= now`) from the
    /// maintained scalar index so `purge_expired` avoids a full O(N) scan.
    /// Falls back to an empty set if the index is not present.
    pub(crate) fn scalar_lookup_int_le(&self, field: &str, max: i64) -> Vec<u128> {
        match &self.scalar_index {
            Some(si) => si.lookup_int_le(field, max),
            None => Vec::new(),
        }
    }

    /// Rebuild the scalar index from backend metadata (relational fields).
    ///
    /// Called at open/reopen and by `rebuild_index` — the index is otherwise
    /// only maintained incrementally on writes (insert/delete/vacuum), so a
    /// reopen would start with an empty index and TTL purge would miss every
    /// pre-existing expired record.
    pub(crate) fn rebuild_scalar_index(&self) -> Result<()> {
        use crate::backend::BackendPartition;
        use crate::storage::ops::{deserialize_node_payload, NodeMetadata};

        let Some(si) = &self.scalar_index else {
            return Ok(());
        };
        let entries = self.backend.scan(BackendPartition::Default)?;
        for (key, value) in entries {
            let Ok(key_arr) = <[u8; 16]>::try_from(key.as_slice()) else {
                continue;
            };
            let id = u128::from_le_bytes(key_arr);
            let Ok(metadata) = deserialize_node_payload::<NodeMetadata>(&value, "node metadata")
            else {
                continue;
            };
            for (field, fv) in &metadata.relational {
                si.insert(field, fv, id);
            }
        }
        Ok(())
    }
}

// ─── Filesystem Snapshots ──────────────────────────────────

/// Unix: hard-link (O(1) per file). Windows/WASM wrapper unifies
/// [`std::fs::copy`]'s `u64` return with the walker's `()` contract.
#[cfg(unix)]
fn mirror_file(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::hard_link(src, dst)
}

#[cfg(any(windows, target_arch = "wasm32"))]
fn mirror_file(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::copy(src, dst).map(|_| ())
}

/// Does the live engine keep writing to this `data_dir` file after a snapshot?
///
/// Hard-linking such a file lets every later write mutate the snapshot through
/// the shared inode: the sharded WAL grows with post-snapshot records, so
/// reopening the snapshot replays them (`test_hardlink_snapshot_independence`
/// reopened with the *modified* payload — real isolation bug exposed by the
/// ASan job). Vstore segments are also rewritten in place by `set_tombstone`
/// (and re-sealed when encrypted), so a hard link leaks later deletes into
/// the snapshot. These files are copied instead. Rotated WAL archives
/// (`vanta.shard<N>.wal.<timestamp>` and `vanta.wal.<timestamp>`) are
/// replaced rather than rewritten in place and may be hard-linked.
fn is_mutable_after_snapshot(name: &std::ffi::OsStr) -> bool {
    let name = name.to_string_lossy();
    name == "vanta.wal" // single-shard WAL (wal_shards = 1)
        || name == "vanta.wal.shards" // shard-layout sidecar
        || name == ".wal_ship_marker" // wal-shipping progress marker (rewritten)
        || name == "vector_index.bin" // HNSW image (rewritten by every flush)
        || name == "vector_store.vanta"
        || (name.starts_with("vstore_L") && name.ends_with(".vanta"))
        // ADR-0057 W6: the encryption registry is rewritten on every writable
        // open (a new nonce epoch). A hard link would let that write mutate
        // the snapshot through the shared inode — the snapshot's copy of the
        // registry would then describe epochs the live database never used.
        // The name is the literal of `crypto::registry::REGISTRY_FILE_NAME`
        // (the crypto module is feature-gated; this function is not).
        || name == "encryption.registry"
        // FIND-318: the rollback copy is rewritten IN PLACE by
        // `fs::copy` (O_TRUNC) on every master-key rotation — same
        // shared-inode hazard as the registry itself. (The mirror gate only
        // hard-links rotated WAL archives, so this entry is defensive
        // documentation of the mutable family.)
        || name == "encryption.registry.bak"
        // Active WAL shards `vanta.shard<N>.wal`. Rotated archives carry a
        // `.wal.<timestamp>` suffix instead, so they do NOT match here.
        || (name.starts_with("vanta.shard") && name.ends_with(".wal"))
}

/// Recursively mirror `src` into `dst`, hard-linking (Unix) or copying
/// (Windows/WASM) regular files.
///
/// Files the live engine keeps writing to after the snapshot
/// ([`is_mutable_after_snapshot`]) are ALWAYS copied — a hard link would let
/// post-snapshot appends mutate the snapshot through the shared inode. Only
/// immutable rotated WAL archives are hard-linked on Unix. All other files
/// are copied, including files not yet classified by the engine.
///
/// Skips the `snapshots` subdirectory: it lives INSIDE `data_dir` (created by
/// [`StorageEngine::create_snapshot`] itself), so recursing into it would nest
/// `snapshots/snapshots/...` forever (FIND-25).
fn mirror_data_dir(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if name == "snapshots" {
                continue;
            }
            let sub = dst.join(&name);
            std::fs::create_dir_all(&sub)?;
            mirror_data_dir(&path, &sub)?;
        } else if kind.is_file() && !name.to_string_lossy().ends_with(".tmp") {
            let dst_path = dst.join(&name);
            let name_str = name.to_string_lossy();
            let rotated_wal = (name_str.starts_with("vanta.shard") && name_str.contains(".wal."))
                || name_str.starts_with("vanta.wal.");
            let rotated_wal = rotated_wal
                && name_str.rsplit_once(".wal.").is_some_and(|(_, suffix)| {
                    !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
                });
            if rotated_wal && !is_mutable_after_snapshot(&name) {
                mirror_file(&path, &dst_path)?;
            } else {
                std::fs::copy(&path, &dst_path).map(|_| ())?;
            }
        } else if !kind.is_file() {
            return Err(std::io::Error::other(format!(
                "unsupported snapshot entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SnapshotBackendKind {
    Fjall,
    RocksDb,
}

impl SnapshotBackendKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fjall => "fjall",
            Self::RocksDb => "rocksdb",
        }
    }
}

fn snapshot_backend_kind(kind: BackendKind) -> Result<SnapshotBackendKind> {
    match kind {
        BackendKind::Fjall => Ok(SnapshotBackendKind::Fjall),
        BackendKind::RocksDb => Ok(SnapshotBackendKind::RocksDb),
        BackendKind::InMemory => Err(crate::error::Error::InvalidInput(
            "filesystem snapshots require a persistent backend".into(),
        )),
    }
}

fn mirror_rocksdb_checkpoint(
    checkpoint: &dyn crate::backend::Snapshotable,
    storage_root: &std::path::Path,
    snap_root: &std::path::Path,
) -> Result<()> {
    let checkpoint_dir = snap_root.join(".backend_checkpoint");
    checkpoint.checkpoint(&checkpoint_dir)?;
    let schema = storage_root.join(".vanta.schema");
    match std::fs::symlink_metadata(&schema) {
        Ok(metadata) if metadata.file_type().is_file() => {
            std::fs::copy(&schema, checkpoint_dir.join(".vanta.schema"))?;
        }
        Ok(_) => {
            return Err(crate::error::Error::Io(std::io::Error::other(format!(
                "unsupported backend entry: {}",
                schema.display()
            ))));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    mirror_backend_to(&checkpoint_dir, snap_root, SnapshotBackendKind::RocksDb)?;
    std::fs::remove_dir_all(checkpoint_dir)?;
    Ok(())
}

/// Mirror the KV backend files into the snapshot (FIND-33).
///
/// The backend opens under the storage root (`<storage_path>/`, sibling of
/// `data_dir` — see `init_storage`), so on a fresh layout the backend LSM
/// files live directly under `storage_root`. We recursively mirror them into
/// `<snap_root>/backend/`, preserving relative paths. The
/// `.vanta.lock` file is intentionally NOT mirrored — the lock is process-
/// local and must be acquired afresh by the next opener.
///
/// fjall's ACTIVE journal (`<id>.jnl`, pre-allocated and opened in append
/// mode) is copied, never hard-linked: post-snapshot backend writes append to
/// the same inode, which would mutate the captured backend (same shared-inode
/// class as the `data/` WAL in [`mirror_data_dir`]). Sealed journals (lower
/// ids) are immutable until fjall deletes them — the hard link keeps their
/// bytes alive across the unlink, so they stay hard-linked. Fjall tables and
/// blobs are also immutable and linked; version metadata is copied.
/// Copy versions before their tables and retry if compaction removes a file
/// or changes a keyspace's `current` during the mirror.
fn mirror_backend_to(
    storage_root: &std::path::Path,
    snap_root: &std::path::Path,
    backend_kind: SnapshotBackendKind,
) -> std::io::Result<()> {
    if !storage_root.exists() {
        return Ok(());
    }
    let dst = snap_root.join("backend");
    for _ in 0..3 {
        if dst.exists() {
            std::fs::remove_dir_all(&dst)?;
        }
        std::fs::create_dir_all(&dst)?;
        let attempt = (|| {
            let active_journal = if backend_kind == SnapshotBackendKind::Fjall {
                active_journal_id(storage_root)
            } else {
                None
            };
            mirror_backend_dir(
                storage_root,
                storage_root,
                &dst,
                backend_kind,
                active_journal,
            )?;
            if backend_unstable()
                || (backend_kind == SnapshotBackendKind::Fjall
                    && !backend_currents_match(storage_root, &dst)?)
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "backend changed",
                ));
            }
            Ok(())
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        }
    }
    if dst.exists() {
        std::fs::remove_dir_all(&dst)?;
    }
    Err(std::io::Error::other(
        "backend kept changing during snapshot",
    ))
}

fn is_backend_entry(name: &std::ffi::OsStr, backend_kind: SnapshotBackendKind) -> bool {
    backend_entry_is_dir(name, backend_kind).is_some()
}

fn backend_entry_is_dir(name: &std::ffi::OsStr, backend_kind: SnapshotBackendKind) -> Option<bool> {
    let name = name.to_str()?;
    if name == ".vanta.schema" {
        return Some(false);
    }
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match backend_kind {
        SnapshotBackendKind::Fjall => {
            if name == "keyspaces" {
                Some(true)
            } else if matches!(name, "version" | "lock")
                || name.strip_suffix(".jnl").is_some_and(digits)
            {
                Some(false)
            } else {
                None
            }
        }
        SnapshotBackendKind::RocksDb => {
            if name == "archive" {
                Some(true)
            } else if matches!(name, "CURRENT" | "LOCK" | "IDENTITY" | "LOG")
                || ["LOG.old.", "MANIFEST-", "OPTIONS-"].iter().any(|prefix| {
                    name.strip_prefix(prefix).is_some_and(|s| {
                        digits(s)
                            || (*prefix == "OPTIONS-"
                                && s.strip_suffix(".dbtmp").is_some_and(digits))
                    })
                })
                || [".log", ".sst", ".ldb", ".blob", ".dbtmp"]
                    .iter()
                    .any(|suffix| name.strip_suffix(suffix).is_some_and(digits))
            {
                Some(false)
            } else {
                None
            }
        }
    }
}

fn validate_backend_entry(
    name: &std::ffi::OsStr,
    file_kind: std::fs::FileType,
    backend_kind: SnapshotBackendKind,
) -> std::io::Result<()> {
    let valid = match backend_entry_is_dir(name, backend_kind) {
        Some(true) => file_kind.is_dir(),
        Some(false) => file_kind.is_file(),
        None => false,
    };
    if valid {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "unsupported backend entry: {}",
            name.to_string_lossy()
        )))
    }
}

#[cfg(any(feature = "fs2", feature = "encryption"))]
fn is_pending_restore_tmp(name: &str) -> bool {
    name.starts_with(".restore_pending.") && name.ends_with(".tmp")
}

fn is_restore_staging(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|name| {
        ["data.pre_restore_", "backend.pre_restore_"]
            .iter()
            .any(|prefix| {
                name.strip_prefix(prefix).is_some_and(|suffix| {
                    !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
                })
            })
    })
}

fn restore_requires_fs2() -> crate::error::Error {
    crate::error::Error::Validation {
        field: "snapshot_restore".into(),
        reason: "snapshot restore requires the fs2 feature for exclusive locking".into(),
    }
}

fn mirror_backend_dir(
    root: &std::path::Path,
    src: &std::path::Path,
    dst: &std::path::Path,
    backend_kind: SnapshotBackendKind,
    active_journal: Option<u64>,
) -> std::io::Result<()> {
    let mut entries = std::fs::read_dir(src)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let order = if name == "current" || name.starts_with('v') {
            0
        } else if name == "tables" || name == "blobs" {
            2
        } else {
            1
        };
        (order, name.into_owned())
    });
    for entry in entries {
        let name = entry.file_name();
        let kind = entry.file_type()?;
        if src == root && !is_backend_entry(&name, backend_kind) {
            if !kind.is_file() && !kind.is_dir() {
                return Err(std::io::Error::other(format!(
                    "unsupported root entry: {}",
                    entry.path().display()
                )));
            }
            continue;
        }
        let path = entry.path();
        let target = dst.join(&name);
        if src == root {
            validate_backend_entry(&name, kind, backend_kind)?;
        }
        if kind.is_dir() {
            std::fs::create_dir_all(&target)?;
            mirror_backend_dir(root, &path, &target, backend_kind, active_journal)?;
        } else if kind.is_file() {
            let relative = path.strip_prefix(root).map_err(std::io::Error::other)?;
            let under_keyspaces = relative
                .components()
                .next()
                .is_some_and(|c| c.as_os_str() == "keyspaces");
            let immutable = under_keyspaces
                && path
                    .parent()
                    .and_then(|p| p.file_name())
                    .is_some_and(|n| n == "tables" || n == "blobs");
            let sealed_journal = src == root
                && name
                    .to_string_lossy()
                    .strip_suffix(".jnl")
                    .and_then(|id| id.parse::<u64>().ok())
                    .is_some_and(|id| active_journal.is_some_and(|active| id < active));
            if immutable || sealed_journal {
                mirror_file(&path, &target)?;
            } else {
                std::fs::copy(&path, &target)?;
            }
        } else {
            return Err(std::io::Error::other(format!(
                "unsupported backend entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Highest `*.jnl` id under `storage_root` — fjall's active journal
/// (`journal/recovery.rs` continues the max id; only that one is appended).
/// `None` when no journal exists (or the directory is unreadable).
fn active_journal_id(storage_root: &std::path::Path) -> Option<u64> {
    let mut max_id: Option<u64> = None;
    for entry in std::fs::read_dir(storage_root).ok()?.flatten() {
        let name = entry.file_name();
        if let Some(stem) = name.to_string_lossy().strip_suffix(".jnl") {
            if let Ok(id) = stem.parse::<u64>() {
                max_id = Some(max_id.map_or(id, |m: u64| m.max(id)));
            }
        }
    }
    max_id
}

fn backend_currents_match(
    root: &std::path::Path,
    snapshot: &std::path::Path,
) -> std::io::Result<bool> {
    let keyspaces = snapshot.join("keyspaces");
    if !keyspaces.exists() {
        return Ok(true);
    }
    for entry in std::fs::read_dir(keyspaces)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let current = entry.path().join("current");
        let live = root
            .join("keyspaces")
            .join(entry.file_name())
            .join("current");
        let captured = std::fs::read(&current)?;
        if captured != std::fs::read(live)? || captured.len() < 8 {
            return Ok(false);
        }
        let version = u64::from_le_bytes(captured[..8].try_into().map_err(std::io::Error::other)?);
        let version_file = entry.path().join(format!("v{version}"));
        if !version_file.is_file() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn backend_unstable() -> bool {
    #[cfg(feature = "failpoints")]
    fail::fail_point!("snapshot_backend_unstable", |_| true);
    false
}

const SNAPSHOT_FORMAT: &[u8] = b"vantadb-snapshot v1\n";
const BACKEND_MANIFEST_HEADER: &[u8] = b"vantadb-snapshot-backend v1\n";

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SnapshotInventory {
    backend_kind: SnapshotBackendKind,
    files: Vec<SnapshotInventoryFile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SnapshotInventoryFile {
    path: String,
    len: u64,
}

fn validate_regular_tree(dir: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            validate_regular_tree(&entry.path())?;
        } else if !kind.is_file() {
            return Err(std::io::Error::other(format!(
                "unsupported snapshot entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn validate_live_backend(
    storage_root: &std::path::Path,
    snapshot_backend: &std::path::Path,
    backend_kind: SnapshotBackendKind,
) -> std::io::Result<()> {
    let mut root_names = std::collections::HashSet::new();
    for entry in std::fs::read_dir(storage_root)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        if !kind.is_file() && !kind.is_dir() {
            return Err(std::io::Error::other(format!(
                "unsupported root entry: {}",
                entry.path().display()
            )));
        }
        if is_backend_entry(&name, backend_kind) {
            validate_backend_entry(&name, kind, backend_kind)?;
            if kind.is_dir() {
                validate_regular_tree(&entry.path())?;
            }
        }
        root_names.insert(name);
    }
    for entry in std::fs::read_dir(snapshot_backend)? {
        let name = entry?.file_name();
        if !root_names.contains(&name) {
            let target = storage_root.join(&name);
            match std::fs::symlink_metadata(&target) {
                Ok(_) => {
                    return Err(std::io::Error::other(format!(
                        "snapshot backend target aliases another root entry: {}",
                        target.display()
                    )));
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
    }
    Ok(())
}

fn inventory_snapshot_dir(
    root: &std::path::Path,
    dir: &std::path::Path,
    backend_kind: SnapshotBackendKind,
    files: &mut Vec<SnapshotInventoryFile>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if dir == root.join("backend") {
            validate_backend_entry(&entry.file_name(), kind, backend_kind)?;
        }
        if kind.is_dir() {
            inventory_snapshot_dir(root, &path, backend_kind, files)?;
        } else if kind.is_file() {
            let relative = path.strip_prefix(root).map_err(std::io::Error::other)?;
            let mut parts = Vec::new();
            for component in relative.components() {
                let std::path::Component::Normal(name) = component else {
                    return Err(std::io::Error::other("invalid snapshot file path"));
                };
                let name = name
                    .to_str()
                    .ok_or_else(|| std::io::Error::other("non-UTF-8 snapshot file path"))?;
                if name.contains('\\')
                    || (cfg!(windows) && (name.contains(':') || name.ends_with(['.', ' '])))
                {
                    return Err(std::io::Error::other("invalid snapshot file path"));
                }
                parts.push(name);
            }
            files.push(SnapshotInventoryFile {
                path: parts.join("/"),
                len: entry.metadata()?.len(),
            });
        } else {
            return Err(std::io::Error::other(format!(
                "unsupported snapshot entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn snapshot_inventory(
    snap_root: &std::path::Path,
    backend_kind: SnapshotBackendKind,
) -> std::io::Result<SnapshotInventory> {
    let mut files = Vec::new();
    for name in ["data", "backend"] {
        let path = snap_root.join(name);
        let kind = std::fs::symlink_metadata(&path)?.file_type();
        if !kind.is_dir() {
            return Err(std::io::Error::other(format!(
                "snapshot {name} is not a directory"
            )));
        }
        inventory_snapshot_dir(snap_root, &path, backend_kind, &mut files)?;
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(SnapshotInventory {
        backend_kind,
        files,
    })
}

fn write_snapshot_format(snap_root: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let path = snap_root.join("snapshot.format");
    let mut marker = StdFile::create(&path)?;
    marker.write_all(SNAPSHOT_FORMAT)?;
    marker.sync_all()?;
    crate::utils::fs::sync_parent_dir(&path)
}

fn write_backend_manifest(
    snap_root: &std::path::Path,
    backend_kind: SnapshotBackendKind,
) -> std::io::Result<()> {
    use std::io::Write;
    let inventory = snapshot_inventory(snap_root, backend_kind)?;
    let path = snap_root.join("backend.manifest");
    let result = (|| {
        let mut marker = StdFile::create(&path)?;
        marker.write_all(BACKEND_MANIFEST_HEADER)?;
        serde_json::to_writer(&mut marker, &inventory).map_err(std::io::Error::other)?;
        marker.write_all(b"\n")?;
        marker.sync_all()?;
        crate::utils::fs::sync_parent_dir(&path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

fn validate_snapshot(snap_root: &std::path::Path) -> Result<Option<SnapshotBackendKind>> {
    let fail = |reason: String| crate::error::Error::Validation {
        field: "snapshot".into(),
        reason: format!("{}: {reason}", snap_root.display()),
    };
    let format_path = snap_root.join("snapshot.format");
    let manifest_path = snap_root.join("backend.manifest");
    for path in [&format_path, &manifest_path] {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(fail(format!(
                    "snapshot marker is not a regular file: {}",
                    path.display()
                )));
            }
            Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
                return Err(fail(err.to_string()));
            }
            _ => {}
        }
    }
    let format = std::fs::read(&format_path);
    let manifest = std::fs::read(&manifest_path);
    match (format, manifest) {
        (Err(a), Err(b))
            if a.kind() == std::io::ErrorKind::NotFound
                && b.kind() == std::io::ErrorKind::NotFound =>
        {
            let kind = std::fs::symlink_metadata(snap_root.join("data"))
                .map_err(|e| fail(e.to_string()))?
                .file_type();
            if !kind.is_dir() {
                return Err(fail("legacy snapshot data is not a directory".into()));
            }
            validate_regular_tree(&snap_root.join("data")).map_err(|e| fail(e.to_string()))?;
            tracing::warn!(path = %snap_root.display(), "legacy snapshot has no backend capture; restoring data only");
            Ok(None)
        }
        (Ok(format), Ok(manifest)) => {
            if format != SNAPSHOT_FORMAT {
                return Err(fail("unknown snapshot format".into()));
            }
            let payload = manifest
                .strip_prefix(BACKEND_MANIFEST_HEADER)
                .ok_or_else(|| fail("unknown backend manifest format".into()))?;
            let expected: SnapshotInventory = serde_json::from_slice(payload)
                .map_err(|e| fail(format!("invalid backend inventory: {e}")))?;
            for file in &expected.files {
                let path = &file.path;
                if path.contains('\\')
                    || !(path.starts_with("data/") || path.starts_with("backend/"))
                {
                    return Err(fail(format!("invalid inventory path: {path:?}")));
                }
                let mut normalized = Vec::new();
                for component in std::path::Path::new(path).components() {
                    let std::path::Component::Normal(name) = component else {
                        return Err(fail(format!("invalid inventory path: {path:?}")));
                    };
                    let name = name
                        .to_str()
                        .ok_or_else(|| fail("invalid inventory path".into()))?;
                    if cfg!(windows) && (name.contains(':') || name.ends_with(['.', ' '])) {
                        return Err(fail(format!("invalid inventory path: {path:?}")));
                    }
                    normalized.push(name);
                }
                if normalized.join("/") != *path {
                    return Err(fail(format!("invalid inventory path: {path:?}")));
                }
            }
            let actual = snapshot_inventory(snap_root, expected.backend_kind)
                .map_err(|e| fail(e.to_string()))?;
            if actual != expected {
                return Err(fail(
                    "snapshot inventory differs from captured files".into(),
                ));
            }
            Ok(Some(expected.backend_kind))
        }
        _ => Err(fail(
            "snapshot format or backend manifest missing or unreadable".into(),
        )),
    }
}

fn copy_backend_dir(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if kind.is_dir() {
            copy_backend_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), target)?;
        } else {
            return Err(std::io::Error::other(format!(
                "unsupported backend entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn sync_restored_tree(path: &std::path::Path, skip_snapshots: bool) -> std::io::Result<()> {
    let kind = std::fs::symlink_metadata(path)?.file_type();
    if kind.is_file() {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        file.sync_all()?;
    } else if kind.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if skip_snapshots && entry.file_name() == "snapshots" {
                continue;
            }
            sync_restored_tree(&entry.path(), false)?;
        }
        crate::utils::fs::sync_parent_dir(&path.join(".sync"))?;
    } else {
        return Err(std::io::Error::other(format!(
            "unsupported restored entry: {}",
            path.display()
        )));
    }
    Ok(())
}

impl StorageEngine {
    /// Acquire `insert_lock` with the configured timeout (ERR-010 pattern).
    ///
    /// Native: `try_lock_for` so a contended lock fails with `Timeout` after
    /// `insert_lock_timeout_ms`. wasm32-unknown-unknown: `try_lock_for` panics
    /// (parking_lot's `to_deadline` internally computes a
    /// `std::time::Instant::now()`, unsupported on that target) — and on this
    /// single-threaded platform a timed wait could never make progress anyway:
    /// only this thread could release the lock, so a held lock means
    /// re-entrance. A plain `try_lock` is therefore both panic-free and
    /// strictly more correct there, mapping failure to the same `Timeout`.
    #[inline]
    pub(crate) fn acquire_insert_lock(&self, operation: &str) -> Result<FairMutexGuard<'_, ()>> {
        #[cfg(not(target_arch = "wasm32"))]
        let guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ));
        #[cfg(target_arch = "wasm32")]
        let guard = self.insert_lock.try_lock();
        guard.ok_or_else(|| crate::error::Error::Timeout {
            operation: operation.into(),
            duration_ms: self.config.insert_lock_timeout_ms,
        })
    }

    /// Acquire `insert_lock` for the `create_snapshot` mirror (FIND-651).
    ///
    /// Asymmetry vs [`Self::acquire_insert_lock`]: a WRITER that cannot get
    /// the lock within `insert_lock_timeout_ms` fails with the typed,
    /// retriable [`crate::error::Error::Timeout`]; the SNAPSHOT instead
    /// out-waits transient contention — a writer must never be the reason a
    /// backup fails. It retries through several timeout windows (bounded —
    /// a genuinely stuck critical section still surfaces `Timeout` rather
    /// than hanging the caller forever).
    #[cfg(not(target_arch = "wasm32"))]
    fn acquire_snapshot_quiesce(&self) -> Result<FairMutexGuard<'_, ()>> {
        const ATTEMPTS: u32 = 5;
        let mut last_err: Option<crate::error::Error> = None;
        for attempt in 1..=ATTEMPTS {
            match self
                .acquire_insert_lock("acquire insert_lock in create_snapshot (FIND-651 quiesce)")
            {
                Ok(guard) => return Ok(guard),
                Err(e) => {
                    tracing::debug!(
                        attempt,
                        max_attempts = ATTEMPTS,
                        error = %e,
                        "snapshot quiesce: retrying after contention"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| {
            crate::error::Error::generic_error("snapshot quiesce: no attempt ran")
        }))
    }

    /// wasm32: the engine is single-threaded, so `insert_lock` is either free
    /// or held by this same thread — one `try_lock` decides both cases (the
    /// retry loop above could never observe a release).
    #[cfg(target_arch = "wasm32")]
    fn acquire_snapshot_quiesce(&self) -> Result<FairMutexGuard<'_, ()>> {
        self.acquire_insert_lock("acquire insert_lock in create_snapshot (FIND-651 quiesce)")
    }

    /// Whether a transaction is currently active on this engine.
    ///
    /// WIRE-06: the group-commit ingestion path must fall back to the
    /// per-record `insert()` inside a transaction — `insert()` buffers writes
    /// into the transaction write set (ERR-013) while `batch_insert_with_opts`
    /// applies directly, which would break transaction isolation.
    #[cfg(feature = "async-ingestion")]
    #[inline]
    pub(crate) fn has_active_transaction(&self) -> bool {
        self.txn.has_active()
    }

    /// Create a filesystem snapshot of the live data directory and backend.
    ///
    /// # Consistency (FIND-25, FIND-651)
    ///
    /// Before imaging, the engine is flushed via [`Self::flush`]
    /// (insert_lock, HNSW drain, backend flush, vector-index save, ERR-010
    /// pattern). The capture then holds `insert_lock` across the whole
    /// data+backend mirror: every path that mutates a mirrored file (insert,
    /// delete, batch staging, txn WAL markers, LSM promotion/compaction,
    /// `compact_wal` rotation and privacy redaction, partition writes, MVCC
    /// GC, manual backend compaction) runs under that same lock, so the
    /// mirror observes a file set frozen at a commit boundary. A writer that
    /// cannot enter the critical section within `insert_lock_timeout_ms`
    /// fails with the typed, retriable [`crate::error::Error::Timeout`] —
    /// the snapshot itself out-waits transient writer traffic and only fails
    /// on I/O errors or a persistently stuck critical section.
    ///
    /// # Performance trade-off
    ///
    /// The quiesce adds one full flush per snapshot: this snapshot is no
    /// longer O(1)-instant. Correctness beats speed here — a torn backup is
    /// worse than a slower one. Callers needing high-frequency snapshots
    /// should rate-limit instead of bypassing the flush.
    ///
    /// On Unix, immutable files (rotated WAL archives, sealed Fjall journals,
    /// and Fjall tables/blobs)
    /// are hard-linked (O(1) per file — kernel
    /// directory entries pointing at the same inode); files the engine keeps
    /// writing to after the snapshot (vstore segments, active WAL shards, HNSW image —
    /// [`is_mutable_after_snapshot`]) are copied instead, so no post-snapshot
    /// write can reach the snapshot through a shared inode. On Windows/WASM,
    /// everything falls back to [`std::fs::copy`] (O(n) per file).
    /// Subdirectories under `data_dir`
    /// (if any appear in future layouts) are mirrored recursively; the
    /// engine-owned `snapshots/` directory is excluded.
    ///
    /// Note: before FIND-33 this captured `data_dir` only — the KV backend
    /// directory lives beside `data_dir` under the storage root, and a
    /// snapshot taken after `compact_wal()` (which archives WAL segments) lost
    /// any state that lived only in the backend KV (namespace_index,
    /// internal_metadata, checkpoint_seq). The snapshot now also mirrors
    /// the backend files into `<snap_dir>/backend/` (FIND-33). RocksDB is
    /// captured through its native checkpoint, then copied into the snapshot.
    /// A completed capture has a `snapshot.format` and an inventoried
    /// `backend.manifest`.
    /// Existing snapshot names are rejected. The `.vanta.lock` file
    /// is intentionally excluded — the lock is process-local and must be
    /// acquired by the next opener.
    ///
    /// Opening the snapshot path directly gives a data-only view and writes
    /// Fjall files into the snapshot root. Use [`Self::snapshot_restore`] to
    /// restore the captured backend before reopening the database.
    #[cfg(unix)]
    pub fn create_snapshot(&self, name: &str) -> crate::error::Result<FsSnapshot> {
        self.create_snapshot_inner(name)
    }

    /// Create a filesystem snapshot (Windows/WASM fallback using copy).
    ///
    /// Same flush-then-image semantics as the Unix variant — see its
    /// documentation for the consistency and performance trade-offs. In
    /// particular (FIND-33), the snapshot also captures the backend KV files
    /// under `<snap_dir>/backend/` so it survives a subsequent `compact_wal()`.
    /// RocksDB is captured through its native checkpoint before those files
    /// are copied.
    ///
    /// Opening the snapshot path directly gives a data-only view and writes
    /// Fjall files into the snapshot root. Restore through
    /// [`Self::snapshot_restore`] for the captured backend. The mirror runs
    /// under the `insert_lock` quiesce barrier described on the Unix variant
    /// (FIND-651) — concurrent writers block up to `insert_lock_timeout_ms`
    /// and then fail with retriable [`crate::error::Error::Timeout`].
    #[cfg(any(windows, target_arch = "wasm32"))]
    pub fn create_snapshot(&self, name: &str) -> crate::error::Result<FsSnapshot> {
        self.create_snapshot_inner(name)
    }

    fn capture_rocksdb_backend(
        &self,
        storage_root: &std::path::Path,
        snap_root: &std::path::Path,
    ) -> Result<()> {
        let checkpoint = self.backend.as_snapshotable().ok_or_else(|| {
            crate::error::Error::backend_error(
                "RocksDB snapshot requires native checkpoint support",
            )
        })?;
        mirror_rocksdb_checkpoint(checkpoint, storage_root, snap_root)
    }

    fn create_snapshot_inner(&self, name: &str) -> crate::error::Result<FsSnapshot> {
        Self::validate_snapshot_name(name)?;
        let backend_kind = snapshot_backend_kind(self.config.backend_kind)?;

        let snapshots = self.data_dir.join("snapshots");
        std::fs::create_dir_all(&snapshots)?;
        let snap_dir = snapshots.join(name);
        std::fs::create_dir(&snap_dir)?;
        let snap_data = snap_dir.join("data");
        let capture = (|| -> Result<()> {
            write_snapshot_format(&snap_dir)?;
            std::fs::create_dir(&snap_data)?;
            #[cfg(feature = "failpoints")]
            fail::fail_point!("snapshot_create_fail", |_| {
                Err(crate::error::Error::Io(std::io::Error::other(
                    "Simulated snapshot create I/O failure",
                )))
            });
            // FIND-651: every mutation path that touches a mirrored file
            // holds `insert_lock` (insert/delete/batch/txn WAL markers,
            // batch staging, LSM compaction, WAL rotation/redaction,
            // partition writes, MVCC GC, manual compaction). ONE quiesce
            // guard covers the checkpoint flush AND the data+backend mirror:
            // the file set is frozen at the drained commit boundary — no
            // mid-write copy, no cross-file skew, no rotation mid-capture,
            // and checkpoint_seq is simultaneous with the mirrored bytes.
            let _guard = self.acquire_snapshot_quiesce()?;
            if !self.read_only {
                self.flush_locked()?;
            }
            mirror_data_dir(&self.data_dir, &snap_data)?;
            let storage_root = self.data_dir.parent().unwrap_or(&self.data_dir);
            match backend_kind {
                SnapshotBackendKind::Fjall => {
                    mirror_backend_to(storage_root, &snap_dir, backend_kind)?;
                }
                SnapshotBackendKind::RocksDb => {
                    self.capture_rocksdb_backend(storage_root, &snap_dir)?;
                }
            }
            write_backend_manifest(&snap_dir, backend_kind)?;
            Ok(())
        })();
        if let Err(error) = capture {
            if let Err(cleanup) = std::fs::remove_dir_all(&snap_dir) {
                tracing::warn!(path = %snap_dir.display(), error = %cleanup, "could not remove incomplete snapshot");
            }
            return Err(error);
        }
        Ok(FsSnapshot {
            path: snap_dir,
            created_at: Instant::now(),
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

    /// Validate a snapshot name as a plain filesystem identifier.
    ///
    /// Trust boundary: the name becomes a path segment under
    /// `<data_dir>/snapshots/`, so path separators, `.`/`..`, or control
    /// characters would allow traversal out of the snapshots directory.
    /// Mirrors the MCP-34a guard in `vantadb-mcp` (defense in depth — the MCP
    /// layer also validates before dispatching).
    fn validate_snapshot_name(name: &str) -> crate::error::Result<()> {
        use std::path::Component;
        let mut components = std::path::Path::new(name).components();
        let single_normal =
            matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
            || name.chars().any(char::is_control)
            || !single_normal
            || (cfg!(windows) && (name.contains(':') || name.ends_with(['.', ' '])))
        {
            return Err(crate::error::Error::InvalidInput(format!(
                "snapshot name must be a plain identifier (no path separators, '.', '..', or control characters): {name:?}"
            )));
        }
        Ok(())
    }

    /// Restore the database directory from a physical snapshot (MCP-34b).
    ///
    /// # Exclusivity (embedded API contract)
    ///
    /// This is a static associated function taking the storage *root* path —
    /// deliberately NOT `&self`. The directory swap requires that no engine
    /// holds the database open: on Windows open handles prevent a swap; on
    /// Unix an open handle could keep writing into the renamed-aside directory,
    /// silently forking state. The fs2 lock rejects open handles on both.
    /// Callers must close/drop every handle
    /// first (see [`crate::sdk::Embedded::restore_from`] for the full
    /// close → restore → reopen flow).
    ///
    /// # Safety / rollback
    ///
    /// The live `<root>/data` directory is renamed aside to a staging sibling
    /// (`<root>/data.pre_restore_<nanos>`, atomic same-volume rename) instead
    /// of deleted. All snapshots are moved back into the fresh `data_dir`
    /// before the copy-back so they survive the restore. If any step fails
    /// after the rename, the partial data_dir is removed and the staged
    /// original is renamed back; rollback failures retain staging and report
    /// recovery paths. Restored files and directories are synced before
    /// staging is removed. A process interruption can leave staging behind:
    /// subsequent opens fail closed until it is recovered manually. For
    /// snapshots with both format and manifest markers, backend entries are
    /// staged and restored as well; legacy snapshots restore only `data/`.
    ///
    /// Note: unlike RES-02 §2a step 3 (`<snap>/pre_restore_<ts>`), the staging
    /// sibling lives beside `data_dir` — `snapshots/` is INSIDE `data_dir`, so
    /// renaming data_dir under its own snapshot would nest them within
    /// themselves.
    ///
    /// Returns the restored `<root>/data` path; reopen with
    /// [`crate::sdk::Embedded::open_with_config`] — HNSW/text indexes are
    /// rebuilt from storage on open (proven by tests/index_reconstruction.rs).
    /// This static API has no backend configuration: reopen using the same
    /// backend as the snapshot. Complete restores log the captured kind as a
    /// warning. This API does not migrate between backends.
    pub fn snapshot_restore(
        storage_root: &std::path::Path,
        name: &str,
    ) -> crate::error::Result<std::path::PathBuf> {
        Self::snapshot_restore_with_backend(storage_root, name).map(|(path, _)| path)
    }

    /// Restore a snapshot and report whether it included backend state.
    ///
    /// Reopen using the snapshot's backend; this API does not migrate backends.
    pub fn snapshot_restore_with_backend(
        storage_root: &std::path::Path,
        name: &str,
    ) -> crate::error::Result<(std::path::PathBuf, bool)> {
        Self::snapshot_restore_inner(storage_root, name, None)
    }

    pub(crate) fn snapshot_restore_for_backend(
        storage_root: &std::path::Path,
        name: &str,
        expected_backend: BackendKind,
    ) -> crate::error::Result<(std::path::PathBuf, bool)> {
        Self::snapshot_restore_inner(storage_root, name, Some(expected_backend))
    }

    fn snapshot_restore_inner(
        storage_root: &std::path::Path,
        name: &str,
        expected_backend: Option<BackendKind>,
    ) -> crate::error::Result<(std::path::PathBuf, bool)> {
        Self::validate_snapshot_name(name)?;
        if !cfg!(feature = "fs2") {
            return Err(restore_requires_fs2());
        }
        let _ = Self::check_snapshot_exists(storage_root, name)?;
        let config = Config {
            file_lock_timeout_ms: 5,
            ..Config::default()
        };
        let _lock = Self::acquire_db_lock(storage_root, &config)?;
        Self::check_restore_staging(storage_root)?;
        Self::restore_locked(storage_root, name, expected_backend)
    }

    fn check_snapshot_exists(
        storage_root: &std::path::Path,
        name: &str,
    ) -> Result<Option<SnapshotBackendKind>> {
        let snap = storage_root.join("data/snapshots").join(name);
        match std::fs::symlink_metadata(&snap) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(crate::error::Error::NotFound {
                    kind: "snapshot".to_string(),
                    id: name.to_string(),
                })
            }
            Err(err) => Err(err.into()),
            Ok(metadata) if !metadata.file_type().is_dir() => {
                Err(crate::error::Error::Validation {
                    field: "snapshot".into(),
                    reason: format!("snapshot is not a directory: {}", snap.display()),
                })
            }
            Ok(_) => validate_snapshot(&snap),
        }
    }

    /// Schedule a snapshot restore for the next writable database open.
    ///
    /// The marker is durable and a later schedule replaces the previous one.
    /// Writes made before the next open are discarded by the restore.
    pub fn schedule_snapshot_restore(
        storage_root: &std::path::Path,
        name: &str,
    ) -> Result<PathBuf> {
        Self::validate_snapshot_name(name)?;
        let _lock = Self::acquire_pending_restore_lock(storage_root)?;
        let _ = Self::check_snapshot_exists(storage_root, name)?;
        let marker = storage_root.join(".restore_pending");
        match std::fs::symlink_metadata(&marker) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(crate::error::Error::Validation {
                    field: "pending_snapshot_restore".into(),
                    reason: format!(
                        "pending restore marker is not a regular file: {}",
                        marker.display()
                    ),
                });
            }
            Ok(_) => {
                if let Ok(previous) = std::fs::read_to_string(&marker) {
                    let replaced = previous.lines().nth(1).unwrap_or("<malformed>");
                    tracing::warn!(snapshot = replaced, "replacing pending snapshot restore");
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        let contents = format!("vantadb-restore-pending v1\n{name}\n");
        crate::utils::fs::write_atomic_unique(&marker, contents.as_bytes())?;
        Ok(marker)
    }

    fn check_restore_staging(storage_root: &std::path::Path) -> Result<()> {
        let mut staging = Vec::new();
        for entry in std::fs::read_dir(storage_root)? {
            let entry = entry?;
            if is_restore_staging(&entry.file_name()) {
                staging.push(entry.path());
            }
        }
        if staging.is_empty() {
            return Ok(());
        }
        staging.sort();
        let paths = staging
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(crate::error::Error::Validation {
            field: "snapshot_restore".into(),
            reason: format!("unfinished snapshot restore; manual recovery required for: {paths}"),
        })
    }

    fn acquire_pending_restore_lock(storage_root: &std::path::Path) -> Result<StdFile> {
        #[cfg(not(feature = "fs2"))]
        {
            let _ = storage_root;
            Err(restore_requires_fs2())
        }
        #[cfg(feature = "fs2")]
        {
            let path = storage_root.join(".restore_pending.lock");
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)?;
            let start = Instant::now();
            let timeout = std::time::Duration::from_secs(5);
            let mut delay = std::time::Duration::from_millis(5);
            loop {
                if fs2::FileExt::try_lock_exclusive(&file).is_ok() {
                    return Ok(file);
                }
                #[cfg(feature = "failpoints")]
                fail::fail_point!("snapshot_pending_lock_contended");
                if start.elapsed() >= timeout {
                    return Err(crate::error::Error::DatabaseBusy(format!(
                        "pending snapshot restore lock is busy: {}",
                        storage_root.display()
                    )));
                }
                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(delay.min(timeout.saturating_sub(start.elapsed())));
                delay = (delay * 2).min(std::time::Duration::from_millis(100));
            }
        }
    }

    #[cfg(feature = "fs2")]
    fn cleanup_pending_restore_temps(storage_root: &std::path::Path) {
        let entries = match std::fs::read_dir(storage_root) {
            Ok(entries) => entries,
            Err(err) => {
                tracing::warn!(path = %storage_root.display(), error = %err, "could not inspect pending restore temporary files");
                return;
            }
        };
        for entry in entries {
            match entry {
                Ok(entry) if is_pending_restore_tmp(&entry.file_name().to_string_lossy()) => {
                    if let Err(err) = std::fs::remove_file(entry.path()) {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            tracing::warn!(path = %entry.path().display(), error = %err, "could not remove pending restore temporary file");
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "could not inspect pending restore temporary file")
                }
                _ => {}
            }
        }
    }

    fn apply_pending_restore(
        storage_root: &std::path::Path,
        read_only: bool,
        expected_backend: BackendKind,
    ) -> Result<()> {
        let marker = storage_root.join(".restore_pending");
        let failure = |reason: String| crate::error::Error::Validation {
            field: "pending_snapshot_restore".into(),
            reason: format!(
                "pending snapshot restore failed: {reason}; remove {} to open the current state",
                marker.display()
            ),
        };
        let metadata = match std::fs::symlink_metadata(&marker) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) if read_only => {
                tracing::warn!(path = %marker.display(), error = %err, "could not inspect pending restore on read-only open");
                return Ok(());
            }
            Err(err) => return Err(failure(err.to_string())),
        };
        if read_only {
            tracing::warn!(path = %marker.display(), "ignoring pending restore on read-only open");
            return Ok(());
        }
        if !metadata.file_type().is_file() {
            return Err(failure("marker is not a regular file".into()));
        }
        let contents = std::fs::read_to_string(&marker).map_err(|e| failure(e.to_string()))?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("snapshot_pending_after_read");
        let name = contents
            .strip_prefix("vantadb-restore-pending v1\n")
            .and_then(|s| s.strip_suffix('\n'))
            .filter(|s| !s.contains('\n'))
            .ok_or_else(|| failure("malformed marker".into()))?;
        Self::validate_snapshot_name(name).map_err(|e| failure(e.to_string()))?;
        let _ =
            Self::check_snapshot_exists(storage_root, name).map_err(|e| failure(e.to_string()))?;
        if let Err(error) = Self::restore_locked(storage_root, name, Some(expected_backend)) {
            return match Self::check_restore_staging(storage_root) {
                Ok(()) => Err(failure(error.to_string())),
                Err(staging_error) => Err(crate::error::Error::Validation {
                    field: "pending_snapshot_restore".into(),
                    reason: format!(
                        "pending snapshot restore failed: {error}; {staging_error}; manual recovery may be required; marker retained at {}",
                        marker.display()
                    ),
                }),
            };
        }
        std::fs::remove_file(&marker).map_err(|e| failure(e.to_string()))?;
        crate::utils::fs::sync_parent_dir(&marker).map_err(|e| failure(e.to_string()))?;
        Ok(())
    }

    fn restore_locked(
        storage_root: &std::path::Path,
        name: &str,
        expected_backend: Option<BackendKind>,
    ) -> crate::error::Result<(std::path::PathBuf, bool)> {
        let data_dir = storage_root.join("data");
        let snap_dir = data_dir.join("snapshots").join(name);
        let snap_data = snap_dir.join("data");
        let backend_kind = Self::check_snapshot_exists(storage_root, name)?;
        if let Some(expected_backend) = expected_backend {
            let expected = snapshot_backend_kind(expected_backend)?;
            if let Some(actual) = backend_kind {
                if actual != expected {
                    return Err(crate::error::Error::Validation {
                        field: "backend_kind".into(),
                        reason: format!(
                            "snapshot backend '{}' does not match requested backend '{}'",
                            actual.as_str(),
                            expected.as_str()
                        ),
                    });
                }
            }
        } else if let Some(actual) = backend_kind {
            tracing::warn!(
                snapshot_backend = actual.as_str(),
                "static snapshot restore has no backend configuration; reopen with the same backend"
            );
        }
        let backend_restored = backend_kind.is_some();
        if let Some(backend_kind) = backend_kind {
            validate_live_backend(storage_root, &snap_dir.join("backend"), backend_kind)?;
        }

        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("snapshot_restore_fail", |_| {
                Err(crate::error::Error::Io(std::io::Error::other(
                    "Simulated snapshot restore I/O failure",
                )))
            });
        }

        // Stage the live directory aside (atomic same-volume rename). A
        // nanosecond stamp avoids collisions between consecutive restores.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| crate::error::Error::Io(std::io::Error::other(e)))?
            .as_nanos();
        let staging = storage_root.join(format!("data.pre_restore_{nanos}"));
        let backend_staging = storage_root.join(format!("backend.pre_restore_{nanos}"));
        std::fs::rename(&data_dir, &staging)?;

        let mut staged_backend = Vec::new();
        let mut restored_backend = Vec::new();
        let mut snapshots_moved = false;
        let swapped = (|| -> std::io::Result<()> {
            crate::utils::fs::sync_parent_dir(&staging)?;
            if let Some(backend_kind) = backend_kind {
                std::fs::create_dir(&backend_staging)?;
                crate::utils::fs::sync_parent_dir(&backend_staging)?;
                for entry in std::fs::read_dir(storage_root)? {
                    let entry = entry?;
                    let name = entry.file_name();
                    if is_backend_entry(&name, backend_kind) {
                        validate_backend_entry(&name, entry.file_type()?, backend_kind)?;
                        std::fs::rename(entry.path(), backend_staging.join(&name))?;
                        staged_backend.push(name);
                    }
                }
                crate::utils::fs::sync_parent_dir(&backend_staging.join(".sync"))?;
                crate::utils::fs::sync_parent_dir(&backend_staging)?;
            }
            std::fs::create_dir_all(&data_dir)?;
            // Keep every snapshot alive across the swap: move them from the
            // staged tree back into the fresh data_dir before copying the
            // snapshot contents over it.
            let staged_snaps = staging.join("snapshots");
            if staged_snaps.exists() {
                std::fs::rename(staged_snaps, data_dir.join("snapshots"))?;
                snapshots_moved = true;
            }
            mirror_data_dir(&snap_data, &data_dir)?;
            if let Some(backend_kind) = backend_kind {
                for entry in std::fs::read_dir(snap_dir.join("backend"))? {
                    let entry = entry?;
                    let name = entry.file_name();
                    if !is_backend_entry(&name, backend_kind) {
                        return Err(std::io::Error::other(
                            "snapshot contains a reserved backend entry",
                        ));
                    }
                    let kind = entry.file_type()?;
                    validate_backend_entry(&name, kind, backend_kind)?;
                    let target = storage_root.join(&name);
                    restored_backend.push(name);
                    if kind.is_dir() {
                        copy_backend_dir(&entry.path(), &target)?;
                    } else {
                        std::fs::copy(entry.path(), target)?;
                    }
                    #[cfg(feature = "failpoints")]
                    fail::fail_point!("snapshot_restore_backend_fail", |_| {
                        Err(std::io::Error::other("Simulated backend restore failure"))
                    });
                }
            }
            #[cfg(feature = "failpoints")]
            fail::fail_point!("snapshot_restore_sync_fail", |_| {
                Err(std::io::Error::other("simulated restore sync failure"))
            });
            sync_restored_tree(&data_dir, true)?;
            for name in &restored_backend {
                sync_restored_tree(&storage_root.join(name), false)?;
            }
            crate::utils::fs::sync_parent_dir(&staging.join(".sync"))?;
            if backend_restored {
                crate::utils::fs::sync_parent_dir(&backend_staging.join(".sync"))?;
            }
            crate::utils::fs::sync_parent_dir(&data_dir)?;
            Ok(())
        })();

        match swapped {
            Ok(()) => {
                std::fs::remove_dir_all(&staging).map_err(|e| {
                    crate::error::Error::Io(std::io::Error::other(format!(
                        "restore applied but staging cleanup failed at {}; manual recovery required: {e}",
                        staging.display()
                    )))
                })?;
                if backend_restored {
                    std::fs::remove_dir_all(&backend_staging).map_err(|e| {
                        crate::error::Error::Io(std::io::Error::other(format!(
                            "restore applied but staging cleanup failed at {}; manual recovery required: {e}",
                            backend_staging.display()
                        )))
                    })?;
                }
                crate::utils::fs::sync_parent_dir(&data_dir).map_err(|e| {
                    crate::error::Error::Io(std::io::Error::other(format!(
                        "restore applied but staging cleanup sync failed at {}; manual review required: {e}",
                        storage_root.display()
                    )))
                })?;
                Ok((data_dir, backend_restored))
            }
            Err(e) => {
                let mut rollback_errors = Vec::new();
                for name in restored_backend {
                    let path = storage_root.join(name);
                    let removal = match std::fs::symlink_metadata(&path) {
                        Ok(metadata) if metadata.file_type().is_dir() => {
                            std::fs::remove_dir_all(&path)
                        }
                        Ok(_) => std::fs::remove_file(&path),
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                        Err(err) => Err(err),
                    };
                    if let Err(err) = removal {
                        tracing::error!(path = %path.display(), error = %err, "restore rollback failed");
                        rollback_errors.push(format!("{}: {err}", path.display()));
                    }
                }
                for name in staged_backend {
                    let source = backend_staging.join(&name);
                    let target = storage_root.join(&name);
                    #[cfg(feature = "failpoints")]
                    let restored = (|| {
                        fail::fail_point!("snapshot_restore_rollback_fail", |_| {
                            Err(std::io::Error::other("simulated rollback failure"))
                        });
                        std::fs::rename(&source, &target)
                    })();
                    #[cfg(not(feature = "failpoints"))]
                    let restored = std::fs::rename(&source, &target);
                    if let Err(err) = restored {
                        tracing::error!(path = %source.display(), error = %err, "restore rollback failed");
                        rollback_errors.push(format!(
                            "{} -> {}: {err}",
                            source.display(),
                            target.display()
                        ));
                    }
                }
                let snapshots = data_dir.join("snapshots");
                let snapshots_preserved = if snapshots_moved {
                    match std::fs::rename(&snapshots, staging.join("snapshots")) {
                        Ok(()) => true,
                        Err(err) => {
                            tracing::error!(path = %snapshots.display(), error = %err, "restore rollback failed");
                            rollback_errors.push(format!(
                                "{} -> {}: {err}",
                                snapshots.display(),
                                staging.join("snapshots").display()
                            ));
                            false
                        }
                    }
                } else {
                    true
                };
                if snapshots_preserved {
                    let removal = match std::fs::remove_dir_all(&data_dir) {
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                        result => result,
                    };
                    match removal.and_then(|()| std::fs::rename(&staging, &data_dir)) {
                        Ok(()) => {}
                        Err(err) => {
                            tracing::error!(path = %staging.display(), error = %err, "restore rollback failed");
                            rollback_errors.push(format!(
                                "{} -> {}: {err}",
                                staging.display(),
                                data_dir.display()
                            ));
                        }
                    }
                }
                if backend_restored && rollback_errors.is_empty() {
                    if let Err(err) = std::fs::remove_dir(&backend_staging) {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            tracing::error!(path = %backend_staging.display(), error = %err, "restore rollback cleanup failed");
                            rollback_errors.push(format!("{}: {err}", backend_staging.display()));
                        }
                    }
                }
                if rollback_errors.is_empty() {
                    if let Err(err) = crate::utils::fs::sync_parent_dir(&data_dir) {
                        tracing::error!(path = %storage_root.display(), error = %err, "restore rollback sync failed");
                        rollback_errors.push(format!("{}: {err}", storage_root.display()));
                    }
                }
                if !rollback_errors.is_empty() {
                    return Err(crate::error::Error::Io(std::io::Error::other(format!(
                        "{e}; rollback incomplete; manual recovery required; staging: {}, {}; errors: {}",
                        staging.display(), backend_staging.display(), rollback_errors.join("; ")
                    ))));
                }
                Err(crate::error::Error::Io(e))
            }
        }
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
