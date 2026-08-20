---
title: "ADR-028: Core decay as durable supersession (no automatic scoring/deletion)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, decay, consolidation, supersession]
created: 2026-08-20
last_reviewed: 2026-08-20
---

# ADR-028: Core decay as durable supersession (no automatic scoring/deletion)

## Context

FEAT-03 (plan `2026-08-19-vanta-studio-fase4.md`, Task 17) delivers "consolidación
asistida": mark duplicated/outdated memory records with a visible diff, following
the Mem0 memory-decay / Cognee memify pattern (research `docs/research/human-facing-db-ui/`,
SYNTHESIS §4 OPERACIONES:155, 03 lección 5:252). User decision D16 (2026-08-20)
expands scope to "todo": (a) UI + (b) core decay.

Discovery of the current core (2026-08-20) established:

- **Hard TTL already exists end-to-end.** `put(ttl_ms)` → `FIELD_EXPIRES_AT_MS`
  (`src/sdk/serialization/mod.rs:26`) → `VantaMemoryRecord.expires_at_ms`
  (`src/sdk/types.rs:200`) → `purge_expired()` (`src/sdk/api.rs:821`) with full
  derived-index/text-index cleanup, exposed in Python (`purge_expired`) and CLI
  (`/purge-expired`). `VantaNamespaceStats` already counts `expiring_soon`/`expired`.
- **Recency is an internal heuristic only.** `UnifiedNode.last_accessed`/`hits`
  (`src/node/unified.rs:38-43`) feed cache eviction (`eviction_score`), not search
  scoring; search does not write `last_accessed` on hits.
- **Versioning exists.** `version` + `version_history` (VS-CORE-07), stored as
  `__vanta_`-prefixed relational fields.
- **Search does not filter expired or superseded records at read time** (no
  expiry check in `src/sdk/search/`).
- System fields use a proven pattern: `FIELD_*` constants stored as
  `__vanta_`-prefixed `relational` fields, stripped from user metadata on
  materialization (`memory_record_from_node`).

What is missing is the *supersession* concept from the research: a durable,
first-class marker that record A is superseded by record B, plus read-side
filtering so superseded records can be hidden without being destroyed.

## Decision

Implement **core decay as durable supersession**, not as automatic
scoring/deletion:

1. **Persistent marker (core, first-class).** Add to `VantaMemoryRecord`
   (`src/sdk/types.rs`):
   - `superseded_by: Option<String>` — key of the successor record (same namespace).
   - `superseded_at_ms: Option<u64>` — when the supersession was recorded.
   Persisted as `FIELD_SUPERSEDED_BY` / `FIELD_SUPERSEDED_AT_MS` node relational
   fields (same `__vanta_` pattern as TTL/version). `#[serde(default)]` — additive,
   non-breaking for existing dumps and exports.
2. **Explicit API, no background magic.** `supersede(namespace, old_key, new_key)`
   on `VantaEmbedded` (opt-in `supersedes` on `VantaMemoryInput` for ingest-time
   linking is a cuttable P2). The trigger is a deliberate client action (the UI
   confirm in FEAT-03a, or an agent ingest flow) — the core never decides "this
   looks like a duplicate".
3. **Read-side filter.** `exclude_superseded: bool` (default `false`) on
   `VantaMemorySearchRequest` and `VantaMemoryListOptions` — superseded records
   stay in storage (soft-dead, recoverable) but can be hidden from results.
4. **Explicitly NOT in scope (documented for follow-up):**
   - Mem0-style recency decay (search-score multiplier by `last_accessed`): it is
     client retrieval policy, changes global search semantics, and requires
     writing `last_accessed` per hit (hot-path write amplification, Regla 8/9).
     Deferred; the building block (`last_accessed`) already exists.
   - Cognee-style automatic consolidation worker: rejecting — the core must not
     delete/mark user data from an embedding-similarity threshold without policy
     context (no LLM in core, false-positive risk = data loss). Duplicate
     *detection* stays client-side (FEAT-03a already uses kNN search for
     candidates); the core provides the mechanism, the client the policy.

Rationale: the research's "decay" (SYNTHESIS:155) is precisely "marcar
duplicados/superados ... más humano que solo TTL duro" — a *marker plus diff*,
which must be durable and shared across clients to be useful. The marker belongs
in the core (single source of truth, survives restarts, shows in exports/UI);
the *decision* of what is a duplicate belongs to the application. Forcing
automatic scoring/deletion into the core would be the expensive, data-loss-prone
version nobody asked for yet.

## Consequences

- **Pros**
  - Minimal, additive change: 2 optional fields + 1 API method + 1 read filter;
    no background thread, no scoring change, no migration (serde default).
  - FEAT-03a (UI) gets a durable marker: mark → persist → hide → shows in
    exports and any client; matches `__vanta_` field pattern already proven by
    TTL/version.
  - Backward compatible: old dumps without the fields deserialize to `None`.
  - Soft-dead records remain recoverable (undo/papelera possible later).
- **Cons**
  - `supersede()` writes two records (old updated + linked) — two WAL appends,
    not a single atomic operation. A crash between the two leaves a dangling
    `superseded_by` (old marked, new present anyway — the marker is still
    self-consistent; full 2PC is deferred to ACID Phase 0, same as `insert`).
  - Superseded records still consume storage until `purge_expired`/manual delete
    (accepted: soft-dead is the point).
  - Does not deliver "automatic" decay — users wanting Mem0-style recency scoring
    must wait for the follow-up.
- **Noted gap (out of contract):** search/list do not filter expired records at
  read time either. TTL lifecycle filtering (`exclude_expired`) is a separate
  small feature; tracked here so the UI's "expired/expiring" states (07 Fix 3)
  can be wired later without re-discovery.

## Alternatives Considered

### A. Mem0-style recency decay in search scoring
- Pros: matches Mem0 product behavior; no schema change.
- Cons: search-semantics change (breaking-ish, needs opt-in flag); per-hit
  `last_accessed` write on the read hot path (Regla 8/9 — needs benchmark
  before/after); threshold/policy belongs to the client.
- Rejected for this phase; documented as follow-up.

### B. Automatic consolidation worker (Cognee memify)
- Pros: "self-cleaning" memory; strongest product story.
- Cons: background thread + concurrency model change; needs config
  (thresholds, cadence); embedding-only detection without LLM policy context →
  false-positive deletions = data loss; violates the embedded-DB contract of not
  destroying user data implicitly.
- Rejected.

### C. UI-only (superseded_by as arbitrary metadata, no core field)
- Pros: zero core change.
- Cons: marker is a string in a map — not typed, not filterable in search/list,
  not consistent across clients, not visible in records/export without ad-hoc
  parsing; the plan explicitly scoped (b) core decay under D16 ("quiero todo").
- Rejected: the whole point of (b) is making the marker first-class.

## References

- Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` Task 17, DEFER:207, Riesgos:217.
- Research: SYNTHESIS §4 OPERACIONES:155; 03 lección 5:252; 07 Fix 3:89.
- Pattern: `FIELD_EXPIRES_AT_MS` handling in `src/sdk/serialization/mod.rs`,
  `purge_expired` in `src/sdk/api.rs:821`.