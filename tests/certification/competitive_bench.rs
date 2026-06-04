//! ═══════════════════════════════════════════════════════════════════════════
//! NON-COMPARABLE BENCHMARK — VantaDB vs SIFT1M Ground Truth
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! Stress-oriented dataset benchmark using SIFT1M ground truth.
//!
//! **Release mode is required.** Debug/dev profiles disable SIMD/inlining and
//! produce misleading latency numbers:
//!   cargo test --test competitive_bench --release -- --nocapture
//!
//! Scenario classes:
//! - Product (Cosine): shipped product metric; SIFT L2 ground truth is not comparable.
//! - Stress (L2): honest L2 measurement against SIFT ground truth.
//! - Stress (L2 Mmap): zero-copy mmap path at 100K scale where page locality matters.
//!
//! Requires: datasets/sift/{sift_base.fvecs, sift_query.fvecs, sift_groundtruth.ivecs}

use console::style;
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;
use vantadb::index::{CPIndex, HnswConfig, IndexBackend, VectorRepresentations};
use vantadb::node::DistanceMetric;

#[path = "../common/mod.rs"]
mod common;

use common::sift_loader::{read_fvecs, read_ivecs};
use common::{TerminalReporter, VantaHarness};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScenarioClass {
    ProductCosine,
    StressL2,
    StressL2Mmap,
}

impl ScenarioClass {
    fn label(self) -> &'static str {
        match self {
            ScenarioClass::ProductCosine => "product-cosine",
            ScenarioClass::StressL2 => "stress-l2",
            ScenarioClass::StressL2Mmap => "stress-l2-mmap",
        }
    }
}

struct ScenarioResult {
    scale: usize,
    config_name: String,
    class: ScenarioClass,
    recall: f64,
    p50_us: f64,
    _p95_us: f64,
    p99_us: f64,
    qps: f64,
    build_secs: f64,
}

fn assert_release_profile() {
    if cfg!(debug_assertions) {
        panic!(
            "competitive_bench must run with --release (debug/dev profiles skew latency by 10-20x)"
        );
    }
}

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

fn measure_latency(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64, f64) {
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

fn build_in_memory_index(
    config: &HnswConfig,
    scale_base: &[Vec<f32>],
    scale: usize,
) -> (CPIndex, f64) {
    let idx = CPIndex::new_with_config(config.clone());
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
    (idx, t0.elapsed().as_secs_f64())
}

fn build_mmap_index(
    config: &HnswConfig,
    scale_base: &[Vec<f32>],
    scale: usize,
    mmap_path: &Path,
) -> (CPIndex, f64) {
    let mut idx = CPIndex::new_with_config(config.clone());
    idx.backend = IndexBackend::new_mmap(mmap_path.to_path_buf());
    let pb = TerminalReporter::create_progress(scale as u64, "Inserting vectors (mmap backend)");
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

    idx.sync_to_mmap()
        .expect("mmap sync failed during benchmark setup");
    let loaded = CPIndex::load_from_file(mmap_path, false).expect("mmap cold load failed");
    (loaded, t0.elapsed().as_secs_f64())
}

#[test]
fn sift1m_competitive_benchmark() {
    assert_release_profile();

    let base_path = Path::new("datasets/sift/sift_base.fvecs");
    let query_path = Path::new("datasets/sift/sift_query.fvecs");
    let gt_path = Path::new("datasets/sift/sift_groundtruth.ivecs");

    if !base_path.exists() {
        println!("SIFT dataset not found at datasets/sift/. Skipping.");
        println!("Download from: http://corpus-texmex.irisa.fr/");
        return;
    }

    let mut harness = VantaHarness::new("SIFT1M_Competitive");

    let base_vectors = harness.execute("Load SIFT Base (1M × 128D)", || {
        read_fvecs(base_path).expect("Failed to read sift_base.fvecs")
    });

    let query_vectors = harness.execute("Load SIFT Queries (10K × 128D)", || {
        read_fvecs(query_path).expect("Failed to read sift_query.fvecs")
    });

    let groundtruth = harness.execute("Load Ground Truth", || {
        read_ivecs(gt_path).expect("Failed to read sift_groundtruth.ivecs")
    });

    assert_eq!(base_vectors[0].len(), 128);
    assert_eq!(query_vectors[0].len(), 128);
    println!(
        "\n  {} Dataset: {} base, {} queries, {} GT entries",
        style("OK").green().bold(),
        base_vectors.len(),
        query_vectors.len(),
        groundtruth.len()
    );

    let k = 10;
    let mut all_results: Vec<ScenarioResult> = Vec::new();

    let balanced_cos = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 200,
        ef_search: 100,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
    };
    let high_recall_cos = HnswConfig {
        m: 32,
        m_max0: 64,
        ef_construction: 400,
        ef_search: 200,
        ml: 1.0 / (32_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
    };
    let balanced_l2 = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 200,
        ef_search: 100,
        ml: 1.0 / (16_f64).ln(),
        distance_metric: DistanceMetric::Euclidean,
    };
    let high_recall_l2 = HnswConfig {
        m: 32,
        m_max0: 64,
        ef_construction: 400,
        ef_search: 200,
        ml: 1.0 / (32_f64).ln(),
        distance_metric: DistanceMetric::Euclidean,
    };

    for &scale in &[10_000usize, 100_000] {
        let scale_base = &base_vectors[..scale];

        let scenarios: Vec<(&str, ScenarioClass, HnswConfig)> = vec![
            (
                "Balanced Cos",
                ScenarioClass::ProductCosine,
                balanced_cos.clone(),
            ),
            (
                "High Recall Cos",
                ScenarioClass::ProductCosine,
                high_recall_cos.clone(),
            ),
            ("Balanced L2", ScenarioClass::StressL2, balanced_l2.clone()),
            (
                "High Recall L2",
                ScenarioClass::StressL2,
                high_recall_l2.clone(),
            ),
        ];

        for (config_name, class, config) in scenarios {
            let block_name = format!("SIFT {}K — {}", scale / 1000, config_name);
            let (index, build_secs) = harness.execute(&block_name, || {
                build_in_memory_index(&config, scale_base, scale)
            });

            let recall = calculate_recall(&index, &query_vectors, &groundtruth, k);
            let (p50, p95, p99, qps) = measure_latency(&index, &query_vectors, k);

            all_results.push(ScenarioResult {
                scale,
                config_name: config_name.to_string(),
                class,
                recall,
                p50_us: p50,
                _p95_us: p95,
                p99_us: p99,
                qps,
                build_secs,
            });
        }

        if scale == 100_000 {
            let tmp = TempDir::new().expect("temp dir for mmap benchmark");
            let mmap_path = tmp.path().join("sift_100k_mmap.bin");
            let block_name = "SIFT 100K — High Recall L2 Mmap";
            let (index, build_secs) = harness.execute(block_name, || {
                build_mmap_index(&high_recall_l2, scale_base, scale, &mmap_path)
            });

            let recall = calculate_recall(&index, &query_vectors, &groundtruth, k);
            let (p50, p95, p99, qps) = measure_latency(&index, &query_vectors, k);

            all_results.push(ScenarioResult {
                scale,
                config_name: "High Recall L2 Mmap".to_string(),
                class: ScenarioClass::StressL2Mmap,
                recall,
                p50_us: p50,
                _p95_us: p95,
                p99_us: p99,
                qps,
                build_secs,
            });
        }
    }

    println!("\n");
    TerminalReporter::block_header("SIFT1M BENCHMARK RESULTS");

    println!(
        "  {}",
        style("╭──────────┬──────────────────┬───────────────┬──────────┬────────────┬────────────┬────────────┬──────────╮").dim()
    );
    println!(
        "  {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        style("│").dim(),
        style(" Scale   ").bold(),
        style("│").dim(),
        style("    Config      ").bold(),
        style("│").dim(),
        style("   Class    ").bold(),
        style("│").dim(),
        style("Recall@10").bold(),
        style("│").dim(),
        style(" p50 (µs)  ").bold(),
        style("│").dim(),
        style(" p99 (µs)  ").bold(),
        style("│").dim(),
        style("   QPS     ").bold(),
        style("│").dim(),
        style("Build(s) ").bold(),
        style("│").dim(),
    );
    println!(
        "  {}",
        style("├──────────┼──────────────────┼───────────────┼──────────┼────────────┼────────────┼────────────┼──────────┤").dim()
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
            "  {} {:>7}K {} {:^16} {} {:^13} {} {} {} {:>9.1} {} {:>9.1} {} {:>9.0} {} {:>7.1} {}",
            style("│").dim(),
            r.scale / 1000,
            style("│").dim(),
            r.config_name,
            style("│").dim(),
            r.class.label(),
            style("│").dim(),
            recall_styled,
            style("│").dim(),
            r.p50_us,
            style("│").dim(),
            r.p99_us,
            style("│").dim(),
            r.qps,
            style("│").dim(),
            r.build_secs,
            style("│").dim(),
        );
    }

    println!(
        "  {}",
        style("╰──────────┴──────────────────┴───────────────┴──────────┴────────────┴────────────┴────────────┴──────────╯").dim()
    );

    println!(
        "\n  {} product-cosine: shipped Cosine metric (SIFT L2 GT not comparable)",
        style("i").blue()
    );
    println!(
        "  {} stress-l2 / stress-l2-mmap: honest L2 path against SIFT ground truth",
        style("i").blue()
    );
    println!(
        "  {} Run only with: cargo test --test competitive_bench --release -- --nocapture",
        style("i").blue()
    );

    for r in &all_results {
        assert!(r.recall > 0.0, "Zero recall indicates a broken search path");
        assert!(r.qps > 0.0, "Zero QPS indicates search is hanging");
    }

    TerminalReporter::success("SIFT1M benchmark completed.");
}
