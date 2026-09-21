//! HNSW graph types: nodes, config, backends and similarity ordering.
//! Split from graph.rs (FIND-48) — re-exported via graph/mod.rs.

// Same canonical-path spelling as `serialize/file.rs` (F3X edge-audit parity).
#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile_mmap::MmapMut;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type NeighborVec = SmallVec<[u128; 32]>;

pub(crate) const ENTRY_POINT_NONE: u128 = u128::MAX;
/// Re-exported from the neutral leaf (`crate::index_port`): plain data shared by
/// both sides of the storage↔index boundary (F3X; old paths keep working per BND-04).
pub use crate::index_port::FreshHnswReport;
pub use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
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
