//! Pipeline worker (port of TDAM `MC/services/pipeline-worker.ts`, single
//! process — MEM-16).
//!
//! Consumes prioritized tasks from the state backend, serializes per-session
//! work through TTL locks, retries failures and dead-letters exhausted tasks.
//! [`MemoryTaskHandler`] wires the task kinds to the existing pipeline
//! modules: L0 capture (`l0_recorder`) → L1 extraction + dedup
//! (`l1_extractor`/`l1_dedup`) → L2 scenes (`scene_extractor`) → L3 persona
//! (`persona_trigger`/`persona_generator`), with counters tracked in the
//! [`Checkpoint`](crate::utils::checkpoint::Checkpoint).
//!
//! Not ported from TDAM: multi-worker pending recovery, `claimStaleTasks`,
//! Prometheus metrics (single-process scope).

use crate::context_engine::{
    assemble_with_recall, load_active, AssembleConfig, ChatMessage, ChatRole, TokenEstimator,
};
use crate::core::abstractions::LlmRunner;
use crate::core::conversation::{L0Recorder, L0Role};
use crate::core::hooks::{perform_auto_recall, AutoRecallParams, RecallConfig};
use crate::core::persona::{
    evaluate_persona_trigger, generate_persona, get_persona, has_persona_body,
    PersonaGenerateParams,
};
use crate::core::prompts::l1_extraction::{epoch_ms_to_rfc3339, PromptMode};
use crate::core::record::{
    extract_l1_segments, read_session_records, run_l1_dedup, L1DedupConfig, L1ExtractorConfig,
};
use crate::core::scene::{extract_scenes_with_llm, list_scenes, SceneMemoryInput};
use crate::core::state::{TaskKind, TaskPayload};
use crate::offload::state_manager::OffloadStateManager;
use crate::utils::checkpoint::CheckpointManager;
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::Clock;
use crate::utils::sanitize::sanitize_key;

/// Handles one task. Errors are retryable; after `max_retries` the task is
/// dead-lettered.
pub trait TaskHandler {
    /// Process one task. `Err` schedules a retry.
    fn handle(&mut self, task: &TaskPayload) -> Result<(), String>;
}

/// Worker configuration.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Attempts per task before dead-lettering (total runs = max_retries).
    pub max_retries: u32,
    /// Per-session lock TTL in ms.
    pub lock_ttl_ms: u64,
    /// Max tasks consumed per [`PipelineWorker::run_once`].
    pub batch_size: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            lock_ttl_ms: 60_000,
            batch_size: 8,
        }
    }
}

/// One dead-lettered task with its last error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterEntry {
    pub task: TaskPayload,
    pub error: String,
}

/// Run statistics of one [`PipelineWorker::run_once`] pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunStats {
    pub processed: usize,
    pub failed: usize,
    pub skipped_locked: usize,
}

/// Configuration of the post-L3 context-assembly phase (MEM-43).
///
/// The worker is ONE caller of [`assemble_with_recall`]; external hosts keep
/// calling it directly — this phase makes the engine productive inside the
/// L0→L3 cycle without changing its API.
#[derive(Debug, Clone)]
pub struct ContextAssemblyConfig {
    /// Master switch (`context_compression_enabled`): when `false` the
    /// handler skips assembly entirely (no silent-compression surprise).
    pub enabled: bool,
    /// Shared token budget for compress → MMD → recall (MEM-37 contract:
    /// the union is guaranteed ≤ budget).
    pub budget_tokens: u64,
}

impl Default for ContextAssemblyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            budget_tokens: 8192,
        }
    }
}

/// Task consumer over a [`LocalStateBackend`].
pub struct PipelineWorker<'a, C: Clock> {
    backend: &'a LocalStateBackend<C>,
    config: WorkerConfig,
    owner: String,
    dead_letters: Vec<DeadLetterEntry>,
}

impl<'a, C: Clock> PipelineWorker<'a, C> {
    pub fn new(backend: &'a LocalStateBackend<C>, config: WorkerConfig) -> Self {
        Self {
            backend,
            config,
            owner: format!("worker-{}", std::process::id()),
            dead_letters: Vec::new(),
        }
    }

    /// Consume up to `batch_size` tasks and run them through `handler`.
    ///
    /// A task whose session is locked is left for the next pass (it stays at
    /// the queue head — consume order makes this safe without requeueing).
    pub fn run_once(&mut self, handler: &mut dyn TaskHandler) -> RunStats {
        let mut stats = RunStats::default();
        for _ in 0..self.config.batch_size {
            let Some(task) = self.backend.consume_task() else {
                break;
            };
            let lock_key = session_lock_key(&task.session_id);
            if !self
                .backend
                .acquire_lock(&lock_key, &self.owner, self.config.lock_ttl_ms)
            {
                stats.skipped_locked += 1;
                // Requeue at the back so one busy session cannot starve others,
                // and stop this pass — the copy must not be re-consumed here.
                let mut deferred = task.clone();
                deferred.created_at_ms = self.backend.now_ms();
                self.backend.enqueue_task(deferred);
                break;
            }

            // Release BEFORE acting on the outcome: every branch below ends
            // this pass, and a held lock would deadlock the retried task.
            let outcome = handler.handle(&task);
            self.backend.release_lock(&lock_key, &self.owner);

            match outcome {
                Ok(()) => stats.processed += 1,
                Err(error) => {
                    let mut retry = task.clone();
                    retry.attempts += 1;
                    if retry.attempts >= self.config.max_retries {
                        tracing::warn!(task_id = %task.id, %error, "task dead-lettered");
                        stats.failed += 1;
                        self.dead_letters.push(DeadLetterEntry { task, error });
                    } else {
                        // Requeue for another attempt (back of its priority
                        // class) and stop the pass — never re-consume it here.
                        tracing::warn!(task_id = %task.id, attempt = retry.attempts, %error, "task retry");
                        retry.created_at_ms = self.backend.now_ms();
                        self.backend.enqueue_task(retry);
                        break;
                    }
                }
            }
        }
        stats
    }

    /// Tasks that exhausted their attempts (oldest first).
    pub fn dead_letters(&self) -> &[DeadLetterEntry] {
        &self.dead_letters
    }

    /// Clear the dead-letter log (after inspection/replay).
    pub fn clear_dead_letters(&mut self) {
        self.dead_letters.clear();
    }
}

/// Lock key serializing all pipeline phases of one session.
fn session_lock_key(session_id: &str) -> String {
    format!("pipeline_lock:{session_id}")
}

/// Concrete handler wiring task kinds to the existing L0→L3 modules.
///
/// Generic over `R: LlmRunner` (the trait is not dyn-compatible). Every LLM
/// failure degrades per Principio 4: the task fails into the worker's
/// retry/dead-letter path and stored data is never lost or overwritten.
pub struct MemoryTaskHandler<'a, R: LlmRunner> {
    db: vantadb::sdk::VantaEmbedded,
    runner: &'a R,
    extractor_config: L1ExtractorConfig,
    dedup_config: L1DedupConfig,
    trigger_every_n: usize,
    context_config: ContextAssemblyConfig,
}

impl<'a, R: LlmRunner> MemoryTaskHandler<'a, R> {
    pub fn new(
        db: vantadb::sdk::VantaEmbedded,
        runner: &'a R,
        extractor_config: L1ExtractorConfig,
        dedup_config: L1DedupConfig,
        trigger_every_n: usize,
    ) -> Self {
        Self {
            db,
            runner,
            extractor_config,
            dedup_config,
            trigger_every_n,
            context_config: ContextAssemblyConfig::default(),
        }
    }

    /// Override the post-L3 context-assembly configuration (builder-style).
    /// `new()` keeps its signature so existing callers are untouched.
    pub fn with_context_config(mut self, config: ContextAssemblyConfig) -> Self {
        self.context_config = config;
        self
    }

    fn run_l1(&mut self, session_id: &str) -> Result<(), String> {
        match self.run_l1_inner(session_id) {
            Ok(()) => Ok(()),
            Err(err) => {
                // MEM-41 provenance: the L1 writer never sees extraction/LLM
                // failures, so the failed generation is logged here.
                // Best-effort (P4).
                crate::core::memory_generation_log::record_best_effort(
                    &self.db,
                    &crate::core::memory_generation_log::GenerationLogEntry::new(
                        crate::core::memory_generation_log::GenerationLayer::L1,
                        crate::core::memory_generation_log::GenerationStatus::Failed,
                        session_id,
                        None,
                        Some(err.clone()),
                    ),
                );
                Err(err)
            }
        }
    }

    fn run_l1_inner(&mut self, session_id: &str) -> Result<(), String> {
        let recorder = L0Recorder::new(self.db.clone());
        let messages = recorder
            .read_messages(session_id)
            .map_err(|e| format!("L0 read failed: {e}"))?;

        let checkpoints = CheckpointManager::new(&self.db);
        let checkpoint = checkpoints.read().map_err(|e| e.to_string())?;
        let runner_state = checkpoints.get_runner_state(&checkpoint, session_id);
        let previous_scene = if runner_state.last_scene_name.is_empty() {
            None
        } else {
            Some(runner_state.last_scene_name)
        };

        let (result, memories) = extract_l1_segments(
            self.runner,
            &messages,
            previous_scene.as_deref(),
            &self.extractor_config,
        );
        if !result.success {
            return Err("L1 extraction failed".to_string());
        }
        if memories.is_empty() {
            return Ok(()); // nothing passed the quality gate — no-op
        }

        let records = run_l1_dedup(
            &self.db,
            self.runner,
            session_id,
            session_id,
            &memories,
            &self.dedup_config,
        )
        .map_err(|e| format!("L1 dedup failed: {e}"))?;

        checkpoints
            .add_memories_extracted(records.len() as u64)
            .map_err(|e| e.to_string())?;
        if let Some(scene) = result.last_scene_name.as_deref() {
            update_runner_state(&checkpoints, session_id, |state| {
                state.last_scene_name = scene.to_string();
                state.last_l1_cursor = messages.last().map(|m| m.timestamp_ms).unwrap_or(0);
            })
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn run_l2(&mut self, session_id: &str) -> Result<(), String> {
        let records = read_session_records(&self.db, session_id)
            .map_err(|e| format!("L1 record read failed: {e}"))?;
        if records.is_empty() {
            return Ok(());
        }
        let inputs: Vec<SceneMemoryInput> = records
            .iter()
            .map(|r| SceneMemoryInput {
                id: r.id.clone(),
                content: r.content.clone(),
                created_at: r.created_at.clone(),
            })
            .collect();

        let checkpoints = CheckpointManager::new(&self.db);
        let checkpoint = checkpoints.read().map_err(|e| e.to_string())?;
        let previous_scene = checkpoints
            .get_runner_state(&checkpoint, session_id)
            .last_scene_name;
        let result = extract_scenes_with_llm(
            &self.db,
            session_id,
            self.runner,
            &inputs,
            if previous_scene.is_empty() {
                None
            } else {
                Some(&previous_scene)
            },
        );
        if !result.success {
            return Err(format!("L2 scene extraction failed: {:?}", result.error));
        }

        checkpoints
            .increment_scenes_processed()
            .map_err(|e| e.to_string())?;
        checkpoints
            .merge_pipeline_states_owned(session_id, |state| {
                state.l2_last_extraction_time =
                    epoch_ms_to_rfc3339(crate::core::conversation::now_ms());
            })
            .map_err(|e| e.to_string())
    }

    fn run_l3(&mut self, session_id: &str) -> Result<(), String> {
        let checkpoints = CheckpointManager::new(&self.db);
        let checkpoint = checkpoints.read().map_err(|e| e.to_string())?;

        let has_scene_blocks = !list_scenes(&self.db, session_id)
            .map_err(|e| format!("scene index read failed: {e}"))?
            .is_empty();
        let has_body = get_persona(&self.db, session_id)
            .map_err(|e| format!("persona read failed: {e}"))?
            .map(|p| has_persona_body(&p.content))
            .unwrap_or(false);

        let input = checkpoints.persona_trigger_input(&checkpoint, has_scene_blocks, has_body);
        let trigger = evaluate_persona_trigger(&input, self.trigger_every_n);
        if !trigger.should {
            return Ok(()); // quiet session — nothing to do
        }

        let total_processed = checkpoint.total_processed;
        let result = generate_persona(
            &self.db,
            self.runner,
            &PersonaGenerateParams {
                session_key: session_id,
                total_processed: total_processed as usize,
                prompt_mode: PromptMode::Chat,
                trigger_info: Some(trigger.reason.clone()),
            },
        );
        if !result.success {
            return Err(format!("L3 persona generation failed: {:?}", result.error));
        }

        let now = crate::core::conversation::now_ms();
        checkpoints
            .mark_persona_generated(total_processed, now, &epoch_ms_to_rfc3339(now))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Post-L3 context assembly (MEM-43): runs [`assemble_with_recall`] over
    /// the session history — compress → inject active MMD → inject recall
    /// blocks — against ONE shared token budget, and persists the assembled
    /// context under `context/<session>` key `__assembled` (read it back with
    /// [`load_assembled_context`]).
    ///
    /// LLM-free by construction (the engine never calls the runner), so the
    /// only failures are store-level and propagate into the worker's
    /// retry/dead-letter path (Principio 4 — nothing partial is written: the
    /// record is a single upsert).
    fn run_context_assembly(&self, session_id: &str) -> Result<(), String> {
        let l0 = L0Recorder::new(self.db.clone())
            .read_messages(session_id)
            .map_err(|e| format!("L0 read failed: {e}"))?;
        let msgs: Vec<ChatMessage> = l0
            .iter()
            .map(|m| ChatMessage {
                role: match m.role {
                    L0Role::User => ChatRole::User,
                    L0Role::Assistant => ChatRole::Assistant,
                },
                content: m.content.clone(),
                id: m.id.clone().map(|s| sanitize_key(&s)),
            })
            .collect();

        let active_mmd = load_active(&self.db, session_id)
            .map_err(|e| format!("active MMD load failed: {e}"))?;

        // Recall query: what the user last asked — the natural "what would
        // the host need injected next" signal. Empty session → empty query,
        // which `perform_auto_recall` treats as skip-L1-search.
        let query = l0
            .iter()
            .rev()
            .find(|m| m.role == L0Role::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let recalled = perform_auto_recall(
            &self.db,
            AutoRecallParams {
                user_text: query,
                session_key: session_id,
                isolation: None,
                config: RecallConfig::default(),
            },
        )
        .map_err(|e| format!("auto-recall failed: {e}"))?;
        let (prepend, append) = match recalled {
            Some(r) => (r.prepend_context, r.append_system_context),
            None => (None, None),
        };

        let cursor = OffloadStateManager::new(self.db.clone())
            .last_offloaded_tool_call_id(session_id)
            .map_err(|e| format!("offload cursor read failed: {e}"))?;

        let out = assemble_with_recall(
            msgs,
            self.context_config.budget_tokens,
            &TokenEstimator::default(),
            0,
            &AssembleConfig::default(),
            active_mmd,
            prepend.as_deref(),
            append.as_deref(),
            cursor.as_deref(),
        )
        .map_err(|e| format!("context assembly failed: {e}"))?;
        tracing::debug!(
            session = %session_id,
            mode = ?out.report.mode,
            msgs_before = out.report.msgs_before,
            msgs_conserved = out.report.msgs_conserved,
            mmd_injected = out.mmd_injected,
            recall_injected = out.recall_injected,
            "context assembled post-L3"
        );

        let payload = serde_json::to_string(&out).map_err(|e| e.to_string())?;
        self.db
            .put(vantadb::sdk::VantaMemoryInput {
                namespace: assembled_namespace(session_id),
                key: sanitize_key(ASSEMBLED_KEY),
                payload,
                metadata: vantadb::sdk::VantaMemoryMetadata::new(),
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
            })
            .map(|_| ())
            .map_err(|e| format!("assembled context write failed: {e}"))
    }
}

impl<'a, R: LlmRunner> TaskHandler for MemoryTaskHandler<'a, R> {
    fn handle(&mut self, task: &TaskPayload) -> Result<(), String> {
        match task.kind {
            TaskKind::L1 | TaskKind::Flush => self.run_l1(&task.session_id),
            TaskKind::L2 => self.run_l2(&task.session_id),
            // L3 runs first; the context-assembly phase is strictly post-L3
            // (order asserted by the D19 e2e test).
            TaskKind::L3 => {
                self.run_l3(&task.session_id)?;
                if self.context_config.enabled {
                    self.run_context_assembly(&task.session_id)?;
                }
                Ok(())
            }
        }
    }
}

/// Assembled-context record key inside `context/<session>`.
const ASSEMBLED_KEY: &str = "__assembled";

/// `context/<sanitized-session>` — assembled-context records namespace.
fn assembled_namespace(session_id: &str) -> String {
    format!(
        "context/{}",
        crate::utils::sanitize::sanitize_component(session_id, 128, false)
    )
}

/// Read back the last persisted post-L3 assembly for a session. Missing or
/// corrupt record → `None` (never fatal, mirrors `load_active`).
pub fn load_assembled_context(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
) -> Result<Option<crate::context_engine::IntegratedContext>, String> {
    match db
        .get(
            &assembled_namespace(session_id),
            &sanitize_key(ASSEMBLED_KEY),
        )
        .map_err(|e| format!("assembled context read failed: {e}"))?
    {
        None => Ok(None),
        Some(record) => match serde_json::from_str(&record.payload) {
            Ok(ctx) => Ok(Some(ctx)),
            Err(err) => {
                tracing::warn!(session = %session_id, "corrupt assembled context, ignoring: {err}");
                Ok(None)
            }
        },
    }
}

/// Patch one session's runner state inside the checkpoint.
fn update_runner_state(
    checkpoints: &CheckpointManager<'_>,
    session_id: &str,
    f: impl FnOnce(&mut crate::utils::checkpoint::RunnerSessionState),
) -> Result<(), crate::utils::checkpoint::CheckpointError> {
    let checkpoint = checkpoints.read()?;
    let mut state = checkpoints.get_runner_state(&checkpoint, session_id);
    f(&mut state);
    let mut checkpoint = checkpoint;
    checkpoint
        .runner_states
        .insert(session_id.to_string(), state);
    checkpoints.write(&checkpoint)
}
