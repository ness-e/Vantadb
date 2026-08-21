//! L1 memory extraction (MEM-10): split + one LLM call + tolerant parse.
//!
//! Port of TDAM `MemoryCore/src/core/record/l1-extractor.ts` reimplemented in
//! Rust. Pipeline:
//! 1. Quality-gate the L0 messages (strict L1 filter).
//! 2. Split into background (older, context-only) + new (recent, extractable).
//! 3. ONE LLM call (`task_id: "l1-extraction"`) producing a JSON array of
//!    scene segments.
//! 4. Tolerant parse + type normalization.
//! 5. Build [`L1ExtractionResult`].
//!
//! Writing + dedup live in MEM-11 (`records`/`stored_count` stay empty here).
//! LLM failure degrades to `success: false` with no data loss — L0 messages
//! are never touched (Principio 4).

use std::time::Duration;

use crate::core::abstractions::{ExtractedMemory, L1ExtractionResult, LlmRunParams, LlmRunner};
use crate::core::conversation::L0Message;
use crate::core::prompts::l1_extraction::{
    extract_memories_system_prompt, format_extraction_prompt, PromptMode,
};
use crate::offload::local_llm::parsers::l1_parser::parse_l1_extraction;

/// Configuration for one L1 extraction run (TDAM defaults).
#[derive(Debug, Clone)]
pub struct L1ExtractorConfig {
    /// Max NEW messages sent to the LLM in one call (default 10).
    pub max_new_messages: usize,
    /// Max BACKGROUND messages for context (default 5).
    pub max_background_messages: usize,
    /// Max memories kept per extraction run (default 10).
    pub max_memories_per_session: usize,
    /// Prompt family: chat (persona/episodic/instruction) or code/work.
    pub prompt_mode: PromptMode,
}

impl Default for L1ExtractorConfig {
    fn default() -> Self {
        Self {
            max_new_messages: 10,
            max_background_messages: 5,
            max_memories_per_session: 10,
            prompt_mode: PromptMode::Chat,
        }
    }
}

/// LLM timeout for one extraction call.
const LLM_TIMEOUT: Duration = Duration::from_secs(180);

/// Extract core memories from recorded L0 messages.
///
/// Returns an empty `success` result when no message passes the quality gate
/// or when the LLM call fails — the caller (MEM-11+) decides what to do with
/// `success: false`; the L0 buffer is never consumed here.
///
/// Generic over `R: LlmRunner` because the trait's `complete_json` helper is
/// generic and therefore not `dyn`-compatible.
pub fn extract_l1_memories<R: LlmRunner>(
    runner: &R,
    messages: &[L0Message],
    previous_scene_name: Option<&str>,
    config: &L1ExtractorConfig,
) -> L1ExtractionResult {
    extract_l1_segments(runner, messages, previous_scene_name, config).0
}

/// Like [`extract_l1_memories`] but also returns the extracted memories so
/// the pipeline worker (MEM-16) can feed dedup without re-parsing. Behavior
/// is otherwise identical.
pub fn extract_l1_segments<R: LlmRunner>(
    runner: &R,
    messages: &[L0Message],
    previous_scene_name: Option<&str>,
    config: &L1ExtractorConfig,
) -> (L1ExtractionResult, Vec<ExtractedMemory>) {
    let qualified: Vec<L0Message> = messages
        .iter()
        .filter(|m| should_extract_l1(&m.content))
        .cloned()
        .collect();
    if qualified.is_empty() {
        return (empty_result(), Vec::new());
    }

    // Split: new = last max_new; background = up to max_bg immediately before.
    let new_start = qualified.len().saturating_sub(config.max_new_messages);
    let new_messages = &qualified[new_start..];
    let bg_end = new_start;
    let bg_start = bg_end.saturating_sub(config.max_background_messages);
    let background_messages: &[L0Message] = if bg_end > 0 {
        &qualified[bg_start..bg_end]
    } else {
        &[]
    };

    let params = LlmRunParams {
        prompt: format_extraction_prompt(new_messages, background_messages, previous_scene_name),
        system_prompt: Some(extract_memories_system_prompt(config.prompt_mode)),
        task_id: "l1-extraction".to_string(),
        timeout: Some(LLM_TIMEOUT),
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };

    let raw = match runner.run(&params) {
        Ok(raw) => raw,
        Err(err) => {
            tracing::warn!(error = %err, "L1 extraction LLM call failed; degrading to empty result");
            return (failed_result(), Vec::new());
        }
    };

    let scenes = parse_l1_extraction(&raw);

    let mut extracted: Vec<_> = Vec::new();
    let mut scene_names: Vec<String> = Vec::new();
    for scene in &scenes {
        scene_names.push(scene.scene_name.clone());
        extracted.extend(scene.memories.iter().cloned());
    }
    extracted.truncate(config.max_memories_per_session);

    let extracted_count = extracted.len();
    let last_scene_name = scene_names.last().cloned();
    (
        L1ExtractionResult {
            success: true,
            extracted_count,
            stored_count: 0,
            records: Vec::new(),
            scene_names,
            last_scene_name,
        },
        extracted,
    )
}

/// Strict L1 quality gate (port of TDAM `sanitize.ts:shouldExtractL1`).
/// Rejects structural noise, slash commands, and pure-symbol content that
/// could never become a meaningful memory. L0 captures everything; L1 keeps
/// only what the extractor should see.
fn should_extract_l1(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Framework-injected noise messages (TDAM `isFrameworkNoise`, sanitize.ts:233-255).
    if is_framework_noise(trimmed) {
        return false;
    }
    // Slash commands are framework directives, not user content.
    if trimmed.starts_with('/') {
        return false;
    }
    // Pure question marks never carry memory content.
    if trimmed.chars().all(|c| c == '?' || c == '？') {
        return false;
    }
    // 1-5 chars of pure non-alphanumeric symbols (e.g. "!!!", "->->").
    if trimmed.chars().count() <= 5
        && trimmed
            .chars()
            .all(|c| !c.is_alphanumeric() && !c.is_whitespace())
    {
        return false;
    }
    true
}

/// Framework-injected noise that should never be captured: bootstrap
/// placeholders, session-reset instructions, session-start acks and
/// memory-flush acks (port of TDAM `sanitize.ts:isFrameworkNoise`, 233-255).
fn is_framework_noise(t: &str) -> bool {
    // Google turn-order bootstrap placeholder.
    if t == "(session bootstrap)" {
        return true;
    }
    // Framework session-reset instruction ("A new session was started via /new or /reset").
    if t.starts_with("A new session was started via") {
        return true;
    }
    // AI's pure ack of session startup: "✅ New session started · model: ...".
    if let Some(rest) = t.strip_prefix('✅') {
        if rest.trim_start().starts_with("New session started") {
            return true;
        }
    }
    // Pre-compaction memory flush prompt injected by the framework.
    if t.starts_with("Pre-compaction memory flush") {
        return true;
    }
    // AI's NO_REPLY ack of memory flush (bare "NO_REPLY").
    if t == "NO_REPLY" {
        return true;
    }
    false
}

fn empty_result() -> L1ExtractionResult {
    L1ExtractionResult {
        success: true,
        extracted_count: 0,
        stored_count: 0,
        records: Vec::new(),
        scene_names: Vec::new(),
        last_scene_name: None,
    }
}

fn failed_result() -> L1ExtractionResult {
    L1ExtractionResult {
        success: false,
        ..empty_result()
    }
}

#[cfg(test)]
mod tests {
    use super::should_extract_l1;

    #[test]
    fn quality_gate_accepts_real_content() {
        assert!(should_extract_l1("User prefers dark mode"));
        assert!(should_extract_l1("我更喜欢深色模式")); // CJK passes
        assert!(should_extract_l1("a"));
    }

    #[test]
    fn quality_gate_rejects_noise() {
        assert!(!should_extract_l1(""));
        assert!(!should_extract_l1("   "));
        assert!(!should_extract_l1("?????"));
        assert!(!should_extract_l1("？"));
        assert!(!should_extract_l1("/clear"));
        assert!(!should_extract_l1("(session bootstrap)"));
        assert!(!should_extract_l1("A new session was started via /new"));
        assert!(!should_extract_l1("✅ New session started · model: gpt-4o"));
        assert!(!should_extract_l1(
            "Pre-compaction memory flush of 12 messages"
        ));
        assert!(!should_extract_l1("NO_REPLY"));
        assert!(!should_extract_l1("!!!"));
        assert!(!should_extract_l1("->->"));
    }

    #[test]
    fn quality_gate_accepts_similar_content() {
        // Same noise marker *embedded* (not at start) is kept — TDAM matches
        // exact/prefix, not substring.
        assert!(should_extract_l1(
            "note: A new session was started via /new"
        ));
        assert!(should_extract_l1(
            "the pre-compaction memory flush ran fine"
        ));
    }
}
