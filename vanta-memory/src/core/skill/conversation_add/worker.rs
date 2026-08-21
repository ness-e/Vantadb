//! SkillConversationExtractWorker (MEM-17) — single-shot consumption of one
//! archived task.
//!
//! Port of TDAM `conversation-add/extract-worker.ts` `runOnce` reduced to the
//! single-process core (no Redis BRPOP/locks — dispatch is MEM-16's
//! [`crate::services::pipeline_worker::PipelineWorker`]). Preserved semantics:
//! - **Ghost check**: a task whose archive is missing is dropped, never
//!   extracted from nothing (TDAM §④).
//! - **Idempotent re-consumption**: a `done` task short-circuits.
//! - **Principio 4**: an LLM failure leaves the task `pending` (retryable) —
//!   the archive data is never lost and the sink never sees partial output.

use crate::core::abstractions::LlmRunner;
use crate::core::skill::skill_extractor::{extract_skills_with_llm, ExtractMessage, SkillSummary};
use vantadb::sdk::VantaEmbedded;

use super::archive::{ArchiveStore, SkillArchiveError, SkillTaskEntry};
use super::compressor::SkillMessage;
use super::sink::{SkillCoreSink, SkillSinkCounts};

/// Outcome of one [`run_skill_extract_once`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillWorkerOutcome {
    /// No pending task with that id.
    NothingPending,
    /// Task already applied — idempotent no-op.
    AlreadyDone,
    /// Task referenced a missing archive → dropped (ghost).
    GhostDropped,
    /// Extraction ran; candidates applied to the sink.
    Applied {
        counts: Option<SkillSinkCounts>,
        candidate_count: usize,
    },
    /// LLM extraction failed — task left `pending` for retry.
    ExtractionFailed { error: String },
}

/// Consume one skill-extraction task end-to-end:
/// read task → ghost check → extract (LLM) → sink (idempotent) → mark done.
pub fn run_skill_extract_once<R: LlmRunner>(
    db: &VantaEmbedded,
    runner: &R,
    session_id: &str,
    task_id: &str,
    existing_skills: &[SkillSummary],
) -> Result<SkillWorkerOutcome, SkillArchiveError> {
    let store = ArchiveStore::new(db);
    let Some(entry) = store.read_task(session_id, task_id)? else {
        return Ok(SkillWorkerOutcome::NothingPending);
    };
    if entry.status == "done" {
        return Ok(SkillWorkerOutcome::AlreadyDone);
    }

    // Ghost check: no archive → drop the task (TDAM §④).
    let Some(messages) = store.read_archive(session_id, &entry.archive_key)? else {
        store.set_task_status(&entry, "dropped")?;
        return Ok(SkillWorkerOutcome::GhostDropped);
    };

    let extract_messages: Vec<ExtractMessage> = messages
        .iter()
        .map(|SkillMessage { role, content }| ExtractMessage::new(role.clone(), content.clone()))
        .collect();
    let result = extract_skills_with_llm(
        runner,
        &extract_messages,
        existing_skills,
        &Default::default(),
    );
    if !result.success {
        // Principio 4: leave the task pending — retryable, nothing lost.
        return Ok(SkillWorkerOutcome::ExtractionFailed {
            error: result.error.unwrap_or_else(|| "unknown".into()),
        });
    }
    if entry.status == "dropped" {
        return Ok(SkillWorkerOutcome::GhostDropped);
    }

    let candidate_count = result.candidates.len();
    let sink = SkillCoreSink::new(db);
    let counts = sink.apply_candidates(
        session_id,
        task_id,
        &result.candidates,
        crate::core::conversation::now_ms(),
    )?;
    let done: SkillTaskEntry = store.set_task_status(&entry, "done")?;
    debug_assert_eq!(done.status, "done");
    Ok(SkillWorkerOutcome::Applied {
        counts,
        candidate_count,
    })
}
