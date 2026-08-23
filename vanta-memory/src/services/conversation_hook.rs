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
//! // Later / on a host thread: drain queued tasks through the MEM-16 worker.
//! run_bridge_pass(&queue, db.clone(), &runner);
//! ```
//!
//! Principio 4: without a configured runner hosts simply never call
//! [`run_bridge_pass`] — messages stay safely captured in `l0/` and queued L1
//! tasks remain pending; nothing is lost, nothing blocks the HTTP response.

use crate::core::abstractions::LlmRunner;
use crate::core::conversation::{now_ms, L0Capture, L0Message, L0Recorder, L0Role};
use crate::core::record::{L1DedupConfig, L1ExtractorConfig};
use crate::core::state::{TaskKind, TaskPayload};
use crate::services::pipeline_worker::{MemoryTaskHandler, PipelineWorker, RunStats, WorkerConfig};
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::{Clock, SystemClock};
use std::sync::Arc;
use vantadb::sdk::VantaEmbedded;

/// Implements [`vantadb::cli_server::ConversationTrigger`]: captures the saved
/// message into L0 (LLM-free — data is never lost) and enqueues an L1
/// extraction task on the shared queue for the MEM-16 worker. The session id
/// is the decimal thread id string, so memories land in `l1/<thread_id>`.
pub struct HttpCaptureBridge<C: Clock = SystemClock> {
    db: VantaEmbedded,
    queue: Arc<LocalStateBackend<C>>,
}

impl<C: Clock> HttpCaptureBridge<C> {
    pub fn new(db: VantaEmbedded, queue: Arc<LocalStateBackend<C>>) -> Self {
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

/// Drive one MEM-16 worker pass over the bridge queue with the given runner.
///
/// `trigger_every_n` is set to `usize::MAX` so the persona trigger never fires
/// here — a conversation-add pass extracts memories, it does not regenerate
/// personas. Hosts wanting the full cycle can build their own
/// [`MemoryTaskHandler`] over the same queue instead.
pub fn run_bridge_pass<C: Clock, R: LlmRunner>(
    queue: &LocalStateBackend<C>,
    db: VantaEmbedded,
    runner: &R,
) -> RunStats {
    let mut worker = PipelineWorker::new(queue, WorkerConfig::default());
    let mut handler = MemoryTaskHandler::new(
        db,
        runner,
        L1ExtractorConfig::default(),
        L1DedupConfig::default(),
        usize::MAX,
    );
    worker.run_once(&mut handler)
}
