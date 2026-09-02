//! Score semantics micro-bench (RES-05 — follow-up RES-04).
//!
//! Contract: bench pure `f32` helpers in `src/api/scores.rs` —
//! `rrf_contribution`, `cosine_distance_to_*`, relevance.
//! Reuses `canonical_p99` fixed profile (`common::apply_fixed_profile`)
//! for reproducible criterion estimates. Pure O(1), no alloc, inline.
//!
//! Design (ponytail: delegate, no SIMD):
//! - Batch of 10k deterministic distances in [0,2] + ranks in [1,100].
//! - Each bench iterates with `black_box` so optimizer can't elide.
//! - Group profile identical to `canonical_p99` (warmup 3s, measure 5s).

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use vantadb::api::scores::{
    cosine_distance_to_relevance, cosine_distance_to_similarity,
    cosine_distance_to_similarity_clamped, cosine_similarity_to_distance, rrf_contribution,
    rrf_contribution_0based,
};

mod common;

/// Deterministic distances in [0,2] (covers cosine distance domain).
fn gen_distances(n: usize) -> Vec<f32> {
    // xorshift determinístico — sin rand dep extra
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    (0..n)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let v = (state as f64 / u64::MAX as f64) as f32; // [0,1)
            v * 2.0 // [0,2)
        })
        .collect()
}

fn bench_scores_semantics(c: &mut Criterion) {
    let mut group = c.benchmark_group("scores_semantics");
    common::apply_fixed_profile(&mut group);

    // Reuse one batch for all micro-benches — ponytail: no per-bench alloc
    let distances = gen_distances(10_000);
    let ranks: Vec<usize> = (0..10_000).map(|i| (i % 100) + 1).collect();

    group.bench_function("rrf_wire_10k", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &r in &ranks {
                acc += black_box(rrf_contribution(Some(r), None));
            }
            black_box(acc)
        })
    });

    group.bench_function("rrf_0based_10k", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for (i, _) in ranks.iter().enumerate() {
                acc += black_box(rrf_contribution_0based(i % 100, None));
            }
            black_box(acc)
        })
    });

    group.bench_function("cosine_distance_to_similarity_10k", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &d in &distances {
                acc += black_box(cosine_distance_to_similarity(d));
            }
            black_box(acc)
        })
    });

    group.bench_function("cosine_distance_to_similarity_clamped_10k", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &d in &distances {
                acc += black_box(cosine_distance_to_similarity_clamped(d));
            }
            black_box(acc)
        })
    });

    group.bench_function("cosine_similarity_to_distance_10k", |b| {
        // similarity in [-1,1] derived from distances
        let sims: Vec<f32> = distances.iter().map(|d| 1.0 - d).collect();
        b.iter(|| {
            let mut acc = 0.0f32;
            for &s in &sims {
                acc += black_box(cosine_similarity_to_distance(s));
            }
            black_box(acc)
        })
    });

    group.bench_function("cosine_distance_to_relevance_10k", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &d in &distances {
                acc += black_box(cosine_distance_to_relevance(d));
            }
            black_box(acc)
        })
    });

    // ponytail: pure f32 O(1) helpers, batch SIMD if hot path shows in profiling

    group.finish();
}

criterion_group!(benches, bench_scores_semantics);
criterion_main!(benches);
