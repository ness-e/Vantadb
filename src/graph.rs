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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::node::UnifiedNode;
    use crate::storage::{BackendKind, StorageEngine};
    use crate::Edge;
    use tempfile::tempdir;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");
        (storage, dir)
    }

    fn insert_node(storage: &StorageEngine, id: u64, edges: Vec<(u64, f32)>) {
        let mut node = UnifiedNode::new(id);
        node.edges = edges
            .into_iter()
            .map(|(target, weight)| Edge {
                target,
                weight,
                label: String::new(),
            })
            .collect();
        storage.insert(&node).unwrap();
    }

    fn build_chain(storage: &StorageEngine, count: u64) {
        for i in 0..count {
            let edges = if i < count - 1 {
                vec![(i + 1, 1.0)]
            } else {
                vec![]
            };
            insert_node(storage, i, edges);
        }
    }

    #[test]
    fn test_bfs_chain_traversal() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 5);
        let result = traverser.bfs_traverse(&[0], 10).unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_bfs_depth_limit() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 10);
        let result = traverser.bfs_traverse(&[0], 2).unwrap();
        assert_eq!(result, vec![0, 1, 2]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_bfs_disconnected_roots() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0)]);
        insert_node(traverser.storage, 1, vec![(2, 1.0)]);
        insert_node(traverser.storage, 2, vec![]);
        insert_node(traverser.storage, 3, vec![(4, 1.0)]);
        insert_node(traverser.storage, 4, vec![]);

        let result = traverser.bfs_traverse(&[0, 3], 10).unwrap();
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        assert!(result.contains(&4));
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_dfs_chain_traversal() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 5);
        let result = traverser.dfs_traverse(&[0], 10).unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_dfs_depth_limit() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 10);
        let result = traverser.dfs_traverse(&[0], 2).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_bfs_empty_roots() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        let result = traverser.bfs_traverse(&[], 10).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dfs_empty_roots() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        let result = traverser.dfs_traverse(&[], 10).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_bfs_diamond_graph() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0), (2, 1.0)]);
        insert_node(traverser.storage, 1, vec![(3, 1.0)]);
        insert_node(traverser.storage, 2, vec![(3, 1.0)]);
        insert_node(traverser.storage, 3, vec![]);

        let result = traverser.bfs_traverse(&[0], 10).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(&result[0..3], &[0, 1, 2]);
        assert_eq!(result[3], 3);
    }

    #[test]
    fn test_dfs_diamond_graph() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0), (2, 1.0)]);
        insert_node(traverser.storage, 1, vec![(3, 1.0)]);
        insert_node(traverser.storage, 2, vec![(3, 1.0)]);
        insert_node(traverser.storage, 3, vec![]);

        let result = traverser.dfs_traverse(&[0], 10).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 0);
        assert_eq!(result[2], 3);
    }

    #[test]
    fn test_topological_sort_chain() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 5);
        let result = traverser.topological_sort(&[0]).unwrap();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0), (2, 1.0)]);
        insert_node(traverser.storage, 1, vec![(3, 1.0)]);
        insert_node(traverser.storage, 2, vec![(3, 1.0)]);
        insert_node(traverser.storage, 3, vec![]);

        let result = traverser.topological_sort(&[0]).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 0);
        assert_eq!(result[3], 3);
    }

    #[test]
    fn test_topological_sort_cycle_detection() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0)]);
        insert_node(traverser.storage, 1, vec![(2, 1.0)]);
        insert_node(traverser.storage, 2, vec![(0, 1.0)]);

        let result = traverser.topological_sort(&[0]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cycle detected"));
    }

    #[test]
    fn test_is_dag_true() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        build_chain(traverser.storage, 3);
        assert!(traverser.is_dag(&[0]).unwrap());
    }

    #[test]
    fn test_is_dag_false() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(1, 1.0)]);
        insert_node(traverser.storage, 1, vec![(0, 1.0)]);
        assert!(!traverser.is_dag(&[0]).unwrap());
    }

    #[test]
    fn test_bfs_nonexistent_node() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        let result = traverser.bfs_traverse(&[999], 10).unwrap();
        assert_eq!(result, vec![999]);
    }

    #[test]
    fn test_bfs_self_loop() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node(traverser.storage, 0, vec![(0, 1.0)]);
        let result = traverser.bfs_traverse(&[0], 10).unwrap();
        assert_eq!(result, vec![0]);
    }
}
