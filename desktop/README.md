# Vanta Studio (desktop)

Human-facing desktop console for [VantaDB](https://github.com/ness-e/Vantadb) —
a Tauri v2 app (Rust shell + React frontend) for inspecting and operating a
VantaDB database from a local window.

The same UI runs against three interchangeable backends without code changes
(see [Transport modes](#transport-modes)). For the full design, user guide and
per-mode instructions, see [`docs/desktop/`](../docs/desktop/README.md)
([ARCHITECTURE.md](../docs/desktop/ARCHITECTURE.md),
[GUIDE.md](../docs/desktop/GUIDE.md)).

## Prerequisites

- **Node.js ≥ 20** (Vite 7 — `vite` requires Node 20.19+ / 22.12+)
- **Rust** — the shell crate pins `rust-version = "1.94.1"`
  (`desktop/src-tauri/Cargo.toml`)
- **Tauri v2 prerequisites** for your OS
  ([tauri.app](https://v2.tauri.app/start/prerequisites/)) — on Windows that
  is the WebView2 runtime and MSVC build tools; on Linux the webkit2gtk
  packages; on macOS Xcode command line tools

## Install & run (development)

```bash
cd desktop
npm install

# Native desktop app (Tauri shell + embedded VantaDB engine)
npm run tauri dev

# Web console served by a separate vantadb-server (HTTP /api/v2/*)
# start the server first, then:
npm run dev            # plain browser → HTTP backend (vite proxies /api → 127.0.0.1:8090)

# 100% in-browser standalone (WASM engine + OPFS persistence, no server)
npm run build:wasm     # outputs dist-wasm/
npm run preview        # serve the built output
```

The Tauri shell (`desktop/src-tauri`) links the local `vantadb` crate from the
workspace root and is a **separate Cargo workspace** (own `Cargo.toml`,
isolated from the repo-root workspace) — build it from `desktop/src-tauri`.

## Transport modes

| Mode | Backend | Best for | Persistence |
|------|---------|----------|-------------|
| **Native embedded** (default in Tauri) | `NativeConnection` → embedded `vantadb` engine (fjall) | Single-user desktop use, maximum performance | On-disk directory of your choice |
| **HTTP server** | `ServerConnection` → REST `/api/v2/*` on a running `vantadb-server` | Remote server, multiple users, Bearer auth | Server-side |
| **WASM-OPFS** | `WasmBackend` → `vantadb-wasm` in the browser | Standalone browser demo, offline, zero install | OPFS (IndexedDB fallback) |

Transport is selected automatically at load time (`desktop/src/transport.ts`):
inside Tauri it uses IPC; in a plain browser it uses HTTP (`VITE_VANTA_API_BASE`
overrides the origin); with `VITE_VANTA_MODE=wasm` or `npm run build:wasm` it
runs fully in-browser against WASM + OPFS.

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run dev` | Vite dev server (port 1420, proxies `/api` → `127.0.0.1:8090`) |
| `npm run build` | `tsc && vite build` — Tauri/web production build (`dist/`) |
| `npm run build:wasm` | Standalone browser build with WASM engine (`dist-wasm/`) |
| `npm run preview` | Preview the built output |
| `npm test` | Frontend unit tests (vitest) |
| `npm run tauri dev` | Run the desktop app (starts Vite, then the Tauri shell) |
| `npm run tauri build` | Produce OS installers (NSIS/MSI on Windows) |

## Development layout

- `desktop/src/` — React frontend (Vite + Tailwind 4 + TanStack Table/Virtual)
- `desktop/src-tauri/src/` — Rust shell: IPC commands (`commands/`),
  multi-connection layer (`connections/`: native, server, MCP spawn)
- `desktop/src-tauri/tauri.conf.json` — app window, deep-link scheme `vanta://`,
  bundle targets (NSIS/MSI)
- `desktop/DESIGN_DECISIONS.md` — binding design tokens/theming rules

## Installer status

> ⚠️ **No public installer yet.** Bundling targets (NSIS/MSI) are configured and
> `npm run tauri build` produces local installers, but there is **no public
> release channel / download page** — the desktop app is in development and
> must be run from source. Track the release in the project backlog.

## Related docs

- [`docs/desktop/README.md`](../docs/desktop/README.md) — app-level docs
  (installation, IPC command surface, troubleshooting)
- [`docs/desktop/ARCHITECTURE.md`](../docs/desktop/ARCHITECTURE.md) —
  multi-connection model, transports, lifecycle
- [`docs/desktop/GUIDE.md`](../docs/desktop/GUIDE.md) — per-mode user guide
- ADRs: [ADR-026](../docs/architecture/adr/ADR-026-vanta-studio-fase3-rest-dashboard.md),
  [ADR-027](../docs/architecture/adr/ADR-027-fase4-cierre-deuda-rest-wasm-opfs.md),
  [ADR-028](../docs/architecture/adr/ADR-028-core-decay-supersession.md)

## License

Apache-2.0