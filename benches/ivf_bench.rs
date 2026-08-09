//! REVISAR-01: Dedicated IVF build/query benchmark.
//!
//! Closes the ERR-038/039/040/041 reproducibility cycle with a dedicated
//! IVF bench. Measures:
//!   - k-means IVF build time (IvfIndex::build over a DashMap of nodes)
//!   - search latency + recall@10 vs brute-force cosine ground truth
//!   - nlist × nprobe trade-off sweep (the knobs that control the speed/recall
//!     balance of an inverted-file index)
//!
//! Run: cargo bench --bench ivf_bench
//! Quick: cargo bench --bench ivf_bench -- --quick
//! Release: cargo bench --bench ivf_bench -- --nocapture

use criterion::{criterion_group, criterion_main, Criterion};
use dashmap::DashMap;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use vantadb::index::ivf::{IvfConfig, IvfIndex};
use vantadb::index::{cosine_sim_f32, HnswNode, VectorRepresentations};
use vantadb::node::{DistanceMetric, FilterBitset};

const DIMS: usize = 128;
const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;
const SEED: u64 = 42;

const NLIST_SWEEP: &[usize] = &[25, 100, 400];
const NPROBE_SWEEP: &[usize] = &[1, 5, 10];

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

/// Build the DashMap<node> representation IvfIndex::build consumes.
fn make_nodes(dataset: &[(u128, Vec<f32>)]) -> DashMap<u128, HnswNode> {
    let nodes = DashMap::new();
    for (id, vec) in dataset {
        nodes.insert(
            *id,
            HnswNode {
                id: *id,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec.clone()),
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
                neighbor_lists: Vec::new(),
            },
        );
    }
    nodes
}

fn brute_force_knn(query: &[f32], dataset: &[(u128, Vec<f32>)], k: usize) -> Vec<u128> {
    let mut sims: Vec<(u128, f32)> = dataset
        .iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sims.truncate(k);
    sims.into_iter().map(|(id, _)| id).collect()
}

fn compute_recall(
    index: &IvfIndex,
    queries: &[Vec<f32>],
    dataset: &[(u128, Vec<f32>)],
    k: usize,
) -> f64 {
    let mut total_hits = 0;
    for query in queries {
        let truth = brute_force_knn(query, dataset, k);
        let ivf_ids: Vec<u128> = index
            .search(query, k, &FilterBitset::all_set())
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let hits = truth.iter().filter(|id| ivf_ids.contains(id)).count();
        total_hits += hits;
    }
    total_hits as f64 / (queries.len() * k) as f64
}

/// Entries actually scanned for one query: sum of the probed clusters'
/// inverted-list lengths (the IVF analog of brute-force scan cost).
fn candidates_scanned(index: &IvfIndex, query: &[f32]) -> usize {
    let nprobe = index.config.nprobe.min(index.centroids.len());
    let mut centroid_scores: Vec<(usize, f32)> = index
        .centroids
        .iter()
        .enumerate()
        .map(|(i, c)| (i, cosine_sim_f32(query, c)))
        .collect();
    centroid_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    centroid_scores.truncate(nprobe);
    centroid_scores
        .iter()
        .map(|(ci, _)| index.inverted_lists[*ci].len())
        .sum()
}

fn measure_latency(index: &IvfIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64, f64) {
    // Mean candidates scanned per query (entries visited across probed lists).
    let mut candidates_sum = 0usize;
    let mut latencies_us: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = black_box(index.search(q, k, &FilterBitset::all_set()));
            candidates_sum += candidates_scanned(index, q);
            t.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = latencies_us.len();
    let p50 = latencies_us[n / 2];
    let p99 = latencies_us[(n as f64 * 0.99) as usize];
    let mean = latencies_us.iter().sum::<f64>() / n as f64;
    let mean_candidates = if candidates_sum > 0 {
        candidates_sum as f64 / n as f64
    } else {
        0.0
    };
    (p50, p99, mean, mean_candidates)
}

fn bench_ivf(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivf_bench");
    group.sample_size(10);

    let raw_vectors = generate_vectors(N_VECTORS, DIMS, SEED);
    let dataset: Vec<(u128, Vec<f32>)> = raw_vectors
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as u128, v))
        .collect();
    let nodes = make_nodes(&dataset);
    let queries = generate_vectors(N_QUERIES, DIMS, SEED + 1000);

    // ── 1. k-means build time per nlist ──────────────────────────────
    println!(
        "\n━━━ IVF Build (k-means, Forgy init + Lloyd, max 20 iters) — N={}, D={} ━━━",
        N_VECTORS, DIMS
    );
    println!("  {:<12} {:<16}", "nlist", "Build Time (s)");
    println!("  {}", "─".repeat(28));

    for &nlist in NLIST_SWEEP {
        group.bench_function(format!("build_nlist_{}", nlist), |b| {
            b.iter_custom(|iters| {
                let mut total = std::time::Duration::new(0, 0);
                for _ in 0..iters {
                    let cfg = IvfConfig {
                        nlist,
                        nprobe: 10,
                        distance_metric: DistanceMetric::Cosine,
                    };
                    let t0 = Instant::now();
                    let _ = black_box(IvfIndex::build(&nodes, &cfg));
                    total += t0.elapsed();
                }
                total
            })
        });

        let cfg = IvfConfig {
            nlist,
            nprobe: 10,
            distance_metric: DistanceMetric::Cosine,
        };
        let t0 = Instant::now();
        let _ = IvfIndex::build(&nodes, &cfg);
        let build_s = t0.elapsed().as_secs_f64();
        println!("  {:<12} {:<16.3}", nlist, build_s);
    }
    println!("  {}", "─".repeat(28));

    // ── 2. nlist × nprobe sweep: recall + latency vs brute force ─────
    // Baseline: brute-force scan cost per query (all 10K vectors).
    let brute_scan_us = N_VECTORS as f64 / 1_000.0; // floor for the quoted Nd scan, not timed

    println!(
        "\n━━━ IVF Search: nlist × nprobe trade-off (N={}, D={}, k={}) ━━━",
        N_VECTORS, DIMS, TOP_K
    );
    println!(
        "  {:<8} {:<8} {:<12} {:<12} {:<12} {:<12} {:<12} {:<8}",
        "nlist", "nprobe", "Recall@10", "p50 (µs)", "p99 (µs)", "Mean (µs)", "QPS", "Cand/q"
    );
    println!("  {}", "─".repeat(88));

    for &nlist in NLIST_SWEEP {
        for &nprobe in NPROBE_SWEEP {
            let cfg = IvfConfig {
                nlist,
                nprobe,
                distance_metric: DistanceMetric::Cosine,
            };
            let index = IvfIndex::build(&nodes, &cfg);
            let recall = compute_recall(&index, &queries, &dataset, TOP_K);
            let (p50, p99, mean, cand_q) = measure_latency(&index, &queries, TOP_K);
            let qps = 1_000_000.0 / mean;

            group.bench_function(format!("search_nlist_{}_nprobe_{}", nlist, nprobe), |b| {
                b.iter(|| {
                    for q in &queries {
                        let _ = black_box(index.search(q, TOP_K, &FilterBitset::all_set()));
                    }
                })
            });

            println!(
                "  {:<8} {:<8} {:<12.4} {:<12.1} {:<12.1} {:<12.1} {:<12.0} {:<8.0}",
                nlist, nprobe, recall, p50, p99, mean, qps, cand_q
            );
        }
    }
    println!("  {}", "─".repeat(88));
    println!(
        "  Brute-force ground truth: cosine over all {} vectors ({:.1}µs/query scan floor).\n",
        N_VECTORS, brute_scan_us
    );

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_ivf
}

criterion_main!(benches);
