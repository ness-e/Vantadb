---
title: Milestone v0.2.0 — Repo Alignment & Reliability Gate
type: operations
status: active
tags: [vantadb, operations, milestone]
last_reviewed: 2026-07-01
---

# Milestone v0.2.0: Repo Alignment & Reliability Gate

## Objective
Close the gap between the existing technical core and how the repository explains, measures, and exposes it to embedded consumers.

## Main Topics

### 1. Repo truth
- [x] Reposition the project as embedded persistent memory + vector retrieval + structured fields.
- [x] Remove premature claims about universal multi-model and competitive hybrid text.
- [x] Convert SIFT1M into a stress/recovery benchmark, not a competitive one.

### 2. Memory telemetry contract
- [x] Correct process units in certification.
- [x] Make explicit that process telemetry does not equate to logical index footprint.
- [x] Add controlled harness for cold start, ingestion, replay, and restart.

### 3. Embedded SDK stabilization
- [x] Keep `src/sdk.rs` as a stable internal boundary.
- [x] Keep the Python binding behind that boundary.
- [x] Prepare CI for wheels/TestPyPI and defer production PyPI, signing, and installers.

### 4. Reliability gate
- [x] Durability recovery green
- [x] Index reconstruction green
- [x] Backend parity green
- [x] Memory telemetry harness green
- [x] Python SDK smoke green
- [x] Python SDK pytest green

## Success Criteria
- The README describes exactly what the core does today.
- Memory metrics have explicit type, unit, and confidence level.
- The Python SDK works locally via source-install without touching engine internals.
- The next cycle is cleared and now advances on `canonical model`, `namespaces`, and `put/get/delete/list + WAL/recovery`.

## Extension memory-mvp-core

The subsequent block has already been implemented in the repo:

- Canonical persistent memory model in the SDK, separate from `UnifiedNode`.
- First-class namespaces with logical identity `namespace + key`.
- Minimal API `put/get/delete/list/search` with vector-only, BM25 text-only, basic phrase query, and Hybrid Retrieval v1 with RRF/minimal planner.
- Python SDK with memory flow and legacy API compatibility.
- Embedded CLI for `put/get/list`.
- Manual ANN rebuild from VantaFile/canonical storage.
- Stable JSONL export/import for persistent memory.
- Derived indexes `namespace_index` and `payload_index` rebuildable from canonical records.

## Additional Evidence

- `memory_api`
- `memory_export_import`
- `derived_indexes`
- `memory_brutality`

`memory_brutality` covers recovery without explicit flush, loss of `vector_index.bin`, manual rebuild, export/import round-trip, and smoke test of 10K records with namespaces and filters.
