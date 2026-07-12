#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile::MmapMut;
use dashmap::DashMap;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use portable_atomic::AtomicU128;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BinaryHeap;
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use twox_hash::XxHash64;

pub type NeighborVec = SmallVec<[u128; 32]>;

pub(crate) const ENTRY_POINT_NONE: u128 = u128::MAX;
pub(crate) const MAX_VEC_F32_LEN: usize = 10_000_000;

use super::distance::*;
pub use crate::node::{DistanceMetric, FilterBitset, SendPtr, VectorRepresentations};

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

pub(crate) const VECTOR_INDEX_VERSION: u16 = 6;

pub struct HnswNode {
    pub id: u128,
    pub bitset: FilterBitset,
    pub vec_data: VectorRepresentations,
    pub neighbors: Vec<NeighborVec>,
    pub storage_offset: u64,
    pub inv_cached_norm: f32,
    pub norm_sq: f32,
    pub flags: u32,
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
}

const fn default_flat_threshold() -> Option<usize> {
    Some(10000)
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 32,
            m_max0: 64,
            ef_construction: 400,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: Some(10000),
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

impl Ord for NodeSim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
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
        match other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            cmp => cmp,
        }
    }
}

pub struct CPIndex {
    pub nodes: DashMap<u128, HnswNode, BuildHasherDefault<XxHash64>>,
    pub max_layer: AtomicUsize,
    pub entry_point: AtomicU128,
    pub backend: IndexBackend,
    pub config: HnswConfig,
    pub total_nodes: AtomicU64,
    pub(crate) rng: parking_lot::Mutex<rand::rngs::StdRng>,
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
                VectorRepresentations::MmapFull(_, _) => {}
                VectorRepresentations::Binary(b) => total += b.len() * std::mem::size_of::<u64>(),
                VectorRepresentations::Turbo(t) => total += t.len(),
                VectorRepresentations::SQ8(d, _) => total += d.len() + 4,
                VectorRepresentations::None => {}
            }
            for layer in &node.neighbors {
                total +=
                    layer.len() * std::mem::size_of::<u128>() + std::mem::size_of::<NeighborVec>();
            }
            total += std::mem::size_of::<HnswNode>();
        }
        total += self.total_nodes.load(Ordering::Relaxed) as usize * 60;
        total
    }

    fn random_layer(&self) -> usize {
        let mut rng = self.rng.lock();
        let r: f64 = rng.random_range(0.0001..1.0);
        (-r.ln() * self.config.ml).floor() as usize
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
            .max_by_key(|kv| kv.value().neighbors.len())
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
            return true;
        }

        if vec_data.is_none() {
            self.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data: vec_data.clone(),
                    neighbors: vec![NeighborVec::new()],
                    storage_offset,
                    inv_cached_norm: 0.0,
                    norm_sq: 0.0,
                    flags: 0,
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
    ) {
        if self.validate_node(id, bitset.clone(), &vec_data, storage_offset) {
            return;
        }

        self.insert_hnsw(id, bitset, vec_data, storage_offset);
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
    ) {
        let level = self.random_layer();
        let ef_cons = self.config.ef_construction;

        let (inv_cached_norm, norm_sq) = self.compute_cached_norms(&vec_data);

        let query_f32 = vec_data.to_f32();

        let node = HnswNode {
            id,
            bitset,
            vec_data,
            neighbors: vec![NeighborVec::new(); level + 1],
            storage_offset,
            inv_cached_norm,
            norm_sq,
            flags: 0,
        };

        let ep = match self.get_entry_point() {
            None => {
                self.set_entry_point(id);
                self.max_layer.store(level, Ordering::Release);
                self.nodes.insert(id, node);
                self.total_nodes.fetch_add(1, Ordering::Relaxed);
                return;
            }
            Some(entry) => entry,
        };

        self.nodes.insert(id, node);
        self.total_nodes.fetch_add(1, Ordering::Relaxed);

        let query_f32 = match query_f32 {
            Some(v) => v,
            None => return,
        };

        let (query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                let norm = f32_l2_norm(&query_f32);
                if norm < f32::EPSILON {
                    self.nodes.remove(&id);
                    self.total_nodes.fetch_sub(1, Ordering::Relaxed);
                    return;
                }
                (Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(&query_f32);
                (Some(norm), None)
            }
        };

        let mut curr_entry_points = vec![ep];
        let top_layer = self.max_layer.load(Ordering::Acquire);

        for layer in (level + 1..=top_layer).rev() {
            let mut w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                &crate::node::ALL_BITSET,
                None,
                self.config.distance_metric,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            let w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                ef_cons,
                layer,
                &crate::node::ALL_BITSET,
                None,
                self.config.distance_metric,
            );

            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };

            curr_entry_points = w.iter().map(|ns| ns.1).collect();
            let selected_neighbors = self.select_neighbors(w, m_max);

            if let Some(mut n) = self.nodes.get_mut(&id) {
                n.neighbors[layer] = selected_neighbors.clone();
            }

            for &neighbor_id in &selected_neighbors {
                let (needs_shrink, current_neighbors) = {
                    if let Some(mut neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                        if layer < neighbor_node.neighbors.len() {
                            if !neighbor_node.neighbors[layer].contains(&id) {
                                neighbor_node.neighbors[layer].push(id);
                            }

                            if neighbor_node.neighbors[layer].len() > m_max {
                                (true, neighbor_node.neighbors[layer].clone())
                            } else {
                                (false, NeighborVec::new())
                            }
                        } else {
                            (false, NeighborVec::new())
                        }
                    } else {
                        (false, NeighborVec::new())
                    }
                };

                if needs_shrink {
                    self.shrink_neighbors(neighbor_id, m_max, &current_neighbors, layer);
                }
            }
        }

        self.update_metadata(level, id);
    }

    #[inline]
    fn shrink_neighbors(
        &self,
        neighbor_id: u128,
        m_max: usize,
        current_neighbors: &[u128],
        layer: usize,
    ) {
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
            let pruned = self.select_neighbors(cand_heap, m_max);
            if let Some(mut neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                neighbor_node.neighbors[layer] = pruned;
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
                if let Some(node) = self.nodes.get(&node_id) {
                    for layer in (0..node.neighbors.len()).rev() {
                        for &neighbor_id in &node.neighbors[layer] {
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
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::ALL_BITSET;

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
            index.add(
                i as u128,
                FilterBitset::new(),
                VectorRepresentations::Full(v.clone()),
                0,
            );
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
}
