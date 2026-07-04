//! Storage engine: persistent vector store, WAL, HNSW index coordination.

pub(crate) mod archive;
pub(crate) mod engine;
pub(crate) mod ops;
pub(crate) mod vfile;
pub(crate) mod wal;

// Re-export public types from engine
pub use engine::{
    BackendKind, BackendPartition, EvictionReport, IndexRebuildReport, MemoryStats, StorageEngine,
};
