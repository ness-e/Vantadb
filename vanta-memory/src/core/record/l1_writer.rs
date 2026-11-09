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

use vantadb::error::Error;
use vantadb::sdk::{
    default_confidence, ConfidenceClass, DedupPolicy, Embedded, MemoryInput, MemoryMetadata, Value,
};

use crate::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, MemoryRecord, MemoryType,
};
use crate::core::conversation::sanitize_key;
use crate::core::profile::profile_sync::ProfileIsolation;
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::record::l1_reader::{l1_namespace, read_record};
use crate::core::record::lifecycle::mark_contradiction;

/// Errors surfaced by the L1 writer/reader surface. One error type for the
/// whole L1 layer so callers depend on a single contract.
#[derive(Debug, Error)]
pub enum L1Error {
    #[error("vantadb: {0}")]
    Vanta(#[from] Error),
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
    let mut cfg = vantadb::config::Config::default().llm_cfg();
    cfg.strict_embeddings = false;
    let provider = vantadb::llm::LocalOnnxProvider::from_llm_cfg(&cfg)
        .unwrap_or_else(|_| vantadb::llm::LocalOnnxProvider::new_dummy(384));
    let provider = std::sync::Arc::new(provider);
    Arc::new(move |text: &str| {
        use vantadb::llm::EmbeddingProvider;
        provider.embed(text).ok()
    })
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

/// Default tenancy stamp for pipeline-written L1 records.
///
/// The extraction pipeline knows no tenant, but cross-session Agent/Team
/// recall matches on `agent_id`/`team_id` — unstamped records would stay
/// invisible outside their own session (incl. the synthetic MCP session).
/// Stamping the default isolation keeps D22 semantics: explicitly-tenanted
/// records are untouched, and legacy `None` records stay session-only.
fn default_tenancy() -> (Option<String>, Option<String>) {
    let iso = ProfileIsolation::default();
    (Some(iso.team_id), Some(iso.agent_id))
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
    db: &Embedded,
    session_key: &str,
    session_id: &str,
    memory: &ExtractedMemory,
    decision: &DedupDecision,
    now_ms: u64,
    idx: usize,
    embed: Option<&EmbedFn>,
) -> Result<Option<MemoryRecord>, L1Error> {
    let Some(planned) = plan_write(
        db,
        session_key,
        session_id,
        memory,
        decision,
        now_ms,
        idx,
        embed,
    )?
    else {
        return Ok(None);
    };
    let ns = l1_namespace(session_key);
    persist_planned(db, &ns, &planned)?;
    log_and_mark(db, &ns, session_key, &planned, now_ms)?;
    Ok(Some(planned.record))
}

/// One planned L1 write: the record + its node vector, the update/merge
/// targets to delete before persisting, and the MEMG-01 contradiction ids to
/// flag after persisting. Planning is persistence-free so the batch path can
/// group-commit every record of a flush in one `put_batch` (MEMG-11).
struct PlannedWrite {
    record: MemoryRecord,
    vector: Option<Vec<f32>>,
    /// Storage keys (`sanitize_key`) of update/merge targets. In the batch
    /// path they are pruned AFTER the `put_batch` commit and only when the
    /// batch did not rewrite the same key (write-first, MEMG-27).
    deletes: Vec<String>,
    /// Explicit contradiction record ids (MEMG-01), flagged post-commit.
    contradicts: Vec<String>,
}

/// Build the record a decision resolves to (store or merge), loading merge
/// targets from the store. Pure planning: nothing is persisted here.
#[allow(clippy::too_many_arguments)]
fn plan_write(
    db: &Embedded,
    session_key: &str,
    session_id: &str,
    memory: &ExtractedMemory,
    decision: &DedupDecision,
    now_ms: u64,
    idx: usize,
    embed: Option<&EmbedFn>,
) -> Result<Option<PlannedWrite>, L1Error> {
    let now = epoch_ms_to_rfc3339(now_ms);
    let (team_id, agent_id) = default_tenancy();
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
                team_id: team_id.clone(),
                user_id: None,
                agent_id: agent_id.clone(),
                vector: None,
                heat: 0,
                superseded_by: None,
            };
            let vector = embed_vector(embed, &record.content);
            Ok(Some(PlannedWrite {
                record,
                vector,
                deletes: Vec::new(),
                contradicts: decision.contradicts.clone(),
            }))
        }
        DedupAction::Update | DedupAction::Merge => {
            let targets = load_targets(db, session_key, &decision.target_ids)?;

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
                team_id,
                user_id: None,
                agent_id,
                vector: None,
                heat: 0,
                superseded_by: None,
            };
            let vector = embed_vector(embed, &record.content);
            // Delete replaced records first; then upsert the merged one.
            let deletes = targets.iter().map(|t| sanitize_key(&t.id)).collect();
            Ok(Some(PlannedWrite {
                record,
                vector,
                deletes,
                contradicts: decision.contradicts.clone(),
            }))
        }
    }
}

/// Persist one planned write: delete update/merge targets, then upsert the
/// record (vector on the node, stripped from the payload).
fn persist_planned(db: &Embedded, ns: &str, planned: &PlannedWrite) -> Result<(), L1Error> {
    for key in &planned.deletes {
        db.delete(ns, key)?;
    }
    put_record(db, ns, &planned.record, planned.vector.clone())?;
    Ok(())
}

/// Post-commit per-record side effects shared by both write paths: the MEM-41
/// generation-log entry (best-effort, never blocks the write — P4) and the
/// MEMG-01 contradiction marking (supersede pointer, never delete; no-op on
/// the common path).
fn log_and_mark(
    db: &Embedded,
    ns: &str,
    session_key: &str,
    planned: &PlannedWrite,
    now_ms: u64,
) -> Result<(), L1Error> {
    crate::core::memory_generation_log::record_best_effort(
        db,
        &crate::core::memory_generation_log::GenerationLogEntry::new(
            crate::core::memory_generation_log::GenerationLayer::L1,
            crate::core::memory_generation_log::GenerationStatus::Succeeded,
            session_key,
            Some(&planned.record.id),
            None,
        ),
    );

    if !planned.contradicts.is_empty() {
        mark_contradicted_targets(
            db,
            ns,
            session_key,
            &planned.contradicts,
            &planned.record.id,
            now_ms,
        )?;
    }
    Ok(())
}

/// MEMG-01: flag the records the new memory explicitly contradicts.
///
/// Reuses MEM-60 [`mark_contradiction`] — the exact semantics dream
/// consolidation uses (`superseded_by` pointer, old record preserved, never
/// deleted). Conservative by construction:
/// - only ids the judgment listed as explicit contradictions,
/// - already-superseded records are left untouched (their provenance chain
///   is not rewritten),
/// - the new record itself and unknown ids are skipped silently.
///
/// The new record must be persisted first: its id is the pointer's
/// `new_key`. `skip` decisions never reach this helper (nothing persisted).
///
/// Failure semantics: a store error while marking propagates (`Err`) even
/// though the new record is already persisted — consistent with the batch's
/// existing partial-write model (a mid-batch error retries the flush; the
/// generation log is the only best-effort path by design).
fn mark_contradicted_targets(
    db: &Embedded,
    ns: &str,
    session_key: &str,
    contradicts: &[String],
    new_key: &str,
    now_ms: u64,
) -> Result<(), L1Error> {
    let new_storage_key = sanitize_key(new_key);
    for target_id in contradicts {
        // Compare storage keys: an id written differently that sanitizes to
        // the new record's key would resolve to the new record itself.
        if sanitize_key(target_id) == new_storage_key {
            continue;
        }
        let Some(mut old) = read_record(db, session_key, target_id)? else {
            tracing::debug!(record_id = %target_id, "l1 contradiction target not found; skipped");
            continue;
        };
        if old.superseded_by.is_some() {
            tracing::debug!(
                record_id = %target_id,
                "l1 contradiction target already superseded; skipped"
            );
            continue;
        }
        mark_contradiction(&mut old, new_key.to_string(), now_ms);
        // Preserve the old record's node vector on re-persist (vectors live
        // on the node, not inside the payload — see `put_record`).
        let vector = old.vector.take();
        put_record(db, ns, &old, vector)?;
    }
    Ok(())
}

/// Apply a batch of decisions (one per pending memory) and return the records
/// that were actually persisted. Group-commit entry point for the pipeline
/// (MEMG-11): every record of the flush is persisted in ONE `put_batch`
/// (single WAL `batch_append` + KV `write_batch` + per-record index ops)
/// instead of N sequential `put`s.
pub fn apply_dedup_batch(
    db: &Embedded,
    session_key: &str,
    session_id: &str,
    memories: &[ExtractedMemory],
    decisions: &[DedupDecision],
    now_ms: u64,
    embed: Option<&EmbedFn>,
) -> Result<Vec<MemoryRecord>, L1Error> {
    let ns = l1_namespace(session_key);

    // Phase 1 — plan every write (record + vector + deletes + contradictions).
    let mut planned_writes: Vec<PlannedWrite> = Vec::with_capacity(memories.len());
    for (idx, memory) in memories.iter().enumerate() {
        let decision = decisions
            .get(idx)
            .cloned()
            .unwrap_or_else(|| DedupDecision {
                record_id: String::new(),
                action: DedupAction::Store,
                target_ids: vec![],
                contradicts: vec![],
                merged_content: None,
                merged_type: None,
                merged_priority: None,
                merged_timestamps: None,
            });
        if let Some(planned) = plan_write(
            db,
            session_key,
            session_id,
            memory,
            &decision,
            now_ms,
            idx,
            embed,
        )? {
            planned_writes.push(planned);
        }
    }

    // Phase 2 — group-commit FIRST (write-first protocol, MEMG-27): the
    // consolidated state lands in one `put_batch` before any old key is
    // removed. If the batch fails — upfront validation is all-or-nothing
    // (shape, vector dimension, quota; src/sdk/api/memory.rs) — it returns
    // `Err` here and zero deletes have run, so every pre-flush record
    // survives. Past validation, `put_batch` commits per `batch_size` chunk;
    // deletes still only run after an `Ok` covering the whole flush, so even
    // a mid-batch failure leaves old and new records coexisting — never a
    // loss of the previous state.
    let inputs = planned_writes
        .iter()
        .map(|planned| record_input(&ns, &planned.record, planned.vector.clone()))
        .collect::<Result<Vec<MemoryInput>, L1Error>>()?;
    if !inputs.is_empty() {
        db.put_batch(inputs)?;
    }

    // Phase 3 — prune ONLY the keys the batch did not rewrite:
    //   keys_to_prune = old_keys \ new_batch_keys
    // A target key that coincides with a new record's key was already
    // overwritten atomically inside `put_batch`; deleting it now would
    // destroy the record the flush just wrote (silently — the merge would
    // look committed yet be gone).
    let new_keys: BTreeSet<String> = planned_writes
        .iter()
        .map(|planned| sanitize_key(&planned.record.id))
        .collect();
    for planned in &planned_writes {
        for key in &planned.deletes {
            if !new_keys.contains(key) {
                db.delete(&ns, key)?;
            }
        }
    }

    // Phase 4 — post-commit per-record side effects (generation log +
    // contradiction marking), same semantics as the sequential path.
    let mut written = Vec::with_capacity(planned_writes.len());
    for planned in &planned_writes {
        log_and_mark(db, &ns, session_key, planned, now_ms)?;
        written.push(planned.record.clone());
    }
    Ok(written)
}

/// Load target records by id; missing ids are skipped silently (defensive —
/// a stale target id must not fail the whole batch).
fn load_targets(
    db: &Embedded,
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

/// Persist one record under `ns` (key = sanitized record id). The caller
/// passes the record with `vector: None` and the node vector separately —
/// vectors live on the node, not inside the payload. Shared with the dream
/// promotion path (`core::dream`), which writes consolidated records verbatim.
///
/// **v2 write-side semantics (single write point — MEMG-12):** every L1 write
/// (extraction store/update/merge, contradiction re-persist and dream
/// promotion) declares its `valid_at_ms` (content birth) and confidence
/// (`Asserted` + D_a, MGR-12 §3.4) explicitly instead of leaving both to
/// system defaults. The values are intrinsic to the record — `created_at` is
/// the birth; merges already carry the earliest target birth.
/// Build the `MemoryInput` for one record under `ns` (key = sanitized record
/// id, type/priority metadata, serialized payload, node vector separate).
///
fn default_l1_dedup_policy() -> DedupPolicy {
    match std::env::var("VANTA_L1_DEDUP_POLICY").as_deref() {
        Ok("off") | Ok("none") | Ok("false") | Ok("0") => DedupPolicy::Off,
        Ok("exact") => DedupPolicy::Exact,
        _ => DedupPolicy::Semantic,
    }
}

/// Single input builder for both write paths (MEMG-11): `put_record` (single
/// put) and `apply_dedup_batch` (group-commit via `put_batch`). Carries the
/// v2 write-side fields declared in `put_record`'s doc (valid_at + confidence
/// — MEMG-12 single write point).
fn record_input(
    ns: &str,
    record: &MemoryRecord,
    vector: Option<Vec<f32>>,
) -> Result<MemoryInput, L1Error> {
    let mut metadata = MemoryMetadata::new();
    metadata.insert(
        "type".into(),
        Value::String(type_name(record.memory_type).to_string()),
    );
    metadata.insert("priority".into(), Value::Int(record.priority as i64));
    if let Some(obj) = record.metadata.as_object() {
        for (k, v) in obj {
            match v {
                serde_json::Value::Bool(b) => {
                    metadata.insert(k.clone(), Value::Bool(*b));
                }
                serde_json::Value::Number(num) => {
                    if let Some(i) = num.as_i64() {
                        metadata.insert(k.clone(), Value::Int(i));
                    } else if let Some(f) = num.as_f64() {
                        metadata.insert(k.clone(), Value::Float(f));
                    }
                }
                serde_json::Value::String(s) => {
                    metadata.insert(k.clone(), Value::String(s.clone()));
                }
                serde_json::Value::Array(arr) => {
                    let mut strings = Vec::new();
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            strings.push(s.to_string());
                        }
                    }
                    if !strings.is_empty() {
                        metadata.insert(k.clone(), Value::ListString(strings));
                    }
                }
                _ => {}
            }
        }
    }
    // Content birth → validity start (ADR-046 §D7/D8). `None` only when the
    // birth does not parse or is 0 — the core then defaults to write-time
    // `created_at_ms` (`materialize_valid_at`).
    let valid_at_ms = rfc3339_to_epoch_ms(&record.created_at).filter(|ms| *ms > 0);
    Ok(MemoryInput {
        namespace: ns.to_string(),
        key: sanitize_key(&record.id),
        payload: serde_json::to_string(record)?,
        metadata,
        vector,
        sparse_vector: None,
        ttl_ms: None,
        valid_at_ms,
        confidence_class: Some(ConfidenceClass::Asserted),
        confidence: Some(default_confidence()),
        dedup_policy: Some(default_l1_dedup_policy()),
        ..Default::default()
    })
}

pub fn put_record(
    db: &Embedded,
    ns: &str,
    record: &MemoryRecord,
    vector: Option<Vec<f32>>,
) -> Result<vantadb::sdk::MemoryRecord, L1Error> {
    debug_assert!(
        record.vector.is_none(),
        "put_record expects the vector stripped from the record; pass it via the vector arg"
    );
    // Returns the stored record carrying the core-assigned storage version —
    // the rollback lineage (MEMG-17) reads `version` off it.
    let stored = db.put(record_input(ns, record, vector)?)?;

    // Graph edges for derived records/axioms (TEC-15)
    let session_key = ns.strip_prefix("l1/").unwrap_or(ns);
    let node_id =
        crate::core::ontology::entity_node_id(&format!("record:{session_key}:{}", record.id));
    let mut sources = Vec::new();
    if let Some(arr) = record
        .metadata
        .get("source_episode_ids")
        .and_then(|v| v.as_array())
    {
        for item in arr {
            if let Some(s) = item.as_str() {
                sources.push(s.to_string());
            }
        }
    }
    for src in &record.source_message_ids {
        if !sources.contains(src) {
            sources.push(src.clone());
        }
    }
    for src_id in sources {
        let parent_node_id =
            crate::core::ontology::entity_node_id(&format!("episode:{session_key}:{src_id}"));
        let _ = db.add_edge(node_id, parent_node_id, "derived_from", None, None);
    }

    Ok(stored)
}

/// Inverse of [`epoch_ms_to_rfc3339`]: the L1 record's `created_at` is the
/// content birth, and `valid_at_ms` must reflect it explicitly (ADR-046 §D7:
/// v1 normalization `valid_at = created_at`). `None` when the string does not
/// parse or predates the epoch — the core then defaults to `created_at_ms` at
/// write time.
fn rfc3339_to_epoch_ms(iso: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(iso)
        .ok()
        .and_then(|dt| u64::try_from(dt.timestamp_millis()).ok())
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
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use vantadb::config::Config;
    use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
    use vantadb::storage::BackendKind;

    use super::{apply_dedup_batch, generate_memory_id, EmbedFn};
    use crate::core::abstractions::{
        DedupAction, DedupDecision, ExtractedMemory, MemoryRecord, MemoryType,
    };
    use crate::core::record::l1_reader::{read_record, read_session_records};

    fn open_db() -> Embedded {
        Embedded::open_with_config(Config {
            backend_kind: BackendKind::InMemory,
            ..Config::default()
        })
        .expect("open embedded")
    }

    fn memory(content: &str) -> ExtractedMemory {
        ExtractedMemory {
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            source_message_ids: vec![],
            scene_name: "prefs".into(),
            metadata: serde_json::Value::Null,
        }
    }

    fn decision(record_id: &str, action: DedupAction) -> DedupDecision {
        DedupDecision {
            record_id: record_id.into(),
            action,
            target_ids: vec![],
            contradicts: vec![],
            merged_content: None,
            merged_type: None,
            merged_priority: None,
            merged_timestamps: None,
        }
    }

    fn existing_record(id: &str, content: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "prefs".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
            created_at: "2026-08-20T10:00:00.000Z".into(),
            updated_at: "2026-08-20T10:00:00.000Z".into(),
            version: 1,
            session_key: "sess".into(),
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

    fn put_raw(db: &Embedded, ns: &str, record: &MemoryRecord) {
        db.put(MemoryInput {
            namespace: ns.to_string(),
            key: record.id.clone(),
            payload: serde_json::to_string(record).expect("serialize"),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .expect("put");
    }

    #[test]
    fn id_is_deterministic() {
        assert_eq!(
            generate_memory_id(1_700_000_000_000, 3),
            "m_1700000000000_3"
        );
    }

    /// MEMG-27 RED: a flush whose `put_batch` fails must not lose the
    /// pre-existing records. The old order (delete targets → put_batch)
    /// left `old1` deleted with no replacement; the write-first order fails
    /// before any delete runs.
    ///
    /// Failure injection: two planned records carry vectors of different
    /// dims — `put_batch` upfront dimension validation is all-or-nothing
    /// (FIND-411), so the batch errors deterministically on every backend.
    #[test]
    fn failed_flush_preserves_preexisting_records() {
        let db = open_db();
        put_raw(&db, "l1/sess", &existing_record("old1", "user used emacs"));

        let mut update = decision("m_c", DedupAction::Update);
        update.target_ids = vec!["old1".into()];
        update.merged_content = Some("user switched to vim".into());
        let memories = [memory("merge into old1"), memory("fresh store")];
        let decisions = [update, decision("m_d", DedupAction::Store)];

        // Embed hook yields dim 3 for the first record and dim 4 for the
        // second → the batch fails dimension validation before any write.
        let next_dim = AtomicUsize::new(3);
        let embed: EmbedFn = Arc::new(move |_| {
            let dim = next_dim.swap(4, Ordering::SeqCst);
            Some(vec![0.1_f32; dim])
        });

        let res = apply_dedup_batch(
            &db,
            "sess",
            "sess",
            &memories,
            &decisions,
            1_700_000_000_000,
            Some(&embed),
        );
        assert!(res.is_err(), "mixed vector dims must fail the batch");

        let remaining = read_session_records(&db, "sess").expect("read session");
        assert!(
            remaining.iter().any(|r| r.id == "old1"),
            "write-first: a failed flush must leave pre-existing records intact; \
             got: {:?}",
            remaining.iter().map(|r| r.id.clone()).collect::<Vec<_>>()
        );
    }

    /// MEMG-27: an update whose new record id sanitizes to the SAME storage
    /// key as its target. `put_batch` already overwrote that key; the prune
    /// phase must only delete `old_keys \ new_batch_keys` — a naive
    /// delete-all would destroy the record the batch just wrote.
    #[test]
    fn coincident_key_survives_write_first_prune() {
        let db = open_db();
        put_raw(&db, "l1/sess", &existing_record("m_same", "v1 content"));

        let mut update = decision("m_same", DedupAction::Update);
        update.target_ids = vec!["m_same".into()];
        update.merged_content = Some("v2 content".into());
        let memories = [memory("update in place")];

        let written = apply_dedup_batch(
            &db,
            "sess",
            "sess",
            &memories,
            &[update],
            1_700_000_000_000,
            None,
        )
        .expect("batch write");
        assert_eq!(written.len(), 1);

        let rec = read_record(&db, "sess", "m_same")
            .expect("read")
            .expect("record rewritten by the batch must survive the prune");
        assert_eq!(rec.content, "v2 content");
        assert_eq!(rec.version, 2, "merge bumps payload version");
    }

    /// MEMG-27: merge targets whose keys are NOT rewritten by the batch
    /// (merge A+B → new id C) must still be purged after the commit.
    #[test]
    fn merged_orphan_keys_are_pruned() {
        let db = open_db();
        put_raw(&db, "l1/sess", &existing_record("old_a", "fact A"));
        put_raw(&db, "l1/sess", &existing_record("old_b", "fact B"));

        let mut merge = decision("m_new", DedupAction::Merge);
        merge.target_ids = vec!["old_a".into(), "old_b".into()];
        merge.merged_content = Some("facts A+B".into());
        let memories = [memory("merge A and B")];

        apply_dedup_batch(
            &db,
            "sess",
            "sess",
            &memories,
            &[merge],
            1_700_000_000_000,
            None,
        )
        .expect("batch write");

        let remaining = read_session_records(&db, "sess").expect("read session");
        let ids: Vec<&str> = remaining.iter().map(|r| r.id.as_str()).collect();
        assert!(
            ids.contains(&"m_new"),
            "merged record persisted; got {ids:?}"
        );
        assert!(
            !ids.contains(&"old_a") && !ids.contains(&"old_b"),
            "orphan merge targets must be pruned; got {ids:?}"
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
