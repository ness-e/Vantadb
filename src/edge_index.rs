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
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_index_remove_edge() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.remove_edge(1, 2);
    }

    #[test]
    fn test_edge_index_remove_all_for_node() {
        let idx = EdgeIndex::new();
        idx.insert(1, 2);
        idx.insert(2, 1);
        idx.insert(1, 3);
        idx.insert(3, 4);
        idx.remove_all_for_node(1);
    }
}
