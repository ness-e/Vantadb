//! Orchestration utilities: pipeline manager, timers, locks, checkpointing.
//!
//! MEM-16: injectable clock ([`managed_timer`]), local state backend
//! ([`local_backend`]), timer scanner, persistent checkpoint and the
//! pipeline managers + factory.

pub mod checkpoint;
pub mod local_backend;
pub mod managed_timer;
pub mod pipeline_factory;
pub mod pipeline_manager;
pub mod stateful_pipeline_manager;
pub mod timer_scanner;

pub use checkpoint::{Checkpoint, CheckpointError, CheckpointManager, RunnerSessionState};
pub use local_backend::{BackendSnapshot, LocalStateBackend, PipelineSessionStatePatch};
pub use managed_timer::{Clock, FakeClock, ManagedTimer, SystemClock};
pub use pipeline_manager::{l1_idle_member, MemoryPipelineManager, PipelineConfig};
pub use stateful_pipeline_manager::StatefulPipelineManager;
pub use timer_scanner::TimerScanner;
