# Desktop + Admin Console — DESKTOP/ADMIN

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### DESKTOP-01: Investigar Tauri como plataforma desktop para VantaDB
- **Fuente:** Backlog (Investigaciones)
- **Fecha:** 2026-08-04
- **Objetivo:** Evaluar Tauri (v2) como plataforma desktop para VantaDB: integración Rust nativa (`vantadb` como dep directa), casos de uso desktop AI app privada con memoria local, comparativa vs Electron, effort estimate MVP desktop, y recomendación de arquitectura. Sin implementación — solo investigación + recomendación.
- **Resultado:** ✅ Doc: `docs/research/DESKTOP-01-tauri-plataforma-desktop.md` (20.9KB, 208 líneas). **Recomendación: SÍ — Tauri v2** con integración Rust nativa (`vantadb` en `src-tauri/`, `VantaEmbedded` en managed state, commands async `vanta_ingest`/`vanta_search`, SIN bridge WASM/OPFS). Tauri v2.11.5 (01-jul-2026) vs Electron v43.2.0 (21-jul-2026). Comparativa: bundle 2-10MB vs 80-200MB; RAM idle ~50MB vs ~120MB+; backend Rust+WebView nativo vs Node+Chromium; mobile iOS/Android ✅ vs ❌. Effort MVP: ≈8-13 días hábiles. Nota: origen GTM original (`docs_backup_2026-06-30/`) ya no existe en el repo. Cero cambios de código.
- **Ids:** `DESKTOP-01`

### DESKTOP-02: Scaffold Tauri v2 + propio workspace
- **Fuente:** Backlog (Phase 12 — DESKTOP)
- **Fecha:** 2026-08-06
- **Objetivo:** `create-tauri-app` en `desktop/`; `src-tauri/Cargo.toml` con `[workspace] members=["."]` desacoplado del raíz; tauri.conf + capabilities mínimas; command `ping`.
- **Resultado:** ✅ Tauri v2 (React+Vite+TS) con `ping`, `AppState`, capabilities `core:default`, `com.vantada.desktop`. Verificado `cargo check` + raíz invariante. Commits `9feefea7`.
- **Ids:** `DESKTOP-02`

### DESKTOP-03: Integrar crate `vantadb` + managed state + healthcheck
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Dep `vantadb` con `default-features=false` + subset, `AppState { manager, config }` managed, command `vanta_health`.
- **Resultado:** ✅ `vanta_health` abre `VantaEmbedded` en temp dir, devuelve `HealthReport{backend:"fjall"}`; doble open del path → `VantaError::Lock`. `HealthReport` ganó campo `backend`. 17 tests. Commit `759e2d3e`.
- **Ids:** `DESKTOP-03`

### DESKTOP-04: Trait `VantaConnection` + tipos + errores
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Contrato multi-connection object-safe con DTOs serde compartidos y `VantaError` unificado.
- **Resultado:** ✅ `VantaConnection` async_trait object-safe + 9 tipos serde devuelve por todas las vías + `VantaError` `#[non_exhaustive]` (Native/Http/Mcp/... + Lock/Timeout). 17 tests serde roundtrip. Commits `dd7d25a1`, `363c3f8a7`.
- **Ids:** `DESKTOP-04`

### DESKTOP-05: NativeConnection
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** `VantaEmbedded` embebida, ops en `spawn_blocking`, lock de path, capabilities.
- **Resultado:** ✅ `NativeConnection::open` con lock de path duplicado → `VantaError::Lock`, ops en `spawn_blocking`, health→"fjall". 4 tests (trait roundtrip + lock). Commit `5cebcc29`. Wired en `mod.rs`.
- **Ids:** `DESKTOP-05`

### DESKTOP-06: Commands CRUD async
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Commands Tauri `vanta_connect/disconnect/list_connections/set_active/ingest/ingest_batch/search/get/delete/list` delegando al adaptador activo.
- **Resultado:** ✅ `ConnectionManager` (tokio RwLock, HashMap + active_id, 14 métodos) reemplazando placeholder `manager: ()`; 11 commands registrados. E2E connect→ingest→search ordenado. 24 tests lib total. Commit `9d2d5319`.
- **Ids:** `DESKTOP-06`

### DESKTOP-07: Frontend MVP
- **Fuente:** GitHub (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** React+Vite MVP: ConnectionPanel, IngestForm, SearchBar, ResultsList, hook, bridge `vanta.ts`.
- **Resultado:** ✅ bridge tipado + 5 componentes + single-file CSS; `npm run build` (tsc+vite) exit 0. Commit `10c161aa`.
- **Ids:** `DESKTOP-07`

### DESKTOP-08: Cliente IQL tipado
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Wrapper reqwest (json) config url/port/token timeout; statements IQL mapeados; validar contra `HTTP_API.md`/`cli_server.rs`.
- **Resultado:** ✅ `ServerClient` mapea 8 statements IQL (health, metrics, query POST `/api/v2/query`, put/get/delete/list/search) con auth Bearer; `success:false` → `VantaError::Http`. 28 tests (11 mock + 17 unit). WIP en `wire_types.rs`. Commit `b7aff3a0`.
- **Ids:** `DESKTOP-08`

### DESKTOP-09: ServerConnection
- **Fuente:** GitHub (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Implementa el trait sobre el client IQL; connect valida auth/health; timeouts; `success:false` como error de dominio.
- **Resultado:** ✅ `ServerConnection` delegando a `ServerClient`, timeouts → `VantaError::Timeout`, capabilities→[Http]; test e2e con server real gateado por `VANTADB_TEST_SERVER=1`. 21 tests lib + 2 e2e. Commit `a5f2da1b`.
- **Ids:** `DESKTOP-09`

### DESKTOP-10: Wire Server en commands + UI
- **Fuente:** GitHub (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Selector muestra vía "Server" con url/puerto/token; conexión entra al registry y puede ser activa.
- **Resultado:** ✅ `ConnectionSelector.tsx` (loopback-only url/port/token, bridge invoke sin fetch directo); `vanta_connect` ya soportaba `via:"server"`. `npm run build` ++ `cargo check` exit 0. Commit ``, de no dia `7619c3cb`.
- **Ids:** `DESKTOP-10`

### DESKTOP-11: Spawn manager subproceso MCP
- **Fuente:** Backlog (Phase 12)
- **Fecha:** 2026-08-06
- **Objetivo:** Localizar binario `vantadb-server`; confirmar flag `--mcp`; tokio Command con stdio piped, stderr a log, timeout arranque.
- **Resultado:** ✅ `McpSpawn` con `tokio::process::Command`, stderr→log temporal, timeout; spawn+kill limpio; test gateado si falta binario. Commit d`d62c1c0c`.
- **Ids:** `DESKTOP-11`

### ADMIN-01: Command vanta_metrics IPC
- **Fuente:** Backlog (Phase 12 — Fase 7 Consola Admin)
- **Fecha:** 2026-08-08
- **Objetivo:** Exponer el snapshot de métricas operativas del core como comando Tauri.
- **Resultado:** ✅ `desktop/src-tauri/src/commands/metrics.rs` con `#[tauri::command] vanta_metrics` → usa `VantaEmbedded::operational_metrics()` (`VantaOperationalMetrics` ya `Serialize`, exportado en `vantadb`); 37 campos incl. `derived_prefix_scans`, `derived_full_scan_fallbacks`. Cero cambios al core. Commit `d77559f3`.
- **Ids:** `ADMIN-01`

### ADMIN-02: Métricas vivas (delta entre snapshots)
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Frontend calcula deltas/rates entre snapshots consecutivos (poll 3-5s).
- **Resultado:** ✅ Convergido en ADMIN-04 (`b62fff7c`): grid con tiles muestra deltas imports/queries/scans, RSS y rate por poll 4s; código propio eliminado por duplicación (lección: ADMIN-02 solapa con ADMIN-04/05 — deberían fusionarse). Contract verificado con `npm run build`.
- **Ids:** `ADMIN-02`

### ADMIN-03: Migrar UI al design system web (modo claro)
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Reemplazar tema oscuro de `App.css` por tokens de `web/globals.css` (cream/ink/neon) y eliminar `ConnectionSelector.tsx` muerto.
- **Resultado:** ✅ `App.css` reescrito (95+/225−) con tokens cream `#FBF9F5`/ink `#000`/neon `#FF5500`/paper, bordes 2-3px, sombra dura `6px 6px 0 #000`, radius 0; clases preservadas (sin tocar TSX); `ConnectionSelector.tsx` eliminado (+0 refs). `npm run build` OK. Commit `847ab080`.
- **Ids:** `ADMIN-03`

### ADMIN-04: Dashboard grid (metro-style) con poll 3-5s
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Layout de cards con polling en cadena y estados health por vía.
- **Resultado:** ✅ `MetricsGrid.tsx` metro 6 tiles (RSS, Records, Queries, Scans, WAL Replay, Text Index) con delta + trend (▲/▼), poll inline `setInterval` 4s, cleanup; header con health badge y last-poll. Grid responsive 3→2→1 col con design system. `npm run build` OK. Commit `b62fff7c`.
- **Ids:** `ADMIN-04`

### ADMIN-05: KPIs derivados
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** KPIs calculados a partir del snapshot con spinner y derivados simples.
- **Resultado:** ✅ `KpiCards.tsx` (113 líneas): memory efficiency (mmap/RSS), hybrid query share, import error rate, WAL rec/ms, HNSW bytes/node con guard div-by-zero y sparkline CSS puro (ring 12). Consolidó `vanta.ts` — interfaz `OperationalMetrics` única + `metrics()` (fix TS2393 de dos interfaces duplicadas). Commit `4dcf268e`.
- **Ids:** `ADMIN-05`

### ADMIN-06: SOP panels (WAL replay / Reindex / Health) con semáforo
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Flujo con estado idle → running → done|error y botones de acción/re-run.
- **Resultado:** ✅ `SopPanel.tsx` con 3 paneles accionables: WAL Replay + Reindex muestran último valor del snapshot (Refresh) — el core no expone triggers, documentado; Health llama `vanta_health` en vivo. Extendido bridge TS con 4 campos (`startup_ms`, `ann_rebuild_ms`, `derived_rebuild_ms`, `text_index_rebuild_ms`). `npm run build` OK. Commit `f20d67a4`.
- **Ids:** `ADMIN-06`

### ADMIN-07: Data Explorer
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Tabla navegable con paginación y score.
- **Resultado:** ✅ `DataExplorer.tsx`: browse (`vanta_list`) + search (`vanta_search` con score) + tabla id/ns/text/score + "Load more" con limit creciente 50→100→200 (core sin offset/cursor — `ponytail:` documentado). Cero cambios Rust. `npm run build` OK. Commit `7a19a9f5`.
- **Ids:** `ADMIN-07`

### ADMIN-08: Panel Procesos & Conexiones
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Listar conexiones activas y procesos con kill/remove desde UI.
- **Resultado:** ✅ `ProcessPanel.tsx` (69 líneas): lista de conexiones con botón shutdown por entrada (`vanta_disconnect` existente) + placeholder Subprocesses documentado (sin `McpSpawnRegistry` en core; `McpSpawn` nunca instanciado). Cero Rust inventado. `npm run build` OK. Commit `f5c69788`.
- **Ids:** `ADMIN-08`

### ADMIN-09: Snapshot export + persistencia
- **Fuente:** Backlog (Phase 12 — Fase 7)
- **Fecha:** 2026-08-08
- **Objetivo:** Exportar snapshot JSON con timestamp y persistir último en disco/localStorage.
- **Resultado:** ✅ `ExportPanel.tsx`: blob download JSON (URL.createObjectURL + `<a download>`) + localStorage con timestamp; sin plugins Tauri nuevos. `npm run build` OK. Commit `e0e8ff3a`.
- **Ids:** `ADMIN-09`

### DESKTOP-20: Lifecycle shutdown_all
- **Fuente:** Backlog (Phase 12 — Fase 5)
- **Fecha:** 2026-08-08
- **Objetivo:** Cerrar todos los subprocesos/conexiones al salir, con graceful + kill forzoso.
- **Resultado:** ✅ `shutdown_all(grace)` en `ConnectionManager` (release lock, non-native primero, native última con flush vía `db.close()`; `timeout(5s)` → `VantaError::Other` + Drop force-kill `McpSpawn`) y hook en `RunEvent::ExitRequested` (validado contra docs.rs tauri 2.11.5). Test `shutdown_all_empties_registry_and_disconnects` 2/2. Commit `45f8bed8`.
- **Ids:** `DESKTOP-20`
