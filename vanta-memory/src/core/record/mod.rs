//! L1 record pipeline: extraction (MEM-10), dedup + write (MEM-11).

/// L1 memory extraction from recorded L0 messages (MEM-10).
pub mod l1_extractor;

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
pub use l1_extractor::{extract_l1_memories, L1ExtractorConfig};
pub use l1_reader::{l1_namespace, read_record, read_session_records, recall_candidates};
pub use l1_writer::{apply_dedup_batch, generate_memory_id, write_memory, L1Error};
