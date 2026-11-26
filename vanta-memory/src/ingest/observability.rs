//! MEM-72 — a degraded ingest runner is visible instead of silent.
//!
//! [`IngestRunnerProvider::Local`](super::runner_config::IngestRunnerProvider)
//! (the default) builds
//! [`ConcreteRunner::None`](super::runner_config::ConcreteRunner), whose every
//! `run` is `LlmError::NotConfigured`.
//! From the outside "running and extracting nothing" and "running with no
//! engine" look identical: the scheduler pass skips and the queue never
//! drains. This module keeps a process-wide, fixed-size snapshot of *why*
//! nothing happens, and warns **once per degraded episode** — not per pass.
//!
//! Readers: hosts and the MCP `capabilities` tool call [`status_with_queue`]
//! (or [`status`], which has no queue to read and says so). Writers, all
//! inside the pipeline: [`note_runner`] (config → runner, from
//! `runner_config::IngestRunnerCfg::build_runner`), [`note_scheduler_started`],
//! [`note_pass`], [`note_skipped_pass`], [`note_backlog`] (the scheduler
//! loop / pass / enqueue path) and [`note_backfill`] (FIND-745's startup sweep
//! of captured-but-unconsolidated L0 turns).
//!
//! ```ignore
//! let queue = QueueSnapshot { depth: 3, claimed: 0 };      // read NOW
//! let s = vanta_memory::ingest::observability::status_with_queue(Some(queue));
//! match s.health {
//!     IngestHealth::Broken => { /* queued work nobody will drain */ }
//!     IngestHealth::Unknown => { /* this process cannot see a queue */ }
//!     _ => {}
//! }
//! ```
//!
//! ADR-0065: the queue occupancy reported by [`status_with_queue`] is read by
//! the caller at answer time — never a photograph of the previous pass. The
//! pass sample lives in [`PassSnapshot::queue`], next to the `at_ms` that dates
//! it. [`IngestStatus::is_broken`] is derived from the live read only.
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

/// Queue occupancy: waiting tasks and live leases (ADR-0065 D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct QueueSnapshot {
    /// Tasks waiting to be claimed.
    pub depth: usize,
    /// Claimed-but-unfinished tasks (a live worker's leases).
    pub claimed: usize,
}

/// Four-state ingest health (ADR-0065 D3). Serialized as
/// `unknown | idle | draining | broken`.
///
/// `unknown` exists because a boolean cannot say "I do not know", and "I do
/// not know" reported as `false` is the defect this module exists to delete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IngestHealth {
    /// This process cannot see a queue, or has no evidence of hosting the
    /// pipeline (a read-only writer, or a writer whose loop is disabled before
    /// the first pass). A proxy does not answer from here — it forwards the
    /// call, so the client reads the writer's own status.
    #[default]
    Unknown,
    /// Nothing queued.
    Idle,
    /// Queued work **and** something in this process can drain it.
    Draining,
    /// Queued work and nothing in this process will drain it.
    Broken,
}

/// Who can drain the queue in this process (ADR-0065 D4). Serialized as
/// `none | loop | on_demand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DrainPath {
    /// No drainer: degraded, a loop that started and stopped, or no loop and
    /// no pass ever observed.
    #[default]
    None,
    /// A scheduler loop is running here.
    Loop,
    /// No loop, but passes are driven from a request hook (`run_pass`).
    OnDemand,
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
    /// Queue occupancy **sampled by this pass**. `at_ms` dates the sample:
    /// this is history, never "now" (ADR-0065 D2).
    pub queue: QueueSnapshot,
}

/// Why a startup backfill sweep enqueued nothing (FIND-745). Serialized as
/// `nothing | requeued | marker`.
///
/// A sweep that re-queues nothing has three honest readings, and "silent" is
/// not one of them: nothing was captured, the marker already covers every
/// captured turn, or the turns were already queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BackfillReason {
    /// Nothing captured, or every pending session was already queued.
    #[default]
    Nothing,
    /// At least one session was re-enqueued.
    Requeued,
    /// Every captured session is already consolidated (the marker did it).
    Marker,
    /// The sweep could not run (storage/read error). Distinct from `nothing`:
    /// "no sweep" and "a sweep that failed" are different facts.
    Failed,
}

/// Outcome of the last startup backfill sweep (FIND-745).
///
/// Published inside `ingest` on purpose (ADR-0065 D6: additive seam, no second
/// registry): a backfill that re-queues turns must be visible, with its count,
/// in the same place the queue is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BackfillSnapshot {
    /// Wall clock of the sweep (ms since the Unix epoch).
    pub at_ms: u64,
    /// Captured sessions scanned.
    pub sessions: usize,
    /// L1 tasks re-enqueued (one per unconsolidated session).
    pub requeued: usize,
    /// Unconsolidated L0 **messages** covered by those tasks (a turn is one
    /// `L0Capture`, which may carry several messages).
    pub messages: usize,
    /// What the sweep concluded.
    pub reason: BackfillReason,
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
    /// A scheduler loop ever ran in this process (separates "never hosted"
    /// from "hosted and died" — ADR-0065 D4).
    pub scheduler_ever_started: bool,
    /// Queue occupancy read **when this answer was built**, in this process.
    /// `None` = no live read: this process cannot see a queue and must not
    /// pretend otherwise (ADR-0065 D1).
    pub queue: Option<QueueSnapshot>,
    /// Last pass outcome; `None` = no pass observed in this process.
    pub last_pass: Option<PassSnapshot>,
    /// Derived at publish time from the fields above (ADR-0065 D3).
    pub health: IngestHealth,
    /// Derived at publish time: who can drain here (ADR-0065 D4).
    pub drain_path: DrainPath,
    /// Process-lifetime count of degraded-with-backlog warnings emitted
    /// (mechanical proof of "once per episode").
    pub degraded_warnings: u32,
    /// Last startup backfill sweep (FIND-745); `None` = no sweep attempted in
    /// this process (proxy mode, read-only writer, or loop disabled).
    pub backfill: Option<BackfillSnapshot>,
    /// Human-readable diagnosis, `None` when the pipeline is healthy.
    pub notice: Option<String>,
}

impl Default for IngestStatus {
    fn default() -> Self {
        Self {
            runner: RunnerState::Unknown,
            degraded: false,
            scheduler_running: false,
            scheduler_ever_started: false,
            queue: None,
            last_pass: None,
            health: IngestHealth::Unknown,
            drain_path: DrainPath::None,
            degraded_warnings: 0,
            backfill: None,
            notice: None,
        }
    }
}

impl IngestStatus {
    /// `true` when queued work exists **and** nothing here will drain it.
    ///
    /// Reads the live queue only (ADR-0065 D5): a photograph in
    /// [`Self::last_pass`] can never make this `false` for queued work. It
    /// still cannot express `unknown` — `false` means "not proven broken";
    /// read [`Self::health`] when the answer matters.
    pub fn is_broken(&self) -> bool {
        self.health == IngestHealth::Broken
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

/// Record that a scheduler loop is alive in this process. `ever_started` is
/// the process-lifetime half: it stays `true` after the loop is gone, which is
/// what separates "never hosted" from "hosted and died" (ADR-0065 D4).
pub fn note_scheduler_started() {
    let mut st = state();
    st.status.scheduler_running = true;
    st.status.scheduler_ever_started = true;
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
        queue,
    });
    if queue.depth == 0 {
        // Drained: the episode is over, a future backlog warns again.
        st.warned = false;
    }
}

/// Record a pass skipped because no runner was configured, and warn **once**
/// per degraded episode (runner `none` + non-empty queue).
pub fn note_skipped_pass(queue: QueueSnapshot) {
    let emit = {
        let mut st = state();
        st.status.last_pass = Some(PassSnapshot {
            at_ms: now_ms(),
            skipped: true,
            processed: 0,
            failed: 0,
            timers_fired: 0,
            queue,
        });
        // A skip *is* the evidence that no runner exists: record it even if the
        // config path never reported (defensive, keeps `runner` and `skipped`
        // from contradicting each other).
        st.status.runner = RunnerState::None;
        st.status.degraded = true;
        latch_backlog(&mut st, queue)
    };
    if emit {
        warn_backlog(queue);
    }
}

/// Record the outcome of a startup backfill sweep (FIND-745).
///
/// Additive on purpose: it never touches `last_pass` (a sweep is not a pass,
/// ADR-0065 D7) and it never derives health — the single health predicate
/// stays in `derived_health`.
pub fn note_backfill(report: crate::core::conversation::BackfillReport) {
    let reason = if report.requeued > 0 {
        BackfillReason::Requeued
    } else if report.consolidated > 0 {
        BackfillReason::Marker
    } else {
        BackfillReason::Nothing
    };
    state().status.backfill = Some(BackfillSnapshot {
        at_ms: now_ms(),
        sessions: report.sessions,
        requeued: report.requeued,
        messages: report.messages,
        reason,
    });
}

/// Record a sweep that could not run (FIND-745).
///
/// `backfill: null` means "no sweep was attempted here"; a failed sweep must
/// not hide behind that, so it publishes `reason: "failed"` with zero counts.
pub fn note_backfill_failed() {
    state().status.backfill = Some(BackfillSnapshot {
        at_ms: now_ms(),
        sessions: 0,
        requeued: 0,
        messages: 0,
        reason: BackfillReason::Failed,
    });
}

/// Record backlog observed **without** a pass (the enqueue path).
///
/// ADR-0065 D7: an enqueue is not a pass and must not impersonate one, so this
/// touches only the warn latch — never `last_pass`. "Degraded with backlog" is
/// a broken pipeline and must read as such in the log instead of `idle`.
pub fn note_backlog(queue: QueueSnapshot) {
    let emit = latch_backlog(&mut state(), queue);
    if emit {
        warn_backlog(queue);
    }
}

/// Set/clear the once-per-episode latch; `true` = emit the warning now.
fn latch_backlog(st: &mut State, queue: QueueSnapshot) -> bool {
    if queue.depth == 0 {
        // Nothing waiting: a runner-less pipeline is idle, not broken. Draining
        // also closes the episode, so a future backlog warns again.
        st.warned = false;
        return false;
    }
    if st.warned {
        return false;
    }
    st.warned = true;
    st.status.degraded_warnings = st.status.degraded_warnings.saturating_add(1);
    true
}

/// Callers must have dropped the state lock (never log while holding it).
fn warn_backlog(queue: QueueSnapshot) {
    tracing::warn!(
        queue_depth = queue.depth,
        claimed = queue.claimed,
        "memory ingest: the host built no runner (runner: none) with queued work: \
         these tasks will never be extracted. Set VANTADB_INGEST_PROVIDER=ollama|openai \
         (openai also needs VANTADB_OPENAI_API_KEY) to enable extraction, or drain the queue."
    );
}

/// Who can drain the queue in this process (ADR-0065 D4).
///
/// Precedence is the ADR's: degraded wins (no engine ⇒ nothing drains), then a
/// live loop, then an on-demand host that has run a pass at least once.
fn derived_drain_path(s: &IngestStatus) -> DrainPath {
    if s.degraded {
        return DrainPath::None;
    }
    if s.scheduler_running {
        return DrainPath::Loop;
    }
    if !s.scheduler_ever_started && s.last_pass.is_some() {
        return DrainPath::OnDemand;
    }
    DrainPath::None
}

/// Health from the live queue plus the evidence of hosting the pipeline
/// (ADR-0065 D3). Precedence, in this order:
///
/// ```text
/// unknown   queue is None, OR no evidence of hosting here
/// broken    depth > 0 and drain_path == none
/// draining  depth > 0
/// idle      otherwise
/// ```
fn derived_health(s: &IngestStatus) -> IngestHealth {
    let Some(q) = s.queue else {
        return IngestHealth::Unknown;
    };
    if s.runner == RunnerState::Unknown && !s.scheduler_running && s.last_pass.is_none() {
        return IngestHealth::Unknown;
    }
    if q.depth > 0 && s.drain_path == DrainPath::None {
        return IngestHealth::Broken;
    }
    if q.depth > 0 {
        return IngestHealth::Draining;
    }
    IngestHealth::Idle
}

/// Fill the derived triple the ADR mandates, in its order: `queue` (caller's
/// live read), then `drain_path`, then `health`, then `notice`.
fn with_live_queue(mut s: IngestStatus, queue: Option<QueueSnapshot>) -> IngestStatus {
    s.queue = queue;
    s.drain_path = derived_drain_path(&s);
    s.health = derived_health(&s);
    s.notice = notice(&s);
    s
}

/// Current pipeline status with the queue read **now** by the caller
/// (ADR-0065 D6: the seam is additive — no registry, no second copy).
pub fn status_with_queue(queue: Option<QueueSnapshot>) -> IngestStatus {
    with_live_queue(state().status.clone(), queue)
}

/// Current pipeline status **without** a queue read.
///
/// Always reports `queue: null` / `health: unknown`: it has no queue, so it
/// says so instead of publishing a photograph that reads as "now".
pub fn status() -> IngestStatus {
    status_with_queue(None)
}

/// Process-wide warning counter (mechanical proof of "once per episode").
pub fn degraded_warnings() -> u32 {
    state().status.degraded_warnings
}

fn notice(s: &IngestStatus) -> Option<String> {
    let depth = s.queue.map(|q| q.depth).unwrap_or(0);
    if s.health == IngestHealth::Broken {
        if s.degraded {
            return Some(format!(
                "ingest runner is none: {} queued task(s) will never be extracted (the pass is \
                 skipped). Set VANTADB_INGEST_PROVIDER=ollama|openai to enable extraction.",
                depth
            ));
        }
        return Some(format!(
            "{} queued task(s) and no scheduler will drain them in this process \
             (drain_path: none). A loop that started and stopped, or none that was ever \
             hosted, leaves the backlog stuck: restart the host or drive a pass.",
            depth
        ));
    }
    if s.health == IngestHealth::Unknown {
        if s.queue.is_none() {
            return Some(
                "cannot see a queue here: this process does not host the ingest pipeline, so \
                 its health is unknown (not idle). Read `ingest` from the process that owns \
                 the queue."
                    .to_string(),
            );
        }
        return Some(
            "no evidence of hosting the ingest pipeline here (no runner observed, no pass, no \
             loop), so its health is unknown rather than idle."
                .to_string(),
        );
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

    /// The derived triple is only consistent when it is computed (ADR-0065
    /// § Negative consequences), so the notice cases go through
    /// `with_live_queue` — same path `status_with_queue` takes.
    #[test]
    fn notice_separates_idle_lack_of_work_from_a_broken_pipeline() {
        let idle = with_live_queue(
            IngestStatus {
                runner: RunnerState::Ollama,
                last_pass: Some(PassSnapshot {
                    at_ms: 1,
                    skipped: false,
                    processed: 0,
                    failed: 0,
                    timers_fired: 0,
                    queue: QueueSnapshot::default(),
                }),
                ..IngestStatus::default()
            },
            Some(QueueSnapshot::default()),
        );
        assert_eq!(idle.health, IngestHealth::Idle);
        assert!(!idle.is_broken());
        assert_eq!(idle.notice, None);

        // Degraded (no engine) with a backlog: broken, and the text says why.
        let broken = with_live_queue(
            IngestStatus {
                runner: RunnerState::None,
                degraded: true,
                ..IngestStatus::default()
            },
            Some(QueueSnapshot {
                depth: 3,
                claimed: 0,
            }),
        );
        assert_eq!(broken.health, IngestHealth::Broken);
        assert!(broken.is_broken());
        let text = notice(&broken).expect("degraded with backlog carries a notice");
        assert!(text.contains("3 queued task(s)"), "{text}");
        assert!(text.contains("runner is none"), "{text}");

        // A healthy runner with a backlog and no drainer: broken for a
        // different reason, and the text must NOT blame the runner (ADR-0065
        // § D3 / FIND-748 C2).
        let no_drainer = with_live_queue(
            IngestStatus {
                runner: RunnerState::Ollama,
                ..IngestStatus::default()
            },
            Some(QueueSnapshot {
                depth: 2,
                claimed: 0,
            }),
        );
        assert_eq!(no_drainer.health, IngestHealth::Broken);
        let text = notice(&no_drainer).expect("a backlog with no drainer carries a notice");
        assert!(text.contains("2 queued task(s)"), "{text}");
        assert!(!text.contains("runner is none"), "{text}");

        // No live read => unknown, never idle.
        let unknown = with_live_queue(
            IngestStatus {
                runner: RunnerState::Ollama,
                ..IngestStatus::default()
            },
            None,
        );
        assert_eq!(unknown.health, IngestHealth::Unknown);
        assert!(!unknown.is_broken());
        let text = notice(&unknown).expect("unknown names its cause");
        assert!(text.contains("cannot see a queue here"), "{text}");

        // A queue read exists but this process has no evidence of hosting the
        // pipeline (no runner, no loop, no pass): still unknown, and it says
        // which of the two causes it is.
        let no_evidence = with_live_queue(
            IngestStatus::default(),
            Some(QueueSnapshot {
                depth: 0,
                claimed: 0,
            }),
        );
        assert_eq!(no_evidence.health, IngestHealth::Unknown);
        assert_eq!(no_evidence.drain_path, DrainPath::None);
        assert!(!no_evidence.is_broken());
        let text = notice(&no_evidence).expect("unknown names its cause");
        assert!(text.contains("no evidence of hosting"), "{text}");
    }
}
