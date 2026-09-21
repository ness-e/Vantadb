//! Context engine: token estimation, emergency truncation, compaction
//! report types. Foundation for MEM-22 (context compaction).
//!
//! LLM-free by construction (Principio 4). Consumes only std + serde +
//! thiserror; `mmd.rs` additionally persists through the VantaDB SDK.

mod compressor;
mod engine;
mod mmd;
mod mmd_injector;
mod report_store;
mod token_estimator;
mod types;

pub use compressor::{
    apply_boundary, build_memory_scores, msg_fingerprint, score_message, AggressiveBoundary,
    MemoryScoreMap,
};
pub use engine::{
    assemble, assemble_with_recall, AssembleConfig, AssembleOutput, IntegratedContext,
    RECALL_APPEND_MARKER, RECALL_PREPEND_MARKER,
};
pub use mmd::{
    fingerprint, list_history, load_active, push_history, save_active, TaskMemory,
    MAX_MMD_CONTENT_CHARS,
};
pub use mmd_injector::{inject_mmd, MMD_CONTEXT_MARKER};
pub use report_store::{
    list_compaction_reports, record_compaction_report, PersistedCompactionReport,
    COMPACTION_REPORT_PREFIX,
};
pub use token_estimator::{emergency_truncate, truncate_content, TokenEstimator};
pub use types::{ChatMessage, ChatRole, CompactionMode, CompactionReport, ContextError};
