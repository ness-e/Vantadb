//! L1 two-phase dedup pipeline (MEM-11).
//!
//! Port of TDAM `core/record/l1-dedup.ts` (2 phases, no literal copy):
//! 1. **Phase 1 — candidate recall (LLM-free):** for each new memory, recall a
//!    top-k pool of existing records via keyword overlap (see `l1_reader`).
//! 2. **Phase 2 — batch LLM judgment:** ONE `LlmRunner` call judges every new
//!    memory against its candidate pool and returns one [`DedupDecision`] per
//!    memory (`store|update|merge|skip`).
//!
//! Degradation (Principio 4 — the LLM is optional, the pipeline never loses
//! data): no candidates → all `store`; runner fails → all `store`; tolerant
//! parse fails → all `store`.

use crate::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmRunParams, LlmRunner, MemoryRecord,
};
use crate::core::prompts::l1_dedup::{
    format_batch_conflict_prompt, get_conflict_detection_system_prompt, CandidateMatch,
};
use crate::core::prompts::PromptMode;
use crate::core::record::l1_reader::{read_session_records, recall_candidates};
use crate::core::record::l1_writer::{apply_dedup_batch, generate_memory_id, EmbedFn, L1Error};
use crate::offload::local_llm::parsers::json_utils::extract_json;
use crate::offload::local_llm::parsers::l1_parser::normalize_type;

/// LLM task id for the single conflict-detection call (stable for metrics).
pub const CONFLICT_DETECTION_TASK_ID: &str = "l1-conflict-detection";

/// A new memory with a transient id assigned before the LLM call.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMemory {
    pub record_id: String,
    pub memory: ExtractedMemory,
}

/// Config for the dedup pipeline.
#[derive(Clone)]
pub struct L1DedupConfig {
    /// How many candidate records to recall per new memory (phase 1).
    pub recall_top_k: usize,
    /// Prompt family: chat (persona/episodic/instruction) vs work types.
    pub prompt_mode: PromptMode,
    /// Optional embedding hook for L1 writes (MEM-46). `Some(hook)` enables
    /// the D38 dual-pool semantic ranking in recall/dedup/query (MEM-47);
    /// `None` keeps records vector-free — behavior identical to pre-MEM-46.
    /// MEM-63 auto-on: with the `embed-local` feature compiled, the default
    /// config wires `local_embedding_hook()` automatically; without the
    /// feature the default is `None` (callers opt in with
    /// [`Self::with_local_provider`] or [`Self::with_embed`]).
    pub embed: Option<EmbedFn>,
}

impl std::fmt::Debug for L1DedupConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("L1DedupConfig")
            .field("recall_top_k", &self.recall_top_k)
            .field("prompt_mode", &self.prompt_mode)
            .field("embed", &self.embed.is_some())
            .finish()
    }
}

impl Default for L1DedupConfig {
    /// MEM-63 auto-on: with the `embed-local` feature compiled and a working
    /// `LocalOnnxProvider` available (384-d fallback), the default config wires
    /// `local_embedding_hook()` so callers get the D38 dual-pool path without
    /// an explicit `.with_local_provider()` call. Without the feature the
    /// default keeps `embed: None` (keyword-only, no provider available) so the
    /// default build stays lean.
    fn default() -> Self {
        #[cfg(feature = "embed-local")]
        let embed = Some(crate::core::record::l1_writer::local_embedding_hook());
        #[cfg(not(feature = "embed-local"))]
        let embed = None;
        Self {
            recall_top_k: 5,
            prompt_mode: PromptMode::Chat,
            embed,
        }
    }
}

impl L1DedupConfig {
    /// Attach a local ONNX embedding hook (`LocalOnnxProvider`, 384-d) when
    /// `embed-local` is enabled; otherwise leaves the config unchanged
    /// (keyword-only recall). Replaces the legacy hash `dim=8` fallback
    /// (MEM-47 — now `dim >= 64`, here 384).
    pub fn with_local_provider(mut self) -> Self {
        #[cfg(feature = "embed-local")]
        {
            self.embed = Some(crate::core::record::l1_writer::local_embedding_hook());
        }
        #[cfg(not(feature = "embed-local"))]
        {
            // No local provider compiled — keep keyword-only path.
            let _ = &mut self;
        }
        self
    }

    /// Attach an explicit embedding hook (overwrites `with_local_provider`).
    pub fn with_embed(mut self, hook: EmbedFn) -> Self {
        self.embed = Some(hook);
        self
    }
}

/// Assign deterministic transient ids (`m_{now}_{idx}`) to raw memories.
pub fn prepare_pending(memories: &[ExtractedMemory], now_ms: u64) -> Vec<PendingMemory> {
    memories
        .iter()
        .enumerate()
        .map(|(idx, memory)| PendingMemory {
            record_id: generate_memory_id(now_ms, idx),
            memory: memory.clone(),
        })
        .collect()
}

/// Phase 1: recall top-k candidate pools per new memory from persisted
/// session records. Returns `(pending, matches)` where a match is only
/// produced when at least one candidate was found.
///
/// `embed` (MEM-47/D38): optional embedding hook — when present AND the pool
/// carries vectors, semantic similarity joins candidate ranking; records
/// without a vector keep the keyword-overlap gate.
pub fn recall_candidate_matches(
    pending: &[PendingMemory],
    existing: &[MemoryRecord],
    top_k: usize,
    embed: Option<&EmbedFn>,
) -> Vec<CandidateMatch> {
    pending
        .iter()
        .map(|p| CandidateMatch {
            record_id: p.record_id.clone(),
            memory: p.memory.clone(),
            candidates: recall_candidates(existing, p.memory.content.as_str(), top_k, embed),
        })
        .collect()
}

/// Two-phase dedup. Never fails and never drops memories: every input memory
/// yields exactly one decision. Runner failures degrade to `store` (Principio 4).
pub fn batch_dedup<R: LlmRunner>(
    runner: &R,
    pending: &[PendingMemory],
    existing: &[MemoryRecord],
    config: &L1DedupConfig,
) -> Vec<DedupDecision> {
    let matches = recall_candidate_matches(
        pending,
        existing,
        config.recall_top_k,
        config.embed.as_ref(),
    );
    let any_candidates = matches.iter().any(|m| !m.candidates.is_empty());

    if pending.is_empty() || !any_candidates {
        // Nothing to judge: store everything.
        return matches
            .iter()
            .map(|m| store_decision(&m.record_id))
            .collect();
    }

    let system_prompt = get_conflict_detection_system_prompt(config.prompt_mode);
    let prompt = format_batch_conflict_prompt(&matches);

    let params = LlmRunParams {
        prompt,
        system_prompt: Some(system_prompt),
        task_id: CONFLICT_DETECTION_TASK_ID.to_string(),
        timeout: None,
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };

    match runner.run(&params) {
        Ok(raw) => parse_batch_result(&raw, &matches),
        Err(err) => {
            tracing::warn!(error = %err, "l1 conflict-detection LLM call failed; storing all memories");
            matches
                .iter()
                .map(|m| store_decision(&m.record_id))
                .collect()
        }
    }
}

/// Tolerant parse of the LLM decision array (deuda MEM-10: uses
/// `json_utils::extract_json` + `normalize_type`, never strict deserialize).
/// Every expected record_id yields exactly one decision — missing or
/// malformed entries fall back to `store`.
pub fn parse_batch_result(raw: &str, matches: &[CandidateMatch]) -> Vec<DedupDecision> {
    let parsed: Option<Vec<serde_json::Value>> = extract_json(raw);
    let mut by_id: std::collections::HashMap<String, DedupDecision> =
        std::collections::HashMap::new();
    if let Some(items) = parsed {
        for item in items {
            if let Some(decision) = decision_from_value(&item) {
                by_id.insert(decision.record_id.clone(), decision);
            }
        }
    }

    matches
        .iter()
        .map(|m| {
            by_id
                .get(&m.record_id)
                .cloned()
                .unwrap_or_else(|| store_decision(&m.record_id))
        })
        .collect()
}

/// Pipeline entry point: read session records → recall → batch dedup → write.
pub fn run_l1_dedup<R: LlmRunner>(
    db: &vantadb::sdk::VantaEmbedded,
    runner: &R,
    session_key: &str,
    session_id: &str,
    memories: &[ExtractedMemory],
    config: &L1DedupConfig,
) -> Result<Vec<MemoryRecord>, L1Error> {
    let now = crate::core::conversation::now_ms();
    let pending = prepare_pending(memories, now);
    let existing = read_session_records(db, session_key)?;
    let decisions = batch_dedup(runner, &pending, &existing, config);
    let raw_memories: Vec<ExtractedMemory> = pending.iter().map(|p| p.memory.clone()).collect();
    apply_dedup_batch(
        db,
        session_key,
        session_id,
        &raw_memories,
        &decisions,
        now,
        config.embed.as_ref(),
    )
}

/// A `store` decision for a record id (safe default everywhere).
fn store_decision(record_id: &str) -> DedupDecision {
    DedupDecision {
        record_id: record_id.to_string(),
        action: DedupAction::Store,
        target_ids: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    }
}

/// Build a [`DedupDecision`] from a raw JSON object, tolerantly. Invalid
/// actions become `store`; invalid `merged_type` becomes `None` (keeps the
/// original type — never fails the batch).
fn decision_from_value(v: &serde_json::Value) -> Option<DedupDecision> {
    let obj = v.as_object()?;
    let record_id = obj.get("record_id")?.as_str()?.trim().to_string();
    if record_id.is_empty() {
        return None;
    }

    let action = match obj.get("action").and_then(serde_json::Value::as_str) {
        Some("store") => DedupAction::Store,
        Some("update") => DedupAction::Update,
        Some("merge") => DedupAction::Merge,
        Some("skip") => DedupAction::Skip,
        _ => DedupAction::Store,
    };

    let target_ids = string_array(obj.get("target_ids"));
    let merged_content = obj
        .get("merged_content")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let merged_type = obj
        .get("merged_type")
        .and_then(serde_json::Value::as_str)
        .and_then(normalize_type);
    let merged_priority = obj
        .get("merged_priority")
        .and_then(serde_json::Value::as_i64)
        .map(|v| v as i32);
    let merged_timestamps = obj
        .get("merged_timestamps")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        });

    Some(DedupDecision {
        record_id,
        action,
        target_ids,
        merged_content,
        merged_type,
        merged_priority,
        merged_timestamps,
    })
}

fn string_array(v: Option<&serde_json::Value>) -> Vec<String> {
    v.and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        batch_dedup, parse_batch_result, prepare_pending, recall_candidate_matches, L1DedupConfig,
    };
    use crate::core::abstractions::{
        DedupAction, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryRecord, MemoryType,
    };
    use std::sync::{Arc, Mutex};

    fn memory(content: &str) -> ExtractedMemory {
        ExtractedMemory {
            content: content.into(),
            memory_type: MemoryType::Episodic,
            priority: 70,
            source_message_ids: vec![],
            scene_name: "s".into(),
            metadata: serde_json::Value::Null,
        }
    }

    fn record(id: &str, content: &str) -> MemoryRecord {
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
            vector: None,
            heat: 0,
            superseded_by: None,
        }
    }

    #[derive(Default)]
    struct ScriptedRunner {
        calls: Arc<Mutex<usize>>,
    }

    impl LlmRunner for ScriptedRunner {
        fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
            *self.calls.lock().unwrap() += 1;
            Ok("[]".to_string())
        }
    }

    #[test]
    fn prepare_pending_assigns_deterministic_ids() {
        let pending = prepare_pending(&[memory("a"), memory("b")], 1_700_000_000_000);
        assert_eq!(pending[0].record_id, "m_1700000000000_0");
        assert_eq!(pending[1].record_id, "m_1700000000000_1");
    }

    #[test]
    fn recall_matches_only_with_candidates() {
        let pending = prepare_pending(&[memory("user prefers dark mode")], 0);
        let existing = vec![record("m1", "user prefers dark mode")];
        let matches = recall_candidate_matches(&pending, &existing, 5, None);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].candidates.len(), 1);

        let no_existing: Vec<MemoryRecord> = vec![];
        let empty = recall_candidate_matches(&pending, &no_existing, 5, None);
        assert_eq!(empty[0].candidates.len(), 0);
    }

    #[test]
    fn empty_or_no_candidates_store_all() {
        let runner = ScriptedRunner::default();
        let pending = prepare_pending(&[memory("a")], 0);
        let config = L1DedupConfig::default();

        // No existing records → no LLM call, all store.
        let decisions = batch_dedup(&runner, &pending, &[], &config);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].action, DedupAction::Store);
        assert_eq!(*runner.calls.lock().unwrap(), 0);
    }

    #[test]
    fn runner_failure_stores_all() {
        struct Failing;
        impl LlmRunner for Failing {
            fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
                Err(LlmError::Other("boom".into()))
            }
        }
        let pending = prepare_pending(&[memory("user prefers dark mode")], 0);
        let existing = vec![record("m1", "user prefers dark mode")];
        let decisions = batch_dedup(&Failing, &pending, &existing, &L1DedupConfig::default());
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].action, DedupAction::Store);
    }

    #[test]
    fn parse_tolerates_markdown_and_missing_entries() {
        let matches = vec![crate::core::prompts::CandidateMatch {
            record_id: "m_0".into(),
            memory: memory("a"),
            candidates: vec![record("m1", "x")],
        }];
        let raw = "```json\n[{\"record_id\": \"m_0\", \"action\": \"skip\"}]\n```";
        let decisions = parse_batch_result(raw, &matches);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].action, DedupAction::Skip);

        // Missing entry → store fallback.
        let missing = parse_batch_result("[]", &matches);
        assert_eq!(missing[0].action, DedupAction::Store);
    }

    #[test]
    fn parse_invalid_action_and_type_degrade_gracefully() {
        let matches = vec![crate::core::prompts::CandidateMatch {
            record_id: "m_0".into(),
            memory: memory("a"),
            candidates: vec![],
        }];
        let raw = r#"[{"record_id": "m_0", "action": "explode", "merged_type": "not_a_type"}]"#;
        let decisions = parse_batch_result(raw, &matches);
        assert_eq!(decisions[0].action, DedupAction::Store);
        assert_eq!(decisions[0].merged_type, None);
    }

    #[test]
    fn parse_valid_merge_decision() {
        let matches = vec![crate::core::prompts::CandidateMatch {
            record_id: "m_0".into(),
            memory: memory("a"),
            candidates: vec![],
        }];
        let raw = r#"[{"record_id": "m_0", "action": "merge", "target_ids": ["m1"], "merged_content": "merged", "merged_type": "work_method", "merged_priority": 90, "merged_timestamps": ["2026-08-20T10:00:00.000Z"]}]"#;
        let decisions = parse_batch_result(raw, &matches);
        assert_eq!(decisions[0].action, DedupAction::Merge);
        assert_eq!(decisions[0].target_ids, vec!["m1".to_string()]);
        assert_eq!(decisions[0].merged_type, Some(MemoryType::WorkMethod));
        assert_eq!(decisions[0].merged_priority, Some(90));
        assert_eq!(decisions[0].merged_content.as_deref(), Some("merged"));
    }

    #[cfg(feature = "embed-local")]
    #[test]
    fn with_local_provider_wires_384d_dummy_vectors() {
        // EMB-04: with_local_provider must wire LocalOnnxProvider (384d) even without 691MB model
        let config = L1DedupConfig::default().with_local_provider();
        assert!(config.embed.is_some(), "embed-local must wire a hook");
        let hook = config.embed.unwrap();
        let v = hook("hola mundo").expect("hook must produce vector");
        assert_eq!(
            v.len(),
            384,
            "multilingual-e5-small dim 384 (MEM-47 dim>=64)"
        );
        let w = hook("hello world").expect("hook must produce vector");
        assert_eq!(w.len(), 384);
        // deterministic dummy keeps multi >0.60 as EMB-02 contract
        let cos = {
            let dot: f32 = v.iter().zip(w.iter()).map(|(a, b)| a * b).sum();
            let norm_v: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_w: f32 = w.iter().map(|x| x * x).sum::<f32>().sqrt();
            dot / (norm_v * norm_w)
        };
        assert!(
            cos > 0.60,
            "multilingual dummy cosine must be >0.60, got {cos}"
        );
        let cos_self: f32 = v.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        // self cosine should be ~1.0 (L2 normalized)
        assert!(cos_self > 0.99, "self cosine >0.99, got {cos_self}");
    }

    #[cfg(not(feature = "embed-local"))]
    #[test]
    fn with_local_provider_without_feature_stays_keyword_only() {
        let config = L1DedupConfig::default().with_local_provider();
        assert!(config.embed.is_none(), "without embed-local, no hook wired");
    }

    // MEM-63 auto-on: with_local_provider stays explicit; default() now
    // wires the hook itself when the feature is compiled.

    #[cfg(feature = "embed-local")]
    #[test]
    fn default_wires_local_provider_when_feature_on() {
        let config = L1DedupConfig::default();
        assert!(
            config.embed.is_some(),
            "MEM-63 auto-on: default must wire local_embedding_hook() when embed-local is compiled"
        );
        let hook = config.embed.unwrap();
        let v = hook("hola mundo").expect("auto-on hook must produce vector");
        assert_eq!(v.len(), 384, "multilingual-e5-small dim 384");
    }

    #[cfg(not(feature = "embed-local"))]
    #[test]
    fn default_stays_keyword_only_without_feature() {
        let config = L1DedupConfig::default();
        assert!(
            config.embed.is_none(),
            "without embed-local, default must stay keyword-only"
        );
    }
}
