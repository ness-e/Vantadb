#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Canonical P99 benchmark (FND-10 / Regla 9).
//! consumo guard — baseline anti-regresión GOV-B3: `cargo bench --bench canonical_p99 --no-run` must compile.
//!
//! Contract: **insert 100k GloVe-100d vectors + search 1000 held-out queries**,
//! reporting P99 + recall@10 (from the dataset's exact ground truth).
//! This is the canonical no-regression baseline for every performance change
//! (Regla 9 — "No optimizar sin medir"): run `cargo bench -p vantadb --bench
//! canonical_p99` before and after a perf change and compare the numbers.
//!
//! Design (mirrors `hnsw_pure.rs`):
//! - Pure in-memory `CPIndex` (no storage I/O) with a fixed HNSW config.
//! - Real data: `data/benchmark/glove-100/` (GloVe, normalized, angular) —
//!   corpus, queries and exact ground truth are SEPARATED (ANN-Benchmarks
//!   methodology), unlike the old seed-42 synthetic corpus whose queries were
//!   members of the index (trivially optimistic recall/latency).
//! - Insert measured via `iter_custom` (full 100k build per iteration,
//!   corpus loaded once outside the timed region).
//! - Search latency histogram computed per query and printed as p50/p95/p99,
//!   plus recall@10 against the file ground truth (untimed).

use criterion::{criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};
use vantadb::index::{CPIndex, FilterBitset, HnswConfig, VectorRepresentations};

mod common;

const TOP_K: usize = 10;

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

fn to_vecs(flat: &[f32], dims: usize) -> Vec<Vec<f32>> {
    flat.chunks_exact(dims).map(|c| c.to_vec()).collect()
}

fn bench_canonical_p99(c: &mut Criterion) {
    let ds = common::load_standard_dataset("glove-100");
    assert_eq!(
        ds.metric, "angular",
        "canonical baseline requires angular (cosine) data"
    );
    let vectors = to_vecs(&ds.train, ds.dims);
    let queries = to_vecs(&ds.test, ds.dims);
    assert!(
        TOP_K <= ds.k,
        "bench TOP_K must not exceed the dataset ground-truth k"
    );
    println!(
        "canonical_p99 dataset: glove-100 ({} train × {}d, {} held-out queries, k={})",
        ds.n_train, ds.dims, ds.n_queries, ds.k
    );

    // ---- Insert 100k × 100d ----
    let mut group = c.benchmark_group("canonical_p99");
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);

    group.bench_function("insert_100k_glove100d", |b| {
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

    // ---- Search + recall@10 against the file ground truth (untimed) ----
    let index = build_index(&vectors);

    let mut latencies: Vec<Duration> = Vec::with_capacity(queries.len());
    let mut hits = 0usize;
    for (qi, query) in queries.iter().enumerate() {
        let start = Instant::now();
        let result: Vec<u128> = std::hint::black_box(index.search_nearest(
            std::hint::black_box(query),
            None,
            None,
            &FilterBitset::all_set(),
            TOP_K,
            None,
        ))
        .into_iter()
        .map(|(id, _)| id)
        .collect();
        latencies.push(start.elapsed());
        let gt_len = ds.ground_truth.len();
        let truth = &ds.ground_truth[(qi * ds.k).min(gt_len)..((qi + 1) * ds.k).min(gt_len)];
        hits += truth
            .iter()
            .filter(|id| result.contains(&(**id as u128)))
            .count();
    }
    let recall = hits as f64 / (queries.len() * TOP_K) as f64;
    latencies.sort_unstable();
    let p50 = percentile(&latencies, 0.50);
    let p95 = percentile(&latencies, 0.95);
    let p99 = percentile(&latencies, 0.99);
    println!(
        "canonical_p99 search ({} queries, {}d, top_k={}): p50={:?} p95={:?} p99={:?} recall@10={:.4}",
        queries.len(),
        ds.dims,
        TOP_K,
        p50,
        p95,
        p99,
        recall
    );
    // Machine-readable summary for the dashboard (metricas/benchmarks/).
    common::write_metric_json(
        "canonical_summary.json",
        &format!(
            "{{\"dataset\":\"glove-100\",\"n\":{},\"dims\":{},\"queries\":{},\"p50_ms\":{},\"p95_ms\":{},\"p99_ms\":{},\"recall\":{recall:.4}}}",
            ds.n_train,
            ds.dims,
            queries.len(),
            p50.as_secs_f64() * 1000.0,
            p95.as_secs_f64() * 1000.0,
            p99.as_secs_f64() * 1000.0
        ),
    );

    group.bench_function("search_1000_queries_glove100d", |b| {
        b.iter(|| {
            for query in &queries {
                std::hint::black_box(index.search_nearest(
                    std::hint::black_box(query),
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
