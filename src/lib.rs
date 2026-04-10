//! # VantaDB — Embedded Multimodal Database Engine
//!
//! Unified engine for **Vector** (embeddings), **Graph** (edges),
//! and **Relational** (typed fields) data in a single storage layer.

pub mod api;
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
pub mod server;
pub mod storage;
pub mod vector;
pub mod wal;

// Re-exports for ergonomic API
pub use engine::{EngineStats, InMemoryEngine, QueryResult, SourceType};
pub use error::{Result, VantaError};
pub use node::{Edge, FieldValue, NodeFlags, RelFields, UnifiedNode, VectorRepresentations};
pub use wal::{WalReader, WalRecord, WalWriter};
