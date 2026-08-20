use super::super::builder::VantaEmbedded;
use super::super::types::*;
use crate::error::Result;
use web_time::Instant;

impl VantaEmbedded {
    pub(super) fn hybrid_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        text_query: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
        query_sparse: Option<&crate::node::SparseVector>,
        method: Option<crate::index::IndexType>,
        rrf_k: f32,
        candidate_k: Option<usize>,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let started = Instant::now();
        if top_k == 0 {
            crate::metrics::record_hybrid_query(0, 0);
            return Ok(Vec::new());
        }

        let budget = crate::planner::hybrid_candidate_budget(top_k, candidate_k);
        let lexical_hits = self.lexical_search(namespace, text_query, filters, budget)?;
        let vector_hits = self.vector_memory_search(
            namespace,
            query_vector,
            filters,
            budget,
            distance_metric,
            method,
        )?;
        let mut hits = match query_sparse {
            Some(query_sparse) => {
                let sparse_hits =
                    self.sparse_memory_search(namespace, query_sparse, filters, budget)?;
                crate::planner::fuse_rrf_many(vec![lexical_hits, vector_hits, sparse_hits], rrf_k)
            }
            None => crate::planner::fuse_rrf(lexical_hits, vector_hits, rrf_k),
        };
        let candidates_fused = hits.len() as u64;
        hits.truncate(top_k);
        crate::metrics::record_hybrid_query(started.elapsed().as_millis() as u64, candidates_fused);
        Ok(hits)
    }
}
