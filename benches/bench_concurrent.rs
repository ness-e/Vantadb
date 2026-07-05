use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vantadb::index::FilterBitset;
use vantadb::node::{UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

fn generate_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.random::<f32>()).collect())
        .collect()
}

fn main() {
    let dim = 128;
    let initial_count = 10_000;
    let test_duration = Duration::from_secs(3);

    println!("============================================================");
    println!("   VANTA HNSW CONCURRENCY BENCHMARK (BASELINE VS FINE-GRAINED)  ");
    println!("============================================================");
    println!("Dimension: {}, Initial Nodes: {}", dim, initial_count);
    println!("Running each test scenario for {:?}...", test_duration);

    // 1. Setup StorageEngine and populate with 10k vectors
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    println!("Initializing database at {}...", db_path);

    let storage = Arc::new(StorageEngine::open(db_path).unwrap());

    println!("Generating {} vectors...", initial_count);
    let vectors = generate_vectors(initial_count, dim, 42);

    println!("Inserting {} nodes sequentially...", initial_count);
    let start_insert = Instant::now();
    for (id, vec) in vectors.into_iter().enumerate() {
        let mut node = UnifiedNode::new(id as u64);
        node.vector = VectorRepresentations::Full(vec);
        storage.insert(&node).unwrap();
    }
    println!(
        "Inserted {} nodes in {:?}",
        initial_count,
        start_insert.elapsed()
    );

    // Generate queries
    let query_pool = Arc::new(generate_vectors(1000, dim, 1337));

    // Test for different thread counts
    let thread_counts = [1, 4, 8, 16];

    println!("\n--- SCENARIO 1: READ-ONLY CONCURRENT SEARCHES ---");
    println!(
        "{:<10} | {:<15} | {:<12} | {:<12}",
        "Threads", "Throughput (QPS)", "p50 Latency", "p99 Latency"
    );
    println!("{}", "-".repeat(60));

    for &t in &thread_counts {
        run_read_only_bench(storage.clone(), query_pool.clone(), t, test_duration);
    }

    println!("\n--- SCENARIO 2: MIXED READ-WRITE CONCURRENCY ---");
    println!("(1 Thread constantly inserting new vectors while T threads search)");
    println!(
        "{:<10} | {:<15} | {:<12} | {:<12} | {:<15}",
        "Threads", "Throughput (QPS)", "p50 Latency", "p99 Latency", "Insert Rate"
    );
    println!("{}", "-".repeat(70));

    for &t in &thread_counts {
        run_mixed_bench(storage.clone(), query_pool.clone(), t, test_duration, dim);
    }
    println!("============================================================");
}

fn run_read_only_bench(
    storage: Arc<StorageEngine>,
    query_pool: Arc<Vec<Vec<f32>>>,
    num_threads: usize,
    duration: Duration,
) {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    let start = Instant::now();

    for thread_idx in 0..num_threads {
        let storage = storage.clone();
        let query_pool = query_pool.clone();
        let stop_signal = stop_signal.clone();

        handles.push(thread::spawn(move || {
            let mut local_latencies = Vec::new();
            let mut queries_done = 0;
            let mut query_idx = thread_idx % query_pool.len();

            while !stop_signal.load(Ordering::Relaxed) {
                let query = &query_pool[query_idx];
                let q_start = Instant::now();

                // Perform query
                {
                    let hnsw = storage.hnsw.load();
                    let vstore = storage.vector_store.read();
                    let _results = hnsw.search_nearest(
                        query,
                        None,
                        None,
                        &FilterBitset::all_set(),
                        10,
                        Some(&vstore),
                    );
                    std::hint::black_box(_results);
                }

                let elapsed = q_start.elapsed().as_micros() as u64;
                local_latencies.push(elapsed);
                queries_done += 1;

                query_idx = (query_idx + num_threads) % query_pool.len();
            }

            (queries_done, local_latencies)
        }));
    }

    // Run for the duration
    thread::sleep(duration);
    stop_signal.store(true, Ordering::Relaxed);

    let mut total_queries = 0;
    let mut all_latencies = Vec::new();

    for handle in handles {
        let (queries, latencies) = handle.join().unwrap();
        total_queries += queries;
        all_latencies.extend(latencies);
    }

    let actual_duration = start.elapsed();
    let qps = total_queries as f64 / actual_duration.as_secs_f64();

    all_latencies.sort_unstable();
    let p50 = if !all_latencies.is_empty() {
        format!("{:.1} µs", all_latencies[all_latencies.len() / 2] as f64)
    } else {
        "N/A".to_string()
    };
    let p99 = if !all_latencies.is_empty() {
        let idx = (all_latencies.len() as f64 * 0.99) as usize;
        let idx = idx.min(all_latencies.len() - 1);
        format!("{:.1} µs", all_latencies[idx] as f64)
    } else {
        "N/A".to_string()
    };

    println!(
        "{:<10} | {:<15.1} | {:<12} | {:<12}",
        num_threads, qps, p50, p99
    );
}

fn run_mixed_bench(
    storage: Arc<StorageEngine>,
    query_pool: Arc<Vec<Vec<f32>>>,
    num_threads: usize,
    duration: Duration,
    dim: usize,
) {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let insert_count = Arc::new(AtomicUsize::new(0));

    // Spawn 1 writer thread
    let writer_handle = {
        let storage = storage.clone();
        let stop_signal = stop_signal.clone();
        let insert_count = insert_count.clone();

        thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(999);
            let mut current_id = 20_000u64;

            while !stop_signal.load(Ordering::Relaxed) {
                // Generate random vector
                let vec: Vec<f32> = (0..dim).map(|_| rng.random::<f32>()).collect();
                let mut node = UnifiedNode::new(current_id);
                node.vector = VectorRepresentations::Full(vec);

                if storage.insert(&node).is_ok() {
                    insert_count.fetch_add(1, Ordering::Relaxed);
                    current_id += 1;
                } else {
                    // Backoff if error
                    thread::sleep(Duration::from_millis(1));
                }
            }
        })
    };

    // Spawn T search threads
    let mut search_handles = Vec::new();
    let start = Instant::now();

    for thread_idx in 0..num_threads {
        let storage = storage.clone();
        let query_pool = query_pool.clone();
        let stop_signal = stop_signal.clone();

        search_handles.push(thread::spawn(move || {
            let mut local_latencies = Vec::new();
            let mut queries_done = 0;
            let mut query_idx = thread_idx % query_pool.len();

            while !stop_signal.load(Ordering::Relaxed) {
                let query = &query_pool[query_idx];
                let q_start = Instant::now();

                // Perform query
                {
                    let hnsw = storage.hnsw.load();
                    let vstore = storage.vector_store.read();
                    let _results = hnsw.search_nearest(
                        query,
                        None,
                        None,
                        &FilterBitset::all_set(),
                        10,
                        Some(&vstore),
                    );
                    std::hint::black_box(_results);
                }

                let elapsed = q_start.elapsed().as_micros() as u64;
                local_latencies.push(elapsed);
                queries_done += 1;

                query_idx = (query_idx + num_threads) % query_pool.len();
            }

            (queries_done, local_latencies)
        }));
    }

    // Run for the duration
    thread::sleep(duration);
    stop_signal.store(true, Ordering::Relaxed);

    // Join threads
    let _ = writer_handle.join();

    let mut total_queries = 0;
    let mut all_latencies = Vec::new();

    for handle in search_handles {
        let (queries, latencies) = handle.join().unwrap();
        total_queries += queries;
        all_latencies.extend(latencies);
    }

    let actual_duration = start.elapsed();
    let qps = total_queries as f64 / actual_duration.as_secs_f64();
    let inserts_done = insert_count.load(Ordering::Relaxed);
    let insert_rate = inserts_done as f64 / actual_duration.as_secs_f64();

    all_latencies.sort_unstable();
    let p50 = if !all_latencies.is_empty() {
        format!("{:.1} µs", all_latencies[all_latencies.len() / 2] as f64)
    } else {
        "N/A".to_string()
    };
    let p99 = if !all_latencies.is_empty() {
        let idx = (all_latencies.len() as f64 * 0.99) as usize;
        let idx = idx.min(all_latencies.len() - 1);
        format!("{:.1} µs", all_latencies[idx] as f64)
    } else {
        "N/A".to_string()
    };

    println!(
        "{:<10} | {:<15.1} | {:<12} | {:<12} | {:<15.1}",
        num_threads, qps, p50, p99, insert_rate
    );
}
