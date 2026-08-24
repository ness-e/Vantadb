# VantaDB Desktop — Architecture

This document describes the **actual** multi-connection model of the desktop
app (`desktop/`), as implemented in `desktop/src-tauri/src/connections/`.
Strategic decisions live in the ADRs — this document references them and does
not duplicate them:

- [ADR-026](../architecture/adr/ADR-026-vanta-studio-fase3-rest-dashboard.md) —
  full REST surface `/api/v2/*`, embedded dashboard, local-first loopback (D11/D12).
- [ADR-027](../architecture/adr/ADR-027-fase4-cierre-deuda-rest-wasm-opfs.md) —
  Fase 4 closure: REST debt, WASM/OPFS backbone (D13–D15).
- [ADR-028](../architecture/adr/ADR-028-core-decay-supersession.md) —
  core decay supersession semantics surfaced by the consolidation UI.

## Overview

Transport is pluggable at two layers:

```
┌─────────────────────────── Console (React, desktop/src/) ───────────────────────────┐
│  VantaTransport (transport.ts)                                                      │
│  ├── TauriBackend   → Tauri IPC invoke("vanta_*")            [in Tauri shell]       │
│  ├── HttpBackend    → fetch /api/v2/* on vantadb-server      [plain browser]        │
│  └── WasmBackend    → vantadb-wasm + OPFS/IndexedDB          [--mode wasm build]    │
└─────────────────────────────────────────────────────────────────────────────────────┘
                    │ (TauriBackend only)
┌────────────────────▼──── Shell (Rust, desktop/src-tauri/src/) ──────────────────────┐
│  ConnectionManager (manager.rs)                                                     │
│  registry: HashMap<String, Box<dyn VantaConnection>> + active_id                    │
│  ├── NativeConnection  (native.rs)  → embedded vantadb::VantaEmbedded (fjall)       │
│  └── ServerConnection  (server.rs)  → ServerClient → HTTP /api/v2/* (Bearer auth)   │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## The `VantaConnection` contract

`connections/trait.rs` defines the async trait every backend adapter implements:
lifecycle (`connect`/`disconnect`), metadata (`info`, `capabilities`,
`audit_log_path`), and data operations (`ingest`, `put`, `search`, `get`,
versions, IQL `query`, namespace stats, batch delete by filter, graph BFS/DFS/
degree, export). Optional capabilities report `VantaError::Unsupported` via
trait defaults — e.g. only native connections expose an audit log (VS-12) or a
raw embedded handle for the memory pipeline (MEM-53).

## ConnectionManager

`connections/manager.rs` — managed Tauri state holding:

- **Registry**: `HashMap<String, Box<dyn VantaConnection>>` behind one
  `tokio::sync::RwLock`. `add()` connects (health validation) then registers and
  activates; `remove(id)` disconnects and releases backend resources.
- **`active_id`**: every data command resolves `active_id` first, then dispatches
  to that connection. No active connection →
  `VantaError::Unsupported("no active connection; call vanta_connect first")`.
  Removing the *active* connection automatically falls back to another
  registered key (arbitrary insertion-order pick), or clears selection if none
  remain.


> The former `ConnectionSelector` abstraction was eliminated (ADMIN-03); the
> manager's `active_id` field is the single selection mechanism.

## Transports

### `NativeConnection` (native.rs)

Wraps `vantadb::VantaEmbedded` directly. All synchronous SDK calls run on the
blocking pool via `tokio::task::spawn_blocking`, so the async trait never stalls
the runtime.

- **Path lock**: opening a path already locked by another writer surfaces the
  core's `DatabaseBusy` as `VantaError::Lock` — callers can branch on it. One
  writer per path, enforced by the engine.
- **Audit** (VS-12): enabled by default at `<path>/audit.jsonl`; configurable or
  disableable via `open_with_audit`.
- **id**: `native:<path>`.

### `ServerConnection` (server.rs)

Adapter over `ServerClient` (`server_client.rs`), the typed HTTP client for
`vantadb-server`. The client owns transport concerns (URL building, Bearer auth,
envelope validation, `success:false` → typed error mapping); the adapter maps
contract DTOs to IQL signatures and enforces a per-op timeout
(`VantaError::Timeout`). Health/auth are validated in `connect()` against
`/health`; a later 401 surfaces as `Http { kind: Unauthorized }`.

### WASM (frontend-only)

There is no Rust-side Wasm connection. The standalone browser console
(`vite build --mode wasm` → `dist-wasm/`) bundles the `vantadb-wasm`
wasm-bindgen module: `WasmBackend` lazily imports it, opens OPFS persistence
(`connect_persistent`, IndexedDB fallback), and persists after mutating
commands. Scope and closure: ADR-027.

## Lifecycle & shutdown

`shutdown_all(grace)` (DESKTOP-20), called from `lib.rs` on
`RunEvent::ExitRequested` with `SHUTDOWN_GRACE = 5s`:

1. Takes the whole registry out and clears `active_id` (idempotent on empty).
2. Disconnects non-native connections first; **native last**, so its `close()`
   flushes pending writes.
3. Each disconnect is bounded by `grace`; a hung adapter times out and is
   dropped — any sidecar it owns is force-killed by `McpSpawn`'s `Drop`, so no
   orphaned children survive.

Returns one `(id, result)` pair per connection for logging.

## IPC commands

`commands/connection.rs` exposes `vanta_connect` (tagged `ConnectTarget`:
`{via:"native", path}` | `{via:"server", config}`), `vanta_disconnect`,
`vanta_list_connections`, `vanta_set_active`, and `vanta_health`. Data commands
live in `commands/memory.rs` and route through the manager's active connection.
