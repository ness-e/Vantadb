//! Pipeline state types (port of TDAM `MC/core/state/types.ts`, reimplemented
//! in Rust — MEM-16).
//!
//! These are the data contracts shared by [`crate::utils::local_backend`],
//! the pipeline managers and [`crate::services::pipeline_worker`]. The TDAM
//! `IStateBackend` trait is intentionally NOT ported: there is exactly one
//! backend (local, in-process) — Redis is banned by Principio 7. Extract a
//! trait when a second backend actually exists.

use serde::{Deserialize, Serialize};

/// Per-session pipeline state managed by the managers (TDAM
/// `PipelineSessionState`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PipelineSessionState {
    /// Conversation rounds since the last L1 trigger.
    pub conversation_count: u64,
    /// ISO timestamp of the last L1 extraction completion ("" = never).
    pub last_extraction_time: String,
    /// ISO timestamp cursor for incremental extraction reads ("" = never).
    pub last_extraction_updated_time: String,
    /// Epoch ms of the last capture notification.
    pub last_active_time_ms: u64,
    /// Mirrors `conversation_count` at L1 completion time (L2 tracking).
    pub l2_pending_l1_count: u64,
    /// Warm-up threshold for L1 triggering; 0 = graduated (use the configured
    /// interval directly). Doubles 1→2→4→… after each L1 completion.
    pub warmup_threshold: u64,
    /// ISO timestamp of the last L2 extraction completion ("" = never).
    pub l2_last_extraction_time: String,
}

/// Pipeline task kinds. Offload variants of the TDAM `TaskPayload.type` are
/// not ported (offload orchestration is F6+, deferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskKind {
    /// L1 memory extraction over buffered L0 messages.
    L1,
    /// L2 scene extraction over stored memories.
    L2,
    /// L3 persona trigger evaluation + generation.
    L3,
    /// Force-drain a session buffer regardless of threshold.
    Flush,
}

/// One unit of pipeline work flowing through the state backend queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskPayload {
    /// Stable task id (`t_{created_at_ms}_{seq}`).
    pub id: String,
    /// What the worker should run.
    pub kind: TaskKind,
    /// Session this task belongs to (lock granularity).
    pub session_id: String,
    /// 0 = high, 1 = normal, 2 = low (lower value runs first).
    pub priority: u8,
    /// Epoch ms creation time (tie-breaker within a priority).
    pub created_at_ms: u64,
    /// Execution attempts so far (worker-managed; not part of identity).
    #[serde(default)]
    pub attempts: u32,
}

/// A timer registered in the backend (TDAM `TimerEntry`; the instance-id
/// routing field is not needed in-process).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimerEntry {
    /// Timer member name (e.g. `l1_idle:<session>`).
    pub member: String,
    /// Absolute epoch ms when the timer fires.
    pub fire_at_ms: u64,
}

/// Parameters for the atomic capture operation (buffer append + counter bump +
/// threshold check + timer set in one critical section).
#[derive(Debug, Clone)]
pub struct CaptureAtomicParams {
    /// Session being captured.
    pub session_id: String,
    /// Serialized message to append to the buffer (optional).
    pub message_json: Option<String>,
    /// Conversation count that triggers an immediate L1 enqueue.
    pub threshold: u64,
    /// When the idle timer should fire if the threshold is not reached.
    pub fire_at_ms: u64,
    /// Idle timer member name.
    pub timer_member: String,
    /// Task enqueued when the threshold fires.
    pub task: TaskPayload,
    /// Current time (injected — keeps the operation deterministic).
    pub now_ms: u64,
    /// Rounds added by this capture (default 1).
    pub rounds: u64,
}

/// Result of the atomic capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureAtomicResult {
    /// Whether the threshold fired and an L1 task was enqueued.
    pub triggered: bool,
    /// Conversation count after the capture (0 when triggered).
    pub conversation_count: u64,
}
