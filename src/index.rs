use rand::Rng;
use std::collections::HashMap;

// Reutilizamos la lógica SIMD centralizada en node.rs
pub use crate::node::VectorRepresentations;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let va = VectorRepresentations::Full(a.to_vec());
    let vb = VectorRepresentations::Full(b.to_vec());
    va.cosine_similarity(&vb).unwrap_or(0.0)
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
        let mut curr_node_id = match self.entry_point {
            Some(id) => id,
            None => return Vec::new(),
        };

        // Phase 1: Descend layers from max_layer down to 1
        for layer in (1..=self.max_layer).rev() {
            curr_node_id = self.greedy_step(curr_node_id, query_vec, layer);
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
                    std::cmp::Ordering::Equal => other.1.cmp(&self.1), // Smaller ID is preferred when similarities are equal
                    cmp => cmp,
                }
            }
        }

        // Add start point for layer 0
        if let Some(node) = self.nodes.get(&curr_node_id) {
            let sim = cosine_similarity(query_vec, &node.vec_data);
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
            if neighborhood_results.len() >= top_k * 400 { break; } // Bounded search limit increased for orthogonal vector search

            // Explore neighbors
            if let Some(node) = self.nodes.get(&id) {
                if let Some(neighbors) = node.neighbors.get(0) {
                    for &neighbor_id in neighbors {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);
                            if let Some(neighbor_node) = self.nodes.get(&neighbor_id) {
                                if neighbor_node.bitset & query_mask == query_mask {
                                    let n_sim = cosine_similarity(query_vec, &neighbor_node.vec_data);
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
