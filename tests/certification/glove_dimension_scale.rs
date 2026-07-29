//! GloVe Dimension Scale Certification
//!
//! Verifies HNSW recall holds across different vector dimensionalities
//! using real GloVe embeddings (100d and 300d).
//!
//! Requires: data/benchmark/glove.6B.100d.txt, data/benchmark/glove.6B.300d.txt
//! Skips gracefully if either file is missing.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use console::style;
use std::path::Path;
use vantadb::index::{CPIndex, HnswConfig, IndexType, VectorRepresentations};
use vantadb::node::{DistanceMetric, FilterBitset, ALL_BITSET};

const GLOVE_100D_PATH: &str = "data/benchmark/glove.6B.100d.txt";
const GLOVE_300D_PATH: &str = "data/benchmark/glove.6B.300d.txt";
const N_VECTORS: usize = 5_000;
const N_QUERIES: usize = 100;
const TOP_K: usize = 10;

fn load_glove(path: &str, max_lines: usize, expected_dim: usize) -> Vec<Vec<f32>> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  {} Can't open {}: {}", style("⚠").yellow(), path, e);
            return Vec::new();
        }
    };
    content
        .lines()
        .take(max_lines)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let vec: Vec<f32> = parts[1..].iter().filter_map(|s| s.parse().ok()).collect();
            if vec.len() == expected_dim {
                Some(vec)
            } else {
                None
            }
        })
        .collect()
}

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u128> {
    let q_repr = VectorRepresentations::Full(query.to_vec());
    let mut scored: Vec<(u128, f32)> = dataset
        .iter()
        .map(|(id, vec)| {
            let v_repr = VectorRepresentations::Full(vec.clone());
            let sim = q_repr.cosine_similarity(&v_repr).unwrap_or(0.0);
            (*id as u128, sim)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored.into_iter().map(|(id, _)| id).collect()
}

fn run_dimension_test(
    harness: &mut VantaHarness,
    label: &str,
    path: &str,
    dim: usize,
    recall_threshold: f64,
) {
    let vectors = harness.execute(&format!("Load {}-{}d", label, dim), || {
        let v = load_glove(path, N_VECTORS, dim);
        assert!(
            !v.is_empty(),
            "{}: no vectors loaded (expected dim={})",
            label,
            dim
        );
        assert_eq!(
            v.len(),
            N_VECTORS,
            "{}: expected {} vectors, got {}",
            label,
            N_VECTORS,
            v.len()
        );
        v
    });

    let (base, queries) = harness.execute("Split base/queries", || {
        let split = N_VECTORS - N_QUERIES;
        (vectors[..split].to_vec(), vectors[split..].to_vec())
    });

    let dataset: Vec<(u64, Vec<f32>)> = base
        .iter()
        .enumerate()
        .map(|(i, v)| (i as u64, v.clone()))
        .collect();

    let config = HnswConfig {
        m: 24,
        m_max0: 48,
        ef_construction: 200,
        ef_search: 100,
        ml: 1.0 / (24_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: IndexType::Hnsw,
    };

    let index = harness.execute(&format!("Build HNSW {}-{}d", label, dim), || {
        let idx = CPIndex::new_with_config(config);
        let pb =
            TerminalReporter::create_progress(dataset.len() as u64, &format!("Inserting {}d", dim));
        for (id, vec) in &dataset {
            idx.add(
                (*id).into(),
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                0,
            );
            pb.inc(1);
        }
        pb.finish_and_clear();
        idx
    });

    let recall = harness.execute(&format!("Compute Recall {}-{}d", label, dim), || {
        let mut hits = 0;
        let pb =
            TerminalReporter::create_progress(N_QUERIES as u64, &format!("Searching {}d", dim));
        for query in &queries {
            let truth = brute_force_knn(query, &dataset, TOP_K);

            let results = index.search_nearest(query, None, None, &ALL_BITSET, TOP_K, None);
            let hnsw_ids: Vec<u128> = results.into_iter().map(|(id, _)| id).collect();

            hits += truth.iter().filter(|id| hnsw_ids.contains(id)).count();
            pb.inc(1);
        }
        pb.finish_and_clear();
        hits as f64 / (N_QUERIES * TOP_K) as f64
    });

    println!(
        "  {} Recall@{:<2} {}-{}d:  {:.4}  (threshold ≥ {:.2})",
        if recall >= recall_threshold {
            style("✅").green()
        } else {
            style("❌").red()
        },
        TOP_K,
        label,
        dim,
        recall,
        recall_threshold,
    );

    assert!(
        recall >= recall_threshold,
        "{}-{}d recall {:.4} below threshold {:.2}",
        label,
        dim,
        recall,
        recall_threshold,
    );
}

#[test]
fn glove_dimension_scale() {
    // Graceful skip if datasets missing
    if !Path::new(GLOVE_100D_PATH).exists() || !Path::new(GLOVE_300D_PATH).exists() {
        eprintln!(
            "  {} GloVe datasets not found at data/benchmark/. Skipping.",
            style("ℹ").blue()
        );
        eprintln!("    Download via: scripts/download_benchmark_datasets.ps1");
        return;
    }

    let mut harness = VantaHarness::new("GLOVE DIMENSION SCALE");

    TerminalReporter::block_header("GloVe Dimension Scale Certification");
    println!(
        "  {} {} vectors per dim, {} queries, recall@{}",
        style("•").dim(),
        N_VECTORS,
        N_QUERIES,
        TOP_K,
    );

    run_dimension_test(&mut harness, "GloVe", GLOVE_100D_PATH, 100, 0.93);
    run_dimension_test(&mut harness, "GloVe", GLOVE_300D_PATH, 300, 0.85);

    TerminalReporter::success("GloVe dimension scale certification passed.");
}
