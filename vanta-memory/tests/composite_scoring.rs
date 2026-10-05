// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-21 — composite scoring over the L1 recall: ordering metric
//! before/after.
//!
//! Seeds one session with a fixture where the legacy relevance order and the
//! composite order differ, then pins both with exact positions:
//!
//! | record | overlap | age | priority | legacy rank | composite rank |
//! |--------|---------|-----|----------|-------------|----------------|
//! | r2     | 2       | 0d  | 90       | 2           | 1              |
//! | r1     | 3       | 30d | 10       | 1           | 2              |
//! | r3     | 2       | 60d | 50       | 3           | 3              |
//! | r4     | 1       | 90d | 20       | 4           | 4              |
//!
//! Policy, not calibration: the expected orders are hand-derived from the
//! declared weights (CrewAI defaults 500/300/200 per-mille) and MEMG-07
//! retention (`2^(−age/half_life)`, episodic half-life = 7d) with a fixed
//! `now_ms`; no accuracy claim is made.

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall, perform_auto_recall_scored, AutoRecallParams, InjectionPolicy,
    RecallConfig, RecallResult,
};
use vanta_memory::core::record::{l1_namespace, CompositeScoring};
use vantadb::config::Config;
use vantadb::sdk::Embedded;

/// Fixed instant: 2026-01-31T00:00:00.000Z (same fixture instant as the
/// MEMG-07 tests). Ages: r1 = 30d (2026-01-01), r3 = 60d (2025-12-02),
/// r4 = 90d (2025-11-02).
const NOW_MS: u64 = 1_769_817_600_000;

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

/// Episodic fixture with explicit priority + updated_at (composite signals).
fn aged(id: &str, content: &str, updated_at: &str, priority: i32) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Episodic,
        priority,
        scene_name: "ops".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![updated_at.into()],
        created_at: updated_at.into(),
        updated_at: updated_at.into(),
        version: 1,
        session_key: "sess-1".into(),
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

/// Persist an L1 record exactly as `read_session_records` expects it.
fn put_l1(db: &Embedded, record: &MemoryRecord) {
    use vantadb::sdk::{MemoryInput, MemoryMetadata};
    db.put(MemoryInput {
        namespace: l1_namespace(&record.session_key),
        key: record.id.clone(),
        payload: serde_json::to_string(record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put l1 record");
}

fn seed(db: &Embedded) {
    put_l1(
        db,
        &aged(
            "r1",
            "deploy pipeline postgres notes",
            "2026-01-01T00:00:00.000Z",
            10,
        ),
    );
    put_l1(
        db,
        &aged("r2", "deploy pipeline", "2026-01-31T00:00:00.000Z", 90),
    );
    put_l1(
        db,
        &aged("r3", "deploy postgres", "2025-12-02T00:00:00.000Z", 50),
    );
    put_l1(
        db,
        &aged("r4", "postgres notes", "2025-11-02T00:00:00.000Z", 20),
    );
}

fn params<'a>(user_text: &'a str) -> AutoRecallParams<'a> {
    AutoRecallParams {
        user_text,
        session_key: "sess-1",
        isolation: None,
        config: RecallConfig::default(),
    }
}

fn recalled_ids(result: &Option<RecallResult>) -> Vec<String> {
    result
        .as_ref()
        .expect("content to inject")
        .recalled_memories
        .iter()
        .map(|m| m.source_key.clone())
        .collect()
}

fn legacy_ids(db: &Embedded, query: &str) -> Vec<String> {
    recalled_ids(&perform_auto_recall(db, params(query), None).expect("legacy recall"))
}

fn composite_ids(db: &Embedded, query: &str) -> Vec<String> {
    recalled_ids(
        &perform_auto_recall_scored(
            db,
            params(query),
            None,
            &InjectionPolicy::allow_all(),
            &CompositeScoring::default(),
            NOW_MS,
        )
        .expect("scored recall"),
    )
}

// ── before: legacy relevance-only order ──

#[test]
fn legacy_recall_orders_by_overlap_only() {
    let db = db();
    seed(&db);
    assert_eq!(
        legacy_ids(&db, "deploy pipeline postgres"),
        vec!["r1", "r2", "r3", "r4"],
        "legacy: overlap desc, then updated_at desc"
    );
}

// ── after: composite order (metric) ──

#[test]
fn composite_recall_promotes_recent_and_important() {
    let db = db();
    seed(&db);
    let query = "deploy pipeline postgres";
    let before = legacy_ids(&db, query);
    let after = composite_ids(&db, query);

    assert_eq!(
        after,
        vec!["r2", "r1", "r3", "r4"],
        "composite: fresh + important r2 leads; relevance keeps r1 over r3/r4"
    );

    // Ordering metric (fixture): the rank of the fresh+important record
    // improves 2 → 1; the stale+weak record stays last.
    let rank = |ids: &[String], id: &str| ids.iter().position(|x| x == id).unwrap() + 1;
    assert_eq!(rank(&before, "r2"), 2);
    assert_eq!(rank(&after, "r2"), 1);
    assert!(rank(&after, "r2") < rank(&before, "r2"), "metric improves");
    assert_eq!(rank(&after, "r4"), 4, "stale + weak stays last");
}

#[test]
fn uniform_signals_keep_legacy_order() {
    // Same age + priority → composite order equals the legacy order
    // (neutrality: the opt-in only re-ranks when recency/importance differ).
    let db = db();
    put_l1(
        &db,
        &aged(
            "u1",
            "deploy pipeline postgres",
            "2026-01-31T00:00:00.000Z",
            80,
        ),
    );
    put_l1(
        &db,
        &aged("u2", "deploy pipeline", "2026-01-31T00:00:00.000Z", 80),
    );
    assert_eq!(
        composite_ids(&db, "deploy pipeline postgres"),
        vec!["u1", "u2"]
    );
    assert_eq!(
        legacy_ids(&db, "deploy pipeline postgres"),
        vec!["u1", "u2"]
    );
}

#[test]
fn scored_pass_leaves_legacy_path_untouched() {
    // Opt-in by construction: running the scored pass does not change what
    // the legacy entry point returns on the same session.
    let db = db();
    seed(&db);
    let query = "deploy pipeline postgres";
    let before = legacy_ids(&db, query);
    let _ = composite_ids(&db, query);
    let after = legacy_ids(&db, query);
    assert_eq!(before, after);
}
