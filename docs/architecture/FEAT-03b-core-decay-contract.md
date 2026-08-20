# FEAT-03b — Core Decay Implementation Contract

Status: **ready for vanta-worker** · Decision: **ADR-028**
Deliverable of FEAT-03b (plan Task 17, D16 "Quiero todo" — (b) core decay).
This is a contract only: the core changes below are to be implemented by
vanta-worker in a separate task, NOT by this task.

## Goal

Make the consolidation marker from FEAT-03a **durable and first-class in the
core**: record A can be marked as *superseded by* record B, and superseded
records can be excluded from search/list without being destroyed. Explicitly
**not** Mem0-style recency scoring, **not** automatic consolidation (see
ADR-028 Alternatives).

## API surface (final)

```rust
// src/sdk/types.rs — VantaMemoryRecord (additive, #[serde(default)]):
pub superseded_by: Option<String>,    // key of successor, same namespace
pub superseded_at_ms: Option<u64>,    // when supersession was recorded

// src/sdk/types.rs — VantaMemorySearchRequest (additive, default false):
pub exclude_superseded: bool,
// src/sdk/types.rs — VantaMemoryListOptions (additive, default false):
pub exclude_superseded: bool,

// src/sdk/api.rs — VantaEmbedded:
pub fn supersede(&self, namespace: &str, old_key: &str, new_key: &str) -> VantaResult<()>
```

P2 (cuttable): `supersedes: Option<String>` on `VantaMemoryInput` for ingest-time
linking (`put` marks the old record as superseded by the new one). Cut if it
complicates the put signature; the explicit `supersede()` method covers the UI
flow.

## Implementation steps (in order)

1. **Field constants** — `src/sdk/serialization/mod.rs`: add
   `FIELD_SUPERSEDED_BY: &str = "__vanta_superseded_by"` and
   `FIELD_SUPERSEDED_AT_MS: &str = "__vanta_superseded_at_ms"` next to the other
   `FIELD_*` consts (lines 14-26).
2. **Serialize/deserialize** — same file:
   - `memory_record_from_node` (≈L309): read both fields as
     `Option<String>`/`Option<u64>`; add to the `fields.remove(...)` block (≈L317).
   - `node_from_record`/record→node path (≈L405): write both fields when `Some`.
   - Ensure user metadata can never collide: `validate_metadata` already rejects
     `__vanta_` prefix? — **verify**; if not, extend the rejection list.
3. **Types** — `src/sdk/types.rs`: add the two fields to `VantaMemoryRecord`
   (near L200, with `#[serde(default)]`), and `exclude_superseded: bool` to
   `VantaMemorySearchRequest` (near L209) and `VantaMemoryListOptions`.
4. **`supersede()`** — `src/sdk/api.rs` (near `purge_expired`, L821):
   - Validate namespace/key; `old_key != new_key`.
   - `get(old)` → must exist and `superseded_by.is_none()` (idempotency guard).
   - `get(new)` → must exist (supersession links existing records, does not insert).
   - Write old with `superseded_by = new_key`, `superseded_at_ms = now`,
     version bumped — reuse the existing put/upsert path so WAL + derived
     indexes stay consistent.
   - Document partial-failure window: two WAL appends, not atomic; a crash
     between them leaves old marked with no impact on new. Add a `ponytail:`
     comment naming the ceiling (full atomicity deferred to ACID Phase 0).
5. **Read filter** — `src/sdk/search/` (materialization of hits) and
   `src/sdk/list_impl` (or wherever `list` assembles records): when
   `exclude_superseded` is set, drop hits whose record has `superseded_by.is_some()`.
   Apply at final assembly, not in the index query (no index change needed).
6. **Python bindings** — `vantadb-python/src/lib.rs`:
   - `supersede(namespace, old_key, new_key)` method.
   - Getters `superseded_by` / `superseded_at_ms` on the record wrapper
     (mirror `expires_at_ms`, vantadb-python/src/types.rs:120).
   - `search_memory` / `list_memory` accept `exclude_superseded`.
   - Async wrappers in `vantadb-python/vantadb_py/__init__.py`.
7. **CLI (optional, cheap)** — `src/cli_handlers/crud.rs` already prints record
   fields; surface the two new fields in get/list output. Cut if lead prefers
   Python-only exposure this phase.

## Tests (must land with the change)

- Serialization roundtrip: superseded fields survive `record → node → record`
  (extend existing tests in `src/sdk/serialization/mod.rs`).
- Backward compat: record dump *without* the fields deserializes to `None`
  (extend existing test at mod.rs:952-1016).
- `supersede()`: marks old, leaves new untouched, errors on missing keys,
  idempotency (second call errors — already superseded).
- Search/list: `exclude_superseded=true` hides superseded, `false` (default)
  keeps current behavior.
- Python: `supersede` + getters + `exclude_superseded` smoke test in
  `vantadb-python/tests/test_sdk.py`.

## Verification

```bash
cargo nextest run --profile audit --workspace --build-jobs 2
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
pytest vantadb-python/tests/test_sdk.py
```

## Out of scope (explicit)

- No search scoring changes; `last_accessed` is NOT written on search hits.
- No background/consolidation worker; no thresholds/config.
- No `exclude_expired` / TTL read-filter (separate follow-up, ADR-028 "Noted gap").
- No `desktop/` changes (FEAT-03a, parallel task).
- No migration script: serde defaults make existing data compatible.

## Risks

- Metadata collision if `validate_metadata` does not reject `__vanta_` (step 2
  verify).
- `supersede()` non-atomicity: accepted, documented (step 4).
- Per-hit filter cost: one `Option::is_some()` check per hit when flag set —
  negligible; benchmark only if search throughput regresses (Regla 8/9).

## Done when

- All steps merged with tests green (verification block), API surface matches
  this contract, Python smoke tests pass, ADR-028 status flipped to `accepted`
  by the lead after review.