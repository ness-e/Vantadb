use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, oneshot, Mutex};

use crate::error::Result;
use crate::node::{FieldValue, UnifiedNode};
use crate::storage::StorageEngine;

/// A unit of work to be processed by the async ingestion pipeline.
#[derive(Debug, Clone)]
pub struct IngestionTask {
    /// Unique node identifier.
    pub id: u128,
    /// Vector embedding data.
    pub vector: Vec<f32>,
    /// Associated text content stored as a relational field.
    pub text: String,
    /// Optional metadata key-value pairs stored as relational fields.
    pub metadata: HashMap<String, String>,
}

/// An async ingestion pipeline that distributes [`IngestionTask`] items across
/// a configurable pool of worker tasks.
///
/// Each task is converted into a [`UnifiedNode`] and inserted into the
/// [`StorageEngine`]. The caller receives a `Future<Output=Result<u128>>`
/// resolving to the insertion duration in microseconds.
pub struct AsyncIngestionPipeline {
    sender: mpsc::Sender<(IngestionTask, oneshot::Sender<Result<u128>>)>,
}

impl AsyncIngestionPipeline {
    /// Create a new pipeline with the given number of workers.
    ///
    /// `worker_count` defaults to 4 when `None` is passed. At least one worker
    /// is always created.
    pub fn new(engine: Arc<StorageEngine>, worker_count: Option<usize>) -> Self {
        let count = worker_count.unwrap_or(4).max(1);
        let (tx, rx) = mpsc::channel::<(IngestionTask, oneshot::Sender<Result<u128>>)>(1024);
        let rx = Arc::new(Mutex::new(rx));

        for _ in 0..count {
            let rx = Arc::clone(&rx);
            let engine = Arc::clone(&engine);
            tokio::spawn(async move {
                Self::worker_loop(rx, engine).await;
            });
        }

        Self { sender: tx }
    }

    /// Submit a task and return a future that resolves to the insertion
    /// duration in microseconds.
    pub async fn submit(&self, task: IngestionTask) -> Result<u128> {
        let (tx, rx) = oneshot::channel();
        self.sender.send((task, tx)).await.map_err(|_| {
            crate::error::VantaError::IoError(std::io::Error::other(
                "async ingestion pipeline closed",
            ))
        })?;
        rx.await.map_err(|_| {
            crate::error::VantaError::IoError(std::io::Error::other(
                "worker task terminated before responding",
            ))
        })?
    }

    async fn worker_loop(
        rx: Arc<Mutex<mpsc::Receiver<(IngestionTask, oneshot::Sender<Result<u128>>)>>>,
        engine: Arc<StorageEngine>,
    ) {
        loop {
            let item = {
                let mut lock = rx.lock().await;
                lock.recv().await
            };
            match item {
                Some((task, response_tx)) => {
                    let start = Instant::now();
                    let result = Self::process(&engine, task);
                    let _ = response_tx.send(result.map(|_| start.elapsed().as_micros()));
                }
                None => break,
            }
        }
    }

    fn process(engine: &StorageEngine, task: IngestionTask) -> Result<()> {
        let mut node = UnifiedNode::with_vector(task.id, task.vector);
        if !task.text.is_empty() {
            node.set_field("text", FieldValue::String(task.text));
        }
        for (key, value) in &task.metadata {
            node.set_field(key.as_str(), FieldValue::String(value.clone()));
        }
        engine.insert(&node)
    }
}
