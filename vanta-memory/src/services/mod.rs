//! Service layer: worker loop and scheduler glue.
//!
//! Filled in by MEM-16 (see plan file).

/// Pipeline task worker: prioritized consumption, per-session locks,
/// retry/dead-letter, and the L0→L3 [`pipeline_worker::MemoryTaskHandler`].
pub mod pipeline_worker;
