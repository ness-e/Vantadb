use super::builder::VantaEmbedded;
use crate::error::Result;
use crate::gds::GraphDataScience;
use std::collections::HashMap;
use tracing;

impl VantaEmbedded {
    /// Compute PageRank for the subgraph reachable from the given roots.
    ///
    /// - `roots`: starting node IDs for edge discovery
    /// - `max_iterations`: maximum iterations (default: 100)
    /// - `damping`: PageRank damping factor (default: 0.85)
    /// - `tolerance`: convergence threshold (default: 1e-6)
    #[tracing::instrument(skip(self), err)]
    pub fn graph_page_rank(
        &self,
        roots: &[u128],
        max_iterations: usize,
        damping: f64,
        tolerance: f64,
    ) -> Result<HashMap<u128, f64>> {
        let engine = self.engine_handle()?;
        let gds = GraphDataScience::new(&engine);
        gds.page_rank(roots, max_iterations, damping, tolerance)
    }

    /// Compute degree centrality (in/out degree counts) for the subgraph
    /// reachable from the given roots.
    ///
    /// Returns a map of `node_id → (in_degree, out_degree)`.
    #[tracing::instrument(skip(self), err)]
    pub fn graph_degree_centrality(&self, roots: &[u128]) -> Result<HashMap<u128, (usize, usize)>> {
        let engine = self.engine_handle()?;
        let gds = GraphDataScience::new(&engine);
        gds.degree_centrality(roots)
    }
}
