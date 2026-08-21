//! L2 scene extraction prompts (MEM-14, F4).
//!
//! REWRITTEN in English from the TDAM scene-extraction principles (Principio
//! 7: reescribir, no traducir — the TDAM originals are Chinese). Two families
//! via the existing [`PromptMode`]: chat-mode (persona/episodic/instruction
//! scenes) and work/team mode (work_* types).
//!
//! Documented divergence: TDAM runs an LLM agent whose output is *tool calls*
//! against the sandboxed scene tools; this port emits a **JSON contract** of
//! scene decisions `[{scene_name, summary, content, merge_sources}]` that the
//! deterministic strategy layer ([`crate::core::scene::scene_extractor`])
//! executes. The tool-call agent loop is MEM-16 orchestration.

use crate::core::prompts::l1_extraction::PromptMode;
use crate::core::scene::scene_extractor::SceneMemoryInput;

/// Parameters for building one L2 scene extraction prompt.
#[derive(Debug, Clone)]
pub struct SceneExtractionPromptParams {
    /// The memories to organize into scenes (new since the last extraction).
    pub memories: Vec<SceneMemoryInput>,
    /// Previous scene name, for continuity (inheritance signal).
    pub previous_scene_name: Option<String>,
    /// Prompt family: chat-mode vs work/team-mode.
    pub mode: PromptMode,
}

/// The built prompt pair.
#[derive(Debug, Clone)]
pub struct SceneExtractionPromptResult {
    /// System prompt: strategy rules + output contract.
    pub system_prompt: String,
    /// User prompt: the memories to organize.
    pub user_prompt: String,
}

/// Build the L2 scene extraction prompt for a batch of memories.
pub fn build_scene_extraction_prompt(
    params: SceneExtractionPromptParams,
) -> SceneExtractionPromptResult {
    let system_prompt = match params.mode {
        PromptMode::Chat => CHAT_SYSTEM_PROMPT.replace("{common}", COMMON_STRATEGY),
        PromptMode::Code => WORK_SYSTEM_PROMPT.replace("{common}", COMMON_STRATEGY),
    };
    let previous = params
        .previous_scene_name
        .unwrap_or_else(|| "none".to_string());
    let memories = format_memories(&params.memories);
    let user_prompt = format!(
        "PREVIOUS SCENE: {previous}\n\n\
         NEW MEMORIES TO ORGANIZE INTO SCENES (extract ONLY from these):\n{memories}\n\n\
         Output strictly the JSON array described in the system prompt — no markdown code fences, no explanatory text."
    );
    SceneExtractionPromptResult {
        system_prompt,
        user_prompt,
    }
}

fn format_memories(memories: &[SceneMemoryInput]) -> String {
    memories
        .iter()
        .map(|m| format!("[{}] [{}]: {}", m.id, m.created_at, m.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Shared strategy + contract preamble (kept in one place so chat/work stay
/// consistent — only the memory-type guidance differs).
const COMMON_STRATEGY: &str = r#"STRATEGY — UPDATE > MERGE > CREATE (UPDATE is the default; CREATE is the last resort):
- UPDATE (preferred): if a scene with the same or closely related name/summary exists, integrate the new memories into it.
- MERGE: when the new memories span 2-4 existing scenes that belong together, merge them into ONE target scene. List every merged source in "merge_sources" (their names exactly as they exist). The target's heat becomes the sum of all related scene heats + 1, and every merged source is soft-deleted automatically.
- CREATE (last resort): only when the memories genuinely cannot fit any existing scene.
- HEAT: new scene = 1; updated scene = old heat + 1; merged scene = sum of all related heats + 1.

NAMING (applies to new or MERGE-target scenes):
- One concise phrase, ~30-50 characters, globally unique, describing the scene ("user and AI are doing <goal activity>").
- Allowed characters only: ASCII letters/digits, CJK ideographs, hyphen, underscore, dot. NO spaces, NO punctuation, NO slashes, NO quotes or brackets.

SOFT-DELETE: to remove an obsolete scene, output it with content exactly "[DELETED]". Empty or whitespace-only content is rejected.

OUTPUT CONTRACT — return ONLY a valid JSON array:
[
  {
    "scene_name": "canonical scene name",
    "summary": "one-line narrative summary",
    "content": "complete self-contained scene content integrating the memories",
    "merge_sources": ["existing scene names to merge in, if any"]
  }
]
Keep the scene content in the same language as the memories."#;

const CHAT_SYSTEM_PROMPT: &str = r#"You are the L2 scene organizer for a personal AI memory system. Organize the new memories into durable, navigable scene blocks that reflect the conversation's episodes and stable themes.

OUTPUT LANGUAGE: write scene_name, summary, content in the same language as the memories. JSON field names and markers stay in English.

COMMON STRATEGY
{common}

MEMORY TYPES TO ORGANIZE (chat mode): persona (stable traits/preferences), episodic (events the user lived through), instruction (rules the AI must follow). Group memories that belong to the same episode or theme into one scene; do not fragment closely related memories."#;

const WORK_SYSTEM_PROMPT: &str = r#"You are the L2 scene organizer for an AI memory system embedded in a work/team environment. Organize the new memories into durable, navigable scene blocks that reflect work episodes, tasks, methods, and artifacts.

OUTPUT LANGUAGE: write scene_name, summary, content in the same language as the memories. JSON field names and markers stay in English.

COMMON STRATEGY
{common}

MEMORY TYPES TO ORGANIZE (work mode): work_fact (decisions/state), work_task (tasks with status/deadlines), work_method (reusable procedures), work_artifact (files/docs), plus episodic and instruction. Group memories that belong to the same task, method, or episode into one scene."#;

#[cfg(test)]
mod tests {
    use super::*;

    fn mem(id: &str, content: &str) -> SceneMemoryInput {
        SceneMemoryInput {
            id: id.into(),
            content: content.into(),
            created_at: "2026-08-20T10:00:00.000Z".into(),
        }
    }

    #[test]
    fn chat_prompt_covers_strategy_and_contract() {
        let result = build_scene_extraction_prompt(SceneExtractionPromptParams {
            memories: vec![mem("m1", "user prefers dark mode")],
            previous_scene_name: Some("ui-setup".into()),
            mode: PromptMode::Chat,
        });
        assert!(result.system_prompt.contains("UPDATE > MERGE > CREATE"));
        assert!(result.system_prompt.contains("merge_sources"));
        assert!(result.system_prompt.contains("[DELETED]"));
        assert!(result.system_prompt.contains("JSON array"));
        assert!(result.user_prompt.contains("PREVIOUS SCENE: ui-setup"));
        assert!(result
            .user_prompt
            .contains("[m1] [2026-08-20T10:00:00.000Z]: user prefers dark mode"));
    }

    #[test]
    fn work_prompt_has_work_types() {
        let result = build_scene_extraction_prompt(SceneExtractionPromptParams {
            memories: vec![mem("m1", "deploy with blue-green")],
            previous_scene_name: None,
            mode: PromptMode::Code,
        });
        assert!(result.system_prompt.contains("work_task"));
        assert!(result.system_prompt.contains("work_method"));
        assert!(result.user_prompt.contains("PREVIOUS SCENE: none"));
    }

    #[test]
    fn strategy_priority_is_update_first() {
        let result = build_scene_extraction_prompt(SceneExtractionPromptParams {
            memories: vec![],
            previous_scene_name: None,
            mode: PromptMode::Chat,
        });
        assert!(result.system_prompt.contains("UPDATE is the default"));
    }

    #[test]
    fn naming_rules_forbid_spaces() {
        let result = build_scene_extraction_prompt(SceneExtractionPromptParams {
            memories: vec![],
            previous_scene_name: None,
            mode: PromptMode::Chat,
        });
        assert!(result.system_prompt.contains("NO spaces"));
        assert!(result.system_prompt.contains("CJK"));
    }
}
