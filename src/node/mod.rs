//! Core multimodel node types: vector representations, graph edges, relational
//! fields, tiering flags, and the unified in-memory node.
//!
//! Split from the original 2 078-line god module into focused submodules
//! (REVIEW-04). The public surface is unchanged: `lib.rs` re-exports the same
//! types, and internal `crate::node::*` paths keep resolving.

mod bitset;
mod disk;
mod edge;
mod field;
mod flags;
mod label;
mod unified;
mod vector_data;

// Public surface (mirrors pre-split `pub` items)
pub use bitset::{FilterBitset, ALL_BITSET};
pub use disk::DiskNodeHeader;
pub use edge::{Edge, EvictionWeights};
pub use field::{FieldValue, RelFields};
pub use flags::{AccessTracker, NodeFlags, NodeTier};
pub use unified::UnifiedNode;
pub use vector_data::{DistanceMetric, SparseVector, VectorRepresentations};

// Internal re-exports used by crate::node::* paths outside this module
pub(crate) use label::LabelIntern;
