// RES-03 — A/B bench del pipeline de ingesta asíncrono (`AsyncIngestionPipeline`).
// Mide throughput (ops/s) de la ruta submit → canal compartido → workers →
// `spawn_blocking(insert)` → ack, con N producers × {1,2,4} consumers.
// Objetivo: decidir con datos si el patrón `Arc<Mutex<Receiver>>` (tokio mpsc
// es single-consumer por diseño) introduce contención real vs. el coste de
// inserción. Regla 9: medir antes de rediseñar.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vantadb::config::{SyncMode, VantaConfig};
use vantadb::ingestion::{AsyncIngestionPipeline, IngestionTask};
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::storage::{BatchInsertOptions, InsertMode, StorageEngine};

const DIM: usize = 16;
// BATCH=400: con la ingesta dominada por el camino de escritura del motor
// (~100 ops/s por worker), 2000 tareas tardaban ~20 s/batch; 400 mantiene el
// signal y hace viable la matriz 2×3 ×2 corridas.
const BATCH: usize = 400;
/// Submits concurrentes por producer: mantiene saturados hasta 4 workers
/// sin medir el coste de spawn por tarea.
const INFLIGHT_CHUNK: usize = 16;
const PRODUCER_COUNTS: [usize; 2] = [1, 4];
const WORKER_COUNTS: [usize; 3] = [1, 2, 4];

fn make_task(id: usize) -> IngestionTask {
    IngestionTask {
        id: id as u128,
        vector: (0..DIM)
            .map(|d| ((id * 7 + d) % 23) as f32 / 23.0)
            .collect(),
        text: String::new(),
        metadata: HashMap::new(),
    }
}

/// Corre `BATCH` tareas end-to-end (incluye acks) y devuelve la duración.
async fn run_batch(engine: Arc<StorageEngine>, producers: usize, workers: usize) -> Duration {
    let pipeline = Arc::new(AsyncIngestionPipeline::new(engine, Some(workers)));
    let per = BATCH / producers;

    let mut handles = Vec::new();
    for p in 0..producers {
        let pipeline = Arc::clone(&pipeline);
        handles.push(tokio::spawn(async move {
            let ids: Vec<usize> = (0..per).map(|i| p * per + i).collect();
            for chunk in ids.chunks(INFLIGHT_CHUNK) {
                let futs: Vec<_> = chunk
                    .iter()
                    .map(|&id| {
                        let pipeline = Arc::clone(&pipeline);
                        async move { pipeline.submit(make_task(id)).await.map(|_| ()) }
                    })
                    .collect();
                futures::future::try_join_all(futs)
                    .await
                    .expect("pipeline submit failed");
            }
        }));
    }

    let start = Instant::now();
    for h in handles {
        h.await.unwrap();
    }
    let elapsed = start.elapsed();
    drop(pipeline); // cierra el canal: los workers terminan solos
    elapsed
}

fn bench_ingestion_concurrent(c: &mut Criterion) {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap(),
    );

    let mut group = c.benchmark_group("ingestion_concurrent");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- RES-03: ASYNC INGESTION PIPELINE (BATCH={BATCH}, DIM={DIM}) ---");
    eprintln!(
        "{:<10} | {:<8} | {:<14}",
        "Producers", "Workers", "Throughput (ops/s)"
    );
    eprintln!("{}", "-".repeat(38));

    for &p in &PRODUCER_COUNTS {
        for &w in &WORKER_COUNTS {
            let rt = Arc::clone(&rt);
            group.bench_function(BenchmarkId::new(format!("p{p}"), w), |b| {
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    let mut last = Duration::ZERO;
                    for _ in 0..iters {
                        // DB fresca por lote: aísla el coste de ingestión del
                        // crecimiento del índice entre celdas.
                        let dir = tempdir().unwrap();
                        let engine =
                            Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
                        let elapsed = rt.block_on(async move { run_batch(engine, p, w).await });
                        total += elapsed;
                        last = elapsed;
                    }
                    let ops_s = BATCH as f64 / last.as_secs_f64();
                    eprintln!("{:<10} | {:<8} | {:>14.0}", p, w, ops_s);
                    std::hint::black_box(ops_s);
                    total
                })
            });
        }
    }

    group.finish();
}

// FIND-61 spike (bench-only, 0 prod code, timebox ≤1d) — desglose
// insert_lock vs fsync + prototype micro-batching. NO toca src/ ni defaults:
// todo vive en este harness. Ver BENCHMARKS §13.1 + ADR-037 + task FIND-61.
//
// Nota Never*: `SyncMode::Never` NO tiene rama propia en `WalWriter::maybe_sync`
// (src/wal.rs:376-389 — solo `Always` vs `else threshold=1`); con
// `flush_threshold=None` fsyncea igual que Periodic-default. `Never*` =
// `Never + flush_threshold=Some(1_000_000)` bench-only: WAL bytes sin fsync
// (solo lock+HNSW+memcpy). Sin este threshold el A/B no aisla nada.
const FIND61_NEVER_THRESHOLD: usize = 1_000_000;
const FIND61_BATCH_NS: [usize; 3] = [8, 16, 32];

fn open_engine_with_sync(
    dir: &std::path::Path,
    mode: SyncMode,
    threshold: Option<usize>,
) -> Arc<StorageEngine> {
    let mut cfg = VantaConfig::default().with_sync_mode(mode);
    if let Some(t) = threshold {
        cfg = cfg.with_flush_threshold(t);
    }
    Arc::new(StorageEngine::open_with_config(dir.to_str().unwrap(), Some(cfg)).unwrap())
}

/// Igual que `run_batch` pero abriendo el engine con el SyncMode pedido.
async fn run_batch_with_sync(
    dir: &tempfile::TempDir,
    producers: usize,
    workers: usize,
    mode: SyncMode,
    threshold: Option<usize>,
) -> Duration {
    let engine = open_engine_with_sync(dir.path(), mode, threshold);
    run_batch_on_engine(engine, producers, workers).await
}

async fn run_batch_on_engine(
    engine: Arc<StorageEngine>,
    producers: usize,
    workers: usize,
) -> Duration {
    let pipeline = Arc::new(AsyncIngestionPipeline::new(engine, Some(workers)));
    let per = BATCH / producers;

    let mut handles = Vec::new();
    for p in 0..producers {
        let pipeline = Arc::clone(&pipeline);
        handles.push(tokio::spawn(async move {
            let ids: Vec<usize> = (0..per).map(|i| p * per + i).collect();
            for chunk in ids.chunks(INFLIGHT_CHUNK) {
                let futs: Vec<_> = chunk
                    .iter()
                    .map(|&id| {
                        let pipeline = Arc::clone(&pipeline);
                        async move { pipeline.submit(make_task(id)).await.map(|_| ()) }
                    })
                    .collect();
                futures::future::try_join_all(futs)
                    .await
                    .expect("pipeline submit failed");
            }
        }));
    }

    let start = Instant::now();
    for h in handles {
        h.await.unwrap();
    }
    let elapsed = start.elapsed();
    drop(pipeline);
    elapsed
}

fn task_to_node(task: &IngestionTask) -> UnifiedNode {
    let mut node = UnifiedNode::with_vector(task.id, task.vector.clone());
    if !task.text.is_empty() {
        node.set_field("text", FieldValue::String(task.text.clone()));
    }
    for (key, value) in &task.metadata {
        node.set_field(key.as_str(), FieldValue::String(value.clone()));
    }
    node
}

/// Prototype bench-only: acumula N tasks → 1 `batch_insert_with_opts`
/// (skip_existing_check=true IDs frescos, skip_wal=false, Incremental) bajo UN
/// guard ERR-010. Devuelve (total, latencias por task): cada task del batch
/// ackea junta tras el batch → su ack-latency = latencia del batch.
/// Ventana de pérdida ante crash = N writes (batch en memoria no-acked).
fn run_batched(engine: &Arc<StorageEngine>, batch_n: usize) -> (Duration, Vec<Duration>) {
    let tasks: Vec<IngestionTask> = (0..BATCH).map(make_task).collect();
    let mut latencies: Vec<Duration> = Vec::with_capacity(BATCH);
    let total_start = Instant::now();
    for chunk in tasks.chunks(batch_n) {
        let nodes: Vec<UnifiedNode> = chunk.iter().map(task_to_node).collect();
        let opts = BatchInsertOptions {
            skip_existing_check: true,
            skip_wal: false,
            insert_mode: InsertMode::Incremental,
            ..Default::default()
        };
        let start = Instant::now();
        engine
            .batch_insert_with_opts(&nodes, opts)
            .expect("FIND-61 batch_insert");
        let elapsed = start.elapsed();
        for _ in 0..chunk.len() {
            latencies.push(elapsed);
        }
    }
    let total = total_start.elapsed();
    (total, latencies)
}

fn percentile_dur(sorted: &[Duration], q: f64) -> Duration {
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

fn bench_find61_sync_ab(c: &mut Criterion) {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap(),
    );

    let mut group = c.benchmark_group("find61_sync");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- FIND-61: SYNC A/B (BATCH={BATCH}, DIM={DIM}) ---");
    eprintln!("{:<14} | {:<14}", "mode", "Throughput (ops/s)");
    eprintln!("{}", "-".repeat(32));

    // (label, SyncMode, threshold, producers, workers)
    let cells: [(&str, SyncMode, Option<usize>, usize, usize); 3] = [
        ("always_p1w1", SyncMode::Always, None, 1, 1),
        (
            "never_star_p1w1",
            SyncMode::Never,
            Some(FIND61_NEVER_THRESHOLD),
            1,
            1,
        ),
        (
            "never_star_p1w4",
            SyncMode::Never,
            Some(FIND61_NEVER_THRESHOLD),
            1,
            4,
        ),
    ];

    for (label, mode, threshold, p, w) in cells {
        let rt = Arc::clone(&rt);
        group.bench_function(label, |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                let mut last = Duration::ZERO;
                for _ in 0..iters {
                    let dir = tempdir().unwrap();
                    let elapsed = rt
                        .block_on(async { run_batch_with_sync(&dir, p, w, mode, threshold).await });
                    total += elapsed;
                    last = elapsed;
                }
                let ops_s = BATCH as f64 / last.as_secs_f64();
                eprintln!("{:<14} | {:>14.0}", label, ops_s);
                std::hint::black_box(ops_s);
                total
            })
        });
    }

    group.finish();
}

fn bench_find61_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("find61_batch");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- FIND-61: MICRO-BATCH PROTOTYPE (BATCH={BATCH}, DIM={DIM}) ---");
    eprintln!("{:<6} | {:<14} | {:<14}", "N", "Throughput", "p50-ack");
    eprintln!("{}", "-".repeat(40));

    for &n in &FIND61_BATCH_NS {
        group.bench_function(BenchmarkId::new("n", n), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                let mut last_p50 = Duration::ZERO;
                let mut last_ops = 0.0;
                for _ in 0..iters {
                    let dir = tempdir().unwrap();
                    let engine = open_engine_with_sync(dir.path(), SyncMode::Periodic, None);
                    let (elapsed, mut lats) = run_batched(&engine, n);
                    lats.sort_unstable();
                    let p50 = percentile_dur(&lats, 0.50);
                    total += elapsed;
                    last_p50 = p50;
                    last_ops = BATCH as f64 / elapsed.as_secs_f64();
                }
                eprintln!("{:<6} | {:>14.0} | {:>14?}", n, last_ops, last_p50);
                std::hint::black_box(last_ops);
                total
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ingestion_concurrent,
    bench_find61_sync_ab,
    bench_find61_batch
);
criterion_main!(benches);
