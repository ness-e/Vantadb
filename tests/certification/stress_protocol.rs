//! ═══════════════════════════════════════════════════════════════════════════
//! STRESS PROTOCOL — VantaDB HNSW Certification Suite
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! This is NOT a unit test. It is a full certification protocol that must pass
//! before the HNSW engine is considered validated for production use.
//!
//! Run with: cargo test --test stress_protocol -- --nocapture
//! Sequential execution is enforced to maintain console output integrity.

use console::style;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::time::Instant;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};

#[path = "../common/mod.rs"]
mod common;
use common::*;

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn gen_vectors(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    (0..count)
        .into_par_iter() // Parallel generation
        .map(|i| {
            let mut rng = StdRng::seed_from_u64(seed + i as u64); // Distinct seed per vector
            let mut v: Vec<f32> = (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > f32::EPSILON {
                v.iter_mut().for_each(|x| *x /= norm);
            }
            v
        })
        .collect()
}

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u64> {
    let mut scored: Vec<(u64, f32)> = dataset
        .par_iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();

    // OPTIMIZATION: Only find top K instead of sorting everything
    if scored.len() > k {
        scored.select_nth_unstable_by(k - 1, |a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(id, _)| id).collect()
}

fn compute_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    dataset: &[(u64, Vec<f32>)],
    k: usize,
) -> f64 {
    let pb = TerminalReporter::create_progress(queries.len() as u64, "Computing Recall");
    let total: f64 = queries
        .par_iter()
        .map(|q| {
            let truth = brute_force_knn(q, dataset, k);
            let hnsw: Vec<u64> = index
                .search_nearest(q, None, None, u128::MAX, k, None)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            let hits = truth.iter().filter(|id| hnsw.contains(id)).count();
            pb.inc(1);
            hits as f64 / k as f64
        })
        .sum();
    pb.finish_and_clear();
    total / queries.len() as f64
}

fn build_index(dataset: &[(u64, Vec<f32>)], config: HnswConfig) -> CPIndex {
    let mut idx = CPIndex::new_with_config(config);
    let pb = TerminalReporter::create_progress(dataset.len() as u64, "Building HNSW");
    for (id, vec) in dataset {
        idx.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
        pb.inc(1);
    }
    pb.finish_and_clear();
    idx
}

fn measure_latency_percentiles(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64) {
    let mut latencies: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = index.search_nearest(q, None, None, u128::MAX, k, None);
            t.elapsed().as_nanos() as f64 / 1000.0 // µs
        })
        .collect();
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = latencies.len();
    let p50 = latencies[n / 2];
    let p95 = latencies[(n as f64 * 0.95) as usize];
    let p99 = latencies[(n as f64 * 0.99) as usize];
    (p50, p95, p99)
}

fn estimate_memory_bytes(index: &CPIndex) -> usize {
    let mut total: usize = 0;
    for node in index.nodes.values() {
        match &node.vec_data {
            VectorRepresentations::Full(v) => total += v.len() * 4,
            VectorRepresentations::Binary(b) => total += b.len() * 8,
            VectorRepresentations::Turbo(t) => total += t.len(),
            VectorRepresentations::None => {}
        }
        for layer in &node.neighbors {
            total += layer.len() * 8 + 24;
        }
        total += 8 + 16 + 8 + 24;
    }
    total += index.nodes.len() * 60;
    total
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFIED CERTIFICATION RUNNER (Strict Logic Preservation)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn stress_protocol_certification() {
    TerminalReporter::suite_banner("VANTA HNSW STRESS & PERFORMANCE PROTOCOL", 7);
    let mut harness = VantaHarness::new("VANTA STRESS PROTOCOL");

    // BLOCK 1: Recall
    harness.execute("BLOCK 1 — GROUND TRUTH RECALL (50K/128D)", || {
        let n = 50_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        TerminalReporter::sub_step("Generating synthetic datasets...");
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = gen_vectors(n_queries, dims, seed + 9999);
        let config = HnswConfig {
            m: 16, // Optimized for faster certification
            m_max0: 32,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (16_f64).ln(),
        };
        let index = build_index(&dataset, config);
        let recall = compute_recall(&index, &queries, &dataset, k);
        let status_msg = format!("Recall@{}: {:.4} (Required >= 0.95)", k, recall);
        assert!(recall >= 0.95, "BLOCK 1 FAILED: {}", status_msg);
        TerminalReporter::success(&format!("PASSED: {}", status_msg));
    });

    // BLOCK 2: Scaling
    harness.execute("BLOCK 2 — SCALING (10K → 50K → 100K)", || {
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        let scales = [10_000, 50_000, 100_000];
        let mut results = Vec::new();
        for &n in &scales {
            TerminalReporter::sub_step(&format!("Processing scale: {} vectors", n));
            let config = HnswConfig {
                m: 32,
                m_max0: 64,
                ef_construction: if n <= 10_000 {
                    200
                } else if n <= 50_000 {
                    400
                } else {
                    500
                },
                ef_search: if n <= 10_000 {
                    100
                } else if n <= 50_000 {
                    200
                } else {
                    300
                },
                ml: 1.0 / (32_f64).ln(),
            };
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let queries = gen_vectors(n_queries, dims, seed + 9999);
            let t0 = Instant::now();
            let index = build_index(&dataset, config);
            let build_s = t0.elapsed().as_secs_f64();
            let recall = compute_recall(&index, &queries, &dataset, k);
            let (p50, p95, _) = measure_latency_percentiles(&index, &queries, k);
            let mem_mb = estimate_memory_bytes(&index) as f64 / (1024.0 * 1024.0);
            results.push((n, recall, p50, p95, build_s, mem_mb));
        }

        println!(
            "\n  {}",
            style("SCALING PERFORMANCE SUMMARY").bold().underlined()
        );
        println!(
            "  {}",
            style(
                "╭───────────┬────────────┬──────────────┬──────────────┬───────────┬──────────╮"
            )
            .dim()
        );
        println!(
            "  {} {} {} {} {} {} {} {} {} {} {} {} {}",
            style("│").dim(),
            style("  Dataset  ").bold().white(),
            style("│").dim(),
            style(" Recall@10  ").bold().white(),
            style("│").dim(),
            style("  Lat p50(µs) ").bold().white(),
            style("│").dim(),
            style("  Lat p95(µs) ").bold().white(),
            style("│").dim(),
            style(" Build(s)  ").bold().white(),
            style("│").dim(),
            style(" RAM(MB)  ").bold().white(),
            style("│").dim()
        );
        println!(
            "  {}",
            style(
                "├───────────┼────────────┼──────────────┼──────────────┼───────────┼──────────┤"
            )
            .dim()
        );
        for (n, rec, p50, p95, b_s, mem) in &results {
            let recall_style = if *rec >= 0.95 {
                style(format!("{:.4}", rec)).green().bold()
            } else if *rec >= 0.90 {
                style(format!("{:.4}", rec)).yellow().bold()
            } else {
                style(format!("{:.4}", rec)).red().bold()
            };
            println!(
                "  {} {:>9} {}   {}   {}  {:>10.1} {}  {:>10.1} {}  {:>7.2} {}  {:>6.1} {}",
                style("│").dim(),
                format!("{}K", n / 1000),
                style("│").dim(),
                recall_style,
                style("│").dim(),
                p50,
                style("│").dim(),
                p95,
                style("│").dim(),
                b_s,
                style("│").dim(),
                mem,
                style("│").dim()
            );
        }
        println!(
            "  {}",
            style(
                "╰───────────┴────────────┴──────────────┴──────────────┴───────────┴──────────╯"
            )
            .dim()
        );

        assert!(results[0].1 >= 0.95);
        assert!(results[1].1 >= 0.90);
        assert!(results[2].1 >= 0.85);
        let recall_drop = results[0].1 - results[2].1;
        assert!(
            recall_drop < 0.15,
            "Catastrophic degradation: {:.4}",
            recall_drop
        );
        assert!(results[2].2 < 50_000.0, "100K p50 too slow");
        TerminalReporter::success("BLOCK 2 PASSED.");
    });

    // BLOCK 3: Memory
    harness.execute("BLOCK 3 — MEMORY MEASUREMENT", || {
        let dims = 128;
        let seed = 2024;
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let sizes = [1_000, 5_000, 10_000, 50_000];
        let mut memories = Vec::new();
        for &n in &sizes {
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let index = build_index(&dataset, config.clone());
            let m_bytes = estimate_memory_bytes(&index);
            let m_mb = m_bytes as f64 / (1024. * 1024.);
            TerminalReporter::info(&format!(
                "{:>6} vectors → {:>6.2} MB ({:.0} bytes/vector)",
                n,
                m_mb,
                m_bytes as f64 / n as f64
            ));
            memories.push(m_mb);
        }
        let ratio = memories[3] / memories[1]; // 50K / 5K
        assert!(
            (5.0..=15.0).contains(&ratio),
            "Growth ratio {:.2}x not proportional",
            ratio
        );
        TerminalReporter::success("BLOCK 3 PASSED.");
    });

    // BLOCK 4: Persistence
    harness.execute("BLOCK 4 — PERSISTENCE ROUND-TRIP", || {
        let n = 10_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = gen_vectors(n_queries, dims, seed + 9999);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let original = build_index(&dataset, config);
        let recall_before = compute_recall(&original, &queries, &dataset, k);
        let tmp = tempfile::NamedTempFile::new().unwrap();
        original.persist_to_file(tmp.path()).unwrap();
        let file_size = std::fs::metadata(tmp.path()).unwrap().len();
        TerminalReporter::info(&format!(
            "File size: {:.2} MB",
            file_size as f64 / (1024. * 1024.)
        ));
        let loaded = CPIndex::load_from_file(tmp.path()).unwrap();
        assert_eq!(loaded.nodes.len(), n);
        let recall_after = compute_recall(&loaded, &queries, &dataset, k);
        assert!((recall_before - recall_after).abs() < 0.001);
        loaded.validate_index().unwrap();
        TerminalReporter::success("BLOCK 4 PASSED.");
    });

    // BLOCK 5: Edge Cases (5a-5g)
    harness.execute("BLOCK 5 — EDGE CASES", || {
        let k = 5;
        let d = 64;
        TerminalReporter::sub_step("5a: Empty index...");
        let empty = CPIndex::new();
        assert!(empty
            .search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
            .is_empty());

        TerminalReporter::sub_step("5b: Single node...");
        let mut single = CPIndex::new();
        single.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        assert_eq!(
            single
                .search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
                .len(),
            1
        );

        TerminalReporter::sub_step("5c: Two nodes...");
        let mut two = CPIndex::new();
        two.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        two.add(2, u128::MAX, VectorRepresentations::Full(vec![-1.0; d]), 0);
        assert_eq!(
            two.search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
                .len(),
            2
        );

        TerminalReporter::sub_step("5d: Zero vector...");
        let mut zvec = CPIndex::new();
        zvec.add(1, u128::MAX, VectorRepresentations::Full(vec![0.0; d]), 0);
        assert!(!zvec
            .search_nearest(&vec![0.0; d], None, None, u128::MAX, k, None)
            .is_empty());

        TerminalReporter::sub_step("5e: Duplicate ID...");
        let mut dup = CPIndex::new();
        dup.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        dup.add(1, u128::MAX, VectorRepresentations::Full(vec![-1.0; d]), 0);
        assert_eq!(dup.nodes.len(), 1);

        TerminalReporter::sub_step("5f: Dimension Mismatch...");
        let mut dvec = CPIndex::new();
        dvec.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        let _ = dvec.search_nearest(&vec![1.0; 128], None, None, u128::MAX, k, None);

        TerminalReporter::sub_step("5g: k > n...");
        let results = dvec.search_nearest(&vec![1.0; d], None, None, u128::MAX, 100, None);
        assert!(results.len() == 1);

        TerminalReporter::success("BLOCK 5 PASSED.");
    });

    // BLOCK 6: Consistency
    harness.execute("BLOCK 6 — GRAPH CONSISTENCY", || {
        let n = 50_000;
        let dims = 128;
        let seed = 2024;
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 400,
            ef_search: 200,
            ml: 1.0 / (32_f64).ln(),
        };
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let index = build_index(&dataset, config);
        index.validate_index().unwrap();
        let stats = index.stats();
        TerminalReporter::info(&format!(
            "Nodes: {} | Orphans: {} | Avg L0 Conn: {:.1}",
            stats.node_count, stats.orphan_count, stats.avg_connections_l0
        ));
        assert!(stats.orphan_count <= 1);
        TerminalReporter::success("BLOCK 6 PASSED.");
    });

    // BLOCK 7: Latency
    harness.execute("BLOCK 7 — LATENCY PERCENTILES", || {
        let n1 = 10000;
        let n2 = 50000;
        let dims = 128;
        let seed = 2024;
        let mut results = Vec::new();
        for &n in &[n1, n2] {
            let config = HnswConfig {
                m: 32,
                m_max0: 64,
                ef_construction: if n <= 10000 { 200 } else { 400 },
                ef_search: if n <= 10000 { 100 } else { 200 },
                ml: 1.0 / (32_f64).ln(),
            };
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let queries = gen_vectors(200, dims, seed + 9999);
            let index = build_index(&dataset, config);
            let (p50, p95, p99) = measure_latency_percentiles(&index, &queries, 10);
            TerminalReporter::info(&format!(
                "{}K vectors -> p50: {:.1}µs | p95: {:.1}µs | p99: {:.1}µs",
                n / 1000,
                p50,
                p95,
                p99
            ));
            results.push(p50);
        }
        let s_factor = results[1] / results[0];
        TerminalReporter::info(&format!("Latency scale factor (50K/10K): {:.2}x", s_factor));
        // Threshold: 8.0x accounts for CPU cache/thermal variance between runs.
        // Theoretical HNSW: ~1.7x for 5x data. Practical observed: 2.6x–5.6x.
        // See docs/problemas_encontrados_en_tests.md for analysis.
        assert!(s_factor < 8.0, "Latency scales too fast: {:.2}x", s_factor);
        TerminalReporter::success("BLOCK 7 PASSED.");
    });
}
