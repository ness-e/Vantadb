use rand::Rng;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use memmap2::MmapMut;

// Reutilizamos la lógica SIMD centralizada en node.rs
pub use crate::node::VectorRepresentations;
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

// ─── Magic Header for neural_index.bin ────────────────────────────
const NEURAL_INDEX_MAGIC: &[u8; 8] = b"CXHNSW01";
const NEURAL_INDEX_VERSION: u32 = 1;

/// Hybrid Similarity Routing
/// Routes the similarity calculation based on the Node's vector representation.
pub fn calculate_similarity(
    raw_query: &[f32], 
    quantized_query_1bit: Option<&[u64]>,
    quantized_query_3bit: Option<(&[u8], f32)>, 
    node_vec: &VectorRepresentations
) -> f32 {
    match node_vec {
        VectorRepresentations::Binary(b) => {
            if let Some(q1) = quantized_query_1bit {
                rabitq_similarity(q1, b)
            } else {
                0.0 // Fast fallback if query isn't pre-quantized
            }
        },
        VectorRepresentations::Turbo(t) => {
            if let Some((q3, max_abs)) = quantized_query_3bit {
                turbo_quant_similarity(q3, max_abs, t, 1.0) 
            } else {
                0.0
            }
        },
        VectorRepresentations::Full(f) => {
            // Direct F32 fallback
            let va = VectorRepresentations::Full(raw_query.to_vec());
            let vb = VectorRepresentations::Full(f.to_vec());
            va.cosine_similarity(&vb).unwrap_or(0.0)
        },
        VectorRepresentations::None => 0.0,
    }
}

/// Simplified HNSW node with embedded filter and multi-layer neighbors
pub struct HnswNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: VectorRepresentations,
    /// Vec of layers, where each layer contains a list of neighbor IDs
    pub neighbors: Vec<Vec<u64>>,
}

// ─── Index Backend ────────────────────────────────────────────────

/// Determines where the HNSW graph data resides
#[derive(Debug)]
pub enum IndexBackend {
    /// Standard heap-allocated storage (fast, RAM-intensive)
    InMemory,
    /// Memory-mapped file backend (low RAM, disk-backed)
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

/// HNSW Co-located Pre-filter Index (CP-Index)
pub struct CPIndex {
    pub nodes: HashMap<u64, HnswNode>,
    pub max_layer: usize,
    pub entry_point: Option<u64>,
    pub backend: IndexBackend,
}

impl CPIndex {
    pub fn new() -> Self {
        Self { 
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend: IndexBackend::InMemory,
        }
    }

    pub fn with_backend(backend: IndexBackend) -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend,
        }
    }

    fn random_layer() -> usize {
        // Simplified probabilistic layer assignment (-ln(U) * mL)
        let mut rng = rand::thread_rng();
        let mut layer = 0;
        while rng.gen_bool(0.5) && layer < 4 { // Max 5 layers for MVP
            layer += 1;
        }
        layer
    }

    pub fn add(&mut self, id: u64, bitset: u128, vec_data: VectorRepresentations) {
        if vec_data.is_none() {
            return;
        }

        let level = Self::random_layer();
        let mut neighbors = vec![Vec::new(); level + 1];

        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_layer = level;
        } else {
            // MVP: Just fully connect to entry point across valid layers to maintain navigation.
            let ep = self.entry_point.unwrap();
            for l in 0..=level {
                if l <= self.max_layer {
                    neighbors[l].push(ep);
                    if let Some(ep_node) = self.nodes.get_mut(&ep) {
                        if l < ep_node.neighbors.len() {
                            ep_node.neighbors[l].push(id);
                        }
                    }
                }
            }
            if level > self.max_layer {
                self.entry_point = Some(id);
                self.max_layer = level;
            }
        }

        self.nodes.insert(id, HnswNode {
            id,
            bitset,
            vec_data,
            neighbors,
        });
    }

    /// HNSW Greedy Search
    pub fn search_nearest(
        &self, 
        query_vec: &[f32], 
        q_1bit: Option<&[u64]>, 
        q_3bit: Option<(&[u8], f32)>, 
        query_mask: u128, 
        top_k: usize
    ) -> Vec<(u64, f32)> {
        let mut curr_node_id = match self.entry_point {
            Some(id) => id,
            None => return Vec::new(),
        };

        // Phase 1: Descend layers from max_layer down to 1
        for layer in (1..=self.max_layer).rev() {
            curr_node_id = self.greedy_step(curr_node_id, query_vec, q_1bit, q_3bit, layer);
        }

        // Phase 2: Greedy local search at layer 0 (Topological navigation)
        let mut visited = std::collections::HashSet::new();
        let mut candidates = std::collections::BinaryHeap::new();
        
        // Use custom wrapper to store (similarity, node_id) in BinaryHeap (Max-Heap)
        #[derive(PartialEq)]
        struct NodeSim(f32, u64);
        impl Eq for NodeSim {}
        impl PartialOrd for NodeSim {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
        impl Ord for NodeSim {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                match self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal) {
                    std::cmp::Ordering::Equal => other.1.cmp(&self.1),
                    cmp => cmp,
                }
            }
        }

        // Add start point for layer 0
        if let Some(node) = self.nodes.get(&curr_node_id) {
            let sim = calculate_similarity(query_vec, q_1bit, q_3bit, &node.vec_data);
            candidates.push(NodeSim(sim, curr_node_id));
            visited.insert(curr_node_id);
        }

        let mut neighborhood_results = Vec::new();

        while let Some(NodeSim(sim, id)) = candidates.pop() {
            // Only include in results if the node passes the bitset filter
            if let Some(node) = self.nodes.get(&id) {
                if node.bitset & query_mask == query_mask {
                    neighborhood_results.push((id, sim));
                }
            }
            if neighborhood_results.len() >= top_k * 400 { break; }

            // Explore neighbors
            if let Some(node) = self.nodes.get(&id) {
                if let Some(neighbors) = node.neighbors.get(0) {
                    for &neighbor_id in neighbors {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);
                            if let Some(neighbor_node) = self.nodes.get(&neighbor_id) {
                                if neighbor_node.bitset & query_mask == query_mask {
                                    let n_sim = calculate_similarity(query_vec, q_1bit, q_3bit, &neighbor_node.vec_data);
                                    candidates.push(NodeSim(n_sim, neighbor_id));
                                }
                            }
                        }
                    }
                }
            }
        }

        neighborhood_results.sort_by(|a, b| {
            match b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal) {
                std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                cmp => cmp,
            }
        });
        neighborhood_results.truncate(top_k);
        neighborhood_results
    }

    fn greedy_step(&self, enter_id: u64, query_vec: &[f32], q_1b: Option<&[u64]>, q_3b: Option<(&[u8], f32)>, layer: usize) -> u64 {
        let mut curr = enter_id;
        if let Some(node) = self.nodes.get(&curr) {
            let mut curr_dist = calculate_similarity(query_vec, q_1b, q_3b, &node.vec_data);
            loop {
                let mut best_neighbor = curr;
                let mut best_dist = curr_dist;

                if layer < node.neighbors.len() {
                    for &neighbor_id in &node.neighbors[layer] {
                        if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                            let dist = calculate_similarity(query_vec, q_1b, q_3b, &neighbor.vec_data);
                            if dist > best_dist {
                                best_dist = dist;
                                best_neighbor = neighbor_id;
                            }
                        }
                    }
                }

                if best_neighbor == curr {
                    break;
                }
                curr = best_neighbor;
                curr_dist = best_dist;
            }
        }
        curr
    }

    // ─── Serialization (Binary Format) ──────────────────────────────

    /// Serializes the entire HNSW graph to a flat byte vector.
    /// Format: [MAGIC:8][VERSION:4][max_layer:8][entry_point:9][node_count:8]
    ///         [per node: id:8 bitset:16 vec_type:1 vec_len:8 vec_data:N neighbor_layer_count:8 [per layer: count:8 [neighbor_ids:8*N]]]
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.nodes.len() * 256);
        
        // Header
        buf.extend_from_slice(NEURAL_INDEX_MAGIC);
        buf.extend_from_slice(&NEURAL_INDEX_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.max_layer as u64).to_le_bytes());
        
        // Entry point (1 byte exists flag + 8 bytes id)
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
        
        // Node count
        let node_count = self.nodes.len() as u64;
        buf.extend_from_slice(&node_count.to_le_bytes());
        
        // Nodes
        for node in self.nodes.values() {
            // ID
            buf.extend_from_slice(&node.id.to_le_bytes());
            // Bitset
            buf.extend_from_slice(&node.bitset.to_le_bytes());
            
            // Vector data
            match &node.vec_data {
                VectorRepresentations::Full(f) => {
                    buf.push(1); // type tag
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
            
            // Neighbors
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

    /// Deserializes from a byte slice, validating the magic header.
    pub fn deserialize_from_bytes(data: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};
        
        if data.len() < 29 { // minimum: magic(8) + version(4) + max_layer(8) + ep_flag(1) + ep_id(8)
            return Err(Error::new(ErrorKind::InvalidData, "Neural index file too small"));
        }
        
        let mut pos = 0;
        
        // Magic
        if &data[pos..pos+8] != NEURAL_INDEX_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic header in neural_index.bin"));
        }
        pos += 8;
        
        // Version
        let version = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
        if version != NEURAL_INDEX_VERSION {
            return Err(Error::new(ErrorKind::InvalidData, format!("Unsupported index version: {}", version)));
        }
        pos += 4;
        
        // Max layer
        let max_layer = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
        pos += 8;
        
        // Entry point
        let ep_exists = data[pos];
        pos += 1;
        let ep_id = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
        pos += 8;
        let entry_point = if ep_exists == 1 { Some(ep_id) } else { None };
        
        // Node count
        if pos + 8 > data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node count"));
        }
        let node_count = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
        pos += 8;
        
        let mut nodes = HashMap::with_capacity(node_count);
        
        for _ in 0..node_count {
            // ID
            if pos + 8 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node id")); }
            let id = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
            pos += 8;
            
            // Bitset
            if pos + 16 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated bitset")); }
            let bitset = u128::from_le_bytes(data[pos..pos+16].try_into().unwrap());
            pos += 16;
            
            // Vector data
            if pos + 1 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec type")); }
            let vec_type = data[pos];
            pos += 1;
            
            if pos + 8 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec len")); }
            let vec_len = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
            pos += 8;
            
            let vec_data = match vec_type {
                1 => { // Full f32
                    let byte_len = vec_len * 4;
                    if pos + byte_len > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated f32 vec")); }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 4;
                        v.push(f32::from_le_bytes(data[start..start+4].try_into().unwrap()));
                    }
                    pos += byte_len;
                    VectorRepresentations::Full(v)
                }
                2 => { // Binary u64
                    let byte_len = vec_len * 8;
                    if pos + byte_len > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated binary vec")); }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 8;
                        v.push(u64::from_le_bytes(data[start..start+8].try_into().unwrap()));
                    }
                    pos += byte_len;
                    VectorRepresentations::Binary(v.into_boxed_slice())
                }
                3 => { // Turbo u8
                    if pos + vec_len > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated turbo vec")); }
                    let v = data[pos..pos+vec_len].to_vec();
                    pos += vec_len;
                    VectorRepresentations::Turbo(v.into_boxed_slice())
                }
                _ => {
                    VectorRepresentations::None
                }
            };
            
            // Neighbors
            if pos + 8 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated neighbor layers")); }
            let layer_count = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
            pos += 8;
            
            let mut neighbors = Vec::with_capacity(layer_count);
            for _ in 0..layer_count {
                if pos + 8 > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated neighbor count")); }
                let neighbor_count = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
                pos += 8;
                
                let byte_len = neighbor_count * 8;
                if pos + byte_len > data.len() { return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated neighbor ids")); }
                let mut layer_neighbors = Vec::with_capacity(neighbor_count);
                for i in 0..neighbor_count {
                    let start = pos + i * 8;
                    layer_neighbors.push(u64::from_le_bytes(data[start..start+8].try_into().unwrap()));
                }
                pos += byte_len;
                neighbors.push(layer_neighbors);
            }
            
            nodes.insert(id, HnswNode { id, bitset, vec_data, neighbors });
        }
        
        Ok(Self {
            nodes,
            max_layer,
            entry_point,
            backend: IndexBackend::InMemory,
        })
    }

    // ─── File Persistence ───────────────────────────────────────────

    /// Persist the current HNSW state to disk as `neural_index.bin`.
    pub fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        let data = self.serialize_to_bytes();
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        writer.flush()?;
        eprintln!("💾 [HNSW] Index persisted to {} ({} nodes, {} bytes)", path.display(), self.nodes.len(), data.len());
        Ok(())
    }

    /// Load the HNSW graph from a `neural_index.bin` file.
    /// Returns None if the file doesn't exist or is corrupt (fallback to rebuild).
    pub fn load_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };
        
        // MMap Hybrid: Mapping the file into memory saves peak RAM allocation during startup.
        let mmap = match unsafe { memmap2::MmapOptions::new().map(&file) } {
            Ok(m) => m,
            Err(e) => {
                eprintln!("⚠️ [HNSW] Failed to mmap neural_index.bin ({}). Will rebuild.", e);
                return None;
            }
        };
        
        match Self::deserialize_from_bytes(&mmap) {
            Ok(index) => {
                eprintln!("🧠 [HNSW] Cold-start: Loaded {} nodes from {} (MMap Hybrid)", index.nodes.len(), path.display());
                Some(index)
            }
            Err(e) => {
                eprintln!("⚠️ [HNSW] Corrupt neural_index.bin ({}). Will rebuild and overwrite.", e);
                None
            }
        }
    }

    /// Create or open a memory-mapped file and write the current index into it.
    /// This is the MMap backend persistence path used in Survival mode.
    pub fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        let path = match &self.backend {
            IndexBackend::MMapFile { path, .. } => path.clone(),
            _ => return Ok(()),
        };

        let data = self.serialize_to_bytes();

        // Create/truncate the file to the exact size
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.set_len(data.len() as u64)?;

        // Memory-map it
        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;
        
        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(mapped);
        }
        
        eprintln!("💾 [HNSW/MMap] Synced {} nodes to {} ({} bytes)", self.nodes.len(), path.display(), data.len());
        Ok(())
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}
