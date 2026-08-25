# FIND-10: TS packaging ESM-only + errores genéricos (WASM_ERROR indistinguible)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md
- **Fuente:** plan file Task 8 (Wave 3)
- **Esfuerzo:** 🟡 3h
- **Prioridad:** 🟡
- **Tipo:** TypeScript/WASM bindings (DX packaging + error discrimination)
- **Turns estimados:** 8
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación worker; commit + verify del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3 (paso wasm → paso TS → verify)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts/src/vantadb.ts` (7 callers de `wrapWasmError`), `vantadb-ts/src/native.ts` (wrapNativeError, NO tocar), tests `vantadb-ts/src/__tests__/hardening.test.ts` (describe wrapWasmError + integración) |
| Callees | `vantadb-wasm/pkg/` (artefacto regenerado, git-ignored) → `vantadb-wasm/src/lib.rs::to_js_err` (choke point de errores); `src/error.rs` (enum VantaError, variants mapeadas); `vantadb-wasm` (crate, dependencia ESM-only) |
| Implicaciones | API pública TS NO cambia (firmas/mensajes intactos). `VantaError.code` gana valores nuevos (CORRUPT/RESOURCE_LIMIT/etc.) — aditivo. exports map gana condición `require` → ESM consumers intactos, CJS `require("vantadb")` pasa a funcionar en Node ≥22.12 (require(esm)). Mensajes de error wasm byte-idénticos (no rompe tests existentes que matchean texto). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-ts/package.json` (61L), `vantadb-ts/src/errors.ts` (vía codegraph, verbatim 57L), `vantadb-ts/src/vantadb.ts` (1256L — header 1-170 + métodos _wasm/connect/open/put/get/search/etc. vía codegraph), `vantadb-ts/src/native.ts` (310L, NO se toca), `vantadb-ts/src/__tests__/hardening.test.ts` (470L), `vantadb-ts/tsconfig.json`, `vantadb-ts/vitest.config.ts`, `vantadb-wasm/src/lib.rs` (to_js_err :1553-1555 + open/connect :381-460 + struct VantaDB), `vantadb-wasm/pkg/package.json` (ESM-only), `vantadb-wasm/pkg/vantadb_wasm.js` (generado, sin TLA), `src/error.rs` (enum VantaError completo, vía codegraph), `src/sdk/search/mod.rs:125-154` (ERR-028 → InvalidInput), `.opencode/rules/js-ecosystem.md` (R-1/R-2), docs oficiales Node (esm.html#require + packages.html#conditional-exports)
- **Archivos referenciados hacia dentro:** `vantadb-ts/src/errors.ts` — `wrapWasmError` (7 callers en vantadb.ts), `VantaError` (12 callers), `ErrorCode` type. `vantadb-wasm/src/lib.rs::to_js_err` — todos los métodos wasm-bindgen del crate (60+ callers). `vantadb-ts/package.json` — consumed por npm consumers, vitest, tsc, release-npm-61.yml.
- **Archivos que referencian a los editados:** `vantadb-ts/src/vantadb.ts` (importa de ./errors.js), tests hardening/vanta/subclients (importan VantaError/wrapWasmError), `vantadb-ts/package-lock.json` (package.json metadata — engines/exports no requieren lock update), CI `release-npm-61.yml` (wasm-pack build --release + npm test).
- **Veredicto impacto:** **bajo-medio** — cambios aditivos en binding TS + 1 función en binding wasm (to_js_err). No toca core SDK (src/), no toca API pública TS, no toca hot paths, no introduce deps nuevas. El rebuild de `vantadb-wasm/pkg` es artefacto git-ignored (R-2) regenerado por CI con `wasm-pack build --release`.

## Contrato
`npm run build` (vantadb-ts) exit 0; `require("vantadb")` funciona (condición `require` en exports → require(esm), Node ≥22.12) O decisión documentada — se implementa el exports + `engines` + README; errores distinguen corrupto/not-found/validación vía `code` estructural del wasm (Reflect::set en `to_js_err`) + fallback por prefijo de mensaje en TS. Tests vitest nuevos verdes.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) mensajes de error wasm byte-idénticos (`e.to_string()` intacto — tests existentes matchean texto: hardening `/zero-norm|undefined|InvalidInput/i`, `toThrow(VantaError)`); (2) API pública TS síncrona intacta (no convertir a async); (3) `VantaError` pasa-through en wrapWasmError; (4) import consumers ESM intactos (condición `import` no cambia); (5) sin `expect`/`unwrap` en código nuevo (constraint worker); (6) pkg/dist son artefactos regenerados (R-2) — nunca editarlos a mano.
- **Comandos de verificación:** `npm run build` (vantadb-ts) ✅ · `npm test` (vitest) ✅ · `npx tsc --noEmit` ✅ · `node -e "require('vantadb')"` (desde vantadb-ts, self-reference) ✅ · `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅
- **Deuda pendiente:** wasm32 `wasm-pack build --dev` regenera pkg localmente (CI lo regenera con --release). El README tiene ejemplos con `await db.put()` (stale vs API sync) — hallazgo colateral ruteado a FIND-06 (docs).

## Recitation (canónico — estructura única)

- `activeGoal`: FIND-10 — fix DX TS: `require("vantadb")` + errores distinguibles (corrupto/not-found/validación).
- `lastAction`: DISCOVERY completo — codegraph + reglas JS + docs oficiales Node; decisión de diseño tomada; task file creado con Regla 0 mapeada.
- `result`: `PARTIAL` (en ejecución)
- `nextAction`: Step 1 — `vanta_error_code()` + `to_js_err` con `code` en `vantadb-wasm/src/lib.rs`; rebuild pkg.
- `contract`:
  - `verificacion`: `npm run build` (ts) exit 0 + `npm test` verde + `require("vantadb")` OK + errores distinguen (test unit + integración real wasm)
  - `evidencia`:
    - claim: `require('vantadb')` falla hoy — ERR_PACKAGE_PATH_NOT_EXPORTED (exports sin condición `require`; resolución CJS no usa `import`)
      evidencia: node -e require('vantadb') en vantadb-ts (Node 24.16.0) + docs nodejs.org/api/packages.html#conditional-exports
      confianza: alta
    - claim: require(esm) del grafo síncrono funciona (sin TLA) — `require('./dist/vantadb.js')` OK en Node 24; `vantadb_wasm.js` no tiene top-level await
      evidencia: node -e require('./dist/vantadb.js') + nodejs.org/api/esm.html#require
      confianza: alta
    - claim: wasm aplana VantaError a string — `to_js_err` = `js_sys::Error::new(&e.to_string())`, sin discriminante
      evidencia: vantadb-wasm/src/lib.rs:1553-1555
      confianza: alta
    - claim: variantes VantaError mapeables a las 3 clases del contrato (NotFound/ValidationError/InvalidInput/DimensionMismatch → validación; IncompatibleFormat/WALVersionMismatch/SerializationError/SchemaError → corrupto; NodeNotFound/NotFound → not-found)
      evidencia: src/error.rs:91-266 (enum VantaError)
      confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/FIND-10.md`
  - `invariantes`: mensajes wasm intactos; API sync intacta; pkg/dist regenerados
  - `deuda`: ninguna (diferida: dual build CJS real con tsup — ver Notas)
  - `queda_pendiente`: lead verifica mecánico y commitea (NO COMMIT del worker); CI regenera pkg con --release
- `nextTask`: MCP-33 (uphill) o la que el lead asigne.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Negativo. Se elimina deuda DX real (require roto + errores indistinguibles) a costo de ~40 líneas. No introduce unsafe, clones ni deps nuevas. La clasificación por prefijo de mensaje es fallback de defensa (depende de strings Display del core estables) — marcada con `ponytail:`. Dual build CJS real (tsup) sería over-engineering hoy: el grafo wasm es ESM-only y la API es síncrona (ver Notas).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| Task | Contrato verifica: build ts exit 0 + require() OK + error classification tests verdes |
| Commit | Lo ejecuta el lead (NO COMMIT del worker). Conventional `fix:` (DX TS packaging + errores) |
| Release | No aplica (worker no release; CI regenera pkg con wasm-pack --release) |

## Herramientas necesarias
- node/npm (build, test, require), wasm-pack 0.15.0 (rebuild pkg), cargo (check wasm target), codegraph_explore

## Investigation Notes
- `vantadb-wasm/pkg` es ESM-only (`"type": "module"`, generado wasm-pack bundler). La API TS es 100% síncrona (connect/put/get/search sync). Un dual build CJS real (tsup) emitiría `require("vantadb-wasm")` → ERR_REQUIRE_ESM en el grafo wasm → rompe el build en runtime. Alternativa async (dynamic import) rompería la API sync → breaking. **Decisión: ESM-only + condición `require` al mismo archivo ESM → require(esm) funciona en Node ≥22.12 (grafo sin TLA, verificado empíricamente).** `engines: node >=22.12` documenta el floor. Docs: nodejs.org/api/packages.html#conditional-exports ("Expected formats include ... ES modules" para la condición require) + esm.html#require ("only supports loading synchronous ES modules").
- Errores: `to_js_err` es el único choke point del crate wasm (60+ callers). Adjuntar `err.code` vía `js_sys::Reflect::set` mantiene el mensaje intacto (cero breakage de tests que matchean texto) y da discriminante estructural. `wrapWasmError` lee `e.code` si es un código conocido; si no (pkg viejo), clasifica por prefijo del mensaje Display de VantaError; si no, WASM_ERROR.
- ERR-028 (zero-norm cosine) → `VantaError::InvalidInput` → VALIDATION_ERROR. Test de integración determinístico disponible.
- vitest externaliza `vantadb-wasm` (Node nativo) — los tests corren contra el pkg rebuild local.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — NO aplica:** no toca trust boundaries, auth, input de usuario (validaciones del core intactas), ni dependencias (cero deps nuevas). FFI wasm: el cambio adjunta una propiedad `code` a un objeto Error que ya se creaba — no introduce unsafe ni punteros.
- [x] **PERFORMANCE — NO aplica:** `wrapWasmError` corre solo en el path de error (no hot path). `to_js_err` idem. Un `Reflect::set` por error es despreciable. Regla 9 no aplica (no es optimización).

## Context Save Point (2026-08-25)

**Estado de implementación:** COMPLETO. Todos los checks verdes. NO COMMIT (worker — lo ejecuta el lead).

**Verificación mecánica obtenida:**
- `npm run build` (vantadb-ts) ✅ exit 0
- `npm test` (vitest) ✅ 261/261 (8 files, incluye 8 tests FIND-10 nuevos)
- `npx tsc --noEmit` ✅ exit 0
- `node -e "require('vantadb')"` ✅ — self-reference desde vantadb-ts, put/get roundtrip OK (Node 24.16, require(esm))
- `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ · `cargo check -p vantadb-wasm` (host) ✅ · `cargo clippy -p vantadb-wasm --all-targets -- -D warnings` ✅ · `rustfmt --check vantadb-wasm/src/lib.rs` ✅
- Probe real wasm: `get('', 'k')` → `code: VALIDATION_ERROR`; zero-norm search → `VALIDATION_ERROR`; `add_edge` nodos faltantes → `NOT_FOUND` (discriminante `err.code` presente en pkg release)

**Hallazgo de proceso:** `wasm-pack build --dev` produce un wasm que trapea `unreachable` en put/close (debug asserts del core) — las pruebas TS deben usar pkg `--release` (igual que CI). El build `--dev` NO es apto para correr la suite vitest.

**Archivos tocados (solo estos, 6):** `vantadb-wasm/src/lib.rs` (+37 to_js_err/vanta_error_code), `vantadb-ts/src/errors.ts` (códigos + classifyWasmError + wrapWasmError), `vantadb-ts/package.json` (exports require + engines), `vantadb-ts/README.md` (Module Formats + Errors + Runtimes), `vantadb-ts/src/__tests__/hardening.test.ts` (+8 tests), `.opencode/skills/campaign-executor/tasks/FIND-10.md`. pkg/ y dist/ regenerados (git-ignored). Otros archivos en `git status` (AGENTS.md, Backlog.md, DESKTOP-*, reviews, lessons.md) son de workers concurrentes / lead — NO los toca este worker.

**Próximo paso (lead):** `git add vantadb-wasm/src/lib.rs vantadb-ts/package.json vantadb-ts/src/errors.ts vantadb-ts/README.md vantadb-ts/src/__tests__/hardening.test.ts .opencode/skills/campaign-executor/tasks/FIND-10.md` + commit `fix: FIND-10 — TS require(esm) support + distinguishable VantaError codes`. Rebuild pkg con --release en CI (ya hecho localmente).

## Steps

### Step 1: Discriminante estructural en el wasm (to_js_err + vanta_error_code)
- **Archivos:** `vantadb-wasm/src/lib.rs`
- **Acción:** agregar `fn vanta_error_code(e: &VantaError) -> &'static str` (match sobre variants → NOT_FOUND/VALIDATION_ERROR/CORRUPT/RESOURCE_LIMIT/TIMEOUT/BUSY/IO_ERROR/WASM_ERROR) + `to_js_err` adjunta `code` vía `js_sys::Reflect::set` (sin `expect`, degradación silenciosa). Mensaje intacto.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ + `cargo clippy -p vantadb-wasm --all-targets -- -D warnings` ✅ + `rustfmt --check` ✅
- **Estado:** ✅ COMPLETED

### Step 2: wrapWasmError clasifica (errors.ts)
- **Archivos:** `vantadb-ts/src/errors.ts`
- **Acción:** ERROR_CODES += CORRUPT/RESOURCE_LIMIT/TIMEOUT/BUSY/IO_ERROR; `classifyWasmError(message)` (regex prefijos Display VantaError); `wrapWasmError` lee `e.code` conocido (prioridad) → fallback `classifyWasmError` → WASM_ERROR. Clase VantaError preservada.
- **Verify:** `npx tsc --noEmit` ✅ + tests unit FIND-10 ✅
- **Estado:** ✅ COMPLETED

### Step 3: package.json — condición require + engines
- **Archivos:** `vantadb-ts/package.json`
- **Acción:** exports "." y "./types" → `{ types, import, require }` (require apunta al mismo ESM); `"engines": { "node": ">=22.12" }`.
- **Verify:** `node -e "require('vantadb')"` self-reference OK ✅; `npm run build` OK ✅
- **Estado:** ✅ COMPLETED

### Step 4: README — sección CommonJS + códigos de error
- **Archivos:** `vantadb-ts/README.md`
- **Acción:** sección "Module Formats (ESM / CommonJS)" (require(esm) Node ≥22.12, <22.12 → import o vantadb-node), sección "Errors" con tabla de códigos + ejemplo, tabla Runtimes actualizada (Node 22.12+).
- **Verify:** contenido revisado (sin gate mecánico)
- **Estado:** ✅ COMPLETED

### Step 5: Tests (RED→GREEN)
- **Archivos:** `vantadb-ts/src/__tests__/hardening.test.ts`
- **Acción:** 6 tests unit wrapWasmError/classifyWasmError (code prop prioridad, unknown code → WASM_ERROR, fallback por prefijo NOT_FOUND/VALIDATION/CORRUPT) + 2 tests integración real wasm (zero-norm → VALIDATION_ERROR; addEdge nodos faltantes → NOT_FOUND).
- **Verify:** `npm test` ✅ 261/261 (8 files) con pkg --release; `npx vitest run hardening -t FIND-10` → 8 passed
- **Estado:** ✅ COMPLETED

### Step 6: Verify full + cierre
- **Archivos:** — (verify) + task file
- **Acción:** `npm run build` ✅ + `npm test` ✅ + `npx tsc --noEmit` ✅ + `require('vantadb')` empírico ✅ + task file actualizado (Context Save Point + RESULTADO). NO commit (lead).
- **Verify:** contrato completo ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 3). Requiere rebuild local de `vantadb-wasm/pkg` (git-ignored) para que vitest tome el `code`.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador — delegado por el lead al cierre (NO COMMIT del worker).

- **Revisor:** vanta-review (designado por el lead)
- **Enfoque:** ¿la condición `require` apuntando a ESM + engines ≥22.12 es la decisión correcta vs dual build tsup? ¿la clasificación por prefijo de mensaje es aceptable como fallback? ¿Reflect::set en to_js_err sin expect es correcto?
- **Cómo se probó:** evidencia mecánica en Steps 1-6 (cargo check wasm, wasm-pack build, tsc, vitest, require empírico).
- **Veredicto:** pendiente

## Notas
- **Dual build CJS (tsup) — descartado documentado:** el grafo `vantadb-wasm` es ESM-only; un build CJS del wrapper `require("vantadb-wasm")` falla en Node <22.12 (ERR_REQUIRE_ESM) y rompería la API síncrona si se resolviera con dynamic import. El camino oficial (docs Node) es condición `require` → ESM (require(esm) para módulos síncronos). Upgrade path si se necesita CJS real: rebuild wasm con `--target nodejs` (genera CJS) + tsup dual — trabajo futuro, no AHORA (YAGNI).
- **Hallazgo colateral:** README de vantadb-ts muestra `await db.put()` pero la API es síncrona — ruteado a FIND-06 (docs translation task, mismo plan).
- **Node floor:** require(esm) unflagged desde 22.12.0/20.19.0/18.20.6; import de módulos wasm sin flag desde 22.19.0/24.5.0. `engines >=22.12` es el floor documentado; README nota el requisito wasm (22.19+ recomendado).