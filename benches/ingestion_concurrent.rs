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
use vantadb::ingestion::{AsyncIngestionPipeline, IngestionTask};
use vantadb::storage::StorageEngine;

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

criterion_group!(benches, bench_ingestion_concurrent);
criterion_main!(benches);
