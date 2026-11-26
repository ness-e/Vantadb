#![allow(clippy::expect_used, clippy::unwrap_used)]
//! HNSW puro en memoria sobre datos reales (GloVe-100): `insert_10k` +
//! `search_100` (lote de 100 queries held-out). La versión rápida del
//! baseline canónico para iteración local (Smoke Windows).
//!
//! Run: cargo bench --bench hnsw_pure

use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Instant;
use vantadb::index::{CPIndex, FilterBitset, HnswConfig, VectorRepresentations};

mod common;

const N_VECTORS: usize = 10_000;
const N_QUERIES: usize = 100;
const TOP_K: usize = 10;

fn to_vecs(flat: &[f32], dims: usize) -> Vec<Vec<f32>> {
    flat.chunks_exact(dims).map(|c| c.to_vec()).collect()
}

fn bench_hnsw_pure(c: &mut Criterion) {
    let ds = common::load_standard_dataset("glove-100");
    let vectors = to_vecs(&ds.train[..N_VECTORS * ds.dims], ds.dims);
    let queries = to_vecs(&ds.test[..N_QUERIES * ds.dims], ds.dims);
    println!(
        "hnsw_pure dataset: glove-100 subset ({} train × {}d, {} held-out queries)",
        N_VECTORS, ds.dims, N_QUERIES
    );

    let mut group = c.benchmark_group("hnsw_pure");
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);

    group.bench_function("insert_10k", |b| {
        // Corpus cargado UNA vez fuera de la región timed (medir RNG+alloc
        // junto al build contamina la medición); por iter solo clones+adds.
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            for _ in 0..iters {
                let config = HnswConfig {
                    m: 16,
                    m_max0: 32,
                    ef_construction: 100,
                    ef_search: 50,
                    ml: 1.0 / (16_f64).ln(),
                    distance_metric: vantadb::node::DistanceMetric::Cosine,
                    flat_threshold: None,
                    index_type: vantadb::index::IndexType::Hnsw,
                    auto_tune: false,
                };
                let index = CPIndex::new_with_config(config);

                let start = Instant::now();
                for (id, vec) in vectors.iter().enumerate() {
                    let _ = index.add(
                        id as u128,
                        FilterBitset::all_set(),
                        VectorRepresentations::Full(vec.clone()),
                        0,
                    );
                }
                total_duration += start.elapsed();
            }
            total_duration
        })
    });

    group.bench_function("search_10k", |b| {
        let config = HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 100,
            ef_search: 50,
            ml: 1.0 / (16_f64).ln(),
            distance_metric: vantadb::node::DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: vantadb::index::IndexType::Hnsw,
            auto_tune: false,
        };
        let index = CPIndex::new_with_config(config);

        for (id, vec) in vectors.iter().enumerate() {
            let _ = index.add(
                id as u128,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }

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
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hnsw_pure);
criterion_main!(benches);
