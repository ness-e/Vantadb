# Vanta Studio (P26) — consola desktop Fases 0-3

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Vanta Studio (P26): Fase 0 — consola human-facing desktop (2026-08-18)

> Plan `docs/plans/2026-08-18-vanta-studio-fase0.md` (archivado). Registro completo por tarea; resumen ejecutivo en `docs/Backlog.md` P26.

### VS-00: Prototipo HTML Fase 0 core (3 pantallas navegables)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** fijar el diseño (HOME/MEMORIAS/Inspector) antes de codificar React; Tailwind v4 browser CDN con tokens reales de `web/`.
- **Resultado:** ✅ `desktop/prototype/index.html` + 3 screenshots (validado con Playwright 1440×900).
- **Ids:** `VS-00`

### VS-01: Tailwind v4 + tokens manga/linocut + tema toggle
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** base estética de toda la Fase 0 — tokens de `web/` replicados, paleta dark propia, a11y focus + reduced-motion.
- **Resultado:** ✅ `desktop/package.json`, `desktop/src/index.css`, `main.tsx`, `index.html` — commit `fa6c1427`. Build verde.
- **Ids:** `VS-01`

### VS-03: Workspace unificado (WorkspaceShell)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** reestructurar App.tsx a Sidebar + Topbar + superficie central + Inspector (P4 anti split-attention).
- **Resultado:** ✅ `desktop/src/App.tsx` + `WorkspaceShell.tsx` — commits incrementales `f8250aea`/`399adaa6`/`270b34ba` (sin commit propio, shell creció con las tareas que monta).
- **Ids:** `VS-03`

### VS-04: HOME/overview (Fix 1)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** 7 cards de resumen con encoding redundante (Shneiderman overview-first).
- **Resultado:** ✅ `desktop/src/components/home/HomeOverview.tsx` — commit `f8250aea`.
- **Ids:** `VS-04`

### VS-05: MEMORIAS grid virtualizado
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** TanStack Table v9 + Virtual con paginación por cursor (VS-CORE-01).
- **Resultado:** ✅ `desktop/src/components/DataExplorer.tsx` — commit `14b6bfc8`.
- **Ids:** `VS-05`

### VS-06: Inspector master-detail (4 tabs + CodeMirror + commit explícito)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** General/Payload/Metadata/Vector con edición JSON (CodeMirror 6), nunca auto-guardar.
- **Resultado:** ✅ `desktop/src/components/inspector/*.tsx` + slice WorkspaceShell — commit `399adaa6`.
- **Ids:** `VS-06`

### VS-07/VS-08/VS-09: Filtros compuestos + undo/papelera + command palette
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** P5 filtros AND/OR visuales; P8 soft-delete + restore; P9 Ctrl+K con acciones.
- **Resultado:** ✅ `FiltersBuilder.tsx`/`undo.ts`/`TrashLens.tsx`/`CommandPalette.tsx` — commit `270b34ba` (VS-07/08/09 juntos).
- **Ids:** `VS-07`, `VS-08`, `VS-09`

### VS-10: Bridge Tauri put/update (crítico)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** hallazgo del revisor — no existía `put` en el bridge; destraba el Inspector (Guardar/TTL).
- **Resultado:** ✅ `desktop/src-tauri/src/commands/data.rs` + `vanta.ts` — commit `d5453682`.
- **Ids:** `VS-10`

### VS-11: Bridge DTO enriquecido (crítico)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** `MemoryRecord` sin version/node_id/timestamps/TTL/vector → imposible grid + inspector.
- **Resultado:** ✅ `desktop/src-tauri/src/connections/types.rs` + `vanta.ts` — commit `98c2d6c8`.
- **Ids:** `VS-11`

### VS-CORE-01: Cursor/paginación en bridge desktop
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** el core ya tenía cursor; faltaba exponerlo en Desktop (`vanta_list` + `listPage`).
- **Resultado:** ✅ bridge Tauri + `vanta.ts` — commit `424ffe03`.
- **Ids:** `VS-CORE-01`

### VS-CORE-02: Contadores por namespace + stats TTL
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** `namespace_stats` single-pass para sidebar + HOME.
- **Resultado:** ✅ `src/sdk/api.rs` + `types.rs` + serialización — commit `822f7742`. 1785 lib tests verdes.
- **Ids:** `VS-CORE-02`

### VS-02: MARK variante desktop (asistente de datos)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Fecha:** 2026-08-18
- **Objetivo:** port del componente MARK (mascota VantaDB) de web → desktop sin Anime.js, con variante data-driven para el workspace.
- **Resultado:** ✅ 4 archivos en `desktop/src/components/mark/` (`use-mark-interaction.ts`, `Mark.tsx`, `mark-studio.tsx`, `mark.css`) — commit `2573d8a5`. Follow rAF lerp exp (τ 60/130ms), squint React puro, blink WAAPI (cierre 60ms inQuad → hold 50ms → apertura 120ms outQuad), pulse nodos CSS keyframes `transform-box: fill-box`, SMIL glow condicional a reduced-motion; variante `MarkStudio` (idle/loading/empty/error); CSS plano namespaced `.vmark-*` (VS-01 Tailwind pendiente). `npm run build` verde (3×). Web de referencia intacta.
- **Ids:** `VS-02`

### VS-CORE-03: Exponer `explain` en el bridge desktop (re-scopeado: consumir, no crear)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** el core ya produce `VantaSearchExplanation`; el bridge desktop era el único que no lo exponía. Añadir `SearchQuery.explain: bool` + `SearchResult.explanation: Option<ExplanationHit>` (espejo 1:1 de `VantaSearchExplanationHit`).
- **Resultado:** ✅ `desktop/src-tauri/src/connections/{types,native,server,mod,manager}.rs` + `desktop/src/vanta.ts` — commit `2a1f3012`. 41 lib + 15 integración verdes; `npm run build` verde. Core intacto.
- **Ids:** `VS-CORE-03`

### VS-12: Audit log en desktop (configurar `audit_log_path` + comando `vanta_audit_events`)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** el core ya escribe el JSONL (opt-in); el desktop ni lo configuraba ni podía leerlo.
- **Resultado:** ✅ `NativeConnection::open` configura audit (default `<storage>/audit.jsonl`), comando `vanta_audit_events` (tail/filtros/cursor) + `vanta.ts` `auditEvents()` — commit `2a1f3012`. 11 tests audit verdes; build verde.
- **Ids:** `VS-12`

### VS-CORE-07: Retención de versiones históricas en `VantaMemoryRecord` (D2 completo)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** retener snapshots de versiones (cap 32 FIFO aprobado, snapshot del record nuevo, import sin snapshots) + API `get_version`/`versions` + exposición bridge.
- **Resultado:** ✅ `src/sdk/version_history.rs` (nuevo, 11 tests), partición Fjall `Versions`, hooks en put/put_batch/delete/purge_expired, `VantaConfig.version_history_limit`; bridge: `vanta_get_version`/`vanta_versions` + `vanta.ts` `getVersion`/`versions` — commits `be0812a4`/`b6997e59`. 1785 lib tests verdes (1 fallo preexistente `test_consolidate_node_with_binary_vector` en maintenance.rs, fuera de scope); 42 desktop lib verdes. Doble consumidor: P26 Studio (Historial+Diff) + P27 memory.
- **Ids:** `VS-CORE-07`

### VS-13: Lente RETRIEVAL (¿por qué recuperó esto?)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** barra de consulta + desglose de score como barras apiladas (BM25/HNSW/RRF) usando `explain` de VS-CORE-03.
- **Resultado:** ✅ `desktop/src/components/lens/retrieval/` (retrieval-core.ts + ScoreBars.tsx + RetrievalLens.tsx) + slice aditivo en WorkspaceShell — commit `0411117e`. Self-check 15 asserts PASS; build verde.
- **Ids:** `VS-13`

### VS-15: ACTIVITY + Timeline (audit log filtrable y agrupado)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** Timeline agrupada por hora/día + Activity filtrable (namespace/op/outcome) con cursor de VS-12; empty state honesto si audit no configurado.
- **Resultado:** ✅ `desktop/src/components/activity/` (logic.ts + EventChip.tsx + Timeline.tsx + ActivityPanel.tsx) + slice aditivo WorkspaceShell — commit `0411117e`. Self-check fixture JSONL PASS; build verde.
- **Ids:** `VS-15`

### VS-16: Deep links `vanta://` + export de vistas + reporte markdown
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** URI scheme `vanta://` (Tauri v2, plugins deep-link 2.4.9 + single-instance 2.4.3 verificados contra docs oficiales), export JSONL de la vista actual, reporte markdown con copiar/descargar.
- **Resultado:** ✅ `lib.rs` (pending_deep_links + register_all), `tauri.conf.json` (scheme), `vanta.ts` `parseVantaUrl`, `components/export/`, `useDeepLink.ts` — commit `0411117e`. 14/14 tests node; deep link manual OK (1 solo proceso); build verde.
- **Ids:** `VS-16`

### VS-17: Favoritos/historial de búsqueda + Copy-as
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** favoritos ★ (localStorage) + historial de búsqueda re-ejecutable + copy-as (JSON/key/markdown) sin deps nuevas.
- **Resultado:** ✅ `desktop/src/store/{favorites,search-history}.ts`, `components/copy/`, grupos FAVORITOS/HISTORIAL en CommandPalette, ★+copiar en DataExplorer/Inspector — commit `cdcaf268`. Self-check roundtrip PASS; build verde.
- **Ids:** `VS-17`

### VS-18: Encoding redundante (color + ícono + texto) en chips/badges (A11y)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** estados solo-color ganan ícono+texto/patrón (TTL, vector, metadata duplicada, tab activo) — AA en claro y dark, reduced-motion respetado.
- **Resultado:** ✅ `index.css` (.stripes-neon) + DataExplorer/GeneralTab/MetadataTab/Inspector — commit `cdcaf268`. Checklist AA documentado; build verde. ScoreBars ya tenía encoding (no duplicar).
- **Ids:** `VS-18`

### VS-14: Historial+Diff entre versiones (tab en Inspector)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase1.md`
- **Fecha:** 2026-08-18
- **Objetivo:** lista de versiones + diff (payload line-diff / metadata KV / vector) + revert explícito P6. Desbloqueada tras VS-CORE-07.
- **Resultado:** ✅ `historial-diff.ts` + `historial-tab.tsx` + tab HISTORIAL aditivo en Inspector — commit `5796b2f9`. Self-check 27 asserts PASS; build verde. Revert restaura payload+metadata+TTL (vector no — limitación vantaPut Fase 0, declarada en confirmación).
- **Ids:** `VS-14`

### VS-CORE-06: IQL en desktop (bridge vanta_query + autocompletado)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** exponer `VantaEmbedded.query` en bridge Tauri (`vanta_query` → `VantaQueryResult` con Read/Write/StaleContext) + autocompletado IQL core-side (`autocomplete_prefix` sobre `parse_statement`) → `vanta_iql_autocomplete`; wrapper `queryIql()` tipado. Desbloquea GRAFO-03.
- **Resultado:** ✅ commit `ebf9acc1`. verify full verde (cargo check, tests trait query roundtrip, npm build).
- **Ids:** `VS-CORE-06`

### VS-CORE-04: Exportar selección/query con filtro
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** `export_namespace(path, namespace, filter: Option<VantaMemoryFilter>)` aditivo (None = export completo actual) + `export_namespace_filtered` WASM + `exportNamespace(path, namespace, filter?)` TS + comando Tauri `vanta_export_namespace`. Desbloquea OP-02 batch export.
- **Resultado:** ✅ commit `a62088b7` + repair `7429f81a` (callers vantadb-python + clippy mcp_tests). 16/16 impl_export + integración; npm build TS+desktop verde.
- **Ids:** `VS-CORE-04`

### VS-CORE-05: Batch delete con filtro desde UI
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** exponer `delete_by_filter` en WASM → TS → bridge Tauri (`vanta_delete_by_filter(namespace, filter) -> u64`) con protección anti borrado total (filtro vacío rechazado). Desbloquea OP-02 batch delete.
- **Resultado:** ✅ commit `15172349` + repair `39a6369c` (re-exports filtros root + clippy tests + learnings). nextest 1948/1948 (2 skip).
- **Ids:** `VS-CORE-05`

### GRAFO-01: Bridge Tauri de grafos (bfs/dfs + degree)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** DTOs `VantaGraphNodeInfo`/`VantaGraphEdgeInfo`/`VantaGraphTraversalResult` + trait methods graph_bfs/graph_dfs/graph_degree (default Unsupported) + comandos `vanta_graph_bfs/dfs/degree` + WASM `graph_filtered_traversal`/`graph_degree` + TS wrappers. Base de datos del visor R3F.
- **Resultado:** ✅ commit `b5eaabad`. 1917 core + 53 desktop lib tests.
- **Ids:** `GRAFO-01`

### GRAFO-02: Visor R3F force-directed (toon+outline manga)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** canvas R3F con force-directed (d3-force en tick manual por useFrame fuera del render React — positionsRef, cero re-renders), toon shading + outline negro, tamaño por grado, expand incremental con reheat alpha, prefers-reduced-motion → radial estático (matchMedia; drei v10 no exporta useReducedMotion). Re-feedback del usuario: "Implementar force-directed (Recommended)".
- **Resultado:** ✅ commit `c23b1761`. r3f v9 + drei v10 (React 19.1.0 real en desktop, no 18). npm build verde (tsc 0 err), chunk lazy 263 kB gzip.
- **Ids:** `GRAFO-02`

### GRAFO-03: IQL console embebida (CodeMirror + autocompletado + highlight)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** consola IQL en lente GRAFO: CodeMirror (@uiw/react-codemirror@4.25.11, React 19 compat) + autocompletado (CompletionSource → `iqlAutocomplete`) + Ctrl+Enter → `queryIql()` → highlightIds → halo cian en GraphNode; historial localStorage `vanta.iql.history`.
- **Resultado:** ✅ commit `f62548a2`. Primer intento devolvió resultado vacío → RESUME misma sesión funcionó. npm build verde; tests parser autocomplete 7/7.
- **Ids:** `GRAFO-03`

### ESPACIO-01: Scatterplot UMAP en worker (regl-scatterplot + lasso)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** proyección 2D UMAP-js en web worker (fitAsync cancelable, seed mulberry32, NDC [-1,1], cap 100k) + regl-scatterplot (zoom/pan, hover tooltip, lasso SHIFT+drag, color por namespace) + surface "espacio" en WorkspaceShell. La proyección nunca en hilo principal (04 anti-patrón 5).
- **Resultado:** ✅ commit `0e772ba3`. Primer intento vacío con deps instaladas → RESUME. vitest worker 3/3; build verde.
- **Ids:** `ESPACIO-01`

### ESPACIO-02: Mapa como herramienta (selección lasso → batch ops + undo)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** selección lasso → barra de acciones: export JSONL client-side (`recordsToJsonl`+`downloadText`) y eliminar con undo batch (`undoStore.softDeleteBatch`, 1 entry + snapshot) + confirmación que muestra cantidad. Decisión: `deleteByFilter`/`exportNamespace` NO expresan "key ∈ {k1..kN}" (AND-filter) → client-side.
- **Resultado:** ✅ commit `889000ed`. undo.test 3/3, node:test 14/14; fix timeout vitest worker (`vi.setConfig({testTimeout: 60_000})` — test.setTimeout no existe en vitest 4).
- **Ids:** `ESPACIO-02`

### OP-01: Import CSV/JSON pegado (parser + preview + ingest lote)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** textarea pegado (CSV `key,payload,metadata_json` / JSONL / array) → parse + preview editable (filas ✓/✗) → confirmación con cantidad → `ingestBatch` → report. Máx 1000 registros (truncation + aviso), chunking 50. Errores de parse nunca silenciosos.
- **Resultado:** ✅ commit `6fc4df91`. parseImport.test 18/18 vitest; chunk lazy 10.9 kB gzip; botón ⤒ IMPORT en MEMORIAS + `key={gridKey}` remount.
- **Ids:** `OP-01`

### OP-02: Batch ops en grid (selección múltiple + export/eliminar con undo)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase2.md`
- **Fecha:** 2026-08-19
- **Objetivo:** checkbox de fila + select-all (página actual) → barra: Exportar (n) .jsonl client-side y Eliminar (n) con confirmación que muestra cantidad + undo snapshot (`softDeleteBatch`). Selección por `Set<string>` de `${ns}:${id}`. Retomado de sub-agente cancelado (abort sin task_id → digest del task file a agente fresco).
- **Resultado:** ✅ commit `a88cd1b0`. batchSelection.test 4/4 vitest; build tsc+vite verde; fix a11y (sort button solo envuelve columnas sortables).
- **Ids:** `OP-02`

### WEB-00: Abstraer `vanta.ts` de Tauri invoke (transporte pluggable)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** interface `VantaTransport { call<T>(cmd, args?) }` + `TauriBackend` (invoke) + `HttpBackend` stub + factory por entorno; TODAS las funciones de `vanta.ts` delegan a `transport.call` (refactor mecánico 1:1, 55 exports sin cambio de firma).
- **Resultado:** ✅ `desktop/src/transport.ts` (nuevo) + `desktop/src/vanta.ts` — commit `0cccd326`. npm build verde; node:test deep-link 8/8 (vitest da falso negativo pre-existente: el test usa node:test). Hallazgo: contrato del plan pedía vitest pero el runner real del repo es node:test.
- **Ids:** `WEB-00`

### WEB-01: REST: superficie de la consola (CRUD + search + list + IQL + health/metrics/audit)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** endpoints v2 que la UI consume: health, records CRUD (+batch, +versions, +delete_by_filter), list con cursor, search, autocomplete, audit. Patrón `/api/v2/query` existente, errores `{success:false,error}` + status HTTP.
- **Resultado:** ✅ `src/cli_server.rs` (11 rutas + helpers `run_db_op`/`vanta_error_status`) + `src/audit.rs` (AuditEvent Deserialize) + literales ServerState en tests — commit `c81bc23a`. 17/17 tests cli_server, smoke real con Invoke-RestMethod OK (search requiere `query_vector` no null + `text_query`).
- **Ids:** `WEB-01`

### WEB-02: REST: resto del SDK (export/import, graph, mantenimiento, threads, snapshots)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** export/import, graph bfs/dfs/degree/pagerank/centrality, maintenance purge/compact/flush/rebuild-index, threads CRUD, snapshots list/create. Mismo contrato de errores que WEB-01.
- **Resultado:** ✅ `src/cli_server.rs` (16 rutas nuevas, +801 líneas) — commit `c856b3bd`. 22/22 tests cli_server; smoke real 18 endpoints. Divergencias documentadas: graph/centrality→degree_centrality (GDS real), compact→compact_layout, thread_id viaja string en wire (u128 > u64::MAX), create_snapshot requiere fjall (no InMemory).
- **Ids:** `WEB-02`

### WEB-03: Servir estáticos `/dashboard` + SPA fallback + flag CLI
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** flag `vanta serve --dashboard-dir <path>` (real: `vanta-cli server --dashboard-dir`), ServeDir + fallback index.html solo para rutas sin extensión, 404 con hint si no configurado, fuera del middleware auth (D12).
- **Resultado:** ✅ `src/cli_server.rs` (`mount_dashboard`), `src/config.rs`, `src/cli.rs`, `src/bin/vanta-cli.rs`, `src/cli_handlers/server.rs`, Cargo.toml (tower-http fs + tower directa — axum 0.8 no re-exporta service_fn) — commit `62d63377` + `0da6d33c` (completions/Cargo.lock). 24/24 tests; smoke 5/5.
- **Ids:** `WEB-03`

### WEB-04: `HttpBackend` real (fetch REST) + factory por entorno
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** completar el stub de WEB-00: `HttpBackend.call` → fetch REST vía `vanta-http-map.ts` (mapeo cmd→método/URL), errores deserializados del shape del server, base `""` o `VITE_VANTA_API_BASE`.
- **Resultado:** ✅ `desktop/src/vanta-http-map.ts` + `desktop/src/vanta-http-map.test.ts` (14 tests) + `desktop/src/transport.ts` — commit `8b2bc14f`. 23 comandos reales (no ~55 como estimaba el plan): 15 mapeados a REST, 8 rechazos descriptivos (multi-conexión Tauri-only: connect/disconnect/list/set_active; metrics no-JSON; graph DTO incompatible), deep-link ya no-op.
- **Ids:** `WEB-04`

### WEB-05: Build web de la consola (Vite base `/dashboard/`, sin Tauri)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** `vite build --mode web` → `dist-web/` (base `/dashboard/`), guard runtime Tauri-only (useDeepLink skip listen, useConnectionState sintetiza conexión embebida), ocultar ConnectionPanel en web, build desktop intacto.
- **Resultado:** ✅ `desktop/vite.config.ts` (defineConfig por mode), `useDeepLink.ts`, `useConnectionState.ts`, `App.tsx`, `WorkspaceShell.tsx`, `.gitignore` (dist-web) — commit `42d2b26a`. Builds desktop+web verdes; preview 4173 OK. Hallazgo: imports `@tauri-apps/api/*` no rompen build (romperían runtime) → guards runtime, no import dinámico.
- **Ids:** `WEB-05`

### WEB-06: E2E Playwright contra server real + docs/ADR
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-studio-fase3.md`
- **Fecha:** 2026-08-19
- **Objetivo:** `desktop/scripts/selfcheck-web-e2e.ts` (Playwright): server real + dist-web → HOME con datos, grid, edición, borrado con undo, search híbrida. ADR D11/D12 + Backlog.
- **Resultado:** ✅ script E2E 11 checks exit 0 — commit `583dad9a` (incluye fix namespace default REST: `ListParams.namespace` default `"default"` — bug real cazado por el E2E: la consola web lista/busca sin namespace y REST 400eaba). devDep playwright@1.61.1. ADR-026 en `docs/architecture/`. Lead arregló `tests/cli_tests.rs` (firma cmd_server 9 args — roto por WEB-03, no detectado por verify `--lib`).
- **Ids:** `WEB-06`
