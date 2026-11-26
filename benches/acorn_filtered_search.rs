// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! ACORN Filtered Vector Search Benchmark.
//!
//! Measures VantaDB's ACORN filtered graph navigation performance
//! across selectivity levels (1%, 5%, 10%, 50%, 100%): recall@10 (computed
//! once, untimed) + Criterion search latency per category (the tracked
//! signal in estimates.json for bench_regression.py).
//!
//! Run: cargo bench --bench acorn_filtered_search

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use vantadb::index::{auto_tune::AutoTune, cosine_sim_f32, CPIndex, HnswConfig, IndexType};
use vantadb::node::{DistanceMetric, FilterBitset, VectorRepresentations};

mod common;

const DIMS: usize = 128;
const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 200;
const TOP_K: usize = 10;
const CORPUS_SEED: u64 = 42;
const QUERY_SEED: u64 = 1042;

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

fn bench_acorn(c: &mut Criterion) {
    AutoTune::set_ef(100);

    let raw_vectors = generate_vectors(N_VECTORS, DIMS, CORPUS_SEED);
    let queries = generate_vectors(N_QUERIES, DIMS, QUERY_SEED);

    // Bitsets encode selectivity levels (nested: bit0 ⊂ bit1 ⊂ bit2 ⊂ bit3).
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
    let index = CPIndex::new_with_config(config);
    for (id, vec, bitset) in &dataset {
        let _ = index.add(
            *id,
            bitset.clone(),
            VectorRepresentations::Full(vec.clone()),
            0,
        );
    }

    let categories = [
        ("selectivity_1pct", Some(0)),
        ("selectivity_5pct", Some(1)),
        ("selectivity_10pct", Some(2)),
        ("selectivity_50pct", Some(3)),
        ("selectivity_100pct", None),
    ];

    let mut group = c.benchmark_group("acorn_filtered_search");
    common::apply_fixed_profile(&mut group);
    let mut cells: Vec<String> = Vec::new();
    for (name, bit_idx) in categories {
        let mask = match bit_idx {
            Some(bit) => {
                let mut m = FilterBitset::new();
                m.set_bit(bit);
                m
            }
            None => FilterBitset::all_set(),
        };
        // Recall: computed once outside the timed region (ground truth scan
        // is O(N) per query and must not pollute the latency measurement).
        let mut total_hits = 0usize;
        let mut total_possible = 0usize;
        for query in &queries {
            let truth = brute_force_filtered_knn(query, &dataset, &mask, TOP_K);
            let results: Vec<u128> = index
                .search_nearest(query, None, None, &mask, TOP_K, None)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            total_possible += truth.len();
            total_hits += truth.iter().filter(|id| results.contains(id)).count();
        }
        let recall = if total_possible > 0 {
            total_hits as f64 / total_possible as f64
        } else {
            1.0
        };
        println!("acorn {name}: recall@10 = {recall:.4}");
        cells.push(format!(
            "{{\"selectivity\":\"{name}\",\"recall\":{recall:.4}}}"
        ));

        group.bench_function(name, |b| {
            b.iter(|| {
                for query in &queries {
                    let _ = black_box(index.search_nearest(
                        black_box(query),
                        None,
                        None,
                        &mask,
                        TOP_K,
                        None,
                    ));
                }
            });
        });
    }
    group.finish();
    // Machine-readable curve for the dashboard (metricas/benchmarks/).
    common::write_metric_json(
        "recall_acorn.json",
        &format!(
            "{{\"n\":{N_VECTORS},\"dims\":{DIMS},\"queries\":{N_QUERIES},\"k\":{TOP_K},\"cells\":[{}]}}",
            cells.join(",")
        ),
    );
}

criterion_group!(benches, bench_acorn);
criterion_main!(benches);
