# Plan de Ejecución: Vanta Studio — Fase 4 (WASM/OPFS + cierre de deuda + diferenciadores)

> **Campaign ID:** e7b31c4a-8d2f-4a7e-9c1b-6f5a3e8d2c40
> **Inicio:** 2026-08-19
> **Estado:** 🚧 EN EJECUCIÓN — W0 (DOC) y W1 (REST-01..06) ✅ completas (commits f7e39005, 0bf9609e, 9ec506d8, 7b3cfea2, b81a8bf9, 1b71d300, 08109a55, 8ad119eb); W2 (WASM-01..04) en curso. 18 tareas: 10/18.
> **Fuente:** auditoría multi-agente 2026-08-19 (4 sub-agentes read-only: research original `docs/research/human-facing-db-ui/` + Fases 0/1 + Fases 2/3 + cross-check git/registro) → gaps consolidados en el digest del lead; decisiones del usuario 2026-08-19 (ver Decisiones).
> **Modo:** secuencial con waves paralelas por archivos disjuntos (patrón Fase 3). FAIL_MODE=parallel.

## Decisiones del usuario (2026-08-19)

| # | Decisión | Valor |
|---|----------|-------|
| D13 | Alcance Fase 4 | **Cierre de deuda + WASM/OPFS + 3 diferenciadores del research** — F4.0 reconciliación documental (registro canónico) · F4.1 cierre deuda REST (rate limiter, `/api/v2/metrics` JSON, graph_v2, cursor, namespace_stats, IQL server) · F4.2 WASM/OPFS backbone (D10) · F4.3 slider de pesos híbridos + superficie Índices/salud + consolidación asistida |
| D14 | Reconciliación documental | **Sí, como Fase 4.0 dentro del plan** — Backlog P26 (18/19 filas obsoletas), task files stale, hash `9c27f5e9`→`4c26b285` (plan F1), Fase 0 ausente de progreso, WEB-001.md erróneo. Bloqueante para planear correctamente; barato (~1 wave de docs). |
| D15 | Auth dashboard | **Mantener sin auth (local-first)** — D12 sigue vigente: bind 127.0.0.1, sin auth para consola local/web. MEM-05 (auth 3 capas) sigue siendo workstream aparte si algún día se expone fuera de loopback. |

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 4 reconciliación documental + 6 cierre deuda REST + 4 WASM/OPFS + 3 diferenciadores research + 1 E2E/cierre | auth fuerte 3 capas (MEM-05) · PaCMAP→WASM (Fase 5) · Matriz heatmap+dendrograma (Fase 5) · temporalidad del grafo valid_at/invalid_at (requiere core, Fase 5+) · 3D toggle en ESPACIO (solo si el usuario pide) · SSE streaming | `web/` Next.js marketing (no tocar) · MCP stdio en web (D10) · vs. tareas de diseño/UI ajenas (web-design-audit es otra campaña) | test core roto `test_consolidate_node_with_binary_vector` solo si el root cause excede el scope (→ Fase 5) |

## Orden de ejecución

1. **Wave 0 — Reconciliación documental (registro canónico):** Backlog P26 + task files stale + Fase 0 en progreso + cabeceras/hashes. Sin esto, el resto del plan se construye sobre estado falso (ver dictamen cross-check: "planear sobre estado falso").
2. **Wave 1 — Cierre deuda REST (server Rust, `src/cli_server.rs` + bridge desktop):** rate limiter calibrado, `/api/v2/metrics` JSON, graph_v2 (DTO desktop), cursor real en server, `namespace_stats` en bridge, IQL vía ServerConnection. Cierra los 8 rechazos de `vanta-http-map` y los gaps VS-CORE-01/02.
3. **Wave 2 — WASM/OPFS backbone (D10):** transporte WASM real (persistencia OPFS/IDB), consola 100% browser sin server, drag&drop `.vdbdump`/JSONL.
4. **Wave 3 — Diferenciadores del research (los 3 "prometidos nunca tocados"):** slider de pesos híbridos, superficie Índices/salud, consolidación asistida.
5. **Wave 4 — Verificación E2E + cierre:** E2E ampliado (dashboard server + consola WASM standalone) + ADR-027 (D13/D14/D15) + Backlog/CHANGELOG + archivo del plan.

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración/estados la hace el lead (tarea DOC-01 = lead, no sub-agente)
- `web/` (Next.js marketing site) — NO es la consola; no tocar
- `docs/plans/2026-08-19-web-design-audit.md` + `SKILLS-MANIFEST.md` — workstreams ajenos, no commitear
- `src/sdk/` (tipos públicos) — cambios solo vía task file con contrato
- `desktop/src-tauri/` — tocar solo si una tarea lo exige explícitamente (REST-05 namespace_stats SÍ lo exige)
- Otros workstreams (web/remotion, completions, assets, README.md) — nunca tocar/commitear
- Plan file: lead es único dueño (regla de Fase 1/3 — sub-agentes NO escriben recitation en el plan)

---

## Wave 0 — Reconciliación documental (registro canónico)

> **Rutas:** `docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/*.md`, `docs/progreso/README.md`, `docs/avance/activo/*.md`, `docs/plans/archive/2026-08-18-vanta-studio-fase{0,1}.md`. Sin código. Ejecutada por el **lead** (Backlog/plan/progreso son del lead) — no se delega a sub-agente.

### Task 1: DOC-01 — Reconciliar Backlog P26 (18/19 filas obsoletas) + línea resumen
- **Archivos clave:** `docs/Backlog.md` (L53 línea resumen "🔴 Alta (Fase 2: grafo R3F)" → actualizar; L660-678 tabla P26: filas VS-01..11 + VS-CORE-01..07)
- **Gate Justificación:** dictamen cross-check 🔴 Bloqueante #1 — el registro canónico de backlog está en estado pre-ejecución pese a F0 (14/19) + F1 (9/9) + F2 (10/10) + F3 (7/7) completas. Cualquier decisión sobre P26 leída del Backlog hoy es falsa.
- **Contrato:** actualizar la tabla P26 con estados reales por fase (✅ Hecho + commit o fase), marcar VS-00..18 / VS-CORE-01..07 / GRAFO-01..03 / ESPACIO-01..02 / OP-01..02 / WEB-00..06 según `docs/progreso/README.md`; corregir línea resumen L53 y header de P26 (nota de Fase 3 ya existe — verificar coherencia); NO eliminar filas (historia), solo estados.
- **Verificación:** `rg -n "Pendiente" docs/Backlog.md | rg "VS-|WEB-|GRAFO-|ESPACIO-|OP-"` → solo las verdaderamente pendientes (VS-* sin implementar: revisar VS-01 Tailwind); tabla coherente con progreso.
- **Estado:** ⏳ PENDING

### Task 2: DOC-02 — Cerrar/actualizar task files stale (VS-02, VS-CORE-07, tasks F2 3/5/9/10, WEB-02, WEB-001)
- **Archivos clave:** `.opencode/skills/campaign-executor/tasks/{VS-02,VS-CORE-07,VS-CORE-04,VS-CORE-05,GRAFO-02,OP-01,OP-02,WEB-02,WEB-001}.md`
- **Gate Justificación:** dictamen cross-check — estados ⏳ IN PROGRESS / ⬜ PENDING cuando planes y progreso dicen ✅; `WEB-001.md` es artefacto legacy erróneo (WASM playground, campaign 2026-08-04, ya ✅ en BACKLOG_HISTORY:94) que colisiona con WEB-01 y WEB-00..06.
- **Contrato:** actualizar estados a ✅ COMPLETO con commit real (ver progreso/README); WEB-001.md → mover a un prefijo no colisionante (ej. renombrar `WEB-001` → `WASM-PLAYGROUND-001`) o marcar como legacy/cerrado con nota al BACKLOG_HISTORY; tasks 3/5/9/10 de F2 y WEB-02 de F3 → completed. NO borrar archivos (historia), solo estado + nota.
- **Verificación:** `rg -l "IN PROGRESS|PENDING" .opencode/skills/campaign-executor/tasks/` → solo tareas verdaderamente activas; `rg "WEB-001" docs/Backlog.md` coherente.
- **Estado:** ⏳ PENDING

### Task 3: DOC-03 — Fase 0 en registro canónico + plan F0 archivado + hash F1 corregido
- **Archivos clave:** `docs/progreso/README.md` (sección Detalle), `docs/avance/activo/desktop.md`, `docs/plans/2026-08-18-vanta-studio-fase0.md` → mover a `docs/plans/archive/`, `docs/plans/archive/2026-08-18-vanta-studio-fase1.md` (L5)
- **Gate Justificación:** dictamen cross-check — Fase 0 ausente del registro canónico (solo VS-02 tiene entrada en progreso, 14 tareas HECHO sin entradas); plan F0 nunca archivado; plan F1 L5 cita commit inexistente `9c27f5e9` (cierre real `4c26b285`).
- **Contrato:** agregar entradas VS-00..18 + VS-CORE-01..03 de Fase 0 a progreso/README (formato existente, con commits reales — verificar con `git log`); espejar en desktop.md; mover plan F0 a archive/ + entrada "Planes archivados" + retrospectiva (patrón F1/F2/F3); corregir hash en plan F1 (o anotar errata).
- **Verificación:** `git cat-file -t 4c26b285` OK; `rg -c "VS-0" docs/progreso/README.md` sube; plan F0 en archive/.
- **Estado:** ⏳ PENDING

### Task 4: DOC-04 — Cabeceras/estados stale restantes
- **Archivos clave:** `docs/progreso/README.md` (cabecera L5/L11/L21 fecha ~2026-08-02/~95% vs contenido 2026-08-19; L630 comentario muerto "movido a ARCHIVO_HISTORICO.md"), `docs/plans/archive/2026-08-18-vanta-studio-fase3.md` (L102 WEB-06 ⏳ PENDING vs header 7/7; L5 "7 commits" lista 8 hashes), `docs/avance/activo/desktop.md` (header "Fase 1" agrupa Fase 2)
- **Gate Justificación:** dictamen cross-check media — cabeceras desactualizadas contradicen contenido y confunden cualquier lectura futura.
- **Contrato:** corregir fecha/estado en cabeceras de progreso/README; marcar WEB-06 ✅ (commit 583dad9a) en plan F3 archivado + conteo de commits correcto; corregir header desktop.md; eliminar comentario muerto L630.
- **Verificación:** lectura visual: cabeceras coherentes con contenido; `rg "2026-08-02" docs/progreso/README.md` sin hits en cabecera.
- **Estado:** ⏳ PENDING

---

## Wave 1 — Cierre deuda REST (server + bridge)

> **Rutas:** `src/cli_server.rs`, `src/sdk/api.rs`, `desktop/src-tauri/src/connections/server_client.rs`, `desktop/src/vanta-http-map.ts` (+test), `desktop/src/vanta.ts`. Sub-agente: vanta-worker (Rust core/server) — verify mecánico del lead obligatorio (`cargo check --features server --tests`).

### Task 5: REST-01 — Calibrar rate limiter para ráfagas UI (429 en consola)
- **Archivos clave:** `src/cli_server.rs` (L198-214 governor rpm=100 burst=10)
- **Gate Justificación:** auditoría F2/F3 Alta #1 — una ráfaga UI normal (~12 reqs: grid + inspector + sidebar) recibe 429; la consola web degrada. E2E usa `VANTADB_RATE_LIMIT_RPM=0` (escape, no solución).
- **Contrato:** default revisado para ráfagas UI locales (ej. rpm 600 / burst 60, o burst≥rpm completo en loopback) con env var documentada; respuesta 429 con `Retry-After` y shape `{success:false,error}`; NO relajar si `require_auth` activo (sigue fail-closed AUD-021); tests de ráfaga (12+ reqs consecutivas sin 429 en loopback).
- **Verificación:** `cargo test --features server` verde; script de humo: 20 GETs secuenciales → 0×429; E2E sin `VANTADB_RATE_LIMIT_RPM=0` pasa.
- **Estado:** ✅ COMPLETO (commit b81a8bf9)

### Task 6: REST-02 — `/api/v2/metrics` JSON (métricas del motor en shape JSON)
- **Archivos clave:** `src/cli_server.rs` (L195 hoy solo `/metrics` Prometheus), `src/sdk/` (fuente de métricas: hnsw_nodes_count, dims, LSM/WAL, collection stats)
- **Gate Justificación:** auditoría F2/F3 Alta #1 — 1 de 8 rechazos de `vanta-http-map` es `vanta_metrics` (sin endpoint JSON); además FEAT-02 (superficie Índices/salud) lo consume.
- **Contrato:** `GET /api/v2/metrics` → JSON con métricas del motor (mismo shape que `namespace_stats`/`VantaMetrics` existente — verificar fuente real); CORS igual que resto; documentado en CONFIGURATION.md/API.md.
- **Verificación:** `cargo test --features server` verde; curl `/api/v2/metrics` → 200 JSON con campos esperados; `vanta_metrics` deja de estar en la lista de rechazos vanta-http-map.
- **Estado:** ✅ COMPLETO (commit b81a8bf9)

### Task 7: REST-03 — Endpoint `graph_v2` con DTO desktop (cierra 3 rechazos vanta-http-map)
- **Archivos clave:** `src/cli_server.rs`, `desktop/src/vanta-http-map.ts` (+test), `src/sdk/gds.rs`/graph DTOs
- **Gate Justificación:** auditoría F2/F3 Alta #1 — graph_bfs/dfs/degree rechazados en web por DTO incompatible (u128 > u64::MAX en wire JSON). La lente GRAFO no existe en web por esto.
- **Contrato:** endpoint(s) `/api/v2/graph/v2/*` (o ajuste del existente) que serialice DTO con u128 seguro (string en wire, patrón thread_id de Fase 3); mapeo `vanta_graph_bfs/dfs/degree` en vanta-http-map → dejan de ser rechazos; test de roundtrip con IDs u128 grandes.
- **Verificación:** `cargo test --features server` verde; node:test vanta-http-map verde (8 rechazos → 5); curl con id > u64::MAX.
- **Estado:** ✅ COMPLETO (commit b81a8bf9)

### Task 8: REST-04 — Cursor real en server (paginación list/search — gap VS-CORE-01)
- **Archivos clave:** `src/cli_server.rs` (hoy `next_cursor: None` — server no pagina), `src/sdk/api.rs` (listPage con cursor)
- **Gate Justificación:** auditoría F0/F1 Alta #3 — VS-CORE-01 expuso cursor en native/WASM pero `server.rs` lo ignora; la consola web/Fase 3 no pagina; afecta ServerConnection.
- **Contrato:** `GET /api/v2/list` y `POST /api/v2/search` devuelven `next_cursor` real (mismo cursor del core, serialización string segura); paginación verificable: 2 llamadas con limit N devuelven N y resto; tests.
- **Verificación:** `cargo test --features server` verde; curl list con limit=2 → cursor → 2ª página sin duplicados.
- **Estado:** ✅ COMPLETO (commit 1b71d300)

### Task 9: REST-05 — `namespace_stats` en bridge desktop (gap VS-CORE-02)
- **Archivos clave:** `desktop/src-tauri/src/connections/{mod,native,server}.rs`, `desktop/src/vanta.ts`, `desktop/src/components/` (sidebar/HOME)
- **Gate Justificación:** auditoría F0/F1 Alta #4 — `namespace_stats` implementado en core pero ausente en bridge desktop → fallback local `list+count` (VS-04); sidebar/HOME con stats aproximadas; FEAT-02 lo necesita.
- **Contrato:** comando `vanta_namespace_stats` (espejo core) + wrapper `vanta.ts` `namespaceStats()`; Sidebar/HOME consumen stats reales (counts por namespace, dims, hnsw_nodes_count si disponibles); fallback local solo si el backend no lo soporta; build desktop verde.
- **Verificación:** `cargo test` desktop verde; `npm run build` verde; sidebar muestra stats reales de DB temp.
- **Estado:** ✅ COMPLETO (commit 08109a55)

### Task 10: REST-06 — IQL vía ServerConnection (consola IQL completa en web)
- **Archivos clave:** `desktop/src-tauri/src/connections/server_client.rs` (default `Unsupported`), `desktop/src/vanta-http-map.ts` (+test), `src/cli_server.rs` (query ya existe)
- **Gate Justificación:** auditoría F2/F3 Alta #4 — `queryResultFromResponse` truncado (ponytail) + `ServerConnection` hereda `Unsupported` → consola IQL degrada en web (sin graph_bfs, Read/Write/StaleContext truncados).
- **Contrato:** `ServerConnection.query` implementado (HTTP `/api/v2/query`), `queryResultFromResponse` completo (Read/Write/StaleContext, sin truncar); mapeo `vanta_query` + `vanta_iql_autocomplete` en vanta-http-map; tests roundtrip IQL en web.
- **Verificación:** node:test vanta-http-map verde; smoke: query IQL real desde browser contra server devuelve contexto completo.
- **Estado:** ✅ COMPLETO (commit 8ad119eb)

---

## Wave 2 — WASM/OPFS backbone (D10)

> **Rutas:** `vantadb-wasm/` (existe: `OpfsStorage`/`IdbStorage`/`OpfsWorkerProxy` — verificar API actual antes de contratar), `desktop/src/transport.ts` (factory), `desktop/src/` (nuevo modo standalone). Sub-agente: vanta-worker (bindings WASM + TS) — verificar contra docs oficiales de OPFS/IndexedDB antes de implementar (Regla 0 del lead).

### Task 11: WASM-01 — Persistencia browser real (OPFS/IndexedDB) probada
- **Archivos clave:** `vantadb-wasm/` (OpfsStorage/IdbStorage/OpfsWorkerProxy — estado real), `desktop/src/` (nuevo módulo storage browser)
- **Gate Justificación:** D10/D13 — la consola 100% browser sin server requiere persistencia real en el navegador; Fase 3 DEFER:110 dice que ya existen los primitivos pero nadie los probó de punta a punta.
- **Contrato:** inventario de lo que existe en vantadb-wasm (storage adapters, worker proxy) con verify de build; prueba E2E de persistencia: abrir consola WASM → put 10 records → reload browser → records persisten (OPFS); documentar límites (cuota, Safari, worker dedicado).
- **Verificación:** demo/test node + browser con reload; build vantadb-wasm verde; docs con límites reales verificados (no supuestos).
- **Estado:** ⏳ PENDING

### Task 12: WASM-02 — Transporte WASM backend en factory (consola sin server)
- **Archivos clave:** `desktop/src/transport.ts`, `desktop/src/vanta.ts` (factory), `desktop/src/vanta-wasm-map.ts` (nuevo: cmd → método WASM)
- **Gate Justificación:** D10 — el transporte abstracto (WEB-00) fue diseñado para enchufar WASM; sin `WasmBackend` la consola no puede correr 100% browser.
- **Contrato:** `WasmBackend.call(cmd, args)` → métodos del wrapper WASM (mapeo 1:1 con vanta_*; los no disponibles degradan con aviso, patrón WEB-04); factory `getTransport()`: Tauri → HTTP (con server) → WASM (sin server); `vanta.ts` sin cambio de firma; build desktop + web verdes.
- **Verificación:** `npm run build` (desktop+web) verde; node:test wasm-map verde; smoke: misma consola corre contra WASM.
- **Estado:** ⏳ PENDING

### Task 13: WASM-03 — Consola standalone 100% browser (modo sin server)
- **Archivos clave:** `desktop/vite.config.ts` (nuevo mode `wasm` o flag), `desktop/src/main.tsx`/`App.tsx` (modo standalone), `desktop/src/useConnectionState.ts` (conexión WASM implícita)
- **Gate Justificación:** D10 — el objetivo de Fase 4: "la consola se sirve desde el proceso embebido vía REST" (F3) → ahora también 100% browser sin servidor (WASM/OPFS).
- **Contrato:** build standalone (ej. `vite build --mode wasm`) → archivos estáticos que corren la consola completa contra WASM+OPFS sin ningún server; surfaces HOME/MEMORIAS/ACTIVITY/ÍNDICES/IQL funcionales; los comandos multi-conexión Tauri-only degradan con aviso; documentado (README/ADR).
- **Verificación:** build standalone verde; servido estático → navegar y hacer CRUD persistente (reload); E2E de smoke sin server.
- **Estado:** ⏳ PENDING

### Task 14: WASM-04 — Drag&drop `.vdbdump`/JSONL (import de archivos reales)
- **Archivos clave:** `desktop/src/components/` (ImportPaste → ImportDrop), OP-01 (textarea pegado existe)
- **Gate Justificación:** auditoría research Media — OP-01 es textarea pegado, no drag&drop; el research (01 lección 8:183, 02 §9:120) pide snapshots Qdrant-style `.vdbdump`/JSONL por drag&drop; natural en consola browser (File API + OPFS).
- **Contrato:** drop zone en MEMORIAS/import: arrastrar `.vdbdump`/`.jsonl`/`.csv` → parse (reuso parser OP-01) → preview → ingest; en modo WASM: leer File via FileReader/File System Access → persistir; modo server: subir multipart o base64 al endpoint import existente.
- **Verificación:** E2E: drop file real → records en grid; node:test parser reusado verde; build verde.
- **Estado:** ⏳ PENDING

---

## Wave 3 — Diferenciadores del research (los 3 "prometidos nunca tocados")

> **Rutas:** `desktop/src/components/lens/retrieval/` (VS-13), `desktop/src/components/` (sidebar/ÍNDICES placeholder — VS-03), core/sdk (consolidación). Sub-agente: vanta-worker (UI+core leve) con verify del lead; decisiones de contrato del usuario si aplica (patrón D5).

### Task 15: FEAT-01 — Slider de pesos híbridos BM25/vector en RETRIEVAL
- **Archivos clave:** `desktop/src/components/lens/retrieval/` (RetrievalLens.tsx, ScoreBars.tsx), `src/sdk/api.rs` (search — verificar si acepta pesos híbridos hoy), `src/cli_server.rs`/bridge (exponer si falta)
- **Gate Justificación:** auditoría research Alta #2 — el único elemento que falta de la barra RETRIEVAL es el slider de pesos (texto + vector-picker + filtros ya existen; SYNTHESIS §4 RETRIEVAL:123, 01 lección 2:177). Diferenciador de memoria priorizado por el research.
- **Contrato:** verificar si `search`/`hybrid_search` acepta `alpha`/pesos (BM25 vs vector); si no, exponerlo (core aditivo si es trivial, sino REST wrapper); UI: slider (0=BM25 puro, 1=vector puro, default=RRF/50) en barra RETRIEVAL; ScoreBars reflejan el peso activo; tests + build verde.
- **Verificación:** `cargo test` core verde (si toca core); node:test/build verde; smoke: slider cambia resultados visiblemente en DB temp.
- **Estado:** ⏳ PENDING

### Task 16: FEAT-02 — Superficie Índices/salud (placeholder de VS-03 → real)
- **Archivos clave:** `desktop/src/components/` (sidebar ÍNDICES placeholder), `src/cli_server.rs`/bridge (`namespace_stats`, `/api/v2/metrics` — REST-02/REST-05 lo alimentan), `src/sdk/` (fuente: hnsw_nodes_count, dims, LSM/WAL)
- **Gate Justificación:** auditoría research Alta #3 — la superficie Índices/salud sigue placeholder (VS-03:75); SYNTHESIS §4 OPERACIONES:150 la pide (hnsw_nodes_count, dims, LSM/WAL, charts del motor).
- **Contrato:** surface ÍNDICES real: counts por namespace (REST-05), dims, hnsw_nodes_count, LSM/WAL status (si el core lo expone — verificar; si no, exponer wrapper mínimo), salud (health endpoint); charts simples (reuso de patterns ScoreBars); funciona en desktop (bridge) y web (REST).
- **Verificación:** build verde; smoke: sidebar muestra stats reales ≠ placeholder; si falta métrica core → documentar gap + task follow-up (no mentir en UI).
- **Estado:** ⏳ PENDING

### Task 17: FEAT-03 — Consolidación asistida (duplicados/superados con diff visible)
- **Archivos clave:** `desktop/src/components/` (nueva surface/lente), `src/sdk/api.rs` (search/hybrid — fuente de candidatos), core si requiere (decay Mem0/memify Cognee pattern)
- **Gate Justificación:** auditoría research Alta #2 — nunca tocado (SYNTHESIS §4 OPERACIONES:155, 03 lección 5:252, 07 Fix 3:89); diferenciador de memoria: marcar registros duplicados/superados (misma entidad, versiones nuevas) con diff visible.
- **Contrato:** definir con el usuario el alcance mínimo viable (D16): (a) UI-only: detectar candidatos por similitud (search kNN) + diff visible entre pares + sugerencia de "superado por" (metadata `superseded_by`); (b) core decay: si requiere decay automático → task core separada con contrato. Entregar al menos (a) en esta fase; (b) documentado como follow-up.
- **Verificación:** node:test (lógica de detección/diff) verde; smoke: DB temp con duplicados → surface los marca con diff; build verde.
- **Estado:** ⏳ PENDING (requiere decisión D16 del usuario antes de ejecutar)

---

## Wave 4 — Verificación E2E + cierre

### Task 18: VER-01 — E2E ampliado + ADR-027 + cierre de fase
- **Archivos clave:** `desktop/scripts/selfcheck-web-e2e.ts` (ampliar), `docs/architecture/` (ADR-027), `docs/Backlog.md` (estados F4), `docs/CHANGELOG.md`
- **Gate Justificación:** cierre de fase — probar ambos modos: dashboard servido por server (F3 + deuda REST) y consola standalone WASM/OPFS (F4); documentar D13/D14/D15.
- **Contrato:** E2E ampliado: (a) server: ráfaga sin 429 (REST-01), metrics JSON (REST-02), graph_v2 roundtrip (REST-03), paginación (REST-04), IQL completo (REST-06); (b) standalone: WASM build → navegar → CRUD persistente con reload (WASM-03); ADR-027 (D13/D14/D15 + deuda REST cerrada + WASM/OPFS); Backlog: P26 filas F4 actualizadas, tasks DOC-*/REST-*/WASM-*/FEAT-*/VER-* completadas; CHANGELOG entrada Unreleased; plan → archive/ con retrospectiva (métrica verify retries/tarea, baseline >90% primer intento).
- **Verificación:** E2E server + standalone exit 0; docs presentes; `pwsh -File scripts/validate-docs-coverage.ps1` exit 0; `git log` coherente.
- **Estado:** ⏳ PENDING

---

## DEFER table

| Item | A cuándo | Motivo |
|------|----------|--------|
| Auth fuerte 3 capas en server | MEM-05 (memory engine) | D15/D12; hoy local-first loopback; WASM no tiene server que autenticar |
| PaCMAP → core WASM (`embedding_projection`) | Fase 5 | Diferido en F2; UMAP-js lo sustituye hoy; evolución de ESPACIO |
| Vista Matriz (heatmap + dendrograma top-k) | Fase 5 | 3ª vista complementaria del research; requiere trabajo de proyección nuevo |
| Temporalidad del grafo (valid_at/invalid_at) | Fase 5+ | el core no modela tiempo; cambio de modelo de datos grande |
| 3D toggle en ESPACIO | solo si el usuario pide | anti-patrón 6 del research ("3D por moda") |
| SSE streaming en `/api/v2/query` | — | no requerido por la consola actual |
| Decay automático (Mem0/memify) en core | follow-up FEAT-03 (b) | definir D16 primero |
| Matriz de duplicados por embedding (vs textual) | FEAT-03 iteración | MVP textual primero |

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Wave 0 (docs) se percibe como "no código" y se salta | Alto | Es bloqueante (dictamen 🔴); sin estados reales, el resto del plan lee estado falso; DOC-01..04 son del lead, 1 wave completa |
| OPFS/IndexedDB API cambia o no soporta cuota esperada | Med | WASM-01 verifica límites reales ANTES de contratar; fallback IndexedDB si OPFS no disponible (Safari) |
| Slider de pesos requiere cambio core (search no acepta pesos hoy) | Med | FEAT-01 verifica primero; si requiere core → tarea core aditiva con contrato (patrón VS-CORE-*) |
| `namespace_stats`/metrics no expone todo lo que FEAT-02 quiere | Med | FEAT-02 documenta gap + follow-up; no mentir en UI (placeholder honesto) |
| Rate limiter "calibrado" rompe AUD-021 (fail-closed) | Med | REST-01 solo relaja loopback sin auth; con `require_auth` sigue fail-closed |
| WASM bundle grande (>1 MB) afecta consola standalone | Bajo | lazy chunks (patrón GRAFO-02 600 kB); medir en WASM-02 |
| graph_v2 DTO introduce breaking change en REST existente | Med | Endpoint NUEVO (`/v2/graph/v2/*`), no tocar el existente; semver-checks en publish |

## RECITATION (progreso — patrón lead)

- **Estado:** ⏳ PLANEADO — 18 tareas (DOC-01..04, REST-01..06, WASM-01..04, FEAT-01..03, VER-01), campaña `e7b31c4a-8d2f-4a7e-9c1b-6f5a3e8d2c40`. Auditoría multi-agente 2026-08-19 completada (4 sub-agentes, digest del lead). Decisiones D13/D14/D15 aprobadas. No ejecutado.
- **Próximo:** ejecutar `/pipeline run docs/plans/2026-08-19-vanta-studio-fase4.md` — Wave 0 (reconciliación documental, lead) antes de delegar Wave 1.