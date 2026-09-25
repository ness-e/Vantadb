# EST-03: ci-gate — medir `main` HEAD + política de conclusión tolerante

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-estabilizacion-pendiente.md` (§EST-03) + `docs/dev/plans/2026-09-24-sesion-continuidad.md` (§4.3, propuesta corregida)
- **Backlog:** fila `EST-03` en P57 (`docs/dev/Backlog.md`)
- **Creado:** 2026-09-24
- **last-synced:** 2026-09-24
- **Estado:** ✅ COMPLETED (2026-09-24)
- **Tipo:** CI/workflow (`.github/workflows/ci-gate.yml`) — ejecución inline (subagentes caídos por provider gating)

## Objetivo
Que el check reusable `ci-gate / Main is green` (invocado por `fuzz.yml`, `heavy-bench-nightly.yml`, `heavy-certification.yml` vía `needs: ci-gate`) evalúe el CI del **HEAD de `main`** y no derive en rojo eterno en cada PR.

## Blast Radius
- **Callers (dependen del job):** `fuzz.yml` (PR + schedule), `heavy-bench-nightly.yml` (schedule), `heavy-certification.yml` (schedule). Si el gate falla → los jobs dependientes se **skipean** (intencional para heavy; en PRs skipea fuzz).
- **No tocar:** nombre del job (`Main is green`), rulesets de `main`/`develop` (11 contexts requeridos — este check **no** está entre ellos; verificado 2026-09-24), los workflows callers.
- **Riesgo de la corrección:** tolerar `missing` permite un heavy run si un check nunca corrió en main (decisión owner A); los fallos reales siguen bloqueando.

## Decisión de diseño (owner, 2026-09-24)
**Opción A** (aprobada): resolver `main` HEAD vía API + conclusión **más reciente por nombre** (`sort_by(.started_at) | last`) + `success|skipped|neutral` = pass + `failure|timed_out|cancelled|action_required` = fail + `sin runs` = WARN tolerado.
- Evidencia dry-run contra `main@ae72803f`: 13/13 PASS (OSV-Scanner y Unused Deps sin runs en main → tolerados; `Analyze` = skipped → aceptado).
- Rechazadas: B (missing=FAIL — exigía triggers nuevos de OSV/machete en main primero), C (diferir).
- Contexto: el check NO es requerido por los rulesets verificados; no bloquea merge, pero (a) su rojo skipea jobs de fuzz en PRs y (b) contradecía su header ("if the CI of the current main commit is red…").

## Contrato
> (a) Simulación local del script contra `main` HEAD → `FAILED=0`; (b) incluir un check en failure (`Lint Markdown`) → `FAILED=1`; (c) nombre inexistente → WARN sin FAIL; (d) `actionlint .github/workflows/ci-gate.yml` sin errores; (e) tras push, `gh pr checks 222` → `ci-gate / Main is green` = **pass**.

## Herramientas
- `gh` (API check-runs/rulesets), `bash`/`jq` (validación del filtro vía gojq de `gh api --jq`), `actionlint`, edit inline.

## Steps
### Step 1: diff del workflow
- **Archivos:** `.github/workflows/ci-gate.yml`
- **Acción:** eliminar el bloque PR-head-SHA (`HEAD_SHA: ${{ github.event.pull_request.head.sha }}`); resolver `SHA=$(gh api repos/$REPO/commits/main --jq .sha)`; 1 sola llamada API a check-runs + `jq` por nombre con `sort_by(.started_at) | last`; política A en el `case`.
- **Verify:** diff + lectura.
- **Estado:** ✅

### Step 2: verificación local de la lógica
- **Acción:** validar el filtro jq con `gh api --jq` (gojq) sobre los check-runs reales de `main` y la política con los 3 casos (real → 0; `Lint Markdown` → 1; inexistente → WARN).
- **Verify:** salidas esperadas registradas.
- **Estado:** ✅

### Step 3: actionlint + commit + push
- **Verify:** `actionlint` ok + pre-commit hook ok + `git push origin develop` (pre-push salta cargo: sin `.rs`).
- **Estado:** ✅

### Step 4: verificación en PR #222
- **Verify:** `gh pr checks 222` → `ci-gate / Main is green` = pass tras el refresh del run de fuzz.
- **Estado:** ✅

### Step 5: cierre (progreso Trigger 1)
- Backlog: eliminar fila `EST-03` (P57) + totales; avance → `docs/dev/avance/activo/ci-cd.md`; nota en plan EST + `sesion-continuidad.md` §4.3; coverage scripts; commit.
- **Estado:** ✅

## Dependencias
- Ninguna. (Arco: estabilización pre-0.7.0; el fix anterior `9705b434` fue refutado por evidencia: run `36061913564`, 13 checks `<not found>` a los 14s.)

## Notas
- `ci-gate.yml` es reusable (`on: workflow_call`); en PRs aparece como `ci-gate / Main is green` dentro del run de fuzz.
- `filter=latest` de la API no deduplica por nombre (hay entradas duplicadas de distintos suites) → el script toma la más reciente por `started_at` de forma determinista.

## Context Save Point
- **Fecha:** 2026-09-24
- **Branch:** `develop`
- **CI pendiente:** no — verificado `ci-gate / Main is green` = **pass** en PR #222 (run `36084499760`, commit `0c27a960`)
- **Decisiones:** Opción A aprobada por owner (main HEAD + skipped OK + missing tolerado con WARN).
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** `EST-10` (barrido API stale).
