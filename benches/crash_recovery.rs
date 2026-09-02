// ponytail: bench setup unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Crash-recovery benchmark — closes critical gap (TBH-09 / TASK-09).
//!
//! Measures **startup latency** (open + WAL replay time) for a pre-populated
//! database across a 3-point corpus sweep.
//!
//! Why this exists: TBH-08 measured WAL **write throughput**; the other end
//! of the durability story — **how long it takes to recover after a crash**
//! — was unmeasured. Without this bench, optimizations to `recover_state`
//! or `StorageEngine::open` are blind: a startup regression can ship
//! unnoticed (Regla 9 — "No optimizar sin medir").
//!
//! Design (Ponytail: bench mínimo, mide lo que dice medir):
//! - 3 corpus sizes [100, 10_000, 100_000] (plan-mandated).
//! - Per `iter_custom` iteration: fresh tempdir, pre-populate N records with
//!   `SyncMode::Always` (so the WAL is genuinely durable on disk after setup),
//!   drop the engine, then re-open with default config and time the open call.
//! - Pre-populate happens OUTSIDE the timed region (the warmup-like setup
//!   is what `iter_custom`'s iters loop runs repeatedly — we measure only
//!   the recovery call).
//! - Metric: wall-time of `StorageEngine::open_with_config()` (the engine
//!   already reports this via `metrics::record_startup`).
//! - Throughput axis: `Throughput::Elements(N)` → opens/sec at corpus N.
//!
//! Run:  `cargo bench -p vantadb --bench crash_recovery`

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::time::Instant;
use tempfile::tempdir;
use vantadb::config::{SyncMode, VantaConfig};
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

mod common;

/// Corpus sizes to sweep — exactly what the plan specifies.
/// Plan line 103: "sweep corpus size [100, 10k, 100k]".
const CORPUS_SIZES: &[u64] = &[100, 10_000, 100_000];

/// Pre-populate a fresh engine at `db_path` with `count` insert records
/// using `SyncMode::Always` so every record reaches disk before we close
/// the engine (simulates a real crash leaving durable WAL behind).
fn pre_populate(db_path: &str, count: u64) {
    let config = VantaConfig {
        sync_mode: SyncMode::Always,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(db_path, Some(config))
        .expect("pre_populate: open_with_config");
    for i in 1..=count {
        let node = UnifiedNode::new(i as u128);
        engine.insert(&node).expect("pre_populate: insert");
    }
    // Drop closes WAL and flushes — record durability is now a property
    // of the on-disk `vanta.wal` file, exactly like a real crash would leave it.
    drop(engine);
}

/// Run one corpus-size sample end-to-end.
/// `iter_custom` ensures the corpus is regenerated every iteration so the
/// open timings stay comparable across the 10-sample sweep.
fn bench_one(c: &mut Criterion, corpus_size: u64) {
    let mut group = c.benchmark_group(format!("crash_recovery/corpus_{corpus_size}"));
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);
    group.throughput(Throughput::Elements(corpus_size));

    group.bench_function("open_with_replay", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            for _ in 0..iters {
                // 1. Fresh tempdir per iter — comparable recovery timings
                let dir = tempdir().expect("tempdir for crash_recovery bench");
                let db_path = dir.path().to_str().expect("tempdir path utf8");

                // 2. Pre-populate (NOT timed — measures only the recovery path)
                pre_populate(db_path, corpus_size);

                // 3. Timed: open the database (open + WAL replay)
                let started = Instant::now();
                let engine = StorageEngine::open_with_config(db_path, None)
                    .expect("crash_recovery: open_with_config");
                total += started.elapsed();
                black_box(engine);

                // 4. dir dropped at end of scope — WAL is gone, next iter starts fresh
            }
            total
        });
    });

    group.finish();
}

fn bench_crash_recovery(c: &mut Criterion) {
    for &n in CORPUS_SIZES {
        bench_one(c, n);
    }
}

criterion_group!(benches, bench_crash_recovery);
criterion_main!(benches);
