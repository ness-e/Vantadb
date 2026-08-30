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

/// L2 scene contracts + LLM-free scene index (MEM-12).
pub mod scene;

/// L3 persona: trigger heuristics + first/incremental generation (MEM-15).
pub mod persona;

/// Pipeline state contracts shared by managers, backends and the worker
/// (MEM-16).
pub mod state;

/// Skill extraction from transcripts + conversation-add pipeline with an
/// idempotent sink (MEM-17).
pub mod skill;

/// Auto-recall: prepend/append context injection with 3 recall modes
/// (MEM-18).
pub mod memory_prompt;

/// Team+agent profile scoping + persona sync (MEM-18).
pub mod profile;

/// Generation-log provenance for L1/L2/L3 generations, best-effort and
/// consultable per session/layer (MEM-41).
pub mod memory_generation_log;

/// MEM-61: Dreaming — idle consolidation (sleep-time tiering, Letta pattern).
/// Pure LLM-free primitives (`detect_idle`, `merge_duplicates`,
/// `resolve_contradictions`, `normalize_relative_dates`) + [`Dreamer`] trait
/// extension point. Writes a separate `dream/<session>/<run_id>` namespace —
/// the original `l1/<session>` store is **never** mutated.
pub mod dream;
