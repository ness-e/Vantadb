//! # ConnectomeDB — Neural-Inspired Multimodel Database for Local AI
//!
//! Unified engine for **Vector** (embeddings), **Graph** (edges),
//! and **Relational** (typed fields) data in a single storage layer.
//!
//! ConnectomeDB maps connections between data the way neurons connect
//! in a brain — unifying three paradigms in one local-first engine.
//!
//! ## Nomenclature (Biological Aliases)
//! - **Neuron** = `UnifiedNode` (the fundamental data unit)
//! - **Synapse** = `Edge` (weighted connection between neurons)
//! - **Cortex** = `LogicalPlan` (the query decision engine)

pub mod error;
pub mod node;
pub mod wal;
pub mod engine;
pub mod query;
pub mod parser;
pub mod eval;
pub mod storage;
pub mod index;
pub mod governor;
pub mod integrations;
pub mod executor;
pub mod graph;
pub mod server;
#[cfg(feature = "python_sdk")]
pub mod python;
pub mod columnar;
pub mod metrics;
pub mod gc;
pub mod governance;
pub mod llm;
pub mod hardware;

// Re-exports for ergonomic API
pub use error::{ConnectomeError, Result};
pub use node::{UnifiedNode, VectorData, Edge, FieldValue, NodeFlags, RelFields};
pub use node::{Neuron, Synapse}; // Biological aliases
pub use engine::{InMemoryEngine, EngineStats, QueryResult, SourceType};
pub use wal::{WalWriter, WalReader, WalRecord};
