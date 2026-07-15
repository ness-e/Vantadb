//! ═══════════════════════════════════════════════════════════════════════════
//! STRESS PROTOCOL — VantaDB HNSW Certification Suite
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! This is NOT a unit test. It is a full certification protocol that must pass
//! before the HNSW engine is considered validated for production use.
//!
//! Run with: cargo test --test stress_protocol -- --nocapture
//! Sequential execution is enforced to maintain console output integrity.
//!
//! ## Performance Optimization
//! Shared indexes are pre-built once and reused across blocks with identical
//! construction parameters. Datasets are regenerated on-demand (deterministic
//! via seed) to avoid holding >100MB of vector data in memory alongside indexes.

#![cfg(feature = "rayon")]

use console::style;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::time::Instant;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};
use vantadb::node::FilterBitset;

#[path = "../common/mod.rs"]
mod common;
use common::*;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

const DIMS: usize = 128;
const SEED: u64 = 2024;
const QUERY_SEED: u64 = SEED + 9999;
const K: usize = 10;

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn gen_vectors(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    (0..count)
        .into_par_iter() // Parallel generation
        .map(|i| {
            let mut rng = StdRng::seed_from_u64(seed + i as u64); // Distinct seed per vector
            let mut v: Vec<f32> = (0..dims).map(|_| rng.random_range(-1.0..1.0)).collect();
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > f32::EPSILON {
                v.iter_mut().for_each(|x| *x /= norm);
            }
            v
        })
        .collect()
}

/// Generate an indexed dataset: (id, vector) pairs.
/// Deterministic — same (count, dims, seed) always produces identical output.
fn gen_dataset(count: usize, dims: usize, seed: u64) -> Vec<(u64, Vec<f32>)> {
    gen_vectors(count, dims, seed)
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as u64, v))
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
                .search_nearest(q, None, None, &vantadb::node::ALL_BITSET, k, None)
                .into_iter()
                .map(|(id, _)| id as u64)
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
    let idx = CPIndex::new_with_config(config);
    let pb = TerminalReporter::create_progress(dataset.len() as u64, "Building HNSW");
    for (id, vec) in dataset {
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
}

fn measure_latency_percentiles(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64) {
    let mut latencies: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = index.search_nearest(q, None, None, &vantadb::node::ALL_BITSET, k, None);
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

// ═══════════════════════════════════════════════════════════════════════════
// SHARED CONFIGS
// ═══════════════════════════════════════════════════════════════════════════

/// m=32, ef_c=200, ef_s=100 — used by Blocks 2(10K), 3(10K), 4, 7(10K)
fn config_base() -> HnswConfig {
    HnswConfig {
        m: 32,
        m_max0: 64,
        ef_construction: 200,
        ef_search: 100,
        ml: 1.0 / (32_f64).ln(),
        distance_metric: vantadb::node::DistanceMetric::Cosine,
        flat_threshold: None,
    }
}

/// m=32, ef_c=400, ef_s=200 — used by Blocks 2(50K), 6, 7(50K)
fn config_50k_high() -> HnswConfig {
    HnswConfig {
        m: 32,
        m_max0: 64,
        ef_construction: 400,
        ef_search: 200,
        ml: 1.0 / (32_f64).ln(),
        distance_metric: vantadb::node::DistanceMetric::Cosine,
        flat_threshold: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFIED CERTIFICATION RUNNER (Shared Index Pool)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn stress_protocol_certification() {
    TerminalReporter::suite_banner("VANTA HNSW STRESS & PERFORMANCE PROTOCOL", 7);
    let mut harness = VantaHarness::new("VANTA STRESS PROTOCOL");

    // ─── Phase 0: Build shared indexes ───────────────────────────
    // Datasets are generated inside scoped blocks and dropped immediately
    // after index construction to minimize peak memory. Indexes retain
    // their own copies of vector data via VectorRepresentations::Full.

    TerminalReporter::sub_step("Building shared 10K index (m=32, ef_c=200)...");
    let t0 = Instant::now();
    let shared_idx_10k = {
        let ds = gen_dataset(10_000, DIMS, SEED);
        build_index(&ds, config_base())
    }; // ds dropped — only index survives (~11 MB)
    let shared_10k_build_s = t0.elapsed().as_secs_f64();

    TerminalReporter::sub_step("Building shared 50K index (m=32, ef_c=400)...");
    let t0 = Instant::now();
    let shared_idx_50k = {
        let ds = gen_dataset(50_000, DIMS, SEED);
        build_index(&ds, config_50k_high())
    }; // ds dropped — only index survives (~58 MB)
    let shared_50k_build_s = t0.elapsed().as_secs_f64();

    TerminalReporter::info(&format!(
        "Shared indexes ready: 10K in {:.1}s, 50K in {:.1}s",
        shared_10k_build_s, shared_50k_build_s
    ));

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 1: Recall (unique m=16 config — cannot share)
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 1 — GROUND TRUTH RECALL (50K/128D)", || {
        TerminalReporter::sub_step("Generating synthetic datasets...");
        let dataset = gen_dataset(50_000, DIMS, SEED);
        let queries = gen_vectors(100, DIMS, QUERY_SEED);
        let config = HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 200,
            ef_search: 250,
            ml: 1.0 / (16_f64).ln(),
            distance_metric: vantadb::node::DistanceMetric::Cosine,
            flat_threshold: None,
        };
        let index = build_index(&dataset, config);
        let recall = compute_recall(&index, &queries, &dataset, K);
        let status_msg = format!("Recall@{}: {:.4} (Required >= 0.95)", K, recall);
        assert!(recall >= 0.95, "BLOCK 1 FAILED: {}", status_msg);
        TerminalReporter::success(&format!("PASSED: {}", status_msg));
    });

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 2: Scaling (reuses shared 10K and 50K, only builds 100K)
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 2 — SCALING (10K → 50K → 100K)", || {
        let queries = gen_vectors(100, DIMS, QUERY_SEED);
        let mut results = Vec::new();

        // ── 10K: reuse shared index, regenerate dataset for brute-force ──
        {
            TerminalReporter::sub_step("Processing scale: 10000 vectors (shared)");
            let ds = gen_dataset(10_000, DIMS, SEED);
            let recall = compute_recall(&shared_idx_10k, &queries, &ds, K);
            let (p50, p95, _) = measure_latency_percentiles(&shared_idx_10k, &queries, K);
            let mem_mb = shared_idx_10k.estimate_memory_bytes() as f64 / (1024.0 * 1024.0);
            results.push((10_000, recall, p50, p95, shared_10k_build_s, mem_mb));
        } // ds dropped

        // ── 50K: reuse shared index ──
        {
            TerminalReporter::sub_step("Processing scale: 50000 vectors (shared)");
            let ds = gen_dataset(50_000, DIMS, SEED);
            let recall = compute_recall(&shared_idx_50k, &queries, &ds, K);
            let (p50, p95, _) = measure_latency_percentiles(&shared_idx_50k, &queries, K);
            let mem_mb = shared_idx_50k.estimate_memory_bytes() as f64 / (1024.0 * 1024.0);
            results.push((50_000, recall, p50, p95, shared_50k_build_s, mem_mb));
        } // ds dropped

        // ── 100K: build fresh (unique ef_c=500, ef_s=300) ──
        {
            TerminalReporter::sub_step("Processing scale: 100000 vectors");
            let ds = gen_dataset(100_000, DIMS, SEED);
            let config_100k = HnswConfig {
                m: 32,
                m_max0: 64,
                ef_construction: 500,
                ef_search: 300,
                ml: 1.0 / (32_f64).ln(),
                distance_metric: vantadb::node::DistanceMetric::Cosine,
                flat_threshold: None,
            };
            let t0 = Instant::now();
            let idx_100k = build_index(&ds, config_100k);
            let build_s = t0.elapsed().as_secs_f64();
            let recall = compute_recall(&idx_100k, &queries, &ds, K);
            let (p50, p95, _) = measure_latency_percentiles(&idx_100k, &queries, K);
            let mem_mb = idx_100k.estimate_memory_bytes() as f64 / (1024.0 * 1024.0);
            results.push((100_000, recall, p50, p95, build_s, mem_mb));
        } // ds + idx_100k dropped

        // ── Print summary table ──
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

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 3: Memory (consistent ef_c=200 across all sizes)
    // Reuses shared 10K index (same config); builds others fresh.
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 3 — MEMORY MEASUREMENT", || {
        let sizes = [1_000, 5_000, 10_000, 50_000];
        let mut memories = Vec::new();
        for &n in &sizes {
            // 10K matches shared_idx_10k config exactly; others built fresh
            let owned_index;
            let index: &CPIndex = if n == 10_000 {
                &shared_idx_10k
            } else {
                let ds = gen_dataset(n, DIMS, SEED);
                owned_index = build_index(&ds, config_base());
                &owned_index
            };
            let m_bytes = index.estimate_memory_bytes();
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

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 4: Persistence (reuses shared 10K index)
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 4 — PERSISTENCE ROUND-TRIP", || {
        let n = 10_000;
        let n_queries = 100;
        let ds = gen_dataset(n, DIMS, SEED);
        let queries = gen_vectors(n_queries, DIMS, QUERY_SEED);

        let recall_before = compute_recall(&shared_idx_10k, &queries, &ds, K);
        let tmp = tempfile::NamedTempFile::new().unwrap();
        shared_idx_10k.persist_to_file(tmp.path()).unwrap();
        let file_size = std::fs::metadata(tmp.path()).unwrap().len();
        TerminalReporter::info(&format!(
            "File size: {:.2} MB",
            file_size as f64 / (1024. * 1024.)
        ));
        let loaded = CPIndex::load_from_file(tmp.path(), false).unwrap();
        assert_eq!(loaded.nodes.len(), n);
        let recall_after = compute_recall(&loaded, &queries, &ds, K);
        assert!((recall_before - recall_after).abs() < 0.001);
        loaded.validate_index().unwrap();
        TerminalReporter::success("BLOCK 4 PASSED.");
    });

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 5: Edge Cases (5a-5g) — lightweight, no sharing needed
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 5 — EDGE CASES", || {
        let k = 5;
        let d = 64;
        TerminalReporter::sub_step("5a: Empty index...");
        let empty = CPIndex::new();
        assert!(empty
            .search_nearest(
                &vec![1.0; d],
                None,
                None,
                &vantadb::node::ALL_BITSET,
                k,
                None
            )
            .is_empty());

        TerminalReporter::sub_step("5b: Single node...");
        let single = CPIndex::new();
        single.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![1.0; d]),
            0,
        );
        assert_eq!(
            single
                .search_nearest(
                    &vec![1.0; d],
                    None,
                    None,
                    &vantadb::node::ALL_BITSET,
                    k,
                    None
                )
                .len(),
            1
        );

        TerminalReporter::sub_step("5c: Two nodes...");
        let two = CPIndex::new();
        two.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![1.0; d]),
            0,
        );
        two.add(
            2,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![-1.0; d]),
            0,
        );
        assert_eq!(
            two.search_nearest(
                &vec![1.0; d],
                None,
                None,
                &vantadb::node::ALL_BITSET,
                k,
                None
            )
            .len(),
            2
        );

        TerminalReporter::sub_step("5d: Zero vector...");
        let zvec = CPIndex::new();
        zvec.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![0.0; d]),
            0,
        );
        assert_eq!(
            zvec.search_nearest(
                &vec![0.0; d],
                None,
                None,
                &vantadb::node::ALL_BITSET,
                k,
                None
            )
            .len(),
            1
        );

        TerminalReporter::sub_step("5e: Duplicate ID...");
        let dup = CPIndex::new();
        dup.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![1.0; d]),
            0,
        );
        dup.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![-1.0; d]),
            0,
        );
        assert_eq!(dup.nodes.len(), 1);

        TerminalReporter::sub_step("5f: Dimension Mismatch...");
        let dvec = CPIndex::new();
        dvec.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![1.0; d]),
            0,
        );
        let _ = dvec.search_nearest(
            &vec![1.0; 128],
            None,
            None,
            &vantadb::node::ALL_BITSET,
            k,
            None,
        );

        TerminalReporter::sub_step("5g: k > n...");
        let results = dvec.search_nearest(
            &vec![1.0; d],
            None,
            None,
            &vantadb::node::ALL_BITSET,
            100,
            None,
        );
        assert!(results.len() == 1);

        TerminalReporter::success("BLOCK 5 PASSED.");
    });

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 6: Consistency (reuses shared 50K index)
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 6 — GRAPH CONSISTENCY", || {
        shared_idx_50k.validate_index().unwrap();
        let stats = shared_idx_50k.stats();
        TerminalReporter::info(&format!(
            "Nodes: {} | Orphans: {} | Avg L0 Conn: {:.1}",
            stats.node_count, stats.orphan_count, stats.avg_connections_l0
        ));
        assert!(stats.orphan_count <= 1);
        TerminalReporter::success("BLOCK 6 PASSED.");
    });

    // ═══════════════════════════════════════════════════════════════
    // BLOCK 7: Latency (reuses shared 10K and 50K indexes)
    // ═══════════════════════════════════════════════════════════════

    harness.execute("BLOCK 7 — LATENCY PERCENTILES", || {
        let queries = gen_vectors(200, DIMS, QUERY_SEED);
        let mut results = Vec::new();

        // 10K: shared index (ef_s=100) ✓
        let (p50, p95, p99) = measure_latency_percentiles(&shared_idx_10k, &queries, K);
        TerminalReporter::info(&format!(
            "10K vectors -> p50: {:.1}µs | p95: {:.1}µs | p99: {:.1}µs",
            p50, p95, p99
        ));
        results.push(p50);

        // 50K: shared index (ef_s=200) ✓
        let (p50, p95, p99) = measure_latency_percentiles(&shared_idx_50k, &queries, K);
        TerminalReporter::info(&format!(
            "50K vectors -> p50: {:.1}µs | p95: {:.1}µs | p99: {:.1}µs",
            p50, p95, p99
        ));
        results.push(p50);

        let s_factor = results[1] / results[0];
        TerminalReporter::info(&format!("Latency scale factor (50K/10K): {:.2}x", s_factor));
        // Threshold: 8.0x accounts for CPU cache/thermal variance between runs.
        // Theoretical HNSW: ~1.7x for 5x data. Practical observed: 2.6x–5.6x.
        // See docs/problemas_encontrados_en_tests.md for analysis.
        assert!(s_factor < 8.0, "Latency scales too fast: {:.2}x", s_factor);
        TerminalReporter::success("BLOCK 7 PASSED.");
    });
}
