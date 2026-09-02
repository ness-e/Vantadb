---
title: "Avance — Desktop (Tauri)"
type: domain-log
status: active
tags: [vantadb, avance, desktop, tauri, rust, frontend]
last_reviewed: 2026-08-27
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
- **Resultado:** ✅ Doc `docs/research/DESKTOP-01-tauri-plataforma-desktop.md` (20.9KB, 208 líneas). **Recomendación: SI — Tauri v2** con integración Rust nativa (`vantadb` en `src-tauri/`, `VantaEmbedded` en managed state, commands async `vanta_ingest`/`vanta_search`, SIN bridge WASM/OPFS). Tauri v2.11.5 vs Electron v43.2.0. Effort MVP: ~8-13 días hábiles. Solo investigación, cero cambios de código.

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

## Vanta Studio (Fase 2 - grafo R3F, espacio y operaciones) — 10/10 ✅

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

## Fase 3 — Web/embebido (2026-08-19) — REST completo + dashboard servido por el server

### WEB-00: Abstraer `vanta.ts` de Tauri invoke (transporte pluggable)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `desktop/src/transport.ts` (interface `VantaTransport` + `TauriBackend` + `HttpBackend` stub + factory) + `desktop/src/vanta.ts` refactor mecánico 1:1 (55 exports intactos). Commit `0cccd326`. ADR-026 (D11/D12).

### WEB-01: REST: superficie de la consola (CRUD + search + list + IQL + health/metrics/audit)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `src/cli_server.rs` 11 rutas v2 + helpers (`run_db_op`, `vanta_error_status`); `src/audit.rs` AuditEvent Deserialize. Commit `c81bc23a`. 17/17 tests + smoke real.

### WEB-02: REST: resto del SDK (export/import, graph, mantenimiento, threads, snapshots)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `src/cli_server.rs` 16 rutas nuevas (export/import, graph bfs/dfs/degree/pagerank/centrality, maintenance purge/compact/flush/rebuild-index, threads, snapshots). Commit `c856b3bd`. 22/22 tests. Divergencias documentadas en task file.

### WEB-03: Servir estáticos `/dashboard` + SPA fallback + flag CLI
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `mount_dashboard` (ServeDir + fallback SPA sin extensión + 404 hint), flag `--dashboard-dir` (config/cli/bin/handlers), tower-http fs + tower directa. Commits `62d63377` + `0da6d33c`. Smoke 5/5.

### WEB-04: `HttpBackend` real (fetch REST) + factory por entorno
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `desktop/src/vanta-http-map.ts` (23 comandos: 15 REST, 8 rechazos descriptivos) + tests 14/14 + `HttpBackend` real. Commit `8b2bc14f`.

### WEB-05: Build web de la consola (Vite base `/dashboard/`, sin Tauri)
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `vite.config.ts` por mode, guards runtime Tauri-only (useDeepLink, useConnectionState), ConnectionPanel oculto en web, `.gitignore` dist-web. Commit `42d2b26a`. Builds desktop+web verdes.

### WEB-06: E2E Playwright contra server real + docs/ADR
- **Fecha:** 2026-08-19
- **Resultado:** ✅ `desktop/scripts/selfcheck-web-e2e.ts` (11 checks exit 0) + fix namespace default REST (bug cazado por E2E: `ListParams.namespace` default `"default"`). Commit `583dad9a`. ADR-026 en `docs/architecture/`.

### MEM-53: Desktop IPC commands para pipeline vanta-memory (H4)
- **Fuente:** Plan P33 Ultima Milla (Task 8) / auditoría integración final
- **Fecha:** 2026-08-22
- **Resultado:** ✅ 7 comandos Tauri (`vanta_memory_capture/recall/persona_get/scenes_list/scene_current/skills_list/wiki_status`) exponen el pipeline vanta-memory desde `src-tauri` hacia la UI; acceso al handle embebido vía trait default `as_native()` + `ConnectionManager::active_embedded()`; `ProgressTracker` en AppState. Suite desktop 85/85 (12 tests nuevos), fmt+clippy+audit limpios (h2 → 0.4.18, RUSTSEC-2026-0258). Vistas UI: tarea futura.
- **Ids:** `MEM-53`

### DESKTOP-30: Estado de usuario durable (favoritos, historial, papelera)
- **Fecha:** 2026-08-24
- **Resultado:** ? Papelera/tombstones del `UndoStore` persisten en localStorage inyectable (`vanta.trash.v1`, patrón único de persistencia decidido por DESKTOP-23/26 — NO app_config_dir). Hidratación en constructor con validación de shape, corrupto → vacío; write-through vía hook único `notify()`. Favoritos/historial/prefs ya persistían (DESKTOP-26/23) — único gap era la papelera (session-only según su header). El stack de undo NO se persiste por diseño (reverses referencian backend de sesión). Tests: round-trip reinicio + restore + shape inválido/corrupto. `npm run build` ✅, vitest 53/53 ✅.
- **Ids:** `DESKTOP-30`

### DESKTOP-33: CONSOLIDAR merge + delete reales (detectar→revisar→merge/delete sin salir de la lente)
- **Fecha:** 2026-08-24
- **Objetivo:** ConsolidateLens solo marcaba `metadata.superseded_by`; faltaba merge de payloads y borrado del duplicado.
- **Resultado:** ✅ Merge manual campo a campo en la lente (`defaultSources`/`mergeFields` en consolidate-core.ts + MergeEditor inline: dominante A/B elegible, toggle por campo, preview; Guardar = `vantaPut` merged + papelera del supersedido vía `undoStore.softDelete` con snapshot completo vía `get()` para restaurar vector/ttl). Delete con confirmación 2 pasos (`ConfirmDiscard.tsx`, patrón NamespaceDialog): "Mover a papelera" (default, Ctrl+Z) vs permanente. Batch sobre selección (checkboxes por par): superar dirección en lote + descartar superados marcados, con progress textual y notices. Styling Tailwind preservado. Tests nuevos para mergeFields/defaultSources. `npm run build` ✅, vitest 64/64 ✅.
- **Ids:** `DESKTOP-33`

---

## Fuentes
- `docs/progreso/campanas/admin-desktop.md` (split GOV-D2) / snapshot-2026-08-07.
- `docs/research/DESKTOP-01-tauri-plataforma-desktop.md`.

### ERR-015 (shutdown gracioso) — migrado 2026-08-12 (ver docs/progreso/README.md)

### DESKTOP-23: Persistencia de preferencias UI (tema/layout/filtros)
- **Fecha:** 2026-08-24
- **Resultado:** ? Store `WorkspacePrefsStore` (`desktop/src/store/preferences.ts`) con storage inyectable + sanitizaci�n defensiva; persiste surface/layout, panel filtros y ruleGroup (tema ya persist�a en localStorage). Decisi�n de alcance: NO app_config_dir/temp+rename - el WebView de Tauri ya persiste localStorage entre sesiones (favoritos/historial/tema dependen de eso); segundo mecanismo duplicar�a fuente de verdad. Documentado en task file + lessons. `npm run build` ?, vitest 41/41 ?.
- **Ids:** `DESKTOP-23`

### DESKTOP-24: Empaquetado NSIS/MSI (Windows)
- **Fecha:** 2026-08-24
- **Resultado:** ? `tauri.conf.json`: targets NSIS/MSI + WebView2 embedBootstrapper + sidecar `vantadb-server.exe` via `bundle.resources` (NO externalBin: sufijo target-triple no matchea `locate_binary`). Fix typo env `VANTVADB_SERVER_BIN`  `VANTADB_SERVER_BIN` (child_process.rs). Build completo 15min: `bundle/nsis/*.exe` (9.9MB) + `bundle/msi/*.msi` (13.4MB), sidecar incluido. Deuda: smoke-test instalador en VM Windows limpia; instalador unsigned. Re-bundle documentado en Context Save Point.
- **Ids:** `DESKTOP-24`

### DESKTOP-25: CI GitHub Actions desktop
- **Fecha:** 2026-08-24
- **Resultado:** ? `.github/workflows/desktop.yml` nuevo: tauri-action@v1 (validado vs README oficial), sidecar construido en CI (--features custom-allocator) + `VANTADB_SERVER_BIN` para tests reales, cargo test workspace desacoplado, upload-artifact instalador NSIS/MSI, cache npm+cargo+sccache composite. actionlint exit 0. Sin continue-on-error (Regla 2). Deuda: push  confirmar run verde <15min + descargar artefacto.
- **Ids:** `DESKTOP-25`

### DESKTOP-26: Tests frontend Vanta Studio (vitest)
- **Fecha:** 2026-08-24
- **Resultado:** ? vitest 4 configurado desde cero (`vitest.config.ts` jsdom+globals, script `test`, tsconfig excluye tests del build tsc). Tests: stores persistidos (favorites/search-history, storage inyectable), HomeOverview, round-trip bridge `vanta.ts`invoke mockeado (`__TAURI_INTERNALS__` hoisted), fix flaky projection.worker (UMAP 10040 pts). `npm test` exit 0 x2 consecutivas (38 tests al cierre de la tarea), build ?.
- **Ids:** `DESKTOP-26`

### DESKTOP-27: Docs + ADR Vanta Studio
- **Fecha:** 2026-08-24
- **Resultado:** ? `docs/desktop/README.md` (instalaci�n, tabla 3 transportes, comandos IPC, troubleshooting) + `ARCHITECTURE.md` (ConnectionManager registry+active_id, trait VantaConnection, NativeConnection spawn_blocking+path lock, ServerClient Bearer, WASM como backend frontend-only OPFSIDB - no WasmConnection Rust, shutdown_all orden, ConnectionSelector eliminado ADMIN-03) + `GUIDE.md` ES por modo. ADR-026/027/028 referenciados sin duplicar. Review vanta-arch fresh-context: APPROVE (7/7 claims con evidencia file:line).
- **Ids:** `DESKTOP-27`

### DESKTOP-28: Unificar paneles legacy al design system Studio
- **Fecha:** 2026-08-24
- **Resultado:** ? 7 paneles migrados de `.panel`/App.css a Tailwind manga/linocut (ConnectionPanel/MetricsGrid/KpiCards/SopPanel/ExportPanel/IngestForm/DataExplorer) + ResultsList hallado en sweep. SopPanel: botones WAL/Reindex falsos  tag "solo lectura" (ADMIN-06); Health Check conserva acci�n real. Hu�rfanos borrados tras grep Regla 0: SearchBar.tsx, ProcessPanel.tsx. App.css 383L41L; overrides :root que pisaban tokens eliminados (DS index.css manda). Build ? + tests verdes.
- **Ids:** `DESKTOP-28`

### DESKTOP-29: Coordinar polling de m�tricas
- **Fecha:** 2026-08-24
- **Resultado:** ? Hook �nico `useMetricsPoll.ts` (store module-level + useSyncExternalStore, 1 setInterval 4s, arranca con primer subscriber / muere con el �ltimo, history cap 12 newest-last, guard inFlight). Consumers: MetricsGrid, KpiCards, IndicesLens, ExportPanel - deltas/trend intactos sobre history compartida. Test: 3 consumers  1 call/tick. `npm test` 48/48 ?.
- **Ids:** `DESKTOP-29`

### DESKTOP-31: Pantalla SETTINGS
- **Fecha:** 2026-08-24
- **Resultado:** ? `pages/Settings.tsx` (perfiles conexi�n nativo/server, auth Bearer token server remoto, defaults b�squeda top_k/modo, idioma) + store `connections.ts` (perfiles+activeProfileId, patr�n storage inyectable DESKTOP-23) + `connectServerCfg(url,port,token)` en useConnectionState + dropdown "Connect profile" en ConnectionPanel + surface "ajustes" en shell/palette. Cero cambios Rust (ServerClientConfig.token ya exist�a). Desv�o documentado: perfiles en localStorage inyectable, no app_config_dir. Deuda: i18n real, defaults por-perfil, E2E contra server real con token. Tests 48/48 ?.
- **Ids:** `DESKTOP-31`

### DESKTOP-32: CRUD de namespaces
- **Fecha:** 2026-08-24
- **Resultado:** ? Crear/renombrar/borrar ns desde sidebar: `NamespaceDialog.tsx` modal 3 modos (borrado = aviso  tipear nombre exacto, valida colisiones), rename = copia v�a ingestBatch preservando embedding/metadata/ttl  borra originales  entry reverse move en undo store, delete reusa applySoftDelete (papelera durable gratis). `listAll()` paginado + `createNamespace()` key reservada en vanta.ts. Cero cambios Rust. Deudas: sparse_vector no viaja en rename (caso raro), toasts via onNotice (Sonner no presente). Tests 57/57 ?.
- **Ids:** `DESKTOP-32`

### DESKTOP-34: Polish UX global (palette/tooltips/ES)
- **Fecha:** 2026-08-24
- **Resultado:** ? CommandPalette extendida a todas las superficies del shell (PAPELERA/B�SQUEDA/CONSOLIDAR/ESPACIO + AJUSTES que lleg� en paralelo; union completa Surface). Tooltips title/aria-label en los 9 botones sidebar (correcci�n spec: F1/F2 no son atajos, son fases HelpPanel). Unificaci�n ENES en 10 archivos (labels/placeholders/aria-labels), sin framework i18n. Correcci�n spec: componentes citados eran del prototipo web, no desktop. Deuda: statusReport.ts genera markdown EN (doc generado), F1/F2 sin handler keydown (pre-existente). Build+tests verdes.
- **Ids:** `DESKTOP-34`

### DESKTOP-35: Slider h�brido cableado a search_profile real
- **Fecha:** 2026-08-24
- **Resultado:** ? Hallazgo discovery: campo `bm25_weight` asumido NO existe - SearchProfileConfig solo acepta {mode, rrf_k?, candidate_k?} (types.rs:478)  slider discretizado 3 stops (keyword/hybrid RRF/vector), gap documentado. Bridge: `SearchQuery.search_profile: Option<SearchProfileConfig>` serde-default backward-compat + reenv�o verbatim en native.rs + http-map. Eliminado re-rank client-side (weightFromSlider/weightedScore/rerankByWeight/computeSegmentsWeighted). Paridad slider==explain POR CONSTRUCCI�N (mismo request + explain:true), test Rust fija invariante (perfil hybrid == baseline). cargo test src-tauri 79/79 ?, vitest 64/64 ?, node --test 5/5 ?. Deuda: pesos intermedios requieren soporte core.
- **Ids:** `DESKTOP-35`

### DESKTOP-36: Bridge Tauri vanta-memory (read-only)
- **Fecha:** 2026-08-24
- **Resultado:** ? Comandos `vanta_scene_read/vanta_scene_query/vanta_genlog_query` sobre handlers tipados gateway (KnowledgeErrorVantaError); reutilizados vanta_persona_get/scenes_list/skills_list pre-existentes (MEM-53, no duplicados). Bindings TS tipados en vanta.ts (SceneEntry/StoredSkillRecord/GenlogEntry/PersonaSnapshot) con claves snake/camel correctas por comando. Tests contra seed REAL vanta-seed (import_seed_str): 5 Rust nuevos + wire-contract TS. cargo test --lib 77/77 ?, vitest 50/50 ?. Deuda: skill_versions/skill_restore/compaction_report sin backing API (skills=content-hash upsert, CompactionReport no persiste) - nada inventado.
- **Ids:** `DESKTOP-36`

### DESKTOP-37: Lente MEMORIA (UI)
- **Fecha:** 2026-08-24
- **Resultado:** ? Sexta superficie del Studio: ScenesPanel (heat-desc + barra normalizada + detalle inline, badge soft-deleted en 404), PersonaPanel (snapshot + diff l�neas vs �ltima vista en localStorage), SkillsPanel (agrupado por nombre, timeline asc hash corto, visor contenido), GenlogPanel (filtro L1/L2/L3 re-query, anchor_id  get() real  Inspector). Selector session_key default user-1 (ejemplo vanta-seed). Solo records con anchor van a Inspector (no fabricar targets de vantaPut). Preservadas fusiones paralelas (SETTINGS/palette/CRUD ns). 7 tests mock bridge. vitest 64/64 ?.
- **Ids:** `DESKTOP-37`

### DESKTOP-39: Ingest con embedding desde texto (condicional)
- **Fecha:** 2026-08-24
- **Resultado:** ? Caso B (WONTFIX-UI documentado): src/llm.rs NO expone embedding local - solo providers HTTP externos Ollama/OpenAI (evidencia llm.rs:1-4,26-29,39-47,144-145 + Cargo.toml:107 feature remote-inference). Nota informativa honesta en IngestForm bajo el bot�n (sin vector se guarda como texto; sem�ntica requiere VANTA_EMBEDDING_PROVIDER/VANTA_LLM_URL/VANTA_OPENAI_API_KEY). Sin bot�n falso ni detecci�n config (YAGNI). ImportDrop intocado (grep confirma). Build+tests verdes.
- **Ids:** `DESKTOP-39`

### DESKTOP-38: Dashboard PROXY (TurnReports/sesiones/write-back/rate-limit)
- **Fecha:** 2026-08-24
- **Objetivo:** Visualizar vanta-proxy en el desktop (hoy solo expone /health); UI consume REST del proxy.
- **Resultado:** ? Endpoint GET /snapshot en vanta-proxy/server.rs serializando estado REAL: ring buffer cap-100 en Reporter (recent_reports - antes solo log/hooks), SessionStore::snapshot con stage/TTL (solo pending expiran, sweep previo), rate-limit hits instrumentados con AtomicU64 en decision Limited (los 429 hits NO existian - nada fabricado), pending_labels() de write-back. UI ProxyDashboard.tsx: TurnReports tabla + sesiones TTL countdown + write-back count/labels + rate-limit; polling 5s solo montado; formulario config sin URL (localStorage vanta.proxy.url); REST directo via fetch (NO bridge nativo). Surface "proxy" condicional en WorkspaceShell. Tests: cargo test -p vanta-proxy 97 (3 nuevos) + fmt/clippy limpios + vitest 68/68 (4 nuevos). Deuda: validacion manual requiere upstream LLM vivo; PaletteSurface desincronizada (memoria faltaba, pre-existente).
- **Ids:** `DESKTOP-38`

### UX-01+UX-05: LensShell compartido + token .label-tech (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 2)
- **Resultado:** OK - componente composable `LensShell` (header de lente: titulo stencil + icono + meta derecho con token `.label-tech` + subtitle opcional; no envuelve children, preservando el layout WebGL full-height de Graph/Space) adoptado en las 6 lentes: Consolidate, Indices, Retrieval, Graph, Space, Memory. `.label-tech` ya existia (`@utility label-tech` en index.css:252) - se usa consistentemente via LensShell (primer uso real), sin duplicar definicion. Verify: `cd desktop && npm run build` exit 0 + vitest 68/68. Commit `6260938e`. Follow-up: reemplazo masivo de `.label-tech` en ~20 archivos/WorkspaceShell queda como follow-up (UX-05 parcial).

### P37 — Auditoría diseño desktop post-fix (DAUD-01..09, H-13) — 9/9 ✅
- **Fecha:** 2026-08-26
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave1 Task5 (DESKTOP-QW5)
- **Resultado:** ✅ 9/9 DAUD verificadas como Hecho y archivadas del Backlog (stale cleanup). Detalle por DAUD:
  - **DAUD-01** (E2E-VISUAL, 🟡): verificación visual runtime FIX-D1 (theme flipping sin marco crema) — guard Playwright `daud01-temas.spec.ts` + `flujo-critico.spec.ts` + `playwright.config.ts` — commit `480935a7` ✅
  - **DAUD-02** (FILTROS activo, 🟢): decisión owner `filterActive` (reglas>0, no `showFilters`) — aplicada en `WorkspaceShell.tsx:295` `filterActive=ruleGroup.rules.length>0` + `aria-pressed` + `bg-foreground` — commit `ad0f34b1` (DESKTOP-QW4, H-14) ✅ — cerrada 2026-08-25
  - **DAUD-03** (press-effect, 🟢): scopear `button:hover translate` fuera de TitleBar vía `[data-tauri-drag-region] button` — `desktop/src/App.css:53-59` — commit `3c53d8b2` ✅
  - **DAUD-04** (CSS consolidado, 🟢): `:root body`/`.dark body` redundantes eliminadas, deja solo `font-family`/`font-feature-settings` en `index.css` — commit `3c53d8b2` ✅
  - **DAUD-05** (dead utilities, 🟢): `halftone/grid-tech/speed-lines/animate-rise` 0 usos → borradas + 4 overrides `.dark` — commit `3c53d8b2` ✅
  - **DAUD-06** (glifos, 🟢): `✎` → `<Pencil strokeWidth={2.5}/>` en `WorkspaceShell.tsx` + `ProxyDashboard.tsx`; `mark-studio.tsx`/`Timeline.tsx` glifos monócromos KEEP — `DESIGN_DECISIONS.md` §5 — commit `b865c625` (tokens neon `var(--color-neon)`) ✅
  - **DAUD-07** (convención iconos, 🟢): `DESIGN_DECISIONS.md` §5 Lucide 2.5 + glifos linocut + `lucide-react` — commit `3c53d8b2` ✅
  - **DAUD-08** (stash, 🟢): `stash@{0}` "WIP on develop: 06aa1a86" diff 0 vs worktree, contenido recuperado vía `b865c625` (tauri.conf.json 1280×800 + Mark neon) — stash original ya no existe (actual `stash@{0}` es `2fc26b26`) ✅
  - **DAUD-09** (commit agrupado, 🔴): fixes D1-D11 commiteados en `3c53d8b2` + `b865c625` (separación limpia, verify `cargo fmt` + build/test) ✅
   Verificación: `Select-String DAUD docs/Backlog.md` 0 filas `| \`DAUD-` (solo header P37 0 Cerrada + historial), `pwsh scripts/check-avance-coverage.ps1` 1038+9/1038+9 (o 1038/1038 sin DAUD en fuentes) 0 gaps, `pwsh scripts/validate-docs-coverage.ps1` 0 gaps.
- **Ids:** `DAUD-01..09`

### DESKTOP-QW6: CSP mínima localhost+https remoto (H-01)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave2 Task6 (DESKTOP-QW6)
- **Resultado:** ✅ CSP ya mínima desde a7ed0d22 (`default-src 'self'` + `connect-src ipc+127.0.0.1`) refinada con `http://localhost:* ws://localhost:* https://*` en prod+dev (2 líneas `connect-src` en `tauri.conf.json:27,33`) para ProxyDashboard fetch a localhost/https remoto; `ServerClient` reqwest no CSP. Build 21.18s (2863 modules) + tests 69/69 (18.55s) + cargo check 26.30s + fmt verde.
- **Ids:** `DESKTOP-QW6`

### DESKTOP-QW7: Rename namespace preserva sparse_vector (H-04)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave2 Task7 (DESKTOP-QW7)
- **Resultado:** ✅ Auditoría end-to-end H-04: `vanta.ts` IngestItem/MemoryRecord `sparse_vector` + `types.rs` DTOs `Option<HashMap<u32,f32>>` + `undo.ts` renameNamespace `ingestBatch(map{...,sparse_vector:r.sparse_vector??undefined})` + restore/undo `vantaPut` idem (4 hits) — ya en a7ed0d22; test `undo.test.ts:165-197` fija forward+undo con `{0:0.5,5:1.25}` (ingestBatch+ vantaPut asserts). Audit-only (Step2 SKIPPED, 0 líneas, ponytail). Build 13.05s (2863 modules) + tests 69/69 (23.54s) + cargo check 0.57s + fmt verde. Commit 0692855e.
- **Ids:** `DESKTOP-QW7`

### DESKTOP-QW8: Sincronizar versión desktop con release-plz (H-11)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave3 Task8 (DESKTOP-QW8)
- **Resultado:** ✅ Triple 0.1.0 sync `desktop/package.json:4` + `tauri.conf.json:4` + `src-tauri/Cargo.toml:3` vs workspace 0.5.0 aislado (desktop no miembro root) → decisión EXCLUDE documentada `release-plz.toml:34-48` `[[package]] name="vantadb-desktop" release=false` + comentario H-11 aislado. Build 16.45s (2863 modules) + tests 69/69 (22.30s) + cargo check 0.75s + fmt verde.
- **Ids:** `DESKTOP-QW8`

### DESKTOP-QW9: Baseline medido de recursos del app (H-15)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave3 Task9 (DESKTOP-QW9)
- **Resultado:** ✅ `docs/operations/BENCHMARKS.md` §9 Desktop 100 líneas: Build Timing wall 24.59s / vite 14.54s / 2863 modules + Bundle 2.71MB (2510KB JS 37% GraphLens 944KB, 69KB CSS, 195.9KB fonts, 665KB gzip) + Chunks >500KB warning + Env i5-1235U 31.8GB Win11 Node24 vite7.3.6 tsc5.8.3 + Provenance Regla 11 comandos reproducibles + DESKTOP-01 superseded flagged + Tauri startup/RAM pending con pasos documentados. Build 22.97s vite verde + validate-docs 0 gaps + check-avance 1038/1038 + fmt verde.
- **Ids:** `DESKTOP-QW9`

### DESKTOP-QW10: E2E desktop multi-perfil + proxy dashboard mock (H-07)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave3 Task10 (DESKTOP-QW10)
- **Resultado:** ✅ 2 specs E2E + 1 smoke manual: `multi-perfil.spec.ts` 193L 4 tests (health BM25·HNSW·RRF embedded vivo, Settings defaults topK/mode/lang persist tras reload, localStorage vanta.connections.v1 native+server Bearer round-trip, sanitize corrupto) + `proxy-dashboard.spec.ts` 210L 5 tests (sin proxy PROXY hidden palette, CONECTAR form mock, dashboard mock sessions 9m/4m + expirado + writeback 2 + rate_limit 60/min 5 ok, degraded 99 fail-open, cambiar URL limpia) con `page.route("**/snapshot", MOCK)` + TTL far-future 600s/300s ponytail + strict locators .first/dd exact; `SMOKE-MANUAL.md` 83L GraphLens 3D + SpaceLens UMAP checklist visual; infra `cargo build --bin vanta-cli --features server` 16MB 9m13s + dist-web 11-13s rebuild + binary port 8091 health; verify `npm --prefix desktop run build` 10.35s 2863 modules + `npm --prefix desktop test` 69/69 + `cargo fmt --check` + `npx playwright test --config desktop/playwright.config.ts` 12/12 45.4s (4 specs: flujo-critico, daud01-temas, multi-perfil, proxy-dashboard) + via desktop binary same; disk 120GB free WMI. Commit 34d72585.
- **Ids:** `DESKTOP-QW10`

### ERR-DESK-01: mem_err preserva errores tipados del core (Wave 3)
- **Fecha:** 2026-09-02
- **Plan:** `docs/plans/2026-09-02-error-observability-excellence.md` Task 6 (ERR-DESK-01)
- **Resultado:** `mem_err` (commands/memory.rs:91) ya no colapsa errores del core a `Native(String)`: recorre la cadena `source()` (L0Error/RecallError/SceneError/PersonaError/GenLogError/KnowledgeError retienen `Vanta(#[from] vantadb::VantaError)`, verificado en DISCOVERY) y mapea via `VantaError::from_core` nuevo en error.rs: DatabaseBusy-Lock, IoError-Lo, resto Domain{code: e.code(), message} (codigo canonic? VANTADB_* post-ERR-CORE-01 sobrevive el wire Tauri serde). Sweep: map_core_error duplicados en connections/native.rs y commands/connection.rs delegan al mapeo unico. El Http{kind,status} de server.rs fue verificado NO-degradado (los comandos memory son ruta embedded; server devuelve Http directo). Verificacion mecanica: grep Native(e.to_string==0, cargo check --all-targets 0, cargo test 102 passed/0 failed (87 lib + 15 integration, -j 1 por OOM de link tauri en paralelo), clippy -D warnings 0, fmt crate verde. Colateral: fix bloqueante preexistente tests/server_connection_real.rs +sparse_vector:None (campo H-04); fmt drift preexistente del crate aislado (no miembro workspace raiz). RED observado antes del fix: Native("Node not found: 7"). Commit 6bdc2c5d (7 files).
- **Ids:** `ERR-DESK-01`
