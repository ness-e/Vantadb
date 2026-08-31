use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vantadb::index::auto_tune::AutoTune;
use vantadb::index::FilterBitset;
use vantadb::node::{UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

const DIM: usize = 128;
const INITIAL_COUNT: usize = 10_000;
const TEST_DURATION: Duration = Duration::from_secs(3);
const THREAD_COUNTS: [usize; 4] = [1, 4, 8, 16];

fn generate_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.random::<f32>()).collect())
        .collect()
}

/// Setup the shared storage + query pool once before running the bench group.
/// Returns `(storage, query_pool)`. Done outside `bench_function` so it isn't
/// included in the timed iter (criterion warmup is separate).
fn setup_storage() -> (Arc<StorageEngine>, Arc<Vec<Vec<f32>>>) {
    AutoTune::set_ef(100);
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    eprintln!("[bench_concurrent] initializing storage at {}...", db_path);

    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    eprintln!("[bench_concurrent] inserting {} nodes...", INITIAL_COUNT);

    let start_insert = Instant::now();
    let vectors = generate_vectors(INITIAL_COUNT, DIM, 42);
    for (id, vec) in vectors.into_iter().enumerate() {
        let mut node = UnifiedNode::new(id as u128);
        node.vector = VectorRepresentations::Full(vec);
        storage.insert(&node).unwrap();
    }
    eprintln!(
        "[bench_concurrent] inserted {} nodes in {:?}",
        INITIAL_COUNT,
        start_insert.elapsed()
    );

    let query_pool = Arc::new(generate_vectors(1000, DIM, 1337));
    (storage, query_pool)
}

fn bench_concurrent(c: &mut Criterion) {
    let (storage, query_pool) = setup_storage();

    // Measure baseline (t=1) ONCE before the criterion loop so the per-thread
    // bench functions can use a stable speedup/efficiency reference. The
    // baseline itself is NOT a `bench_function` — it's printed via `eprintln!`
    // and re-printed for each subsequent (t>1) iteration in `run_read_only_bench`.
    eprintln!();
    eprintln!("[bench_concurrent] measuring baseline (t=1) for speedup reference...");
    let baseline_qps =
        run_read_only_bench(storage.clone(), query_pool.clone(), 1, TEST_DURATION, 0.0);

    let mut group = c.benchmark_group("bench_concurrent");
    group.sample_size(10);

    eprintln!();
    eprintln!("--- SCENARIO 1: READ-ONLY CONCURRENT SEARCHES ---");
    eprintln!(
        "{:<8} | {:<16} | {:<12} | {:<12} | {:<10} | {:<10}",
        "Threads", "Throughput (QPS)", "p50 Latency", "p99 Latency", "Speedup", "Efficiency"
    );
    eprintln!("{}", "-".repeat(78));

    for &t in &THREAD_COUNTS {
        let storage = storage.clone();
        let query_pool = query_pool.clone();
        // t=1 uses baseline_qps=0.0 so speedup=1.00x/eff=100% (baseline is the
        // t=1 run we just did, already printed above). For t>1 we pass the
        // measured baseline so the in-iter printout is meaningful.
        let bench_baseline = if t == 1 { 0.0 } else { baseline_qps };
        group.bench_function(format!("read_only/t{}", t), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let _ = run_read_only_bench(
                        storage.clone(),
                        query_pool.clone(),
                        t,
                        TEST_DURATION,
                        bench_baseline,
                    );
                    total += TEST_DURATION;
                }
                total
            });
        });
    }

    eprintln!();
    eprintln!("--- SCENARIO 2: MIXED READ-WRITE CONCURRENCY ---");
    eprintln!("(1 Thread constantly inserting new vectors while T threads search)");
    eprintln!(
        "{:<8} | {:<16} | {:<12} | {:<12} | {:<15}",
        "Threads", "Throughput (QPS)", "p50 Latency", "p99 Latency", "Insert Rate"
    );
    eprintln!("{}", "-".repeat(72));

    for &t in &THREAD_COUNTS {
        let storage = storage.clone();
        let query_pool = query_pool.clone();
        group.bench_function(format!("mixed_rw/t{}", t), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    run_mixed_bench(storage.clone(), query_pool.clone(), t, TEST_DURATION, DIM);
                    total += TEST_DURATION;
                }
                total
            });
        });
    }

    group.finish();
}

fn run_read_only_bench(
    storage: Arc<StorageEngine>,
    query_pool: Arc<Vec<Vec<f32>>>,
    num_threads: usize,
    duration: Duration,
    baseline_qps: f64,
) -> f64 {
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
                    let vstore = storage.vector_store[0].read();
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

    let speedup_str = if baseline_qps > 0.0 {
        format!("{:.2}x", qps / baseline_qps)
    } else {
        "1.00x".to_string()
    };

    let eff_str = if baseline_qps > 0.0 {
        format!(
            "{:.1}%",
            (qps / (baseline_qps * num_threads as f64)) * 100.0
        )
    } else {
        "100.0%".to_string()
    };

    eprintln!(
        "{:<8} | {:<16.1} | {:<12} | {:<12} | {:<10} | {:<10}",
        num_threads, qps, p50, p99, speedup_str, eff_str
    );
    qps
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
                let mut node = UnifiedNode::new(current_id.into());
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
                    let vstore = storage.vector_store[0].read();
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

    eprintln!(
        "{:<10} | {:<15.1} | {:<12} | {:<12} | {:<15.1}",
        num_threads, qps, p50, p99, insert_rate
    );
}

criterion_group!(benches, bench_concurrent);
criterion_main!(benches);
