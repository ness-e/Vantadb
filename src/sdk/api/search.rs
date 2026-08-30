//! Vector search and similarity operations on `VantaEmbedded`.
//!
//! Owns the K-NN path (`search_vector`) and key-anchored similarity
//! (`similar_to_key`). Namespace-level operations that share the search path
//! (`count`, `delete_by_filter`, `namespace_stats`) live in `namespaces.rs`.
//!
//! Extracted from `sdk::api` (REVIEW-12, 2026-08-30).

use super::super::builder::VantaEmbedded;
use super::super::serialization::{
    memory_record_from_node, validate_key, validate_namespace,
};
use super::super::types::*;
use crate::error::Result;

impl VantaEmbedded {
    /// K-NN vector search across all nodes via HNSW index.
    #[tracing::instrument(skip(self, vector), err)]
    pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
        if vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }
        let engine = self.engine_handle()?;
        let hnsw = engine.hnsw.load();
        // ERR-028: mirror the AUDREP-55 up-front guard using the index's own
        // metric (matches `search_nearest` exactly) so the legacy K-NN path
        // also reports InvalidInput instead of a silent empty result.
        if hnsw.config.distance_metric == crate::node::DistanceMetric::Cosine
            && crate::index::f32_l2_norm(vector) < f32::EPSILON
        {
            return Err(crate::error::VantaError::InvalidInput(
                "zero-norm cosine query vector is undefined; use a non-zero vector \
                 or the euclidean distance metric (AUDREP-55, ERR-028)"
                    .into(),
            ));
        }
        let results = {
            // ponytail: search reads from L0 only. Multi-level search
            // will need a segment-merged view.
            let vs = engine.vector_store[0].read();
            hnsw.search_nearest(
                vector,
                None,
                None,
                &crate::node::ALL_BITSET,
                top_k,
                Some(&*vs),
            )
        };
        Ok(results
            .into_iter()
            .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
            .collect())
    }

    /// Find records similar to an existing record identified by `key`.
    ///
    /// Retrieves the vector stored under `key` in `namespace` and performs a
    /// K-NN vector similarity search against the HNSW index. Results are
    /// post-filtered to the same namespace and the source record itself is
    /// excluded from the output.
    ///
    /// # Errors
    /// * [`VantaError::NotFound`] if `key` does not exist in `namespace`.
    /// * [`VantaError::NoVectorForKey`] if the record exists but carries no vector.
    ///
    /// # Notes
    /// `search_vector()` queries the global HNSW index (all namespaces). Results
    /// are post-filtered to `namespace` to preserve namespace isolation semantics.
    #[tracing::instrument(skip(self), err)]
    pub fn similar_to_key(
        &self,
        namespace: &str,
        key: &str,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let record =
            self.get(namespace, key)?
                .ok_or_else(|| crate::error::VantaError::NotFound {
                    kind: "memory record".into(),
                    id: format!("{namespace}/{key}"),
                })?;

        let vector = record.vector.ok_or_else(|| {
            crate::error::VantaError::NoVectorForKey(format!("{namespace}/{key}"))
        })?;

        // Search top_k + 1 to account for the source record itself being in results.
        let raw_hits = self.search_vector(&vector, top_k + 1)?;

        let engine = self.engine_handle()?;
        let raw_ids: Vec<u128> = raw_hits.iter().map(|h| h.node_id).collect();
        let nodes = engine.get_many(&raw_ids)?;

        let hits: Vec<VantaMemorySearchHit> = raw_hits
            .into_iter()
            .zip(nodes)
            .filter_map(|(hit, node)| {
                memory_record_from_node(&node).and_then(|r| {
                    if r.namespace == namespace && r.key != key {
                        Some(VantaMemorySearchHit {
                            record: r,
                            score: 1.0 - hit.distance,
                            explanation: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .take(top_k)
            .collect();

        Ok(hits)
    }
}