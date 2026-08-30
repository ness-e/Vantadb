//! FIND-24 — Cursor-paginated `list` performance regression bench.
//!
//! Compares the per-call latency of `VantaEmbedded::list(limit=N, cursor=K)`
//! over a 10k-record namespace, for both the first page (cursor=0) and a
//! deep cursor (cursor=9000). The fix should make both pages O(limit) instead
//! of O(namespace_size): deep-page latency must not grow linearly with
//! cursor.
//!
//! Run: `cargo bench -p vantadb --bench list_window --features server -- --quick`
//!
//! Baseline (pre-FIND-24, debug build): `list(limit=100, cursor=9000)` was
//! ~6.7 ms/node × 100 nodes ≈ 670 ms per page (because the prefix scan
//! loaded all 10k IDs then sliced).
//! Target (post-FIND-24, this bench): `list(limit=100, cursor=9000)` should
//! stay within ~2× of `list(limit=100, cursor=0)` — both pages do O(limit)
//! work, only the prefix-scan offset differs.
//!
//! NOTE: timings are only meaningful in `--release` mode. CI uses debug for
//! correctness; perf numbers here are qualitative.

use criterion::{criterion_group, criterion_main, Criterion};
use vantadb::config::VantaConfig;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions};

const TOTAL_RECORDS: usize = 10_000;
const PAGE_SIZE: usize = 100;
const DEEP_CURSOR: usize = 9_000;

fn build_db_with_n_records(total: usize) -> VantaEmbedded {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().into_owned(),
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open db");

    for i in 0..total {
        db.put(VantaMemoryInput::new(
            "list-window-bench",
            format!("k-{i:06}"),
            format!("payload {i}"),
        ))
        .expect("put");
    }

    db
}

fn bench_list_window(c: &mut Criterion) {
    let db = build_db_with_n_records(TOTAL_RECORDS);
    let ns = "list-window-bench";

    // First page — exercises cursor=0 path (still dedup-aware).
    c.bench_function("list/limit_100/cursor_0", |b| {
        b.iter(|| {
            let page = db
                .list(
                    ns,
                    VantaMemoryListOptions {
                        limit: PAGE_SIZE,
                        cursor: Some(0),
                        ..Default::default()
                    },
                )
                .expect("list");
            assert_eq!(page.records.len(), PAGE_SIZE);
            page
        })
    });

    // Deep page — the FIND-24 hot path: cursor=9_000 should NOT require
    // scanning the first 9_000 IDs in memory.
    c.bench_function("list/limit_100/cursor_9000", |b| {
        b.iter(|| {
            let page = db
                .list(
                    ns,
                    VantaMemoryListOptions {
                        limit: PAGE_SIZE,
                        cursor: Some(DEEP_CURSOR),
                        ..Default::default()
                    },
                )
                .expect("list");
            assert_eq!(page.records.len(), PAGE_SIZE);
            page
        })
    });
}

criterion_group!(benches, bench_list_window);
criterion_main!(benches);
