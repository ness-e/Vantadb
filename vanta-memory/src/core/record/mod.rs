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
/// silent deletion).
pub mod lifecycle;

/// L1 memory reader + LLM-free candidate recall (MEM-11).
pub mod l1_reader;

/// L1 memory writer — applies dedup decisions to the store (MEM-11).
pub mod l1_writer;

/// L1 two-phase dedup pipeline (MEM-11).
pub mod l1_dedup;

pub use l1_dedup::{
    batch_dedup, parse_batch_result, prepare_pending, run_l1_dedup, L1DedupConfig, PendingMemory,
    CONFLICT_DETECTION_TASK_ID,
};
pub use l1_extractor::{extract_l1_memories, extract_l1_segments, L1ExtractorConfig};
pub use l1_reader::{l1_namespace, read_record, read_session_records, recall_candidates};
pub use l1_writer::{apply_dedup_batch, generate_memory_id, write_memory, EmbedFn, L1Error};

#[cfg(feature = "embeddings")]
pub use l1_writer::core_embedding_hook;
