// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-72 + ADR-0065 — the ingest surface must not lie towards "fine".
//!
//! Default `IngestRunnerProvider::Local` → `ConcreteRunner::None`: every
//! extraction is `NotConfigured` and the scheduler pass skips. From outside,
//! "running and extracting nothing" and "running with no engine" looked
//! identical. These tests pin the distinction the surface must report:
//! runner in use + last pass + **live** queue occupancy + derived health, and
//! **one** warning per degraded episode (not per pass).
//!
//! ADR-0065 (FIND-748) replaced the pass photograph with a live probe, so the
//! phases below also pin the two directions of "what feeds `is_broken()`".

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::ingest::observability::{
    self, DrainPath, IngestHealth, QueueSnapshot, RunnerState,
};
use vanta_memory::services::scheduler::run_pass;
use vanta_memory::utils::{FakeClock, LocalStateBackend};
use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

struct NoMemories;

impl LlmRunner for NoMemories {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        // Valid, empty extraction: the pass completes and taps the worker.
        Ok("[]".to_string())
    }
}

fn open_db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

/// One test on purpose: the registry is process-wide, so separate `#[test]`s
/// would share it and become order-dependent (nextest isolates processes,
/// `cargo test` does not). Phases below read as the state machine itself, and
/// their **order matters** — state accumulates.
#[test]
fn queue_is_a_live_probe_and_health_is_four_state() {
    // ═══ F1 — scheduler never started, work queued, healthy runner ═══
    // The FIND-748 hole: today's code reports `is_broken() == false` here
    // because the only queue it knows is the last pass's photograph.
    observability::note_runner(RunnerState::Ollama);
    // Deliberately NO `note_scheduler_started` / `note_pass` before this read.
    let live = QueueSnapshot {
        depth: 3,
        claimed: 0,
    };
    let s = observability::status_with_queue(Some(live));
    assert_eq!(s.queue, Some(live), "queue is the live value, not a photo");
    assert_eq!(s.last_pass, None, "an enqueue does not invent a pass");
    assert_eq!(s.drain_path, DrainPath::None);
    assert_eq!(s.health, IngestHealth::Broken);
    assert!(s.is_broken());
    assert!(
        s.notice.as_deref().unwrap_or("").contains('3'),
        "the notice must name the backlog: {:?}",
        s.notice
    );

    // ═══ MEM-72 — degraded but idle is legitimate; backlog warns once ═══
    observability::note_runner(RunnerState::None);
    observability::note_skipped_pass(QueueSnapshot::default());
    let idle = observability::status_with_queue(Some(QueueSnapshot::default()));
    assert_eq!(idle.runner, RunnerState::None);
    assert!(idle.degraded, "runner `none` is degraded");
    assert_eq!(idle.health, IngestHealth::Idle, "no work → nothing broken");
    assert!(!idle.is_broken());
    assert_eq!(idle.degraded_warnings, 0, "no work, no warning");

    let backlog = QueueSnapshot {
        depth: 4,
        claimed: 1,
    };
    for _ in 0..3 {
        observability::note_skipped_pass(backlog);
    }
    let broken = observability::status_with_queue(Some(backlog));
    assert_eq!(broken.health, IngestHealth::Broken, "degraded + backlog");
    assert!(broken.is_broken());
    assert_eq!(broken.queue, Some(backlog));
    assert_eq!(
        broken
            .last_pass
            .expect("the skipped pass is recorded")
            .queue,
        backlog,
        "ADR-0065 D2: the pass sample lives in `last_pass.queue`"
    );
    assert_eq!(
        broken.degraded_warnings, 1,
        "one warning per degraded episode, not one per pass"
    );
    let last = broken.last_pass.expect("the skipped pass is recorded");
    assert!(last.skipped, "a skip must not look like a real pass");
    let notice = broken.notice.expect("legible degraded notice");
    assert!(notice.contains("4 queued task(s)"), "{notice}");

    // Draining closes the episode; a new backlog warns again.
    observability::note_skipped_pass(QueueSnapshot::default());
    observability::note_skipped_pass(QueueSnapshot {
        depth: 2,
        claimed: 0,
    });
    assert_eq!(
        observability::degraded_warnings(),
        2,
        "a new episode is a new warning"
    );

    // ═══ F2 — after a real pass the surface reads idle ═══
    observability::note_runner(RunnerState::Ollama);
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    queue.enqueue_task(TaskPayload {
        id: String::new(),
        kind: TaskKind::L1,
        session_id: "obs-pass".to_string(),
        priority: 1,
        created_at_ms: 0,
        attempts: 0,
    });

    let stats = run_pass(&queue, db.clone(), &NoMemories);
    let (depth, claimed) = queue.queue_occupancy();
    let s = observability::status_with_queue(Some(QueueSnapshot { depth, claimed }));
    assert_eq!(s.queue, Some(QueueSnapshot::default()), "the task went out");
    assert_eq!(s.health, IngestHealth::Idle);
    assert!(!s.is_broken());
    assert_eq!(s.notice, None);
    let last = s.last_pass.expect("run_pass publishes its outcome");
    assert!(!last.skipped, "the pass really ran");
    assert_eq!(
        last.processed,
        stats.worker.processed + stats.reclaimed.processed
    );
    assert_eq!(last.failed, stats.worker.failed + stats.reclaimed.failed);
    assert_eq!(last.queue.depth, 0, "the pass sampled an empty queue");

    // ═══ F3 — what `is_broken()` reads, in both directions ═══
    // (a) the photograph says 5, the live queue says 0 → NOT broken.
    observability::note_pass(
        0,
        0,
        0,
        QueueSnapshot {
            depth: 5,
            claimed: 0,
        },
    );
    let s = observability::status_with_queue(Some(QueueSnapshot::default()));
    assert_eq!(
        s.last_pass.as_ref().unwrap().queue.depth,
        5,
        "the photograph lives in last_pass"
    );
    assert!(
        !s.is_broken(),
        "is_broken() cannot depend on the photograph"
    );

    // (b) the photograph says 0, the live queue says 2, the runner is none.
    observability::note_runner(RunnerState::None);
    observability::note_pass(0, 0, 0, QueueSnapshot::default());
    let s = observability::status_with_queue(Some(QueueSnapshot {
        depth: 2,
        claimed: 0,
    }));
    assert_eq!(s.last_pass.as_ref().unwrap().queue.depth, 0);
    assert_eq!(s.health, IngestHealth::Broken);
    assert!(s.is_broken());

    // ═══ F4 — no live read never answers `idle` ═══
    let s = observability::status();
    assert_eq!(s.queue, None);
    assert_eq!(s.health, IngestHealth::Unknown);
    assert!(
        !s.is_broken(),
        "false = «not proven broken», never «healthy»"
    );
    assert!(
        s.notice
            .as_deref()
            .unwrap_or("")
            .contains("cannot see a queue here"),
        "unknown must name its cause: {:?}",
        s.notice
    );
    let wire = serde_json::to_value(&s).expect("status serializes");
    assert_eq!(wire["queue"], serde_json::json!(null));
    assert_eq!(wire["health"], serde_json::json!("unknown"));
    assert_eq!(wire["drain_path"], serde_json::json!("none"));

    // ═══ F5 — reading does not move the latch ═══
    let before = observability::degraded_warnings();
    for _ in 0..3 {
        let _ = observability::status_with_queue(Some(QueueSnapshot {
            depth: 1,
            claimed: 0,
        }));
    }
    assert_eq!(
        observability::degraded_warnings(),
        before,
        "a read must not emit a warning"
    );

    // ═══ F6 — a loop that started and died, with work queued (MEM-72 class) ═══
    observability::note_runner(RunnerState::Ollama);
    observability::note_scheduler_started();
    observability::note_scheduler_stopped();
    let s = observability::status_with_queue(Some(QueueSnapshot {
        depth: 1,
        claimed: 0,
    }));
    assert_eq!(
        s.drain_path,
        DrainPath::None,
        "hosted-and-died is not `on_demand`"
    );
    assert_eq!(s.health, IngestHealth::Broken);
    assert!(s.is_broken());
}
