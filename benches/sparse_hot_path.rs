//! AUDIT-02: sparse hot-path micro-opt gate de medición.
//!
//! Atribuye tiempo en el hot-path de búsqueda sparse (inverted index / posting
//! lists, NO full-scan) entre: (a) parse de serialización JSON del sparse por
//! hit, (b) escritura de serialización JSON (write path), (c) sort completo de
//! candidatos vs sort parcial top-k. Dataset realista (~1-2% densidad, query
//! con dims populares que generan muchos candidatos).

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;
use tempfile::TempDir;
use vantadb::{
    SparseVector, VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaMemoryRecord,
    VantaMemorySearchHit, VantaMemorySearchRequest,
};

const VOCAB: u32 = 2000;
const N_DOCS: usize = 5_000;
const NNZ_PER_DOC: usize = 24; // ~1.2% densidad → posting lists realistas
const N_HOT: u32 = 16; // dims compartidas por TODOS los docs → query genera N_DOC candidatos
const TOP_K: usize = 10;

fn build_sparse(rng_seed: usize) -> SparseVector {
    let mut v = SparseVector::new();
    // Dims calientes compartidas (e.g. términos de alta frecuencia).
    for d in 0..N_HOT {
        v.insert(d, (rng_seed as f32 % 5.0) + 0.1);
    }
    // Dims de cola por documento.
    let mut x = (rng_seed as u64).wrapping_mul(0x9E3779B97F4A7C15);
    for _ in N_HOT..NNZ_PER_DOC as u32 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let dim = N_HOT + (x as u32) % (VOCAB - N_HOT);
        v.insert(dim, (x >> 32) as f32 / u32::MAX as f32);
    }
    v
}

struct Fixture {
    _dir: TempDir,
    db: VantaEmbedded,
}

fn build_fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open bench db");
    for i in 0..N_DOCS {
        let mut input = VantaMemoryInput::new(
            "bench/sparse",
            format!("doc-{i:05}"),
            format!("sparse doc number {i}"),
        );
        input.sparse_vector = Some(build_sparse(i));
        db.put(input).expect("put bench record");
    }
    Fixture { _dir: dir, db }
}

fn query_sparse() -> SparseVector {
    let mut q = SparseVector::new();
    for d in 0..N_HOT / 2 {
        q.insert(d, 1.0);
    }
    q
}

// Comparador exacto de crate::planner::sort_hits (orden desc, tie por key, node_id).
fn sort_hits_sim(hits: &mut [VantaMemorySearchHit]) {
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.record.key.cmp(&b.record.key))
            .then(a.record.node_id.cmp(&b.record.node_id))
    });
}

// Propuesta de fix candidato 2: select_nth parcial + sort solo del top-k.
fn sort_top_k(hits: &mut Vec<VantaMemorySearchHit>, k: usize) {
    let k = k.min(hits.len());
    if k == 0 {
        return;
    }
    hits.select_nth_unstable_by(k - 1, |a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.record.key.cmp(&b.record.key))
            .then(a.record.node_id.cmp(&b.record.node_id))
    });
    hits[..k].sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.record.key.cmp(&b.record.key))
            .then(a.record.node_id.cmp(&b.record.node_id))
    });
    hits.truncate(k);
}

fn make_hits(n: usize) -> Vec<VantaMemorySearchHit> {
    (0..n)
        .map(|i| VantaMemorySearchHit {
            record: VantaMemoryRecord {
                namespace: "bench/sparse".into(),
                key: format!("doc-{i:05}"),
                payload: format!("sparse doc number {i}"),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: i as u64,
                updated_at_ms: i as u64,
                version: 1,
                node_id: i as u128,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
            },
            score: ((i % 97) as f32) * 0.01,
            explanation: None,
        })
        .collect()
}

fn bench_sparse_hot_path(c: &mut Criterion) {
    let fixture = build_fixture();

    // Número de candidatos que realmente materializa la query: fuerza sort_hits
    // sobre C candidatos, top_k=10. Lo medimos una vez con top_k grande.
    let req_count = VantaMemorySearchRequest {
        namespace: "bench/sparse".into(),
        query_sparse: Some(query_sparse()),
        top_k: N_DOCS,
        ..Default::default()
    };
    let all = fixture.db.search(req_count).expect("count search");
    let candidate_count = all.len();
    eprintln!("[AUDIT-02] query document candidates materialized = {candidate_count}");

    let mut group = c.benchmark_group("sparse/hot-path");
    group.sample_size(20);
    group.bench_function("search_total_top10", |b| {
        b.iter(|| {
            let req = VantaMemorySearchRequest {
                namespace: "bench/sparse".into(),
                query_sparse: Some(query_sparse()),
                top_k: TOP_K,
                ..Default::default()
            };
            let hits = fixture.db.search(req).expect("sparse search");
            black_box(hits);
        })
    });

    // ── 2. Sort: full sort de todos los candidatos vs top-k parcial ──
    group.bench_function("sort_full_candidates", |b| {
        b.iter_batched(
            || make_hits(candidate_count),
            |mut hits| {
                sort_hits_sim(&mut hits);
                black_box(hits.len());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("sort_topk_partial", |b| {
        b.iter_batched(
            || make_hits(candidate_count),
            |mut hits| {
                sort_top_k(&mut hits, TOP_K);
                black_box(hits.len());
            },
            BatchSize::SmallInput,
        )
    });

    // ── 3. Serialización JSON del sparse (write y parse per-hit) ───────────────
    let sample = build_sparse(12345); // SparseVector realista (NNZ approx)
    let sample_json = serde_json::to_string(&sample).expect("serialize");
    group.bench_function("serialize_write_one", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&sample)).expect("ok")))
    });
    group.bench_function("serialize_read_parse_one", |b| {
        b.iter(|| {
            let v: SparseVector = serde_json::from_str(black_box(&sample_json)).expect("ok");
            black_box(v.len())
        })
    });

    // ── 4. Serialización ListFloat (ADR-019 / P2-7) vs serde_json ─────────────
    // Dataset representativo: lista DENSA de f32 (todos los dims del vocabulario),
    // distinto del sample sparse (24 NNZ) de los arms serde_json de arriba.
    let dense: SparseVector = {
        let mut v = SparseVector::new();
        for d in 0..VOCAB {
            v.insert(d, (d as f32 % 5.0) + 0.1);
        }
        v
    };
    // Los helpers del path ListFloat son privados a `sdk::serialization`
    // (`sparse_vector_to_field` / `sparse_vector_from_field`), así que el bench
    // inlinea la operación exacta que ejecutan (mismo patrón que los arms
    // serde_json inlinean serde_json::to_string/from_str).
    let dense_flat: Vec<f64> = {
        let mut flat = Vec::with_capacity(dense.0.len() * 2);
        for (dim, weight) in &dense.0 {
            flat.push(*dim as f64);
            flat.push(*weight as f64);
        }
        flat
    };
    group.bench_function("listfloat_encode_one", |b| {
        b.iter(|| {
            let mut flat = Vec::with_capacity(dense.0.len() * 2);
            for (dim, weight) in &dense.0 {
                flat.push(*dim as f64);
                flat.push(*weight as f64);
            }
            black_box(flat.len())
        })
    });
    group.bench_function("listfloat_decode_one", |b| {
        b.iter(|| {
            let mut map = std::collections::BTreeMap::new();
            for pair in dense_flat.chunks_exact(2) {
                map.insert(pair[0] as u32, pair[1] as f32);
            }
            black_box(map.len())
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sparse_hot_path);
criterion_main!(benches);
