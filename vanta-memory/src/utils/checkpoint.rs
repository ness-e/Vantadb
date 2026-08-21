//! Checkpoint manager (port of TDAM `MC/utils/checkpoint.ts`, reimplemented
//! over the VantaDB store — MEM-16).
//!
//! TDAM persists a JSON file with per-file promise locks and optional
//! distributed locks. Here persistence is the VantaDB store itself
//! (Principio 2): one JSON record under namespace `pipeline_checkpoint`.
//! Single-record read-modify-write keeps mutations atomic in-process.
//!
//! This module pays the MEM-15 debt: [`Checkpoint`] exposes the counters that
//! [`crate::core::persona::evaluate_persona_trigger`] consumes as pure input
//! (`request_persona_update`, `scenes_processed`,
//! `memories_since_last_persona`, `last_persona_at`).

use serde::{Deserialize, Serialize};

use crate::core::conversation::{sanitize_component, sanitize_key};
use crate::core::state::PipelineSessionState;

/// Per-session runner state (TDAM `RunnerSessionState`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RunnerSessionState {
    /// Epoch ms of the newest L0 message captured for this session.
    pub last_captured_timestamp: u64,
    /// Epoch ms of the last message processed by L1.
    pub last_l1_cursor: u64,
    /// Last scene name from the most recent L1 extraction (continuity).
    pub last_scene_name: String,
}

/// Global pipeline checkpoint (TDAM `Checkpoint`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Checkpoint {
    /// Total messages processed across all time.
    pub total_processed: u64,
    /// Epoch ms of the last persona generation (0 = never).
    pub last_persona_at: u64,
    /// RFC3339 of the last persona generation ("" = never).
    pub last_persona_time: String,
    /// The agent explicitly requested a persona update (P1 trigger input).
    pub request_persona_update: bool,
    /// Why the update was requested.
    pub persona_update_reason: String,
    /// Memories stored since the last persona generation (P4 trigger input).
    pub memories_since_last_persona: u64,
    /// Scenes processed so far (P2/P3 trigger input).
    pub scenes_processed: u64,
    /// Total L1 memories extracted across all time.
    pub total_memories_extracted: u64,
    /// Per-session runner state (L0/L1 cursors).
    pub runner_states: std::collections::BTreeMap<String, RunnerSessionState>,
    /// Per-session manager state (written via [`CheckpointManager::merge_pipeline_states`]).
    pub pipeline_states: std::collections::BTreeMap<String, PipelineSessionState>,
}

/// Errors surfaced by the checkpoint manager.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CheckpointError {
    /// Underlying store error.
    #[error("checkpoint store: {0}")]
    Store(#[from] vantadb::error::VantaError),
    /// Serialization error.
    #[error("checkpoint serialization: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Persistent checkpoint manager over the VantaDB store.
pub struct CheckpointManager<'a> {
    db: &'a vantadb::sdk::VantaEmbedded,
    namespace: String,
    key: String,
}

impl<'a> CheckpointManager<'a> {
    /// Open a manager over an embedded database. The namespace is sanitized
    /// to the safe namespace set (`[A-Za-z0-9._/-]`, ≤128 bytes).
    pub fn new(db: &'a vantadb::sdk::VantaEmbedded) -> Self {
        Self {
            db,
            namespace: format!("pipeline_{}", sanitize_component("checkpoint", 128, false)),
            key: sanitize_key("checkpoint.json"),
        }
    }

    /// Read the checkpoint (defaults when nothing was persisted yet).
    pub fn read(&self) -> Result<Checkpoint, CheckpointError> {
        match self.db.get(&self.namespace, &self.key)? {
            Some(record) => Ok(serde_json::from_str(&record.payload)?),
            None => Ok(Checkpoint::default()),
        }
    }

    /// Persist a full checkpoint.
    pub fn write(&self, checkpoint: &Checkpoint) -> Result<(), CheckpointError> {
        use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata};
        self.db.put(VantaMemoryInput {
            namespace: self.namespace.clone(),
            key: self.key.clone(),
            payload: serde_json::to_string(checkpoint)?,
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }

    /// Read-modify-write helper.
    fn mutate(&self, f: impl FnOnce(&mut Checkpoint)) -> Result<Checkpoint, CheckpointError> {
        let mut checkpoint = self.read()?;
        f(&mut checkpoint);
        self.write(&checkpoint)?;
        Ok(checkpoint)
    }

    /// Record a persona generation and reset its counters (TDAM
    /// `markPersonaGenerated`).
    pub fn mark_persona_generated(
        &self,
        total_processed: u64,
        now_ms: u64,
        rfc3339: &str,
    ) -> Result<Checkpoint, CheckpointError> {
        self.mutate(|cp| {
            cp.last_persona_at = now_ms;
            cp.last_persona_time = rfc3339.to_string();
            cp.memories_since_last_persona = 0;
            cp.request_persona_update = false;
            cp.persona_update_reason.clear();
            cp.total_processed = total_processed;
        })
    }

    /// Flag an explicit agent persona-update request (P1).
    pub fn set_persona_update_request(&self, reason: &str) -> Result<(), CheckpointError> {
        self.mutate(|cp| {
            cp.request_persona_update = true;
            cp.persona_update_reason = reason.to_string();
        })
        .map(|_| ())
    }

    /// Clear the explicit request flag without generating.
    pub fn clear_persona_request(&self) -> Result<(), CheckpointError> {
        self.mutate(|cp| {
            cp.request_persona_update = false;
            cp.persona_update_reason.clear();
        })
        .map(|_| ())
    }

    /// Count one more processed scene (P2/P3 trigger input).
    pub fn increment_scenes_processed(&self) -> Result<u64, CheckpointError> {
        let cp = self.mutate(|cp| cp.scenes_processed += 1)?;
        Ok(cp.scenes_processed)
    }

    /// Add stored-memory counts after an L1 run.
    pub fn add_memories_extracted(&self, stored_count: u64) -> Result<(), CheckpointError> {
        self.mutate(|cp| {
            cp.total_memories_extracted += stored_count;
            cp.memories_since_last_persona += stored_count;
        })
        .map(|_| ())
    }

    /// Get (or default) the runner state of a session.
    pub fn get_runner_state(
        &self,
        checkpoint: &Checkpoint,
        session_id: &str,
    ) -> RunnerSessionState {
        checkpoint
            .runner_states
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Merge manager-owned per-session states into the checkpoint (TDAM
    /// `mergePipelineStates`; only these fields are touched).
    pub fn merge_pipeline_states(
        &self,
        states: &std::collections::BTreeMap<String, PipelineSessionState>,
    ) -> Result<(), CheckpointError> {
        self.mutate(|cp| {
            for (session, state) in states {
                cp.pipeline_states.insert(session.clone(), state.clone());
            }
        })
        .map(|_| ())
    }

    /// Patch one session's manager-owned pipeline state.
    pub fn merge_pipeline_states_owned(
        &self,
        session_id: &str,
        f: impl FnOnce(&mut PipelineSessionState),
    ) -> Result<(), CheckpointError> {
        self.mutate(|cp| {
            let state = cp
                .pipeline_states
                .entry(session_id.to_string())
                .or_default();
            f(state);
        })
        .map(|_| ())
    }

    /// Build the pure trigger input for
    /// [`crate::core::persona::evaluate_persona_trigger`] from the checkpoint
    /// plus live-store facts — the MEM-16 orchestration half of MEM-15's debt.
    pub fn persona_trigger_input(
        &self,
        checkpoint: &Checkpoint,
        has_scene_blocks: bool,
        has_persona_body: bool,
    ) -> crate::core::persona::PersonaTriggerInput {
        crate::core::persona::PersonaTriggerInput {
            request_persona_update: checkpoint.request_persona_update,
            request_reason: if checkpoint.persona_update_reason.is_empty() {
                None
            } else {
                Some(checkpoint.persona_update_reason.clone())
            },
            scenes_processed: checkpoint.scenes_processed as usize,
            memories_since_last_persona: checkpoint.memories_since_last_persona as usize,
            has_scene_blocks,
            previously_generated: checkpoint.last_persona_at > 0,
            has_persona_body,
        }
    }
}
