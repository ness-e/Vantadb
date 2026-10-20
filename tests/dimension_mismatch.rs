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
