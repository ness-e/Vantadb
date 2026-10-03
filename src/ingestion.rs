use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot, Mutex};

use crate::config::InsertBatchConfig;
use crate::error::Result;
use crate::node::{FieldValue, UnifiedNode};
use crate::storage::{BatchInsertOptions, InsertMode, StorageEngine};

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
    /// `worker_count` defaults to 1 when `None` is passed. At least one worker
    /// is always created. Default is 1 (not 4): RES-03 measured -31% (w=2) /
    /// -43% (w=4) vs w=1 on the serial insert path — see
    /// `docs/user/operations/BENCHMARKS.md` §13.
    ///
    /// Opt-in group-commit batching (WIRE-06) is read from
    /// [`Config::insert_batch`](crate::config::Config::insert_batch) on the
    /// engine: when `enabled`, workers accumulate tasks into one
    /// `batch_insert_with_opts` commit (see [`Self::worker_loop_batched`]).
    /// Default (`enabled = false`) keeps the per-record path byte-identical.
    pub fn new(engine: Arc<StorageEngine>, worker_count: Option<usize>) -> Self {
        // ponytail: single worker — engine insert path is serial (global insert_lock
        // + WAL fsync per write); more workers only add convoy. Scale up when
        // FIND-59/FUT-12 lift the serial ceiling (see BENCHMARKS §13).
        let count = worker_count.unwrap_or(1).max(1);
        let batch = engine.config.insert_batch;
        if batch.enabled && count > 1 {
            // Batch building drains the shared receiver under one lock; extra
            // workers only contend for it (and the engine insert path is serial
            // anyway). Keep the default of 1 worker with batching enabled.
            tracing::warn!(
                workers = count,
                "insert batching enabled with multiple workers: batches are built by one worker at a time; use 1 worker"
            );
        }
        let (tx, rx) = mpsc::channel::<(IngestionTask, oneshot::Sender<Result<u128>>)>(
            batch.max_queued_records.max(1),
        );
        let rx = Arc::new(Mutex::new(rx));

        for _ in 0..count {
            let rx = Arc::clone(&rx);
            let engine = Arc::clone(&engine);
            tokio::spawn(async move {
                if batch.enabled {
                    Self::worker_loop_batched(rx, engine, batch).await;
                } else {
                    Self::worker_loop(rx, engine).await;
                }
            });
        }

        Self { sender: tx }
    }

    /// Submit a task and return a future that resolves to the insertion
    /// duration in microseconds.
    pub async fn submit(&self, task: IngestionTask) -> Result<u128> {
        let (tx, rx) = oneshot::channel();
        self.sender.send((task, tx)).await.map_err(|_| {
            crate::error::Error::Io(std::io::Error::other("async ingestion pipeline closed"))
        })?;
        rx.await.map_err(|_| {
            crate::error::Error::Io(std::io::Error::other(
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
                    let engine_clone = Arc::clone(&engine);
                    let run_res =
                        tokio::task::spawn_blocking(move || Self::process(&engine_clone, task))
                            .await;

                    let result = match run_res {
                        Ok(res) => res,
                        Err(e) => Err(crate::error::Error::generic_error(format!(
                            "Ingestion worker task panicked: {}",
                            e
                        ))),
                    };
                    let _ = response_tx.send(result.map(|_| start.elapsed().as_micros()));
                }
                None => break,
            }
        }
    }

    /// Group-commit worker loop (WIRE-06, opt-in via [`InsertBatchConfig`]).
    ///
    /// Collects up to `max_batch_records` tasks — waiting at most `max_wait_ms`
    /// from the first dequeue for the batch to fill, whichever comes first
    /// (ADR-038 §Design) — and commits the whole batch with ONE
    /// `batch_insert_with_opts` call: 1× `insert_lock`, 1× WAL `batch_append`
    /// per shard (≤1 fsync per shard per cycle), 1× HNSW bulk insert.
    ///
    /// Every waiter is acked only after the commit completes (ack ⇒ applied +
    /// durable per `SyncMode`), matching the per-record path's contract. A
    /// single-task batch and the transaction-active fallback use the
    /// per-record `insert()` so error semantics and transaction buffering
    /// (ERR-013) stay identical.
    ///
    /// The gather window is driven by a `spawn_blocking` sleep + `oneshot`
    /// (tokio features `rt`+`sync` only — `tokio::time` is not part of the
    /// `async-ingestion` feature set): a batch that fills from the already
    /// queued tasks costs no timer at all.
    async fn worker_loop_batched(
        rx: Arc<Mutex<mpsc::Receiver<(IngestionTask, oneshot::Sender<Result<u128>>)>>>,
        engine: Arc<StorageEngine>,
        batch_cfg: InsertBatchConfig,
    ) {
        loop {
            let mut nodes: Vec<UnifiedNode> = Vec::with_capacity(batch_cfg.max_batch_records);
            let mut waiters: Vec<(oneshot::Sender<Result<u128>>, Instant)> =
                Vec::with_capacity(batch_cfg.max_batch_records);

            {
                let mut guard = rx.lock().await;
                let deadline = match guard.recv().await {
                    Some((task, response_tx)) => {
                        waiters.push((response_tx, Instant::now()));
                        nodes.push(Self::task_to_node(task));
                        Instant::now() + Duration::from_millis(batch_cfg.max_wait_ms)
                    }
                    None => break,
                };

                // Fast path: take everything already queued (no timer spawned
                // when the batch fills from the backlog).
                Self::drain_queued(
                    &mut guard,
                    &mut nodes,
                    &mut waiters,
                    batch_cfg.max_batch_records,
                );

                // Slow path: wait up to the remaining window for stragglers
                // (a fresh timer per wait; none is spawned while the backlog
                // still has records to drain).
                while nodes.len() < batch_cfg.max_batch_records {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        break;
                    }
                    let timer = Self::spawn_window_timer(remaining);
                    tokio::pin!(timer);
                    let mut closed = false;
                    tokio::select! {
                        biased;
                        _ = &mut timer => break, // window elapsed
                        item = guard.recv() => match item {
                            Some((task, response_tx)) => {
                                waiters.push((response_tx, Instant::now()));
                                nodes.push(Self::task_to_node(task));
                            }
                            // Channel closed: commit what we have; the next
                            // iteration sees `None` and exits.
                            None => closed = true,
                        },
                    }
                    if closed {
                        break;
                    }
                    Self::drain_queued(
                        &mut guard,
                        &mut nodes,
                        &mut waiters,
                        batch_cfg.max_batch_records,
                    );
                }
            }

            let results = Self::commit_batch(&engine, nodes).await;
            let ok = results.iter().all(|r| r.is_ok());
            tracing::debug!(records = results.len(), ok, "group-commit batch committed");
            for ((response_tx, start), result) in waiters.into_iter().zip(results) {
                let _ = response_tx.send(result.map(|()| start.elapsed().as_micros()));
            }
        }
    }

    /// Non-blocking drain of everything currently queued (bounded by `max`).
    ///
    /// `TryRecvError::Disconnected` is treated as "nothing more" — the outer
    /// loop's next blocking `recv()` observes the close and exits.
    fn drain_queued(
        guard: &mut mpsc::Receiver<(IngestionTask, oneshot::Sender<Result<u128>>)>,
        nodes: &mut Vec<UnifiedNode>,
        waiters: &mut Vec<(oneshot::Sender<Result<u128>>, Instant)>,
        max: usize,
    ) {
        while nodes.len() < max {
            match guard.try_recv() {
                Ok((task, response_tx)) => {
                    waiters.push((response_tx, Instant::now()));
                    nodes.push(Self::task_to_node(task));
                }
                Err(_) => break,
            }
        }
    }

    /// Sleep `remaining` on a blocking thread and signal the returned future.
    ///
    /// Bounded by the gather window (`max_wait_ms`), so the blocking-pool
    /// occupancy is one short sleep per batch that did not fill from the
    /// backlog — never on the async executor thread.
    fn spawn_window_timer(remaining: Duration) -> oneshot::Receiver<()> {
        let (timeout_tx, timeout_rx) = oneshot::channel::<()>();
        tokio::task::spawn_blocking(move || {
            std::thread::sleep(remaining);
            let _ = timeout_tx.send(());
        });
        timeout_rx
    }

    /// Commit one collected batch, one result per node (order-preserving).
    ///
    /// Multi-node batches run through `batch_insert_with_opts` — batch-atomic
    /// in the sense that every waiter receives an error if the commit fails
    /// (the engine reports one result per batch). `Error` is not `Clone`, so a
    /// failed batch surfaces a message-preserving per-waiter error. Single-node
    /// batches and transactions use the per-record `insert()` path.
    async fn commit_batch(engine: &Arc<StorageEngine>, nodes: Vec<UnifiedNode>) -> Vec<Result<()>> {
        let n = nodes.len();
        let engine_clone = Arc::clone(engine);
        let run = tokio::task::spawn_blocking(move || {
            if n == 1 || engine_clone.has_active_transaction() {
                // Per-record path: exact legacy semantics (single task, or
                // buffered into the active transaction per ERR-013).
                nodes
                    .iter()
                    .map(|node| engine_clone.insert(node))
                    .collect::<Vec<_>>()
            } else {
                let opts = BatchInsertOptions {
                    // Correct for UPSERTs: the pipeline cannot guarantee fresh
                    // IDs (the FIND-61 prototype could and used `true`).
                    skip_existing_check: false,
                    skip_wal: false,
                    // Force Incremental: `Auto` would pick `Rebuild` for batches
                    // above `incremental_threshold` (1000), leaving the HNSW
                    // index without entries until a manual rebuild.
                    insert_mode: InsertMode::Incremental,
                    ..Default::default()
                };
                match engine_clone.batch_insert_with_opts(&nodes, opts) {
                    Ok(()) => (0..nodes.len()).map(|_| Ok(())).collect(),
                    Err(e) => {
                        let msg = format!("group-commit batch insert failed: {e}");
                        (0..nodes.len())
                            .map(|_| Err(crate::error::Error::generic_error(msg.clone())))
                            .collect()
                    }
                }
            }
        })
        .await;

        match run {
            Ok(results) => results,
            Err(join_err) => {
                let msg = format!("ingestion batch worker panicked: {join_err}");
                (0..n)
                    .map(|_| Err(crate::error::Error::generic_error(msg.clone())))
                    .collect()
            }
        }
    }

    /// Convert one queued task into a node (shared by both worker loops;
    /// takes the task by value so the vector is moved, never cloned).
    fn task_to_node(task: IngestionTask) -> UnifiedNode {
        let mut node = UnifiedNode::with_vector(task.id, task.vector);
        if !task.text.is_empty() {
            node.set_field("text", FieldValue::String(task.text));
        }
        for (key, value) in &task.metadata {
            node.set_field(key.as_str(), FieldValue::String(value.clone()));
        }
        node
    }

    fn process(engine: &StorageEngine, task: IngestionTask) -> Result<()> {
        engine.insert(&Self::task_to_node(task))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::tempdir;

    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_delivers_every_submitted_task() {
        let dir = tempdir().unwrap();
        let engine = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
        let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(2));

        for i in 0..16u128 {
            let task = IngestionTask {
                id: i,
                vector: vec![0.25; 4],
                text: format!("node {i}"),
                metadata: HashMap::new(),
            };
            let micros = pipeline.submit(task).await.unwrap();
            assert!(micros > 0, "insert duration must be measured");
        }

        for i in 0..16u128 {
            let node = engine.get(i).unwrap();
            assert!(node.is_some(), "task {i} must be persisted");
        }
    }

    // ─── WIRE-06: group-commit batching ────────────────────────────

    /// Engine with opt-in group-commit batching (default bounds: 32/1ms/1024).
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn batched_engine(dir: &std::path::Path) -> Arc<StorageEngine> {
        let cfg = Config::default().with_insert_batching(InsertBatchConfig {
            enabled: true,
            ..Default::default()
        });
        Arc::new(StorageEngine::open_with_config(dir.to_str().unwrap(), Some(cfg)).unwrap())
    }

    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn task(id: u128, text: &str) -> IngestionTask {
        IngestionTask {
            id,
            vector: vec![0.25; 4],
            text: text.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// `allow`d asserts live here so each test reads as a specification.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    async fn submit_all(pipeline: &AsyncIngestionPipeline, count: u128) -> Vec<Result<u128>> {
        let subs: Vec<_> = (0..count)
            .map(|i| pipeline.submit(task(i, &format!("node {i}"))))
            .collect();
        futures::future::join_all(subs).await
    }

    /// Batched mode: every task is acked, every record is persisted, and the
    /// WAL holds exactly one record per task (single `batch_append` per shard).
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn batched_pipeline_acks_every_task_and_persists_all() {
        let dir = tempdir().unwrap();
        let engine = batched_engine(dir.path());
        let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(1));

        let results = submit_all(&pipeline, 8).await;
        for (i, r) in results.iter().enumerate() {
            assert!(r.is_ok(), "task {i} must be acked after the batch commit");
        }

        for i in 0..8u128 {
            assert!(
                engine.get(i).unwrap().is_some(),
                "task {i} must be visible right after its ack"
            );
        }
        assert_eq!(
            engine.wal.as_ref().unwrap().total_record_count(),
            8,
            "WAL must hold exactly one record per task (no duplicates, no drops)"
        );
    }

    /// Batched mode + reopen: WAL replay and the KV/vstore stores must bring
    /// every record back (durability of the batch commit). Uses an explicit
    /// runtime so dropping it deterministically releases the worker's engine
    /// handle before the reopen (the file lock is process-scoped).
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[test]
    fn batched_pipeline_reopen_recovers_every_record() {
        let dir = tempdir().unwrap();
        let engine = batched_engine(dir.path());
        {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let results = rt.block_on(async {
                let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(1));
                let results = submit_all(&pipeline, 16).await;
                drop(pipeline);
                results
            });
            for (i, r) in results.iter().enumerate() {
                assert!(r.is_ok(), "task {i} must be acked before reopen");
            }
            // Dropping the runtime cancels the worker task, releasing its
            // `Arc<StorageEngine>` clone.
            drop(rt);
        }
        drop(engine);

        let engine2 = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
        for i in 0..16u128 {
            assert!(
                engine2.get(i).unwrap().is_some(),
                "node {i} must survive reopen (WAL replay + stores)"
            );
        }
    }

    /// Batch-atomic error: if the commit fails, EVERY waiter gets an error and
    /// nothing is persisted (deterministic failure via the read-only guard).
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn batched_pipeline_surfaces_batch_failure_to_every_waiter() {
        let dir = tempdir().unwrap();
        let mut engine = StorageEngine::open_with_config(
            dir.path().to_str().unwrap(),
            Some(Config::default().with_insert_batching(InsertBatchConfig {
                enabled: true,
                ..Default::default()
            })),
        )
        .unwrap();
        // Post-open flip: the next write guard rejects with Validation.
        engine.config.read_only = true;
        let engine = Arc::new(engine);

        let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(1));
        let subs: Vec<_> = (0..6u128).map(|i| pipeline.submit(task(i, ""))).collect();
        let results = futures::future::join_all(subs).await;

        for (i, r) in results.iter().enumerate() {
            assert!(r.is_err(), "task {i} must receive the batch failure");
        }
        for i in 0..6u128 {
            assert!(
                engine.get(i).unwrap().is_none(),
                "nothing may be persisted on a failed batch"
            );
        }
    }

    /// Transaction isolation: with a transaction active the batched path falls
    /// back to the per-record `insert()`, which buffers into the transaction —
    /// records stay invisible to the backend until commit (ERR-013).
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn batched_pipeline_defers_to_active_transaction_buffer() {
        use crate::backend::BackendPartition;

        let dir = tempdir().unwrap();
        let engine = batched_engine(dir.path());
        let txn_id = engine.begin_transaction().unwrap();
        let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(1));

        let subs: Vec<_> = (0..4u128).map(|i| pipeline.submit(task(i, ""))).collect();
        for r in futures::future::join_all(subs).await {
            assert!(r.is_ok(), "buffered inserts must ack");
        }

        for i in 0..4u128 {
            let key = i.to_le_bytes();
            assert!(
                engine
                    .backend
                    .get(BackendPartition::Default, &key)
                    .unwrap()
                    .is_none(),
                "node {i} must not reach the backend while the txn is open"
            );
        }

        engine.commit_transaction(txn_id).unwrap();
        for i in 0..4u128 {
            assert!(
                engine.get(i).unwrap().is_some(),
                "node {i} must be applied after commit"
            );
        }
    }

    /// Overwrite (UPSERT) through the batched path: re-inserting an id updates
    /// the record instead of duplicating it.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn batched_pipeline_overwrite_updates_single_record() {
        let dir = tempdir().unwrap();
        let engine = batched_engine(dir.path());
        let pipeline = AsyncIngestionPipeline::new(Arc::clone(&engine), Some(1));

        pipeline.submit(task(7, "first")).await.unwrap();
        pipeline.submit(task(7, "second")).await.unwrap();

        let node = engine.get(7).unwrap().expect("overwritten node must exist");
        assert!(
            matches!(node.relational.get("text"), Some(FieldValue::String(s)) if s == "second"),
            "overwrite must keep the latest payload"
        );
        assert_eq!(
            engine.wal.as_ref().unwrap().total_record_count(),
            2,
            "each write is a WAL record; replay converges to one node"
        );
    }
}
