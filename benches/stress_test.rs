use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::env;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

fn run_stress_test(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Abrir motor con BlockCache (2GB) y Bloom Filter (10 bit/key)
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let rt = Runtime::new().unwrap();

    let is_ultra = env::var("STRESS_LEVEL").unwrap_or_default() == "ULTRA";
    let num_nodes = if is_ultra { 1_000_000 } else { 100_000 };

    println!(
        "💉 Inyectando {} nodos... (Stress Level: {})",
        num_nodes,
        if is_ultra { "ULTRA" } else { "NORMAL" }
    );

    // Inyección Preparatoria
    for i in 1..=num_nodes {
        let node = UnifiedNode::new(i);
        storage.insert(&node).unwrap();
    }
    println!("✅ Inyección finalizada.");

    let mut group = c.benchmark_group("The Memory Abyss");
    group.sample_size(10);

    group.bench_function("Point Lookup Valido", |b: &mut criterion::Bencher| {
        b.to_async(&rt).iter(|| async {
            // Nodo que seguro existe, forzando fetch real
            let _ = black_box(storage.get(500).unwrap());
        });
    });

    group.bench_function(
        "Point Lookup Spurious (Bloom Filter Reject)",
        |b: &mut criterion::Bencher| {
            b.to_async(&rt).iter(|| async {
                // Nodo que seguro NO existe. El Bloom Filter rechaza el I/O disk fetch al instante.
                let _ = black_box(storage.get(num_nodes + 9999).unwrap());
            });
        },
    );

    group.finish();
}

criterion_group!(benches, run_stress_test);
criterion_main!(benches);
