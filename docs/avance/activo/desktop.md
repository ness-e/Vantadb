---
title: "Avance — Desktop (Tauri)"
type: domain-log
status: active
tags: [vantadb, avance, desktop, tauri, rust, frontend]
last_reviewed: 2026-08-07
aliases: [DESKTOP]
---

# Avance — Desktop (Tauri)

> Registro consolidado de la Fase 12 — DESKTOP: cliente Tauri v2 con integración Rust nativa del crate `vantadb`. **IDs originales conservados (DESKTOP-01..11).**

## Cobertura rápida

- **Decisión de plataforma:** Tauri v2 (no Electron) — bundle 2-10MB vs 80-200MB, RAM idle ~50MB vs ~120MB+, backend Rust nativo + WebView.
- **Arquitectura:** `desktop/` workspace desacoplado; crate `vantadb` con `default-features=false`; trait `VantaConnection` object-safe (Native/Server); frontend React+Vite+TS con bridge tipado.
- **Estado:** 11/11 tareas ✅ (04-08 al 06-08). Cero cambios de código en el raíz.

---

## Investigación

### DESKTOP-01: Investigar Tauri como plataforma desktop
- **Fecha:** 2026-08-04
- **Resultado:** ✅ Doc `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` (20.9KB, 208 líneas). **Recomendación: SI — Tauri v2** con integración Rust nativa (`vantadb` en `src-tauri/`, `VantaEmbedded` en managed state, commands async `vanta_ingest`/`vanta_search`, SIN bridge WASM/OPFS). Tauri v2.11.5 vs Electron v43.2.0. Effort MVP: ~8-13 días hábiles. Solo investigación, cero cambios de código.

## Scaffold & core desktop

### DESKTOP-02: Scaffold Tauri v2 + workspace propio
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `create-tauri-app` en `desktop/`; `src-tauri/Cargo.toml` con `[workspace] members=["."]` desacoplado del raíz; tauri.conf + capabilities mínimas; command `ping`. Tauri v2 (React+Vite+TS), capabilities `core:default`, `com.vantada.desktop`. Commit `9feefea7`.

### DESKTOP-03: Integrar crate `vantadb` + managed state + healthcheck
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Dep `vantadb` con `default-features=false` + subset, `AppState { manager, config }` managed, command `vanta_health`. `vanta_health` abre `VantaEmbedded` en temp dir, devuelve `HealthReport{backend:"fjall"}`; doble open del path → `VantaError::Lock`. `HealthReport` ganó campo `backend`. 17 tests. Commit `759e2d3e`.

### DESKTOP-04: Trait `VantaConnection` + tipos + errores
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `VantaConnection` async_trait object-safe + 9 tipos serde devueltos por todas las vías + `VantaError` `#[non_exhaustive]` (Native/Http/Mcp/... + Lock/Timeout). 17 tests serde roundtrip. Commits `dd7d25a1`, `363c3f8a7`.

### DESKTOP-05: NativeConnection
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `VantaEmbedded` embebida, ops en `spawn_blocking`, lock de path, capabilities. `NativeConnection::open` con lock de path duplicado → `VantaError::Lock`, ops en `spawn_blocking`, health "fjall". 4 tests (trait roundtrip + lock). Commit `5cebcc29`.

### DESKTOP-06: Commands CRUD async
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Commands Tauri `vanta_connect/disconnect/list_connections/set_active/ingest/ingest_batch/search/get/delete/list` delegando al adaptador activo. `ConnectionManager` (tokio RwLock, HashMap + active_id, 14 métodos) reemplaza placeholder `manager: ()`; 11 commands registrados. E2E connect→ingest→search ordenado. 24 tests lib total. Commit `9d2d5319`.

## Frontend

### DESKTOP-07: Frontend MVP
- **Fecha:** 2026-08-06
- **Resultado:** ✅ React+Vite MVP: ConnectionPanel, IngestForm, SearchBar, ResultsList, hook, bridge `vanta.ts`. Bridge tipado + 5 componentes + single-file CSS; `npm run build` (tsc+vite) exit 0. Commit `10c161aa`.

## Conexión server

### DESKTOP-08: Cliente IQL tipado
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `ServerClient` wrapper reqwest (json) config url/port/token timeout; mapea 8 statements IQL (health, metrics, query POST `/api/v2/query`, put/get/delete/list/search) con auth Bearer; `success:false` → `VantaError::Http`. Validado contra `HTTP_API.md`/`cli_server.rs`. 28 tests (11 mock + 17 unit). Commit `b7aff3a0`.

### DESKTOP-09: ServerConnection
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Implementa el trait sobre el client IQL; connect valida auth/health; timeouts; `success:false` como error de dominio. `ServerConnection` delegando a `ServerClient`, timeouts → `VantaError::Timeout`, capabilities [Http]; test e2e con server real gateado por `VANTADB_TEST_SERVER=1`. 21 tests lib + 2 e2e. Commit `a5f2da1b`.

### DESKTOP-10: Wire Server en commands + UI
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `ConnectionSelector.tsx` (loopback-only url/port/token, bridge invoke sin fetch directo); `vanta_connect` ya soportaba `via:"server"`. `npm run build` + `cargo check` exit 0. Commit `7619c3cb`.

### DESKTOP-11: Spawn manager subproceso MCP
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `McpSpawn` con `tokio::process::Command`, stderr→log temporal, timeout; spawn+kill limpio; test gateado si falta binario. Localiza `vantadb-server`; flag `--mcp`; stdio piped. Commit `d62c1c0c`.

## Vanta Studio (Fase 0 — consola human-facing)

### VS-02: MARK variante desktop (asistente de datos)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ Port de la mascota MARK de `web/src/components/vanta/mark/` a `desktop/src/components/mark/` (4 archivos) sin Anime.js: `use-mark-interaction.ts` (follow rAF lerp exp τ 60/130ms, squint React puro, blink WAAPI cierre 60ms inQuad→hold 50ms→apertura 120ms outQuad con setAttribute final, reduced-motion sin follow/blink), `Mark.tsx` (grafo 10 nodos/18 edges, face ring/SMIL glow/esfera/ojos, SfxLabels, tag, hint), `mark-studio.tsx` (variante data-driven idle/loading/empty/error), `mark.css` (clases namespaced `.vmark-*`, keyframes pulse/ambient `transform-box: fill-box`, media query reduced-motion). `npm run build` verde (3×). Commit `2573d8a5`.

## Vanta Studio (Fase 1 - explicabilidad y tiempo) — 9/9 ✅

### VS-CORE-03: Exponer `explain` en el bridge desktop
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `SearchQuery.explain` + `SearchResult.explanation` (espejo 1:1 de `VantaSearchExplanationHit`) en `desktop/src-tauri/src/connections/` + `vanta.ts`. Commit `2a1f3012`. Core intacto (consumir, no crear).

### VS-12: Audit log en desktop
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `NativeConnection::open` configura `audit_log_path` (default `<storage>/audit.jsonl`); comando `vanta_audit_events` (tail/filtros/cursor) + `auditEvents()` en `vanta.ts`. Commit `2a1f3012`.

### VS-13: Lente RETRIEVAL (desglose de score)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `desktop/src/components/lens/retrieval/` (retrieval-core.ts, ScoreBars.tsx, RetrievalLens.tsx): barras apiladas BM25/HNSW/RRF con encoding redundante, consume `explain`. Commit `0411117e`.

### VS-15: ACTIVITY + Timeline
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `desktop/src/components/activity/` (ActivityPanel + Timeline agrupada hora/día, filtros namespace/op/outcome con cursor, empty state honesto). Commit `0411117e`.

### VS-16: Deep links `vanta://` + export + reporte markdown
- **Fecha:** 2026-08-18
- **Resultado:** ✅ plugins `tauri-plugin-deep-link@2.4.9` + `single-instance@2.4.3` (verificados vs docs oficiales); `parseVantaUrl` + `useDeepLink`; export JSONL de la vista actual; reporte markdown copiar/descargar. 14/14 tests. Commit `0411117e`.

### VS-17: Favoritos/historial + Copy-as
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `store/favorites.ts` + `store/search-history.ts` (localStorage), grupos FAVORITOS/HISTORIAL en CommandPalette, botones copy-as JSON/key/markdown. Sin deps nuevas. Commit `cdcaf268`.

### VS-18: Encoding redundante chips/badges (A11y)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `.stripes-neon` + ícono/texto en TTL/vector/metadata/tab activo — AA claro+dark, reduced-motion. Commit `cdcaf268`.

### VS-14: Historial+Diff entre versiones
- **Fecha:** 2026-08-18
- **Resultado:** ✅ `historial-diff.ts` + `historial-tab.tsx` (tab HISTORIAL en Inspector): lista v1..vN, diff payload/metadata/vector, revert explícito P6. Desbloqueada por VS-CORE-07. Commit `5796b2f9`.

### VS-CORE-07: Retención de versiones (core) — ver `activo/core-engine.md`
- **Fecha:** 2026-08-18
- **Resultado:** ✅ Core: `src/sdk/version_history.rs` + partición Fjall `Versions` (cap 32 FIFO aprobado); bridge: `getVersion`/`versions` en `vanta.ts`. Commits `be0812a4`/`b6997e59`.

### VS-CORE-04: Exportar selección/query con filtro
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `export_namespace(path, namespace, filter: Option<VantaMemoryFilter>)` aditivo + WASM `export_namespace_filtered` + TS `exportNamespace(path, namespace, filter?)` + comando Tauri `vanta_export_namespace`. Commits `a62088b7`/`7429f81a`. Desbloquea batch export (OP-02).

### VS-CORE-05: Batch delete con filtro desde UI
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `delete_by_filter` en WASM → TS → bridge Tauri (`vanta_delete_by_filter(namespace, filter) -> u64`) con protección anti borrado total (filtro vacío rechazado). Commits `15172349`/`39a6369c`. Desbloquea batch delete (OP-02).

### VS-CORE-06: IQL en desktop (bridge vanta_query + autocompletado)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ DTO `VantaQueryResult` (Read/Write/StaleContext) + comando `vanta_query` + `vanta_iql_autocomplete` (shim core-side sobre `parse_statement`) + wrapper `queryIql()` en `vanta.ts`. Commit `ebf9acc1`. Desbloquea IQL console (GRAFO-03).

### GRAFO-01: Bridge Tauri de grafos (bfs/dfs + degree)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ DTOs `VantaGraphNodeInfo`/`VantaGraphEdgeInfo`/`VantaGraphTraversalResult` + trait graph_bfs/graph_dfs/graph_degree (default Unsupported) + comandos `vanta_graph_bfs/dfs/degree` + WASM `graph_filtered_traversal`/`graph_degree`. Commit `b5eaabad`.

### GRAFO-02: Visor R3F force-directed (toon+outline manga)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ Canvas R3F (r3f v9 + drei v10 — React 19.1.0 real) con d3-force@3 en tick manual por useFrame (positionsRef, cero re-renders), toon+outline, expand incremental con reheat alpha, prefers-reduced-motion → radial estático. Commit `c23b1761`. Re-feedback D5 del usuario: force-directed obligatorio.

### GRAFO-03: IQL console embebida (CodeMirror + autocompletado + highlight)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `IqlConsole.tsx` (@uiw/react-codemirror@4.25.11) + CompletionSource → `iqlAutocomplete` + Ctrl+Enter → `queryIql()` → highlightIds → halo cian; historial localStorage `vanta.iql.history`. Commit `f62548a2`.

### ESPACIO-01: Scatterplot UMAP en worker (regl-scatterplot + lasso)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `projection.worker.ts` (UMAP-js fitAsync cancelable, seed mulberry32, NDC [-1,1], cap 100k) + `SpaceLens.tsx` (regl-scatterplot: zoom/pan, hover tooltip, lasso SHIFT+drag, color por namespace) + surface "espacio". Commit `0e772ba3`.

### ESPACIO-02: Mapa como herramienta (selección lasso → batch ops + undo)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ Selección lasso → export JSONL client-side (`recordsToJsonl`+`downloadText`) + eliminar con undo batch (`undoStore.softDeleteBatch`, 1 entry + snapshot) + confirmación con cantidad. Decisión: `deleteByFilter`/`exportNamespace` no expresan "key ∈ set" → client-side. Commit `889000ed`.

### OP-01: Import CSV/JSON pegado
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `parseImport.ts` (CSV `key,payload,metadata_json`/JSONL/array, preview editable ✓/✗, máx 1000, chunking 50) + `ImportPaste.tsx` lazy + botón ⤒ IMPORT en MEMORIAS + `key={gridKey}` remount. Commit `6fc4df91`. 18/18 vitest.

### OP-02: Batch ops en grid (selección múltiple + export/eliminar con undo)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ Checkbox de fila + select-all (página actual) → barra: Exportar (n) .jsonl client-side + Eliminar (n) con confirmación + undo snapshot (`softDeleteBatch`). Selección por `Set<string>` de `${ns}:${id}`. Fix a11y (sort button solo envuelve columnas sortables). Commit `a88cd1b0`. Retomado de sub-agente cancelado.

---

## Fuentes
- `docs/progreso/README.md` §Detalle de Tareas Completadas (DESKTOP-01..11) / snapshot-2026-08-07.
- `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md`.

### ERR-015 (shutdown gracioso) — migrado 2026-08-12 (ver docs/progreso/README.md)
