//! Context offload: state manager, cursor, storage, after-tool-call hooks.
//!
//! Filled in by MEM-20 (see plan file).

/// Data contracts for context offload (cursor state, buffered tool pairs).
pub mod types;

/// Local-LLM offload: tolerant parsing of LLM-produced JSON (MEM-10).
pub mod local_llm;
