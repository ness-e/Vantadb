//! Internal benchmark for 10K synthetic corpus inserts, rebuild, and query latencies.

use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

fn generate_vector(i: usize) -> Vec<f32> {
    let v0 = (i % 10) as f32 / 10.0;
    let v1 = ((i + 1) % 10) as f32 / 10.0;
    let v2 = ((i + 2) % 10) as f32 / 10.0;
    let v3 = ((i + 3) % 10) as f32 / 10.0;
    vec![v0, v1, v2, v3]
}

#[test]
fn test_benchmark_internal_10k() {
    let dir = tempdir().expect("create temp dir");
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open vanta db");

    let num_records = 10000;
    let namespace = "bench/main";

    println!("Starting 10K synthetic insert benchmark...");
    let start_insert = Instant::now();
    for i in 0..num_records {
        let payload = format!(
            "synthetic memory record with token_{} category_{} and keyword_{}",
            i % 100,
            i % 10,
            i % 500
        );
        let mut input = VantaMemoryInput::new(namespace, format!("doc-{:05}", i), payload);
        input.vector = Some(generate_vector(i));
        db.put(input).expect("put record");
    }
    db.flush().expect("flush inserts");
    let insert_duration = start_insert.elapsed();
    let insert_duration_ms = insert_duration.as_millis() as f64;
    let insert_throughput = (num_records as f64) / (insert_duration.as_secs_f64());
    println!(
        "Inserted {} records in {:.2} ms ({:.2} records/sec)",
        num_records, insert_duration_ms, insert_throughput
    );

    // Measure index rebuild time
    println!("Measuring text-index rebuild time...");
    let start_rebuild = Instant::now();
    let rebuild_result = db.rebuild_index().expect("rebuild index");
    let rebuild_duration = start_rebuild.elapsed();
    let rebuild_duration_ms = rebuild_duration.as_millis() as f64;
    assert!(rebuild_result.success);
    println!("Rebuild completed in {:.2} ms", rebuild_duration_ms);

    // Latency tests: warm up first
    let search_iterations = 1000;
    let mut hybrid_latencies = Vec::new();
    let mut vector_latencies = Vec::new();
    let mut text_latencies = Vec::new();

    // Query 1: Hybrid RRF Search
    println!(
        "Benchmarking Hybrid RRF Search ({} iterations)...",
        search_iterations
    );
    for i in 0..search_iterations {
        let q_vec = generate_vector(i);
        let text_q = format!("token_{} keyword_{}", i % 100, i % 500);
        let start_query = Instant::now();
        let results = db
            .search(VantaMemorySearchRequest {
                namespace: namespace.to_string(),
                query_vector: q_vec,
                filters: Default::default(),
                text_query: Some(text_q),
                top_k: 10,
                ..Default::default()
            })
            .expect("search");
        let elapsed = start_query.elapsed().as_secs_f64() * 1000.0; // milliseconds
        hybrid_latencies.push(elapsed);
        let _ = results.len();
    }

    // Query 2: Vector-only HNSW Search
    println!(
        "Benchmarking Vector-only HNSW Search ({} iterations)...",
        search_iterations
    );
    for i in 0..search_iterations {
        let q_vec = generate_vector(i);
        let start_query = Instant::now();
        let results = db
            .search(VantaMemorySearchRequest {
                namespace: namespace.to_string(),
                query_vector: q_vec,
                filters: Default::default(),
                text_query: None,
                top_k: 10,
                ..Default::default()
            })
            .expect("search");
        let elapsed = start_query.elapsed().as_secs_f64() * 1000.0;
        vector_latencies.push(elapsed);
        let _ = results.len();
    }

    // Query 3: Text-only BM25 Search
    println!(
        "Benchmarking Text-only BM25 Search ({} iterations)...",
        search_iterations
    );
    for i in 0..search_iterations {
        let text_q = format!("token_{} keyword_{}", i % 100, i % 500);
        let start_query = Instant::now();
        let results = db
            .search(VantaMemorySearchRequest {
                namespace: namespace.to_string(),
                query_vector: Vec::new(),
                filters: Default::default(),
                text_query: Some(text_q),
                top_k: 10,
                ..Default::default()
            })
            .expect("search");
        let elapsed = start_query.elapsed().as_secs_f64() * 1000.0;
        text_latencies.push(elapsed);
        let _ = results.len();
    }

    let calculate_percentiles = |mut latencies: Vec<f64>| -> (f64, f64, f64) {
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = latencies.len();
        let p50 = latencies[(len as f64 * 0.50) as usize];
        let p95 = latencies[((len as f64 * 0.95) as usize).min(len - 1)];
        let p99 = latencies[((len as f64 * 0.99) as usize).min(len - 1)];
        (p50, p95, p99)
    };

    let (h_p50, h_p95, h_p99) = calculate_percentiles(hybrid_latencies);
    let (v_p50, v_p95, v_p99) = calculate_percentiles(vector_latencies);
    let (t_p50, t_p95, t_p99) = calculate_percentiles(text_latencies);

    println!(
        "Hybrid   p50: {:.3}ms, p95: {:.3}ms, p99: {:.3}ms",
        h_p50, h_p95, h_p99
    );
    println!(
        "Vector   p50: {:.3}ms, p95: {:.3}ms, p99: {:.3}ms",
        v_p50, v_p95, v_p99
    );
    println!(
        "Text     p50: {:.3}ms, p95: {:.3}ms, p99: {:.3}ms",
        t_p50, t_p95, t_p99
    );

    let report = json!({
        "insert": {
            "total_records": num_records,
            "total_duration_ms": insert_duration_ms,
            "throughput_records_per_sec": insert_throughput
        },
        "rebuild": {
            "duration_ms": rebuild_duration_ms
        },
        "query_hybrid": {
            "p50_ms": h_p50,
            "p95_ms": h_p95,
            "p99_ms": h_p99
        },
        "query_vector": {
            "p50_ms": v_p50,
            "p95_ms": v_p95,
            "p99_ms": v_p99
        },
        "query_text": {
            "p50_ms": t_p50,
            "p95_ms": t_p95,
            "p99_ms": t_p99
        }
    });

    let report_path = "vanta_benchmark_report.json";
    let mut file = File::create(report_path).expect("create report file");
    file.write_all(
        serde_json::to_string_pretty(&report)
            .expect("pretty json")
            .as_bytes(),
    )
    .expect("write report json");

    println!("Benchmark report saved to {}", report_path);
}
