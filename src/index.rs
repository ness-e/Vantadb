use std::collections::HashMap;
use rand::Rng;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_v = wide::f32x8::ZERO;
    let mut norm_a_v = wide::f32x8::ZERO;
    let mut norm_b_v = wide::f32x8::ZERO;

    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();

    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = wide::f32x8::from([a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5], a_chunk[6], a_chunk[7]]);
        let vb = wide::f32x8::from([b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5], b_chunk[6], b_chunk[7]]);
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

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

/// Simplified HNSW node with embedded filter and multi-layer neighbors
pub struct HnswNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: Vec<f32>,
    /// Vec of layers, where each layer contains a list of neighbor IDs
    pub neighbors: Vec<Vec<u64>>,
}

/// HNSW Co-located Pre-filter Index (CP-Index)
pub struct CPIndex {
    pub nodes: HashMap<u64, HnswNode>,
    pub max_layer: usize,
    pub entry_point: Option<u64>,
}

impl CPIndex {
    pub fn new() -> Self {
        Self { 
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
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

    pub fn add(&mut self, id: u64, bitset: u128, vec_data: Option<Vec<f32>>) {
        let vec_data = match vec_data {
            Some(v) => v,
            None => return, // Only index nodes with vectors
        };

        let level = Self::random_layer();
        let mut neighbors = vec![Vec::new(); level + 1];

        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_layer = level;
        } else {
            // MVP: Just fully connect to entry point across valid layers to maintain navigation.
            // * Real HNSW would do a greedy search to find actual nearest neighbors to connect.
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
    pub fn search_nearest(&self, query_vec: &[f32], query_mask: u128, top_k: usize) -> Vec<(u64, f32)> {
        let mut results = Vec::new();

        let mut curr_node_id = match self.entry_point {
            Some(id) => id,
            None => return results,
        };

        // Phase 1: Descend layers from max_layer down to 1
        for layer in (1..=self.max_layer).rev() {
            curr_node_id = self.greedy_step(curr_node_id, query_vec, layer);
        }

        // Phase 2: Exhaustive local search at layer 0 (Neighborhood scanning)
        // MVP logic: linear scan around the vicinity of the found node.
        for (id, node) in &self.nodes {
            if node.bitset & query_mask == query_mask {
                let sim = cosine_similarity(query_vec, &node.vec_data);
                results.push((*id, sim));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    fn greedy_step(&self, enter_id: u64, query_vec: &[f32], layer: usize) -> u64 {
        let mut curr = enter_id;
        if let Some(node) = self.nodes.get(&curr) {
            let mut curr_dist = cosine_similarity(query_vec, &node.vec_data);
            loop {
                let mut best_neighbor = curr;
                let mut best_dist = curr_dist;

                if layer < node.neighbors.len() {
                    for &neighbor_id in &node.neighbors[layer] {
                        if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                            let dist = cosine_similarity(query_vec, &neighbor.vec_data);
                            if dist > best_dist { // Higher cosine sim is better
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
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}
