// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-11 Step 0 (pre-mortem #1): the D38 dual-pool semantics — a legacy
//! record WITHOUT a vector is never dropped from recall — must survive the
//! migration to the core hybrid search (BM25 + HNSW + RRF).
//!
//! These tests are RED until the `RecallConfig.core_search` flag exists
//! (compile error E0560 is the expected first failure); after Step 2 they must
//! pass with the flag ON while the legacy path stays byte-identical.

use std::sync::Arc;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallMode, RecallScope,
};
use vanta_memory::core::profile::ProfileIsolation;
use vanta_memory::core::record::l1_reader::l1_namespace;
use vanta_memory::core::record::EmbedFn;
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn l1_record(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "ui-setup".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![],
        created_at: "2026-08-20T10:00:00Z".into(),
        updated_at: "2026-08-20T10:00:00Z".into(),
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

/// Persist an L1 record exactly as `read_session_records` expects it: payload
/// serialized with `vector: None`, the node carrying the vector separately
/// (mirrors `put_record`).
fn put_l1(db: &Embedded, record: &MemoryRecord, vector: Option<Vec<f32>>) {
    db.put(MemoryInput {
        namespace: l1_namespace(&record.session_key),
        key: record.id.clone(),
        payload: serde_json::to_string(record).expect("serialize"),
        metadata: MemoryMetadata::new(),
        vector,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put l1 record");
}

fn fixed_embed(vector: Vec<f32>) -> EmbedFn {
    Arc::new(move |_text: &str| Some(vector.clone()))
}

fn recall_with(
    db: &Embedded,
    user_text: &str,
    config: RecallConfig,
    embed: Option<&EmbedFn>,
) -> Option<vanta_memory::core::hooks::RecallResult> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text,
            session_key: "sess-1",
            isolation: Some(ProfileIsolation::default()),
            config,
        },
        embed,
    )
    .expect("recall")
}

/// Guard: the safe default keeps the legacy dual-pool path (flag off).
#[test]
fn core_search_flag_default_is_off() {
    assert!(
        !RecallConfig::default().core_search,
        "core_search must default to false (safe default, legacy path)"
    );
}

/// Baseline (legacy path, passes today): a vectorless record that shares
/// significant terms is recalled by the keyword gate.
#[test]
fn legacy_recall_keeps_vectorless_record() {
    let db = db();
    put_l1(
        &db,
        &l1_record("legacy-1", "user prefers dark mode for the editor"),
        None,
    );

    let out = recall_with(
        &db,
        "what about dark mode preference in the editor?",
        RecallConfig::default(),
        None,
    )
    .expect("recall returns content");

    assert!(
        out.recalled_memories
            .iter()
            .any(|m| m.content.contains("dark mode")),
        "legacy keyword gate must recall the vectorless record"
    );
}

/// PRE-MORTEM #1 (the migration test): with the core hybrid path enabled, a
/// record WITHOUT a vector must still be recalled through the BM25 text arm —
/// the dual-pool promise "a legacy record is never dropped" holds on the core
/// engine too.
#[test]
fn core_hybrid_recall_keeps_vectorless_record() {
    let db = db();
    // Legacy record: NO vector on the node.
    put_l1(
        &db,
        &l1_record("legacy-1", "user prefers dark mode for the editor"),
        None,
    );
    // A vector-bearing record with unrelated content (only vector candidate).
    put_l1(
        &db,
        &l1_record("vec-1", "team ships releases on fridays"),
        Some(vec![1.0, 0.0, 0.0, 0.0]),
    );

    let embed = fixed_embed(vec![0.0, 1.0, 0.0, 0.0]);
    let out = recall_with(
        &db,
        "dark mode preference in the editor",
        RecallConfig {
            core_search: true,
            mode: RecallMode::Hybrid,
            scope: RecallScope::Session,
            ..RecallConfig::default()
        },
        Some(&embed),
    )
    .expect("recall returns content");

    assert!(
        out.recalled_memories
            .iter()
            .any(|m| m.content.contains("dark mode")),
        "core hybrid path must never drop a vectorless record (BM25 text arm)"
    );
}

/// The semantic arm works on the core path: a record with no shared terms is
/// recalled via HNSW when its vector matches the query embedding.
#[test]
fn core_hybrid_recall_finds_semantic_match() {
    let db = db();
    put_l1(
        &db,
        &l1_record("sem-1", "alpha bravo charlie"),
        Some(vec![1.0, 0.0, 0.0, 0.0]),
    );

    let embed = fixed_embed(vec![1.0, 0.0, 0.0, 0.0]);
    let out = recall_with(
        &db,
        "zulu yankee xray",
        RecallConfig {
            core_search: true,
            mode: RecallMode::Hybrid,
            scope: RecallScope::Session,
            ..RecallConfig::default()
        },
        Some(&embed),
    )
    .expect("recall returns content");

    assert!(
        out.recalled_memories
            .iter()
            .any(|m| m.content.contains("alpha bravo")),
        "HNSW arm must recall the semantically matching record"
    );
}

/// D38 also holds in `Embedding` mode: the core path always runs the text arm,
/// so a vectorless record with shared terms is still recalled (review P2-01
/// High finding — the legacy dual-pool kept the keyword pool in this mode).
#[test]
fn core_hybrid_embedding_mode_keeps_vectorless_record() {
    let db = db();
    put_l1(
        &db,
        &l1_record("legacy-1", "user prefers dark mode for the editor"),
        None,
    );

    let embed = fixed_embed(vec![1.0, 0.0, 0.0, 0.0]);
    let out = recall_with(
        &db,
        "dark mode preference in the editor",
        RecallConfig {
            core_search: true,
            mode: RecallMode::Embedding,
            scope: RecallScope::Session,
            ..RecallConfig::default()
        },
        Some(&embed),
    )
    .expect("recall returns content");

    assert!(
        out.recalled_memories
            .iter()
            .any(|m| m.content.contains("dark mode")),
        "Embedding mode on the core path must not drop a vectorless record"
    );
}

/// Budget parity: the flag path respects `max_results` like the legacy path.
#[test]
fn core_hybrid_respects_max_results() {
    let db = db();
    for (id, content) in [
        ("m1", "user prefers dark mode"),
        ("m2", "user likes dark themes"),
        ("m3", "user chose the dark variant"),
    ] {
        put_l1(&db, &l1_record(id, content), None);
    }

    let out = recall_with(
        &db,
        "dark mode preference",
        RecallConfig {
            core_search: true,
            mode: RecallMode::Keyword,
            scope: RecallScope::Session,
            max_results: 2,
            ..RecallConfig::default()
        },
        None,
    )
    .expect("recall returns content");

    assert_eq!(
        out.recalled_memories.len(),
        2,
        "core path must honor max_results"
    );
}

/// Cross-session scope (D22) on the core path: another session of the same
/// agent is merged in; another agent's session stays invisible.
#[test]
fn core_hybrid_scope_agent_merges_other_sessions() {
    let db = db();
    let mut other = l1_record("cross-1", "user prefers dark mode in other session");
    other.session_key = "sess-2".into();
    other.agent_id = Some("default".into());
    other.team_id = Some("default".into());
    put_l1(&db, &other, None);

    let mut stranger = l1_record("stranger-1", "user prefers dark mode from stranger");
    stranger.session_key = "sess-3".into();
    stranger.agent_id = Some("other-agent".into());
    stranger.team_id = Some("default".into());
    put_l1(&db, &stranger, None);

    let out = recall_with(
        &db,
        "dark mode preference",
        RecallConfig {
            core_search: true,
            mode: RecallMode::Keyword,
            scope: RecallScope::Agent,
            ..RecallConfig::default()
        },
        None,
    )
    .expect("recall returns content");

    assert!(
        out.recalled_memories
            .iter()
            .any(|m| m.content.contains("other session")),
        "same-agent other session must be visible under Agent scope"
    );
    assert!(
        !out.recalled_memories
            .iter()
            .any(|m| m.content.contains("stranger")),
        "another agent's session must stay invisible"
    );
}

/// Measurement instrument (explicit run only — no timing asserts; the printed
/// numbers are recorded in the MEMG-11 task file A/B):
/// `cargo nextest run -p vanta-memory --test recall_core_hybrid --run-ignored ignored-only --nocapture`
#[test]
#[ignore = "measurement instrument — run explicitly with --run-ignored ignored-only --nocapture"]
fn measurement_recall_legacy_vs_core() {
    let db = db();
    let n = 2000usize;
    let topics = [
        "dark mode",
        "vim keys",
        "postgres",
        "travel plans",
        "coffee",
    ];
    for i in 0..n {
        let content = format!("user memory item {i} about {}", topics[i % topics.len()]);
        put_l1(&db, &l1_record(&format!("m-{i:05}"), &content), None);
    }
    let query = "dark mode preference";
    let runs = 20usize;

    // Warmup (both paths).
    let _ = recall_with(&db, query, RecallConfig::default(), None);
    let _ = recall_with(
        &db,
        query,
        RecallConfig {
            core_search: true,
            ..RecallConfig::default()
        },
        None,
    );

    for core in [false, true] {
        let start = std::time::Instant::now();
        for _ in 0..runs {
            let out = recall_with(
                &db,
                query,
                RecallConfig {
                    core_search: core,
                    ..RecallConfig::default()
                },
                None,
            );
            assert!(out.is_some(), "fixture must recall something");
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0 / runs as f64;
        eprintln!("[MEMG-11 A/B] recall core_search={core} pool={n} avg={ms:.2}ms/query");
    }
}
