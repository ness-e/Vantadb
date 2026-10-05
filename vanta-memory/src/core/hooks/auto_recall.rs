//! Auto-recall hook (MEM-18): injects relevant L1 memories + persona + scene
//! navigation into the agent context before it starts processing.
//!
//! Split (TDAM `auto-recall.ts` prompt-cache parity):
//! - [`RecallResult::prepend_context`] — L1 relevant memories, dynamic
//!   per-turn, prepended to the user prompt.
//! - [`RecallResult::append_system_context`] — persona + scene navigation +
//!   tools guide, stable across turns, appended to the system prompt so
//!   cache-friendly providers can reuse the region.
//!
//! Three recall modes ([`RecallMode`]): `keyword`, `embedding`, `hybrid`
//! (TDAM strategy names). Since MEM-47 the embedding/hybrid modes actually
//! run when the caller passes an [`crate::core::record::l1_writer::EmbedFn`]
//! hook AND the pool carries vectors (D38 dual-pool ranking): records with a
//! usable vector rank by cosine similarity, records without one keep the
//! keyword-overlap gate — a legacy record is never dropped. With the
//! `embed-local` feature compiled and a working [`LocalOnnxProvider`]
//! available, [`crate::core::record::L1DedupConfig::default`] now wires
//! `local_embedding_hook()` automatically (MEM-63 auto-on), so callers no
//! longer need to call `.with_local_provider()` explicitly. Without the
//! feature the call falls back to keyword overlap exactly as before MEM-47.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use vantadb::sdk::Embedded;

use crate::core::abstractions::MemoryRecord;
use crate::core::conversation::sanitize_key;
use crate::core::persona::persona_generator::{get_persona, persona_namespace, PersonaError};
use crate::core::profile::profile_sync::{
    build_profile_isolation_scope, profile_namespace, read_scoped_persona, ProfileIsolation,
};
use crate::core::record::l1_reader::{
    cosine_similarity, l1_namespace, overlap_score, read_namespace_records, read_session_records,
    rrf_merge, significant_terms, MIN_COSINE_SIMILARITY,
};
use crate::core::record::l1_writer::EmbedFn;
use crate::core::record::L1Error;
use crate::core::scene::scene_index::{list_scenes, scene_namespace, SceneError};
use crate::core::scene::scene_navigation::{generate_scene_navigation, strip_scene_navigation};

/// A single recalled L1 memory with its keyword-overlap score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecalledMemory {
    pub content: String,
    pub score: usize,
    /// Serialized [`MemoryType`] tag (`persona`, `episodic`, ...).
    #[serde(rename = "type")]
    pub memory_type: String,
    /// VER-04: source namespace the memory was read from (`l1/<session>`),
    /// for the surface injection audit. Empty on legacy payloads.
    #[serde(default)]
    pub source_namespace: String,
    /// VER-04: source record key inside [`Self::source_namespace`], for the
    /// surface injection audit. Empty on legacy payloads.
    #[serde(default)]
    pub source_key: String,
}

/// VER-04: governance summary of one recall pass — the namespaces that
/// actually fed the injection and the ones skipped by the ACL policy.
/// Denials are never silent: surfaces record them in the injection audit.
///
/// Both lists are bounded to [`Self::MAX_ENTRIES`]; when more namespaces were
/// involved the last slot degrades to [`Self::OVERFLOW_MARKER`] (the audit
/// stays bounded without pretending the list is complete).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecallGovernance {
    /// Namespaces that contributed content (dedup; bounded).
    pub sources: Vec<String>,
    /// Namespaces skipped by the injection ACL (dedup; bounded).
    pub denied: Vec<String>,
}

impl RecallGovernance {
    /// Max entries kept per list (keeps audit events bounded).
    pub const MAX_ENTRIES: usize = 16;
    /// Marker replacing the last slot when more entries existed (bounded,
    /// never silent).
    pub const OVERFLOW_MARKER: &'static str = "…overflow";

    fn note(list: &mut Vec<String>, namespace: &str) {
        if list.iter().any(|ns| ns == namespace) {
            return;
        }
        if list.len() < Self::MAX_ENTRIES {
            list.push(namespace.to_string());
        } else if list.last().map(String::as_str) != Some(Self::OVERFLOW_MARKER) {
            // Saturated: keep the bound and mark the overflow (review F3).
            if let Some(last) = list.last_mut() {
                *last = Self::OVERFLOW_MARKER.to_string();
            }
        }
    }

    /// Record a namespace that contributed content to this injection.
    pub fn source(&mut self, namespace: &str) {
        Self::note(&mut self.sources, namespace);
    }

    /// Record a namespace denied by the ACL (skipped, never injected).
    pub fn deny(&mut self, namespace: &str) {
        Self::note(&mut self.denied, namespace);
    }
}

/// VER-04: opt-in injection ACL — an allowlist of namespace prefixes for the
/// surfaces that feed memory into model context (proxy `<vanta-memory>`
/// block, MCP recall/context tools, `mem: search`). An empty list allows
/// everything (today's behavior); anything not matched by a prefix is denied
/// (`deny fuera de scope`).
///
/// Prefix semantics are boundary-aware: `l1/sess-1` allows `l1/sess-1` and
/// `l1/sess-1/...` but never `l1/sess-12`; a trailing `/` is tolerated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InjectionPolicy {
    allow_prefixes: Vec<String>,
}

impl InjectionPolicy {
    /// Allow every namespace (default; keeps existing behavior).
    pub fn allow_all() -> Self {
        Self::default()
    }

    /// Build from an operator-provided prefix list (blank/`/`-only entries
    /// are ignored; trailing `/` is stripped).
    pub fn from_prefixes(prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let allow_prefixes = prefixes
            .into_iter()
            .map(|p| p.into())
            .map(|p| p.trim().trim_end_matches('/').to_string())
            .filter(|p| !p.is_empty())
            .collect();
        Self { allow_prefixes }
    }

    /// True when no restriction is configured (allow-all).
    pub fn is_empty(&self) -> bool {
        self.allow_prefixes.is_empty()
    }

    /// Whether `namespace` is inside the allowed scope.
    pub fn allows(&self, namespace: &str) -> bool {
        if self.allow_prefixes.is_empty() {
            return true;
        }
        self.allow_prefixes.iter().any(|prefix| {
            namespace == prefix
                || namespace
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
    }
}

/// Result of an auto-recall pass. `Ok(None)` from [`perform_auto_recall`]
/// means "nothing to inject" — callers must not build empty blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallResult {
    /// L1 relevant memories — dynamic per-turn, prepend to user prompt.
    pub prepend_context: Option<String>,
    /// Persona + scene navigation + tools guide — stable, append to system
    /// prompt (cacheable).
    pub append_system_context: Option<String>,
    /// Structured view of what was recalled (metrics / observability).
    pub recalled_memories: Vec<RecalledMemory>,
    /// Persona body loaded during recall (navigation stripped), if any.
    pub persona: Option<String>,
    /// Effective mode used (after degradation).
    pub effective_mode: RecallMode,
    /// VER-04: ACL/budget governance summary of this pass (sources fed,
    /// namespaces denied). Always populated; empty when nothing was read.
    pub governance: RecallGovernance,
}

/// Recall search mode. Names match TDAM `cfg.recall.strategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallMode {
    /// Keyword overlap over persisted L1 records (LLM-free).
    Keyword,
    /// Embedding cosine similarity. Runs whenever the caller supplied an
    /// embedding hook AND the pool carries vectors (D38 dual-pool ranking).
    /// Without a hook, the call falls back to [`RecallMode::Keyword`] and a
    /// legacy record is never dropped (it still passes the keyword gate).
    /// With the `embed-local` feature compiled and a working
    /// [`LocalOnnxProvider`] available, [`crate::core::record::L1DedupConfig::default`]
    /// wires `local_embedding_hook()` automatically (MEM-63 auto-on), so
    /// callers no longer need `.with_local_provider()` for the dual-pool path
    /// to engage.
    Embedding,
    /// Keyword + embedding merged with RRF. Same auto-on rules as
    /// [`RecallMode::Embedding`] (D38 dual-pool keyword fallback preserved).
    Hybrid,
}

impl RecallMode {
    /// The mode actually executed after resource-based degradation. The
    /// embedding/hybrid strategies only "take" when the caller supplied an
    /// embedding hook (MEM-47) — otherwise they degrade to [`RecallMode::Keyword`].
    pub fn effective(self, embeddings_available: bool) -> Self {
        match self {
            Self::Keyword => Self::Keyword,
            Self::Embedding | Self::Hybrid if embeddings_available => self,
            Self::Embedding | Self::Hybrid => Self::Keyword,
        }
    }
}

/// Cross-session reach of the L1 recall pool (D22). `Session` replicates the
/// pre-MEM-40 behavior; `Agent`/`Team` widen it to other sessions whose
/// records carry a matching `agent_id` / `team_id`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallScope {
    /// Only the current session's L1 records.
    Session,
    /// Current session + other sessions of the same agent (default: memory
    /// accumulates across sessions of one agent, TDAM de-facto behavior,
    /// without its cross-agent leak).
    #[default]
    Agent,
    /// Current session + every session of the same team.
    Team,
}

/// Configuration for the auto-recall hook (TDAM `cfg.recall` subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallConfig {
    /// Search strategy (default: hybrid, TDAM default).
    pub mode: RecallMode,
    /// Cross-session scope of the L1 recall pool (default: agent, D22).
    pub scope: RecallScope,
    /// Maximum memories injected per turn (TDAM `maxResults`, default 5).
    pub max_results: usize,
    /// Minimum shared-term score for a record to be recalled (analog of the
    /// TDAM BM25 `scoreThreshold`; our scores are overlap counts).
    pub min_overlap: usize,
    /// Per-memory char budget before truncation (TDAM `maxCharsPerMemory`).
    pub max_chars_per_memory: Option<usize>,
    /// Total char budget across all recalled lines (TDAM
    /// `maxTotalRecallChars`).
    pub max_total_recall_chars: Option<usize>,
}

impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            mode: RecallMode::Hybrid,
            scope: RecallScope::Agent,
            max_results: 5,
            min_overlap: 1,
            max_chars_per_memory: None,
            max_total_recall_chars: None,
        }
    }
}

/// Errors surfaced by the auto-recall hook.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RecallError {
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::Error),
    /// Persona layer failure.
    #[error("persona: {0}")]
    Persona(#[from] PersonaError),
    /// Scene index failure.
    #[error("scene: {0}")]
    Scene(#[from] SceneError),
    /// Profile sync failure (scoped persona read).
    #[error("profile: {0}")]
    Profile(#[from] crate::core::profile::ProfileSyncError),
    /// A persisted L1 record failed to deserialize (skipped upstream, but
    /// surfaced if the read itself cannot proceed).
    #[error("malformed l1 record payload: {0}")]
    MalformedRecord(String),
}

impl From<L1Error> for RecallError {
    fn from(err: L1Error) -> Self {
        match err {
            L1Error::Vanta(v) => Self::Vanta(v),
            L1Error::Serde(s) => Self::MalformedRecord(s.to_string()),
        }
    }
}

/// Static guide appended to the stable context so the agent knows how to
/// retrieve deeper information when the injected snippets are not enough
/// (English rewrite of TDAM `MEMORY_TOOLS_GUIDE`; tool names land with the
/// MCP tools of MEM-19/20).
pub const MEMORY_TOOLS_GUIDE: &str = "<memory-tools-guide>\n\
When the memory snippets above are not enough to answer, actively search for more:\n\n\
- memory_search: structured memories (L1) — preferences, events, rules.\n\
- conversation_search: raw conversation (L0) — exact wording, timeline detail.\n\
- read_file (paths in the scene navigation): full picture of a located scene.\n\n\
Limit: at most 3 searches per turn combined. If nothing is found after 3 \
searches, the information is not in memory — answer with what you have.\n\
</memory-tools-guide>";

/// Run one auto-recall pass over an open embedded database.
///
/// Empty `user_text` skips the memory search but still injects persona and
/// scene navigation (TDAM parity). When neither memories nor persona nor
/// scenes yield content, returns `Ok(None)` — never an empty block.
///
/// Allow-all convenience wrapper over [`perform_auto_recall_governed`]
/// (existing callers keep their signature and behavior untouched).
pub fn perform_auto_recall(
    db: &Embedded,
    params: AutoRecallParams<'_>,
    embed: Option<&EmbedFn>,
) -> Result<Option<RecallResult>, RecallError> {
    perform_auto_recall_governed(db, params, embed, &InjectionPolicy::allow_all())
}

/// VER-04: auto-recall with an explicit injection ACL. Sources whose
/// namespace is outside `policy` are skipped (never injected) and reported in
/// [`RecallResult::governance`] — denials are never silent.
pub fn perform_auto_recall_governed(
    db: &Embedded,
    params: AutoRecallParams<'_>,
    embed: Option<&EmbedFn>,
    policy: &InjectionPolicy,
) -> Result<Option<RecallResult>, RecallError> {
    let config = params.config;
    let isolation = params.isolation.unwrap_or_default();
    let mut governance = RecallGovernance::default();

    // ── L1 search (skipped on empty user text) ──
    let mut recalled = Vec::new();
    let mut semantic_ran = false;
    if !params.user_text.trim().is_empty() {
        // Own-session records are always visible (legacy records carry no
        // agent/team metadata and must not vanish when the scope widens).
        let mut records = Vec::new();
        let current_ns = l1_namespace(params.session_key);
        if policy.allows(&current_ns) {
            records = read_session_records(db, params.session_key)?;
            if !records.is_empty() {
                governance.source(&current_ns);
            }
        } else {
            governance.deny(&current_ns);
        }
        if config.scope != RecallScope::Session {
            records.extend(read_scoped_records(
                db,
                config.scope,
                &isolation,
                params.session_key,
                policy,
                &mut governance,
            )?);
        }
        let (hits, used_semantic) = search_records(&records, params.user_text, &config, embed);
        semantic_ran = used_semantic;
        recalled = hits;
    }

    // ── L3 persona (scoped by team+agent via profile_sync) ──
    let scoped_ns = profile_namespace(&build_profile_isolation_scope(&isolation));
    let scoped = if policy.allows(&scoped_ns) {
        read_scoped_persona(db, &isolation)?
    } else {
        governance.deny(&scoped_ns);
        None
    };
    let session_persona_ns = persona_namespace(params.session_key);
    let persona_body = match scoped {
        Some(content) => {
            let body = strip_scene_navigation(&content).trim().to_string();
            let body = (!body.is_empty()).then_some(body);
            if body.is_some() {
                governance.source(&scoped_ns);
            }
            body
        }
        // Fallback: session-level persona written directly by MEM-15 without a
        // scope sync yet (ACL-checked like any other source).
        None => {
            if policy.allows(&session_persona_ns) {
                let body = get_persona(db, params.session_key)?
                    .map(|p| strip_scene_navigation(&p.content).trim().to_string())
                    .filter(|b| !b.is_empty());
                if body.is_some() {
                    governance.source(&session_persona_ns);
                }
                body
            } else {
                governance.deny(&session_persona_ns);
                None
            }
        }
    };

    // ── L2 scene navigation ──
    let scene_ns = scene_namespace(params.session_key);
    let entries = if policy.allows(&scene_ns) {
        let entries = list_scenes(db, params.session_key)?;
        if !entries.is_empty() {
            governance.source(&scene_ns);
        }
        entries
    } else {
        governance.deny(&scene_ns);
        Vec::new()
    };
    let navigation = (!entries.is_empty()).then(|| generate_scene_navigation(&entries));

    if recalled.is_empty() && persona_body.is_none() && navigation.is_none() {
        // Review F1: a deny-all pass still returns `Some` — the ACL denials
        // are the payload the surfaces audit. The context fields stay `None`
        // (wire-identical: callers replace `None` with their "no memories"
        // text / null envelope). `Ok(None)` remains reserved for a pass with
        // nothing to inject AND nothing denied (default allow-all path).
        if governance.denied.is_empty() {
            return Ok(None);
        }
    }

    // Dynamic part → prepend (user prompt); stable parts → append (system).
    let prepend_context = (!recalled.is_empty()).then(|| {
        format!(
            "<relevant-memories>\n{}\n</relevant-memories>",
            recalled
                .iter()
                .map(|m| m.line.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    });

    let mut stable_parts: Vec<String> = Vec::new();
    if let Some(persona) = &persona_body {
        stable_parts.push(format!("<user-persona>\n{persona}\n</user-persona>"));
    }
    if let Some(nav) = &navigation {
        stable_parts.push(format!("<scene-navigation>\n{nav}\n</scene-navigation>"));
    }
    if !stable_parts.is_empty() || prepend_context.is_some() {
        stable_parts.push(MEMORY_TOOLS_GUIDE.to_string());
    }
    let append_system_context = (!stable_parts.is_empty()).then(|| stable_parts.join("\n\n"));

    Ok(Some(RecallResult {
        prepend_context,
        append_system_context,
        recalled_memories: recalled.into_iter().map(|m| m.memory).collect(),
        persona: persona_body,
        effective_mode: config.mode.effective(semantic_ran),
        governance,
    }))
}

/// MEMG-02: report the outcome of a recalled memory back to the store —
/// closes the recall → outcome → confidence loop.
///
/// Maps a [`RecalledMemory`] hit to the core
/// [`vantadb::Embedded::reinforce`] op using the identity the recall already
/// carries (`source_namespace` + `source_key`, VER-04). The host declares the
/// outcome explicitly ([`vantadb::ReinforceOutcome`]) — the engine never
/// infers it (no silent feedback). Policy (bump/decay/window) lives in the
/// core op; see `docs/api/scores.md` §Reinforcement.
///
/// `source_key` is sanitized with the same rule the L1 writer used for the
/// stored key (`sanitize_key`, `put_record`), so a hit round-trips to its
/// record. Errors with `InvalidInput` when the hit carries no source identity
/// (payload recalled before VER-04) instead of guessing namespace/key.
pub fn reinforce_recalled(
    db: &Embedded,
    recalled: &RecalledMemory,
    outcome: vantadb::ReinforceOutcome,
) -> Result<vantadb::MemoryRecord, L1Error> {
    if recalled.source_namespace.is_empty() || recalled.source_key.is_empty() {
        return Err(L1Error::Vanta(vantadb::Error::InvalidInput(
            "recalled memory carries no source identity (legacy payload) — cannot reinforce"
                .to_string(),
        )));
    }
    let key = sanitize_key(&recalled.source_key);
    Ok(db.reinforce(&recalled.source_namespace, &key, outcome)?)
}

/// Cross-session L1 records visible under the given scope (D22): every
/// `l1/*` namespace except the current session's, filtered by the record's
/// own `agent_id` / `team_id`. Records without the matching metadata are
/// invisible cross-session — they stay session-only (legacy fallback).
/// Namespaces outside the injection `policy` are skipped and reported in
/// `governance` (VER-04 ACL enforcement, never silent).
///
/// # ponytail: full scan of all `l1/*` namespaces per recall
/// O(#sessions + #records) via `list_namespaces`; fine at hundreds of
/// sessions. Upgrade path (plan stop-condition): sessions-per-agent index.
fn read_scoped_records(
    db: &Embedded,
    scope: RecallScope,
    isolation: &ProfileIsolation,
    current_session: &str,
    policy: &InjectionPolicy,
    governance: &mut RecallGovernance,
) -> Result<Vec<MemoryRecord>, RecallError> {
    let current_ns = l1_namespace(current_session);
    let mut out = Vec::new();
    for ns in db.list_namespaces()? {
        if !ns.starts_with("l1/") || ns == current_ns {
            continue;
        }
        if !policy.allows(&ns) {
            governance.deny(&ns);
            continue;
        }
        let mut contributed = false;
        for record in read_namespace_records(db, &ns)? {
            let visible = match scope {
                RecallScope::Session => true,
                RecallScope::Agent => {
                    record.agent_id.as_deref() == Some(isolation.agent_id.as_str())
                }
                RecallScope::Team => record.team_id.as_deref() == Some(isolation.team_id.as_str()),
            };
            if visible {
                out.push(record);
                contributed = true;
            }
        }
        if contributed {
            governance.source(&ns);
        }
    }
    Ok(out)
}

/// Parameters of one auto-recall pass.
#[derive(Debug, Clone)]
pub struct AutoRecallParams<'a> {
    /// Raw user text of the current turn.
    pub user_text: &'a str,
    /// Session key whose L1 records / scenes are searched.
    pub session_key: &'a str,
    /// L2/L3 profile scope; defaults to team=default, agent=default.
    pub isolation: Option<ProfileIsolation>,
    /// Recall configuration.
    pub config: RecallConfig,
}

/// One scored + formatted recall hit (internal).
struct RecallHit {
    line: String,
    memory: RecalledMemory,
}

/// D38 dual-pool search over L1 records (MEM-47): records WITH a usable
/// vector rank by cosine similarity against the embedded query; records
/// WITHOUT one keep the keyword-overlap gate — a legacy record is never
/// dropped just because it has no vector. Both pools fuse via reciprocal-rank
/// fusion so term counts and similarities never compete directly. Without an
/// embed hook (or with mode `Keyword`) this is byte-identical to the
/// pre-MEM-47 keyword path.
///
/// Returns `(hits, semantic_ran)` — `semantic_ran` reports whether vector
/// ranking actually contributed, keeping [`RecallMode::effective`] honest.
/// Note on scores: [`RecalledMemory::score`] stays the keyword-overlap count;
/// a record matched purely via similarity reports `0` there (its cosine lives
/// in the internal ranking only).
fn search_records(
    records: &[MemoryRecord],
    query: &str,
    config: &RecallConfig,
    embed: Option<&EmbedFn>,
) -> (Vec<RecallHit>, bool) {
    let terms = significant_terms(query);
    let query_vector = match (config.mode != RecallMode::Keyword, embed) {
        (true, Some(hook)) => hook(query),
        _ => None,
    }
    .filter(|v| !v.is_empty() && v.iter().any(|&x| x != 0.0));
    if terms.is_empty() && query_vector.is_none() {
        return (Vec::new(), false);
    }

    // Partition into pools. A record whose stored vector mismatches the query
    // dimensions (or is zero-norm) falls back to keyword scoring like any
    // vector-free record.
    let min_overlap = config.min_overlap.max(1);
    let mut keyword_pool: Vec<(usize, &MemoryRecord)> = Vec::new();
    let mut vector_pool: Vec<(f32, &MemoryRecord)> = Vec::new();
    for record in records {
        let semantic_score = match (&record.vector, &query_vector) {
            (Some(record_vector), Some(query_vec)) => cosine_similarity(record_vector, query_vec),
            _ => None,
        };
        match semantic_score {
            Some(sim) if sim >= MIN_COSINE_SIMILARITY => vector_pool.push((sim, record)),
            _ => {
                let score = overlap_score(&record.content, query);
                if score >= min_overlap {
                    keyword_pool.push((score, record));
                }
            }
        }
    }
    keyword_pool.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });
    vector_pool.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });

    let semantic_ran = !vector_pool.is_empty();
    let ordered: Vec<&MemoryRecord> = if !semantic_ran {
        keyword_pool.iter().map(|(_, r)| *r).collect()
    } else if keyword_pool.is_empty() {
        vector_pool.iter().map(|(_, r)| *r).collect()
    } else {
        let keyword_ids: Vec<String> = keyword_pool.iter().map(|(_, r)| r.id.clone()).collect();
        let vector_ids: Vec<String> = vector_pool.iter().map(|(_, r)| r.id.clone()).collect();
        rrf_merge(&keyword_ids, &vector_ids, usize::MAX)
            .into_iter()
            .filter_map(|id| records.iter().find(|r| r.id == id))
            .collect()
    };

    let hits: Vec<RecallHit> = ordered
        .into_iter()
        .take(config.max_results)
        .map(|record| {
            let line = format_memory_line(record);
            let memory_type = serde_json::to_string(&record.memory_type)
                .unwrap_or_else(|_| "\"unknown\"".to_string());
            let memory_type = memory_type.trim_matches('"').to_string();
            let content = line
                .split_once("] ")
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_else(|| line.clone());
            RecallHit {
                line,
                memory: RecalledMemory {
                    content,
                    score: keyword_pool
                        .iter()
                        .find(|(_, r)| r.id == record.id)
                        .map_or(0, |(score, _)| *score),
                    memory_type,
                    // VER-04: source identity for the surface injection audit.
                    source_namespace: l1_namespace(&record.session_key),
                    source_key: record.id.clone(),
                },
            }
        })
        .collect();

    // Budget operates on lines; re-pair the surviving lines with their
    // structured payloads by position.
    let budgeted_lines = apply_recall_budget(hits.iter().map(|h| h.line.clone()).collect(), config);
    let budgeted = budgeted_lines
        .into_iter()
        .zip(hits)
        .map(|(line, mut hit)| {
            hit.memory.content = line
                .split_once("] ")
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_else(|| line.clone());
            hit.line = line;
            hit
        })
        .collect();
    (budgeted, semantic_ran)
}

/// Format one record as a rich natural-language line (TDAM
/// `formatMemoryLine`): `- [type|scene] content (activity time: ...)`.
fn format_memory_line(record: &MemoryRecord) -> String {
    let memory_type =
        serde_json::to_string(&record.memory_type).unwrap_or_else(|_| "\"unknown\"".to_string());
    let memory_type = memory_type.trim_matches('"');
    let tag = if record.scene_name.is_empty() {
        memory_type.to_string()
    } else {
        format!("{memory_type}|{}", record.scene_name)
    };
    let mut line = format!("- [{tag}] {}", record.content);

    let start = metadata_time(record, "activity_start_time");
    let end = metadata_time(record, "activity_end_time");
    let point = record.timestamps.first();
    if let (Some(s), Some(e)) = (&start, &end) {
        line.push_str(&format!(" (activity time: {s} ~ {e})"));
    } else if let Some(s) = &start {
        line.push_str(&format!(" (activity time: from {s})"));
    } else if let Some(e) = &end {
        line.push_str(&format!(" (activity time: until {e})"));
    } else if let Some(p) = point {
        line.push_str(&format!(" (activity time: {p})"));
    }
    line
}

/// Read an ISO-ish timestamp field out of the record metadata, if present.
fn metadata_time(record: &MemoryRecord, key: &str) -> Option<String> {
    record
        .metadata
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

const MIN_TRUNCATED_LINE_CHARS: usize = 40;
const TRUNCATION_SUFFIX: &str = "...";

/// Apply the per-memory and total char budgets (TDAM `applyRecallBudget`):
/// truncate lines to `max_chars_per_memory`, then drop/truncate to fit
/// `max_total_recall_chars`. Char-boundary-safe (Rust `char`s are code
/// points, matching TDAM's surrogate-pair-safe slicing).
fn apply_recall_budget(lines: Vec<String>, config: &RecallConfig) -> Vec<String> {
    let per_memory = normalize_budget(config.max_chars_per_memory);
    let total = normalize_budget(config.max_total_recall_chars);
    if per_memory.is_none() && total.is_none() {
        return lines;
    }

    let mut budgeted: Vec<String> = Vec::with_capacity(lines.len());
    let mut used = 0usize;
    for line in lines {
        let bounded = per_memory.map_or_else(|| line.clone(), |max| truncate_line(&line, max));
        let Some(total_max) = total else {
            budgeted.push(bounded);
            continue;
        };
        let separator = usize::from(!budgeted.is_empty());
        let remaining = total_max.saturating_sub(used + separator);
        if remaining == 0 {
            break;
        }
        if bounded.chars().count() > remaining {
            if remaining >= MIN_TRUNCATED_LINE_CHARS {
                let fitted = truncate_line(&bounded, remaining);
                budgeted.push(fitted);
            }
            break;
        }
        used += separator + bounded.chars().count();
        budgeted.push(bounded);
    }
    budgeted
}

/// `None` for missing/non-positive budgets (TDAM `normalizeBudgetLimit`).
fn normalize_budget(value: Option<usize>) -> Option<usize> {
    value.filter(|v| *v > 0)
}

/// Truncate to `max_chars` code points, appending the suffix when it fits.
/// Consolidated in `utils::text_utils` (MEM-19); kept as a thin alias so the
/// two internal call sites stay readable.
fn truncate_line(line: &str, max_chars: usize) -> String {
    crate::utils::text_utils::truncate_with_suffix(line, max_chars, TRUNCATION_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abstractions::{MemoryRecord, MemoryType};

    fn record(id: &str, content: &str, updated: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "ui-setup".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec![],
            created_at: updated.into(),
            updated_at: updated.into(),
            version: 1,
            session_key: "sk".into(),
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

    #[test]
    fn modes_degrade_to_keyword_without_embedding_resources() {
        assert_eq!(RecallMode::Keyword.effective(false), RecallMode::Keyword);
        assert_eq!(RecallMode::Embedding.effective(false), RecallMode::Keyword);
        assert_eq!(RecallMode::Hybrid.effective(false), RecallMode::Keyword);
        // With an embedding hook the declared modes actually run (MEM-47).
        assert_eq!(RecallMode::Embedding.effective(true), RecallMode::Embedding);
        assert_eq!(RecallMode::Hybrid.effective(true), RecallMode::Hybrid);
    }

    #[test]
    fn injection_policy_empty_allows_everything() {
        let policy = InjectionPolicy::allow_all();
        assert!(policy.is_empty());
        assert!(policy.allows("l1/anything"));
        assert!(policy.allows("persona/x"));
        assert!(policy.allows(""));
    }

    #[test]
    fn injection_policy_prefix_boundaries_are_exact() {
        let policy = InjectionPolicy::from_prefixes(["l1/sess-1", "persona/", " scene/s1 "]);
        assert!(policy.allows("l1/sess-1"));
        assert!(policy.allows("l1/sess-1/sub"));
        assert!(
            !policy.allows("l1/sess-12"),
            "prefix must respect the / boundary"
        );
        assert!(policy.allows("persona/anything"), "trailing / is tolerated");
        assert!(policy.allows("scene/s1"), "surrounding blanks are trimmed");
        assert!(!policy.allows("scene/s10"));
        assert!(!policy.allows("l1/sess-2"));
    }

    #[test]
    fn injection_policy_ignores_blank_entries() {
        let policy = InjectionPolicy::from_prefixes(["", "  ", "/"]);
        assert!(policy.is_empty(), "blank prefixes must not restrict");
        assert!(policy.allows("l1/x"));
    }

    #[test]
    fn recall_governance_bounds_lists_and_marks_overflow() {
        let mut governance = RecallGovernance::default();
        for i in 0..(RecallGovernance::MAX_ENTRIES + 5) {
            governance.deny(&format!("l1/s{i}"));
        }
        assert_eq!(governance.denied.len(), RecallGovernance::MAX_ENTRIES);
        assert_eq!(
            governance.denied.last().map(String::as_str),
            Some(RecallGovernance::OVERFLOW_MARKER),
            "saturation must be visible, not silent"
        );
        // Duplicates neither grow the list nor disturb the marker.
        governance.deny("l1/s0");
        assert_eq!(governance.denied.len(), RecallGovernance::MAX_ENTRIES);
    }

    #[test]
    fn formats_type_scene_and_activity_range() {
        let mut r = record("m1", "planned the trip", "2026-08-20T10:00:00Z");
        r.metadata = serde_json::json!({
            "activity_start_time": "2026-05-01",
            "activity_end_time": "2026-05-10"
        });
        let line = format_memory_line(&r);
        assert!(line.starts_with("- [persona|ui-setup] planned the trip"));
        assert!(line.contains("(activity time: 2026-05-01 ~ 2026-05-10)"));
    }

    #[test]
    fn falls_back_to_point_timestamp_when_no_range() {
        let mut r = record("m1", "worked late", "2026-08-20T10:00:00Z");
        r.timestamps = vec!["2026-03-01T14:30:00Z".into()];
        let line = format_memory_line(&r);
        assert!(line.contains("(activity time: 2026-03-01T14:30:00Z)"));
    }

    #[test]
    fn truncation_is_char_boundary_safe_and_appends_suffix() {
        let line = "áéíóú".repeat(20);
        let cut = truncate_line(&line, 12);
        assert_eq!(cut.chars().count(), 12);
        assert!(cut.ends_with(TRUNCATION_SUFFIX));
        assert_eq!(truncate_line("short", 12), "short");
    }

    #[test]
    fn total_budget_truncates_then_drops() {
        let config = RecallConfig {
            max_total_recall_chars: Some(60),
            ..RecallConfig::default()
        };
        let lines: Vec<String> = (0..5)
            .map(|i| format!("- [t] item{i} {}", "x".repeat(25)))
            .collect();
        let out = apply_recall_budget(lines, &config);
        let used: usize = out.iter().map(|l| l.chars().count()).sum::<usize>() + out.len() - 1;
        assert!(used <= 60);
        assert!(out.len() < 5);
    }

    #[test]
    fn no_budget_is_passthrough() {
        let lines = vec!["a".into(), "b".into()];
        assert_eq!(
            apply_recall_budget(lines.clone(), &RecallConfig::default()),
            lines
        );
    }
}
