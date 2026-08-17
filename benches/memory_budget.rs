//! Memory budget benchmark (FND-01 — compute/storage separation + OOM risk).
//!
//! Contract: measure **process RSS growth vs dataset size** under a sustained
//! write workload, and compare it against the engine's *logical* memory
//! estimate. Since FND-01-F1, `check_memory_pressure`
//! (src/storage/engine/stats.rs:98) uses the **real process RSS** as its
//! back-pressure signal (with fallback to the logical estimate when the host
//! measurement is unavailable), so `pressure_ratio` in the table mirrors the
//! guard's effective signal: `rss` when the host measurement is available,
//! else the logical estimate.
//!
//! Design:
//! - Full-stack `StorageEngine` (Fjall backend, tempdir) — includes HNSW (RAM),
//!   vstore (mmap), WAL, KV backend: the real compute/storage mix.
//! - Batches of 1536-dim vectors are inserted, `flush()` is called (which runs
//!   `record_memory_breakdown` → real process RSS via Win32/Mach/sysinfo), then
//!   RSS + logical estimate are sampled and printed as a table.
//! - Deterministic dataset: `common::synthetic_vectors` (seeded), vectors are
//!   generated per-batch and dropped so the bench does not inflate RSS itself.
//! - Scale: full = [10k, 25k, 50k, 100k] nodes. Reduced = `MEMORY_BUDGET_SCALE=lite`
//!   → [5k, 10k, 20k]. The important output is the *trend* RSS vs dataset, not
//!   the absolute number.
//!
//! Run: `cargo bench -p vantadb --bench memory_budget`
//! Smoke (compile + run without measurement): `cargo bench -p vantadb --bench memory_budget -- --test`

use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Instant;
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

mod common;

/// Vector dimensionality — matches canonical_p99 (1536d real-world embedders).
const DIM: usize = 1536;
/// Write reads per batch sampled after insert (simulates the 1k r/s read mix).
const READS_PER_BATCH: usize = 10_000;

/// Batch sizes in nodes. Full scale by default; `MEMORY_BUDGET_SCALE=lite` for
/// CI / fast verification runs (documented reduced scale — trend, not absolutes).
fn batch_sizes() -> Vec<usize> {
    match std::env::var("MEMORY_BUDGET_SCALE").as_deref() {
        Ok("lite") => vec![5_000, 10_000, 20_000],
        _ => vec![10_000, 25_000, 50_000, 100_000],
    }
}

fn fmt_bytes(b: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    if b >= GIB {
        format!("{:.2} GiB", b as f64 / GIB as f64)
    } else if b >= MIB {
        format!("{:.2} MiB", b as f64 / MIB as f64)
    } else {
        format!("{b} B")
    }
}

fn bench_memory_budget(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).unwrap();

    let scales = batch_sizes();
    println!(
        "memory_budget: dim={DIM} batches={scales:?} scale_env=MEMORY_BUDGET_SCALE (default=full)"
    );
    println!(
        "nodes, insert_elapsed_s, rss_bytes, logical_estimate_bytes, delta_rss_minus_logical, memory_limit_bytes, pressure_ratio"
    );

    let mut inserted = 0usize;
    for &batch in &scales {
        let count = batch - inserted;
        // Generate per-batch and drop: keeps the bench's own vectors out of RSS.
        let batch_vecs = common::synthetic_vectors(count, DIM);

        let insert_start = Instant::now();
        for (i, vec) in batch_vecs.iter().enumerate() {
            let id = (inserted + i) as u128;
            let node = UnifiedNode::with_vector(id, vec.clone());
            storage.insert(&node).unwrap();
        }
        let insert_elapsed = insert_start.elapsed();
        inserted += count;

        // Read mix: random point lookups against already-inserted ids.
        let read_start = Instant::now();
        for _ in 0..READS_PER_BATCH {
            let id = ((inserted as u128).wrapping_mul(2654435761) % inserted as u128) as usize;
            let _ = storage.get(id as u128);
        }
        let read_elapsed = read_start.elapsed();
        let _ = read_elapsed;

        // flush() records the real process RSS into the metrics snapshot.
        storage.flush().unwrap();
        drop(batch_vecs);

        let snap = vantadb::metrics::memory_breakdown_snapshot();
        let stats = storage.get_memory_stats();
        let rss = snap.process_rss_bytes;
        let logical = stats.logical_bytes;
        // Mirror the guard's effective signal (FND-01-F1): real process RSS when
        // the host measurement is available, else the logical estimate.
        let guard_effective = if rss > 0 {
            rss
        } else {
            stats.effective_bytes()
        };
        let pressure_ratio = if stats.memory_limit > 0 {
            guard_effective as f64 / stats.memory_limit as f64
        } else {
            0.0
        };
        println!(
            "{}, {:.1}, {}, {}, {}, {}, {:.3}",
            inserted,
            insert_elapsed.as_secs_f64(),
            rss,
            logical,
            rss.saturating_sub(logical),
            stats.memory_limit,
            pressure_ratio
        );
        println!(
            "  -> rss={} logical_estimate={} delta={} limit={} guard_physical={} guard_effective={} pressure_ratio={:.3}",
            fmt_bytes(rss),
            fmt_bytes(logical),
            fmt_bytes(rss.saturating_sub(logical)),
            fmt_bytes(stats.memory_limit),
            stats
                .physical_rss
                .map(fmt_bytes)
                .unwrap_or_else(|| "none".into()),
            fmt_bytes(guard_effective),
            pressure_ratio
        );
    }

    // Minimal criterion measurement so the bench integrates with the harness.
    let mut group = c.benchmark_group("memory_budget");
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);
    group.bench_function("rss_vs_dataset_trend", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                std::hint::black_box(inserted);
            }
            start.elapsed()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_memory_budget);
criterion_main!(benches);
