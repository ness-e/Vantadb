---
title: "Semantic events and provenance log (MEMG-26 spec)"
kind: reference
status: proposed
description: "Spec-first semantic event model for causal memory history: event types, envelope fields, causal links and wire rules; a layer over VantaDB, not a WAL replacement"
tags: [vantadb, api, memory, provenance]
---

# Semantic events and provenance log (MEMG-26 spec)

> **Status:** `proposed` — spec-first by Backlog design (types, envelope,
> wire) plus a minimal slice later. There is **no enforcement and no store**
> in this spec. This layer answers *causal history* (what happened, who did
> it, why, with what evidence). Durability stays with the WAL, which answers
> a different question. Not to be confused with FIND-303 (that event log is
> sync transport).

## What exists today (provenance pieces, no unified log)

| Piece | Symbol | Semantics |
|-------|--------|-----------|
| Consolidation runs | `DreamRun` / `DreamRunMeta` (`vanta-memory/src/core/dream/mod.rs:157,196`, persisted under `dream/<session>/<run_id>`) | What a consolidation pass scanned, merged, normalized, and produced |
| Contradiction provenance | `ContradictionProvenance` (`DreamRun.contradicted_ids`, `dream/mod.rs:181`) | Which records contradicted which, without deleting originals |
| Generation log | `GenerationLogEntry` under `genlog/<session>` (`docs/api/VANTA_MEMORY.md`) | Per-session L1/L2/L3 generation provenance (MEM-41) |
| Write receipts | `WriteReceipt` (`docs/api/WRITE_RECEIPTS.md`, `put_certified`) | Attested projection of a write, with declared `out_of_scope` limits |
| Erasure receipts | `ErasureReceipt` (MEMG-17, per-scope DEK destruction) | Verifiable proof that scoped data became unrecoverable |
| Export integrity | `ExportReport.sha256` + manifest (`src/sdk/types/record.rs:469`, MEMG-15) | Byte-level integrity of checkpoints |
| Deterministic address | `memory_node_id(ns, key)` (`src/sdk/serialization/mod.rs:80`) | The stable id every event references |

None of these is a log; together they are the vocabulary the log reuses.

## Proposed: event model (spec, unenforced)

### Envelope (every event)

```json
{
  "event_id": "evt-01K2...",
  "event_type": "MemoryCaptured",
  "timestamp_ms": 1760572800000,
  "actor": "sub-ego-3",
  "project": "ego-desktop",
  "session": "sess-9f2",
  "task": "task-51",
  "source": "l1/sess-9f2",
  "input_refs": ["ns:key"],
  "output_refs": ["ns:key"],
  "decision_refs": [],
  "causal_parent": "evt-01K1...",
  "confidence": 0.8,
  "result": "ok",
  "evidence": { "run_id": "dream-..." }
}
```

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `event_id` | string | required | Unique per event (host-generated) |
| `event_type` | enum | required | One of the types below; unknown tokens rejected, never inferred |
| `timestamp_ms` | u64 | required | Wall-clock epoch ms of the occurrence, not of logging |
| `actor` | string | `""` | Who/what acted (`agent:{id}`, `user:{id}`, `system`) |
| `project` / `session` / `task` | string | `""` | Scope coordinates (see `memory-scopes.md`); empty = unscoped |
| `source` | string | `""` | Namespace (or URI) the event was observed in |
| `input_refs` / `output_refs` | string[] | `[]` | `ns:key` record refs consumed / produced |
| `decision_refs` | string[] | `[]` | Decision ids this event supports or implements |
| `causal_parent` | string \| null | `null` | The single event this one follows from; chains form the causal trail |
| `confidence` | float | `1.0` | Actor's confidence in `[0, 1]`; `1.0` = mechanical fact |
| `result` | enum | `"ok"` | `ok` \| `failed` \| `superseded`. `failed` carries the error in `evidence` |
| `evidence` | object | `{}` | Type-specific payload (run reports, diffs, receipts); never key material |

### Event types

| Type | Emitted when | `evidence` carries |
|------|--------------|--------------------|
| `MemoryCaptured` | An L1 record is written | record id, session |
| `MemoryDerived` | A record derives from parents (put derivado) | parent keys (`derived_from`) |
| `MemorySuperseded` | A record supersedes another | old → new ids (`superseded_by` edge) |
| `MemoryValidated` | Confidence/validity affirmed | new confidence, validator |
| `MemoryQuarantined` | Content quarantined | reason code |
| `MemoryPromoted` | Dream promotion applied | `DreamRun.run_id` |
| `DecisionCreated` / `DecisionApproved` | A decision record is created / approved | decision id, approver |
| `TaskCreated` / `TaskStarted` / `TaskCompleted` / `TaskFailed` | Host task lifecycle | task id; on failure, the error |
| `ArtifactChanged` | A work artifact revision lands | artifact id, diff ref |
| `SourceObserved` | An external source is ingested | source URI, ingest id |

### Wire rules

- Events serialize as newline-delimited JSON (one object per line), same
  framing as the audit JSONL, but in a **separate stream** (never mixed into
  the audit log or the WAL).
- `event_type` values are `PascalCase` wire literals; the envelope rejects
  unknown types (a reader must not guess).
- `causal_parent` chains are advisory: a missing parent never invalidates
  the child (crash windows may drop links; see the generation-log
  best-effort precedent).
- No retroactive edits: corrections are new events (`MemorySuperseded`,
  or a follow-up with `causal_parent` pointing at the error).

### Explicitly out of scope

- The store (namespace layout, retention, indexing) → minimal slice later.
- Transport/sync of events → FIND-303 owns transport; this spec owns meaning.
- Enforcement or gating on events → MGR-04 / FIND-301.
