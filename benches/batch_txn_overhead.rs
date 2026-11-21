// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Benchmark: implicit WAL-transaction overhead of `batch_insert` (FIND-640)
//!
//! Measures the multi-chunk batch path on a **disk-backed** engine with WAL
//! enabled — the in-memory engine has no WAL and cannot exercise the
//! Begin/Prepare/Commit frame added by FIND-640.
//!
//! Gate: post-change vs pre-change must not regress >10%.

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Duration;
use vantadb::config::{Config, SyncMode};
use vantadb::node::{FieldValue, UnifiedNode, VectorRepresentations};
use vantadb::storage::{BackendKind, StorageEngine};

const DIM: usize = 384;
const SIZES: &[usize] = &[1_000, 3_000];
const SAMPLE_SIZE: usize = 10;

fn make_engine(dir: &std::path::Path) -> StorageEngine {
    StorageEngine::open_with_config(
        dir.to_str().expect("utf8 path"),
        Some(Config {
            backend_kind: BackendKind::Fjall,
            // SyncMode::Never keeps fsync policy constant across runs so the
            // measured delta is the txn-frame CPU/serialization cost, not IO.
            sync_mode: SyncMode::Never,
            ..Default::default()
        }),
    )
    .expect("disk engine")
}

fn make_nodes(count: usize) -> Vec<UnifiedNode> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..count)
        .map(|i| {
            let mut node = UnifiedNode::new(i as u128);
            node.vector = VectorRepresentations::Full(
                (0..DIM).map(|_| rng.random::<f32>()).collect(),
            );
            node.relational
                .insert("tag".to_string(), FieldValue::String("v1".to_string()));
            node
        })
        .collect()
}

fn bench_batch_txn_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_txn_overhead");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(Duration::from_secs(10));

    for &n in SIZES {
        group.bench_function(format!("fresh_wal_{n}"), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let dir = tempfile::tempdir().expect("tempdir");
                    let engine = make_engine(dir.path());
                    let nodes = make_nodes(n);
                    let start = std::time::Instant::now();
                    engine
                        .batch_insert(black_box(&nodes))
                        .expect("batch_insert fresh");
                    total += start.elapsed();
                }
                total
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_batch_txn_frame);
criterion_main!(benches);
