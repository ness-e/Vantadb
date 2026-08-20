//! LLM prompts for the memory pipeline (MEM-10, MEM-11).
//!
//! L1 extraction and conflict-detection prompts are REWRITTEN in English from
//! the TDAM principles (Principio 7: reescribir, no traducir). The TDAM
//! originals are Chinese and host-specific (`MemoryCore/src/core/prompts/
//! l1-extraction.ts`, `l1-dedup.ts`); this port restates the principles for a
//! host-neutral Rust pipeline.

/// L1 extraction prompt family.
pub mod l1_extraction;

/// L1 conflict-detection (dedup) prompt family (MEM-11).
pub mod l1_dedup;

pub use l1_dedup::{
    format_batch_conflict_prompt, get_conflict_detection_system_prompt, CandidateMatch,
};
pub use l1_extraction::{extract_memories_system_prompt, format_extraction_prompt, PromptMode};
