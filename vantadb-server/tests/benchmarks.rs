//! Performance benchmarks for vantadb-server.
//!
//! These are NOT pass/fail tests — they measure latency and throughput
//! under load and print results for manual inspection or CI aggregation.
//! Run with: cargo test --features tls --test benchmarks -- --nocapture

#[path = "helpers/mod.rs"]
mod helpers;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use vantadb_server::server::{app, ServerState};

// ─── Helpers ─────────────────────────────────────────────────────────────

struct BenchContext {
    base: String,
    _dir: tempfile::TempDir,
    _handle: tokio::task::JoinHandle<()>,
    counter: Arc<AtomicU64>,
}

async fn setup_bench(concurrency: usize) -> BenchContext {
    let (_dir, state) = helpers::build_server_state(Path::new("db"), None, concurrency);
    let router = app(state, 0);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    BenchContext {
        base,
        _dir,
        _handle: handle,
        counter: Arc::new(AtomicU64::new(1000)),
    }
}

fn next_id(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::Relaxed)
}

fn make_insert_body(counter: &AtomicU64) -> String {
    let id = next_id(counter);
    format!(
        r#"{{"query":"INSERT NODE#{} TYPE Bench {{ value: \"benchmark\" }}"}}"#,
        id
    )
}

fn percentiles(mut d: Vec<f64>) -> (f64, f64, f64, f64, f64) {
    d.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let len = d.len();
    let min = d[0];
    let max = d[len - 1];
    let p50 = d[(len as f64 * 0.50).floor() as usize];
    let p95 = d[(len as f64 * 0.95).floor() as usize];
    let p99 = d[(len as f64 * 0.99).floor() as usize];
    (min, max, p50, p95, p99)
}

fn print_separator() {
    println!("{}", "-".repeat(60));
}

// ─── Benchmarks ──────────────────────────────────────────────────────────

#[tokio::test]
async fn bench_latency_serial_inserts() {
    let ctx = setup_bench(100).await;
    let client = reqwest::Client::new();

    // Warmup
    for _ in 0..20 {
        let body = make_insert_body(&ctx.counter);
        client
            .post(format!("{}/api/v2/query", ctx.base))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
    }

    // Measure 1000 serial INSERT latencies
    const N: usize = 1000;
    let mut latencies = Vec::with_capacity(N);
    for _ in 0..N {
        let body = make_insert_body(&ctx.counter);
        let start = Instant::now();
        let resp = client
            .post(format!("{}/api/v2/query", ctx.base))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(resp.status(), 200);
        latencies.push(elapsed);
    }

    let (min, max, p50, p95, p99) = percentiles(latencies);
    print_separator();
    println!("  BENCH: Serial INSERT latency ({} requests)", N);
    print_separator();
    println!("  min:  {:.3} ms", min);
    println!("  p50:  {:.3} ms", p50);
    println!("  p95:  {:.3} ms", p95);
    println!("  p99:  {:.3} ms", p99);
    println!("  max:  {:.3} ms", max);
    println!();
}

#[tokio::test]
async fn bench_throughput_concurrent_inserts() {
    let ctx = setup_bench(100).await;
    let client = reqwest::Client::new();

    // Warmup
    for _ in 0..20 {
        let body = make_insert_body(&ctx.counter);
        client
            .post(format!("{}/api/v2/query", ctx.base))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
    }

    const CONCURRENCY: usize = 32;
    const REQS_PER_TASK: usize = 32;
    const TOTAL: usize = CONCURRENCY * REQS_PER_TASK;

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..CONCURRENCY {
        let c = client.clone();
        let url = format!("{}/api/v2/query", ctx.base);
        let counter = ctx.counter.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..REQS_PER_TASK {
                let body = make_insert_body(&counter);
                let resp = c
                    .post(&url)
                    .header("content-type", "application/json")
                    .body(body)
                    .send()
                    .await
                    .unwrap();
                assert_eq!(resp.status(), 200);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();

    print_separator();
    println!(
        "  BENCH: Throughput ({} concurrent × {} req)",
        CONCURRENCY, REQS_PER_TASK
    );
    print_separator();
    println!("  total:     {} requests", TOTAL);
    println!("  elapsed:   {:.3} s", elapsed);
    println!("  throughput: {:.0} req/s", TOTAL as f64 / elapsed);
    println!();
}

#[tokio::test]
async fn bench_throughput_health_endpoint() {
    let ctx = setup_bench(100).await;
    let client = reqwest::Client::new();

    // Warmup
    for _ in 0..10 {
        client
            .get(format!("{}/health", ctx.base))
            .send()
            .await
            .unwrap();
    }

    const CONCURRENCY: usize = 64;
    const REQS_PER_TASK: usize = 50;
    const TOTAL: usize = CONCURRENCY * REQS_PER_TASK;

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..CONCURRENCY {
        let c = client.clone();
        let url = format!("{}/health", ctx.base);
        handles.push(tokio::spawn(async move {
            for _ in 0..REQS_PER_TASK {
                let resp = c.get(&url).send().await.unwrap();
                assert_eq!(resp.status(), 200);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();

    print_separator();
    println!(
        "  BENCH: Health endpoint throughput ({} concurrent)",
        CONCURRENCY
    );
    print_separator();
    println!("  total:     {} requests", TOTAL);
    println!("  elapsed:   {:.3} s", elapsed);
    println!("  throughput: {:.0} req/s", TOTAL as f64 / elapsed);
    println!();
}

#[tokio::test]
async fn bench_latency_health_serial() {
    let ctx = setup_bench(100).await;
    let client = reqwest::Client::new();

    for _ in 0..20 {
        client
            .get(format!("{}/health", ctx.base))
            .send()
            .await
            .unwrap();
    }

    const N: usize = 2000;
    let mut latencies = Vec::with_capacity(N);
    for _ in 0..N {
        let start = Instant::now();
        let resp = client
            .get(format!("{}/health", ctx.base))
            .send()
            .await
            .unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(resp.status(), 200);
        latencies.push(elapsed);
    }

    let (min, max, p50, p95, p99) = percentiles(latencies);
    print_separator();
    println!("  BENCH: Health serial latency ({} requests)", N);
    print_separator();
    println!("  min:  {:.3} ms", min);
    println!("  p50:  {:.3} ms", p50);
    println!("  p95:  {:.3} ms", p95);
    println!("  p99:  {:.3} ms", p99);
    println!("  max:  {:.3} ms", max);
    println!();
}

#[tokio::test]
async fn bench_latency_with_auth() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().join("db").to_str().unwrap()).unwrap());
    let state = Arc::new(ServerState {
        storage,
        semaphore: Arc::new(tokio::sync::Semaphore::new(100)),
        api_key: Some(Arc::from("bench-key")),
        rbac_config: Default::default(),
    });
    let router = app(state, 0);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let _handle = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let counter = Arc::new(AtomicU64::new(2000));

    // Warmup
    for _ in 0..20 {
        let body = make_insert_body(&counter);
        client
            .post(format!("{}/api/v2/query", base))
            .header("content-type", "application/json")
            .header("Authorization", "Bearer bench-key")
            .body(body)
            .send()
            .await
            .unwrap();
    }

    const N: usize = 500;
    let mut latencies = Vec::with_capacity(N);
    for _ in 0..N {
        let body = make_insert_body(&counter);
        let start = Instant::now();
        let resp = client
            .post(format!("{}/api/v2/query", base))
            .header("content-type", "application/json")
            .header("Authorization", "Bearer bench-key")
            .body(body)
            .send()
            .await
            .unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(resp.status(), 200);
        latencies.push(elapsed);
    }

    let (min, max, p50, p95, p99) = percentiles(latencies);
    print_separator();
    println!("  BENCH: Auth middleware latency ({} requests)", N);
    print_separator();
    println!("  min:  {:.3} ms", min);
    println!("  p50:  {:.3} ms", p50);
    println!("  p95:  {:.3} ms", p95);
    println!("  p99:  {:.3} ms", p99);
    println!("  max:  {:.3} ms", max);
    println!();
}
