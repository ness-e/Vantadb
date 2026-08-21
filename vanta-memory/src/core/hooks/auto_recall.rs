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
//! (TDAM strategy names). The crate has no LLM-free embeddings and no vector
//! API exposed yet, so `embedding`/`hybrid` degrade to keyword overlap — the
//! same documented ceiling as MEM-11's `recall_candidates` (Principio 4:
//! degrade, never block).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use vantadb::sdk::VantaEmbedded;

use crate::core::abstractions::MemoryRecord;
use crate::core::persona::persona_generator::{get_persona, PersonaError};
use crate::core::profile::profile_sync::{read_scoped_persona, ProfileIsolation};
use crate::core::record::l1_reader::{overlap_score, read_session_records, significant_terms};
use crate::core::record::L1Error;
use crate::core::scene::scene_index::{list_scenes, SceneError};
use crate::core::scene::scene_navigation::{generate_scene_navigation, strip_scene_navigation};

/// A single recalled L1 memory with its keyword-overlap score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecalledMemory {
    pub content: String,
    pub score: usize,
    /// Serialized [`MemoryType`] tag (`persona`, `episodic`, ...).
    #[serde(rename = "type")]
    pub memory_type: String,
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
}

/// Recall search mode. Names match TDAM `cfg.recall.strategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallMode {
    /// Keyword overlap over persisted L1 records (LLM-free).
    Keyword,
    /// Embedding cosine similarity. Degrades to [`RecallMode::Keyword`] in
    /// this crate until a vector API is wired (documented ceiling).
    Embedding,
    /// Keyword + embedding merged with RRF. Degrades to
    /// [`RecallMode::Keyword`] for the same reason.
    Hybrid,
}

impl RecallMode {
    /// The mode actually executed after resource-based degradation.
    pub fn effective(self) -> Self {
        match self {
            Self::Keyword => Self::Keyword,
            // ponytail: embedding/hybrid collapse to keyword overlap; wire the
            // VantaDB vector index once its search API reaches this crate.
            Self::Embedding | Self::Hybrid => Self::Keyword,
        }
    }
}

/// Configuration for the auto-recall hook (TDAM `cfg.recall` subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallConfig {
    /// Search strategy (default: hybrid, TDAM default).
    pub mode: RecallMode,
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
    Vanta(#[from] vantadb::error::VantaError),
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
pub fn perform_auto_recall(
    db: &VantaEmbedded,
    params: AutoRecallParams<'_>,
) -> Result<Option<RecallResult>, RecallError> {
    let config = params.config;

    // ── L1 search (skipped on empty user text) ──
    let mut recalled = Vec::new();
    if !params.user_text.trim().is_empty() {
        let records = read_session_records(db, params.session_key)?;
        recalled = search_keyword(&records, params.user_text, &config);
    }

    // ── L3 persona (scoped by team+agent via profile_sync) ──
    let isolation = params.isolation.unwrap_or_default();
    let scoped = read_scoped_persona(db, &isolation)?;
    let persona_body = match scoped {
        Some(content) => {
            let body = strip_scene_navigation(&content).trim().to_string();
            (!body.is_empty()).then_some(body)
        }
        // Fallback: session-level persona written directly by MEM-15 without a
        // scope sync yet.
        None => get_persona(db, params.session_key)?
            .map(|p| strip_scene_navigation(&p.content).trim().to_string())
            .filter(|b| !b.is_empty()),
    };

    // ── L2 scene navigation ──
    let entries = list_scenes(db, params.session_key)?;
    let navigation = (!entries.is_empty()).then(|| generate_scene_navigation(&entries));

    if recalled.is_empty() && persona_body.is_none() && navigation.is_none() {
        return Ok(None);
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
        effective_mode: config.mode.effective(),
    }))
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

/// Keyword-overlap search over the session's L1 records, formatted into
/// `- [type|scene] content` lines with budget applied.
fn search_keyword(records: &[MemoryRecord], query: &str, config: &RecallConfig) -> Vec<RecallHit> {
    let terms = significant_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(usize, &MemoryRecord)> = records
        .iter()
        .map(|r| (overlap_score(&r.content, query), r))
        .filter(|(score, _)| *score >= config.min_overlap.max(1))
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });

    let hits: Vec<RecallHit> = scored
        .into_iter()
        .take(config.max_results)
        .map(|(score, r)| {
            let line = format_memory_line(r);
            let memory_type =
                serde_json::to_string(&r.memory_type).unwrap_or_else(|_| "\"unknown\"".to_string());
            let memory_type = memory_type.trim_matches('"').to_string();
            let content = line
                .split_once("] ")
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_else(|| line.clone());
            RecallHit {
                line,
                memory: RecalledMemory {
                    content,
                    score,
                    memory_type,
                },
            }
        })
        .collect();

    // Budget operates on lines; re-pair the surviving lines with their
    // structured payloads by position.
    let budgeted_lines = apply_recall_budget(hits.iter().map(|h| h.line.clone()).collect(), config);
    budgeted_lines
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
        .collect()
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
fn truncate_line(line: &str, max_chars: usize) -> String {
    if line.chars().count() <= max_chars {
        return line.to_string();
    }
    if max_chars <= TRUNCATION_SUFFIX.chars().count() {
        return line.chars().take(max_chars).collect();
    }
    let keep = max_chars - TRUNCATION_SUFFIX.chars().count();
    let head: String = line.chars().take(keep).collect();
    format!("{}{TRUNCATION_SUFFIX}", head.trim_end())
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
        }
    }

    #[test]
    fn modes_degrade_to_keyword_without_embedding_resources() {
        assert_eq!(RecallMode::Keyword.effective(), RecallMode::Keyword);
        assert_eq!(RecallMode::Embedding.effective(), RecallMode::Keyword);
        assert_eq!(RecallMode::Hybrid.effective(), RecallMode::Keyword);
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
