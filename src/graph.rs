//! Local graph traversal helper.
//!
//! VantaDB stores local edges in its internal node model, but v0.1.x does not claim to be a
//! full-featured graph database or graph query engine.

use crate::error::Result;
use crate::storage::StorageEngine;
use std::collections::{HashMap, HashSet};

/// Graph traversal helper with BFS, DFS, and topological sort.
pub struct GraphTraverser<'a> {
    /// Reference to the storage engine.
    storage: &'a StorageEngine,
}

impl<'a> GraphTraverser<'a> {
    /// Create a new graph traverser.
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Evaluates a Breadth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// Uses level-at-a-time batching (`get_many`) to avoid N+1 storage lookups.
    pub fn bfs_traverse(&self, roots: &[u128], max_depth: usize) -> Result<Vec<u128>> {
        let mut visited = HashSet::new();
        let mut results = Vec::new();
        let mut current_level: Vec<u128> = roots.to_vec();

        for depth in 0..=max_depth {
            if current_level.is_empty() {
                break;
            }

            // Deduplicate and filter already-visited nodes
            let mut next_level = Vec::new();
            let mut unvisited = Vec::new();
            for &id in &current_level {
                if visited.insert(id) {
                    unvisited.push(id);
                    results.push(id);
                }
            }

            if depth == max_depth || unvisited.is_empty() {
                continue;
            }

            // Batch-fetch all nodes at the current depth
            let nodes = self.storage.get_many(&unvisited)?;
            for node in &nodes {
                for edge in &node.edges {
                    if !visited.contains(&edge.target) {
                        next_level.push(edge.target);
                    }
                }
            }

            // Deduplicate next_level before the next iteration
            next_level.sort();
            next_level.dedup();
            current_level = next_level;
        }

        Ok(results)
    }

    /// BFS with label filtering. When `labels` is non-empty, only edges whose
    /// `label_id` is in the set are followed. Uses `UnifiedNode.label_index`
    /// for O(1) per-label lookups when available.
    pub fn bfs_traverse_filtered(
        &self,
        roots: &[u128],
        max_depth: usize,
        labels: &[u32],
    ) -> Result<Vec<u128>> {
        let mut visited = HashSet::new();
        let mut results = Vec::new();
        let mut current_level: Vec<u128> = roots.to_vec();

        for depth in 0..=max_depth {
            if current_level.is_empty() {
                break;
            }

            // Deduplicate and filter already-visited nodes
            let mut next_level = Vec::new();
            let mut unvisited = Vec::new();
            for &id in &current_level {
                if visited.insert(id) {
                    unvisited.push(id);
                    results.push(id);
                }
            }

            if depth == max_depth || unvisited.is_empty() {
                continue;
            }

            // Batch-fetch all nodes at this depth
            let nodes = self.storage.get_many(&unvisited)?;
            for node in &nodes {
                if !labels.is_empty() {
                    // Use label_index if available for O(1) per-label lookups
                    if !node.label_index.is_empty() {
                        for &lid in labels {
                            for &target in node.targets_by_label(lid) {
                                if !visited.contains(&target) {
                                    next_level.push(target);
                                }
                            }
                        }
                    } else {
                        // Fallback: scan edges and filter by label_id
                        for edge in &node.edges {
                            if labels.contains(&edge.label_id) && !visited.contains(&edge.target) {
                                next_level.push(edge.target);
                            }
                        }
                    }
                } else {
                    // No label filter: follow all edges (same as regular bfs)
                    for edge in &node.edges {
                        if !visited.contains(&edge.target) {
                            next_level.push(edge.target);
                        }
                    }
                }
            }

            // Deduplicate next_level before next iteration
            next_level.sort();
            next_level.dedup();
            current_level = next_level;
        }

        Ok(results)
    }

    /// DFS with label filtering. Discovers edges using label-aware discovery,
    /// then traverses the cached subgraph to avoid N+1 storage lookups.
    pub fn dfs_traverse_filtered(
        &self,
        roots: &[u128],
        max_depth: usize,
        labels: &[u32],
    ) -> Result<Vec<u128>> {
        let edges = self.discover_edges_filtered(roots, max_depth, labels)?;
        let mut visited = HashSet::new();
        let mut results = Vec::new();
        for &root in roots {
            dfs_from_cache(root, &mut visited, &mut results, &edges);
        }
        Ok(results)
    }

    /// Evaluates a Depth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// Uses a two-phase approach: first discovers all reachable nodes via batched
    /// level-at-a-time lookups (`get_many`), then traverses from the cached edges
    /// to eliminate N+1 storage reads.
    pub fn dfs_traverse(&self, roots: &[u128], max_depth: usize) -> Result<Vec<u128>> {
        let edges = self.discover_edges(roots, max_depth)?;

        let mut visited = HashSet::new();
        let mut results = Vec::new();

        for &root in roots {
            dfs_from_cache(root, &mut visited, &mut results, &edges);
        }

        Ok(results)
    }

    /// Performs a topological sort on the subgraph reachable from the given roots.
    /// Returns an error if a cycle is detected (not a DAG).
    ///
    /// Uses a two-phase approach: first discovers all reachable nodes via batched
    /// level-at-a-time lookups (`get_many`), then runs the topo-sort from the
    /// cached edges to eliminate N+1 storage reads.
    pub fn topological_sort(&self, roots: &[u128]) -> Result<Vec<u128>> {
        let max_depth = usize::MAX;
        let edges = self.discover_edges(roots, max_depth)?;

        let mut state = HashMap::new();
        let mut order = Vec::new();

        for &root in roots {
            topo_from_cache(root, &mut state, &mut order, &edges)?;
        }

        order.reverse();
        Ok(order)
    }

    /// Checks if the subgraph reachable from the given roots is a Directed Acyclic Graph (DAG)
    /// (i.e. contains no cycles).
    pub fn is_dag(&self, roots: &[u128]) -> Result<bool> {
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

    /// BFS-style batched discovery: uses `get_many` at each level to build an
    /// edge cache, avoiding N+1 individual `get()` calls.
    fn discover_edges(
        &self,
        roots: &[u128],
        max_depth: usize,
    ) -> Result<HashMap<u128, Vec<crate::node::Edge>>> {
        let mut edges: HashMap<u128, Vec<crate::node::Edge>> = HashMap::new();
        let mut current_level: Vec<u128> = roots.to_vec();

        for depth in 0..=max_depth {
            if current_level.is_empty() {
                break;
            }

            let mut unvisited = Vec::new();
            for &id in &current_level {
                if !edges.contains_key(&id) {
                    unvisited.push(id);
                }
            }

            if unvisited.is_empty() {
                break;
            }

            if depth == max_depth {
                break;
            }

            let nodes = self.storage.get_many(&unvisited)?;
            let mut next_level = Vec::new();
            for node in &nodes {
                edges.entry(node.id).or_insert_with(|| node.edges.clone());
                for edge in &node.edges {
                    if !edges.contains_key(&edge.target) {
                        next_level.push(edge.target);
                    }
                }
            }

            next_level.sort();
            next_level.dedup();
            current_level = next_level;
        }

        Ok(edges)
    }

    /// BFS-style batched discovery with label filtering.
    /// Only caches edges whose label_id is in `labels` (or all edges if `labels` is empty).
    fn discover_edges_filtered(
        &self,
        roots: &[u128],
        max_depth: usize,
        labels: &[u32],
    ) -> Result<HashMap<u128, Vec<crate::node::Edge>>> {
        let mut edges: HashMap<u128, Vec<crate::node::Edge>> = HashMap::new();
        let mut current_level: Vec<u128> = roots.to_vec();

        for depth in 0..=max_depth {
            if current_level.is_empty() {
                break;
            }

            let mut unvisited = Vec::new();
            for &id in &current_level {
                if !edges.contains_key(&id) {
                    unvisited.push(id);
                }
            }

            if unvisited.is_empty() || depth == max_depth {
                break;
            }

            let nodes = self.storage.get_many(&unvisited)?;
            let mut next_level = Vec::new();
            for node in &nodes {
                // Collect only matching edges
                if !labels.is_empty() {
                    if !node.label_index.is_empty() {
                        // Use label_index for O(1) per-label lookups
                        let mut matching = Vec::new();
                        for &lid in labels {
                            for &target in node.targets_by_label(lid) {
                                // Find the corresponding Edge for weight metadata
                                if let Some(edge) = node
                                    .edges
                                    .iter()
                                    .find(|e| e.target == target && e.label_id == lid)
                                {
                                    matching.push(edge.clone());
                                }
                            }
                        }
                        edges.insert(node.id, matching);
                    } else {
                        // Fallback: scan and filter
                        let matching: Vec<crate::node::Edge> = node
                            .edges
                            .iter()
                            .filter(|e| labels.contains(&e.label_id))
                            .cloned()
                            .collect();
                        edges.insert(node.id, matching);
                    }
                } else {
                    edges.insert(node.id, node.edges.clone());
                }

                // Queue unvisited targets
                for edge in edges.get(&node.id).unwrap() {
                    if !edges.contains_key(&edge.target) {
                        next_level.push(edge.target);
                    }
                }
            }

            next_level.sort();
            next_level.dedup();
            current_level = next_level;
        }

        Ok(edges)
    }
}

fn dfs_from_cache(
    node_id: u128,
    visited: &mut HashSet<u128>,
    results: &mut Vec<u128>,
    edges: &HashMap<u128, Vec<crate::node::Edge>>,
) {
    if !visited.insert(node_id) {
        return;
    }

    results.push(node_id);

    if let Some(node_edges) = edges.get(&node_id) {
        for edge in node_edges {
            dfs_from_cache(edge.target, visited, results, edges);
        }
    }
}

fn topo_from_cache(
    node_id: u128,
    state: &mut HashMap<u128, u8>,
    order: &mut Vec<u128>,
    edges: &HashMap<u128, Vec<crate::node::Edge>>,
) -> Result<bool> {
    match state.get(&node_id) {
        Some(1) => return Err(crate::error::VantaError::CycleDetected),
        Some(2) => return Ok(true),
        _ => {}
    }

    state.insert(node_id, 1);

    if let Some(node_edges) = edges.get(&node_id) {
        for edge in node_edges {
            topo_from_cache(edge.target, state, order, edges)?;
        }
    }

    state.insert(node_id, 2);
    order.push(node_id);

    Ok(true)
}

#[cfg(test)]
#[allow(missing_docs)]
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

    fn insert_node(storage: &StorageEngine, id: u128, edges: Vec<(u128, f32)>) {
        let mut node = UnifiedNode::new(id);
        node.edges = edges
            .into_iter()
            .map(|(target, weight)| Edge {
                target,
                weight,
                label_id: 0,
            })
            .collect();
        storage.insert(&node).unwrap();
    }

    fn insert_node_with_labeled_edges(
        storage: &StorageEngine,
        id: u128,
        edges: Vec<(u128, u32, f32)>,
    ) {
        let mut node = UnifiedNode::new(id);
        for (target, label_id, weight) in edges {
            node.add_weighted_edge(target, label_id, weight);
        }
        storage.insert(&node).unwrap();
    }

    fn build_chain(storage: &StorageEngine, count: u128) {
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

    #[test]
    fn test_bfs_filtered_basic() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        // 0 → 1 (label=1), 0 → 2 (label=2)
        insert_node_with_labeled_edges(traverser.storage, 0, vec![(1, 1, 1.0), (2, 2, 1.0)]);
        insert_node_with_labeled_edges(traverser.storage, 1, vec![]);
        insert_node_with_labeled_edges(traverser.storage, 2, vec![]);

        // BFS with label=1 should only reach node 1
        let result = traverser.bfs_traverse_filtered(&[0], 10, &[1]).unwrap();
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(!result.contains(&2), "label=1 filter should exclude node 2");
    }

    #[test]
    fn test_bfs_filtered_no_match() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node_with_labeled_edges(traverser.storage, 0, vec![(1, 42, 1.0)]);
        insert_node_with_labeled_edges(traverser.storage, 1, vec![]);

        // Filter by label that doesn't exist → stops at root
        let result = traverser.bfs_traverse_filtered(&[0], 10, &[99]).unwrap();
        assert_eq!(
            result,
            vec![0],
            "should only return root when no edges match"
        );
    }

    #[test]
    fn test_bfs_filtered_empty_labels() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        insert_node_with_labeled_edges(traverser.storage, 0, vec![(1, 1, 1.0), (2, 2, 1.0)]);
        insert_node_with_labeled_edges(traverser.storage, 1, vec![]);
        insert_node_with_labeled_edges(traverser.storage, 2, vec![]);

        // Empty label filter = no filter, should follow all edges
        let result = traverser.bfs_traverse_filtered(&[0], 10, &[]).unwrap();
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(result.contains(&2));
    }

    #[test]
    fn test_dfs_filtered_basic() {
        let (storage, _dir) = setup_storage();
        let traverser = GraphTraverser::new(Box::leak(Box::new(storage)));
        // 0 → 1 (label=1), 0 → 2 (label=2), 1 → 3 (label=1)
        insert_node_with_labeled_edges(traverser.storage, 0, vec![(1, 1, 1.0), (2, 2, 1.0)]);
        insert_node_with_labeled_edges(traverser.storage, 1, vec![(3, 1, 1.0)]);
        insert_node_with_labeled_edges(traverser.storage, 2, vec![]);
        insert_node_with_labeled_edges(traverser.storage, 3, vec![]);

        // DFS with label=1 should reach 0,1,3 but NOT 2
        let result = traverser.dfs_traverse_filtered(&[0], 10, &[1]).unwrap();
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(result.contains(&3));
        assert!(!result.contains(&2), "label=1 filter should exclude node 2");
    }
}
