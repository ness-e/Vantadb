#![allow(clippy::expect_used, clippy::unwrap_used)]
//! At-rest encryption cost benchmark — W7 of the encryption wiring plan
//! (`docs/dev/plans/2026-10-20-wiring-encryption-feature.md`).
//!
//! **Why a new bench and not `benches/canonical_p99.rs`:** `canonical_p99`
//! measures a **pure in-memory** `CPIndex` (no storage I/O at all). With and
//! without the `encryption` feature it would measure the same thing (≈0
//! delta) — it cannot price the at-rest cost. This bench drives the **real
//! `StorageEngine`** (WAL + VantaFile + backend-KV + HNSW image on disk), so
//! the ciphertext envelopes are actually on the write and the read path.
//!
//! **Two runs of the same source; the only difference is the compiled
//! feature** (and, with it, the key):
//!
//! ```powershell
//! cargo bench -p vantadb --bench encrypted_p99                     # plaintext
//! cargo bench -p vantadb --bench encrypted_p99 --features encryption # encrypted
//! ```
//!
//! The master key is set **programmatically** (`Config::encryption_key` — the
//! first of the three key sources of ADR-0057 D2), so the bench never
//! depends on the environment. With the feature compiled out, the field is
//! simply ignored and the database stays plaintext (byte-identical path).
//!
//! **Dataset:** the canonical one — seed 42, 100k × 1536d — drawn exactly
//! like `benches/canonical_p99.rs` (`StdRng::seed_from_u64(42)`), so the
//! numbers are comparable with the canonical baseline. The vectors are
//! streamed (one chunk at a time) instead of held all at once: the values
//! are identical, the RSS is not inflated by the bench itself.
//!
//! **Phases** (each one timed):
//!
//! 1. `insert` — 100k nodes with a 1536d vector into a real engine
//!    (Fjall backend + WAL + vstore, on a tempdir).
//! 2. `search` — 1000 queries through `engine.hnsw` **plus the vstore** (the
//!    engine reads the candidate vectors from the store: a zero-copy mmap
//!    read in plaintext mode, a GCM decrypt per node in encrypted mode).
//! 3. `flush` — `engine.flush()`: the **whole** HNSW image is rewritten (one
//!    AEAD over the whole image when encrypted).
//! 4. `cold` — close + reopen: the whole image is read back and decrypted
//!    (the mmap zero-copy is lost — the declared limit of ADR-0057 §5.2).
//!
//! **Protocol:** 3 runs, median reported (plan §5, W7). The `search` phase is
//! also measured by criterion with the shared fixed profile
//! (`benches/common::apply_fixed_profile`: warmup 3 s / measure 5 s).
//!
//! **Self-checks (a run that cannot prove what it measured does not report
//! a number):** the search must return hits (an empty search would be "fast"
//! for the wrong reason), and the **WAL header on disk** must say v3
//! (plaintext) or v4 (encrypted) — the direct proof of the mode, not just
//! the presence of the registry file.
//!
//! Run: `cargo bench -p vantadb --bench encrypted_p99 [--features encryption]`

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use tempfile::{tempdir, TempDir};
use vantadb::config::Config;
use vantadb::node::{FilterBitset, UnifiedNode};
use vantadb::storage::StorageEngine;

mod common;

const DIM: usize = 1536;
const N_VECTORS: usize = 100_000;
const N_QUERIES: usize = 1_000;
const TOP_K: usize = 10;
const SEED: u64 = 42;

/// Master key of the bench — 32 raw bytes, hex-encoded (AES-256).
///
/// A **published** key: it protects a throwaway tempdir, nothing else.
const BENCH_KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

/// Nodes inserted per outer loop iteration. The vectors are streamed **one at
/// a time** (never a batch in RAM): the values are identical to the canonical
/// dataset whatever this is, and the bench's own RSS stays flat.
const INSERT_CHUNK: usize = 2_000;

/// Warm-up passes over the query set before the measured one.
///
/// **Default 0** — the same methodology as `benches/canonical_p99.rs` (one
/// measured pass over the 1000 queries, no warm-up), so the numbers are
/// comparable with the canonical baseline.
///
/// `ENCRYPTED_P99_WARMUP=1` adds a discarded pass that warms the OS page
/// cache of the vstore: it removes the page-fault noise of a cold file from
/// the measurement (on a shared box that noise is ±100 % and swamps the
/// effect). It also evicts the CPU cache, so the absolute numbers change —
/// only the *comparison* between the two variants is meaningful.
fn warmup_passes() -> usize {
    std::env::var("ENCRYPTED_P99_WARMUP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// WAL fsync batch (`Config::flush_threshold`).
///
/// `Config::default()` has `sync_mode = Periodic` and no threshold, and the
/// WAL's default periodic threshold is **1** → an fsync per record, which
/// buries the encryption cost under the fsync latency of *both* variants.
/// Batching to 1000 records keeps the delta visible. Identical in the
/// plaintext and the encrypted run, so the comparison stays fair.
///
/// The same knob also arms the engine's auto-flush (`apply_insert`), but
/// that branch is **unreachable on this path**: `insert()` holds
/// `insert_lock` while it calls `apply_insert`, and the auto-flush only
/// fires if `insert_lock.try_lock()` succeeds — it cannot. So the threshold
/// acts purely as the WAL fsync batch here (verified in the W7 note).
const WAL_FLUSH_THRESHOLD: usize = 1_000;

/// One measured run.
#[derive(Debug, Clone, Copy)]
struct RunMetrics {
    insert: Duration,
    search_p50: Duration,
    search_p95: Duration,
    search_p99: Duration,
    flush: Duration,
    cold: Duration,
    /// Hits of the first query — the sanity check of the search.
    hits: usize,
}

/// Nearest-rank percentile over sorted latencies (mirrors `canonical_p99.rs`).
fn percentile(sorted: &[Duration], q: f64) -> Duration {
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

/// Median of a set of durations (odd count → the middle sample).
fn median(values: &mut [Duration]) -> Duration {
    values.sort_unstable();
    values[values.len() / 2]
}

/// Emit a measurement line **and persist it to a file**.
///
/// The file is not a convenience: Rust flushes `println!` at process exit,
/// so a run that is killed at the very end (the 100k run hit the machine's
/// commit limit while the criterion group was still holding the reopened
/// engine) loses every number it printed. The file is written and flushed
/// after every line, so a killed run still leaves its numbers behind.
///
/// Path: `ENCRYPTED_P99_OUT` (default `target/w7-results.txt`).
fn report(line: &str) {
    println!("{line}");
    let path =
        std::env::var("ENCRYPTED_P99_OUT").unwrap_or_else(|_| "target/w7-results.txt".to_string());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
}

/// The canonical dataset: one vector at a time, in the canonical order.
fn next_vector(rng: &mut StdRng) -> Vec<f32> {
    (0..DIM).map(|_| rng.random::<f32>()).collect()
}

/// The canonical query set — a fresh seed-42 stream, exactly as
/// `canonical_p99.rs` builds it.
fn generate_queries() -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(SEED);
    (0..N_QUERIES).map(|_| next_vector(&mut rng)).collect()
}

/// The engine configuration under test.
///
/// The key is always set: with the `encryption` feature compiled in the
/// database is encrypted from its first byte; with the feature compiled out
/// the field is ignored and the database stays plaintext. Same source, same
/// data, only the feature differs.
fn bench_config() -> Config {
    Config::default()
        .with_encryption(BENCH_KEY_HEX.to_string())
        .with_flush_threshold(WAL_FLUSH_THRESHOLD)
}

/// The search phase: 1000 queries through the index **and** the vector store.
///
/// The vector store is what makes this an at-rest measurement: the engine
/// scores the candidates from the store (a zero-copy mmap read in plaintext
/// mode, a per-node GCM decrypt in encrypted mode).
///
/// Returns the percentiles **and** the number of hits of the first query —
/// the sanity check that both variants really search the same index (a search
/// that returned no candidate would be "fast" for the wrong reason).
fn measure_search(
    engine: &StorageEngine,
    queries: &[Vec<f32>],
) -> (Duration, Duration, Duration, usize) {
    let hnsw = engine.hnsw.load();
    let vs = engine.vector_store[0].read();
    let mut latencies: Vec<Duration> = Vec::with_capacity(queries.len());
    let mut hits = 0usize;
    for (i, query) in queries.iter().enumerate() {
        let start = Instant::now();
        let result = hnsw.search_nearest(
            query,
            None,
            None,
            &FilterBitset::all_set(),
            TOP_K,
            Some(&*vs),
        );
        latencies.push(start.elapsed());
        if i == 0 {
            hits = result.len();
        }
    }
    drop(vs);
    drop(hnsw);
    latencies.sort_unstable();
    (
        percentile(&latencies, 0.50),
        percentile(&latencies, 0.95),
        percentile(&latencies, 0.99),
        hits,
    )
}

/// The on-disk WAL format version of the database under `<root>/data`.
///
/// This is the **direct** proof that the run measured what it claims: the
/// WAL header is cleartext, so its version says whether the frames on disk
/// are the encrypted layout (v4) or the plaintext one (v3). The registry
/// file alone would only prove that a key was configured.
fn wal_format_version(data_dir: &std::path::Path) -> Option<u16> {
    let mut candidates: Vec<std::path::PathBuf> = std::fs::read_dir(data_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vanta") && n.ends_with(".wal"))
        })
        .collect();
    candidates.sort();
    let path = candidates.first()?;
    let bytes = std::fs::read(path).ok()?;
    let header =
        vantadb::wal::WalHeader::deserialize(&bytes[..vantadb::wal::WalHeader::SIZE]).ok()?;
    Some(header.base.format_version)
}

/// One full run: open → insert → search → flush → close → reopen.
fn run_once(dir: &TempDir, queries: &[Vec<f32>], n: usize) -> (RunMetrics, StorageEngine) {
    let db_path = dir.path().to_str().expect("tempdir path is utf-8");
    let engine =
        StorageEngine::open_with_config(db_path, Some(bench_config())).expect("open engine");

    // ---- insert ----
    let mut rng = StdRng::seed_from_u64(SEED);
    let insert_start = Instant::now();
    let mut id: u128 = 0;
    while (id as usize) < n {
        let batch = (n - id as usize).min(INSERT_CHUNK);
        for _ in 0..batch {
            let vector = next_vector(&mut rng);
            engine
                .insert(&UnifiedNode::with_vector(id, vector))
                .expect("insert node");
            id += 1;
        }
    }
    let insert = insert_start.elapsed();

    // ---- search ----
    // Optional warm-up passes first (page cache), then the measured one.
    for _ in 0..warmup_passes() {
        let _ = measure_search(&engine, queries);
    }
    let (p50, p95, p99, hits) = measure_search(&engine, queries);
    // A search that found nothing would be "fast" for the wrong reason: the
    // bench must not publish a number for a run that measured an empty
    // index. Fail loudly instead.
    assert!(
        hits > 0,
        "the measured search returned 0 hits — nothing was measured"
    );

    // ---- flush (the whole HNSW image is rewritten) ----
    let flush_start = Instant::now();
    engine.flush().expect("flush");
    let flush = flush_start.elapsed();

    // ---- cold start (close + reopen: the image is read back and decrypted) ----
    drop(engine);
    let cold_start = Instant::now();
    let engine =
        StorageEngine::open_with_config(db_path, Some(bench_config())).expect("reopen engine");
    let cold = cold_start.elapsed();

    (
        RunMetrics {
            insert,
            search_p50: p50,
            search_p95: p95,
            search_p99: p99,
            flush,
            cold,
            hits,
        },
        engine,
    )
}

fn bench_encrypted_p99(c: &mut Criterion) {
    let n: usize = std::env::var("ENCRYPTED_P99_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(N_VECTORS);
    let runs: usize = std::env::var("ENCRYPTED_P99_RUNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let queries = generate_queries();
    let mut metrics: Vec<RunMetrics> = Vec::with_capacity(runs);
    let mut last: Option<(TempDir, StorageEngine)> = None;

    for run in 0..runs {
        let dir = tempdir().expect("tempdir for the bench database");
        let (m, engine) = run_once(&dir, &queries, n);
        report(&format!(
            "encrypted_p99 run={}/{} n={} insert_s={:.3} search_p50_ms={:.4} search_p95_ms={:.4} \
             search_p99_ms={:.4} flush_s={:.3} cold_s={:.3} hits={}",
            run + 1,
            runs,
            n,
            m.insert.as_secs_f64(),
            m.search_p50.as_secs_f64() * 1000.0,
            m.search_p95.as_secs_f64() * 1000.0,
            m.search_p99.as_secs_f64() * 1000.0,
            m.flush.as_secs_f64(),
            m.cold.as_secs_f64(),
            m.hits,
        ));
        metrics.push(m);
        if run + 1 == runs {
            // Keep the last database alive: the criterion group below measures
            // the search on the **reopened** engine (the realistic state).
            last = Some((dir, engine));
        }
    }

    // The mode under test — the **only** self-check that the run measured
    // what it claims: the WAL header on disk says whether the frames are the
    // encrypted layout (v4) or the plaintext one (v3). The registry file
    // alone would only prove that a key was configured.
    let (dir, engine) = last.expect("the bench ran at least once");
    let data_dir = dir.path().join("data");
    let registry = data_dir.join("encryption.registry");
    let wal_version = wal_format_version(&data_dir);
    let mode = match wal_version {
        Some(v) if v == vantadb::wal::WAL_ENCRYPTED_FORMAT_VERSION => "encrypted",
        Some(v) if v == vantadb::wal::WAL_FORMAT_VERSION => "plaintext",
        other => {
            let listing: Vec<String> = std::fs::read_dir(&data_dir)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or_default();
            panic!(
                "unexpected WAL format version on disk: {other:?} (data dir listing: {listing:?})"
            );
        }
    };
    // The registry and the WAL must agree: a WAL v4 without the registry (or
    // the other way round) would mean the mode was decided inconsistently.
    assert_eq!(
        registry.exists(),
        mode == "encrypted",
        "the registry and the WAL format version disagree about the mode",
    );
    let mut inserts: Vec<Duration> = metrics.iter().map(|m| m.insert).collect();
    let mut p50s: Vec<Duration> = metrics.iter().map(|m| m.search_p50).collect();
    let mut p95s: Vec<Duration> = metrics.iter().map(|m| m.search_p95).collect();
    let mut p99s: Vec<Duration> = metrics.iter().map(|m| m.search_p99).collect();
    let mut flushes: Vec<Duration> = metrics.iter().map(|m| m.flush).collect();
    let mut colds: Vec<Duration> = metrics.iter().map(|m| m.cold).collect();
    report(&format!(
        "encrypted_p99 MEDIAN mode={mode} wal_version={wal_version:?} n={n} runs={runs} insert_s={:.3} \
         search_p50_ms={:.4} search_p95_ms={:.4} search_p99_ms={:.4} flush_s={:.3} cold_s={:.3} \
         hits={}",
        median(&mut inserts).as_secs_f64(),
        median(&mut p50s).as_secs_f64() * 1000.0,
        median(&mut p95s).as_secs_f64() * 1000.0,
        median(&mut p99s).as_secs_f64() * 1000.0,
        median(&mut flushes).as_secs_f64(),
        median(&mut colds).as_secs_f64(),
        metrics.iter().map(|m| m.hits).min().unwrap_or(0),
    ));

    // ---- criterion: the search loop on the reopened engine ----
    let mut group = c.benchmark_group("encrypted_p99");
    common::apply_fixed_profile(&mut group);
    group.sample_size(10);
    group.bench_function("search_1000_queries_1536d", |b| {
        b.iter(|| {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store[0].read();
            for query in &queries {
                std::hint::black_box(hnsw.search_nearest(
                    query,
                    None,
                    None,
                    &FilterBitset::all_set(),
                    TOP_K,
                    Some(&*vs),
                ));
            }
        })
    });
    group.finish();
}

criterion_group!(benches, bench_encrypted_p99);
criterion_main!(benches);
