#![allow(clippy::expect_used, clippy::unwrap_used)]
//! FIND-411 — the vector dimension was never validated.
//!
//! A vector whose length differs from the dimension of the vectors held by
//! the index used to be accepted *and* scored as a **perfect** match: the
//! SIMD distance kernels returned `0.0` for two vectors of different
//! lengths, and `0.0` is the best possible Euclidean score. The mismatched
//! candidate therefore outranked every valid one, and the three index
//! paths (flat scan, HNSW over the vector store, HNSW over the inline
//! vector data) disagreed on the result.
//!
//! The dimension is now validated at the entry points: a write and a query
//! whose length differs from the index dimension are rejected with
//! [`Error::DimensionMismatch`].

use vantadb::{connect, DistanceMetric, Error, MemoryInput, MemorySearchRequest};

fn record(namespace: &str, key: &str, vector: Vec<f32>) -> MemoryInput {
    MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: key.to_string(),
        vector: Some(vector),
        ..Default::default()
    }
}

fn request(namespace: &str, query_vector: Vec<f32>, top_k: usize) -> MemorySearchRequest {
    MemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        distance_metric: DistanceMetric::Euclidean,
        top_k,
        ..Default::default()
    }
}

fn dimension_mismatch(err: &Error) -> (usize, usize) {
    match err {
        Error::DimensionMismatch { expected, got } => (*expected, *got),
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
}

#[test]
fn put_rejects_a_vector_whose_dimension_differs_from_the_index() {
    let db = connect(":memory:").expect("open in-memory db");

    // The first vector establishes the dimension of the index.
    db.put(record("ns", "a", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let err = match db.put(record("ns", "b", vec![1.0, 0.0, 0.0, 0.0, 0.0])) {
        Ok(_) => panic!("a 5d vector must be rejected by a 3d index"),
        Err(err) => err,
    };
    assert_eq!(
        dimension_mismatch(&err),
        (3, 5),
        "the error must carry the expected and the received dimension"
    );
}

#[test]
fn put_batch_rejects_a_vector_whose_dimension_differs_from_the_index() {
    let db = connect(":memory:").expect("open in-memory db");

    db.put(record("ns", "a", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let err = match db.put_batch(vec![
        record("ns", "b", vec![0.0, 1.0, 0.0]),
        record("ns", "c", vec![1.0, 0.0, 0.0, 0.0, 0.0]),
    ]) {
        Ok(_) => panic!("a 5d vector must be rejected by a 3d index"),
        Err(err) => err,
    };
    assert_eq!(dimension_mismatch(&err), (3, 5));

    // The batch is rejected up-front: nothing of the batch was written.
    let hits = db
        .search(request("ns", vec![1.0, 0.0, 0.0], 10))
        .expect("search");
    let keys: Vec<&str> = hits.iter().map(|hit| hit.record.key.as_str()).collect();
    assert_eq!(
        keys,
        vec!["a"],
        "a rejected batch must leave no record behind"
    );
}

#[test]
fn search_rejects_a_query_vector_whose_dimension_differs_from_the_index() {
    let db = connect(":memory:").expect("open in-memory db");

    db.put(record("ns", "a", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let err = match db.search(request("ns", vec![1.0, 0.0, 0.0, 0.0, 0.0], 1)) {
        Ok(hits) => panic!("expected DimensionMismatch, got {} hits", hits.len()),
        Err(err) => err,
    };
    assert_eq!(
        dimension_mismatch(&err),
        (3, 5),
        "the error must carry the expected and the received dimension"
    );
}

#[test]
fn euclidean_dimension_mismatch_ranks_matching_dim_first() {
    // The Trigger of the finding: a 3d record and a 5d record in the same
    // namespace, searched with a 3d query under Euclidean.
    //
    // The 5d write is rejected (it would otherwise be scored as a perfect
    // match and outrank the valid candidate), and the search returns the
    // 3d record.
    let db = connect(":memory:").expect("open in-memory db");

    db.put(record("ns", "a", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let err = match db.put(record("ns", "b", vec![1.0, 0.0, 0.0, 0.0, 0.0])) {
        Ok(_) => panic!("the 5d vector must be rejected"),
        Err(err) => err,
    };
    assert_eq!(dimension_mismatch(&err), (3, 5));

    let hits = db
        .search(request("ns", vec![1.0, 0.0, 0.0], 1))
        .expect("search");
    assert_eq!(hits.len(), 1, "only the 3d record is a candidate");
    assert_eq!(
        hits[0].record.key, "a",
        "the record with the matching dimension must win"
    );
}

/// FIND-411: the score the SDK reports under Euclidean is the **distance**
/// scale, not the squared distance — the flat scan and the HNSW traversal
/// must report the same score for the same candidate. A corpus under
/// `flat_threshold` used to report `-d²` while a bigger one reported `-d`.
#[test]
fn the_sdk_reports_the_euclidean_distance_scale() {
    let db = connect(":memory:").expect("open in-memory db");
    db.put(record("ns", "a", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let hits = db
        .search(request("ns", vec![1.0, 0.0, 0.0], 1))
        .expect("search");
    assert_eq!(hits.len(), 1);
    let score = hits[0].score;
    // The distance between (1,0,0) and (0,1,0) is sqrt(2).
    assert!(
        (score + std::f32::consts::SQRT_2).abs() < 1e-5,
        "the SDK must report the distance scale (-1.4142135), not the squared distance \
         (-2.0), got {score}"
    );
}

/// FIND-411 (O5): `put` and `put_batch` must apply the **same** rule — the
/// dimension of the index is the only authority. `put` used to fall back to
/// the dimension of the record being replaced when the index held no vector,
/// so the same upsert was accepted by one API and rejected by the other.
#[test]
fn put_and_put_batch_apply_the_same_dimension_rule() {
    let db = connect(":memory:").expect("open in-memory db");

    // The first vector of the base establishes its dimension.
    db.put_batch(vec![record("ns", "a", vec![0.0, 1.0, 0.0])])
        .expect("the 3d vector must be accepted");

    // `put` accepts the base dimension and rejects another one.
    db.put(record("ns", "b", vec![1.0, 0.0, 0.0]))
        .expect("a 3d vector must be accepted");
    let err = match db.put(record("ns", "c", vec![1.0, 0.0, 0.0, 0.0, 0.0])) {
        Ok(_) => panic!("a 5d vector must be rejected by a 3d index"),
        Err(err) => err,
    };
    assert_eq!(dimension_mismatch(&err), (3, 5));

    // `put_batch` rejects it exactly the same way.
    let err = match db.put_batch(vec![record("ns", "d", vec![1.0, 0.0, 0.0, 0.0, 0.0])]) {
        Ok(_) => panic!("a 5d vector must be rejected by a 3d index"),
        Err(err) => err,
    };
    assert_eq!(dimension_mismatch(&err), (3, 5));
}

/// FIND-411 (O4): the bulk import validated the dimension **per chunk**, so a
/// mismatch in chunk N left the records 1..N-1 written. The whole payload is
/// validated before any write.
#[test]
fn bulk_import_rejects_a_mismatched_vector_before_writing_anything() {
    let db = connect(":memory:").expect("open in-memory db");
    db.put(record("ns", "seed", vec![0.0, 1.0, 0.0]))
        .expect("the 3d vector must be accepted");

    let inputs = vec![
        record("ns", "k1", vec![0.0, 1.0, 0.0]),
        record("ns", "k2", vec![1.0, 0.0, 0.0, 0.0, 0.0]),
    ];
    let total = inputs.len() as u64;
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"VDBJSON\n");
    payload.push(0x01);
    payload.extend_from_slice(&total.to_le_bytes());
    payload.extend_from_slice(&serde_json::to_vec(&inputs).expect("serialize"));

    let err = match db.bulk_import_stream(&mut payload.as_slice()) {
        Ok(_) => panic!("a 5d vector must be rejected by a 3d index"),
        Err(err) => err,
    };
    assert_eq!(dimension_mismatch(&err), (3, 5));

    // Nothing was written — not even the records that precede the mismatch.
    assert!(
        db.get("ns", "k1").expect("get").is_none(),
        "a rejected bulk import must leave no record behind"
    );
    assert!(
        db.get("ns", "k2").expect("get").is_none(),
        "a rejected bulk import must leave no record behind"
    );
}
