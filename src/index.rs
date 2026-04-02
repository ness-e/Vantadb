use std::collections::HashMap;

/// Simplified HNSW node with embedded filter
pub struct CpNode {
    pub id: u64,
    pub bitset: u128,
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

    pub fn add(&mut self, id: u64, bitset: u128) {
        self.nodes.insert(id, CpNode {
            id,
            bitset,
            neighbors: Vec::new(),
        });
    }

    pub fn add_link(&mut self, from: u64, to: u64) {
        if let Some(node) = self.nodes.get_mut(&from) {
            node.neighbors.push(to);
        }
    }

    /// Filters neighbors by ensuring they match the bitmask exactly.
    /// Skips expensive distance calc instantly!
    pub fn filter_neighbors(&self, id: u64, query_mask: u128) -> Vec<u64> {
        if let Some(node) = self.nodes.get(&id) {
            node.neighbors.iter()
                .filter_map(|&n_id| {
                    if let Some(n) = self.nodes.get(&n_id) {
                        if n.bitset & query_mask == query_mask {
                            return Some(n_id);
                        }
                    }
                    None
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}
