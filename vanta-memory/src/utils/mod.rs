//! Orchestration utilities: pipeline manager, timers, locks, checkpointing.
//!
//! MEM-16: injectable clock ([`managed_timer`]), local state backend
//! ([`local_backend`]), timer scanner, persistent checkpoint and the
//! pipeline managers + factory.

pub mod backup;
pub mod checkpoint;
pub mod local_backend;
pub mod managed_timer;
pub mod pipeline_factory;
pub mod pipeline_manager;
pub mod sanitize;
pub mod stateful_pipeline_manager;
pub mod task_checkpoint;
pub mod text_utils;
pub mod timer_scanner;

pub use backup::{create_snapshot, list_snapshots, restore_snapshot};
pub use checkpoint::{Checkpoint, CheckpointError, CheckpointManager, RunnerSessionState};
pub use local_backend::{BackendSnapshot, LocalStateBackend, PipelineSessionStatePatch};
pub use managed_timer::{Clock, FakeClock, ManagedTimer, SystemClock};
pub use pipeline_manager::{l1_idle_member, MemoryPipelineManager, PipelineConfig};
pub use sanitize::{looks_like_prompt_injection, sanitize_text, should_capture_l0};
pub use stateful_pipeline_manager::StatefulPipelineManager;
pub use task_checkpoint::{
    TaskCheckpoint, TaskCheckpointError, TaskCheckpointManager, TaskState, TASK_CHECKPOINT_VERSION,
};
pub use text_utils::{pick_recent_unique, truncate_chars, truncate_with_suffix};
pub use timer_scanner::TimerScanner;
