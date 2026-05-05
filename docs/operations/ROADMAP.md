# VantaDB Operational Roadmap

This roadmap is the repo-side source of truth for the next implementation
steps. External research and audit reports are inputs for planning, but they
are not copied into this document and are not release evidence by themselves.

## Current Baseline

Implemented and covered in the repo:

- Embedded persistent memory records with canonical `namespace + key` identity.
- WAL-backed recovery and ANN rebuild from canonical persisted state.
- Fjall as the default KV backend, with RocksDB retained as an explicit
  fallback.
- Namespace and metadata-filter derived indexes rebuilt from canonical records.
- Persistent text index schema v3 with TF, positions, DF, document length, and
  namespace corpus stats.
- BM25 text-only memory search, basic quoted phrase filters, vector-only HNSW
  search, and Hybrid Retrieval v1 with RRF.
- Rust SDK, Python SDK, and embedded CLI flows for memory CRUD, search,
  rebuild, JSONL export/import, and text-index audit.
- Python wheel CI with manual TestPyPI gate; production PyPI publication and
  signing remain deferred.

## Phase 1: Operational Truth and Auditability

Goal: keep the public repo aligned with the real implementation and make
derived text-index drift diagnosable without mutating state.

Acceptance gate:

- README, architecture, operations docs, Python metadata, and changelog reflect
  BM25, phrase query support, Hybrid Retrieval v1, and current limits.
- `VantaEmbedded::audit_text_index(namespace)` reports structural text-index
  health without repair.
- `vanta-cli audit-index --db <path> [--namespace <ns>] [--json]` runs in
  read-only mode and exits non-zero on drift.
- Text-index audit works on read-only opens and recommends `rebuild-index`
  instead of repairing.

## Phase 2: Backup and Restore Hardening

Goal: make backup behavior explicit by backend and validate restore paths that
operators can actually use.

Acceptance gate:

- Fjall cold-copy restore is tested after the database is closed.
- RocksDB native checkpoint remains documented as backend-specific behavior.
- JSONL export/import is documented as logical data movement, not a physical
  backup with snapshot semantics.
- Restore validation includes canonical records, BM25/phrase text search, and
  hybrid search.

## Phase 3: Python Release Engineering

Goal: make Python distribution repeatable without prematurely claiming a
production PyPI release.

Acceptance gate:

- `vantadb-python/pyproject.toml` points to the real repository.
- Wheel validation installs the generated wheel by resolved path.
- TestPyPI upload remains manual and secret-gated.
- Production PyPI publication, signing, and installer policy stay deferred
  until the release process is verified.

## Phase 4: Search Quality v2

Goal: improve retrieval quality after the operational baseline is stable.

Candidate work:

- Analyzer pipeline design for tokenizer evolution.
- Unicode folding, stopwords, stemming, and richer language handling.
- Stable ranking explanation or snippets only if exposed through additive APIs.
- Planner improvements such as weighted RRF or cardinality-aware budgets.

Explicitly deferred until this phase or later:

- Phrase search beyond exact consecutive token filters.
- Rich highlighting/snippets as public output.
- Competitive hybrid-search parity claims.
- Market benchmark claims from SIFT or Euclidean work.

## Validation Commands

```powershell
cargo fmt --check
cargo test text_index --lib
cargo test --test memory_api --test memory_export_import --test derived_indexes --test derived_index_recovery --test derived_index_prefix_scan --test operational_metrics --test text_index_recovery
cargo test --test memory_brutality -- --nocapture
python -m maturin build --manifest-path .\vantadb-python\Cargo.toml --out .\target\wheels
$wheel = (Get-ChildItem .\target\wheels\vantadb_py-*.whl | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName; python -m pip install --force-reinstall $wheel
python -m pytest vantadb-python/tests/test_sdk.py -v
```

## Deferred Claims

Do not describe VantaDB as an enterprise database, managed cloud, universal
multimodel platform, PyPI-ready production package, or competitive hybrid
search product. The current claim is narrower: embedded persistent memory with
vector retrieval, BM25 lexical retrieval, basic phrase filtering, and Hybrid
Retrieval v1 using deterministic RRF.
