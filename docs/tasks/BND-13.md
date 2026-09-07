# BND-13 — docs/api/NODE_SDK.md completa (plan 2026-09-07-backlog-triage, Task 5, Wave2)

> Plan: `docs/plans/2026-09-07-backlog-triage.md` (Task 5, Wave2)
> Estado: ✅ COMPLETED 2026-09-07
> Ejecución: SARL STRATEGY — sub-agente vanta-docs cayó por infra ("model not available")
> antes de implementar; el lead recuperó su WIP (DISCOVERY PARTIAL + matriz + README
> pointer) y cerró inline. Sin trabajo perdido.

## Contrato (plan Task 5, literal)

- `docs/api/NODE_SDK.md` existe con quickstart + matriz native-vs-wasm + ejemplos por runtime
- `scripts/validate-docs-coverage.ps1` sin gaps nuevos para `vantadb-node`
- Sin números de performance sin fuente (Regla 11)

## Verificación real

- Base `a86c7e4e` ya traía quickstart + API reference + runtimes CJS/TS/Bun/Deno.
- WIP recuperado añadía: matriz native-vs-WASM con fairness caveat + Bun/Deno
  ampliados + nota runtime truth `bigint` (FIND-BND12-01) + sección Benchmark
  que difiere a `BENCHMARKS.md §15` (TS-09) + pointer README→docs/api.
- `validate-docs-coverage.ps1` → 0 gaps; `grep ms|QPS|faster` → solo `ms` en
  nombres de API (`ttl_ms`, `duration_ms`) y cita a §15 — 0 claims sin fuente.

## Blast radius

Callers: `vantadb-node/README.md` (pointer) · `docs/api/` (ningún otro doc cita
NODE_SDK como fuente). Doc-only, 0 archivos de código tocados.

## Steps

### Step 1: Completar matriz + runtimes + BigInt + benchmark (recuperado de WIP) ✅
### Step 2: validate-docs-coverage + Regla 11 grep ✅
### Step 3: Commit selectivo (lead) ✅

## Notas

- Colateral BND-12 FIND-BND12-01 documentado como runtime truth, no como firma
  deseada — el fix de `index.d.ts` queda para su dueño.
- UX-A11Y-01/WEB-02 en worktree no tocados (scope discipline).
