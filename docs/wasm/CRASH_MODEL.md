# WASM Crash and Durability Model

> Last reviewed: 2026-08-22 — sync post-PERF-08 (differential persistence, `vantadb-wasm/src/lib.rs`).

## Persistence Model

- Data lives in WASM linear memory until explicitly saved.
- Since PERF-08, `save()` (OPFS) and `save_idb()` (IndexedDB) use **differential persistence** via `PersistCache` (`vantadb-wasm/src/lib.rs:265-299`). Only records whose `version` changed — or that were deleted — since the last successful persist are re-serialized; unchanged records reuse their cached JSON string (`persist_payload`, `vantadb-wasm/src/lib.rs:661-720`).
- **What is serialized:** the dirty set (`put`/`delete` and other mutation entry points feed it) plus deletions. If nothing changed since the last persist, the file write is skipped entirely (`vantadb-wasm/src/lib.rs:743`, `:761`). A failed write invalidates the cache so the next save does a full rebuild instead of silently skipping (`vantadb-wasm/src/lib.rs:736-739`, `:756-757`).
- **Bulk operations** whose changed keys are not individually known (import/bulk/reindex/purge) mark the cache invalid (`cache_invalid`) and force a one-time full rebuild + rewrite on the next save (`vantadb-wasm/src/lib.rs:276-288`, `:671-684`).
- The output is always a complete `db_state.json`: a valid `Vec<VantaMemoryRecord>` JSON array, byte-for-byte loadable by `load()`/`load_idb()`. "Differential" refers to serialization cost only — there is no append-only delta log; whenever anything changed, the whole snapshot file is rewritten.
- No WAL, no crash recovery, no `fsync` guarantees.
- `load()` / `load_idb()` parse `db_state.json` back into memory via `import_records()` and seed the cache from the loaded snapshot, so the next `save` is a no-op unless a mutation occurs (`populate_cache_from_records`, `vantadb-wasm/src/lib.rs:625-650`).

## Storage Backend Comparison

| Backend | API | Cross-Tab | Atomic Writes | Availability |
|---------|-----|-----------|---------------|-------------|
| **InMemory** | `new VantaDB()` / `VantaDB.open()` | N/A | N/A | Always |
| **OPFS** (`connect_persistent`) | `FileSystemFileHandle` + `createWritable` | ❌ None | ❌ Not atomic | `navigator.storage` check |
| **IDB** (`connect_idb`) | IndexedDB + BroadcastChannel | ✅ `"vantadb-sync"` channel | ⚠️ Atomic per `put` | `globalThis.indexedDB` check |
| **Worker** (`connect_worker`, feature-gated) | `MessageChannel` → OPFS in Worker | ❌ None | ❌ Not atomic | Feature flag `opfs` |

## Crash Scenarios

| Scenario | Data Loss | Mitigation |
|----------|-----------|------------|
| Tab reload without `save()` | **All unsaved data lost.** Nothing is persisted until `save()` / `save_idb()` is called. | Auto-save via `beforeunload` listener (application-side). |
| Browser crash during `save()` (OPFS) | **`db_state.json` may be corrupt.** `createWritable` + `write` + `close` is not atomic. | Future: atomic rename (`write tmp → move`) + checksum footer. |
| Browser crash during `save_idb()` (IDB) | **Low risk.** IndexedDB transactions are atomic per `put`. The single-key model means the write either completes or doesn't. | No further mitigation needed at the storage layer. |
| Two tabs writing to OPFS simultaneously | **Silent corruption.** OPFS has no locking or notification mechanism. Both tabs write `db_state.json` independently; last write wins with no merge. | Use `connect_idb()` for multi-tab scenarios. Future: Web Locks API. |
| Two tabs writing to IDB simultaneously | **Race condition.** BroadcastChannel notifies peers, but no lock prevents concurrent writes. BroadcastChannel is informational only. | Future: `navigator.locks.request("vantadb-write")` to serialize writes. |
| Worker death during write | **Unknown file state.** The main thread receives a timeout error (5s), but the state of the file on disk is indeterminate (OPFS write may be partial). | Future: checksum verification on `load()`. Exponential backoff retry mitigates transient worker failures. |
| `db_state.json` read failure on `load()` | **Graceful degradation.** Missing file = empty state (no-op). Corrupt JSON = deserialization error propagated to caller. | Applications should handle `load()` errors and fall back to empty state. |

## What Survives a Crash

| Operation | After tab reload + `load()` |
|-----------|---------------------------|
| Insert followed by `save()` | ✅ Data restored |
| Insert without `save()` | ❌ Data lost |
| `save()` during mid-write crash | ❌ File corrupt (OPFS) / ✅ Atomic (IDB) |
| Multi-tab insert with `connect_idb()` | ⚠️ Last tab to save wins. Other tabs' changes are silently overwritten. |

## Best Practices for Users

1. **Call `save()` / `save_idb()` explicitly after meaningful mutations.** There is no auto-save. Data exists only in WASM linear memory until you persist it.

2. **Use `connect_idb()` for multi-tab scenarios.** The IDB backend has `BroadcastChannel("vantadb-sync")` notifications. OPFS (`connect_persistent()`) has no cross-tab mechanism — two tabs writing to the same OPFS storage will corrupt each other.

3. **Single-tab users: `connect_persistent()` (OPFS) is simpler** — no dependency on IndexedDB, cleaner API. Worker backend (`connect_worker()`) is for offloading I/O off the main thread.

4. **Handle `load()` errors.** If `db_state.json` is corrupt (e.g., from a crash during OPFS write), `load()` returns a `JsValue` error. Catch it and initialize an empty database.

5. **Register a `beforeunload` handler** in your application to call `save()` on tab close. Note that `beforeunload` cannot reliably run async operations in all browsers — consider periodic auto-save as a fallback.

6. **Know the scale limits.** Since PERF-08, `save()` serializes only changed records, so steady-state saves are cheap. Bulk operations (import/reindex/purge) still trigger a full rebuild + rewrite, and a save that touches many records can still block the event loop for seconds on large datasets. Batch mutations and call `save()` once.

## Known Gaps (not covered by differential persistence)

Differential persistence changes *when* work happens, not the durability model. These gaps are tracked in [WASM_STANDALONE.md](../api/WASM_STANDALONE.md) ("Known limits (verified)"):

- **OPFS requires a secure context** — only served over `https` or `http://127.0.0.1`/localhost.
- **Storage quotas are browser-managed** — Chromium ~60% of disk, Firefox 2 GB, Safari ~1 GB per origin.
- **Persistence round-trip gap**: `put` metadata → `get`/`list` may return `metadata: {}` in the WASM in-memory open (`ShreddedRowStore`), and IQL reads resolve against the graph store rather than memory records (WASM-02 open items).
- The crash scenarios below remain accurate as stated: no WAL, no atomicity on OPFS writes, and no cross-tab locking. Differential persistence does not address any of them.

## Related Documentation

- [WASM Storage Review](../architecture/WASM_STORAGE_REVIEW.md) — Full gap analysis with recommendations
- [ADR-008](../../docs/architecture/adr/008_wasm_support_strategy.md) — WASM architecture decisions
- `vantadb-wasm/src/opfs.rs` — OPFS backend implementation
- `vantadb-wasm/src/idb.rs` — IndexedDB backend implementation
- `vantadb-wasm/src/worker.rs` — Worker bridge implementation
