# WASM Storage Review — 2026-07-22

## Scope

Review of all storage backends used when VantaDB runs in a browser WASM environment
(`wasm32-unknown-unknown` target). Covers persistence, delete, append, multi-tab
coordination, crash consistency, test coverage, and documentation.

---

## Storage Backends

### 1. InMemoryBackend (core engine)

| Field | Value |
|-------|-------|
| **Defined in** | `vantadb/src/backends/in_memory.rs` |
| **Enabled by** | `vantadb-wasm/src/lib.rs:65` — hardcodes `BackendKind::InMemory` |
| **What it stores** | All KV partitions in `BTreeMap`s inside WASM linear memory |
| **Persistence** | None. All data lost when WASM instance is dropped (tab reload/navigate) |
| **WAL** | Skipped — `src/storage/engine/init.rs:41-45`: no WAL, no VantaFile for InMemory |
| **Usage** | `new VantaDB()`, `VantaDB.open()`, `VantaDB.create()` — always in-memory |

### 2. OpfsStorage (OPFS file-based persistence)

| Field | Value |
|-------|-------|
| **Defined in** | `vantadb-wasm/src/opfs.rs` |
| **Enabled by** | `connect_persistent()` in `vantadb-wasm/src/lib.rs:294` |
| **What it provides** | `read_file`, `write_file`, `delete_file`, `append_file` |
| **How persistence works** | `save()` serializes ALL in-memory records to `db_state.json` as one JSON blob via `opfs.write_file`. `load()` reads `db_state.json` back into memory |
| **Granularity** | Full dump only — no incremental persistence |
| **Detection** | `OpfsStorage::is_available()` checks `navigator.storage` |
| **API surface** | Uses `FileSystemFileHandle`, `createWritable`, `getFile`, `arrayBuffer` |

### 3. IdbStorage (IndexedDB persistence, OPFS fallback)

| Field | Value |
|-------|-------|
| **Defined in** | `vantadb-wasm/src/idb.rs` |
| **Enabled by** | `connect_idb()` in `vantadb-wasm/src/lib.rs:314` |
| **How persistence works** | Same `db_state.json` key. `save_idb()` writes all records to IDB, `load_idb()` reads them back |
| **Inline JS bridge** | `#[wasm_bindgen(inline_js = r#"..."#)]` at line 5-54 — injects IndexedDB + BroadcastChannel JS at wasm-bindgen registration time |
| **BroadcastChannel** | Named `"vantadb-sync"`. Sends `{type: "data-changed", key}` on write/delete |
| **Subscribe API** | `IdbStorage::subscribe(cb)` returns an unsubscribe closure |
| **Detection** | `IdbStorage::is_available()` checks `globalThis.indexedDB` |

### 4. OpfsWorkerProxy (Web Worker bridge)

| Field | Value |
|-------|-------|
| **Defined in** | `vantadb-wasm/src/worker.rs` (proxy), `vantadb-wasm/src/opfs_bridge.js` (worker spawn) |
| **Enabled by** | `connect_worker()` in `vantadb-wasm/src/lib.rs:334` (feature-gated: `#[cfg(feature = "opfs")]`) |
| **How it works** | JS `spawnOpfsWorker()` creates a Worker from a Blob URL. Rust `OpfsWorkerProxy` sends `WorkerRequest` messages via `MessageChannel` |
| **Message protocol** | Init / Read / Write / Append / Delete with typed responses and timeout (5s) |
| **Retry** | Exponential backoff: 1s, 2s, max 2 retries on timeout/abort errors |
| **State** | `#[cfg(feature = "opfs")]` — gated, not compiled by default |
| **Usage** | Worker reads `db_state.json` on init. Same full-dump persistence |

---

## Gap Analysis

| Concern | Status | Evidence |
|---------|--------|----------|
| **Delete semantics** | ✅ OPFS | `OpfsStorage::delete_file` handles `NotFoundError` gracefully (no-op). `OpfsFile::delete` calls `remove()`. |
| | ✅ IDB | `IdbStorage::delete_file` -> `objectStore.delete(key)` + BroadcastChannel notification. |
| | ⚠️ Incremental delete | Delete at record level works in memory. But persistence is full-dump only (`db_state.json`). No incremental delete to storage — every `save()` rewrites everything. For 1M records, deleting one record still rewrites 1M. |
| **Append semantics** | ✅ OPFS | `OpfsFile::append()` with `{keepExistingData: true}`. Worker protocol has `Append` message type. |
| | ❌ IDB | No append API. IDB `put` replaces the entire value. The `db_state.json` single-blob model doesn't support append. |
| | ❌ Persistence path | `save()`/`save_idb()` always replace the entire file — no incremental append to storage. The 'append' capability exists at the OPFS/file layer but is never used for the main persistence path. |
| **Multi-tab coordination** | ✅ IDB (partial) | BroadcastChannel `"vantadb-sync"` sends notifications on write/delete. `subscribe()` API exposed. `has_broadcast_channel()` detection. |
| | ❌ OPFS | No cross-tab coordination. OPFS has no built-in notification mechanism. Two tabs writing to the same OPFS storage will silently corrupt each other's data. |
| | ❌ Worker | No cross-tab coordination. |
| | ❌ Web Locks | Neither IDB nor OPFS backends use the Web Locks API for coordination. If two tabs write concurrently, last write wins with no merge or conflict detection. |
| | ❌ Stale in-memory state | `subscribe()` notifies a tab that data changed, but there's no mechanism to reconcile the in-memory state with the changed storage — the tab would need to `load()` and lose unsaved changes. |
| **Crash consistency** | ❌ OPFS | `write_file` -> `createWritable` + `write` + `close` is not atomic. If the browser crashes between `write` and `close`, the file is in an indeterminate state. No checksum, no versioning, no rollback. |
| | ⚠️ IDB | IDB transactions are atomic within a single `put`, but the single `db_state.json` key means no version management. No checksum, no recovery from partial state (though IDB's transactional nature makes partial writes less likely). |
| | ❌ Worker | Same OPFS crash issues + additional risk: if the worker dies mid-write, the main thread gets a timeout error but the state of the file is unknown. The 5s timeout could mask slow writes. |
| | ❌ InMemory | No persistence at all — data loss is the expected behavior. |
| | ❌ WAL | WASM mode explicitly skips WAL (`init.rs:41`). No write-ahead logging, no crash recovery, no durability guarantees. |
| **Dedicated tests** | ✅ OPFS | 6 distinct OPFS tests in `wasm_tests.rs`: read/write cycle, overwrite, nonexistent read, nonexistent delete, isolated directories, binary data, large file (10KB). |
| | ❌ IDB | Zero tests for `IdbStorage`. The inline JS bridge has no test coverage. |
| | ❌ Worker | Zero tests for `OpfsWorkerProxy` or the Worker message protocol. |
| | ❌ Persistence round-trip | No test that: create DB → put data → `save()` → `load()` → verify data survives. |
| | ❌ Crash consistency | No test for interrupted writes, partial file recovery, or checksum verification. |
| | ❌ Multi-tab | No test for BroadcastChannel notification delivery or multi-tab coordination. |
| | ❌ CI execution | `wasm_tests.rs` requires `wasm-pack test --chrome`. Not run in CI. Not run on Firefox/Safari. |
| **Documentation** | ⚠️ ADR-008 exists | Documents Phase 1 (InMemory) and Phase 2 (OPFS Future). However, the code has already surpassed the documented state: OPFS and IDB persistence exist, but ADR-008 still calls them "Future: OPFS Persistence (Phase 2)". |
| | ❌ ADR outdated | ADR-008 talks about `FileSystemSyncAccessHandle` and `SharedArrayBuffer` which aren't used. The actual implementation uses async handles and BroadcastChannel — undocumented. |
| | ❌ TS SDK docs misleading | `vantadb.ts:105` says "WASM backend always uses an in-memory engine", but `connect_persistent()` and `connect_idb()` exist and provide persistent backends. |
| | ❌ Storage differences | No documentation explaining how OPFS/IndexedDB semantics differ from filesystem storage (no `fsync`, no atomic rename, no directory fsync, quota limits). |
| | ❌ Crash model | No documented guarantees about what survives a crash/tab reload. Users have no way to know they must call `save()` explicitly. |
| | ❌ Multi-tab model | No documented behavior for multi-tab scenarios. |

---

## Recommendations

### P1 — Critical (data loss / correctness risk)

1. **Add IDB storage tests** — `IdbStorage::read_file`, `write_file`, `delete_file` have zero coverage. Write 5+ tests in `wasm_tests.rs` mirroring the OPFS coverage (read/write cycle, overwrite, nonexistent, binary, large).

2. **Add persistence round-trip test** — The most critical missing test: `save()` then `load()` and verify records survive. This is the primary user-facing persistence path.

3. **Fix ADR-008 to reflect reality** — The code has OPFS + IDB + Worker persistence. Update the ADR to document the actual implementation, not the planned one. Remove "Future Phase 2" language for features that exist today.

### P2 — High (correctness / DX)

4. **Add Web Locks coordination** — Use `navigator.locks.request()` when the IDB backend writes to prevent multi-tab corruption:
   ```rust
   // In JS bridge: navigator.locks.request("vantadb-write", () => {
   //   tx.objectStore(STORE_NAME).put(data, key);
   // });
   ```
   This prevents concurrent writes from different tabs. Backlog reference: `NUEVO-12`.

5. **Add OPFS crash resilience** — Before writing `db_state.json`, write to `db_state.tmp` first, then atomically rename (OPFS supports `move()`). Add a checksum (SHA-256 footer) so corrupted files are detected on `load()`.

6. **Fix misleading TS SDK docs** — Line 105 of `vantadb.ts` claims WASM is always in-memory. Remove or update this comment — `connect_persistent()`, `connect_idb()`, and `connect_worker()` all provide persistence.

7. **Document crash model** — Add a section in `docs/architecture/` or `docs/api/TS_SDK.md` explaining:
   - Data is in-memory until `save()` / `save_idb()` is called
   - `save()` replaces the entire snapshot — not incremental
   - Crash during `save()` may cause data loss
   - OPFS is not `fsync`-equivalent
   - Multi-tab: `connect_idb()` has BroadcastChannel notifications; `connect_persistent()` has none

### P3 — Medium (testing quality)

8. **Implement Worker tests** — Add `wasm_bindgen_test` tests for `OpfsWorkerProxy` that exercise the full message protocol (init, read, write, append, delete, timeout, error recovery).

9. **Consolidate duplicate tests** — ~15 tests are identical between `lib.rs` and `wasm_tests.rs` (DRV-042). Keep them in `wasm_tests.rs` (external integration tests) and remove from `lib.rs` to reduce maintenance burden.

10. **Run WASM tests in CI** — Add a CI workflow step for `wasm-pack test --chrome --firefox`. Currently these tests only run manually. Use `headless` mode.

### P4 — Low (nice to have)

11. **Incremental persistence** — Instead of serializing ALL records on every `save()`, implement a WAL-like append log (each mutation is appended to a file, and a background compact merges into the main snapshot). This is the path toward OPFS being a true storage backend rather than a dump/restore layer.

12. **VantaFile/vector store for WASM** — Currently `VantaFile::create_in_memory(64 * MIB)` is used for the vector store in WASM mode. For large datasets, consider an OPFS-backed vector store that pages vectors to browser storage rather than keeping all vectors in linear memory.

---

## Summary

```
                     ┌──────────────────────┐
                     │   VantaDB WASM        │
                     │   (wasm32-unknown-    │
                     │    unknown)           │
                     └──────┬───────────────┘
                            │ BackendKind::InMemory
                            ▼
              ┌─────────────────────────────┐
              │   InMemoryBackend           │ ← All KV in WASM heap
              │   No WAL, no persistence    │
              │   except explicit save()    │
              └──────────┬──────────────────┘
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
     ┌──────────┐ ┌──────────┐ ┌──────────────┐
     │OPFS      │ │IDB       │ │Worker        │
     │(opfs.rs) │ │(idb.rs)  │ │(worker.rs)   │
     │File-based│ │KV store  │ │MessageChan   │
     │┌────────┐│ │+ BCastCh │ │+ retry       │
     ││append  ││ │+ sub     │ │┌────────────┐│
     ││no atom ││ │no atom   │ ││OPFS proxied││
     │└────────┘│ │└─────────┘│ │└────────────┘│
     └──────────┘ └──────────┘ └──────────────┘
```

**Key risks (ordered by severity):**

1. ❌ **IDB backend has zero tests** — the inline JS bridge is untested and could break silently
2. ❌ **No persistence round-trip test** — the primary user flow (save → load) is untested
3. ❌ **OPFS crash writes can corrupt data** — no atomic rename, no checksum, no rollback
4. ❌ **No Web Locks** — two tabs with OPFS storage will corrupt each other
5. ⚠️ **ADR-008 is factually outdated** — documents Phase 1 as current, but Phase 2 features exist
