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
use vantadb::config::{Config, InsertBatchConfig, SyncMode};
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
    run_batch_chunked(engine, producers, workers, INFLIGHT_CHUNK)
        .await
        .0
}

/// Igual que `run_batch` con el tamaño del chunk de submits en vuelo
/// parametrizable (WIRE-06: las celdas ON usan chunk=32 — la celda N=32 de
/// FIND-61 — y chunk=1 como testigo de la penalización de la ventana cuando no
/// hay submits concurrentes). Devuelve `(duración, latencias por task en µs)`;
/// las latencias las mide el worker (submit → ack), como en §13.1.
async fn run_batch_chunked(
    engine: Arc<StorageEngine>,
    producers: usize,
    workers: usize,
    inflight: usize,
) -> (Duration, Vec<u128>) {
    let pipeline = Arc::new(AsyncIngestionPipeline::new(engine, Some(workers)));
    let per = BATCH / producers;

    let mut handles = Vec::new();
    for p in 0..producers {
        let pipeline = Arc::clone(&pipeline);
        handles.push(tokio::spawn(async move {
            let ids: Vec<usize> = (0..per).map(|i| p * per + i).collect();
            let mut latencies: Vec<u128> = Vec::with_capacity(per);
            for chunk in ids.chunks(inflight) {
                let futs: Vec<_> = chunk
                    .iter()
                    .map(|&id| {
                        let pipeline = Arc::clone(&pipeline);
                        async move { pipeline.submit(make_task(id)).await }
                    })
                    .collect();
                let results = futures::future::try_join_all(futs)
                    .await
                    .expect("pipeline submit failed");
                latencies.extend(results);
            }
            latencies
        }));
    }

    let start = Instant::now();
    let mut latencies: Vec<u128> = Vec::with_capacity(BATCH);
    for h in handles {
        latencies.extend(h.await.unwrap());
    }
    let elapsed = start.elapsed();
    drop(pipeline); // cierra el canal: los workers terminan solos
    (elapsed, latencies)
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
    let mut cfg = Config::default().with_sync_mode(mode);
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
    run_batch_chunked(engine, producers, workers, INFLIGHT_CHUNK)
        .await
        .0
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

// ─── WIRE-06: group-commit opt-in (A/B OFF vs ON) ───────────────────
//
// Productización del prototipo FIND-61 (Tabla 2): el pipeline acumula tareas
// y commitea con UN `batch_insert_with_opts` (1× insert_lock + 1× batch_append
// por shard + HNSW bulk). Celdas:
//   off_p1w1        — baseline (§13, sin batching), chunk=32
//   on_p1w1         — batching ON (config default), chunk=32 → gate ≥5×
//   on_w2_p1w1      — ventana 2 ms (sweep de tuning bajo carga)
//   on_w5_p1w1      — ventana 5 ms (sweep de tuning bajo carga)
//   on_serial_p1w1  — batching ON con chunk=1: testigo de la penalización de
//                     la ventana (max_wait_ms) cuando no hay concurrencia.
//
// Reproduce (Regla 11):
//   cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "wire06_group_commit"
const WIRE06_INFLIGHT: usize = 32;

fn bench_wire06_group_commit(c: &mut Criterion) {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap(),
    );

    let mut group = c.benchmark_group("wire06_group_commit");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- WIRE-06: GROUP-COMMIT OPT-IN (BATCH={BATCH}, DIM={DIM}) ---");
    eprintln!(
        "{:<16} | {:<6} | {:<14} | {:<9} | {:<11} | {:<11}",
        "cell", "chunk", "Throughput", "vs OFF", "p50-ack", "p99-ack"
    );
    eprintln!("{}", "-".repeat(78));

    // (label, batching enabled, wait_ms override, submits en vuelo por chunk)
    let cells: [(&str, bool, Option<u64>, usize); 5] = [
        ("off_p1w1", false, None, WIRE06_INFLIGHT),
        ("on_p1w1", true, None, WIRE06_INFLIGHT),
        ("on_w2_p1w1", true, Some(2), WIRE06_INFLIGHT),
        ("on_w5_p1w1", true, Some(5), WIRE06_INFLIGHT),
        ("on_serial_p1w1", true, None, 1),
    ];
    let mut off_ops_s = 0.0_f64;

    for (label, enabled, wait_ms, inflight) in cells {
        let rt = Arc::clone(&rt);
        group.bench_function(label, |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                let mut last = Duration::ZERO;
                let mut last_p50 = 0u128;
                let mut last_p99 = 0u128;
                for _ in 0..iters {
                    let dir = tempdir().unwrap();
                    let mut batch = InsertBatchConfig {
                        enabled,
                        ..Default::default()
                    };
                    if let Some(ms) = wait_ms {
                        batch.max_wait_ms = ms;
                    }
                    let cfg = Config::default().with_insert_batching(batch);
                    let engine = Arc::new(
                        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(cfg))
                            .unwrap(),
                    );
                    let (elapsed, mut lats) =
                        rt.block_on(run_batch_chunked(engine, 1, 1, inflight));
                    lats.sort_unstable();
                    last_p50 = percentile_u128(&lats, 0.50);
                    last_p99 = percentile_u128(&lats, 0.99);
                    total += elapsed;
                    last = elapsed;
                }
                let ops_s = BATCH as f64 / last.as_secs_f64();
                if !enabled {
                    off_ops_s = ops_s;
                }
                let ratio = if off_ops_s > 0.0 {
                    ops_s / off_ops_s
                } else {
                    f64::NAN
                };
                eprintln!(
                    "{:<16} | {:<6} | {:>14.0} | {:>8.2}x | {:>8.2}ms | {:>8.2}ms",
                    label,
                    inflight,
                    ops_s,
                    ratio,
                    last_p50 as f64 / 1000.0,
                    last_p99 as f64 / 1000.0
                );
                std::hint::black_box(ops_s);
                total
            })
        });
    }

    group.finish();
}

/// Nearest-rank percentile over sorted microsecond latencies.
fn percentile_u128(sorted: &[u128], q: f64) -> u128 {
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

/// Abre un engine con batching ON/OFF y corre `BATCH` tareas p1/w1 con chunk
/// `WIRE06_INFLIGHT` — media pareja del A/B interleaved (WIRE-06).
async fn run_batch_cfg(dir: &std::path::Path, enabled: bool) -> (Duration, Vec<u128>) {
    let cfg = Config::default().with_insert_batching(InsertBatchConfig {
        enabled,
        ..Default::default()
    });
    let engine =
        Arc::new(StorageEngine::open_with_config(dir.to_str().unwrap(), Some(cfg)).unwrap());
    run_batch_chunked(engine, 1, 1, WIRE06_INFLIGHT).await
}

/// Ruta directa (sin pipeline) del prototipo FIND-61 Tabla 2: acumula chunks de
/// `batch_n` tasks y llama `batch_insert_with_opts` (WIRE-06 diagnóstico).
fn run_batched_direct(
    engine: &Arc<StorageEngine>,
    batch_n: usize,
    skip_existing: bool,
) -> Duration {
    let tasks: Vec<IngestionTask> = (0..BATCH).map(make_task).collect();
    let start = Instant::now();
    for chunk in tasks.chunks(batch_n) {
        let nodes: Vec<UnifiedNode> = chunk.iter().map(task_to_node).collect();
        let opts = BatchInsertOptions {
            skip_existing_check: skip_existing,
            skip_wal: false,
            insert_mode: InsertMode::Incremental,
            ..Default::default()
        };
        engine
            .batch_insert_with_opts(&nodes, opts)
            .expect("direct batch_insert");
    }
    start.elapsed()
}

/// WIRE-06 diagnóstico de atribución (misma ventana): (1) costo del
/// existence-check requerido para UPSERT (`skip_existing_check=false`) vs el
/// atajo del prototipo (`true`, solo IDs frescos); (2) costo del plumbing del
/// pipeline (cola + ventana + spawn_blocking + acks) vs la llamada directa.
fn bench_wire06_attribution(c: &mut Criterion) {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap(),
    );

    let mut group = c.benchmark_group("wire06_attribution");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- WIRE-06: ATRIBUCIÓN (direct skip=false/true vs pipeline ON) ---");
    eprintln!(
        "{:<8} | {:<12} | {:<12} | {:<12} | {:<10} | {:<10}",
        "sample", "direct false", "direct true", "pipeline ON", "check x", "pipeline x"
    );
    eprintln!("{}", "-".repeat(76));

    let mut sample = 0;
    group.bench_function("attribution", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let dir = tempdir().unwrap();
                let d1 = dir.path().join("d1");
                let d2 = dir.path().join("d2");
                let d3 = dir.path().join("d3");
                std::fs::create_dir_all(&d1).unwrap();
                std::fs::create_dir_all(&d2).unwrap();
                std::fs::create_dir_all(&d3).unwrap();

                let t_false = {
                    let engine = open_engine_with_sync(&d1, SyncMode::Periodic, None);
                    run_batched_direct(&engine, WIRE06_INFLIGHT, false)
                };
                let t_true = {
                    let engine = open_engine_with_sync(&d2, SyncMode::Periodic, None);
                    run_batched_direct(&engine, WIRE06_INFLIGHT, true)
                };
                let t_on = rt.block_on(run_batch_cfg(&d3, true)).0;

                sample += 1;
                eprintln!(
                    "{:<8} | {:>12.0} | {:>12.0} | {:>12.0} | {:>9.2}x | {:>9.2}x",
                    format!("s{sample}"),
                    BATCH as f64 / t_false.as_secs_f64(),
                    BATCH as f64 / t_true.as_secs_f64(),
                    BATCH as f64 / t_on.as_secs_f64(),
                    t_false.as_secs_f64() / t_true.as_secs_f64(),
                    t_on.as_secs_f64() / t_false.as_secs_f64(),
                );
                total += t_false + t_true + t_on;
            }
            total
        })
    });

    group.finish();
}

/// A/B pareado (WIRE-06): OFF y ON se miden en la MISMA ventana de tiempo,
/// alternando el orden por iteración, para neutralizar la deriva de carga de
/// la máquina (en celdas secuenciales el ratio intra-corrida varía ~3.5–9×
/// según qué celda agarra la fase cargada). Imprime el ratio por sample.
fn bench_wire06_paired_ab(c: &mut Criterion) {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap(),
    );

    let mut group = c.benchmark_group("wire06_paired_ab");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- WIRE-06: PAIRED A/B (OFF vs ON, misma ventana) ---");
    eprintln!(
        "{:<14} | {:<12} | {:<12} | {:<8}",
        "sample", "OFF ops/s", "ON ops/s", "ON/OFF"
    );
    eprintln!("{}", "-".repeat(54));

    let mut sample = 0;
    group.bench_function("off_vs_on", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            let mut off_sum = Duration::ZERO;
            let mut on_sum = Duration::ZERO;
            for i in 0..iters {
                let dir = tempdir().unwrap();
                let off_dir = dir.path().join("off");
                let on_dir = dir.path().join("on");
                std::fs::create_dir_all(&off_dir).unwrap();
                std::fs::create_dir_all(&on_dir).unwrap();
                // Alterna el orden para cancelar el sesgo de deriva (review P2-01:
                // con el schedule real iters==1, por eso se combina con el contador
                // de muestra; par: OFF→ON; impar: ON→OFF).
                let (off, on) = rt.block_on(async {
                    if (i + sample) % 2 == 0 {
                        let a = run_batch_cfg(&off_dir, false).await;
                        let b = run_batch_cfg(&on_dir, true).await;
                        (a, b)
                    } else {
                        let b = run_batch_cfg(&on_dir, true).await;
                        let a = run_batch_cfg(&off_dir, false).await;
                        (a, b)
                    }
                });
                off_sum += off.0;
                on_sum += on.0;
                total += off.0 + on.0;
            }
            sample += 1;
            let off_ops = BATCH as f64 * iters as f64 / off_sum.as_secs_f64();
            let on_ops = BATCH as f64 * iters as f64 / on_sum.as_secs_f64();
            eprintln!(
                "{:<14} | {:>12.0} | {:>12.0} | {:>7.2}x",
                format!("s{sample}"),
                off_ops,
                on_ops,
                on_ops / off_ops
            );
            std::hint::black_box(on_ops);
            total
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ingestion_concurrent,
    bench_find61_sync_ab,
    bench_find61_batch,
    bench_wire06_group_commit,
    bench_wire06_paired_ab,
    bench_wire06_attribution
);
criterion_main!(benches);
