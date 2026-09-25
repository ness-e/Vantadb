# FIND-125 — resync tipos `vantadb-ts` contra core/wasm (`tsc` 0)

> **Plan:** `docs/dev/plans/2026-09-19-ci-green.md` (Wave1, primera en secuencia) · **Campaign:** 0ad2d7e2-94e3-4313-8f5c-e8d57c08a6af
> **Estado:** ⏳ IN PROGRESS · **Ruta:** vanta-worker · **Branch:** develop · **Commit:** `fix: FIND-125 — ...`
> **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠 Media-Alta · **Wave:** Wave1 (FIND-126 después, la ejecuta el orquestador)

## 1. TAREA — objetivo + contrato + acceptance criteria

**Objetivo:** eliminar los 16 errores `tsc` en `vantadb-ts/src/vantadb.ts` causados por drift entre los tipos del SDK TS (`vantadb-ts/src/types.ts`), el `.d.ts` hand-written del binding (`vantadb-wasm` pkg) y el código Rust real (fuente de verdad).

**Contrato (ley):**
- (a) `npx tsc --noEmit` → 0 errores en `vantadb-ts/`
- (b) tests TS verdes (`npm test` / `vitest run` en `vantadb-ts/`)
- (c) **cero cambios de runtime** — solo tipos. Si un fix exige runtime → Stop y re-scope (no reescribir el SDK).

**Errores (reproducidos localmente 2026-09-19, idénticos al log CI run 35413944241):**

| # | Loc | Error |
|---|-----|-------|
| 1 | 395:30 | `Property 'runtime_profile' does not exist on type 'Capabilities'` (wasm .d.ts) |
| 2 | 546:5 | `next_cursor: string \| undefined` no asignable a `number \| undefined` (`MemoryListPage`) |
| 3 | 603:37 | `Record<string, unknown>` → `SearchRequestInput` (`search`) |
| 4 | 657:55 | idem (`search_multi` vía `searchMulti`) |
| 5 | 801:5 | `SearchExplanation` → `Record<string, unknown>` (`explainSearch` return) |
| 6 | 802:40 | `Record<string, unknown>` → `SearchRequestInput` (`explain_memory_search`) |
| 7 | 829:5 | `ExportReport` sin `namespaces, path, duration_ms` (`exportNamespace`) |
| 8 | 870:5 | idem (`exportAll`) |
| 9 | 938:5 | `ImportReport` sin `inserted, updated, skipped, duration_ms` (`importFile`) |
| 10 | 1090:5 | wasm `OperationalMetrics.mmap_resident_bytes?: string` vs SDK `string \| null` |
| 11 | 1108:5 | `IqlResult` vs `QueryResult` (`query`) |
| 12 | 1181:20 | wasm `NodeRecord` vs SDK `NodeRecord` (`fields`/`Value` vs `MetadataValue`) |
| 13 | 1322:5 | `string[]` vs `GraphBfsResult` (`bigint[]`) |
| 14 | 1346:5 | `string[]` vs `GraphDfsResult` |
| 15 | 1366:5 | `string[]` vs `GraphTopologicalSortResult` |
| 16 | 1413:5 | `string[]` vs `GraphBfsResult` (`graphFilteredTraversal`) |

**Stop del plan:** drift exige rediseño de tipos → DEFER con diagnóstico (no reescribir SDK). No disparado: todo se resuelve con resync + casts de frontera (ver §7).

## 2. ARCHIVOS — clave / relacionados / prohibidos

**Clave (con :línea del error):**
- `vantadb-ts/src/vantadb.ts` (395, 546, 603, 657, 801, 802, 829, 870, 938, 1090, 1108, 1181, 1322, 1346, 1366, 1413)
- `vantadb-ts/src/types.ts` (`MemoryListPage:60-63`, `ListOptions:54-58`, `OperationalMetrics:166-204`, `Capabilities:206-212`, resto como referencia correcta)
- Fuente de verdad (solo lectura): `src/sdk/types.rs:129-245` (OperationalMetrics, Capabilities, RuntimeProfile:61-68, StorageTier:72-76), `src/sdk/types/record.rs:179-203` (Export/ImportReport), `src/sdk/types/graph.rs:14-32` (QueryResult), `vantadb-wasm/src/lib.rs` (capabilities:1123, SearchRequest:152-168, ListOptions:179-189, next_cursor_to_js:224-229, JsNodeRecord:234-247, JsOperationalMetrics:269-318, graph_bfs/dfs/topo/filtered:1769-1891, query:1675, get_node:1714, export/import:1383-1551, metrics:1667)

**Relacionados (callers/callees):**
- `vantadb-ts/src/guards.ts` (`_buildSearchRequest` base usada por los 3 errores de search; `_mapRecord`)
- `vantadb-ts/src/native.ts` (segundo consumidor de `Capabilities, ListOptions, MemoryListPage` — nativo retorna `next_cursor?: number`, por eso el resync usa unión `string \| number`)
- `vantadb-ts/src/__tests__/` (excluidos de `tsc` por tsconfig, corren en vitest: `vanta.test.ts`, `dx04.test.ts`, `hardening.test.ts:360` round-trip cursor, `types.test.ts`, `integration.test.ts`)
- `vantadb-ts/tests/graph.test.ts` (TS-01: wire graph = `bigint[]` — prueba de la fuente de verdad)
- `vantadb-ts/package.json` (`test: vitest run`), `vantadb-ts/tsconfig.json` (include solo `src/**/*.ts`, excluye `__tests__`)
- `vantadb-node/index.d.ts:74-83` (nativo: `cursor?: number`, `next_cursor?: number` — justifica la unión)

**PROHIBIDOS (no tocar):**
- `src/` Rust (solo lectura como fuente), `vantadb-wasm/` (ni `src/vantadb_wasm.d.ts` ni `pkg/` — el drift del .d.ts se absorbe con casts, no se edita otro paquete), `web/` (FIND-123 ✅), `desktop/` (FIND-124 ✅), `reparacion.bat`, `.opencode/`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation al cierre), `C:/Users/Eros/.vantadb*` (datos vivos). Reescribir el SDK prohibido.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-ts/src/types.ts` (259L), `vantadb-ts/src/guards.ts` (268L), `vantadb-wasm/pkg/vantadb_wasm.d.ts` (794L), `tests/graph.test.ts` (97L), `__tests__/types.test.ts` (100L); parciales con foco en errores: `vantadb.ts` (1487L, leído entero en 3 ventanas), `native.ts:1-60,195-334`, `lib.rs` (focos listados arriba), `types.rs:120-259`, `record.rs:170-229`, `graph.rs:1-60`, `vanta.test.ts:290-409,560-679`, `dx04.test.ts:1-45,270-319,395-432`, `hardening.test.ts` (focos), `portability.test.ts:130-163`.
- **Referencias hacia dentro (lo que vantadb.ts consume):** `types.ts` (tipos SDK), `guards.ts` (`_mapRecord`, `buildSearchRequestBase`), `metadata.ts` (normalize*), `errors.ts` (`wrapWasmError`), `vantadb-wasm` (tipos del binding).
- **Referencias entrantes (quién consume lo que cambio):** `native.ts` consume `Capabilities, ListOptions, MemoryListPage` (verificado compatible con unión); `__tests__/` consumen todo en runtime (vitest no typecheckea; comportamiento intacto); `tests/*.test.ts` importan `Client` (sin cambios de firma pública salvo `ListOptions.cursor`/`MemoryListPage.next_cursor` que se ensanchan, y `OperationalMetrics` que gana opcionales + 2 requeridos — ningún test los construye literalmente, verificado por grep).
- **Veredicto:** impacto bajo y acotado a 3 archivos (`types.ts`, `vantadb.ts`, +1 línea en `native.ts` — cast borrado por el ensanchamiento de `cursor`; detectado por tsc en Step 1). Sin símbolos públicos nuevos. Sin cambios de runtime (casts `as unknown as` + ensanchamientos de tipo). Humble Object (§V.1: `vantadb-ts` traduce DTO↔externo, cero lógica) — los casts viven en la frontera, correcto por capa.

## 3. DEPENDENCIAS

- **Wave1 primera en secuencia** (Wave0 ✅ ×2: FIND-124 `797059bf`, FIND-123 `f0d6e9cb`). Sin tareas bloqueantes.
- **NextTask tras cierre:** FIND-126 (la ejecuta el orquestador, no yo).
- Gate D evaluado: blast radius 2 archivos, sin símbolos públicos nuevos, contrato no ambiguo, resync tipos-only (no feature-add) → **no disparado** (sin `question` disponible en toolbox; motivo registrado).

## 4. REFERENCIAS

- **Rules (leída completa antes de actuar):** `.opencode/rules/js-ecosystem.md` (R-1 persistencia doc, **R-2 `pkg/` artefacto — no se edita ni se cita como fuente**, R-3 standalone intencional, R-4 op-gate/drenaje).
- **Norma transversal:** `.opencode/references/clean-code-clean-architecture.md` Ap. V (Humble Objects: casts en frontera OK; severidades; `any` prohibido → se usa `unknown`).
- **Refs:** `definition-of-done.md` (standing checklist + DoD VantaDB: `tsc --noEmit`, tests, OCR, docs si aplica), `dev-tools.md` (comandos).
- **Commands:** `pipeline.md` (ejecución), `audit.md` (verify).
- **SPEC.md raíz:** sin cambios (0 greenfield).
- **Tabla Spec:** N/A (resync interno, sin símbolos públicos nuevos — justificación por evidencia: §7 muestra que cada fix preserva la firma pública o la ensancha de forma compatible).

## 5. SKILLS (SDP Paso 0b — ejecutado real vía `campaign_discover_skills_v2` phase=BUILD)

Keywords: `[typescript, tsc, tipos, drift, wasm, resync]` → 8 devueltas (scores 1.00):

| Skill | Cuándo aplica (1 línea) |
|---|---|
| campaign-executor | Base: state machine PLAN→ACT→VERIFY de esta tarea |
| source-driven-development | Base TS-SDK: fuente de verdad = código Rust/wasm real, no docs |
| incremental-implementation | Lifecycle BUILD: 2 slices (types.ts → vantadb.ts) + verify por slice |
| test-driven-development | Lifecycle BUILD: reproducir (tsc RED) → fix mínimo → verde; Prove-It si un fix rompe tests |
| context-engineering | Lifecycle BUILD: context pack Rules→Plan→Source→Error aplicado en DISCOVERY |
| doubt-driven-development | Lifecycle BUILD: stakes altos (SDK público npm roto) — verificación adversarial de cada cast |
| frontend-ui-engineering | Lifecycle (no aplica a este slice — sin `web/`; registrada por SDP, no se carga) |
| api-and-interface-design | Lifecycle BUILD: cambios en tipos públicos (`cursor`, métricas) evaluados por compat |

**Cargadas:** systematic-debugging (bug → clasificación 16 errores) · test-driven-development · source-driven-development · incremental-implementation · context-engineering · api-and-interface-design. (`SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design + systematic-debugging (bug)`. SKILLS_CARGADAS se declara en RESULTADO.)

## 6. HERRAMIENTAS + MCP

- `npx tsc --noEmit` en `vantadb-ts/` (contrato a; reproducido: 16 errores)
- `npm test` (= `vitest run`) en `vantadb-ts/` (contrato b)
- `campaign_verify_cmd` (contrato; **bug exit -1 conocido → bash directa + mención en RESULTADO**)
- codegraph: N/A (TS-only, vale tsc + `rg`; sin `.codegraph` para este slice no se necesita)
- Cargo: N/A (cero Rust). Internet: N/A (todo local; Notion Paso 0c N/A — sin tool Notion disponible + tarea local tipos-only).
- Staging selectivo: `git add vantadb-ts/src/types.ts vantadb-ts/src/vantadb.ts` (+ este task file si aplica). NO PUSH (solo vanta-lead).

## 7. INVESTIGACIÓN CÓDIGO — blast radius: los 16 errores clasificados, fuente de verdad por tipo

**Hallazgo central:** en 12/16 errores el drift está en el `.d.ts` hand-written del binding (`pkg/vantadb_wasm.d.ts`, fuente `vantadb-wasm/src/vantadb_wasm.d.ts`), NO en `types.ts` ni en el código de `vantadb.ts`. El runtime Rust manda y `vantadb.ts` ya lo trata correctamente — por eso el fix es casts de frontera (cero runtime). En 3/16 el drift está en `types.ts` (cursor + métricas). Detalle:

| # | Tipo | Fuente de verdad (código real) | Drift | Fix tipos-only |
|---|------|-------------------------------|-------|----------------|
| 1 | `Capabilities.runtime_profile` | core `Capabilities{runtime_profile: RuntimeProfile}` (`types.rs:234`) serializado por `capabilities()` (`lib.rs:1123-1127`); `RuntimeProfile` es enum→string (`types.rs:61-68`); nativo SÍ lo expone (`native.ts:212` compila) | `.d.ts` inventó `backend: string` y omitió `runtime_profile` | cast `raw as unknown as Capabilities` (395) |
| 2 | `MemoryListPage/ListOptions` | wasm: cursor acepta string\|number (`deserialize_cursor`, `lib.rs:200-220`), `next_cursor` sale como decimal string (`next_cursor_to_js:224-229`); nativo: `cursor?: number, next_cursor?: number` (`index.d.ts:74-83`) | `types.ts` solo `number` (rompe wasm) | `cursor?: string \| number`, `next_cursor?: string \| number` (ambos backends) |
| 3-6 | `SearchRequestInput` | `from_js::<SearchRequest>` (`lib.rs:152-168`): `filters` = tagged `MemoryMetadata`, `text_query: Option<String>` (null=None); builder TS ya emite eso | `.d.ts`: `filters?: Record<string, MetadataValue>` (JSON-shape, mal), `text_query?: string` (sin null) | `_buildSearchRequest` retorna `SearchRequestInput` (import type) con 1 cast interno |
| 6b | `explainSearch` return | runtime = struct explanation (`to_js`, `lib.rs:1375-1379`); SDK declara `Record<string, unknown>` | firma SDK más ancha que el struct | cast resultado `as unknown as Record<string, unknown>` |
| 7-8 | `ExportReport` | `to_js(&report)` del **core** `{records_exported, namespaces, path, duration_ms}` (`record.rs:179-188`) | `.d.ts` `{output_path, bytes_written}` fictional; `types.ts` CORRECTO | cast ×2 |
| 9 | `ImportReport` | core `{inserted, updated, skipped, errors, duration_ms}` (`record.rs:192-203`) | `.d.ts` `{records_imported, errors[]}` fictional; `types.ts` CORRECTO | cast ×1 |
| 10 | `OperationalMetrics` | `JsOperationalMetrics` (`lib.rs:269-318`): todo `String`, `mmap/jemalloc: Option→undefined`, +`nan_sanitization_count, metadata_drop_count` | `types.ts`: `string \| null` requeridos, sin los 2 campos | opcionales `?: string` + 2 campos `string` |
| 11 | `query` | core `QueryResult` externally-tagged `{Read, Write, StaleContext}` (`graph.rs:14-32`, `to_js` directo `lib.rs:1675-1679`); ejemplo JSDoc usa `result.Read` | `.d.ts` `IqlResult {kind...}` fictional; `types.ts` CORRECTO | cast ×1 |
| 12 | `NodeRecord` | `JsNodeRecord` (`lib.rs:234-266`): `fields` tagged `Value`, edges core con `target`, `tier: StorageTier` = solo `Hot\|Cold` (`types.rs:72-76`) | `.d.ts`: `target_id`, `"Warm"` fictional; `types.ts` CORRECTO | `as unknown as NodeRecord` ×1 (mapeo `target→BigInt` existente intacto) |
| 13-16 | graph `bigint[]` | `to_js(&Vec<u128>)` → `BigInt[]` (`lib.rs:1790-1837,1886-1890`); **probado por `tests/graph.test.ts` TS-01** (`bigint[]`, orden BFS, sin campos ficticios) | `.d.ts` `string[]`; `types.ts` CORRECTO | cast ×4 |

**NOTICED BUT NOT TOUCHING (fuera de scope, sin FIND por falta de evidencia de fallo):** (a) `ExportReport`/`ImportReport` core usan `u64` → en runtime via `to_js` serían `bigint`, pero `types.ts` declara `number` — éxito de export/import nunca observado en tests (portability solo cubre el throw por falta de `std::fs` en WASM); cambiar `number→bigint` sería breaking público y excede los 16 errores. (b) `QueryResult.Write.node_id`/`StaleContext.node_id`: runtime `u128→BigInt`, `types.ts` dice `string` — mismo caso. Se dejan como están (scope discipline); el orquestador decide si abren FIND.

## 8. INVESTIGACIÓN PROBLEMA

¿Tipos core también drifted o solo TS desactualizado? **Mixto, resuelto a favor del runtime Rust en cada caso:** core (`src/sdk/`) es consistente y correcto; el drift vive (i) en el `.d.ts` hand-written del binding (12/16: `backend`, `output_path/bytes_written`, `records_imported/errors[]`, `IqlResult{kind}`, `target_id/Warm`, `string[]` en grafos, `MetadataValue`-filters, `mmap string|null` ausente) y (ii) en `types.ts` (3/16: cursor/next_cursor `number`-only, métricas `string\|null` + 2 campos faltantes). `vantadb.ts` no tiene bugs de runtime — solo fricción de tipos contra el `.d.ts`. Verificado contra `lib.rs` + core, no contra docs.

## 9. INVESTIGACIÓN INTERNET

N/A (todo local; sin ambigüedad de APIs externas).

## 10. VALIDACIÓN + CIERRE

- Verify contrato: `npx tsc --noEmit` 0 + `npm test` verde + `git diff --stat` (solo `types.ts`, `vantadb.ts`).
- OCR delegation: N/A-justificado si es tipos-only (advisory sin API key; se intenta `pwsh dev-tools/ocr-review.ps1`, si no aplica se registra).
- DoD 3 niveles: (1) Correctness — AC (a)(b)(c); (2) Quality — diff mínimo, sin `any` (solo `unknown`), scope limpio; (3) Ship — commit `fix:` selectivo, sin push, recitation + RESULTADO §7.
- P2-01 (vanta-review) lo hace el orquestador (no yo). Gates D/V/C: D no disparado (§3); V/C se evalúan en cierre (2 fallas mismo-error → Gate V; colaterales → Gate C; sin `question` en toolbox → STOP + reporte si disparan).

## Steps atómicos

- [x] **Step 0 — DISCOVERY** (este file): reproducir tsc, clasificar 16 errores, fuente de verdad, task file. Verify: `npx tsc --noEmit` muestra 16 errores (RED).
- [x] **Step 1 — `types.ts` resync** (~15 líneas): `cursor`/`next_cursor` → `string | number`, métricas mmap/jemalloc → `?: string` + `nan_sanitization_count/metadata_drop_count: string`, +1 cast en `native.ts:312` (exigido por el ensanchamiento; borrado, cero runtime). Verify: tsc 16→14 (546 y 1090 fuera), resto idénticos + 1 nuevo en `native.ts` resuelto en el acto.
- [x] **Step 2 — `vantadb.ts` casts de frontera** (~15 líneas + 1 import type): `SearchRequestInput` import + return type de `_buildSearchRequest` (1 cast); casts en capabilities/explain/export×2/import/query/node/graph×4. Verify: **tsc 0 (exit 0)**.
- [x] **Step 3 — tests + cierre**: `npm test` **12 files / 311 tests passed**; `git diff --stat` limpio (3 archivos, 66+/26-); commit `fix: FIND-125 — ...`; recitation completed; RESULTADO §7.

## Context Save Point
- Rama `develop`; tsc repro: 16 errores (output guardado arriba §1).
- Tras Step 1: esperar 14 errores (quedan 395, 603, 657, 801, 802, 829, 870, 938, 1108, 1181, 1322, 1346, 1366, 1413).
- Tras Step 2: esperar 0 errores.
- `native.ts` debe seguir verde en ambos steps (mismo `tsc --noEmit` lo cubre).
- Si un cast revela shape runtime distinta al correr tests → STOP (contrato c) y re-scope, no "arreglar" runtime.

## P2-01 follow-up (lead, 2026-09-19 — veredicto approve)

- Conteo frontera corregido: **12× `as unknown as` en `vantadb.ts` + 1 narrow en `native.ts:312`** (total 13 casts de frontera, cero runtime). Cualquier mención a "14 casts" en reportes previos queda superada por este conteo verificado (`rg` en vivo).
