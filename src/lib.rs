//! # IADBMS — Multimodel Database for Local AI
//!
//! Unified engine for **Vector** (embeddings), **Graph** (edges),
//! and **Relational** (typed fields) data in a single storage layer.
//!
//! ## Fase 1: In-memory engine + WAL (bincode)
//! ## Fase 2: RocksDB + Parser (Nom) + CBO
//! ## Fase 3: CP-Index (HNSW+bitset) + Ollama integration

pub mod error;
pub mod node;
pub mod wal;
pub mod engine;
pub mod query;
pub mod parser;
pub mod storage;
pub mod index;
pub mod governor;
pub mod integrations;

// Re-exports for ergonomic API
pub use error::{IadbmsError, Result};
pub use node::{UnifiedNode, VectorData, Edge, FieldValue, NodeFlags, RelFields};
pub use engine::{InMemoryEngine, EngineStats, QueryResult, SourceType};
pub use wal::{WalWriter, WalReader, WalRecord};
