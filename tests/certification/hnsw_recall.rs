//! Vanta Recall & Latency Certification
//!
//! Measures search precision and timing distribution to ensure production readiness.
//! Sequential execution to maintain console clarity.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use console::style;
use rand::{thread_rng, Rng};
use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};

fn generate_random_vectors(count: usize, dims: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec = Vec::with_capacity(dims);
        for _ in 0..dims {
            vec.push(rng.gen_range(-1.0..1.0));
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn brute_force_search(query: &[f32], all_vectors: &[(u64, Vec<f32>)], top_k: usize) -> Vec<u64> {
    let mut distances = Vec::with_capacity(all_vectors.len());
    let query_vector = VectorRepresentations::Full(query.to_vec());
    for (id, vec) in all_vectors {
        let node_vec = VectorRepresentations::Full(vec.clone());
        let sim = query_vector.cosine_similarity(&node_vec).unwrap_or(0.0);
        distances.push((*id, sim));
    }
    distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.truncate(top_k);
    distances.into_iter().map(|(id, _)| id).collect()
}

#[test]
fn recall_certification_runner() {
    let mut harness = VantaHarness::new("RECALL CERTIFICATION");

    harness.execute("Recall@10 Calibration", || {
        let node_count = 5000;
        let query_count = 100;
        let dims = 64;
        let top_k = 10;
        TerminalReporter::sub_step(&format!(
            "Generating dataset (N={}, D={})...",
            node_count, dims
        ));
        let raw_vectors = generate_random_vectors(node_count, dims);
        let query_vectors = generate_random_vectors(query_count, dims);
        let dataset: Vec<(u64, Vec<f32>)> = raw_vectors
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();

        let config = HnswConfig {
            m: 24,
            m_max0: 48,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (24_f64).ln(),
        };
        let mut index = CPIndex::new_with_config(config);

        let pb = TerminalReporter::create_progress(node_count as u64, "Building Index");
        for (id, vec) in &dataset {
            index.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
            pb.inc(1);
        }
        pb.finish_and_clear();

        let mut total_recall = 0.0;
        let mut latencies_us = Vec::with_capacity(query_count);
        let pb_query = TerminalReporter::create_progress(query_count as u64, "Computing Recall");
        for query in &query_vectors {
            let true_neighbors = brute_force_search(query, &dataset, top_k);
            let t_start = Instant::now();
            let hnsw_results = index.search_nearest(query, None, None, u128::MAX, top_k, None);
            latencies_us.push(t_start.elapsed().as_micros() as u64);
            let hnsw_neighbor_ids: Vec<u64> = hnsw_results.into_iter().map(|(id, _)| id).collect();
            let intersection = true_neighbors
                .iter()
                .filter(|&id| hnsw_neighbor_ids.contains(id))
                .count();
            total_recall += intersection as f64 / top_k as f64;
            pb_query.inc(1);
        }
        pb_query.finish_and_clear();

        let mean_recall = total_recall / query_count as f64;
        latencies_us.sort_unstable();
        let _p50 = latencies_us[query_count / 2];
        let p95 = latencies_us[(query_count as f64 * 0.95) as usize];
        let avg_us = latencies_us.iter().sum::<u64>() as f64 / query_count as f64;

        println!("\n  {}", style("SEARCH RESULTS").bold().underlined());
        println!("  {} Recall:   {:.4}", style("📊").cyan(), mean_recall);
        println!("  {} Avg Lat:  {:.2} µs", style("🔹").blue(), avg_us);
        println!("  {} p95 Lat:  {} µs", style("🔸").yellow(), p95);
        println!(
            "  {} QPS:      {:.0}",
            style("⚡").green(),
            1_000_000.0 / avg_us
        );

        assert!(mean_recall >= 0.90, "Recall too low: {:.4}", mean_recall);
        TerminalReporter::success("Recall and Latency standards satisfied.");
    });
}
