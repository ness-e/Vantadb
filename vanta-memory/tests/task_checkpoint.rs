// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-20 — resumable task checkpoints (dim 1): contract tests.
//!
//! The task checkpoint is a separate domain from the pipeline checkpoint
//! (`tests/pipeline_manager.rs`): one JSON record per task under
//! `task_checkpoints`, mutated through [`TaskCheckpointManager`] RMW calls.
//!
//! What could break: the namespace colliding with `pipeline_checkpoint`,
//! `begin` clobbering in-progress progress, `advance` losing partial results
//! across manager instances, and misuse (advance on finished or missing
//! checkpoints) passing silently.

use serde_json::json;
use vanta_memory::utils::{
    CheckpointManager, TaskCheckpointError, TaskCheckpointManager, TaskState,
    TASK_CHECKPOINT_VERSION,
};
use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

#[test]
fn begin_creates_in_progress_checkpoint_at_step_zero() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);

    let cp = manager.begin("t1").expect("begin");

    assert_eq!(cp.version, TASK_CHECKPOINT_VERSION);
    assert_eq!(cp.step, 0);
    assert_eq!(cp.state, TaskState::InProgress);
    assert!(cp.partial.is_empty(), "no partial results yet");
    assert_eq!(
        manager.load("t1").expect("load"),
        Some(cp),
        "round-trip through the store returns the same checkpoint"
    );
}

#[test]
fn begin_is_idempotent_and_preserves_progress() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);
    manager.begin("t1").expect("begin");
    manager.advance("t1", json!({"out": "a"})).expect("advance");

    let again = manager.begin("t1").expect("begin again");

    assert_eq!(again.step, 1, "begin must not reset progress");
    assert_eq!(again.partial, vec![json!({"out": "a"})]);
    assert_eq!(again.state, TaskState::InProgress);
}

#[test]
fn advance_records_partial_results_in_order_across_instances() {
    let db = open_db();
    {
        let manager = TaskCheckpointManager::new(&db);
        manager.begin("t1").expect("begin");
        manager
            .advance("t1", json!({"step": 0}))
            .expect("advance 0");
        manager
            .advance("t1", json!({"step": 1}))
            .expect("advance 1");
    }

    // New manager instance over the same store (host restart, in-process).
    let manager = TaskCheckpointManager::new(&db);
    let cp = manager.load("t1").expect("load").expect("exists");

    assert_eq!(cp.step, 2);
    assert_eq!(cp.partial, vec![json!({"step": 0}), json!({"step": 1})]);
    assert_eq!(
        cp.partial.len() as u64,
        cp.step,
        "one partial entry per completed step"
    );
}

#[test]
fn task_checkpoint_and_pipeline_checkpoint_live_in_separate_namespaces() {
    let db = open_db();
    // Pipeline checkpoint: its own record under `pipeline_checkpoint`.
    CheckpointManager::new(&db)
        .set_persona_update_request("test")
        .expect("pipeline checkpoint write");
    // Task checkpoint: its own record under `task_checkpoints`.
    TaskCheckpointManager::new(&db).begin("t1").expect("begin");

    assert!(db
        .get("pipeline_checkpoint", "checkpoint.json")
        .expect("get")
        .is_some());
    assert!(db.get("task_checkpoints", "t1").expect("get").is_some());
    // No cross-contamination: neither key exists in the other namespace.
    assert!(db
        .get("task_checkpoints", "checkpoint.json")
        .expect("get")
        .is_none());
    assert!(db.get("pipeline_checkpoint", "t1").expect("get").is_none());

    // The task record carries only task fields — no pipeline fields leak in.
    let raw = db
        .get("task_checkpoints", "t1")
        .expect("get")
        .expect("exists");
    let value: serde_json::Value = serde_json::from_str(&raw.payload).expect("json payload");
    for field in ["version", "step", "state", "partial"] {
        assert!(value.get(field).is_some(), "task field {field} present");
    }
    for field in ["total_processed", "runner_states", "pipeline_states"] {
        assert!(
            value.get(field).is_none(),
            "pipeline field {field} must not leak into a task checkpoint"
        );
    }
}

#[test]
fn advance_on_missing_checkpoint_is_not_found() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);

    let err = manager
        .advance("nope", json!(1))
        .expect_err("advance without begin must fail");

    assert!(matches!(err, TaskCheckpointError::NotFound(id) if id == "nope"));
}

#[test]
fn delete_removes_the_checkpoint() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);
    manager.begin("t1").expect("begin");

    manager.delete("t1").expect("delete");
    assert_eq!(manager.load("t1").expect("load"), None);

    // Idempotent: deleting a missing checkpoint is a no-op.
    manager.delete("t1").expect("delete again");
}

// ═══ Resumable semantics (plan contract, Task 43 L1241) ═══

fn fjall_config(storage_path: &str) -> Config {
    Config {
        storage_path: storage_path.to_string(),
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    }
}

/// Contract test: interruption mid-way → a new instance resumes from the
/// checkpoint without repeating completed steps. Fjall + close/reopen = a
/// real host restart (durability), not just a new manager in-process.
#[test]
fn resume_after_interruption_does_not_repeat_completed_steps() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let config = fjall_config(&db_path);

    // Host A: runs steps 0 and 1 of a 5-step task, then dies mid-task.
    {
        let db = Embedded::open_with_config(config.clone()).expect("open fjall");
        let manager = TaskCheckpointManager::new(&db);
        manager.begin("t1").expect("begin");
        manager
            .advance("t1", json!({"step": 0}))
            .expect("advance 0");
        manager
            .advance("t1", json!({"step": 1}))
            .expect("advance 1");
        db.close().expect("close mid-task");
    }

    // Host B: a new instance over the durable store resumes the task.
    let db = Embedded::open_with_config(config).expect("reopen");
    let manager = TaskCheckpointManager::new(&db);
    let cp = manager
        .load("t1")
        .expect("load")
        .expect("checkpoint survived the restart");

    assert_eq!(cp.step, 2, "resume point = first uncompleted step");
    assert_eq!(cp.state, TaskState::InProgress);
    assert_eq!(
        cp.partial,
        vec![json!({"step": 0}), json!({"step": 1})],
        "completed-step results are preserved"
    );

    let mut executed = Vec::new();
    for step in cp.step..5 {
        executed.push(step);
        manager
            .advance("t1", json!({"step": step}))
            .expect("advance");
    }
    manager.complete("t1").expect("complete");

    assert_eq!(executed, vec![2, 3, 4], "completed steps are not repeated");
    let done = manager.load("t1").expect("load").expect("exists");
    assert_eq!(done.step, 5);
    assert_eq!(done.state, TaskState::Completed);
    assert_eq!(done.partial.len(), 5);
}

#[test]
fn advance_after_completion_is_rejected() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);
    manager.begin("t1").expect("begin");
    manager.advance("t1", json!(0)).expect("advance");
    manager.complete("t1").expect("complete");

    let err = manager
        .advance("t1", json!(1))
        .expect_err("advance after complete must fail");
    assert!(matches!(
        err,
        TaskCheckpointError::NotInProgress {
            state: TaskState::Completed,
            ..
        }
    ));

    // Completing twice is also rejected (misuse is loud, not silent).
    let err = manager
        .complete("t1")
        .expect_err("complete twice must fail");
    assert!(matches!(err, TaskCheckpointError::NotInProgress { .. }));
}

#[test]
fn fail_preserves_partial_for_inspection() {
    let db = open_db();
    let manager = TaskCheckpointManager::new(&db);
    manager.begin("t1").expect("begin");
    manager.advance("t1", json!({"step": 0})).expect("advance");

    let failed = manager.fail("t1").expect("fail");

    assert_eq!(failed.state, TaskState::Failed);
    assert_eq!(
        failed.partial,
        vec![json!({"step": 0})],
        "partial kept for inspection"
    );
    assert_eq!(
        failed.step, 1,
        "progress kept: the host sees where it stopped"
    );

    // fail on a missing checkpoint is NotFound (same guard as advance).
    let err = manager.fail("nope").expect_err("fail missing must fail");
    assert!(matches!(err, TaskCheckpointError::NotFound(_)));
}
