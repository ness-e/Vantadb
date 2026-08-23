//! Service layer: worker loop and scheduler glue.
//!
//! Filled in by MEM-16 (see plan file).

/// Pipeline task worker: prioritized consumption, per-session locks,
/// retry/dead-letter, and the L0L3 [`pipeline_worker::MemoryTaskHandler`].
pub mod pipeline_worker;

/// MEM-55: `/conversation/add` → memory pipeline bridge (feature `http-server`).
#[cfg(feature = "http-server")]
pub mod conversation_hook;
