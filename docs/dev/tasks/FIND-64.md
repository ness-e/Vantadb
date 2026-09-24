# FIND-64 — `'vanta-memory/**'` en paths CI (`ci-rust-10.yml` push+PR)

> **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 3, Wave0 — disjunto de FIND-90/91: workflow vs Rust)
> **Estado:** ⏳ IN PROGRESS (re-scope CI 2026-09-15; scope previo 2026-09-07 archivado abajo, NO continuar esos steps)
> **Appetite:** 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴 Alta
> **Branch:** `develop` · **Commit:** `ci: FIND-64 — ...` (solo tras verify mecánico)
> **Ruta:** vanta-lead (CI) · **nextTask:** FIND-63 (Wave1)
> **SDP:** `campaign_discover_skills_v2` archivosClave=`.github/workflows/ci-rust-10.yml` phase=BUILD
> contractKeywords=[github-actions, paths, vanta-memory, actionlint] → base
> `ci-cd-and-automation`, `doubt-driven-development`, `campaign-executor`, `progreso`
> (+ lifecycle; `frontend-ui-engineering`/`api-and-interface-design` descartadas por irrelevantes al scope YAML/CI).
> Sugeridas del plan: `ci-cd-and-automation` ✅ cargada, `git-workflow-and-versioning` ✅ cargada (commit `ci:`).
> **SKILLS_CARGADAS:** ci-cd-and-automation, git-workflow-and-versioning (+ base campaign-executor, progreso, ponytail full)
> **Referencias:** `.opencode/rules/release-ci.md` (leída — reglas 2/5 aplican: sin sccache duplicado, sin `continue-on-error` nuevo),
> `docs/dev/operations/CI_POLICY.md` (contexto two-tier Fast Gate; paths citados en `:157,438`). Sin símbolos → sin Spec.
> **Deuda previa (fuera de scope, no tocar):** `DeprecationWarning` `vantadb_py` en adapters (viene del scope 2026-09-07).

## HALLAZGO — divergencia task file STALE (reportada, no silenciada)

El task file describía scope 2026-09-07 (fix `integrations/llamaindex` `put_batch` legacy, 3/3 steps DONE, 23 passed,
sin commit por regla lead). El plan vigente 2026-09-15 re-scopeó FIND-64 a gap CI (`ci-rust-10.yml` paths) y el
Backlog `:207` ya refleja el scope CI. Acción: DISCOVERY completo para el scope CI actual + sección legacy
archivada al pie ("Scope previo 2026-09-07 ya cerrado"). No se continúan steps viejos, no se toca
`integrations/llamaindex/` (prohibido en este scope).

## Contrato

- [ ] Entrada `'vanta-memory/**'` presente en `on.push.paths` Y `on.pull_request.paths` de `ci-rust-10.yml`
- [ ] `actionlint` verde sobre el workflow editado
- [ ] Parse YAML OK (`python -c "import yaml"`)
- [ ] `git diff --check` limpio + sin typo que rompa el trigger
- [ ] Grep global: ningún otro workflow con el mismo gap sin ticket (gemelo → nuevo FIND-*, no scope-creep)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/ci-rust-10.yml` (608L — triggers `:3-36`, resto jobs solo contexto),
  `.github/workflows/ci-rustdoc.yml:16-35` (gap gemelo), `docs/dev/Backlog.md:204-207` (fila FIND-64 = scope CI),
  `.opencode/rules/release-ci.md` (42L), `.opencode/task-system/prompts/findings.md` (routing hallazgos),
  `Cargo.toml:704,716` (membresía `vanta-memory` en workspace).
- **Referencias hacia dentro (qué usa el trigger):** `on.push.paths` (`:6-19`) y `on.pull_request.paths` (`:22-35`);
  ambas listas contienen `'vantadb-*/**'` (`:17,33`) + `'integrations/**'` pero NO `'vanta-memory/**'`.
- **Referencias entrantes (quién dispara el workflow):** pushes a `main`/`develop` y PRs a `main` cuyos archivos
  matcheen `paths`. Hoy un push que toca SOLO `vanta-memory/**` no matchea ningún patrón → Fast Gate ciego.
- **Veredicto:** blast radius = 1 archivo + 2 bloques trigger (2 líneas añadidas, mismo patrón existente).
  Sin cambio de API pública, sin símbolos nuevos, sin hot path, sin cargo build (tarea YAML/docs).
  Riesgo 🟢 mínimo; riesgos residuales: typo YAML (mitiga `actionlint`+parse) y gap gemelo (mitiga grep global+FIND ticket).
- **Gate D:** no dispara (≤10 archivos, sin hot path/API pública/símbolos nuevos, contrato mecánico) — fix directo.

## Root cause

`vantadb-*/**` no matchea `vanta-memory/`: el segmento literal difiere (`vantadb-` vs `vanta-…`; 6º char `d` vs `-`).
`vanta-memory/` existe en repo raíz y es workspace member (`Cargo.toml:704,716`), por lo que compila/testea en
`--workspace` pero sus cambios en solitario nunca disparan `ci-rust-10.yml`. Fix: 2 líneas (un patrón por trigger).

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | DISCOVERY scope CI + re-scope task file (Regla 0, contrato, steps) + archivar legacy | ✅ DONE | este task file |
| 2 | Fix: agregar `'vanta-memory/**'` tras `'vantadb-*/**'` en push y PR | ✅ DONE — `git diff` muestra exactamente +2 líneas (`:18` push, `:35` PR) | `git diff` 2 líneas |
| 3 | Verify contrato: YAML parse + `actionlint` + `git diff --check` + grep global gap gemelo | ✅ DONE — YAML OK (ambos triggers listan `vanta-memory/**`); `actionlint` exit 0 sin findings; `git diff --check` limpio; `rg vanta-memory .github/workflows/` = 0 hits pre-fix | comandos abajo |
| 4 | Hallazgo gap gemelo (`ci-rustdoc.yml:24,33`) → fila `FIND-92` en Backlog (no scope-creep) | ✅ DONE — fila añadida tras FIND-91 (findings.md: ticket en el momento, no solo anotado) | `rg FIND-92 docs/dev/Backlog.md` |
| 5 | Commit `ci:` (solo workflow + task file + Backlog) + `campaign_update_task_state` completed + RESULTADO | ✅ DONE — commit `ci: FIND-64` con exactamente 3 archivos (fuera de scope intacto: `.opencode/`, `completions/`, `Cargo.lock`, `integrations/llamaindex/`) | hash + bloque |

## Pre-mortem (del plan — verificado en DISCOVERY)

1. Otra workflow con el mismo gap → **confirmado**: `ci-rustdoc.yml:24,33` usa `vantadb-*/**` sin `vanta-memory/**`
   (+ `rg vanta-memory .github/workflows/` = 0 hits: ningún workflow la menciona). Va a FIND-92, no al diff.
2. Path con typo rompe el trigger → validar con `actionlint` (disponible: `actionlint.exe`) + parse YAML (pyyaml OK).

## Herramientas

- Lectura: `Get-Content .github/workflows/ci-rust-10.yml`
- Gap gemelo: `rg -n "vantadb-\*/\*\*|vanta-memory" .github/workflows/`
- Verify: `python -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci-rust-10.yml'))"`,
  `actionlint .github/workflows/ci-rust-10.yml`, `git diff --check`
- Cierre: `campaign_verify_cmd`, `campaign_update_task_state`
- Prohibidos (M): `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, GOV-C4 stash,
  `integrations/llamaindex/` (scope viejo, no tocar). `git status` trae M fuera de scope → commit solo 3 archivos.

## Context Save Point

- 2026-09-15 DISCOVERY (scope CI): plan + Backlog `:207` + workflow + `ci-rustdoc.yml` + `release-ci.md` +
  `findings.md` leídos; `campaign_detect_task_type` = devops; workflow bug-fix cargado;
  `ci-cd-and-automation` + `git-workflow-and-versioning` cargadas; branch `develop` ✔;
  `actionlint.exe` + pyyaml disponibles ✔; `vanta-memory` member `Cargo.toml:704,716` ✔.
  Siguiente: Step 2 (edit 2 líneas).

---

## Scope previo 2026-09-07 ya cerrado (legacy — archivado, no continuar)

> Plan origen: `docs/dev/plans/2026-09-07-backlog-triage.md` (Task 1, Wave0). Scope: fix `put_batch` legacy roto en
> `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:130-134` (6-tuplas posicionales vs firma kwargs actual
> `vantadb-python/src/lib.rs:484`). Estado final: 3/3 steps ✅ DONE — repro `TypeError` confirmado, fix a columnas
> directas con kwargs, verify 23 passed + grep kwargs ✅. Diff listo SIN commit (regla: solo vanta-lead commitea).
> NOTICED BUT NOT TOUCHING entonces: `DeprecationWarning 'vantadb_py' → 'import vantadb'` (`vectorstore.py:7`,
> migrar en tarea aparte). Este scope se da por cerrado a nivel task file; el commit del diff legacy queda fuera
> del presente scope CI (archivo prohibido en esta tarea).
