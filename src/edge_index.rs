#![allow(dead_code)]
use crate::error::Result;
use dashmap::DashSet;

/// An in-memory adjacency index for directed edges using a concurrent hash set.
pub(crate) struct EdgeIndex {
    edges: DashSet<(u64, u64)>,
}

impl EdgeIndex {
    /// Create a new empty edge index.
    pub fn new() -> Self {
        Self {
            edges: DashSet::new(),
        }
    }

    /// Insert a directed edge from `from` to `to`.
    pub fn insert(&self, from: u64, to: u64) {
        self.edges.insert((from, to));
    }

    /// Remove all outgoing edges from a given node.
    pub fn remove_from(&self, from: u64) {
        self.edges.retain(|(f, _)| *f != from);
    }

    /// Remove a specific directed edge.
    pub fn remove_edge(&self, from: u64, to: u64) {
        self.edges.remove(&(from, to));
    }

    /// Check whether a directed edge exists.
    pub fn has_edge(&self, from: u64, to: u64) -> bool {
        self.edges.contains(&(from, to))
    }

    /// Return all target nodes reachable from a given source.
    pub fn outgoing(&self, from: u64) -> Vec<u64> {
        self.edges
            .iter()
            .filter(|e| e.0 == from)
            .map(|e| e.1)
            .collect()
    }

    /// Return all source nodes that point to a given target.
    pub fn incoming(&self, to: u64) -> Vec<u64> {
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

    /// Verify referential integrity (currently a no-op placeholder).
    pub fn verify_referential_integrity(&self) -> Result<()> {
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

        assert_eq!(idx.outgoing(99), Vec::<u64>::new());
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
    fn test_edge_index_remove_from() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(1, 3);
        idx.insert(2, 3);
        idx.remove_from(1);

        assert!(!idx.has_edge(1, 2));
        assert_eq!(idx.len(), 1);
        assert!(idx.has_edge(2, 3));
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
