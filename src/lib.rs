//! # VantaDB — Embedded Persistent Memory Engine
//!
//! Embedded core for durable local memory, vector retrieval,
//! and structured fields.

pub(crate) mod backend;
pub(crate) mod backends;
pub mod columnar;
pub mod config;
#[cfg(feature = "cli")]
pub mod console;
pub mod engine;
pub mod error;
#[cfg(feature = "experimental")]
pub mod eval;
#[cfg(feature = "experimental")]
pub mod executor;
pub mod gc;
#[cfg(feature = "governance")]
pub mod governance;
pub mod governor;
#[cfg(feature = "experimental")]
pub mod graph;
pub mod hardware;
pub mod index;
pub mod integrations;
#[cfg(feature = "llm")]
pub mod llm;
pub mod metadata;
pub mod metrics;
pub mod node;
#[cfg(feature = "experimental")]
pub mod parser;
pub mod planner;
#[cfg(feature = "python_sdk")]
pub mod python;
pub mod query;
pub mod sdk;

pub mod storage;
pub(crate) mod text_index;
pub mod vector;
pub mod wal;

// Re-exports for ergonomic API
pub use engine::{EngineStats, InMemoryEngine, QueryResult, SourceType};
pub use error::{Result, VantaError};
pub use node::{
    DistanceMetric, Edge, FieldValue, NodeFlags, RelFields, UnifiedNode, VectorRepresentations,
};
pub use sdk::{
    VantaBm25TermContribution, VantaCapabilities, VantaEdgeRecord, VantaEmbedded,
    VantaExportReport, VantaFields, VantaHybridFusionReport, VantaImportReport,
    VantaIndexRebuildReport, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage,
    VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNodeInput, VantaNodeRecord, VantaOperationalMetrics, VantaQueryResult,
    VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit, VantaSearchHit,
    VantaStorageTier, VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};
pub use storage::BackendKind;
pub use wal::{WalReader, WalRecord, WalWriter};
