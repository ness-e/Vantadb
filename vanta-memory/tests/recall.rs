// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-18 D19 — auto-recall (prepend/append split, 3 modes), memory-prompt
//! resolver/composer, and profile sync. All storage runs against an in-memory
//! VantaDB; no LLM involved (recall is LLM-free).

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType, PersonaMode, SceneIndexEntry};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallMode, RecallScope,
};
use vanta_memory::core::memory_prompt::{
    compose_memory_system_prompt, resolve_memory_prompt, MemoryPromptLayer, MemoryPromptRecord,
    MemoryPromptSettingRecord, MemoryPromptSource, MemoryPromptStore, PromptStatus, ResolveTarget,
};
use vanta_memory::core::persona::persona_generator::PersonaRecord;
use vanta_memory::core::profile::{
    build_profile_isolation_scope, parse_profile_isolation_scope, read_scoped_persona,
    sync_persona_to_scope, ProfileIsolation,
};
use vanta_memory::core::record::l1_reader::l1_namespace;
use vanta_memory::core::scene::scene_navigation::generate_scene_navigation;
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;

fn db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
}

fn record(id: &str, content: &str) -> MemoryRecord {
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
fn write_persona(db: &VantaEmbedded, body: &str) {
    let ns = "persona/sess-1";
    // Navigation footer in the exact format MEM-15 generates (strip depends
    // on its header).
    let nav = generate_scene_navigation(&[SceneIndexEntry {
        filename: "ui-setup".into(),
        summary: "summary".into(),
        heat: 3,
        created: "2026-08-20T10:00:00Z".into(),
        updated: "2026-08-20T11:00:00Z".into(),
    }]);
    let record = PersonaRecord {
        content: format!("{body}\n\n{nav}\n"),
        mode: PersonaMode::First,
        generated_at_ms: 1_000,
        generated_at: "2026-08-20T10:00:00Z".into(),
    };
    use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata};
    db.put(VantaMemoryInput {
        namespace: ns.into(),
        key: "persona.md".into(),
        payload: serde_json::to_string(&record).unwrap(),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put persona");
}

/// Persist an L1 record exactly as `read_session_records` expects it.
fn put_l1(db: &VantaEmbedded, record: &MemoryRecord) {
    use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata};
    db.put(VantaMemoryInput {
        namespace: l1_namespace(&record.session_key),
        key: record.id.clone(),
        payload: serde_json::to_string(record).unwrap(),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put l1 record");
}

#[test]
fn recall_splits_dynamic_prepend_from_stable_append() {
    let db = db();
    put_l1(&db, &record("m1", "user prefers dark mode"));
    write_persona(&db, "The user is a night owl.");

    let out = perform_auto_recall(
        &db,
        AutoRecallParams {
            user_text: "what does the user prefer about dark mode?",
            session_key: "sess-1",
            isolation: Some(ProfileIsolation::default()),
            config: RecallConfig::default(),
        },
        None,
    )
    .expect("recall")
    .expect("content to inject");

    // Dynamic L1 memories → prepend.
    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.starts_with("<relevant-memories>"));
    assert!(prepend.contains("[persona|ui-setup] user prefers dark mode"));
    assert!(!prepend.contains("<user-persona>"));

    // Stable persona + guide → append.
    let append = out.append_system_context.expect("append");
    assert!(append.contains("<user-persona>\nThe user is a night owl.\n</user-persona>"));
    assert!(append.contains("<memory-tools-guide>"));
    assert_eq!(out.effective_mode, RecallMode::Keyword); // hybrid degraded
    assert_eq!(out.recalled_memories.len(), 1);
    assert_eq!(out.recalled_memories[0].score, 3); // {user, dark, mode}
}

#[test]
fn empty_user_text_still_injects_persona_and_scenes() {
    let db = db();
    write_persona(&db, "Persona body only.");

    let out = perform_auto_recall(
        &db,
        AutoRecallParams {
            user_text: "   ",
            session_key: "sess-1",
            isolation: None,
            config: RecallConfig::default(),
        },
        None,
    )
    .expect("recall")
    .expect("persona to inject");

    assert!(out.prepend_context.is_none());
    assert!(out
        .append_system_context
        .expect("append")
        .contains("Persona body only."));
}

#[test]
fn nothing_to_inject_returns_none() {
    let db = db();
    let out = perform_auto_recall(
        &db,
        AutoRecallParams {
            user_text: "anything",
            session_key: "sess-1",
            isolation: None,
            config: RecallConfig::default(),
        },
        None,
    )
    .expect("recall");
    assert!(out.is_none());
}

#[test]
fn all_three_modes_declared_and_degrade_to_keyword() {
    for mode in [
        RecallMode::Keyword,
        RecallMode::Embedding,
        RecallMode::Hybrid,
    ] {
        let db = db();
        put_l1(&db, &record("m1", "user prefers dark mode"));
        let out = perform_auto_recall(
            &db,
            AutoRecallParams {
                user_text: "dark mode preferences",
                session_key: "sess-1",
                isolation: None,
                config: RecallConfig {
                    mode,
                    ..RecallConfig::default()
                },
            },
            None,
        )
        .expect("recall")
        .expect("hit");
        assert_eq!(out.effective_mode, RecallMode::Keyword);
        assert_eq!(out.recalled_memories.len(), 1);
    }
}

#[test]
fn budget_limits_total_prepended_chars() {
    let db = db();
    for i in 0..5 {
        put_l1(
            &db,
            &record(&format!("m{i}"), &format!("shared topic item{i}")),
        );
    }
    let out = perform_auto_recall(
        &db,
        AutoRecallParams {
            user_text: "shared topic",
            session_key: "sess-1",
            isolation: None,
            config: RecallConfig {
                max_results: 5,
                max_total_recall_chars: Some(80),
                ..RecallConfig::default()
            },
        },
        None,
    )
    .expect("recall")
    .expect("hits");
    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.chars().count() < 200);
    assert!(out.recalled_memories.len() < 5);
}

// ── profile sync ──

#[test]
fn persona_sync_is_idempotent_and_scoped_read_works() {
    let db = db();
    write_persona(&db, "Stable persona body.");
    let iso = ProfileIsolation {
        team_id: "team-a".into(),
        agent_id: "agent-1".into(),
    };

    let first = sync_persona_to_scope(&db, "sess-1", &iso).expect("sync");
    assert!(first.updated);
    assert!(!first.skipped_no_persona);

    // Re-run: unchanged content → no rewrite.
    let second = sync_persona_to_scope(&db, "sess-1", &iso).expect("sync");
    assert!(!second.updated);

    // Scoped read returns the navigation-stripped body.
    let body = read_scoped_persona(&db, &iso).expect("read").expect("body");
    assert_eq!(body, "Stable persona body.");
    assert!(!body.contains("scene-navigation"));

    // No persona → skipped, not an error.
    let none = sync_persona_to_scope(&db, "other-session", &iso).expect("sync");
    assert!(none.skipped_no_persona);
}

#[test]
fn scope_build_parse_roundtrip() {
    let iso = ProfileIsolation {
        team_id: "t".into(),
        agent_id: "a".into(),
    };
    let scope = build_profile_isolation_scope(&iso);
    assert_eq!(parse_profile_isolation_scope(&scope), Some(iso));
}

// ── memory prompt resolver + composer ──

#[derive(Default)]
struct FakeStore {
    setting: Option<MemoryPromptSettingRecord>,
    prompt: Option<MemoryPromptRecord>,
}

impl MemoryPromptStore for FakeStore {
    fn get_setting(
        &self,
        _setting_id: &str,
    ) -> Result<
        Option<MemoryPromptSettingRecord>,
        vanta_memory::core::memory_prompt::MemoryPromptError,
    > {
        Ok(self.setting.clone())
    }

    fn get_prompt(
        &self,
        _prompt_id: &str,
    ) -> Result<Option<MemoryPromptRecord>, vanta_memory::core::memory_prompt::MemoryPromptError>
    {
        Ok(self.prompt.clone())
    }
}

fn agent_setting(layer: MemoryPromptLayer) -> (MemoryPromptSettingRecord, MemoryPromptRecord) {
    (
        MemoryPromptSettingRecord {
            setting_id: "mps:a/t/a/l1".into(),
            target_type: "agent".into(),
            team_id: Some("t".into()),
            agent_id: Some("a".into()),
            layer,
            memory_prompt_id: "mp_1".into(),
            updated_at_ms: 1,
        },
        MemoryPromptRecord {
            memory_prompt_id: "mp_1".into(),
            name: "strategy".into(),
            layer,
            prompt: "focus on rules".into(),
            version: 2,
            status: PromptStatus::Active,
            created_at_ms: 1,
            updated_at_ms: 1,
        },
    )
}

#[test]
fn resolver_finds_active_agent_prompt_and_composer_appends_it() {
    let (setting, prompt) = agent_setting(MemoryPromptLayer::L1);
    let store = FakeStore {
        setting: Some(setting),
        prompt: Some(prompt),
    };
    let resolved = resolve_memory_prompt(
        &store,
        ResolveTarget {
            team_id: Some("t"),
            agent_id: Some("a"),
            layer: MemoryPromptLayer::L1,
        },
    )
    .expect("resolve")
    .expect("resolved");
    assert_eq!(resolved.source, MemoryPromptSource::Agent);

    let composed = compose_memory_system_prompt("SYSTEM", Some(&resolved));
    assert!(composed.starts_with("SYSTEM"));
    assert!(composed.contains("focus on rules"));
    assert!(composed.contains("<SYSTEM_CUSTOM_STRATEGY_GUARD priority=\"highest\">"));
}

#[test]
fn composer_passthrough_without_resolution() {
    assert_eq!(compose_memory_system_prompt("SYSTEM", None), "SYSTEM");
}

// ── MEM-40: recall scope híbrido (D22) ──

/// L1 record with explicit cross-session scope metadata.
fn scoped_record(
    id: &str,
    content: &str,
    session_key: &str,
    team_id: Option<&str>,
    agent_id: Option<&str>,
) -> MemoryRecord {
    MemoryRecord {
        session_key: session_key.into(),
        team_id: team_id.map(Into::into),
        agent_id: agent_id.map(Into::into),
        ..record(id, content)
    }
}

fn recall_with_scope(
    db: &VantaEmbedded,
    scope: RecallScope,
    isolation: ProfileIsolation,
) -> Option<vanta_memory::core::hooks::RecallResult> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text: "user prefers dark mode",
            session_key: "sess-1",
            isolation: Some(isolation),
            config: RecallConfig {
                scope,
                ..RecallConfig::default()
            },
        },
        None,
    )
    .expect("recall")
}

#[test]
fn scope_session_replicates_current_behavior() {
    let db = db();
    put_l1(&db, &record("m1", "user prefers dark mode"));
    // Same agent, different session — NOT visible under Session scope.
    put_l1(
        &db,
        &scoped_record(
            "m2",
            "user prefers dark mode too",
            "sess-2",
            None,
            Some("agent-1"),
        ),
    );

    let out = recall_with_scope(
        &db,
        RecallScope::Session,
        ProfileIsolation {
            team_id: "team-a".into(),
            agent_id: "agent-1".into(),
        },
    )
    .expect("content");

    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.contains("dark mode"));
    assert!(
        !prepend.contains("too"),
        "cross-session record leaked into Session scope"
    );
    assert_eq!(out.recalled_memories.len(), 1);
}

#[test]
fn scope_agent_finds_other_session_of_same_agent() {
    let db = db();
    put_l1(&db, &record("m1", "user prefers dark mode"));
    put_l1(
        &db,
        &scoped_record(
            "m2",
            "user likes vim editor",
            "sess-2",
            None,
            Some("agent-1"),
        ),
    );

    let out = recall_with_scope(
        &db,
        RecallScope::Agent,
        ProfileIsolation {
            team_id: "team-a".into(),
            agent_id: "agent-1".into(),
        },
    )
    .expect("content");

    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.contains("dark mode"));
    assert!(
        prepend.contains("vim editor"),
        "same-agent cross-session record not recalled"
    );
}

#[test]
fn scope_team_filters_by_team_id_across_agents() {
    let db = db();
    put_l1(&db, &record("m1", "user prefers dark mode"));
    // Different agent, same team — visible under Team scope only.
    put_l1(
        &db,
        &scoped_record(
            "m2",
            "user wants dark dashboards",
            "sess-3",
            Some("team-a"),
            Some("agent-9"),
        ),
    );

    let out = recall_with_scope(
        &db,
        RecallScope::Team,
        ProfileIsolation {
            team_id: "team-a".into(),
            agent_id: "agent-1".into(),
        },
    )
    .expect("content");

    let prepend = out.prepend_context.expect("prepend");
    assert!(
        prepend.contains("dashboards"),
        "same-team record not recalled under Team scope"
    );

    // Same record is invisible to a DIFFERENT team.
    let other_team = recall_with_scope(
        &db,
        RecallScope::Team,
        ProfileIsolation {
            team_id: "team-b".into(),
            agent_id: "agent-1".into(),
        },
    );
    if let Some(out) = other_team {
        if let Some(prepend) = out.prepend_context {
            assert!(!prepend.contains("dashboards"));
        }
    }
}

#[test]
fn scope_agent_isolates_other_agents_and_legacy_records() {
    let db = db();
    put_l1(&db, &record("m1", "user prefers dark mode"));
    // Agent B's memory.
    put_l1(
        &db,
        &scoped_record(
            "mb",
            "agent b stores dark mode secrets",
            "sess-b",
            Some("team-a"),
            Some("agent-b"),
        ),
    );
    // Legacy record without metadata in another session.
    put_l1(
        &db,
        &scoped_record("ml", "legacy dark mode note", "sess-l", None, None),
    );

    let out = recall_with_scope(
        &db,
        RecallScope::Agent,
        ProfileIsolation {
            team_id: "team-a".into(),
            agent_id: "agent-a".into(),
        },
    )
    .expect("own-session content");

    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.contains("dark mode")); // own session still visible
    assert!(
        !prepend.contains("secrets"),
        "ISOLATION LEAK: agent A saw agent B's memory"
    );
    assert!(
        !prepend.contains("legacy"),
        "metadata-less record from another session must stay session-only"
    );
}

#[test]
fn search_multi_merges_hits_across_namespaces() {
    use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata, VantaMemorySearchRequest};

    let db = db();
    for (ns, key, text) in [
        ("l1/sess-a", "m1", "user prefers dark mode"),
        ("l1/sess-b", "m2", "team uses dark dashboards"),
    ] {
        db.put(VantaMemoryInput {
            namespace: ns.into(),
            key: key.into(),
            payload: text.into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .expect("put l1 payload");
    }

    let hits = db
        .search_multi(
            &["l1/sess-a", "l1/sess-b"],
            VantaMemorySearchRequest {
                text_query: Some("dark".into()),
                top_k: 10,
                ..VantaMemorySearchRequest::default()
            },
        )
        .expect("search_multi");

    let namespaces: std::collections::HashSet<&str> =
        hits.iter().map(|h| h.record.namespace.as_str()).collect();
    assert!(
        namespaces.contains("l1/sess-a") && namespaces.contains("l1/sess-b"),
        "search_multi did not cover both namespaces: {namespaces:?}"
    );
    for pair in hits.windows(2) {
        assert!(
            pair[0].score >= pair[1].score,
            "hits not sorted by descending score"
        );
    }
}
