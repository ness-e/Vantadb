// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-10 — trusted/tainted namespace classes (MGR-04) on the VER-04 recall
//! hook: tainted content never feeds the prompt by default, denials are
//! reported (never silent), and `include_tainted` is the explicit review
//! opt-in. RED before `InjectionPolicy::from_parts`/`TrustClass` exist:
//! this file refuses to compile until the trust API lands.

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall_governed, AutoRecallParams, InjectionPolicy, RecallConfig, RecallScope,
    TrustClass,
};
use vanta_memory::core::profile::ProfileIsolation;
use vanta_memory::core::record::l1_reader::l1_namespace;
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn record(session: &str, id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Episodic,
        priority: 80,
        scene_name: "ui-setup".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![],
        created_at: "2026-08-20T10:00:00Z".into(),
        updated_at: "2026-08-20T10:00:00Z".into(),
        version: 1,
        session_key: session.into(),
        session_id: "".into(),
        task_id: None,
        team_id: Some("default".into()),
        user_id: None,
        agent_id: Some("default".into()),
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

/// Persist an L1 record exactly as `read_session_records` expects it.
fn put_l1(db: &Embedded, record: &MemoryRecord) {
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

fn write_persona(db: &Embedded, session: &str, body: &str) {
    use vanta_memory::core::abstractions::PersonaMode;
    use vanta_memory::core::persona::persona_generator::{
        persona_namespace, PersonaRecord, PERSONA_KEY,
    };
    let record = PersonaRecord {
        content: body.into(),
        mode: PersonaMode::First,
        generated_at_ms: 1_000,
        generated_at: "2026-08-20T10:00:00Z".into(),
    };
    db.put(MemoryInput {
        namespace: persona_namespace(session),
        key: PERSONA_KEY.into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put persona");
}

fn params<'a>(user_text: &'a str, session_key: &'a str) -> AutoRecallParams<'a> {
    AutoRecallParams {
        user_text,
        session_key,
        isolation: Some(ProfileIsolation::default()),
        config: RecallConfig {
            scope: RecallScope::Agent,
            ..RecallConfig::default()
        },
    }
}

#[test]
fn tainted_session_namespace_is_not_injected_and_the_denial_is_reported() {
    let db = db();
    put_l1(
        &db,
        &record("sess-1", "m1", "user prefers dark mode in vim"),
    );

    let policy = InjectionPolicy::from_parts(std::iter::empty::<String>(), ["l1/sess-1"], false);
    assert_eq!(
        policy.trust_class("l1/sess-1"),
        TrustClass::Tainted,
        "the classification must be observable"
    );

    let out = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &policy,
    )
    .expect("governed recall must not error")
    .expect("denials must be reported as payload, not dropped");

    assert!(
        out.prepend_context.is_none(),
        "tainted content must not inject: {:?}",
        out.prepend_context
    );
    assert!(out.recalled_memories.is_empty());
    assert!(
        out.governance.denied.iter().any(|ns| ns == "l1/sess-1"),
        "denial must be visible in governance: {:?}",
        out.governance.denied
    );
}

#[test]
fn include_tainted_opt_in_restores_injection_for_review_workflows() {
    let db = db();
    put_l1(
        &db,
        &record("sess-1", "m1", "user prefers dark mode in vim"),
    );

    let policy = InjectionPolicy::from_parts(std::iter::empty::<String>(), ["l1/sess-1"], true);
    let out = perform_auto_recall_governed(&db, params("dark mode", "sess-1"), None, &policy)
        .expect("recall")
        .expect("explicit opt-in pass injects content");

    let prepend = out.prepend_context.expect("opt-in must inject");
    assert!(prepend.contains("dark mode in vim"));
    assert!(
        out.governance.denied.is_empty(),
        "no denials with opt-in: {:?}",
        out.governance.denied
    );
}

#[test]
fn tainted_persona_and_scene_sources_are_denied_and_reported() {
    let db = db();
    write_persona(&db, "sess-1", "prefers vim keybindings");

    let policy = InjectionPolicy::from_parts(
        std::iter::empty::<String>(),
        ["persona/sess-1", "scene/sess-1"],
        false,
    );
    let out = perform_auto_recall_governed(&db, params("", "sess-1"), None, &policy)
        .expect("recall")
        .expect("denials must be reported even when nothing injects");

    assert!(out.persona.is_none(), "tainted persona must not inject");
    assert!(out.append_system_context.is_none());
    assert!(
        out.governance
            .denied
            .iter()
            .any(|ns| ns == "persona/sess-1"),
        "persona denial must be visible: {:?}",
        out.governance.denied
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "scene/sess-1"),
        "scene denial must be visible: {:?}",
        out.governance.denied
    );
}

#[test]
fn only_tainted_namespaces_are_denied_trusted_content_still_injects() {
    let db = db();
    put_l1(
        &db,
        &record("sess-1", "m1", "user prefers dark mode in vim"),
    );
    put_l1(
        &db,
        &record("sess-2", "m2", "user prefers dark mode in emacs"),
    );

    let policy = InjectionPolicy::from_parts(std::iter::empty::<String>(), ["l1/sess-2"], false);
    let out = perform_auto_recall_governed(&db, params("dark mode", "sess-1"), None, &policy)
        .expect("recall")
        .expect("trusted content injects");

    let prepend = out.prepend_context.expect("trusted content must inject");
    assert!(prepend.contains("vim"));
    assert!(
        !prepend.contains("emacs"),
        "tainted cross-session content must not leak into the prompt"
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "l1/sess-2"),
        "cross-session denial must be visible: {:?}",
        out.governance.denied
    );
}
