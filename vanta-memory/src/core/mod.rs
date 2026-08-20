//! Core memory pipeline: L0 capture, L1 extraction/dedup, L2 scenes, L3
//! persona, skill extraction, recall, memory prompt. Host-neutral.
//!
//! Filled in by MEM-09..18, MEM-21 (see plan file).

/// Edition-neutral contracts: data types (L1 records, dedup decisions, scene
/// META, persona modes) + the host-neutral [`LlmRunner`] abstraction.
///
/// [`LlmRunner`]: abstractions::LlmRunner
pub mod abstractions;
