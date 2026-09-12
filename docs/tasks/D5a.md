# D5a — Endurecer validación en frontera TS

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: frontera/binding TS (validación de input, trust boundary JS→Rust).
- Alcance: `vantadb-ts/src/metadata.ts:38-50` (`normalizeValue`), `vantadb-ts/src/guards.ts:154-165` (`buildSearchRequestBase`), `vantadb-ts/src/vantadb.ts:565-578` (`_buildSearchRequest` glue).
- Patrón a seguir: `vantadb-ts/src/native.ts:94-104` (lanza `DbError` con `ERROR_CODES.VALIDATION_ERROR` ante forma taggeada irreconocible / tipo no soportado).
- Blast radius: `normalizeMetadata` / `normalizeFilterItems` (usan `normalizeValue`); `Client.search` / `NativeVantaDB.search` (usan `buildSearchRequestBase`); tests `src/__tests__/hardening.test.ts`, `flat-metadata.test.ts`, `vanta.test.ts`.
- Baseline: sin `/cleanCA` mecánico disponible en este entorno; baseline = tests vitest + `tsc --noEmit` + `eslint` según `vantadb-ts/package.json`.

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia:
  - `vantadb-ts/src/metadata.ts`: `normalizeValue` valida forma taggeada (objeto de 1 clave conocida con payload del tipo correcto, números finitos) y lanza `DbError(VALIDATION_ERROR)` ante `undefined`, funciones/símbolos, arrays, objetos malformados, `NaN`/`Infinity`; importa `DbError`/`ERROR_CODES` + `isValidValue`-local estricto. Sin `any`.
  - `vantadb-ts/src/guards.ts`: `buildSearchRequestBase` valida `namespace` (string no vacío), `query_vector` (array no vacío de números finitos), `top_k` (default 10, entero ≥0 finito), `distance_metric` (`Cosine`|`Euclidean`), `explain` (boolean); lanza `DbError(VALIDATION_ERROR)` con prefijo `buildSearchRequestBase:`. Sin `any`.
  - `vantadb-ts/src/__tests__/d5a-validation.test.ts` (nuevo): tests vitest RED→GREEN de ambas validaciones.
- NO cambia:
  - `vantadb-ts/src/vantadb.ts:1094+` (D5d, prohibido) ni resto de binds (`native.ts`, `errors.ts`, `types.ts`).
  - Semántica happy-path: valores planos y formas taggeadas válidas siguen pasando igual; `top_k ?? 10`, `distance_metric ?? Cosine`, `explain ?? false` preservados.
  - Mensajes snapshot de `_mapRecord` intactos.
- Archivos exactos: los 3 de arriba (lectura: `types.ts`, `errors.ts`, `native.ts:63-108`, `vantadb.ts:565-578`).
- Verify (bash, sin campaign MCP):
  - `npm test` (vitest run) en `vantadb-ts/`
  - `npx tsc --noEmit` en `vantadb-ts/`
  - `npm run lint` en `vantadb-ts/`

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Step 1 — Slice 1 `normalizeValue` estricta: test RED (11 fallan por razón correcta) → GREEN en `metadata.ts` → focado 16/16 + `tsc` limpio. ✅ 2026-09-12
- [x] Step 2 — Slice 2 `buildSearchRequestBase` valida: RED→GREEN en `guards.ts`; colateral mínimo `searchMulti` (`namespace:""` sintético → `namespaces[0]` + valida array no vacío) para no romper `integration.test.ts`. ✅ 2026-09-12
- [x] Step 3 — Cierre: `npm test` 296/296 (11 files) + `tsc --noEmit` limpio + `eslint` limpio; 1 expectativa migrada (`hardening.test.ts` "empty vector" → `DbError/VALIDATION_ERROR`, documentada); sin `any` (solo mención en comentario); `vantadb.ts` solo hunk @@-640 (D5d intacto); sin commit (lead). ✅ 2026-09-12

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO con ✅/🟡/❌, STEPS_OK, PROXIMO_STEP, COMMIT_HASH ninguno, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS ≥10.
- `/cleanCA vantadb-ts/src/metadata.ts vantadb-ts/src/guards.ts`: E1/A2 sin hallazgo nuevo provocado (validación en frontera = pago de deuda).
- Recitation: objetivo D5a, estado, última acción, contrato + outputs reales, invariantes (happy-path intacto, sin `any`, sin tocar D5d), deuda (ninguna o slices ☐ restantes).
