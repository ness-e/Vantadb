---
title: Repo Alignment Checklist
type: operations
status: active
tags: [vantadb, operations, checklist]
last_reviewed: 2026-07-01
---

# Repo Alignment Checklist

This checklist defines the immediate repository cut after the initial technical release. Its goal is no longer "push distribution," but to align narrative, telemetry, and surface area with the actual state of the core.

## 1. Claims and documentation

- [x] README repositioned as embedded persistent memory + vector retrieval.
- [x] Universal multi-model claims toned down or removed.
- [x] "Hybrid search" claims scoped to Hybrid Retrieval v1 with simple planner + RRF, without competitive parity.
- [x] SIFT1M labeled as a non-comparable benchmark for competitiveness while the engine remains cosine-only.
- [x] Architecture documentation rewritten to reflect the current product boundary.

## 2. Naming and technical consistency

- [x] Major legacy naming remnants removed from tests and public descriptions.
- [x] The stable SDK boundary is documented as `src/sdk.rs`.
- [x] The Python package has CI for wheels/TestPyPI prepared, but does not promise production PyPI.

## 3. Observability and metrics

- [x] Memory telemetry contract documented.
- [x] Process metrics separated from logical index metrics.
- [x] The repo makes explicit which metrics are reliable and which remain experimental.
- [x] Controlled memory harness added for cold start, ingestion, replay, and restart.

## 4. Reliability gate

- [x] `durability_recovery`
- [x] `index_reconstruction`
- [x] `backend_tests`
- [x] `memory_telemetry`
- [x] `python_sdk_boundary`
- [x] Python SDK smoke
- [x] `pytest vantadb-python/tests/test_sdk.py -v`

## 5. Explicitly deferred work

- [x] Production PyPI and signing remain outside this cycle.
- [x] Public ranking/debug, rich snippets, and competitive claims remain outside this cycle.
- [x] First-class namespaces and canonical model move to the next MVP block.

## 6. Next active block

- [x] Canonical memory model separated from `UnifiedNode`.
- [x] First-class namespaces with `namespace + key`.
- [x] Minimal API `put/get/delete/list/search`.
- [x] Python SDK flow for persistent memory.
- [x] Minimal embedded CLI `put/get/list`.

## 7. Operational block memory-mvp-core

- [x] Manual ANN rebuild exposed in Rust SDK, Python SDK, and CLI.
- [x] JSONL export/import by namespace and full base.
- [x] Persistent derived indexes for namespace and scalar metadata filters.
- [x] Derived index rebuild from canonical records.
- [x] Brutality suite covering recovery, index loss, export/import, and 10K record smoke test.

## 8. Remaining open limits

- [x] Optimize derived indexes with real iterator/prefix scans in the backend.
- [x] Add structured telemetry for `startup_ms`, `wal_replay_ms`, `wal_records_replayed`, `rebuild_ms`, `records_exported`, and `records_imported`.
- [x] Harden recovery of stale/corrupt derived indexes.
- [x] Document mutation protocol and recoverable versioning.
- [x] Design text index before implementing BM25/RRF.
- [x] Prepare wheels/TestPyPI without activating production PyPI publishing.
- [ ] Keep signing and production PyPI publishing outside the cycle until release policy is stabilized.

## 9. Next technical cut

- [x] Convert the text scaffold into a persistent, rebuildable inverted index.
- [x] Define BM25 text-only over the text index, without competitive claims yet.
- [x] Define minimal RRF/planner over lexical and vector rankings.
- [ ] Evaluate Euclidean/SIFT only as an enabler for serious benchmarking.

## 10. Hybrid v1 operational closeout

- [x] Restore source-of-truth tracker in `seguimiento de proyecto.csv`.
- [x] Document phase closeout in `docs/archive/TEXT_INDEX_PHASE_1_CLOSEOUT.md`.
- [x] Enable `text_query` text-only and hybrid v1 with minimal RRF/planner.
- [x] Harden Hybrid v1 with certification, deterministic corpus, and internal planner/RRF debug.
- [x] Add text positions v3, basic phrase query, explain/snippet debug-only, wheel CI, and hybrid benchmark on real embedded corpus.
- [x] Expose read-only structural audit of text index in Rust/Python SDK and CLI.
- [x] Create `docs/operations/ROADMAP.md` as the operational process roadmap.

## 11. Next operational hardening

- [x] Document that JSONL export/import is not a physical backup.
- [x] Validate restore via cold copy for the default Fjall backend.
- [ ] Keep TestPyPI as a manual gate before any production PyPI release.
- [ ] Open Search Quality v2 only after operational closeout: analyzer, Unicode folding, stopwords/stemming, and public snippets remain deferred.
