# Next 5 Tasks: Operational Memory MVP

This file is the repo-side mirror of the active task board for the current MVP block.

## Completed gate tasks

- Repo narrative aligned to embedded persistent memory + vector retrieval.
- Memory telemetry contract and controlled harness added.
- Fjall remains the default backend, with RocksDB kept as optional fallback.
- Embedded SDK boundary is stable enough for local Python source-install flows.
- Reliability gate passed for durability, reconstruction, backend parity, memory telemetry, Rust SDK boundary, Python smoke, and Python SDK pytest.
- Canonical persistent memory model, namespaces, memory CRUD/search, Python memory flow, and embedded CLI `put/get/list` are implemented.

## Completed operational tasks

- [x] Close baseline `memory-mvp-core` in repo docs and tests.
- [x] Add explicit ANN rebuild through Rust SDK, Python SDK, and CLI.
- [x] Add JSONL memory export/import by namespace and full database.
- [x] Add derived namespace and payload indexes for namespace listing and equality filters.
- [x] Add brutality/KPI tests covering recovery, rebuild, export/import, and 10K records.
- [x] Add persistent BM25-ready text-index stats and text-only lexical retrieval.
- [x] Add Hybrid Retrieval v1 with simple planner and RRF fusion.
- [x] Harden Hybrid Retrieval v1 with deterministic certification corpus and debug-only planner/RRF report.

## Current product surface

- Rust SDK: `put/get/delete/list/search/list_namespaces/rebuild_index/export_namespace/export_all/import_records/import_file`.
- Python SDK: `put/get_memory/delete_memory/list_memory/search_memory/rebuild_index/export_namespace/export_all/import_file`.
- CLI: embedded `put/get/list/rebuild-index/export/import`.
- Search supports vector-only, BM25 text-only, and hybrid text+vector retrieval.
- Hybrid retrieval uses a minimal planner and RRF over independently ranked text/vector candidates.
- Debug builds expose internal hybrid plan certification for tests; it is not a stable SDK API.
- Text-index postings and BM25 stats for payload are persisted internally and maintained as a derived index.

## Known limits still accepted

- Derived indexes are persisted, reconstructible, and queried through backend prefix scans.
- Text-index postings/stats are persisted, reconstructible, and validated through state/count markers plus debug structural audit.
- Startup, WAL replay, ANN rebuild, derived rebuild, text-index rebuild/repair, lexical query, hybrid query, planner routing, export, import, and import errors have structured operational metrics.
- Derived-index state is validated on open and repaired from canonical records when stale or corrupt.
- Text-index state is validated on writable open and repaired from canonical records when stale or corrupt.
- Export/import is JSONL v1 and intentionally simple; it is not a migration framework or backup format with checksums.
- Text-only lexical search and simple hybrid text+vector search are wired into public memory search.
- The server wrapper is not the primary product boundary.

## Deferred tasks

- Public ranking explanations and advanced hybrid debug output.
- Euclidean/SIFT competitive benchmark validation.
- PyPI/wheels/signing.
- Server wrapper decision.
- Phrase queries, snippets, positions, and tokenizer evolution beyond `lowercase-ascii-alnum`.
