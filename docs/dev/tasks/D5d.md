# D5d — Simetrizar DTO id `number|bigint` en get/delete/addEdge (+removeEdge)

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline)
- Tipo: frontera/binding TS (DTO público, trust boundary JS→WASM).
- Alcance: `vantadb-ts/src/vantadb.ts:1146` (`getNode`), `:1174` (`deleteNode`), `:1193` (`addEdge`), `:1226` (`removeEdge`) + interfaz `GraphClient` `:76-85` + delegados `graph` `:288-299`. `insertNode` `:1107` ya acepta `number|bigint` con guard safe-integer (`:1114-1119`).
- Wire verificado (no asumido): `vantadb-wasm/src/lib.rs:1714,1723,1732,1754` reciben `id: &str`; `parse_node_id` `:2290` parsea `u128` ("decimal u128 string"). `String(bigint)` es exacto → el wire NO impone 2⁵³. Decisión: **simetrizar** (opción fix, no doc).
- Blast radius: iface `GraphClient` + 4 delegados `graph` (cambio mecánico requerido por `strictFunctionTypes`); `getNode` ya mapea `edges[].target` string→bigint (`:1152-1157`); tests `src/__tests__/vanta.test.ts:584-640`, `hardening.test.ts`, `subclients.test.ts`, `integration.test.ts`, `tests/graph.test.ts`.
- NOTICED BUT NOT TOUCHING: `graphBfs/graphDfs/roots: number[]` (misma asimetría, fuera del scope "los 3" → follow-up, no este slice).
- Baseline: `npm test` (vitest run) + `npx tsc --noEmit` + `npm run lint` en `vantadb-ts/` (sin campaign MCP, adaptador §10).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia:
  - `vantadb-ts/src/vantadb.ts`: 4 firmas `Client` + 4 miembros `GraphClient` + 4 delegados `graph`: `number` → `number | bigint`; mismo guard safe-integer que `insertNode` (throw `DbError INVALID_ARGUMENT` ante `number` no-safe, guía a bigint); JSDoc `@param` actualizado ("number or bigint").
  - `vantadb-ts/tests/bigint-ids.test.ts` (nuevo): RED→GREEN — round-trip bigint >2⁵³ en get/delete/addEdge/removeEdge + throw ante number inseguro + delegados `db.graph.*` con bigint.
- NO cambia:
  - `guards.ts`/`metadata.ts` (D5a, prohibido salvo test común ya existente — no se toca).
  - `graphBfs/roots` y resto de binds (`native.ts`, `errors.ts`, `types.ts`).
  - Semántica happy-path: `String(id)` idéntico; numbers safe pasan igual.
- Archivos exactos: los 2 de arriba (lectura: `vantadb-wasm/src/lib.rs:1714-1762,2290-2296`, `src/__tests__/vanta.test.ts:584-640`).
- Verify (bash, sin campaign MCP, en `vantadb-ts/`):
  - `npx tsc --noEmit`
  - `npm test` (vitest run)
  - `npm run lint`

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Step 1 — RED: test `bigint-ids.test.ts` con bigint >2⁵³; `tsc` 12× TS2345 (bigint no asignable a number) = razón correcta; vitest 2 failed (guards ausentes) | 5 passed. ✅ 2026-09-12
- [x] Step 2 — GREEN: 6 grupos de edición en `vantadb.ts` (iface + delegados + 4 métodos con guard safe-integer + JSDoc); `tsc` limpio + vitest 7/7 + eslint limpio (literales inseguros centralizados en `UNSAFE_NUMBER` con disable justificado). ✅ 2026-09-12
- [ ] Step 3 — Cierre: RESULTADO + recitation; sin commit (lo ejecuta el lead).

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO con ✅/🟡/❌, STEPS_OK, PROXIMO_STEP, COMMIT_HASH ninguno, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS ≥10.
- Sin `/cleanCA` mecánico en este entorno; cierre = vitest + `tsc --noEmit` + `eslint` verdes y DTO simétrico documentado en JSDoc.
- Recitation: objetivo D5d, estado, última acción, contrato + outputs reales, invariantes (D5a intacto, happy-path intacto, sin `any`), deuda (roots `number[]` follow-up).
