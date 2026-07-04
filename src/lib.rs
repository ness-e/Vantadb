#![doc(html_root_url = "https://docs.rs/vantadb/0.1.5/vantadb/")]
#![warn(missing_docs)]

//! # VantaDB — Embedded Persistent Memory Engine
//!
//! Embedded core for durable local memory, vector retrieval,
//! and structured fields.

pub(crate) mod backend;
pub(crate) mod backends;
/// Binary header format for all persisted VantaDB files.
pub mod binary_header;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod cli_handlers;
#[cfg(feature = "server")]
pub mod cli_server;
#[cfg(feature = "arrow")]
pub mod columnar;
pub mod config;
#[cfg(feature = "cli")]
pub mod console;
pub(crate) mod edge_index;
pub mod engine;
/// Core error types for all VantaDB operations.
pub mod error;
pub mod executor;
pub mod gc;
pub mod governor;
pub mod graph;
pub mod hardware;
pub mod index;
pub mod integrations;
#[cfg(feature = "remote-inference")]
pub mod llm;
pub(crate) mod memory_governor;
pub mod metadata;
pub mod metrics;
/// Database migration engine for format upgrades.
pub mod migration;
/// Core node, edge, and field value types.
pub mod node;
pub mod parser;
pub mod physical_plan;
pub mod planner;
#[cfg(feature = "python_sdk")]
pub mod python;
pub mod query;
pub(crate) mod rbac;
pub mod sdk;
pub mod serialization;

pub(crate) mod scalar_index;
/// Storage schema versioning and compatibility checks.
pub mod schema;
pub mod storage;
pub(crate) mod text_index;
#[cfg(feature = "advanced-tokenizer")]
pub mod tokenizer;
pub mod utils;
pub mod vector;
/// Write-ahead log reader, writer, and record types.
pub mod wal;
pub(crate) mod wal_sharded;

// Re-exports for ergonomic API
pub use binary_header::VantaHeader;
pub use engine::{EngineStats, InMemoryEngine, QueryResult, SourceType};
pub use error::{Result, VantaError};
pub use node::{
    DistanceMetric, Edge, FieldValue, NodeFlags, RelFields, UnifiedNode, VectorRepresentations,
};
pub use sdk::{
    connect, VantaBm25TermContribution, VantaCapabilities, VantaEdgeRecord, VantaEmbedded,
    VantaExportReport, VantaFields, VantaHybridFusionReport, VantaImportReport,
    VantaIndexRebuildReport, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage,
    VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNodeInput, VantaNodeRecord, VantaOperationalMetrics, VantaQueryResult,
    VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit, VantaSearchHit,
    VantaStorageTier, VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};
pub use storage::BackendKind;
pub use utils::compute_confidence_friction;
pub use wal::{WalReader, WalRecord, WalWriter};

#[cfg(feature = "failpoints")]
pub use fail::FailScenario;

/// Configure a failpoint by name with the given actions
#[cfg(feature = "failpoints")]
pub fn cfg_failpoint(name: &str, actions: &str) -> std::result::Result<(), String> {
    fail::cfg(name, actions).map_err(|e| format!("{:?}", e))
}

/// Remove a previously configured failpoint by name
#[cfg(feature = "failpoints")]
pub fn remove_failpoint(name: &str) {
    fail::remove(name);
}
