//! Skill extraction + conversation-add pipeline (MEM-17, F4).
//!
//! Port of TDAM `MC/core/skill/skill-extractor.ts` (587) +
//! `MC/core/skill/conversation-add/*` + prompts, reduced to the functional
//! core a single-process Rust pipeline exercises (see task file MEM-17.md for
//! the documented ponytail deviations: no Redis queue/worker-pool/wire —
//! dispatch is MEM-16's [`crate::services::pipeline_worker::PipelineWorker`];
//! persistence is VantaDB records, never COS JSONL).
//!
//! Pipeline: buffer messages → compress tool payloads → oversize fallback →
//! trigger (archive FIRST, then task entry) → worker (extract via LLM review
//! prompt) → sink (IDEMPOTENT: per-task cursor + content-hash upsert).
//!
//! **MEM-64 — skill_versions + CompactionReport:** the sink writes an
//! append-only `skill_versions` history per `(scope, name)` to
//! `skills_extract/{scope}/_versions/{name}/{version_seq}`; the context
//! engine persists a [`crate::context_engine::CompactionReport`] per run at
//! `context/{session}/compaction_reports/{run_id}`.

pub mod conversation_add;
pub mod prompts;
pub mod skill_extractor;

pub use conversation_add::{
    apply_oversize_strategy, compress_message, prepare_archive_payload, run_skill_extract_once,
    trigger_archive, ArchiveStore, OversizeOptions, OversizeResult, SkillCoreSink, SkillMessage,
    SkillSinkCounts, SkillTaskEntry, SkillVersion, SkillWorkerOutcome, TriggerResult,
    COMPRESS_DEFAULTS, OVERSIZE_DEFAULTS,
};
pub use skill_extractor::{
    extract_skills_with_llm, format_transcript, sanitize_generated_query, truncate_head_tail,
    ExtractMessage, ExtractedSkillCandidate, SkillExtractionResult, SkillExtractorConfig,
    SkillSummary,
};
