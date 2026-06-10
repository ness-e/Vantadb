use std::sync::mpsc;

/// Types of invalidation events emitted by the reactive protocol.
#[derive(Debug, Clone)]
pub enum InvalidationEvent {
    /// A node's quantized representation diverged from its FP32 ground truth.
    /// The epoch was incremented and the node re-quantized.
    PremiseInvalidated {
        node_id: u64,
        old_epoch: u32,
        new_epoch: u32,
        reason: String,
    },
    /// A node was flagged as INVALIDATED and purged from the graph.
    InvalidatedPurged { node_id: u64, reason: String },
    /// Hardware profile changed, forcing a full re-benchmark.
    EnvironmentDrift { old_hash: u64, new_hash: u64 },
}

/// Dispatcher that manages a synchronous MPSC channel for invalidation events.
/// Producers (SleepWorker, DevilsAdvocate) send events.
/// Consumers (MCP API, Webhooks, Logging) receive and act on them.
pub struct InvalidationDispatcher {
    sender: mpsc::Sender<InvalidationEvent>,
    receiver: Option<mpsc::Receiver<InvalidationEvent>>,
}

impl InvalidationDispatcher {
    /// Create a new dispatcher. Bounded capacity is not used for standard blocking channels.
    pub fn new(_capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Some(receiver),
        }
    }

    /// Get a clone of the sender for producers (SleepWorker, etc.)
    pub fn sender(&self) -> mpsc::Sender<InvalidationEvent> {
        self.sender.clone()
    }

    /// Take ownership of the receiver (call once, give to the consumer task).
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<InvalidationEvent>> {
        self.receiver.take()
    }

    /// Emit a PREMISE_INVALIDATED event.
    pub fn emit_premise_invalidated(
        sender: &mpsc::Sender<InvalidationEvent>,
        node_id: u64,
        old_epoch: u32,
        new_epoch: u32,
        reason: String,
    ) {
        let event = InvalidationEvent::PremiseInvalidated {
            node_id,
            old_epoch,
            new_epoch,
            reason,
        };
        if let Err(e) = sender.send(event) {
            eprintln!(
                "⚠️ [Invalidation] Failed to emit PREMISE_INVALIDATED: {}",
                e
            );
        }
    }

    /// Emit a INVALIDATED_PURGED event.
    pub fn emit_invalidated_purged(
        sender: &mpsc::Sender<InvalidationEvent>,
        node_id: u64,
        reason: String,
    ) {
        let event = InvalidationEvent::InvalidatedPurged { node_id, reason };
        if let Err(e) = sender.send(event) {
            eprintln!("⚠️ [Invalidation] Failed to emit INVALIDATED_PURGED: {}", e);
        }
    }
}

/// Background consumer task that logs invalidation events.
/// In production this would forward to MCP/Webhooks.
pub fn invalidation_listener(receiver: mpsc::Receiver<InvalidationEvent>) {
    while let Ok(event) = receiver.recv() {
        match &event {
            InvalidationEvent::PremiseInvalidated {
                node_id,
                old_epoch,
                new_epoch,
                reason,
            } => {
                eprintln!(
                    "🔴 [INVALIDATION] PREMISE_INVALIDATED: Node {} | Epoch {} → {} | Reason: {}",
                    node_id, old_epoch, new_epoch, reason
                );
            }
            InvalidationEvent::InvalidatedPurged { node_id, reason } => {
                eprintln!(
                    "🧨 [INVALIDATION] INVALIDATED_PURGED: Node {} | Reason: {}",
                    node_id, reason
                );
            }
            InvalidationEvent::EnvironmentDrift { old_hash, new_hash } => {
                eprintln!(
                    "🦎 [INVALIDATION] ENVIRONMENT_DRIFT: Hardware signature changed {} → {}",
                    old_hash, new_hash
                );
            }
        }
    }
    eprintln!("[INVALIDATION] Listener channel closed. Dispatcher shut down.");
}
