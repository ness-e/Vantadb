// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! WAL throughput benchmark — closes critical gap (TBH-08 / TASK-08).
//!
//! Measures raw `WalWriter` throughput (ops/sec) + per-record latency p50/p95/p99
//! across a 2-axis sweep: **SyncMode** × **batch size**.
//!
//! Why this exists: `benches/incremental_bench.rs` uses `skip_wal: true` in 5
//! spots, so no existing bench actually exercises the WAL hot path. Without
//! this measurement, a WAL change could silently degrade ingest throughput
//! (Regla 9 — "No optimizar sin medir").
//!
//! Design (Ponytail: bench mínimo, mide lo que dice medir):
//! - 3 SyncMode × 4 batch sizes = 12 samples
//! - Constant 10_000 records per `iter` (so Throughput::Elements is comparable)
//! - `iter_custom` recreates the tempdir each iteration so fsync is measured
//!   cold (otherwise the 2nd iter would open an existing WAL with bytes already
//!   on disk and the numbers would not be comparable).
//! - Payload: `WalRecord::Insert(UnifiedNode::new(id))` (~24 B serialized)
//!   — represents the smallest real WAL write; larger records would inflate
//!   serialization cost but not the I/O surface this bench measures.
//! - Latency histogram printed per sample (p50/p95/p99 in microseconds).
//!
//! Run:  `cargo bench -p vantadb --bench wal_throughput`

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion, Throughput};
use std::hint::black_box;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vantadb::config::SyncMode;
use vantadb::node::UnifiedNode;
use vantadb::wal::{WalRecord, WalWriter};

mod common;

/// Total records written per `iter` (constant across sweep — makes
/// `Throughput::Elements` directly comparable across samples).
const RECORDS_PER_ITER: u64 = 10_000;

/// Batch sizes to sweep — covers single-record worst case up to amortized
/// large batches. Plan-mandated: [1, 100, 1_000, 10_000].
const BATCH_SIZES: &[usize] = &[1, 100, 1_000, 10_000];

/// Nearest-rank percentile over sorted latencies (mirrors `canonical_p99.rs`).
fn percentile(sorted: &[Duration], q: f64) -> Duration {
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

/// Run one full sweep iteration: open a fresh WAL, write `RECORDS_PER_ITER`
/// records using the requested `(sync_mode, batch_size)`, return per-record
/// wall-time latencies (one sample per `append` call).
///
/// `iter_custom` ensures the tempdir (and therefore the WAL file) is recreated
/// every iteration — so fsync measurements stay cold and comparable.
fn run_iter(sync_mode: SyncMode, batch_size: usize) -> Vec<Duration> {
    let dir = tempdir().expect("tempdir for wal bench");
    let path = dir.path().join("wal_throughput.wal");
    let mut writer = WalWriter::open(&path, sync_mode).expect("open WalWriter");

    let mut latencies: Vec<Duration> = Vec::with_capacity(RECORDS_PER_ITER as usize);
    let mut id_counter: u128 = 0;

    // Buffered path: one payload reused across the batch (batch_append) —
    // but per-record latency samples are still meaningful since each call
    // hits the BufWriter and (depending on threshold) the sync path.
    // For batch_size == 1 we use the single-record `append` API.
    let batches = RECORDS_PER_ITER / batch_size as u64;
    debug_assert!(batches * batch_size as u64 == RECORDS_PER_ITER);

    for _ in 0..batches {
        let start = Instant::now();
        if batch_size == 1 {
            let record = WalRecord::Insert(UnifiedNode::new(id_counter));
            id_counter += 1;
            writer.append(&record).expect("wal append");
        } else {
            let records: Vec<WalRecord> = (0..batch_size)
                .map(|_| {
                    let r = WalRecord::Insert(UnifiedNode::new(id_counter));
                    id_counter += 1;
                    r
                })
                .collect();
            writer.batch_append(&records).expect("wal batch_append");
        }
        // For the batched path the latency is the whole batch — divide to get
        // per-record cost (criterion will report it on the throughput axis
        // anyway, but the histogram below is more useful per-record).
        let elapsed = start.elapsed();
        let per_record = elapsed / batch_size as u32;
        latencies.push(per_record);
    }

    black_box(writer.bytes_written());
    latencies
}

/// Emit a `criterion::Throughput::Elements` benchmark for one
/// (sync_mode, batch_size) pair.
fn bench_one(c: &mut Criterion, sync_mode: SyncMode, batch_size: usize) {
    let label = format!(
        "{}/batch_{}",
        match sync_mode {
            SyncMode::Always => "always",
            SyncMode::Periodic => "periodic",
            SyncMode::Never => "never",
        },
        batch_size
    );

    let mut group: BenchmarkGroup<'_, WallTime> =
        c.benchmark_group(format!("wal_throughput/{label}"));
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);
    group.throughput(Throughput::Elements(RECORDS_PER_ITER));

    group.bench_function("records", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            let mut last_latencies: Vec<Duration> = Vec::new();
            for _ in 0..iters {
                let lats = run_iter(sync_mode, batch_size);
                total += lats.iter().sum::<Duration>();
                last_latencies = lats;
            }
            // Print p50/p95/p99 of the last iteration's per-record latencies
            // (one-shot, stdout-only — criterion owns its own estimate).
            if !last_latencies.is_empty() {
                let mut sorted = last_latencies.clone();
                sorted.sort_unstable();
                let p50 = percentile(&sorted, 0.50);
                let p95 = percentile(&sorted, 0.95);
                let p99 = percentile(&sorted, 0.99);
                println!(
                    "wal_throughput/{label} per-record latencies: p50={:?} p95={:?} p99={:?}",
                    p50, p95, p99
                );
            }
            total
        });
    });

    group.finish();
}

fn bench_wal_throughput(c: &mut Criterion) {
    for &sync_mode in &[SyncMode::Always, SyncMode::Periodic, SyncMode::Never] {
        for &batch_size in BATCH_SIZES {
            bench_one(c, sync_mode, batch_size);
        }
    }
}

criterion_group!(benches, bench_wal_throughput);
criterion_main!(benches);
