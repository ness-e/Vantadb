// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-72 — the degraded ingest runner is *visible*, not silent.
//!
//! Default `IngestRunnerProvider::Local` → `ConcreteRunner::None`: every
//! extraction is `NotConfigured` and the scheduler pass skips. From outside,
//! "running and extracting nothing" and "running with no engine" looked
//! identical. These tests pin the distinction the surface must report:
//! runner in use + last pass + queue depth, and **one** warning per degraded
//! episode (not per pass).

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::ingest::observability::{self, QueueSnapshot, RunnerState};
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
/// `cargo test` does not). Phases below read as the state machine itself.
#[test]
fn degraded_runner_with_backlog_warns_once_per_episode_not_per_pass() {
    // Phase 1 — degraded but idle is a legitimate state, not a broken pipeline.
    observability::note_runner(RunnerState::None);
    observability::note_skipped_pass(QueueSnapshot::default());
    let idle = observability::status();
    assert_eq!(idle.runner, RunnerState::None);
    assert!(idle.degraded, "runner `none` is degraded");
    assert!(!idle.is_broken(), "no backlog → nothing is broken");
    assert_eq!(idle.degraded_warnings, 0, "no work, no warning");

    // Phase 2 — a runner-less pass with queued work IS a broken pipeline, and
    // the warning fires exactly once across repeated passes.
    let backlog = QueueSnapshot {
        depth: 4,
        claimed: 1,
    };
    for _ in 0..3 {
        observability::note_skipped_pass(backlog);
    }
    let broken = observability::status();
    assert!(broken.is_broken(), "degraded + backlog = broken");
    assert_eq!(broken.queue.depth, 4);
    assert_eq!(broken.queue.claimed, 1);
    assert_eq!(
        broken.degraded_warnings, 1,
        "one warning per degraded episode, not one per pass"
    );
    let last = broken.last_pass.expect("the skipped pass is recorded");
    assert!(last.skipped, "a skip must not look like a real pass");
    let notice = broken.notice.expect("legible degraded notice");
    assert!(notice.contains("4 queued task(s)"), "{notice}");

    // Phase 3 — draining the queue closes the episode; a new backlog warns again.
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

    // Phase 4 — a real runner clears the degraded state (and can extract).
    observability::note_runner(RunnerState::Ollama);
    let healthy = observability::status();
    assert!(!healthy.degraded);
    assert!(!healthy.is_broken());
    assert_eq!(healthy.runner, RunnerState::Ollama);

    // Phase 5 — the loop's liveness flag must be able to go back to false:
    // a stopped loop reported as running is the same class of lie as a silent
    // runner (the loop and its `Drop` both clear it).
    observability::note_scheduler_started();
    assert!(observability::status().scheduler_running);
    observability::note_scheduler_stopped();
    assert!(
        !observability::status().scheduler_running,
        "a stopped loop must not keep reporting itself alive"
    );

    // Phase 6 — the pass publishes its own outcome, so "idle" (nothing to
    // ingest) is readable without a scheduler loop.
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

    let status = observability::status();
    let last = status.last_pass.expect("run_pass publishes its outcome");
    assert!(!last.skipped, "the pass really ran");
    assert_eq!(
        last.processed,
        stats.worker.processed + stats.reclaimed.processed
    );
    assert_eq!(last.failed, stats.worker.failed + stats.reclaimed.failed);
    assert_eq!(status.queue.depth, 0, "the task was consumed");
    assert!(!status.is_broken());
}
