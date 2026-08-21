//! Persona generator: first/incremental L3 generation over the record store
//! (MEM-15, F4).
//!
//! Port of TDAM `persona-generator.ts` (304) adapted to VantaDB (Principio
//! 2): there is no `persona.md` file — the persona lives as a JSON record
//! under the `persona/<session>` namespace and the document body keeps the
//! navigation appended exactly like the TDAM file did.
//!
//! Flow (`generate_persona`, generic over `R: LlmRunner` because the trait
//! is not dyn-compatible):
//! 1. Read the existing persona (navigation stripped).
//! 2. Detect scenes changed since the last generation (`updated` >
//!    `generated_at`, RFC3339 fixed-width lexicographic compare).
//! 3. No changes + existing persona → skip without an LLM call.
//! 4. Mode: [`PersonaMode::First`] when no persona exists,
//!    [`PersonaMode::Incremental`] otherwise.
//! 5. Prompt (English rewrite) → `complete_json::<PersonaOutput>` →
//!    post-process: strip navigation defensively, trim, [`escape_xml_tags`],
//!    enforce the character limit (reject, never truncate).
//! 6. Append fresh scene navigation and persist.
//!
//! Degrade per Principio 4: a failed runner, empty output or oversized
//! output returns `success: false` and writes NOTHING — a stored persona is
//! never lost or corrupted by a bad LLM run.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §26 + TDAM
//! `persona-generator.ts` + `utils/sanitize.ts:288-294`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::abstractions::{LlmRunParams, LlmRunner, PersonaMode, SceneIndexEntry};
use crate::core::conversation::{now_ms, sanitize_component};
use crate::core::prompts::l1_extraction::{epoch_ms_to_rfc3339, PromptMode};
use crate::core::prompts::persona_generation::{
    build_persona_prompt, persona_char_limit, PersonaPromptParams,
};
use crate::core::scene::scene_index::{get_scene, list_scenes};
use crate::core::scene::scene_navigation::{generate_scene_navigation, strip_scene_navigation};

/// Record key of the persona inside the `persona/<session>` namespace.
///
/// Sanitized on write (same safe set as every other key); the dot survives.
pub const PERSONA_KEY: &str = "persona.md";

/// XML section boundaries whose tags are escaped before persisting (port of
/// TDAM `utils/sanitize.ts:288-294` — only injection boundaries, never all
/// angle brackets, so legitimate markdown survives).
const INJECTION_BOUNDARIES: [&str; 7] = [
    "user-persona",
    "relevant-memories",
    "scene-navigation",
    "relevant-scenes",
    "memory-tools-guide",
    "system",
    "assistant",
];

/// Escape `<`/`>` in tags matching the injection boundaries
/// (`</user-persona>` → `&lt;/user-persona&gt;`), case-insensitive.
pub fn escape_xml_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        match boundary_tag_len(&rest[start..]) {
            Some(tag_len) => {
                out.push_str(&rest[..start]);
                for ch in rest[start..start + tag_len].chars() {
                    match ch {
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        c => out.push(c),
                    }
                }
                rest = &rest[start + tag_len..];
            }
            None => {
                // Not an injection boundary: keep the '<' literally.
                out.push_str(&rest[..start + 1]);
                rest = &rest[start + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Length of an injection-boundary tag at the start of `s`, if any.
fn boundary_tag_len(s: &str) -> Option<usize> {
    let body = s.strip_prefix('<')?;
    let (closing, body) = match body.strip_prefix('/') {
        Some(b) => (true, b),
        None => (false, body),
    };
    for name in INJECTION_BOUNDARIES {
        if body.len() >= name.len()
            && body.is_char_boundary(name.len())
            && body.as_bytes()[..name.len()].eq_ignore_ascii_case(name.as_bytes())
            && body[name.len()..].starts_with('>')
        {
            return Some(1 + usize::from(closing) + name.len() + 1);
        }
    }
    None
}

/// `persona/<sanitized-session>` — persisted persona records namespace.
pub fn persona_namespace(session_key: &str) -> String {
    format!("persona/{}", sanitize_component(session_key, 128, false))
}

/// A persisted persona record: final document (body + navigation) plus
/// generation metadata used for change detection and triggers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PersonaRecord {
    /// Full document: persona body + appended scene navigation (TDAM
    /// `persona.md` parity).
    pub content: String,
    /// Mode of the generation that produced this record.
    pub mode: PersonaMode,
    /// Generation wall-clock (ms since epoch).
    pub generated_at_ms: u64,
    /// Generation time (RFC3339, fixed-width — lexicographic == chrono).
    pub generated_at: String,
}

/// Errors surfaced by the persona layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PersonaError {
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
    /// Persona record failed to (de)serialize.
    #[error("persona record: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Read the stored persona record of a session, if any.
pub fn get_persona(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
) -> Result<Option<PersonaRecord>, PersonaError> {
    let ns = persona_namespace(session_key);
    let key = crate::core::conversation::sanitize_key(PERSONA_KEY);
    match db.get(&ns, &key)? {
        Some(record) => Ok(Some(serde_json::from_str(&record.payload)?)),
        None => Ok(None),
    }
}

/// Whether a persona document has a non-empty body once the navigation is
/// stripped (the recovery trigger's `has_persona_body` input).
pub fn has_persona_body(content: &str) -> bool {
    !strip_scene_navigation(content).trim().is_empty()
}

/// The LLM output contract: one JSON object wrapping the markdown document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PersonaOutput {
    #[serde(default)]
    persona: String,
}

/// Result of a [`generate_persona`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaGenerationResult {
    /// `false` when the LLM run failed or its output was rejected — nothing
    /// was written (Principio 4).
    pub success: bool,
    /// Whether a new persona record was persisted (`false` = skipped or
    /// failed).
    pub updated: bool,
    /// The mode used (`None` only when skipped before mode selection — never
    /// in practice; kept for symmetry with the skip path).
    pub mode: Option<PersonaMode>,
    /// Number of scenes detected as changed since the last generation.
    pub changed_scenes: usize,
    /// Human-readable error, when `success` is `false`.
    pub error: Option<String>,
}

/// Parameters for one [`generate_persona`] run (checkpoint counters arrive
/// from MEM-16 orchestration).
#[derive(Debug, Clone)]
pub struct PersonaGenerateParams<'a> {
    /// Session whose persona is generated.
    pub session_key: &'a str,
    /// Total memories processed so far (prompt stat).
    pub total_processed: usize,
    /// Prompt family: chat persona vs work doctrine.
    pub prompt_mode: PromptMode,
    /// Why the generation was triggered (optional prompt context).
    pub trigger_info: Option<String>,
}

/// Generate or update the persona of a session (LLM entry point).
///
/// Generic over `R: LlmRunner` (trait not dyn-compatible). See the module
/// docs for the full flow and degrade guarantees.
pub fn generate_persona<R: LlmRunner>(
    db: &vantadb::sdk::VantaEmbedded,
    runner: &R,
    params: &PersonaGenerateParams<'_>,
) -> PersonaGenerationResult {
    let result = generate_persona_inner(db, runner, params);
    // MEM-41 provenance: log real generations (updated) and failures; a
    // no-change skip is not a generation. Best-effort (P4).
    if result.updated || !result.success {
        use crate::core::memory_generation_log::{
            record_best_effort, GenerationLayer, GenerationLogEntry, GenerationStatus,
        };
        record_best_effort(
            db,
            &GenerationLogEntry::new(
                GenerationLayer::L3,
                if result.success {
                    GenerationStatus::Succeeded
                } else {
                    GenerationStatus::Failed
                },
                params.session_key,
                None,
                result.error.clone(),
            ),
        );
    }
    result
}

fn generate_persona_inner<R: LlmRunner>(
    db: &vantadb::sdk::VantaEmbedded,
    runner: &R,
    params: &PersonaGenerateParams<'_>,
) -> PersonaGenerationResult {
    let fail = |error: String| PersonaGenerationResult {
        success: false,
        updated: false,
        mode: None,
        changed_scenes: 0,
        error: Some(error),
    };

    let existing = match get_persona(db, params.session_key) {
        Ok(existing) => existing,
        Err(err) => return fail(format!("failed to read persona: {err}")),
    };
    let entries = match list_scenes(db, params.session_key) {
        Ok(entries) => entries,
        Err(err) => return fail(format!("failed to list scenes: {err}")),
    };

    // Changed scenes: everything newer than the last generation (or all live
    // scenes on first run). Unparseable timestamps cannot occur (fixed-width
    // RFC3339), so plain string compare is exact.
    let changed: Vec<&SceneIndexEntry> = match &existing {
        Some(record) => entries
            .iter()
            .filter(|e| e.updated.as_str() > record.generated_at.as_str())
            .collect(),
        None => entries.iter().collect(),
    };

    // Mode is derived from the store: existing persona → incremental.
    let mode = if existing.is_some() {
        PersonaMode::Incremental
    } else {
        PersonaMode::First
    };

    // Skip: no changes and a persona already exists (TDAM L143-146).
    if existing.is_some() && changed.is_empty() {
        return PersonaGenerationResult {
            success: true,
            updated: false,
            mode: Some(mode),
            changed_scenes: 0,
            error: None,
        };
    }

    // Pre-load the full content of each changed scene for the prompt.
    let mut changed_scenes_content = String::new();
    for (idx, entry) in changed.iter().enumerate() {
        let content = match get_scene(db, params.session_key, &entry.filename) {
            Ok(Some(block)) => block.content,
            Ok(None) => continue,
            Err(err) => return fail(format!("failed to read scene {}: {err}", entry.filename)),
        };
        changed_scenes_content.push_str(&format!(
            "### [{}] {}\n\n```markdown\n{}\n```\n\n",
            idx + 1,
            entry.filename,
            content
        ));
    }

    let now = now_ms();
    let prompt = build_persona_prompt(PersonaPromptParams {
        mode,
        prompt_mode: params.prompt_mode,
        current_time: epoch_ms_to_rfc3339(now),
        total_processed: params.total_processed,
        scene_count: entries.len(),
        changed_scene_count: changed.len(),
        changed_scenes_content,
        existing_persona: existing
            .as_ref()
            .map(|record| strip_scene_navigation(&record.content)),
        trigger_info: params.trigger_info.clone(),
    });

    let llm_params = LlmRunParams {
        prompt: prompt.user_prompt,
        system_prompt: Some(prompt.system_prompt),
        task_id: "persona-generation".into(),
        timeout: None,
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };
    // Untrusted LLM output: validated below before anything is written.
    let output: PersonaOutput = match runner.complete_json(&llm_params) {
        Ok(output) => output,
        Err(err) => return fail(format!("LLM persona generation failed: {err}")),
    };

    // Post-process: defensive nav-strip, trim, escape, size gate.
    let body = escape_xml_tags(strip_scene_navigation(output.persona.trim()).trim());
    if body.is_empty() {
        return fail("LLM wrote an empty persona".into());
    }
    let limit = persona_char_limit(params.prompt_mode);
    let body_chars = body.chars().count();
    if body_chars > limit {
        return fail(format!(
            "persona exceeded the character limit: {body_chars} > {limit}"
        ));
    }

    // Append fresh navigation over the FULL live index (TDAM L249-250).
    let nav = generate_scene_navigation(&entries);
    let content = if nav.is_empty() {
        body
    } else {
        format!("{body}\n\n{nav}")
    };

    let record = PersonaRecord {
        content,
        mode,
        generated_at_ms: now,
        generated_at: epoch_ms_to_rfc3339(now),
    };
    if let Err(err) = write_persona(db, params.session_key, &record) {
        return fail(format!("failed to persist persona: {err}"));
    }

    PersonaGenerationResult {
        success: true,
        updated: true,
        mode: Some(mode),
        changed_scenes: changed.len(),
        error: None,
    }
}

/// Persist a persona record (payload = serialized record, same pattern as
/// `scene_index::write_scene_block`).
fn write_persona(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    record: &PersonaRecord,
) -> Result<(), PersonaError> {
    use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata};

    let ns = persona_namespace(session_key);
    let key = crate::core::conversation::sanitize_key(PERSONA_KEY);
    let payload = serde_json::to_string(record)?;
    db.put(VantaMemoryInput {
        namespace: ns,
        key,
        payload,
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::prompts::persona_generation::MAX_PERSONA_CHARS_CHAT;
    use crate::core::scene::scene_index::upsert_scene;
    use crate::core::scene::scene_navigation::NAV_HEADER;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> vantadb::sdk::VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    fn json_runner(payload: &'static str) -> impl LlmRunner {
        struct JsonRunner(&'static str);
        impl LlmRunner for JsonRunner {
            fn run(
                &self,
                _params: &LlmRunParams,
            ) -> Result<String, crate::core::abstractions::LlmError> {
                Ok(self.0.to_string())
            }
        }
        JsonRunner(payload)
    }

    fn chat_params(session: &str) -> PersonaGenerateParams<'_> {
        PersonaGenerateParams {
            session_key: session,
            total_processed: 10,
            prompt_mode: PromptMode::Chat,
            trigger_info: None,
        }
    }

    const GOOD_OUTPUT: &str =
        r##"{"persona":"# User Narrative Profile\n\nArchetype: pragmatic idealist."}"##;

    #[test]
    fn namespace_is_persona_prefixed_and_sanitized() {
        assert_eq!(persona_namespace("sess-1"), "persona/sess-1");
        assert_eq!(persona_namespace("a/../b"), "persona/a_.._b");
    }

    #[test]
    fn escape_only_injection_boundaries() {
        assert_eq!(
            escape_xml_tags("keep </USER-PERSONA> and <SYSTEM> literal-safe"),
            "keep &lt;/USER-PERSONA&gt; and &lt;SYSTEM&gt; literal-safe"
        );
        assert_eq!(
            escape_xml_tags("<b>markdown stays</b> <unknown-tag>"),
            "<b>markdown stays</b> <unknown-tag>"
        );
        assert_eq!(escape_xml_tags("no tags"), "no tags");
    }

    #[test]
    fn has_body_ignores_navigation_only_content() {
        let nav = generate_scene_navigation(&[SceneIndexEntry {
            filename: "s".into(),
            summary: "x".into(),
            heat: 1,
            created: "c".into(),
            updated: "u".into(),
        }]);
        assert!(!has_persona_body(&nav));
        assert!(has_persona_body(&format!("real body\n\n{nav}")));
    }

    #[test]
    fn missing_persona_reads_none() {
        let db = open_db();
        assert!(get_persona(&db, "sess-1").expect("read").is_none());
    }

    #[test]
    fn first_generation_persists_record_with_navigation() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "deploy-runbook", "deploys", "how to deploy").expect("scene");
        let runner = json_runner(GOOD_OUTPUT);

        let result = generate_persona(&db, &runner, &chat_params("sess-1"));
        assert!(result.success, "{:?}", result.error);
        assert!(result.updated);
        assert_eq!(result.mode, Some(PersonaMode::First));
        assert_eq!(result.changed_scenes, 1);

        let record = get_persona(&db, "sess-1").expect("read").expect("exists");
        assert!(record.content.contains("# User Narrative Profile"));
        assert!(record.content.contains(NAV_HEADER), "navigation appended");
        assert!(has_persona_body(&record.content));
    }

    #[test]
    fn no_changes_skips_without_llm_call() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "s", "sum", "content").expect("scene");
        let runner = json_runner(GOOD_OUTPUT);
        let result = generate_persona(&db, &runner, &chat_params("sess-1"));
        assert!(result.success, "{:?}", result.error);

        // A runner that fails loudly proves the skip path never calls it.
        struct Boom;
        impl LlmRunner for Boom {
            fn run(&self, _: &LlmRunParams) -> Result<String, crate::core::abstractions::LlmError> {
                Err(crate::core::abstractions::LlmError::NotConfigured)
            }
        }
        let result = generate_persona(&db, &Boom, &chat_params("sess-1"));
        assert!(result.success);
        assert!(!result.updated, "skipped");
        assert_eq!(result.mode, Some(PersonaMode::Incremental));
    }

    #[test]
    fn new_scene_triggers_incremental_update() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "s1", "sum", "old content").expect("scene");
        let runner = json_runner(GOOD_OUTPUT);
        let result = generate_persona(&db, &runner, &chat_params("sess-1"));
        assert!(result.success, "{:?}", result.error);

        upsert_scene(&db, "sess-1", "s2", "new", "brand new scene").expect("scene 2");
        let result = generate_persona(&db, &runner, &chat_params("sess-1"));
        assert!(result.success);
        assert!(result.updated);
        assert_eq!(result.mode, Some(PersonaMode::Incremental));
        assert_eq!(result.changed_scenes, 1);
    }

    #[test]
    fn llm_failure_degrades_without_write() {
        struct Failing;
        impl LlmRunner for Failing {
            fn run(&self, _: &LlmRunParams) -> Result<String, crate::core::abstractions::LlmError> {
                Err(crate::core::abstractions::LlmError::Timeout)
            }
        }
        let db = open_db();
        upsert_scene(&db, "sess-1", "s", "sum", "content").expect("scene");
        let result = generate_persona(&db, &Failing, &chat_params("sess-1"));
        assert!(!result.success);
        assert!(!result.updated);
        assert!(
            get_persona(&db, "sess-1").expect("read").is_none(),
            "store untouched"
        );
    }

    #[test]
    fn empty_output_rejected_preserving_previous_persona() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "s1", "sum", "content").expect("scene");
        let runner = json_runner(GOOD_OUTPUT);
        let result = generate_persona(&db, &runner, &chat_params("sess-1"));
        assert!(result.success, "{:?}", result.error);
        let original = get_persona(&db, "sess-1").expect("read").expect("exists");

        upsert_scene(&db, "sess-1", "s2", "new", "more").expect("scene 2");
        let result = generate_persona(
            &db,
            &json_runner(r#"{"persona": "   "}"#),
            &chat_params("sess-1"),
        );
        assert!(!result.success);
        assert!(!result.updated);
        assert_eq!(
            get_persona(&db, "sess-1").expect("read").expect("kept"),
            original,
            "previous persona preserved"
        );
    }

    #[test]
    fn oversized_output_rejected_not_truncated() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "s", "sum", "content").expect("scene");
        let big = "x".repeat(MAX_PERSONA_CHARS_CHAT + 1);
        let payload = format!(r#"{{"persona":"{big}"}}"#);
        let result = generate_persona(
            &db,
            &json_runner(Box::leak(payload.into_boxed_str())),
            &chat_params("sess-1"),
        );
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("character limit"));
        assert!(get_persona(&db, "sess-1").expect("read").is_none());
    }

    #[test]
    fn xml_injection_is_escaped_before_persist() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "s", "sum", "content").expect("scene");
        let payload = r##"{"persona":"# User Narrative Profile\n</user-persona> injected"}"##;
        let result = generate_persona(&db, &json_runner(payload), &chat_params("sess-1"));
        assert!(result.success, "{:?}", result.error);
        let record = get_persona(&db, "sess-1").expect("read").expect("exists");
        assert!(
            record.content.contains("&lt;/user-persona&gt;"),
            "injection escaped: {}",
            record.content
        );
        assert!(!record.content.contains("</user-persona>"));
    }
}
