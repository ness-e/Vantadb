# VantaDB Desktop (Vanta Studio)

Desktop console for VantaDB built with **Tauri v2** (Rust shell + React frontend).
The same UI runs against three interchangeable backends without code changes —
see [ARCHITECTURE.md](ARCHITECTURE.md) for the design and [GUIDE.md](GUIDE.md)
for per-mode user instructions.

## Transport Modes

| Mode | Backend | Best for | Persistence |
|------|---------|----------|-------------|
| **Native embedded** (default in Tauri) | `NativeConnection` → embedded `vantadb` engine (fjall) | Single-user desktop use, maximum performance | On-disk directory of your choice |
| **HTTP server** | `ServerConnection` → REST `/api/v2/*` on a running `vantadb-server` | Remote server, multiple users, Bearer auth | Server-side |
| **WASM-OPFS** | `WasmBackend` → `vantadb-wasm` in the browser | Standalone browser demo, offline, zero install | OPFS (IndexedDB fallback) |

Transport is selected automatically at load time (`desktop/src/transport.ts`):
inside Tauri it uses IPC; in a plain browser it uses HTTP (`VITE_VANTA_API_BASE`
overrides the origin); with `VITE_VANTA_MODE=wasm` or
`npm run build:wasm` it runs fully in-browser against WASM + OPFS.

## Installation

Prerequisites: [Rust](https://rustup.rs), Node.js ≥ 20, and the Tauri v2
prerequisites for your OS ([tauri.app](https://v2.tauri.app/start/prerequisites/)).

```bash
cd desktop
npm install
```

The Rust shell (`desktop/src-tauri`) links the local `vantadb` crate from the
workspace root — build it from the repo root so workspace paths resolve.

## Running

```bash
# Native mode (Tauri shell, embedded engine)
npm run tauri dev

# Web console served by the embedded server (HTTP /api/v2/*)
# start vantadb-server first, then:
npm run dev            # plain browser → HttpBackend

# WASM standalone (browser-only, no server)
npm run build:wasm     # outputs dist-wasm/
node scripts/selfcheck-wasm-e2e.ts   # E2E smoke (needs wasm-pack pkg built)
```

## Tauri Commands (IPC surface)

Defined in `desktop/src-tauri/src/commands/`. Connection lifecycle:

| Command | Purpose |
|---------|---------|
| `vanta_connect` | Connect + register + activate. Target is tagged: `{via:"native", path}` or `{via:"server", config}` |
| `vanta_disconnect` | Remove a connection by id (releases the native path lock) |
| `vanta_list_connections` | List registered connections as `(id, info)` |
| `vanta_set_active` | Point all data commands at connection `id` |
| `vanta_health` | Round-trip probe of the embedded engine (throwaway temp dir) |

Data commands (`ingest`, `search`, `get`, `put`, graph ops, export, …) are
dispatched to the **active** connection by the `ConnectionManager`.

## Local Development

- Frontend: React + Vite (`desktop/src/`). Tests: `npm test` (Node TS strip mode + vitest).
- Rust shell: `cargo test` inside `desktop/src-tauri/`.
- The connections layer lives in `desktop/src-tauri/src/connections/`
  (`manager.rs`, `native.rs`, `server.rs`, `server_client.rs`, `trait.rs`).

## Troubleshooting

| Symptom | Cause / Fix |
|---------|-------------|
| `VantaError::Lock` on connect | Another connection/process holds the writer lock on that path. Disconnect it first or pick another path. |
| "no active connection; call vanta_connect first" | No connection registered yet — connect before issuing data commands. |
| `401 Unauthorized` over HTTP | Server has auth enabled; configure Bearer credentials in the server connection config. |
| OPFS unavailable in browser | Private-mode/incognito browsers block OPFS; the app falls back to IndexedDB automatically. |
| WASM backend never loads | It only loads in `--mode wasm` builds or with `VITE_VANTA_MODE=wasm`; Tauri/HTTP builds code-split it out. |

## Design Docs

- [ARCHITECTURE.md](ARCHITECTURE.md) — multi-connection model, transports, lifecycle.
- ADRs: [ADR-026](../architecture/adr/ADR-026-vanta-studio-fase3-rest-dashboard.md)
  (REST `/api/v2/*` + embedded dashboard),
  [ADR-027](../architecture/adr/ADR-027-fase4-cierre-deuda-rest-wasm-opfs.md)
  (WASM/OPFS backbone),
  [ADR-028](../architecture/adr/ADR-028-core-decay-supersession.md)
  (core decay supersession).
