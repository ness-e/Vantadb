//! `MemoryPipelineManager` (port of TDAM `MC/utils/pipeline-manager.ts`,
//! in-process variant — MEM-16).
//!
//! Owns the capture → L1 trigger flow: buffers messages, applies the warm-up
//! threshold schedule (1→2→4→… capped at the configured interval), and either
//! enqueues an L1 task when the threshold fires or arms an idle timer so quiet
//! sessions still flush. All deadlines go through the injected [`Clock`].
//!
//! Not ported from TDAM: stale-session GC and pending-session recovery
//! (single-process scope; revisit with a second backend).

use crate::core::state::{TaskKind, TaskPayload};
use crate::utils::local_backend::{LocalStateBackend, PipelineSessionStatePatch};
use crate::utils::managed_timer::Clock;

/// Idle-timer member for a session.
pub fn l1_idle_member(session_id: &str) -> String {
    format!("l1_idle:{session_id}")
}

/// Configuration for the manager.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Conversation rounds that trigger L1 once warm-up has graduated.
    pub every_n_conversations: u64,
    /// Idle timeout: a session silent this long flushes its buffer.
    pub l1_idle_timeout_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            every_n_conversations: 10,
            l1_idle_timeout_ms: 30_000,
        }
    }
}

/// In-process pipeline manager over a [`LocalStateBackend`].
pub struct MemoryPipelineManager<'a, C: Clock> {
    backend: &'a LocalStateBackend<C>,
    config: PipelineConfig,
}

impl<'a, C: Clock> MemoryPipelineManager<'a, C> {
    pub fn new(backend: &'a LocalStateBackend<C>, config: PipelineConfig) -> Self {
        Self { backend, config }
    }

    /// Effective threshold for a session: warm-up value while it is >0,
    /// otherwise the configured interval.
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

    /// Record one conversation round for a session. Buffered message is
    /// optional. Returns whether the threshold fired.
    pub fn notify_conversation(
        &self,
        session_id: &str,
        message_json: Option<&str>,
        rounds: u64,
    ) -> bool {
        let now = self.backend.now_ms();
        let params = crate::core::state::CaptureAtomicParams {
            session_id: session_id.to_string(),
            message_json: message_json.map(str::to_string),
            threshold: self.effective_threshold(session_id),
            fire_at_ms: now.saturating_add(self.config.l1_idle_timeout_ms),
            timer_member: l1_idle_member(session_id),
            task: self.l1_task(session_id, now),
            now_ms: now,
            rounds,
        };
        self.backend.capture_atomic(params).triggered
    }

    /// Force-drain a session regardless of threshold/timer.
    pub fn flush_session(&self, session_id: &str) {
        let now = self.backend.now_ms();
        self.backend.remove_timer(&l1_idle_member(session_id));
        self.backend.enqueue_task(self.l1_task(session_id, now));
    }

    /// Called after a successful L1 run: advance warm-up (1→2→4→… capped),
    /// record extraction timestamps and clear the idle timer.
    pub fn mark_l1_complete(&self, session_id: &str, rfc3339_now: &str) {
        let current = self
            .backend
            .get_session_state(session_id)
            .unwrap_or_default();
        let next_warmup = if current.warmup_threshold == 0 {
            // Already graduated.
            0
        } else {
            (current.warmup_threshold * 2).min(self.config.every_n_conversations)
        };
        self.backend.update_session_state(
            session_id,
            PipelineSessionStatePatch {
                last_extraction_time: Some(rfc3339_now),
                warmup_threshold: Some(next_warmup),
                ..Default::default()
            },
        );
        self.backend.remove_timer(&l1_idle_member(session_id));
    }

    /// Seed the warm-up schedule for a brand-new session (starts at 1).
    pub fn start_session(&self, session_id: &str) {
        self.backend.update_session_state(
            session_id,
            PipelineSessionStatePatch {
                warmup_threshold: Some(1),
                ..Default::default()
            },
        );
    }

    /// Live state of a session, if any.
    pub fn session_state(
        &self,
        session_id: &str,
    ) -> Option<crate::core::state::PipelineSessionState> {
        self.backend.get_session_state(session_id)
    }

    fn l1_task(&self, session_id: &str, now_ms: u64) -> TaskPayload {
        TaskPayload {
            id: String::new(), // assigned by the queue on enqueue
            kind: TaskKind::L1,
            session_id: session_id.to_string(),
            priority: 1,
            created_at_ms: now_ms,
            attempts: 0,
        }
    }
}
