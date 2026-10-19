// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-11 Step 1: pipeline L1 writes via `put_batch` (group-commit).
//!
//! - Parity: `apply_dedup_batch` (batch path) must produce the exact same
//!   records as sequential `write_memory` calls (store/update/merge/skip +
//!   contradiction marking).
//! - Measurement (ignored by default): sequential `put` vs `put_batch` on a
//!   realistic store, run explicitly for the MEMG-11 A/B:
//!   `cargo nextest run -p vanta-memory --test l1_batch_write --run-ignored ignored-only --nocapture`

use std::time::Instant;

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, MemoryRecord, MemoryType,
};
use vanta_memory::core::record::{
    apply_dedup_batch, generate_memory_id, read_session_records, write_memory,
};
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};

fn open_db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open embedded")
}

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        source_message_ids: vec![],
        scene_name: "prefs".into(),
        metadata: serde_json::Value::Null,
    }
}

fn decision(record_id: &str, action: DedupAction) -> DedupDecision {
    DedupDecision {
        record_id: record_id.into(),
        action,
        target_ids: vec![],
        contradicts: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    }
}

fn existing_record(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "prefs".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
        version: 1,
        session_key: "sk".into(),
        session_id: "".into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

fn put_raw(db: &Embedded, ns: &str, record: &MemoryRecord) {
    db.put(MemoryInput {
        namespace: ns.to_string(),
        key: record.id.clone(),
        payload: serde_json::to_string(record).expect("serialize"),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put");
}

/// Characterization: batch path == sequential path, field by field.
#[test]
fn apply_dedup_batch_matches_write_memory_sequential() {
    let memories = [
        memory("user prefers dark mode"),
        memory("covered by existing memory"),
        memory("user switched to vim keybindings"),
        memory("user no longer likes coffee"),
    ];
    let mut update = decision("m_c", DedupAction::Update);
    update.target_ids = vec!["old1".into()];
    update.merged_content = Some("user switched to vim keybindings (merged)".into());
    update.merged_priority = Some(90);
    let mut contradict = decision("m_d", DedupAction::Store);
    contradict.contradicts = vec!["old2".into()];
    let decisions = [
        decision("m_a", DedupAction::Store),
        decision("m_b", DedupAction::Skip),
        update,
        contradict,
    ];

    // DB 1 — batch path (apply_dedup_batch; put_batch after the migration).
    let batch_db = open_db();
    put_raw(
        &batch_db,
        "l1/sess-p",
        &existing_record("old1", "user used emacs"),
    );
    put_raw(
        &batch_db,
        "l1/sess-p",
        &existing_record("old2", "user likes coffee"),
    );
    let batch_written = apply_dedup_batch(
        &batch_db,
        "sess-p",
        "sess-p",
        &memories,
        &decisions,
        1_700_000_000_000,
        None,
    )
    .expect("batch write");

    // DB 2 — sequential path (write_memory one by one).
    let seq_db = open_db();
    put_raw(
        &seq_db,
        "l1/sess-p",
        &existing_record("old1", "user used emacs"),
    );
    put_raw(
        &seq_db,
        "l1/sess-p",
        &existing_record("old2", "user likes coffee"),
    );
    let mut seq_written = Vec::new();
    for (idx, m) in memories.iter().enumerate() {
        if let Some(record) = write_memory(
            &seq_db,
            "sess-p",
            "sess-p",
            m,
            &decisions[idx],
            1_700_000_000_000,
            idx,
            None,
        )
        .expect("sequential write")
        {
            seq_written.push(record);
        }
    }

    let mut batch_all = read_session_records(&batch_db, "sess-p").expect("read batch");
    let mut seq_all = read_session_records(&seq_db, "sess-p").expect("read seq");
    batch_all.sort_by(|a, b| a.id.cmp(&b.id));
    seq_all.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(
        batch_written.len(),
        seq_written.len(),
        "same number of persisted records"
    );
    assert_eq!(batch_all.len(), seq_all.len(), "same final record set");
    for (b, s) in batch_all.iter().zip(seq_all.iter()) {
        assert_eq!(b.id, s.id);
        assert_eq!(b.content, s.content);
        assert_eq!(b.version, s.version);
        assert_eq!(b.priority, s.priority);
        assert_eq!(b.superseded_by, s.superseded_by);
        assert_eq!(b.created_at, s.created_at);
    }
    // Contradiction marking survives the batch path (MEMG-01 semantics).
    let old2 = batch_all
        .iter()
        .find(|r| r.id == "old2")
        .expect("old2 preserved");
    assert_eq!(old2.superseded_by.as_deref(), Some("m_d"));
    // Update target was replaced by the merged record.
    assert!(batch_all.iter().all(|r| r.id != "old1"));
}

/// Measurement instrument (explicit run only — no timing asserts, Regla 9/11
/// numbers come from the printed output recorded in the task file A/B).
#[test]
#[ignore = "measurement instrument — run explicitly with --run-ignored ignored-only --nocapture"]
fn measurement_sequential_puts_vs_put_batch() {
    let batch_size = 20usize;
    for store in [0usize, 200, 2000] {
        // Seed the store (untimed).
        let db = open_db();
        let ns = "l1/bench-session";
        for i in 0..store {
            put_raw(&db, ns, &existing_record(&format!("seed-{i:05}"), "seeded"));
        }

        // Sequential: batch_size fresh records via db.put.
        let seq_start = Instant::now();
        for i in 0..batch_size {
            put_raw(
                &db,
                ns,
                &existing_record(&format!("seq-{i:03}"), "sequential"),
            );
        }
        let seq_ms = seq_start.elapsed().as_secs_f64() * 1000.0;

        // Batch: batch_size fresh records via db.put_batch (one call).
        let inputs: Vec<MemoryInput> = (0..batch_size)
            .map(|i| MemoryInput {
                namespace: ns.to_string(),
                key: format!("batch-{i:03}"),
                payload: serde_json::to_string(&existing_record("x", "batched"))
                    .expect("serialize"),
                metadata: MemoryMetadata::new(),
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
                ..Default::default()
            })
            .collect();
        let batch_start = Instant::now();
        db.put_batch(inputs).expect("put_batch");
        let batch_ms = batch_start.elapsed().as_secs_f64() * 1000.0;

        eprintln!(
            "[MEMG-11 A/B] store={store} batch={batch_size} sequential={seq_ms:.2}ms batch={batch_ms:.2}ms ratio={:.2}x",
            seq_ms / batch_ms
        );
    }

    // Large batch on a mid store: group-commit territory.
    let db = open_db();
    let ns = "l1/bench-session";
    for i in 0..2000 {
        put_raw(&db, ns, &existing_record(&format!("seed-{i:05}"), "seeded"));
    }
    let n = 200usize;
    let seq_start = Instant::now();
    for i in 0..n {
        put_raw(
            &db,
            ns,
            &existing_record(&format!("seq2-{i:03}"), "sequential"),
        );
    }
    let seq_ms = seq_start.elapsed().as_secs_f64() * 1000.0;
    let inputs: Vec<MemoryInput> = (0..n)
        .map(|i| MemoryInput {
            namespace: ns.to_string(),
            key: format!("batch2-{i:03}"),
            payload: serde_json::to_string(&existing_record("x", "batched")).expect("serialize"),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .collect();
    let batch_start = Instant::now();
    db.put_batch(inputs).expect("put_batch");
    let batch_ms = batch_start.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "[MEMG-11 A/B] store=2000 batch={n} sequential={seq_ms:.2}ms batch={batch_ms:.2}ms ratio={:.2}x",
        seq_ms / batch_ms
    );
}

/// Keep the deterministic-id helper honest for the batch path (regression:
/// the migration must not change generated ids).
#[test]
fn generated_ids_stay_deterministic() {
    assert_eq!(
        generate_memory_id(1_700_000_000_000, 7),
        "m_1700000000000_7"
    );
}
