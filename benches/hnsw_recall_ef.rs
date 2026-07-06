//! TSK-144: HNSW recall vs latency benchmark.
//!
//! Sweeps ef_search values and measures recall@10, p50/p99 latency, and build
//! time. Results are designed for comparison against hnswlib on the same
//! synthetic dataset (produce numbers for papers).
//!
//! Run: cargo bench --bench hnsw_recall_ef
//! Release mode: cargo bench --bench hnsw_recall_ef -- --nocapture

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};
use vantadb::node::DistanceMetric;
use vantadb::node::FilterBitset;

const DIMS: usize = 128;
const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;
const SEED: u64 = 42;

const EF_SWEEP: &[usize] = &[10, 20, 50, 100, 200, 400];

fn generate_vectors(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec: Vec<f32> = (0..dims).map(|_| rng.random_range(-1.0..1.0)).collect();
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u64> {
    let mut sims: Vec<(u64, f32)> = dataset
        .iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sims.truncate(k);
    sims.into_iter().map(|(id, _)| id).collect()
}

fn compute_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    dataset: &[(u64, Vec<f32>)],
    k: usize,
) -> f64 {
    let mut total_hits = 0;
    for query in queries {
        let truth = brute_force_knn(query, dataset, k);
        let hnsw_ids: Vec<u64> = index
            .search_nearest(query, None, None, &FilterBitset::all_set(), k, None)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let hits = truth.iter().filter(|id| hnsw_ids.contains(id)).count();
        total_hits += hits;
    }
    total_hits as f64 / (queries.len() * k) as f64
}

fn measure_latency(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64) {
    let mut latencies_us: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ =
                black_box(index.search_nearest(q, None, None, &FilterBitset::all_set(), k, None));
            t.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = latencies_us.len();
    let p50 = latencies_us[n / 2];
    let p99 = latencies_us[(n as f64 * 0.99) as usize];
    let mean = latencies_us.iter().sum::<f64>() / n as f64;
    (p50, p99, mean)
}

fn bench_hnsw_recall_ef(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw_recall_ef");
    group.sample_size(10);

    // Build dataset once
    let base_config = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 200,
        ef_search: 200,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
    };

    let raw_vectors = generate_vectors(N_VECTORS, DIMS, SEED);
    let dataset: Vec<(u64, Vec<f32>)> = raw_vectors
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as u64, v))
        .collect();
    let queries = generate_vectors(N_QUERIES, DIMS, SEED + 1000);

    // Build index (measured)
    let build_time = {
        let index = CPIndex::new_with_config(base_config.clone());
        let t0 = Instant::now();
        for (id, vec) in &dataset {
            index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }
        let elapsed = t0.elapsed();
        println!(
            "\n  Build time (N={}, D={}): {:.3}s",
            N_VECTORS,
            DIMS,
            elapsed.as_secs_f64()
        );
        elapsed
    };

    group.bench_function("build_index", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::new(0, 0);
            for _ in 0..iters {
                let idx = CPIndex::new_with_config(base_config.clone());
                let t0 = Instant::now();
                for (id, vec) in &dataset {
                    idx.add(
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
        let index = CPIndex::new_with_config(HnswConfig {
            ef_search: ef,
            ..base_config.clone()
        });
        for (id, vec) in &dataset {
            index.add(
                *id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }

        let recall = compute_recall(&index, &queries, &dataset, TOP_K);
        let (p50, p99, mean) = measure_latency(&index, &queries, TOP_K);
        let qps = 1_000_000.0 / mean;
        results.push((ef, recall, p50, p99, mean, qps));

        group.bench_function(format!("search_ef_{}", ef), |b| {
            b.iter(|| {
                for q in &queries {
                    let _ = black_box(index.search_nearest(
                        q,
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
        N_VECTORS, DIMS, TOP_K
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
    println!("  Recall measured against brute-force cosine similarity ground truth.\n");

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hnsw_recall_ef
}

criterion_main!(benches);
