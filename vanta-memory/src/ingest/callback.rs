//! Ingest progress channel (MEM-31 — TDAM `engines/wiki/manager.ts:110-121`
//! port, D32: internal channel + polling, **no HTTP**).
//!
//! The worker reports [`IngestProgress`] snapshots into a shared
//! [`ProgressTracker`]; any other handle (desktop bridge, MCP tool) polls
//! [`ProgressTracker::wiki_status`] by `run_id`. Updates from a stale
//! `run_id` (packet from an older build) are discarded — the in-memory twin
//! of the core WikiStore late-packet guard (`src/wiki/store.rs:219-239`,
//! MEM-28 persists one `run_id` per build).
//!
//! P4 best-effort: the channel **never blocks** the ingest — every store
//! access uses [`std::sync::Mutex::try_lock`] and drops the update on
//! contention.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Minimum interval between same-phase progress emissions
/// (TDAM manager.ts:110 `PROGRESS_THROTTLE_MS = 500`).
pub const PROGRESS_THROTTLE_MS: u64 = 500;

/// Summary text limit (TDAM callback.ts:129 asks the LLM for ≤100 chars).
pub const SUMMARY_MAX_CHARS: usize = 100;

/// Page-list cap for summary generation (TDAM callback.ts:140 `.slice(0, 20)`).
pub const SUMMARY_MAX_PAGES: usize = 20;

/// Build phase of an ingest run (TDAM manager.ts:110-121). `Done`/`Failed`
/// are terminal states appended locally (TDAM maps them to wiki status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestPhase {
    Extracting,
    Merging,
    Indexing,
    Done,
    Failed,
}

/// One progress snapshot of an ingest run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestProgress {
    /// Build this snapshot belongs to; snapshots under any other active
    /// run_id are dropped by the tracker (late-packet guard).
    pub run_id: String,
    pub phase: IngestPhase,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    /// 0..=100 derived from `completed / total`.
    pub percent: u8,
}

impl IngestProgress {
    /// Build a snapshot with `percent` derived from completed/total.
    pub fn new(
        run_id: impl Into<String>,
        phase: IngestPhase,
        total: usize,
        completed: usize,
        failed: usize,
        skipped: usize,
    ) -> Self {
        let percent = if total == 0 {
            match phase {
                IngestPhase::Done => 100,
                _ => 0,
            }
        } else {
            completed
                .checked_mul(100)
                .and_then(|n| n.checked_div(total))
                .map_or(0, |n| n.min(100) as u8)
        };
        Self {
            run_id: run_id.into(),
            phase,
            total,
            completed,
            failed,
            skipped,
            percent,
        }
    }
}

/// Truncate a generated wiki summary to the TDAM limit (`≤100` chars,
/// char-boundary safe).
pub fn truncate_summary(summary: &str) -> String {
    summary.chars().take(SUMMARY_MAX_CHARS).collect()
}

/// Cap a page list for summary generation to `SUMMARY_MAX_PAGES` entries
/// (TDAM callback.ts:140 `.slice(0, 20)`).
pub fn cap_summary_pages<T>(pages: &[T]) -> &[T] {
    &pages[..pages.len().min(SUMMARY_MAX_PAGES)]
}

// ── tracker ──

#[derive(Default)]
struct TrackerState {
    /// Currently accepted build. Anything else is a stale packet.
    active_run_id: Option<String>,
    latest: Option<IngestProgress>,
    last_emit_ms: u64,
}

/// Shared, cloneable ingest-progress registry (D32).
///
/// Clone it so another thread/handle can poll [`Self::wiki_status`] while
/// the worker pushes updates. Lock-free for callers under contention:
/// updates use `try_lock` and are silently dropped (P4 best-effort).
#[derive(Clone, Default)]
pub struct ProgressTracker {
    inner: Arc<Mutex<TrackerState>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Accept future updates only from `run_id`; resets throttling state.
    /// Called by the worker right after `begin_processing` hands out the id.
    pub fn begin_run(&self, run_id: &str) {
        if let Ok(mut st) = self.inner.try_lock() {
            st.active_run_id = Some(run_id.to_string());
            st.latest = None;
            st.last_emit_ms = 0;
        }
    }

    /// Report progress (throttled). Returns `true` when stored.
    ///
    /// Throttle policy (TDAM manager.ts:117-121): a phase change always
    /// emits; within one phase only monotonic percent increases emit, and
    /// only after [`PROGRESS_THROTTLE_MS`] — except extracting ≥90%, which
    /// bypasses the interval so the panel sees completion approaching.
    pub fn update_progress(&self, progress: IngestProgress) -> bool {
        self.update_progress_at(progress, crate::core::conversation::now_ms())
    }

    /// Same as [`Self::update_progress`] with an injected clock (tests).
    pub fn update_progress_at(&self, progress: IngestProgress, now_ms: u64) -> bool {
        // P4: contention or poisoning must never stall the ingest — drop.
        let Ok(mut st) = self.inner.try_lock() else {
            return false;
        };
        if st.active_run_id.as_deref() != Some(progress.run_id.as_str()) {
            return false; // stale packet from an older build
        }
        if let Some(prev) = &st.latest {
            if prev.phase == progress.phase {
                if progress.percent <= prev.percent {
                    return false;
                }
                let near_end = progress.phase == IngestPhase::Extracting && progress.percent >= 90;
                if !near_end && now_ms.saturating_sub(st.last_emit_ms) < PROGRESS_THROTTLE_MS {
                    return false;
                }
            } // phase change falls through: always emitted
        }
        st.last_emit_ms = now_ms;
        st.latest = Some(progress);
        true
    }

    /// Latest snapshot for `run_id`, pollable from any cloned handle.
    /// A stale/unknown `run_id` returns `None`.
    pub fn wiki_status(&self, run_id: &str) -> Option<IngestProgress> {
        let st = self.inner.lock().ok()?;
        if st.active_run_id.as_deref() != Some(run_id) {
            return None;
        }
        st.latest.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN: &str = "wikirun-t1";

    fn prog(phase: IngestPhase, percent: u8) -> IngestProgress {
        let mut p = IngestProgress::new(RUN, phase, 10, 0, 0, 0);
        p.percent = percent;
        p.completed = usize::from(percent) * 10 / 100;
        p
    }

    impl IngestProgress {
        fn with_run(self, run_id: &str) -> Self {
            Self {
                run_id: run_id.to_string(),
                ..self
            }
        }
    }

    // ── (a) run_id viejo descartado ──

    #[test]
    fn stale_run_id_updates_are_discarded() {
        let t = ProgressTracker::new();
        t.begin_run(RUN);
        assert!(t.update_progress_at(prog(IngestPhase::Extracting, 10), 0));

        t.begin_run("wikirun-r2"); // rebuild started: old run is now stale
        assert!(
            !t.update_progress_at(prog(IngestPhase::Extracting, 50), 1000),
            "late packet from old run must be rejected"
        );
        assert_eq!(t.wiki_status(RUN), None, "old run_id not queryable");
        assert_eq!(
            t.wiki_status("wikirun-r2"),
            None,
            "no snapshot yet for new run"
        );
        assert!(t.update_progress_at(prog(IngestPhase::Merging, 10).with_run("wikirun-r2"), 1001));
        assert_eq!(
            t.wiki_status("wikirun-r2").map(|p| p.phase),
            Some(IngestPhase::Merging)
        );
    }

    #[test]
    fn unknown_run_id_never_accepted_without_begin_run() {
        let t = ProgressTracker::new();
        assert!(!t.update_progress_at(prog(IngestPhase::Extracting, 5), 0));
        assert_eq!(t.wiki_status(RUN), None);
    }

    // ── (b) throttle 500ms ──

    #[test]
    fn throttle_drops_same_phase_bursts_before_500ms() {
        let t = ProgressTracker::new();
        t.begin_run(RUN);
        assert!(
            t.update_progress_at(prog(IngestPhase::Extracting, 10), 0),
            "first emits"
        );
        assert!(!t.update_progress_at(prog(IngestPhase::Extracting, 30), 100));
        assert!(!t.update_progress_at(prog(IngestPhase::Extracting, 50), 499));
        assert!(
            t.update_progress_at(prog(IngestPhase::Extracting, 70), 500),
            "interval elapsed"
        );

        // Monotonic guard: regression or equal percent never re-emits.
        assert!(!t.update_progress_at(prog(IngestPhase::Extracting, 70), 1500));
        // Near-extract-end bypass (TDAM manager.ts:120-121).
        assert!(t.update_progress_at(prog(IngestPhase::Extracting, 90), 600));
    }

    #[test]
    fn phase_change_always_emits_regardless_of_interval() {
        let t = ProgressTracker::new();
        t.begin_run(RUN);
        assert!(t.update_progress_at(prog(IngestPhase::Extracting, 95), 0));
        assert!(t.update_progress_at(prog(IngestPhase::Merging, 5), 1));
        assert!(t.update_progress_at(prog(IngestPhase::Indexing, 5), 2));
        assert!(t.update_progress_at(prog(IngestPhase::Done, 100), 3));
    }

    // ── (c) summary truncado a límites TDAM ──

    #[test]
    fn summary_truncated_to_100_chars_and_pages_capped_to_20() {
        let long = "á".repeat(250); // multi-byte: proves char-boundary safety
        let cut = truncate_summary(&long);
        assert_eq!(cut.chars().count(), SUMMARY_MAX_CHARS);

        let pages: Vec<u8> = (0..25).collect();
        assert_eq!(cap_summary_pages(&pages).len(), SUMMARY_MAX_PAGES);
        assert_eq!(cap_summary_pages(&pages[..5]).len(), 5);
    }

    // ── (d) el canal nunca bloquea el ingest ──

    #[test]
    fn contended_channel_drops_update_instead_of_blocking() {
        let t = ProgressTracker::new();
        t.begin_run(RUN);
        // Hold the lock: try_lock fails immediately (no deadlock, same thread).
        let guard = t.inner.lock().expect("hold");
        let started = std::time::Instant::now();
        let stored = t.update_progress_at(prog(IngestPhase::Extracting, 10), 0);
        assert!(started.elapsed() < std::time::Duration::from_millis(100));
        assert!(!stored, "update dropped under contention");
        drop(guard);
        // Channel recovers once released.
        assert!(t.update_progress_at(prog(IngestPhase::Extracting, 10), 0));
    }

    // ── (e/f helpers) percent derivation ──

    #[test]
    fn percent_derives_from_completed_total() {
        let p = IngestProgress::new(RUN, IngestPhase::Merging, 8, 2, 1, 1);
        assert_eq!(p.percent, 25);
        let zero = IngestProgress::new(RUN, IngestPhase::Indexing, 0, 0, 0, 0);
        assert_eq!(zero.percent, 0);
        let done = IngestProgress::new(RUN, IngestPhase::Done, 0, 0, 0, 0);
        assert_eq!(done.percent, 100);
    }

    // ── (f) consultable desde otro handle ──

    #[test]
    fn wiki_status_pollable_from_cloned_handle() {
        let producer = ProgressTracker::new();
        let consumer = producer.clone(); // other thread/handle
        producer.begin_run(RUN);
        assert!(producer.update_progress_at(prog(IngestPhase::Merging, 40), 0));

        let snap = consumer.wiki_status(RUN).expect("visible cross-handle");
        assert_eq!(snap.phase, IngestPhase::Merging);
        assert_eq!(snap.total, 10);
        assert_eq!(snap.completed, 4);
        assert_eq!(snap.failed, 0);
        assert_eq!(snap.skipped, 0);
        assert_eq!(snap.percent, 40);
    }
}
