#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Benchmark: `purge_expired` — full-scan vs TTL-index candidate selection.
//!
//! MOD-04: measures `VantaEmbedded::purge_expired()` on a dataset of N records
//! where `expired` of them carry a TTL already past (should be purged) and the
//! rest are live (no TTL / not expired). All records carry a dense vector so the
//! baseline full-scan (`scan_nodes` clones every vector) is representative.
//!
//! The BEFORE run (baseline) exercises the O(N) full-scan implementation; the
//! AFTER run exercises the selective scalar-index candidate path. Compare
//! medians with `cargo bench --bench purge_expired`.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use vantadb::config::VantaConfig;
use vantadb::storage::BackendKind;
use vantadb::{VantaEmbedded, VantaMemoryInput};

const DIM: usize = 128;
// Dataset shapes: (total records, expired records)
const SHAPES: &[(usize, usize)] = &[(4_000, 100), (4_000, 1_000)];

fn build_db(total: usize, expired: usize) -> VantaEmbedded {
    let config = VantaConfig {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open in-memory bench db");

    let mut rng = StdRng::seed_from_u64(42);
    for i in 0..total {
        let mut input =
            VantaMemoryInput::new("bench/purge", format!("rec-{i:06}"), format!("rec{i}"));
        // Dense vector so the baseline full-scan pays vector-clone cost per node.
        let vec: Vec<f32> = (0..DIM).map(|_| rng.random::<f32>()).collect();
        input.vector = Some(vec);
        // `expired` of the records get a 1ms TTL; building the fixture takes
        // well over 1ms, so by the time `purge_expired` runs they have lapsed.
        if i < expired {
            input.ttl_ms = Some(1);
        }
        db.put(input).expect("put bench record");
    }
    db
}

fn bench_purge_expired(c: &mut Criterion) {
    let mut group = c.benchmark_group("purge_expired");
    for &(total, expired) in SHAPES {
        group.bench_function(format!("total_{total}_expired_{expired}"), |b| {
            b.iter_batched(
                || build_db(total, expired),
                |db| {
                    let n = black_box(db.purge_expired().expect("purge_expired"));
                    assert_eq!(n, expired as u64, "should purge exactly `expired` records");
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, bench_purge_expired);
criterion_main!(benches);
