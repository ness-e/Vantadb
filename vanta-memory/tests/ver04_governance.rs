// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! VER-04: injection governance on the shared auto-recall hook — namespace ACL
//! by prefix (opt-in allowlist) and per-hit source identity for the injection
//! audit. RED before `perform_auto_recall_governed`/`InjectionPolicy` exist:
//! this file refuses to compile until the governed API lands.

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall, perform_auto_recall_governed, AutoRecallParams, InjectionPolicy,
    RecallConfig, RecallScope,
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
fn governed_recall_denies_out_of_scope_sessions() {
    let db = db();
    put_l1(
        &db,
        &record("sess-1", "m1", "user prefers dark mode in vim"),
    );
    put_l1(
        &db,
        &record("sess-2", "m2", "user prefers dark mode in emacs"),
    );

    // Opt-in ACL: only the current session's namespaces are in scope.
    let policy = InjectionPolicy::from_prefixes(["l1/sess-1", "persona/sess-1", "scene/sess-1"]);
    let out = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &policy,
    )
    .expect("recall")
    .expect("content to inject");

    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.contains("vim"), "in-scope hit missing: {prepend}");
    assert!(
        !prepend.contains("emacs"),
        "out-of-scope session was injected: {prepend}"
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "l1/sess-2"),
        "denial must be reported (never silent), got {:?}",
        out.governance.denied
    );
    assert!(
        out.governance.sources.iter().any(|ns| ns == "l1/sess-1"),
        "in-scope source must be reported, got {:?}",
        out.governance.sources
    );
}

#[test]
fn allow_all_policy_matches_ungoverned_recall() {
    let db = db();
    put_l1(&db, &record("sess-1", "m1", "user prefers dark mode"));
    write_persona(&db, "sess-1", "The user is a night owl.");

    let ungoverned = perform_auto_recall(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
    )
    .expect("recall")
    .expect("content");
    let governed = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &InjectionPolicy::allow_all(),
    )
    .expect("recall")
    .expect("content");

    // Empty allowlist = allow-all: byte-identical behavior to today.
    assert_eq!(ungoverned.prepend_context, governed.prepend_context);
    assert_eq!(
        ungoverned.append_system_context,
        governed.append_system_context
    );
    assert!(governed.governance.denied.is_empty());
}

#[test]
fn governed_recall_denies_persona_and_scene_out_of_scope() {
    let db = db();
    put_l1(&db, &record("sess-1", "m1", "user prefers dark mode"));
    write_persona(&db, "sess-1", "The user is a night owl.");
    vanta_memory::core::scene::scene_index::upsert_scene(
        &db,
        "sess-1",
        "runbook",
        "deploys",
        "how to deploy",
    )
    .expect("seed scene");

    // ACL without persona/scene namespaces: only the L1 pool survives.
    let policy = InjectionPolicy::from_prefixes(["l1/sess-1"]);
    let out = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &policy,
    )
    .expect("recall")
    .expect("content");

    assert!(out.prepend_context.expect("prepend").contains("dark mode"));
    let append = out.append_system_context.unwrap_or_default();
    assert!(
        !append.contains("<user-persona>"),
        "denied persona leaked into the stable block: {append}"
    );
    assert!(
        !append.contains("<scene-navigation>"),
        "denied scene navigation leaked into the stable block: {append}"
    );
    assert!(
        out.governance
            .denied
            .iter()
            .any(|ns| ns == "persona/sess-1"),
        "persona denial must be reported, got {:?}",
        out.governance.denied
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "scene/sess-1"),
        "scene denial must be reported, got {:?}",
        out.governance.denied
    );
}

#[test]
fn recalled_memories_carry_source_identity_for_audit() {
    let db = db();
    put_l1(&db, &record("sess-1", "m1", "user prefers dark mode"));

    let out = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &InjectionPolicy::allow_all(),
    )
    .expect("recall")
    .expect("content");

    let first = out.recalled_memories.first().expect("one hit");
    assert_eq!(first.source_namespace, "l1/sess-1");
    assert_eq!(first.source_key, "m1");
}

// ── Review F1: a deny-all pass must not collapse into `Ok(None)` ────────────
#[test]
fn deny_all_pass_still_reports_denials_instead_of_none() {
    let db = db();
    put_l1(&db, &record("sess-1", "m1", "user prefers dark mode"));
    write_persona(&db, "sess-1", "The user is a night owl.");

    // ACL denies every source namespace: nothing gets injected, but the
    // denials ARE the payload the surfaces audit — returning `Ok(None)` here
    // would make the ACL silent (review F1).
    let policy = InjectionPolicy::from_prefixes(["other/"]);
    let out = perform_auto_recall_governed(
        &db,
        params("what does the user prefer about dark mode?", "sess-1"),
        None,
        &policy,
    )
    .expect("recall")
    .expect("deny-all must still return Some so surfaces can audit the denials");

    assert!(out.prepend_context.is_none(), "nothing may be injected");
    assert!(
        out.append_system_context.is_none(),
        "nothing may be injected"
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "l1/sess-1"),
        "L1 denial lost: {:?}",
        out.governance.denied
    );
    assert!(
        out.governance
            .denied
            .iter()
            .any(|ns| ns == "persona/sess-1"),
        "persona denial lost: {:?}",
        out.governance.denied
    );
    assert!(
        out.governance.denied.iter().any(|ns| ns == "scene/sess-1"),
        "scene denial lost: {:?}",
        out.governance.denied
    );

    // Regression: an allow-all pass with nothing to inject stays `Ok(None)`
    // (wire unchanged for the default configuration).
    let empty = perform_auto_recall_governed(
        &db,
        params("nothing matches this at all", "sess-empty"),
        None,
        &InjectionPolicy::allow_all(),
    )
    .expect("recall");
    assert!(empty.is_none(), "allow-all empty must stay Ok(None)");
}
