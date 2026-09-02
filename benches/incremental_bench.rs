// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Benchmark: Incremental vs Rebuild Insert Performance
//!
//! Measures the performance difference between the old rebuild behavior and the
//! new incremental insert behavior for various batch sizes (10 — 2000).
//!
//! Three scenarios per batch size:
//!   - **Rebuild**:   `InsertMode::Rebuild` + explicit `rebuild_vector_index()`
//!   - **Auto**:      `InsertMode::Auto` (default threshold 1000)
//!   - **Incremental**: `InsertMode::Incremental` (explicit per-node HNSW insert)

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::{Duration, Instant};
use vantadb::config::VantaConfig;
use vantadb::node::{FilterBitset, UnifiedNode};
use vantadb::storage::{BackendKind, BatchInsertOptions, InsertMode, StorageEngine};

// ─── Constants ───────────────────────────────────────────────────────

const DIM: usize = 768;
const BATCH_SIZES: &[usize] = &[10, 50, 100, 500, 1000, 2000];
const SAMPLE_SIZE: usize = 10;

// ─── Helpers ─────────────────────────────────────────────────────────

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

fn generate_vectors(count: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.random::<f32>()).collect())
        .collect()
}

fn make_nodes(count: usize, dim: usize) -> Vec<UnifiedNode> {
    let vectors = generate_vectors(count, dim);
    vectors
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            let mut node = UnifiedNode::new(i as u128);
            node.vector = vantadb::node::VectorRepresentations::Full(v);
            node
        })
        .collect()
}

/// Compute recall@10: for each node, query with its vector and check if its
/// own node_id appears in the top-10 results.
fn recall_at_10(engine: &StorageEngine, n: usize, dim: usize) -> f64 {
    let mut hits = 0u64;
    let mask = FilterBitset::new(); // empty mask = match everything
    for i in 0..n {
        let vec = generate_vectors(n, dim)[i].clone();
        let idx = engine.hnsw.load();
        let results = idx.search_nearest(
            black_box(&vec),
            None,
            None,
            black_box(&mask),
            black_box(10),
            None,
        );
        if results.iter().any(|(id, _)| *id == i as u128) {
            hits += 1;
        }
    }
    hits as f64 / n.max(1) as f64
}

// ─── Benchmark function ─────────────────────────────────────────────

fn bench_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_bench");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(Duration::from_secs(30));

    for &batch_size in BATCH_SIZES {
        // ── Rebuild path (old behaviour) ──────────────────────────
        group.bench_function(format!("rebuild_{}", batch_size), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(batch_size, DIM);
                    let opts = BatchInsertOptions {
                        skip_existing_check: true,
                        skip_wal: true,
                        insert_mode: InsertMode::Rebuild,
                        ..Default::default()
                    };
                    let start = Instant::now();
                    engine
                        .batch_insert_with_opts(black_box(&nodes), black_box(opts))
                        .expect("batch_insert_with_opts (Rebuild)");
                    engine.rebuild_vector_index().expect("rebuild_vector_index");
                    total += start.elapsed();
                }
                total
            })
        });

        // ── Auto path (new default behaviour) ─────────────────────
        group.bench_function(format!("auto_{}", batch_size), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(batch_size, DIM);
                    let opts = BatchInsertOptions {
                        skip_existing_check: true,
                        skip_wal: true,
                        insert_mode: InsertMode::Auto,
                        ..Default::default()
                    };
                    let start = Instant::now();
                    engine
                        .batch_insert_with_opts(black_box(&nodes), black_box(opts))
                        .expect("batch_insert_with_opts (Auto)");
                    total += start.elapsed();
                }
                total
            })
        });

        // ── Incremental path (explicit) ───────────────────────────
        group.bench_function(format!("incremental_{}", batch_size), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let engine = make_engine();
                    let nodes = make_nodes(batch_size, DIM);
                    let opts = BatchInsertOptions {
                        skip_existing_check: true,
                        skip_wal: true,
                        insert_mode: InsertMode::Incremental,
                        ..Default::default()
                    };
                    let start = Instant::now();
                    engine
                        .batch_insert_with_opts(black_box(&nodes), black_box(opts))
                        .expect("batch_insert_with_opts (Incremental)");
                    total += start.elapsed();
                }
                total
            })
        });
    }

    group.finish();

    // ─── Recall quality (run once, not per iteration) ─────────────
    println!();
    println!("═════════════════════════════════════════════════════════");
    println!("  Recall@10 Quality Check (dim={})", DIM);
    println!("═════════════════════════════════════════════════════════");
    for &batch_size in &[50, 500, 2000] {
        let nodes = make_nodes(batch_size, DIM);

        // Incremental recall
        let engine_inc = make_engine();
        engine_inc
            .batch_insert_with_opts(
                &nodes,
                BatchInsertOptions {
                    skip_existing_check: true,
                    skip_wal: true,
                    insert_mode: InsertMode::Incremental,
                    ..Default::default()
                },
            )
            .expect("incremental insert");
        let recall_inc = recall_at_10(&engine_inc, batch_size.min(100), DIM);

        // Rebuild recall
        let engine_rebuild = make_engine();
        engine_rebuild
            .batch_insert_with_opts(
                &nodes,
                BatchInsertOptions {
                    skip_existing_check: true,
                    skip_wal: true,
                    insert_mode: InsertMode::Rebuild,
                    ..Default::default()
                },
            )
            .expect("rebuild insert");
        engine_rebuild.rebuild_vector_index().expect("rebuild");
        let recall_rebuild = recall_at_10(&engine_rebuild, batch_size.min(100), DIM);

        println!(
            "  batch_size={:5}  Incremental recall@10={:.3}  Rebuild recall@10={:.3}  parity={:.1}%",
            batch_size,
            recall_inc,
            recall_rebuild,
            if recall_rebuild > 0.0 {
                (recall_inc / recall_rebuild) * 100.0
            } else {
                0.0
            }
        );
    }
    println!("═════════════════════════════════════════════════════════");
}

// ─── Criterion harness ──────────────────────────────────────────────

criterion_group!(benches, bench_incremental);
criterion_main!(benches);
