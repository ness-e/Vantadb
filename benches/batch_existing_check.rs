// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Benchmark: `batch_insert` per-node existence probe cost (ERR-037)
//!
//! Measures the default upsert-safe path of `batch_insert`. Before ERR-037 each
//! node ran the full `get()` read-path (active-txn lock, cache **write** lock,
//! KV metadata read, HNSW lookup, vstore mmap vector read + clone that was
//! discarded). After ERR-037, `existing_for_batch()` reads only `relational` +
//! `edges` from a shared read-only cache peek or the KV metadata blob.
//!
//! Scenarios per batch size:
//!   - `fresh_{n}`:     default `batch_insert` on n new ids (probe miss)
//!   - `overwrite_{n}`: re-`batch_insert` over n cached ids (probe hit +
//!     decrement/increment cardinality bookkeeping)
//!   - `skip_fresh_{n}`: `skip_existing_check` reference ceiling (no probe)

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::{Duration, Instant};
use vantadb::config::VantaConfig;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::storage::{BackendKind, BatchInsertOptions, InsertMode, StorageEngine};

const DIM: usize = 768;
const SIZES: &[usize] = &[100, 1_000, 10_000];
const SAMPLE_SIZE: usize = 10;

fn make_engine() -> StorageEngine {
    StorageEngine::open_with_config(
        ":memory:",
        Some(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        }),
    )
    .expect("in-memory engine")
}

fn make_nodes(count: usize, tag: &str) -> Vec<UnifiedNode> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..count)
        .map(|i| {
            let mut node = UnifiedNode::new(i as u128);
            node.vector = vantadb::node::VectorRepresentations::Full(
                (0..DIM).map(|_| rng.random::<f32>()).collect(),
            );
            node.relational
                .insert("tag".to_string(), FieldValue::String(tag.to_string()));
            // Hot tier → pre-insert populates the volatile cache, so the
            // overwrite scenario exercises the cache-hit probe path (the one
            // ERR-037 optimized). Default is Cold → cache stays empty and the
            // scenario would measure the backend-miss path instead.
            node.tier = vantadb::node::NodeTier::Hot;
            node
        })
        .collect()
}

fn bench_existing_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_existing_check");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(Duration::from_secs(20));

    for &n in SIZES {
        // Default path: per-node existence probe (ERR-037 target).
        group.bench_function(format!("fresh_{n}"), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(n, "v1");
                    let start = Instant::now();
                    engine
                        .batch_insert(black_box(&nodes))
                        .expect("batch_insert fresh");
                    total += start.elapsed();
                }
                total
            })
        });

        // Re-insert over cached nodes: probe hit + decrement/increment bookkeeping.
        group.bench_function(format!("overwrite_{n}"), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(n, "v1");
                    engine.batch_insert(black_box(&nodes)).expect("pre-insert");
                    let updated = make_nodes(n, "v2");
                    let start = Instant::now();
                    engine
                        .batch_insert(black_box(&updated))
                        .expect("batch_insert overwrite");
                    total += start.elapsed();
                }
                total
            })
        });

        // Diagnostic control: overwrite write-path with the probe skipped
        // (isolates probe cost from pre-insert cache state + write phases).
        group.bench_function(format!("overwrite_skip_{n}"), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(n, "v1");
                    engine.batch_insert(black_box(&nodes)).expect("pre-insert");
                    let updated = make_nodes(n, "v2");
                    let opts = BatchInsertOptions {
                        skip_existing_check: true,
                        ..Default::default()
                    };
                    let start = Instant::now();
                    engine
                        .batch_insert_with_opts(black_box(&updated), black_box(opts))
                        .expect("overwrite skip probe");
                    total += start.elapsed();
                }
                total
            })
        });

        // Reference ceiling: no existence probe at all (fresh ids, no WAL).
        group.bench_function(format!("skip_fresh_{n}"), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(n, "v1");
                    let opts = BatchInsertOptions {
                        skip_existing_check: true,
                        skip_wal: true,
                        insert_mode: InsertMode::Rebuild,
                        ..Default::default()
                    };
                    let start = Instant::now();
                    engine
                        .batch_insert_with_opts(black_box(&nodes), black_box(opts))
                        .expect("batch_insert skip check");
                    total += start.elapsed();
                }
                total
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_existing_check);
criterion_main!(benches);
