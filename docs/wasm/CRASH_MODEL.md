# WASM Crash and Durability Model

## Persistence Model

- Data lives in WASM linear memory until explicitly saved.
- `save()` (OPFS) / `save_idb()` (IndexedDB) / worker `save()` serialize **ALL** in-memory records to `db_state.json` (full dump via `serde_json`).
- No incremental persistence — every save rewrites the entire state.
- No WAL, no crash recovery, no `fsync` guarantees.
- `load()` / `load_idb()` parse `db_state.json` back into memory via `import_records()`.

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

6. **Know the scale limits.** Full-dump persistence on every `save()` means write cost grows linearly with dataset size. At 500K+ records, `save()` may take several seconds. Batch mutations and call `save()` once.

## Related Documentation

- [WASM Storage Review](../../docs/WASM_STORAGE_REVIEW.md) — Full gap analysis with recommendations
- [ADR-008](../../docs/architecture/adr/008_wasm_support_strategy.md) — WASM architecture decisions
- `vantadb-wasm/src/opfs.rs` — OPFS backend implementation
- `vantadb-wasm/src/idb.rs` — IndexedDB backend implementation
- `vantadb-wasm/src/worker.rs` — Worker bridge implementation
