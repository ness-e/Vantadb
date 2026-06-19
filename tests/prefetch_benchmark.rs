/// Benchmark comparing search performance with and without prefetch
/// on the current hardware. This helps determine if prefetch benefits
/// modern SSDs or if it's only useful for HDDs/slow storage.
use std::time::Instant;
use tempfile::TempDir;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};
use vantadb::DistanceMetric;

fn insert_vectors(db: &VantaEmbedded, count: usize, dim: usize) {
    for i in 0..count {
        let vec: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
        let mut input = VantaMemoryInput::new("bench", i.to_string(), format!("doc {}", i));
        input.vector = Some(vec);
        db.put(input).unwrap();
    }
}

fn measure_search_latency(db: &VantaEmbedded, query: Vec<f32>, _dim: usize, iterations: usize, top_k: usize) -> f64 {
    // Warmup
    for _ in 0..10 {
        let request = VantaMemorySearchRequest {
            namespace: "bench".to_string(),
            query_vector: query.clone(),
            filters: Default::default(),
            text_query: None,
            top_k,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
        };
        let _ = db.search(request);
    }

    // Measure
    let start = Instant::now();
    for _ in 0..iterations {
        let request = VantaMemorySearchRequest {
            namespace: "bench".to_string(),
            query_vector: query.clone(),
            filters: Default::default(),
            text_query: None,
            top_k,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
        };
        let _ = db.search(request);
    }
    start.elapsed().as_secs_f64() / iterations as f64 * 1000.0
}

#[test]
fn test_prefetch_impact_on_search_latency() {
    let vector_count = 500;
    let vector_dim = 64;
    let query_iterations = 50;
    let top_k = 10;

    let query: Vec<f32> = (0..vector_dim).map(|_| rand::random::<f32>()).collect();

    // --- Run with prefetch ENABLED (default) ---
    let avg_prefetch_on = {
        let dir = TempDir::new().unwrap();
        let db = VantaEmbedded::open(dir.path().to_str().unwrap()).unwrap();
        insert_vectors(&db, vector_count, vector_dim);
        std::env::remove_var("VANTA_DISABLE_PREFETCH");
        measure_search_latency(&db, query.clone(), vector_dim, query_iterations, top_k)
    };
    println!("PREFETCH_ON:  avg {:.3}ms over {} queries", avg_prefetch_on, query_iterations);

    // --- Run with prefetch DISABLED ---
    let avg_prefetch_off = {
        let dir = TempDir::new().unwrap();
        let db = VantaEmbedded::open(dir.path().to_str().unwrap()).unwrap();
        insert_vectors(&db, vector_count, vector_dim);
        std::env::set_var("VANTA_DISABLE_PREFETCH", "1");
        measure_search_latency(&db, query.clone(), vector_dim, query_iterations, top_k)
    };
    println!("PREFETCH_OFF: avg {:.3}ms over {} queries", avg_prefetch_off, query_iterations);

    let ratio = avg_prefetch_on / avg_prefetch_off;
    println!();
    println!("=== PREFETCH IMPACT REPORT ===");
    println!("Prefetch ON:  {:.3}ms", avg_prefetch_on);
    println!("Prefetch OFF: {:.3}ms", avg_prefetch_off);
    println!("Ratio (on/off): {:.3}x", ratio);
    if ratio < 0.95 {
        println!("→ Prefetch is BENEFICIAL on this hardware (faster with prefetch)");
    } else if ratio > 1.05 {
        println!("→ Prefetch is HARMFUL on this hardware (slower with prefetch)");
    } else {
        println!("→ Prefetch has NO SIGNIFICANT IMPACT on this hardware (within noise)");
    }
    println!("================================");
}
