#![doc(html_root_url = "https://docs.rs/vantadb/0.3.0/vantadb/")]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_unsafe)]

//! # VantaDB — Embedded Persistent Memory Engine
//!
//! Durable local memory with vector (HNSW) and lexical (BM25) retrieval,
//! structured fields, property graphs, and a DSL query planner — all in
//! one embedded Rust crate.
//!
//! ## Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`VantaEmbedded`](sdk/struct.VantaEmbedded.html) | Top-level engine handle. Open/close, CRUD, search, graph ops. |
//! | [`InMemoryEngine`](engine/struct.InMemoryEngine.html) | In-memory engine with WAL persistence. |
//! | [`UnifiedNode`](node/struct.UnifiedNode.html) | Single node representation (fields, vector, edges, metadata). |
//! | [`VantaMemoryRecord`](sdk/struct.VantaMemoryRecord.html) | A stored memory record with namespace, key, payload, vector, metadata. |
//! | [`VantaError`](error/enum.VantaError.html) | Typed error enum covering validation, I/O, serialization, and engine errors. |
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `encryption` | AES-256-GCM at-rest encryption |
//! | `cli` | Interactive REPL and CLI commands |
//! | `server` | HTTP server (axum) |
//! | `arrow` | Apache Arrow columnar export |
//! | `python_sdk` | Python bindings (via PyO3) |
//! | `wal-shipping` | Async WAL shipping to replicas |
//! | `pitr` | Point-in-time recovery from WAL archives |
//! | `async-ingestion` | Background ingestion worker pool |
//! | `governance` | Governance policy engine |
//! | `remote-inference` | Remote LLM inference integration |
//!
//! ## Quick Example
//!
//! ```rust,no_run
//! use vantadb::sdk::{VantaEmbedded, VantaMemoryInput};
//! use vantadb::config::VantaConfig;
//!
//! let config = VantaConfig::default();
//! let engine = VantaEmbedded::open_with_config(config).unwrap();
//!
//! engine.put(VantaMemoryInput {
//!     namespace: "docs".into(),
//!     key: "example".into(),
//!     payload: "Hello, VantaDB!".into(),
//!     ..Default::default()
//! }).unwrap();
//!
//! let record = engine.get("docs", "example").unwrap();
//! assert_eq!(record.unwrap().payload, "Hello, VantaDB!");
//! engine.close().unwrap();
//! ```

/// AES-256-GCM at-rest encryption for storage files.
#[cfg(feature = "encryption")]
pub mod crypto;

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
#[cfg(feature = "governance")]
pub mod governance;
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
pub mod sync_ext;

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

/// Async WAL shipping to remote replica (behind feature "wal-shipping").
#[cfg(feature = "wal-shipping")]
pub mod wal_shipping;

/// WAL archival and point-in-time recovery (behind feature "pitr").
#[cfg(feature = "pitr")]
pub mod wal_archiver;

/// Async ingestion pipeline for offloading node insertion to a worker pool.
#[cfg(feature = "async-ingestion")]
pub mod ingestion;
/// Async transcript file I/O and processing.
#[cfg(feature = "async-io")]
pub mod transcript;

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
