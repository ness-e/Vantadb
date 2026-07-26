//! HNSW index construction, serialization, and search operations.

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
