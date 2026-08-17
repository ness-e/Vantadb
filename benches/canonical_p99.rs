//! Canonical P99 benchmark (FND-10 / Regla 9).
//!
//! Contract: **insert 100k vectors × 1536 dims + search**, reporting P99.
//! This is the canonical no-regression baseline for every performance change
//! (Regla 9 — "No optimizar sin medir"): run `cargo bench -p vantadb --bench
//! canonical_p99` before and after a perf change and compare the numbers.
//!
//! Design (mirrors `hnsw_pure.rs`):
//! - Pure in-memory `CPIndex` (no storage I/O) with a fixed HNSW config.
//! - Deterministic dataset: `StdRng::seed_from_u64(42)` — byte-identical
//!   vectors across runs and machines.
//! - Insert measured via `iter_custom` (full 100k build per iteration).
//! - Search latency histogram computed per query and printed as p50/p95/p99.

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use vantadb::index::{CPIndex, FilterBitset, HnswConfig, VectorRepresentations};

mod common;

const DIM: usize = 1536;
const N_VECTORS: usize = 100_000;
const N_QUERIES: usize = 1_000;
const TOP_K: usize = 10;

fn generate_vectors(count: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.random::<f32>()).collect())
        .collect()
}

fn hnsw_config() -> HnswConfig {
    HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 100,
        ef_search: 50,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: vantadb::node::DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: vantadb::index::IndexType::Hnsw,
        auto_tune: false,
    }
}

fn build_index(vectors: &[Vec<f32>]) -> CPIndex {
    let index = CPIndex::new_with_config(hnsw_config());
    for (id, vec) in vectors.iter().enumerate() {
        let _ = index.add(
            id as u128,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec.clone()),
            0,
        );
    }
    index
}

/// Nearest-rank percentile over sorted latencies.
fn percentile(sorted: &[Duration], q: f64) -> Duration {
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

fn bench_canonical_p99(c: &mut Criterion) {
    let vectors = generate_vectors(N_VECTORS, DIM);
    let queries = generate_vectors(N_QUERIES, DIM);

    // ---- Insert 100k × 1536d ----
    let mut group = c.benchmark_group("canonical_p99");
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);

    group.bench_function("insert_100k_1536d", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            for _ in 0..iters {
                let start = Instant::now();
                let _index = build_index(&vectors);
                total_duration += start.elapsed();
            }
            total_duration
        })
    });

    // ---- Search 1000 queries, P99 histogram ----
    let index = build_index(&vectors);

    let mut latencies: Vec<Duration> = Vec::with_capacity(N_QUERIES);
    for query in &queries {
        let start = Instant::now();
        std::hint::black_box(index.search_nearest(
            query,
            None,
            None,
            &FilterBitset::all_set(),
            TOP_K,
            None,
        ));
        latencies.push(start.elapsed());
    }
    latencies.sort_unstable();
    let p50 = percentile(&latencies, 0.50);
    let p95 = percentile(&latencies, 0.95);
    let p99 = percentile(&latencies, 0.99);
    println!(
        "canonical_p99 search ({} queries, {}d, top_k={}): p50={:?} p95={:?} p99={:?}",
        N_QUERIES, DIM, TOP_K, p50, p95, p99
    );

    group.bench_function("search_1000_queries_1536d", |b| {
        b.iter(|| {
            for query in &queries {
                std::hint::black_box(index.search_nearest(
                    query,
                    None,
                    None,
                    &FilterBitset::all_set(),
                    TOP_K,
                    None,
                ));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_canonical_p99);
criterion_main!(benches);
