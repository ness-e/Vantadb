//! Storage engine: persistent vector store, WAL, HNSW index coordination.

pub mod archive;
pub(crate) mod engine;
pub mod lock;
pub(crate) mod ops;
pub mod vfile;
pub(crate) mod vfile_crypto;
pub(crate) mod vfile_mmap;
pub(crate) mod wal;

// Re-export public types from engine
pub use engine::{
    BackendKind, BackendPartition, BatchInsertOptions, EvictionReport, FsSnapshot,
    IndexRebuildReport, InsertMode, MemoryStats, MergeReport, PipelineMode, PipelineReport,
    StorageEngine, VacuumReport,
};
