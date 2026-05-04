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
- Prefix-scan-backed derived index lookups
- Persistent inverted text index for memory payload postings, derived from canonical records
- BM25 lexical retrieval for text-only memory `text_query`
- Hybrid Retrieval v1 for memory search using simple planner + RRF over BM25 and vector rankings
- Operational metrics for startup, WAL replay, rebuild, text-index rebuild/repair, lexical text queries, hybrid queries, planner routes, export, import, and import errors
- Stale/corrupt derived-index state repair on open
- Stale/corrupt text-index state repair on writable open
- Source-install Python binding through a stable embedded boundary

## Claims intentionally deferred

- Universal multimodel database
- Advanced hybrid ranking, learned fusion, ranking explanations, or competitive hybrid-search parity claims
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

The product-building cycle no longer starts with the memory MVP primitives,
text-index persistence substrate, BM25 text-only retrieval, or simple RRF
hybrid retrieval; those are implemented in the repo. The next product cycle can
focus on search quality and distribution without inflating public claims.

Euclidean support remains a benchmark-enabling task, not a public product claim.

## Current execution state

- Canonical memory records are represented as SDK-level types, not as public `UnifiedNode`.
- Namespaces are first-class at the SDK boundary via `namespace + key`.
- The embedded SDK exposes `put`, `get`, `delete`, `list`, `search`, and `list_namespaces`.
- Python exposes memory-specific methods while preserving legacy node methods.
- Manual rebuild is exposed as `rebuild_index` in Rust/Python and `vanta-cli rebuild-index`.
- JSONL export/import is exposed in Rust/Python and `vanta-cli export/import`.
- Derived namespace and payload indexes are persisted and rebuilt from canonical records.
- Derived namespace and payload lookups use backend prefix scans.
- Derived-index state is validated on open and repaired from canonical records when missing, corrupt, or stale.
- Text-index postings are persisted in a dedicated backend partition and rebuilt from canonical payloads.
- Text-index postings store TF and small derived BM25 stats for DF, document length, and namespace corpus length.
- Text-index state is validated on writable open and repaired from canonical records when missing, corrupt, incompatible, or count-stale.
- Text-only `text_query` executes BM25 lexical retrieval with metadata filters and deterministic ordering.
- Hybrid `text_query + query_vector` executes both rankings and fuses them with RRF under a minimal planner.
- Operational metrics are exposed through Rust/Python SDK.
- The CLI is embedded-first for `put/get/list/rebuild-index/export/import` and no longer requires a local server for the first useful memory flow.
- Public text-only `text_query` and simple hybrid text+vector retrieval are enabled.

## Current validation evidence

- `cargo test --test memory_api --test memory_export_import --test derived_indexes`
- `cargo test --test derived_index_prefix_scan --test derived_index_recovery --test operational_metrics`
- `cargo test text_index --lib`
- `cargo test --test text_index_recovery`
- `cargo test --test memory_brutality -- --nocapture`
- `python -m pytest vantadb-python/tests/test_sdk.py -v`
- `memory_brutality` includes recovery without explicit flush, vector-index deletion plus manual rebuild, JSONL export/import, and a 10K-record namespace/filter/export/import smoke.
