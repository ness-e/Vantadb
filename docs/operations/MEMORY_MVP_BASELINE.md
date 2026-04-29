# Memory MVP Core Baseline

Date: 2026-04-29

## Baseline Name

`memory-mvp-core`

## Implemented Surface

- Rust SDK: `put`, `get`, `delete`, `list`, `search`, `list_namespaces`, `rebuild_index`, `export_namespace`, `export_all`, `import_records`, `import_file`.
- Python SDK: `put`, `get_memory`, `delete_memory`, `list_memory`, `search_memory`, `rebuild_index`, `export_namespace`, `export_all`, `import_file`.
- CLI: `put`, `get`, `list`, `rebuild-index`, `export`, `import`.

## Data Contract

- Identity: deterministic `namespace + "\0" + key` hash.
- Payload: UTF-8 text.
- Metadata: scalar `VantaValue` values only.
- Vector: optional `Vec<f32>`.
- Canonical storage remains `UnifiedNode` internally; public consumers use SDK memory types.

## Derived State

- ANN index is derived from VantaFile/storage and can be rebuilt manually.
- `NamespaceIndex` maps `namespace + NUL + key` to `node_id`.
- `PayloadIndex` maps `namespace + NUL + field + NUL + encoded_scalar_value + NUL + key` to `node_id`.
- Derived indexes are rebuilt from canonical records during `rebuild_index`.

## Validation Evidence

- `cargo test --test memory_api --test memory_export_import --test derived_indexes`
- `cargo test --test memory_brutality -- --nocapture`
- `python -m pytest vantadb-python/tests/test_sdk.py -v`
- CLI smoke: `put`, `list`, `export`, `import`, `get`, `rebuild-index`

The brutality suite covers recovery without explicit flush, manual rebuild after deleting `vector_index.bin`, JSONL export/import, and a 10K-record namespace/filter/export/import smoke.

## Explicit Limits

- Search remains cosine vector + structured filters. BM25/RRF is not implemented.
- Derived index scans currently materialize the index partition through the backend abstraction; prefix iterators are a later optimization.
- JSONL export/import is an operational interchange format, not a full backup system with checksums or transactional snapshots.
- PyPI, wheels, signing, server mode hardening, and enterprise features remain outside this baseline.
