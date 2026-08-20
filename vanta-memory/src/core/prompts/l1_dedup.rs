//! L1 conflict-detection prompts (MEM-11).
//!
//! Reimplemented in English from the TDAM conflict-detection principles
//! (`MemoryCore/src/core/prompts/l1-dedup.ts`) — single batch LLM call that
//! judges every new memory against its candidate pool and returns one
//! `DedupDecision` per new memory (`store|update|merge|skip`).
//!
//! Two families mirror [`crate::core::prompts::PromptMode`]: chat
//! (persona/episodic/instruction) and work/team (work_* types).

use crate::core::abstractions::{ExtractedMemory, MemoryRecord};
use crate::core::prompts::PromptMode;

/// One new memory plus its top candidate pool for conflict judgment.
///
/// `record_id` is the transient id assigned by the pipeline before the LLM
/// call; the LLM echoes it back so decisions map 1:1 to new memories.
/// `candidates` are existing persisted records recalled without LLM (phase 1).
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateMatch {
    pub record_id: String,
    pub memory: ExtractedMemory,
    pub candidates: Vec<MemoryRecord>,
}

/// Build the system prompt for an L1 conflict-detection run.
pub fn get_conflict_detection_system_prompt(mode: PromptMode) -> String {
    match mode {
        PromptMode::Chat => CHAT_SYSTEM_PROMPT.into(),
        PromptMode::Code => WORK_SYSTEM_PROMPT.into(),
    }
}

/// Build the user prompt: a deduplicated candidate pool (existing memories)
/// followed by the new memories to judge, each with the candidate ids it
/// should consider. One call judges the whole batch (phase 2).
pub fn format_batch_conflict_prompt(matches: &[CandidateMatch]) -> String {
    if matches.is_empty() {
        return "NEW MEMORIES TO JUDGE:\n[]".to_string();
    }

    // Unified candidate pool, deduped by record_id so the LLM reuses ids
    // across memories (TDAM l1-dedup.ts:158-194 builds the same pool).
    let mut pool: Vec<&MemoryRecord> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for m in matches {
        for c in &m.candidates {
            if !seen.contains(&c.id.as_str()) {
                seen.push(&c.id);
                pool.push(c);
            }
        }
    }

    let pool_json = serde_json::to_string_pretty(
        &pool
            .iter()
            .map(|c| {
                serde_json::json!({
                    "record_id": c.id,
                    "content": c.content,
                    "type": c.memory_type,
                    "priority": c.priority,
                    "scene_name": c.scene_name,
                    "timestamps": c.timestamps,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());

    let new_json = serde_json::to_string_pretty(
        &matches
            .iter()
            .map(|m| {
                serde_json::json!({
                    "record_id": m.record_id,
                    "content": m.memory.content,
                    "type": m.memory.memory_type,
                    "priority": m.memory.priority,
                    "scene_name": m.memory.scene_name,
                    "candidate_ids": m.candidates.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());

    format!(
        "CANDIDATE POOL (existing memories you may target):\n{pool_json}\n\n\
         ============================================================\n\n\
         NEW MEMORIES TO JUDGE:\n{new_json}"
    )
}

const CHAT_SYSTEM_PROMPT: &str = r#"You are an expert in long-term memory deduplication.
You receive a candidate pool of EXISTING memories and a list of NEW memories. For every new memory, decide whether to:
- "store" — append it as a new record (it does not conflict with anything).
- "update" — replace one or more existing records with the new memory (same fact, new details win).
- "merge" — fuse the new memory into one or more existing records; the merged record must keep all still-valid information from both sides.
- "skip" — the new memory is already fully covered by existing records (duplicate, lower-quality restatement).

Merging across types is allowed when the best single type for the fused record differs from the originals (e.g. a persona trait that becomes an episodic event). When several existing records are relevant, merge them all into ONE record via target_ids.

Priority rules: keep the highest priority that still makes sense after the merge; a strict global instruction (priority -1) always wins and can never be downgraded.

OUTPUT: Return ONLY a valid JSON array, one object per NEW memory, in the same order:
[
  {
    "record_id": "the new memory's record_id from the input",
    "action": "store|update|merge|skip",
    "target_ids": ["existing record ids to remove/replace"],
    "merged_content": "full merged content (update/merge only; omit for store/skip)",
    "merged_type": "best type after merge (omit if unchanged)",
    "merged_priority": 80,
    "merged_timestamps": ["union of all relevant timestamps, deduplicated and sorted"]
  }
]
Rules:
- Every new memory MUST appear exactly once; never drop one.
- "store" needs no target_ids; "update"/"merge" need at least one target_id from the candidate pool; "skip" needs no merged_* fields.
- For "store", only record_id and action are required.
- Output strictly the JSON array — no markdown code fences, no explanatory text."#;

const WORK_SYSTEM_PROMPT: &str = r#"You are an expert in long-term memory deduplication for an AI assistant embedded in a work/team environment.
You receive a candidate pool of EXISTING memories and a list of NEW memories. For every new memory, decide whether to:
- "store" — append it as a new record (it does not conflict with anything).
- "update" — replace one or more existing records with the new memory (same fact, new details win).
- "merge" — fuse the new memory into one or more existing records; the merged record must keep all still-valid information from both sides.
- "skip" — the new memory is already fully covered by existing records (duplicate, lower-quality restatement).

Merging across types is allowed when the best single type for the fused record differs from the originals (e.g. a work_task that becomes a work_method once finished). When several existing records are relevant, merge them all into ONE record via target_ids.

Priority rules: keep the highest priority that still makes sense after the merge; a strict global instruction (priority -1) always wins and can never be downgraded.

OUTPUT: Return ONLY a valid JSON array, one object per NEW memory, in the same order:
[
  {
    "record_id": "the new memory's record_id from the input",
    "action": "store|update|merge|skip",
    "target_ids": ["existing record ids to remove/replace"],
    "merged_content": "full merged content (update/merge only; omit for store/skip)",
    "merged_type": "best type after merge (omit if unchanged)",
    "merged_priority": 80,
    "merged_timestamps": ["union of all relevant timestamps, deduplicated and sorted"]
  }
]
Rules:
- Every new memory MUST appear exactly once; never drop one.
- "store" needs no target_ids; "update"/"merge" need at least one target_id from the candidate pool; "skip" needs no merged_* fields.
- For "store", only record_id and action are required.
- Output strictly the JSON array — no markdown code fences, no explanatory text."#;

#[cfg(test)]
mod tests {
    use crate::core::abstractions::{ExtractedMemory, MemoryRecord, MemoryType};

    use super::{
        format_batch_conflict_prompt, get_conflict_detection_system_prompt, CandidateMatch,
    };
    use crate::core::prompts::PromptMode;

    fn memory_record(id: &str, content: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "s".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
            created_at: "2026-08-20T10:00:00.000Z".into(),
            updated_at: "2026-08-20T10:00:00.000Z".into(),
            version: 1,
            session_key: "sk".into(),
            session_id: "".into(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
        }
    }

    fn extracted(content: &str) -> ExtractedMemory {
        ExtractedMemory {
            content: content.into(),
            memory_type: MemoryType::Episodic,
            priority: 70,
            source_message_ids: vec![],
            scene_name: "s".into(),
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn chat_prompt_has_all_four_actions() {
        let p = get_conflict_detection_system_prompt(PromptMode::Chat);
        assert!(p.contains("\"store\""));
        assert!(p.contains("\"update\""));
        assert!(p.contains("\"merge\""));
        assert!(p.contains("\"skip\""));
        assert!(p.contains("record_id"));
        assert!(p.contains("target_ids"));
    }

    #[test]
    fn work_prompt_mentions_work_types_context() {
        let p = get_conflict_detection_system_prompt(PromptMode::Code);
        assert!(p.contains("work/team environment"));
        assert!(p.contains("\"store\""));
    }

    #[test]
    fn empty_matches_yield_empty_judgment_list() {
        let p = format_batch_conflict_prompt(&[]);
        assert!(p.contains("NEW MEMORIES TO JUDGE:\n[]"));
    }

    #[test]
    fn pool_is_unified_and_deduped() {
        let matches = vec![
            CandidateMatch {
                record_id: "new_1".into(),
                memory: extracted("alpha"),
                candidates: vec![memory_record("m_1", "same")],
            },
            CandidateMatch {
                record_id: "new_2".into(),
                memory: extracted("beta"),
                candidates: vec![memory_record("m_1", "same"), memory_record("m_2", "other")],
            },
        ];
        let p = format_batch_conflict_prompt(&matches);
        // Pool contains m_1 exactly once (deduped across matches).
        assert_eq!(p.matches("\"record_id\": \"m_1\"").count(), 1);
        assert!(p.contains("\"record_id\": \"m_2\""));
        // Every new memory is present with its candidate_ids.
        assert!(p.contains("\"record_id\": \"new_1\""));
        assert!(p.contains("\"record_id\": \"new_2\""));
        assert!(p.contains("\"candidate_ids\""));
    }

    #[test]
    fn candidate_pool_carries_judgment_fields() {
        let matches = vec![CandidateMatch {
            record_id: "new_1".into(),
            memory: extracted("alpha"),
            candidates: vec![memory_record("m_1", "same")],
        }];
        let p = format_batch_conflict_prompt(&matches);
        assert!(p.contains("\"timestamps\""));
        assert!(p.contains("\"priority\""));
        assert!(p.contains("\"scene_name\""));
    }
}
