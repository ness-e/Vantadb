# MOD-24 — Nits TS agrupados (dedup + guard + JSDoc)

- **Estado:** ✅ COMPLETO (verify full verde, sin commit — vanta-lead commitea)
- **Plan:** `docs/plans/2026-09-07-backlog-triage.md` (Task 2, Wave0)
- **Branch:** develop (sin commit — solo vanta-lead)
- **Appetite:** max 1d · **Esfuerzo:** 🟡 4-6h
- **Archivos clave:** `vantadb-ts/src/guards.ts`, `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`
- **Contrato:** `npm run build && npx vitest run && npx eslint .` en `vantadb-ts/` → 0 errors; `grep -rn "function _mapRecord" vantadb-ts/src --include=*.ts` → 1 definición
- **SDP:** code-simplification (dedup, pedida) + base auto (campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design) · keywords: dedup/maprecord/buildsearchrequest/validatevector/jsdoc · ponytail full siempre
- **SKILLS_CARGADAS:** code-simplification, ponytail(full); contexto: SDP v2 lifecycle BUILD

## Gate D — evaluado, NO disparado

Blast radius = 3 archivos src + 1 test, sin hot path, sin cambio wire, sin símbolos públicos nuevos (`_mapRecord`/`buildSearchRequestBase` internos a `guards.ts`, no re-exportados en el índice), contrato mecánico. → Sin `question`, ejecución directa.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `guards.ts` (112L), `native.ts` (380L), `vantadb.ts` (1422L) + `types.ts:60-110` + `vanta.test.ts:200-238` + `vantadb-node/index.d.ts:87-95`.
- **Referencias hacia dentro (lo que toco depende de):** `errors.js` (VantaError/ERROR_CODES), `types.js` (MemoryRecord/SearchRequest/SearchHit), `metadata.js` (normalizeMetadata / normalizeMetadataForNative en cada backend).
- **Referencias entrantes (quién usa lo que toco):**
  - `_mapRecord` ×2 defs idénticas (`vantadb.ts:134`, `native.ts:110`); 0 importadores externos — solo métodos `put/putBatch/get/list/search*` del propio archivo. Mensajes de error con snapshot: `"expected an object, got X"`, `"invalid MemoryRecord structure..."` — se preservan byte-idénticos.
  - `validateVector` (`guards.ts:91`): solo tests (`vanta.test.ts:217-236`) + re-export índice. **Type-lie confirmado:** firma `asserts v is Float32Array` pero `Array.isArray(v)` rechaza Float32Array reales y acepta `number[]` mintiendo al compilador.
  - `_buildSearchRequest` ×2 (`vantadb.ts:581` → `Record<string,unknown>` con `filters ?? {}`, `text_query ?? null`, `exclude_superseded`; `native.ts:342` → `NativeSearchRequest` con `filters|undefined`, `text_query ?? undefined`, sin `exclude_superseded`). Wire distinto por backend → **NO unificar wire**, solo extraer base común.
  - `distance: h.score as number` en 4 sitios (wasm search/searchMulti/similarToKey, native search); `SearchHit.distance` ya documentado en `types.ts:86-88`.
- **Veredicto:** refactor interno puro, 0 cambio wire, 0 API pública nueva. Riesgo bajo; rollback = revert por step.

## Spec (cambios de comportamiento — solo 1)

| # | Decisión | Evidencia |
|---|----------|-----------|
| 1 | `validateVector` acepta `number[] \| Float32Array`, firma `asserts v is number[] \| Float32Array` | type-lie actual (código leído); tests exigen `number[]` sin throw (`vanta.test.ts:219`); pre-mortem plan: "mantener assert + overload documentado"; Float32Array = zero-copy downstream, number[] = copia |
| 2 | `_mapRecord` único en `guards.ts`, mensajes idénticos | diff textual ×2 idéntico; contrato exige 1 definición |
| 3 | `buildSearchRequestBase` extrae `{namespace, query_vector, top_k, distance_metric, explain}`; cada backend añade su `filters/text_query(+exclude_superseded wasm)` | wires difieren (null vs undefined, `{}` vs undefined) — unificar rompería deserializers |
| 4 | JSDoc: `validateVector` (number[] vs Float32Array + zero-copy), helpers nuevos, nota `score→distance` en `@returns` de search ×4 | plan: "JSDoc distance/score", scope R4#3-#10 |

## Steps atómicos

- [x] **Step 0 — DISCOVERY + baseline:** contexto, blast radius, `npm run build` ✅, `vitest` 278/278 ✅ (31s), `eslint` 0 ✅. Mensajes snapshot registrados arriba.
- [x] **Step 1 — RED:** añadidos 2 tests Float32Array en `vanta.test.ts`. Verify: focused vitest → 1 falla (`passes on Float32Array`), resto verde — RED por razón correcta (`Array.isArray` rechaza Float32Array reales).
- [x] **Step 2 — guards.ts (GREEN core):** `_mapRecord` único + JSDoc, `validateVector` acepta `number[] | Float32Array` + JSDoc, `buildSearchRequestBase` + tipo. Verify: build ✅ + focused verde.
- [x] **Step 3 — vantadb.ts:** borrado `_mapRecord` local, import de `guards.js`, `_buildSearchRequest` sobre base, notas JSDoc `score→distance` ×3. Verify: build ✅ + `vanta.test.ts` 107/107 ✅.
- [x] **Step 4 — native.ts + CIERRE:** borrado `_mapRecord` local, import de `guards.js`, `_buildSearchRequest` sobre base (wire intacto: `undefined`, sin `exclude_superseded`), nota JSDoc en search. Verify full: build ✅ + vitest 280/280 ✅ + eslint 0 ✅ + `function _mapRecord` → 1 ✅. Sin commit.

## Stop conditions

Diff >5 archivos tracked → parar (presupuesto: guards, vantadb, native, vanta.test = 4). Divergencia MOD-22/23 → DEFER. Wire nativo tocado accidentalmente → revert step.

## Context Save Point

Baseline 2026-09-07: build ✅ · vitest 278 ✅ · eslint 0 ✅. `node_modules` presente (sin `npm ci`). Tests corren con warnings experimentales WASM (ruido, ignorar). `NativeSearchRequest` = `vantadb-node/index.d.ts:87-95` (filters/text_query/top_k/distance_metric/explain opcionales, sin `exclude_superseded`).
