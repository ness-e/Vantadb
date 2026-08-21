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

/// Apply one dedup decision for a new memory. Returns the persisted record, or
/// `None` when the decision was `skip`.
///
/// `now_ms` seeds both the id (when the decision has none) and the RFC3339
/// timestamps; `idx` disambiguates records written in the same batch.
pub fn write_memory(
    db: &VantaEmbedded,
    session_key: &str,
    session_id: &str,
    memory: &ExtractedMemory,
    decision: &DedupDecision,
    now_ms: u64,
    idx: usize,
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
            };
            put_record(db, &ns, &record)?;
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
            };
            put_record(db, &ns, &record)?;
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
        if let Some(record) =
            write_memory(db, session_key, session_id, memory, &decision, now_ms, idx)?
        {
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

fn put_record(db: &VantaEmbedded, ns: &str, record: &MemoryRecord) -> Result<(), L1Error> {
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
        vector: None,
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
}
