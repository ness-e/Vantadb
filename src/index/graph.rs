use crate::index::search::SearchProfile;
#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile::MmapMut;
use ahash::RandomState;
use dashmap::DashMap;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use portable_atomic::AtomicU128;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BinaryHeap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub type NeighborVec = SmallVec<[u128; 32]>;

pub(crate) const ENTRY_POINT_NONE: u128 = u128::MAX;
pub(crate) const MAX_VEC_F32_LEN: usize = 10_000_000;

use super::distance::*;
pub use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};

#[inline(always)]
#[allow(unused_variables)]
pub(crate) fn prefetch_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // SAFETY: `madvise` is async-signal-safe. Takes a pointer+len derived from
        // the owned mmap; invalid offsets are ignored by the kernel.
        unsafe {
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_WILLNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Memory::{PrefetchVirtualMemory, WIN32_MEMORY_RANGE_ENTRY};
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        // SAFETY: `GetCurrentProcess` returns a pseudo-handle (always valid).
        // `PrefetchVirtualMemory` takes a validated pointer+len from the owned mmap;
        // invalid ranges are best-effort.
        unsafe {
            let addr = mmap_ptr.add(offset) as *mut core::ffi::c_void;
            let entry = WIN32_MEMORY_RANGE_ENTRY {
                VirtualAddress: addr,
                NumberOfBytes: len,
            };
            let process_handle = GetCurrentProcess();
            PrefetchVirtualMemory(process_handle, 1, std::ptr::addr_of!(entry), 0);
        }
    }

    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}

#[inline(always)]
/// # Safety
///
/// `mmap_ptr` must point to a valid mmap region, and `offset + len` must be
/// within that region. The caller must ensure the mapping is not concurrently
/// unmapped or resized.
#[allow(unused_variables)]
pub unsafe fn release_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // SAFETY: caller guarantees `mmap_ptr` + `offset + len` is within a valid
        // mmap region. `madvise` with `MADV_DONTNEED` is async-signal-safe; the
        // mapping itself remains valid after the hint.
        unsafe {
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_DONTNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        let _ = (mmap_ptr, offset, len);
    }

    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}

use crate::config::PrefetchMode;

/// Report from a single FreshHNSW repair pass.
///
/// FreshHNSW scans all nodes in the HNSW graph and removes
/// neighbor links that point to node IDs no longer present in the index
/// ("orphan links" left behind by delete operations).
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
use std::sync::OnceLock;

static PREFETCH_MODE: OnceLock<PrefetchMode> = OnceLock::new();

pub fn set_prefetch_mode(mode: PrefetchMode) {
    let _ = PREFETCH_MODE.set(mode);
}

#[inline(always)]
pub(crate) fn should_prefetch() -> bool {
    if let Some(mode) = PREFETCH_MODE.get() {
        return mode.is_prefetch_enabled();
    }
    let mode = std::env::var("VANTA_PREFETCH")
        .ok()
        .map(|v| PrefetchMode::from_env_value(&v));
    let disabled = std::env::var("VANTA_DISABLE_PREFETCH")
        .ok()
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    match (mode, disabled) {
        (Some(m), _) => m.is_prefetch_enabled(),
        (_, true) => false,
        _ => true,
    }
}

/// Current HNSW vector index format version.
pub const VECTOR_INDEX_VERSION: u16 = 8;

pub struct HnswNode {
    pub id: u128,
    pub bitset: FilterBitset,
    pub vec_data: VectorRepresentations,
    pub storage_offset: u64,
    pub inv_cached_norm: f32,
    pub norm_sq: f32,
    pub flags: u32,
    /// Inline neighbor lists per layer (index = layer number).
    /// Populated during insert/shrink to avoid a separate `neighbor_index` DashMap
    /// lookup in `search_layer`. Falls back to `neighbor_index` if empty.
    /// Thread-safe because DashMap shard RwLock protects concurrent reads/writes.
    pub neighbor_lists: Vec<NeighborVec>,
}

impl HnswNode {
    /// Returns a zero-copy borrow of the vector data as `&[f32]`.
    pub fn vector_slice(&self) -> Option<&[f32]> {
        self.vec_data.as_f32_slice()
    }
}

#[derive(Debug)]
pub enum IndexBackend {
    InMemory,
    MMapFile {
        path: PathBuf,
        mmap: Option<MmapMut>,
    },
}

impl IndexBackend {
    pub fn new_mmap(path: PathBuf) -> Self {
        IndexBackend::MMapFile { path, mmap: None }
    }

    pub fn is_mmap(&self) -> bool {
        matches!(self, IndexBackend::MMapFile { .. })
    }

    pub fn mmap_path(&self) -> Option<&Path> {
        match self {
            IndexBackend::MMapFile { path, .. } => Some(path.as_path()),
            IndexBackend::InMemory => None,
        }
    }

    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        match self {
            IndexBackend::MMapFile { mmap: Some(m), .. } => {
                crate::storage::vfile::get_resident_bytes(m.as_ptr(), m.len())
            }
            IndexBackend::MMapFile { path, mmap: None } => {
                let file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::debug!(
                            "mmap_resident_bytes fallback: failed to open {}: {e}",
                            path.display()
                        );
                        return None;
                    }
                };
                // SAFETY: `file` is a valid open handle; `Mmap::map` checks the
                // resulting pointer internally and returns `Err` on failure.
                let mmap = match unsafe { crate::storage::vfile::Mmap::map(&file) } {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!(
                            "mmap_resident_bytes fallback: failed to mmap {}: {e}",
                            path.display()
                        );
                        return None;
                    }
                };
                crate::storage::vfile::get_resident_bytes(mmap.as_ptr(), mmap.len())
            }
            IndexBackend::InMemory => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub m_max0: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f64,
    #[serde(default)]
    pub distance_metric: DistanceMetric,
    /// If `Some(n)`, use brute-force flat scan instead of HNSW graph
    /// when the number of nodes is below this threshold.
    /// Default: `Some(10000)`. Set to `None` to always use HNSW.
    #[serde(default = "default_flat_threshold")]
    pub flat_threshold: Option<usize>,
    /// Index type: HNSW (default) or IVF.
    /// IVF is rebuilt lazily on first search after load.
    #[serde(default)]
    pub index_type: crate::index::IndexType,
    /// Whether adaptive ef_search auto-tuning is enabled.
    /// Default: `false`.
    #[serde(default)]
    pub auto_tune: bool,
}

const fn default_flat_threshold() -> Option<usize> {
    Some(10000)
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 32,
            m_max0: 64,
            ef_construction: 100,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: Some(10000),
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct NodeSim(pub(crate) f32, pub(crate) u128);

impl Eq for NodeSim {}

impl PartialOrd for NodeSim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Total order approximating cosine similarity for HNSW heap bookkeeping
/// (AUDREP-29).
///
/// The plain `f32` ordering is partial: `NaN` compares `Equal` to everything
/// via `partial_cmp(...).unwrap_or(Equal)`. A node whose similarity is `NaN`
/// would therefore _never_ be evicted from the candidate set, tainting the
/// graph topology. This function imposes a deterministic total order and
/// pins every `NaN` below every finite value, so a `NaN` neighbour sorts to
/// the bottom and is pruned first.
#[inline]
pub(crate) fn total_cmp_sim(a: f32, b: f32) -> std::cmp::Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => a.total_cmp(&b),
    }
}

impl Ord for NodeSim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match total_cmp_sim(self.0, other.0) {
            std::cmp::Ordering::Equal => other.1.cmp(&self.1),
            cmp => cmp,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct NodeSimMin(pub(crate) f32, pub(crate) u128);

impl Eq for NodeSimMin {}

impl PartialOrd for NodeSimMin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSimMin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match total_cmp_sim(other.0, self.0) {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            cmp => cmp,
        }
    }
}

pub struct CPIndex {
    pub nodes: DashMap<u128, HnswNode>,
    pub max_layer: AtomicUsize,
    pub entry_point: AtomicU128,
    pub backend: IndexBackend,
    pub config: HnswConfig,
    pub total_nodes: AtomicU64,
    /// RNG for HNSW level assignment (`random_layer`).
    ///
    /// # Contention note (REV-012)
    /// Parking lot Mutex is fast (no syscall uncontested), hold time ≈2-5µs
    /// (one `random_range` call). Micro-batching (HNSW_BATCH_SIZE=64) means
    /// 64 acquisitions per batch → ~128-320µs serialized insert_lock time.
    ///
    /// DashMap sharding (`nodes`) is adequate — default shard count is
    /// `num_cpus * 4`, so concurrent inserts to different shards see no
    /// contention. `search_layer` only holds shard read locks briefly.
    ///
    /// ponytail: Not a measured bottleneck. If profiling later shows this
    /// as hot, switch to `thread_local! { static RNG: RefCell<SmallRng> }`
    /// seeded from `seed_from_u64(42 ^ thread_id)` — eliminates the Mutex
    /// entirely (~20 line change, no correctness impact on HNSW topology
    /// since layer assignment is idempotent across runs).
    pub(crate) rng: parking_lot::Mutex<rand::rngs::StdRng>,
    /// Lazy-built IVF index. Will be `None` until first search with
    /// `config.index_type == IndexType::Ivf`.
    pub ivf_index: parking_lot::Mutex<Option<crate::index::ivf::IvfIndex>>,
    /// Node count the cached `ivf_index` was built over. When `nodes.len()`
    /// diverges (vectors added/removed after the lazy build), the cached IVF
    /// is stale and must be rebuilt on the next search (AUDREP-09).
    pub ivf_built_at_node_count: AtomicUsize,
    /// Flat, lock-friendly neighbor list index.
    /// `pub(crate)` because `HnswNeighborIndex` is only `pub(crate)`.
    pub(crate) neighbor_index: crate::index::neighbor_index::HnswNeighborIndex,
}

use crate::index::distance::f32_l2_norm;

#[inline]
pub(crate) fn cached_norms_for_metric(
    metric: DistanceMetric,
    vec_data: &VectorRepresentations,
) -> (f32, f32) {
    if metric == DistanceMetric::Euclidean || metric == DistanceMetric::Cosine {
        vec_data
            .as_f32_slice()
            .map(|s| {
                let norm = f32_l2_norm(s);
                if norm > f32::EPSILON {
                    (1.0 / norm, norm * norm)
                } else {
                    (0.0, 0.0)
                }
            })
            .unwrap_or((0.0, 0.0))
    } else {
        (0.0, 0.0)
    }
}

impl CPIndex {
    fn init(config: HnswConfig, backend: IndexBackend) -> Self {
        Self {
            nodes: Default::default(),
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend,
            config,
            total_nodes: AtomicU64::new(0),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
            ivf_index: parking_lot::Mutex::new(None),
            ivf_built_at_node_count: AtomicUsize::new(0),
            neighbor_index: crate::index::neighbor_index::HnswNeighborIndex::new(),
        }
    }

    pub fn new() -> Self {
        Self::init(HnswConfig::default(), IndexBackend::InMemory)
    }

    pub fn new_with_config(config: HnswConfig) -> Self {
        Self::init(config, IndexBackend::InMemory)
    }

    pub fn with_backend(backend: IndexBackend) -> Self {
        Self::init(HnswConfig::default(), backend)
    }

    pub fn estimate_memory_bytes(&self) -> usize {
        let mut total = 0usize;
        for r in self.nodes.iter() {
            let node = r.value();
            match &node.vec_data {
                VectorRepresentations::Full(v) => total += v.len() * std::mem::size_of::<f32>(),
                VectorRepresentations::MmapFull(_) => {}
                VectorRepresentations::Binary(b) => total += b.len() * std::mem::size_of::<u64>(),
                VectorRepresentations::Turbo(t) => total += t.len(),
                VectorRepresentations::SQ8(d, _) => total += d.len() + 4,
                VectorRepresentations::None => {}
            }
            total += std::mem::size_of::<HnswNode>();
        }
        total += self.total_nodes.load(Ordering::Relaxed) as usize * 60;
        // Rough estimate for neighbor index storage (DashMap entries)
        for entry in self.neighbor_index.id_to_meta.iter() {
            let num_layers = *entry.value();
            total += num_layers
                * (std::mem::size_of::<(u128, usize)>() + std::mem::size_of::<NeighborVec>());
        }
        total
    }

    fn random_layer(&self) -> usize {
        let mut rng = self.rng.lock();
        // ERR-018: the previous `random_range(0.0001..1.0)` clamped the
        // geometric tail — with the default ml = 1/ln(32) the max achievable
        // level was floor(-ln(0.0001) * ml) = 2, so no node could ever live
        // above layer 2 and sparse/low-degree graphs lost recall. Sample the
        // full unit interval instead: `-(1-u).ln()` is the standard log
        // transform (identical to -ln(u) over (0,1]) but cannot hit `inf` on
        // an exact-zero draw, so the tail is unbounded and follows the
        // intended geometric distribution P(level >= k) = M^-k.
        let u: f64 = rng.random(); // [0.0, 1.0)
        (-(1.0 - u).ln() * self.config.ml).floor() as usize
    }

    #[inline]
    pub fn get_entry_point(&self) -> Option<u128> {
        let ep = self.entry_point.load(Ordering::Relaxed);
        if ep == ENTRY_POINT_NONE {
            None
        } else {
            Some(ep)
        }
    }

    pub fn find_new_entry_point(&self) -> Option<u128> {
        self.nodes
            .iter()
            .max_by_key(|kv| self.neighbor_index.num_layers(*kv.key()).unwrap_or(0))
            .map(|kv| *kv.key())
    }

    #[inline]
    pub fn set_entry_point(&self, id: u128) {
        self.entry_point.store(id, Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn fast_similarity(
        &self,
        query_vec: &[f32],
        query_norm: Option<f32>,
        query_inv_norm: Option<f32>,
        node: &HnswNode,
        metric: DistanceMetric,
    ) -> f32 {
        match metric {
            DistanceMetric::Cosine => {
                if let Some(q_inv_norm) = query_inv_norm {
                    let node_inv_norm = node.inv_cached_norm;
                    if node_inv_norm > f32::EPSILON {
                        if let Some(node_slice) = node.vec_data.as_f32_slice() {
                            return cosine_sim_cached_norms(
                                query_vec,
                                q_inv_norm,
                                node_slice,
                                node_inv_norm,
                            );
                        }
                    }
                }
                calculate_similarity(query_vec, query_norm, None, None, &node.vec_data, metric)
            }
            DistanceMetric::Euclidean => {
                if let Some(node_slice) = node.vec_data.as_f32_slice() {
                    if node.norm_sq > f32::EPSILON {
                        if let Some(qn) = query_norm {
                            let query_norm_sq = qn * qn;
                            return -euclidean_distance_sq_with_norms(
                                query_vec,
                                query_norm_sq,
                                node_slice,
                                node.norm_sq,
                            );
                        }
                    }
                    -euclidean_distance_squared_f32(query_vec, node_slice)
                } else {
                    calculate_similarity(query_vec, query_norm, None, None, &node.vec_data, metric)
                }
            }
            // Sparse vectors are searched via a dedicated brute-force path (see
            // VantaEmbedded::sparse_memory_search), never through the dense HNSW.
            DistanceMetric::SparseDot => 0.0,
        }
    }

    fn validate_node(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: &VectorRepresentations,
        storage_offset: u64,
    ) -> bool {
        if let Some(mut node) = self.nodes.get_mut(&id) {
            node.bitset = bitset;
            node.vec_data = vec_data.clone();
            node.storage_offset = storage_offset;
            (node.inv_cached_norm, node.norm_sq) = self.compute_cached_norms(&node.vec_data);
            return true;
        }

        if vec_data.is_none() {
            self.neighbor_index.allocate(id, 1);
            self.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data: vec_data.clone(),
                    storage_offset,
                    inv_cached_norm: 0.0,
                    norm_sq: 0.0,
                    flags: 0,
                    neighbor_lists: Vec::new(),
                },
            );
            self.total_nodes.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        false
    }

    #[tracing::instrument(skip(self, vec_data), level = "debug")]
    pub fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> Result<(), crate::error::VantaError> {
        if self.validate_node(id, bitset.clone(), &vec_data, storage_offset) {
            return Ok(());
        }

        self.insert_hnsw(id, bitset, vec_data, storage_offset)
    }

    /// Add a node with a pre-computed HNSW layer level (avoids `random_layer()`).
    /// Used for parallel rebuild where each thread computes its own levels
    /// to avoid contention on the shared RNG mutex.
    pub fn add_with_level(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
        level: usize,
    ) -> Result<(), crate::error::VantaError> {
        if self.validate_node(id, bitset.clone(), &vec_data, storage_offset) {
            return Ok(());
        }

        self.insert_hnsw_with_level(id, bitset, vec_data, storage_offset, level)
    }

    #[inline]
    pub(crate) fn compute_cached_norms(&self, vec_data: &VectorRepresentations) -> (f32, f32) {
        cached_norms_for_metric(self.config.distance_metric, vec_data)
    }

    fn insert_hnsw(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> Result<(), crate::error::VantaError> {
        let level = self.random_layer();
        let ef_cons = self.config.ef_construction;

        let (inv_cached_norm, norm_sq) = self.compute_cached_norms(&vec_data);

        let query_f32 = match vec_data.to_f32() {
            Some(v) => v,
            None => return Ok(()),
        };

        // AUDREP-27: reject zero-norm vectors up-front, before any graph
        // mutation. Cosine similarity is undefined for a zero vector; the old
        // code inserted the node and then silently removed it, leaving the
        // caller believing the insert succeeded (and occasionally leaking an
        // entry point / neighbour allocation). Fail loudly instead.
        if self.config.distance_metric == DistanceMetric::Cosine {
            let norm = f32_l2_norm(&query_f32);
            if norm < f32::EPSILON {
                return Err(crate::error::VantaError::InvalidInput(format!(
                    "cannot index node {id}: zero-norm vector is undefined under \
                     cosine similarity"
                )));
            }
        }

        self.neighbor_index.allocate(id, level + 1);
        let empty_layers = vec![NeighborVec::new(); level + 1];

        let node = HnswNode {
            id,
            bitset,
            vec_data,
            storage_offset,
            inv_cached_norm,
            norm_sq,
            flags: 0,
            neighbor_lists: empty_layers,
        };

        let ep = match self.get_entry_point() {
            None => {
                self.set_entry_point(id);
                self.max_layer.store(level, Ordering::Release);
                self.nodes.insert(id, node);
                self.total_nodes.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            Some(entry) => entry,
        };

        self.nodes.insert(id, node);
        self.total_nodes.fetch_add(1, Ordering::Relaxed);

        let (query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                // Zero-norm was rejected up-front (AUDREP-27).
                let norm = f32_l2_norm(&query_f32);
                (Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(&query_f32);
                (Some(norm), None)
            }
            // SparseDot has its own brute-force search path; unused here.
            DistanceMetric::SparseDot => (None, None),
        };

        let mut curr_entry_points = vec![ep];
        let mut visited: std::collections::HashSet<u128, RandomState> =
            std::collections::HashSet::with_capacity_and_hasher(
                ef_cons.saturating_mul(3),
                RandomState::new(),
            );
        let top_layer = self.max_layer.load(Ordering::Acquire);

        for layer in (level + 1..=top_layer).rev() {
            visited.clear();
            let mut w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                &crate::node::ALL_BITSET,
                false, // no ACORN during construction
                None,
                self.config.distance_metric,
                &mut visited,
                &mut SearchProfile::new(),
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            visited.clear();
            let w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                ef_cons,
                layer,
                &crate::node::ALL_BITSET,
                false, // no ACORN during construction
                None,
                self.config.distance_metric,
                &mut visited,
                &mut SearchProfile::new(),
            );

            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };

            curr_entry_points = w.iter().map(|ns| ns.1).collect();
            let selected_neighbors = self.select_neighbors(w, m_max, |_| false);

            // Connect reverse links first (reads &selected_neighbors by ref),
            // then store the pruned list — avoids cloning for set_neighbors.
            self.connect_layer_neighbors(id, &selected_neighbors, layer, m_max);

            // Populate both neighbor_index and inline cache.
            // Inline cache avoids a 2nd DashMap fallback in search_layer during rebuild.
            let inline_cache = selected_neighbors.clone();
            self.neighbor_index
                .set_neighbors(id, layer, selected_neighbors);
            if let Some(mut node_ref) = self.nodes.get_mut(&id) {
                if node_ref.neighbor_lists.len() > layer {
                    node_ref.neighbor_lists[layer] = inline_cache;
                }
            }
        }

        self.update_metadata(level, id);

        Ok(())
    }

    fn insert_hnsw_with_level(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
        level: usize,
    ) -> Result<(), crate::error::VantaError> {
        let ef_cons = self.config.ef_construction;

        let (inv_cached_norm, norm_sq) = self.compute_cached_norms(&vec_data);

        let query_f32 = match vec_data.to_f32() {
            Some(v) => v,
            None => return Ok(()),
        };

        // AUDREP-27: reject zero-norm up-front, before any graph mutation.
        if self.config.distance_metric == DistanceMetric::Cosine {
            let norm = f32_l2_norm(&query_f32);
            if norm < f32::EPSILON {
                return Err(crate::error::VantaError::InvalidInput(format!(
                    "cannot index node {id}: zero-norm vector is undefined under \
                     cosine similarity"
                )));
            }
        }

        self.neighbor_index.allocate(id, level + 1);
        let empty_layers = vec![NeighborVec::new(); level + 1];

        let node = HnswNode {
            id,
            bitset,
            vec_data,
            storage_offset,
            inv_cached_norm,
            norm_sq,
            flags: 0,
            neighbor_lists: empty_layers,
        };

        let ep = match self.get_entry_point() {
            None => {
                self.set_entry_point(id);
                self.max_layer.store(level, Ordering::Release);
                self.nodes.insert(id, node);
                self.total_nodes.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            Some(entry) => entry,
        };

        self.nodes.insert(id, node);
        self.total_nodes.fetch_add(1, Ordering::Relaxed);

        let (query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                // Zero-norm was rejected up-front (AUDREP-27).
                let norm = f32_l2_norm(&query_f32);
                (Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(&query_f32);
                (Some(norm), None)
            }
            // SparseDot has its own brute-force search path; unused here.
            DistanceMetric::SparseDot => (None, None),
        };

        let mut curr_entry_points = vec![ep];
        let mut visited: std::collections::HashSet<u128, RandomState> =
            std::collections::HashSet::with_capacity_and_hasher(
                ef_cons.saturating_mul(3),
                RandomState::new(),
            );
        let top_layer = self.max_layer.load(Ordering::Acquire);

        for layer in (level + 1..=top_layer).rev() {
            visited.clear();
            let mut w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                &crate::node::ALL_BITSET,
                false,
                None,
                self.config.distance_metric,
                &mut visited,
                &mut SearchProfile::new(),
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            visited.clear();
            let w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                ef_cons,
                layer,
                &crate::node::ALL_BITSET,
                false,
                None,
                self.config.distance_metric,
                &mut visited,
                &mut SearchProfile::new(),
            );

            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };

            curr_entry_points = w.iter().map(|ns| ns.1).collect();
            let selected_neighbors = self.select_neighbors(w, m_max, |_| false);

            // Connect reverse links first (reads &selected_neighbors by ref),
            // then store the pruned list — avoids cloning for set_neighbors.
            self.connect_layer_neighbors(id, &selected_neighbors, layer, m_max);

            // Populate both neighbor_index and inline cache.
            let inline_cache = selected_neighbors.clone();
            self.neighbor_index
                .set_neighbors(id, layer, selected_neighbors);
            if let Some(mut node_ref) = self.nodes.get_mut(&id) {
                if node_ref.neighbor_lists.len() > layer {
                    node_ref.neighbor_lists[layer] = inline_cache;
                }
            }
        }

        self.update_metadata(level, id);

        Ok(())
    }

    fn connect_layer_neighbors(
        &self,
        id: u128,
        selected_neighbors: &NeighborVec,
        layer: usize,
        m_max: usize,
    ) {
        for &neighbor_id in selected_neighbors {
            // Single DashMap access: try-add reverse link + check if shrink needed.
            // Replaces the old 3-access pattern (add_neighbor + len_neighbors + get_neighbors).
            let (_added, maybe_full_list) =
                self.neighbor_index
                    .try_add_and_get_if_full(neighbor_id, layer, id, m_max);

            if let Some(full_list) = maybe_full_list {
                self.shrink_neighbors(neighbor_id, m_max, &full_list, layer);
            }
        }
    }

    #[inline]
    fn shrink_neighbors(
        &self,
        neighbor_id: u128,
        m_max: usize,
        current_neighbors: &[u128],
        layer: usize,
    ) {
        // 1. Read the node's vector data and cached norm
        let (nb_vec, nb_inv_norm) = match self.nodes.get(&neighbor_id) {
            Some(n) => (
                n.vec_data.as_f32_slice().map(|s| s.to_vec()),
                n.inv_cached_norm,
            ),
            None => (None, 0.0),
        };

        if let Some(nb_v) = nb_vec {
            let mut cand_heap = BinaryHeap::new();
            let q_norm = if nb_inv_norm > f32::EPSILON {
                Some(1.0 / nb_inv_norm)
            } else {
                None
            };
            let q_inv_norm = if nb_inv_norm > f32::EPSILON {
                Some(nb_inv_norm)
            } else {
                None
            };
            for &n_target in current_neighbors {
                if let Some(nt) = self.nodes.get(&n_target) {
                    let d = self.fast_similarity(
                        &nb_v,
                        q_norm,
                        q_inv_norm,
                        &nt,
                        self.config.distance_metric,
                    );
                    cand_heap.push(NodeSimMin(d, n_target));
                }
            }
            // INV-024 M-8 (reachability): never drop the last remaining
            // incoming link of a node. The just-added reverse link
            // (neighbor_id → new_node) carries inbound_count == 1 right after
            // connect_layer_neighbors pushed it, so it survives the prune; the
            // more important case is later prunes: an old node X that sits at
            // the bottom of a saturated list must NOT be evicted if it is X's
            // only remaining incoming edge, otherwise X loses ALL in-edges and
            // becomes an island (unreachable from the entry point by directed
            // BFS) while still being present in `self.nodes`.
            // The scan MUST cover every candidate: an early exit once
            // `pruned.len() >= m_max` would silently drop last-inbound nodes
            // ranked after the cutoff. Over-capacity lists are the accepted
            // price for the invariant.
            //
            // AUD-014: delegate selection to the canonical `select_neighbors`
            // (NodeSimMin::Ord tie-break: id ascending) — previously this block
            // re-implemented top-M with a pure-sim comparator, producing a
            // different (arbitrary) tie order than the insert path.
            let pruned = self.select_neighbors(cand_heap, m_max, |cand| {
                self.neighbor_index.inbound_count(cand) <= 1
            });
            // Populate both neighbor_index and inline cache.
            let inline_cache = pruned.clone();
            self.neighbor_index
                .set_neighbors(neighbor_id, layer, pruned);
            if let Some(mut node_ref) = self.nodes.get_mut(&neighbor_id) {
                if node_ref.neighbor_lists.len() > layer {
                    node_ref.neighbor_lists[layer] = inline_cache;
                }
            }
        }
    }

    fn update_metadata(&self, level: usize, id: u128) {
        let current_max = self.max_layer.load(Ordering::Acquire);
        if level > current_max {
            self.max_layer.fetch_max(level, Ordering::Release);
            self.set_entry_point(id);
        }
    }

    pub(crate) fn serialization_order(&self) -> Vec<u128> {
        use std::collections::{HashSet, VecDeque};

        let mut order = Vec::with_capacity(self.nodes.len());
        let mut seen = HashSet::new();

        if let Some(ep) = self.get_entry_point() {
            let mut queue = VecDeque::new();
            queue.push_back(ep);
            seen.insert(ep);

            while let Some(node_id) = queue.pop_front() {
                order.push(node_id);
                if self.nodes.contains_key(&node_id) {
                    let num_layers = self.neighbor_index.num_layers(node_id).unwrap_or(0);
                    for layer in (0..num_layers).rev() {
                        let neighbors = self
                            .neighbor_index
                            .get_neighbors(node_id, layer)
                            .unwrap_or_default();
                        for &neighbor_id in &neighbors {
                            if seen.insert(neighbor_id) {
                                queue.push_back(neighbor_id);
                            }
                        }
                    }
                }
            }
        }

        let mut orphans: Vec<u128> = self
            .nodes
            .iter()
            .map(|r| *r.key())
            .filter(|id| !seen.contains(id))
            .collect();
        orphans.sort_unstable();
        order.extend(orphans);
        order
    }

    /// Scans all nodes in the graph and removes neighbor links that point
    /// to node IDs that no longer exist in `self.nodes` (orphan links).
    ///
    /// Orphan links accumulate when nodes are removed via `apply_delete`
    /// (which removes the node from `self.nodes` but does not update the
    /// neighbor lists of surviving nodes). This degrades search quality
    /// over time because the graph becomes less navigable.
    ///
    /// # ponytail
    /// O(n × m × layers) scan. For ~10M nodes with M=32, that is ~320M
    /// `contains_key` checks. Do not optimize prematuramente.
    ///
    /// # Deadlock avoidance
    /// DashMap's `iter()` locks **all** shards. Calling `contains_key()` or
    /// `get_mut()` while the iter lock is held would deadlock. This method
    /// uses a three-phase approach:
    ///   1. Snapshot all existing node IDs into a local HashSet.
    ///   2. Scan neighbor lists under the iter lock (read-only, using the
    ///      snapshot for existence checks). Record (node_id, layer) pairs
    ///      that need repair.
    ///   3. Repair each recorded pair with `get_mut()` — iter lock is
    ///      released, so only one shard is locked at a time.
    pub fn repair_orphan_links(&self) -> FreshHnswReport {
        let start = std::time::Instant::now();

        // Phase 1: Snapshot all existing node IDs into a local HashSet.
        let active_nodes: std::collections::HashSet<u128> =
            self.nodes.iter().map(|kv| *kv.key()).collect();

        // Phase 2: Scan neighbor lists via neighbor_index.for_each(), identify
        // orphan links (neighbor IDs not in active_nodes). Record (node_id, layer)
        // pairs that need repair and count all individual orphan links found.
        let mut scanned_nodes: u64 = 0;
        let mut total_layers: u64 = 0;
        let mut orphan_count: u64 = 0;
        let mut to_repair: Vec<(u128, usize)> = Vec::new();

        self.neighbor_index.for_each(|node_id, layers| {
            scanned_nodes += 1;
            for (layer_idx, neighbors) in layers.iter().enumerate() {
                total_layers += 1;
                let mut layer_has_orphan = false;
                for &nid in neighbors {
                    if !active_nodes.contains(&nid) {
                        orphan_count += 1;
                        layer_has_orphan = true;
                    }
                }
                if layer_has_orphan {
                    to_repair.push((node_id, layer_idx));
                }
            }
        });

        // Phase 3: Repair each recorded layer — retain only links whose
        // target ID exists in active_nodes.
        for (node_id, layer) in &to_repair {
            self.neighbor_index
                .retain_neighbors(*node_id, *layer, |nid| active_nodes.contains(nid));
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        FreshHnswReport {
            scanned_nodes,
            total_layers,
            repaired_links: orphan_count,
            duration_ms,
            success: true,
        }
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a random HNSW layer level from a generic RNG.
/// Used by parallel rebuild to avoid contention on `CPIndex::rng` mutex.
pub fn random_layer_from_config<R: rand::Rng>(config: &HnswConfig, rng: &mut R) -> usize {
    // ERR-018: same fix as `CPIndex::random_layer` — draw from the full unit
    // interval so the geometric tail is not truncated at
    // floor(-ln(0.0001) * ml) = 2 (with the default ml = 1/ln(32)).
    let u: f64 = rng.random(); // [0.0, 1.0)
    (-(1.0 - u).ln() * config.ml).floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::ALL_BITSET;

    // ── Miri tests for unsafe patterns ──────────────────────────────
    //
    // graph.rs has 3 unsafe patterns: prefetch_mmap_vector (madvise),
    // release_mmap_vector (madvise), and mmap_resident_bytes (Mmap::map).
    // These all require actual system calls that Miri cannot execute
    // (MIRI_NO_HOST_FALLBACK=1).
    //
    // INSTEAD, these Miri tests exercise HNSW graph construction and
    // search. This transitively covers the unsafe in distance.rs
    // (chunks_exact + unwrap_unchecked — 14 blocks) through the
    // insert_hnsw → search_layer → fast_similarity → distance kernel
    // call chain, plus the dispatch via select_kernels().

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_graph_hnsw_build_and_search() {
        let config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None, // force HNSW path
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        // Insert vectors — this calls insert_hnsw → distance kernels
        for i in 0u128..5 {
            let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
            index
                .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }
        assert_eq!(index.nodes.len(), 5);
        assert!(index.get_entry_point().is_some());

        // Search — this calls search_layer → fast_similarity → distance kernels
        let query: Vec<f32> = (0..8).map(|d| (d as f32).sin()).collect();
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 3, None);
        assert!(!results.is_empty());
        for &(id, score) in &results {
            assert!(score.is_finite(), "score for id={} should be finite", id);
        }
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_graph_hnsw_euclidean() {
        let config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Euclidean,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        // Insert points in 4D — tests the f32x8 kernels with size < 8 (sub-chunk path)
        let vectors: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
        for (i, v) in vectors.iter().enumerate() {
            index
                .add(
                    i as u128,
                    FilterBitset::new(),
                    VectorRepresentations::Full(v.clone()),
                    0,
                )
                .expect("test vectors are non-zero-norm");
        }

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 4, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0, "identical vector should be closest");
        for &(_, score) in &results {
            assert!(score.is_finite(), "Euclidean score should be finite");
        }
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_graph_hnsw_multiple_layers() {
        // Insert enough vectors to trigger multiple HNSW layers
        let config = HnswConfig {
            m: 4,
            m_max0: 8,
            ef_construction: 100,
            ef_search: 100,
            ml: 1.0 / (4_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        for i in 0u128..50 {
            let v: Vec<f32> = (0..16).map(|d| ((i * 16 + d) as f32).cos()).collect();
            index
                .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }
        assert_eq!(index.nodes.len(), 50);

        let query: Vec<f32> = (0..16).map(|d| (d as f32).cos()).collect();
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
        assert_eq!(results.len(), 5);
        for &(_, score) in &results {
            assert!(score.is_finite());
        }
    }

    #[cfg(miri)]
    #[test]
    #[ignore] // croaring (C FFI) can't run under Miri
    fn miri_graph_entry_point_management() {
        let index = CPIndex::new();
        assert!(index.get_entry_point().is_none());
        assert!(index.find_new_entry_point().is_none());

        // Add a node → entry point should be set
        index
            .add(
                42,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![1.0, 0.0, 0.0, 0.0]),
                0,
            )
            .expect("test vector is non-zero-norm");
        assert_eq!(index.get_entry_point(), Some(42));

        // Check that we can set entry point
        index.set_entry_point(99);
        assert_eq!(index.get_entry_point(), Some(99));
    }

    /// Euclidean distance invariants: identical vectors → score ≈ 0.0,
    /// all scores ≤ 0 (negative distance), descending order.
    #[test]
    fn test_euclidean_distance_metric() {
        let index = CPIndex::new_with_config(HnswConfig {
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        });

        let vectors: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
        for (i, v) in vectors.iter().enumerate() {
            index
                .add(
                    i as u128,
                    FilterBitset::new(),
                    VectorRepresentations::Full(v.clone()),
                    0,
                )
                .expect("test vectors are non-zero-norm");
        }

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 4, None);

        assert!(
            !results.is_empty(),
            "Euclidean search should return results"
        );

        let (closest_id, closest_score) = results[0];
        assert_eq!(
            closest_id, 0,
            "identical vector should be closest (id=0), got id={}",
            closest_id
        );
        assert!(
            closest_score.abs() < 0.01,
            "identical vector should have score ~0.0, got {}",
            closest_score
        );

        for (_id, score) in &results {
            assert!(
                *score <= 0.001,
                "Euclidean scores must be <= 0, got {}",
                score
            );
        }

        for window in results.windows(2) {
            assert!(
                window[0].1 >= window[1].1 - f32::EPSILON,
                "Euclidean scores must be descending: {} < {}",
                window[0].1,
                window[1].1
            );
        }
    }

    // ── AUDREP-27: zero-norm rejection ──────────────────────────────

    #[test]
    fn test_add_zero_norm_vector_rejected() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // Old behaviour: insert_hnsw inserted the node, then silently removed
        // it on zero norm, so `add` returned success while the node vanished.
        let err = index
            .add(
                1,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![0.0, 0.0, 0.0]),
                0,
            )
            .expect_err("zero-norm vector must be rejected under cosine");
        assert!(
            matches!(err, crate::error::VantaError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );
        // The rejection happens before any graph mutation: no node survives,
        // no total_nodes increment, no entry point left behind.
        assert_eq!(index.nodes.len(), 0);
        assert_eq!(index.total_nodes.load(Ordering::Relaxed), 0);
        assert!(index.get_entry_point().is_none());

        // A subsequent valid vector inserts normally and becomes the entry point.
        index
            .add(
                2,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![1.0, 0.0, 0.0]),
                0,
            )
            .expect("valid non-zero-norm vector should insert");
        assert_eq!(index.nodes.len(), 1);
        assert_eq!(index.get_entry_point(), Some(2));
    }

    // ── AUD-29: NaN total ordering / eviction ───────────────────────

    #[test]
    fn test_nodesim_nan_total_order_evicts_extreme() {
        // NaN is pinned below every finite value in the total order, so it
        // sorts to the extreme low end and is pruned first.
        assert_eq!(total_cmp_sim(f32::NAN, f32::NAN), std::cmp::Ordering::Equal);
        assert_eq!(total_cmp_sim(f32::NAN, 0.5), std::cmp::Ordering::Less);
        assert_eq!(total_cmp_sim(0.5, f32::NAN), std::cmp::Ordering::Greater);
        assert_eq!(
            total_cmp_sim(-f32::MAX, f32::NAN),
            std::cmp::Ordering::Greater
        );
        // Finite ordering is unchanged.
        assert_eq!(total_cmp_sim(0.3, 0.5), std::cmp::Ordering::Less);

        // End-to-end: a NaN neighbour is evicted from the top-M candidate set.
        let index = CPIndex::new();
        let mut heap = BinaryHeap::new();
        heap.push(NodeSimMin(f32::NAN, 99));
        heap.push(NodeSimMin(0.9, 0));
        heap.push(NodeSimMin(0.7, 1));
        let selected = index.select_neighbors(heap, 2, |_| false);
        assert!(!selected.contains(&99), "NaN neighbour must be evicted");
        assert!(selected.contains(&0) && selected.contains(&1));
    }

    // ── AUD-014: deterministic tie-break / single selection path ───────

    #[test]
    fn test_select_neighbors_tie_break_deterministic_across_heap_orders() {
        let index = CPIndex::new();
        // Tied similarities — the only differentiator must be node id (asc).
        let candidates = [
            NodeSimMin(0.7, 30),
            NodeSimMin(0.7, 10),
            NodeSimMin(0.7, 20),
            NodeSimMin(0.3, 5),
        ];

        // Different heap push orders simulate different construction paths
        // (insert vs shrink) producing the same candidate set.
        let run = |push_order: &[usize]| {
            let mut heap = BinaryHeap::new();
            for &i in push_order {
                heap.push(candidates[i].clone());
            }
            index.select_neighbors(heap, 2, |_| false)
        };

        let a = run(&[0, 1, 2, 3]);
        let b = run(&[3, 2, 1, 0]);
        assert_eq!(
            a.as_slice(),
            &[10u128, 20u128],
            "tie-break must be by ascending node id"
        );
        assert_eq!(
            a.as_slice(),
            b.as_slice(),
            "identical candidate set must yield identical neighbor lists \
             regardless of heap push order (AUD-014)"
        );
    }

    #[test]
    fn test_shrink_neighbors_keeps_last_inbound_over_capacity() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 2,
            m_max0: 4,
            ef_construction: 8,
            ef_search: 8,
            ml: 1.0 / (2_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // A (id 0) is the shrink target; B (1) / C (2) are close to A, D (3)
        // is farthest and ranks beyond m_max; E (4) is an unrelated carrier.
        for (id, v) in [
            (0u128, vec![1.0, 0.0, 0.0]),
            (1u128, vec![0.99, 0.01, 0.0]),
            (2u128, vec![0.9, 0.1, 0.0]),
            (3u128, vec![-1.0, 0.0, 0.0]),
            (4u128, vec![0.0, 1.0, 0.0]),
        ] {
            index
                .add(id, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }

        // Reset construction topology so the test fully controls inbound state.
        // inbound_count spans ALL layers, so every layer of every node must
        // be emptied before seeding the exact last-inbound scenario.
        for id in 0u128..5 {
            let layers = index.neighbor_index.num_layers(id).unwrap_or(0);
            for layer in 0..layers {
                index
                    .neighbor_index
                    .set_neighbors(id, layer, NeighborVec::new());
            }
        }
        // A's saturated list [B, C, D]. D's ONLY inbound reference is A's own
        // list (inbound_count == 1) → the shrink must NOT evict it.
        index
            .neighbor_index
            .set_neighbors(0, 0, NeighborVec::from_slice(&[1, 2, 3]));

        index.shrink_neighbors(0, 2, &[1, 2, 3], 0);

        let pruned = index
            .neighbor_index
            .get_neighbors(0, 0)
            .expect("A has layer 0");
        assert_eq!(
            pruned.as_slice(),
            &[1u128, 2u128, 3u128],
            "last-inbound node D must survive the shrink and keep rank order"
        );
        assert_eq!(
            pruned.len(),
            3,
            "over-capacity list is the accepted price for INV-024"
        );
    }

    #[test]
    fn test_shrink_neighbors_evicts_non_last_inbound() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 2,
            m_max0: 4,
            ef_construction: 8,
            ef_search: 8,
            ml: 1.0 / (2_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        for (id, v) in [
            (0u128, vec![1.0, 0.0, 0.0]),
            (1u128, vec![0.99, 0.01, 0.0]),
            (2u128, vec![0.9, 0.1, 0.0]),
            (3u128, vec![-1.0, 0.0, 0.0]),
            (4u128, vec![0.0, 1.0, 0.0]),
        ] {
            index
                .add(id, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }

        for id in 0u128..5 {
            let layers = index.neighbor_index.num_layers(id).unwrap_or(0);
            for layer in 0..layers {
                index
                    .neighbor_index
                    .set_neighbors(id, layer, NeighborVec::new());
            }
        }
        // Same saturated list, but D now has a SECOND inbound reference (E's
        // list) → D is evictable and must be dropped to stay within m_max.
        index
            .neighbor_index
            .set_neighbors(0, 0, NeighborVec::from_slice(&[1, 2, 3]));
        index
            .neighbor_index
            .set_neighbors(4, 0, NeighborVec::from_slice(&[3]));

        index.shrink_neighbors(0, 2, &[1, 2, 3], 0);

        let pruned = index
            .neighbor_index
            .get_neighbors(0, 0)
            .expect("A has layer 0");
        assert_eq!(
            pruned.as_slice(),
            &[1u128, 2u128],
            "non-last-inbound D must be evicted to enforce m_max"
        );
    }

    // ── repair_orphan_links ─────────────────────────────────────────

    #[test]
    fn test_repair_orphan_links_empty_index() {
        let index = CPIndex::new();
        let report = index.repair_orphan_links();
        assert_eq!(report.scanned_nodes, 0);
        assert_eq!(report.total_layers, 0);
        assert_eq!(report.repaired_links, 0);
        assert!(report.success);
    }

    #[test]
    fn test_repair_orphan_links_no_orphans() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 4,
            m_max0: 8,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (4_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // Insert nodes A, B, C — they form a connected graph with no orphans
        for i in 0u128..5 {
            let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
            index
                .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }

        let report = index.repair_orphan_links();
        assert!(report.scanned_nodes > 0, "should scan at least one node");
        assert_eq!(report.repaired_links, 0, "no orphans expected");
        assert!(report.success);
    }

    #[test]
    fn test_repair_orphan_links_after_delete() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // Insert nodes 0, 1, 2 — they link to each other via HNSW
        for i in 0u128..5 {
            let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
            index
                .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }
        assert_eq!(index.nodes.len(), 5);

        // Manually add orphan links: give node 0 a link to node 99 (doesn't exist)
        {
            let mut l0 = index.neighbor_index.get_neighbors(0, 0).unwrap_or_default();
            if !l0.contains(&99) {
                l0.push(99);
            }
            index.neighbor_index.set_neighbors(0, 0, l0);
        }
        // Give node 1 a link to node 999 (doesn't exist)
        {
            let mut l0 = index.neighbor_index.get_neighbors(1, 0).unwrap_or_default();
            if !l0.contains(&999) {
                l0.push(999);
            }
            index.neighbor_index.set_neighbors(1, 0, l0);
        }

        // Remove node 2 from the index entirely (simulating delete)
        let removed_node = index.nodes.remove(&2);
        assert!(removed_node.is_some(), "node 2 should exist before removal");

        // Now node 0 and node 1 both have orphan links to deleted/never-existing nodes.
        // Node 2's neighbors (which we removed the node for) can't be checked since
        // the node is gone, but other nodes that linked to node 2 now have orphan links.

        let report = index.repair_orphan_links();
        assert!(report.scanned_nodes > 0, "should scan nodes");
        assert!(
            report.repaired_links >= 2,
            "should repair at least 2 orphan links (99, 999), got {}",
            report.repaired_links
        );
        assert!(report.success);

        // Verify the orphans were actually removed
        if let Some(l0) = index.neighbor_index.get_neighbors(0, 0) {
            assert!(!l0.contains(&99), "node 0 should no longer link to 99");
        };
        if let Some(l0) = index.neighbor_index.get_neighbors(1, 0) {
            assert!(!l0.contains(&999), "node 1 should no longer link to 999");
        };
    }

    #[test]
    fn test_repair_orphan_links_multiple_layers() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 4,
            m_max0: 8,
            ef_construction: 100,
            ef_search: 100,
            ml: 1.0 / (4_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        });

        // Insert enough nodes to create multi-layer graph
        for i in 0u128..30 {
            let v: Vec<f32> = (0..16).map(|d| ((i * 16 + d) as f32).cos()).collect();
            index
                .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
                .expect("test vectors are non-zero-norm");
        }
        assert_eq!(index.nodes.len(), 30);

        // Inject orphan links at layers 0 and 1
        for node_id in [0u128, 5, 10] {
            let num_layers = index.neighbor_index.num_layers(node_id).unwrap_or(0);
            for layer in 0..num_layers {
                let mut l = index
                    .neighbor_index
                    .get_neighbors(node_id, layer)
                    .unwrap_or_default();
                if !l.contains(&100) {
                    l.push(100);
                }
                if !l.contains(&200) {
                    l.push(200);
                }
                if !l.contains(&300) {
                    l.push(300);
                }
                index.neighbor_index.set_neighbors(node_id, layer, l);
            }
        }

        // Remove some nodes to create more orphans
        for id in [15u128, 20, 25] {
            index.nodes.remove(&id);
        }

        let report = index.repair_orphan_links();
        assert!(report.scanned_nodes > 0, "should scan nodes");
        assert!(report.repaired_links > 0, "should repair orphan links");
        assert!(
            report.total_layers >= report.scanned_nodes,
            "total layers >= scanned nodes"
        );
        assert!(report.success);

        // Verify orphans are gone and legit links remain
        for node_id in [0u128, 5, 10] {
            let num_layers = index.neighbor_index.num_layers(node_id).unwrap_or(0);
            for layer in 0..num_layers {
                let l = index
                    .neighbor_index
                    .get_neighbors(node_id, layer)
                    .unwrap_or_default();
                assert!(
                    !l.contains(&100),
                    "node {node_id} layer {layer} should not link to 100"
                );
                assert!(
                    !l.contains(&200),
                    "node {node_id} layer {layer} should not link to 200"
                );
            }
        }
    }

    // ── ERR-018: layer distribution ─────────────────────────────────────
    // The old `random_range(0.0001..1.0)` draw truncated the geometric tail:
    // with the default ml = 1/ln(32) the max achievable level was
    // floor(-ln(0.0001) * ml) = 2, so graphs never grew past layer 2 and
    // sparse/low-degree recall degraded. These tests prove the fixed sampler
    // follows P(level >= k) = M^-k and that real inserts reach level 3+.

    #[test]
    fn random_layer_follows_geometric_distribution() {
        // M=4 → ml = 1/ln(4); expect P(level>=2) = 1/16, P(level>=3) = 1/64.
        let config = HnswConfig {
            m: 4,
            m_max0: 8,
            ef_construction: 100,
            ef_search: 100,
            ml: 1.0 / (4_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        const N: usize = 2000;
        let mut hist = [0usize; 4]; // level buckets: 0, 1, 2, >=3
        for _ in 0..N {
            let lvl = index.random_layer();
            assert!(lvl < 200, "runaway level {lvl}");
            hist[lvl.min(3)] += 1;
        }
        // P(level>=2) = 6.25% → expect ~125 of 2000.
        assert!(hist[2] + hist[3] > 40, "too few level-2+ draws: {hist:?}");
        // P(level>=3) = 1.5625% → expect ~31 of 2000. Structurally 0 under
        // the old capped sampler (max level was 2).
        assert!(
            hist[3] > 5,
            "no level >= 3 draws — layer cap regression: {hist:?}"
        );
    }

    #[test]
    fn insert_reach_layer_three() {
        let config = HnswConfig {
            m: 4,
            m_max0: 8,
            ef_construction: 20,
            ef_search: 20,
            ml: 1.0 / (4_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None, // force HNSW (default threshold brute-forces <10k nodes)
            index_type: crate::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        // Deterministic pseudo-random vectors (LCG with fixed seed).
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut dims = [0f32; 16];
        let mut ge3 = 0usize; // nodes with level >= 3 (i.e. 4+ allocated layers)

        const N: u128 = 2000;
        for i in 0..N {
            for d in &mut dims {
                state = state
                    .wrapping_mul(6364_1362_2384_6793_005)
                    .wrapping_add(1442_6950_4088_9634_07);
                *d = (state >> 33) as f32 / (1u64 << 31) as f32;
            }
            index
                .add(
                    i,
                    FilterBitset::new(),
                    VectorRepresentations::Full(dims.to_vec()),
                    0,
                )
                .expect("insert should succeed");
            if index.neighbor_index.num_layers(i).unwrap_or(0) >= 4 {
                ge3 += 1;
            }
        }

        assert_eq!(index.nodes.len(), N as usize);
        // With M=4 → P(level>=3) = 1/64 ≈ 1.56% → expect ~31 of 2000.
        assert!(
            ge3 > 5,
            "too few inserts reached layer 3+ — layer cap regression: {ge3}"
        );
    }
}
