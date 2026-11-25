//! MCP-43: the `--mcp` writer hosts the `vanta-memory` scheduler and drains
//! [`crate::threads::memory_queue`] — the queue `thread_send` fills (MCP-44).
//!
//! ADR-0054: the queue lives in process-local memory, so only the process that
//! owns the DB can drain it. This module owns the *driver* (WIRE-15's
//! `spawn_memory_scheduler`); the pass itself is `vanta_memory`'s, unchanged.
//!
//! # Who gets a loop
//!
//! - **Writer** (`run_stdio_server_auto` opened the storage): one loop.
//! - **Promoted proxy** (MCP-48 re-elected this process into the writer role):
//!   one loop, started the moment it owns the storage.
//! - **Plain proxy**: **no loop** — the queue belongs to the remote writer and
//!   this process cannot drain it. A local consumer there would be a second
//!   reader over someone else's data (the class of bug MCP-48 closed).
//!
//! [`start`] is idempotent on purpose: one loop per process, and a redundant
//! call is a no-op (never a second consumer over the same queue).
//!
//! # Lifetime
//!
//! The loop must be stopped **before** the storage is flushed and before the
//! discovery file is dropped — a pass mid-claim over a flushed engine is the
//! shutdown bug this module's pre-mortem names. [`stop`] signals and joins;
//! `MemoryScheduler`'s `Drop` remains the best-effort net for early-exit paths.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use vanta_memory::core::abstractions::LlmRunner;
use vanta_memory::ingest::observability;
use vanta_memory::ingest::runner_config::{build_ingest_runner, ConcreteRunner};
use vanta_memory::services::scheduler::{spawn_memory_scheduler, MemoryScheduler};
use vantadb::storage::StorageEngine;

/// Default pass interval for `--mcp` (ms).
///
/// The HTTP host uses 60 s because it mirrors the TTL sweeper; an agent session
/// needs the previous turn recallable on the *next* turn, so this host ticks
/// far more often. A pass over an empty queue is a no-op (timer scan + empty
/// worker + reclaim), which is what makes a short interval cheap.
pub(crate) const DEFAULT_INTERVAL_MS: u64 = 5_000;

fn slot() -> MutexGuard<'static, Option<MemoryScheduler>> {
    static SLOT: OnceLock<Mutex<Option<MemoryScheduler>>> = OnceLock::new();
    // A poisoned mutex means a panic while holding it; the slot is still
    // structurally sound (an Option), same convention as `LocalStateBackend`.
    SLOT.get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Set while [`stop`] is shutting the loop down. The slot alone cannot express
/// "a stop is in progress": `stop` must move the handle out before awaiting the
/// join (a std guard cannot span an await), which would otherwise let a
/// concurrent `start` spawn a second loop in that window.
fn stopping() -> &'static AtomicBool {
    static STOPPING: AtomicBool = AtomicBool::new(false);
    &STOPPING
}

/// Pass interval from the environment (`VANTADB_SCHEDULER_INTERVAL_MS`, the
/// same knob the HTTP host reads — one name for one meaning). `0` disables the
/// loop; an unparseable value warns and keeps the default (R-5: never panic on
/// operator config).
pub(crate) fn interval_ms_from_env() -> u64 {
    match std::env::var("VANTADB_SCHEDULER_INTERVAL_MS") {
        Err(_) => DEFAULT_INTERVAL_MS,
        Ok(raw) => match raw.trim().parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                tracing::warn!(
                    value = %raw,
                    default = DEFAULT_INTERVAL_MS,
                    "invalid VANTADB_SCHEDULER_INTERVAL_MS, using the MCP default"
                );
                DEFAULT_INTERVAL_MS
            }
        },
    }
}

/// Start the process's memory scheduler over the thread capture queue.
///
/// Returns `true` only when this call actually spawned the loop. Production
/// entry point: the runner is built per pass from the same env/TOML surface the
/// HTTP host uses (FIND-112), and a runner-less config maps to `None` so the
/// pass skips observably instead of burning tasks through retry/dead-letter
/// (P4; MEM-72 records the skip).
pub(crate) fn start(storage: &std::sync::Arc<StorageEngine>) -> bool {
    let interval_ms = interval_ms_from_env();
    let cfg = crate::threads::ingest_runner_cfg(storage);
    start_with(storage, interval_ms, move || {
        match build_ingest_runner(&cfg) {
            // `None` == no engine: skip the pass, leave the queue untouched.
            None | Some(ConcreteRunner::None) => None,
            other => other,
        }
    })
}

/// [`start`] with an explicit interval and runner factory (the injection point
/// tests need: the production factory resolves to `None` without a configured
/// provider, which would make the loop skip every pass).
fn start_with<F, R>(storage: &std::sync::Arc<StorageEngine>, interval_ms: u64, factory: F) -> bool
where
    F: Fn() -> Option<R> + Send + Sync + 'static,
    R: LlmRunner + Send + 'static,
{
    if interval_ms == 0 {
        tracing::info!(
            "MCP scheduler disabled (VANTADB_SCHEDULER_INTERVAL_MS=0): captured turns stay queued"
        );
        return false;
    }
    let mut running = slot();
    if stopping().load(Ordering::SeqCst) {
        return false; // a shutdown is in progress — do not resurrect a loop
    }
    if running.is_some() {
        return false; // one loop per process — never a second consumer
    }
    let scheduler = spawn_memory_scheduler(
        crate::threads::memory_queue().clone(),
        vantadb::Embedded::from_engine(storage.clone()),
        Duration::from_millis(interval_ms),
        factory,
    );
    // The loop notes itself inside its spawned task, which on a current-thread
    // runtime only runs at the next await. `capabilities` must not report
    // `scheduler_running: false` in that window — the surface exists to be
    // believed.
    observability::note_scheduler_started();
    *running = Some(scheduler);
    tracing::info!(
        interval_ms,
        "MCP scheduler started: draining the thread capture queue"
    );
    true
}

/// Stop and join the loop, if this process has one. Idempotent.
///
/// Ceiling (shared with the HTTP host, WIRE-14/16): a pass already handed to
/// `spawn_blocking` cannot be cancelled (MOD-11), so the join waits for the pass
/// in flight — bounded by the runner's own timeout. `Drop` remains the net for
/// early-exit paths where no one calls this.
pub(crate) async fn stop() {
    let scheduler = {
        // Flag first, then the slot: a `start` that already holds the slot lock
        // when we set it re-checks the flag under that same lock and bails.
        stopping().store(true, Ordering::SeqCst);
        slot().take()
    };
    if let Some(scheduler) = scheduler {
        // Graceful: signal + join. `Drop` would abort, which skips the loop's
        // own `note_scheduler_stopped` (MEM-72) and can cut a pass mid-claim.
        scheduler.shutdown().await;
        tracing::info!("MCP scheduler stopped");
    }
    stopping().store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    // ponytail: one test — the queue and the ingest status are process-wide,
    // so the phases must run in order in one test (same rule MEM-72 followed).
    use super::*;
    use crate::handlers::tools::handle_tools_call;
    use crate::{McpConfig, McpProfile};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use vanta_memory::core::abstractions::{LlmError, LlmRunParams};
    use vantadb::executor::Executor;

    /// L1 extraction response shape per `vanta-memory/tests/l1_extractor.rs`.
    const EXTRACTION_JSON: &str = r#"[
      {"scene_name": "Storage", "message_ids": [], "memories": [
        {"content": "The user prefers the fjall storage backend", "type": "preference", "priority": 80, "source_message_ids": []}
      ]}
    ]"#;

    /// Covers the two LLM calls an L1 pass makes (`vanta-memory/tests/scheduler.rs`).
    struct ScriptedRunner;

    impl LlmRunner for ScriptedRunner {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            match params.task_id.as_str() {
                "l1-extraction" => Ok(EXTRACTION_JSON.to_string()),
                "l1-conflict-detection" => {
                    let judged = params
                        .prompt
                        .split("NEW MEMORIES TO JUDGE")
                        .nth(1)
                        .unwrap_or(&params.prompt);
                    let record_id = judged
                        .split("\"record_id\": \"")
                        .nth(1)
                        .and_then(|rest| rest.split('"').next())
                        .unwrap_or("m_0");
                    Ok(format!(
                        r#"[{{"record_id": "{record_id}", "action": "store"}}]"#
                    ))
                }
                _ => Ok("[]".to_string()),
            }
        }
    }

    #[test]
    fn interval_env_is_one_knob_with_zero_as_off_and_a_safe_default() {
        // Pure config read: no scheduler, no queue (the other test owns those).
        std::env::remove_var("VANTADB_SCHEDULER_INTERVAL_MS");
        assert_eq!(interval_ms_from_env(), DEFAULT_INTERVAL_MS);

        std::env::set_var("VANTADB_SCHEDULER_INTERVAL_MS", "0");
        assert_eq!(interval_ms_from_env(), 0, "0 is the documented off switch");

        std::env::set_var("VANTADB_SCHEDULER_INTERVAL_MS", " 1500 ");
        assert_eq!(
            interval_ms_from_env(),
            1500,
            "surrounding spaces are trimmed"
        );

        std::env::set_var("VANTADB_SCHEDULER_INTERVAL_MS", "not-a-number");
        assert_eq!(
            interval_ms_from_env(),
            DEFAULT_INTERVAL_MS,
            "operator config never panics (R-5): warn + default"
        );
        std::env::remove_var("VANTADB_SCHEDULER_INTERVAL_MS");
    }

    /// The wave's real DoD: `--mcp` writer → `thread_send` → **scheduler tick**
    /// → `memory_recall` returns the turn. The tick comes from the interval
    /// ticker the loop schedules (`run_pass` is never called by hand here), so
    /// the driver — not just the pass — is under test. `TimerScanner`'s own
    /// expiry logic is covered by `vanta-memory`; what this proves is that an
    /// L1 task enqueued by `thread_send` is drained without any other host.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn scheduler_tick_consolidates_a_captured_turn_and_stop_clears_the_flag() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = Arc::new(
            StorageEngine::open(dir.path().to_str().expect("utf8")).expect("open storage"),
        );
        let executor = Executor::new(&storage);
        let config = McpConfig {
            profile: McpProfile::Full,
            ..Default::default()
        };
        let call = |name: &str, args: Value| -> Value {
            let res = handle_tools_call(
                &Some(json!({ "name": name, "arguments": args })),
                &executor,
                &storage,
                &config,
            )
            .expect("tool call must succeed");
            serde_json::from_str(res["content"][0]["text"].as_str().expect("text content"))
                .expect("JSON payload")
        };

        // No scheduler yet: this is the "proxy / not hosted" honest state.
        assert!(
            !observability::status().scheduler_running,
            "a process that has not started a loop must not claim one"
        );

        let thread_id = call("thread_create", json!({ "title": "mcp chat" }))["thread_id"]
            .as_str()
            .expect("thread id")
            .to_string();
        let sent = call(
            "thread_send",
            json!({
                "thread_id": thread_id,
                "role": "user",
                "content": "I prefer the fjall storage backend for this project",
            }),
        );
        assert_eq!(
            sent["memory"],
            json!({ "captured": true, "queued": true }),
            "the turn must feed the pipeline: {sent}"
        );

        // Queue alone is not memory: nothing has drained it yet.
        assert_eq!(
            crate::threads::memory_queue().list_queued_tasks().len(),
            1,
            "thread_send enqueued exactly one L1 task"
        );
        let before = call(
            "memory_recall",
            json!({ "query": "fjall storage backend", "scope": "agent" }),
        );
        assert_eq!(
            before["recalled"],
            json!([]),
            "without a pass the turn is not recallable: {before}"
        );

        // Writer mode: start the loop (same call `run_stdio_server_auto` makes).
        assert!(
            start_with(&storage, 25, || Some(ScriptedRunner)),
            "the writer's first start must spawn the loop"
        );
        assert!(
            !start_with(&storage, 25, || Some(ScriptedRunner)),
            "a second start must not add a second consumer"
        );
        assert!(
            observability::status().scheduler_running,
            "the loop exists as of start(), not as of the first tick"
        );

        // Wait for the *scheduled* timer tick to consolidate it. The contract is
        // the effect (`memory_recall` returns the turn) — and the effect can
        // land before `run_pass` publishes, so the pass outcome is awaited
        // separately below instead of being sampled on the loop's cadence.
        let mut consolidated = false;
        for _ in 0..150 {
            tokio::time::sleep(Duration::from_millis(40)).await;
            let now = call(
                "memory_recall",
                json!({ "query": "fjall storage backend", "scope": "agent" }),
            );
            let hits = now["recalled"].as_array().expect("recalled array");
            if hits
                .iter()
                .any(|h| h["content"].as_str().unwrap_or_default().contains("fjall"))
            {
                consolidated = true;
                break;
            }
        }
        assert!(
            consolidated,
            "no scheduled pass consolidated the captured turn (status: {:?})",
            observability::status()
        );

        // A real pass (with the runner) published the outcome — not a skip.
        // `processed` is deliberately NOT asserted: the loop republishes
        // `last_pass` every tick, so an empty pass can overwrite the counting
        // one between two samples. The causal proof is the effect above plus
        // the L1 task leaving the queue below.
        let mut ran = false;
        for _ in 0..100 {
            if observability::status()
                .last_pass
                .is_some_and(|p| !p.skipped)
            {
                ran = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(
            ran,
            "the scheduler published no non-skipped pass: it never ran with a runner"
        );
        let still_queued = crate::threads::memory_queue()
            .list_queued_tasks()
            .into_iter()
            .any(|t| {
                t.kind == vanta_memory::core::state::TaskKind::L1 && t.session_id == thread_id
            });
        assert!(
            !still_queued,
            "the L1 task left the queue: the scheduler consumed it"
        );

        // The turn is now memory, without ever calling `memory_put` (the poll
        // above proved it via `memory_recall`); the surface must also report
        // the loop it is actually running.
        assert_eq!(
            call("capabilities", json!({}))["ingest"]["scheduler_running"],
            json!(true),
            "capabilities must report the loop it is actually running"
        );

        // Shutdown order: stop the loop, then the storage is flushed.
        stop().await;
        assert!(
            !observability::status().scheduler_running,
            "a stopped loop must not keep claiming `scheduler_running: true`"
        );
        stop().await; // idempotent
    }
}
