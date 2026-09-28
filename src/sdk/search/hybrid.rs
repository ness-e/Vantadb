use super::super::builder::Embedded;
use super::super::types::*;
use crate::error::Result;
use web_time::Instant;

impl Embedded {
    /// Hybrid (lexical + dense + optional sparse) memory search fused with RRF.
    ///
    /// `boost` opts into the deterministic entity-cluster boost (WIRE-05); the
    /// returned [`EntityBoostReport`] carries per-hit provenance and is empty
    /// when `boost` is `None` or no cluster has co-occurring candidates.
    pub(super) fn hybrid_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        text_query: &str,
        filters: &MemoryMetadata,
        top_k: usize,
        distance_metric: crate::node::DistanceMetric,
        query_sparse: Option<&crate::node::SparseVector>,
        method: Option<crate::index::IndexType>,
        rrf_k: f32,
        candidate_k: Option<usize>,
        boost: Option<&EntityBoost>,
    ) -> Result<(Vec<MemorySearchHit>, EntityBoostReport)> {
        let started = Instant::now();
        if top_k == 0 {
            crate::metrics::record_hybrid_query(0, 0);
            return Ok((Vec::new(), EntityBoostReport::default()));
        }

        let budget = super::fusion::hybrid_candidate_budget(top_k, candidate_k);
        let lexical_hits = self.lexical_search(namespace, text_query, filters, budget)?;
        let vector_hits = self.vector_memory_search(
            namespace,
            query_vector,
            filters,
            budget,
            distance_metric,
            method,
        )?;
        let (mut hits, boost_report) = match query_sparse {
            Some(query_sparse) => {
                let sparse_hits =
                    self.sparse_memory_search(namespace, query_sparse, filters, budget)?;
                super::fusion::fuse_rrf_many_with_entity_boost(
                    vec![lexical_hits, vector_hits, sparse_hits],
                    rrf_k,
                    boost,
                )
            }
            None => {
                super::fusion::fuse_rrf_with_entity_boost(lexical_hits, vector_hits, rrf_k, boost)
            }
        };
        let candidates_fused = hits.len() as u64;
        hits.truncate(top_k);
        crate::metrics::record_hybrid_query(started.elapsed().as_millis() as u64, candidates_fused);
        Ok((hits, boost_report))
    }
}
