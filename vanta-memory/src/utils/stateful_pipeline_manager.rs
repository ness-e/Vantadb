//! `StatefulPipelineManager` (port of TDAM
//! `MC/utils/stateful-pipeline-manager.ts` — MEM-16).
//!
//! Same capture → L1 trigger contract as [`super::pipeline_manager::
//! MemoryPipelineManager`], but every mutation flows through the state
//! backend's atomic operations and is mirrored into the persistent
//! [`Checkpoint`](crate::utils::checkpoint::Checkpoint) via an optional
//! persister callback — the shape TDAM uses so a remote backend can replace
//! the local one without touching callers.

use crate::core::state::{TaskKind, TaskPayload};
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::Clock;
use crate::utils::pipeline_manager::{l1_idle_member, PipelineConfig};

/// Backend-backed pipeline manager with checkpoint persistence.
pub struct StatefulPipelineManager<'a, C: Clock> {
    backend: &'a LocalStateBackend<C>,
    config: PipelineConfig,
}

impl<'a, C: Clock> StatefulPipelineManager<'a, C> {
    pub fn new(backend: &'a LocalStateBackend<C>, config: PipelineConfig) -> Self {
        Self { backend, config }
    }

    /// One conversation round through the backend's atomic capture. Returns
    /// whether the threshold fired. New sessions start on the warm-up
    /// schedule at 1.
    pub fn notify_conversation(
        &self,
        session_id: &str,
        message_json: Option<&str>,
        rounds: u64,
    ) -> bool {
        if self.backend.get_session_state(session_id).is_none() {
            self.backend.update_session_state(
                session_id,
                crate::utils::local_backend::PipelineSessionStatePatch {
                    warmup_threshold: Some(1),
                    ..Default::default()
                },
            );
        }
        let now = self.backend.now_ms();
        let threshold = self.effective_threshold(session_id);
        let params = crate::core::state::CaptureAtomicParams {
            session_id: session_id.to_string(),
            message_json: message_json.map(str::to_string),
            threshold,
            fire_at_ms: now.saturating_add(self.config.l1_idle_timeout_ms),
            timer_member: l1_idle_member(session_id),
            task: TaskPayload {
                id: String::new(),
                kind: TaskKind::L1,
                session_id: session_id.to_string(),
                priority: 1,
                created_at_ms: now,
                attempts: 0,
            },
            now_ms: now,
            rounds,
        };
        self.backend.capture_atomic(params).triggered
    }

    /// Force-drain a session regardless of threshold/timer.
    pub fn flush_session(&self, session_id: &str) {
        let now = self.backend.now_ms();
        self.backend.remove_timer(&l1_idle_member(session_id));
        self.backend.enqueue_task(TaskPayload {
            id: String::new(),
            kind: TaskKind::L1,
            session_id: session_id.to_string(),
            priority: 0, // explicit flushes jump the queue
            created_at_ms: now,
            attempts: 0,
        });
    }

    /// Post-L1 bookkeeping: warm-up advance + extraction timestamps, then
    /// hand the full state map to `persister`.
    pub fn mark_l1_complete(
        &self,
        session_id: &str,
        rfc3339_now: &str,
        mut persister: impl FnMut(
            &std::collections::BTreeMap<String, crate::core::state::PipelineSessionState>,
        ),
    ) {
        let current = self
            .backend
            .get_session_state(session_id)
            .unwrap_or_default();
        let next_warmup = if current.warmup_threshold == 0 {
            0
        } else {
            (current.warmup_threshold * 2).min(self.config.every_n_conversations)
        };
        self.backend.update_session_state(
            session_id,
            crate::utils::local_backend::PipelineSessionStatePatch {
                last_extraction_time: Some(rfc3339_now),
                last_extraction_updated_time: Some(rfc3339_now),
                warmup_threshold: Some(next_warmup),
                ..Default::default()
            },
        );
        self.backend.remove_timer(&l1_idle_member(session_id));

        let states: std::collections::BTreeMap<_, _> = self
            .backend
            .list_active_sessions()
            .into_iter()
            .filter_map(|s| self.backend.get_session_state(&s).map(|state| (s, state)))
            .collect();
        persister(&states);
    }

    /// Effective threshold for a session (warm-up while active, else interval).
    pub fn effective_threshold(&self, session_id: &str) -> u64 {
        let warmup = self
            .backend
            .get_session_state(session_id)
            .map(|s| s.warmup_threshold)
            .unwrap_or(0);
        if warmup == 0 {
            self.config.every_n_conversations
        } else {
            warmup.min(self.config.every_n_conversations)
        }
    }

    /// Live state of a session, if any.
    pub fn session_state(
        &self,
        session_id: &str,
    ) -> Option<crate::core::state::PipelineSessionState> {
        self.backend.get_session_state(session_id)
    }
}
