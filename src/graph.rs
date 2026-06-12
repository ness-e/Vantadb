//! Local graph traversal helper.
//!
//! VantaDB stores local edges in its internal node model, but v0.1.x does not claim to be a
//! full-featured graph database or graph query engine.

use crate::error::Result;
use crate::storage::StorageEngine;
use std::collections::{HashMap, HashSet, VecDeque};

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

    /// Evaluates a Depth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    pub fn dfs_traverse(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let mut visited = HashSet::new();
        let mut results = Vec::new();

        for &root in roots {
            self.dfs_visit(root, 0, max_depth, &mut visited, &mut results)?;
        }

        Ok(results)
    }

    fn dfs_visit(
        &self,
        node_id: u64,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<u64>,
        results: &mut Vec<u64>,
    ) -> Result<()> {
        if !visited.insert(node_id) {
            return Ok(());
        }

        results.push(node_id);

        if depth < max_depth {
            if let Ok(Some(node)) = self.storage.get(node_id) {
                for edge in &node.edges {
                    self.dfs_visit(edge.target, depth + 1, max_depth, visited, results)?;
                }
            }
        }

        Ok(())
    }

    /// Performs a topological sort on the subgraph reachable from the given roots.
    /// Returns an error if a cycle is detected (not a DAG).
    pub fn topological_sort(&self, roots: &[u64]) -> Result<Vec<u64>> {
        let mut state = HashMap::new(); // Node ID -> Color (1 for Gray, 2 for Black)
        let mut order = Vec::new();

        for &root in roots {
            self.topo_visit(root, &mut state, &mut order)?;
        }

        // El orden topológico es el reverso del orden de finalización DFS
        order.reverse();
        Ok(order)
    }

    fn topo_visit(
        &self,
        node_id: u64,
        state: &mut HashMap<u64, u8>,
        order: &mut Vec<u64>,
    ) -> Result<()> {
        match state.get(&node_id) {
            Some(1) => {
                return Err(crate::error::VantaError::Execution(format!(
                    "Cycle detected at node {}",
                    node_id
                )));
            }
            Some(2) => return Ok(()),
            _ => {}
        }

        // Marcar como Gris (visitando)
        state.insert(node_id, 1);

        // Visitar sucesores
        if let Ok(Some(node)) = self.storage.get(node_id) {
            for edge in &node.edges {
                self.topo_visit(edge.target, state, order)?;
            }
        }

        // Marcar como Negro (finalizado)
        state.insert(node_id, 2);
        order.push(node_id);

        Ok(())
    }

    /// Checks if the subgraph reachable from the given roots is a Directed Acyclic Graph (DAG)
    /// (i.e. contains no cycles).
    pub fn is_dag(&self, roots: &[u64]) -> Result<bool> {
        match self.topological_sort(roots) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("Cycle detected") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}
