//! L1 batch extract+dedup in one LLM call (MEM-69).
//!
//! Fuses the MEM-10 extraction slice (`split_messages` + one `l1-extraction`
//! call) with the MEM-11 dedup judgment (`batch_dedup` semantics) into a
//! single `task_id: "l1-extract-dedup"` round-trip: −40/50% tokens per flush
//! (one system prompt instead of two).
//!
//! Reuses the exact tolerances of both paths instead of a second parser:
//! [`memory_from_value`](crate::offload::local_llm::parsers::l1_parser) for
//! memories, [`decision_from_value`](crate::core::record::l1_dedup) for the
//! inline `dedup` judgment. Degradation (Principio 4): runner failure →
//! `success: false`, empty; missing/malformed `dedup` → `store` (never drops
//! a memory). Opt-in primitive: `pipeline_worker` is untouched (wiring is a
//! follow-up slice).

use std::time::Duration;

use serde_json::Value;

use crate::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, L1ExtractionResult, LlmRunParams, LlmRunner,
    MemoryRecord,
};
use crate::core::conversation::L0Message;
use crate::core::prompts::l1_extraction::{
    extract_memories_system_prompt, format_extraction_prompt,
};
use crate::core::record::l1_dedup::{decision_from_value, prepare_pending, L1DedupConfig};
use crate::core::record::l1_extractor::{should_extract_l1, split_messages, L1ExtractorConfig};
use crate::core::record::l1_reader::recall_candidates;
use crate::core::record::PendingMemory;
use crate::offload::local_llm::parsers::json_utils::extract_json;
use crate::offload::local_llm::parsers::l1_parser::memory_from_value;

/// LLM task id for the fused extract+dedup call (stable for metrics).
pub const EXTRACT_DEDUP_TASK_ID: &str = "l1-extract-dedup";

/// LLM timeout for the single batch call (same as extraction).
const LLM_TIMEOUT: Duration = Duration::from_secs(180);

/// Scene name used when the LLM omits it (mirrors `l1_parser`).
const DEFAULT_SCENE: &str = "unknown-scene";

/// Extract + judge dedup in ONE LLM call.
///
/// Returns `(result, pending, decisions)` mirroring [`extract_l1_segments`](crate::core::record::l1_extractor::extract_l1_segments)
/// (`records` empty, `stored_count` 0 — writing lives in the caller) plus one
/// [`DedupDecision`] per memory (alignment by construction). `now_ms` is a
/// parameter so transient ids stay deterministic in tests.
pub fn extract_dedup_batch<R: LlmRunner>(
    runner: &R,
    messages: &[L0Message],
    existing: &[MemoryRecord],
    previous_scene_name: Option<&str>,
    extractor_config: &L1ExtractorConfig,
    dedup_config: &L1DedupConfig,
    now_ms: u64,
) -> (L1ExtractionResult, Vec<PendingMemory>, Vec<DedupDecision>) {
    let qualified: Vec<L0Message> = messages
        .iter()
        .filter(|m| should_extract_l1(&m.content))
        .cloned()
        .collect();
    if qualified.is_empty() {
        return (empty_result(), Vec::new(), Vec::new());
    }

    let (new_messages, background_messages) = split_messages(
        &qualified,
        extractor_config.max_new_messages,
        extractor_config.max_background_messages,
    );

    // LLM-free candidate pool, unioned by id over the NEW messages (bounded
    // by top_k per message — same gate as phase 1 of `batch_dedup`).
    let pool = recall_pool(
        new_messages,
        existing,
        dedup_config.recall_top_k,
        dedup_config.embed.as_ref(),
    );

    let params = LlmRunParams {
        prompt: format_batch_prompt(
            new_messages,
            background_messages,
            previous_scene_name,
            &pool,
        ),
        system_prompt: Some(batch_system_prompt(extractor_config)),
        task_id: EXTRACT_DEDUP_TASK_ID.to_string(),
        timeout: Some(LLM_TIMEOUT),
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };

    let raw = match runner.run(&params) {
        Ok(raw) => raw,
        Err(err) => {
            tracing::warn!(error = %err, "L1 batch extract+dedup LLM call failed; degrading to empty result");
            return (failed_result(), Vec::new(), Vec::new());
        }
    };

    let (scene_names, parsed) = parse_batch_response(&raw);
    let extracted_count = parsed.len().min(extractor_config.max_memories_per_session);

    let mut memories: Vec<ExtractedMemory> = Vec::with_capacity(extracted_count);
    let mut judgments: Vec<Option<Value>> = Vec::with_capacity(extracted_count);
    for (memory, judgment) in parsed.into_iter().take(extracted_count) {
        memories.push(memory);
        judgments.push(judgment);
    }

    let pending = prepare_pending(&memories, now_ms);
    let decisions = pending
        .iter()
        .zip(judgments.iter())
        .map(|(p, raw)| decision_for(&p.record_id, raw.as_ref()))
        .collect();

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
        pending,
        decisions,
    )
}

/// LLM-free recall pool for the batch prompt: top-k per NEW message,
/// unioned by record id (first occurrence wins).
fn recall_pool(
    new_messages: &[L0Message],
    existing: &[MemoryRecord],
    top_k: usize,
    embed: Option<&crate::core::record::l1_writer::EmbedFn>,
) -> Vec<MemoryRecord> {
    let mut pool: Vec<MemoryRecord> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for m in new_messages {
        for candidate in recall_candidates(existing, m.content.as_str(), top_k, embed) {
            if !seen.contains(&candidate.id) {
                seen.push(candidate.id.clone());
                pool.push(candidate);
            }
        }
    }
    pool
}

/// Combined user prompt: extraction section (shared formatter) + existing
/// pool + inline-judgment instructions.
fn format_batch_prompt(
    new_messages: &[L0Message],
    background_messages: &[L0Message],
    previous_scene_name: Option<&str>,
    pool: &[MemoryRecord],
) -> String {
    let extraction =
        format_extraction_prompt(new_messages, background_messages, previous_scene_name);
    format!(
        "{extraction}\n\n\
         ============================================================\n\n\
         EXISTING MEMORIES (dedup candidates — reference by record_id):\n\
         {pool_json}\n\n\
         ============================================================\n\n\
         TASK — DEDUP JUDGMENT (same call, no second round-trip):\n\
         For each extracted memory add a \"dedup\" object:\n\
         {{\"action\": \"store|update|merge|skip\", \"target_ids\": [...], \
         \"merged_content\": \"...\", \"merged_type\": \"...\", \
         \"merged_priority\": 80, \"merged_timestamps\": [...]}}:\n\
         - \"store\" — new, conflicts with nothing (action only).\n\
         - \"skip\" — fully covered by existing record_ids in \"target_ids\".\n\
         - \"update\"/\"merge\" — fuse with \"target_ids\" (must reference \
         EXISTING MEMORIES above); include the merged_* fields.\n\
         - Every memory MUST carry exactly one \"dedup\" object; a missing or \
         malformed one is stored as-is.",
        pool_json = format_pool(pool),
    )
}

/// System prompt: extraction family + dedup-judgment addendum.
fn batch_system_prompt(config: &L1ExtractorConfig) -> String {
    format!(
        "{}\n\nADDITIONAL TASK — DEDUP JUDGMENT:\n\
         Every extracted memory carries a \"dedup\" judgment \
         (store|update|merge|skip + target_ids referencing the EXISTING \
         MEMORIES list). Judge against those candidates only.",
        extract_memories_system_prompt(config.prompt_mode),
    )
}

fn format_pool(pool: &[MemoryRecord]) -> String {
    if pool.is_empty() {
        return "none".to_string();
    }
    let items: Vec<Value> = pool
        .iter()
        .map(|r| {
            serde_json::json!({
                "record_id": r.id,
                "content": r.content,
                "type": r.memory_type,
                "priority": r.priority,
                "scene_name": r.scene_name,
            })
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
}

/// Parse the batch response into `(scene_names, [(memory, raw_dedup)])`.
/// Memories use the exact extraction tolerance; `dedup` travels inline per
/// memory so alignment holds by construction (never zip-by-index).
fn parse_batch_response(raw: &str) -> (Vec<String>, Vec<(ExtractedMemory, Option<Value>)>) {
    let items: Option<Vec<Value>> = extract_json(raw);
    let mut scene_names: Vec<String> = Vec::new();
    let mut parsed: Vec<(ExtractedMemory, Option<Value>)> = Vec::new();
    for item in items.unwrap_or_default() {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let scene_name = obj
            .get("scene_name")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_SCENE)
            .to_string();
        scene_names.push(scene_name.clone());
        let memories = obj
            .get("memories")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for m in &memories {
            if let Some(memory) = memory_from_value(m, &scene_name) {
                let judgment = m.as_object().and_then(|o| o.get("dedup")).cloned();
                parsed.push((memory, judgment));
            }
        }
    }
    (scene_names, parsed)
}

/// Build the decision for one memory: inject the transient id into the raw
/// judgment and reuse the exact dedup tolerance; anything missing or
/// malformed degrades to `store` (safe default — the memory is kept).
fn decision_for(record_id: &str, raw: Option<&Value>) -> DedupDecision {
    match raw.and_then(Value::as_object) {
        Some(obj) => {
            let mut with_id = obj.clone();
            with_id.insert(
                "record_id".to_string(),
                Value::String(record_id.to_string()),
            );
            decision_from_value(&Value::Object(with_id))
                .unwrap_or_else(|| store_decision(record_id))
        }
        None => store_decision(record_id),
    }
}

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
