# Mutation and Recovery Protocol

Date: 2026-04-29

## Canonical Mutation Order

Persistent memory mutations follow this order:

1. Append the mutation to WAL.
2. Write canonical record state to VantaFile and backend metadata.
3. Update ANN derived state.
4. Update namespace/payload derived indexes.
5. Update text-index postings derived from memory payload.
6. Flush/persist when requested by caller or close path.

Canonical record state remains the source of truth. ANN, namespace indexes,
payload indexes, and text indexes are derived materializations.

## Operation Semantics

- `put`: upserts by deterministic `namespace + "\0" + key`, preserves `created_at_ms`, increments `version`, replaces stale payload-index entries for changed metadata, replaces stale text postings for changed payload, and updates derived-index state counts.
- `delete`: tombstones/removes the canonical record through the engine delete path and removes namespace/payload index entries plus text postings for the previous record.
- `import_records`: imports through exact memory records, preserves exported timestamps/version, updates existing records by identity, and rebuilds derived indexes before returning.
- `rebuild_index`: rebuilds ANN from canonical VantaFile/storage and then rebuilds derived namespace/payload indexes and text-index postings from canonical records.
- `open`: validates derived-index state and text-index state, then rebuilds derived indexes if a state marker is missing, corrupt, schema-incompatible, or count-inconsistent.

## Failure Behavior

- If ANN artifact is missing, startup or manual rebuild reconstructs it from canonical VantaFile/storage.
- If derived-index state is missing or corrupt, startup reconstructs namespace/payload indexes from canonical records.
- If text-index state is missing, corrupt, incompatible, or count-stale, writable startup reconstructs text postings from canonical memory payloads.
- If a prefix lookup finds no candidates and no valid derived-index state exists, the SDK may fall back to canonical scan as a recovery mode.
- If JSONL import contains invalid lines, valid records are still imported and errors are counted in the import report and operational metrics.

## Explicit Limits

- WAL replay metrics are operational diagnostics, not marketing claims.
- JSONL export/import is an interchange flow, not a transactional backup system.
- Text query execution remains disabled until BM25/RRF and planner behavior are implemented.
