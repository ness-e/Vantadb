use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::Rng;
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vantadb::node::{FieldValue, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let mut vec: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
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
            "CI (Survival)"
        } else {
            "Release (1M)"
        }
    );
    println!("============================================================");

    let storage = Arc::new(StorageEngine::open("high_density_bench_db").unwrap());

    // Seed the database
    println!(
        "Seeding database with {} nodes (This may take a while)...",
        target_nodes
    );
    rt.block_on(async {
        for i in 1..=target_nodes {
            let mut node = UnifiedNode::new(i as u64);
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

    group.bench_function("knn_search_768d", |b| {
        b.iter_batched(
            || generate_random_vector(dim),
            |query_vec| {
                rt.block_on(async {
                    let results = storage
                        .hnsw
                        .read()
                        .unwrap()
                        .search_nearest(&query_vec, None, None, 0, 10);
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
    spam_group.sample_size(10); // Very intensive, 10 samples

    spam_group.bench_function("50k_spam_mutations", |b| {
        b.iter_batched(
            || {
                let mut dummy_nodes = Vec::with_capacity(50_000);
                for i in 0..50_000 {
                    let mut node = UnifiedNode::new((target_nodes + 1 + i) as u64);
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
                    // Inject the 50k nodes. Logarithmic friction should limit damage without heavy performance degradation on safe nodes
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

    // Clean up
    let _ = std::fs::remove_dir_all("high_density_bench_db");
}

criterion_group!(benches, high_density_benchmark);
criterion_main!(benches);
