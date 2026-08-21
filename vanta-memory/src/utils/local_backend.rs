//! `LocalStateBackend` — in-process pipeline state backend (port of TDAM
//! `MC/core/state/local-backend.ts`, reimplemented in Rust — MEM-16).
//!
//! Replaces the TDAM Redis backend with plain process-local maps (Principio 7:
//! no Redis). Buffers, session states, timers, the prioritized task queue and
//! TTL locks all live behind one mutex; every deadline comparison goes through
//! an injected [`Clock`] so tests are deterministic.
//!
//! Key sanitization does not apply here: these keys never reach the VantaDB
//! store (that boundary is enforced where records are persisted).

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::state::{
    CaptureAtomicParams, CaptureAtomicResult, PipelineSessionState, TaskPayload, TimerEntry,
};
use crate::utils::managed_timer::Clock;

#[derive(Debug, Default)]
struct Inner {
    buffers: HashMap<String, Vec<String>>,
    states: HashMap<String, PipelineSessionState>,
    /// member → absolute fire-at ms.
    timers: HashMap<String, u64>,
    /// Kept sorted: priority asc, then created_at asc.
    queue: Vec<TaskPayload>,
    /// lock key → (owner, expire_at ms).
    locks: HashMap<String, (String, u64)>,
    next_task_seq: u64,
}

/// In-process state backend. Cheap to clone-free share: wrap in `Arc` if
/// several owners need it.
#[derive(Debug)]
pub struct LocalStateBackend<C: Clock> {
    clock: C,
    inner: Mutex<Inner>,
}

impl<C: Clock> LocalStateBackend<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            inner: Mutex::new(Inner::default()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // A poisoned mutex means a panic while holding it; the state is still
        // structurally valid (plain maps), so recover instead of spreading the
        // poison to callers that cannot do anything with it.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Current epoch ms according to the injected clock.
    pub fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// Access the injected clock (tests drive the fake through this).
    pub fn clock(&self) -> &C {
        &self.clock
    }

    // ═══ Buffer ═══

    /// Append a serialized message to a session buffer.
    pub fn append_buffer(&self, session_id: &str, message_json: &str) {
        self.lock()
            .buffers
            .entry(session_id.to_string())
            .or_default()
            .push(message_json.to_string());
    }

    /// Drain (and clear) a session buffer.
    pub fn drain_buffer(&self, session_id: &str) -> Vec<String> {
        self.lock().buffers.remove(session_id).unwrap_or_default()
    }

    /// Current buffered message count for a session.
    pub fn buffer_len(&self, session_id: &str) -> usize {
        self.lock()
            .buffers
            .get(session_id)
            .map(Vec::len)
            .unwrap_or(0)
    }

    // ═══ Session state ═══

    pub fn get_session_state(&self, session_id: &str) -> Option<PipelineSessionState> {
        self.lock().states.get(session_id).cloned()
    }

    /// Patch-merge into the session state (creating it when missing).
    pub fn update_session_state(&self, session_id: &str, patch: PipelineSessionStatePatch<'_>) {
        let mut inner = self.lock();
        let state = inner
            .states
            .entry(session_id.to_string())
            .or_insert_with(|| PipelineSessionState {
                last_active_time_ms: self.clock.now_ms(),
                ..PipelineSessionState::default()
            });
        if let Some(v) = patch.conversation_count {
            state.conversation_count = v;
        }
        if let Some(v) = patch.last_extraction_time {
            state.last_extraction_time = v.to_string();
        }
        if let Some(v) = patch.last_extraction_updated_time {
            state.last_extraction_updated_time = v.to_string();
        }
        if let Some(v) = patch.last_active_time_ms {
            state.last_active_time_ms = v;
        }
        if let Some(v) = patch.l2_pending_l1_count {
            state.l2_pending_l1_count = v;
        }
        if let Some(v) = patch.warmup_threshold {
            state.warmup_threshold = v;
        }
        if let Some(v) = patch.l2_last_extraction_time {
            state.l2_last_extraction_time = v.to_string();
        }
    }

    pub fn delete_session_state(&self, session_id: &str) {
        let mut inner = self.lock();
        inner.states.remove(session_id);
        inner.buffers.remove(session_id);
    }

    /// All sessions with live state.
    pub fn list_active_sessions(&self) -> Vec<String> {
        self.lock().states.keys().cloned().collect()
    }

    // ═══ Timers ═══

    /// Set (or replace) a timer for `member`.
    pub fn set_timer(&self, member: &str, fire_at_ms: u64) {
        self.lock().timers.insert(member.to_string(), fire_at_ms);
    }

    /// Set a timer only when `fire_at_ms` is earlier than the current one
    /// (downward-only pattern). Returns whether the timer was set.
    pub fn set_timer_if_earlier(&self, member: &str, fire_at_ms: u64) -> bool {
        let mut inner = self.lock();
        match inner.timers.get(member) {
            Some(current) if fire_at_ms >= *current => false,
            _ => {
                inner.timers.insert(member.to_string(), fire_at_ms);
                true
            }
        }
    }

    pub fn remove_timer(&self, member: &str) {
        self.lock().timers.remove(member);
    }

    /// Remove and return every timer whose deadline passed at `now_ms`.
    pub fn take_expired_timers(&self, now_ms: u64) -> Vec<TimerEntry> {
        let mut inner = self.lock();
        let expired: Vec<(String, u64)> = inner
            .timers
            .iter()
            .filter(|(_, at)| **at <= now_ms)
            .map(|(member, at)| (member.clone(), *at))
            .collect();
        for (member, _) in &expired {
            inner.timers.remove(member);
        }
        expired
            .into_iter()
            .map(|(member, fire_at_ms)| TimerEntry { member, fire_at_ms })
            .collect()
    }

    // ═══ Task queue ═══

    /// Enqueue a task keeping the queue ordered by (priority, created_at).
    pub fn enqueue_task(&self, mut task: TaskPayload) {
        let mut inner = self.lock();
        task.id = format!("t_{}_{}", task.created_at_ms, inner.next_task_seq);
        inner.next_task_seq += 1;
        let pos = inner
            .queue
            .iter()
            .position(|t| {
                t.priority > task.priority
                    || (t.priority == task.priority && t.created_at_ms > task.created_at_ms)
            })
            .unwrap_or(inner.queue.len());
        inner.queue.insert(pos, task);
    }

    /// Pop the highest-priority oldest task, if any.
    pub fn consume_task(&self) -> Option<TaskPayload> {
        let mut inner = self.lock();
        if inner.queue.is_empty() {
            None
        } else {
            Some(inner.queue.remove(0))
        }
    }

    /// Queue depth split by priority class (0 = high).
    pub fn queue_depth(&self) -> (usize, usize) {
        let inner = self.lock();
        let high = inner.queue.iter().filter(|t| t.priority == 0).count();
        (high, inner.queue.len() - high)
    }

    /// Snapshot of queued tasks (priority order).
    pub fn list_queued_tasks(&self) -> Vec<TaskPayload> {
        self.lock().queue.clone()
    }

    // ═══ Locks ═══

    fn clean_expired_locks(inner: &mut Inner, now_ms: u64) {
        inner.locks.retain(|_, (_, expire_at)| *expire_at > now_ms);
    }

    /// Acquire a TTL lock. Fails when held by anyone (expired locks are
    /// collected first).
    pub fn acquire_lock(&self, key: &str, owner: &str, ttl_ms: u64) -> bool {
        let now = self.clock.now_ms();
        let mut inner = self.lock();
        Self::clean_expired_locks(&mut inner, now);
        if inner.locks.contains_key(key) {
            return false;
        }
        inner.locks.insert(
            key.to_string(),
            (owner.to_string(), now.saturating_add(ttl_ms)),
        );
        true
    }

    /// Extend the TTL of a lock only for its owner.
    pub fn renew_lock(&self, key: &str, owner: &str, ttl_ms: u64) -> bool {
        let now = self.clock.now_ms();
        let mut inner = self.lock();
        match inner.locks.get_mut(key) {
            Some((lock_owner, expire_at)) if lock_owner == owner => {
                *expire_at = now.saturating_add(ttl_ms);
                true
            }
            _ => false,
        }
    }

    /// Release a lock only for its owner.
    pub fn release_lock(&self, key: &str, owner: &str) {
        let mut inner = self.lock();
        if inner.locks.get(key).map(|(o, _)| o.as_str()) == Some(owner) {
            inner.locks.remove(key);
        }
    }

    // ═══ Atomic capture ═══

    /// One critical section: append to the buffer, bump the conversation
    /// counter, then either enqueue the L1 task (threshold reached — counter
    /// resets, idle timer removed) or arm the idle timer.
    pub fn capture_atomic(&self, params: CaptureAtomicParams) -> CaptureAtomicResult {
        if let Some(message) = params.message_json.as_deref() {
            self.append_buffer(&params.session_id, message);
        }

        let (triggered, count) = {
            let mut inner = self.lock();
            let mut fired = false;
            let count = {
                let state = inner
                    .states
                    .entry(params.session_id.clone())
                    .or_insert_with(|| PipelineSessionState {
                        last_active_time_ms: params.now_ms,
                        ..PipelineSessionState::default()
                    });
                state.conversation_count += params.rounds.max(1);
                state.last_active_time_ms = params.now_ms;

                if state.conversation_count >= params.threshold {
                    state.conversation_count = 0;
                    fired = true;
                    0
                } else {
                    state.conversation_count
                }
            };
            if fired {
                inner.timers.remove(&params.timer_member);
            } else {
                inner
                    .timers
                    .insert(params.timer_member.clone(), params.fire_at_ms);
            }
            (fired, count)
        };

        if triggered {
            self.enqueue_task(params.task);
        }
        CaptureAtomicResult {
            triggered,
            conversation_count: count,
        }
    }

    // ═══ Lifecycle / snapshot ═══

    /// Drop all state (shutdown).
    pub fn destroy(&self) {
        let mut inner = self.lock();
        inner.buffers.clear();
        inner.states.clear();
        inner.timers.clear();
        inner.queue.clear();
        inner.locks.clear();
    }

    /// Diagnostic counts.
    pub fn snapshot(&self) -> BackendSnapshot {
        let inner = self.lock();
        BackendSnapshot {
            sessions: inner.states.len(),
            buffers: inner.buffers.len(),
            timers: inner.timers.len(),
            queue: inner.queue.len(),
            locks: inner.locks.len(),
        }
    }
}

/// Patch for [`LocalStateBackend::update_session_state`] (`None` = keep).
#[derive(Debug, Clone, Default)]
pub struct PipelineSessionStatePatch<'a> {
    pub conversation_count: Option<u64>,
    pub last_extraction_time: Option<&'a str>,
    pub last_extraction_updated_time: Option<&'a str>,
    pub last_active_time_ms: Option<u64>,
    pub l2_pending_l1_count: Option<u64>,
    pub warmup_threshold: Option<u64>,
    pub l2_last_extraction_time: Option<&'a str>,
}

/// Diagnostic snapshot of backend contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BackendSnapshot {
    pub sessions: usize,
    pub buffers: usize,
    pub timers: usize,
    pub queue: usize,
    pub locks: usize,
}
