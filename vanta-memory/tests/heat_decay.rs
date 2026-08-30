//! MEM-60 — Heat + decay + contradiction (integration test).
//!
//! Verifies the contract:
//!   - `cargo test -p vanta-memory --test heat_decay 2>&1 | Select-String
//!     "ok|PASS" | Measure-Object | Select-Object Count` >= 1
//!
//! AND the regex on the record module file:
//!   - `Select-String -Path "vanta-memory/src/core/record/mod.rs" -Pattern
//!     "heat.*decay|contradiction" | Measure-Object | Select-Object Count` >= 1

use serde_json::json;
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::lifecycle::{
    bump_heat, decay_heat, is_prune_eligible, mark_contradiction, DEFAULT_HEAT,
    PRUNE_HEAT_THRESHOLD,
};

fn open_db() -> VantaEmbedded {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

fn fixture() -> MemoryRecord {
    MemoryRecord {
        id: "m1".into(),
        content: "user prefers dark mode".into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "ui-setup".into(),
        source_message_ids: vec!["msg_1".into()],
        metadata: json!({"activity_start_time": "2026-08-20T10:00:00Z"}),
        timestamps: vec!["2026-08-20T10:00:00Z".into()],
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
        heat: DEFAULT_HEAT,
        superseded_by: None,
    }
}

/// Persistence roundtrip — write a record with `heat = 0`, read it back,
/// bump heat, decay heat, write again, read again. The decay should land on
/// the threshold and the record should be prune-eligible after enough passes.
#[test]
fn heat_bump_then_decay_persists_and_reaches_prune_threshold() {
    let db = open_db();
    let namespace = "l1/sess-1";
    let key = "m1";

    let r = fixture();
    db.put(VantaMemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: serde_json::to_string(&r).unwrap(),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("initial put");

    // Simulate "useful" reads — each read bumps heat by 1.
    for _ in 0..4 {
        let raw = db
            .get(namespace, key)
            .expect("get")
            .expect("exists")
            .payload;
        let mut parsed: MemoryRecord = serde_json::from_str(&raw).expect("parse");
        bump_heat(&mut parsed, 1);
        let payload = serde_json::to_string(&parsed).unwrap();
        db.put(VantaMemoryInput {
            namespace: namespace.to_string(),
            key: key.to_string(),
            payload,
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .expect("re-put");
    }

    // Read back the warmed-up record.
    let raw = db
        .get(namespace, key)
        .expect("get")
        .expect("exists")
        .payload;
    let mut warmed: MemoryRecord = serde_json::from_str(&raw).expect("parse");
    assert_eq!(
        warmed.heat, 4,
        "4 reads → heat=4 (DEFAULT_HEAT=0 → +4 bumps)"
    );

    // Apply decay passes until we hit the prune threshold.
    let mut passes = 0u32;
    while !is_prune_eligible(&warmed) {
        decay_heat(&mut warmed);
        passes += 1;
        assert!(passes < 32, "decay must converge within 32 passes");
    }
    assert!(
        warmed.heat <= PRUNE_HEAT_THRESHOLD,
        "eligible: heat={}, threshold={}",
        warmed.heat,
        PRUNE_HEAT_THRESHOLD
    );
}

/// Contradiction provenance — a new record invalidates an old one. The OLD
/// record is preserved with `superseded_by` set (provenance chain). The NEW
/// record is fresh.
#[test]
fn contradiction_marks_old_with_superseded_by_keeps_new() {
    let mut old = fixture();
    let old_id = old.id.clone();

    let provenance = mark_contradiction(&mut old, "m_new", 1_700_000_000_000);

    assert_eq!(old.superseded_by.as_deref(), Some("m_new"));
    assert_eq!(old.id, old_id, "old id preserved");
    assert_eq!(
        old.content, "user prefers dark mode",
        "old content preserved"
    );
    assert_eq!(provenance.old_key, old_id, "provenance carries the old key");
    assert_eq!(
        provenance.new_key, "m_new",
        "provenance carries the new key"
    );
    assert_eq!(provenance.recorded_at_ms, 1_700_000_000_000);
    assert_eq!(provenance.namespace, "l1/sess-1");

    // No field in `old` was zeroed — we never delete silently.
    assert!(!old.content.is_empty());
    // `heat` is preserved as-is (whatever it was before the call); the
    // provenance lives in `superseded_by`, not in any heat mutation.
    assert!(old.superseded_by.is_some(), "provenance chain established");
}

/// Wire backward-compat — a record JSON written before MEM-60 (no `heat`,
/// no `superseded_by`) still parses as cold + live.
#[test]
fn legacy_record_without_lifecycle_fields_parses_as_cold_live() {
    let legacy = json!({
        "id": "m_old",
        "content": "legacy content",
        "type": "persona",
        "priority": 50,
        "scene_name": "old-scene",
        "source_message_ids": [],
        "metadata": null,
        "timestamps": [],
        "created_at": "2024-01-01T00:00:00.000Z",
        "updated_at": "2024-01-01T00:00:00.000Z",
        "version": 1,
        "session_key": "sess-old",
        "session_id": "",
    });
    let raw = serde_json::to_string(&legacy).unwrap();
    let parsed: MemoryRecord = serde_json::from_str(&raw).expect("legacy parse");
    assert_eq!(parsed.heat, DEFAULT_HEAT);
    assert!(parsed.superseded_by.is_none(), "not superseded");
    // DEFAULT_HEAT (0) <= PRUNE_HEAT_THRESHOLD (1) → legacy cold record is
    // immediately prune-eligible by the maintenance gate (the gate's call,
    // not this test's; we only verify the field defaults landed correctly).
    assert!(is_prune_eligible(&parsed));
}
