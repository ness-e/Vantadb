use super::builder::VantaEmbedded;
use crate::error::Result;
use tracing;

impl VantaEmbedded {
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
}
