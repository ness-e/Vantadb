//! Wiki knowledge store (MEM-28, F7 — D27: LLM-free store in core).
//!
//! A wiki space is a managed set of markdown pages with a build lifecycle
//! `pending → processing → ready | failed` (TDAM wiki-service parity). The
//! store persists two record families in the `InternalMetadata` partition —
//! the exact [`crate::entity::EntityStore`] partition pattern (D4), no new
//! storage mechanism:
//!
//! - `wiki:{namespace}::{slug}` — the [`Wiki`] lifecycle record (state,
//!   `run_id`, `sync_error`, optimistic `version`);
//! - `wiki:{namespace}::{slug}:page:{path}` — one [`WikiPage`] per managed
//!   page, addressed by its canonical path (`type + title`, TDAM dedup,
//!   wiki-service.ts:392-410) and always `locked: true`.
//!
//! The store is intentionally dumb: it owns state transitions, key
//! sanitization and cascade delete. Content merging/ingest (LLM) lives in
//! `vanta-memory` (MEM-30) and drives these transitions via the SDK.

pub mod state;
pub mod store;

pub use state::WikiState;
pub use store::{canonical_path, Wiki, WikiPage, WikiStore};

/// Maximum length of a stored `sync_error` (TDAM parity, ≤500 chars).
pub(crate) const SYNC_ERROR_MAX_CHARS: usize = 500;

#[cfg(test)]
mod tests;
