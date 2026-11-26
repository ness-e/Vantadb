//! MEM-55: bridges the core HTTP server's `POST /conversation/add` route to
//! the memory pipeline.
//!
//! The core cannot depend on this crate (Cargo forbids the cycle:
//! `vanta-memory → vantadb`), so the hook point is the additive
//! [`vantadb::cli_server::ConversationTrigger`] trait and this module provides
//! the implementation, next to the pipeline it drives.
//!
//! Wiring for hosts (feature `http-server`):
//!
//! ```ignore
//! let queue = Arc::new(LocalStateBackend::new(SystemClock));
//! state.conversation_trigger = Some(Arc::new(HttpCaptureBridge::new(db.clone(), queue.clone())));
//! // Later / on a host thread: run one pass of the L0→L3 planner.
//! let stats = run_bridge_pass(&queue, db.clone(), &runner);
//! ```
//!
//! That pass **is** [`crate::services::scheduler::run_pass`] — the same one
//! `vantadb-server` drives in production (WIRE-16): expired timers, one worker
//! batch whose L1 → L2 → L3 cascade is enqueued through this queue, and
//! stale-claim reclaim. The L3 persona regenerates at
//! [`crate::services::scheduler::PERSONA_TRIGGER_EVERY_N`] memories (plus the
//! cold-start / recovery / first-scene triggers).
//!
//! One pass per process (ADR-0054): if the host already runs
//! `spawn_memory_scheduler`, it does not need this, and never run both
//! concurrently — they share the `worker-{pid}` claim owner.
//!
//! Principio 4: without a configured runner hosts simply never call
//! `run_bridge_pass` - messages stay safely captured in `l0/` and queued L1
//! tasks remain pending; nothing is lost, nothing blocks the HTTP response.

use crate::core::abstractions::LlmRunner;
use crate::core::conversation::{now_ms, L0Capture, L0Message, L0Recorder, L0Role};
use crate::core::state::{TaskKind, TaskPayload};
use crate::services::scheduler::PassStats;
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::{Clock, SystemClock};
use std::sync::Arc;
use vantadb::sdk::Embedded;

/// Implements [`vantadb::cli_server::ConversationTrigger`]: captures the saved
/// message into L0 (LLM-free — data is never lost) and enqueues an L1
/// extraction task on the shared queue for the MEM-16 worker. The session id
/// is the decimal thread id string, so memories land in `l1/<thread_id>`.
pub struct HttpCaptureBridge<C: Clock = SystemClock> {
    db: Embedded,
    queue: Arc<LocalStateBackend<C>>,
}

impl<C: Clock> HttpCaptureBridge<C> {
    pub fn new(db: Embedded, queue: Arc<LocalStateBackend<C>>) -> Self {
        Self { db, queue }
    }
}

impl<C: Clock> vantadb::cli_server::ConversationTrigger for HttpCaptureBridge<C> {
    fn trigger(
        &self,
        thread_id: u128,
        role: &str,
        content: &str,
    ) -> std::result::Result<(), String> {
        let session_id = thread_id.to_string();
        let l0_role: L0Role = role
            .parse()
            .map_err(|e| format!("invalid conversation role {role:?}: {e}"))?;

        let recorder = L0Recorder::new(self.db.clone());
        recorder
            .record_turn(
                &L0Capture {
                    session_id: session_id.clone(),
                    messages: vec![L0Message {
                        id: None,
                        role: l0_role,
                        content: content.to_string(),
                        timestamp_ms: now_ms(),
                    }],
                },
                None,
            )
            .map_err(|e| format!("L0 capture failed: {e}"))?;

        self.queue.enqueue_task(TaskPayload {
            id: String::new(), // enqueue_task assigns the stable id
            kind: TaskKind::L1,
            session_id,
            priority: 1,
            created_at_ms: self.queue.now_ms(),
            attempts: 0,
        });
        Ok(())
    }
}

/// Drive one pass of the L0→L3 planner over the bridge queue with `runner`.
///
/// This is [`crate::services::scheduler::run_pass`] (one implementation, one
/// behavior): expired timers dispatched into queue tasks, one
/// `PipelineWorker` batch —
/// whose L1 → L2 → L3 cascade is enqueued through this same queue — and
/// stale-claim reclaim. The L3 persona regenerates at
/// [`PERSONA_TRIGGER_EVERY_N`](crate::services::scheduler::PERSONA_TRIGGER_EVERY_N)
/// memories, plus the cold-start / recovery / first-scene triggers. A host
/// wanting a different cadence builds its own
/// `MemoryTaskHandler`
/// and `PipelineWorker`
/// over the same queue.
///
/// One pass per process (ADR-0054): a host that already runs
/// `spawn_memory_scheduler` does not need this, and running both concurrently
/// is a host bug — they share the `worker-{pid}` claim owner, so a
/// session-lock loser requeues a clone that runs the LLM work twice.
///
/// MEM-71: this used to build a bare handler with `trigger_every_n =
/// usize::MAX` **and no queue wiring**, so a host following these docs reached
/// L1 only — L2 was never enqueued, so L3 could not run either. The
/// "extracts memories, does not regenerate personas" comment described a
/// MEM-55 scope boundary that ADR-0054 already superseded ("`run_bridge_pass`
/// se generaliza al pass del driver").
pub fn run_bridge_pass<C: Clock, R: LlmRunner>(
    queue: &LocalStateBackend<C>,
    db: Embedded,
    runner: &R,
) -> PassStats {
    crate::services::scheduler::run_pass(queue, db, runner)
}
