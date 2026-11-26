//! FIND-259 — an L1 payload whose `type` is outside the seven valid
//! `MemoryType` values must be **observable**, and the reader must keep
//! returning exactly what it returned before the counter existed.
//!
//! One test on purpose: the counters are process-wide, so a second test writing
//! L1 payloads in parallel would make the counts non-deterministic under
//! `cargo test` (nextest isolates each test in its own process, but the suite
//! must not depend on the runner).
#![allow(clippy::expect_used, clippy::unwrap_used)]

use vanta_memory::core::hooks::{perform_auto_recall, AutoRecallParams, RecallConfig, RecallScope};
use vanta_memory::core::record::l1_reader::{
    decode_stats, l1_namespace, read_active_namespace_records, reset_decode_stats,
};
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn put_payload(db: &Embedded, ns: &str, key: &str, payload: &str) {
    db.put(MemoryInput {
        namespace: ns.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put payload");
}

/// A complete L1 envelope (FIND-699: a partial one would fail the decode for
/// reasons that have nothing to do with the `type`).
fn envelope(id: &str, content: &str, ty: &str) -> String {
    serde_json::json!({
        "id": id,
        "content": content,
        "type": ty,
        "priority": 60,
        "scene_name": "dev",
        "source_message_ids": [],
        "timestamps": [],
        "created_at": "2026-08-22T10:00:00Z",
        "updated_at": "2026-08-22T10:00:00Z",
        "session_key": "sess-1",
    })
    .to_string()
}

fn recall(db: &Embedded, query: &str) -> Option<vanta_memory::core::hooks::RecallResult> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text: query,
            session_key: "sess-1",
            isolation: None,
            config: RecallConfig {
                scope: RecallScope::Session,
                ..RecallConfig::default()
            },
        },
        None,
    )
    .expect("recall never errors")
}

#[test]
fn decode_failures_are_counted_by_cause_and_reader_output_is_unchanged() {
    let db = db();
    let ns = l1_namespace("sess-1");
    reset_decode_stats();

    // ── (a) `type` outside the valid set: a write bug, counted as such ──────
    for i in 0..3 {
        let id = format!("bad-{i}");
        put_payload(
            &db,
            &ns,
            &id,
            &envelope(&id, "user prefers dark mode", "fact"),
        );
    }
    // Behaviour first: the fallback still returns every payload, so the reader's
    // output is byte-identical to the pre-counter behaviour.
    let active = read_active_namespace_records(&db, &ns).expect("read namespace");
    assert_eq!(active.len(), 3, "the fallback still returns every record");

    let stats = decode_stats();
    assert_eq!(stats.invalid_type, 3, "three invalid `type` payloads");
    assert_eq!(
        stats.unparseable, 0,
        "`type` bugs are never counted as generic"
    );

    // The invisible mode from the finding: a query that does not happen to
    // overlap the raw payload returns nothing, and the counter is the only
    // evidence the record exists.
    assert!(
        recall(&db, "zzz qqq").is_none(),
        "a type-invalid record is invisible to recall"
    );

    // And when the query does overlap, what comes back is NOT the written
    // memory: the declared content/type are gone (this is the degradation the
    // counter now makes observable, not a behaviour this task changes).
    let hit = recall(&db, "user prefers dark mode").expect("fallback record ranks by raw payload");
    let recalled = &hit.recalled_memories[0];
    assert_eq!(recalled.memory_type, "episodic");
    assert!(
        recalled.content.starts_with('{'),
        "content is the raw payload, not the declared content: {}",
        recalled.content
    );
    assert!(
        recalled.content.contains("\"type\":\"fact\""),
        "the invalid type is only visible inside the raw payload"
    );

    // ── (b) a plain payload (ADR / document) is a different cause ───────────
    // Separate namespace so the counts stay exact: re-reading a namespace
    // re-classifies its payloads (the counters count decodes, not records).
    reset_decode_stats();
    let plain_ns = l1_namespace("sess-2");
    put_payload(&db, &plain_ns, "doc-1", "plain note, not an L1 envelope");
    let plain = read_active_namespace_records(&db, &plain_ns).expect("read namespace");
    assert_eq!(plain.len(), 1, "the plain payload also survives the read");
    let stats = decode_stats();
    assert_eq!(stats.unparseable, 1, "no `type` field → generic cause");
    assert_eq!(stats.invalid_type, 0, "a plain payload is not a type bug");

    // A valid envelope keeps the counters untouched (the fallback is only for
    // payloads the decode rejects).
    reset_decode_stats();
    let good_ns = l1_namespace("sess-3");
    put_payload(
        &db,
        &good_ns,
        "good-1",
        &envelope("good-1", "team deploys on Fridays", "work_fact"),
    );
    let decoded = read_active_namespace_records(&db, &good_ns).expect("read namespace");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].id, "good-1");
    assert_eq!(
        decode_stats(),
        Default::default(),
        "valid payloads never count"
    );
}
