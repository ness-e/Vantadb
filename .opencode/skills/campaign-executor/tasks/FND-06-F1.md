# FND-06-F1: Zero-norm fallback silencioso en TS (ERR-028) — propagar error del core

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W1)
- **Fuente:** FND-06 H1 (reporte core-bindings-boundaries) + ERR-028
- **Estado:** ✅ COMPLETED (stale — plan previo archivado, cerrada por vanta-lead 2026-08-20)
- **Prioridad:** 🟡

## Objetivo
`vantadb-ts/src/vantadb.ts:333-353` hace fallback silencioso a Euclidean cuando detecta zero-norm en búsqueda cosine — enmascara el comportamiento del core. El core YA rechaza zero-norm cosine (`src/index/search/tests.rs:281-328` — "zero-norm cosine rejected"). El fix: eliminar el fallback en TS y propagar el error del core (o error claro en TS), alineando con native.ts (drift documentado en `native.ts:250-260`).

## Archivos clave
- `vantadb-ts/src/vantadb.ts:333-353`, `vantadb-ts/src/native.ts:250-260`, `vantadb-ts/src/__tests__/hardening.test.ts:204` (test zero-norm), `src/index/search/tests.rs:281-328` (referencia core), `.opencode/rules/api-contract.md` (R-8: lógica de negocio = core; bindings = glue)

## Steps
1. ✅ DISCOVERY: leído fallback actual (vantadb.ts:333-358 — zeroNorm detection + `Euclidean` override), native.ts:250-265 (pasa through, sin fallback), core `src/sdk/search/mod.rs:106-120` (rechaza con `VantaError::InvalidInput("zero-norm cosine query vector is undefined...")`), propagación WASM `vantadb-wasm/src/lib.rs:994` (`map_err(to_js_err)` → JsValue → `wrapWasmError` → VantaError), tests que dependen del fallback (hardening.test.ts:204 hybrid `[0,0,0]`, load.test.ts:31,108 throughput `[0,0,0,0]`). Core YA correcto — sin gap real, sin re-routing.
2. ✅ Implementar: eliminado el fallback silencioso en `vantadb.ts:_buildSearchRequest` — ahora pasa `distance_metric: request.distance_metric ?? "Cosine"` sin override. `native.ts` comentario drift actualizado (ya alineados). No se tocó core (R-8).
3. ✅ Test: `hardening.test.ts` — test "hybrid search" usa vector válido `[1,0,0]`; agregado test dedicado "zero-norm cosine query throws instead of falling back to Euclidean (ERR-028)" con `toThrow(/zero-norm|undefined|InvalidInput/i)` + "zero-norm euclidean query is accepted". `load.test.ts` — queries de throughput cambiados a `[1,0,0,0]`.
4. ✅ Verificar: `npm test` en vantadb-ts → 6 files, 225 tests pass. `npx tsc --noEmit` → limpio. Core no tocado (no aplica cargo check).
5. ✅ Task file + RESULTADO

## Contrato (verify mecánico)
- ✅ grep del fallback silencioso en vantadb.ts → eliminado (0 matches; restantes son comentario loader/field del core/comentario test nuevo)
- ✅ Test de zero-norm en TS espera error (`hardening.test.ts` — `zero-norm cosine query throws...`)
- ✅ `npm test` / runner TS pasa (225/225)
- ✅ Sin gap real en core: `src/sdk/search/mod.rs:106-120` ya rechaza zero-norm cosine con `InvalidInput` — no hubo re-routing a vanta-engine

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- R-8 api-contract: si el fix requiere lógica en core → es re-routing, no implementar en bindings
- No cambiar API pública del TS SDK (solo comportamiento del caso zero-norm: de fallback silencioso a error)

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a (corrección de comportamiento)

## Resultado
```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```