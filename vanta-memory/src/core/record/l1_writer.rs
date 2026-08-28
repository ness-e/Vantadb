//! L1 memory writer — applies dedup decisions to the VantaDB store (MEM-11).
//!
//! Phase 2 persistence: each [`DedupDecision`] maps to a store mutation:
//! - `store` → `put` a brand-new record (id = decision.record_id or generated).
//! - `update`/`merge` → `delete` target records + `put` the merged/updated
//!   record with `version = max(targets)+1` and merged_* fields.
//! - `skip` → no-op.
//!
//! Persistence goes through the VantaDB SDK (Principio 2) — namespace
//! `l1/<session>`, key = record id, payload = serialized [`MemoryRecord`].

use std::collections::BTreeSet;
use std::sync::Arc;

use thiserror::Error;

use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue};

use crate::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, MemoryRecord, MemoryType,
};
use crate::core::conversation::sanitize_key;
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::record::l1_reader::{l1_namespace, read_record};

/// Errors surfaced by the L1 writer/reader surface. One error type for the
/// whole L1 layer so callers depend on a single contract.
#[derive(Debug, Error)]
pub enum L1Error {
    #[error("vantadb: {0}")]
    Vanta(#[from] VantaError),
    #[error("malformed l1 record payload: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Generate a deterministic, collision-safe record id: `m_{now_ms}_{idx}`.
pub fn generate_memory_id(now_ms: u64, idx: usize) -> String {
    format!("m_{now_ms}_{idx}")
}

/// Best-effort embedding hook for L1 writes (MEM-46).
///
/// Maps record content to a dense vector. Returning `None` (provider failure,
/// empty result) must never block the write — the record is stored without a
/// vector instead (P4). `None` as the hook itself means embeddings are
/// disabled, which is the default.
pub type EmbedFn = Arc<dyn Fn(&str) -> Option<Vec<f32>> + Send + Sync>;

/// Build an [`EmbedFn`] from the core embedding provider factory
/// (`vantadb::llm::get_embedding_provider`, selected by env
/// `VANTA_EMBEDDING_PROVIDER`: `openai` | `ollama` default).
///
/// Requires the `embeddings` feature passthrough (`vantadb/remote-inference`);
/// host code decides whether to attach it to [`crate::core::record::L1DedupConfig`].
#[cfg(feature = "embeddings")]
pub fn core_embedding_hook() -> EmbedFn {
    let provider = vantadb::llm::get_embedding_provider();
    Arc::new(move |text: &str| provider.embed(text).ok())
}

/// Build an [`EmbedFn`] from the local ONNX provider (`embed-local`).
///
/// Uses `LocalOnnxProvider` (ort+tokenizers, dim 384 for
/// `multilingual-e5-small`) with deterministic 384-d fallback when the
/// 691 MB model is not downloaded — keeps CI green. Vectors are
/// L2-normalized and satisfy MEM-47 `dim >= 64`.
/// Host code attaches it via `L1DedupConfig::with_local_provider()`.
#[cfg(feature = "embed-local")]
pub fn local_embedding_hook() -> EmbedFn {
    // Reuse the core factory which already selects `LocalOnnxProvider` when
    // `embed-local` is enabled (env `VANTA_LOCAL_MODEL` or default path).
    // Wrapped in Arc so the hook is `Send+Sync + 'static`.
    let provider = std::sync::Arc::new(vantadb::llm::get_embedding_provider());
    Arc::new(move |text: &str| provider.embed(text).ok())
}

/// Fallback for `L1DedupConfig::with_local_provider()` when `embed-local`
/// is not compiled — leaves embeddings disabled (keyword-only recall) so
/// `cargo check` without the feature still passes. With `embed-local`,
/// the provider yields 384-d vectors (MEM-47 dim>=64).
#[cfg(not(feature = "embed-local"))]
pub fn local_embedding_hook() -> EmbedFn {
    Arc::new(|_: &str| None)
}

/// Best-effort embed: a hook failure logs a warning and yields `None` so the
/// write proceeds without a vector (P4 — never blocks, never loses data).
fn embed_vector(embed: Option<&EmbedFn>, content: &str) -> Option<Vec<f32>> {
    match embed {
        None => None,
        Some(hook) => match hook(content) {
            Some(v) => Some(v),
            None => {
                tracing::warn!("l1 embedding failed; storing record without vector");
                None
            }
        },
    }
}

/// Apply one dedup decision for a new memory. Returns the persisted record, or
/// `None` when the decision was `skip`.
///
/// `now_ms` seeds both the id (when the decision has none) and the RFC3339
/// timestamps; `idx` disambiguates records written in the same batch.
///
/// `embed` (MEM-46): optional embedding hook — when present, the persisted
/// record carries a dense vector; a hook failure stores the record without
/// one (best-effort, P4).
pub fn write_memory(
    db: &VantaEmbedded,
    session_key: &str,
    session_id: &str,
    memory: &ExtractedMemory,
    decision: &DedupDecision,
    now_ms: u64,
    idx: usize,
    embed: Option<&EmbedFn>,
) -> Result<Option<MemoryRecord>, L1Error> {
    let ns = l1_namespace(session_key);
    let now = epoch_ms_to_rfc3339(now_ms);
    let record_id = if decision.record_id.trim().is_empty() {
        generate_memory_id(now_ms, idx)
    } else {
        decision.record_id.clone()
    };

    match decision.action {
        DedupAction::Skip => Ok(None),
        DedupAction::Store => {
            let record = MemoryRecord {
                id: record_id,
                content: memory.content.clone(),
                memory_type: memory.memory_type,
                priority: memory.priority,
                scene_name: memory.scene_name.clone(),
                source_message_ids: memory.source_message_ids.clone(),
                metadata: memory.metadata.clone(),
                timestamps: vec![now.clone()],
                created_at: now.clone(),
                updated_at: now,
                version: 1,
                session_key: session_key.to_string(),
                session_id: session_id.to_string(),
                task_id: None,
                team_id: None,
                user_id: None,
                agent_id: None,
                vector: None,
            };
            let vector = embed_vector(embed, &record.content);
            put_record(db, &ns, &record, vector)?;
            // MEM-41 provenance: best-effort, never blocks the write (P4).
            crate::core::memory_generation_log::record_best_effort(
                db,
                &crate::core::memory_generation_log::GenerationLogEntry::new(
                    crate::core::memory_generation_log::GenerationLayer::L1,
                    crate::core::memory_generation_log::GenerationStatus::Succeeded,
                    session_key,
                    Some(&record.id),
                    None,
                ),
            );
            Ok(Some(record))
        }
        DedupAction::Update | DedupAction::Merge => {
            let targets = load_targets(db, session_key, &decision.target_ids)?;

            // Delete replaced records first; then upsert the merged one.
            for target in &targets {
                db.delete(&ns, &sanitize_key(&target.id))?;
            }

            // Union of all relevant timestamps, deduped and sorted (decision
            // wins; fallback = all target timestamps + now).
            let timestamps: Vec<String> =
                BTreeSet::from_iter(decision.merged_timestamps.clone().unwrap_or_else(|| {
                    let mut ts: Vec<String> =
                        targets.iter().flat_map(|t| t.timestamps.clone()).collect();
                    ts.push(now.clone());
                    ts
                }))
                .into_iter()
                .collect();

            let base_version = targets.iter().map(|t| t.version).max().unwrap_or(0);
            let earliest = targets
                .iter()
                .map(|t| t.created_at.clone())
                .min()
                .unwrap_or_else(|| now.clone());

            let record = MemoryRecord {
                id: record_id,
                content: decision
                    .merged_content
                    .clone()
                    .unwrap_or_else(|| memory.content.clone()),
                memory_type: decision.merged_type.unwrap_or(memory.memory_type),
                priority: decision.merged_priority.unwrap_or(memory.priority),
                scene_name: memory.scene_name.clone(),
                source_message_ids: memory.source_message_ids.clone(),
                metadata: memory.metadata.clone(),
                timestamps,
                created_at: earliest,
                updated_at: now,
                version: base_version + 1,
                session_key: session_key.to_string(),
                session_id: session_id.to_string(),
                task_id: None,
                team_id: None,
                user_id: None,
                agent_id: None,
                vector: None,
            };
            let vector = embed_vector(embed, &record.content);
            put_record(db, &ns, &record, vector)?;
            // MEM-41 provenance: best-effort, never blocks the write (P4).
            crate::core::memory_generation_log::record_best_effort(
                db,
                &crate::core::memory_generation_log::GenerationLogEntry::new(
                    crate::core::memory_generation_log::GenerationLayer::L1,
                    crate::core::memory_generation_log::GenerationStatus::Succeeded,
                    session_key,
                    Some(&record.id),
                    None,
                ),
            );
            Ok(Some(record))
        }
    }
}

/// Apply a batch of decisions (one per pending memory) and return the records
/// that were actually persisted. Convenience entry point for the pipeline.
pub fn apply_dedup_batch(
    db: &VantaEmbedded,
    session_key: &str,
    session_id: &str,
    memories: &[ExtractedMemory],
    decisions: &[DedupDecision],
    now_ms: u64,
    embed: Option<&EmbedFn>,
) -> Result<Vec<MemoryRecord>, L1Error> {
    let mut written = Vec::new();
    for (idx, memory) in memories.iter().enumerate() {
        let decision = decisions
            .get(idx)
            .cloned()
            .unwrap_or_else(|| DedupDecision {
                record_id: String::new(),
                action: DedupAction::Store,
                target_ids: vec![],
                merged_content: None,
                merged_type: None,
                merged_priority: None,
                merged_timestamps: None,
            });
        if let Some(record) = write_memory(
            db,
            session_key,
            session_id,
            memory,
            &decision,
            now_ms,
            idx,
            embed,
        )? {
            written.push(record);
        }
    }
    Ok(written)
}

/// Load target records by id; missing ids are skipped silently (defensive —
/// a stale target id must not fail the whole batch).
fn load_targets(
    db: &VantaEmbedded,
    session_key: &str,
    target_ids: &[String],
) -> Result<Vec<MemoryRecord>, L1Error> {
    let mut targets = Vec::new();
    for id in target_ids {
        if let Some(record) = read_record(db, session_key, id)? {
            targets.push(record);
        } else {
            tracing::debug!(record_id = %id, "l1 merge/update target not found; skipped");
        }
    }
    Ok(targets)
}

fn put_record(
    db: &VantaEmbedded,
    ns: &str,
    record: &MemoryRecord,
    vector: Option<Vec<f32>>,
) -> Result<(), L1Error> {
    let mut metadata = VantaMemoryMetadata::new();
    metadata.insert(
        "type".into(),
        VantaValue::String(type_name(record.memory_type).to_string()),
    );
    metadata.insert("priority".into(), VantaValue::Int(record.priority as i64));
    db.put(VantaMemoryInput {
        namespace: ns.to_string(),
        key: sanitize_key(&record.id),
        payload: serde_json::to_string(record)?,
        metadata,
        vector,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    Ok(())
}

/// Serde snake_case name of a memory type (matches the wire contract).
fn type_name(memory_type: MemoryType) -> &'static str {
    match memory_type {
        MemoryType::Persona => "persona",
        MemoryType::Episodic => "episodic",
        MemoryType::Instruction => "instruction",
        MemoryType::WorkFact => "work_fact",
        MemoryType::WorkTask => "work_task",
        MemoryType::WorkMethod => "work_method",
        MemoryType::WorkArtifact => "work_artifact",
    }
}

#[cfg(test)]
mod tests {
    use super::generate_memory_id;

    #[test]
    fn id_is_deterministic() {
        assert_eq!(
            generate_memory_id(1_700_000_000_000, 3),
            "m_1700000000000_3"
        );
    }

    #[cfg(feature = "embed-local")]
    #[test]
    fn local_embedding_hook_produces_384d_vectors() {
        let hook = super::local_embedding_hook();
        let v = hook("test content").expect("hook must produce vector");
        assert_eq!(v.len(), 384, "local hook dim 384");
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-3, "L2 normalized, got {norm}");
    }

    #[cfg(feature = "embed-local")]
    #[test]
    fn local_hook_multilingual_cosine_contract() {
        let hook = super::local_embedding_hook();
        let a = hook("hola mundo").expect("embed");
        let b = hook("hello world").expect("embed");
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let n_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let n_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        let cos = dot / (n_a * n_b);
        assert!(cos > 0.60, "multilingual cosine >0.60 got {cos}");
        let self_dot: f32 = a.iter().zip(a.iter()).map(|(x, y)| x * y).sum();
        assert!(self_dot > 0.99, "self cosine >0.99 got {self_dot}");
    }
}
