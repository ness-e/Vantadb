//! LLM prompts for the memory pipeline (MEM-10).
//!
//! L1 extraction prompts are REWRITTEN in English from the TDAM extraction
//! principles (Principio 7: reescribir, no traducir). The TDAM originals are
//! Chinese and host-specific (`MemoryCore/src/core/prompts/l1-extraction.ts`);
//! this port restates the principles for a host-neutral Rust pipeline.

/// L1 extraction prompt family.
pub mod l1_extraction;

pub use l1_extraction::{extract_memories_system_prompt, format_extraction_prompt, PromptMode};
