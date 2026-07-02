use super::builder::VantaEmbedded;
use crate::error::Result;
use tracing;

impl VantaEmbedded {
    /// Breadth-first traversal from one or more root nodes up to `max_depth`.
    /// Returns visited node ids in BFS order.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_bfs(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.bfs_traverse(roots, max_depth)
    }

    /// Depth-first traversal from one or more root nodes up to `max_depth`.
    /// Returns visited node ids in DFS order.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_dfs(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.dfs_traverse(roots, max_depth)
    }

    /// Topological sort starting from the given root nodes.
    /// Returns an error if the graph contains a cycle.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_topological_sort(&self, roots: &[u64]) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.topological_sort(roots)
    }

    /// Check whether the subgraph reachable from `roots` is a directed acyclic graph (DAG).
    #[tracing::instrument(skip(self), err)]
    pub fn graph_is_dag(&self, roots: &[u64]) -> Result<bool> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.is_dag(roots)
    }
}
