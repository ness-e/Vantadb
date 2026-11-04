// ponytail: blanket allow - unwraps with documented invariants.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Tests for ADRs / Conventions prioritization & Bug->Fix causal traceability (PI-2/PI-3, MGR-23/MGR-24).

use std::sync::Arc;
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallMode, RecallScope,
};
use vantadb::sdk::Embedded;
use vantadb::skills::{BugFixPattern, SkillStore};
use vantadb::storage::StorageEngine;

fn open_test_engine() -> (Arc<StorageEngine>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let engine =
        Arc::new(StorageEngine::open(tmp.path().to_str().expect("path")).expect("engine open"));
    (engine, tmp)
}

#[test]
fn test_adr_recall_priority_and_superseded_exclusion() {
    let (engine, _tmp) = open_test_engine();
    let db = Embedded::from_engine(engine);

    // 1. Put an active ADR
    let mut adr_active = vantadb::sdk::MemoryInput::new(
        "l1/sess-1",
        "adr-002",
        "[ADR:ADR-002] Use RocksDB WAL for durability\nContext: Crash recovery guarantees\nConsequences: Higher disk usage",
    );
    adr_active
        .metadata
        .insert("category".into(), vantadb::sdk::Value::String("adr".into()));
    adr_active.metadata.insert(
        "decision_id".into(),
        vantadb::sdk::Value::String("ADR-002".into()),
    );
    adr_active.metadata.insert(
        "status".into(),
        vantadb::sdk::Value::String("accepted".into()),
    );
    adr_active.metadata.insert(
        "modules".into(),
        vantadb::sdk::Value::ListString(vec!["storage".into(), "wal".into()]),
    );
    adr_active.metadata.insert(
        "context".into(),
        vantadb::sdk::Value::String("Crash recovery guarantees".into()),
    );
    adr_active.metadata.insert(
        "decision".into(),
        vantadb::sdk::Value::String("Use RocksDB WAL for durability".into()),
    );
    db.put(adr_active).expect("put adr_active");

    // 2. Put a superseded ADR
    let mut adr_superseded = vantadb::sdk::MemoryInput::new(
        "l1/sess-1",
        "adr-001",
        "[ADR:ADR-001] In-memory log buffer\nContext: Fast logging\nConsequences: Potential data loss",
    );
    adr_superseded
        .metadata
        .insert("category".into(), vantadb::sdk::Value::String("adr".into()));
    adr_superseded.metadata.insert(
        "decision_id".into(),
        vantadb::sdk::Value::String("ADR-001".into()),
    );
    adr_superseded.metadata.insert(
        "status".into(),
        vantadb::sdk::Value::String("superseded".into()),
    );
    adr_superseded.metadata.insert(
        "superseded_by".into(),
        vantadb::sdk::Value::String("ADR-002".into()),
    );
    adr_superseded.metadata.insert(
        "modules".into(),
        vantadb::sdk::Value::ListString(vec!["storage".into()]),
    );
    db.put(adr_superseded).expect("put adr_superseded");

    // 3. Put normal episodic memory
    let normal_mem = vantadb::sdk::MemoryInput::new(
        "l1/sess-1",
        "mem-note",
        "User discussed storage configuration and memory settings yesterday",
    );
    db.put(normal_mem).expect("put normal_mem");

    // 4. Perform auto-recall for a turn discussing "storage"
    let params = AutoRecallParams {
        user_text: "How should we implement storage crash recovery?",
        session_key: "sess-1",
        isolation: None,
        config: RecallConfig {
            mode: RecallMode::Keyword,
            scope: RecallScope::Session,
            max_results: 5,
            min_overlap: 1,
            max_chars_per_memory: None,
            max_total_recall_chars: None,
            core_search: false,
        },
    };

    let result = perform_auto_recall(&db, params, None)
        .expect("auto recall")
        .expect("recalled content");

    let contents: Vec<&str> = result
        .recalled_memories
        .iter()
        .map(|m| m.content.as_str())
        .collect();

    // Active ADR must be prioritized first
    assert!(!contents.is_empty(), "Must have recalled memories");
    assert!(
        contents[0].contains("ADR-002"),
        "Active ADR must be prioritized first: {:?}",
        contents
    );

    // Superseded ADR must NOT be in the recalled memories
    for content in &contents {
        assert!(
            !content.contains("ADR-001"),
            "Superseded ADR must be excluded from recall: {content}"
        );
    }
}

#[test]
fn test_bugfix_skill_store_and_causal_edges() {
    let (engine, _tmp) = open_test_engine();
    let skill_store = SkillStore::new(&engine);

    let pattern = BugFixPattern {
        id: "bf_wal_corrupt".to_string(),
        symptom: "Checksum mismatch error during WAL replay".to_string(),
        root_cause: "Torn write when power was cut before fsync completed".to_string(),
        fix_summary: "Add CRC32 footer and verify frame length on recovery".to_string(),
        files_touched: vec!["src/storage/wal.rs".into(), "src/storage/frame.rs".into()],
        regression_test: "test_wal_torn_write_recovery".to_string(),
    };

    // 1. Record pattern
    let write_res = skill_store
        .record_bugfix(pattern.clone())
        .expect("record bugfix");
    assert_eq!(write_res.record.name, "bugfix_bf_wal_corrupt");

    // 2. Query by symptom substring
    let matches = skill_store
        .query_bugfix("Checksum mismatch during WAL replay")
        .expect("query bugfix");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].id, "bf_wal_corrupt");
    assert_eq!(matches[0].root_cause, pattern.root_cause);
    assert_eq!(matches[0].fix_summary, pattern.fix_summary);

    // 3. Query by affected file
    let file_matches = skill_store
        .query_bugfix_by_file("src/storage/wal.rs")
        .expect("query bugfix by file");
    assert_eq!(file_matches.len(), 1);
    assert_eq!(file_matches[0].id, "bf_wal_corrupt");

    // Unrelated file yields no matches
    let no_matches = skill_store
        .query_bugfix_by_file("src/network/http.rs")
        .expect("query bugfix unrelated file");
    assert!(no_matches.is_empty());
}
