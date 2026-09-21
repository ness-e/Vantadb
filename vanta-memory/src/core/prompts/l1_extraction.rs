//! L1 extraction prompts (MEM-10).
//!
//! Reimplemented in English from the TDAM extraction principles — scene
//! segmentation + core-memory extraction in one LLM call — for a host-neutral
//! pipeline. Two families: [`PromptMode::Chat`] (persona/episodic/instruction)
//! and [`PromptMode::Code`] (work_* types, team/tool contexts).

use crate::core::conversation::L0Message;

/// Extraction prompt family: chat-mode (persona/episodic/instruction) vs
/// work/team mode (work_* types). Mirrors TDAM `MemoryPromptMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptMode {
    #[default]
    Chat,
    Code,
}

/// Build the system prompt for an L1 extraction run.
pub fn extract_memories_system_prompt(mode: PromptMode) -> String {
    match mode {
        PromptMode::Chat => CHAT_SYSTEM_PROMPT.into(),
        PromptMode::Code => WORK_SYSTEM_PROMPT.into(),
    }
}

/// Build the user prompt: previous scene + background (context-only) + the
/// NEW messages to extract from.
pub fn format_extraction_prompt(
    new_messages: &[L0Message],
    background_messages: &[L0Message],
    previous_scene_name: Option<&str>,
) -> String {
    let previous = previous_scene_name.unwrap_or("none");
    let background = if background_messages.is_empty() {
        "none".to_string()
    } else {
        format_messages(background_messages)
    };
    let new = format_messages(new_messages);
    format!(
        "OUTPUT LANGUAGE: write scene_name and memory content in the dominant \
         language of the user messages below. JSON field names, type values, \
         and ISO timestamps stay in English.\n\n\
         PREVIOUS SCENE: {previous}\n\n\
         BACKGROUND CONVERSATION (context only — NEVER extract memories from here):\n{background}\n\n\
         ============================================================\n\n\
         NEW MESSAGES TO EXTRACT FROM (use timestamps to infer time; extract ONLY from here):\n{new}"
    )
}

/// Convert an epoch-millisecond timestamp to an RFC 3339 UTC string
/// (`2023-11-14T22:13:20.000Z`). Uses the Howard Hinnant civil-from-days
/// algorithm — no chrono dependency (MEM-10 constraint).
pub fn epoch_ms_to_rfc3339(ms: u64) -> String {
    let days = ms.div_euclid(86_400_000);
    let millis_of_day = ms.rem_euclid(86_400_000);
    let (hour, rest) = (millis_of_day / 3_600_000, millis_of_day % 3_600_000);
    let (minute, sec_and_ms) = (rest / 60_000, rest % 60_000);
    let (second, millis) = (sec_and_ms / 1_000, sec_and_ms % 1_000);

    // Howard Hinnant civil_from_days: days since 1970-01-01 -> (y, m, d).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

fn format_messages(messages: &[L0Message]) -> String {
    messages
        .iter()
        .map(|m| {
            format!(
                "[{}] [{}] [{}]: {}",
                m.id.as_deref().unwrap_or("?"),
                m.role,
                epoch_ms_to_rfc3339(m.timestamp_ms),
                m.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

const CHAT_SYSTEM_PROMPT: &str = r#"You are an expert in scene segmentation and memory extraction.
Your task is to analyze the user's conversation, detect scene transitions, and extract structured core memories.

OUTPUT LANGUAGE: write all free-text fields (scene_name, memory content) in the same language as the user messages. JSON field names, type values, and ISO timestamps stay in English.

TASK 1 — SCENE SEGMENTATION
Analyze the NEW MESSAGES, together with the PREVIOUS SCENE, and determine the current conversation scene.
- Inherit: if there is no clear transition, continue the previous scene.
- Switch: the user gives an explicit instruction (e.g. "change topic"), intent changes, or a new independent goal appears.
- One conversation may have one or several scenes (when the topic switches multiple times).
- Naming: a single sentence, ~30-50 characters (or equivalent length), globally unique, describing what is happening ("The user and I are doing <goal activity>").

TASK 2 — MEMORY EXTRACTION
Using the background for context, extract core memories ONLY from the NEW MESSAGES.
General principles:
1. Quality over quantity: skip trivial chat, temporary instructions, one-off operations; drop unreliable edge information.
2. Self-contained: a memory must stand alone outside this conversation; the subject is the user or the AI.
3. Consolidate: strongly related or causal messages merge into ONE memory; never fragment.
Supported types (follow these rules strictly):
- "persona" — stable attributes, preferences, skills, values, habits. Priority 80-100 for health/taboos/core traits; 50-70 for general preferences/skills; below 50 discard.
- "episodic" — objectively occurred actions, decisions, plans, or results (no subjective feelings). Use timestamps to infer absolute time; when determinable, put "activity_start_time"/"activity_end_time" (ISO 8601) in metadata. Priority 80-100 important events; 60-70 normal complete activities; below 60 discard.
- "instruction" — long-term behavior rules, format preferences, tone control the user wants the AI to follow. Priority 90-100 core behavior rules; 70-80 important requirements; below 70 discard.
Do NOT extract: trivial chat/greetings; one-off tool requests ("translate this for me"); repeated content; the AI's own behavior/output; subjective feelings; anything outside the three types.

TASK 3 — OUTPUT FORMAT (JSON)
Return ONLY a valid JSON array. Each item is a scene with its message range and extracted memories:
[
  {
    "scene_name": "generated or inherited scene name",
    "message_ids": ["message id list for this scene"],
    "memories": [
      {
        "content": "complete, self-contained memory statement",
        "type": "persona|episodic|instruction",
        "priority": 80,
        "source_message_ids": ["msg_id_1", "msg_id_2"],
        "metadata": {}
      }
    ]
  }
]
metadata: episodic -> {"activity_start_time": "ISO8601", "activity_end_time": "ISO8601"} when determinable; otherwise {}.
If the whole conversation has no meaningful memories, still output the scene segmentation with an empty memories array.
Output strictly the JSON array — no markdown code fences, no explanatory text."#;

const WORK_SYSTEM_PROMPT: &str = r#"You are an expert in scene segmentation and memory extraction for an AI assistant embedded in a work/team environment.

OUTPUT LANGUAGE: write all free-text fields (scene_name, memory content) in the same language as the user messages. JSON field names, type values, and ISO timestamps stay in English.

TASK 1 — SCENE SEGMENTATION
Analyze the NEW MESSAGES, together with the PREVIOUS SCENE, and determine the current conversation scene.
- Inherit: if there is no clear transition, continue the previous scene.
- Switch: the user gives an explicit instruction (e.g. "change topic"), intent changes, or a new independent goal appears.
- Naming: a single sentence, ~30-50 characters (or equivalent length), globally unique, describing what is happening.

TASK 2 — MEMORY EXTRACTION
Using the background for context, extract core memories ONLY from the NEW MESSAGES.
General principles:
1. Quality over quantity: skip trivial chat, temporary instructions, one-off operations.
2. Self-contained: a memory must stand alone outside this conversation; the subject is the user, the team, or the AI.
3. Consolidate: strongly related or causal messages merge into ONE memory.
Supported types:
- "persona" — stable attributes, preferences, skills, values, habits (80-100 health/taboos/core traits; 50-70 general; below 50 discard).
- "work_fact" — immutable facts about the work context: stack, infrastructure, users, company, decisions already made (75-100; below 60 discard).
- "work_task" — task, plan, project, or issue with state and deadlines (80-100 active/blocked; below 65 discard).
- "work_method" — reusable methods, procedures, workflows, shortcuts, tools usage (70-100; below 60 discard).
- "work_artifact" — artifacts, documents, files, resources with path/location (75-100; below 60 discard).
- "episodic" — objectively occurred actions, decisions, plans, or results (80-100 important; 60-70 normal; below 60 discard).
- "instruction" — long-term behavior rules, format preferences, tone control (90-100 core; 70-80 important; below 70 discard).
Do NOT extract: trivial chat; one-off tool requests; repeated content; the AI's own behavior/output; subjective feelings.

TASK 3 — OUTPUT FORMAT (JSON)
Return ONLY a valid JSON array of scenes:
[
  {
    "scene_name": "generated or inherited scene name",
    "message_ids": ["message id list for this scene"],
    "memories": [
      {
        "content": "complete, self-contained memory statement",
        "type": "persona|work_fact|work_task|work_method|work_artifact|episodic|instruction",
        "priority": 80,
        "source_message_ids": ["msg_id_1", "msg_id_2"],
        "metadata": {}
      }
    ]
  }
]
metadata: for work_artifact include {"path": "..."} when known; for episodic include activity_start_time/activity_end_time (ISO 8601) when determinable.
If the whole conversation has no meaningful memories, still output the scene segmentation with an empty memories array.
Output strictly the JSON array — no markdown code fences, no explanatory text."#;

#[cfg(test)]
mod tests {
    use crate::core::conversation::{L0Message, L0Role};

    use super::{
        epoch_ms_to_rfc3339, extract_memories_system_prompt, format_extraction_prompt, PromptMode,
    };

    fn msg(id: &str, role: L0Role, content: &str, ts: u64) -> L0Message {
        L0Message {
            id: Some(id.to_string()),
            role,
            content: content.to_string(),
            timestamp_ms: ts,
        }
    }

    #[test]
    fn known_epoch_millis_convert() {
        // 1700000000000 ms == 2023-11-14T22:13:20.000Z
        assert_eq!(
            epoch_ms_to_rfc3339(1_700_000_000_000),
            "2023-11-14T22:13:20.000Z"
        );
        // 0 ms == Unix epoch
        assert_eq!(epoch_ms_to_rfc3339(0), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn chat_prompt_covers_scene_and_memory_tasks() {
        let p = extract_memories_system_prompt(PromptMode::Chat);
        assert!(p.contains("SCENE SEGMENTATION"));
        assert!(p.contains("MEMORY EXTRACTION"));
        assert!(p.contains("persona"));
        assert!(p.contains("episodic"));
        assert!(p.contains("instruction"));
    }

    #[test]
    fn code_prompt_has_work_types() {
        let p = extract_memories_system_prompt(PromptMode::Code);
        assert!(p.contains("work_fact"));
        assert!(p.contains("work_task"));
        assert!(p.contains("work_artifact"));
    }

    #[test]
    fn prompt_separates_background_from_new() {
        let new = vec![msg("n1", L0Role::User, "fresh message", 1000)];
        let bg = vec![msg("b1", L0Role::User, "old message", 100)];
        let p = format_extraction_prompt(&new, &bg, Some("Previous scene"));
        assert!(p.contains("PREVIOUS SCENE: Previous scene"));
        assert!(p.contains("[b1] [user] [1970-01-01T00:00:00.100Z]: old message"));
        assert!(p.contains("[n1] [user] [1970-01-01T00:00:01.000Z]: fresh message"));
        // Background is explicitly context-only; new is the extraction target.
        assert!(p.contains("BACKGROUND CONVERSATION (context only"));
        assert!(p.contains("NEW MESSAGES TO EXTRACT FROM"));
    }

    #[test]
    fn previous_scene_none_and_no_background() {
        let new = vec![msg("n1", L0Role::User, "hello", 0)];
        let p = format_extraction_prompt(&new, &[], None);
        assert!(p.contains("PREVIOUS SCENE: none"));
        assert!(p.contains(
            "BACKGROUND CONVERSATION (context only — NEVER extract memories from here):\nnone"
        ));
    }
}
