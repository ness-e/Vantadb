//! Graph Data Science algorithms (PageRank, centrality) over [`GraphAccumulator`]
//! and [`GraphTraverser`].
//!
//! # Warning
//!
//! These are single-threaded, sequential implementations suitable for graphs with
//! up to ~10K nodes.  For larger graphs, consider parallelizing with `rayon`.

use crate::error::Result;
use crate::graph::GraphTraverser;
use crate::storage::StorageEngine;
use std::collections::HashMap;

/// Graph Data Science operations on a [`StorageEngine`].
///
/// Each method discovers the subgraph reachable from the given roots and runs
/// the algorithm in memory on the cached edge set.
pub struct GraphDataScience<'a> {
    storage: &'a StorageEngine,
}

impl<'a> GraphDataScience<'a> {
    /// Create a new GDS session borrowing the storage engine.
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Compute **PageRank** for the subgraph reachable from `roots`.
    ///
    /// Classic iterative PageRank with damping factor:
    ///
    /// ```text
    /// PR(u) = (1 - d) / N + d · Σ(PR(v) / out_degree(v))   over v → u
    /// ```
    ///
    /// Stops when the L1 delta across all nodes drops below `tolerance` or
    /// `max_iterations` is reached.
    pub fn page_rank(
        &self,
        roots: &[u128],
        max_iterations: usize,
        damping: f64,
        tolerance: f64,
    ) -> Result<HashMap<u128, f64>> {
        let traverser = GraphTraverser::new(self.storage);
        let edges = traverser.discover_edges(roots, usize::MAX)?;

        // Collect all unique nodes from edge keys + targets
        let mut all_nodes: Vec<u128> = edges.keys().copied().collect();
        for targets in edges.values() {
            for edge in targets {
                if !all_nodes.contains(&edge.target) {
                    all_nodes.push(edge.target);
                }
            }
        }

        let n = all_nodes.len();
        if n == 0 {
            return Ok(HashMap::new());
        }

        // Pre-compute out-degree for every node
        let mut out_degree: HashMap<u128, usize> = HashMap::new();
        for (&node, edge_list) in &edges {
            out_degree.insert(node, edge_list.len());
        }
        // Nodes with no outgoing edges have out_degree 0 (already default).

        // Initialize rank = 1.0 / N
        let init = 1.0 / n as f64;
        let mut rank: HashMap<u128, f64> = HashMap::with_capacity(n);
        for &node in &all_nodes {
            rank.insert(node, init);
        }

        // Build reverse edge index for fast lookup of in-links
        // in_neighbors[target] = list of (source, out_degree_of_source)
        let mut in_neighbors: HashMap<u128, Vec<(u128, usize)>> = HashMap::new();
        for (&source, edge_list) in &edges {
            let od = out_degree.get(&source).copied().unwrap_or(0);
            for edge in edge_list {
                in_neighbors
                    .entry(edge.target)
                    .or_default()
                    .push((source, od));
            }
        }

        for _iter in 0..max_iterations {
            // Compute the total dangling mass: rank of all nodes with out_degree == 0
            let mut total_dangling = 0.0;
            for (&node, &r) in &rank {
                if out_degree.get(&node).copied().unwrap_or(0) == 0 {
                    total_dangling += r;
                }
            }
            let dangling_contribution = total_dangling / n as f64;

            let mut new_rank: HashMap<u128, f64> = HashMap::with_capacity(n);

            for &node in &all_nodes {
                let mut sum = 0.0;
                // Sum contributions from all in-links
                if let Some(sources) = in_neighbors.get(&node) {
                    for &(source, od) in sources {
                        if od > 0 {
                            sum += rank.get(&source).copied().unwrap_or(0.0) / od as f64;
                        }
                    }
                }
                // Teleport + damping + dangling redistribution
                let teleport = (1.0 - damping) / n as f64;
                let pr = teleport + damping * (sum + dangling_contribution);
                new_rank.insert(node, pr);
            }

            // Compute L1 delta
            let mut delta = 0.0;
            for &node in &all_nodes {
                let old = rank.get(&node).copied().unwrap_or(0.0);
                let new = new_rank.get(&node).copied().unwrap_or(0.0);
                delta += (new - old).abs();
            }

            rank = new_rank;

            if delta < tolerance {
                break;
            }
        }

        Ok(rank)
    }

    /// Compute **degree centrality** for the subgraph reachable from `roots`.
    ///
    /// Returns `(in_degree, out_degree)` counts for each discovered node.
    /// Out-degree is the number of edges originating from the node.
    /// In-degree is the number of edges pointing to the node.
    pub fn degree_centrality(&self, roots: &[u128]) -> Result<HashMap<u128, (usize, usize)>> {
        let traverser = GraphTraverser::new(self.storage);
        let edges = traverser.discover_edges(roots, usize::MAX)?;

        // Collect all unique nodes
        let mut all_nodes: Vec<u128> = edges.keys().copied().collect();
        for targets in edges.values() {
            for edge in targets {
                if !all_nodes.contains(&edge.target) {
                    all_nodes.push(edge.target);
                }
            }
        }

        let mut result: HashMap<u128, (usize, usize)> = HashMap::with_capacity(all_nodes.len());
        for &node in &all_nodes {
            let out_deg = edges.get(&node).map_or(0, |e| e.len());
            result.insert(node, (0, out_deg));
        }

        // Count in-degrees: for each edge, increment target's in_degree
        for edge_list in edges.values() {
            for edge in edge_list {
                result.entry(edge.target).and_modify(|(in_deg, _)| {
                    *in_deg += 1;
                });
            }
        }

        Ok(result)
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::node::UnifiedNode;
    use crate::storage::{BackendKind, StorageEngine};
    use crate::Edge;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
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

    // ── PageRank ──

    #[test]
    fn test_page_rank_chain() {
        let (storage, _dir) = setup_storage();
        // 0 → 1 → 2
        insert_node(&storage, 0, vec![(1, 1.0)]);
        insert_node(&storage, 1, vec![(2, 1.0)]);
        insert_node(&storage, 2, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let ranks = gds.page_rank(&[0], 100, 0.85, 1e-6).unwrap();

        // Sum should be ≈ 1.0
        let sum: f64 = ranks.values().sum();
        assert!(
            (sum - 1.0).abs() < 1e-4,
            "PageRank sum should ≈ 1.0, got {sum}"
        );

        // In PageRank, sink nodes (no outgoing edges) accumulate rank because
        // they receive link mass but do not distribute it.  So rank(2) > rank(0).
        assert!(
            ranks[&2] > ranks[&0],
            "sink node rank(2)={} should be > source rank(0)={}",
            ranks[&2],
            ranks[&0]
        );
        // All ranks should be positive
        assert!(
            ranks.values().all(|&r| r > 0.0),
            "all ranks should be positive"
        );
    }

    #[test]
    fn test_page_rank_diamond() {
        let (storage, _dir) = setup_storage();
        // 0 → {1, 2} → 3
        insert_node(&storage, 0, vec![(1, 1.0), (2, 1.0)]);
        insert_node(&storage, 1, vec![(3, 1.0)]);
        insert_node(&storage, 2, vec![(3, 1.0)]);
        insert_node(&storage, 3, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let ranks = gds.page_rank(&[0], 100, 0.85, 1e-6).unwrap();

        let sum: f64 = ranks.values().sum();
        assert!(
            (sum - 1.0).abs() < 1e-4,
            "PageRank sum should ≈ 1.0, got {sum}"
        );

        // Nodes 1 and 2 are symmetric and should have equal rank
        let diff = (ranks[&1] - ranks[&2]).abs();
        assert!(
            diff < 1e-6,
            "symmetric nodes should have equal rank, diff={diff}"
        );
    }

    #[test]
    fn test_page_rank_convergence() {
        let (storage, _dir) = setup_storage();
        // Chain of 10 nodes
        for i in 0..10u128 {
            let edges = if i < 9 { vec![(i + 1, 1.0)] } else { vec![] };
            insert_node(&storage, i, edges);
        }

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));

        // High tolerance should converge quickly
        let ranks_loose = gds.page_rank(&[0], 100, 0.85, 1e-2).unwrap();
        // Low tolerance should need more iterations but still converge
        let ranks_tight = gds.page_rank(&[0], 100, 0.85, 1e-10).unwrap();

        // Both should have reasonable sums
        let sum_loose: f64 = ranks_loose.values().sum();
        let sum_tight: f64 = ranks_tight.values().sum();
        assert!(
            (sum_loose - 1.0).abs() < 1e-2,
            "loose sum should ≈ 1.0, got {sum_loose}"
        );
        assert!(
            (sum_tight - 1.0).abs() < 1e-4,
            "tight sum should ≈ 1.0, got {sum_tight}"
        );
    }

    #[test]
    fn test_page_rank_disconnected_roots() {
        let (storage, _dir) = setup_storage();
        // Two disconnected graphs: 0→1 and 2→3
        insert_node(&storage, 0, vec![(1, 1.0)]);
        insert_node(&storage, 1, vec![]);
        insert_node(&storage, 2, vec![(3, 1.0)]);
        insert_node(&storage, 3, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let ranks = gds.page_rank(&[0, 2], 100, 0.85, 1e-6).unwrap();

        let sum: f64 = ranks.values().sum();
        assert!(
            (sum - 1.0).abs() < 1e-4,
            "PageRank sum should ≈ 1.0, got {sum}"
        );

        // Sink nodes (1, 3) accumulate rank; in a 2-node chain sink > source.
        assert!(
            ranks[&1] > ranks[&0],
            "sink rank(1)={} should be > source rank(0)={}",
            ranks[&1],
            ranks[&0]
        );
        assert!(
            ranks[&3] > ranks[&2],
            "sink rank(3)={} should be > source rank(2)={}",
            ranks[&3],
            ranks[&2]
        );
        // All ranks positive
        assert!(ranks.values().all(|&r| r > 0.0));
    }

    // ── Degree Centrality ──

    #[test]
    fn test_degree_centrality_basic() {
        let (storage, _dir) = setup_storage();
        // 0 → 1 → 2
        insert_node(&storage, 0, vec![(1, 1.0)]);
        insert_node(&storage, 1, vec![(2, 1.0)]);
        insert_node(&storage, 2, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let degrees = gds.degree_centrality(&[0]).unwrap();

        assert_eq!(degrees.len(), 3);
        // Node 0: out=1, in=0
        assert_eq!(degrees[&0], (0, 1));
        // Node 1: out=1, in=1
        assert_eq!(degrees[&1], (1, 1));
        // Node 2: out=0, in=1
        assert_eq!(degrees[&2], (1, 0));
    }

    #[test]
    fn test_degree_centrality_disconnected() {
        let (storage, _dir) = setup_storage();
        // 0 → 1, 2 → 3 (two disconnected components)
        insert_node(&storage, 0, vec![(1, 1.0)]);
        insert_node(&storage, 1, vec![]);
        insert_node(&storage, 2, vec![(3, 1.0)]);
        insert_node(&storage, 3, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let degrees = gds.degree_centrality(&[0, 2]).unwrap();

        assert_eq!(degrees.len(), 4);
        assert_eq!(degrees[&0], (0, 1));
        assert_eq!(degrees[&1], (1, 0));
        assert_eq!(degrees[&2], (0, 1));
        assert_eq!(degrees[&3], (1, 0));
    }

    #[test]
    fn test_degree_centrality_diamond() {
        let (storage, _dir) = setup_storage();
        // 0 → {1, 2} → 3
        insert_node(&storage, 0, vec![(1, 1.0), (2, 1.0)]);
        insert_node(&storage, 1, vec![(3, 1.0)]);
        insert_node(&storage, 2, vec![(3, 1.0)]);
        insert_node(&storage, 3, vec![]);

        let gds = GraphDataScience::new(Box::leak(Box::new(storage)));
        let degrees = gds.degree_centrality(&[0]).unwrap();

        assert_eq!(degrees.len(), 4);
        assert_eq!(degrees[&0], (0, 2));
        assert_eq!(degrees[&1], (1, 1));
        assert_eq!(degrees[&2], (1, 1));
        assert_eq!(degrees[&3], (2, 0));
    }
}
