//! TSK-143: Fjall vs RocksDB backend benchmark.
//!
//! Compares insert throughput, get latency, and memory usage between Fjall
//! and RocksDB through the full StorageEngine path.
//!
//! Run: cargo bench --bench backend_compare
//! Requires features: default (includes "fjall" and "rocksdb")

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use tempfile::TempDir;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;
use vantadb::BackendKind;

const NUM_RECORDS: usize = 5_000;
const QUERY_SAMPLE: usize = 500;

fn setup_engine(backend: BackendKind) -> (StorageEngine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let config = VantaConfig {
        backend_kind: backend,
        storage_path: dir.path().to_string_lossy().to_string(),
        wal_shards: 0,
        mmap_hnsw: false,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
        .expect("open storage engine");
    (engine, dir)
}

fn insert_batch(engine: &StorageEngine, count: usize) -> std::time::Duration {
    let start = Instant::now();
    for i in 0..count {
        let node = UnifiedNode::new(i as u128);
        engine.insert(&node).expect("insert");
    }
    start.elapsed()
}

fn bench_backend_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_insert");
    group.sample_size(10);

    for backend in &[BackendKind::Fjall, BackendKind::RocksDb] {
        let label = format!("{:?}", backend);
        group.bench_function(format!("{}_single_insert", label), |b| {
            let (engine, _dir) = setup_engine(*backend);
            // Pre-insert some records so the engine is "warm"
            for i in 0..100u64 {
                engine.insert(&UnifiedNode::new(i.into())).unwrap();
            }
            let mut counter = 100u64;
            b.iter(|| {
                let node = UnifiedNode::new(counter.into());
                engine.insert(&node).unwrap();
                counter += 1;
                black_box(counter);
            });
        });
    }
    group.finish();
}

fn bench_backend_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_get");
    group.sample_size(10);

    for backend in &[BackendKind::Fjall, BackendKind::RocksDb] {
        let label = format!("{:?}", backend);
        group.bench_function(format!("{}_random_get", label), |b| {
            let (engine, _dir) = setup_engine(*backend);
            for i in 0..NUM_RECORDS as u64 {
                engine.insert(&UnifiedNode::new(i.into())).unwrap();
            }
            let mut rng = StdRng::seed_from_u64(42);
            let query_ids: Vec<u64> = (0..QUERY_SAMPLE)
                .map(|_| rng.random_range(0..NUM_RECORDS as u64))
                .collect();
            let mut idx = 0;
            b.iter(|| {
                let id = query_ids[idx % query_ids.len()];
                idx += 1;
                let _ = black_box(engine.get(id.into()).expect("get"));
            });
        });
    }
    group.finish();
}

fn bench_bulk_insert_throughput() {
    println!("\n━━━ Bulk Insert Throughput ({} records) ━━━", NUM_RECORDS);
    for backend in &[BackendKind::Fjall, BackendKind::RocksDb] {
        let (engine, _dir) = setup_engine(*backend);
        let elapsed = insert_batch(&engine, NUM_RECORDS);
        let throughput = NUM_RECORDS as f64 / elapsed.as_secs_f64();
        println!(
            "  {:>8?} | {:>6} records in {:>8.3}s | {:.0} records/sec",
            backend,
            NUM_RECORDS,
            elapsed.as_secs_f64(),
            throughput
        );
        let stats = engine.get_memory_stats();
        let rss_mb = stats
            .physical_rss
            .map(|b| b as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);
        println!(
            "  {:>8} | logical: {:>8.1} MB | physical_rss: {:>8.1} MB | nodes: {}",
            "",
            stats.logical_bytes as f64 / (1024.0 * 1024.0),
            rss_mb,
            stats.node_count
        );
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

fn bench_get_latency_distribution() {
    println!(
        "━━━ Get Latency Distribution ({} random lookups) ━━━",
        QUERY_SAMPLE
    );
    for backend in &[BackendKind::Fjall, BackendKind::RocksDb] {
        let (engine, _dir) = setup_engine(*backend);
        for i in 0..NUM_RECORDS as u64 {
            engine.insert(&UnifiedNode::new(i.into())).unwrap();
        }
        let mut rng = StdRng::seed_from_u64(42);
        let mut latencies: Vec<f64> = (0..QUERY_SAMPLE)
            .map(|_| {
                let id = rng.random_range(0..NUM_RECORDS as u64);
                let t = Instant::now();
                let _ = engine.get(id.into()).expect("get");
                t.elapsed().as_nanos() as f64 / 1000.0
            })
            .collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = latencies[QUERY_SAMPLE / 2];
        let p95 = latencies[(QUERY_SAMPLE as f64 * 0.95) as usize];
        let p99 = latencies[(QUERY_SAMPLE as f64 * 0.99) as usize];
        let mean = latencies.iter().sum::<f64>() / QUERY_SAMPLE as f64;
        println!(
            "  {:>8?} | p50: {:>8.2} µs | p95: {:>8.2} µs | p99: {:>8.2} µs | mean: {:>8.2} µs",
            backend, p50, p95, p99, mean
        );
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

fn bench_backend_compare(c: &mut Criterion) {
    bench_bulk_insert_throughput();
    bench_get_latency_distribution();
    bench_backend_insert(c);
    bench_backend_get(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs(2));
    targets = bench_backend_compare
}

criterion_main!(benches);
