// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-47 D19 — semantic recall end-to-end (D38 dual-pool ranking + keyword
//! fallback). All storage runs against an in-memory VantaDB and the
//! "embedding provider" is a deterministic fake: a small synonym table maps
//! paraphrase families to the same basis vector; anything else hashes to a
//! normalized pseudo-vector (FNV/LCG), so distinct texts are near-orthogonal.
//! No network, no feature flags (P4 — the hook itself IS the opt-in).

use std::sync::Arc;

use vanta_memory::core::abstractions::{ExtractedMemory, MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallMode, RecallScope,
};
use vanta_memory::core::profile::ProfileIsolation;
use vanta_memory::core::record::l1_dedup::{prepare_pending, recall_candidate_matches};
use vanta_memory::core::record::l1_writer::EmbedFn;
use vanta_memory::core::scene::upsert_scene;
use vanta_memory::gateway::{scene_query, SceneQueryRequest};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};

// ── deterministic fake embeddings ─────────────────────────────────────────

const DIM: usize = 64;

/// Basis vector for a semantic group (64 dims, mutually orthogonal).
fn unit(group: usize) -> Vec<f32> {
    let mut v = vec![0.0; DIM];
    v[group % 8] = 1.0;
    v
}

/// Normalized FNV-seeded LCG vector: same text → identical vector; distinct
/// texts → near-orthogonal (cosine well below MIN_COSINE_SIMILARITY).
/// Components are zero-centered — non-negative pseudo-vectors would all fall
/// in the same octant and score spuriously high cosine.
fn hash_unit(text: &str) -> Vec<f32> {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut state = h | 1;
    let mut v = Vec::with_capacity(DIM);
    for _ in 0..DIM {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        v.push(((((state >> 32) & 0xFFFF_FFFF) as f32) / (u32::MAX as f32)) - 0.5);
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut v {
        *x /= norm;
    }
    v
}

/// Fake provider: two curated paraphrase families + hashed fallback.
/// Family A: "staying up late coding" ≈ "nighttime programming sessions"
/// Family B: "dark themes reduce eye strain" ≈ "dimmed palettes are easier on the eyes"
fn fake_hook() -> EmbedFn {
    Arc::new(|text: &str| {
        let family = if text.contains("staying up late") || text.contains("nighttime programming") {
            Some(0)
        } else if text.contains("dark themes") || text.contains("dimmed palettes") {
            Some(1)
        } else {
            None
        };
        Some(match family {
            Some(group) => unit(group),
            None => hash_unit(text),
        })
    })
}

/// Test-local keyword-overlap counter mirroring `significant_terms` semantics
/// (words ≥ 3 chars, lowercased) — proves the zero-shared-term preconditions.
fn shared_terms(a: &str, b: &str) -> usize {
    let terms = |s: &str| -> std::collections::HashSet<String> {
        s.split(|c: char| !c.is_alphanumeric() && c != '_')
            .map(str::to_lowercase)
            .filter(|t| t.chars().count() >= 3)
            .collect()
    };
    terms(a).intersection(&terms(b)).count()
}

// ── fixtures ──────────────────────────────────────────────────────────────

fn db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
}

fn record(id: &str, content: &str, session_key: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Episodic,
        priority: 50,
        scene_name: "dev".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![],
        created_at: "2026-08-22T10:00:00Z".into(),
        updated_at: "2026-08-22T10:00:00Z".into(),
        version: 1,
        session_key: session_key.into(),
        session_id: String::new(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

/// Persist an L1 record exactly as `read_session_records` expects it, with an
/// explicit node vector (mirrors MEM-46's writer behavior).
fn put_l1(db: &VantaEmbedded, rec: &MemoryRecord, vector: Option<Vec<f32>>) {
    db.put(VantaMemoryInput {
        namespace: vanta_memory::core::record::l1_reader::l1_namespace(&rec.session_key),
        key: rec.id.clone(),
        payload: serde_json::to_string(rec).expect("serialize record"),
        metadata: VantaMemoryMetadata::new(),
        vector,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("put l1 record");
}

fn recall(
    db: &VantaEmbedded,
    query: &str,
    session: &str,
    scope: RecallScope,
    embed: Option<&EmbedFn>,
    isolation: Option<ProfileIsolation>,
) -> Option<vanta_memory::core::hooks::RecallResult> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text: query,
            session_key: session,
            isolation,
            config: RecallConfig {
                scope,
                ..RecallConfig::default()
            },
        },
        embed,
    )
    .expect("auto recall never errors")
}

fn default_isolation() -> ProfileIsolation {
    ProfileIsolation::default()
}

// ── (a) paraphrase matches by vector, zero shared keywords ────────────────

#[test]
fn paraphrase_with_zero_keyword_overlap_is_recalled_by_vector() {
    let content = "she enjoys staying up late coding marathons";
    let query = "nighttime programming sessions delight her";
    assert_eq!(
        shared_terms(content, query),
        0,
        "precondition: paraphrase pair must share no significant terms"
    );

    let db = db();
    let mut rec = record("m1", content, "sess-1");
    rec.vector = Some(fake_hook()(content).unwrap());
    put_l1(&db, &rec, rec.vector.clone());

    let out = recall(
        &db,
        query,
        "sess-1",
        RecallScope::Session,
        Some(&fake_hook()),
        Some(default_isolation()),
    )
    .expect("content injected");

    assert_eq!(out.effective_mode, RecallMode::Hybrid, "vector path ran");
    assert!(
        out.prepend_context
            .as_deref()
            .unwrap_or_default()
            .contains("coding marathons"),
        "paraphrase query must recall the record by similarity"
    );
}

// ── (b) record WITHOUT vector falls back to keyword overlap (D38) ─────────

#[test]
fn legacy_record_without_vector_still_recalls_via_keyword_overlap() {
    let db = db();
    // Legacy record: no vector, shares keywords with the query.
    put_l1(
        &db,
        &record("m-legacy", "user prefers dark mode", "sess-1"),
        None,
    );
    // Vector record whose stored vector is unrelated to the query family.
    let mut unrelated = record("m-vec", "quarterly budget review notes", "sess-1");
    unrelated.vector = Some(hash_unit("unrelated domain entirely"));
    put_l1(&db, &unrelated, unrelated.vector.clone());

    let hook = fake_hook();
    let query = "remind me about the dark mode preference";

    let with_hook = recall(
        &db,
        query,
        "sess-1",
        RecallScope::Session,
        Some(&hook),
        Some(default_isolation()),
    )
    .expect("content injected");
    assert!(with_hook
        .prepend_context
        .as_deref()
        .unwrap_or_default()
        .contains("user prefers dark mode"));
    assert!(
        !with_hook
            .prepend_context
            .as_deref()
            .unwrap_or_default()
            .contains("budget"),
        "orthogonal vector record must not surface"
    );

    // Parity: without a hook the legacy hit survives identically (fallback
    // preserves pre-MEM-47 behavior).
    let without_hook = recall(
        &db,
        query,
        "sess-1",
        RecallScope::Session,
        None,
        Some(default_isolation()),
    )
    .expect("content injected");
    assert_eq!(without_hook.effective_mode, RecallMode::Keyword);
    assert!(without_hook
        .prepend_context
        .as_deref()
        .unwrap_or_default()
        .contains("user prefers dark mode"));
}

// ── (c) dedup recalls semantic candidates ─────────────────────────────────

#[test]
fn dedup_phase1_finds_semantic_candidates_without_shared_terms() {
    let content = "she enjoys staying up late coding marathons";
    let incoming = "nighttime programming sessions delight her";
    assert_eq!(shared_terms(content, incoming), 0);

    let existing = vec![{
        let mut rec = record("m-existing", content, "sess-1");
        rec.vector = Some(fake_hook()(content).unwrap());
        rec
    }];
    let memories = vec![ExtractedMemory {
        content: incoming.into(),
        memory_type: MemoryType::Episodic,
        priority: 50,
        source_message_ids: vec![],
        scene_name: "dev".into(),
        metadata: serde_json::Value::Null,
    }];
    let pending = prepare_pending(&memories, 1_000);

    let hook = fake_hook();
    let semantic = recall_candidate_matches(&pending, &existing, 5, Some(&hook));
    assert_eq!(
        semantic[0].candidates.len(),
        1,
        "vector similarity must surface the candidate"
    );
    assert_eq!(semantic[0].candidates[0].id, "m-existing");

    // Without the hook the same pool yields nothing (pure keyword gate).
    let keyword_only = recall_candidate_matches(&pending, &existing, 5, None);
    assert!(
        keyword_only[0].candidates.is_empty(),
        "zero keyword overlap must stay empty on the legacy path"
    );
}

// ── (d) knowledge query uses vector ───────────────────────────────────────

#[test]
fn scene_query_ranks_semantically_when_hook_present() {
    let db = db();
    let summary = "late-night work habits";
    let content = "she enjoys staying up late coding marathons";
    upsert_scene(&db, "sess-1", "habits", summary, content).expect("seed scene");
    upsert_scene(&db, "sess-1", "other", "misc", "grocery list reminders").expect("seed other");

    let request = SceneQueryRequest {
        session_key: "sess-1".into(),
        // Zero keyword overlap with the target block's summary+content.
        keyword: "nighttime programming sessions".into(),
        top_k: None,
    };
    assert_eq!(
        shared_terms(&format!("{summary} {content}"), &request.keyword),
        0,
        "precondition: no shared terms between query and scene"
    );

    // Keyword-only path (None hook): nothing matches.
    let legacy = scene_query(&db, &request, None).expect("query");
    assert!(legacy.hits.is_empty(), "legacy path must not match");

    // Semantic path: the hook ranks the block first despite zero overlap.
    let resp = scene_query(&db, &request, Some(&fake_hook())).expect("query");
    assert_eq!(resp.hits.len(), 1);
    assert_eq!(resp.hits[0].scene_name, "habits");
}

// ── (e) RecallScope is respected in vector mode ───────────────────────────

#[test]
fn recall_scope_gates_semantic_hits_across_sessions() {
    let query = "nighttime programming sessions delight her";
    let content = "she enjoys staying up late coding marathons";
    let hook = fake_hook();

    let db = db();
    let mut cross = record("m-cross", content, "sess-2");
    cross.agent_id = Some("agent-1".into());
    cross.vector = Some(hook(content).unwrap());
    put_l1(&db, &cross, cross.vector.clone());

    let iso = ProfileIsolation {
        team_id: "team-a".into(),
        agent_id: "agent-1".into(),
    };

    // Session scope: cross-session record invisible even though it matches.
    let scoped = recall(
        &db,
        query,
        "sess-1",
        RecallScope::Session,
        Some(&hook),
        Some(default_isolation()),
    );
    assert!(
        scoped.is_none_or(|r| !r
            .prepend_context
            .as_deref()
            .unwrap_or_default()
            .contains("coding marathons")),
        "Session scope leaked a cross-session semantic hit"
    );

    // Agent scope: same semantic hit now visible.
    let widened = recall(
        &db,
        query,
        "sess-1",
        RecallScope::Agent,
        Some(&hook),
        Some(iso),
    )
    .expect("agent scope injects");
    assert!(widened
        .prepend_context
        .as_deref()
        .unwrap_or_default()
        .contains("coding marathons"));
}
