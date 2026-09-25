# EST-05: Verificar benchmark verde post-`4b5b137e` (perf-bench + API Docs Version)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-estabilizacion-pendiente.md` (§EST-05) + `docs/dev/plans/2026-09-24-sesion-continuidad.md` (§4.1)
- **Fuente:** Backlog P57 (fila `EST-05`) — tarea de verificación (sin edición)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** CI/CD / DevOps (`campaign_detect_task_type` → `devops`, checks: `yamllint .github/`)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-24
- **last-synced:** 2026-09-24
- **Estado:** ✅ COMPLETED (2026-09-24)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno — tarea de verificación, cero ediciones de código/config |
| Callees | GitHub Actions: `perf-bench.yml` (job `benchmark`), `gate-docs.yml` (check `Check API Docs Version`) |
| Implicaciones | No cambia contratos ni comportamiento; su resultado habilita el cierre de la ola EST (pre-0.7.0) |

## Impacto mapeado (Regla 0)

> GATE: no se edita ningún archivo en esta tarea (verificación read-only sobre runs de CI).

- **Archivos leídos (completos):** `.github/workflows/perf-bench.yml` (triggers + inputs), `docs/dev/plans/2026-09-24-estabilizacion-pendiente.md` §EST-05, `docs/dev/plans/2026-09-24-sesion-continuidad.md` §4.1.
- **Archivos referenciados hacia dentro:** n/a (sin ediciones).
- **Archivos que referencian a los editados:** n/a.
- **Veredicto impacto:** **bajo** — read-only; el único artefacto producido es este task file + registro en avance.

## Contrato
> "`benchmark` (perf-bench job) en verde en los runs del fix `4b5b137e` y posteriores (36040468608 = EN el commit del fix; 36086858245 = posterior con head `b461c9e8`) **y** `Check API Docs Version` en pass sobre el PR head `3bbfbe45`; evidencia registrada con IDs de run."

## Spec (SDD)
N/A — no es feature-add: no agrega símbolos/contratos públicos nuevos, no toca bindings/endpoints/tools. Es una verificación de CI con cero ediciones (justificación válida por Phase 1b).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el fix del bench (`4b5b137e`, `Client`/`search`) debe seguir siendo la referencia de la medición; `perf-bench` no debe volver a fallar por API vieja (Regla 11: números con bench reproducible). **El gate de regresión está INERTE (baseline vacío): verde ≠ cobertura — activación trackeada en `FIND-153`.**
- **Comandos de verificación:** `gh run list --workflow=perf-bench.yml --limit 2 --json headSha,conclusion` → los runs post-fix en `success`; `gh pr checks 222 | Select-String 'Check API Docs Version'` → `pass`.
- **Deuda pendiente:** ninguna.

## Recitation (canónico)

    contract:
      verificacion: `gh run view 36086858245 --json conclusion` → success ✅ + `gh pr checks 222` → Check API Docs Version pass ✅ + `campaign_verify_cmd` (run 36086858245 == success) → passed=true ✅ + review P2-01 ✅ (ronda 3)
      evidencia:
        - claim: perf-bench post-fix verde en 2 runs consecutivos
          evidencia: runs 36040468608 (head 4b5b137e) y 36086858245 (head b461c9e8) → conclusion=success
          confianza: alta
        - claim: el bench midió de verdad (3 iteraciones reales) — el "compare" es NO-OP por diseño (baseline vacío)
          evidencia: run 36086858245 log del step benchmark → "Ingestion Completed in 3.8135s (262.23 records/sec)" + "Vector HNSW -> p50: 0.8824 ms" (×3 iteraciones); step compare → "##[warning]No baseline stored yet (benchmarks/python_baseline.json empty)" y exit 0 (`perf-bench.yml:126-130`) — NO existe comparación de regresión en ningún run verde
          confianza: alta (medición real) / n/a (el gate de regresión NO corre — hallazgo FIND-153)
        - claim: Check API Docs Version en verde
          evidencia: PR #222 checks → pass (runs 36086918465, 36086924276)
          confianza: alta
      artefactos:
        - docs/dev/tasks/EST-05.md
      invariantes: el bench debe seguir midiendo con la API canónica (Client/search); no re-introducir API vieja en benchmarks/
      deuda: ninguna
      queda_pendiente: ninguno (review P2-01 por `vanta-review` completado en 2 rondas + confirmación final)

## Deuda técnica (Regla 6)
Sin deuda — cero ediciones.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato verificable cumple (runs verdes registrados) | ✅ |
| **Commit** | Commit atómico docs (task file + progreso), conventional | ✅ (al cierre) |
| **Release** | N/A — no aplica (sin cambios de código/config; justificado) | N/A |

## Herramientas necesarias
- `gh` (run view / pr checks), campaign MCP (`campaign_detect_task_type`, `campaign_discover_skills_v2`, `campaign_verify_cmd`), codegraph_explore (blast radius — N/A documentado).

**Skills cargadas (SDP):** `campaign-executor` (base task-system) · `progreso` (cierre) · `ci-cd-and-automation` (dominio CI/CD) · `doubt-driven-development` (gate de review/fallback). SDP automatizado devolvió 8 candidatas; N/A justificadas: `systematic-debugging` (no hay fallo a diagnosticar), `browser-testing-with-devtools` (sin navegador), `api-and-interface-design` + `spec-driven-development` (sin API pública/feature-add), `performance-optimization` (el fix ya está hecho; esta tarea solo verifica).

## Investigation Notes
- `campaign_detect_task_type`: `devops` / "CI/CD / DevOps", estimate 🟢 5-10 turns, checks `yamllint .github/` (no aplica: sin ediciones).
- `codegraph_explore "perf-bench benchmark ... ci-rust workflow"`: 42 símbolos de código no relacionados (los workflows YAML no están en el grafo) → blast radius de código N/A confirmado.
- Historial perf-bench: **4 fallos consecutivos inmediatamente previos al fix** (`b3251a0d`, `7a36d988`, `3122eea1`, `1d2dd7d8`); historia completa: **9 failures (7 develop + 2 main) + 2 success** → fix `4b5b137e` → success `36040468608` (EN el commit del fix) → success `36086858245` (posterior, head `b461c9e8`).
- **Hallazgo nuevo (review P2-01):** el gate de regresión de `perf-bench` está **inerte** — `benchmarks/python_baseline.json` vacío (`"benchmarks": {}`, `updated: 2026-08-12`) → el compare sale con warning y exit 0; un run verde NO cubre regresiones. Registrado como `FIND-153` (activar con `workflow_dispatch` + `update_baseline=true` + commit del artifact).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 → 1 (review) → 0 |
| % completado | 100% verificación; cierre pendiente de gate Review |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- **SECURITY:** no aplica — read-only sobre runs de CI; sin trust boundaries ni inputs.
- **PERFORMANCE:** no aplica directamente — la tarea VERIFICA el bench (el fix de performance ya se hizo en `4b5b137e`); evidencia = runs verdes.

## Steps

### Step 1: Evidencia perf-bench post-fix
- **Archivos:** n/a (read-only)
- **Acción:** listar runs de `perf-bench.yml` y confirmar success en los posteriores a `4b5b137e`
- **Verify:** `gh run list --workflow=perf-bench.yml --limit 2` → success/success (con `--limit 3` la 3ª fila es un failure pre-fix)
- **Evidencia:** `36040468608` (4b5b137e, success) · `36086858245` (b461c9e8, success) · 4 fallos consecutivos pre-fix (9 en historia)
- **Estado:** ✅ COMPLETED

### Step 2: Confirmar que el bench midió de verdad (compare no-op documentado)
- **Archivos:** n/a (read-only)
- **Acción:** inspeccionar jobs/steps del run más reciente
- **Verify:** `gh run view 36086858245 --log` → 3 iteraciones reales con métricas + warning `No baseline stored yet` (compare no-op)
- **Evidencia:** steps verdes + log: "Ingestion Completed in 3.8135s (262.23 records/sec)" ×3 iteraciones reales; **compare = no-op documentado** (`##[warning]No baseline stored yet…` → exit 0)
- **Estado:** ✅ COMPLETED

### Step 3: Confirmar `Check API Docs Version` en PR #222
- **Archivos:** n/a (read-only)
- **Acción:** leer checks del PR
- **Verify:** `gh pr checks 222 | Select-String 'Check API Docs Version'` → pass
- **Evidencia:** pass ×2 (runs 36086918465, 36086924276)
- **Estado:** ✅ COMPLETED

### Step 4: Review por agente distinto (GATE P2-01) + cierre progreso
- **Archivos:** este task file + `Backlog.md` + `avance/activo/operaciones.md`
- **Acción:** review adversarial (ARTIFACT+CONTRACT, sin CLAIM) por agente distinto; registrar veredicto; remover fila del Backlog; registrar avance
- **Verify:** veredicto registrado en §Review + `validate-docs-coverage.ps1` 0 gaps
- **Estado:** ✅ COMPLETED (review ✅ ronda 3; coverage 0 gaps; fila removida del Backlog; avance registrado)

## Dependencias
- `EST-02`/`EST-10` ✅ (fixes previos de API en bench/tests) — completadas.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, sesión `ses_f2989599affe37zDBkd68gbsQa`) — ronda 1: **❌ REQUEST CHANGES** (1 Critical + 1 Required + 1 Optional + ~4 Nits); ronda 2: **❌ 1 required pendiente** (fila `FIND-153` sin registrar) + nits — aplicados; ronda 3: confirmación final.
- **Enfoque:** ¿la evidencia de runs demuestra el contrato sin auto-reporte? ¿alternativa mejor de verificación?
- **Cómo se probó:** IDs de run + steps inspeccionados vía `gh` (no auto-reporte); reproducibles con los comandos del Contrato. El revisor re-ejecutó todo (`gh run list/view`, `gh pr checks`, `git merge-base`, `git show`) y verificó ambos clauses del contrato de forma independiente (PASS).
- **Checklist anti-hábitos tóxicos:** sin invención de salidas (todo `gh` ejecutado en sesión) · sin declarar done sin verificar · sin ignorar fallos (los 9 failures históricos documentados) · sin huérfanos.
- **Veredicto:** ronda 1 ❌ → cambios → ronda 2 ❌ (FIND-153 pendiente) → cambios → **ronda 3 ✅ APPROVE** (gate P2-01 cerrado)

## Notas
- El "benchmark" de la sesión-continuidad §4.1 era el job del workflow `PERF: Benchmarks — Python Integration` (**push con paths + `workflow_dispatch`; SIN `schedule`**), no un check de PR — de ahí que se verificara por `gh run list`, no por `gh pr checks`.
- Los 4 fallos consecutivos pre-fix quedan como historial (9 en total); la verificación exige ≥1 run post-fix verde (hay 2).
- Contexto PR: `Check API Docs Version` está verde sobre el **PR head `3bbfbe45`**; PR #222 global sigue **BLOCKED** con checks no relacionados pendientes (Clippy, SemVer, CodeQL Analyze, Desktop, Fuzz).
