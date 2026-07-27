//! HNSW index construction, serialization, and search operations.
//! Defines [`VecIndex`] — the pluggable trait over all index backends.

pub(crate) mod auto_tune;
pub(crate) mod core; // tests only
pub(crate) mod distance;
pub(crate) mod flat;
pub(crate) mod graph;

pub(crate) mod ivf;

pub(crate) mod refresh;
pub(crate) mod search;
pub(crate) mod serialize;
pub(crate) mod stats;

use crate::storage::vfile::VantaFile;

pub use distance::*;
pub use graph::*;

/// Supported vector index types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IndexType {
    /// Hierarchical Navigable Small World graph index (default).
    #[default]
    Hnsw,
    /// Inverted File index with flat (brute-force) encoding.
    Ivf,
}

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
        vector_store: Option<&VantaFile>,
        distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)>;

    /// Add a single node to the index.
    #[allow(dead_code)]
    fn add(
        &self,
        id: u128,
        bitset: crate::node::FilterBitset,
        vec_data: crate::node::VectorRepresentations,
        storage_offset: u64,
    );

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
