//! L1 record pipeline: extraction (MEM-10), dedup + write (MEM-11).
//!
//! MEM-60: lifecycle module — heat bump on access, decay over time,
//! contradiction provenance (never silent delete; superseded_by chain).

/// L1 memory extraction from recorded L0 messages (MEM-10).
pub mod l1_extractor;

/// MEM-60: heat + decay + contradiction tracking for L1 records.
///
/// `bump_heat` runs on every successful `read` (signal of usefulness);
/// `decay_heat` runs on the periodic maintenance pass (signal of
/// forgetting). `mark_contradiction` writes a `superseded_by` pointer
/// to the new record — the OLD record is preserved (provenance, never
/// silent deletion). MEMG-07 adds the forgetting curve
/// (`retention_factor` / `effective_heat` / `scan_decay`): a declared
/// per-type half-life policy, read-side — it deprioritizes, never purges.
pub mod lifecycle;

/// L1 memory reader + LLM-free candidate recall (MEM-11).
pub mod l1_reader;

/// MEMG-21: composite L1 scoring — recency + relevance + importance.
///
/// Opt-in re-ranking of recall candidates: `composite = w_rel·relevance +
/// w_rec·recency + w_imp·importance`, where recency is MEMG-07's
/// [`lifecycle::retention_factor`] (consumed, not reimplemented), relevance is
/// the caller's raw pool score (min-max normalized over the candidate set,
/// Park et al. §4.1) and importance is the record's declared `priority`.
/// Declared defaults (policy, not calibration) mirror CrewAI's composite
/// scoring: relevance 500 · recency 300 · importance 200 per-mille.
pub mod scoring;

/// L1 memory writer — applies dedup decisions to the store (MEM-11).
pub mod l1_writer;

/// L1 two-phase dedup pipeline (MEM-11).
pub mod l1_dedup;

/// L1 batch extract+dedup in one LLM call (MEM-69).
pub mod l1_batch;

/// MEM-68: optional capture-approval gate (default off, never-block).
pub mod approval;

pub use approval::{
    should_gate, ApprovalError, CaptureApprovalConfig, CaptureApprovalQueue, PendingCapture,
};
pub use l1_batch::{extract_dedup_batch, EXTRACT_DEDUP_TASK_ID};
pub use l1_dedup::{
    batch_dedup, parse_batch_result, prepare_pending, run_l1_dedup, L1DedupConfig, PendingMemory,
    CONFLICT_DETECTION_TASK_ID,
};
pub use l1_extractor::{extract_l1_memories, extract_l1_segments, L1ExtractorConfig};
pub use l1_reader::{
    diff_records, l1_namespace, read_record, read_record_version, read_record_versions,
    read_session_records, recall_candidates, run_decay_pass, RecordFieldChange, RecordVersion,
};
pub use l1_writer::{apply_dedup_batch, generate_memory_id, write_memory, EmbedFn, L1Error};
pub use scoring::{
    composite_rank, composite_score, importance_score, CompositeScoring, ScoringWeights,
};

/// Canonical single-record L1 write (vector stripped, node vector separate).
/// Shared with the dream promotion path (VER-07).
pub(crate) use l1_writer::put_record;

#[cfg(feature = "embeddings")]
pub use l1_writer::core_embedding_hook;
