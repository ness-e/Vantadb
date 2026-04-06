use tokio::sync::mpsc;

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
    /// A node was flagged as HALLUCINATION and purged from the graph.
    HallucinationPurged {
        node_id: u64,
        reason: String,
    },
    /// Hardware profile changed, forcing a full re-benchmark.
    EnvironmentDrift {
        old_hash: u64,
        new_hash: u64,
    },
}

/// Dispatcher that manages an async MPSC channel for invalidation events.
/// Producers (SleepWorker, DevilsAdvocate) send events.
/// Consumers (MCP API, Webhooks, Logging) receive and act on them.
pub struct InvalidationDispatcher {
    sender: mpsc::Sender<InvalidationEvent>,
    receiver: Option<mpsc::Receiver<InvalidationEvent>>,
}

impl InvalidationDispatcher {
    /// Create a new dispatcher with bounded channel capacity.
    /// The capacity acts as backpressure: if the consumer is slow,
    /// producers will await (not block the Tokio runtime).
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
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
    pub async fn emit_premise_invalidated(
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
        if let Err(e) = sender.send(event).await {
            eprintln!("⚠️ [Invalidation] Failed to emit PREMISE_INVALIDATED: {}", e);
        }
    }

    /// Emit a HALLUCINATION_PURGED event.
    pub async fn emit_hallucination_purged(
        sender: &mpsc::Sender<InvalidationEvent>,
        node_id: u64,
        reason: String,
    ) {
        let event = InvalidationEvent::HallucinationPurged { node_id, reason };
        if let Err(e) = sender.send(event).await {
            eprintln!("⚠️ [Invalidation] Failed to emit HALLUCINATION_PURGED: {}", e);
        }
    }
}

/// Background consumer task that logs invalidation events.
/// In production this would forward to MCP/Webhooks.
pub async fn invalidation_listener(mut receiver: mpsc::Receiver<InvalidationEvent>) {
    while let Some(event) = receiver.recv().await {
        match &event {
            InvalidationEvent::PremiseInvalidated { node_id, old_epoch, new_epoch, reason } => {
                eprintln!(
                    "🔴 [INVALIDATION] PREMISE_INVALIDATED: Node {} | Epoch {} → {} | Reason: {}",
                    node_id, old_epoch, new_epoch, reason
                );
            }
            InvalidationEvent::HallucinationPurged { node_id, reason } => {
                eprintln!(
                    "🧨 [INVALIDATION] HALLUCINATION_PURGED: Node {} | Reason: {}",
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
