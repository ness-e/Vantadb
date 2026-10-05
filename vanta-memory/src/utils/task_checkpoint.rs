//! Resumable task checkpoints (dim 1 working memory — MEMG-20).
//!
//! A **task checkpoint** tracks one host task across steps so a new instance
//! can resume after an interruption without repeating completed steps. It is
//! deliberately a separate domain from the pipeline checkpoint
//! ([`super::checkpoint`], TDAM `Checkpoint`): separate types, separate
//! namespace (`task_checkpoints`, never `pipeline_checkpoint`), no shared
//! fields. The pipeline checkpoint tracks cursors/counters of the L0→L3
//! pipeline; this module tracks `step` + partial results of a host task.
//!
//! Persistence follows the module principle (Principio 2): one JSON record
//! per task under namespace `task_checkpoints` (key = sanitized `task_id`),
//! mutated through read-modify-write calls — single-writer per `task_id`
//! keeps mutations atomic in-process. This module is **not** a task engine:
//! it stores progress, it does not execute or orchestrate steps.
//!
//! **Host limit:** resuming after the agent's context compaction is the
//! host's concern — this manager persists the *task* state (step + partial
//! results), not the agent's conversational context (that lives in
//! [`crate::context_engine`]). Checkpoint granularity is one entry per
//! completed step; mid-step progress is not persisted.
//!
//! **At-least-once resume:** a crash between a step's side effect and its
//! `advance` re-runs that step on resume — host steps should be idempotent
//! (or reconciled) under resume. Task ids are sanitized like every key in
//! the crate (`sanitize_key`: unsafe characters → `_`, ≤512 bytes); keep ids
//! within the safe set to avoid silent collisions (`a/b` → `a_b`).
//!
//! ```no_run
//! use vanta_memory::utils::{TaskCheckpointManager, TaskState};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let config = vantadb::config::Config {
//! #     backend_kind: vantadb::storage::BackendKind::InMemory,
//! #     ..Default::default()
//! # };
//! # let db = vantadb::sdk::Embedded::open_with_config(config)?;
//! let checkpoints = TaskCheckpointManager::new(&db);
//!
//! // Resume-safe start: returns the existing checkpoint when one is already
//! // there (idempotent), a fresh step-0 checkpoint otherwise.
//! let cp = checkpoints.begin("task-1")?;
//! for step in cp.step..3 {
//!     let result = serde_json::json!({ "step": step }); // host work…
//!     checkpoints.advance("task-1", result)?; // step += 1, partial preserved
//! }
//! checkpoints.complete("task-1")?;
//! assert_eq!(checkpoints.load("task-1")?.map(|c| c.state), Some(TaskState::Completed));
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::core::conversation::{sanitize_component, sanitize_key};

/// Schema version written into every [`TaskCheckpoint`] record.
pub const TASK_CHECKPOINT_VERSION: u32 = 1;

/// Lifecycle state of the host task tracked by a [`TaskCheckpoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TaskState {
    /// Steps remain to run; a fresh instance resumes at [`TaskCheckpoint::step`].
    InProgress,
    /// All steps finished; nothing to resume.
    Completed,
    /// The task failed; the checkpoint (with its partial results) is kept for
    /// inspection or retry by the host.
    Failed,
}

/// Resumable checkpoint of one host task (dim 1 working memory).
///
/// `step` is the **next step to run**: steps `< step` are already completed
/// and their results are in `partial` (`partial[i]` is step `i`'s result, so
/// `partial.len() == step` while the checkpoint is advanced through
/// [`TaskCheckpointManager`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskCheckpoint {
    /// Schema version of this record ([`TASK_CHECKPOINT_VERSION`] at write time).
    pub version: u32,
    /// Next step to run (0-based); steps `< step` are completed.
    pub step: u64,
    /// Lifecycle state of the task.
    pub state: TaskState,
    /// One entry per completed step, in order.
    pub partial: Vec<serde_json::Value>,
}

/// Errors surfaced by the task checkpoint manager.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TaskCheckpointError {
    /// Underlying store error.
    #[error("task checkpoint store: {0}")]
    Store(#[from] vantadb::error::Error),
    /// Serialization error.
    #[error("task checkpoint serialization: {0}")]
    Serde(#[from] serde_json::Error),
    /// No checkpoint exists for the task (`advance`/`complete`/`fail` before
    /// `begin`).
    #[error("task checkpoint not found: {0}")]
    NotFound(String),
    /// The checkpoint is not in progress (`advance`/`complete`/`fail` on a
    /// completed or failed task).
    #[error("task checkpoint for '{task_id}' is {state:?}, not in progress")]
    NotInProgress { task_id: String, state: TaskState },
}

/// Persistent task checkpoint manager over the VantaDB store.
///
/// One JSON record per task under namespace `task_checkpoints` (key =
/// sanitized `task_id`). Mutations are read-modify-write calls: keep a single
/// writer per `task_id` (same in-process atomicity discipline as the pipeline
/// [`CheckpointManager`](super::checkpoint::CheckpointManager)). Keys go
/// through the crate's key sanitization (safe set `[A-Za-z0-9._-]`,
/// ≤512 bytes) — keep `task_id`s within it to avoid silent collisions
/// (`a/b` and `a_b` would share a record).
pub struct TaskCheckpointManager<'a> {
    db: &'a vantadb::sdk::Embedded,
    namespace: String,
}

impl<'a> TaskCheckpointManager<'a> {
    /// Open a manager over an embedded database.
    pub fn new(db: &'a vantadb::sdk::Embedded) -> Self {
        Self {
            db,
            namespace: format!("task_{}", sanitize_component("checkpoints", 128, false)),
        }
    }

    /// Read one task checkpoint (`None` when the task was never begun).
    pub fn load(&self, task_id: &str) -> Result<Option<TaskCheckpoint>, TaskCheckpointError> {
        match self.db.get(&self.namespace, &sanitize_key(task_id))? {
            Some(record) => Ok(Some(serde_json::from_str(&record.payload)?)),
            None => Ok(None),
        }
    }

    /// Ensure a checkpoint exists and return it: creates a fresh one at step 0
    /// when absent, or returns the existing record unchanged (idempotent — a
    /// re-run never clobbers in-progress progress). Deliberate restart =
    /// [`Self::delete`] + `begin`.
    pub fn begin(&self, task_id: &str) -> Result<TaskCheckpoint, TaskCheckpointError> {
        if let Some(existing) = self.load(task_id)? {
            return Ok(existing);
        }
        let checkpoint = TaskCheckpoint {
            version: TASK_CHECKPOINT_VERSION,
            step: 0,
            state: TaskState::InProgress,
            partial: Vec::new(),
        };
        self.write(task_id, &checkpoint)?;
        Ok(checkpoint)
    }

    /// Record one completed step: appends `step_result` to the partial
    /// results and advances `step` by one (read-modify-write).
    pub fn advance(
        &self,
        task_id: &str,
        step_result: serde_json::Value,
    ) -> Result<TaskCheckpoint, TaskCheckpointError> {
        self.mutate(task_id, |cp| {
            cp.partial.push(step_result);
            cp.step += 1;
        })
    }

    /// Mark the task as completed (read-modify-write).
    pub fn complete(&self, task_id: &str) -> Result<TaskCheckpoint, TaskCheckpointError> {
        self.mutate(task_id, |cp| cp.state = TaskState::Completed)
    }

    /// Mark the task as failed, keeping its partial results for inspection
    /// (read-modify-write).
    pub fn fail(&self, task_id: &str) -> Result<TaskCheckpoint, TaskCheckpointError> {
        self.mutate(task_id, |cp| cp.state = TaskState::Failed)
    }

    /// Remove the checkpoint. Idempotent: deleting a missing checkpoint is a
    /// no-op.
    pub fn delete(&self, task_id: &str) -> Result<(), TaskCheckpointError> {
        self.db.delete(&self.namespace, &sanitize_key(task_id))?;
        Ok(())
    }

    /// Read-modify-write on an in-progress checkpoint.
    fn mutate(
        &self,
        task_id: &str,
        f: impl FnOnce(&mut TaskCheckpoint),
    ) -> Result<TaskCheckpoint, TaskCheckpointError> {
        let mut checkpoint = self
            .load(task_id)?
            .ok_or_else(|| TaskCheckpointError::NotFound(task_id.to_string()))?;
        if checkpoint.state != TaskState::InProgress {
            return Err(TaskCheckpointError::NotInProgress {
                task_id: task_id.to_string(),
                state: checkpoint.state,
            });
        }
        f(&mut checkpoint);
        self.write(task_id, &checkpoint)?;
        Ok(checkpoint)
    }

    /// Persist a full checkpoint record.
    fn write(&self, task_id: &str, checkpoint: &TaskCheckpoint) -> Result<(), TaskCheckpointError> {
        use vantadb::sdk::{MemoryInput, MemoryMetadata};
        self.db.put(MemoryInput {
            namespace: self.namespace.clone(),
            key: sanitize_key(task_id),
            payload: serde_json::to_string(checkpoint)?,
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })?;
        Ok(())
    }
}
