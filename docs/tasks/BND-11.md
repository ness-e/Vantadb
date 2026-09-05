# TASK BND-11: Tipado fuerte index.d.ts (eliminar any)

## Metadata
- **Plan file:** `docs/plans/2026-08-28-backlog-triage.md`
- **Creado:** 2026-08-28T10:30
- **last-synced:** 2026-08-28T10:30
- **Estado:** ✅ COMPLETED (idempotente — trabajo ya completado en commit a86c7e4e)
- **Ruta:** vanta-worker

## Blast Radius

**Callers → Callees → Implicaciones**

- `vantadb-node/index.d.ts` — archivo de declaraciones TypeScript público; consumido por usuarios del SDK Node.js
- `vantadb-node/src/lib.rs` — bindings napi-rs con `#[napi(ts_arg_type=...)]` y `#[napi(ts_return_type=...)]` para generar/override tipos TS
- `vantadb-node/dts-header.d.ts` — tipos manuales añadidos (commit a86c7e4e) para complementar lo que napi-rs no genera
- `vantadb-node/tests/api.test.ts` — 25 tests nuevos que validan los tipos en tiempo de compilación (tsc --noEmit)
- `docs/api/NODE_SDK.md` — documentación actualizada con ejemplos tipados

**Implicaciones:** cambio aditivo en tipos públicos (mejora DX, no breaking para código que ya compila). Semver minor por ampliación de contrato de tipos. No toca `vantadb` core, `wal.rs`, `vector/`, `storage/`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `vantadb-node/index.d.ts` (329 líneas — leído completo)
  - `vantadb-node/src/lib.rs` (841 líneas — leído completo)
  - `vantadb-node/dts-header.d.ts` (201 líneas — leído completo)
  - `vantadb-node/tests/api.test.ts` (374 líneas — leído completo)
  - `docs/api/NODE_SDK.md` (222 líneas — leído completo)
- **Referencias hacia dentro:** `vantadb::sdk` types (`VantaMemoryInput`, `VantaMemorySearchRequest`, `VantaMemoryMetadata`, `VantaNodeInput`, `VantaSearchExplanation`, `VantaRuntimeProfile`, `TraversalDirection`, `DistanceMetric`)
- **Referencias entrantes:** usuarios finales del SDK Node.js que importan `vantadb-node`; `vantadb-ts` wrapper TS no depende de este archivo
- **Veredicto:** cambio seguro — tipos más restrictivos, no menos. Eliminación de `any` es strict improvement para DX.

## Contrato

`Select-String -Path "vantadb-node/index.d.ts" -Pattern ":\s*any\b" | Measure-Object | Select-Object Count` == 0

Verificación mecánica:
1. `npx tsc --noEmit --project vantadb-node/tsconfig.json` (0 errors)
2. `cargo check -p vantadb-node` (compila)
3. `cargo test -p vantadb-node` (tests pasan)

## Herramientas

- codegraph_explore (blast radius)
- terminal: npx tsc, cargo check/test
- grep/Select-String (verificación contrato)

## Steps

### Step 1: Discovery — verificar estado actual de index.d.ts
- **Archivos:** `vantadb-node/index.d.ts`
- **Acción:** Ejecutar contrato de verificación (`Select-String ... any\b`) para confirmar conteo actual. Revisar diff del commit a86c7e4e para entender qué cambios se hicieron.
- **Verify:** `Select-String -Path "vantadb-node/index.d.ts" -Pattern ":\s*any\b" | Measure-Object | Select-Object Count` == 0
- **Estado:** ✅ DONE (verificado: 0 matches)

### Step 2: Verificar napi-rs type overrides en lib.rs
- **Archivos:** `vantadb-node/src/lib.rs`
- **Acción:** Confirmar que todos los métodos públicos usan `#[napi(ts_arg_type=...)]` y `#[napi(ts_return_type=...)]` con tipos concretos (no `any`). Verificar que `MemoryInput`, `SearchRequest`, `MemoryListOptions`, `GraphFilterOptions`, `ConnectOptions`, `GraphNodeInput` están mapeados a interfaces TS correspondientes en index.d.ts.
- **Verify:** `cargo check -p vantadb-node` + inspección visual de atributos `ts_arg_type`/`ts_return_type`
- **Estado:** ✅ DONE (commit a86c7e4e añadió overrides completos)

### Step 3: Verificar tipos manuales en dts-header.d.ts
- **Archivos:** `vantadb-node/dts-header.d.ts`
- **Acción:** Confirmar que el archivo provee tipos para casos que napi-rs no puede inferir (uniones discriminadas, tagged enums como `VantaValue`, generics complejos). Verificar que no hay `any` residual.
- **Verify:** `Select-String -Path "vantadb-node/dts-header.d.ts" -Pattern ":\s*any\b" | Measure-Object | Select-Object Count` == 0
- **Estado:** ✅ DONE (tipos manuales completos sin `any`)

### Step 4: Verificar tests de tipos en api.test.ts
- **Archivos:** `vantadb-node/tests/api.test.ts`
- **Acción:** Confirmar que los 25 tests ejercitan todas las APIs públicas con tipos concretos y que `npx tsc --noEmit` pasa sin errores. Tests cubren: put/get/list/search/delete, graph operations, explainSearch, capabilities, filtered traversal, degree centrality.
- **Verify:** `npx tsc --noEmit --project vantadb-node/tsconfig.json` + `npm test` (en vantadb-node/)
- **Estado:** ✅ DONE (tests pasan, tsc 0 errors)

### Step 5: Verificar docs/api/NODE_SDK.md actualizada
- **Archivos:** `docs/api/NODE_SDK.md`
- **Acción:** Confirmar que la documentación refleja los tipos fuertes con ejemplos que no usan `any` implícito. Verificar ejemplos de `put`, `search`, `list`, `filter`, graph operations.
- **Verify:** inspección visual + `scripts/validate-docs-coverage.ps1` (0 gaps para NODE_SDK.md)
- **Estado:** ✅ DONE (docs actualizadas en commit a86c7e4e)

### Step 6: Cierre — verify full + plan file update
- **Acción:** Ejecutar verify full del contrato + actualizar plan file (marcar BND-11 ✅ COMPLETED) + crear Context Save Point en task file.
- **Verify:** `campaign_verify_cmd` para cada gate: fmt, clippy, nextest, docs
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (Wave 1; puede ejecutarse en paralelo con PY-01, FIND-40, GOV-TK3, SRV-04)

## Notas
- Trabajo completado idempotentemente en commit `a86c7e4e` (2026-08-26) como parte de "node hardening w1: BND-11/12/13, PERF-BENCH-01"
- El plan file original marcaba ⬜ PENDING por error de triage (backlog no sincronizado con realidad del código)
- Contrato ya satisfecho: 0 ocurrencias de `:\s*any\b` en index.d.ts
- Semver: minor (ampliación de contrato de tipos públicos)

## Context Save Point
- **Fecha:** 2026-08-28T11:30
- **Branch:** develop
- **CI pendiente:** no — verify full completado (verificado en esta sesión):
  - `Select-String -Path "vantadb-node/index.d.ts" -Pattern ":\s*any\b" | Measure-Object | Select-Object Count` → **0** ✅ (CONTRATO)
  - `cargo fmt --check` ✅
  - `cargo clippy -p vantadb -- -D warnings` ✅
  - `cargo clippy -p vantadb-node -- -D warnings` ✅
  - `cargo nextest run --profile audit -p vantadb --build-jobs 2` → **2083 passed** ✅
  - `cargo check -p vantadb-node` ✅
  - `npm run build` (vantadb-node/) ✅
  - `npm test` (vantadb-node/) → **25 passed** ✅
  - `pwsh scripts/check-avance-coverage.ps1` → 1038/1038 IDs cubiertos ✅
- **Decisiones:** tarea cerrada idempotente sin edición — el trabajo ya estaba hecho en commit a86c7e4e (2026-08-26). Solo se crea task file para trazabilidad y se actualiza plan file + docs/avance/activo/bindings.md.
- **Problemas conocidos:** ninguno (pre-existing vantadb-server compilation error no relacionado con BND-11)
- **Próxima tarea:** PY-01 (Paridad graph_bfs_filtered en Python binding)