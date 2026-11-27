//! Scheduler service: pull-based pass + feature-gated loop helper (WIRE-15).
//!
//! ADR-0054 T2: the crate of logic contributes the reusable *pass* — zero new
//! pipeline logic — and the host contributes the *driver*. This module ships
//! both pieces the host needs:
//!
//! - `run_pass`: one pull-based pass = expired timers dispatched into queue
//!   tasks + one `PipelineWorker` pass + stale-claim reclaim. Always
//!   available (no Tokio): the default build stays pull-based and thread-free.
//! - `spawn_memory_scheduler` (feature `http-server`): the loop helper the
//!   host starts and owns — interval ticks + graceful stop — mirroring the
//!   core TTL sweeper (watch + join). The host joins it through the WIRE-14
//!   `vantadb::cli_server::BackgroundService` seam after its server loop
//!   returns; `MemoryScheduler`'s `Drop` stops it best-effort on early-exit
//!   paths that skip the graceful shutdown.
//!
//! # Runner
//!
//! Each pass builds its runner through the host-provided factory (FIND-112
//! pattern: runner per pass, built from env/TOML on the host side). A factory
//! that returns `None` skips the pass and leaves the queue untouched —
//! Principio 4: nothing is lost, nothing blocks. The skip is published in
//! [`crate::ingest::observability`] and warns **once per degraded episode**
//! when there is queued work (MEM-72), so a runner-less pipeline is never
//! silent.
//! Secrets stay on the host side (R-5): the runner config (including any key)
//! is resolved by [`crate::ingest::runner_config`], never here.
//!
//! # Configuration (env, R-5)
//!
//! This module reads exactly **one** variable, `VANTADB_SCHEDULER_INTERVAL_MS`
//! — the single read site in **this** crate for the pass interval
//! ([`crate::services::scheduler::scheduler_interval_ms_from_env_opt`], ADR-0063). A host that needs a
//! different fallback `unwrap_or`s its own default instead of parsing the
//! environment a second time. (`vantadb-mcp` still carries its own parser
//! until `WIRE-23` consumes this one.)
//!
//! # Single writer (ADR-0054)
//!
//! The scheduler runs in the process that owns the DB (the writer). The
//! queue, timers and leases live in process-local memory
//! (`LocalStateBackend`); running two loops over the same DB is a host bug —
//! spawn one per process.

use crate::core::abstractions::LlmRunner;
use crate::core::record::{L1DedupConfig, L1ExtractorConfig};
use crate::core::state::{TaskKind, TaskPayload};
use crate::ingest::observability::{self, QueueSnapshot};
use crate::services::pipeline_worker::{MemoryTaskHandler, PipelineWorker, RunStats, WorkerConfig};
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::Clock;
use crate::utils::timer_scanner::TimerScanner;
use std::time::Duration;
use vantadb::sdk::Embedded;

#[cfg(feature = "http-server")]
use std::sync::Arc;

/// Default scheduler pass interval (ms), shared by every host: mirrors the core
/// TTL sweeper's `VANTADB_TTL_SWEEP_INTERVAL_MS` default (`src/config.rs` of the
/// core crate). `0` disables the loop.
pub const DEFAULT_INTERVAL_MS: u64 = 60_000;

/// Parse the `VANTADB_SCHEDULER_INTERVAL_MS` value (R-5: warn + safe default,
/// never panic). `0` is a valid value (disables the loop).
pub fn parse_interval_ms(raw: Option<&str>) -> u64 {
    match raw {
        None => DEFAULT_INTERVAL_MS,
        Some(s) => match s.trim().parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                tracing::warn!(
                    value = %s,
                    default = DEFAULT_INTERVAL_MS,
                    "invalid VANTADB_SCHEDULER_INTERVAL_MS, using default"
                );
                DEFAULT_INTERVAL_MS
            }
        },
    }
}

/// Scheduler interval declared in the process environment, or `None` when
/// `VANTADB_SCHEDULER_INTERVAL_MS` is unset — the **single** read site of that
/// variable in this crate (R-5, ADR-0063).
///
/// Hosts that need a different fallback call this and `unwrap_or` their own
/// default (the `--mcp` host's default is 5 000 ms, the HTTP host
/// [`DEFAULT_INTERVAL_MS`]) instead of parsing the environment again. A
/// *present but unparseable* value is not "unset": [`parse_interval_ms`] warns
/// and resolves it to [`DEFAULT_INTERVAL_MS`].
pub fn scheduler_interval_ms_from_env_opt() -> Option<u64> {
    let raw = std::env::var("VANTADB_SCHEDULER_INTERVAL_MS").ok()?;
    Some(parse_interval_ms(Some(raw.as_str())))
}

/// Scheduler interval from the process environment with [`DEFAULT_INTERVAL_MS`]
/// as the fallback: a thin wrapper over
/// [`scheduler_interval_ms_from_env_opt`], which is the only place that reads
/// the environment.
pub fn scheduler_interval_ms_from_env() -> u64 {
    scheduler_interval_ms_from_env_opt().unwrap_or(DEFAULT_INTERVAL_MS)
}

/// L3 persona trigger cadence for scheduler passes: regenerate the persona
/// after this many new memories (mirrors the D19 e2e full-pass cadence).
/// Hosts needing a different cadence construct their own `PipelineWorker` +
/// `MemoryTaskHandler` over the same queue (`PipelineWorker::run_once` drives
/// the batch; the handler wiring shape is in `tests/e2e_flow.rs`). There is no
/// second pass to configure.
///
/// Public since MEM-71: a host (or a test) that needs to reason about the
/// cadence references this number instead of duplicating `50` — and no pass
/// ships a "persona never fires" sentinel like `usize::MAX`.
pub const PERSONA_TRIGGER_EVERY_N: usize = 50;

/// Statistics of one `run_pass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PassStats {
    /// Expired timers consumed from the backend — dispatched into queue tasks
    /// when the member is known (`l1_idle:`); unknown members are skipped.
    pub timers_fired: usize,
    /// Queue tasks consumed by the worker pass.
    pub worker: RunStats,
    /// Stale claims reclaimed (dead worker's lease expired) and processed.
    pub reclaimed: RunStats,
}

/// Run ONE pull-based scheduler pass over the shared queue (ADR-0054 T2).
///
/// The pass reuses the MEM-16 pieces exactly (zero new pipeline logic):
///
/// 1. [`TimerScanner::run_once`] dispatches expired `l1_idle:<session>` timers
///    as `L1` tasks for that session (priority 1). Unknown timer members are
///    consumed and skipped with a debug log (forward-compatible: L2/Dream
///    timers arrive with their producer).
/// 2. One [`PipelineWorker::run_once`] pass runs queued tasks through the
///    L0→L3 `MemoryTaskHandler` with the given `runner`.
/// 3. `PipelineWorker::reclaim_stale` recovers claims whose lease expired
///    (a dead worker's tasks), using the worker defaults for lease and limit.
///
/// LLM failures inside the worker take its existing retry/dead-letter path
/// (Principio 4 — L0 data is never lost).
pub fn run_pass<C: Clock, R: LlmRunner>(
    queue: &LocalStateBackend<C>,
    db: Embedded,
    runner: &R,
) -> PassStats {
    let timers_fired = TimerScanner::new(queue).run_once(|entry| {
        let Some(session_id) = entry.member.strip_prefix("l1_idle:") else {
            // Prefix literal mirrors `pipeline_manager::l1_idle_member` — the
            // only timer member any producer sets today.
            tracing::debug!(
                member = %entry.member,
                "scheduler: unknown timer member, skipping"
            );
            return;
        };
        queue.enqueue_task(TaskPayload {
            id: String::new(), // assigned by the queue on enqueue
            kind: TaskKind::L1,
            session_id: session_id.to_string(),
            priority: 1,
            created_at_ms: entry.fire_at_ms,
            attempts: 0,
        });
    });

    let worker_config = WorkerConfig::default();
    let mut worker = PipelineWorker::new(queue, worker_config.clone());
    let mut handler = MemoryTaskHandler::new(
        db,
        runner,
        L1ExtractorConfig::default(),
        L1DedupConfig::default(),
        PERSONA_TRIGGER_EVERY_N,
    )
    .with_queue(queue);
    let worker_stats = worker.run_once(&mut handler);
    let reclaimed = worker.reclaim_stale(
        &mut handler,
        worker_config.lock_ttl_ms,
        worker_config.batch_size,
    );

    let stats = PassStats {
        timers_fired,
        worker: worker_stats,
        reclaimed,
    };

    // MEM-72: publish the pass outcome (and the post-pass queue depth) so a
    // host can tell "nothing to ingest" from "the pass is not running".
    // ADR-0065: one acquisition for the pair — no torn `queue_depth` +
    // `pending_count` reading (the sample belongs to `last_pass`, never to the
    // published `queue`).
    let (depth, claimed) = queue.queue_occupancy();
    observability::note_pass(
        stats.worker.processed + stats.reclaimed.processed,
        stats.worker.failed + stats.reclaimed.failed,
        stats.timers_fired,
        QueueSnapshot { depth, claimed },
    );

    stats
}

/// Default interval between autonomous background decay passes (24 hours, TEC-07 / Q18).
pub const DEFAULT_DECAY_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Run autonomous decay over active namespaces (TEC-07 / Q18 / FIND-289).
///
/// Discovers active sessions from the queue and active `l1/` namespaces from
/// storage, executes `run_decay_pass`, and for any record with
/// `effective_heat <= PRUNE_HEAT_THRESHOLD`, invokes
/// `Embedded::forget(ns, key, ForgetMode::Soft)` without penalizing write latency.
pub fn run_decay_maintenance_pass<C: Clock>(
    queue: &LocalStateBackend<C>,
    db: &Embedded,
    policy: &crate::core::record::lifecycle::DecayPolicy,
    now_ms: u64,
) -> usize {
    let mut namespaces = std::collections::BTreeSet::new();
    for session in queue.list_active_sessions() {
        namespaces.insert(crate::core::record::l1_namespace(&session));
    }
    if let Ok(all_ns) = db.list_namespaces() {
        for ns in all_ns {
            if ns.starts_with("l1/") {
                namespaces.insert(ns);
            }
        }
    }

    let mut soft_forgotten = 0usize;
    for ns in namespaces {
        let session_key = ns.strip_prefix("l1/").unwrap_or(&ns);
        let _ = crate::core::record::run_decay_pass(db, session_key, policy, now_ms);

        if let Ok(records) = crate::core::record::l1_reader::read_namespace_records(db, &ns) {
            for record in records {
                let heat = crate::core::record::lifecycle::effective_heat(&record, policy, now_ms);
                if heat <= crate::core::record::lifecycle::PRUNE_HEAT_THRESHOLD {
                    match db.forget(
                        &ns,
                        vantadb::sdk::ForgetTarget::Key(record.id.clone()),
                        vantadb::sdk::ForgetMode::Soft,
                    ) {
                        Ok(_) => soft_forgotten += 1,
                        Err(e) => {
                            tracing::warn!(
                                namespace = %ns,
                                key = %record.id,
                                error = %e,
                                "decay pass: soft forget failed"
                            );
                        }
                    }
                }
            }
        }
    }
    soft_forgotten
}

/// Handle for the background memory scheduler loop.
///
/// Dropping the handle stops the loop (best-effort abort);
/// [`MemoryScheduler::shutdown`] performs a graceful stop and join.
///
/// The host joins it through the WIRE-14 `vantadb::cli_server::BackgroundService`
/// seam so no scheduler task outlives the server run; on early-exit paths that
/// skip the graceful join, `Drop` stops it best-effort (the `MemoryTtlSweeper`
/// convention).
#[cfg(feature = "http-server")]
pub struct MemoryScheduler {
    shutdown: tokio::sync::watch::Sender<bool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

#[cfg(feature = "http-server")]
impl MemoryScheduler {
    /// Ask the scheduler loop to stop and wait for it to finish (join).
    pub async fn shutdown(mut self) {
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// WIRE-14 seam: the server bootstrap joins the scheduler through the generic
/// `vantadb::cli_server::BackgroundService` path (same graceful stop as its
/// inherent [`MemoryScheduler::shutdown`]).
#[cfg(feature = "http-server")]
impl vantadb::cli_server::BackgroundService for MemoryScheduler {
    fn shutdown(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            // UFCS: resolve to the inherent graceful stop (signal + join),
            // never this trait method.
            MemoryScheduler::shutdown(*self).await;
        })
    }
}

#[cfg(feature = "http-server")]
impl Drop for MemoryScheduler {
    fn drop(&mut self) {
        // Signal first, then abort: with a live runtime the loop exits cleanly
        // at its next select; the abort guarantees no detached task outlives
        // the handle during shutdown. Covers the server's early-exit path,
        // which drops services without a graceful join (WIRE-14 handoff).
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        // MEM-72: an abort skips the loop's own `note_scheduler_stopped`, so the
        // drop path clears it too (idempotent; a stale `true` would be a lie).
        observability::note_scheduler_stopped();
    }
}

/// Spawn the background scheduler loop: every `interval`, run ONE `run_pass`
/// on a blocking thread (the pass is sync engine work — concurrency-async R1)
/// with a runner built by `runner_factory`.
///
/// The first pass runs immediately (Tokio interval semantics), so tasks left
/// queued are processed on startup; `MissedTickBehavior::Skip` mirrors the
/// TTL sweeper. `interval` must be non-zero — callers that allow disabling
/// (e.g. a `0` config value) skip this call entirely.
///
/// `runner_factory` is called once per pass (FIND-112 pattern: build the
/// runner per pass, e.g. `move || build_ingest_runner(&cfg)`). Returning
/// `None` skips the pass observably and leaves the queue untouched
/// (Principio 4 — nothing lost, nothing blocks); the skip is recorded in
/// [`crate::ingest::observability`] with the backlog depth and warns once per
/// degraded episode (MEM-72).
#[cfg(feature = "http-server")]
pub fn spawn_memory_scheduler<F, R>(
    queue: Arc<LocalStateBackend<crate::utils::managed_timer::SystemClock>>,
    db: Embedded,
    interval: Duration,
    runner_factory: F,
) -> MemoryScheduler
where
    F: Fn() -> Option<R> + Send + Sync + 'static,
    R: LlmRunner + Send + 'static,
{
    spawn_memory_scheduler_with_decay(queue, db, interval, DEFAULT_DECAY_INTERVAL, runner_factory)
}

/// Spawn the background scheduler loop with an explicit autonomous decay interval (TEC-07).
#[cfg(feature = "http-server")]
pub fn spawn_memory_scheduler_with_decay<F, R>(
    queue: Arc<LocalStateBackend<crate::utils::managed_timer::SystemClock>>,
    db: Embedded,
    interval: Duration,
    decay_interval: Duration,
    runner_factory: F,
) -> MemoryScheduler
where
    F: Fn() -> Option<R> + Send + Sync + 'static,
    R: LlmRunner + Send + 'static,
{
    debug_assert!(!interval.is_zero(), "scheduler interval must be non-zero");
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let runner_factory = Arc::new(runner_factory);
    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut decay_ticker =
            tokio::time::interval_at(tokio::time::Instant::now() + decay_interval, decay_interval);
        decay_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // MEM-72: `capabilities` reports `scheduler_running` from here.
        observability::note_scheduler_started();

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                _ = ticker.tick() => {
                    let queue = Arc::clone(&queue);
                    // Depth probe kept outside the pass (the pass consumes its
                    // own clone) so a skipped pass can still report the backlog.
                    let probe = Arc::clone(&queue);
                    let db = db.clone();
                    let runner_factory = Arc::clone(&runner_factory);
                    let outcome = tokio::task::spawn_blocking(move || {
                        let runner = runner_factory()?;
                        Some(run_pass(&queue, db, &runner))
                    })
                    .await;
                    match outcome {
                        Ok(Some(stats)) => {
                            if stats.timers_fired > 0
                                || stats.worker.processed > 0
                                || stats.reclaimed.processed > 0
                            {
                                tracing::info!(
                                    timers_fired = stats.timers_fired,
                                    processed = stats.worker.processed + stats.reclaimed.processed,
                                    failed = stats.worker.failed + stats.reclaimed.failed,
                                    "memory scheduler pass"
                                );
                            } else {
                                tracing::debug!("memory scheduler pass: idle");
                            }
                        }
                        Ok(None) => {
                            tracing::debug!(
                                "memory scheduler: no runner configured; skipping pass"
                            );
                            // MEM-72: degraded is a legitimate state, but
                            // degraded *with backlog* is a broken pipeline —
                            // the status records it and warns once per episode.
                            let (depth, claimed) = probe.queue_occupancy();
                            observability::note_skipped_pass(QueueSnapshot { depth, claimed });
                        }
                        Err(join_error) => {
                            tracing::warn!(
                                error = %join_error,
                                "memory scheduler pass task failed to join"
                            );
                        }
                    }
                }
                _ = decay_ticker.tick() => {
                    let queue = Arc::clone(&queue);
                    let db = db.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let now_ms = queue.now_ms();
                        let policy = crate::core::record::lifecycle::DecayPolicy::default();
                        let forgotten = run_decay_maintenance_pass(&queue, &db, &policy, now_ms);
                        if forgotten > 0 {
                            tracing::info!(forgotten, "autonomous decay pass completed");
                        }
                    })
                    .await;
                }
            }
        }
        // MEM-72: the loop is gone (graceful break) — stop claiming otherwise.
        observability::note_scheduler_stopped();
    });
    MemoryScheduler {
        shutdown: shutdown_tx,
        handle: Some(handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The interval rule moved here from `vantadb-server` (ADR-0063 migration
    /// row 2). The default is asserted to the byte: a move must not change what
    /// an operator gets when nothing is configured.
    #[test]
    fn interval_defaults_match_the_values_the_host_shipped() {
        assert_eq!(DEFAULT_INTERVAL_MS, 60_000, "the default must not move");
        assert_eq!(parse_interval_ms(None), DEFAULT_INTERVAL_MS);
        assert_eq!(parse_interval_ms(Some("0")), 0, "0 disables the loop");
        assert_eq!(parse_interval_ms(Some(" 1500 ")), 1500, "trimmed");
        assert_eq!(parse_interval_ms(Some(" abc ")), DEFAULT_INTERVAL_MS);
        assert_eq!(parse_interval_ms(Some("-5")), DEFAULT_INTERVAL_MS);
    }

    /// R-5, measured: `_opt` is the primitive and `from_env` is its `unwrap_or`,
    /// so the declared interval comes back as declared and is never re-read.
    #[test]
    fn interval_env_resolves_once_and_the_wrapper_only_falls_back() {
        const VAR: &str = "VANTADB_SCHEDULER_INTERVAL_MS";

        std::env::remove_var(VAR);
        assert_eq!(
            scheduler_interval_ms_from_env_opt(),
            None,
            "unset is the only None: no declared interval"
        );
        assert_eq!(scheduler_interval_ms_from_env(), DEFAULT_INTERVAL_MS);

        std::env::set_var(VAR, " 1500 ");
        assert_eq!(scheduler_interval_ms_from_env_opt(), Some(1500));
        assert_eq!(scheduler_interval_ms_from_env(), 1500);
        // Same answer on a second call: no state, nothing re-derived.
        assert_eq!(scheduler_interval_ms_from_env_opt(), Some(1500));

        std::env::set_var(VAR, "0");
        assert_eq!(scheduler_interval_ms_from_env_opt(), Some(0));
        assert_eq!(
            scheduler_interval_ms_from_env(),
            0,
            "0 is the documented off switch"
        );

        std::env::set_var(VAR, "not-a-number");
        assert_eq!(
            scheduler_interval_ms_from_env_opt(),
            Some(DEFAULT_INTERVAL_MS),
            "present-but-unparseable is R-5 (warn + default), not 'unset'"
        );
        std::env::remove_var(VAR);
    }
}
