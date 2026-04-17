//! HNSW Hard Validation — Vanta Certification Suite
//!
//! Validates the algorithmic correctness, stability, and edge-case handling
//! of the HNSW engine under heavy loads and adverse data distributions.
//!
//! Run with: cargo test --test hnsw_validation -- --nocapture
//! Sequential execution is enforced to maintain console output integrity.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use console::style;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn generate_vectors_seeded(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
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

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u64> {
    let mut sims: Vec<(u64, f32)> = dataset
        .iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sims.truncate(k);
    sims.into_iter().map(|(id, _)| id).collect()
}

fn build_index(dataset: &[(u64, Vec<f32>)], config: HnswConfig, block_msg: &str) -> CPIndex {
    let pb = TerminalReporter::create_progress(dataset.len() as u64, block_msg);
    let mut index = CPIndex::new_with_config(config);
    for (id, vec) in dataset {
        index.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
        pb.inc(1);
    }
    pb.finish_and_clear();
    index
}

fn compute_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    dataset: &[(u64, Vec<f32>)],
    k: usize,
    block_msg: &str,
) -> f64 {
    let pb = TerminalReporter::create_progress(queries.len() as u64, block_msg);
    let mut total_recall = 0.0;
    for query in queries {
        let truth = brute_force_knn(query, dataset, k);
        let hnsw_ids: Vec<u64> = index
            .search_nearest(query, None, None, u128::MAX, k, None)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let hits = truth.iter().filter(|id| hnsw_ids.contains(id)).count();
        total_recall += hits as f64 / k as f64;
        pb.inc(1);
    }
    pb.finish_and_clear();
    total_recall / queries.len() as f64
}

// ═══════════════════════════════════════════════════════════════════════
// UNIFIED CERTIFICATION RUNNER (Full Logic Expansion)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn hnsw_hard_validation_certification() {
    let mut harness = VantaHarness::new("HNSW HARD VALIDATION");

    // ─────────────────────────────────────────────────────────────────
    // SCALE VALIDATIONS
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Scale Check: 1K Vectors", || {
        let n = 1_000;
        let dims = 128;
        let k = 10;
        let n_queries = 200;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.95)", recall));
        assert!(recall >= 0.95);
    });

    harness.execute("Scale Check: 10K Vectors", || {
        let n = 10_000;
        let dims = 128;
        let k = 10;
        let n_queries = 200;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 400,
            ef_search: 200,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.90)", recall));
        assert!(recall >= 0.90);
    });

    harness.execute("Scale Check: 50K Vectors", || {
        let n = 50_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 500,
            ef_search: 350,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.85)", recall));
        assert!(recall >= 0.85);
    });

    // ─────────────────────────────────────────────────────────────────
    // STABILITY VALIDATIONS
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Determinism: Same Query -> Same Result", || {
        let n = 5_000;
        let dims = 64;
        let k = 10;
        let seed = 99;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(20, dims, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        for query in &queries {
            let first = index.search_nearest(query, None, None, u128::MAX, k, None);
            let first_ids: Vec<u64> = first.iter().map(|(id, _)| *id).collect();
            for _ in 1..5 {
                let repeat = index.search_nearest(query, None, None, u128::MAX, k, None);
                let repeat_ids: Vec<u64> = repeat.iter().map(|(id, _)| *id).collect();
                assert_eq!(first_ids, repeat_ids);
            }
        }
        TerminalReporter::success("Consistency verified for all query batches.");
    });

    harness.execute("Recall vs ef_search Degradation Curve", || {
        let n = 10_000;
        let seed = 77;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(100, 64, seed + 500);
        let mut index = build_index(&dataset, HnswConfig::default(), "Building");
        let ef_values = [10, 20, 50, 100, 200];
        let mut prev_recall = 0.0;
        for &ef in &ef_values {
            index.config.ef_search = ef;
            let recall = compute_recall(&index, &queries, &dataset, 10, &format!("ef={}", ef));
            TerminalReporter::info(&format!("  ef_search={:>3} → recall={:.4}", ef, recall));
            assert!(recall >= prev_recall - 0.02);
            prev_recall = recall;
        }
        assert!(prev_recall >= 0.95);
    });

    // ─────────────────────────────────────────────────────────────────
    // EDGE CASES (Individual Blocks for Transparency)
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Edge Case: Duplicate Vectors", || {
        let dims = 32;
        let k = 5;
        let iv = vec![1.0; dims];
        let mut index = CPIndex::new();
        for i in 0..100 {
            index.add(i, u128::MAX, VectorRepresentations::Full(iv.clone()), 0);
        }
        let results = index.search_nearest(&iv, None, None, u128::MAX, k, None);
        assert_eq!(results.len(), k);
        for (_, sim) in &results {
            assert!((sim - 1.0).abs() < 0.01);
        }
        TerminalReporter::success("100 identical vectors handled correctly.");
    });

    harness.execute("Edge Case: Zero Vector Resilience", || {
        let mut index = CPIndex::new();
        index.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; 32]), 0);
        index.add(2, u128::MAX, VectorRepresentations::Full(vec![0.0; 32]), 0);
        let res = index.search_nearest(&vec![1.0; 32], None, None, u128::MAX, 3, None);
        assert!(!res.is_empty());
        TerminalReporter::success("Zero vector in index did not cause panics.");
    });

    harness.execute("Edge Case: Single Node Index", || {
        let mut index = CPIndex::new();
        index.add(42, u128::MAX, VectorRepresentations::Full(vec![1.0; 16]), 0);
        let res = index.search_nearest(&vec![1.0; 16], None, None, u128::MAX, 10, None);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0, 42);
    });

    harness.execute("Edge Case: Empty Index", || {
        let res = CPIndex::new().search_nearest(&vec![1.0; 16], None, None, u128::MAX, 10, None);
        assert!(res.is_empty());
    });

    harness.execute("Stress: High Dimensionality (768D)", || {
        let n = 1_000;
        let dims = 768;
        let k = 10;
        let n_queries = 50;
        let seed = 55;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building 768D");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching 768D");
        TerminalReporter::info(&format!("Recall@10 (768D): {:.4}", recall));
        assert!(recall >= 0.90);
    });

    // ─────────────────────────────────────────────────────────────────
    // ACCURACY & COVERAGE
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Validation: Top-1 Accuracy Correctness", || {
        let n = 5_000;
        let seed = 33;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(200, 64, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        let mut hits = 0;
        for q in &queries {
            let truth = brute_force_knn(q, &dataset, 1);
            let res = index.search_nearest(q, None, None, u128::MAX, 1, None);
            if !res.is_empty() && res[0].0 == truth[0] {
                hits += 1;
            }
        }
        let acc = hits as f64 / queries.len() as f64;
        TerminalReporter::info(&format!("Top-1 Precision: {:.4}", acc));
        assert!(acc >= 0.95);
    });

    harness.execute("Validation: Recall@K Sweep (1 to 50)", || {
        let n = 10_000;
        let seed = 88;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(100, 64, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        for &k in &[1, 5, 10, 20, 50] {
            let recall = compute_recall(&index, &queries, &dataset, k, &format!("Recall@K={}", k));
            TerminalReporter::info(&format!("  Recall@{:>2}: {:.4}", k, recall));
            assert!(recall >= 0.80);
        }
    });

    harness.execute("Validation: Memory proportionality", || {
        let dims = 64;
        let seed = 44;
        let ds1 = generate_vectors_seeded(1000, dims, seed)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect::<Vec<_>>();
        let idx1 = build_index(&ds1, HnswConfig::default(), "Building 1K");
        let ds5 = generate_vectors_seeded(5000, dims, seed)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect::<Vec<_>>();
        let idx5 = build_index(&ds5, HnswConfig::default(), "Building 5K");
        let links1: usize = idx1
            .nodes
            .values()
            .map(|n| n.neighbors.iter().map(|l| l.len()).sum::<usize>())
            .sum();
        let links5: usize = idx5
            .nodes
            .values()
            .map(|n| n.neighbors.iter().map(|l| l.len()).sum::<usize>())
            .sum();
        let ratio = links5 as f64 / links1 as f64;
        TerminalReporter::info(&format!("Memory Growth Factor (5x N): {:.2}x links", ratio));
        assert!(ratio >= 3.0 && ratio <= 8.0);
    });

    println!(
        "\n{}",
        style("VANTA HNSW HARD VALIDATION COMPLETE").green().bold()
    );
}
