//! Local-first auto-consolidation (MCP-41): heuristic extract over conversation
//! turns, applied through the existing L2 strategy — no LLM key required.
//!
//! This closes the mem0/graphiti gap at the `extract` link: `consolidate`
//! ([`decide_strategy`]/[`apply_strategy`]/[`extract_scenes`], MEM-14) and
//! `recall` (`scene_list`/`scene_query` keyword mode, MEM-21) were already
//! LLM-free. Only the turn→[`SceneExtraction`] mapping needed an LLM
//! ([`extract_scenes_with_llm`]); [`extract_local`] replaces it with a
//! deterministic heuristic, and [`auto_consolidate`] is the one-shot
//! extract→store entry point. Recall stays where it is (gateway + MCP
//! `scene_*` tools) by design.
//!
//! Heuristic contract (deliberately weak — documented, not guaranteed):
//! one live scene per non-empty turn, named after its top significant terms
//! (sorted for determinism; `significant_terms` iterates a `HashSet`) with a
//! `turn-{index}` fallback. Same turns re-consolidated hit UPDATE (heat +1,
//! TDAM parity: prefer UPDATE over CREATE). No `merge_sources` are ever
//! emitted — overlap judgement needs an LLM (follow-up: MCP `scene_consolidate`
//! design in ADR-040).

use serde::{Deserialize, Serialize};

use crate::core::record::l1_reader::significant_terms;
use crate::core::scene::filename_normalizer::normalize_scene_name;
use crate::core::scene::scene_extractor::{
    extract_scenes, SceneExtraction, SceneExtractionResult, SceneExtractorError,
};
use crate::core::scene::scene_tools::MAX_CONTENT_BYTES;

/// Max significant terms composing a derived scene name (keeps names short and
/// stable; the store cap is `MAX_SCENE_NAME_BYTES`).
const MAX_NAME_TERMS: usize = 3;
/// Max chars of a turn kept in the scene summary (well under
/// `MAX_SUMMARY_BYTES` for any UTF-8 input).
const SUMMARY_CHARS: usize = 280;
/// Max chars of the role label stored per line (`role: content`).
const ROLE_CHARS: usize = 32;

/// One conversation turn to consolidate (boundary input: an MCP caller or any
/// host feeds these; roles/contents are untrusted text, never executed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LocalTurn {
    /// Speaker label (`user`, `assistant`, …). Empty → `"turn"`.
    pub role: String,
    /// Turn text. Empty/whitespace-only turns are skipped.
    pub content: String,
}

impl LocalTurn {
    /// Build a turn (`role`/`content` truncated to safe bounds at use time).
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// Heuristically map conversation turns to [`SceneExtraction`]s (pure,
/// deterministic, LLM-free). Empty input → empty vec (the caller reports
/// `empty_extraction` without touching the store).
pub fn extract_local(turns: &[LocalTurn]) -> Vec<SceneExtraction> {
    turns
        .iter()
        .enumerate()
        .filter_map(|(index, turn)| {
            if turn.content.trim().is_empty() {
                return None;
            }
            let role = truncate_chars(turn.role.trim(), ROLE_CHARS);
            let role = if role.is_empty() { "turn" } else { role };
            let content = truncate_bytes(
                &format!("{role}: {}", turn.content.trim()),
                MAX_CONTENT_BYTES,
            )
            .to_string();
            let summary = truncate_chars(&content, SUMMARY_CHARS).to_string();
            Some(SceneExtraction {
                scene_name: scene_name_for(&content, index),
                summary,
                content,
                merge_sources: Vec::new(),
            })
        })
        .collect()
}

/// One-shot local-first consolidation: [`extract_local`] + [`extract_scenes`]
/// (UPDATE>MERGE>CREATE + heat, MEM-14). The store gains one live scene per
/// non-empty turn; recall via `scene_list`/`scene_query` (unchanged).
pub fn auto_consolidate(
    db: &vantadb::sdk::Embedded,
    session_key: &str,
    turns: &[LocalTurn],
) -> Result<SceneExtractionResult, SceneExtractorError> {
    if session_key.is_empty() {
        return Err(SceneExtractorError::Invalid(
            "session_key must not be empty".into(),
        ));
    }
    extract_scenes(db, session_key, &extract_local(turns))
}

/// Derive a stable scene name from a turn's significant terms (sorted —
/// `HashSet` order is random). Falls back to `turn-{index}`; always passes
/// through [`normalize_scene_name`] (never empty: `"scene"` fallback).
fn scene_name_for(content: &str, index: usize) -> String {
    let mut terms: Vec<String> = significant_terms(content).into_iter().collect();
    terms.sort();
    let joined = terms
        .into_iter()
        .take(MAX_NAME_TERMS)
        .collect::<Vec<_>>()
        .join("-");
    let raw = if joined.is_empty() {
        format!("turn-{index}")
    } else {
        truncate_chars(&joined, 128).to_string()
    };
    normalize_scene_name(&raw)
}

/// Byte-safe prefix truncation (never splits UTF-8).
fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Char-prefix truncation for display-length caps.
fn truncate_chars(s: &str, max_chars: usize) -> &str {
    if s.chars().count() <= max_chars {
        return s;
    }
    let end = s
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scene::scene_index::{get_scene, list_scenes};
    use crate::core::scene::scene_tools::MAX_SUMMARY_BYTES;
    use crate::gateway::knowledge_handlers::{scene_query, SceneQueryRequest};
    use vantadb::config::Config;
    use vantadb::storage::BackendKind;

    fn open_db() -> vantadb::sdk::Embedded {
        let config = Config {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..Config::default()
        };
        vantadb::sdk::Embedded::open_with_config(config).expect("open in-memory db")
    }

    fn turn(role: &str, content: &str) -> LocalTurn {
        LocalTurn::new(role, content)
    }

    #[test]
    fn empty_turns_are_empty_extraction_and_touch_nothing() {
        let db = open_db();
        let result = auto_consolidate(&db, "sess-1", &[]).expect("empty ok");
        assert!(result.success);
        assert!(result.empty_extraction);
        assert!(result.applied.is_empty());
        assert!(list_scenes(&db, "sess-1").expect("list").is_empty());
    }

    #[test]
    fn whitespace_turns_are_skipped() {
        assert!(extract_local(&[turn("user", "   "), turn("assistant", "\n\t")]).is_empty());
    }

    #[test]
    fn empty_session_is_rejected() {
        let db = open_db();
        let err = auto_consolidate(&db, "", &[turn("user", "hola")]).expect_err("must reject");
        assert!(err.to_string().contains("session_key"));
    }

    #[test]
    fn single_turn_creates_heat_one_scene() {
        let db = open_db();
        let result = auto_consolidate(
            &db,
            "sess-1",
            &[turn("user", "user researched pricing tiers")],
        )
        .expect("consolidate");
        assert!(result.success);
        assert_eq!(result.applied.len(), 1);
        let scenes = list_scenes(&db, "sess-1").expect("list");
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].heat, 1);
        let block = get_scene(&db, "sess-1", &scenes[0].filename)
            .expect("get")
            .expect("exists");
        assert!(block.content.contains("pricing"));
    }

    #[test]
    fn repeat_consolidation_updates_heat() {
        let db = open_db();
        let turns = [turn("user", "we deploy with cargo")];
        auto_consolidate(&db, "sess-1", &turns).expect("first");
        let second = auto_consolidate(&db, "sess-1", &turns).expect("second");
        assert!(second.applied.iter().any(|a| matches!(
            a,
            crate::core::scene::scene_extractor::SceneAction::Updated { .. }
        )));
        let scenes = list_scenes(&db, "sess-1").expect("list");
        assert_eq!(scenes.len(), 1, "same name → UPDATE, not a second scene");
        assert_eq!(scenes[0].heat, 2);
    }

    #[test]
    fn extract_is_deterministic_and_never_emits_merges() {
        let turns = [
            turn("user", "pricing tiers deploy cargo"),
            turn("", "  hi  "),
        ];
        let first = extract_local(&turns);
        let second = extract_local(&turns);
        assert_eq!(first, second);
        assert!(first.iter().all(|e| e.merge_sources.is_empty()));
        assert_eq!(first.len(), 2);
    }

    #[test]
    fn oversize_content_is_truncated_within_store_caps() {
        let big = "palabra ".repeat(300_000); // ~2.4 MB > 1 MiB cap
        let out = extract_local(&[turn("user", &big)]);
        assert_eq!(out.len(), 1);
        assert!(out[0].content.len() <= MAX_CONTENT_BYTES);
        assert!(out[0].summary.len() <= MAX_SUMMARY_BYTES);
        assert!(out[0].content.is_char_boundary(out[0].content.len()));
    }

    #[test]
    fn vertical_loop_consolidated_scenes_are_recallable() {
        let db = open_db();
        auto_consolidate(
            &db,
            "sess-1",
            &[turn("user", "user researched pricing tiers for deployment")],
        )
        .expect("consolidate");
        let resp = scene_query(
            &db,
            &SceneQueryRequest {
                session_key: "sess-1".to_string(),
                keyword: "pricing".to_string(),
                top_k: None,
            },
            None,
        )
        .expect("recall");
        assert_eq!(resp.hits.len(), 1, "extract→consolidate→recall local-first");
        assert!(resp.hits[0].score > 0);
    }
}
