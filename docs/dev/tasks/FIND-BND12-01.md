# FIND-BND12-01 — index.d.ts u64 → BigInt (4 decls)

> **Plan:** `docs/dev/plans/2026-09-07-followup-bench-a11y.md` (Task 1, Wave0)
> **Estado:** ⏳ IN PROGRESS
> **Tipo:** bug-fix (type-lie runtime) · **Fase:** BUILD
> **SDP:** source-driven-development, incremental-implementation, test-driven-development, context-engineering, systematic-debugging, api-and-interface-design, doubt-driven-development + ponytail (full) — frontend-ui-engineering descartado (no es web/), campaign-executor auto-cargado vía MCP

## Contrato

- `grep -c "Promise<number>" vantadb-node/index.d.ts` → 0 en esas 4 decls (bigint)
- `npm test` en `vantadb-node/` 34 passed (existentes intactos)
- Nuevo test que aserta `typeof await db.count(...) === "bigint"`

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-node/index.d.ts` (líneas 1-30 header, 330-381 decls), `vantadb-node/src/lib.rs` (500-579), `vantadb-node/dts-header.d.ts` (header fuente), `vantadb-node/package.json` (napi 3.8.2, dtsHeaderFile), `vantadb-node/tests/api.test.ts` (410-462), `docs/api/NODE_SDK.md` (líneas 183-201 bigint truth ya documentada)
- **Referencias hacia dentro:** `lib.rs` 4 fns → `engine.compact_layout/purge_expired/delete_by_filter/count` (core `src/sdk/api/namespaces.rs`, `src/storage/archive.rs`) — todos devuelven `u64`; napi-rs mapea `u64`→BigInt runtime
- **Referencias entrantes:** `tests/api.test.ts:431-451` ya aserta runtime `0n`/`2n` (BND-12 dejó evidencia); `NODE_SDK.md:183-201` ya documenta bigint truth; `bench/bench-abi.mjs` — verificar que no asuma number (fuera de scope si no)
- **Veredicto:** impacto bajo, 2 archivos + 1 test. Fix real en `lib.rs` vía `#[napi(ts_return_type = "Promise<bigint>")]` (mecanismo documentado https://napi.rs/docs/concepts/types-overwrite, ya citado en dts-header + usado 12× en el archivo) + sync manual de las 4 líneas del `.d.ts` (el `.d.ts` se regenera en `npm run build`, el attr lo fija de forma permanente). No toca core/engine — propiedad de Arch/Engine intacta.

## Gate D

No disparado: blast radius 2 archivos, sin símbolos públicos nuevos (solo corrección de tipo), contrato mecánico no ambiguo. (Sin tool `question` disponible en este entorno; decisión registrada.)

## Steps

- [x] **Step 1 — lib.rs attrs:** `#[napi(ts_return_type = "Promise<bigint>")]` en `compact_layout`, `purge_expired`, `delete_by_filter`, `count` → verify `cargo check --manifest-path vantadb-node/Cargo.toml` ✅ (1m00s, sin warnings)
- [x] **Step 2 — index.d.ts sync:** 4× `Promise<number>` → `Promise<bigint>` (líneas 349, 359, 364, 366) → verify `Select-String Promise<number>` = 0, `Promise<bigint>` = 4 ✅
- [x] **Step 3 — test + suite:** nuevo test `u64-returning methods resolve with typeof bigint` (count + purgeExpired) → verify `npm test` en `vantadb-node/` 35 passed (34 existentes intactos + 1 nuevo) ✅

## Spec

N/A — bug-fix mecánico con contrato medible (type-lie runtime, 4 líneas + test assert). Sin decisiones de diseño abiertas.
