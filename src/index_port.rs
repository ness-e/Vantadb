//! Neutral index-port leaf (F3X): breaks the `storage ↔ index` module cycle.
//!
//! Consumers depend DOWNWARD on this leaf (`std` + external crates + `crate::node`
//! + `crate::error` only — same profile as `search_profile.rs`). The leaf names NO
//!   concrete `storage`/`index` type, so `storage → index_port ← index` edges cannot cycle.
//!
//! `pub` (not `pub(crate)`) is load-bearing: benchmarks call `dyn IndexPort` methods
//! through `storage.hnsw`, and external targets cannot name `pub(crate)` traits.
//! New surface is additive; breaking `pub` signature changes are covered by ADR-042.
//! Plain-data index types (`IndexType`, `FreshHnswReport`) live here so both sides
//! name the leaf instead of each other (re-exported at their old paths for compat).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::node::{DiskNodeHeader, DistanceMetric, FilterBitset, VectorRepresentations};

/// Sealed-trait machinery (H3 review): implementors live in OTHER modules
/// (`port_impl`, `vfile`), so a fully-private seal module is unusable here
/// (impls couldn't name it; `pub(crate)` bounds trip `private_bounds` under
/// `-D warnings`). `#[doc(hidden)] pub` is the workable seal: external crates
/// can call the traits but cannot implement them without naming hidden
/// internals (explicitly unsupported).
#[doc(hidden)]
pub mod sealed {
    /// Marker supertrait: implement only inside this crate.
    pub trait Sealed {}
}

/// Supported vector index types.
///
/// Moved from `crate::index` (F3X): plain data (std + serde only), re-exported at
/// the old path for compat. The port needs it for `search_with_method` routing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// Hierarchical Navigable Small World graph index (default).
    #[default]
    Hnsw,
    /// Inverted File index with flat (brute-force) encoding.
    Ivf,
    /// Brute-force flat scan — O(n) on every search.
    Flat,
    /// DiskANN-style Vamana graph (in-memory, no disk I/O).
    DiskAnn,
    /// SCANN-style scalar quantization (SQ8) with re-ranking.
    Scann,
}

/// Report from a single FreshHNSW repair pass.
///
/// Moved from `crate::index::graph::types` (F3X): plain data (std only),
/// re-exported at the old paths for compat. FreshHNSW scans all nodes in the
/// HNSW graph and removes neighbor links that point to node IDs no longer
/// present in the index ("orphan links" left behind by delete operations).
#[derive(Debug, Clone, Copy, Default)]
pub struct FreshHnswReport {
    /// Number of HNSW nodes scanned.
    pub scanned_nodes: u64,
    /// Total number of layers (across all nodes) checked.
    pub total_layers: u64,
    /// Number of orphan neighbor links removed.
    pub repaired_links: u64,
    /// Duration of the repair pass in milliseconds.
    pub duration_ms: u64,
    /// Whether the pass completed successfully.
    pub success: bool,
}

/// Read-only view of the vector store that search needs (B1 + firmas FQ H1).
pub trait VectorStoreRef: Send + Sync {
    /// Read a node header at `offset` (`None` = out of range / corrupt).
    fn read_header(&self, offset: u64) -> Option<DiskNodeHeader>;
    /// Whole backing mapping (search rescans it for disk-resident vectors).
    fn mmap_bytes(&self) -> &[u8];
}

/// Mmap role behind which the `storage::vfile` calls in `graph/types.rs` hide (B2/B3).
pub trait MmapBackend: sealed::Sealed + Send + Sync {
    /// Whether the backend is file-mapped (vs in-memory).
    fn is_mmap(&self) -> bool;
    /// Path of the mapped file, if any.
    fn mmap_path(&self) -> Option<&Path>;
    /// Resident mmap bytes, if measurable.
    fn mmap_resident_bytes(&self) -> Option<u64>;
    /// Flush live state into the mapping (no-op when nothing is mapped).
    fn sync_to_mmap(&mut self) -> std::io::Result<()>;
}

/// Port the engine consumes and the index implements (A1–A4 + H4 surface).
/// Idempotent on rebuild; atomic where the backend supports it (documented per impl).
/// Errors: ONLY `crate::error::Error` — no `panic`/`Option`/ad-hoc strings at the boundary.
///
/// Every method is object-safe (`&self`/`&mut self` receivers, no `Self` in
/// argument/return position except `Sized`-bound statics) so `dyn IndexPort`
/// works as the cycle-breaking handle. Creation goes through the `port_impl`
/// factories (a `dyn` handle cannot call `Sized`-bound statics).
pub trait IndexPort: MmapBackend + sealed::Sealed + Send + Sync {
    /// Open an existing index file (factory-called; `None` file → `NotFound` io error).
    fn open_index(path: &Path, use_mmap: bool) -> Result<Self, Error>
    where
        Self: Sized;
    /// Deserialize from bytes, attaching an (initially lazy) mmap backend when
    /// `mmap_path` is `Some` (factory-called).
    fn rebuild_from_bytes(data: &[u8], mmap_path: Option<PathBuf>) -> Result<Self, Error>
    where
        Self: Sized;
    /// Dyn-compatible copy with the same backend kind (replaces `fresh_index_like`).
    fn fresh_box(&self, index_path: PathBuf) -> Result<Box<dyn IndexPort>, Error>;
    /// Stream-persist to `path` (used by the non-mmap save path).
    fn persist_to_file(&self, path: &Path) -> std::io::Result<()>;
    /// Atomically persist to `path` with a live mmap backend and return the
    /// re-mapped index (moved `save_vector_index` dance; RCU-friendly: no `&mut`).
    fn persist_mmap(&self, path: &Path) -> Result<Box<dyn IndexPort>, Error>;
    /// HNSW search entry point (also serves benches/SDK through the `dyn` handle).
    fn search_nearest(
        &self,
        query_vec: &[f32],
        q_1bit: Option<&[u64]>,
        q_3bit: Option<(&[u8], f32)>,
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
    ) -> Vec<(u128, f32)>;
    /// Backend-routed search (Flat / IVF / HNSW / SCANN).
    fn search_with_method(
        &self,
        method: IndexType,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<(u128, f32)>, Error>;
    /// Pluggable-backend search (mirrors `VecIndex::search`; the `dyn`-compatible
    /// spelling so `vec_index()` callers keep working through the port).
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
        distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)>;
    /// Scan all nodes and remove orphan neighbor links.
    fn repair_orphan_links(&self) -> FreshHnswReport;
    /// Storage offset of a node, if present.
    fn storage_offset_of(&self, id: u128) -> Option<u64>;
    /// Cloned vector payload of a node (rare legacy-rescue path only — not hot).
    fn stored_vector(&self, id: u128) -> Option<VectorRepresentations>;
    /// Single-lookup entry view as `(storage_offset, cloned vector)` for legacy
    /// rescue paths that need both fields from the same instant.
    fn node_view(&self, id: u128) -> Option<(u64, VectorRepresentations)>;
    /// Whether the node's vector is SQ8-quantized (`None` = absent; mirrors the
    /// `collect_actions` predicate shape exactly).
    fn is_sq8_vector(&self, id: u128) -> Option<bool>;
    /// Presence probe.
    fn contains_node(&self, id: u128) -> bool;
    /// Number of indexed nodes.
    fn node_count(&self) -> usize;
    /// Whether the index has no nodes.
    fn is_empty(&self) -> bool;
    /// Monotonic insert counter (paired decrements happen ONLY in vacuum — legacy pairing preserved).
    fn total_nodes(&self) -> u64;
    /// Decrement the insert counter (vacuum path only — see `total_nodes`).
    fn decrement_total_nodes(&self, by: u64);
    /// Current entry point, if any.
    fn entry_point(&self) -> Option<u128>;
    /// Replace the entry point (Relaxed ordering, as before).
    fn set_entry_point(&self, id: u128);
    /// Replacement entry point after a delete, if any node remains.
    fn find_new_entry_point(&self) -> Option<u128>;
    /// Highest occupied layer (cache-warmer top-layer walk).
    fn max_layer(&self) -> usize;
    /// All indexed node IDs (snapshot order; callers hold `insert_lock`, so no live-mutation skew).
    fn all_node_ids(&self) -> Vec<u128>;
    /// Neighbor IDs of a node at `layer` (empty when absent).
    fn layer_neighbors(&self, id: u128, layer: usize) -> Vec<u128>;
    /// Inline per-layer neighbor list of a node (no `neighbor_index` fallback;
    /// mirrors the `neighbor_lists` fast path exactly).
    fn inline_neighbors(&self, id: u128, layer: usize) -> Vec<u128>;
    /// Full scan as `(id, storage_offset, flags)` triples (maintenance/vacuum/stats paths).
    fn scan_entries(&self) -> Vec<(u128, u64, u32)>;
    /// Full integrity validation (test/maintenance topology checks).
    fn validate_index(&self) -> Result<(), Vec<String>>;
    /// Number of layers occupied by a node (0 when absent).
    fn node_layers(&self, id: u128) -> usize;
    /// Estimated heap usage in bytes.
    fn estimate_memory_bytes(&self) -> usize;
    /// Random level draw for inserts (same distribution as `random_layer_from_config`).
    fn random_level(&self) -> usize;
    /// Deterministic level draws sharing one RNG stream (seed, n draws) —
    /// replicates the pre-split single-`StdRng` loop exactly (bulk rebuilds).
    fn bulk_levels_seeded(&self, seed: u64, n: usize) -> Vec<usize>;
    /// Configured flat-scan threshold (routing reads).
    fn flat_threshold(&self) -> Option<usize>;
    /// Set the flat-scan threshold (init path only).
    fn set_flat_threshold(&mut self, threshold: Option<usize>);
    /// Configured distance metric (routing reads).
    fn distance_metric(&self) -> DistanceMetric;
    /// Configured backend kind (routing reads).
    fn index_kind(&self) -> IndexType;
    /// Overwrite an entry's storage offset, `false` when absent.
    fn set_storage_offset(&self, id: u128, offset: u64) -> bool;
    /// Remove a node, `false` when absent (does NOT touch `total_nodes` — legacy pairing).
    fn remove_node(&self, id: u128) -> bool;
    /// Insert or replace a node entry.
    fn add_node(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> Result<(), Error>;
    /// Insert or replace a node entry at an explicit level.
    fn add_node_with_level(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
        level: usize,
    ) -> Result<(), Error>;
}
