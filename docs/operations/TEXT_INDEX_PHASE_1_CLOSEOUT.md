# Text Index Phase 1 Closeout

Date: 2026-05-04

Note: this document records the phase-1 state before BM25 and Hybrid Retrieval
v1. It is historical evidence, not the current product boundary. The current
architecture state is tracked in `docs/architecture/TEXT_INDEX_DESIGN.md`.

## Status

Closed for phase 1. VantaDB now has a persistent inverted index for memory
payload tokens. The index is internal, derived from canonical memory records,
and rebuilt or repaired from those records.

## Implemented

- Dedicated `text_index` backend partition.
- Posting key contract: `namespace + "\0" + token + "\0" + key`.
- Tokenizer: `lowercase-ascii-alnum`.
- One posting per unique token per memory record.
- Put/update/delete maintenance in the derived-index mutation batch.
- Manual rebuild through `rebuild_index`.
- Import rebuild after JSONL import.
- Writable-open repair when state is missing, corrupt, incompatible, or count-stale.
- Operational metrics: `text_index_rebuild_ms`, `text_postings_written`, `text_index_repairs`.

## Explicitly Deferred

- BM25 scoring.
- RRF fusion.
- Text/vector/filter planner.
- Ranking debug output.
- Public text search.

`text_query` remains disabled publicly because postings alone do not define a
ranking contract. Enabling it before BM25/RRF would expand the product boundary
beyond the implemented behavior.

## Validation Evidence

```powershell
cargo fmt --check
cargo test text_index --lib
cargo test --test memory_api --test memory_export_import --test derived_indexes --test derived_index_recovery --test derived_index_prefix_scan --test operational_metrics --test text_index_recovery
cargo test --test memory_brutality -- --nocapture
python -m maturin build --manifest-path .\vantadb-python\Cargo.toml --out .\target\wheels
python -m pip install --force-reinstall .\target\wheels\vantadb_py-0.1.0-cp38-abi3-win_amd64.whl
python -m pytest vantadb-python/tests/test_sdk.py -v
```

All listed checks passed locally. The Python wheel rebuild was required because
the installed `vantadb_py` extension was stale.

## Next Gate

The next technical task should start with BM25 base over the existing persistent
postings. It should not claim hybrid search until BM25, RRF, and planner
behavior are implemented and tested.
