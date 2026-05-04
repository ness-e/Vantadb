//! # VantaDB — Embedded Persistent Memory Engine
//!
//! Embedded core for durable local memory, vector retrieval,
//! and structured fields.

pub mod api;
pub(crate) mod backend;
pub(crate) mod backends;
pub mod columnar;
pub mod console;
pub mod engine;
pub mod error;
pub mod eval;
pub mod executor;
pub mod gc;
pub mod governance;
pub mod governor;
pub mod graph;
pub mod hardware;
pub mod index;
pub mod integrations;
pub mod llm;
pub mod metrics;
pub mod node;
pub mod parser;
#[cfg(feature = "python_sdk")]
pub mod python;
pub mod query;
pub mod sdk;
pub mod server;
pub mod storage;
pub(crate) mod text_index;
pub mod vector;
pub mod wal;

// Re-exports for ergonomic API
pub use engine::{EngineStats, InMemoryEngine, QueryResult, SourceType};
pub use error::{Result, VantaError};
pub use node::{Edge, FieldValue, NodeFlags, RelFields, UnifiedNode, VectorRepresentations};
pub use sdk::{
    VantaCapabilities, VantaEdgeRecord, VantaEmbedded, VantaExportReport, VantaFields,
    VantaImportReport, VantaIndexRebuildReport, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryListPage, VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit,
    VantaMemorySearchRequest, VantaNodeInput, VantaNodeRecord, VantaOpenOptions,
    VantaOperationalMetrics, VantaQueryResult, VantaRuntimeProfile, VantaSearchHit,
    VantaStorageTier, VantaValue,
};
pub use storage::BackendKind;
pub use wal::{WalReader, WalRecord, WalWriter};
