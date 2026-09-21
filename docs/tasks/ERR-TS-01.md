# TASK ERR-TS-01: Unificar TS/WASM codes VANTADB_* + wrapNativeError + guards VantaError

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 4, Wave 2)
- **Creado:** 2026-09-02T21:10
- **last-synced:** 2026-09-02T21:10
- **Estado:** ⏳ IN PROGRESS
- **Ruta:** vanta-worker
- **SDP:** campaign-executor, ponytail (full), incremental-implementation, test-driven-development, context-engineering (embedded §3a)

## Blast Radius (DISCOVERY 2026-09-02)

**Verificados en fuente:**
- Core: `VantaError::code()` es `pub` (`src/error.rs:306`) → devuelve 9 códigos wire `VANTADB_*` (+ `VANTADB_CLOSED` lifecycle-only, nunca retornado). Espejo 1:1 con `ERROR_CODES` TS sin prefijo. **Opción ponytail confirmada:** WASM llama `e.code()` directo, elimina tabla duplicada.
- `vantadb-wasm/src/lib.rs`: `vanta_error_code` (1930, tabla 30→8 propia) tiene UN solo caller: `to_js_err` (1972). Sin tests que asserten code strings en el crate. → borrar función, usar `e.code()`.
- `vantadb-node/src/lib.rs`: `map_err` (640) = `Error::from_reason(e.to_string())` — único punto de conversión core→napi (vía `spawn_blocking:737`. Los demás `from_reason` son validación local de inputs, fuera de scope). → extender `map_err` con prefijo `"{code}: {Display}"`.
- `vantadb-ts/src/errors.ts`: `ERROR_CODES` 10 valores sin prefijo; NO exportado; `classifyWasmError` retorna string-literals; `KNOWN_CODES` deriva de los values. Consumers internos usan literales crudos en native.ts (6× "VALIDATION_ERROR" + 1 "CLOSED" + "NATIVE_ERROR") y vantadb.ts (2 "VALIDATION_ERROR" + 1 "CLOSED").
- `vantadb-ts/src/native.ts:34` `wrapNativeError` → "NATIVE_ERROR" (fuera de ERROR_CODES — bug). `guards.ts:84,89,93` → `TypeError`/`RangeError` nativos.
- **Tests que impactan:** `hardening.test.ts` (2 asserts "VALIDATION_ERROR"), `native-error.test.ts` (2 asserts "NATIVE_ERROR"), `vanta.test.ts:223-231` (TypeError/RangeError de validateVector). `e2e/` del crate: sin hits de code strings (verificado rg) — NO se toca (fuera de scope).
- **Spec:** `docs/api/ERROR_HANDLING.md` §1.1 l.60 documenta "TS/Python normalize to the unprefixed names **until**..." + §1.2 "All ten table codes map 1:1 from the unprefixed TS names" → el rename de VALORES a `VANTADB_*` es convergencia al contrato canónico YA especificado, no ambigüedad → **Gate P NO se dispara**. §4 (tabla TS) debe actualizarse con los valores nuevos.
- **Consumidores externos de valores wire:** `web/` y `desktop/` tienen su propio TS (no importan ERROR_CODES de vantadb-ts según grep); consumidores npm del paquete `vantadb` que comparen `err.code === "VALIDATION_ERROR"` → BREAKING documentado (CHANGELOG + §4 spec).

## Contrato (user brief, canónico)

```
grep -c '"NATIVE_ERROR"' vantadb-ts/src/errors.ts == 0
grep -c 'TypeError(' vantadb-ts/src/guards.ts == 0
npm run build (vantadb-ts) exit 0
npx vitest run (vantadb-ts) 0 failed
cargo check -p vantadb-wasm --all-targets 0
cargo check --manifest-path vantadb-node/Cargo.toml --all-targets 0
cargo clippy --workspace --all-targets --all-features -- -D warnings 0
```
(Plan file añade `grep code.*GenericFailure vantadb-node/src/lib.rs >= 1` → satisfecho con `Error::new(Status::GenericFailure, "{code}: …")`.)

## Steps

### Step 1: WASM — `vanta_error_code` → `e.code()` del core
- **Archivos:** `vantadb-wasm/src/lib.rs`
- **Acción:** borrar la tabla duplicada (24L), `to_js_err` llama `e.code()`. Doc comment: codes canónicos VANTADB_* ahora, mismo contrato que Rust core.
- **Verify:** `cargo check -p vantadb-wasm --all-targets` + clippy crate
- **Estado:** ✅ DONE

### Step 2: Node — `map_err` propaga code
- **Archivos:** `vantadb-node/src/lib.rs`
- **Acción:** `map_err` → `Error::new(Status::GenericFailure, format!("{}: {}", e.code(), e))`. Doc header línea 15. Test unitario puro: `map_err_includes_canonical_code_prefix` (NodeNotFound → reason arranca "VANTADB_NOT_FOUND: ").
- **Verify:** `cargo check --manifest-path vantadb-node/Cargo.toml --all-targets` + `cargo test --manifest-path vantadb-node/Cargo.toml`
- **Estado:** ✅ DONE

### Step 3: TS errors.ts — valores VANTADB_* + export ERROR_CODES
- **Archivos:** `vantadb-ts/src/errors.ts`
- **Acción:** valores → `VANTADB_*`; `export const ERROR_CODES`; `classifyWasmError` retorna `ERROR_CODES.*`.
- **Verify:** vitest
- **Estado:** ✅ DONE

### Step 4: TS native.ts — wrapNativeError parsea "CODE: msg"
- **Archivos:** `vantadb-ts/src/native.ts`
- **Acción:** parse prefijo `^VANTADB_[A-Z0-9_]+:` → code known + message sin prefijo; fallback `classifyWasmError`; sin "NATIVE_ERROR". Literales → `ERROR_CODES.*`. Doc del class actualizado. RED test nuevo en native-error.test.ts (parse de prefijo) antes del fix.
- **Verify:** `npx vitest run -t native` 
- **Estado:** ✅ DONE

### Step 5: TS guards.ts — VantaError(VALIDATION_ERROR) en validateVector
- **Archivos:** `vantadb-ts/src/guards.ts`
- **Acción:** TypeError/RangeError → `new VantaError(ERROR_CODES.VALIDATION_ERROR, msg)`. BREAKING documentado. Tests vanta.test.ts actualizados.
- **Verify:** grep `TypeError(` == 0 + vitest
- **Estado:** ✅ DONE

### Step 6: TS vantadb.ts — literales → ERROR_CODES
- **Archivos:** `vantadb-ts/src/vantadb.ts`
- **Acción:** "VALIDATION_ERROR"×2 + "CLOSED"×1 → `ERROR_CODES.*` (consistencia, mismo valor → sin cambio de comportamiento).
- **Verify:** npm run build + vitest
- **Estado:** ✅ DONE

### Step 7: Docs — ERROR_HANDLING.md §4 + CHANGELOG +TYPESCRIPT_SDK
- **Archivos:** `docs/api/ERROR_HANDLING.md`, `docs/CHANGELOG.md`, `docs/api/TYPESCRIPT_SDK.md` (verificar si lista codes)
- **Acción:** snippet §4 → valores VANTADB_*; ejemplo `err.code === "VALIDATION_ERROR"` → prefijado; nota de alineación (precedente ERR-DOCS-01 editó CHANGELOG sección Unreleased).
- **Verify:** grep VANTADB_ en §4
- **Estado:** ✅ DONE

### Step 8: Cierre — verify completo del contrato + commit + plan/avance + recitation
- **Verify:** los 7 comandos del contrato + clippy node standalone
- **Estado:** ✅ DONE

## Dependencias
- Wave 1: ERR-CORE-01 (`code()` pub, commit e1fe7ec2) ✅ lista

## Notas
- Opción elegida (no Gate P): mantener KEYS TS legibles (`ERROR_CODES.VALIDATION_ERROR`), cambiar solo VALORES wire a `VANTADB_*` — el brief lo autoriza explícitamente y la spec §1.2 ya lo documenta como convergencia pendiente.
- `VANTADB_CLOSED`: TS lo emite (lifecycle en SDK), core nunca lo retorna — consistente con §1.1.
- NOTICED BUT NOT TOUCHING: `napi::Error` no puede adjuntar propiedad `code` extra sin API `throwError` custom (el Status enum no lo soporta) → prefijo en message, parse en TS (una sola dirección de conversión). `spawn_blocking` join error (736) no lleva code (no es VantaError) — cae a fallback classifyWasmError.

## Context Save Point
- **Fecha:** 2026-09-02T23:55
- **Branch:** develop
- **CI pendiente:** no
- **Commits:** `2686bea2` (fix ts/wasm/node) + `c68e2dca` (style node closure) — hooks pre-commit verdes
- **Decisiones:** (1) `e.code()` directo en WASM (elimina tabla, ponytail) sobre extender la tabla local 30→10; (2) Node: prefijo `"{code}: {msg}"` en `Error::new(Status::GenericFailure, …)` sobre propiedad extra (napi no la soporta trivialmente); (3) TS: values VANTADB_*, keys sin prefijo, `ERROR_CODES` exportado + literales crudos eliminados en src (previene drift futuro — la bug raíz de este task); (4) guards→VantaError BREAKING documentado (tests internos actualizados, sin consumidores internos de TypeError); (5) `cause` ES2022 añadido por promesa §4.3 del propio spec.
- **Verificación final:** grep NATIVE_ERROR errors.ts=0 ✓ · TypeError( guards.ts=0 ✓ · npm run build=0 ✓ · vitest 249/278 (29 fails=FIND-52 preexistentes, unit ERR-TS-01 20/20 ✓) · wasm check/test ✓ · node check/test 4/4 + E2E `VANTADB_VALIDATION_ERROR: Validation error on read_only:…` ✓ · clippy workspace+node ✓ · fmt ✓ (node crate now fmt-clean)
- **Problemas conocidos:** (a) FIND-52 bloquea "vitest 0 failed" en esta máquina — HEAD tiene panics std::time/Condvar bajo wasm32 con pkg reconstruido + rustc ICE en release (dev-profile workaround, no commitear como artefacto final); (b) `vantadb-node/index.d.ts` regenerado por el rebuild (drift BND-10, +44L) — revocado para mantener scope, documentado en FIND-52; (c) pkg local es build `--dev` (sin -Oz ni optimizar) — regenerar en release cuando FIND-52 se resuelva.
- **Próxima tarea:** Wave 3: ERR-DESK-01 / ERR-WEB-01 / ERR-OBS-01
