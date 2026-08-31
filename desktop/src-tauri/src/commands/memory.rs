//! Memory-pipeline IPC commands (MEM-53 / H4): expose the vanta-memory
//! pipeline (L0 capture, recall, persona, scenes, skills, wiki status)
//! over the active NATIVE (embedded) connection.
//!
//! Pattern: every command clones the embedded handle out of the active
//! connection ([`crate::connections::ConnectionManager::active_embedded`])
//! and runs the sync vanta-memory APIs on the blocking pool — same pattern
//! as the native adapter. Errors propagate as [`crate::error::VantaError`].
//! Non-native active connections fail with `Unsupported`.

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::task::spawn_blocking;

use vantadb::sdk::{VantaMemoryListOptions, VantaMemoryListPage};
use vantadb::VantaEmbedded;

use vanta_memory::context_engine::{
    assemble_with_recall, AssembleConfig, ChatMessage, IntegratedContext, TokenEstimator,
};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoCaptureConfig, AutoCaptureHook, AutoRecallParams, RawMessage,
    RecallConfig, RecallMode, RecallScope, RecalledMemory,
};
use vanta_memory::core::memory_generation_log::{
    query_session, GenerationLayer, GenerationLogEntry,
};
use vanta_memory::core::persona::get_persona;
use vanta_memory::core::scene::{current_scene, list_scenes, upsert_scene, SceneBlock};
use vanta_memory::core::skill::conversation_add::StoredSkill;
use vanta_memory::gateway::{
    scene_query as gateway_scene_query, scene_read as gateway_scene_read, SceneQueryHit,
    SceneQueryRequest, SceneReadRequest,
};
use vanta_memory::ingest::callback::IngestProgress;
#[cfg(test)]
use vanta_memory::ingest::callback::ProgressTracker;

use crate::connections::ConnectionManager;
use crate::error::VantaError;

// Wire DTOs ──────────────────────────────────────────────────────────────

/// One raw conversation message as delivered by the frontend (MEM-53).
///
/// Mirrors `vanta_memory::RawMessage` with owned/optional fields because
/// Tauri deserializes IPC args into owned values.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryMessage {
    /// Stable message id when the host has one; `None` falls back to a
    /// derived `t{timestamp_ms}_{index}` key.
    #[serde(default)]
    pub id: Option<String>,
    /// Host role string (`user` | `assistant`; others are filtered).
    pub role: String,
    pub content: String,
    /// Unix-ms timestamp; `None` falls back to the recorder's clock.
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
}

/// Outcome of one L0 capture pass (`vanta_memory_capture`).
///
/// Mirror of `vanta_memory::AutoCaptureResult`, which is not `Serialize`.
#[derive(Debug, Clone, Serialize)]
pub struct CaptureOutcome {
    pub recorded_count: usize,
    /// Messages dropped by role filter, empty-content filter, or cursor.
    pub filtered_messages: usize,
    /// New cursor value after the pass.
    pub cursor_ms: u64,
}

/// Outcome of one auto-recall pass (`vanta_memory_recall`).
///
/// Mirror of `vanta_memory::RecallResult` (not `Serialize`); `Ok(None)`
/// from the hook becomes `Ok(None)` here too — never an empty block.
#[derive(Debug, Clone, Serialize)]
pub struct RecallOutcome {
    /// L1 relevant memories — dynamic per-turn, prepend to user prompt.
    pub prepend_context: Option<String>,
    /// Persona + scene navigation + tools guide — stable, cache-friendly.
    pub append_system_context: Option<String>,
    pub recalled_memories: Vec<RecalledMemory>,
    pub persona: Option<String>,
    pub effective_mode: RecallMode,
}

// Sync cores (run on the blocking pool) ──────────────────────────────────

fn mem_err(e: impl std::fmt::Display) -> VantaError {
    VantaError::Native(e.to_string())
}

/// Run a sync closure on the blocking pool, mapping join failures.
async fn offload<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, VantaError> + Send + 'static,
) -> Result<T, VantaError> {
    spawn_blocking(f)
        .await
        .map_err(|e| VantaError::Native(format!("blocking task failed: {e}")))?
}

/// L0 capture: filter roles → sanitize → record via the idempotent recorder.
fn run_capture(
    db: &VantaEmbedded,
    session_id: &str,
    messages: Vec<MemoryMessage>,
) -> Result<CaptureOutcome, VantaError> {
    let msgs = messages
        .into_iter()
        .map(|m| RawMessage {
            id: m.id,
            role: m.role,
            content: m.content,
            timestamp_ms: m.timestamp_ms,
        })
        .collect();
    let hook = AutoCaptureHook::new(db.clone(), AutoCaptureConfig::default());
    let result = hook.capture(session_id, msgs).map_err(mem_err)?;
    Ok(CaptureOutcome {
        recorded_count: result.recorded_count,
        filtered_messages: result.filtered_messages,
        cursor_ms: result.cursor_ms,
    })
}

/// Auto-recall: relevant L1 memories + persona + scene navigation.
fn run_recall(
    db: &VantaEmbedded,
    user_text: &str,
    session_key: &str,
    config: RecallConfig,
) -> Result<Option<RecallOutcome>, VantaError> {
    let params = AutoRecallParams {
        user_text,
        session_key,
        isolation: None,
        config,
    };
    // No embedding hook: embedding/hybrid degrade to keyword (crate contract).
    Ok(perform_auto_recall(db, params, None)
        .map_err(mem_err)?
        .map(|r| RecallOutcome {
            prepend_context: r.prepend_context,
            append_system_context: r.append_system_context,
            recalled_memories: r.recalled_memories,
            persona: r.persona,
            effective_mode: r.effective_mode,
        }))
}

/// Every stored skill across all `skills_extract/*` namespaces, most
/// recently updated first. Delegation-only: namespaces come from the SDK,
/// payloads parse as `StoredSkill`.
fn run_skills_list(db: &VantaEmbedded) -> Result<Vec<StoredSkill>, VantaError> {
    let mut skills = Vec::new();
    for ns in db.list_namespaces().map_err(mem_err)? {
        if !ns.starts_with("skills_extract/") {
            continue;
        }
        let mut cursor: Option<usize> = None;
        loop {
            let page: VantaMemoryListPage = db
                .list(
                    &ns,
                    VantaMemoryListOptions {
                        limit: 1000,
                        cursor,
                        ..Default::default()
                    },
                )
                .map_err(mem_err)?;
            for record in &page.records {
                if let Ok(skill) = serde_json::from_str::<StoredSkill>(&record.payload) {
                    skills.push(skill);
                }
            }
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
    }
    skills.sort_by_key(|s| std::cmp::Reverse(s.updated_at_ms));
    Ok(skills)
}

/// Real context-engine consolidation (MEM-58): compress → inject MMD →
/// inject recall blocks via [`assemble_with_recall`] against ONE shared
/// token budget. Recall blocks come from the same auto-recall hook as
/// `vanta_memory_recall` when both `user_text` and `session_key` are given;
/// without them the engine still compacts but injects nothing. The full
/// [`IntegratedContext`] (report mode/tokens + injected messages) travels
/// back over IPC — the wire type IS the engine type (already serde).
fn run_assemble(
    db: &VantaEmbedded,
    messages: Vec<ChatMessage>,
    budget_tokens: u64,
    user_text: Option<String>,
    session_key: Option<String>,
) -> Result<IntegratedContext, VantaError> {
    let (prepend, append) = match (user_text, session_key) {
        (Some(q), Some(sk)) if !q.trim().is_empty() => {
            match run_recall(db, &q, &sk, RecallConfig::default())? {
                Some(r) => (r.prepend_context, r.append_system_context),
                None => (None, None),
            }
        }
        _ => (None, None),
    };
    assemble_with_recall(
        messages,
        budget_tokens,
        &TokenEstimator::default(),
        0,
        &AssembleConfig::default(),
        None,
        prepend.as_deref(),
        append.as_deref(),
        None,
        None,
    )
    .map_err(mem_err)
}

// Commands ───────────────────────────────────────────────────────────────

/// Capture a conversation turn into the L0 layer (MEM-53). Idempotent per
/// cursor; system roles and empty content are filtered, never lost silently.
#[tauri::command]
pub async fn vanta_memory_capture(
    state: State<'_, crate::AppState>,
    session_id: String,
    messages: Vec<MemoryMessage>,
) -> Result<CaptureOutcome, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || run_capture(&db, &session_id, messages)).await
}

/// Auto-recall for the current turn: relevant memories + persona + scene
/// navigation. `scope` widens the L1 pool cross-session (default: agent);
/// `None` result means "nothing to inject".
#[tauri::command]
pub async fn vanta_memory_recall(
    state: State<'_, crate::AppState>,
    user_text: String,
    session_key: String,
    scope: Option<RecallScope>,
) -> Result<Option<RecallOutcome>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || {
        let config = RecallConfig {
            scope: scope.unwrap_or_default(),
            ..Default::default()
        };
        run_recall(&db, &user_text, &session_key, config)
    })
    .await
}

/// Read the stored persona record of a session (`None` = none generated yet).
#[tauri::command]
pub async fn vanta_persona_get(
    state: State<'_, crate::AppState>,
    session_key: String,
) -> Result<Option<vanta_memory::core::persona::PersonaRecord>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || get_persona(&db, &session_key).map_err(mem_err)).await
}

/// List the scene index entries of a session (heat-desc navigation order).
#[tauri::command]
pub async fn vanta_scenes_list(
    state: State<'_, crate::AppState>,
    session_key: String,
) -> Result<Vec<vanta_memory::core::abstractions::SceneIndexEntry>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || list_scenes(&db, &session_key).map_err(mem_err)).await
}

/// The current (most recently updated) scene block of a session, if any.
#[tauri::command]
pub async fn vanta_scene_current(
    state: State<'_, crate::AppState>,
    session_key: String,
) -> Result<Option<vanta_memory::core::scene::SceneBlock>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || current_scene(&db, &session_key).map_err(mem_err)).await
}

/// Every stored extracted skill across `skills_extract/*` namespaces,
/// most recently updated first.
#[tauri::command]
pub async fn vanta_skills_list(
    state: State<'_, crate::AppState>,
) -> Result<Vec<StoredSkill>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || run_skills_list(&db)).await
}

/// Consolidate a chat history through the REAL context engine (MEM-58):
/// shared-budget compress → recall injection, with the engine report
/// (mode/tokens) as outcome. Non-native active connections fail with
/// `Unsupported` → the UI falls back to its client-side heuristic.
// rename_all snake_case: el wrapper TS envía las claves tal cual Rust
// (`budget_tokens`, `session_key`) — el default camelCase de Tauri v2
// no las matchearía ("missing required key budgetTokens").
#[tauri::command(rename_all = "snake_case")]
pub async fn vanta_context_assemble(
    state: State<'_, crate::AppState>,
    messages: Vec<ChatMessage>,
    budget_tokens: u64,
    user_text: Option<String>,
    session_key: Option<String>,
) -> Result<IntegratedContext, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || run_assemble(&db, messages, budget_tokens, user_text, session_key)).await
}

/// Poll wiki-ingest progress for `run_id` (D32 polling channel). Workers
/// push snapshots into the shared [`ProgressTracker`]; an unknown/stale
/// run returns `None`. Desktop ingest runs land in a later task — until
/// then this reports `None` for every run.
#[tauri::command]
pub async fn vanta_wiki_status(
    state: State<'_, crate::AppState>,
    run_id: String,
) -> Result<Option<IngestProgress>, VantaError> {
    Ok(state.progress.wiki_status(&run_id))
}

// Knowledge observability (DESKTOP-36) — read-only gateway over the typed
// knowledge handlers. No writes in v1.

/// Read one live scene block by name (DESKTOP-36). Missing or soft-deleted
/// scenes surface `NotFound` through the gateway envelope.
#[tauri::command(rename_all = "snake_case")]
pub async fn vanta_scene_read(
    state: State<'_, crate::AppState>,
    session_key: String,
    scene_name: String,
) -> Result<SceneBlock, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || {
        gateway_scene_read(&db, &SceneReadRequest { session_key, scene_name })
            .map(|r| r.scene)
            .map_err(mem_err)
    })
    .await
}

/// Keyword search over the live scenes of a session (DESKTOP-36), ranked by
/// term overlap (ties: heat desc). No embed hook — the desktop build is
/// LLM-free, so the gateway degrades to the keyword pool (crate contract).
#[tauri::command(rename_all = "snake_case")]
pub async fn vanta_scene_query(
    state: State<'_, crate::AppState>,
    session_key: String,
    keyword: String,
    top_k: Option<usize>,
) -> Result<Vec<SceneQueryHit>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || {
        gateway_scene_query(
            &db,
            &SceneQueryRequest {
                session_key,
                keyword,
                top_k,
            },
            None,
        )
        .map(|r| r.hits)
        .map_err(mem_err)
    })
    .await
}

/// Consult a session's generation log (MEM-41 provenance, DESKTOP-36):
/// entries ordered oldest → newest, optionally filtered by pipeline layer
/// (`l1` | `l2` | `l3`) and capped by `limit`.
#[tauri::command(rename_all = "snake_case")]
pub async fn vanta_genlog_query(
    state: State<'_, crate::AppState>,
    session_key: String,
    layer: Option<GenerationLayer>,
    limit: Option<usize>,
) -> Result<Vec<GenerationLogEntry>, VantaError> {
    let db = state.manager.active_embedded().await?;
    offload(move || {
        let mut entries = query_session(&db, &session_key, layer).map_err(mem_err)?;
        if let Some(max) = limit {
            entries.truncate(max);
        }
        Ok(entries)
    })
    .await
}

/// Convenience for tests/tools: seed one scene block through the public
/// scene index (kept `pub(crate)`-free — used by the roundtrip tests below).
#[allow(dead_code)]
async fn seed_scene(
    manager: &ConnectionManager,
    session_key: &str,
    name: &str,
    summary: &str,
    content: &str,
) -> Result<(), VantaError> {
    let db = manager.active_embedded().await?;
    let session = session_key.to_string();
    let (name, summary, content) = (name.to_string(), summary.to_string(), content.to_string());
    offload(move || {
        upsert_scene(&db, &session, &name, &summary, &content)
            .map(|_| ())
            .map_err(mem_err)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::native::NativeConnection;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let seq = DIR_SEQ.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "vantadb-desktop-mem53-{}-{seq}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// App state over a fresh native temp-dir connection (made active).
    async fn state() -> (crate::AppState, TempDir) {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(NativeConnection::open(dir.path()).expect("open")))
            .await
            .expect("add");
        (
            crate::AppState {
                manager,
                config: vantadb::config::VantaConfig::default(),
                pending_deep_links: Default::default(),
                progress: ProgressTracker::default(),
                embeddings: Default::default(),
            },
            dir,
        )
    }

    fn msg(role: &str, content: &str) -> MemoryMessage {
        MemoryMessage {
            id: None,
            role: role.into(),
            content: content.into(),
            timestamp_ms: None,
        }
    }

    /// Same as [`msg`] but with a fixed unix-ms timestamp (cursor tests).
    fn msg_at(role: &str, content: &str, ts: u64) -> MemoryMessage {
        MemoryMessage {
            id: None,
            role: role.into(),
            content: content.into(),
            timestamp_ms: Some(ts),
        }
    }

    // ── capture (L0 roundtrip against the embedded DB) ──

    #[tokio::test]
    async fn capture_records_user_and_assistant_turns() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");

        let out = offload(move || {
            run_capture(
                &db,
                "sess-cap",
                vec![
                    msg("user", "remember that deploy uses the red runner"),
                    msg("assistant", "noted, the red runner deploys."),
                    msg("system", "internal noise filtered"),
                    msg("user", "   "),
                ],
            )
        })
        .await
        .expect("capture");

        assert_eq!(out.recorded_count, 2, "user + assistant recorded");
        assert_eq!(out.filtered_messages, 2, "system + blank filtered");
        assert!(out.cursor_ms > 0);

        // Roundtrip: the L0 namespace exists in the embedded store.
        let l0: Vec<String> = st
            .manager
            .active_embedded()
            .await
            .expect("handle")
            .list_namespaces()
            .expect("namespaces")
            .into_iter()
            .filter(|ns| ns.starts_with("l0/"))
            .collect();
        assert_eq!(l0.len(), 1, "one L0 namespace per session: {l0:?}");
    }

    #[tokio::test]
    async fn capture_is_idempotent_per_cursor() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let first = offload({
            let db = db.clone();
            move || {
                run_capture(
                    &db,
                    "sess-idem",
                    vec![msg_at("user", "idempotent turn body", 1_000)],
                )
            }
        })
        .await
        .expect("first");
        let second = offload(move || {
            run_capture(
                &db,
                "sess-idem",
                vec![msg_at("user", "idempotent turn body", 1_000)],
            )
        })
        .await
        .expect("second");

        assert_eq!(first.recorded_count, 1);
        assert_eq!(second.recorded_count, 0, "cursor makes the replay a no-op");
        assert_eq!(second.filtered_messages, 1);
    }

    // ── recall ──

    #[tokio::test]
    async fn recall_injects_scene_navigation_and_degrades_to_keyword() {
        let (st, _dir) = state().await;

        // Seed a scene through the public scene index; recall injects the
        // navigation block even with empty user text (TDAM parity).
        seed_scene(
            &st.manager,
            "sess-rec",
            "deploy-runbook",
            "deploys",
            "how to deploy",
        )
        .await
        .expect("seed scene");

        let db = st.manager.active_embedded().await.expect("handle");
        let out = offload(move || run_recall(&db, "", "sess-rec", RecallConfig::default()))
            .await
            .expect("recall")
            .expect("navigation to inject");

        assert!(out.recalled_memories.is_empty(), "no L1 yet: {out:?}");
        assert!(matches!(out.effective_mode, RecallMode::Keyword));
        let system_ctx = out.append_system_context.expect("system context");
        assert!(
            system_ctx.contains("<scene-navigation>"),
            "scene nav injected: {system_ctx:?}"
        );
    }

    #[tokio::test]
    async fn recall_returns_none_when_nothing_to_inject() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let out = offload(move || {
            run_recall(
                &db,
                "unrelated query words",
                "empty-session",
                RecallConfig::default(),
            )
        })
        .await
        .expect("recall");
        assert!(out.is_none(), "no persona/scenes/memories → None: {out:?}");
    }

    // ── persona / scenes ──

    #[tokio::test]
    async fn persona_get_none_before_generation() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let persona = offload(move || get_persona(&db, "no-persona").map_err(mem_err))
            .await
            .expect("read");
        assert!(persona.is_none());
    }

    #[tokio::test]
    async fn scenes_list_and_current_roundtrip_through_embedded_db() {
        let (st, _dir) = state().await;

        seed_scene(
            &st.manager,
            "sess-scene",
            "deploy-runbook",
            "deploys",
            "how to deploy",
        )
        .await
        .expect("seed scene 1");
        seed_scene(
            &st.manager,
            "sess-scene",
            "oncall-notes",
            "oncall",
            "pager tips",
        )
        .await
        .expect("seed scene 2");

        let db = st.manager.active_embedded().await.expect("handle");
        let entries = offload(move || list_scenes(&db, "sess-scene").map_err(mem_err))
            .await
            .expect("list");
        assert_eq!(entries.len(), 2, "{entries:?}");
        assert!(entries.iter().any(|e| e.filename == "deploy-runbook"));

        let db = st.manager.active_embedded().await.expect("handle");
        let current = offload(move || current_scene(&db, "sess-scene").map_err(mem_err))
            .await
            .expect("current")
            .expect("live scene exists");
        assert!(!current.scene_name.is_empty(), "block present");
    }

    #[tokio::test]
    async fn scenes_empty_session_lists_nothing() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let entries = offload(move || list_scenes(&db, "ghost").map_err(mem_err)).await;
        assert!(entries.expect("list").is_empty());
    }

    // ── skills ──

    #[tokio::test]
    async fn skills_list_roundtrips_written_candidates() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");

        // Write through the crate's own sink (the production write path).
        let candidate = vanta_memory::core::skill::skill_extractor::ExtractedSkillCandidate {
            action: "create".into(),
            name: "rotate-keys".into(),
            description: "rotation runbook".into(),
            content: "steps to rotate keys".into(),
        };
        offload(move || {
            let sink = vanta_memory::core::skill::conversation_add::SkillCoreSink::new(&db);
            sink.apply_candidates("agent-1", "task-1", &[candidate], 100)
                .map_err(mem_err)
        })
        .await
        .expect("apply")
        .expect("applied once");

        let db = st.manager.active_embedded().await.expect("handle");
        let skills = offload(move || run_skills_list(&db)).await.expect("list");
        assert_eq!(skills.len(), 1, "{skills:?}");
        assert_eq!(skills[0].name, "rotate-keys");
        assert!(skills[0].content_hash != 0);
    }

    #[tokio::test]
    async fn skills_list_empty_store_returns_empty() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        assert!(offload(move || run_skills_list(&db))
            .await
            .expect("list")
            .is_empty());
    }

    // ── wiki status ──

    #[tokio::test]
    async fn wiki_status_polls_tracker_by_run_id() {
        let (st, _dir) = state().await;
        assert!(st.progress.wiki_status("unknown-run").is_none());

        // Simulate a worker pushing a snapshot for its run.
        st.progress.begin_run("run-42");
        st.progress.update_progress(IngestProgress::new(
            "run-42",
            vanta_memory::ingest::callback::IngestPhase::Extracting,
            4,
            2,
            0,
            1,
        ));
        let snap = st
            .progress
            .wiki_status("run-42")
            .expect("snapshot for active run");
        assert_eq!(snap.run_id, "run-42");
        assert_eq!(snap.percent, 50);
    }

    // ── context assemble (MEM-58) ──

    fn chat(role: vanta_memory::context_engine::ChatRole, content: &str) -> ChatMessage {
        ChatMessage::new(role, content)
    }

    fn filler_history() -> Vec<ChatMessage> {
        std::iter::once(chat(
            vanta_memory::context_engine::ChatRole::System,
            "protected system prompt",
        ))
        .chain((0..40).map(|i| {
            chat(
                vanta_memory::context_engine::ChatRole::User,
                &format!("filler message {i:03} with padding text"),
            )
        }))
        .collect()
    }

    /// E2E contract (backend available): the REAL pipeline runs — recall
    /// blocks from the auto-recall hook are injected and the engine report
    /// travels back with mode/tokens.
    #[tokio::test]
    async fn assemble_with_backend_runs_real_pipeline_and_injects_recall() {
        let (st, _dir) = state().await;
        seed_scene(
            &st.manager,
            "sess-asm",
            "deploy-runbook",
            "deploys",
            "how to deploy",
        )
        .await
        .expect("seed scene");

        let db = st.manager.active_embedded().await.expect("handle");
        let out = offload(move || {
            run_assemble(
                &db,
                filler_history(),
                10_000,
                Some("deploy question".into()),
                Some("sess-asm".into()),
            )
        })
        .await
        .expect("assemble");

        assert!(out.recall_injected, "scene nav injected: {:?}", out.report);
        assert!(
            out.messages
                .iter()
                .any(|m| m.content.contains("<scene-navigation>")),
            "recall block present in assembled messages"
        );
        assert!(
            out.report.tokens_after > 0,
            "report carries tokens: {:?}",
            out.report
        );
        assert_eq!(out.report.msgs_before, 41);
    }

    /// Compaction path: over-budget history gets compacted by the engine;
    /// without session context nothing is recalled.
    #[tokio::test]
    async fn assemble_without_session_compacts_but_skips_recall() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let out = offload(move || run_assemble(&db, filler_history(), 200, None, None))
            .await
            .expect("assemble");

        use vanta_memory::context_engine::CompactionMode;
        assert!(
            !matches!(out.report.mode, CompactionMode::None),
            "compaction ran: {:?}",
            out.report
        );
        assert!(
            out.report.tokens_after <= 200,
            "under budget: {:?}",
            out.report
        );
        assert!(
            !out.recall_injected,
            "no session → no recall: {:?}",
            out.report
        );
    }

    #[tokio::test]
    async fn assemble_rejects_zero_budget() {
        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        let err = offload(move || {
            run_assemble(
                &db,
                vec![chat(vanta_memory::context_engine::ChatRole::User, "hi")],
                0,
                None,
                None,
            )
        })
        .await
        .expect_err("zero budget rejected");
        assert!(
            matches!(err, VantaError::Native(ref m) if m.contains("chars_per_token")),
            "InvalidConfig surfaced: {err:?}"
        );
    }

    // ── access guard ──

    #[tokio::test]
    async fn active_embedded_requires_an_active_native_connection() {
        let manager = ConnectionManager::new();
        let err = manager.active_embedded().await.expect_err("no active");
        assert!(
            matches!(err, VantaError::Unsupported(_)),
            "expected Unsupported, got: {err:?}"
        );
    }

    // ── knowledge observability (DESKTOP-36) ──

    #[tokio::test]
    async fn scene_read_returns_seeded_block_and_not_found_for_missing() {
        let (st, _dir) = state().await;
        seed_scene(
            &st.manager,
            "sess-read",
            "deploy-runbook",
            "deploys",
            "how to deploy",
        )
        .await
        .expect("seed scene");

        let db = st.manager.active_embedded().await.expect("handle");
        let block = offload(move || {
            gateway_scene_read(
                &db,
                &SceneReadRequest {
                    session_key: "sess-read".into(),
                    scene_name: "deploy-runbook".into(),
                },
            )
            .map(|r| r.scene)
            .map_err(mem_err)
        })
        .await
        .expect("read");
        assert_eq!(block.content, "how to deploy");
        assert!(!block.deleted);

        let db = st.manager.active_embedded().await.expect("handle");
        let err = offload(move || {
            gateway_scene_read(
                &db,
                &SceneReadRequest {
                    session_key: "sess-read".into(),
                    scene_name: "ghost".into(),
                },
            )
            .map_err(mem_err)
        })
        .await
        .expect_err("missing scene");
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[tokio::test]
    async fn scene_query_ranks_by_keyword_overlap() {
        let (st, _dir) = state().await;
        seed_scene(
            &st.manager,
            "sess-q",
            "deploy-runbook",
            "deploys",
            "deploy the service with the red runner",
        )
        .await
        .expect("seed 1");
        seed_scene(&st.manager, "sess-q", "oncall", "oncall", "pager escalation tips")
            .await
            .expect("seed 2");

        let db = st.manager.active_embedded().await.expect("handle");
        let hits = offload(move || {
            gateway_scene_query(
                &db,
                &SceneQueryRequest {
                    session_key: "sess-q".into(),
                    keyword: "deploy runner".into(),
                    top_k: Some(1),
                },
                None,
            )
            .map(|r| r.hits)
            .map_err(mem_err)
        })
        .await
        .expect("query");
        assert_eq!(hits.len(), 1, "top_k respected: {hits:?}");
        assert_eq!(hits[0].scene_name, "deploy-runbook");
        assert!(hits[0].score > 0);
    }

    #[tokio::test]
    async fn genlog_query_filters_by_layer_and_respects_limit() {
        use vanta_memory::core::memory_generation_log::{try_record, GenerationStatus};

        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        offload(move || -> Result<(), VantaError> {
            for (layer, ts) in [
                (GenerationLayer::L1, 100),
                (GenerationLayer::L2, 200),
                (GenerationLayer::L3, 300),
            ] {
                try_record(
                    &db,
                    &GenerationLogEntry {
                        layer,
                        status: GenerationStatus::Succeeded,
                        anchor_id: None,
                        session_key: "sess-log".into(),
                        ts_ms: ts,
                        error: None,
                    },
                )
                .map_err(mem_err)?;
            }
            Ok(())
        })
        .await
        .expect("record");

        let db = st.manager.active_embedded().await.expect("handle");
        let all =
            offload(move || query_session(&db, "sess-log", None).map_err(mem_err))
                .await
                .expect("all");
        assert_eq!(all.len(), 3);
        assert_eq!(
            all.iter().map(|e| e.ts_ms).collect::<Vec<_>>(),
            vec![100, 200, 300],
            "oldest → newest"
        );

        let db = st.manager.active_embedded().await.expect("handle");
        let l2 = offload(move || {
            query_session(&db, "sess-log", Some(GenerationLayer::L2)).map_err(mem_err)
        })
        .await
        .expect("filtered");
        assert_eq!(l2.len(), 1);
        assert!(matches!(l2[0].layer, GenerationLayer::L2));

        let db = st.manager.active_embedded().await.expect("handle");
        let capped = offload(move || {
            Ok::<_, VantaError>({
                let mut entries = query_session(&db, "sess-log", None).map_err(mem_err)?;
                entries.truncate(2);
                entries
            })
        })
        .await
        .expect("capped");
        assert_eq!(capped.len(), 2);
    }

    /// DESKTOP-36 DoD: the wrappers answer against the REAL vanta-seed import
    /// path (`vanta_memory::seed`, same code the `vanta-seed` binary runs).
    #[tokio::test]
    async fn seeded_store_answers_skills_list_persona_and_genlog() {
        use vanta_memory::core::memory_generation_log::{try_record, GenerationStatus};
        use vanta_memory::seed::import_seed_str;

        let (st, _dir) = state().await;
        let db = st.manager.active_embedded().await.expect("handle");
        offload(move || -> Result<(), VantaError> {
            import_seed_str(
                &db,
                r##"{
                    "scope": "seed",
                    "skills": [
                        { "name": "rotate-keys", "description": "rotation runbook",
                          "content": "steps to rotate keys" }
                    ],
                    "persona": { "session_key": "user-1",
                                 "content": "# Profile\nPrefers concise answers." }
                }"##,
            )
            .map_err(mem_err)?;
            try_record(
                &db,
                &GenerationLogEntry {
                    layer: GenerationLayer::L3,
                    status: GenerationStatus::Succeeded,
                    anchor_id: None,
                    session_key: "user-1".into(),
                    ts_ms: 42,
                    error: None,
                },
            )
            .map_err(mem_err)?;
            Ok(())
        })
        .await
        .expect("seed + genlog");

        let db = st.manager.active_embedded().await.expect("handle");
        let skills = offload(move || run_skills_list(&db)).await.expect("skills");
        assert_eq!(skills.len(), 1, "{skills:?}");
        assert_eq!(skills[0].name, "rotate-keys");
        assert_ne!(skills[0].content_hash, 0, "content-hash present");

        let db = st.manager.active_embedded().await.expect("handle");
        let persona = offload(move || get_persona(&db, "user-1").map_err(mem_err))
            .await
            .expect("persona")
            .expect("seeded persona");
        assert!(persona.content.contains("concise answers"));

        let db = st.manager.active_embedded().await.expect("handle");
        let log = offload(move || query_session(&db, "user-1", None).map_err(mem_err))
            .await
            .expect("genlog");
        assert_eq!(log.len(), 1);
        assert!(matches!(log[0].layer, GenerationLayer::L3));
    }
}
