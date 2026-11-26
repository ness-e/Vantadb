---
title: "FIND-728: los pins de portabilidad de `vantadb-ts` afirman un mundo que ya no existe"
kind: task
status: completed
description: "4 rojos pre-existentes de `npm test` en vantadb-ts, destapados por el pre-push reparado en FIND-718: 3 pins afirmaban la limitación 'WASM has no std::fs' sobre ops que ya no tocan el filesystem (reindexHnswFromText, rebuildIndex) y el 4º tenía razón — la frontera WASM/TS reescribía el código canónico del core (VANTADB_VALIDATION_ERROR → VANTADB_INVALID_ARGUMENT)"
id: FIND-728
---

# FIND-728: los pins de portabilidad de `vantadb-ts` afirman un mundo que ya no existe

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b
- **Fuente:** gate `pre-push` (`.githooks/pre-push` → `npm test` en `vantadb-ts`), reparado en `FIND-718` (`fc08c4b35`). Los 4 rojos ya estaban anotados como incógnita en `FIND-156` §Incógnita 2 ("re-verificar tras aterrizar `FIND-710`"): esta tarea es esa re-verificación.
- **Esfuerzo:** 🟢 2-3 h (medición incluida)
- **Prioridad:** 🔴 Alta — bloquea cualquier push que toque `vantadb-ts/` y, con ello, el release v0.10.0
- **Tipo:** bug-fix / tests (binding TS + frontera de errores TS/WASM)
- **Carril:** R
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `npm test` (26 ficheros / 393 tests) y el hook `pre-push`. Nada más consume `portability.test.ts` / `hardening.test.ts` |
| Callees | `vantadb-ts/src/errors.ts` (`fromNativeError` ← `wrapWasmError`), el `#private` `_wasm` de `vantadb.ts`, el binding `vantadb-wasm/pkg` y, detrás, `src/sdk/api/admin.rs` + `src/sdk/search/mod.rs` |
| Implicaciones | `fromNativeError` es compartido por el SDK WASM y por `vantadb/native`: el cambio **solo se activa** si el error trae `raw_code` (lo emite `to_js_err` en WASM). El backend napi no lo emite → comportamiento intacto. Cambia la superficie pública de códigos TS al valor que ya documentan `docs/api/ERROR_HANDLING.md` §1.1 y `docs/api/TS_SDK.md` |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos o en la extensión necesaria:** `vantadb-ts/src/__tests__/portability.test.ts` (158 L, completo), `vantadb-ts/src/__tests__/hardening.test.ts` (L80-159 y L505-561), `vantadb-ts/src/errors.ts` (L1-120 y L150-314), `vantadb-ts/src/vantadb.ts` (L660-689, L739-754, L1189-1283 — `_buildSearchRequest`, `rebuildIndex`, `reindexHnswFromText`, `list`), `vantadb-ts/src/native.ts` (L1160-1170, `rebuildIndex`), `src/sdk/api/admin.rs` (163 L, completo: `rebuild_index` L19-33 y `reindex_hnsw_from_text` L45-113), `src/sdk/api/search.rs` (136 L, completo: guard ERR-028 L26-37), `src/sdk/search/mod.rs` (L185-224 y L240-289), `src/error.rs` (L425-484 `code()`), `vantadb-wasm/src/lib.rs` (L1754-1768 `reindex_hnsw_from_text`, L2196-2254 `from_engine_error`/`to_js_err`), `vantadb-ffi-core/src/lib.rs` (L175-284 `VantaErrorCode`/`as_str`/`from_code`), `docs/api/TS_SDK.md` (L489-530, L690-769), `docs/api/ERROR_HANDLING.md` §1.1 (L112-126), `docs/dev/tasks/FIND-156.md` (L105-164), `vantadb-ts/README.md` (L300-310), `.opencode/rules/js-ecosystem.md`, `.opencode/rules/api-contract.md` (R-8).
- **Referencias hacia dentro:** `portability.test.ts` es el único sitio que fija el contrato de `export*/import*`/`reindexHnswFromText` desde TS; `hardening.test.ts` es el único que fija los códigos de error del engine real. `errors.ts` está importado por `vantadb.ts`, `native.ts` y 6 tests (`hardening`, `native-error`, `dx01`, `d5a-validation`, `wire03`, `vanta`).
- **Referencias hacia fuera:** `vantadb` → `vantadb-wasm` (`file:../vantadb-wasm/pkg`), `vantadb-node` (`file:../vantadb-node`); el pin de errores atraviesa `vantadb-ffi-core::VantaErrorCode`.
- **Veredicto de impacto:** **alto en contrato público TS, bajo en superficie de código**. 25 líneas en `errors.ts` (una rama nueva de precedencia + docstrings), 2 ficheros de test y 14 líneas de doc. Cero cambios en Rust.

## Diagnóstico — los 4 rojos, medidos

`npm test` en `vantadb-ts` sobre `develop` (`972f228d5`): **393 tests, 4 rojos en 2 ficheros**. Reproducidos también en `origin/develop` (`ec1bf9d58`) sin los commits locales, y sin cambios pendientes en `vantadb-wasm/` ni en `src/` → **no es regresión de hoy**.

### Los 3 pins obsoletos (el test afirmaba el mundo anterior)

| Pin | Qué afirmaba | Qué hace hoy (medido) |
|---|---|---|
| `portability.test.ts:150` `reindexHnswFromText rejects with platform IO error` | `toThrow(/operation not supported/i)` | Devuelve report: `{"scanned_nodes":1,"indexed_vectors":0,"skipped_tombstones":0,"duration_ms":0,"derived_rebuild_ms":1,"index_path":"vector_index.bin","success":true}` |
| `portability.test.ts:156` `system subclient delegates export/import/reindex identically` | `db.system.reindexHnswFromText(...)` → `DbError` | Mismo report vía el subcliente |
| `hardening.test.ts:547` `rebuildIndex on empty DB throws IO error on non-persistent` | `toThrow(DbError)` | `{"scanned_nodes":0,"indexed_vectors":0,...,"success":true}` |

**Por qué el pin era falso, con la cadena completa:**

1. `rebuild_index` y `reindex_hnsw_from_text` están implementados para **todos** los backends y **no tienen gate de plataforma**: `src/sdk/api/admin.rs:19` y `:45`. No hay `#[cfg(target_arch = "wasm32")]` en `src/sdk/` (`grep` → 0 hits) y `Error::UnsupportedOperation` ya no se construye en ninguna ruta del SDK (solo en `src/error.rs` y `src/server/errors.rs`).
2. El binding WASM los reenvía sin condición: `vantadb-wasm/src/lib.rs:1756` → `inner.reindex_hnsw_from_text(...)`.
3. Ninguno de los dos toca el filesystem: `rebuild_index` llama `engine.rebuild_vector_index()` + índice derivados/texto/escalar; `reindex_hnsw_from_text` pagina con el `list()` del SDK y delega al engine. El `index_path: "vector_index.bin"` del report es un **nombre lógico** del report, no un `fs` write (el backend `InMemory` lo acepta).
4. El pin se escribió el **2026-09-20** (`06f8f84f8`), y el binding ya exponía `reindex_hnsw_from_text` desde antes del **2026-07-24** (`896b0c69f` sólo le añadió paginación) → **el pin nació falso**; nunca comparó con el binding contra el que corre.
5. La suite corre contra el **binding WASM en Node** (`Client` ← `vantadb.ts` ← `vantadb-wasm/pkg`), no contra `vantadb-node`: los `ExperimentalWarning: Importing WebAssembly module instances` de cada corrida lo confirman. (`vantadb-node` no tiene `reindex_hnsw_from_text`; `grep` → 0 hits.)

**Lo que SÍ sigue siendo cierto y por eso los otros pins pasan:** `exportAll` / `exportNamespace` / `importFile` llegan a `std::fs` de verdad (`src/sdk/serialization/impl_export.rs:351-369`, `:560`) y en `wasm32-unknown-unknown` std devuelve `unsupported` → el mensaje `operation not supported on this platform` sale de **std, no del core**. Esos tres pins no se tocan.

### El 4º rojo era correcto: la frontera perdía el código canónico

`hardening.test.ts:122` `zero-norm cosine search surfaces VANTADB_VALIDATION_ERROR` → recibía `VANTADB_INVALID_ARGUMENT`.

Cadena medida con un probe (el objeto capturado y su `cause`):

```
msg   = "search: Invalid input: zero-norm cosine query vector is undefined; ... (AUDREP-55, ERR-028)"
cause = { code: "ERR_INVALID_INPUT", vanta_code: 6, raw_code: "VANTADB_VALIDATION_ERROR" }
DbError.code (antes del fix) = "VANTADB_INVALID_ARGUMENT"
```

- El core rechaza con `Error::InvalidInput` (`src/sdk/search/mod.rs:256`, guard ERR-028/AUDREP-55) cuyo `code()` canónico es **`VANTADB_VALIDATION_ERROR`** (`src/error.rs:461`, fila de `docs/api/ERROR_HANDLING.md` §1.1 y de `docs/api/TS_SDK.md:709`).
- El binding está bien: `to_js_err` adjunta **las dos** cosas — `code` (alias **grueso** de `vantadb-ffi-core::VantaErrorCode`) y `raw_code` (el `Error::code()` canónico, FFI-02).
- El bug estaba en `vantadb-ts/src/errors.ts:268`: `fromNativeError` leía **primero** el `.code` grueso, y `mapCodeToCanonical("ERR_INVALID_INPUT")` lo colapsa a `VANTADB_INVALID_ARGUMENT`. `VantaErrorCode` no puede distinguir `Validation`/`InvalidInput`/`Iql`/`IqlParse`: `from_code` funde los dos códigos canónicos en `InvalidInput` (`vantadb-ffi-core/src/lib.rs:257-260`). **`raw_code` existía justamente para eso y nadie lo leía.**
- Contradecía el contrato escrito dos veces: `native-error.test.ts:201` ("Every code `Error::code()` emits must reach TS callers unchanged; FFI-02's coarse numeric aliases must never rewrite them") y `.opencode/rules/api-contract.md` R-8 (un binding que reescribe el error del core crea divergencia por plataforma).

## Decisión (a)/(b) — explícita y por caso

- **(a) `reindexHnswFromText` y `rebuildIndex`: el binding DEBE soportarlos** → se corrigen los **pins**, no el código. Evidencia: `src/sdk/api/admin.rs:19,45` sin gate, `vantadb-wasm/src/lib.rs:1756` sin gate, y el report real en runtime. La limitación de WASM queda documentada donde corresponde y **acotada a lo que es cierto**: sólo el grupo `Export / Import` toca `std::fs` (`docs/api/TS_SDK.md` §Maintenance, nota nueva; la cabecera de `portability.test.ts` deja de decir que `reindexHnswFromText` está en ese grupo).
- **(b) `zero-norm`: el error NO debe reescribirse** → el fallo está en la implementación de la frontera TS y se arregla ahí. `fromNativeError` pasa a **preferir `raw_code`** (el `Error::code()` del core) y deja el `.code` grueso como fallback para pkg viejos sin `raw_code`. El test **no se toca**: es el que tenía razón.
- **Descartado:** cambiar la expectativa a `VANTADB_INVALID_ARGUMENT` habría sido "arreglar el test para que no pruebe nada" (pre-mortem a), legitimando una divergencia que contradice dos documentos y el comentario de `vantadb.ts:739-745` ("this layer is glue and must NOT make search decisions… like `native.ts`").

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Reproducir los 4 rojos + identificar el 2º fichero (`hardening.test.ts`), descartar regresión de hoy y binding viejo | ✅ |
| 2 | Discovery: core + binding + frontera de errores + doc de contrato (§1.1, R-8) → decisión (a)/(b) por caso | ✅ |
| 3 | RED: los 4 tests ya fallan por la razón correcta (probe con el `cause` de cada error) | ✅ |
| 4 | GREEN (b): `errors.ts` prefiere `raw_code` → el pin de zero-norm pasa **sin tocarlo** | ✅ |
| 5 | GREEN (a): reescribir los 3 pins a la capacidad real (`report` + efecto observable, sin `skip`/`todo`/relajar) | ✅ |
| 6 | Doc: `docs/api/TS_SDK.md` §Maintenance (`reindexHnswFromText` + nota "no es filesystem-bound") y §`wrapWasmError` (precedencia de `raw_code`) | ✅ |
| 7 | Verify: `npm test` 393/393 · `check-links` · `check-docs` · `ocr-review` | ✅ |
| 8 | Review P2-01 (Adversarial: `docs/api/**` en el diff) + commit local | ✅ |

## Verify

| Comando | Resultado |
|---|---|
| `cd vantadb-ts && npm test` | ✅ **26 ficheros / 394 tests** (antes: 2 ficheros / 4 rojos) |
| `npx vitest run src/__tests__/portability.test.ts` | ✅ 14/14 |
| `npx vitest run src/__tests__/hardening.test.ts` | ✅ 72/72 |
| `npx tsc --noEmit` | ❌ **pre-existente y fuera de scope**: `vantadb-wasm/pkg/vantadb_wasm.d.ts` está desincronizado con `vantadb-wasm/src/vantadb_wasm.d.ts` (`node dev-tools/build-wasm-types.mjs --check` → FAIL). Faltan `AstJson`, `ListOptionsInput`, `MemoryRecordInput`, `SearchRequestInput`; 0 errores en los ficheros tocados. `pkg/` es artefacto gitignored (js-ecosystem R-2): regenerarlo sin recomilar el `.wasm` daría un verde falso |
| `npm run lint` | ❌ **pre-existente**: 6 errores en `native.ts` (3) y `vantadb.ts` (3), ficheros no tocados; `errors.ts` limpio |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 |
| `node scripts/docs/check-docs.mjs` | ✅ exit 0 |
| `pwsh dev-tools/ocr-review.ps1` | ✅ 1 reviewable (`errors.ts`, tests excluidos por `default_path`); revisado contra el Rule Group cargado con `ocr delegate rule` |
| `cargo` (todo) | ✅ no ejecutado: **cero cambios en Rust** |

## Deuda — lo que queda fuera y por qué

- **`vantadb-ts` no tiene typecheck ni lint en ningún gate.** El `pre-push` corre `npm test` y nada más; `npx tsc --noEmit` y `npm run lint` están rojos por causas pre-existentes (pkg types stale + 6 errores ESLint en `native.ts`/`vantadb.ts`). Es la misma clase de fallo que este ticket: un gate que no mide. Fila `FIND-*` propuesta abajo.
- **`vantadb-node` no expone `reindex_hnsw_from_text`** (`grep` → 0 hits en `vantadb-node/src/`), mientras WASM/TS sí. No afecta a este ticket (la suite TS corre contra WASM) pero es paridad pendiente entre bindings.
- **`docs/api/ERROR_HANDLING.md` §1.1 y la tabla de `TS_SDK.md`** siguen teniendo una columna `?` (sin verificar). Con `raw_code` en su sitio, la fila `InvalidInput → VANTADB_VALIDATION_ERROR` ya es mecánicamente cierta para TS/WASM; el resto de bindings no se auditó aquí.

### Fila propuesta para `docs/dev/Backlog.md` (el owner del Backlog la registra)

```
| `FIND-729` | 🟡 Media | **`vantadb-ts` no tiene typecheck ni lint en ningún gate: `tsc --noEmit` y `eslint` están rojos y nadie lo ve** | `vantadb-ts/tsconfig.json` → `vantadb-wasm/pkg/vantadb_wasm.d.ts` · `.githooks/pre-push` | 🟢 2-3h | 🟡 Media | ⬜ Pendiente (2026-11-25) | Destapado al reparar el `pre-push` (`FIND-718`): el hook corre `npm test` y nada más. `npx tsc --noEmit` falla con 4 errores porque `vantadb-wasm/pkg/vantadb_wasm.d.ts` (gitignored, R-2 de `js-ecosystem.md`) está desincronizado con la fuente escrita a mano (`node dev-tools/build-wasm-types.mjs --check` → FAIL: faltan `AstJson`, `ListOptionsInput`, `MemoryRecordInput`, `SearchRequestInput`); `npm run lint` falla con 6 errores pre-existentes en `native.ts` (3) y `vantadb.ts` (3). Contrato: `npx tsc --noEmit` y `npm run lint` exit 0, y el `pre-push` los corre cuando el push toca `vantadb-ts/**` (más `.nvmrc`/`engines` documentando que `pkg/` debe regenerarse antes). | Origen: `FIND-728` (2026-11-25) | Ver: `FIND-728`, `FIND-718`, `FIND-156` |
```

## Lo que esta tarea NO hace

- **No** toca Rust (`src/**`, `vantadb-wasm/**`, `vantadb-node/**`): la decisión (a)/(b) se resolvió sin cambios de core ni de binding.
- **No** regenera `vantadb-wasm/pkg/` ni corre `npm install` (el `pkg` del 24-nov **sí** trae `raw_code`: se verificó en runtime leyendo el `cause` del error, no por fecha).
- **No** usa `skip`, `todo`, `.fails` ni afloja un matcher: los 3 pins obsoletos se reemplazaron por aserciones **más** específicas (report con conteos exactos + "un rebuild no es un wipe").
- **No** toca `.githooks/**` ni hace `git push`.
- **No** edita `Backlog.md`, `plans/**` ni `avance/**` (fuera de la lista de escritura de la tarea): las filas quedan como texto para el orquestador.

## Review (P2-01)

- **Tier:** **Adversarial** — el diff incluye `docs/api/**` (`TS_SDK.md`). Además toca la superficie pública de códigos de error TS.
- **Reviewer:** `vanta-review` (contexto fresco, sin acceso a la sesión que implementó; sesión `ses_deea780b0ffeNNtf6RURBHubCA`).
- **Enfoque:** verificar el diagnóstico contra el código, no contra el resumen — ausencia de gate de plataforma en `src/sdk/api/admin.rs:19,45`; `std::fs` real en `impl_export.rs:88,354,401,651` (el grupo Export/Import); forwarding en `vantadb-wasm/src/lib.rs:1756`; precedencia de `fromNativeError` (raw_code → code → prefijo → regex) sin que el prefijo pueda pisar el código estructurado (`errors.ts:294`); completitud de `mapCodeToCanonical` sobre los 10 códigos canónicos (incl. `VANTADB_TIMEOUT`, que resuelve por `ERROR_CODES` en el `default`); aislamiento de `Client.create()` para los contadores engine-wide.
- **Cómo se probó:** `cd vantadb-ts && npm test` → **26 ficheros / 394 tests** (contrato, corrido por el reviewer: 393 antes de añadir el pin sintético de `raw_code`). `vitest run portability + hardening` ×3 → 86/86 estable, sin flakiness. `npx tsc --noEmit` (4 errores, sólo `types.ts`/`vantadb.ts`, pkg stale) y `npm run lint` (6 errores, sólo `native.ts`/`vantadb.ts`): ninguno en ficheros del diff ⇒ pre-existentes. `git status --short`: sólo los 4 ficheros + este task file.
- **Veredicto:** ✅ **approve** — 0 Critical, 0 Required, 3 Optional, 3 Nits.
- **Aplicado del review:**
  - **O1** — `TS_SDK.md` §`wrapWasmError` anuncia el **segundo** cambio de superficie que el fix arrastra: `Error::ResourceLimit`/`QuotaExceeded` pasa de `VANTADB_QUOTA_EXCEEDED` a **`VANTADB_RESOURCE_LIMIT`** (contract-correct según §1.1, pero silencioso para un `switch` de consumidor).
  - **O2** — `TS_SDK.md:497` nombraba `IndexRebuildReport`, tipo que **no existe en TS** (`vantadb.ts:1243` declara `unknown`): reemplazado por el shape real.
  - **O3** — `classifyWasmError` no tenía patrón `/corrupt/i` y dejaba huérfano el hedge `msg.contains("corrupt")` del binding: añadido (única rama donde la precedencia nueva perdía información).
  - **N1** — el test del subcliente `system` comparaba la misma implementación consigo misma: eliminada la tautología (queda `success` + `scanned_nodes >= 1`).
  - **N3** — pin sintético de la precedencia (`hardening.test.ts`, `prefers the canonical raw_code…`) para que la regla no dependa del binario wasm, más el caso "pkg viejo sin `raw_code`".
- **Descartado con razón:** mover el arreglo a `to_js_err` (`.code` canónico, borrar `raw_code`) — exige recompilar wasm + regenerar `pkg/` (justo lo que está roto en `FIND-729`) y rompe el alias grueso que consumen los otros bindings. El fix TS es el mínimo que funciona y es compatible hacia atrás con pkgs viejos.
