use super::builder::VantaEmbedded;
use crate::accumulator::GraphAccumulator;
use crate::error::Result;
use std::collections::HashMap;
use tracing;

impl VantaEmbedded {
    /// Create a new graph accumulator.
    ///
    /// The accumulator is thread-safe and can be shared across worker threads
    /// for parallel graph algorithms (PageRank, centrality, etc.).
    pub fn graph_create_accumulator(&self) -> GraphAccumulator {
        GraphAccumulator::new()
    }

    /// Atomically add `delta` to the accumulator for `node_id`.
    ///
    /// Returns the previous value (standard fetch-add semantics).
    #[tracing::instrument(skip(self, acc), err)]
    pub fn graph_accumulator_add(
        &self,
        acc: &GraphAccumulator,
        node_id: u128,
        delta: f64,
    ) -> Result<f64> {
        Ok(acc.add(node_id, delta))
    }

    /// Get the current value for `node_id` in the accumulator.
    #[tracing::instrument(skip(self, acc), err)]
    pub fn graph_accumulator_get(
        &self,
        acc: &GraphAccumulator,
        node_id: u128,
    ) -> Result<Option<f64>> {
        Ok(acc.get(node_id))
    }

    /// Capture a consistent snapshot of all accumulator values.
    #[tracing::instrument(skip(self, acc), err)]
    pub fn graph_accumulator_snapshot(&self, acc: &GraphAccumulator) -> Result<HashMap<u128, f64>> {
        Ok(acc.snapshot())
    }

    /// Breadth-first traversal from one or more root nodes up to `max_depth`.
    /// Returns visited node ids in BFS order.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_bfs(&self, roots: &[u128], max_depth: usize) -> Result<Vec<u128>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.bfs_traverse(roots, max_depth)
    }

    /// Depth-first traversal from one or more root nodes up to `max_depth`.
    /// Returns visited node ids in DFS order.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_dfs(&self, roots: &[u128], max_depth: usize) -> Result<Vec<u128>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.dfs_traverse(roots, max_depth)
    }

    /// Breadth-first traversal with label filtering.
    /// Only follows edges whose `label_id` is in `labels`.
    /// When `labels` is empty, acts like `graph_bfs` (no filter).
    #[tracing::instrument(skip(self), err)]
    pub fn graph_bfs_filtered(
        &self,
        roots: &[u128],
        max_depth: usize,
        labels: &[u32],
    ) -> Result<Vec<u128>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.bfs_traverse_filtered(roots, max_depth, labels)
    }

    /// Depth-first traversal with label filtering.
    /// Only follows edges whose `label_id` is in `labels`.
    /// When `labels` is empty, acts like `graph_dfs` (no filter).
    #[tracing::instrument(skip(self), err)]
    pub fn graph_dfs_filtered(
        &self,
        roots: &[u128],
        max_depth: usize,
        labels: &[u32],
    ) -> Result<Vec<u128>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.dfs_traverse_filtered(roots, max_depth, labels)
    }

    /// Topological sort starting from the given root nodes.
    /// Returns an error if the graph contains a cycle.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_topological_sort(&self, roots: &[u128]) -> Result<Vec<u128>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.topological_sort(roots)
    }

    /// Check whether the subgraph reachable from `roots` is a directed acyclic graph (DAG).
    #[tracing::instrument(skip(self), err)]
    pub fn graph_is_dag(&self, roots: &[u128]) -> Result<bool> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.is_dag(roots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_engine_embedded() -> VantaEmbedded {
        VantaEmbedded::test_empty(crate::config::VantaConfig::default())
    }

    #[test]
    fn test_graph_bfs_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_bfs(&[1], 5).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_dfs_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_dfs(&[1], 5).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_topological_sort_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_topological_sort(&[1]).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_is_dag_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_is_dag(&[1]).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_bfs_empty_roots_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_bfs(&[], 0).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_dfs_empty_roots_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_dfs(&[], 0).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_bfs_filtered_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_bfs_filtered(&[1], 5, &[1]).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_graph_dfs_filtered_no_engine() {
        let e = no_engine_embedded();
        let err = e.graph_dfs_filtered(&[1], 5, &[1]).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }
}
