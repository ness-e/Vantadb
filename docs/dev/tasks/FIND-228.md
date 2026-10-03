---
title: "TASK FIND-228: Dedupe de triggers CI (drop `develop` de `push.branches` en 15 workflows)"
kind: task
description: "Quitar develop de push.branches en 15 workflows CI/demo/gate (regla RULES.md §1) + matriz TRIGGERS.md al día + actionlint 0 — corrección de compliance, no cambio de política"
---

# TASK FIND-228: Dedupe de triggers CI (drop `develop` de `push.branches` en 15 workflows)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 5, Wave 2)
- **Fuente:** `docs/dev/Backlog.md` (FIND-228; duplicados medidos en PR #233)
- **Esfuerzo:** 🟡 3-5h | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** CI/CD-DevOps (15 workflows YAML + 3 runbooks; blast radius CI-only)
- **Turns estimados:** 15-30
- **Creado:** 2026-10-02T23:55 | **last-synced:** 2026-10-03T00:04
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE — `vanta-review` sesión `ses_f0015bdd2ffeaDR53sO9f8Xiir`; 2 Optional plegados: comentario `ci-rustdoc` + conteo 22 líneas; nits pre-existentes documentados)
- **Incógnitas (uphill):** 0 — cerradas (inventario + semántica validadas)
- **Pendientes (downhill):** 0 — 5/5 steps completos (commit local en este cierre)
- **Campaign ID:** post-release-0.8.0-20261002

## Contrato

`rg -n 'branches:.*develop' .github/workflows -g '*.yml'` → **solo líneas bajo `pull_request:`** de los 15 workflows verificados por lectura (+ la excepción documentada `perf-bench.yml` push — excluido por FIND-232) + `docs/dev/workflow/TRIGGERS.md` actualizado a la matriz nueva + `actionlint` exit 0 + **un push de prueba a develop NO dispara los workflows arreglados** (el push-test es del owner → comandos exactos de verificación post-push en §Notas).

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)

**No aplica — Phase 1b mecánica: 0 señales.** El cambio no agrega `pub fn`/struct/enum, tool MCP, endpoint HTTP/CLI, método de binding, componente consumible ni capability de usuario: solo quita `develop` de `push.branches` en 15 YAML y actualiza 3 runbooks. No hay decisiones de contrato público abiertas (la regla ya existe en `RULES.md` §1; esto es compliance).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (consumidores del trigger) | GitHub Actions dispatch: eventos `push` (ref `refs/heads/develop`), `pull_request` (ref `refs/pull/N/merge`), `schedule`, `workflow_dispatch`. Branch protection de `main`/`develop` (required checks de PR — no afectados: corren por evento PR). `ci-gate.yml` (reusable): sus REQUIRED names son todos de `ci-rust.yml` + `Analyze` (CodeQL) — ninguno de los 15 (verificado por lectura de `ci-gate.yml:37-51`) |
| Callees | `./.github/actions/rust-setup` (composite), `scripts/docs/*.mjs`, `scripts/demo-*-e2e.ps1`, jobs internos — sin cambios |
| Implicaciones | Sin cambio de contrato público ni de jobs. Se elimina el par duplicado push+PR por SHA en develop (RULES §1). Artefactos: `api-reference-rust` (ci-rustdoc) y bundles desktop **sin consumidores internos** (grep 2026-10-02 + FIND-137 §43) → cobertura preservada vía PR; sin migración de datos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** los 15 workflows (`.github/workflows/{chaos,ci-ai-ides-demo,ci-examples,ci-frameworks-demo,ci-rustdoc,desktop,gate-api-docs,gate-doc-examples,gate-docs-links,gate-docs-secrets,gate-docs,icp02-privacy-demo,injection-governance-demo,providers-ci,wal-verify-demo}.yml`), `docs/dev/workflow/TRIGGERS.md` (71L), `docs/dev/workflow/RULES.md` (218L), `docs/dev/workflow/FAQ.md` (57L), `.opencode/rules/release-ci.md` (42L), `.github/workflows/ci-gate.yml` (75L), `perf-bench.yml` (on:, read-only), `.githooks/pre-commit` (87L)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `./.github/actions/rust-setup` (todos los jobs), `scripts/docs/{check-*,gen-index}.mjs` (gates), `scripts/demo-{privacy,governance,verify}-e2e.ps1` (demos), `scripts/north_star_metric.py`, `scripts/check_openapi_parity.mjs` — ninguno lee el trigger; todos intactos
- **Archivos que referencian a los editados (referencias entrantes):** `TRIGGERS.md` (matriz — se actualiza en el mismo commit), `RULES.md` §1 (regla — amendment se actualiza), `FAQ.md` (mitigación FIND-134 — se actualiza), `ci-gate.yml` (no referencia nombres de estos workflows — verificado), branch protection (required: jobs de `ci-rust.yml`, `gate-docs.yml`, `providers-ci.yml`, CodeQL — vía PR, intactos). `rg 'refs/heads/develop|github\.ref ==|head_ref'` en los 15 → **0 hits** (ningún job condiciona por rama)
- **Veredicto impacto:** **bajo** — el único efecto es que un push a `develop` deja de disparar los 15 (el disparo de PR se conserva; el push a `main` se conserva). Sin artefactos huérfanos, sin gates rotos, sin condicionales de rama.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) NO tocar `perf-bench.yml` (dependencia de verificación post-push de FIND-232); (2) NO tocar `ci-rust.yml` (FIND-226/227/231 en vuelo); (3) `pull_request.branches` de los 15 se conserva exactamente como está (es la vía de validación pre-merge); (4) commit LOCAL, nunca push; (5) no tocar `opencode.jsonc`, plan file, `docs/pipeline-state.json`, `benchmarks/**`
- **Comandos de verificación:** `rg -n 'branches:.*develop' .github/workflows -g '*.yml'` (solo PR + perf-bench), `actionlint` (exit 0), `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` (exit 0)
- **Deuda pendiente:** ninguna (push de prueba post-push es del owner; comandos documentados abajo)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (se eliminan duplicados; no se introduce nada nuevo).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file: `rg` filtrado + `actionlint` 0 + TRIGGERS/RULES/FAQ coherentes. Capa determinista aplicable = actionlint + gates docs (no hay código Rust en el diff → fmt/clippy/nextest N/A, justificado en Notas) |
| **Commit** | Commit atómico (~16 líneas de diff en workflows + docs), `ci:` conventional + task ID, verificación mecánica (nunca auto-reporte) |
| **Release** | N/A — no es release de crate/paquete; el "release" aquí = TRIGGERS.md/RULES coherentes con los `on:` reales (verificado por lectura + rg) |

## Herramientas necesarias

- `actionlint` v1.7.12 (instalado; hook pre-commit lo corre repo-wide)
- `rg` (inventario/contrato), `git` (diff/commit local), `node scripts/docs/*.mjs` (gates docs)
- `pwsh dev-tools/ocr-review.ps1` (OCR delegation, cierre)
- campaign MCP (verify_cmd, update_task_state, memory_write)

**Skills cargadas (SDP):** `ci-cd-and-automation` (pipeline/gates — núcleo de la tarea), `git-workflow-and-versioning` (commit conventional + política develop/main), `documentation-skill` (obligatoria: se editan docs/ + task file + índice), `doubt-driven-development` (verificación adversarial del cambio de triggers), `security-and-hardening` (pinned policy — evaluada: N/A trust boundary, ver §Fases), `deprecation-and-migration` (pinned policy — evaluada: retiro de eventos duplicados, no de código), `incremental-implementation` (edición por lotes con verify), `coordinated-web-search` (validación docs oficiales GitHub) + base auto-cargada (campaign-executor, progreso).

## Investigation Notes

- **Semántica de triggers (docs oficiales GitHub, fetched 2026-10-02):** `pull_request.branches` filtra por rama **base** del PR; `GITHUB_REF` del evento PR es `refs/pull/N/merge` — **distinto** de `refs/heads/develop` del evento push → con `concurrency.group` basado en `github.ref` (patrón del repo) ambos runs corren y no se cancelan. Fuente: <https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows> (§`pull_request`, §`push`). Confirma el par duplicado medido en PR #233 (~80 checks vs ~45 esperados) y que quitar `develop` de `push.branches` no afecta la validación pre-merge (la PR sigue disparando).
- **Inventario 2026-10-02 (rg + lectura uno a uno):** 15 workflows con `push.branches: [main, develop]` + `pull_request` (9 con PR `[main]`, 6 con PR `[main, develop]`). `perf-bench.yml` también lista develop pero está **excluido** (FIND-232). Ninguno condiciona jobs por rama (`refs/heads` scan = 0 hits). `ci-rustdoc` genera artifact `api-reference-rust` (30d) **sin consumidores internos** (grep + FIND-137): se preserva en PR y push a main.
- **Matriz TRIGGERS.md stale:** faltan 10 workflows (los 9 demo/gate de este set + `lurkr-informational`), y la fila `ci-web.yml` apunta a un archivo **eliminado** en `82317140` (W-05: extracción de `web/` a `ness-e/Vantadb-web`). Se corrige la fila muerta y se agregan los 9 de este set; `lurkr-informational` queda fuera de scope (no está en los 15) → nota en la matriz.
- **FAQ.md** documenta el par duplicado y dice "other dual-trigger workflows still double-fire" — queda stale con el fix → se actualiza en el mismo commit (coherencia del docset).
- **RULES.md §1** tiene un amendment 2026-09-25 que declara la regla "objetivo a completar"; FIND-228 la completa → el amendment se actualiza con la excepción documentada.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — causa y alcance medidos; semántica validada contra docs oficiales |
| Pendientes de ejecución (downhill) | 5 — edición 15 workflows → TRIGGERS → RULES/FAQ → gates docs → verify+commit |
| % completado | 20% (discovery cerrado) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: el diff no toca trust boundaries, input de usuario, auth, datos ni dependencias. Reduce superficie de eventos (menos runs). Los `permissions:` de los 15 quedan intactos. Justificado.
- [x] **PERFORMANCE** — no aplica como hot path de código; el objetivo ES costo CI: elimina ~15 runs duplicados por push a develop con PR abierto (medido PR #233: ~80 checks vs ~45 esperados). No hay cambio de motor; sin benchmark de código aplicable. Justificado.

## Steps

### Step 1: Inventario + clasificación por lectura
- **Archivos:** los 15 workflows (bloques `on:`)
- **Acción:** leer `on:` de cada uno; clasificar (a) push con develop → quitar, (b) PR → se queda, (c) especiales → documentar
- **Verify:** `rg -n 'branches:.*develop' .github/workflows -g '*.yml'` (pre-fix: 22 líneas en 16 archivos = 15 push + 6 PR duales + perf-bench; post-fix: 6 PR + perf-bench)
- **Estado:** ✅ DONE (2026-10-02; tabla en Notas)

### Step 2: Editar los 15 workflows (drop `develop` de `push.branches`)
- **Archivos:** los 15 de §Impacto (cada uno: 1 línea `push.branches` → `[main]`, estilo por archivo preservado)
- **Acción:** `develop` sale de `push.branches`; `pull_request` intacto; `schedule`/`dispatch` intactos
- **Verify:** `rg -n 'branches:.*develop' .github/workflows -g '*.yml'` → solo líneas bajo `pull_request:` + `perf-bench.yml:5` (excepción) + `actionlint` exit 0
- **Estado:** ✅ DONE (2026-10-03)

### Step 3: TRIGGERS.md a la matriz nueva
- **Archivos:** `docs/dev/workflow/TRIGGERS.md`
- **Acción:** 6 filas actualizadas (ci-examples, chaos, ci-rustdoc, desktop, gate-docs, providers-ci) + 9 filas nuevas (demos/gates) + quitar fila muerta `ci-web.yml` + notes (excepción perf-bench, duales PR develop, fecha de lectura)
- **Verify:** `rg -n 'develop' docs/dev/workflow/TRIGGERS.md` (solo notas/excepciones) + check-links
- **Estado:** ✅ DONE (2026-10-03)

### Step 4: RULES.md §1 + FAQ.md coherentes
- **Archivos:** `docs/dev/workflow/RULES.md` (§1 amendment + fuentes), `docs/dev/workflow/FAQ.md` (mitigación)
- **Acción:** amendment "objetivo" → "completado 2026-10-02 (FIND-228)" con excepción perf-bench; FAQ idem
- **Verify:** `rg -n 'FIND-228' docs/dev/workflow/RULES.md docs/dev/workflow/FAQ.md`
- **Estado:** ✅ DONE (2026-10-03)

### Step 5: Gates docs + verify + commit local
- **Archivos:** `docs/index.md`, `llms.txt` (gen-index --write por task file nuevo)
- **Acción:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write`; luego verify full (rg + actionlint + `gen-index --check`); OCR delegation; commit local `ci: FIND-228 — ...`
- **Verify:** todos exit 0 + `git show --stat HEAD` solo con los archivos de la tarea
- **Estado:** ✅ DONE (2026-10-03)

## Dependencias

- Wave 2 — sin dependencias bloqueantes (FIND-230 ya commiteó `gate-docs.yml`; base limpia). `perf-bench.yml` NO se toca (FIND-232); `ci-rust.yml` NO se toca (FIND-226/227/231).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — sesión fresca `ses_f0015bdd2ffeaDR53sO9f8Xiir` (reviewer_context ≠ author_context)
- **Enfoque:** ✅ correcto — drop de `develop` de `push.branches` con PR intacto es el patrón del precedente (`ci-rust.yml`/FIND-134); alternativas evaluadas y descartadas con razones (concurrency deja checks `cancelled` en el PR y no elimina el dispatch; `paths` no discrimina evento; guard `if:` suma código y runs skipped)
- **Cómo se probó:** ✅ evidencia real — `rg` filtrado (6 PR + perf-bench), cruce matriz↔`on:` de 37 archivos, `actionlint` 0, check-links/check-docs/gen-index --check 0/0/0, `git grep HEAD` (22 líneas/16 archivos pre-fix), 0 consumidores del artifact `api-reference-rust`, `ci-gate.yml` REQUIRED ajeno a los 15
- **Checklist anti-hábitos tóxicos:** ✅ limpio (no outputs inventados — PR #233 corroborado vía `gh pr view`; no done sin verificar — steps quedaron PENDING durante el review; sin fallos ignorados; sin pasos huérfanos; sin degradación de checks)
- **Hallazgos:** 0 Critical/High/Medium; 2 Optional **plegados** (comentario stale `ci-rustdoc.yml` → `push [main] (FIND-228)`; conteo 22 líneas corregido en este task file); 1 nit pre-existente corregido (`sec-codeql` sin paths en la frase de TRIGGERS) y 1 nits pre-existente documentado (scope RULES.md "27 files 2026-09-22" — statement fechado)
- **Veredicto:** ✅ APPROVE

## Notas

### Clasificación por workflow (2026-10-02, lectura uno a uno)

| Workflow | push.branches (antes) | pull_request.branches | Acción |
|----------|----------------------|----------------------|--------|
| `chaos.yml` | [main, develop] | [main] | drop develop de push |
| `ci-ai-ides-demo.yml` | [main, develop] | [main] | drop develop de push |
| `ci-examples.yml` | [main, develop] | [main] | drop develop de push |
| `ci-frameworks-demo.yml` | [main, develop] | [main] | drop develop de push |
| `ci-rustdoc.yml` | [main, develop] | [main, develop] | drop develop de push (PR dual se conserva — FIND-137) |
| `desktop.yml` | [main, develop] | [main] | drop develop de push |
| `gate-api-docs.yml` | [main, develop] | [main, develop] | drop develop de push (PR dual se conserva) |
| `gate-doc-examples.yml` | [main, develop] | [main, develop] | drop develop de push (PR dual se conserva) |
| `gate-docs-links.yml` | [main, develop] | [main, develop] | drop develop de push (+schedule Mon 07:23 intacto) |
| `gate-docs-secrets.yml` | [main, develop] | [main, develop] | drop develop de push (PR dual se conserva) |
| `gate-docs.yml` | [main, develop] | [main, develop] | drop develop de push (PR dual se conserva) |
| `icp02-privacy-demo.yml` | [main, develop] | [main] | drop develop de push |
| `injection-governance-demo.yml` | [main, develop] | [main] | drop develop de push |
| `providers-ci.yml` | [main, develop] | [main] | drop develop de push |
| `wal-verify-demo.yml` | [main, develop] | [main] | drop develop de push |

**Decisiones explícitas por artefacto (pre-mortem 1):** ci-rustdoc `api-reference-rust` → 0 consumidores internos; se preserva en PR (main+develop) y push main. Desktop installers (win/mac/linux) → 0 consumidores; se preservan en PR/main. Ningún workflow de los 15 depende del push[develop] para producir algo que otro consuma.

**Verificación post-push (owner — push de prueba a develop):**

```powershell
# 1. Runs disparados por el push (event=push, rama develop):
gh run list --branch develop --event push --limit 50 --json workflowName,event,headSha,conclusion
#    ESPERADO: solo 'PERF: Benchmarks — Python Integration' (perf-bench.yml — excepción FIND-232).
#    Cualquier otro nombre = regresión → revisar ese archivo.
# 2. Filtro mecánico de nombres únicos:
gh run list --branch develop --event push --limit 100 --json workflowName --jq '[.[].workflowName] | unique'
#    ESPERADO: ["PERF: Benchmarks — Python Integration"]
# 3. Con PR develop→main abierto: los checks por PR siguen corriendo (event=pull_request) — no es duplicado:
gh run list --branch develop --limit 100 --json workflowName,event --jq '[.[] | select(.event=="pull_request") | .workflowName] | unique'
```

**Nota de alcance:** `lurkr-informational.yml` no está en la matriz (no estaba en los 15) — sigue sin fila (fuera de scope; candidato a fila futura).

**Tool quirk (2026-10-03):** `campaign_validate_scope` no expande brace-globs (`{a,b}.yml`) del blast radius declarado → reportó OUT_OF_SCOPE para los 15 workflows (check advisory, no bloqueante); los paths exactos (docs) pasaron validación. Registrado como lección en memoria.

**Backlog:** la fila `FIND-228` (Backlog.md:421) queda para el pase de progreso del orquestador — convención vigente (FIND-225/FIND-230 completados siguen en Backlog hasta el pase del campaign).
