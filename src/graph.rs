use std::collections::{HashSet, VecDeque};
use crate::error::Result;
use crate::storage::StorageEngine;

pub struct GraphTraverser<'a> {
    storage: &'a StorageEngine,
}

impl<'a> GraphTraverser<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Evaluates a Breadth-First-Search starting from a designated set of root IDs, 
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    pub fn bfs_traverse(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut results = Vec::new();

        for &root in roots {
            queue.push_back((root, 0));
        }

        while let Some((curr_id, depth)) = queue.pop_front() {
            if !visited.insert(curr_id) {
                continue; // Already processed
            }

            // Return all visited items
            results.push(curr_id);

            if depth < max_depth {
                // Fetch the node from the storage engine
                if let Ok(Some(node)) = self.storage.get(curr_id) {
                    for edge in &node.edges {
                        if !visited.contains(&edge.target) {
                            queue.push_back((edge.target, depth + 1));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
