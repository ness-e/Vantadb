//! Edition-neutral contracts for the memory pipeline.
//!
//! Host-neutral, implementation-neutral: nothing here imports deployment
//! specifics. These are the data contracts that L0→L3 consume (extraction,
//! dedup, scenes, persona) plus the [`super::llm_runner::LlmRunner`]
//! abstraction. They mirror the TDAM record contracts
//! (`MemoryCore/src/core/record/l1-writer.ts`, `l1-dedup.ts`,
//! `l1-extractor.ts`) reimplemented in Rust — TDAM is a reference for the
//! model, never code to copy line-by-line.

mod llm_runner;
mod types;

#[cfg(feature = "llm-driver")]
pub use llm_runner::AsyncLlmRunner;
pub use llm_runner::{LlmError, LlmRunParams, LlmRunner};
pub use types::{
    DedupAction, DedupDecision, ExtractedMemory, L1ExtractionResult, MemoryRecord, MemoryType,
    PersonaMode, PersonaTriggerPriority, SceneIndexEntry, SceneMeta, SceneSegment,
};
