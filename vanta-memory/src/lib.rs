//! VantaDB Memory Engine — four-layer (L0-L3) memory pipeline.
//!
//! ```rust
//! assert_eq!(vanta_memory::name(), "vanta-memory");
//! ```
//!
//! Port of the TDAM memory architecture (`docs/research/tdam/`) reimplemented
//! in Rust over the existing VantaDB stack. All persistence lives in the
//! VantaDB store (nodes, `InternalMetadata` partitions, text index, HNSW,
//! core graph) — never external storage.
//!
//! The crate is **host-neutral**: it defines the [`LlmRunner`] abstraction and
//! ships no LLM runtime. With `llm-driver` off (default), every LLM-dependent
//! path degrades to an LLM-free equivalent (local compression, store-all,
//! heuristic dedup) — it never blocks and never loses data.
//!
//! F4 task order: MEM-08a (this scaffold) → MEM-08b (contracts + trait) →
//! MEM-09..21 (L0→L1→L2→L3, triggers, skill extract, recall, cursor, MCP
//! scenes). See `docs/plans/2026-08-18-vanta-memory.md`.
//!
//! [`LlmRunner`]: core::abstractions::LlmRunner

/// Core memory pipeline (L0 capture → L1 extraction/dedup → L2 scenes →
/// L3 persona) and skill extraction. Host-neutral.
pub mod core;

/// LLM-driven orchestration: pipeline manager, timers, locks, checkpointing.
pub mod utils;

/// Service layer: worker loop and scheduler glue.
pub mod services;

/// Host adapters: standalone vs. host (OpenClaw-style) LLM runners.
pub mod adapters;

/// Context offload: state manager, cursor, storage, after-tool-call hooks.
pub mod offload;

/// Context engine: token estimator, emergency truncate, compaction reports.
pub mod context_engine;

/// Gateway-facing handlers (MCP scene/knowledge tools).
pub mod gateway;

/// Crate name (used by smoke test; kept trivial on purpose).
pub fn name() -> &'static str {
    "vanta-memory"
}
