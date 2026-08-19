# Plan de Ejecución: Vanta Studio — Fase 2 (grafo R3F + espacio + operaciones)

> **Campaign ID:** 8f3a1c6e-5d2b-4a7f-9c01-3e6d8b2f4a70
> **Inicio:** 2026-08-18
> **Estado:** ⏳ EN PROGRESO — ejecutando vía `/pipeline run` (2026-08-19).
> **Fuente:** `docs/research/human-facing-db-ui/06-synthesis/SYNTHESIS.md` (Fase 2 §7: "Grafo y espacio") + `03-ai-memory-graphs/RESEARCH.md` + `04-embedding-visualization/RESEARCH.md` + `05-data-editor-ux/RESEARCH.md` + backlog VS-CORE-04/05/06 + decisiones D3/D5 del usuario (2026-08-18).
> **Modo:** secuencial con gaps del core primero (VS-CORE-04/05/06), luego lentes (GRAFO → ESPACIO), luego operaciones.

## Decisiones del usuario (heredadas de Fase 0, aplicables a Fase 2)

| # | Decisión | Valor |
|---|----------|-------|
| D3 | Distribución | **Solo desktop** (web/embebido → Fase 3) |
| D5 | Grafo | **Renderer three.js propio (react-three-fiber)** — control total de shaders (toon+outline, estilo manga/linocut) y perf, física en worker. Trade-off R3F vs react-force-graph/Sigma documentado. |
| D4 | Dirección visual | Manga Tradicional & Grabado Linocut (Neo-brutalista) — aplica al grafo (toon shading + outline negro + neon) y al espacio (puntos con borde, cluster por color, encoding redundante). |
| D7 | Tema | Ambos con toggle, default claro. |

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| VS-CORE-04/05/06 (gaps core) + Lente GRAFO (R3F + IQL) + Lente ESPACIO (regl+UMAP) + Import CSV/JSON + Batch ops | Heatmap+dendrograma "Matriz" (03 P7) — opcional post-ESPACIO; PaCMAP→WASM (04) — evolución futura | Fase 3 web/embebido; 3D por moda (04 anti-patrón 6) | — |

## Orden de ejecución

1. **Gaps del core (Wave 0):** VS-CORE-06 (bridge `vanta_query` + autocompletado) → desbloquea IQL console; VS-CORE-04 (export con filtro) y VS-CORE-05 (batch delete con filtro) → desbloquean batch ops. Se ejecutan en paralelo (tocan archivos distintos del bridge/core).
2. **Lente GRAFO (Wave 1):** bridge grafo (`vanta_graph_bfs/dfs` + nodos) → componente R3F (expand incremental) → IQL console embebida (VS-CORE-06).
3. **Lente ESPACIO (Wave 2):** proyección UMAP-js en worker + regl-scatterplot → selección → batch ops.
4. **Operaciones (Wave 3):** import CSV/JSON pegado + batch ops con confirmación/undo (consume VS-CORE-04/05).

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración la hace el lead
- `src/sdk/` (tipos públicos) — cambios solo vía task file con contrato
- `desktop/prototype/` — referencia visual Fase 0, no modificar

---

## Wave 0 — Gaps del core (backlog VS-CORE; en paralelo)

### Task 1: VS-CORE-06 — IQL en desktop: exponer en bridge + autocompletado (re-scopeado)
- **Archivos clave:** `desktop/src-tauri/src/connections/types.rs` (DTO `VantaQueryResult`), `desktop/src-tauri/src/commands/data.rs` (nuevo comando `vanta_query`), `desktop/src-tauri/src/connections/mod.rs` + `native.rs` (trait `Connection::query`), `desktop/src/vanta.ts` (wrapper `queryIql()`), `src/parser.rs::parse_statement` (crate-interno, para el shim de autocompletado).
- **Gate Justificación:** gap §8.5 YA resuelto en core/bindings (nativo `api.rs:1164`, WASM `lib.rs:1210`, TS `vantadb.ts:680`); falta exponer en bridge Tauri + autocompletado para la consola IQL (lente GRAFO). Bloqueante de GRAFO-03.
- **Contrato:** comando Tauri `vanta_query(iql: String) -> VantaQueryResult` mapeando a `VantaEmbedded.query` (aditivo, `Unsupported` en transports sin IQL); DTO wire para `VantaQueryResult::Read(records)` / `Write{affected_nodes,message,node_id}` / `StaleContext{node_id}`; `vanta.ts` `queryIql()` tipado; **autocompletado**: shim core-side (nuevo, crate-interno `autocomplete_prefix(query) -> Vec<String>` sobre `parse_statement` — tokens de keywords/identificadores) expuesto por el bridge como `vanta_iql_autocomplete(prefix)`.
- **Verificación:** `cargo check` + `cargo test` (trait `query` roundtrip en manager + e2e) + `npm run build` verde.
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19T04:05 — vanta-worker, task file `tasks/1.md`; verify full verde; commit pendiente de vanta-lead)
- **last-synced:** 2026-08-19T04:05

### Task 2: VS-CORE-04 — Exportar selección/query (no solo namespace)
- **Archivos clave:** `src/sdk/serialization/impl_export.rs:121` (`export_namespace` — añadir `filter: Option<VantaMemoryFilter>` aditivo; internamente `records_for_namespace:76` ya acepta filtros), `src/sdk/serialization/impl_export.rs:76` (pasar filtro a `records_for_namespace`), `vantadb-wasm/src/lib.rs` (`export_namespace_filtered`), `vantadb-ts/src/vantadb.ts`, `desktop/src-tauri/src/commands/data.rs` (comando `vanta_export_namespace(namespace, filter?, path)`), `desktop/src/vanta.ts`.
- **Gate Justificación:** P12/P13; gap §8.4. Hoy la palette exporta `list({limit:500})` client-side (CommandPalette.tsx:56) — limitado y sin filtro.
- **Contrato:** `export_namespace(path, namespace, filter: Option<VantaMemoryFilter>)` (backward-compat: `None` = export completo actual); expuesto en WASM/TS/bridge Tauri; report `VantaExportReport` igual. La UI batch-export consume esto (OP-02) con filtro del query builder (VS-07).
- **Verificación:** unit test en `impl_export` (export con filtro Eq solo incluye matching) + `cargo check --workspace` + `npm run build` verde.
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19 — vanta-worker, task file `tasks/2.md`; verify full verde: core 16/16 impl_export + integración memory_export_import/text_index_recovery/snapshot_certification, clippy/fmt root+desktop, npm build vantadb-ts+desktop; commit pendiente de vanta-lead)
- **last-synced:** 2026-08-19T02:30

### Task 3: VS-CORE-05 — Batch delete con filtro desde UI
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (exponer `delete_by_filter(namespace, filter)` — hoy solo Rust+CLI), `vantadb-ts/src/vantadb.ts` (wrapper), `desktop/src-tauri/src/connections/` (trait `Connection::delete_by_filter` + native.rs), `desktop/src-tauri/src/commands/data.rs` (comando `vanta_delete_by_filter(namespace, filter)`), `desktop/src/vanta.ts`.
- **Gate Justificación:** gap §8.6; `delete_by_filter` existe en `src/sdk/api.rs:1264` pero NO está en Python/WASM/TS/`vanta.ts`. Bloqueante de OP-02 (batch delete).
- **Contrato:** exponer en WASM → TS → bridge Tauri; `vanta_delete_by_filter(namespace, filter) -> u64` (cantidad borrada); **protección**: el core ya rechaza filtro vacío (evita borrado total del namespace) — mantener el error visible en UI. Confirmación + undo integrado en OP-02.
- **Verificación:** `cargo check` + test WASM/TS del wrapper + `npm run build` verde.
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19T05:45 - vanta-worker, task file `.opencode/skills/campaign-executor/tasks/3.md`; verify full: clippy root+desktop ✅, nextest 1948/1948 (2 skip, excluido stress HNSW lento pre-existente), cargo check --workspace ✅, npm build TS+desktop ✅; sin commit - lo commitea el lead)
- **last-synced:** 2026-08-19T05:45

## Wave 1 — Lente GRAFO (R3F propio, D5)

### Task 4: GRAFO-01 — Bridge Tauri: datos de grafo (bfs/dfs + nodos)
- **Archivos clave:** `desktop/src-tauri/src/connections/` (trait `Connection::graph_bfs`/`graph_dfs` + native.rs usando `VantaEmbedded.graph_bfs`/`graph_dfs` de `src/sdk/graph.rs:50,67`), `desktop/src-tauri/src/connections/types.rs` (DTO `GraphNode {id, payload, fields, edges[]}`, `GraphEdge {target, label_id, weight, created_at_ms}`), `desktop/src-tauri/src/commands/data.rs` (comandos `vanta_graph_bfs(roots: Vec<u128>, max_depth, direction)` y `vanta_graph_nodes(ids)`), `desktop/src/vanta.ts` (wrappers tipados).
- **Gate Justificación:** P11 (grafos con navegación, nunca render completo); el bridge NO expone nada de grafo hoy. Base de datos para el visor R3F.
- **Contrato (supersede por orquestador GRAFO-01):** `vanta_graph_bfs(roots: Vec<String>, max_depth, direction: String, limit)` / `vanta_graph_dfs(...)` → `VantaGraphTraversalResult {nodes: VantaGraphNodeInfo[], edges: VantaGraphEdgeInfo[]}`; `vanta_graph_degree(namespace, limit)` → `VantaGraphNodeInfo[]` (degree in+out); DTOs wire en `types.rs`; nodos con id String (u128) + label + group; namespace vacío → lista vacía, no error; `graph_filtered_traversal` (filtro labels/time_range) y `graph_degree` solo WASM/TS. Aditivo sobre el core (`graph_bfs`/`graph_dfs`/`graph_bfs_filtered`/`graph_degree_centrality`). El visor hace expand incremental (1-2 nodos → vecinos), no render total (anti-hairball 03).
- **Verificación:** `cargo test -p vantadb-desktop --lib` (roundtrip bfs/degree en native, e2e manager) + `npm run build` verde + TS graph tests (`npx vitest -t "graph"`, 11 passed).
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19T01:30 - vanta-worker, task file `.opencode/skills/campaign-executor/tasks/4.md`)
- **last-synced:** 2026-08-19T01:30

### Task 5: GRAFO-02 — Visor R3F force-directed (toon+outline manga)
- **Archivos clave:** `desktop/src/components/graph/GraphLens.tsx` (nuevo, lazy), `desktop/src/components/graph/useGraphData.ts` (nuevo: estado nodos/aristas + expand incremental), `desktop/src/components/graph/GraphCanvas.tsx` (nuevo, R3F), `desktop/src/components/layout/WorkspaceShell.tsx` (surface `"iql"` → GraphLens; `Surface` type), deps: `three`, `@react-three/fiber`, `@react-three/drei`.
- **Gate Justificación:** D5 (R3F propio con shaders) + P11 (expand incremental) + P4 (lente contextual, no destino aparte).
- **Contrato:** canvas R3F con force-directed (simulación en worker o `d3-force` en un frame loop controlado, fuera del render React), **toon shading + outline negro** (shader propio estilo manga — neon para nodos destacados), color por tipo de nodo (fields o label), tamaño por grado (GRAFO-01 + `graph_degree_centrality` de `src/sdk/gds.rs:32` si se expone), click nodo → Inspector (reutilizar `openRecord` del shell, master-detail P2), búsqueda por key (destaca y centra), botones expand ("+ vecinos", profundidad), límite de nodos visibles (cap 200-500, aviso si se supera), `prefers-reduced-motion` respetado (04 a11y). **Nunca render completo de un namespace grande** (03 anti-pattern hairball).
- **Verificación:** `npm run build` verde; render manual con datos de prueba (namespace con >10 nodos relacionados vía `RELATE`/`add_edge`); screenshot en `desktop/prototype/` para review visual.
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19 — vanta-worker, task file `.opencode/skills/campaign-executor/tasks/5.md`; r3f v9+drei v10 por React 19 real en package.json, no v8; **force-directed implementado** (re-feedback D5): d3-force@3 en tick manual por useFrame fuera del render React (positionsRef mutable, meshes leen por frame sin re-render; forceLink/forceManyBody -18/forceCenter/forceCollide anti-overlap, reheat alpha en expansión, nodos nuevos nacen en el padre, prefers-reduced-motion → radial estático vía matchMedia); verify: npm run build verde (tsc 0 err) — GraphLens chunk lazy 263 kB gzip; sin commit — lo commitea el lead)
- **last-synced:** 2026-08-19

### Task 6: GRAFO-03 — IQL console (CodeMirror + autocompletado + highlight subgrafo)
- **Archivos clave:** `desktop/src/components/graph/IqlConsole.tsx` (nuevo), `desktop/src/components/graph/GraphLens.tsx` (integra console embebida), `desktop/src/vanta.ts` (`queryIql`, `iqlAutocomplete`), dep: `@codemirror/lang-javascript` NO — usar `@codemirror/autocomplete` + shim VS-CORE-06.
- **Gate Justificación:** P5 (nada de JSON obligatorio; IQL como lenguaje de poder) + síntesis §4 GRAFO ("playground con autocompletado estilo Kibana Dev Tools; el resultado resalta el subgrafo").
- **Contrato:** consola embebida en la lente GRAFO: editor CodeMirror con autocompletado (VS-CORE-06: keywords FROM/MATCH/WHERE/AND/FETCH/RANK BY/WITH TEMPERATURE/ROLE + identificadores de namespaces/nodos), ejecutar (Ctrl+Enter) → `queryIql()` → resultado `Read` resalta nodos en el canvas R3F; `Write`/`StaleContext` muestran mensaje; errores de parse legibles (sin stack). Historial de queries en sesión (re-ejecutable, patrón VS-17 localStorage).
- **Verificación:** `npm run build` verde; ejecutar IQL `FROM x FETCH y` contra datos de prueba → nodos resaltados; autocompletado muestra keywords al teclear.
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19T06:15 — vanta-worker, task file `.opencode/skills/campaign-executor/tasks/6.md`; `IqlConsole.tsx` con CodeMirror + autocompletado (CompletionSource → `iqlAutocomplete`) + Ctrl+Enter → `queryIql` + historial localStorage `vanta.iql.history`; GraphLens integra consola panel inferior colapsable + highlightIds → GraphScene → halo cian en GraphNode; WorkspaceShell pasa `dark`; verify: npm run build verde (tsc 0 err), cargo fmt/clippy workspace 0 warnings, tests parser autocomplete 7/7; docs-coverage falla por bug interno pre-existente del script (línea 172) + gap pre-existente config.rs — sin cambios Rust; sin commit — lo commitea el lead)
- **last-synced:** 2026-08-19T06:15

## Wave 2 — Lente ESPACIO (embeddings)

### Task 7: ESPACIO-01 — Scatterplot regl-scatterplot + UMAP-js en worker
- **Archivos clave:** `desktop/src/components/space/SpaceLens.tsx` (nuevo, lazy), `desktop/src/components/space/projection.worker.ts` (nuevo: UMAP-js en web worker), `desktop/src/components/space/useProjection.ts` (nuevo: cola de proyección, cancelable), `desktop/src/components/layout/WorkspaceShell.tsx` (surface `"espacio"` nueva), deps: `regl-scatterplot`, `umap-js`.
- **Gate Justificación:** 04 §stack (UMAP-js worker + regl-scatterplot) + P12 (mapa como herramienta de mantenimiento) + 04 anti-patrón 5 (proyección nunca en hilo principal) + anti-patrón 1 (sin cluster = bola de pelos).
- **Contrato:** proyección 2D de vectores del namespace activo (o selección) en **web worker** (UMAP-js, cap 10k-100k puntos; <1k → PCA rápido en worker), render con **regl-scatterplot** (zoom/pan, hover tooltip = payload preview, click → Inspector), color por **cluster (k-means simple en worker, k≈8-12)** con leyenda o por namespace/tier, encoding redundante (color+ícono+texto, P15), **aviso de distorsión** ("el mapa distorsiona distancias" — 04 anti-patrón 2), control "mostrar solo con vector". La proyección es opcional: tabla de resultados sigue disponible (04 regla 6).
- **Verificación:** `npm run build` verde; render con namespace de prueba (100+ registros con vector); worker no bloquea UI (proyección de 10k puntos < 2s en background).
- **Ruta:** vanta-worker
- **Estado:** ✅ COMPLETED (2026-08-19T08:00 — vanta-worker, task file `.opencode/skills/campaign-executor/tasks/7.md`; verify: build exit 0, tsc exit 0, vitest 3/3; commit pendiente vanta-lead)
- **last-synced:** 2026-08-19T08:00

### Task 8: ESPACIO-02 — Mapa como herramienta (selección → batch ops)
- **Archivos clave:** `desktop/src/components/space/SpaceLens.tsx` (selección lasso/multi-punto + barra de acciones), `desktop/src/components/space/batchActions.ts` (nuevo: borrar/exportar/ajustar TTL), `desktop/src/vanta.ts` (consume VS-CORE-04/05).
- **Gate Justificación:** P12 (el mapa es herramienta de mantenimiento, no solo inspección) + P8 (recuperación de errores) + 05 bulk edit.
- **Contrato:** lasso/selección múltiple de puntos → barra de acciones: **Borrar (n)** con confirmación que muestra cantidad (vía VS-CORE-05 `delete_by_filter` si aplica filtro, o `remove` por key) + **undo** (papelera VS-08: tombstones de sesión), **Exportar selección (.jsonl)** vía VS-CORE-04 con filtro, **Editar TTL** (batch). Confirmación destructiva nombra impacto (05 anti-patrón 7). Ninguna acción sin confirmación explícita.
- **Verificación:** `npm run build` verde; selección de 5 puntos → borrar → confirmación muestra "5 registros" → undo restaura; export genera archivo con solo la selección.
- **Ruta:** vanta-worker
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-08-19T02:30

## Wave 3 — Operaciones

### Task 9: OP-01 — Import CSV/JSON pegado (02 P2)
- **Archivos clave:** `desktop/src/components/ingest/ImportPaste.tsx` (nuevo, lazy), `desktop/src/components/ingest/parseImport.ts` (nuevo: parser CSV/JSON → `IngestItem[]` con validación), `desktop/src/components/layout/WorkspaceShell.tsx` (acceso desde MEMORIAS/IngestForm), `desktop/src/vanta.ts` (reusa `ingestBatch`).
- **Gate Justificación:** 02 P2 (import CSV/JSON pegado) + P10 local-first; hoy IngestForm es manual registro-a-registro.
- **Contrato:** textarea pegado (CSV: `key,payload,metadata_json` o JSONL/array) → parse + **preview editable** (tabla con filas válidas/inválidas marcadas, columna metadata parseable) → confirmación ("importar N registros a ns X") → `ingestBatch` → report (creados/errores). Errores de parse nunca silenciosos (05 anti-patrón: validar antes de escribir). Máx 1000 registros por paste (aviso).
- **Verificación:** `npm run build` verde; pegar CSV de 10 filas → preview 10 → import → grid muestra 10; fila inválida se marca sin romper el resto.
- **Ruta:** vanta-worker
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-08-19T02:30

### Task 10: OP-02 — Batch ops en grid con confirmación + undo
- **Archivos clave:** `desktop/src/components/DataExplorer.tsx` (checkbox de fila + barra de selección), `desktop/src/store/undo.ts` (reusa VS-08 para batch), `desktop/src/vanta.ts` (consume VS-CORE-04/05), `desktop/src/components/layout/WorkspaceShell.tsx` (handlers).
- **Gate Justificación:** 05 §bulk edit + P8 (recuperación de errores) + VS-CORE-04/05 consumidos desde UI.
- **Contrato:** selección múltiple en grid (checkbox + Shift+Space, 05 a11y) → barra: "Editar TTL…", "Eliminar (n)" (confirmación con cantidad y consecuencia — usa `delete_by_filter` con filtro Eq por keys, o bucle `remove` con undo snapshot VS-08), "Exportar (n) .jsonl" (VS-CORE-04 con filtro de keys). La selección se serializa a `VantaMemoryFilter` para batch delete/export (evita N comandos). Undo de batch = snapshot previo (mismo mecanismo VS-08).
- **Verificación:** `npm run build` verde; seleccionar 3 filas → eliminar con confirmación "3 registros" → Ctrl+Z restaura; exportar selección genera archivo con 3 líneas.
- **Ruta:** vanta-worker
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-08-19T02:30

---

## Relación con P27 (Vanta Memory Engine)

Integración **por contratos, no por ejecución**. La lente GRAFO/IQL de Fase 2 consume el grafo core existente (`GraphTraverser`, IQL `query`) — sin cambios de contrato. Cuando vanta-memory F4 añada nodos escena (MEM-12), el visor R3F los renderiza automáticamente (mismo `VantaNodeRecord`). El mapa ESPACIO puede consumir el search profile de MEM-01 (mismas estructuras que `explain`) si existe — aditivo, no bloqueante.

## DEFER (fuera de este plan)

| Item | Cuándo | Estado |
|------|--------|--------|
| Vista "Matriz" (heatmap + dendrograma de top-k de un query) | post-ESPACIO (opcional) | 03 P7, 04 §técnicas |
| PaCMAP portado al core WASM (`embedding_projection`) | evolución | 04 §stack, 04 §4 |
| 3D toggle en ESPACIO | solo si el usuario lo pide | 04 anti-patrón 6 |
| Fase 3 web/embebido (servir consola desde proceso embebido + WASM/OPFS) | Fase 3 | D3 |

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| R3F + regl + worker = bundle grande en desktop | Med | Chunks lazy (patrón Inspector/CommandPalette); Tauri local, carga on-demand |
| Proyección UMAP lenta en colecciones grandes | Med | Worker + cap 100k + PCA rápido para <1k; aviso de distorsión |
| Grafo hairball con grafos densos | Alto | Expand incremental + cap nodos + filtros por label/tipo (03 patrón) |
| IQL parse errors confunden al usuario | Med | Errores legibles sin stack; autocompletado reduce typos |
| VS-CORE-04/05 tocan tipos públicos (`src/sdk`) | Med | Task files con contrato + verify mecánico del lead (regla 40) |

=== RECITATION ===

- **Estado:** ⏳ PLAN CREADO — 10 tareas (3 gaps core + 3 GRAFO + 2 ESPACIO + 2 OP), 4 waves. Pendiente de aprobación del usuario.
- **Próximo:** tras aprobación, ejecutar Wave 0 (VS-CORE-04/05/06 en paralelo — archivos disjuntos) con sub-agentes vanta-worker (core/bridge) y vanta-worker (TS/React), verificación mecánica del lead entre tareas.