// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-61 — Dreaming consolidation idle (integration test).
//!
//! Verifies the contract from `docs/plans/2026-08-29-full-backlog-parallel.md`:
//!   - `cargo test -p vanta-memory --test dreaming 2>&1 | Select-String
//!     "ok|PASS" | Measure-Object | Select-Object Count` >= 1
//!
//! AND the four invariants from `core/dream/mod.rs`:
//!   1. Idle detection works against a real clock.
//!   2. Duplicates land in `dream/<session>/<run_id>` while `l1/<session>`
//!      stays byte-identical.
//!   3. Contradiction provenance is emitted via MEM-60 without mutating
//!      the original L1 records.
//!   4. Relative dates are normalized to absolute ISO-8601.

use serde_json::json;
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::dream::{
    consolidate_session, discard_dream_run, list_dream_runs, load_dream_run, merge_duplicates,
    normalize_relative_dates, promote_dream_run, resolve_contradictions, DreamConfig,
};

fn open_db() -> VantaEmbedded {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

fn put_record(db: &VantaEmbedded, session_id: &str, r: &MemoryRecord) {
    let ns = format!("l1/{}", session_id);
    db.put(VantaMemoryInput {
        namespace: ns,
        key: r.id.clone(),
        payload: serde_json::to_string(r).expect("serialize"),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put l1");
}

fn fixture(id: &str, scene: &str, content: &str, priority: i32) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority,
        scene_name: scene.into(),
        source_message_ids: vec![],
        metadata: json!(null),
        timestamps: vec![],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
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

/// Read every record under `l1/<session>` and return them sorted by id (for
/// stable byte-identity comparisons across run boundaries).
fn read_l1(db: &VantaEmbedded, session_id: &str) -> Vec<MemoryRecord> {
    use vanta_memory::core::record::read_session_records;
    let mut records = read_session_records(db, session_id).expect("read l1");
    records.sort_by(|a, b| a.id.cmp(&b.id));
    records
}

/// 1) Idle detection + run is persisted to `dream/<session>/<run_id>`.
#[test]
fn dream_idle_detected_after_threshold_and_run_persisted() {
    let db = open_db();
    let session_id = "sess-idle-1";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - (60 * 60 * 1000); // 1 hour idle (>= 10 min default)

    put_record(
        &db,
        session_id,
        &fixture("m1", "ui", "user prefers dark mode", 80),
    );

    let config = DreamConfig::default().with_run_id_salt("test-idle");
    let run =
        consolidate_session(&db, session_id, now_ms, last_active_ms, &config).expect("idle run");
    assert_eq!(run.session_id, session_id);
    assert_eq!(run.inputs_scanned, 1);
    assert_eq!(run.runner_label, "none", "no LLM runner configured");
    assert_eq!(run.run_id.len(), 16);
    assert_eq!(run.merged_ids.len(), 0);
    assert_eq!(run.contradicted_ids.len(), 0);
    assert_eq!(run.normalized_count, 0);

    // The run shows up in list_dream_runs.
    let listed = list_dream_runs(&db, session_id).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].run_id, run.run_id);

    // load_dream_run roundtrips.
    let loaded = load_dream_run(&db, session_id, &run.run_id)
        .expect("load")
        .expect("some");
    assert_eq!(loaded.session_id, session_id);
    assert_eq!(loaded.inputs_scanned, 1);
}

/// 2) Non-idle window short-circuits with an error — no run written.
#[test]
fn dream_short_circuits_when_not_idle() {
    let db = open_db();
    let session_id = "sess-active";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60_000; // 1 minute — under default 10 min
    let config = DreamConfig::default();
    let err = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("not idle"), "explicit error: {msg}");
    assert!(list_dream_runs(&db, session_id).unwrap().is_empty());
}

/// 3) Duplicate merge: two near-identical records produce a `merged_ids`
///    entry on the dream run; `l1/<session>` is byte-identical before/after.
#[test]
fn dream_merge_duplicates_persists_to_separate_namespace_and_keeps_l1_intact() {
    let db = open_db();
    let session_id = "sess-dup";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let a = fixture("m1", "ui", "user prefers dark mode", 80);
    let b = fixture("m2", "ui", "user prefers dark mode", 80); // exact same content
    let before = [a.clone(), b.clone()];
    put_record(&db, session_id, &a);
    put_record(&db, session_id, &b);

    let config = DreamConfig::default().with_run_id_salt("test-dup");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();
    assert_eq!(run.inputs_scanned, 2);
    assert!(
        run.merged_ids.contains(&"m1".to_string()) && run.merged_ids.contains(&"m2".to_string()),
        "both duplicates recorded: {:?}",
        run.merged_ids
    );

    // L1 store is byte-identical to before.
    let after = read_l1(&db, session_id);
    assert_eq!(before.len(), after.len(), "no records added to l1");
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b, a, "l1 record mutated");
    }
}

/// 4) Contradiction resolution: lower-priority record is detected via
///    `mark_contradiction` (MEM-60), but the `l1/<session>` record is NOT
///    mutated (it stays with `superseded_by = None` until promotion).
#[test]
fn dream_resolves_contradiction_without_touching_original_l1() {
    let db = open_db();
    let session_id = "sess-contradict";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let winner = fixture("m_new", "ui", "user prefers dark mode", 90);
    let mut loser = fixture("m_old", "ui", "user prefers dark mode", 50);
    loser.created_at = "2026-08-20T09:00:00.000Z".into();
    loser.updated_at = "2026-08-20T09:00:00.000Z".into();
    assert!(loser.superseded_by.is_none(), "loser starts live");

    put_record(&db, session_id, &winner);
    put_record(&db, session_id, &loser);

    let config = DreamConfig::default().with_run_id_salt("test-contradict");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();
    assert_eq!(run.contradicted_ids.len(), 1);
    assert_eq!(run.contradicted_ids[0].old_key, "m_old");
    assert_eq!(run.contradicted_ids[0].new_key, "m_new");

    // Loser is STILL live in l1 — no mutation in the source of truth.
    let after = read_l1(&db, session_id);
    let loser_after = after
        .iter()
        .find(|r| r.id == "m_old")
        .expect("loser still in l1");
    assert!(
        loser_after.superseded_by.is_none(),
        "l1 loser MUST stay live until promotion (MEM-65)"
    );
}

/// 5) Relative-date normalization produces an absolute ISO-8601 in the
///    dream-side `metadata.activity_start_time` while the L1 original keeps
///    the raw relative phrase.
#[test]
fn dream_normalizes_relative_dates_into_dream_namespace() {
    let db = open_db();
    let session_id = "sess-dates";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let mut r = fixture("m1", "ui", "wake-up signal", 80);
    r.metadata = json!({ "activity_start_time": "ayer" });
    let before_raw = r
        .metadata
        .as_object()
        .unwrap()
        .get("activity_start_time")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(before_raw, "ayer");

    put_record(&db, session_id, &r);

    let config = DreamConfig::default().with_run_id_salt("test-dates");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();
    assert_eq!(run.normalized_count, 1, "1 record normalized");

    // L1 still has the raw "ayer" string (no mutation).
    let after = read_l1(&db, session_id);
    assert_eq!(
        after[0]
            .metadata
            .as_object()
            .unwrap()
            .get("activity_start_time")
            .unwrap()
            .as_str()
            .unwrap(),
        "ayer",
        "l1 keeps raw relative phrase"
    );

    // Dream-side copy has the absolute ISO-8601.
    let loaded = load_dream_run(&db, session_id, &run.run_id)
        .unwrap()
        .unwrap();
    assert_eq!(loaded.consolidated.len(), 1);
    let dream_meta = loaded.consolidated[0]
        .metadata
        .as_object()
        .expect("dream meta is object");
    let absolute = dream_meta
        .get("activity_start_time")
        .expect("dream has activity_start_time")
        .as_str()
        .expect("string");
    assert_eq!(absolute.len(), 24, "ISO-8601 with ms: {absolute}");
    assert!(absolute.ends_with('Z'), "UTC suffix: {absolute}");
    assert_ne!(absolute, "ayer", "normalized away from the raw phrase");
}

/// 6) Discard removes the dream run; promote returns the count without
///    mutating l1.
#[test]
fn dream_discard_removes_run_and_promote_returns_count_without_l1_mutation() {
    let db = open_db();
    let session_id = "sess-discard";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let a = fixture("m1", "ui", "x", 80);
    let b = fixture("m2", "ui", "x", 80);
    put_record(&db, session_id, &a);
    put_record(&db, session_id, &b);
    let before = read_l1(&db, session_id);

    let config = DreamConfig::default().with_run_id_salt("test-discard");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();

    // promote returns the count of consolidated records (2 here).
    let promote_count = promote_dream_run(&db, session_id, &run.run_id).unwrap();
    assert_eq!(promote_count, 2);
    let after_promote = read_l1(&db, session_id);
    assert_eq!(
        before, after_promote,
        "promote MUST NOT mutate l1 (pre-mortem)"
    );

    // discard removes the run from list_dream_runs.
    discard_dream_run(&db, session_id, &run.run_id).unwrap();
    let listed = list_dream_runs(&db, session_id).unwrap();
    assert!(listed.is_empty(), "discard removed the run");
}

/// 7) merge_duplicates + resolve_contradictions pure-function sanity checks.
#[test]
fn dream_pure_functions_match_documented_contract() {
    let a = fixture("a", "ui", "same", 80);
    let b = fixture("b", "ui", "same", 80);
    let c = fixture("c", "ui", "different", 80);
    let groups = merge_duplicates(&[a, b, c]);
    assert_eq!(groups.len(), 1, "only the first two share a shingle");
    assert_eq!(groups[0].record_ids.len(), 2);

    let high = fixture("hi", "ui", "x", 90);
    let low = fixture("lo", "ui", "x", 50);
    let p = resolve_contradictions(&[high, low], 1_700_000_000_000);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].new_key, "hi");

    let mut r = fixture("d", "ui", "y", 50);
    r.metadata = json!({ "activity_start_time": "hace 2 días" });
    let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
    assert!(n.absolute.len() == 24);
}
