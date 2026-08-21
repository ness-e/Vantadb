//! Conversation→skill pipeline (MEM-17) — port of TDAM
//! `MC/core/skill/conversation-add/*` consolidated (see task file MEM-17.md
//! for the documented deviations: no Redis queue/pool/wire; VantaDB records
//! instead of COS JSONL).

pub mod archive;
pub mod compressor;
pub mod oversize;
pub mod sink;
pub mod worker;

pub use archive::{
    archive_namespace, prepare_archive_payload, tasks_namespace, trigger_archive,
    trigger_archive_now, ArchiveStore, PreparedPayload, SkillArchiveError, SkillTaskEntry,
    TriggerResult,
};
pub use compressor::{
    compress_message, compress_messages, CompressOptions, SkillMessage, COMPRESS_DEFAULTS,
};
pub use oversize::{apply_oversize_strategy, OversizeOptions, OversizeResult, OVERSIZE_DEFAULTS};
pub use sink::{SkillCoreSink, SkillSinkCounts, StoredSkill};
pub use worker::{run_skill_extract_once, SkillWorkerOutcome};
