use memmap2::MmapMut;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub use crate::node::VectorRepresentations;
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

const VECTOR_INDEX_MAGIC: &[u8; 8] = b"VNTHNSW1";
const VECTOR_INDEX_VERSION: u32 = 2; // Upgraded for config support

#[inline(always)]
pub fn cosine_sim_f32(a: &[f32], b: &[f32]) -> f32 {
    use crate::hardware::{HardwareCapabilities, InstructionSet};
    let caps = HardwareCapabilities::global();
    match caps.instructions {
        InstructionSet::Fallback => {
            let mut dot: f32 = 0.0;
            let mut norm_a: f32 = 0.0;
            let mut norm_b: f32 = 0.0;
            for (va, vb) in a.iter().zip(b.iter()) {
                dot += va * vb;
                norm_a += va * va;
                norm_b += vb * vb;
            }
            let denom = norm_a.sqrt() * norm_b.sqrt();
            if denom < f32::EPSILON {
                0.0
            } else {
                dot / denom
            }
        }
        _ => {
            use wide::f32x8;
            let mut dot_v = f32x8::ZERO;
            let mut norm_a_v = f32x8::ZERO;
            let mut norm_b_v = f32x8::ZERO;
            let chunks_a = a.chunks_exact(8);
            let chunks_b = b.chunks_exact(8);
            let rem_a = chunks_a.remainder();
            let rem_b = chunks_b.remainder();
            for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
                let va = f32x8::from([
                    a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5],
                    a_chunk[6], a_chunk[7],
                ]);
                let vb = f32x8::from([
                    b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5],
                    b_chunk[6], b_chunk[7],
                ]);
                dot_v += va * vb;
                norm_a_v += va * va;
                norm_b_v += vb * vb;
            }
            let mut dot = dot_v.reduce_add();
            let mut norm_a = norm_a_v.reduce_add();
            let mut norm_b = norm_b_v.reduce_add();
            for i in 0..rem_a.len() {
                dot += rem_a[i] * rem_b[i];
                norm_a += rem_a[i] * rem_a[i];
                norm_b += rem_b[i] * rem_b[i];
            }
            let denom = norm_a.sqrt() * norm_b.sqrt();
            if denom < f32::EPSILON {
                0.0
            } else {
                dot / denom
            }
        }
    }
}

pub fn calculate_similarity(
    raw_query: &[f32],
    quantized_query_1bit: Option<&[u64]>,
    quantized_query_3bit: Option<(&[u8], f32)>,
    node_vec: &VectorRepresentations,
) -> f32 {
    match node_vec {
        VectorRepresentations::Binary(b) => {
            if let Some(q1) = quantized_query_1bit {
                rabitq_similarity(q1, b)
            } else {
                0.0
            }
        }
        VectorRepresentations::Turbo(t) => {
            if let Some((q3, max_abs)) = quantized_query_3bit {
                turbo_quant_similarity(q3, max_abs, t, 1.0)
            } else {
                0.0
            }
        }
        VectorRepresentations::Full(f) => {
            // ZERO ALLOCATION: Cálculo SIMD directo sin empaquetar ni clonar
            cosine_sim_f32(raw_query, f)
        }
        VectorRepresentations::None => 0.0,
    }
}

pub struct HnswNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: VectorRepresentations,
    pub neighbors: Vec<Vec<u64>>,
    /// Offset into the VantaFile (Phase 3)
    pub storage_offset: u64,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub m_max0: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f64,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        }
    }
}

// Custom wrapper to store (similarity, node_id) in BinaryHeap (Max-Heap)
#[derive(Clone, PartialEq, Debug)]
struct NodeSim(f32, u64);

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

// Wrapper for Min-Heap (used to track closest in result set)
#[derive(Clone, PartialEq, Debug)]
struct NodeSimMin(f32, u64);

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
    pub nodes: HashMap<u64, HnswNode>,
    pub max_layer: usize,
    pub entry_point: Option<u64>,
    pub backend: IndexBackend,
    pub config: HnswConfig,
    rng: rand::rngs::StdRng,
}

impl CPIndex {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    pub fn new_with_config(config: HnswConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend: IndexBackend::InMemory,
            config,
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    pub fn with_backend(backend: IndexBackend) -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend,
            config: HnswConfig::default(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    fn random_layer(&mut self) -> usize {
        let r: f64 = self.rng.gen_range(0.0001..1.0);
        (-r.ln() * self.config.ml).floor() as usize
    }

    /// Primary search subroutine for HNSW.
    /// Performs a greedy beam search to return the `ef` nearest neighbors
    /// found at `layer`. Candidates are validated against `query_mask`.
    fn search_layer(
        &self,
        query_vec: &[f32],
        entry_points: &[u64],
        ef: usize,
        layer: usize,
        query_mask: u128,
        vector_store: Option<&crate::storage::VantaFile>,
    ) -> BinaryHeap<NodeSimMin> {
        let mut visited = std::collections::HashSet::new();
        let mut candidates = BinaryHeap::new(); // Max-heap: candidates to visit
        let mut results = BinaryHeap::new(); // Min-heap: best `ef` bounds

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                let d = if let Some(vs) = vector_store {
                    // Zero-copy search from VantaFile
                    if let Some(header) = vs.read_header(node.storage_offset) {
                        let vec_start = header.vector_offset as usize;
                        let vec_end = vec_start + (header.vector_len as usize * 4);
                        let vec_data = &vs.mmap[vec_start..vec_end];
                        // Safety: we trust the header.vector_len and bounds checking above
                        let f32_vec: &[f32] = unsafe {
                            std::slice::from_raw_parts(
                                vec_data.as_ptr() as *const f32,
                                header.vector_len as usize,
                            )
                        };
                        cosine_sim_f32(query_vec, f32_vec)
                    } else {
                        0.0
                    }
                } else {
                    calculate_similarity(query_vec, None, None, &node.vec_data)
                };

                candidates.push(NodeSim(d, ep));

                if query_mask == u128::MAX || (node.bitset & query_mask) == query_mask {
                    results.push(NodeSimMin(d, ep));
                }
                visited.insert(ep);
            }
        }

        while let Some(NodeSim(d_cand, cand_id)) = candidates.pop() {
            // Early stopping condition: if candidate is worse than the worst result
            if results.len() >= ef {
                if let Some(worst) = results.peek() {
                    // Because it's a min-heap, peek gives the smallest similarity (worst match)
                    if d_cand < worst.0 {
                        break;
                    }
                }
            }

            if let Some(node) = self.nodes.get(&cand_id) {
                if layer < node.neighbors.len() {
                    for &neighbor_id in &node.neighbors[layer] {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);

                            if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                                let d = if let Some(vs) = vector_store {
                                    if let Some(h) = vs.read_header(neighbor.storage_offset) {
                                        let v_data = &vs.mmap[h.vector_offset as usize
                                            ..(h.vector_offset as usize
                                                + h.vector_len as usize * 4)];
                                        // Safety: trusted bounds and aligned data
                                        let f32_v: &[f32] = unsafe {
                                            std::slice::from_raw_parts(
                                                v_data.as_ptr() as *const f32,
                                                h.vector_len as usize,
                                            )
                                        };
                                        cosine_sim_f32(query_vec, f32_v)
                                    } else {
                                        0.0
                                    }
                                } else {
                                    calculate_similarity(query_vec, None, None, &neighbor.vec_data)
                                };

                                if results.len() < ef
                                    || (results.peek().is_some() && d > results.peek().unwrap().0)
                                {
                                    candidates.push(NodeSim(d, neighbor_id));

                                    if query_mask == u128::MAX
                                        || (neighbor.bitset & query_mask) == query_mask
                                    {
                                        results.push(NodeSimMin(d, neighbor_id));
                                        if results.len() > ef {
                                            results.pop(); // Remove the worst to keep size at `ef`
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Select neighbors using the HNSW paper heuristic (Algorithm 4, Malkov & Yashunin 2018).
    /// Applies spatial diversity from slot 0 — no unconditional acceptance.
    /// keepPrunedConnections=true: fills limited remaining slots with discarded candidates.
    ///
    /// Metric: cosine similarity (higher = closer). The diversity condition is:
    ///   reject if similarity(candidate, selected) > similarity(candidate, query)
    /// This is the correct inversion of the paper's distance-based condition
    /// because cosine similarity is monotonically inverse to angular distance.
    fn select_neighbors(&self, candidates: &mut BinaryHeap<NodeSimMin>, m: usize) -> Vec<u64> {
        // Clone is critically necessary here because `w` is reused by the caller
        // to seed the `entry_points` for the next layer down.
        let sorted = candidates.clone().into_sorted_vec();
        // into_sorted_vec returns ascending order based on NodeSimMin's Ord
        // NodeSimMin Ord equates higher similarity to "Less", meaning best candidates come first!

        let mut selected: Vec<u64> = Vec::with_capacity(m);
        let mut discarded: Vec<u64> = Vec::new();

        for ns in sorted.into_iter() {
            if selected.len() >= m {
                break;
            }

            let cand_id = ns.1;
            let sim_q_cand = ns.0;

            let cand_slice = match self.nodes.get(&cand_id).map(|n| &n.vec_data) {
                Some(VectorRepresentations::Full(v)) => v.as_slice(),
                _ => {
                    selected.push(cand_id);
                    continue;
                }
            };

            let mut is_diverse = true;
            for &sel_id in &selected {
                if let Some(sel_node) = self.nodes.get(&sel_id) {
                    let sim_cand_sel =
                        calculate_similarity(cand_slice, None, None, &sel_node.vec_data);
                    if sim_cand_sel > sim_q_cand {
                        is_diverse = false;
                        break;
                    }
                }
            }

            if is_diverse {
                selected.push(cand_id);
            } else {
                discarded.push(cand_id);
            }
        }

        // keepPrunedConnections: fill remaining slots with discarded candidates.
        // HNSW relies on keeping degree close to M.
        for &disc_id in discarded.iter() {
            if selected.len() >= m {
                break;
            }
            selected.push(disc_id);
        }

        selected
    }

    pub fn add(
        &mut self,
        id: u64,
        bitset: u128,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) {
        // Phase 1.3: Duplicate detection — silently updating an existing node can
        // corrupt the graph's bidirectional links. Log a warning and return early.
        if let Some(node) = self.nodes.get_mut(&id) {
            node.storage_offset = storage_offset;
            node.bitset = bitset;
            // Note: We don't update graph links here to avoid corruption,
            // but updating the offset ensures we point to the latest version.
            return;
        }

        if vec_data.is_none() {
            // Can't index graph nodes without vectors into HNSW layers,
            // but we must still register them in the nodes map to track their storage_offset.
            self.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors: vec![Vec::new()],
                    storage_offset,
                },
            );
            return;
        }

        let level = self.random_layer();
        let ef_cons = self.config.ef_construction;

        // If index is empty
        let ep = match self.entry_point {
            None => {
                self.entry_point = Some(id);
                self.max_layer = level;
                let neighbors = vec![Vec::new(); level + 1];
                self.nodes.insert(
                    id,
                    HnswNode {
                        id,
                        bitset,
                        vec_data,
                        neighbors,
                        storage_offset,
                    },
                );
                return;
            }
            Some(entry) => entry,
        };

        // Query vector as F32 for building the index properly
        let query_f32 = match vec_data.to_f32() {
            Some(v) => v,
            None => return, // Critical failure, vector decode failed
        };

        let mut curr_entry_points = vec![ep];
        let top_layer = self.max_layer;

        // Phase 1: Descend down to the new node's insertion level (or top_layer)
        for layer in (level + 1..=top_layer).rev() {
            let mut w =
                self.search_layer(&query_f32, &curr_entry_points, 1, layer, u128::MAX, None);
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let mut new_neighbors = vec![Vec::new(); level + 1];

        // Phase 2: From node's layer down to 0, find neighbors and connect
        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            let w = self.search_layer(
                &query_f32,
                &curr_entry_points,
                ef_cons,
                layer,
                u128::MAX,
                None,
            );

            // extendCandidates: expand W with the neighbors of its elements
            let mut extended_w = w.clone();
            let mut visited_ext: std::collections::HashSet<u64> = std::collections::HashSet::new();
            for item in w.iter() {
                visited_ext.insert(item.1);
            }

            // Only extend if it does not blow up the search scope pathologically
            if extended_w.len() <= ef_cons {
                for item in w.iter() {
                    if let Some(c_node) = self.nodes.get(&item.1) {
                        if layer < c_node.neighbors.len() {
                            for &adj_id in &c_node.neighbors[layer] {
                                if !visited_ext.contains(&adj_id) {
                                    visited_ext.insert(adj_id);
                                    if let Some(adj_node) = self.nodes.get(&adj_id) {
                                        let sim = calculate_similarity(
                                            &query_f32,
                                            None,
                                            None,
                                            &adj_node.vec_data,
                                        );
                                        extended_w.push(NodeSimMin(sim, adj_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Extract the neighbors to connect (bidirectionally)
            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };
            let selected_neighbors = self.select_neighbors(&mut extended_w, self.config.m);
            new_neighbors[layer] = selected_neighbors.clone();

            // Entry points for next layer = full search results from this layer
            // (select_neighbors clones w internally, so w is still intact here)
            curr_entry_points = w.into_iter().map(|ns| ns.1).collect();

            // Bidirectional links
            for &neighbor_id in &selected_neighbors {
                // Scope mutable access to avoid overlap with immutable `self.nodes.get(&nt)`
                let (needs_shrink, current_neighbors) = {
                    if let Some(neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                        if layer < neighbor_node.neighbors.len() {
                            if !neighbor_node.neighbors[layer].contains(&id) {
                                neighbor_node.neighbors[layer].push(id);
                            }

                            // Shrink connections if they overflow M_max
                            if neighbor_node.neighbors[layer].len() > m_max {
                                (true, neighbor_node.neighbors[layer].clone())
                            } else {
                                (false, Vec::new())
                            }
                        } else {
                            (false, Vec::new())
                        }
                    } else {
                        (false, Vec::new())
                    }
                };

                if needs_shrink {
                    // Zero-Copy Extractor for Pruning
                    let nb_vec = match self.nodes.get(&neighbor_id).map(|n| &n.vec_data) {
                        Some(VectorRepresentations::Full(v)) => Some(v.as_slice()),
                        _ => None,
                    };

                    if let Some(nb_v) = nb_vec {
                        let mut cand_heap = BinaryHeap::new();
                        for &n_target in &current_neighbors {
                            if let Some(nt) = self.nodes.get(&n_target) {
                                let d = calculate_similarity(nb_v, None, None, &nt.vec_data);
                                cand_heap.push(NodeSimMin(d, n_target));
                            }
                        }
                        let pruned = self.select_neighbors(&mut cand_heap, m_max);
                        if let Some(neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                            neighbor_node.neighbors[layer] = pruned;
                        }
                    }
                }
            }
        }

        // Apply new node explicitly
        self.nodes.insert(
            id,
            HnswNode {
                id,
                bitset,
                vec_data,
                neighbors: new_neighbors,
                storage_offset,
            },
        );

        // Update entry point if we created a new highest layer
        if level > self.max_layer {
            self.max_layer = level;
            self.entry_point = Some(id);
        }
    }

    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        _q_1bit: Option<&[u64]>, // We let these pass but currently default to calculate_similarity internal handler
        _q_3bit: Option<(&[u8], f32)>,
        query_mask: u128,
        top_k: usize,
        vector_store: Option<&crate::storage::VantaFile>,
    ) -> Vec<(u64, f32)> {
        let ep = match self.entry_point {
            Some(id) => id,
            None => return Vec::new(),
        };

        let ef_search = (self.config.ef_search * 2).max(top_k);
        let mut curr_entry_points = vec![ep];

        for layer in (1..=self.max_layer).rev() {
            let mut w = self.search_layer(
                query_vec,
                &curr_entry_points,
                1,
                layer,
                u128::MAX,
                vector_store,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let w = self.search_layer(
            query_vec,
            &curr_entry_points,
            ef_search,
            0,
            query_mask,
            vector_store,
        );

        // Extract top-k
        let mut result = w.into_sorted_vec();

        // into_sorted_vec returns highest similarity (best) first!
        result.truncate(top_k);

        result.into_iter().map(|n| (n.1, n.0)).collect()
    }

    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.nodes.len() * 256 + 128);

        buf.extend_from_slice(VECTOR_INDEX_MAGIC);
        buf.extend_from_slice(&VECTOR_INDEX_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.max_layer as u64).to_le_bytes());

        // Config block (only in V2+)
        buf.extend_from_slice(&(self.config.m as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.m_max0 as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_construction as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_search as u64).to_le_bytes());
        buf.extend_from_slice(&self.config.ml.to_le_bytes());

        match self.entry_point {
            Some(ep) => {
                buf.push(1);
                buf.extend_from_slice(&ep.to_le_bytes());
            }
            None => {
                buf.push(0);
                buf.extend_from_slice(&0u64.to_le_bytes());
            }
        }

        let node_count = self.nodes.len() as u64;
        buf.extend_from_slice(&node_count.to_le_bytes());

        for node in self.nodes.values() {
            buf.extend_from_slice(&node.id.to_le_bytes());
            buf.extend_from_slice(&node.bitset.to_le_bytes());
            buf.extend_from_slice(&node.storage_offset.to_le_bytes());

            match &node.vec_data {
                VectorRepresentations::Full(f) => {
                    buf.push(1);
                    buf.extend_from_slice(&(f.len() as u64).to_le_bytes());
                    for &val in f {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Binary(b) => {
                    buf.push(2);
                    buf.extend_from_slice(&(b.len() as u64).to_le_bytes());
                    for &val in b {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Turbo(t) => {
                    buf.push(3);
                    buf.extend_from_slice(&(t.len() as u64).to_le_bytes());
                    buf.extend_from_slice(t);
                }
                VectorRepresentations::None => {
                    buf.push(0);
                    buf.extend_from_slice(&0u64.to_le_bytes());
                }
            }

            let layer_count = node.neighbors.len() as u64;
            buf.extend_from_slice(&layer_count.to_le_bytes());
            for layer in &node.neighbors {
                let neighbor_count = layer.len() as u64;
                buf.extend_from_slice(&neighbor_count.to_le_bytes());
                for &nid in layer {
                    buf.extend_from_slice(&nid.to_le_bytes());
                }
            }
        }

        buf
    }

    pub fn deserialize_from_bytes(data: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        if data.len() < 29 {
            return Err(Error::new(ErrorKind::InvalidData, "Index file too small"));
        }

        let mut pos = 0;

        if &data[pos..pos + 8] != VECTOR_INDEX_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic header"));
        }
        pos += 8;

        let version = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;

        let max_layer = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let mut config = HnswConfig::default();
        if version >= 2 {
            config.m = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.m_max0 = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ef_construction =
                u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ef_search = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ml = f64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;
        }

        if pos >= data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated EP"));
        }
        let ep_exists = data[pos];
        pos += 1;
        if pos + 8 > data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated EP ID"));
        }
        let ep_id = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let entry_point = if ep_exists == 1 { Some(ep_id) } else { None };

        if pos + 8 > data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node count"));
        }
        let node_count = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let mut nodes = HashMap::with_capacity(node_count);

        for _ in 0..node_count {
            if pos + 8 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node id"));
            }
            let id = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;

            if pos + 16 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated bitset"));
            }
            let bitset = u128::from_le_bytes(data[pos..pos + 16].try_into().unwrap());
            pos += 16;

            if pos + 8 > data.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Truncated storage offset",
                ));
            }
            let storage_offset = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;

            if pos + 1 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec type"));
            }
            let vec_type = data[pos];
            pos += 1;

            if pos + 8 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec len"));
            }
            let vec_len = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            let vec_data = match vec_type {
                1 => {
                    let byte_len = vec_len * 4;
                    if pos + byte_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated f32 vec"));
                    }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 4;
                        v.push(f32::from_le_bytes(
                            data[start..start + 4].try_into().unwrap(),
                        ));
                    }
                    pos += byte_len;
                    VectorRepresentations::Full(v)
                }
                2 => {
                    let byte_len = vec_len * 8;
                    if pos + byte_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated binary vec"));
                    }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 8;
                        v.push(u64::from_le_bytes(
                            data[start..start + 8].try_into().unwrap(),
                        ));
                    }
                    pos += byte_len;
                    VectorRepresentations::Binary(v.into_boxed_slice())
                }
                3 => {
                    if pos + vec_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated turbo vec"));
                    }
                    let v = data[pos..pos + vec_len].to_vec();
                    pos += vec_len;
                    VectorRepresentations::Turbo(v.into_boxed_slice())
                }
                _ => VectorRepresentations::None,
            };

            if pos + 8 > data.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Truncated neighbor layers",
                ));
            }
            let layer_count = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            let mut neighbors = Vec::with_capacity(layer_count);
            for _ in 0..layer_count {
                if pos + 8 > data.len() {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Truncated neighbor count",
                    ));
                }
                let neighbor_count =
                    u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
                pos += 8;

                let byte_len = neighbor_count * 8;
                if pos + byte_len > data.len() {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Truncated neighbor ids",
                    ));
                }
                let mut layer_neighbors = Vec::with_capacity(neighbor_count);
                for i in 0..neighbor_count {
                    let start = pos + i * 8;
                    layer_neighbors.push(u64::from_le_bytes(
                        data[start..start + 8].try_into().unwrap(),
                    ));
                }
                pos += byte_len;
                neighbors.push(layer_neighbors);
            }

            nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors,
                    storage_offset,
                },
            );
        }

        Ok(Self {
            nodes,
            max_layer,
            entry_point,
            backend: IndexBackend::InMemory,
            config,
            rng: rand::rngs::StdRng::seed_from_u64(42),
        })
    }

    pub fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        let data = self.serialize_to_bytes();
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        writer.flush()?;
        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW index persisted");
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let mmap = match unsafe { memmap2::MmapOptions::new().map(&file) } {
            Ok(m) => m,
            Err(e) => {
                warn!(err = %e, "Failed to mmap HNSW index file — will rebuild");
                return None;
            }
        };

        match Self::deserialize_from_bytes(&mmap) {
            Ok(index) => {
                info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded index from file");
                // Phase 1.3: Validate integrity on every cold-start
                if let Err(violations) = index.validate_index() {
                    warn!(
                        violation_count = violations.len(),
                        "HNSW index has integrity violations after deserialization"
                    );
                    for v in &violations[..violations.len().min(5)] {
                        warn!(violation = %v, "HNSW integrity violation");
                    }
                }
                Some(index)
            }
            Err(e) => {
                warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                None
            }
        }
    }

    pub fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        let path = match &self.backend {
            IndexBackend::MMapFile { path, .. } => path.clone(),
            _ => return Ok(()),
        };

        let data = self.serialize_to_bytes();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.set_len(data.len() as u64)?;

        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;

        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(mapped);
        }

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW MMap synced");
        Ok(())
    }
}

// ─── Phase 1.1: Index Stats & Integrity Validation ──────────────────────────

/// Snapshot of HNSW index health metrics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total nodes in the index
    pub node_count: usize,
    /// Maximum layer height in the graph
    pub max_layer: usize,
    /// Nodes with zero neighbors on layer 0 (potential orphans)
    pub orphan_count: usize,
    /// Average outgoing connections on layer 0
    pub avg_connections_l0: f32,
    /// Total number of graph integrity violations found
    pub violation_count: usize,
}

impl CPIndex {
    /// Compute a snapshot of index health metrics.
    pub fn stats(&self) -> IndexStats {
        let node_count = self.nodes.len();
        let orphan_count = self
            .nodes
            .values()
            .filter(|n| n.neighbors.is_empty() || n.neighbors[0].is_empty())
            .count();
        let total_l0_connections: usize = self
            .nodes
            .values()
            .map(|n| n.neighbors.first().map(|l| l.len()).unwrap_or(0))
            .sum();
        let avg_connections_l0 = if node_count > 0 {
            total_l0_connections as f32 / node_count as f32
        } else {
            0.0
        };

        IndexStats {
            node_count,
            max_layer: self.max_layer,
            orphan_count,
            avg_connections_l0,
            violation_count: 0, // Updated by validate_index()
        }
    }

    /// Validate the structural integrity of the HNSW graph.
    ///
    /// Checks:
    /// 1. Every neighbor reference points to an existing node
    /// 2. No self-loops
    /// 3. Layer count is consistent with node's reported level
    ///
    /// Returns `Ok(())` if the graph is clean, or a list of violation messages.
    ///
    /// # Performance
    /// O(N × M) where N = node count, M = max neighbors per layer.
    /// Run at startup after deserialization, not in hot paths.
    pub fn validate_index(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        for (id, node) in &self.nodes {
            // Check: layer count should be ≥ 1
            if node.neighbors.is_empty() {
                violations.push(format!(
                    "Node {} has empty neighbors array (expected ≥1 layer)",
                    id
                ));
                continue;
            }

            // Check each layer's neighbor list
            for (layer_idx, layer) in node.neighbors.iter().enumerate() {
                for &neighbor_id in layer {
                    // Self-loop check
                    if neighbor_id == *id {
                        violations.push(format!(
                            "Node {} has a self-loop at layer {}",
                            id, layer_idx
                        ));
                        continue;
                    }
                    // Dangling reference check
                    if !self.nodes.contains_key(&neighbor_id) {
                        violations.push(format!(
                            "Node {} references non-existent neighbor {} at layer {}",
                            id, neighbor_id, layer_idx
                        ));
                    }
                }
            }
        }

        // Check entry point validity
        if let Some(ep) = self.entry_point {
            if !self.nodes.contains_key(&ep) {
                violations.push(format!("Entry point {} does not exist in the node map", ep));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}
