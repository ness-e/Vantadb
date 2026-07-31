//! A4+A3: Parametric HNSW sweep over real datasets.
//!
//! Loads pre-downloaded datasets from `data/benchmark/{name}/`, sweeps
//! M / ef_construction / ef_search, reports build(s), QPS, recall@10,
//! p50(µs), p99(µs).
//!
//! Run: cargo bench --bench param_sweep
//!
//! Data prepared by scripts/download_ground_truth.py.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;
use vantadb::index::{auto_tune::AutoTune, CPIndex, HnswConfig, IndexType, VectorRepresentations};
use vantadb::node::{DistanceMetric, FilterBitset};

// ── Sweep ranges ──────────────────────────────────────────────────────

const M_VALUES: &[usize] = &[8, 12, 16, 24, 32];
const EF_C_VALUES: &[usize] = &[50, 100, 200, 400];
const EF_S_VALUES: &[usize] = &[10, 20, 50, 100, 200, 400];

// ── Data loading ──────────────────────────────────────────────────────

fn load_f32(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).unwrap_or_else(|_| panic!("missing {path}"));
    assert_eq!(bytes.len() % 4, 0);
    let n = bytes.len() / 4;
    let mut v = vec![0.0f32; n];
    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        v[i] = f32::from_le_bytes(chunk.try_into().unwrap());
    }
    v
}

struct Dataset {
    train: Vec<f32>,
    test: Vec<f32>,
    dims: usize,
    n_train: usize,
    n_queries: usize,
    k: usize,
    metric: DistanceMetric,
    /// Brute-force ground truth computed at load time: [n_queries][k] of train indices
    ground_truth: Vec<Vec<u128>>,
}

fn load_dataset(name: &str) -> Dataset {
    let base = PathBuf::from("data/benchmark").join(name);
    let meta_s = std::fs::read_to_string(base.join("meta.json")).unwrap();

    let dims = json_int(&meta_s, "\"dims\"");
    let n_train = json_int(&meta_s, "\"n_train\"");
    let n_queries = json_int(&meta_s, "\"n_test\"");
    let k = json_int(&meta_s, "\"k_ground_truth\"");
    let metric_str = json_str(&meta_s, "\"metric\"");
    let metric = match metric_str {
        "euclidean" => DistanceMetric::Euclidean,
        "angular" | "cosine" => DistanceMetric::Cosine,
        _ => panic!("Unknown metric: {metric_str}"),
    };

    let train = load_f32(&base.join("train.f32").to_string_lossy());
    let test = load_f32(&base.join("test.f32").to_string_lossy());

    // Compute ground truth via brute-force exact search
    println!("  Computing brute-force ground truth for {name} ({n_train}×{dims})...");
    let t0 = Instant::now();
    let ground_truth = compute_brute_force_gt(&train, &test, n_train, n_queries, dims, k, metric);
    println!(
        "  Ground truth computed in {:.1}s",
        t0.elapsed().as_secs_f64()
    );

    Dataset {
        train,
        test,
        dims,
        n_train,
        n_queries,
        k,
        metric,
        ground_truth,
    }
}

fn compute_brute_force_gt(
    train: &[f32],
    test: &[f32],
    n_train: usize,
    n_queries: usize,
    dims: usize,
    k: usize,
    metric: DistanceMetric,
) -> Vec<Vec<u128>> {
    let mut gt = Vec::with_capacity(n_queries);
    for q_idx in 0..n_queries {
        let q = &test[q_idx * dims..(q_idx + 1) * dims];
        let mut scores: Vec<(f32, u128)> = (0..n_train)
            .map(|i| {
                let v = &train[i * dims..(i + 1) * dims];
                let d = match metric {
                    DistanceMetric::Euclidean => -euclidean_sq(q, v),
                    DistanceMetric::Cosine => cosine_sim(q, v),
                };
                (d, i as u128)
            })
            .collect();
        // Sort descending by score (highest similarity = closest)
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        gt.push(scores.into_iter().take(k).map(|(_, id)| id).collect());
    }
    gt
}

fn euclidean_sq(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum()
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum();
    let nb: f32 = b.iter().map(|x| x * x).sum();
    dot / (na * nb).sqrt().max(f32::EPSILON)
}

// ── Minimal JSON helpers ──────────────────────────────────────────────

fn json_int(text: &str, key: &str) -> usize {
    let idx = text
        .find(key)
        .unwrap_or_else(|| panic!("missing key {key}"));
    let after = &text[idx + key.len()..];
    let start = after.find(|c: char| c.is_ascii_digit()).unwrap();
    let end = after[start..]
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len() - start);
    after[start..start + end].parse().unwrap()
}

fn json_str<'a>(text: &'a str, key: &'a str) -> &'a str {
    let idx = text.find(key).unwrap();
    let after = &text[idx + key.len()..];
    let after_colon = &after[after.find(':').unwrap() + 1..];
    let start = after_colon.find('"').unwrap() + 1;
    let end = after_colon[start..].find('"').unwrap();
    &after_colon[start..start + end]
}

// ── Recall computation ────────────────────────────────────────────────

fn recall_at_k(index: &CPIndex, ds: &Dataset) -> f64 {
    let mut hits = 0u64;
    for q_idx in 0..ds.n_queries {
        let query = &ds.test[q_idx * ds.dims..(q_idx + 1) * ds.dims];
        let result: Vec<u128> = index
            .search_nearest(query, None, None, &FilterBitset::all_set(), ds.k, None)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        for &t in &ds.ground_truth[q_idx] {
            if result.contains(&t) {
                hits += 1;
            }
        }
    }
    hits as f64 / (ds.n_queries * ds.k) as f64
}

// ── Build + bench one config ──────────────────────────────────────────

fn build_index(ds: &Dataset, m: usize, ef_c: usize) -> (CPIndex, f64) {
    let idx = CPIndex::new_with_config(HnswConfig {
        m,
        m_max0: m * 2,
        ef_construction: ef_c,
        ef_search: ef_c,
        ml: 1.0 / (m as f64).ln(),
        distance_metric: ds.metric,
        flat_threshold: None,
        index_type: IndexType::Hnsw,
        auto_tune: false,
    });
    let t0 = Instant::now();
    for i in 0..ds.n_train {
        let vec = &ds.train[i * ds.dims..(i + 1) * ds.dims];
        idx.add(
            i as u128,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec.to_vec()),
            0,
        );
    }
    let build_s = t0.elapsed().as_secs_f64();
    (idx, build_s)
}

fn bench_one(ef_s: usize, idx: &mut CPIndex, ds: &Dataset) -> (f64, f64, f64) {
    idx.config.ef_search = ef_s;

    let mut lats = Vec::with_capacity(ds.n_queries);
    for q_idx in 0..ds.n_queries {
        let query = &ds.test[q_idx * ds.dims..(q_idx + 1) * ds.dims];
        let t = Instant::now();
        let _res =
            black_box(idx.search_nearest(query, None, None, &FilterBitset::all_set(), ds.k, None));
        lats.push(t.elapsed().as_secs_f64() * 1e6); // µs
    }
    lats.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = lats.len();
    let mean = lats.iter().sum::<f64>() / n as f64;
    let p50 = lats[n / 2];
    let qps = if mean > 0.0 { 1_000_000.0 / mean } else { 0.0 };
    let recall = recall_at_k(idx, ds);
    (qps, recall, p50)
}

// ── Sweep runner ──────────────────────────────────────────────────────

fn run_sweep(ds: &Dataset, name: &str) {
    println!(
        "\n  ── {name} (D={}, N={}, Nq={}) ──",
        ds.dims, ds.n_train, ds.n_queries
    );
    println!("  Sweeping M={M_VALUES:?}, efC={EF_C_VALUES:?}, efS={EF_S_VALUES:?}");
    println!();
    println!(
        "  {:<4} {:<5} {:<5} {:>10} {:>10} {:>12} {:>8}",
        "M", "efC", "efS", "build(s)", "QPS", "recall@10", "p50µs"
    );
    println!("  {}", "─".repeat(65));

    for &m in M_VALUES {
        for &ef_c in EF_C_VALUES {
            let (mut idx, build_s) = build_index(ds, m, ef_c);

            for &ef_s in EF_S_VALUES {
                AutoTune::set_ef(1); // bypass auto_tuner
                let (qps, recall, p50) = bench_one(ef_s, &mut idx, ds);
                println!("  {m:<4} {ef_c:<5} {ef_s:<5} {build_s:>10.3} {qps:>10.0} {recall:>12.4} {p50:>8.0}");
            }
        }
    }
    println!();
}

// ── Entry point ───────────────────────────────────────────────────────

fn main() {
    let ds = load_dataset("sift-128");
    run_sweep(&ds, "sift-128");
}
