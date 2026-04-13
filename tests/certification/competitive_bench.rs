//! ═══════════════════════════════════════════════════════════════════════════
//! COMPETITIVE BENCHMARK — VantaDB vs SIFT1M Ground Truth
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! Phase 2.1/2.2: Real-world dataset benchmark using the standard SIFT1M
//! dataset (128D, 1M vectors) with pre-computed ground truth.
//!
//! Run with: cargo test --test competitive_bench --release -- --nocapture
//!
//! Requires: datasets/sift/{sift_base.fvecs, sift_query.fvecs, sift_groundtruth.ivecs}

use std::path::Path;
use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};
use console::style;

#[path = "../common/mod.rs"]
mod common;

use common::sift_loader::{read_fvecs, read_ivecs};
use common::{VantaHarness, TerminalReporter};

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate Recall@K by comparing VantaDB results against SIFT1M ground truth.
fn calculate_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    groundtruth: &[Vec<usize>],
    k: usize,
) -> f64 {
    let mut total_hits = 0;

    for (i, query) in queries.iter().enumerate() {
        let results = index.search_nearest(query, None, None, u128::MAX, k, None);
        let gt_k = &groundtruth[i][..k];

        for (id, _score) in &results {
            if gt_k.contains(&(*id as usize)) {
                total_hits += 1;
            }
        }
    }

    total_hits as f64 / (queries.len() * k) as f64
}

/// Measure per-query latency percentiles (p50, p95, p99) in microseconds.
fn measure_latency(
    index: &CPIndex,
    queries: &[Vec<f32>],
    k: usize,
) -> (f64, f64, f64, f64) {
    let mut latencies_us: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = index.search_nearest(q, None, None, u128::MAX, k, None);
            t.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = latencies_us.len();
    let p50 = latencies_us[n / 2];
    let p95 = latencies_us[(n as f64 * 0.95) as usize];
    let p99 = latencies_us[(n as f64 * 0.99) as usize];
    let qps = queries.len() as f64 / (latencies_us.iter().sum::<f64>() / 1_000_000.0);

    (p50, p95, p99, qps)
}

// ═══════════════════════════════════════════════════════════════════════════
// BENCHMARK RUNNER
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sift1m_competitive_benchmark() {
    let base_path = Path::new("datasets/sift/sift_base.fvecs");
    let query_path = Path::new("datasets/sift/sift_query.fvecs");
    let gt_path = Path::new("datasets/sift/sift_groundtruth.ivecs");

    if !base_path.exists() {
        println!("⚠️  SIFT dataset not found at datasets/sift/. Skipping.");
        println!("   Download from: http://corpus-texmex.irisa.fr/");
        return;
    }

    let mut harness = VantaHarness::new("SIFT1M_Competitive");

    // ── Load Dataset ─────────────────────────────────────────────────────
    let base_vectors = harness.execute("Load SIFT Base (1M × 128D)", || {
        read_fvecs(base_path).expect("Failed to read sift_base.fvecs")
    });

    let query_vectors = harness.execute("Load SIFT Queries (10K × 128D)", || {
        read_fvecs(query_path).expect("Failed to read sift_query.fvecs")
    });

    let groundtruth = harness.execute("Load Ground Truth", || {
        read_ivecs(gt_path).expect("Failed to read sift_groundtruth.ivecs")
    });

    // Integrity gate
    assert_eq!(base_vectors[0].len(), 128);
    assert_eq!(query_vectors[0].len(), 128);
    println!(
        "\n  {} Dataset: {} base, {} queries, {} GT entries",
        style("✓").green().bold(),
        base_vectors.len(),
        query_vectors.len(),
        groundtruth.len()
    );

    // ── Benchmark Scenarios ──────────────────────────────────────────────
    // SIFT uses L2 distance, but our engine uses cosine sim.
    // We test recall against the official ground truth anyway —
    // lower recall is expected since we're not matching metric.
    // This is the honest, no-bullshit measurement.

    let scales: Vec<usize> = vec![10_000, 100_000];
    let k = 10;

    struct ScenarioResult {
        scale: usize,
        config_name: String,
        recall: f64,
        p50_us: f64,
        p95_us: f64,
        _p99_us: f64,
        qps: f64,
        build_secs: f64,
    }

    let mut all_results: Vec<ScenarioResult> = Vec::new();

    for &scale in &scales {
        let scale_base = &base_vectors[..scale];

        let configs = vec![
            (
                "Balanced",
                HnswConfig {
                    m: 16,
                    m_max0: 32,
                    ef_construction: 200,
                    ef_search: 100,
                    ml: 1.0 / (16_f64).ln(),
                },
            ),
            (
                "High Recall",
                HnswConfig {
                    m: 32,
                    m_max0: 64,
                    ef_construction: 400,
                    ef_search: 200,
                    ml: 1.0 / (32_f64).ln(),
                },
            ),
        ];

        for (config_name, config) in configs {
            let block_name = format!("SIFT {}K — {}", scale / 1000, config_name);

            let (index, build_secs) = harness.execute(&block_name, || {
                let mut idx = CPIndex::new_with_config(config.clone());
                let pb = TerminalReporter::create_progress(scale as u64, "Inserting vectors");
                let t0 = Instant::now();

                for (id, vec) in scale_base.iter().enumerate() {
                    idx.add(
                        id as u64,
                        u128::MAX,
                        VectorRepresentations::Full(vec.clone()),
                        0,
                    );
                    pb.inc(1);
                }
                pb.finish_and_clear();
                let elapsed = t0.elapsed().as_secs_f64();
                (idx, elapsed)
            });

            // Recall against ground truth
            let recall = calculate_recall(&index, &query_vectors, &groundtruth, k);

            // Latency
            let (p50, p95, p99, qps) = measure_latency(&index, &query_vectors, k);

            all_results.push(ScenarioResult {
                scale,
                config_name: config_name.to_string(),
                recall,
                p50_us: p50,
                p95_us: p95,
                _p99_us: p99,
                qps,
                build_secs,
            });
        }
    }

    // ── Print Report ─────────────────────────────────────────────────────
    println!("\n");
    TerminalReporter::block_header("SIFT1M COMPETITIVE BENCHMARK RESULTS");

    println!(
        "  {}",
        style("╭──────────┬──────────────┬──────────┬────────────┬────────────┬────────────┬──────────╮").dim()
    );
    println!(
        "  {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        style("│").dim(),
        style(" Scale   ").bold(),
        style("│").dim(),
        style("   Config    ").bold(),
        style("│").dim(),
        style("Recall@10").bold(),
        style("│").dim(),
        style(" p50 (µs)  ").bold(),
        style("│").dim(),
        style(" p95 (µs)  ").bold(),
        style("│").dim(),
        style("   QPS     ").bold(),
        style("│").dim(),
        style("Build(s) ").bold(),
        style("│").dim(),
    );
    println!(
        "  {}",
        style("├──────────┼──────────────┼──────────┼────────────┼────────────┼────────────┼──────────┤").dim()
    );

    for r in &all_results {
        let recall_styled = if r.recall >= 0.90 {
            style(format!(" {:.4}  ", r.recall)).green().bold()
        } else if r.recall >= 0.70 {
            style(format!(" {:.4}  ", r.recall)).yellow().bold()
        } else {
            style(format!(" {:.4}  ", r.recall)).red().bold()
        };

        println!(
            "  {} {:>7}K {} {:^12} {} {} {} {:>9.1} {} {:>9.1} {} {:>9.0} {} {:>7.1} {}",
            style("│").dim(),
            r.scale / 1000,
            style("│").dim(),
            r.config_name,
            style("│").dim(),
            recall_styled,
            style("│").dim(),
            r.p50_us,
            style("│").dim(),
            r.p95_us,
            style("│").dim(),
            r.qps,
            style("│").dim(),
            r.build_secs,
            style("│").dim(),
        );
    }

    println!(
        "  {}",
        style("╰──────────┴──────────────┴──────────┴────────────┴────────────┴────────────┴──────────╯").dim()
    );

    println!("\n  {} Dataset: SIFT1M (128D, L2 ground truth)", style("ℹ").blue());
    println!(
        "  {} VantaDB uses cosine similarity — recall gap vs L2 GT is expected.",
        style("ℹ").blue()
    );
    println!(
        "  {} For competitive parity: Recall >= FAISS_recall - 5%",
        style("ℹ").blue()
    );

    // ── Sanity Assertions ────────────────────────────────────────────────
    // We don't fail on recall (metric mismatch), but we fail on crashes.
    for r in &all_results {
        assert!(r.recall > 0.0, "Zero recall indicates a broken search path");
        assert!(r.qps > 0.0, "Zero QPS indicates search is hanging");
    }

    TerminalReporter::success("SIFT1M Competitive Benchmark Complete.");
}
