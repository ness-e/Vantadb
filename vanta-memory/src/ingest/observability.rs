//! MEM-72 — a degraded ingest runner is visible instead of silent.
//!
//! [`IngestRunnerProvider::Local`](super::runner_config::IngestRunnerProvider)
//! (the default) builds
//! [`ConcreteRunner::None`](super::runner_config::ConcreteRunner), whose every
//! `run` is [`LlmError::NotConfigured`](crate::core::abstractions::LlmError).
//! From the outside "running and extracting nothing" and "running with no
//! engine" look identical: the scheduler pass skips and the queue never
//! drains. This module keeps a process-wide, fixed-size snapshot of *why*
//! nothing happens, and warns **once per degraded episode** — not per pass.
//!
//! Readers: hosts and the MCP `capabilities` tool call [`status`]. Writers,
//! all inside the pipeline: [`note_runner`] (config → runner, from
//! `runner_config::IngestRunnerCfg::build_runner`), [`note_scheduler_started`],
//! [`note_pass`] and [`note_skipped_pass`] (the scheduler loop / pass).
//!
//! ```ignore
//! let s = vanta_memory::ingest::observability::status();
//! if s.degraded && s.queue.depth > 0 {
//!     // runner is `none` and nothing will ever drain the queue
//! }
//! ```
//!
//! The state is process-wide on purpose: the scheduler runs in the process
//! that owns the DB (ADR-0054), so the same process — server today, `--mcp`
//! with MCP-43 — is the only one that can answer. Resident footprint is
//! constant (one small struct behind a mutex, no data-dependent growth —
//! memory-budget rule 4).

use std::sync::{Mutex, MutexGuard, OnceLock};

use serde::Serialize;

use crate::core::conversation::now_ms;

/// Which LLM runner the host built for the pipeline (last build wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerState {
    /// No runner built in this process yet (no scheduler, no ingest call).
    #[default]
    Unknown,
    /// `ConcreteRunner::None` — no engine: every extraction is `NotConfigured`.
    None,
    /// Ollama (local HTTP) configured.
    Ollama,
    /// OpenAI-compatible endpoint with a key configured.
    #[serde(rename = "openai")]
    OpenAi,
}

impl RunnerState {
    /// Stable lowercase label (also the serialized form).
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::None => "none",
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
        }
    }

    /// `true` when the pipeline cannot extract anything: there is no engine.
    pub fn is_degraded(self) -> bool {
        matches!(self, Self::None)
    }
}

/// Queue depth as last sampled by a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct QueueSnapshot {
    /// Tasks waiting to be claimed.
    pub depth: usize,
    /// Claimed-but-unfinished tasks (a live worker's leases).
    pub claimed: usize,
}

/// Outcome of the most recent pass (skipped or not).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PassSnapshot {
    /// Wall clock of the pass (ms since the Unix epoch).
    pub at_ms: u64,
    /// `true` = the pass never ran because no runner was configured.
    pub skipped: bool,
    /// Tasks processed (worker + reclaimed), when the pass ran.
    pub processed: usize,
    /// Tasks that failed and went down the retry/dead-letter path.
    pub failed: usize,
    /// Expired timers dispatched into queue tasks.
    pub timers_fired: usize,
}

/// Public, additive view of the ingest pipeline (serde-friendly: this is the
/// shape the MCP `capabilities` tool embeds under `ingest`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngestStatus {
    /// Runner the host built for the last pass.
    pub runner: RunnerState,
    /// `runner == none`: extraction is disabled, permanently, until configured.
    pub degraded: bool,
    /// A scheduler loop is running in this process.
    pub scheduler_running: bool,
    /// Queue depth as of the last pass (`depth == 0 && claimed == 0` = idle).
    pub queue: QueueSnapshot,
    /// Last pass outcome; `None` = no pass observed in this process.
    pub last_pass: Option<PassSnapshot>,
    /// Process-lifetime count of degraded-with-backlog warnings emitted
    /// (mechanical proof of "once per episode").
    pub degraded_warnings: u32,
    /// Human-readable diagnosis, `None` when the pipeline is healthy.
    pub notice: Option<String>,
}

impl Default for IngestStatus {
    fn default() -> Self {
        Self {
            runner: RunnerState::Unknown,
            degraded: false,
            scheduler_running: false,
            queue: QueueSnapshot::default(),
            last_pass: None,
            degraded_warnings: 0,
            notice: None,
        }
    }
}

impl IngestStatus {
    /// The distinction this whole module exists for: "nothing to ingest" vs
    /// "the runner is `none` and nothing will ever be extracted".
    pub fn is_broken(&self) -> bool {
        self.degraded && self.queue.depth > 0
    }
}

#[derive(Debug)]
struct State {
    status: IngestStatus,
    /// Warn already emitted for the current degraded episode.
    warned: bool,
}

fn state() -> MutexGuard<'static, State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    let mutex = STATE.get_or_init(|| {
        Mutex::new(State {
            status: IngestStatus::default(),
            warned: false,
        })
    });
    // A poisoned mutex means a panic while holding it; the state is still
    // structurally sound (plain flags), and callers cannot act on poison —
    // same convention as `LocalStateBackend::lock`.
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Record the runner the config just built (`build_runner` is the only place
/// that maps provider → `ConcreteRunner`). A real engine ends the degraded
/// episode, so the next outage warns again.
pub fn note_runner(runner: RunnerState) {
    let mut st = state();
    st.status.runner = runner;
    st.status.degraded = runner.is_degraded();
    if !runner.is_degraded() {
        st.warned = false;
    }
}

/// Record that a scheduler loop is alive in this process.
pub fn note_scheduler_started() {
    state().status.scheduler_running = true;
}

/// Record that the scheduler loop is gone. Without this the surface would keep
/// reporting a loop that has already stopped — the same class of lie this
/// module exists to delete.
pub fn note_scheduler_stopped() {
    state().status.scheduler_running = false;
}

/// Record a completed pass (the worker actually ran).
pub fn note_pass(processed: usize, failed: usize, timers_fired: usize, queue: QueueSnapshot) {
    let mut st = state();
    st.status.last_pass = Some(PassSnapshot {
        at_ms: now_ms(),
        skipped: false,
        processed,
        failed,
        timers_fired,
    });
    st.status.queue = queue;
    if queue.depth == 0 {
        // Drained: the episode is over, a future backlog warns again.
        st.warned = false;
    }
}

/// Record a pass skipped because no runner was configured, and warn **once**
/// per degraded episode (runner `none` + non-empty queue).
pub fn note_skipped_pass(queue: QueueSnapshot) {
    let mut st = state();
    st.status.last_pass = Some(PassSnapshot {
        at_ms: now_ms(),
        skipped: true,
        processed: 0,
        failed: 0,
        timers_fired: 0,
    });
    st.status.queue = queue;
    // A skip *is* the evidence that no runner exists: record it even if the
    // config path never reported (defensive, keeps `runner` and `skipped`
    // from contradicting each other).
    st.status.runner = RunnerState::None;
    st.status.degraded = true;

    if queue.depth == 0 {
        // Nothing waiting: a runner-less pipeline is idle, not broken.
        st.warned = false;
        return;
    }
    if st.warned {
        return;
    }
    st.warned = true;
    st.status.degraded_warnings = st.status.degraded_warnings.saturating_add(1);
    drop(st); // never hold the lock across a log call
    tracing::warn!(
        queue_depth = queue.depth,
        claimed = queue.claimed,
        "memory ingest: the host built no runner (runner: none) with queued work: \
         these tasks will never be extracted. Set VANTADB_INGEST_PROVIDER=ollama|openai \
         (openai also needs VANTADB_OPENAI_API_KEY) to enable extraction, or drain the queue."
    );
}

/// Current pipeline status (cheap clone; the `notice` is rendered here so the
/// caller never has to re-derive the diagnosis).
pub fn status() -> IngestStatus {
    let mut snapshot = state().status.clone();
    snapshot.notice = notice(&snapshot);
    snapshot
}

/// Process-wide warning counter (mechanical proof of "once per episode").
pub fn degraded_warnings() -> u32 {
    state().status.degraded_warnings
}

fn notice(s: &IngestStatus) -> Option<String> {
    if s.is_broken() {
        return Some(format!(
            "ingest runner is none: {} queued task(s) will never be extracted (the pass is \
             skipped). Set VANTADB_INGEST_PROVIDER=ollama|openai to enable extraction.",
            s.queue.depth
        ));
    }
    if s.degraded {
        return Some(
            "ingest runner is none: extraction is disabled until a provider is configured \
             (nothing queued right now)."
                .to_string(),
        );
    }
    if s.runner == RunnerState::Unknown {
        return Some("no ingest runner observed in this process yet.".to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_state_labels_match_the_serialized_form() {
        // The label is what an operator reads in `capabilities`; the serde form
        // is what a client parses. They must be the same string.
        for (state, label) in [
            (RunnerState::Unknown, "unknown"),
            (RunnerState::None, "none"),
            (RunnerState::Ollama, "ollama"),
            (RunnerState::OpenAi, "openai"),
        ] {
            assert_eq!(state.label(), label);
            assert_eq!(
                serde_json::to_value(state).unwrap(),
                serde_json::json!(label)
            );
        }
        assert!(RunnerState::None.is_degraded());
        assert!(!RunnerState::Ollama.is_degraded());
    }

    #[test]
    fn notice_separates_idle_lack_of_work_from_a_broken_pipeline() {
        let idle = IngestStatus {
            runner: RunnerState::Ollama,
            last_pass: Some(PassSnapshot {
                at_ms: 1,
                skipped: false,
                processed: 0,
                failed: 0,
                timers_fired: 0,
            }),
            ..IngestStatus::default()
        };
        assert!(!idle.is_broken());
        assert_eq!(notice(&idle), None);

        let broken = IngestStatus {
            runner: RunnerState::None,
            degraded: true,
            queue: QueueSnapshot {
                depth: 3,
                claimed: 0,
            },
            ..IngestStatus::default()
        };
        assert!(broken.is_broken());
        let text = notice(&broken).expect("degraded with backlog carries a notice");
        assert!(text.contains("3 queued task(s)"), "{text}");
    }
}
