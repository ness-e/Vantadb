---
title: WASM Persistence Documentation
type: api
status: active
tags: [vantadb, wasm, browser, persistence]
last_reviewed: 2026-08-19
aliases: [WASM_PERSISTENCE]
---

# WASM Persistence Documentation

How VantaDB persists data in the browser: what exists, what the verified
limits are, and where each mechanism applies.

## Storage backends

| Backend | Entry points | Persistence file/key | When to use |
| --- | --- | --- | --- |
| **OPFS** (Origin Private File System) | `connect_persistent(dir)` → `save()` / `load()` | `db_state.json` in a directory inside the OPFS root | Default; file-oriented, atomic-rename writes, CRC-32 footer (see `vantadb-wasm/src/opfs.rs`) |
| **IndexedDB** | `connect_idb(path)` → `save_idb()` / `load_idb()` / `delete_idb()` | object store `state`, key `db_state.json` in database `VantaDB` | Fallback when OPFS is unavailable (e.g. some embedded WebViews) |
| **Worker** (OPFS in a dedicated Web Worker) | `connect_worker(name)` + `worker_write` / `worker_read` / `worker_delete` (only when built with `--features opfs`) | same OPFS file, off main thread | Large payloads that should not block the UI thread; `WORKER_TIMEOUT_MS = 5000` |

The JS-side storage bridge is injected inline (no external file): the
IndexedDB bridge lives in `vantadb-wasm/src/idb.rs` and registers
`globalThis.vantaIdbStorage`. The worker bridge ships as
`vantadb-wasm/src/opfs_bridge.js`.

The persisted format is a JSON array of `VantaMemoryRecord`s. `save()` /
`save_idb()` skip the file write entirely when nothing changed since the last
successful persist (differential cache, PERF-08).

## Verified limits

Sources: MDN and web.dev (fetched 2026-08-19).

### Secure context required

OPFS (`navigator.storage.getDirectory`) is only available in **secure
contexts** (HTTPS or `http://127.0.0.1` / `http://localhost`). On a plain
`http://<hostname>` page the OPFS APIs are absent.

- MDN — Origin private file system: "This feature is available only in secure contexts"
  <https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system>

### OPFS availability

`StorageManager.getDirectory` (the OPFS root handle):

| Browser | Version |
| --- | --- |
| Chrome / Edge | 86+ |
| Firefox | 111+ |
| Safari (desktop + iOS) | 15.2+ |

Baseline: widely available since March 2023. In Safari private browsing,
`getDirectory()` throws (`SecurityError` / `UnknownError`) — the OPFS is
unavailable in private windows.

- MDN — StorageManager.getDirectory
  <https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/getDirectory>
- MDN — Browser compatibility data for `getDirectory` (Safari 15.2)
  <https://github.com/mdn/browser-compat-data>
- WebKit blog — File System Access API with Origin Private File System (Safari 15.2, private windows restriction)
  <https://webkit.org/blog/12257/the-file-system-access-api-with-origin-private-file-system/>

### Quota (OPFS and IndexedDB share the same origin quota)

The OPFS is subject to the browser storage quota, like IndexedDB. Practical
limits per origin (exact numbers differ per browser and disk size):

- **Chrome/Edge (Chromium):** up to 60% of total disk size per origin in both
  persistent and best-effort modes. Quota is based on the disk **total**, not
  the free space, to avoid fingerprinting. In incognito, ~5% of disk.
- **Firefox:** up to 2 GB per eTLD+1 group; browser may use up to 50% of free disk.
- **Safari (WebKit):** roughly 1 GB per origin, prompting to grow in 200 MB
  increments when exceeded (not officially documented); WebKit browsers allow
  ~60% of disk for browser apps with an overall 80% cap.

The real per-origin numbers are exposed by
`navigator.storage.estimate()` → `{ usage, quota }` (plus `usageDetails.fileSystem`
for the OPFS share). Code should always check the estimate and handle
over-quota write failures rather than assuming a fixed limit.

Clearing site data / all browsing data deletes both the OPFS and IndexedDB
state.

- MDN — Origin private file system (quota, estimate, eviction)
  <https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system>
- MDN — Storage quotas and eviction criteria (Chromium 60%, WebKit 60%/80%)
  <https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria>
- web.dev — Storage for the web (Chrome 60% origin / 80% browser, incognito 5%, Firefox 2 GB, Safari ~1 GB)
  <https://web.dev/articles/storage-for-the-web>
- MDN — StorageManager.estimate
  <https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/estimate>

### Web Locks (IndexedDB multi-tab coordination)

The IDB bridge wraps writes in `navigator.locks.request("vantadb-write", ...)`
to serialize concurrent tab writes. `navigator.locks` support:

| Browser | Version |
| --- | --- |
| Chrome / Edge | 69+ / 79+ |
| Firefox | 96+ |
| Safari | 15.4+ |

The bridge degrades gracefully: if `navigator.locks` is absent it runs the
transaction without a lock (`if (typeof navigator !== "undefined" && navigator.locks)`).

- MDN — Web Locks API (browser compatibility)
  <https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API>

### WebAssembly ES module import (build/deployment constraint)

The default (`bundler`) and `web` wasm-pack targets emit an ES module import
of the `.wasm` file (`import * as wasm from "./vantadb_wasm_bg.wasm"`).
**Stable Chrome and Edge do not support WebAssembly as an ES module** as of
2026 (verified empirically 2026-08-19: the browser rejects the module with
"Failed to load module script: Expected a JavaScript-or-Wasm module script
but the server responded with a MIME type of application/wasm"). Deployments
must therefore use a bundler for the `bundler` target, or the `no-modules`
target (`wasm-pack build --target no-modules`) with a classic `<script>` tag.

The `no-modules` target cannot use inline JS snippets (`inline_js`); the
generated glue emits a CommonJS `require()` for them (documented wasm-bindgen
limitation). The E2E harness works around this with a `require` shim that
serves the real snippet file — see `vantadb-wasm/e2e/persist.html`.

- wasm-bindgen guide — JS snippets caveats (only `web` and `bundler` targets support snippets)
  <https://wasm-bindgen.github.io/wasm-bindgen/reference/js-snippets.html>
- MDN — WebAssembly.Module / ES module integration (browser status)
  <https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/JavaScript_interface/ESM_integration>

### Worker-specific OPFS

The synchronous OPFS access handles (`FileSystemSyncAccessHandle`) exist
**only inside Web Workers**; the asynchronous handles are available on the
main thread and in workers. The worker path in VantaDB is opt-in (the `opfs`
feature) and uses the synchronous handles to avoid blocking the main thread.

- MDN — Origin private file system ("a set of synchronous calls available … that can be run inside web workers only")
  <https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system>

## E2E verification

`vantadb-wasm/e2e/` contains a real-browser persistence test:

- `persist.html` — harness page (seed: put 10 records + save; verify after a
  real `page.reload()`: read back 10 records). Runs against OPFS and IndexedDB.
- `e2e-persistence.mjs` — Playwright driver (static server on
  `http://127.0.0.1`, requires Edge installed, drives the harness through a
  reload and asserts `PASS:10` for both storages).

Run:

```bash
wasm-pack build vantadb-wasm --target no-modules --out-dir e2e/pkg-nomodules
node vantadb-wasm/e2e/e2e-persistence.mjs
```

Result (2026-08-19, Edge 151 over `http://127.0.0.1`):

```
[opfs] seed OK: 10 records put + saved
[opfs] after reload: PASS:10
[idb]  seed OK: 10 records put + saved
[idb]  after reload: PASS:10
ALL PASS
```

## Known gaps (not fixed here)

- `connect_persistent` / `connect_idb` do not automatically fall back to
  IndexedDB when OPFS is unavailable — the caller must detect and call
  `connect_idb` explicitly. The "fallback IndexedDB in Safari" scenario in
  the WASM-01 contract is verified manually, not automatic.
- `docs/architecture/WASM_STORAGE_REVIEW.md` predates the atomic-rename +
  CRC-32 write path and the IndexedDB bridge: its claims that there is no
  atomic rename / no checksum / no IDB tests are stale (see `opfs.rs` and
  `tests/wasm_tests.rs`).
- The demo (`vantadb-wasm/demo/app.js`) never calls `save()`/`save_idb()`,
  so its persistence claim is not exercised.