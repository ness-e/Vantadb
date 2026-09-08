//! CPIndex: HNSW core graph (construct, insert, neighbors, maintenance).
//! Split from graph.rs (FIND-48) — re-exported via graph/mod.rs.

use super::types::*;
use crate::index::distance::f32_l2_norm;
use crate::index::distance::*;
use crate::index::search::SearchProfile;
use ahash::RandomState;
use dashmap::DashMap;
use portable_atomic::AtomicU128;
use rand::{Rng, SeedableRng};
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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
    /// Lazy-built SCANN (SQ8) index. `None` until first search with
    /// `config.index_type == IndexType::Scann` or a per-search method override.
    pub scann_index: parking_lot::Mutex<Option<crate::index::scann::ScannIndex>>,
    /// Node count the cached `scann_index` was built over. Rebuilt whenever
    /// `nodes.len()` diverges (same staleness rule as `ivf_built_at_node_count`).
    pub scann_built_at_node_count: AtomicUsize,
    /// Flat, lock-friendly neighbor list index.
    /// `pub(crate)` because `HnswNeighborIndex` is only `pub(crate)`.
    pub(crate) neighbor_index: crate::index::neighbor_index::HnswNeighborIndex,
}
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
            scann_index: parking_lot::Mutex::new(None),
            scann_built_at_node_count: AtomicUsize::new(0),
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

    pub(crate) fn random_layer(&self) -> usize {
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

    /// Remove a node from the graph (ERR-012).
    ///
    /// Calls `neighbor_index.remove_node(id)`, so the deletion is NOT a leak:
    /// the node's outbound lists leave the index and every neighbor's
    /// `inbound` counter is decremented, plus all inbound references to the
    /// deleted id are scrubbed from other nodes' lists. Mirrors the HNSW
    /// removal that `StorageEngine` performs on delete; the engine must call
    /// this (instead of only `nodes.remove`) to keep the neighbor index
    /// consistent.
    pub fn remove_node(&self, id: u128) {
        if self.nodes.remove(&id).is_none() {
            tracing::debug!(node = id, "remove_node: id not found in graph");
        }
        // PERF-23: promote a replacement entry point if we just removed the
        // current one — same logic as `StorageEngine::remove_hnsw_entry`.
        if self.entry_point.load(Ordering::Relaxed) == id {
            let new_ep = self.find_new_entry_point().unwrap_or(ENTRY_POINT_NONE);
            self.entry_point.store(new_ep, Ordering::Relaxed);
        }
        self.neighbor_index.remove_node(id);
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
            // Binary (and any other non-F32-indexable, non-None) vectors must
            // remain retrievable by get() (insert→get contract) even though
            // they are not part of the similarity graph. Mirror the
            // vec_data.is_none() branch of validate_node: allocate, insert the
            // HnswNode with empty neighbor lists, count it — but do not touch
            // the entry point or any HNSW layers. `vec_data` can only reach
            // here when validate_node returned false (i.e. non-None), the
            // is_none() guard is defensive only.
            None => {
                if !vec_data.is_none() {
                    self.neighbor_index.allocate(id, 1);
                    self.nodes.insert(
                        id,
                        HnswNode {
                            id,
                            bitset,
                            vec_data,
                            storage_offset,
                            inv_cached_norm,
                            norm_sq,
                            flags: 0,
                            neighbor_lists: Vec::new(),
                        },
                    );
                    self.total_nodes.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(());
            }
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
            // Binary (and any other non-F32-indexable, non-None) vectors must
            // remain retrievable by get() (insert→get contract) even though
            // they are not part of the similarity graph. Mirror the
            // vec_data.is_none() branch of validate_node: allocate, insert the
            // HnswNode with empty neighbor lists, count it — but do not touch
            // the entry point or any HNSW layers. `vec_data` can only reach
            // here when validate_node returned false (i.e. non-None), the
            // is_none() guard is defensive only.
            None => {
                if !vec_data.is_none() {
                    self.neighbor_index.allocate(id, 1);
                    self.nodes.insert(
                        id,
                        HnswNode {
                            id,
                            bitset,
                            vec_data,
                            storage_offset,
                            inv_cached_norm,
                            norm_sq,
                            flags: 0,
                            neighbor_lists: Vec::new(),
                        },
                    );
                    self.total_nodes.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(());
            }
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
    pub(crate) fn shrink_neighbors(
        &self,
        neighbor_id: u128,
        m_max: usize,
        current_neighbors: &[u128],
        layer: usize,
    ) {
        if let Some(pruned) = self.compute_shrunk_neighbors(neighbor_id, m_max, current_neighbors) {
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

    /// ERR-043: select the survivors list for a saturated neighbor list.
    ///
    /// The source node's vector is read once as a borrowed slice instead of
    /// cloning it per shrink (`as_f32_slice().map(|s| s.to_vec())` was an
    /// O(vec_len) alloc). The DashMap guard lives only inside this helper, so
    /// the caller's subsequent `get_mut` on the same id never contends with it.
    #[inline]
    fn compute_shrunk_neighbors(
        &self,
        neighbor_id: u128,
        m_max: usize,
        current_neighbors: &[u128],
    ) -> Option<NeighborVec> {
        let n = self.nodes.get(&neighbor_id)?;
        let nb_v = n.vec_data.as_f32_slice()?;
        let nb_inv_norm = n.inv_cached_norm;

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
                    nb_v,
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
        Some(self.select_neighbors(cand_heap, m_max, |cand| {
            self.neighbor_index.inbound_count(cand) <= 1
        }))
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
                        // ERR-045: borrow the list instead of cloning it. This
                        // BFS walks every node's lists during compaction, so
                        // get_neighbors() here would be O(N×M) allocations.
                        if let Some(neighbors) =
                            self.neighbor_index.get_neighbors_ref(node_id, layer)
                        {
                            for &neighbor_id in neighbors.iter() {
                                if seen.insert(neighbor_id) {
                                    queue.push_back(neighbor_id);
                                }
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
        let start = web_time::Instant::now();

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
        // target ID exists in active_nodes. Keep the node's inline
        // `neighbor_lists` cache in sync: `search_layer` (including the
        // ACORN second-hop expansion) reads the inline cache first and falls
        // back to `neighbor_index` only when it is empty, so a stale inline
        // list would keep walking the just-removed orphan edges (ERR-020).
        for (node_id, layer) in &to_repair {
            self.neighbor_index
                .retain_neighbors(*node_id, *layer, |nid| active_nodes.contains(nid));
            if let Some(repaired) = self.neighbor_index.get_neighbors(*node_id, *layer) {
                if let Some(mut node_ref) = self.nodes.get_mut(node_id) {
                    if node_ref.neighbor_lists.len() > *layer {
                        node_ref.neighbor_lists[*layer] = repaired;
                    }
                }
            }
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
