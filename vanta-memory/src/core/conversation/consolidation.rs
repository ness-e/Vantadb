//! FIND-745 — startup backfill of captured-but-unconsolidated L0 turns.
//!
//! ADR-0054: the L1 queue is process-local memory, so a captured turn that dies
//! before a pass used to be lost to extraction **forever** — the content
//! survives in `l0/` and the thread store, but nothing re-queues it. `MCP-43`
//! declared that ceiling (decision §Spec-6) because a correct backfill needs a
//! durable marker of *what has already been extracted*.
//!
//! The marker is [`L0Recorder::mark_consolidated`] (`l0_consolidated/<session>`,
//! written only after a pass persisted its result — or deliberately stored
//! nothing). This module reads it and re-enqueues **one L1 task per
//! unconsolidated session** — the L1 unit is the session, not the turn, so N
//! turns of one session cost one task, not N.
//!
//! # Why a session sweep and not "everything"
//!
//! Re-enqueueing every captured session on every boot is exactly the
//! duplication pre-mortem (c) names: turns already extracted re-run the LLM
//! each start. The marker is what separates "captured" from "consolidated".
//!
//! # Proxy mode
//!
//! Never called by a **plain** (unpromoted) proxy: it owns no queue (`MCP-43`),
//! so there is nothing to enqueue into — the host says so in its own startup
//! log. A proxy promoted into the writer role (MCP-48) does sweep, because from
//! that moment it owns both the storage and the queue.
//!
//! # Thread scope
//!
//! [`backfill_unconsolidated`] does engine I/O (namespace enumeration + one L0
//! read per captured session), so the host must call it off the async thread
//! (`.opencode/rules/concurrency-async.md`, R-7/INV-003); the `--mcp` host does
//! it through `spawn_blocking`.

use std::collections::BTreeSet;

use vantadb::sdk::Embedded;

use crate::core::conversation::l0_recorder::{
    L0Error, L0Recorder, CAPTURE_CURSOR_NAMESPACE_PREFIX, CONSOLIDATED_NAMESPACE_PREFIX,
    L0_NAMESPACE_PREFIX,
};
use crate::core::state::{TaskKind, TaskPayload};
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::Clock;

/// Outcome of one startup backfill sweep (FIND-745). Reported, never silent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BackfillReport {
    /// Captured sessions the sweep scanned (`l0/` or `l0_cursor/` present).
    pub sessions: usize,
    /// Sessions re-enqueued: at least one indexable turn after the marker.
    pub requeued: usize,
    /// Unconsolidated indexable **messages** covered by those tasks (a turn is
    /// one `L0Capture`, which may carry several messages).
    pub messages: usize,
    /// Sessions skipped because their marker already covers every turn.
    pub consolidated: usize,
    /// Sessions skipped because an L1 task was already queued.
    pub already_queued: usize,
}

/// Re-enqueue the L0 sessions whose turns no pass has consolidated yet.
///
/// One L1 task per session (the pass reads the whole session anyway); a session
/// already queued is left alone so a sweep can never double the backlog.
///
/// `ponytail:` the sweep enumerates every namespace in the DB at boot (same
/// O(#namespaces) cost `run_decay_maintenance_pass` already pays). If that ever
/// matters, a `l0_cursor/` index namespace is the upgrade path — not a scan
/// per session.
pub fn backfill_unconsolidated<C: Clock>(
    db: &Embedded,
    queue: &LocalStateBackend<C>,
) -> Result<BackfillReport, L0Error> {
    let recorder = L0Recorder::new(db.clone());
    let mut report = BackfillReport::default();

    let mut queued: BTreeSet<String> = queue
        .list_queued_tasks()
        .into_iter()
        .filter(|t| t.kind == TaskKind::L1)
        .map(|t| t.session_id)
        .collect();

    for session in captured_sessions(db)? {
        report.sessions += 1;
        let messages = recorder.read_indexable_messages(&session)?;
        let done = recorder.consolidated_ms(&session)?;
        // Trivial-noise turns are never extracted by design (`NOT-04`), so they
        // never count as pending — otherwise a session whose last turn is "ok"
        // would be re-enqueued on every single boot.
        let pending = messages
            .iter()
            .filter(|m| done.is_none_or(|done| m.timestamp_ms > done))
            .count();
        if pending == 0 {
            if !messages.is_empty() {
                report.consolidated += 1;
            }
            continue;
        }
        if queued.contains(&session) {
            report.already_queued += 1;
            continue;
        }
        queue.enqueue_task(TaskPayload {
            id: String::new(), // assigned by the queue on enqueue
            kind: TaskKind::L1,
            session_id: session.clone(),
            priority: 1,
            created_at_ms: queue.now_ms(),
            attempts: 0,
        });
        queued.insert(session);
        report.requeued += 1;
        report.messages += pending;
    }

    Ok(report)
}

/// Sessions that captured at least one L0 record, in stable order.
///
/// Both prefixes count: the cursor is written with the first record, but a
/// crash between the record put and the cursor write would otherwise hide a
/// session until its next turn.
fn captured_sessions(db: &Embedded) -> Result<Vec<String>, L0Error> {
    let mut sessions = BTreeSet::new();
    for ns in db.list_namespaces()? {
        let session = ns
            .strip_prefix(L0_NAMESPACE_PREFIX)
            .or_else(|| ns.strip_prefix(CAPTURE_CURSOR_NAMESPACE_PREFIX))
            .or_else(|| ns.strip_prefix(CONSOLIDATED_NAMESPACE_PREFIX));
        if let Some(session) = session.filter(|s| !s.is_empty()) {
            sessions.insert(session.to_string());
        }
    }
    Ok(sessions.into_iter().collect())
}

#[cfg(test)]
mod tests {
    //! Process-light: the queue is a *local* backend here, so a fresh instance
    //! is a faithful "restart" (nothing is shared with the other tests).
    use super::*;
    use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
    use crate::core::conversation::{L0Capture, L0Message, L0Role};
    use crate::core::record::read_session_records;
    use crate::services::scheduler::run_pass;
    use crate::utils::managed_timer::SystemClock;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    const EXTRACTION_JSON: &str = r#"[
      {"scene_name": "Storage", "message_ids": [], "memories": [
        {"content": "The user prefers the fjall storage backend", "type": "preference", "priority": 80, "source_message_ids": []}
      ]}
    ]"#;

    /// Counts extraction rounds: the mechanical proof that a consolidated turn
    /// is not re-extracted after a restart.
    struct CountingRunner(Arc<AtomicUsize>);

    impl LlmRunner for CountingRunner {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            match params.task_id.as_str() {
                "l1-extraction" => {
                    self.0.fetch_add(1, Ordering::SeqCst);
                    Ok(EXTRACTION_JSON.to_string())
                }
                "l1-conflict-detection" => Ok("[]".to_string()),
                _ => Ok("[]".to_string()),
            }
        }
    }

    fn capture(db: &Embedded, session: &str, content: &str) -> u64 {
        let ts = crate::core::conversation::now_ms();
        L0Recorder::new(db.clone())
            .record_turn(
                &L0Capture {
                    session_id: session.to_string(),
                    messages: vec![L0Message {
                        id: None,
                        role: L0Role::User,
                        content: content.to_string(),
                        timestamp_ms: ts,
                    }],
                },
                None,
            )
            .expect("capture");
        ts
    }

    fn enqueue_l1(queue: &LocalStateBackend<SystemClock>, session: &str) {
        queue.enqueue_task(TaskPayload {
            id: String::new(),
            kind: TaskKind::L1,
            session_id: session.to_string(),
            priority: 1,
            created_at_ms: queue.now_ms(),
            attempts: 0,
        });
    }

    /// The ticket's contract, turn by turn:
    ///
    /// 1. a turn captured but never passed survives a restart *because* the
    ///    backfill re-queues it and the next pass consolidates it;
    /// 2. a turn consolidated before the "kill" is **not** re-queued after the
    ///    restart — zero new extraction calls for it (the duplication
    ///    pre-mortem);
    /// 3. a turn captured after the restart costs exactly one new extraction.
    #[test]
    fn backfill_requeues_only_unconsolidated_l0_turns_across_restarts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = vantadb::storage::StorageEngine::open(dir.path().to_str().expect("utf8"))
            .expect("open");
        let db = Embedded::from_engine(Arc::new(storage));
        let calls = Arc::new(AtomicUsize::new(0));
        let runner = CountingRunner(Arc::clone(&calls));

        // ── Boot 1: turn 1 captured and consolidated by a real pass.
        let boot1_queue = LocalStateBackend::new(SystemClock);
        let t1 = capture(
            &db,
            "thread-1",
            "I prefer the fjall storage backend for this project",
        );
        enqueue_l1(&boot1_queue, "thread-1");
        run_pass(&boot1_queue, db.clone(), &runner);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "one extraction round for turn 1"
        );
        let marker = L0Recorder::new(db.clone())
            .consolidated_ms("thread-1")
            .expect("marker read")
            .expect("the pass must have written the durable marker");
        assert!(marker >= t1, "marker {marker} covers turn 1 at {t1}");
        assert!(
            read_session_records(&db, "thread-1")
                .expect("l1 read")
                .iter()
                .any(|r| r.content.contains("fjall")),
            "turn 1 is consolidated"
        );

        // ── Restart: the queue is in-memory (ADR-0054), so boot 2 starts empty.
        let boot2_queue = LocalStateBackend::new(SystemClock);
        let report = backfill_unconsolidated(&db, &boot2_queue).expect("backfill");
        assert_eq!(
            report.requeued, 0,
            "the consolidated turn must not be re-queued: {report:?}"
        );
        assert_eq!(report.consolidated, 1, "skipped by marker: {report:?}");
        assert!(
            boot2_queue.list_queued_tasks().is_empty(),
            "no task for an already consolidated session"
        );
        run_pass(&boot2_queue, db.clone(), &runner); // empty pass
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "the extraction count did not rise for the consolidated turn"
        );

        // ── Turn 2 captured after the restart: one new extraction, not two.
        capture(
            &db,
            "thread-1",
            "Please remember the deploy window is Friday morning",
        );
        let boot3_queue = LocalStateBackend::new(SystemClock);
        let report = backfill_unconsolidated(&db, &boot3_queue).expect("backfill");
        assert_eq!(
            report.requeued, 1,
            "the un-consolidated turn comes back: {report:?}"
        );
        assert_eq!(
            report.messages, 1,
            "exactly one pending message: {report:?}"
        );
        run_pass(&boot3_queue, db.clone(), &runner);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "one new extraction round for turn 2"
        );
    }

    /// A turn captured and killed **before any pass** (the row this closes):
    /// the next boot re-queues it, and the pass makes it memory.
    #[test]
    fn backfill_recovers_a_turn_that_died_before_the_first_pass() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = vantadb::storage::StorageEngine::open(dir.path().to_str().expect("utf8"))
            .expect("open");
        let db = Embedded::from_engine(Arc::new(storage));
        capture(
            &db,
            "thread-2",
            "I want the deploy key stored in the vault, not in chat",
        );

        let queue = LocalStateBackend::new(SystemClock);
        let report = backfill_unconsolidated(&db, &queue).expect("backfill");
        assert_eq!(report.sessions, 1, "one captured session: {report:?}");
        assert_eq!(
            report.requeued, 1,
            "the orphan turn is re-queued: {report:?}"
        );

        // A second sweep is a no-op: the session is already queued.
        let again = backfill_unconsolidated(&db, &queue).expect("backfill");
        assert_eq!(
            again.requeued, 0,
            "never two tasks for one session: {again:?}"
        );
        assert_eq!(again.already_queued, 1);

        run_pass(
            &queue,
            db.clone(),
            &CountingRunner(Arc::new(AtomicUsize::new(0))),
        );
        assert!(
            read_session_records(&db, "thread-2")
                .expect("l1 read")
                .iter()
                .any(|r| r.content.contains("fjall")),
            "the recovered turn is consolidated"
        );
        assert_eq!(
            backfill_unconsolidated(&db, &LocalStateBackend::new(SystemClock))
                .expect("backfill")
                .requeued,
            0,
            "after the pass the marker makes the next boot quiet"
        );
    }
}
