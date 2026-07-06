use crate::error::Result;
use dashmap::DashSet;

/// An in-memory adjacency index for directed edges using a concurrent hash set.
///
/// Tracks every directed edge `(source → target)` using `u64` node IDs so that
/// cascade delete (PERF-07) can find incoming edges when a node is removed.
pub(crate) struct EdgeIndex {
    edges: DashSet<(u128, u128)>,
}

impl EdgeIndex {
    /// Create a new empty edge index.
    pub fn new() -> Self {
        Self {
            edges: DashSet::new(),
        }
    }

    /// Insert a directed edge from `from` to `to`.
    pub fn insert(&self, from: u128, to: u128) {
        self.edges.insert((from, to));
    }

    /// Remove all outgoing edges from a given node.
    pub fn remove_outgoing(&self, from: u128) {
        self.edges.retain(|(f, _)| *f != from);
    }

    /// Remove a specific directed edge.
    pub fn remove_edge(&self, from: u128, to: u128) {
        self.edges.remove(&(from, to));
    }

    /// Remove all edges (both incoming and outgoing) for a given node.
    ///
    /// This is the single call needed during cascade delete — it clears every
    /// edge pair that references `node_id` on either side.
    pub fn remove_all_for_node(&self, node_id: u128) {
        self.edges.retain(|(f, t)| *f != node_id && *t != node_id);
    }

    /// Check whether a directed edge exists.
    pub fn has_edge(&self, from: u128, to: u128) -> bool {
        self.edges.contains(&(from, to))
    }

    /// Return all target nodes reachable from a given source.
    pub fn outgoing(&self, from: u128) -> Vec<u128> {
        self.edges
            .iter()
            .filter(|e| e.0 == from)
            .map(|e| e.1)
            .collect()
    }

    /// Return all source nodes that point to a given target.
    pub fn incoming(&self, to: u128) -> Vec<u128> {
        self.edges
            .iter()
            .filter(|e| e.1 == to)
            .map(|e| e.0)
            .collect()
    }

    /// Return the total number of edges in the index.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Verify referential integrity — checks for self-loops.
    /// Full cross-node verification requires access to the node store.
    pub fn verify_referential_integrity(&self) -> Result<()> {
        for item in self.edges.iter() {
            let (from, to) = *item.key();
            if from == to {
                let msg = format!("Self-loop edge detected: node {} references itself", from as u128);
                return Err(crate::error::VantaError::ValidationError {
                    field: "edge".into(),
                    reason: msg,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_index_insert_and_has_edge() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        assert!(idx.has_edge(1, 2));
        assert!(!idx.has_edge(2, 1));
    }

    #[test]
    fn test_edge_index_no_edge() {
        let idx = EdgeIndex::new();
        assert!(!idx.has_edge(1, 2));
    }

    #[test]
    fn test_edge_index_outgoing() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(1, 3);
        idx.insert(2, 4);

        let mut out = idx.outgoing(1);
        out.sort();
        assert_eq!(out, vec![2, 3]);

        assert_eq!(idx.outgoing(99), Vec::<u128>::new());
    }

    #[test]
    fn test_edge_index_incoming() {
        let idx = EdgeIndex::new();
        idx.insert(1, 3);
        idx.insert(2, 3);
        idx.insert(3, 4);

        let mut inc = idx.incoming(3);
        inc.sort();
        assert_eq!(inc, vec![1, 2]);
    }

    #[test]
    fn test_edge_index_remove_edge() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.remove_edge(1, 2);
        assert!(!idx.has_edge(1, 2));
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_edge_index_remove_outgoing() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(1, 3);
        idx.insert(2, 3);
        idx.remove_outgoing(1);

        assert!(!idx.has_edge(1, 2));
        assert_eq!(idx.len(), 1);
        assert!(idx.has_edge(2, 3));
    }

    #[test]
    fn test_edge_index_remove_all_for_node() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(2, 1);
        idx.insert(1, 3);
        idx.insert(3, 4);
        idx.remove_all_for_node(1);

        assert!(!idx.has_edge(1, 2));
        assert!(!idx.has_edge(2, 1));
        assert!(!idx.has_edge(1, 3));
        assert!(idx.has_edge(3, 4));
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn test_edge_index_len() {
        let idx = EdgeIndex::new();
        assert_eq!(idx.len(), 0);
        idx.insert(1, 2);
        idx.insert(1, 3);
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn test_edge_index_verify_referential_integrity() {
        let idx = EdgeIndex::new();
        assert!(idx.verify_referential_integrity().is_ok());
    }

    #[test]
    fn test_edge_index_duplicate_insert() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(1, 2);
        assert_eq!(idx.len(), 1);
    }
}
