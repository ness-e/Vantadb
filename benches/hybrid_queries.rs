use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::TempDir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaMemorySearchRequest, VantaValue,
};

struct HybridBenchFixture {
    _dir: TempDir,
    db: VantaEmbedded,
}

fn bench_input(key: &str, payload: String, vector: Vec<f32>, category: &str) -> VantaMemoryInput {
    let mut input = VantaMemoryInput::new("bench/main", key, payload);
    input.vector = Some(vector);
    input.metadata.insert(
        "category".to_string(),
        VantaValue::String(category.to_string()),
    );
    input
}

fn build_fixture() -> HybridBenchFixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open bench db");

    for index in 0..96 {
        let payload = if index % 6 == 0 {
            format!("alpha fused retrieval candidate {index}")
        } else if index % 3 == 0 {
            format!("alpha lexical only note {index}")
        } else {
            format!("background memory record {index}")
        };
        let vector = if index % 6 == 0 {
            vec![1.0, 0.0, 0.0, 0.0]
        } else if index % 2 == 0 {
            vec![0.8, 0.2, 0.0, 0.0]
        } else {
            vec![0.0, 1.0, 0.0, 0.0]
        };
        let category = if index % 4 == 0 { "keep" } else { "other" };
        db.put(bench_input(
            &format!("doc-{index:03}"),
            payload,
            vector,
            category,
        ))
        .expect("put bench record");
    }
    db.rebuild_index().expect("rebuild bench indexes");

    HybridBenchFixture { _dir: dir, db }
}

fn keep_filter() -> VantaMemoryMetadata {
    let mut filters = VantaMemoryMetadata::new();
    filters.insert(
        "category".to_string(),
        VantaValue::String("keep".to_string()),
    );
    filters
}

fn bench_memory_retrieval_modes(c: &mut Criterion) {
    let fixture = build_fixture();

    c.bench_function("memory text-only bm25 filtered", |b| {
        b.iter(|| {
            let hits = fixture
                .db
                .search(VantaMemorySearchRequest {
                    namespace: "bench/main".to_string(),
                    query_vector: Vec::new(),
                    filters: keep_filter(),
                    text_query: Some("alpha retrieval".to_string()),
                    top_k: 10,
                })
                .expect("text search");
            black_box(hits);
        })
    });

    c.bench_function("memory vector-only filtered", |b| {
        b.iter(|| {
            let hits = fixture
                .db
                .search(VantaMemorySearchRequest {
                    namespace: "bench/main".to_string(),
                    query_vector: vec![1.0, 0.0, 0.0, 0.0],
                    filters: keep_filter(),
                    text_query: None,
                    top_k: 10,
                })
                .expect("vector search");
            black_box(hits);
        })
    });

    c.bench_function("memory hybrid rrf filtered", |b| {
        b.iter(|| {
            let hits = fixture
                .db
                .search(VantaMemorySearchRequest {
                    namespace: "bench/main".to_string(),
                    query_vector: vec![1.0, 0.0, 0.0, 0.0],
                    filters: keep_filter(),
                    text_query: Some("alpha retrieval".to_string()),
                    top_k: 10,
                })
                .expect("hybrid search");
            black_box(hits);
        })
    });
}

criterion_group!(benches, bench_memory_retrieval_modes);
criterion_main!(benches);
