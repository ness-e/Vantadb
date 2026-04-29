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

## Current product surface

- Rust SDK: `put/get/delete/list/search/list_namespaces/rebuild_index/export_namespace/export_all/import_records/import_file`.
- Python SDK: `put/get_memory/delete_memory/list_memory/search_memory/rebuild_index/export_namespace/export_all/import_file`.
- CLI: embedded `put/get/list/rebuild-index/export/import`.
- Search remains vector + structured filters only. `text_query` still returns a clear BM25/RRF deferred error.

## Known limits still accepted

- Derived indexes are persisted and reconstructible, but prefix scans currently materialize the index partition through the backend abstraction.
- WAL replay metrics exist in recovery behavior tests, but structured startup/WAL timing telemetry is still a follow-up hardening item.
- Export/import is JSONL v1 and intentionally simple; it is not a migration framework or backup format with checksums.
- The server wrapper is not the primary product boundary.

## Deferred tasks

- BM25/RRF and planner-backed hybrid ranking.
- Euclidean/SIFT competitive benchmark validation.
- PyPI/wheels/signing.
- Server wrapper decision.
- Prefix-iterator optimization for derived indexes if larger datasets make materialized index scans too expensive.
