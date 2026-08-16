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
        _distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)> {
        // CPIndex already knows its distance metric from config;
        // the _distance_metric argument is accepted for trait compatibility
        // with index types that don't carry their own config (e.g. FlatIndex).
        self.search_nearest(query_vec, None, None, query_mask, top_k, vector_store)
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
