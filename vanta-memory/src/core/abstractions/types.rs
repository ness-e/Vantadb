//! Data contracts for the memory pipeline (L0 → L3).
//!
//! Field names use the **snake_case JSON wire contract** that the L1/L2/L3
//! LLM prompts emit (e.g. `record_id`, `target_ids`, `scene_name` — see TDAM
//! `l1-dedup.ts:parseBatchResult` and `l1-extractor.ts`). Persisting these
//! records goes through the VantaDB store (nodes + metadata), never a foreign
//! JSONL format.
//!
//! Source of truth for the shapes (TDAM clone, reference only):
//! - `MemoryCore/src/core/record/l1-writer.ts` — `MemoryRecord`,
//!   `ExtractedMemory`, `MemoryType`, `DedupDecision`
//! - `MemoryCore/src/core/record/l1-extractor.ts` — `L1ExtractionResult`,
//!   `SceneSegment`
//! - `docs/research/tdam/02-scene-persona.md` §33, §41, §52-53 — scene META,
//!   persona modes/triggers.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// L1 memory types: chat-mode legacy types + code/work-mode team types.
///
/// Source: TDAM `l1-writer.ts:31-38` (7 types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// Stable traits/preferences of the user.
    Persona,
    /// One-off event the user lived through.
    Episodic,
    /// Instructions the user wants followed (global if priority = -1).
    Instruction,
    /// Fact about the user's work/team.
    WorkFact,
    /// An open/completed work task.
    WorkTask,
    /// A method or procedure used at work.
    WorkMethod,
    /// A work artifact (file, doc, code).
    WorkArtifact,
}

/// A memory as extracted by the LLM, **before** dedup / persistence.
///
/// Matches the output of the L1 extraction prompt (TDAM `l1-writer.ts:104-112`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtractedMemory {
    /// Memory content text.
    pub content: String,
    /// Memory type.
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// Priority score 0-100 (higher = more important); -1 = strict global
    /// instruction.
    pub priority: i32,
    /// Source message IDs that contributed to this memory.
    pub source_message_ids: Vec<String>,
    /// Scene name this memory was extracted in (L2 grouping key).
    pub scene_name: String,
    /// Type-specific metadata (e.g. `activity_start_time` for episodic).
    #[serde(default)]
    pub metadata: Value,
}

/// A persisted memory record (L1).
///
/// Source: TDAM `l1-writer.ts:55-98`. VantaDB persists these as store nodes;
/// this struct is the in-memory / wire contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryRecord {
    /// Unique ID for dedup updates.
    pub id: String,
    /// Memory content.
    pub content: String,
    /// Memory type.
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// Priority score 0-100, -1 = strict global instruction.
    pub priority: i32,
    /// Scene name this memory belongs to.
    pub scene_name: String,
    /// Source message IDs that contributed to this memory.
    pub source_message_ids: Vec<String>,
    /// Type-specific metadata (free-form object).
    #[serde(default)]
    pub metadata: Value,
    /// Timestamp trail: all timestamps related to this memory (merge history).
    #[serde(default)]
    pub timestamps: Vec<String>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last update timestamp (ISO 8601).
    pub updated_at: String,
    /// Monotonic version: new memories start at 1; update/merge increments.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Source session key (conversation channel identifier).
    pub session_key: String,
    /// Source session ID (single conversation instance identifier).
    #[serde(default)]
    pub session_id: String,
    /// Optional task dimension for L0/L1 filtering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Tenancy isolation (three-dim); optional pre-isolation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// Tenancy isolation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Tenancy isolation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

fn default_version() -> u32 {
    1
}

/// Dedup action decided by the L1 conflict-detection LLM call.
///
/// Source: TDAM `l1-writer.ts:114`, `l1-dedup.ts:350` (`store|update|merge|skip`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DedupAction {
    /// Append a new record.
    Store,
    /// Remove target records + append the updated record.
    Update,
    /// Remove target records + append the merged record.
    Merge,
    /// Do nothing.
    Skip,
}

/// Batch dedup decision — one per new memory (L1).
///
/// Source: TDAM `l1-writer.ts:123-137` + `l1-dedup.ts:368-376` parse shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DedupDecision {
    /// Which new memory this decision is about.
    pub record_id: String,
    /// Action to apply.
    pub action: DedupAction,
    /// IDs of existing records to replace/remove (update/merge).
    #[serde(default)]
    pub target_ids: Vec<String>,
    /// Merged/updated content text (update/merge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_content: Option<String>,
    /// Best type after merge (update/merge; may differ from original).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_type: Option<MemoryType>,
    /// Priority after merge (update/merge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_priority: Option<i32>,
    /// Union of all related timestamps (update/merge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_timestamps: Option<Vec<String>>,
}

/// Result of running L1 extraction on a conversation batch.
///
/// Source: TDAM `l1-extractor.ts:64-77`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct L1ExtractionResult {
    /// Whether extraction succeeded.
    pub success: bool,
    /// Number of memories extracted.
    pub extracted_count: usize,
    /// Number of memories actually stored (after dedup).
    pub stored_count: usize,
    /// The memory records that were stored.
    #[serde(default)]
    pub records: Vec<MemoryRecord>,
    /// Scene names detected during extraction.
    #[serde(default)]
    pub scene_names: Vec<String>,
    /// Last scene name (for continuity in next extraction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_scene_name: Option<String>,
}

/// A scene segment produced by L1 extraction: groups a set of messages and
/// their memories under one scene name (L2 grouping input).
///
/// Source: `docs/research/tdam/02-scene-persona.md` §33 (TDAM
/// `l1-extractor.ts:52-62`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneSegment {
    /// Scene name this segment belongs to.
    pub scene_name: String,
    /// Message IDs belonging to this segment.
    pub message_ids: Vec<String>,
    /// Memories extracted for this segment.
    #[serde(default)]
    pub memories: Vec<ExtractedMemory>,
}

/// The L2 scene META contract — stable anchor for the scene node in the core
/// graph (LLM-free L2 fallback).
///
/// Source: `docs/research/tdam/02-scene-persona.md` §52 (TDAM
/// `scene-format.ts:18-48`, `-----META-START-----` block).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneMeta {
    /// Creation timestamp (ISO 8601).
    pub created: String,
    /// Last update timestamp (ISO 8601).
    pub updated: String,
    /// Narrative summary of the scene.
    pub summary: String,
    /// Heat score (CREATE=1, UPDATE=old+1, MERGE=sum+1).
    pub heat: u32,
}

/// One entry of the scene index (`scene_index.json` equivalent).
///
/// Source: `docs/research/tdam/02-scene-persona.md` §53 (TDAM
/// `scene-index.ts:9-15`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneIndexEntry {
    /// Scene block filename.
    pub filename: String,
    /// Summary of the scene.
    pub summary: String,
    /// Heat score (descending navigation order).
    pub heat: u32,
    /// Creation timestamp (ISO 8601).
    pub created: String,
    /// Last update timestamp (ISO 8601).
    pub updated: String,
}

/// L3 persona generation modes.
///
/// Source: `docs/research/tdam/02-scene-persona.md` §26 (modos
/// first/incremental; TDAM `persona-generator.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaMode {
    /// Generate the persona from scratch (no prior persona exists).
    First,
    /// Regenerate only from changed scenes since the last generation.
    Incremental,
}

/// L3 persona trigger priorities (highest wins).
///
/// Source: `docs/research/tdam/02-scene-persona.md` §41 (TDAM
/// `persona-trigger.ts:35-96`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaTriggerPriority {
    /// Explicit request (LLM signal `[PERSONA_UPDATE_REQUEST]`).
    P1Request,
    /// Cold start (no persona yet, session active).
    P2ColdStart,
    /// Recovery (persona body is empty).
    P2Recovery,
    /// First scene completed.
    P3FirstScene,
    /// `memories_since_last_persona >= trigger_every_n` (default 50).
    P4MemoryCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_record_roundtrip_snake_case() {
        let record = MemoryRecord {
            id: "m_1".into(),
            content: "user prefers dark mode".into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "ui-setup".into(),
            source_message_ids: vec!["msg_1".into()],
            metadata: serde_json::json!({"activity_start_time": "2026-08-20T10:00:00Z"}),
            timestamps: vec!["2026-08-20T10:00:00Z".into()],
            created_at: "2026-08-20T10:00:00Z".into(),
            updated_at: "2026-08-20T10:00:00Z".into(),
            version: 2,
            session_key: "sk".into(),
            session_id: "si".into(),
            task_id: None,
            team_id: Some("team_a".into()),
            user_id: None,
            agent_id: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        // Wire contract is snake_case + `type` (same alias as the LLM wire) +
        // optional isolation fields omitted.
        assert!(json.contains("\"type\":\"persona\""));
        assert!(json.contains("\"scene_name\":\"ui-setup\""));
        assert!(json.contains("\"source_message_ids\":[\"msg_1\"]"));
        assert!(json.contains("\"team_id\":\"team_a\""));
        assert!(!json.contains("\"user_id\""));
        let back: MemoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn extracted_memory_parses_llm_wire_type_alias() {
        // The LLM emits "type" (not "memory_type") per the extraction prompt.
        let json = r#"{
            "content": "deploy with blue-green",
            "type": "work_method",
            "priority": 60,
            "source_message_ids": ["m1"],
            "scene_name": "deploy-runbook",
            "metadata": {}
        }"#;
        let mem: ExtractedMemory = serde_json::from_str(json).unwrap();
        assert_eq!(mem.memory_type, MemoryType::WorkMethod);
        assert_eq!(mem.scene_name, "deploy-runbook");
    }

    #[test]
    fn dedup_decision_roundtrip_with_merge_fields() {
        let decision = DedupDecision {
            record_id: "m_2".into(),
            action: DedupAction::Merge,
            target_ids: vec!["m_old".into()],
            merged_content: Some("merged text".into()),
            merged_type: Some(MemoryType::Episodic),
            merged_priority: Some(70),
            merged_timestamps: Some(vec!["2026-08-20T10:00:00Z".into()]),
        };
        let json = serde_json::to_string(&decision).unwrap();
        assert!(json.contains("\"action\":\"merge\""));
        assert!(json.contains("\"target_ids\":[\"m_old\"]"));
        let back: DedupDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back, decision);
    }

    #[test]
    fn scene_segment_roundtrip() {
        let segment = SceneSegment {
            scene_name: "deploy".into(),
            message_ids: vec!["m1".into(), "m2".into()],
            memories: vec![ExtractedMemory {
                content: "use feature flags".into(),
                memory_type: MemoryType::Instruction,
                priority: -1,
                source_message_ids: vec!["m1".into()],
                scene_name: "deploy".into(),
                metadata: serde_json::Value::Null,
            }],
        };
        let json = serde_json::to_string(&segment).unwrap();
        let back: SceneSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(back, segment);
    }

    #[test]
    fn persona_trigger_order() {
        assert!(PersonaTriggerPriority::P1Request < PersonaTriggerPriority::P2ColdStart);
        assert!(PersonaTriggerPriority::P2ColdStart < PersonaTriggerPriority::P2Recovery);
        assert!(PersonaTriggerPriority::P2Recovery < PersonaTriggerPriority::P3FirstScene);
        assert!(PersonaTriggerPriority::P3FirstScene < PersonaTriggerPriority::P4MemoryCount);
    }
}
