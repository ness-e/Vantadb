use std::collections::HashMap;

/// Math helper for Cosine Similarity
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

/// Simplified HNSW node with embedded filter
pub struct CpNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: Option<Vec<f32>>,
    pub neighbors: Vec<u64>,
}

/// Co-located Pre-filter Index (CP-Index)
pub struct CPIndex {
    nodes: HashMap<u64, CpNode>,
}

impl CPIndex {
    pub fn new() -> Self {
        Self { nodes: HashMap::new() }
    }

    pub fn add(&mut self, id: u64, bitset: u128, vec_data: Option<Vec<f32>>) {
        self.nodes.insert(id, CpNode {
            id,
            bitset,
            vec_data,
            neighbors: Vec::new(),
        });
    }

    pub fn add_link(&mut self, from: u64, to: u64) {
        if let Some(node) = self.nodes.get_mut(&from) {
            node.neighbors.push(to);
        }
    }

    pub fn search_nearest(&self, query_vec: &[f32], query_mask: u128, top_k: usize) -> Vec<(u64, f32)> {
        let mut results = Vec::new();
        // Naive linear scan for benchmark MVP purposes. Actual HNSW uses greedy routing.
        for (id, node) in &self.nodes {
            if node.bitset & query_mask == query_mask {
                if let Some(v) = &node.vec_data {
                    let sim = cosine_similarity(query_vec, v);
                    results.push((*id, sim));
                }
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}
