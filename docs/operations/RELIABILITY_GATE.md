# Reliability Gate

This note closes the current repo-alignment cycle.

## Allowed claims now

- Embedded persistent memory engine
- WAL-backed recovery
- HNSW vector retrieval using cosine similarity
- Structured metadata stored with records
- Derived ANN reconstruction from canonical persisted state
- Manual ANN rebuild from canonical VantaFile/storage state
- JSONL export/import for persistent memory records
- Namespace-scoped list/search with persisted derived indexes for namespace and equality metadata filters
- Source-install Python binding through a stable embedded boundary

## Claims intentionally deferred

- Universal multimodel database
- True hybrid text + vector retrieval with BM25/RRF
- Competitive parity claims on SIFT1M while cosine-only
- PyPI-ready distribution
- Enterprise, HA, RBAC, or managed-cloud positioning

## Trusted metrics now

- Process-scoped certification memory using the explicit telemetry contract
- Logical HNSW estimate reported by benchmark code
- Recall and latency on synthetic cosine-based validation datasets
- Recovery and reconstruction pass/fail behavior

## Metrics that remain non-decision-grade for marketing

- SIFT1M recall as a competitive benchmark
- Any single process-memory number interpreted as total engine footprint
- mmap/page-cache behavior presented as minimal RAM without extra validation

## Next cycle cut

The product-building cycle no longer starts with the memory MVP primitives; those are implemented in the repo. The next product cycle starts with:

1. prefix-iterator optimization and stale-index recovery hardening
2. structured startup/WAL/rebuild telemetry
3. textual index design before BM25/RRF

Euclidean support is a benchmark-enabling task, not a public product claim for this cycle.

## Current execution state

- Canonical memory records are represented as SDK-level types, not as public `UnifiedNode`.
- Namespaces are first-class at the SDK boundary via `namespace + key`.
- The embedded SDK exposes `put`, `get`, `delete`, `list`, `search`, and `list_namespaces`.
- Python exposes memory-specific methods while preserving legacy node methods.
- Manual rebuild is exposed as `rebuild_index` in Rust/Python and `vanta-cli rebuild-index`.
- JSONL export/import is exposed in Rust/Python and `vanta-cli export/import`.
- Derived namespace and payload indexes are persisted and rebuilt from canonical records.
- The CLI is embedded-first for `put/get/list/rebuild-index/export/import` and no longer requires a local server for the first useful memory flow.

## Current validation evidence

- `cargo test --test memory_api --test memory_export_import --test derived_indexes`
- `cargo test --test memory_brutality -- --nocapture`
- `python -m pytest vantadb-python/tests/test_sdk.py -v`
- `memory_brutality` includes recovery without explicit flush, vector-index deletion plus manual rebuild, JSONL export/import, and a 10K-record namespace/filter/export/import smoke.
