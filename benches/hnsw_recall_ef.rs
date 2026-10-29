#![allow(clippy::expect_used, clippy::unwrap_used)]
//! TSK-144: HNSW recall vs latency benchmark on REAL data (GloVe-100).
//!
//! Sweeps ef_search values and measures recall@10 (against the dataset's
//! exact ground truth — no recompute), p50/p99 latency, and build
//! time. Results are designed for comparison against hnswlib on the same
//! dataset (produce numbers for papers): the ANN-Benchmarks methodology
//! (separate queries + ground truth, recall-QPS tradeoff).
//!
//! Run: cargo bench --bench hnsw_recall_ef
//! Release mode: cargo bench --bench hnsw_recall_ef -- --nocapture

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Instant;
use vantadb::index::{auto_tune::AutoTune, CPIndex, HnswConfig, IndexType, VectorRepresentations};
use vantadb::node::DistanceMetric;
use vantadb::node::FilterBitset;

mod common;

const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;

const EF_SWEEP: &[usize] = &[10, 20, 50, 100, 200, 400];

fn to_vecs(flat: &[f32], dims: usize, count: usize) -> Vec<Vec<f32>> {
    flat[..count * dims]
        .chunks_exact(dims)
        .map(|c| c.to_vec())
        .collect()
}

fn compute_recall(index: &CPIndex, queries: &[Vec<f32>], ground_truth: &[u64], k: usize) -> f64 {
    let mut total_hits = 0;
    for (q_idx, query) in queries.iter().enumerate() {
        let truth = &ground_truth[q_idx * k..(q_idx + 1) * k];
        let hnsw_ids: Vec<u128> = index
            .search_nearest(
                black_box(query),
                None,
                None,
                &FilterBitset::all_set(),
                k,
                None,
            )
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let hits = truth
            .iter()
            .filter(|id| hnsw_ids.contains(&(**id as u128)))
            .count();
        total_hits += hits;
    }
    total_hits as f64 / (queries.len() * k) as f64
}

fn measure_latency(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64) {
    let mut latencies_us: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = black_box(index.search_nearest(
                black_box(q),
                None,
                None,
                &FilterBitset::all_set(),
                k,
                None,
            ));
            t.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = latencies_us.len();
    let p50 = latencies_us[n / 2];
    let p99 = latencies_us[((n as f64 * 0.99) as usize).min(n - 1)];
    let mean = latencies_us.iter().sum::<f64>() / n as f64;
    (p50, p99, mean)
}

fn bench_hnsw_recall_ef(c: &mut Criterion) {
    let ds = common::load_standard_dataset("glove-100");
    assert_eq!(
        ds.metric, "angular",
        "recall bench requires angular (cosine) data"
    );
    let vectors = to_vecs(&ds.train, ds.dims, N_VECTORS);
    let queries = to_vecs(&ds.test, ds.dims, N_QUERIES);
    let ground_truth = ds.ground_truth.clone();
    println!(
        "hnsw_recall_ef dataset: glove-100 subset ({} train × {}d, {} held-out queries + exact GT)",
        N_VECTORS, ds.dims, N_QUERIES
    );
    let dataset: Vec<(u128, Vec<f32>)> = vectors
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as u128, v))
        .collect();

    let mut group = c.benchmark_group("hnsw_recall_ef");
    common::apply_fixed_profile(&mut group);

    // Build dataset once
    let base_config = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 100,
        ef_search: 100,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None, // None = force real HNSW search
        index_type: IndexType::Hnsw,
        auto_tune: false,
    };

    // ── 1. ef_construction Sweep (Build Time vs Recall@10) ─────────────
    println!(
        "\n━━━ HNSW ef_construction Trade-Off Sweep (N={}, D={}) ━━━",
        N_VECTORS, ds.dims
    );
    println!(
        "  {:<16} {:<16} {:<16}",
        "ef_construction", "Build Time (s)", "Recall@10 (ef_s=100)"
    );
    println!("  {}", "─".repeat(50));

    for &ef_c in &[50usize, 100, 200, 400] {
        let cfg = HnswConfig {
            ef_construction: ef_c,
            ef_search: 100,
            ..base_config.clone()
        };
        let index = CPIndex::new_with_config(cfg);
        let t0 = Instant::now();
        for (id, vec) in &dataset {
            let _ = index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }
        let build_s = t0.elapsed().as_secs_f64();
        let recall = compute_recall(&index, &queries, &ground_truth, TOP_K);
        println!("  {:<16} {:<16.3} {:<16.4}", ef_c, build_s, recall);
    }
    println!("  {}", "─".repeat(50));

    // Build index for ef_search sweep (using default ef_construction = 100)
    let build_time = {
        let index = CPIndex::new_with_config(base_config.clone());
        let t0 = Instant::now();
        for (id, vec) in &dataset {
            let _ = index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }
        t0.elapsed()
    };

    group.bench_function("build_index", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::new(0, 0);
            for _ in 0..iters {
                let idx = CPIndex::new_with_config(base_config.clone());
                let t0 = Instant::now();
                for (id, vec) in &dataset {
                    let _ = idx.add(
                        *id,
                        FilterBitset::all_set(),
                        VectorRepresentations::Full(vec.clone()),
                        0,
                    );
                }
                total += t0.elapsed();
            }
            total
        })
    });

    // For each ef_search, benchmark search and compute recall
    let mut results: Vec<(usize, f64, f64, f64, f64, f64)> = Vec::new();

    for &ef in EF_SWEEP {
        // Bypass the global auto_tuner so search uses the configured static ef
        // only (mirrors benches/param_sweep.rs). Without this, tuned_ef could
        // exceed ef_search, contaminating the ef sweep.
        AutoTune::set_ef(1);
        let index = CPIndex::new_with_config(HnswConfig {
            ef_search: ef,
            ..base_config.clone()
        });
        for (id, vec) in &dataset {
            let _ = index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }

        let recall = compute_recall(&index, &queries, &ground_truth, TOP_K);
        let (p50, p99, mean) = measure_latency(&index, &queries, TOP_K);
        let qps = 1_000_000.0 / mean;
        results.push((ef, recall, p50, p99, mean, qps));

        group.bench_function(format!("search_ef_{}", ef), |b| {
            b.iter(|| {
                for q in &queries {
                    let _ = black_box(index.search_nearest(
                        black_box(q),
                        None,
                        None,
                        &FilterBitset::all_set(),
                        TOP_K,
                        None,
                    ));
                }
            })
        });
    }

    println!(
        "\n━━━ HNSW Recall vs Latency (N={}, D={}, k={}) ━━━",
        N_VECTORS, ds.dims, TOP_K
    );
    println!("  Build time: {:.3}s", build_time.as_secs_f64());
    println!(
        "  {:<12} {:<12} {:<12} {:<12} {:<12} {:<12}",
        "ef_search", "Recall@10", "p50 (µs)", "p99 (µs)", "Mean (µs)", "QPS"
    );
    println!("  {}", "─".repeat(72));
    for (ef, recall, p50, p99, mean, qps) in &results {
        println!(
            "  {:<12} {:<12.4} {:<12.1} {:<12.1} {:<12.1} {:<12.0}",
            ef, recall, p50, p99, mean, qps
        );
    }
    println!("  {}", "─".repeat(72));
    println!("  Recall measured against the dataset's exact ground truth.\n");

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hnsw_recall_ef
}

criterion_main!(benches);
