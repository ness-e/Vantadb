// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-07 — Forgetting curve over L1 (integration test).
//!
//! Verifies the contract:
//!   - `cargo nextest run --profile audit -p vanta-memory --test forgetting_curve`
//!   - the pass reports exact known values for a seeded session;
//!   - the pass is idempotent and never mutates or deletes records.

use serde_json::json;
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::lifecycle::{scan_decay, DecayPolicy};
use vanta_memory::core::record::{read_session_records, run_decay_pass};

/// Fixed instants (RFC3339 + epoch ms — no chrono in this test crate).
/// `T0` = 2026-01-01T00:00:00.000Z, `NOW` = 2026-01-31T00:00:00.000Z.
const T0: &str = "2026-01-01T00:00:00.000Z";
const NOW_ISO: &str = "2026-01-31T00:00:00.000Z";
const NOW_MS: u64 = 1_769_817_600_000;
const NS: &str = "l1/sess-1";

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

fn record(id: &str, memory_type: MemoryType, heat: u32, updated_at: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: format!("content of {id}"),
        memory_type,
        priority: 50,
        scene_name: "s".into(),
        source_message_ids: vec![],
        metadata: json!(null),
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
        heat,
        superseded_by: None,
    }
}

fn put(db: &Embedded, r: &MemoryRecord) {
    db.put(MemoryInput {
        namespace: NS.to_string(),
        key: r.id.clone(),
        payload: serde_json::to_string(r).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put record");
}

/// 4 records with known (type, heat, age): the report is exact.
#[test]
fn decay_pass_reports_known_values_for_seeded_session() {
    let db = open_db();
    // 30d old · episodic (7d half-life) · heat 8 → 8×2^(−30/7) ≈ 0.41 → 0.
    put(&db, &record("m_episodic", MemoryType::Episodic, 8, T0));
    // Fresh (updated now) · persona → full retention → 4.
    put(&db, &record("m_persona", MemoryType::Persona, 4, NOW_ISO));
    // Exempt type · ancient → keeps raw 2.
    put(
        &db,
        &record(
            "m_instruction",
            MemoryType::Instruction,
            2,
            "2020-01-01T00:00:00.000Z",
        ),
    );
    // 30d old · work_task (14d half-life) · heat 1 → 2^(−30/14) ≈ 0.23 → 0.
    put(&db, &record("m_worktask", MemoryType::WorkTask, 1, T0));

    let report = run_decay_pass(&db, "sess-1", &DecayPolicy::default(), NOW_MS).expect("pass");

    assert_eq!(report.scanned, 4);
    assert_eq!(report.decayed, 2, "episodic + work_task decayed");
    assert_eq!(report.unchanged, 1, "fresh persona at full retention");
    assert_eq!(report.exempt, 1, "instruction has no half-life");
    assert_eq!(
        report.below_threshold, 2,
        "both decayed records hit effective 0"
    );
    assert_eq!(report.heat_total, 15);
    assert_eq!(report.heat_effective, 6);
    assert_eq!(report.heat_forgotten(), 9);
}

#[test]
fn decay_pass_is_idempotent_and_never_mutates() {
    let db = open_db();
    put(&db, &record("m_episodic", MemoryType::Episodic, 8, T0));
    put(&db, &record("m_persona", MemoryType::Persona, 4, NOW_ISO));

    let payloads_before: Vec<String> = ["m_episodic", "m_persona"]
        .iter()
        .map(|k| db.get(NS, k).expect("get").expect("exists").payload)
        .collect();

    let policy = DecayPolicy::default();
    let first = run_decay_pass(&db, "sess-1", &policy, NOW_MS).expect("pass 1");
    let second = run_decay_pass(&db, "sess-1", &policy, NOW_MS).expect("pass 2");

    assert_eq!(first, second, "same now_ms → same report");
    let payloads_after: Vec<String> = ["m_episodic", "m_persona"]
        .iter()
        .map(|k| db.get(NS, k).expect("get").expect("exists").payload)
        .collect();
    assert_eq!(payloads_before, payloads_after, "stored payloads untouched");
}

#[test]
fn decay_pass_never_deletes_records() {
    let db = open_db();
    put(&db, &record("m_a", MemoryType::Episodic, 8, T0));
    put(&db, &record("m_b", MemoryType::Persona, 4, T0));
    put(&db, &record("m_c", MemoryType::Instruction, 2, T0));

    let _ = run_decay_pass(&db, "sess-1", &DecayPolicy::default(), NOW_MS).expect("pass");

    let mut ids: Vec<String> = read_session_records(&db, "sess-1")
        .expect("read")
        .into_iter()
        .map(|r| r.id)
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        vec!["m_a", "m_b", "m_c"],
        "nothing purged by the curve"
    );
}

#[test]
fn scan_decay_classifies_exempt_separately_from_unchanged() {
    let records = vec![
        record("a", MemoryType::Instruction, 3, T0),  // exempt
        record("b", MemoryType::Persona, 3, NOW_ISO), // unchanged (fresh)
        record("c", MemoryType::Episodic, 3, T0),     // decayed
    ];
    let report = scan_decay(&records, &DecayPolicy::default(), NOW_MS);
    assert_eq!(report.scanned, 3);
    assert_eq!(report.exempt, 1);
    assert_eq!(report.unchanged, 1);
    assert_eq!(report.decayed, 1);
    assert_eq!(
        report.below_threshold, 1,
        "only the decayed episodic crossed ≤1"
    );
}
