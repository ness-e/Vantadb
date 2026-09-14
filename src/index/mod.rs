//! HNSW index construction, serialization, and search operations.
//! Defines `VecIndex` — the pluggable trait over all index backends.

pub mod auto_tune;
pub(crate) mod core; // tests only
pub(crate) mod diskann;
pub(crate) mod distance;
pub(crate) mod flat;
pub(crate) mod graph;

pub mod ivf;
pub(crate) mod neighbor_index;
pub(crate) mod port_impl;
pub(crate) mod refresh;
pub(crate) mod scann;
pub(crate) mod search;
pub(crate) mod serialize;
pub(crate) mod stats;

pub use distance::*;
pub use graph::*;

/// Re-exported from the neutral leaf (`crate::index_port`): routing discriminant
/// shared by both sides of the storage↔index boundary (F3X; BND-04 compat).
pub use crate::index_port::IndexType;

/// Pluggable trait for vector index backends.
///
/// Each backend (HNSW, IVF, flat scan) exposes the same search/add/lifecycle
/// interface so that callers like [`StorageEngine`](crate::storage::engine::StorageEngine)
/// can operate on any index through `Arc<dyn VecIndex>`.
///
/// # ponytail
/// Minimal surface — only methods needed today. Add when a second caller
/// actually requires the new method.
pub(crate) trait VecIndex: Send + Sync {
    /// Search the index for the `top_k` nearest neighbors of `query_vec`.
    ///
    /// Returns a `Vec<(node_id, score)>` sorted by descending similarity.
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &crate::node::FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn crate::index_port::VectorStoreRef>,
        distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)>;

    /// Add a single node to the index.
    ///
    /// Returns an error when the insert is rejected (e.g. non-full vector
    /// under Scann/DiskAnn, read-only IVF after build) so callers can
    /// propagate instead of silently dropping the add (ERR-031).
    #[allow(dead_code)]
    fn add(
        &self,
        id: u128,
        bitset: crate::node::FilterBitset,
        vec_data: crate::node::VectorRepresentations,
        storage_offset: u64,
    ) -> crate::error::Result<()>;

    /// Estimated heap memory usage in bytes.
    #[allow(dead_code)]
    fn estimate_memory_bytes(&self) -> usize;

    /// Number of indexed nodes.
    #[allow(dead_code)]
    fn len(&self) -> usize;

    /// Returns `true` if the index has no nodes.
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Create a vector index of the specified type.
///
/// Returns an `Arc<dyn VecIndex>` wrapping the concrete implementation.
///
/// # ponytail
/// Default configuration for each type. Use `new_with_config` on the
/// concrete structs for custom parameters.
#[allow(dead_code)]
pub(crate) fn create_index(
    index_type: IndexType,
    distance_metric: crate::node::DistanceMetric,
) -> std::sync::Arc<dyn VecIndex> {
    match index_type {
        IndexType::Hnsw => {
            use crate::index::graph::{CPIndex, HnswConfig};
            std::sync::Arc::new(CPIndex::new_with_config(HnswConfig {
                distance_metric,
                ..HnswConfig::default()
            }))
        }
        IndexType::Ivf => {
            use crate::index::ivf::{IvfConfig, IvfIndex};
            std::sync::Arc::new(IvfIndex {
                centroids: Vec::new(),
                inverted_lists: Vec::new(),
                config: IvfConfig {
                    distance_metric,
                    ..IvfConfig::default()
                },
            })
        }
        IndexType::Flat => {
            use crate::index::flat::FlatIndex;
            std::sync::Arc::new(FlatIndex::new(distance_metric))
        }
        IndexType::DiskAnn => {
            use crate::index::diskann::{DiskAnnConfig, DiskAnnIndex};
            std::sync::Arc::new(DiskAnnIndex::new(DiskAnnConfig {
                distance_metric,
                ..DiskAnnConfig::default()
            }))
        }
        IndexType::Scann => {
            use crate::index::scann::ScannIndex;
            std::sync::Arc::new(ScannIndex::new(distance_metric))
        }
    }
}

#[cfg(test)]
mod index_type_tests {
    use super::create_index;
    use crate::node::DistanceMetric;

    #[test]
    fn test_create_hnsw_index() {
        let idx = create_index(super::IndexType::Hnsw, DistanceMetric::Cosine);
        assert_eq!(idx.len(), 0);
        assert!(idx.is_empty());
    }

    #[test]
    fn test_create_flat_index() {
        let idx = create_index(super::IndexType::Flat, DistanceMetric::Cosine);
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_create_ivf_index() {
        let idx = create_index(super::IndexType::Ivf, DistanceMetric::Cosine);
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_create_diskann_index() {
        let idx = create_index(super::IndexType::DiskAnn, DistanceMetric::Cosine);
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_create_scann_index() {
        let idx = create_index(super::IndexType::Scann, DistanceMetric::Cosine);
        assert_eq!(idx.len(), 0);
    }
}
