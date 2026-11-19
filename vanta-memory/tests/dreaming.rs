// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-61 — Dreaming consolidation idle (integration test).
//!
//! Verifies the contract from `docs/dev/plans/2026-08-29-full-backlog-parallel.md`
//! plus the VER-07 promotion contract (`docs/dev/plans/2026-09-26-master-roadmap.md`
//! Task 34):
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
//!
//! AND the VER-07 additions: `plan_promotion` (dry-run) is deterministic and
//! never mutates L1; `promote_dream_run` applies the plan to `l1/<session>`,
//! is idempotent (re-promote → all NOOP), scopes DELETEs to the run's scanned
//! inputs and enforces a fail-closed quality gate on supersedes.

use serde_json::json;
use vantadb::config::Config;
use vantadb::sdk::{ConfidenceClass, Embedded, MemoryInput, MemoryMetadata};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::dream::{
    consolidate_session, discard_dream_run, list_dream_runs, load_dream_run, merge_duplicates,
    normalize_relative_dates, plan_promotion, promote_dream_run, resolve_contradictions,
    write_dream_run, ConsolidationError, DreamConfig, DreamRun, PromotionAction, PromotionReason,
};

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

fn put_record(db: &Embedded, session_id: &str, r: &MemoryRecord) {
    let ns = format!("l1/{}", session_id);
    db.put(MemoryInput {
        namespace: ns,
        key: r.id.clone(),
        payload: serde_json::to_string(r).expect("serialize"),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
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
fn read_l1(db: &Embedded, session_id: &str) -> Vec<MemoryRecord> {
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

    // The dream-side copy carries the supersede mark — `promote_dream_run`
    // applies it verbatim (run.json is the target state; applying the mark at
    // consolidate time is what makes re-promote a full NOOP).
    let loser_copy = run
        .consolidated
        .iter()
        .find(|r| r.id == "m_old")
        .expect("loser copy in consolidated");
    assert_eq!(
        loser_copy.superseded_by.as_deref(),
        Some("m_new"),
        "dream copy must be stamped with the supersede pointer"
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

/// 6) dry-run: per-record diff, deterministic, L1 byte-identical. Real
///    promote applies the plan and is idempotent (re-promote → all NOOP).
#[test]
fn dream_dry_run_is_deterministic_and_promote_applies_idempotently() {
    let db = open_db();
    let session_id = "sess-promote";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let winner = fixture("m_new", "ui", "user prefers dark mode", 90);
    let mut loser = fixture("m_old", "ui", "user prefers dark mode", 50);
    loser.created_at = "2026-08-20T09:00:00.000Z".into();
    loser.updated_at = "2026-08-20T09:00:00.000Z".into();
    let mut dated = fixture("m_date", "audio", "wake-up signal", 80);
    dated.metadata = json!({ "activity_start_time": "ayer" });
    put_record(&db, session_id, &winner);
    put_record(&db, session_id, &loser);
    put_record(&db, session_id, &dated);
    let before = read_l1(&db, session_id);

    let config = DreamConfig::default().with_run_id_salt("test-promote");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();

    // Dry-run: two runs produce the identical plan; L1 stays byte-identical.
    let plan_a = plan_promotion(&db, session_id, &run.run_id).unwrap();
    let plan_b = plan_promotion(&db, session_id, &run.run_id).unwrap();
    assert_eq!(plan_a, plan_b, "double dry-run → identical plan");

    let op = |key: &str| {
        plan_a
            .ops
            .iter()
            .find(|o| o.key == key)
            .unwrap_or_else(|| panic!("no op for {key}: {:?}", plan_a.ops))
    };
    assert_eq!(op("m_new").action, PromotionAction::Noop);
    assert_eq!(op("m_new").reason, PromotionReason::Unchanged);
    assert_eq!(
        (op("m_old").action, op("m_old").reason),
        (PromotionAction::Update, PromotionReason::Supersede)
    );
    assert_eq!(
        (op("m_date").action, op("m_date").reason),
        (PromotionAction::Update, PromotionReason::Normalize)
    );
    assert_eq!(plan_a.ops[0].namespace, format!("l1/{session_id}"));
    assert_eq!(
        before,
        read_l1(&db, session_id),
        "dry-run must not mutate l1"
    );
    assert_eq!(
        run.consolidated
            .iter()
            .find(|r| r.id == "m_old")
            .unwrap()
            .superseded_by
            .as_deref(),
        Some("m_new"),
        "dry-run planning relies on the stamped dream copy"
    );

    // Apply: same plan, real mutation.
    let applied = promote_dream_run(&db, session_id, &run.run_id).unwrap();
    assert_eq!(applied.ops, plan_a.ops, "apply must apply the dry-run plan");
    let counts = applied.counts();
    assert_eq!((counts.add, counts.update), (0, 2));
    assert_eq!((counts.delete, counts.noop), (0, 1));

    let after = read_l1(&db, session_id);
    assert_eq!(
        after
            .iter()
            .find(|r| r.id == "m_old")
            .unwrap()
            .superseded_by
            .as_deref(),
        Some("m_new"),
        "supersede must be applied to l1"
    );
    assert_ne!(
        after
            .iter()
            .find(|r| r.id == "m_date")
            .unwrap()
            .metadata
            .as_object()
            .unwrap()
            .get("activity_start_time")
            .unwrap()
            .as_str()
            .unwrap(),
        "ayer",
        "normalized date must be applied to l1"
    );

    // Idempotent: re-promote → every op NOOP and zero writes.
    let before_second = read_l1(&db, session_id);
    let second = promote_dream_run(&db, session_id, &run.run_id).unwrap();
    assert!(
        second.ops.iter().all(|o| o.action == PromotionAction::Noop),
        "re-promote must be all-NOOP, got: {:?}",
        second.ops
    );
    assert_eq!(
        read_l1(&db, session_id),
        before_second,
        "re-promote must not write to l1"
    );
}

/// 6b) DELETE is scoped to the run's scanned inputs (`input_ids`): a record the
///     run scanned but the consolidated view dropped is deleted (reason Dedup);
///     a record added to L1 after the run is never part of the plan.
#[test]
fn dream_promote_delete_is_scoped_to_scanned_inputs() {
    let db = open_db();
    let session_id = "sess-delete-scope";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    let a = fixture("a", "ui", "keep me", 80);
    let b = fixture("b", "ui", "merged away", 80);
    put_record(&db, session_id, &a);
    put_record(&db, session_id, &b);

    let config = DreamConfig::default().with_run_id_salt("test-delete");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();
    assert!(run.input_ids.contains(&"a".to_string()) && run.input_ids.contains(&"b".to_string()));

    // Simulate an LLM runner that merged "b" into "a" (dropped from the view).
    let mut trimmed = run.clone();
    trimmed.consolidated.retain(|r| r.id != "b");
    write_dream_run(&db, &trimmed).unwrap();

    // A record added AFTER the run must survive the promotion untouched.
    let c = fixture("c", "ui", "added after the run", 80);
    put_record(&db, session_id, &c);

    let plan = plan_promotion(&db, session_id, &trimmed.run_id).unwrap();
    assert!(
        plan.ops.iter().any(|o| o.key == "b"
            && o.action == PromotionAction::Delete
            && o.reason == PromotionReason::Dedup),
        "scanned+merged record must be a DELETE(dedup): {:?}",
        plan.ops
    );
    assert!(
        !plan.ops.iter().any(|o| o.key == "c"),
        "post-run additions must not appear in the plan"
    );

    let applied = promote_dream_run(&db, session_id, &trimmed.run_id).unwrap();
    assert_eq!(applied.counts().delete, 1);
    let after = read_l1(&db, session_id);
    assert!(!after.iter().any(|r| r.id == "b"), "merged record deleted");
    assert!(after.iter().any(|r| r.id == "a"), "keeper survives");
    assert!(
        after.iter().any(|r| r.id == "c"),
        "post-run addition survives"
    );
}

/// 6c) Legacy runs persisted before `input_ids` existed (run.json JSON without
///     the field → `#[serde(default)]` = empty) never delete anything —
///     conservative upgrade path.
#[test]
fn dream_promote_legacy_run_without_input_ids_never_deletes() {
    let db = open_db();
    let session_id = "sess-legacy";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    put_record(&db, session_id, &fixture("a", "ui", "keep me", 80));
    put_record(
        &db,
        session_id,
        &fixture("b", "ui", "legacy drop candidate", 80),
    );

    let config = DreamConfig::default().with_run_id_salt("test-legacy");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();

    // Simulate a pre-VER-07 run.json: serialize the run, DELETE the
    // `input_ids` key entirely, drop "b" from the view, persist the raw JSON.
    let mut legacy_json = serde_json::to_value(&run).unwrap();
    legacy_json
        .as_object_mut()
        .unwrap()
        .remove("input_ids")
        .expect("input_ids present in the serialized run");
    legacy_json["consolidated"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["id"] != "b");
    let ns = format!("dream/{session_id}/{}", run.run_id);
    db.put(MemoryInput {
        namespace: ns,
        key: "run.json".into(),
        payload: serde_json::to_string(&legacy_json).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("write legacy run.json");

    // Deserialization must default the missing field, never fail.
    let loaded = load_dream_run(&db, session_id, &run.run_id)
        .unwrap()
        .unwrap();
    assert!(
        loaded.input_ids.is_empty(),
        "missing field must deserialize as empty (serde default)"
    );

    let plan = plan_promotion(&db, session_id, &run.run_id).unwrap();
    assert!(
        !plan.ops.iter().any(|o| o.action == PromotionAction::Delete),
        "no input_ids → no DELETE: {:?}",
        plan.ops
    );
    promote_dream_run(&db, session_id, &run.run_id).unwrap();
    assert!(
        read_l1(&db, session_id).iter().any(|r| r.id == "b"),
        "unscanned legacy record must survive"
    );
}

/// 6d) Quality gate (fail-closed): a plan whose supersede target would not
///     exist after apply is rejected — by dry-run AND apply alike, with zero
///     mutation; duplicate keys in the view are rejected too.
#[test]
fn dream_promote_quality_gate_blocks_dangling_supersede_and_duplicate_keys() {
    let db = open_db();
    let session_id = "sess-gate";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    put_record(&db, session_id, &fixture("m_new", "ui", "winner", 90));
    let mut loser = fixture("m_old", "ui", "winner", 50);
    loser.created_at = "2026-08-20T09:00:00.000Z".into();
    put_record(&db, session_id, &loser);

    let config = DreamConfig::default().with_run_id_salt("test-gate");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();

    // Dangling supersede: drop the winner from the view while the loser still
    // points at it — applying would delete the winner and leave a dead pointer.
    let mut dangling = run.clone();
    dangling.consolidated.retain(|r| r.id != "m_new");
    write_dream_run(&db, &dangling).unwrap();
    let before = read_l1(&db, session_id);
    assert!(matches!(
        plan_promotion(&db, session_id, &dangling.run_id).unwrap_err(),
        ConsolidationError::QualityGate(_)
    ));
    assert!(matches!(
        promote_dream_run(&db, session_id, &dangling.run_id).unwrap_err(),
        ConsolidationError::QualityGate(_)
    ));
    assert_eq!(before, read_l1(&db, session_id), "gate must not mutate l1");

    // Duplicate keys in the consolidated view are rejected (fail-closed).
    let mut dup = run.clone();
    let first = dup.consolidated[0].clone();
    dup.consolidated.push(first);
    write_dream_run(&db, &dup).unwrap();
    assert!(matches!(
        plan_promotion(&db, session_id, &dup.run_id).unwrap_err(),
        ConsolidationError::QualityGate(_)
    ));
}

/// 6e) Discard removes the dream run from `dream_list`; L1 is untouched.
#[test]
fn dream_discard_removes_run_without_touching_l1() {
    let db = open_db();
    let session_id = "sess-discard";
    let now_ms = 1_700_000_000_000;
    let last_active_ms = now_ms - 60 * 60 * 1000;

    put_record(&db, session_id, &fixture("m1", "ui", "x", 80));
    put_record(&db, session_id, &fixture("m2", "ui", "x", 80));
    let before = read_l1(&db, session_id);

    let config = DreamConfig::default().with_run_id_salt("test-discard");
    let run = consolidate_session(&db, session_id, now_ms, last_active_ms, &config).unwrap();
    assert_eq!(list_dream_runs(&db, session_id).unwrap().len(), 1);

    discard_dream_run(&db, session_id, &run.run_id).unwrap();
    assert!(list_dream_runs(&db, session_id).unwrap().is_empty());
    assert_eq!(
        before,
        read_l1(&db, session_id),
        "discard must not touch l1"
    );
}

/// 6f) UPDATE must not drop an existing L1 vector: when the dream-side copy
///     has `vector: None` (e.g. the node gained one after the run), the apply
///     falls back to the current node's vector instead of rewriting without it.
#[test]
fn dream_promote_update_falls_back_to_current_l1_vector() {
    let db = open_db();
    let session_id = "sess-vector";
    let now_ms = 1_700_000_000_000;

    // L1 record carrying a vector; payload as scanned (no normalized metadata).
    let record = fixture("m1", "ui", "vectorized memory", 80);
    db.put(MemoryInput {
        namespace: format!("l1/{session_id}"),
        key: "m1".into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: Some(vec![0.1, 0.2, 0.3]),
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put l1 with vector");

    // Hand-built run whose consolidated copy differs (metadata) and carries no
    // vector — promotion must update the payload but keep the node vector.
    let mut consolidated = record.clone();
    consolidated.metadata = json!({ "activity_start_time": "2023-11-14T22:13:20.000Z" });
    let run = DreamRun {
        run_id: "vector-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 1,
        input_ids: vec!["m1".into()],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 1,
        runner_label: "none".into(),
        consolidated: vec![consolidated],
    };
    write_dream_run(&db, &run).unwrap();

    let applied = promote_dream_run(&db, session_id, &run.run_id).unwrap();
    let counts = applied.counts();
    assert_eq!((counts.update, counts.noop), (1, 0));

    let after = read_l1(&db, session_id);
    let rec = after.iter().find(|r| r.id == "m1").expect("m1 survives");
    assert_eq!(
        rec.vector,
        Some(vec![0.1, 0.2, 0.3]),
        "UPDATE must preserve the node vector when the dream copy has none"
    );
    assert!(
        rec.metadata
            .as_object()
            .unwrap()
            .get("activity_start_time")
            .is_some(),
        "the payload diff must still be applied"
    );
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

/// FIND-658 (a) RED: write-first on `PromotionAction::Update` — if the upsert
/// fails (here: wrong vector dimension against an established 3-d index), the
/// pre-existing L1 record must survive untouched. The old `db.delete` →
/// `put_record` order permanently deleted `m1` before the put errored.
#[test]
fn dream_promote_update_write_first_preserves_record_on_put_failure() {
    let db = open_db();
    let session_id = "sess-wf";
    let now_ms = 1_700_000_000_000;

    let original = fixture("m1", "ui", "original memory", 80);
    db.put(MemoryInput {
        namespace: format!("l1/{session_id}"),
        key: "m1".into(),
        payload: serde_json::to_string(&original).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: Some(vec![0.1, 0.2, 0.3]),
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed m1 fixing index dim = 3");

    let mut bad_update = original.clone();
    bad_update.content = "updated content".into();
    bad_update.vector = Some(vec![0.1, 0.2, 0.3, 0.4]); // wrong dim → put error

    let run = DreamRun {
        run_id: "wf-fail-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 1,
        input_ids: vec!["m1".into()],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "none".into(),
        consolidated: vec![bad_update],
    };
    write_dream_run(&db, &run).unwrap();

    let err = promote_dream_run(&db, session_id, &run.run_id);
    assert!(err.is_err(), "wrong-dim vector must fail promotion");

    let after = read_l1(&db, session_id);
    let surviving = after
        .iter()
        .find(|r| r.id == "m1")
        .expect("FIND-658: write-first must preserve pre-existing m1 when upsert fails");
    assert_eq!(surviving.content, "original memory");
    assert_eq!(surviving.vector, Some(vec![0.1, 0.2, 0.3]));
}

/// FIND-658 (b) RED: dream promotion (`Add` + `Delete` merge and `Update`
/// supersede) with real high-cosine embeddings (>= 0.92 FastGate threshold)
/// must force `DedupPolicy::Off` so FastGate `NoopReinforce` neither drops
/// the new merged record before old targets are deleted nor skips persisting
/// `superseded_by` on a contradicted record.
#[test]
fn dream_promote_with_high_cosine_embeddings_does_not_lose_or_skip_records() {
    let db = open_db();
    let session_id = "sess-cos-dream";
    let now_ms = 1_700_000_000_000;
    let ns = format!("l1/{session_id}");

    let v_merge_a = vec![0.99_f32, 0.10, 0.05, 0.02];
    let v_merge_b = vec![0.99_f32, 0.11, 0.04, 0.02];
    let v_merged = vec![0.99_f32, 0.105, 0.045, 0.02]; // cos > 0.999 with a & b

    let v_win = vec![0.05_f32, 0.99, 0.10, 0.02];
    let v_lose = vec![0.04_f32, 0.99, 0.11, 0.02]; // cos > 0.999 with v_win

    let m1 = fixture("m1", "ui", "user likes dark mode", 80);
    let m2 = fixture("m2", "ui", "user prefers dark theme", 80);
    let winner = fixture("m_win", "prefs", "user hates coffee now", 90);
    let loser = fixture("m_lose", "prefs", "user loves coffee", 50);

    for (rec, vec) in [
        (&m1, v_merge_a),
        (&m2, v_merge_b),
        (&winner, v_win.clone()),
        (&loser, v_lose.clone()),
    ] {
        db.put(MemoryInput {
            namespace: ns.clone(),
            key: rec.id.clone(),
            payload: serde_json::to_string(rec).unwrap(),
            metadata: MemoryMetadata::new(),
            vector: Some(vec),
            ..Default::default()
        })
        .expect("seed record with vector");
    }

    let mut merged = fixture("m_merged", "ui", "user prefers dark mode theme", 85);
    merged.vector = Some(v_merged.clone());

    let mut loser_sup = loser.clone();
    loser_sup.superseded_by = Some("m_win".into());
    loser_sup.vector = Some(v_lose.clone());

    let mut winner_keep = winner.clone();
    winner_keep.vector = Some(v_win);

    let run = DreamRun {
        run_id: "cos-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 4,
        input_ids: vec!["m1".into(), "m2".into(), "m_win".into(), "m_lose".into()],
        merged_ids: vec!["m_merged".into()],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "none".into(),
        consolidated: vec![merged, winner_keep, loser_sup],
    };
    write_dream_run(&db, &run).unwrap();

    promote_dream_run(&db, session_id, &run.run_id).expect("promote");

    let after = read_l1(&db, session_id);
    let merged_rec = after
        .iter()
        .find(|r| r.id == "m_merged")
        .expect("FIND-658: m_merged must survive (must not be dropped by NoopReinforce)");
    assert_eq!(merged_rec.vector, Some(v_merged));
    assert!(!after.iter().any(|r| r.id == "m1" || r.id == "m2"));

    let loser_after = after
        .iter()
        .find(|r| r.id == "m_lose")
        .expect("m_lose preserved");
    assert_eq!(
        loser_after.superseded_by.as_deref(),
        Some("m_win"),
        "FIND-658: m_lose superseded_by must be persisted despite cos >= 0.92 with m_win"
    );
}

/// FIND-658 (c) RED: `PromotionAction::Update` must preserve the target's
/// engine-level `confidence`, `confidence_class`, and `derived_from` across a
/// real Fjall close/reopen instead of resetting them to `Asserted` / `1.0`.
#[test]
fn dream_promote_update_preserves_confidence_asserted_and_derived() {
    use vantadb::sdk::{ConfidenceClass, DERIVATION_DISCOUNT};

    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().to_string_lossy().to_string();
    let session_id = "sess-conf-dream";
    let ns = format!("l1/{session_id}");
    let now_ms = 1_700_000_000_000;
    let expected_derived = (0.6_f32 * DERIVATION_DISCOUNT).clamp(0.0, 1.0);

    {
        let db = Embedded::open_with_config(Config {
            storage_path: db_path.clone(),
            backend_kind: BackendKind::Fjall,
            ..Config::default()
        })
        .expect("open fjall");

        let m_asserted = fixture("m_asserted", "ui", "decayed preference", 80);
        let m_parent = fixture("m_parent", "ui", "parent fact", 80);
        let m_derived = fixture("m_derived", "ui", "derived preference", 80);

        db.put(MemoryInput {
            namespace: ns.clone(),
            key: "m_asserted".into(),
            payload: serde_json::to_string(&m_asserted).unwrap(),
            metadata: MemoryMetadata::new(),
            confidence_class: Some(ConfidenceClass::Asserted),
            confidence: Some(0.25),
            ..Default::default()
        })
        .expect("seed m_asserted at 0.25");

        db.put(MemoryInput {
            namespace: ns.clone(),
            key: "m_parent".into(),
            payload: serde_json::to_string(&m_parent).unwrap(),
            metadata: MemoryMetadata::new(),
            confidence_class: Some(ConfidenceClass::Asserted),
            confidence: Some(0.6),
            ..Default::default()
        })
        .expect("seed m_parent at 0.6");

        db.put(MemoryInput {
            namespace: ns.clone(),
            key: "m_derived".into(),
            payload: serde_json::to_string(&m_derived).unwrap(),
            metadata: MemoryMetadata::new(),
            confidence_class: Some(ConfidenceClass::Derived),
            confidence: None,
            derived_from: Some(vec!["m_parent".into()]),
            ..Default::default()
        })
        .expect("seed m_derived");

        let mut asserted_upd = m_asserted.clone();
        asserted_upd.metadata = json!({ "activity_start_time": "2023-11-14T22:13:20.000Z" });
        let mut derived_upd = m_derived.clone();
        derived_upd.metadata = json!({ "activity_start_time": "2023-11-14T22:13:20.000Z" });

        let run = DreamRun {
            run_id: "conf-run".into(),
            session_id: session_id.into(),
            started_at_ms: now_ms,
            ended_at_ms: now_ms,
            inputs_scanned: 3,
            input_ids: vec!["m_asserted".into(), "m_parent".into(), "m_derived".into()],
            merged_ids: vec![],
            contradicted_ids: vec![],
            normalized_count: 2,
            runner_label: "none".into(),
            consolidated: vec![asserted_upd, m_parent, derived_upd],
        };
        write_dream_run(&db, &run).unwrap();

        let applied = promote_dream_run(&db, session_id, &run.run_id).expect("promote");
        assert_eq!(applied.counts().update, 2);
    }

    let db = Embedded::open_with_config(Config {
        storage_path: db_path,
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    })
    .expect("reopen fjall");

    let stored_asserted = db
        .get(&ns, "m_asserted")
        .expect("get")
        .expect("m_asserted exists");
    assert_eq!(stored_asserted.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(
        stored_asserted.confidence, 0.25,
        "FIND-658: PromotionAction::Update must preserve decayed Asserted confidence"
    );

    let stored_derived = db
        .get(&ns, "m_derived")
        .expect("get")
        .expect("m_derived exists");
    assert_eq!(stored_derived.confidence_class, ConfidenceClass::Derived);
    assert_eq!(stored_derived.derived_from, vec!["m_parent".to_string()]);
    assert!(
        (stored_derived.confidence - expected_derived).abs() < 1e-6,
        "FIND-658: PromotionAction::Update must preserve Derived confidence {expected_derived}, got {}",
        stored_derived.confidence
    );
}

// ── FIND-658: promotion UPDATE is write-first and preserves the engine-side
// confidence state ─────────────────────────────────────────────────────
//
// Two sibling defects in `promote_dream_run`'s UPDATE arm:
//   (a) `delete` ran BEFORE `put` — a crash/failure in the window lost the
//       L1 record (same class as MEMG-27, whose `persist_planned` upserts
//       first and prunes only surviving non-coincident keys after `Ok`).
//   (b) the re-put declared `Asserted` + `default_confidence()` and no
//       `derived_from` — a feedback-decayed confidence was resurrected at
//       1.0 and a derived record lost its lineage (same class as FIND-281).

/// (b-i) An UPDATE must keep the target's stored confidence/class — the L1
/// payload does not carry them, so they must be read off the stored engine
/// record before the rewrite.
#[test]
fn dream_promotion_update_preserves_engine_confidence_state() {
    let db = open_db();
    let session_id = "sess-preserve-conf";
    let now_ms = 1_700_000_000_000;

    // Seed at a decayed confidence via a direct SDK write — mirrors the
    // FIND-281 fixture (e.g. after feedback decayed the record to 0.3).
    let record = fixture("m1", "ui", "user prefers dark mode", 80);
    db.put(MemoryInput {
        namespace: format!("l1/{session_id}"),
        key: "m1".into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        confidence_class: Some(ConfidenceClass::Asserted),
        confidence: Some(0.3),
        ..Default::default()
    })
    .expect("seed at 0.3");

    // Consolidated copy differing in content → UPDATE(Content).
    let mut consolidated = record.clone();
    consolidated.content = "user prefers light mode".into();
    let run = DreamRun {
        run_id: "conf-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 1,
        input_ids: vec!["m1".into()],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "none".into(),
        consolidated: vec![consolidated],
    };
    write_dream_run(&db, &run).unwrap();

    let applied = promote_dream_run(&db, session_id, &run.run_id).unwrap();
    assert!(applied
        .ops
        .iter()
        .any(|o| o.key == "m1" && o.action == PromotionAction::Update));

    let stored = db
        .get(&format!("l1/{session_id}"), "m1")
        .expect("get")
        .expect("m1 present");
    assert_eq!(
        stored.confidence, 0.3,
        "decayed confidence must be preserved, not resurrected at 1.0"
    );
    assert_eq!(stored.confidence_class, ConfidenceClass::Asserted);
    let payload: MemoryRecord = serde_json::from_str(&stored.payload).expect("l1 payload");
    assert_eq!(
        payload.content, "user prefers light mode",
        "the payload diff must still be applied"
    );
}

/// (b-ii) An UPDATE on a Derived record must re-declare its parent list
/// (V1 requires non-empty `derived_from`) and must NOT re-declare its
/// stored score (D4b — the engine recomputes it from the parents). Pre-fix
/// the rewrite flipped the class to `Asserted` at 1.0 and dropped the
/// lineage.
#[test]
fn dream_promotion_update_preserves_derived_lineage() {
    let db = open_db();
    let session_id = "sess-preserve-derived";
    let now_ms = 1_700_000_000_000;

    let parent = fixture("p1", "ui", "parent fact", 80);
    put_record(&db, session_id, &parent);

    let child = fixture("m_der", "ui", "derived insight", 80);
    db.put(MemoryInput {
        namespace: format!("l1/{session_id}"),
        key: "m_der".into(),
        payload: serde_json::to_string(&child).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        confidence_class: Some(ConfidenceClass::Derived),
        derived_from: Some(vec!["p1".into()]),
        ..Default::default()
    })
    .expect("seed derived");
    let seeded = db
        .get(&format!("l1/{session_id}"), "m_der")
        .expect("get")
        .expect("seeded derived");
    let seeded_score = seeded.confidence;
    assert!(
        seeded_score < 1.0,
        "sanity: derived score is discounted from the parent, got {seeded_score}"
    );

    let mut consolidated = child.clone();
    consolidated.content = "derived insight v2".into();
    let run = DreamRun {
        run_id: "derived-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 1,
        input_ids: vec!["m_der".into()],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "none".into(),
        consolidated: vec![consolidated],
    };
    write_dream_run(&db, &run).unwrap();
    promote_dream_run(&db, session_id, &run.run_id).expect("promote");

    let stored = db
        .get(&format!("l1/{session_id}"), "m_der")
        .expect("get")
        .expect("m_der present");
    assert_eq!(
        stored.confidence_class,
        ConfidenceClass::Derived,
        "provenance class must be preserved"
    );
    assert_eq!(
        stored.derived_from,
        vec!["p1".to_string()],
        "parent lineage must be preserved"
    );
    assert_eq!(
        stored.confidence, seeded_score,
        "score is recomputed from the same parents — identical to stored"
    );
}

/// (a) Write-first: an UPDATE whose `put` fails must leave the
/// pre-promotion record intact. The L1 record carries a 3-dim vector; the
/// consolidated rewrite carries a 4-dim one — FIND-411 dimension
/// validation rejects the `put` up-front. Under the old delete-first order
/// the record was already gone.
#[test]
fn dream_promotion_failed_update_put_leaves_old_record() {
    let db = open_db();
    let session_id = "sess-write-first";
    let now_ms = 1_700_000_000_000;

    // The L1 record itself establishes the index dimension (3).
    let mut record = fixture("m1", "ui", "pre-promotion content", 80);
    record.vector = Some(vec![0.1, 0.2, 0.3]);
    db.put(MemoryInput {
        namespace: format!("l1/{session_id}"),
        key: "m1".into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: record.vector.clone(),
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed m1");
    let before = db
        .get(&format!("l1/{session_id}"), "m1")
        .expect("get")
        .expect("m1")
        .payload;

    let mut consolidated = record.clone();
    consolidated.content = "post-promotion content".into();
    consolidated.vector = Some(vec![0.5; 4]); // dim mismatch → put fails
    let run = DreamRun {
        run_id: "wf-run".into(),
        session_id: session_id.into(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        inputs_scanned: 1,
        input_ids: vec!["m1".into()],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "none".into(),
        consolidated: vec![consolidated],
    };
    write_dream_run(&db, &run).unwrap();

    promote_dream_run(&db, session_id, &run.run_id)
        .expect_err("dimension mismatch must fail the promote");

    let after = db
        .get(&format!("l1/{session_id}"), "m1")
        .expect("get")
        .map(|r| r.payload);
    assert_eq!(
        after.as_deref(),
        Some(before.as_str()),
        "write-first: a failed update put must leave the pre-promotion record intact"
    );
}
