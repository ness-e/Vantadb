#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Hybrid retrieval modes over REAL text (BEIR-SciFact).
//!
//! Corpus: first 1000 SciFact docs; queries: first 50 test queries.
//! Vectors are deterministic random (128d, normalized) — the vector leg
//! measures plumbing, the text leg measures REAL BM25 recall via qrels
//! (computed once, untimed). Category filter keeps the filtered path.
//!
//! Data: data/benchmark/beir-scifact/ (scripts/download_ground_truth.py).
//! Run: cargo bench --bench hybrid_queries

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use tempfile::TempDir;
use vantadb::{Embedded, MemoryInput, MemoryMetadata, MemorySearchRequest, Value};

mod common;

const N_DOCS: usize = 1000;
const N_QUERIES: usize = 50;
const TOP_K: usize = 10;
const DIM: usize = 128;

struct HybridBenchFixture {
    _dir: TempDir,
    db: Embedded,
}

fn bench_input(key: &str, payload: String, vector: Vec<f32>, category: &str) -> MemoryInput {
    let mut input = MemoryInput::new("bench/main", key, payload);
    input.vector = Some(vector);
    input
        .metadata
        .insert("category".to_string(), Value::String(category.to_string()));
    input
}

fn read_jsonl(path: &std::path::Path) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path).expect("read jsonl");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse jsonl"))
        .collect()
}

fn build_fixture() -> (
    HybridBenchFixture,
    Vec<(String, String)>,
    HashMap<String, HashSet<String>>,
) {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/benchmark/beir-scifact");
    let corpus = read_jsonl(&base.join("corpus.jsonl"));
    let queries_all = read_jsonl(&base.join("queries.jsonl"));
    // qrels: query-id \t corpus-id \t score (skip header; score > 0 relevant).
    // NOTE: keep only judgments whose doc is inside our indexed subset —
    // recall is measured against what the bench indexed, not the full corpus.
    let mut subset_ids = std::collections::HashSet::new();
    for doc in corpus.iter().take(N_DOCS) {
        subset_ids.insert(doc["_id"].as_str().expect("doc _id").to_string());
    }
    let mut qrels: HashMap<String, HashSet<String>> = HashMap::new();
    for line in std::fs::read_to_string(base.join("qrels.tsv"))
        .expect("read qrels")
        .lines()
        .skip(1)
    {
        let mut parts = line.split('\t');
        if let (Some(q), Some(d), Some(s)) = (parts.next(), parts.next(), parts.next()) {
            if s.trim().parse::<i32>().unwrap_or(0) > 0 && subset_ids.contains(d) {
                qrels
                    .entry(q.to_string())
                    .or_default()
                    .insert(d.to_string());
            }
        }
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open bench db");
    let mut rng = StdRng::seed_from_u64(7);

    for (i, doc) in corpus.iter().take(N_DOCS).enumerate() {
        let id = doc["_id"].as_str().expect("doc _id").to_string();
        let text = format!(
            "{} {}",
            doc.get("title").and_then(|v| v.as_str()).unwrap_or(""),
            doc.get("text").and_then(|v| v.as_str()).unwrap_or("")
        );
        let mut vec: Vec<f32> = (0..DIM).map(|_| rng.random_range(-1.0..1.0)).collect();
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        let category = if i % 2 == 0 { "keep" } else { "other" };
        db.put(bench_input(&id, text, vec, category))
            .expect("put bench record");
    }
    db.rebuild_index().expect("rebuild bench indexes");

    let queries: Vec<(String, String)> = queries_all
        .iter()
        .filter_map(|q| {
            let qid = q["_id"].as_str()?;
            // Only queries with judged relevant docs inside our subset:
            // queries.jsonl and qrels/test.tsv cover different splits.
            let rel = qrels.get(qid)?;
            if !rel.is_empty() {
                Some((
                    qid.to_string(),
                    q["text"].as_str().expect("query text").to_string(),
                ))
            } else {
                None
            }
        })
        .take(N_QUERIES)
        .collect();
    assert!(
        !queries.is_empty(),
        "no queries with judged docs in the subset — check qrels overlap"
    );
    (HybridBenchFixture { _dir: dir, db }, queries, qrels)
}

fn keep_filter() -> MemoryMetadata {
    let mut filters = MemoryMetadata::new();
    filters.insert("category".to_string(), Value::String("keep".to_string()));
    filters
}

fn bench_memory_retrieval_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_queries");
    common::apply_fixed_profile(&mut group);
    let (fixture, queries, qrels) = build_fixture();
    println!(
        "hybrid_queries dataset: beir-scifact subset ({} docs, {} queries + qrels)",
        N_DOCS,
        queries.len()
    );

    // Text-only recall vs qrels (untimed): the real BM25 quality signal.
    let mut hits = 0usize;
    let mut total = 0usize;
    for (qid, text) in &queries {
        if let Some(rel) = qrels.get(qid) {
            let got: Vec<String> = fixture
                .db
                .search(MemorySearchRequest {
                    namespace: "bench/main".to_string(),
                    query_vector: Vec::new(),
                    filters: MemoryMetadata::new(),
                    text_query: Some(text.clone()),
                    top_k: TOP_K,
                    range: None,
                    group_by: None,
                    mmr: None,
                    cursor: None,
                    ..Default::default()
                })
                .expect("text search")
                .iter()
                .map(|h| h.record.key.clone())
                .collect();
            total += rel.len();
            hits += rel.iter().filter(|id| got.contains(id)).count();
        }
    }
    if total > 0 {
        println!(
            "hybrid text-only recall@{TOP_K} vs qrels: {:.4} ({hits}/{total})",
            hits as f64 / total as f64
        );
    } else {
        println!("hybrid text-only recall: no qrels overlap for the query subset");
    }

    group.bench_function("memory text-only bm25 filtered", |b| {
        b.iter(|| {
            for (_, text) in &queries {
                let hits = fixture
                    .db
                    .search(MemorySearchRequest {
                        namespace: "bench/main".to_string(),
                        query_vector: Vec::new(),
                        filters: keep_filter(),
                        text_query: Some(text.clone()),
                        top_k: 10,
                        range: None,
                        group_by: None,
                        mmr: None,
                        cursor: None,
                        ..Default::default()
                    })
                    .expect("text search");
                black_box(hits);
            }
        })
    });

    group.bench_function("memory vector-only filtered", |b| {
        b.iter(|| {
            for (_, text) in &queries {
                let _ = black_box(text);
                let hits = fixture
                    .db
                    .search(MemorySearchRequest {
                        namespace: "bench/main".to_string(),
                        query_vector: vec![1.0; DIM],
                        filters: keep_filter(),
                        text_query: None,
                        top_k: 10,
                        range: None,
                        group_by: None,
                        mmr: None,
                        cursor: None,
                        ..Default::default()
                    })
                    .expect("vector search");
                black_box(hits);
            }
        })
    });

    group.bench_function("memory hybrid rrf filtered", |b| {
        b.iter(|| {
            for (_, text) in &queries {
                let hits = fixture
                    .db
                    .search(MemorySearchRequest {
                        namespace: "bench/main".to_string(),
                        query_vector: vec![1.0; DIM],
                        filters: keep_filter(),
                        text_query: Some(text.clone()),
                        top_k: 10,
                        range: None,
                        group_by: None,
                        mmr: None,
                        cursor: None,
                        ..Default::default()
                    })
                    .expect("hybrid search");
                black_box(hits);
            }
        })
    });
}

criterion_group!(benches, bench_memory_retrieval_modes);
criterion_main!(benches);
