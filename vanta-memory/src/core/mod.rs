//! Core memory pipeline: L0 capture, L1 extraction/dedup, L2 scenes, L3
//! persona, skill extraction, recall, memory prompt. Host-neutral.
//!
//! Filled in by MEM-09..18, MEM-21 (see plan file).

/// Edition-neutral contracts: data types (L1 records, dedup decisions, scene
/// META, persona modes) + the host-neutral [`LlmRunner`] abstraction.
///
/// [`LlmRunner`]: abstractions::LlmRunner
pub mod abstractions;

/// L0 conversation capture: raw turn recording, idempotent and LLM-free
/// (MEM-09).
pub mod conversation;

/// Automatic capture hooks plugging the pipeline into a host conversation
/// stream (MEM-09).
pub mod hooks;

/// L1 memory extraction: quality gate, scene segmentation + extraction in one
/// LLM call, tolerant parse (MEM-10). Write/dedup lands in MEM-11.
pub mod record;

/// LLM prompts for the memory pipeline: L1 extraction (MEM-10).
pub mod prompts;
