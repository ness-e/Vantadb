//! L1 record pipeline: extraction (MEM-10), write + dedup (MEM-11+).

/// L1 memory extraction from recorded L0 messages (MEM-10).
pub mod l1_extractor;

pub use l1_extractor::{extract_l1_memories, L1ExtractorConfig};
