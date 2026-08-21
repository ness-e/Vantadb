//! Context offload: state manager, cursor, storage, after-tool-call hooks.
//!
//! Filled in by MEM-20 (see plan file).

/// Data contracts for context offload (cursor state, buffered tool pairs).
pub mod types;

/// Persistent per-session offload state + L3 cursor (`lastOffloadedToolCallId`).
pub mod state_manager;

/// Storage of offloaded tool-call entries (`OffloadEntry`) per session.
pub mod storage;

/// Post-tool-call hook: size-threshold offload decision.
pub mod hooks;

pub use state_manager::OffloadError;

/// Local-LLM offload: tolerant parsing of LLM-produced JSON (MEM-10).
pub mod local_llm;
