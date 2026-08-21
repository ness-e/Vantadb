//! Context engine: token estimation, emergency truncation, compaction
//! report types. Foundation for MEM-22 (context compaction).
//!
//! LLM-free by construction (Principio 4). Consumes only std + serde +
//! thiserror; never touches the core `vantadb`.

mod compressor;
mod engine;
mod token_estimator;
mod types;

pub use compressor::{apply_boundary, msg_fingerprint, score_message, AggressiveBoundary};
pub use engine::{assemble, AssembleConfig, AssembleOutput};
pub use token_estimator::{emergency_truncate, truncate_content, TokenEstimator};
pub use types::{ChatMessage, ChatRole, CompactionMode, CompactionReport, ContextError};
