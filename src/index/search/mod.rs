//! HNSW graph search: layer traversal, neighbor selection, and index-level
//! search entry points (`search_nearest` + alternate backends).

mod alternate;
mod layer;
mod nearest;
mod neighbors;
mod pool;
mod profile;

pub(crate) use profile::SearchProfile;

use super::graph::CPIndex;

impl crate::index::VecIndex for CPIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &crate::node::FilterBitset,
        top_k: usize,
        vector_store: Option<&crate::storage::vfile::VantaFile>,
        distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)> {
        // MCP-02: the per-request `distance_metric` from the `VecIndex` trait
        // reaches the actual scoring via `search_nearest_with_metric` (exact
        // for the flat + HNSW paths; config-driven IVF/SCANN routing warns on
        // mismatch instead of silently scoring with the wrong metric).
        self.search_nearest_with_metric(
            query_vec,
            None,
            None,
            query_mask,
            top_k,
            vector_store,
            distance_metric,
        )
    }

    fn add(
        &self,
        id: u128,
        bitset: crate::node::FilterBitset,
        vec_data: crate::node::VectorRepresentations,
        storage_offset: u64,
    ) -> crate::error::Result<()> {
        // ERR-031: propagate the rejection (e.g. zero-norm vector under
        // cosine, AUDREP-27) to the caller instead of dropping it silently.
        CPIndex::add(self, id, bitset, vec_data, storage_offset)
    }

    fn estimate_memory_bytes(&self) -> usize {
        CPIndex::estimate_memory_bytes(self)
    }

    fn len(&self) -> usize {
        self.total_nodes.load(std::sync::atomic::Ordering::Relaxed) as usize
    }
}

#[cfg(test)]
mod tests;
