# WASM Standalone Console (Vanta Studio — mode `wasm`)

The Vanta Studio console can run **100% in the browser** with no server: the
WASM engine (`vantadb-wasm/pkg`) plus OPFS/IndexedDB persistence. This is the
`vite build --mode wasm` build (`desktop/dist-wasm/`, WASM-03).

## Build & run

```bash
# Prerequisite: the wasm-bindgen glue must exist (git-ignored)
wasm-pack build vantadb-wasm

# Build the standalone console
cd desktop
npm run build:wasm            # → dist-wasm/

# Serve it over HTTP on 127.0.0.1 (OPFS requires a secure context)
npx vite preview --outDir dist-wasm
# or any static server on http://127.0.0.1
```

`file://` will not work: the wasm glue is fetched over HTTP and OPFS requires
a secure context (https or http://127.0.0.1/localhost).

## What works

The full console surface (HOME / MEMORIAS / ACTIVITY / ÍNDICES / IQL) runs the
same React code as Tauri/web. The implicit connection shows `via: wasm` and
"WASM local (OPFS/IndexedDB, sin server)". The `WasmBackend` transport
(`desktop/src/transport.ts`) maps `vanta_*` commands to the wasm-bindgen API
(`desktop/src/vanta-wasm-map.ts`):

- `vanta_health`, `vanta_metrics`
- `vanta_ingest`, `vanta_ingest_batch`, `vanta_put`, `vanta_get`, `vanta_delete`
- `vanta_list`, `vanta_delete_by_filter`, `vanta_search`
- Persistence after every mutation (`connect_persistent` → `save()`; IndexedDB
  fallback → `save_idb()`)

Commands with no wire-compatible WASM method degrade with a descriptive error
(the WEB-04 pattern), never an invented call:

| Command | Reason |
|---|---|
| `vanta_connect` / `vanta_disconnect` / `vanta_list_connections` / `vanta_set_active` | Multi-connection management is Tauri-only; WASM owns one implicit DB |
| `vanta_get_version` / `vanta_versions` | Version history is native-only |
| `vanta_query` / `vanta_iql_autocomplete` | The WASM binding's `query()` resolves reads against the graph store, not memory records — `SELECT` returns empty despite records existing; requires engine work |
| `vanta_export_namespace` | Export writes to a filesystem path (Tauri save dialog / server); the browser has no file path |
| `vanta_graph_bfs` / `vanta_graph_dfs` / `vanta_graph_degree` | WASM returns node-id vectors / degrees, not the desktop `{nodes, edges}` DTO |
| `vanta_namespace_stats` | No per-namespace stats method; callers fall back to client-side `list()` counts |
| `vanta_audit_events` | Audit log is server/native-only |

## Known limits (verified)

- **OPFS requires a secure context** — `http://127.0.0.1`/`https` only
  (WASM-01, verified against MDN + web.dev).
- **Storage quotas** are browser-managed: Chromium ~60% of disk, Firefox 2 GB,
  Safari ~1 GB (per-origin; WASM-01 findings).
- **Persistence round-trip gap**: `put` metadata → `get`/`list` may return
  `metadata: {}` in the WASM in-memory open (`ShreddedRowStore`), and IQL reads
  resolve against the graph store rather than memory records (WASM-02 open
  items — engine work required, tracked in the plan).

## Verification

`desktop/scripts/selfcheck-wasm-e2e.ts` runs an E2E smoke (Playwright Edge +
node:http static server): boot → health → ingest via UI → grid row → real
reload → record persists in OPFS. Exit 0 only when everything passes.