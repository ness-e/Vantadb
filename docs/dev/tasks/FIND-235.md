---
title: "TASK FIND-235: `ci-rust-10.md` — conteos reales (21 jobs, cobertura ≥80%) + nota keep-in-sync"
kind: task
description: "Actualizar el runbook del workflow principal: 21 jobs y cobertura ≥80% derivados del YAML real (decía 13/59%), timeouts/comandos/paths corregidos, nota keep-in-sync con link a ci-rust.yml; gates check-links/check-docs exit 0"
---

# TASK FIND-235: `ci-rust-10.md` — conteos reales (21 jobs, cobertura ≥80%) + nota keep-in-sync

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 6, F0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-235; flagged por FIND-231, review ronda 1, 2026-10-03)
- **Esfuerzo:** 🟢 30min | **Appetite:** max 1h
- **Prioridad:** 🟢
- **Tipo:** Docs (runbook CI) — blast radius: 1 doc + task file + 2 artefactos generados (`docs/index.md`, `llms.txt`)
- **Turns estimados:** 5-10
- **Creado:** 2026-10-04T02:52 | **last-synced:** 2026-10-04T03:05
- **Estado:** ✅ COMPLETED (commit local; ACCEPT del orquestador pendiente — review P2-01)
- **Campaign ID:** master-plan-0.9.0-20261004
- **Incógnitas (uphill):** 0 — conteos derivados mecánicamente del YAML (§Evidencia)
- **Pendientes (downhill):** 0 (Steps 1-3 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/index.md` (índice generado — regenerado por cambio de `description`); `docs/dev/master-index.md:275` (link directo, sin cambio); referencias históricas en `docs/dev/avance/**` + `docs/CHANGELOG.md` (registros, NO se tocan) |
| Callees | `.github/workflows/ci-rust.yml` — fuente de verdad de los conteos (solo lectura, sin cambios) |
| Implicaciones | Docs-only: cero cambios de comportamiento. `gen-index --write` regeneró `docs/index.md` + `llms.txt` (sweep convencional: incluye filas de FIND-233/234 en vuelo). Sin API, performance ni datos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/workflow/ci-rust-10.md` (64L — objeto del fix), `.github/workflows/ci-rust.yml` (767L — fuente de verdad; 21 jobs derivados), `docs/dev/tasks/FIND-231.md` (206L — origen del flag + formato task CI), `.opencode/rules/release-ci.md` (42L — Reglas 2/5), `.opencode/task-system/prompts/task.md` (398L — formato canónico), `docs/dev/workflow/ci-web-11.md` (40L — estilo hermano), `scripts/docs/check-links.mjs` (L1-100 — reglas de links).
- **Archivos referenciados hacia dentro (inbound):** grep `ci-rust-10` → `docs/index.md` (generado), `docs/dev/master-index.md:275`, `docs/dev/operations/ci-cd-guide.md:69,86` (refs a `ci-rust-10.yml` — filename viejo del workflow, drift colateral FUERA de scope), `docs/dev/operations/TEST_MAP.md:17,62,99` (`ci-rust-10` + coverage ≥59% stale — fuera de scope), avance/historial/CHANGELOG (históricos — no se tocan).
- **Referencias salientes de los editados:** `ci-rust-10.md` no tenía links; se agrega 1 relativo al workflow (`../../../.github/workflows/ci-rust.yml`), validado por `check-links`.
- **Veredicto impacto:** **BAJO** — docs-only, correctivo; `git revert` de 1 commit restaura. No toca el workflow, plan file, `opencode.jsonc` ni WIP de otros workers.

## Evidencia dura (derivación de conteos — pre-mortem 1)

### 1. Conteo de jobs = 21 (el doc decía 13)

```powershell
$lines = Get-Content .github/workflows/ci-rust.yml
$start = ($lines | Select-String -Pattern '^jobs:').LineNumber
($lines[$start..($lines.Count-1)] | Where-Object { $_ -match '^  [a-z0-9-]+:$' }).Count   # → 21
```

Lista: fmt, clippy, semver-checks, public-api-snapshot, adr-gate, test, test-windows, test-macos, msrv, minimal-versions, coverage, wasm-test, experimental-check, release-combo, audit, osv, machete, miri, deny, sanitizer-asan, sanitizer-tsan.
**Los 8 que faltaban en el doc:** semver-checks, public-api-snapshot, adr-gate, wasm-test, experimental-check, release-combo, osv, machete.

### 2. Coverage threshold = ≥80% (el doc decía 59%)

`.github/workflows/ci-rust.yml:411` (`Enforce coverage threshold (>=80%, ADR-015 coverage policy)`) + `:427` (`ok = pct >= 80.0`).

### 3. Drift adicional derivado del YAML (corregido en el doc)

| Dato | Doc previo | YAML real |
|------|-----------|-----------|
| `clippy` | `--exclude vantadb-wasm` | `--workspace --all-targets --all-features` sin excludes (AUD-018, `:90`) |
| `test-windows` timeout | 30m | 60m (`:264`) |
| `test-windows`/`test-macos` clippy excludes | solo wasm | wasm+server+mcp (`:290`, `:323`) |
| `minimal-versions` timeout | 15m | 30m (`:349`) |
| Paths de trigger | sin `osv-scanner.toml`, `vanta-memory/**`, `providers/**` | presentes (`:16-21`, `:34-39`) |
| Miri | "(continue-on-error)" | Stacked Borrows gatea; Tree Borrows best-effort (`:634-640`) |
| ASan skips | — | salta `test_benchmark_internal_10k` (`:713`) |

## Contrato (del plan)

> conteos reales (jobs/coverage) **derivados del YAML real** + links válidos; `check-docs`/`check-links` exit 0; nota "keep in sync" con el workflow agregada.

| # | Condición | Verificación | Estado |
|---|-----------|--------------|--------|
| 1 | Doc dice 21 jobs + lista completa; cobertura ≥80% | conteo mecánico §Evidencia + revisión del diff | ✅ |
| 2 | Nota "keep in sync" con link al workflow | presente en el doc (L11); `check-links` resuelve el link | ✅ |
| 3 | Gates | `node scripts/docs/check-links.mjs` exit 0 · `node scripts/docs/check-docs.mjs` exit 0 · `gen-index --check` exit 0 (tras `--write`) | ✅ |

## Spec (SDD)

N/A — tarea 100% docs sin decisiones técnicas abiertas (el conteo se deriva mecánicamente; comando en §Evidencia). Las 2 micro-decisiones de presentación se resolvieron por evidencia:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Alcance del fix | (a) solo "13→21" y "59→80" / (b) barrido de todo el drift derivable del YAML (timeouts, comandos, paths, precisión Miri/ASan) | ✅ (b) — la tabla del doc expone Timeout/Qué ejecuta; un dato mal ahí es el mismo defecto del FIND |
| 2 | Anti-drift | nota "keep in sync" sola / nota + link relativo al workflow | ✅ nota + link — el link hace la fuente verificable (check-links lo valida) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `.github/workflows/ci-rust.yml` solo lectura — NO editar; (2) NO tocar plan file, `opencode.jsonc`, `docs/pipeline-state.json` ni WIP de otros workers (`FIND-233.md` untracked, `perf-bench.yml`/`compare_baseline.py` en vuelo); (3) commit **LOCAL**, sin push (política owner); (4) el doc sigue siendo `kind: runbook` con H1 = title; (5) `docs/index.md`/`llms.txt` solo vía `gen-index --write` (nunca a mano).
- **Comandos de verificación:** conteo jobs (§Evidencia → 21) · `node scripts/docs/check-links.mjs` (exit 0) · `node scripts/docs/check-docs.mjs` (exit 0) · `node scripts/docs/gen-index.mjs --check` (exit 0).
- **Deuda pendiente:** ninguna. Colaterales anotados en §Notas (decisión de ticketing = orquestador).

## Deuda técnica (Regla 6)

**Saldo neto de deuda por PR:** Sin deuda (docs-only; 0 código, 0 deps). El fix REDUCE deuda: elimina el drift del runbook de referencia + agrega guard de sincronización (nota + link).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato 1-3: conteos derivados + nota/link + gates exit 0 | ✅ |
| **Commit** | `docs(ci): FIND-235 — …` atómico, verify mecánico, commit LOCAL sin push | ✅ |
| **Release** | n/a (docs-only) | — |

## Herramientas necesarias

- `node scripts/docs/{check-links,check-docs,gen-index}.mjs` · PowerShell (derivación YAML) · OCR delegation (`dev-tools/ocr-review.ps1`).

**Skills cargadas (SDP):** `documentation-skill` (obligatoria — `.md` bajo `docs/`; sintaxis de links, frontmatter, gates), `ci-cd-and-automation` (pinned CI/release — dominio del runbook), `git-workflow-and-versioning` (pinned CI/release — commit conventional local), `doubt-driven-development` (base CI/CD — self-check adversarial del diff), `campaign-executor` + `progreso` (base, auto vía MCP). SDP v3 `campaign_discover_skills_v2` phase=BUILD → 8 candidatas; las lifecycle genéricas (incremental-implementation / test-driven-development / context-engineering) no son materiales para un diff 100% docs sin lógica (descartadas con motivo).

## Investigation Notes

- Derivación de conteos: ver §Evidencia (comandos re-ejecutables). Sin web research (n/a — todo derivable del repo).
- `campaign_verify_cmd` reproduce el bug conocido (exit -1, stdout vacío, `spawnError=false`) → gates vía shell directa (precedente FIND-92/FIND-134), documentado.
- `gen-index`: staleness pre-existente por FIND-233/234 en disco (índice committed 1490 docs vs árbol 1493 tras este task file); el `--write` del cierre los barrió (convención FIND-231/239).
- OCR delegation: mis 4 archivos quedan excluidos por `unsupported_ext` (docs); los 2 reviewables del preview (`perf-bench.yml`, `compare_baseline.py`) pertenecen a FIND-233 (worker paralelo, fuera de scope) → 0 Critical/High atribuibles a FIND-235.
- `validate_scope`: doc ✅ in-scope; `FIND-235.md` OUT_OF_SCOPE (artefacto task-system no listado en "Archivos clave" del plan — mandato de pipeline-full §Discovery, mismo caso que FIND-239).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — conteos derivados del YAML; formato/link validados contra hermanos y `check-links` |
| Pendientes de ejecución (downhill) | 0 — Steps 1-3 ✅ |
| % completado | 100% |

## Steps

### Step 1: Actualizar `ci-rust-10.md` (conteos, listas, nota keep-in-sync)
- **Archivos:** `docs/dev/workflow/ci-rust-10.md`
- **Acción:** 13→21 jobs (tabla completa con los 8 faltantes), coverage 59→≥80%, timeouts/comandos/paths derivados del YAML, nota "keep in sync" con link al workflow.
- **Verify:** `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs` exit 0.
- **Resultado:** ✅ — doc reescrito (78L); gates verdes en Step 2.
- **Estado:** ✅

### Step 2: Task file + regenerar índices
- **Archivos:** `docs/dev/tasks/FIND-235.md`, `docs/index.md` + `llms.txt` (generados)
- **Acción:** crear task file canónico; `node scripts/docs/gen-index.mjs --write`; `--check` exit 0.
- **Verify:** `node scripts/docs/gen-index.mjs --check` exit 0.
- **Resultado:** ✅ — `--write` regeneró `docs/index.md` (+7/−4) y `llms.txt` (+2/−2) con las filas de FIND-233/234/235; `--check` exit 0.
- **Estado:** ✅

### Step 3: Cierre (OCR + evidencia P2-01 + commit local)
- **Archivos:** `docs/dev/tasks/FIND-235.md` (evidencia final), commit
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (Critical/High bloquean); evidencia §Review para el orquestador (leaf sin tool `task`); commit LOCAL `docs(ci): FIND-235 — …` (sin push).
- **Verify:** OCR sin Critical/High propios; `git show --stat` sin archivos ajenos.
- **Resultado:** ✅ — OCR exit 0 (2 reviewables = FIND-233; propios excluidos por `unsupported_ext`); evidencia §Review completa; commit local `docs(ci): FIND-235` (hash en RESULTADO §7 / git log).
- **Estado:** ✅

## Dependencias

- F0 — sin dependencias. nextTask: FIND-236.

## Review (GATE — agente distinto, P2-01)

> Soy leaf (sin tool `task`): evidencia preparada para el orquestador; veredicto pendiente.

- **Paths del diff:** `docs/dev/workflow/ci-rust-10.md`, `docs/dev/tasks/FIND-235.md`, `docs/index.md`, `llms.txt` → **Tier Fast** (docs; ningún glob adversarial).
- **Evidence pack:**
  - **Gates (re-ejecutables):** `check-links` exit 0 (44 broken pre-existentes, budget 58) · `check-docs` exit 0 (GATING all clear) · `gen-index --check` exit 0 (post `--write`).
  - **Derivación independiente:** comando §Evidencia → 21 jobs; threshold ≥80% en YAML:411/427; re-derivable por el revisor sin contexto extra.
  - **OCR delegation:** exit 0; 0 archivos propios reviewables (docs `unsupported_ext`); los 2 reviewables son de FIND-233.
  - **Diff acotado:** 4 paths (doc + task file + 2 generados); sin archivos ajenos.
- **Enfoque sugerido al revisor:** ¿el conteo 21 es re-derivable del YAML? ¿la nota keep-in-sync es suficiente contra re-drift? ¿queda algún dato del doc sin derivar del YAML?
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (comandos ejecutados y registrados); sin done sin gates; sin scope creep (colaterales anotados, no tocados); cada step conectado al contrato.
- **Veredicto:** ⬜ pendiente (`vanta-review` por el orquestador → ACCEPT HARD-07).

## Notas

- **Colaterales NOT TOUCHING (decisión orquestador):** `ci-cd-guide.md:69,86` referencia `ci-rust-10.yml` (filename viejo del workflow); `TEST_MAP.md:62,99` dice `ci-rust-10` + coverage ≥59%. Ambos fuera del blast radius de FIND-235.
- **gen-index:** staleness pre-existente (FIND-233/234 en disco); el `--write` del cierre los barre (convención FIND-231/239: índices en commits de cierre).
- **campaign_verify_cmd:** bug conocido (exit -1 vacío) → gates vía shell directa; documentado.
- **Pre-mortem del plan cubierto:** (1) conteo mal derivado → derivado con comando mecánico (§Evidencia); (2) re-drift → nota + link; (3) links → `check-links` exit 0.
- **SECURITY / PERFORMANCE:** n/a — docs-only (sin trust boundaries, sin hot paths).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ — doc reescrito (21 jobs/≥80%/keep-in-sync), task file creado, índices regenerados, gates 0/0/0, OCR exit 0, commit local `docs(ci): FIND-235`.
- **Próximo step:** review P2-01 (`vanta-review` por el orquestador) → ACCEPT HARD-07.
- **Archivos en vuelo:** ninguno.

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 3/3 total steps
PROXIMO_STEP: review P2-01 (vanta-review por el orquestador) → ACCEPT HARD-07
COMMIT_HASH: ver git log — commit local `docs(ci): FIND-235` (nunca push)
ARCHIVOS: docs/dev/workflow/ci-rust-10.md, docs/dev/tasks/FIND-235.md, docs/index.md + llms.txt (generados)
VERIFY_CONTRATO: pasa (conteo 21 vs YAML · check-links exit 0 · check-docs exit 0 · gen-index --check exit 0)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no (docs-only sin símbolos públicos) D:no (plan F0 full-detail aprobado) V:no (sin fallas de verify) C:no (colaterales anotados en §Notas, sin fix inline)
SKILLS_CARGADAS: documentation-skill · ci-cd-and-automation (pinned) · git-workflow-and-versioning (pinned) · doubt-driven-development (+ base campaign-executor/progreso/ponytail auto vía MCP)
```
