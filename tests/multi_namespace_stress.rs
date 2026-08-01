//! Stress test: concurrent access across 10+ namespaces.
//!
//! Verifies:
//! - Concurrent writes (12 threads, one per namespace, 1000 ops each)
//! - Concurrent searches across namespaces
//! - No cross-namespace data leaks
//! - No deadlocks or crashes
//! - Completes within 60 seconds
//!
//! Uses the `InMemory` backend: the goal is isolation and concurrency
//! correctness, not disk write throughput. The default Fjall backend
//! fsyncs every WAL append (~10ms/put), which alone blows the 60s budget
//! at 1000 ops per namespace and would make the test timing-bound instead
//! of concurrency-bound.

use std::thread;
use std::time::Instant;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::storage::BackendKind;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest};

const NS_COUNT: usize = 12;
const OPS_PER_NS: usize = 1000;
const SEARCHES_PER_NS: usize = 200;

#[test]
fn concurrent_multi_namespace_stress() {
    let started = Instant::now();
    let dir = tempdir().expect("tempdir");
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().into_owned(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).expect("open embedded db");

    let namespaces: Vec<String> = (0..NS_COUNT).map(|i| format!("ns-{i:02}")).collect();

    // ── Phase 1: concurrent writes — one thread per namespace ──
    let mut writers = Vec::new();
    for ns in &namespaces {
        let db = db.clone();
        let ns = ns.clone();
        writers.push(thread::spawn(move || {
            for j in 0..OPS_PER_NS {
                let mut input =
                    VantaMemoryInput::new(&ns, format!("key-{j}"), format!("payload-{ns}-{j}"));
                input.vector = Some(vec![(j % 128) as f32, ns.len() as f32, 0.5]);
                db.put(input).expect("concurrent put must not fail");
            }
        }));
    }
    for h in writers {
        h.join().expect("writer thread panicked");
    }
    eprintln!("[stress] writes: {:?}", started.elapsed());

    // ── Phase 2: isolation — each namespace holds exactly its own records ──
    let mut total_records = 0usize;
    for ns in &namespaces {
        let options = VantaMemoryListOptions {
            limit: OPS_PER_NS,
            ..Default::default()
        };
        let page = db.list(ns, options).expect("list namespace");
        assert_eq!(
            page.records.len(),
            OPS_PER_NS,
            "namespace {ns} count mismatch (cross-namespace leak?)"
        );
        for record in &page.records {
            assert_eq!(
                &record.namespace, ns,
                "cross-namespace leak detected in {ns}"
            );
        }
        total_records += page.records.len();
    }

    let all_namespaces = db.list_namespaces().expect("list all namespaces");
    assert_eq!(
        all_namespaces.len(),
        NS_COUNT,
        "unexpected namespaces: {all_namespaces:?}"
    );
    assert_eq!(
        total_records,
        NS_COUNT * OPS_PER_NS,
        "phantom records detected"
    );
    eprintln!("[stress] isolation verified: {:?}", started.elapsed());

    // ── Phase 3: concurrent searches — verify hits never cross namespaces ──
    let mut searchers = Vec::new();
    for ns in &namespaces {
        let db = db.clone();
        let ns = ns.clone();
        searchers.push(thread::spawn(move || {
            for _ in 0..SEARCHES_PER_NS {
                let request = VantaMemorySearchRequest {
                    namespace: ns.clone(),
                    query_vector: vec![1.0, ns.len() as f32, 0.5],
                    top_k: 10,
                    ..Default::default()
                };
                let hits = db.search(request).expect("concurrent search must not fail");
                for hit in &hits {
                    assert_eq!(&hit.record.namespace, &ns, "cross-namespace leak in search");
                }
            }
        }));
    }
    for h in searchers {
        h.join().expect("searcher thread panicked");
    }
    eprintln!("[stress] searches: {:?}", started.elapsed());

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() < 60,
        "stress test exceeded 60s: {elapsed:?}"
    );
    eprintln!(
        "concurrent_multi_namespace_stress: {NS_COUNT} ns x {OPS_PER_NS} writes + \
         {SEARCHES_PER_NS} searches in {elapsed:?} — no leaks, no crashes"
    );
}
