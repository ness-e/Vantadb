# ISSUE-TS-001: Fix TS SDK (vitest-first)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-fixes.md
- **Creado:** 2026-09-10
- **Estado:** ✅ COMPLETED (verify-first: suite verde sin cambios, 2026-09-10)
- **Tipo (campaign_detect_task_type):** typescript / TypeScript SDK
- **Esfuerzo:** 🟢 ~15 min (verify-first, cero fix necesario)
- **Prioridad:** 🟠 Media-Alta
- **Branch:** develop (sin rama dedicada — cero cambios de código)
- **Commit:** (este cierre)
- **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design + systematic-debugging (bug), test-driven-development (Prove-It)

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `vantadb-ts/package.json` | ~60 | Scripts: `test` = `vitest run`; dep `vantadb-wasm: file:../vantadb-wasm/pkg`; devDeps vitest ^4.1.10 |
| `vantadb-ts/vitest.config.ts` | ~25 | Include `src/**/__tests__/**/*.test.ts` + `tests/**/*.test.ts`; wasm external (no vite-plugin-wasm, Node ≥22 ESM wasm nativo) |
| `vantadb-ts/src/` | — | `vantadb.ts`, `types.ts`, `errors.ts`, `guards.ts`, `metadata.ts`, `native.ts` + `__tests__/` (9 files) |
| `vantadb-ts/tests/` | — | `graph.test.ts` |

### Referencias hacia adentro (outbound references)
- Suite → `vantadb-wasm` (file dep `../vantadb-wasm/pkg`) cargado nativamente por Node; sin transform vite.

### Referencias hacia afuera (inbound)
- Ningún otro workspace depende de `vantadb-ts` en este runner (disjunto de FIND-MCP-001/mcp).

### Veredicto de impacto
**Cero cambios necesarios.** Premisa del issue (`unreachable!` 80/219) stale: `unreachable!` = 0 matches en todo `vantadb-ts/` (grep 2026-09-10). Suite corre verde en develop. Blast a `vantadb-wasm` no materializado (sin fix → sin blast).

## Contrato (verificable mecánicamente)

```
1. `npx vitest run` en vantadb-ts → ✅ 10 files / 280 tests passed, 0 failed (16.10s)
2. `npx tsc --noEmit` en vantadb-ts → ✅ exit 0
3. `unreachable!` en vantadb-ts/ → ✅ 0 matches
```

## Steps

### Step 0: Verify-first — correr suite sin tocar código
- **Archivos:** (ninguno — solo lectura: package.json, vitest.config.ts, src/, tests/)
- **Acción:** `npx vitest run` (bash directa; campaign_verify_cmd con bug exit -1) + grep `unreachable!` + `npx tsc --noEmit`.
- **Verify:** 10 passed / 280 passed / 0 failed; tsc exit 0; unreachable 0.
- **Estado:** ✅ COMPLETED (2026-09-10) — stop condition del plan dispara: suite verde → cerrar como resuelto, no fix fantasma.

## Hallazgos colaterales
Ninguno. Warnings `ExperimentalWarning: Importing WebAssembly module instances` (Node experimental, informativo, no falla tests). WIP ajeno intacto (`M opencode.jsonc`, `M .opencode`, `M docs/dev/Backlog.md`, `?? Investigacion-plan.md` — no tocados ni stageados).

## Context Save Point
N/A — tarea completa, sin pendientes.
