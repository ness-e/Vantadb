#![allow(clippy::expect_used, clippy::unwrap_used)]
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;
use vantadb::index::FilterBitset;
use vantadb::node::{FieldValue, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    // Deterministic seed: runs must be comparable across machines/CI.
    let mut rng = StdRng::seed_from_u64(42);
    let mut vec: Vec<f32> = (0..dim).map(|_| rng.random_range(-1.0..1.0)).collect();
    // Normalize
    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        vec.iter_mut().for_each(|v| *v /= norm);
    }
    vec
}

fn high_density_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let is_ci = env::var("CI").unwrap_or_else(|_| "false".to_string()) == "true";
    let target_nodes = if is_ci { 250_000 } else { 1_000_000 };
    let dim = 768; // BGE-M3 or BAAI/bge-base-en dimensionality

    println!("============================================================");
    println!("VantaDB High Density Benchmark");
    println!("Target Nodes: {}", target_nodes);
    println!("Vector Dimensions: {}", dim);
    println!(
        "Mode: {}",
        if is_ci {
            "CI (Resource Governance)"
        } else {
            "Release (1M)"
        }
    );
    println!("============================================================");

    // TempDir (not a fixed repo path): auto-cleaned on drop, safe in
    // parallel, leaves no artifacts if the job dies mid-run.
    let _db_guard = tempfile::tempdir().expect("tempdir");
    let storage = Arc::new(StorageEngine::open(_db_guard.path().to_str().unwrap()).unwrap());

    // Seed the database
    println!(
        "Seeding database with {} nodes (This may take a while)...",
        target_nodes
    );
    rt.block_on(async {
        for i in 1..=target_nodes {
            let mut node = UnifiedNode::new(i as u128);
            node.relational.insert(
                "content".to_string(),
                FieldValue::String(format!("Node {}", i)),
            );
            node.relational.insert(
                "type".to_string(),
                FieldValue::String("benchmark".to_string()),
            );
            node.vector = VectorRepresentations::Full(generate_random_vector(dim));
            let _ = storage.insert(&node);

            if i % 100_000 == 0 {
                println!("Inserted {}/{}", i, target_nodes);
            }
        }
    });

    // Sub-Task 1: Search K-NN Latency
    let mut group = c.benchmark_group("high_density_search");
    group.sample_size(50); // Less samples due to intensity

    group.bench_function("knn_search_768d", |b: &mut criterion::Bencher| {
        b.iter_batched(
            || generate_random_vector(dim),
            |query_vec| {
                rt.block_on(async {
                    let results = storage.hnsw.load().search_nearest(
                        &query_vec,
                        None,
                        None,
                        &FilterBitset::new(),
                        10,
                        None,
                    );
                    // Force materialization to prevent optimization drop
                    assert!(results.len() <= 10);
                });
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    // Sub-Task 2: Spam Mutations Collision (Logarithmic Friction Validation)
    let mut spam_group = c.benchmark_group("logarithmic_spam_friction");
    let spam_samples = if is_ci { 3 } else { 10 }; // ponytail: CI 6h timeout, 3 samples sufficient for throughput magnitude check
    let spam_inserts: u128 = if is_ci { 5_000 } else { 50_000 }; // ponytail: CI inserts reduced to stay under 6h
    spam_group.sample_size(spam_samples);

    // Unique id base per iteration: reused ids would measure overwrite
    // after the first iter, not spam. Monotonic across iters/samples.
    static SPAM_ID_BASE: AtomicU64 = AtomicU64::new(0);
    SPAM_ID_BASE.store((target_nodes + 1) as u64, Ordering::Relaxed);
    spam_group.bench_function("50k_spam_mutations", |b: &mut criterion::Bencher| {
        b.iter_batched(
            || {
                let base = SPAM_ID_BASE.fetch_add(spam_inserts as u64, Ordering::Relaxed);
                let mut dummy_nodes = Vec::with_capacity(spam_inserts as usize);
                for i in 0..spam_inserts {
                    let mut node = UnifiedNode::new(base as u128 + i);
                    node.relational.insert(
                        "content".to_string(),
                        FieldValue::String(format!("Spam node {}", i)),
                    );
                    // Mock spam identity via origin
                    node.relational.insert(
                        "_owner_role".to_string(),
                        FieldValue::String("malicious_agent".to_string()),
                    );
                    node.relational
                        .insert("_confidence".to_string(), FieldValue::Float(1.0)); // Trust manipulation
                    dummy_nodes.push(node);
                }
                dummy_nodes
            },
            |dummy_nodes| {
                rt.block_on(async {
                    // Inject the spam nodes. Logarithmic friction should limit damage without heavy performance degradation on safe nodes
                    for node in dummy_nodes {
                        // Using raw inserts to simulate bulk spam
                        let _ = storage.insert(&node);
                    }
                });
            },
            BatchSize::LargeInput,
        )
    });
    spam_group.finish();
}

criterion_group!(benches, high_density_benchmark);
criterion_main!(benches);
