# Plan de Ejecución: Vanta Studio — Fase 1 (Explicabilidad y tiempo)

> **Campaign ID:** 8c5e2a9f-1b3d-4e6f-8a7c-9d2b4f6a8e10
> **Inicio:** 2026-08-18
> **Estado:** ⏳ EN PROGRESO (creado 2026-08-18)
> **Fuente:** `docs/research/human-facing-db-ui/06-synthesis/SYNTHESIS.md` §7 Fase 1 + tabla DEFER del plan Fase 0 (líneas 185-200) + contratos verificados contra el core/bridge (2026-08-18).
> **Predecesor:** `docs/plans/2026-08-18-vanta-studio-fase0.md` (✅ FASE 0 COMPLETA, 14/19).
> **Modo:** ondas — bridges/gaps primero (secuencial), luego UI Fase 1 (paralelo con archivos no compartidos).

## Decisiones del usuario (heredadas de Fase 0 + alcance Fase 1)

| # | Decisión | Valor (aplica a Fase 1) |
|---|----------|------------------------|
| D2 | Retención de versiones | **VS-CORE-07** ejecuta D2 completo: planificación → investigación → análisis → implementación (tras aprobación). Historial+Diff **EN ESPERA** hasta que VS-CORE-07 esté aprobado/implementado. |
| D3 | Distribución | Solo desktop. Deep links `vanta://` sí (Tauri v2, registro de URI scheme en Windows). |
| D4/D6 | Estética | Tokens manga/linocut ya en `desktop/src/index.css` (VS-01) — nuevas vistas las reutilizan, no las redefinen. |
| D7 | Tema | Dark ya implementado; nuevas vistas deben soportar `.dark` desde el inicio. |
| P6 | Commit explícito | Se mantiene: nada de auto-guardar en Historial+Diff (revertir/restaurar vN es acción explícita). |
| P7 | Explicabilidad first-class | **Contrato central de esta fase**: desglose de score por barras horizontales (longitud = score, color solo secundario — Cleveland–McGill). |

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 2 gaps bridge (VS-CORE-03 explain, VS-12 audit) + 1 análisis core (VS-CORE-07) + 6 superficies UI (VS-13..18) | Lente GRAFO/IQL (Fase 2, D5), ESPACIO (Fase 2), batch ops (Fase 2), import CSV (Fase 2), web (Fase 3) | Todo lo de Fases 2-3 | Historial+Diff (VS-14) queda bloqueado hasta VS-CORE-07 aprobado |

## Orden de ejecución

1. **Wave 0 — Gaps bridge/core (paralelo, archivos disjuntos):**
   - **VS-CORE-03** (desktop bridge, `types.rs`/`native.rs`/`data.rs`) — exponer `explain`; NADA de core (ya existe).
   - **VS-12** (desktop bridge, `native.rs`/`commands/`) — audit log: configurar `audit_log_path` + comando `vanta_audit_events`.
   - **VS-CORE-07** (core, D2) — análisis → propuesta → implementación tras aprobación del usuario. No toca bridge.
2. **Wave 1 — Superficies UI Fase 1 (paralelo, archivos disjuntos):**
   - **VS-13** Lente RETRIEVAL (depende VS-CORE-03).
   - **VS-14** Historial+Diff (depende VS-CORE-07 aprobado — si no está aprobado al llegar la wave, queda ⏳ bloqueada y se continúa con el resto).
   - **VS-15** ACTIVITY + Timeline (depende VS-12).
   - **VS-16** Deep links + export vistas + reporte markdown (independiente).
   - **VS-17** Favoritos/historial + Copy-as (independiente, localStorage/clipboard).
3. **Wave 2 — A11y transversal:**
   - **VS-18** Encoding redundante (color + ícono + texto) en chips/badges de MEMORIAS/Inspector — se ejecuta al final porque toca componentes de todas las tareas previas.

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración la hace el lead
- `src/sdk/types.rs` — tipos públicos del core: cambios SOLO vía task file con contrato (VS-CORE-07)
- `desktop/src/components/layout/WorkspaceShell.tsx` — archivo compartido; cada tarea declara su slice aditivo en el task file; merge lo hace el lead

---

## Wave 0 — Gaps bridge/core

### Task 1: VS-CORE-03 — Exponer `explain` en el bridge desktop (re-scopeado: consumir, no crear)
- **Archivos clave:** `desktop/src-tauri/src/connections/types.rs` (`SearchQuery:56-71` — NO tiene `explain`; `SearchResult:75-83` — NO tiene `explanation`), `desktop/src-tauri/src/connections/native.rs` (mapeo → `VantaEmbedded.search`), `desktop/src-tauri/src/commands/data.rs:57` (`vanta_search`), referencia core: `src/sdk/types.rs:434-463` (`VantaSearchExplanation`/`VantaSearchExplanationHit`), `src/sdk/search/explain.rs:12` (`explain_memory_search`).
- **Gate Justificación:** SYNTHESIS P7 + §7 Fase 1 "Lente RETRIEVAL: desglose de score por barras apiladas". El core YA produce el desglose (verificado 2026-08-18: `explain_hit` en `src/sdk/search/debug.rs:49` rellena `bm25_terms` + `rrf_text_rank`/`rrf_vector_rank`; `fusion_report` con candidates/RRF-k). El bridge es el único que no lo expone.
- **Contrato (aditivo, backward-compat):** añadir `explain: bool = false` (serde default) a `SearchQuery`; añadir `explanation: Option<ExplanationHit>` a `SearchResult` con shape: `{identity, score, snippet?, matched_tokens, matched_phrases, bm25_terms: [{token, tf, df, doc_len, contribution}], rrf_text_rank?, rrf_vector_rank?}` (espejo de `VantaSearchExplanationHit`); en native.rs, cuando `explain=true`, llamar `search_with_method`/`search` con el request explicado y mapear `VantaSearchExplanationHit` 1:1. `vanta.ts`: `SearchQuery.explain?` + `SearchResult.explanation?`. **No tocar el core.**
- **Verificación:** `cargo check` + `cargo test -j 1` en `desktop/src-tauri` (nuevo test: search con explain=true rellena explanation con bm25_terms/rrf ranks); `npm run build` verde; wire shape verificado con test de roundtrip.
- **Estado:** ⏳ PENDIENTE

### Task 2: VS-12 — Audit log en desktop: configurar `audit_log_path` + comando `vanta_audit_events`
- **Archivos clave:** `desktop/src-tauri/src/connections/native.rs` (abre con solo `path` — verificado: `NativeConnection::open(path)` NO pasa `VantaConfig.audit_log_path`; grep en desktop: 0 matches de audit), `desktop/src-tauri/src/commands/` (nuevo `audit.rs` + registro en `lib.rs`), `src/audit.rs` (`AuditEvent{timestamp,op,namespace,key,outcome,reason}`, `AuditLogger`), `src/config.rs` (`VantaConfig.audit_log_path`).
- **Gate Justificación:** SYNTHESIS §4 OPERACIONES "Activity: audit log filtrable" + "Timeline (Fix 3)" + DEFER Fase 1 "ACTIVITY + Timeline (audit log)". El core ya escribe el JSONL (opt-in); el desktop ni lo configura ni puede leerlo.
- **Contrato:** (a) `NativeConnection::open` acepta audit path opcional y lo pasa a `VantaConfig` (default: `<storage_path>/audit.jsonl`); (b) comando `vanta_audit_events(namespace?, op?, outcome?, limit?, cursor?)` que lee el JSONL del final hacia atrás (tail), filtra en Rust y devuelve `Vec<AuditEvent>` + `next_cursor` (offset); (c) `vanta.ts` `auditEvents()`. Aditivo: no cambia el contrato de conexiones existente; si no hay audit configurado → error claro `Unsupported("audit log no configurado")`.
- **Verificación:** `cargo test -j 1` (test: put/delete generan eventos; filtro por namespace/op; cursor sin solapamiento); `npm run build` verde.
- **Estado:** ⏳ PENDIENTE

### Task 3: VS-CORE-07 — Retención de versiones históricas en `VantaMemoryRecord` (D2 completo)
- **Archivos clave:** `src/sdk/types.rs:175` (`VantaMemoryRecord`), `src/sdk/api.rs:291-295` (put bump in-place `version+1` — verificado: NO retiene snapshots), `src/backends/fjall_backend.rs` (KV extra `versions/{ns}\0{key}\0{ver}` propuesto en Fase 0 Task 19), `src/storage/engine/ops.rs` (write path).
- **Gate Justificación:** D2 del usuario: **planificación → investigación → análisis → implementación**. Destraba Historial+Diff (Fix 3). Cláusula de doble consumidor: lo necesitan P26 (Studio, Historial+Diff) y P27 (memory, offload/skills versionadas) — diseñar UNA vez.
- **Contrato (Fase 1 de la tarea):** ejecutar **análisis de decisión** en el core y entregar **propuesta** (task file con sección "PROPUESTA"): ¿retener snapshots n-1 vs n-k vs solo actual+anterior? ¿coste de almacenamiento/compacción (1 write extra por put)? ¿API de acceso (`get_version(ns,key,ver)` + `versions(ns,key)`)? ¿integración con put_batch/import/expiración? → **Checkpoint humano obligatorio antes de implementar** (aprobación). Implementación SOLO tras aprobación.
- **Verificación (análisis):** propuesta con trade-offs + API propuesta + tests afectados; sin código hasta aprobación.
- **Estado:** ⏳ PENDIENTE

---

## Wave 1 — Superficies UI Fase 1

### Task 4: VS-13 — Lente RETRIEVAL (¿por qué recuperó esto?)
- **Archivos clave:** `desktop/src/components/lens/retrieval/` (nuevo), `desktop/src/components/layout/WorkspaceShell.tsx` (slice aditivo: surface `RETRIEVAL` + entrada sidebar), `desktop/src/vanta.ts` (usa `search` con `explain: true` de VS-CORE-03).
- **Gate Justificación:** SYNTHESIS §4 "Lente RETRIEVAL (Fix 5)": el diferenciador más barato y más pedido del mercado (Mem0 `explain=True`, Zep provenance). P0 de 03.
- **Contrato:** barra de consulta (texto + vector-picker de registro existente + top-k + umbral) + filtros visuales por metadata (reutilizar `filters-core.ts`/`toVantaMemoryFilter` de VS-07) + resultados con **desglose de score como barras horizontales apiladas** (longitud = score; segmentos: BM25 vía `bm25_terms`/`rrf_text_rank`, HNSW vía `rrf_vector_rank`, RRF vía `fusion_report`; color solo secundario — P7/Cleveland–McGill) + por resultado "ver contexto" (vecino semántico → get/neighbors vía `search` con vector del registro; historial del audit si VS-12 ya está). Encoding redundante (barras + número + tooltip). Surface accesible desde cualquier registro seleccionado (P4: lente contextual, no destino aparte).
- **Verificación:** `npm run build` verde; self-check script (shape de barras: componente ausente → segmento 0, no crashea); datos reales de una DB temp con 3 records + query.
- **Estado:** ⏳ PENDIENTE

### Task 5: VS-14 — Historial+Diff entre versiones (tab en Inspector)
- **Archivos clave:** `desktop/src/components/inspector/Inspector.tsx` (VS-06: añadir tab), `desktop/src/components/inspector/historial-tab.tsx` (nuevo), `desktop/src/vanta.ts` (usa API de VS-CORE-07 tras aprobación).
- **Gate Justificación:** SYNTHESIS §4 Inspector "Historial+Diff (Fix 3)": cada versión con su cambio resaltado (payload/metadata/vector) estilo git. DEFER Fase 0: "En espera hasta VS-CORE-07 (D2)".
- **Contrato:** lista de versiones (v1..vN con timestamp) + diff entre dos versiones seleccionadas: payload (line-diff), metadata (KV diff añadido/quitado/cambiado), vector (norma/dim + "cambió" sí/no); **revertir a vN es acción explícita** (botón → confirmación → put → P6). **BLOQUEADA** si VS-CORE-07 no está aprobado — en ese caso el task file queda ⏳ y el plan continúa con VS-15..17.
- **Verificación:** `npm run build` verde; self-check con fixture de 3 versiones.
- **Estado:** ⏳ PENDIENTE (bloqueada por VS-CORE-07)

### Task 6: VS-15 — ACTIVITY + Timeline (audit log filtrable y agrupado en el tiempo)
- **Archivos clave:** `desktop/src/components/activity/` (nuevo: ActivityPanel + Timeline), `desktop/src/components/layout/WorkspaceShell.tsx` (slice aditivo: surface ACTIVITY), `desktop/src/vanta.ts` (`auditEvents()` de VS-12).
- **Gate Justificación:** SYNTHESIS §4 OPERACIONES "Timeline (Fix 3)" + "Activity: audit log filtrable por namespace/op/outcome (equivalente al Profiler/SlowLog de RedisInsight)" + DEFER Fase 1.
- **Contrato:** Timeline unificada (escrituras/actualizaciones/expiraciones/borrados por namespace, agrupada por hora/día) + Activity como tabla filtrable (namespace/op/outcome, paginada con cursor de VS-12, hover → key/record en Inspector). Encoding redundante por op (color+ícono+texto). Empty state honesto si audit no configurado (VS-12 error → banner "audit log no habilitado", con hint de dónde se configura).
- **Verificación:** `npm run build` verde; self-check con audit fixture JSONL.
- **Estado:** ⏳ PENDIENTE

### Task 7: VS-16 — Deep links `vanta://` + export de vistas + reporte markdown
- **Archivos clave:** `desktop/src-tauri/src/lib.rs` (handler deep link; verificar API oficial `tauri-plugin-deep-link` con webfetch ANTES de codificar — Regla técnica 0), `tauri.conf.json`, `desktop/src/vanta.ts` (parseo `vanta://ns/key?query=`), `desktop/src/components/export/` (nuevo), reusa `listPage()`/filtros VS-07.
- **Gate Justificación:** SYNTHESIS §6 "Deep links / portabilidad: URLs `vanta://ns/key?query=...` + export JSON/Markdown/CSV — barato, muy valorado en ecosistema embebido" + DEFER Fase 1 "Deep links + export vistas + reporte legible (markdown)".
- **Contrato:** (a) registro de URI scheme `vanta://` (Windows) y handler que navega a namespace/key/query al abrir la app; (b) export de la **vista actual** (grid filtrada de VS-07 → JSONL; resultado de RETRIEVAL si la lente está activa) no solo namespace completo; (c) **reporte legible markdown** del estado (reusa `namespace_stats` de VS-CORE-02: conteos, tipos, TTL próximos) con botón copiar/descargar. Todo con encoding manga existente.
- **Verificación:** `npm run build` verde; deep link test manual (Windows: `start vanta://...` → app navega); export genera JSONL válido (parseable por import).
- **Estado:** ⏳ PENDIENTE

### Task 8: VS-17 — Favoritos/historial de búsqueda + Copy-as
- **Archivos clave:** `desktop/src/components/palette/CommandPalette.tsx` (VS-09: grupos nuevos), `desktop/src/store/favorites.ts` (nuevo, localStorage), `desktop/src/components/layout/WorkspaceShell.tsx` (slice aditivo mínimo: estado favoritos), `desktop/src/components/` (botones copy-as en grid/inspector).
- **Gate Justificación:** DEFER Fase 1 "Favoritos/historial de búsqueda (02 P1, 05 filtros) — barato (localStorage), complementa Ctrl+K" + "Copy-as (02 P3: copiar registro/query/key en JSON/JSONL/markdown)".
- **Contrato:** (a) favoritos de namespaces/keys con toggle (★) persistidos en localStorage, listados en palette (grupo FAVORITOS) y sidebar; (b) historial de las últimas N búsquedas (localStorage, re-ejecutables desde palette); (c) Copy-as: desde grid/inspector copiar registro completo (JSON), key, o payload (markdown) — botón con feedback "copiado". Sin nuevas deps (navigator.clipboard + localStorage).
- **Verificación:** `npm run build` verde; self-check localStorage roundtrip.
- **Estado:** ⏳ PENDIENTE

---

## Wave 2 — A11y transversal

### Task 9: VS-18 — Encoding redundante (color + ícono + texto) en chips/badges
- **Archivos clave:** `desktop/src/components/DataExplorer.tsx` (chips de metadata/vector/version/TTL de VS-05), `desktop/src/components/inspector/` (badges de VS-06), `desktop/src/components/lens/retrieval/` (barras de VS-13 si ya existe), `desktop/src/components/mark/mark.css` (si aplica).
- **Gate Justificación:** SYNTHESIS P15 "Encoding redundante (color + ícono + texto) para tipos `VantaValue`, TTL y estados — accesible a daltonismo" + DEFER Fase 1 "A11y pass Fase 1 (encoding redundante chips/badges en MEMORIAS/Inspector)".
- **Contrato:** cada estado actualmente solo-color (tipos `VantaValue`, TTL activo/expirando/expirado, version, vector presente/ausente) gana ícono + texto (o patrón) adicionales; **no romper** el layout de VS-05/06; `prefers-reduced-motion` respetado; contraste AA verificado en claro y dark.
- **Verificación:** `npm run build` verde; revisión visual (screenshot de grid + inspector en claro/dark); checklist AA.
- **Estado:** ⏳ PENDIENTE

---

## Relación con P27 (Vanta Memory Engine)

Contratos compartidos ya documentados en plan Fase 0 §Relación (líneas 170-181). En Fase 1 se tocan dos:

| # | Contrato | Lado Studio (P26 Fase 1) | Lado Memory (P27) | Estado |
|---|----------|--------------------------|-------------------|--------|
| 1 | `explain_memory_search` | Lente RETRIEVAL (VS-13) consume `VantaSearchExplanation` vía bridge (VS-CORE-03) | Recall (F4) usa el mismo search | Un contrato, dos consumidores — el core ya lo produce; el bridge lo expone en esta fase |
| 3 | Audit log JSONL compartido | ACTIVITY+Timeline (VS-15) lee el MISMO JSONL que VS-12 configura | Telemetría por capa (MEM-34) escribe en el mismo log | VS-12 hace la disciplina configurable; sin código P27 hoy |

**Punto de diseño compartido (no bloqueante):** VS-CORE-07 (retención de versiones) — diseñar UNA vez (task file con cláusula de doble consumidor). **P27 no bloquea nada de esta fase; la integración real se toca en la 2ª iteración (F4/F5).**

---

## DEFER (fuera de esta fase)

| Item | Cuándo | Estado |
|------|--------|--------|
| Lente GRAFO (R3F propio, D5) + IQL console con autocompletado | Fase 2 | Espera VS-CORE-06 (bridge `vanta_query`) |
| Lente ESPACIO (regl-scatterplot + UMAP-js worker) | Fase 2 | — |
| Batch ops con confirmación + undo | Fase 2 | — |
| Import CSV/JSON pegado | Fase 2 | — |
| Matriz (heatmap + dendrograma de similitud) | Fase 2 | Vista complementaria de GRAFO |
| Fase 3 web/embebido | Fase 3 | D3: solo desktop por ahora |

---

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| VS-CORE-07 (retención versiones) es diseño de core — puede requerir iteración con vanta-arch/engine | Historial+Diff (VS-14) bloqueada | VS-14 queda ⏳ y la wave continúa con VS-15..17; decisión D2 ya preveía esto |
| Audit log no configurado en DBs existentes | ACTIVITY vacío en la primera apertura | VS-12 lo configura desde el inicio en el desktop; empty state honesto con hint |
| Parallelismo tocando `WorkspaceShell.tsx` (lección Fase 0: 3 agentes lo rompieron en WIP) | Builds intermedios rotos | Slices aditivos declarados por tarea; MAX_CONCURRENT 3; merge de integración lo hace el lead antes de commit |
| `tauri-plugin-deep-link` API desconocida | VS-16 bloqueada | Regla técnica 0: webfetch de docs oficiales ANTES de codificar |
| El `explain` del core es solo texto/ranks, no "scores" BM25/HNSW separados por hit | Barras apiladas con segmentos desiguales | `bm25_terms` da desglose BM25; `rrf_*_rank` da posición por rama; `fusion_report` da el contexto RRF — las barras muestran lo que existe, sin inventar scores |

---

=== RECITATION ===