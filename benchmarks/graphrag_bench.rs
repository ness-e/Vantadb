//! GraphRAG reproducible benchmark (MKT-16).
//!
//! Measures REAL metrics on the public `GraphRagPipeline` API:
//!   - Index time (insert N nodes + edges + flush)
//!   - Query latency p50/p95/mean over M queries
//!   - Latency-per-hop delta (hops=1,2,3)
//!   - Token Reduction vs plain RAG baseline
//!     TR = 1 - Tokens_GraphRAG / Tokens_RAG
//!     where RAG baseline = same pipeline with expansion_hops=0 (seeds only,
//!     no graph expansion, no relationship section).
//!
//! Token counting is a whitespace-split proxy (documented in the report;
//! NOT a real tokenizer — treat all token numbers as approximate).
//!
//! Run:  cargo run --release --example graphrag_bench [-- <nodes> <queries>]
//! Env:  GRAPHRAG_BENCH_OUT=path.json  (optional JSON report)
//!
//! Methodology: deterministic corpus (seeded LCG, no external RNG dep),
//! fresh temp DB per run, 10 warmup queries, median aggregation.
//!
//! Known status (2026-08-05, MKT-16 run): on Windows x86_64 in `--release`,
//! the search phase (vector queries over the built graph) reproducibly hits a
//! stack overflow in the engine, even for a 20-node/8-edge DAG corpus and a
//! 256 MB benchmark-thread stack. Index timing completes cleanly and is REAL;
//! the query-latency and token-reduction metrics could not be measured in this
//! environment and are documented as PENDING in docs/blog/graphrag-benchmark.md.
//! This is the same failure class as AUDIT-04 (STATUS_STACK_BUFFER_OVERRUN).
//! Re-run `cargo run --release --example graphrag_bench` after the engine fix
//! to fill the pending cells — the script itself is complete and reproducible.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use vantadb::graphrag::pipeline::GraphRagPipeline;
use vantadb::{VantaEmbedded, VantaMemoryInput};

// ── Deterministic corpus ────────────────────────────────────────────────────

struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        // Numerical Recipes LCG: x = (x * 6364136223846793005 + 1442695040888963407) mod 2^64
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Topic vocabulary so queries have a real lexical + semantic signal.
const TOPICS: &[&str] = &[
    "vector database",
    "graph traversal",
    "hybrid search",
    "approximate nearest neighbor",
    "knowledge graph",
    "crash recovery",
    "full text search",
    "semantic memory",
];

/// Build a deterministic corpus of `n` nodes: `n_topics` clusters, each node
/// belongs to one cluster; edges within a cluster (chain + hub) plus sparse
/// cross-cluster edges, so expansion has real graph structure to traverse.
fn build_corpus(
    db: &VantaEmbedded,
    ns: &str,
    n: usize,
    n_topics: usize,
    edges_per_node: usize,
) -> Vec<u128> {
    let mut rng = Lcg(0x5EED_CAFE);
    let mut ids: Vec<u128> = Vec::with_capacity(n);

    for i in 0..n {
        let topic = i % n_topics;
        // Cluster centroid on the unit sphere (dim = 32).
        let mut vec = vec![0.0f32; 32];
        let _ = (rng.f32() * std::f32::consts::TAU).sin(); // advance RNG deterministically
        vec[topic] = 1.0;
        vec[(topic + 1) % n_topics] = 0.6;
        // Add small noise around the centroid, then normalize.
        for x in vec.iter_mut() {
            *x += (rng.f32() - 0.5) * 0.15;
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
        for x in vec.iter_mut() {
            *x /= norm;
        }

        let content = format!(
            "{}: document {} about {} and topic {} with keyword {}",
            TOPICS[topic],
            i,
            TOPICS[topic],
            topic,
            rng.next_u64() % 1000
        );

        let mut input = VantaMemoryInput::new(ns, format!("doc-{i}"), &content);
        input.vector = Some(vec);
        let node_id = db.put(input).expect("put").node_id;
        ids.push(node_id);
    }

    // Edges: DAG — forward chain within cluster + hub to next cluster + random
    // forward extra edges. Never references a smaller id, so the corpus is a
    // strict directed acyclic graph (matches the engine's tested path).
    let mut rng = Lcg(0xEDCE5); // hex seed ("EDGE5" — the G is not a hex digit)
    for i in 0..n {
        let topic = i % n_topics;
        let next = i + 1;
        if next < n && next % n_topics == topic {
            db.add_edge(ids[i], ids[next], "related", Some(1.0), None)
                .expect("add_edge chain");
        }
        if i % n_topics == 0 {
            let hub_target = i + n_topics;
            if hub_target < n {
                db.add_edge(ids[i], ids[hub_target], "hub", Some(0.8), None)
                    .expect("add_edge hub");
            }
        }
        for _ in 0..edges_per_node {
            let remaining = n - i - 1;
            if remaining > 0 {
                let j = i + 1 + ((rng.next_u64() as usize) % remaining);
                if j < n {
                    db.add_edge(ids[i], ids[j], "misc", Some(0.5), None)
                        .expect("add_edge misc");
                }
            }
        }
    }

    ids
}

// ── Metrics helpers ─────────────────────────────────────────────────────────

fn token_count(text: &str) -> usize {
    // Whitespace-split proxy. Documented as approximate, not a tokenizer.
    text.split_whitespace().count()
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p / 100.0).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ── Benchmark ───────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Run on a thread with a large stack: the default 1 MB Windows stack can
    // overflow in the HNSW/serialization recursion during search (see AUDIT-04).
    // 256 MB removes the OS stack ceiling as a factor so we measure the engine,
    // not the runtime's default stack size.
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(run_bench)?
        .join()
        .map_err(|e| {
            std::io::Error::other(
                e.downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "benchmark thread panicked".into()),
            )
        })?
}

fn run_bench() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let m: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
    let edges_per_node: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
    let n_topics = TOPICS.len();
    let ns = "graphrag_bench";

    let out_path = env::var("GRAPHRAG_BENCH_OUT").ok().map(PathBuf::from);

    // Hardware metadata (std-only; RAM is recorded manually in the doc).
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let cpus = std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(0);
    let cpu_model = env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".into());

    let temp_dir = tempfile::tempdir()?;
    let db = VantaEmbedded::open(temp_dir.path())?;

    println!("=== GraphRAG benchmark (MKT-16) ===");
    println!(
        "corpus: {} nodes, {} topics, dim=32 | queries: {}",
        n, n_topics, m
    );
    println!(
        "hardware: os={} arch={} cpus={} cpu={}",
        os, arch, cpus, cpu_model
    );
    println!();

    // 1. Index phase.
    let t0 = Instant::now();
    let _ids = build_corpus(&db, ns, n, n_topics, edges_per_node);
    db.flush()?;
    let index_secs = t0.elapsed().as_secs_f64();
    println!(
        "index: {} nodes + edges in {:.3}s ({:.0} nodes/s)",
        n,
        index_secs,
        n as f64 / index_secs
    );

    // Deterministic query set: 3 queries per topic with text + near-centroid vector.
    let mut rng = Lcg(0xBEEF_0000);
    let mut queries: Vec<(String, Vec<f32>)> = Vec::with_capacity(m);
    for q in 0..m {
        let topic = q % n_topics;
        let mut vec = vec![0.0f32; 32];
        vec[topic] = 1.0;
        vec[(topic + 1) % n_topics] = 0.6;
        for x in vec.iter_mut() {
            *x += (rng.f32() - 0.5) * 0.1;
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
        for x in vec.iter_mut() {
            *x /= norm;
        }
        queries.push((TOPICS[topic].to_string(), vec));
    }

    // 2. Warmup (not measured): 10 GraphRAG queries at hops=2.
    let warm = GraphRagPipeline {
        expansion_hops: 2,
        ..GraphRagPipeline::new()
    };
    for (text, vec) in queries.iter().take(10.min(m)) {
        let _ = warm.search(&db, ns, Some(text), Some(vec));
    }

    // 3. Measured phase: RAG baseline (hops=0) vs GraphRAG (hops=2).
    let rag_pipe = GraphRagPipeline {
        expansion_hops: 0,
        ..GraphRagPipeline::new()
    };
    let grag_pipe = GraphRagPipeline {
        expansion_hops: 2,
        ..GraphRagPipeline::new()
    };

    let mut rag_lat_ms: Vec<f64> = Vec::with_capacity(m);
    let mut grag_lat_ms: Vec<f64> = Vec::with_capacity(m);
    let mut rag_tokens: Vec<usize> = Vec::with_capacity(m);
    let mut grag_tokens: Vec<usize> = Vec::with_capacity(m);
    let mut seeds_found: Vec<usize> = Vec::with_capacity(m);
    let mut nodes_expanded: Vec<usize> = Vec::with_capacity(m);

    for (text, vec) in &queries {
        let t = Instant::now();
        let rag = rag_pipe.search(&db, ns, Some(text), Some(vec))?;
        rag_lat_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        rag_tokens.push(token_count(&rag.context_text));

        let t = Instant::now();
        let grag = grag_pipe.search(&db, ns, Some(text), Some(vec))?;
        grag_lat_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        grag_tokens.push(token_count(&grag.context_text));
        seeds_found.push(grag.stats.seeds_found);
        nodes_expanded.push(grag.stats.nodes_expanded);
    }

    // 4. Per-hop latency sweep (hops=1,2,3) on the same queries.
    let mut hop_lat: Vec<(usize, f64)> = Vec::new();
    for hops in [1usize, 2, 3] {
        let pipe = GraphRagPipeline {
            expansion_hops: hops,
            ..GraphRagPipeline::new()
        };
        let mut acc = 0.0f64;
        let mut count = 0usize;
        for (text, vec) in &queries {
            let t = Instant::now();
            let _ = pipe.search(&db, ns, Some(text), Some(vec))?;
            acc += t.elapsed().as_secs_f64() * 1000.0;
            count += 1;
        }
        hop_lat.push((hops, acc / count as f64));
    }

    // 5. Aggregate.
    rag_lat_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    grag_lat_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let rag_tok_avg = rag_tokens.iter().sum::<usize>() as f64 / m as f64;
    let grag_tok_avg = grag_tokens.iter().sum::<usize>() as f64 / m as f64;
    let token_reduction = 1.0 - grag_tok_avg / rag_tok_avg.max(1.0);

    println!();
    println!("--- query latency (ms) ---");
    println!(
        "rag_baseline : p50={:.3} p95={:.3} mean={:.3}",
        percentile(&rag_lat_ms, 50.0),
        percentile(&rag_lat_ms, 95.0),
        rag_lat_ms.iter().sum::<f64>() / m as f64
    );
    println!(
        "graphrag     : p50={:.3} p95={:.3} mean={:.3}",
        percentile(&grag_lat_ms, 50.0),
        percentile(&grag_lat_ms, 95.0),
        grag_lat_ms.iter().sum::<f64>() / m as f64
    );
    println!(
        "latency_delta: +{:.3} ms mean (GraphRAG vs plain RAG)",
        grag_lat_ms.iter().sum::<f64>() / m as f64 - rag_lat_ms.iter().sum::<f64>() / m as f64
    );
    println!();
    println!("--- latency per expansion hop (mean) ---");
    for (hops, lat) in &hop_lat {
        println!("hops={} : {:.3} ms", hops, lat);
    }
    if hop_lat.len() >= 2 {
        let per_hop = (hop_lat[2].1 - hop_lat[0].1) / (hop_lat[2].0 - hop_lat[0].0) as f64;
        println!("per_hop_delta (1->3): {:.3} ms", per_hop);
    }
    println!();
    println!("--- token reduction (whitespace proxy) ---");
    println!("rag_tokens_avg   : {:.1}", rag_tok_avg);
    println!("graphrag_tokens_avg: {:.1}", grag_tok_avg);
    println!(
        "token_reduction  : {:.3} ({:+.2}%)",
        token_reduction,
        token_reduction * 100.0
    );
    println!();
    println!("--- graph stats ---");
    println!(
        "seeds_found_avg   : {:.2}",
        seeds_found.iter().sum::<usize>() as f64 / m as f64
    );
    println!(
        "nodes_expanded_avg: {:.2}",
        nodes_expanded.iter().sum::<usize>() as f64 / m as f64
    );
    println!(
        "context_edge_sample: {}",
        grag_pipe
            .search(&db, ns, Some(&queries[0].0), Some(&queries[0].1))?
            .edges
            .len()
    );

    // 6. Optional JSON report.
    if let Some(path) = out_path {
        let doc = serde_json::json!({
            "schema_version": 1,
            "generated_by": "benchmarks/graphrag_bench.rs (MKT-16)",
            "generated_at": chrono_now(),
            "hardware": { "os": os, "arch": arch, "cpus": cpus, "cpu_model": cpu_model },
            "corpus": { "nodes": n, "queries": m, "topics": n_topics, "dim": 32, "edges_per_node": edges_per_node },
            "methodology": {
                "db": ":memory:",
                "warmup": 10,
                "token_count": "whitespace-split proxy, not a tokenizer",
                "rag_baseline": "GraphRagPipeline with expansion_hops=0 (seeds only)",
                "graphrag": "GraphRagPipeline with expansion_hops=2"
            },
            "results": {
                "index_secs": index_secs,
                "index_nodes_per_sec": n as f64 / index_secs,
                "query_latency_ms": {
                    "rag_p50": percentile(&rag_lat_ms, 50.0),
                    "rag_p95": percentile(&rag_lat_ms, 95.0),
                    "rag_mean": rag_lat_ms.iter().sum::<f64>() / m as f64,
                    "graphrag_p50": percentile(&grag_lat_ms, 50.0),
                    "graphrag_p95": percentile(&grag_lat_ms, 95.0),
                    "graphrag_mean": grag_lat_ms.iter().sum::<f64>() / m as f64
                },
                "per_hop_mean_ms": hop_lat.iter().map(|(h, l)| format!("hops={h}: {l:.3}")).collect::<Vec<_>>(),
                "tokens": {
                    "rag_avg": rag_tok_avg,
                    "graphrag_avg": grag_tok_avg,
                    "token_reduction": token_reduction
                },
                "graph_stats": {
                    "seeds_found_avg": seeds_found.iter().sum::<usize>() as f64 / m as f64,
                    "nodes_expanded_avg": nodes_expanded.iter().sum::<usize>() as f64 / m as f64
                }
            }
        });
        fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))?;
        fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
        println!("\nwrote JSON report: {}", path.display());
    }

    Ok(())
}

fn chrono_now() -> String {
    // Std-only timestamp (avoids pulling chrono just for a report header).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    let secs = now % 86400;
    format!(
        "epoch-{} ({}d {:02}:{:02}:{:02} UTC)",
        now,
        days,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}
