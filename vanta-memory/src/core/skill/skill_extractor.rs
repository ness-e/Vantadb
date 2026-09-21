//! SkillExtractor (MEM-17, F4) — transcript → skill candidates via LLM review.
//!
//! Port of TDAM `MC/core/skill/skill-extractor.ts` reduced to the pure-text
//! JSON contract of [`crate::core::abstractions::LlmRunner`] (no tool loop —
//! documented deviation; the deterministic layer validates everything the LLM
//! emits, same trust boundary as L1/L2/L3).
//!
//! Key TDAM mechanics preserved:
//! - **Transcript markers**: turns wrapped in non-natural `<<past-{role}>>`
//!   tags + a final `<<end-of-transcript>>` anchor. `[user]`-style prefixes
//!   are native chat-completion role signals and invite role-capture (TDAM
//!   trace f546ab8c: 1235 tokens of main-conversation continuation, zero tool
//!   calls). The odd tags break that reflex.
//! - **Head-tail truncation** with an explicit `[truncated N chars]` marker.
//! - **Query sanitisation** for BM25 keyword generation.
//! - **Prefix skills block**: existing skills injected ahead of the
//!   transcript so the reviewer prefers update over duplicate-create.

use serde::{Deserialize, Serialize};

use crate::core::abstractions::{LlmRunParams, LlmRunner};
use crate::core::skill::prompts::SKILL_REVIEW_PROMPT;

/// One transcript turn handed to the extractor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractMessage {
    /// `user` | `assistant` | `tool_call` | `tool_result` | `system`.
    /// Unknown roles degrade to `user` (TDAM `toExtractMessages`).
    pub role: String,
    pub content: String,
}

impl ExtractMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// A candidate operation emitted by the review agent. Serde snake_case so the
/// prompt's JSON contract deserializes directly (LLM output is untrusted —
/// revalidated before any write).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedSkillCandidate {
    /// `"create"` or `"update"`. Anything else skips the candidate.
    pub action: String,
    /// Skill name (lowercase-digits-hyphens recommended by the prompt).
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Full reusable body.
    #[serde(default)]
    pub content: String,
}

/// Existing-skill summary for the prefix block (name + short description).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
}

/// Extractor tuning (TDAM constructor defaults).
#[derive(Debug, Clone)]
pub struct SkillExtractorConfig {
    /// Transcript head-tail truncation: chars kept from the start.
    pub head_chars: usize,
    /// Transcript head-tail truncation: chars kept from the end.
    pub tail_chars: usize,
    /// Max existing skills rendered into the prefix block (0 = off).
    pub prefix_skills_limit: usize,
}

impl Default for SkillExtractorConfig {
    fn default() -> Self {
        Self {
            head_chars: 8_000,
            tail_chars: 32_000,
            prefix_skills_limit: 20,
        }
    }
}

/// Result of one extraction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillExtractionResult {
    /// `false` when the LLM run failed or emitted unparseable output — no
    /// candidates were produced and NOTHING was written (Principio 4).
    pub success: bool,
    pub candidates: Vec<ExtractedSkillCandidate>,
    pub error: Option<String>,
}

/// Serialize messages into the marked transcript (TDAM `formatTranscript`).
pub fn format_transcript(messages: &[ExtractMessage]) -> String {
    let body = messages
        .iter()
        .map(|m| format!("<<past-{}>>\n{}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "{body}\n\n<<end-of-transcript>>\nAbove is the past conversation to review. Now decide, \
and respond only per the output contract in the system prompt."
    )
}

/// Head-tail truncation with an explicit omission marker (TDAM
/// `truncateHeadTail`). Char-based (Rust `char`s, not UTF-16 units).
pub fn truncate_head_tail(s: &str, head_chars: usize, tail_chars: usize) -> String {
    let total = s.chars().count();
    if total <= head_chars.saturating_add(tail_chars) {
        return s.to_string();
    }
    let head: String = s.chars().take(head_chars).collect();
    let tail: String = s.chars().skip(total - tail_chars).collect();
    format!(
        "{head}\n\n... [truncated {} chars] ...\n\n{tail}",
        total - head_chars - tail_chars
    )
}

/// Validate a generated BM25 keyword query (TDAM `sanitizeGeneratedQuery`):
/// first non-empty line only, punctuation and FTS5 reserved words stripped,
/// whitespace collapsed, capped at 120 chars. Empty output = nothing found.
pub fn sanitize_generated_query(raw: &str) -> String {
    let first_line = raw.lines().map(str::trim).find(|l| !l.is_empty());
    let Some(line) = first_line else {
        return String::new();
    };
    let cleaned: String = line
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
                ch
            } else {
                ' '
            }
        })
        .collect();
    // Drop FTS5 reserved words (case-insensitive) after punctuation removal.
    let collapsed = cleaned
        .split_whitespace()
        .filter(|w| {
            !matches!(
                w.to_ascii_uppercase().as_str(),
                "AND" | "OR" | "NOT" | "NEAR"
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    if collapsed.chars().count() > 120 {
        collapsed.chars().take(120).collect()
    } else {
        collapsed
    }
}

/// Render one `- name — description` line (shared by all prefix modes).
fn format_skill_line(name: &str, description: &str) -> String {
    let desc: String = description.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut short: String = desc.chars().take(100).collect();
    if desc.chars().count() > 100 {
        short.push('…');
    }
    if short.is_empty() {
        format!("- {name}")
    } else {
        format!("- {name} — {short}")
    }
}

/// Prefix block listing existing skills so the reviewer prefers update/patch
/// over duplicate creates (TDAM full/recent blocks consolidated: this port
/// always knows the exact list it is given, so one honest rendering covers
/// both — the caller passes what it has and the total).
pub fn render_skills_block(items: &[SkillSummary], total: usize) -> String {
    let lines: Vec<String> = items
        .iter()
        .map(|s| format_skill_line(&s.name, &s.description))
        .collect();
    let omitted = total.saturating_sub(items.len());
    let header = if omitted > 0 {
        format!(
            "## Skills you (this agent) own ({}/{} shown)\n\
Most relevant first. {omitted} more not shown — search the rest before creating a duplicate.",
            items.len(),
            total
        )
    } else {
        format!(
            "## Skills you (this agent) own ({} total — full list, no truncation)",
            items.len()
        )
    };
    let hint = "Consider updating an existing skill instead of creating a near-duplicate.";
    let mut block = vec![header.to_string(), hint.to_string()];
    block.extend(lines);
    block.join("\n")
}

/// Run one extraction pass: transcript → prompt → candidates.
///
/// Degrades per Principio 4: empty input succeeds trivially; a runner failure
/// or unparseable output returns `success: false` with NO candidates and no
/// side effects. The exact `Nothing to save.` sentinel maps to a successful
/// empty result.
pub fn extract_skills_with_llm<R: LlmRunner>(
    runner: &R,
    messages: &[ExtractMessage],
    existing_skills: &[SkillSummary],
    config: &SkillExtractorConfig,
) -> SkillExtractionResult {
    if messages.is_empty() {
        return SkillExtractionResult {
            success: true,
            candidates: vec![],
            error: None,
        };
    }

    let truncated = truncate_head_tail(
        &format_transcript(messages),
        config.head_chars,
        config.tail_chars,
    );
    let prompt = if config.prefix_skills_limit > 0 && !existing_skills.is_empty() {
        let block = render_skills_block(existing_skills, existing_skills.len());
        format!("{block}\n\n---\n\n{truncated}")
    } else {
        truncated
    };

    let params = LlmRunParams {
        prompt,
        system_prompt: Some(SKILL_REVIEW_PROMPT.to_string()),
        task_id: "skill-extract".into(),
        timeout: None,
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };

    let raw = match runner.run(&params) {
        Ok(raw) => raw,
        Err(err) => {
            return SkillExtractionResult {
                success: false,
                candidates: vec![],
                error: Some(format!("LLM skill extraction failed: {err}")),
            };
        }
    };

    // Sentinel first: the reviewer may legitimately report nothing to save.
    if raw.trim() == "Nothing to save." {
        return SkillExtractionResult {
            success: true,
            candidates: vec![],
            error: None,
        };
    }

    let parsed = crate::offload::local_llm::parsers::json_utils::extract_json::<
        Vec<ExtractedSkillCandidate>,
    >(&raw);
    let Some(mut candidates) = parsed else {
        return SkillExtractionResult {
            success: false,
            candidates: vec![],
            error: Some("LLM skill extraction returned unparseable output".into()),
        };
    };

    // Untrusted output: keep only well-formed operations.
    candidates.retain(|c| {
        matches!(c.action.as_str(), "create" | "update")
            && !c.name.trim().is_empty()
            && !c.content.trim().is_empty()
    });

    SkillExtractionResult {
        success: true,
        candidates,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ExtractMessage {
        ExtractMessage::new(role, content)
    }

    #[test]
    fn transcript_uses_non_natural_markers_and_end_anchor() {
        let t = format_transcript(&[msg("user", "hi"), msg("assistant", "hello")]);
        assert!(t.contains("<<past-user>>\nhi"));
        assert!(t.contains("<<past-assistant>>\nhello"));
        assert!(t.contains("<<end-of-transcript>>"));
        assert!(!t.contains("[user]"), "no natural role signals");
    }

    #[test]
    fn truncation_keeps_head_and_tail_with_marker() {
        let s: String = (0..100).map(|i| char::from(b'a' + (i % 26))).collect();
        let out = truncate_head_tail(&s, 10, 5);
        assert!(out.starts_with("abcdefghij"));
        assert!(out.ends_with("rstuv"));
        assert!(out.contains("[truncated 85 chars]"));
    }

    #[test]
    fn truncation_passthrough_when_short() {
        assert_eq!(truncate_head_tail("short", 10, 10), "short");
    }

    #[test]
    fn query_sanitizer_strips_punctuation_and_reserved_words() {
        // First non-empty line only (TDAM parity).
        let q = sanitize_generated_query("k8s, crashloop! AND deploy OR \"backoff\"\nmore");
        assert_eq!(q, "k8s crashloop deploy backoff");
    }

    #[test]
    fn query_sanitizer_empty_on_blank_input() {
        assert_eq!(sanitize_generated_query("  \n\n "), "");
    }

    #[test]
    fn skills_block_lists_items_and_reports_omitted() {
        let items = vec![
            SkillSummary {
                name: "a".into(),
                description: "first".into(),
            },
            SkillSummary {
                name: "b".into(),
                description: "".into(),
            },
        ];
        let block = render_skills_block(&items, 5);
        assert!(block.contains("- a — first"));
        assert!(
            block.contains("- b"),
            "empty description falls back to name"
        );
        assert!(block.contains("3 more not shown"));
    }

    #[test]
    fn long_description_truncated_to_100_chars() {
        let line = format_skill_line("x", &"d".repeat(150));
        assert!(line.ends_with('…'));
        assert!(line.chars().count() < 110);
    }
}
