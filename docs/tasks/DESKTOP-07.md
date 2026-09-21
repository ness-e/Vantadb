# DESKTOP-07 — Frontend React+Vite MVP (desktop UI)

> Plan: `docs/plans/2026-08-06-desktop-mvp.md` (Task 6)
> Branch: develop
> Commit: see `git log -1`
> Estado: ✅ COMPLETED

## Objetivo

Frontend del desktop app: bridge TS tipado sobre los commands Tauri (DESK-03/06),
hook de estado de conexión, y 4 componentes (ConnectionPanel / IngestForm /
SearchBar / ResultsList) que arman la demo vertical: health → connect native →
ingest → search, todo vía `invoke`. Solo frontend; commands Rust ya existen.

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 1 | Bridge `desktop/src/vanta.ts`: tipos TS en correspondencia con los serde DTO de `src-tauri/src/connections/types.rs` (HealthReport, ConnectionInfo, IngestItem, SearchQuery, SearchResult, MemoryRecord, Capability, ConnectionStatus, HealthStatus) + `vantaErrorMessage` que deserializa la variante externamente-tagueada de `VantaError`; funciones `health/connectNative/connectServer/disconnect/listConnections/setActive/ingest/ingestBatch/search/get/remove/list` | ✅ | write |
| 2 | Hook `src/hooks/useConnectionState.ts`: orquesta health + conexiones + active id + errores | ✅ | write |
| 3 | Componentes `ConnectionPanel` (badge health, conectar nativo, lista activo/desconectar), `IngestForm` (batch add), `SearchBar` (query + top_k), `ResultsList` (score/texto/namespace) | ✅ | write |
| 4 | `App.tsx` + `App.css` reescritos (layout limpio de 1 CSS conciso) | ✅ | edit/write |
| 5 | `npm run build` en `desktop/` → **exit 0** (tsc + vite build) | ✅ | bash |

## Notas

- Contracto bridge (para DESK-10): ver sección "Bridge contract" a continuación.
- `connectServer` incluida pero NO ejercida hoy (DESK-10 añade la vía server); el
  selector contempla ambas vías en el enum `ConnectTarget` de Rust.
- No se toca `desktop/src-tauri/` (lo gestiona DESK-10), `web/`, `src/` ni raíz.
- El bridge usa `import { invoke } from "@tauri-apps/api/core"` (Tauri v2).

## Bridge function contract (handoff DESK-10)

| fn | invoke | args | return |
|----|--------|------|--------|
| `health()` | `vanta_health` | — | `HealthReport` |
| `connectNative(path)` | `vanta_connect` | `{ target: { via:"native", path } }` | `ConnectionInfo` |
| `connectServer(cfg)` | `vanta_connect` | `{ target: { via:"server", config: ServerClientConfig } }` | `ConnectionInfo` |
| `disconnect(id)` | `vanta_disconnect` | `{ id }` | `void` |
| `listConnections()` | `vanta_list_connections` | — | `[string, ConnectionInfo][]` |
| `setActive(id)` | `vanta_set_active` | `{ id }` | `void` |
| `ingest(records)` | `vanta_ingest` | `{ records: IngestItem[] }` | `string[]` |
| `ingestBatch(records)` | `vanta_ingest_batch` | `{ records }` | `string[]` |
| `search(query)` | `vanta_search` | `{ query: SearchQuery }` | `SearchResult[]` |
| `get(key, ns?)` | `vanta_get` | `{ key, namespace? }` | `MemoryRecord` |
| `remove(key, ns?)` | `vanta_delete` | `{ key, namespace? }` | `void` |
| `list({namespace?,limit?})` | `vanta_list` | `{ namespace?, limit? }` | `MemoryRecord[]` |

`ServerClientConfig = { url: string; port: number; token?: string; timeout?: { secs, nanos } }`
(timeout es `Duration` serde = `{secs,nanos}`).