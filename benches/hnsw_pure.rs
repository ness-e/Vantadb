use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};

fn generate_vectors(count: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.gen::<f32>()).collect())
        .collect()
}

fn bench_hnsw_pure(c: &mut Criterion) {
    let dim = 1536;
    let count = 10_000;
    
    let mut group = c.benchmark_group("hnsw_pure");
    group.sample_size(10);
    
    group.bench_function("insert_10k", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            for _ in 0..iters {
                let vectors = generate_vectors(count, dim);
                let config = HnswConfig {
                    m: 16,
                    m_max0: 32,
                    ef_construction: 100,
                    ef_search: 50,
                    ml: 1.0 / (16_f64).ln(),
                    distance_metric: vantadb::node::DistanceMetric::Cosine,
                };
                let mut index = CPIndex::new_with_config(config);
                
                let start = Instant::now();
                for (id, vec) in vectors.into_iter().enumerate() {
                    index.add(
                        id as u64,
                        u128::MAX,
                        VectorRepresentations::Full(vec),
                        0,
                    );
                }
                total_duration += start.elapsed();
            }
            total_duration
        })
    });

    group.bench_function("search_10k", |b| {
        let vectors = generate_vectors(count, dim);
        let config = HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 100,
            ef_search: 50,
            ml: 1.0 / (16_f64).ln(),
            distance_metric: vantadb::node::DistanceMetric::Cosine,
        };
        let mut index = CPIndex::new_with_config(config);
        
        for (id, vec) in vectors.iter().enumerate() {
            index.add(
                id as u64,
                u128::MAX,
                VectorRepresentations::Full(vec.clone()),
                0,
            );
        }
        
        let queries = generate_vectors(100, dim);
        
        b.iter(|| {
            for query in &queries {
                criterion::black_box(index.search_nearest(query, None, None, u128::MAX, 10, None));
            }
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_hnsw_pure);
criterion_main!(benches);

