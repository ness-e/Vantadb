#![allow(clippy::expect_used, clippy::unwrap_used)]
//! SCH-05 — quarantine containment + selective abstention (ADR-046 §D5, MGR-13).
//!
//! Threat model (MGR-13 §4): quarantined content is untrusted-by-decision and
//! must never reach default retrieval (list/search/recall), only explicit
//! opt-in surfaces. Transitions T1 (write flag), T1c (import opt-in),
//! T1d (apply), T2 (promote), T4 (reject); T3 keeps the review deadline
//! (`keep`, never auto-promotes — I1); sticky across rewrites (I2).

use vantadb::config::Config;
use vantadb::{
    AbstentionReason, BackendKind, Embedded, MemoryInput, MemoryListOptions, MemorySearchRequest,
};

use std::time::{Duration, Instant};

fn in_memory_db() -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    })
    .expect("open in-memory database")
}

fn db_with_confidence_threshold(threshold: f32) -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        confidence_threshold: Some(threshold),
        ..Default::default()
    })
    .expect("open in-memory database")
}

fn db_with_review_days(days: u32) -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        quarantine_review_default_days: days,
        ..Default::default()
    })
    .expect("open in-memory database")
}

fn quarantined_input(namespace: &str, key: &str, payload: &str) -> MemoryInput {
    MemoryInput {
        quarantine: true,
        ..MemoryInput::new(namespace, key, payload)
    }
}

fn listed_keys(db: &Embedded, namespace: &str, include_quarantined: bool) -> Vec<String> {
    let page = db
        .list(
            namespace,
            MemoryListOptions {
                include_quarantined,
                limit: 100,
                ..Default::default()
            },
        )
        .expect("list");
    page.records.into_iter().map(|r| r.key).collect()
}

fn searched_keys(db: &Embedded, namespace: &str, include_quarantined: bool) -> Vec<String> {
    db.search_page(MemorySearchRequest {
        namespace: namespace.into(),
        text_query: Some("content".into()),
        top_k: 10,
        include_quarantined,
        ..Default::default()
    })
    .expect("search")
    .hits
    .into_iter()
    .map(|h| h.record.key)
    .collect()
}

// ── C1: default-exclude + opt-in + get ────────────────────────────────────

#[test]
fn quarantined_content_is_excluded_from_list_and_search_by_default() {
    let db = in_memory_db();
    db.put(MemoryInput::new("ns", "active", "content alpha"))
        .unwrap();
    db.put(quarantined_input("ns", "suspect", "content beta"))
        .unwrap();

    assert_eq!(listed_keys(&db, "ns", false), vec!["active".to_string()]);
    assert_eq!(searched_keys(&db, "ns", false), vec!["active".to_string()]);
}

#[test]
fn include_quarantined_opt_in_returns_isolated_records() {
    let db = in_memory_db();
    db.put(MemoryInput::new("ns", "active", "content alpha"))
        .unwrap();
    db.put(quarantined_input("ns", "suspect", "content beta"))
        .unwrap();

    let mut list_keys = listed_keys(&db, "ns", true);
    list_keys.sort();
    assert_eq!(list_keys, vec!["active".to_string(), "suspect".to_string()]);

    let mut search_keys = searched_keys(&db, "ns", true);
    search_keys.sort();
    assert_eq!(
        search_keys,
        vec!["active".to_string(), "suspect".to_string()]
    );
}

#[test]
fn get_returns_quarantined_record_with_visible_state() {
    let db = in_memory_db();
    db.put(quarantined_input("ns", "suspect", "content beta"))
        .unwrap();

    let record = db.get("ns", "suspect").expect("get").expect("record");
    assert!(record.quarantined_at_ms.is_some());
    assert_eq!(record.quarantine_reason.as_deref(), Some("explicit_write"));
    assert_eq!(record.quarantined_by.as_deref(), Some("system:put"));
    assert!(record.quarantine_review_due_ms.is_some());
}

// ── I2: sticky ────────────────────────────────────────────────────────────

#[test]
fn rewrite_of_quarantined_key_preserves_quarantine() {
    let db = in_memory_db();
    let first = db
        .put(quarantined_input("ns", "sticky", "content v1"))
        .unwrap();
    let second = db
        .put(MemoryInput::new("ns", "sticky", "content v2"))
        .unwrap();

    assert_eq!(
        second.version,
        first.version + 1,
        "rewrite is a real upsert"
    );
    assert_eq!(
        second.quarantined_at_ms, first.quarantined_at_ms,
        "sticky: a plain rewrite must not clear the quarantine"
    );
    assert_eq!(
        second.quarantine_reason.as_deref(),
        Some("explicit_write"),
        "sticky: the original reason survives the rewrite"
    );
    assert!(!listed_keys(&db, "ns", false).contains(&"sticky".to_string()));
}

// ── T1d/T2/T4: ops ────────────────────────────────────────────────────────

#[test]
fn quarantine_apply_marks_existing_record_with_review_deadline() {
    let db = in_memory_db();
    db.put(MemoryInput::new("ns", "manual", "content gamma"))
        .unwrap();

    let record = db
        .quarantine_apply("ns", "manual", Some("policy_match"))
        .expect("apply");
    assert!(record.quarantined_at_ms.is_some());
    assert_eq!(record.quarantine_reason.as_deref(), Some("policy_match"));
    assert_eq!(
        record.quarantined_by.as_deref(),
        Some("system:quarantine_apply")
    );
    let at = record.quarantined_at_ms.unwrap();
    assert_eq!(
        record.quarantine_review_due_ms,
        Some(at + 30 * 24 * 60 * 60 * 1000),
        "D5d: default review deadline is 30 days"
    );
    assert_eq!(listed_keys(&db, "ns", false).len(), 0);
}

#[test]
fn quarantine_promote_clears_state_without_bumping_version() {
    let db = in_memory_db();
    let entered = db
        .put(quarantined_input("ns", "promote-me", "content delta"))
        .unwrap();

    let promoted = db.quarantine_promote("ns", "promote-me").expect("promote");
    assert_eq!(promoted.quarantined_at_ms, None);
    assert_eq!(promoted.quarantine_reason, None);
    assert_eq!(promoted.quarantined_by, None);
    assert_eq!(promoted.quarantine_review_due_ms, None);
    assert_eq!(
        promoted.version, entered.version,
        "T2: state change is not a content change (version unchanged)"
    );
    assert_eq!(
        listed_keys(&db, "ns", false),
        vec!["promote-me".to_string()]
    );
}

#[test]
fn quarantine_promote_rejects_active_record() {
    let db = in_memory_db();
    db.put(MemoryInput::new("ns", "active", "content")).unwrap();
    let err = db
        .quarantine_promote("ns", "active")
        .expect_err("active record is not promotable");
    assert!(
        err.to_string().contains("not quarantined"),
        "unexpected error: {err}"
    );
}

#[test]
fn quarantine_reject_deletes_record_and_is_auditable_transition() {
    let db = in_memory_db();
    db.put(quarantined_input("ns", "reject-me", "content epsilon"))
        .unwrap();

    assert!(db.quarantine_reject("ns", "reject-me").expect("reject"));
    assert!(db
        .get("ns", "reject-me")
        .expect("get after reject")
        .is_none());
    // Rejecting a non-quarantined record is an invalid transition.
    db.put(MemoryInput::new("ns", "active", "content")).unwrap();
    assert!(db.quarantine_reject("ns", "active").is_err());
}

// ── T3: deadline config (keep; never auto-promotion) ─────────────────────

#[test]
fn quarantine_review_deadline_follows_config_and_zero_disables_it() {
    let db = db_with_review_days(0);
    let record = db
        .put(quarantined_input("ns", "no-deadline", "content"))
        .unwrap();
    assert_eq!(record.quarantine_review_due_ms, None);

    let db = db_with_review_days(1);
    let record = db
        .put(quarantined_input("ns", "one-day", "content"))
        .unwrap();
    let at = record.quarantined_at_ms.unwrap();
    assert_eq!(
        record.quarantine_review_due_ms,
        Some(at + 24 * 60 * 60 * 1000)
    );
}

#[test]
fn expired_review_deadline_never_auto_promotes() {
    let db = in_memory_db();
    let record = db
        .put(quarantined_input("ns", "waiting", "content"))
        .unwrap();
    let due = record.quarantine_review_due_ms.unwrap();
    assert!(due > record.quarantined_at_ms.unwrap());

    // Reads at any point never mutate the state (I1: no clock-based promotion).
    let _ = listed_keys(&db, "ns", true);
    let _ = searched_keys(&db, "ns", true);
    let again = db.get("ns", "waiting").expect("get").expect("record");
    assert_eq!(again.quarantined_at_ms, record.quarantined_at_ms);
}

// ── T1c: import ───────────────────────────────────────────────────────────

#[test]
fn import_with_quarantine_option_marks_unreviewed_import() {
    let src = in_memory_db();
    src.put(MemoryInput::new("imp", "k1", "content from file"))
        .unwrap();
    let record = src.get("imp", "k1").unwrap().unwrap();

    let dst = in_memory_db();
    let report = dst.import_records(vec![record], true).expect("import");
    assert_eq!(report.inserted, 1);
    assert_eq!(report.quarantined, 1);

    let imported = dst.get("imp", "k1").unwrap().unwrap();
    assert_eq!(
        imported.quarantine_reason.as_deref(),
        Some("unreviewed_import")
    );
    assert_eq!(imported.quarantined_by.as_deref(), Some("system:import"));
    assert_eq!(listed_keys(&dst, "imp", false).len(), 0);
}

#[test]
fn import_roundtrip_preserves_quarantine_state() {
    let src = in_memory_db();
    src.put(quarantined_input("rt", "suspect", "content"))
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rt.jsonl");
    src.export_namespace(&path, "rt", None).expect("export");

    let dst = in_memory_db();
    let report = dst.import_file(&path, false).expect("import");
    assert_eq!(report.inserted, 1);
    let imported = dst.get("rt", "suspect").unwrap().unwrap();
    assert!(
        imported.quarantined_at_ms.is_some(),
        "roundtrip must preserve the quarantine state"
    );
}

/// Encode records in the bulk transport format (`VDBJSON\n` + version + count).
fn bulk_payload(inputs: &[MemoryInput]) -> Vec<u8> {
    let body = serde_json::to_vec(&inputs).unwrap();
    let mut payload = Vec::new();
    payload.extend_from_slice(b"VDBJSON\n");
    payload.push(0x01);
    payload.extend_from_slice(&(inputs.len() as u64).to_le_bytes());
    payload.extend_from_slice(&body);
    payload
}

#[test]
fn bulk_import_honors_the_quarantine_write_flag() {
    let db = in_memory_db();
    let inputs = vec![
        MemoryInput::new("bulk", "plain", "content one"),
        quarantined_input("bulk", "flagged", "content two"),
    ];
    let mut reader = std::io::Cursor::new(bulk_payload(&inputs));

    let report = db.bulk_import_stream(&mut reader).expect("bulk import");
    assert_eq!(report.quarantined, 1, "one flagged record (F3 counter)");

    // Bulk is a raw transport: imported nodes are addressable by `get` but not
    // by `list` until the derived indexes are rebuilt (pre-existing contract).
    let flagged = db.get("bulk", "flagged").unwrap().unwrap();
    assert!(flagged.quarantined_at_ms.is_some());
    assert_eq!(flagged.quarantine_reason.as_deref(), Some("explicit_write"));
    assert_eq!(
        flagged.quarantined_by.as_deref(),
        Some("system:bulk_import")
    );
    assert!(flagged.quarantine_review_due_ms.is_some());
    let plain = db.get("bulk", "plain").unwrap().unwrap();
    assert!(
        plain.quarantined_at_ms.is_none(),
        "the flag is per-record, not for the whole batch"
    );
}

// ── C4: abstention ────────────────────────────────────────────────────────

#[test]
fn abstention_fires_when_no_candidate_reaches_the_threshold() {
    let db = db_with_confidence_threshold(0.9);
    db.put(MemoryInput {
        confidence: Some(0.5),
        ..MemoryInput::new("ns", "low", "content lorem")
    })
    .unwrap();

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert!(page.hits.is_empty());
    assert!(page.abstained, "low-confidence-only query must abstain");
    assert_eq!(
        page.abstention_reason,
        Some(AbstentionReason::NoCandidatesAboveThreshold)
    );
}

#[test]
fn abstention_stays_off_without_a_configured_threshold() {
    let db = in_memory_db();
    db.put(MemoryInput::new("ns", "any", "content lorem"))
        .unwrap();

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("nomatch".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert!(page.hits.is_empty());
    assert!(!page.abstained, "default OFF: no signal without threshold");
    assert_eq!(page.abstention_reason, None);
}

#[test]
fn abstention_reports_all_quarantined_when_every_candidate_is_isolated() {
    let db = db_with_confidence_threshold(0.1);
    db.put(quarantined_input("ns", "suspect", "content lorem"))
        .unwrap();

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert!(page.hits.is_empty());
    assert!(page.abstained);
    assert_eq!(
        page.abstention_reason,
        Some(AbstentionReason::AllQuarantined)
    );
}

#[test]
fn abstention_returns_kept_candidates_above_the_threshold() {
    let db = db_with_confidence_threshold(0.8);
    db.put(MemoryInput {
        confidence: Some(0.5),
        ..MemoryInput::new("ns", "low", "content lorem")
    })
    .unwrap();
    db.put(MemoryInput::new("ns", "high", "content lorem ipsum"))
        .unwrap();

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(page.hits.len(), 1);
    assert_eq!(page.hits[0].record.key, "high");
    assert!(!page.abstained);
}

// ── C5: trust-aware combination ───────────────────────────────────────────

#[test]
fn search_applies_min_confidence_and_quarantine_filters_together() {
    let db = in_memory_db();
    db.put(MemoryInput {
        confidence: Some(0.4),
        ..MemoryInput::new("ns", "low-active", "content lorem")
    })
    .unwrap();
    db.put(MemoryInput {
        confidence: Some(0.95),
        ..MemoryInput::new("ns", "high-active", "content lorem")
    })
    .unwrap();
    db.put(MemoryInput {
        confidence: Some(0.99),
        ..quarantined_input("ns", "high-quarantined", "content lorem")
    })
    .unwrap();

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 10,
            min_confidence: Some(0.8),
            ..Default::default()
        })
        .expect("search");
    let keys: Vec<String> = page.hits.iter().map(|h| h.record.key.clone()).collect();
    assert_eq!(
        keys,
        vec!["high-active".to_string()],
        "quarantined excluded by default; low confidence filtered by min_confidence"
    );
}

// ── Audit trail of transitions (T1/T2/T4 + apply) ────────────────────────

#[test]
fn quarantine_transitions_are_audited() {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let db = Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        audit_log_path: Some(audit_path.clone()),
        ..Default::default()
    })
    .unwrap();

    // T1 flag + T1d apply both audit `quarantine_enter`.
    db.put(quarantined_input("ns", "a", "content alpha"))
        .unwrap();
    db.put(MemoryInput::new("ns", "b", "content beta")).unwrap();
    db.quarantine_apply("ns", "b", None).unwrap();
    // T2 promote + T4 reject audit their own transition events.
    db.quarantine_promote("ns", "a").unwrap();
    db.quarantine_reject("ns", "b").unwrap();

    drop(db); // flush + close the audit writer

    let content = std::fs::read_to_string(&audit_path).unwrap();
    let ops: Vec<String> = content
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["op"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        ops.iter()
            .filter(|op| op.as_str() == "quarantine_enter")
            .count(),
        2,
        "T1 + T1d must each emit quarantine_enter: {ops:?}"
    );
    assert!(
        ops.contains(&"quarantine_promote".to_string()),
        "T2 must be audited: {ops:?}"
    );
    assert!(
        ops.contains(&"quarantine_reject".to_string()),
        "T4 must be audited: {ops:?}"
    );
}

// ── Fingerprint: the quarantine view is part of the cursor plan ───────────

#[test]
fn cursor_from_default_view_is_rejected_in_include_quarantined_view() {
    let db = in_memory_db();
    for i in 0..5 {
        db.put(MemoryInput::new("ns", format!("k{i}"), "content lorem"))
            .unwrap();
    }
    let first = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 2,
            ..Default::default()
        })
        .expect("first page");
    let cursor = first.next_cursor.expect("full page yields a cursor");

    let err = db
        .search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("lorem".into()),
            top_k: 2,
            include_quarantined: true,
            cursor: Some(cursor),
            ..Default::default()
        })
        .expect_err("cursor from another quarantine view must be rejected");
    assert!(
        err.to_string().contains("SEARCH_CURSOR_INVALID"),
        "stable marker expected, got: {err}"
    );
}

// ── F4: sticky on the raw transport (import + bulk) ───────────────────────

#[test]
fn import_plain_over_quarantined_key_preserves_state() {
    let src = in_memory_db();
    src.put(MemoryInput::new("imp", "k1", "content plain"))
        .unwrap();
    let plain = src.get("imp", "k1").unwrap().unwrap();

    let dst = in_memory_db();
    dst.put(quarantined_input("imp", "k1", "content old"))
        .unwrap();
    let report = dst.import_records(vec![plain], false).expect("import");
    assert_eq!(report.quarantined, 0, "no import-level entry requested");

    let after = dst.get("imp", "k1").unwrap().unwrap();
    assert!(
        after.quarantined_at_ms.is_some(),
        "F4: a plain raw import must not clear an existing quarantine (I2)"
    );
    assert_eq!(after.quarantine_reason.as_deref(), Some("explicit_write"));
}

#[test]
fn bulk_plain_over_quarantined_key_preserves_state() {
    let db = in_memory_db();
    db.put(quarantined_input("bulk", "sticky", "content old"))
        .unwrap();

    let plain = vec![MemoryInput::new("bulk", "sticky", "content new")];
    let mut reader = std::io::Cursor::new(bulk_payload(&plain));
    let report = db.bulk_import_stream(&mut reader).expect("bulk import");
    assert_eq!(report.quarantined, 0);

    let after = db.get("bulk", "sticky").unwrap().unwrap();
    assert_eq!(after.payload, "content new", "the payload is replaced");
    assert!(
        after.quarantined_at_ms.is_some(),
        "F4: a plain bulk record must not clear an existing quarantine (I2)"
    );
    assert_eq!(after.quarantine_reason.as_deref(), Some("explicit_write"));
}

#[test]
fn import_quarantined_count_only_includes_persisted_records() {
    let src = in_memory_db();
    src.put(MemoryInput::new("imp", "good", "content good"))
        .unwrap();
    let good = src.get("imp", "good").unwrap().unwrap();
    // Same node id as `good` but a different key ⇒ boundary rejection.
    let mut bad = good.clone();
    bad.key = "bad".into();

    let dst = in_memory_db();
    let report = dst
        .import_records(vec![good, bad], true)
        .expect("import returns a report even with per-record errors");
    assert_eq!(report.inserted, 1);
    assert_eq!(report.errors, 1);
    assert_eq!(
        report.quarantined, 1,
        "N1: the failed record must not inflate the quarantined count"
    );
}

// ── F3: batch/import/bulk entries are audited ─────────────────────────────

#[test]
fn batch_import_and_bulk_quarantine_entries_are_audited() {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let db = Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        audit_log_path: Some(audit_path.clone()),
        ..Default::default()
    })
    .unwrap();

    // T1 via batch → `quarantine_enter` per entering record.
    db.put_batch(vec![
        quarantined_input("ns", "c1", "content one"),
        MemoryInput::new("ns", "c2", "content two"),
    ])
    .unwrap();

    // T1c via the direct import_records entry → `import_records` audit.
    let src = in_memory_db();
    src.put(MemoryInput::new("imp", "k1", "content")).unwrap();
    let record = src.get("imp", "k1").unwrap().unwrap();
    db.import_records(vec![record], true).unwrap();

    // T1 via bulk → `bulk_import` audit.
    let inputs = vec![quarantined_input("bulk", "flagged", "content")];
    let mut reader = std::io::Cursor::new(bulk_payload(&inputs));
    db.bulk_import_stream(&mut reader).unwrap();

    drop(db); // flush + close the audit writer

    let content = std::fs::read_to_string(&audit_path).unwrap();
    let events: Vec<serde_json::Value> = content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let ops: Vec<&str> = events
        .iter()
        .map(|e| e["op"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        ops.iter().filter(|op| **op == "quarantine_enter").count(),
        1,
        "batch entry must be audited per record: {ops:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| e["op"] == "quarantine_enter" && e["key"] == "c1"),
        "the batch entry event carries the record key: {events:?}"
    );
    assert!(
        ops.contains(&"import_records"),
        "direct import_records must be audited: {ops:?}"
    );
    assert!(
        ops.contains(&"bulk_import"),
        "bulk import must be audited: {ops:?}"
    );
    let bulk_event = events
        .iter()
        .find(|e| e["op"] == "bulk_import")
        .expect("bulk event");
    assert_eq!(bulk_event["reason"], "1 quarantined");
}

// ── SCH-06: bordes TTL+quarantine y supersede+invalid (I3 ortogonalidad) ──

#[test]
fn quarantined_record_with_live_ttl_stays_isolated_and_purge_skips_it() {
    let db = in_memory_db();
    let record = db
        .put(MemoryInput {
            ttl_ms: Some(600_000),
            ..quarantined_input("ns", "ttl-live", "content ttl")
        })
        .unwrap();
    assert!(record.quarantined_at_ms.is_some(), "T1 entered quarantine");
    assert!(
        record.expires_at_ms.is_some(),
        "TTL is set beside quarantine"
    );

    assert_eq!(
        db.purge_expired().unwrap(),
        0,
        "a live TTL must not purge anything"
    );
    assert!(
        listed_keys(&db, "ns", false).is_empty(),
        "quarantine excludes even with a live TTL"
    );
    assert_eq!(listed_keys(&db, "ns", true), vec!["ttl-live"]);
    let fetched = db.get("ns", "ttl-live").unwrap().unwrap();
    assert_eq!(
        fetched.quarantined_at_ms, record.quarantined_at_ms,
        "reads never mutate/promote the quarantine state (I1)"
    );
}

#[test]
fn expired_ttl_purges_quarantined_record_without_promoting_it() {
    let db = in_memory_db();
    let record = db
        .put(MemoryInput {
            ttl_ms: Some(1),
            ..quarantined_input("ns", "ttl-dead", "content ttl")
        })
        .unwrap();
    assert!(
        record.quarantined_at_ms.is_some(),
        "entered quarantine at write time"
    );

    // Condition-driven bounded wait: `purge_expired` returns >0 once the
    // deadline passes (no sleep-then-assert, no fixed sleep).
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut purged = 0;
    while purged == 0 {
        purged = db.purge_expired().unwrap();
        if purged == 0 {
            assert!(
                Instant::now() < deadline,
                "TTL record never became purgeable"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    assert_eq!(purged, 1);

    // Physical delete (TTL axis) — never a promotion, never a ghost in any view.
    assert!(
        db.get("ns", "ttl-dead").unwrap().is_none(),
        "expired record is physically purged"
    );
    assert!(listed_keys(&db, "ns", false).is_empty());
    assert!(
        listed_keys(&db, "ns", true).is_empty(),
        "no lingering quarantined ghost after TTL purge"
    );
}

#[test]
fn supersede_keeps_quarantine_and_aligns_invalid_at_orthogonally() {
    let db = in_memory_db();
    let mut old_input = quarantined_input("ns", "old", "content old");
    old_input.valid_at_ms = Some(1_000); // fixed reference date (past)
    db.put(old_input).unwrap();
    db.put(MemoryInput::new("ns", "new", "content new"))
        .unwrap();
    db.supersede("ns", "old", "new").unwrap();

    let old = db.get("ns", "old").unwrap().unwrap();
    assert!(
        old.quarantined_at_ms.is_some(),
        "I3: supersede must not clear quarantine"
    );
    assert_eq!(old.superseded_by.as_deref(), Some("new"));
    assert_eq!(
        old.invalid_at_ms, old.superseded_at_ms,
        "D3-3: invalid_at stays aligned with superseded_at"
    );
    assert!(old.valid_at_ms <= old.invalid_at_ms.unwrap());

    // Both gates independent: default hides it (quarantine); opt-in shows it
    // even though it is superseded (supersede is visible by default).
    assert_eq!(listed_keys(&db, "ns", false), vec!["new"]);
    assert!(listed_keys(&db, "ns", true).contains(&"old".to_string()));

    // Valid axis is orthogonal: opt-in AS OF before invalid includes it; at
    // the boundary (end exclusive) it is out.
    let t_inv = old.invalid_at_ms.unwrap();
    let as_of = |t: u64| -> Vec<String> {
        db.search_page(MemorySearchRequest {
            namespace: "ns".into(),
            text_query: Some("content".into()),
            top_k: 10,
            as_of_ms: Some(t),
            include_quarantined: true,
            ..Default::default()
        })
        .unwrap()
        .hits
        .into_iter()
        .map(|hit| hit.record.key)
        .collect()
    };
    assert!(as_of(t_inv - 1).contains(&"old".to_string()));
    assert!(!as_of(t_inv).contains(&"old".to_string()));

    // T2 clears quarantine but leaves the supersession axis untouched (I3).
    let promoted = db.quarantine_promote("ns", "old").unwrap();
    assert!(promoted.quarantined_at_ms.is_none());
    assert_eq!(
        promoted.superseded_by.as_deref(),
        Some("new"),
        "promote must not clear supersede"
    );
    assert_eq!(promoted.invalid_at_ms, Some(t_inv));
}
