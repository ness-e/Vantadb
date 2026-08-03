//! NUEVO-18 certification: native sparse vectors + sparse/dense coexistence.

use tempfile::tempdir;
use vantadb::{
    SparseVector, VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemorySearchRequest, VantaValue,
};

fn sparse(pairs: &[(u32, f32)]) -> SparseVector {
    let mut v = SparseVector::new();
    for (dim, value) in pairs {
        v.insert(*dim, *value);
    }
    v
}

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

#[test]
fn sparse_insert_top1_by_sparse_query() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    // Three sparse records with distinct term-weight signatures.
    for (key, terms) in [
        ("sparse-a", vec![(3_u32, 1.0_f32), (7_u32, 0.5_f32)]),
        ("sparse-b", vec![(10_u32, 0.8_f32), (12_u32, 0.3_f32)]),
        ("sparse-c", vec![(3_u32, 0.2_f32)]),
    ] {
        let mut input = VantaMemoryInput::new("agent/main", key, "sparse payload");
        input
            .metadata
            .insert("category".to_string(), field_string("sparse"));
        input.sparse_vector = Some(sparse(&terms));
        db.put(input).expect("put");
    }

    // Query with the exact signature of sparse-a: dot scores are
    // 1.0*1.0 + 0.5*0.5 = 1.25 (a), 0.2*1.0 = 0.2 (c), 0 (b).
    let request = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_sparse: Some(sparse(&[(3, 1.0), (7, 0.5)])),
        top_k: 3,
        ..Default::default()
    };

    let hits = db.search(request).expect("sparse search");
    assert!(
        !hits.is_empty(),
        "sparse query must return the matching record"
    );
    assert_eq!(
        hits[0].record.key, "sparse-a",
        "exact signature must be top-1"
    );
    assert!(hits[0].score > hits[1].score, "scores must be descending");
    assert_eq!(
        hits[1].record.key, "sparse-c",
        "partial overlap ranks second"
    );
}

#[test]
fn sparse_and_dense_coexist() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    // Dense-only record.
    let mut dense = VantaMemoryInput::new("agent/main", "dense-1", "dense payload");
    dense
        .metadata
        .insert("category".to_string(), field_string("dense"));
    dense.vector = Some(vec![1.0, 0.0, 0.0]);
    db.put(dense).expect("put dense");

    // Sparse-only record.
    let mut sparse_rec = VantaMemoryInput::new("agent/main", "sparse-1", "sparse payload");
    sparse_rec
        .metadata
        .insert("category".to_string(), field_string("sparse"));
    sparse_rec.sparse_vector = Some(sparse(&[(5, 1.0)]));
    db.put(sparse_rec).expect("put sparse");

    // Dense-only search still works and finds the dense record.
    let dense_req = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![0.9, 0.1, 0.0],
        top_k: 5,
        ..Default::default()
    };
    let dense_hits = db.search(dense_req).expect("dense search");
    assert!(
        dense_hits.iter().any(|h| h.record.key == "dense-1"),
        "dense path must not regress"
    );

    // Sparse-only search still finds the sparse record.
    let sparse_req = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_sparse: Some(sparse(&[(5, 1.0)])),
        top_k: 5,
        ..Default::default()
    };
    let sparse_hits = db.search(sparse_req).expect("sparse search");
    assert_eq!(sparse_hits[0].record.key, "sparse-1");

    // Sparse + dense together fuse (RRF) without breaking either channel.
    let fused_req = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![1.0, 0.0, 0.0],
        query_sparse: Some(sparse(&[(5, 1.0)])),
        top_k: 5,
        ..Default::default()
    };
    let fused_hits = db.search(fused_req).expect("fused search");
    let keys: Vec<&str> = fused_hits.iter().map(|h| h.record.key.as_str()).collect();
    assert!(
        keys.contains(&"dense-1") && keys.contains(&"sparse-1"),
        "both channels must appear in fused results, got {keys:?}"
    );
}

#[test]
fn sparse_roundtrip_persistence() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("agent/main", "persist-1", "payload");
    input.sparse_vector = Some(sparse(&[(2, 0.25), (9, 1.5)]));
    db.put(input).expect("put");

    let list = db
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list");
    let record = list
        .records
        .iter()
        .find(|r| r.key == "persist-1")
        .expect("record must exist");
    let persisted = record.sparse_vector.as_ref().expect("sparse must persist");
    assert_eq!(persisted.0.get(&2), Some(&0.25));
    assert_eq!(persisted.0.get(&9), Some(&1.5));

    // Reopen the database: sparse vector must survive from disk.
    drop(db);
    let reopened = VantaEmbedded::open(dir.path()).expect("reopen");
    let list2 = reopened
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list2");
    let record2 = list2
        .records
        .iter()
        .find(|r| r.key == "persist-1")
        .expect("record after reopen");
    let persisted2 = record2.sparse_vector.as_ref().expect("sparse after reopen");
    assert_eq!(persisted2.0.get(&2), Some(&0.25));
    assert_eq!(persisted2.0.get(&9), Some(&1.5));
}
