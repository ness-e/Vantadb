//! Native (host) tests for the STRAT-04 threads kernel.
//!
//! These run with std on the host (`cargo test`) — the kernel logic, the
//! lock-free chunk claim and the raw-pointer FFI surface are all exercised
//! with REAL threads via `std::thread::scope` (host atomics are real atomics).

use super::*;

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

/// Independent reference cosine (std float math, single-threaded).
fn reference_cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 0.0 && nb > 0.0 {
        dot / (na * nb)
    } else {
        0.0
    }
}

/// Deterministic pseudo-random vectors in [-1, 1) — no rand dependency.
fn deterministic_vectors(count: usize, dims: usize) -> Vec<f32> {
    (0..count * dims)
        .map(|i| {
            let x = (i as u64).wrapping_mul(2_654_435_761) % 4096;
            (x as f32 / 2048.0) - 1.0
        })
        .collect()
}

fn deterministic_query(dims: usize) -> Vec<f32> {
    (0..dims).map(|i| ((i as f32) * 0.37).cos() * 0.5).collect()
}

/// Run the exported kernel from a single caller over the whole arena.
fn score_single(vectors: &[f32], count: u32, dims: u32, query: &[f32], chunk: u32) -> Vec<f32> {
    let mut scores = vec![f32::NAN; count as usize];
    let cursor = AtomicU32::new(0);
    let status = unsafe {
        score_batch_chunked(
            vectors.as_ptr(),
            count,
            dims,
            query.as_ptr(),
            scores.as_mut_ptr(),
            cursor.as_ptr(),
            chunk,
        )
    };
    assert_eq!(status, STATUS_OK, "kernel returned status {status}");
    scores
}

#[test]
fn cosine_identical_vectors_scores_one() {
    let v = [0.1f32, -0.5, 0.9, 0.3];
    assert!(approx(cosine_similarity(&v, &v), 1.0, 1e-6));
}

#[test]
fn cosine_orthogonal_vectors_scores_zero() {
    let a = [1.0f32, 0.0];
    let b = [0.0f32, 1.0];
    assert!(approx(cosine_similarity(&a, &b), 0.0, 1e-6));
}

#[test]
fn cosine_zero_vector_scores_zero_without_nan() {
    let a = [0.0f32, 0.0];
    let b = [1.0f32, 2.0];
    let s = cosine_similarity(&a, &b);
    assert_eq!(s, 0.0);
    assert!(s.is_finite());
}

#[test]
fn score_batch_chunked_matches_reference_scores() {
    let dims = 16u32;
    let count = 257u32; // deliberately not a multiple of the chunk size
    let vectors = deterministic_vectors(count as usize, dims as usize);
    let query = deterministic_query(dims as usize);

    let scores = score_single(&vectors, count, dims, &query, 7);

    for i in 0..count as usize {
        let v = &vectors[i * dims as usize..(i + 1) * dims as usize];
        let expected = reference_cosine(v, &query);
        assert!(
            approx(scores[i], expected, 1e-5),
            "index {i}: kernel {} vs reference {expected}",
            scores[i]
        );
    }
}

#[test]
fn score_batch_chunked_is_deterministic_across_chunk_sizes() {
    let dims = 8u32;
    let count = 1000u32;
    let vectors = deterministic_vectors(count as usize, dims as usize);
    let query = deterministic_query(dims as usize);

    let chunk_1 = score_single(&vectors, count, dims, &query, 1);
    let chunk_64 = score_single(&vectors, count, dims, &query, 64);
    let chunk_4096 = score_single(&vectors, count, dims, &query, 4096); // > count

    for i in 0..count as usize {
        assert!(approx(chunk_1[i], chunk_64[i], 1e-6), "index {i}");
        assert!(approx(chunk_1[i], chunk_4096[i], 1e-6), "index {i}");
    }
}

#[test]
fn score_batch_chunked_concurrent_callers_write_every_index_exactly_once() {
    let dims = 8u32;
    let count = 4096u32;
    let vectors = deterministic_vectors(count as usize, dims as usize);
    let query = deterministic_query(dims as usize);
    let mut scores = vec![f32::NAN; count as usize];
    let cursor = AtomicU32::new(0);

    // Raw pointers are not `Send`; smuggle them as usize for scoped threads.
    let vectors_ptr = vectors.as_ptr() as usize;
    let query_ptr = query.as_ptr() as usize;
    let scores_ptr = scores.as_mut_ptr() as usize;
    let cursor_ptr = cursor.as_ptr() as usize;

    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(move || {
                let status = unsafe {
                    score_batch_chunked(
                        vectors_ptr as *const f32,
                        count,
                        dims,
                        query_ptr as *const f32,
                        scores_ptr as *mut f32,
                        cursor_ptr as *mut u32,
                        7,
                    )
                };
                assert_eq!(status, STATUS_OK);
            });
        }
    });

    // Every slot was written (a skipped chunk would leave NaN behind).
    assert!(
        scores.iter().all(|s| !s.is_nan()),
        "some indices were never scored — a chunk was lost or double-claimed"
    );
    // Values match the reference.
    for i in 0..count as usize {
        let v = &vectors[i * dims as usize..(i + 1) * dims as usize];
        let expected = reference_cosine(v, &query);
        assert!(
            approx(scores[i], expected, 1e-5),
            "index {i}: kernel {} vs reference {expected}",
            scores[i]
        );
    }
    // The shared cursor advanced past the batch (claim exhaustion).
    assert!(cursor.load(std::sync::atomic::Ordering::Relaxed) >= count);
}

#[test]
fn score_batch_chunked_rejects_null_pointers() {
    let dims = 4u32;
    let count = 4u32;
    let vectors = deterministic_vectors(count as usize, dims as usize);
    let query = deterministic_query(dims as usize);
    let mut scores = vec![0.0f32; count as usize];
    let cursor = AtomicU32::new(0);

    let status = unsafe {
        score_batch_chunked(
            core::ptr::null(),
            count,
            dims,
            query.as_ptr(),
            scores.as_mut_ptr(),
            cursor.as_ptr(),
            2,
        )
    };
    assert_eq!(status, STATUS_NULL);
    // No partial work happened (nothing was dereferenced).
    assert!(scores.iter().all(|s| *s == 0.0));

    let status = unsafe {
        score_batch_chunked(
            vectors.as_ptr(),
            count,
            dims,
            core::ptr::null(),
            scores.as_mut_ptr(),
            cursor.as_ptr(),
            2,
        )
    };
    assert_eq!(status, STATUS_NULL);
}

#[test]
fn score_batch_chunked_rejects_zero_dims_and_zero_chunk() {
    let dims = 4u32;
    let count = 4u32;
    let vectors = deterministic_vectors(count as usize, dims as usize);
    let query = deterministic_query(dims as usize);
    let mut scores = vec![f32::NAN; count as usize];
    let cursor = AtomicU32::new(0);

    let status = unsafe {
        score_batch_chunked(
            vectors.as_ptr(),
            count,
            0,
            query.as_ptr(),
            scores.as_mut_ptr(),
            cursor.as_ptr(),
            2,
        )
    };
    assert_eq!(status, STATUS_BAD_ARGS);

    let status = unsafe {
        score_batch_chunked(
            vectors.as_ptr(),
            count,
            dims,
            query.as_ptr(),
            scores.as_mut_ptr(),
            cursor.as_ptr(),
            0,
        )
    };
    assert_eq!(status, STATUS_BAD_ARGS);
    // Rejected calls performed no work.
    assert!(scores.iter().all(|s| s.is_nan()));
}
