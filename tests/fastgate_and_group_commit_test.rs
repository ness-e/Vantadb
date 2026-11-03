#![allow(clippy::expect_used, clippy::unwrap_used)]
//! FastGate decision pipeline tests and default group_commit concurrency certification.
//!
//! Validates:
//! 1. FastGate real execution via `Embedded::put`:
//!    - DedupPolicy::Exact: exact duplicate does not create a second node, reinforces existing
//!    - DedupPolicy::Semantic:
//!      - Paraphrase (vector cosine >= 0.92): reinforces existing without creating a new node
//!      - Update (vector cosine 0.78..0.92): supersedes previous version with graph lineage edges
//!      - Orthogonal (vector cosine < 0.78): adds as fresh record
//!    - DedupPolicy::Off: no dedup alterations (standard KV behavior)
//!    - Bloom false positive collision test: tiny filter collision never drops distinct records
//!    - Latency measurements of exact stage (< 0.1 ms target)
//! 2. Concurrency: 16 simultaneous writers operating under default group_commit (InsertBatchConfig enabled: true).

use std::sync::Arc;
use tempfile::tempdir;
use vantadb::config::Config;
use vantadb::utils::duplicate_prevention::{token_jaccard_similarity, DuplicatePreventionFilter};
use vantadb::{DedupPolicy, Embedded, MemoryInput};

#[test]
fn test_fastgate_decision_routing_noop_supersede_add() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open database");
    let ns = "agent_memories";

    // ── 0. Seed original fact ──
    let orig_key = "coffee_preference";
    let orig_payload = "User prefers pour-over black coffee without sugar";
    let orig_vec = vec![1.0, 0.0, 0.0];
    let orig_rec = db
        .put(
            MemoryInput::new(ns, orig_key, orig_payload)
                .with_vector(orig_vec.clone())
                .with_dedup_policy(DedupPolicy::Semantic),
        )
        .expect("put original");
    assert_eq!(orig_rec.key, orig_key);
    assert_eq!(orig_rec.version, 1);
    assert!(orig_rec.superseded_by.is_none());

    // ── 1. High similarity / paraphrase -> NoopReinforce via put() ──
    let near_dup_key = "coffee_near_dup";
    let near_dup_payload = "User prefers pour over black coffee without sugar";
    let near_vec = vec![0.98, 0.198, 0.0]; // Cosine = 0.98 >= 0.92
    let jaccard_high = token_jaccard_similarity(orig_payload, near_dup_payload);
    assert!(jaccard_high >= 0.85, "jaccard was {jaccard_high}");

    let high_sim_rec = db
        .put(
            MemoryInput::new(ns, near_dup_key, near_dup_payload)
                .with_vector(near_vec)
                .with_dedup_policy(DedupPolicy::Semantic),
        )
        .expect("put near duplicate");

    // Must reinforce the existing record without creating a second node
    assert_eq!(
        high_sim_rec.key, orig_key,
        "paraphrase put must return reinforced existing record key"
    );
    assert!(
        db.get(ns, near_dup_key).expect("get").is_none(),
        "near_dup_key node must NOT exist in storage"
    );

    // ── 2. Medium similarity -> Supersede with supersedes_edge via put() ──
    let updated_key = "coffee_preference_v2";
    let updated_payload = "User switched to oat milk cappuccino in the morning";
    let updated_vec = vec![0.85, 0.526, 0.0]; // Cosine = 0.85 in 0.78..0.92 band

    let new_rec = db
        .put(
            MemoryInput::new(ns, updated_key, updated_payload)
                .with_vector(updated_vec)
                .with_dedup_policy(DedupPolicy::Semantic),
        )
        .expect("put updated");

    assert_eq!(new_rec.key, updated_key);

    // Verify old record has superseded_by set and invalid_at_ms populated
    let old_rec = db.get(ns, orig_key).expect("get old").expect("present");
    assert_eq!(old_rec.superseded_by.as_deref(), Some(updated_key));
    assert!(old_rec.invalid_at_ms.is_some());

    // Verify graph edges: new_rec --supersedes_edge--> old_rec and old_rec --superseded_by--> new_rec
    let engine = db.engine_handle().expect("engine handle");
    let old_node = engine
        .get(orig_rec.node_id)
        .expect("engine get old")
        .expect("old node");
    let new_node = engine
        .get(new_rec.node_id)
        .expect("engine get new")
        .expect("new node");

    let old_superseded_by_target = old_node
        .edges
        .iter()
        .find(|e| engine.resolve_label(e.label_id).as_deref() == Some("superseded_by"));
    assert!(
        old_superseded_by_target.is_some(),
        "old node must have superseded_by edge"
    );
    assert_eq!(old_superseded_by_target.unwrap().target, new_rec.node_id);

    let new_supersedes_target = new_node
        .edges
        .iter()
        .find(|e| engine.resolve_label(e.label_id).as_deref() == Some("supersedes_edge"));
    assert!(
        new_supersedes_target.is_some(),
        "new node must have supersedes_edge edge"
    );
    assert_eq!(new_supersedes_target.unwrap().target, orig_rec.node_id);

    // ── 3. Low similarity -> Add fresh orthogonal memory via put() ──
    let orthogonal_key = "quantum_study";
    let orthogonal_payload = "User is studying quantum computing algorithms with Qiskit";
    let orthogonal_vec = vec![0.0, 1.0, 0.0]; // Cosine = 0.0 < 0.78

    let ortho_rec = db
        .put(
            MemoryInput::new(ns, orthogonal_key, orthogonal_payload)
                .with_vector(orthogonal_vec)
                .with_dedup_policy(DedupPolicy::Semantic),
        )
        .expect("put orthogonal");
    assert_eq!(ortho_rec.key, orthogonal_key);
    assert!(ortho_rec.superseded_by.is_none());
    assert!(db.get(ns, orthogonal_key).expect("get").is_some());

    // ── 4. Exact duplicate via DedupPolicy::Exact ──
    let exact_dup_key = "coffee_exact_copy";
    let exact_rec = db
        .put(
            MemoryInput::new(ns, exact_dup_key, updated_payload)
                .with_dedup_policy(DedupPolicy::Exact),
        )
        .expect("put exact duplicate");

    assert_eq!(
        exact_rec.key, updated_key,
        "exact duplicate must reinforce and return existing record"
    );
    assert!(
        db.get(ns, exact_dup_key).expect("get").is_none(),
        "exact duplicate key must NOT create a new node"
    );

    // ── 5. Off policy: standard KV upsert without dedup interception ──
    let off_key = "off_duplicate";
    let off_rec = db
        .put(MemoryInput::new(ns, off_key, updated_payload).with_dedup_policy(DedupPolicy::Off))
        .expect("put with policy Off");
    assert_eq!(off_rec.key, off_key);
    assert!(
        db.get(ns, off_key).expect("get").is_some(),
        "DedupPolicy::Off must create normal independent record"
    );

    db.close().expect("close database");
}

#[test]
fn test_bloom_false_positive_no_data_loss() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open database");

    // Inject a tiny 8-bit Bloom filter to force collisions
    let tiny_filter = Arc::new(DuplicatePreventionFilter::new_with_bits(8));
    let db = db.with_dedup_filter(tiny_filter);
    let ns = "collision_test";

    // Insert first record
    let rec1 = db
        .put(
            MemoryInput::new(ns, "item_alpha", "Alpha unique data stream")
                .with_dedup_policy(DedupPolicy::Exact),
        )
        .expect("put alpha");
    assert_eq!(rec1.key, "item_alpha");

    // Insert second record with completely different content.
    // The tiny 8-bit Bloom filter will collide (bits already set), but
    // the confirmation against stored records will detect that this content
    // does not match "item_alpha".
    let rec2 = db
        .put(
            MemoryInput::new(ns, "item_beta", "Beta distinct data payload")
                .with_dedup_policy(DedupPolicy::Exact),
        )
        .expect("put beta under collision");

    assert_eq!(
        rec2.key, "item_beta",
        "Bloom false positive must NOT drop distinct record 'item_beta'"
    );

    // Verify both records are present with their intact data
    let fetched_alpha = db
        .get(ns, "item_alpha")
        .expect("get")
        .expect("alpha exists");
    let fetched_beta = db.get(ns, "item_beta").expect("get").expect("beta exists");

    assert_eq!(fetched_alpha.payload, "Alpha unique data stream");
    assert_eq!(fetched_beta.payload, "Beta distinct data payload");

    db.close().expect("close database");
}

#[test]
fn test_exact_dedup_latency_measurement() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open database");
    let ns = "latency_bench";

    // Seed record
    let seed_payload = "Normalized benchmark sentence for exact dedup latency check";
    db.put(MemoryInput::new(ns, "seed", seed_payload).with_dedup_policy(DedupPolicy::Exact))
        .expect("seed");

    // Run 50 iterations measuring exact check stage and overall reinforced put
    let mut exact_stage_latencies_ns = Vec::with_capacity(50);
    let mut put_latencies_ns = Vec::with_capacity(50);

    let filter = db.dedup_filter();
    let norm_payload = vantadb::utils::duplicate_prevention::normalize_content(seed_payload);
    let norm_hash = vantadb::utils::duplicate_prevention::hash_content(&norm_payload);
    let filter_bytes = format!("{}\0{}", ns, norm_payload);

    for i in 0..50 {
        // Measure exact filter stage directly:
        let stage_start = std::time::Instant::now();
        let hit = filter.check_and_mark(filter_bytes.as_bytes());
        let _ = filter.lookup_exact(ns, norm_hash);
        exact_stage_latencies_ns.push(stage_start.elapsed().as_nanos());
        assert!(hit, "filter must register hit for seen payload");

        let key = format!("dup_{i}");
        let put_start = std::time::Instant::now();
        let _ = db
            .put(MemoryInput::new(ns, key, seed_payload).with_dedup_policy(DedupPolicy::Exact))
            .expect("put exact dup");
        put_latencies_ns.push(put_start.elapsed().as_nanos());
    }

    let min_stage_ns = exact_stage_latencies_ns.iter().min().copied().unwrap_or(0);
    let max_stage_ns = exact_stage_latencies_ns.iter().max().copied().unwrap_or(0);
    let avg_stage_ns =
        exact_stage_latencies_ns.iter().sum::<u128>() / exact_stage_latencies_ns.len() as u128;

    let min_put_ns = put_latencies_ns.iter().min().copied().unwrap_or(0);
    let max_put_ns = put_latencies_ns.iter().max().copied().unwrap_or(0);
    let avg_put_ns = put_latencies_ns.iter().sum::<u128>() / put_latencies_ns.len() as u128;

    println!(
        "[LATENCY REPORT] Exact Dedup Stage Filter (50 runs): min={:.4} ms ({} ns), avg={:.4} ms ({} ns), max={:.4} ms ({} ns)",
        min_stage_ns as f64 / 1_000_000.0,
        min_stage_ns,
        avg_stage_ns as f64 / 1_000_000.0,
        avg_stage_ns,
        max_stage_ns as f64 / 1_000_000.0,
        max_stage_ns
    );
    println!(
        "[LATENCY REPORT] Overall Reinforced put() Call (50 runs): min={:.3} ms ({} ns), avg={:.3} ms ({} ns), max={:.3} ms ({} ns)",
        min_put_ns as f64 / 1_000_000.0,
        min_put_ns,
        avg_put_ns as f64 / 1_000_000.0,
        avg_put_ns,
        max_put_ns as f64 / 1_000_000.0,
        max_put_ns
    );

    db.close().expect("close database");
}

#[test]
fn test_group_commit_concurrency_16_writers() {
    let default_config = Config::default();
    assert!(
        default_config.insert_batch.enabled,
        "default Config must have group_commit / insert_batch enabled"
    );
    assert_eq!(default_config.insert_batch.max_batch_records, 64);

    let dir = tempdir().expect("tempdir");
    let db = Arc::new(Embedded::open(dir.path()).expect("open database"));
    let ns = "concurrent_ns";

    const NUM_WRITERS: usize = 16;
    const WRITES_PER_THREAD: usize = 25;
    let total_expected = NUM_WRITERS * WRITES_PER_THREAD;

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(NUM_WRITERS);
        for writer_id in 0..NUM_WRITERS {
            let db_clone = Arc::clone(&db);
            handles.push(scope.spawn(move || {
                for i in 0..WRITES_PER_THREAD {
                    let key = format!("w_{writer_id}_item_{i}");
                    let payload = format!("Payload from writer {writer_id} iteration {i}");
                    db_clone
                        .put(MemoryInput::new(ns, key, payload))
                        .expect("concurrent put must succeed under group_commit");
                }
            }));
        }
        for handle in handles {
            handle.join().expect("writer thread joined cleanly");
        }
    });

    // Verify all records were stored
    for writer_id in 0..NUM_WRITERS {
        for i in 0..WRITES_PER_THREAD {
            let key = format!("w_{writer_id}_item_{i}");
            let expected_payload = format!("Payload from writer {writer_id} iteration {i}");
            let record = db
                .get(ns, &key)
                .expect("get record")
                .unwrap_or_else(|| panic!("record {key} missing"));
            assert_eq!(record.payload, expected_payload);
            assert_eq!(record.version, 1);
        }
    }

    // Clean close and reopen to verify durability under default group_commit
    let unwrap_db = Arc::try_unwrap(db).expect("exclusive Arc ownership");
    unwrap_db.close().expect("close database");

    let reopened = Embedded::open(dir.path()).expect("reopen database");
    let mut count = 0;
    for writer_id in 0..NUM_WRITERS {
        for i in 0..WRITES_PER_THREAD {
            let key = format!("w_{writer_id}_item_{i}");
            let rec = reopened.get(ns, &key).expect("get after reopen");
            if rec.is_some() {
                count += 1;
            }
        }
    }
    assert_eq!(
        count, total_expected,
        "all 16x25 = {total_expected} records must survive close and reopen"
    );
    reopened.close().expect("close reopened");
}
