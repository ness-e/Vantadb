//! ACORN Filtered Vector Search Benchmark.
//!
//! Measures VantaDB's ACORN filtered graph navigation performance
//! across selectivity levels (1%, 5%, 10%, 50%, 100%).
//!
//! Run: cargo bench --bench acorn_filtered_search

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use vantadb::index::{auto_tune::AutoTune, cosine_sim_f32, CPIndex, HnswConfig, IndexType};
use vantadb::node::{DistanceMetric, FilterBitset, VectorRepresentations};

const DIMS: usize = 128;
const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;
const SEED: u64 = 42;

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

fn brute_force_filtered_knn(
    query: &[f32],
    dataset: &[(u128, Vec<f32>, FilterBitset)],
    mask: &FilterBitset,
    k: usize,
) -> Vec<u128> {
    let mut sims: Vec<(u128, f32)> = dataset
        .iter()
        .filter(|(_, _, bitset)| bitset.matches_mask(mask))
        .map(|(id, vec, _)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sims.truncate(k);
    sims.into_iter().map(|(id, _)| id).collect()
}

fn main() {
    AutoTune::set_ef(100);

    println!("============================================================");
    println!("   VANTA ACORN FILTERED VECTOR SEARCH BENCHMARK            ");
    println!("============================================================");
    println!("Vectors: {}, Dims: {}, Top-K: {}", N_VECTORS, DIMS, TOP_K);

    let raw_vectors = generate_vectors(N_VECTORS, DIMS, SEED);
    let queries = generate_vectors(N_QUERIES, DIMS, SEED + 1000);

    // Assign bitsets to nodes for selectivity levels:
    // Bit 0: 1% selectivity (id % 100 == 0)
    // Bit 1: 5% selectivity (id % 20 == 0)
    // Bit 2: 10% selectivity (id % 10 == 0)
    // Bit 3: 50% selectivity (id % 2 == 0)
    // All set: 100% selectivity
    let mut dataset = Vec::with_capacity(N_VECTORS);
    for (i, vec) in raw_vectors.into_iter().enumerate() {
        let id = i as u128;
        let mut bitset = FilterBitset::new();
        if i % 100 == 0 {
            bitset.set_bit(0);
        }
        if i % 20 == 0 {
            bitset.set_bit(1);
        }
        if i % 10 == 0 {
            bitset.set_bit(2);
        }
        if i % 2 == 0 {
            bitset.set_bit(3);
        }
        dataset.push((id, vec, bitset));
    }

    // Build HNSW index
    let config = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 100,
        ef_search: 100,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: IndexType::Hnsw,
        auto_tune: false,
    };

    println!("Building HNSW index...");
    let t_build = Instant::now();
    let index = CPIndex::new_with_config(config);
    for (id, vec, bitset) in &dataset {
        index.add(
            *id,
            bitset.clone(),
            VectorRepresentations::Full(vec.clone()),
            0,
        );
    }
    println!("Index built in {:.3}s\n", t_build.elapsed().as_secs_f64());

    struct TestCategory {
        name: &'static str,
        selectivity_pct: &'static str,
        bit_idx: Option<usize>,
    }

    let categories = [
        TestCategory {
            name: "1% Selectivity",
            selectivity_pct: "1.0%",
            bit_idx: Some(0),
        },
        TestCategory {
            name: "5% Selectivity",
            selectivity_pct: "5.0%",
            bit_idx: Some(1),
        },
        TestCategory {
            name: "10% Selectivity",
            selectivity_pct: "10.0%",
            bit_idx: Some(2),
        },
        TestCategory {
            name: "50% Selectivity",
            selectivity_pct: "50.0%",
            bit_idx: Some(3),
        },
        TestCategory {
            name: "100% Unfiltered",
            selectivity_pct: "100.0%",
            bit_idx: None,
        },
    ];

    println!(
        "{:<18} | {:<12} | {:<16} | {:<12} | {:<12} | {:<12}",
        "Category", "Selectivity", "Throughput (QPS)", "Recall@10", "p50 Latency", "p99 Latency"
    );
    println!("{}", "-".repeat(95));

    for cat in &categories {
        let mask = match cat.bit_idx {
            Some(bit) => {
                let mut m = FilterBitset::new();
                m.set_bit(bit);
                m
            }
            None => FilterBitset::all_set(),
        };

        // Compute ground truth and recall
        let mut total_hits = 0;
        let mut total_possible = 0;
        for query in &queries {
            let truth = brute_force_filtered_knn(query, &dataset, &mask, TOP_K);
            let results: Vec<u128> = index
                .search_nearest(query, None, None, &mask, TOP_K, None)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            for t in &truth {
                if results.contains(t) {
                    total_hits += 1;
                }
            }
            total_possible += truth.len();
        }
        let recall = if total_possible > 0 {
            total_hits as f64 / total_possible as f64
        } else {
            1.0
        };

        // Measure search latencies
        let mut latencies_us: Vec<f64> = Vec::with_capacity(queries.len());
        let t_start = Instant::now();
        for query in &queries {
            let t0 = Instant::now();
            let _ = black_box(index.search_nearest(query, None, None, &mask, TOP_K, None));
            latencies_us.push(t0.elapsed().as_nanos() as f64 / 1_000.0);
        }
        let total_time_s = t_start.elapsed().as_secs_f64();
        let qps = queries.len() as f64 / total_time_s;

        latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = latencies_us[latencies_us.len() / 2];
        let p99 = latencies_us[(latencies_us.len() as f64 * 0.99) as usize];

        println!(
            "{:<18} | {:<12} | {:<16.1} | {:<12.4} | {:<10.1} µs | {:<10.1} µs",
            cat.name, cat.selectivity_pct, qps, recall, p50, p99
        );
    }

    println!("============================================================");
}
