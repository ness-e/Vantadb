---
title: "TASK FIND-230: Gate mecánico de versiones npm (anti skip-silencioso del tren)"
kind: task
description: "Gate local+CI que falla cuando vantadb-ts/package.json difiere de [workspace.package] + warning visible en el skip 'already published' del publish npm"
---

# TASK FIND-230: Gate mecánico de versiones npm (anti skip-silencioso del tren)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 3, Wave 1)
- **Fuente:** `docs/dev/Backlog.md` (FIND-230, origen release 0.8.0)
- **Esfuerzo:** 🟡 4-6h | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** CI/CD-DevOps (workflow + script Node + runbook; blast radius CI-only)
- **Turns estimados:** 15-30
- **Creado:** 2026-10-03T02:50Z | **last-synced:** 2026-10-03T04:25Z
- **Estado:** ⏳ IN PROGRESS — trabajo **completo, verificado y commiteado** (commit local, sin push); review P2-01 fresco **APPROVE** (sesión `ses_f0049cdbcffeVEwXAY6FxgJDP5`). Pendiente: bookkeeping del orquestador (recitation/plan + push).
- **Incógnitas (uphill):** 0 — diseño decidido (plan + evidencia de runs/PR); sin decisiones abiertas
- **Pendientes (downhill):** 0 steps — cierre local completo
- **Campaign ID:** post-release-0.8.0-20261002

## Evidencia del gap (verificada 2026-10-03)

El release 0.8.0 publicó wasm pero **NO** el SDK TS — en un run **verde**:

| Evidencia | Detalle |
|---|---|
| Run del tag (skip silencioso) | `37045932895` (event `push`, tag `v0.8.0`, 2026-10-02T18:13Z, **conclusion: success**): job `publish-ts` → `Version 0.7.0 already published — skipping`; wasm sí publicó (`0.8.0 not published — will publish`). `vantadb@0.8.0` no salió en ese run |
| Backfill manual | commit `e62e0f62` (`chore(release): sync vantadb-ts package version to 0.8.0`) + dispatches `37082050596`/`37082820490` (en el 2º: TS `0.8.0 not published — will publish`; wasm ya `already published — skipping`) |
| Registry | `vantadb`=0.8.0 (post-backfill), `vantadb-wasm`=0.8.0, `vantadb-node`=**404 (nunca publicado)** |
| Release PR 0.9.0 (abierto) | `gh pr view 238 --json files` → `Cargo.lock`, `Cargo.toml`, `docs/CHANGELOG.md` — **no incluye** `vantadb-ts/package.json` → sin el bump, el tag `v0.9.0` repetiría el skip |
| Causa estructural | `vantadb-wasm` hereda `version.workspace = true` (auto-sync); `vantadb-ts/package.json` NO lo gestiona release-plz → drift posible en cada release |

Mecanismo del skip: `release-npm-61.yml` pregunta a npm por la versión en `package.json`; si ya existe, marca `exists=true` y el step de publish se saltea — el run termina **success sin decir nada** (steps "Check if version already published", L168-179 wasm / L258-269 ts).

## Blast Radius

| Archivo | Cambio previsto |
|---------|----------------|
| `scripts/docs/check-npm-versions.mjs` | **NUEVO** — gate reutilizable local+CI (patrón `check-api-docs.mjs`: `--self-test`, `--json`, exits 0/1/2) |
| `.github/workflows/gate-docs.yml` | job nuevo `check-npm-versions` + paths `vantadb-ts/package.json`, `vantadb-node/package.json` |
| `.github/workflows/release-npm-61.yml` | `::warning::` en los 2 steps "Check if version already published" (wasm + ts) |
| `.github/workflows/release-npm-node.yml` | `::warning::` en su step idéntico (misma clase de skip mudo) |
| `docs/dev/workflow/PUBLISH.md` | sección "Orden del bump npm (FIND-230)" — bump npm en la MISMA rama del Release PR, antes de mergear |
| `docs/dev/workflow/gate-docs-21.md` | lista de jobs 2→5 (stale pre-existente, FIND-174b) + triggers reales |
| `docs/dev/tasks/FIND-230.md` | este archivo (nuevo) |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/*` + `docs/pipeline-state.json` (orquestador), `benchmarks/*` + `.github/workflows/perf-bench.yml` (FIND-232). No renombrar `release.yml`/`release-npm-61.yml`/`release-npm-node.yml` (RULES.md §Excepción — OIDC binding por filename).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/gate-docs.yml` (98L), `.github/workflows/release-npm-61.yml` (275L), `.github/workflows/release-npm-node.yml` (229L), `scripts/docs/check-api-docs.mjs` (1284L, patrón), `scripts/docs/lib.mjs` (exports `ROOT`/`abs` verificados L15/L62 — no se edita), `vantadb-ts/package.json` (73L), `vantadb-node/package.json` (59L), `docs/dev/workflow/PUBLISH.md` (111L), `docs/dev/workflow/RULES.md` (218L), `docs/dev/workflow/gate-docs-21.md` (40L), `.opencode/rules/release-ci.md` (42L, regla del área) + Apéndice V Clean Code.
- **Referencias hacia dentro:** `check-npm-versions.mjs` → `lib.mjs` (`abs`) + `node:fs`/`node:path` (stdlib); `gate-docs.yml` → actions pinneadas por SHA + Node del runner; npm-61/node → steps locales.
- **Referencias entrantes:** `gate-docs.yml` por `TRIGGERS.md:26,67`, `gate-docs-21.md`, `README.md` (inventario), `FAQ.md:23,40`; `release-npm-61.yml` por `PUBLISH.md:44,108`, `README.md:80`, `TRIGGERS.md:54`, `release-npm-61.md`; `PUBLISH.md` por `RULES.md` §See also + índices generados. Ningún consumidor programático.
- **Veredicto impacto:** **BAJO / CI-only.** Aditivo; sin API pública, runtime, storage ni bindings. Los únicos efectos nuevos: un check que puede poner un PR en rojo (por diseño) y `::warning::` en runs de publish.

## Contrato

"`node scripts/docs/check-npm-versions.mjs --self-test` exit 0 · con `vantadb-ts/package.json` alterado a `0.7.0` (≠ workspace 0.8.0) → **exit 1 + mensaje que nombra archivo y versión esperada**; restaurado → **exit 0**. `actionlint` sobre los 3 workflows tocados exit 0. El job `Check npm package versions` de `gate-docs.yml` corre el mismo script. El skip 'already published' de `release-npm-61.yml` (wasm+ts) y `release-npm-node.yml` emite `::warning::` visible."

## Spec (decisiones — resueltas por evidencia, plan Task 3)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde corre el gate | A `gate-docs.yml` job nuevo / B workflow nuevo (16º, ruido FIND-228) / C `gate-api-docs.yml` (semántica surface-vs-docs ajena) | ✅ **A** — release-ci.md §4 agrupa docs+packaging en una regla y el job hermano `check-api-version` ya vive ahí; el workflow ya dispara con `Cargo.toml` (evento del bump) y `scripts/**` |
| 2 | Regla por paquete | `vantadb-ts`: `== workspace` estricto / `vantadb-node`: `== workspace` **o** "never-published" documentada | ✅ ts estricto (publish en cada tag; drift = skip) · node allowlist `['0.7.0']` (registry 404 verificado; sin tren, sin riesgo de skip; al primer release debe `== workspace`) |
| 3 | Implementación | script `.mjs` reutilizable / bash inline | ✅ script (pre-mortem 3 + reuso local) |
| 4 | Skip mudo | `::warning::` en los check-* de npm-61 (y npm-node, misma clase) | ✅ |
| 5 | Orden del bump | bump npm en la MISMA rama del Release PR / bump en develop antes del Release PR | ✅ misma rama — PR #238 no incluye package.json; un bump en develop dejaría `npm ≠ workspace` y el gate lo marcaría igual (viajan juntos) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no tocar `opencode.jsonc`/plan file/pipeline-state; no renombrar workflows de publish (OIDC por filename); el script NO consulta red (determinista) y no publica nada; el gate no debe bloquear a `vantadb-node` mientras siga "never-published"; commit **LOCAL** sin push.
- **Comandos de verificación:** `node scripts/docs/check-npm-versions.mjs --self-test` · `actionlint .github/workflows/gate-docs.yml .github/workflows/release-npm-61.yml .github/workflows/release-npm-node.yml` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` · contrato alter/restore documentado en §Resultado.
- **Deuda pendiente:** ninguna (si aparece: fila `FIND-*`).

## Fases explícitas — SECURITY | PERFORMANCE

- **SECURITY** — N/A justificado: cambio de CI/script offline (`contents: read`); sin input de usuario, sin auth, sin secretos, sin dependencias nuevas. El script lee 3 archivos del repo y compara strings.
- **PERFORMANCE** — N/A justificado: sin hot paths; el job agrega ~15s al Fast Gate (medido al ejecutar).

## Definition of Done

| Nivel | Gate | Estado |
|-------|------|--------|
| Task | Contrato fail/pass documentado + job en CI + warning visible | ⬜ |
| Commit | `ci:` conventional + verify (self-test, actionlint, checks docs) + solo archivos de la tarea | ⬜ |
| Release | `PUBLISH.md` actualizado en el mismo commit (DoD del plan) · nivel release completo (verify.ps1) N/A: sin código Rust/shippable | ⬜ |

## Steps

### Step 1: Crear `scripts/docs/check-npm-versions.mjs`

- **Archivos:** `scripts/docs/check-npm-versions.mjs` (nuevo)
- **Acción:** gate con 2 targets (`vantadb-ts` estricto, `vantadb-node` never-published), `--self-test`, `--json`, annotations `::error file=` bajo `GITHUB_ACTIONS`, exits 0/1/2
- **Verify:** `node scripts/docs/check-npm-versions.mjs --self-test` exit 0 en árbol actual (ts=0.8.0, node=0.7.0)
- **Estado:** ✅ COMPLETED

### Step 2: Job en `gate-docs.yml` + paths

- **Archivos:** `.github/workflows/gate-docs.yml`
- **Acción:** job `check-npm-versions` (checkout+node pinneados, timeout 5, self-test + gate + report on failure) + paths `vantadb-ts/package.json`/`vantadb-node/package.json` en push+PR
- **Verify:** `actionlint .github/workflows/gate-docs.yml` exit 0 + `node scripts/docs/check-npm-versions.mjs` exit 0
- **Estado:** ✅ COMPLETED

### Step 3: `::warning::` en los skips de publish

- **Archivos:** `.github/workflows/release-npm-61.yml` (2 steps), `.github/workflows/release-npm-node.yml` (1 step)
- **Acción:** reemplazar `echo "Version … already published — skipping"` por `::warning::` accionable (referencia a PUBLISH.md § orden del bump)
- **Verify:** `actionlint` ambos exit 0 + grep de las 3 líneas nuevas
- **Estado:** ✅ COMPLETED

### Step 4: Docs (PUBLISH.md + gate-docs-21.md)

- **Archivos:** `docs/dev/workflow/PUBLISH.md`, `docs/dev/workflow/gate-docs-21.md`
- **Acción:** sección "Orden del bump npm (FIND-230)" + job list actualizada
- **Verify:** `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs` + `node scripts/docs/gen-index.mjs --check` exit 0
- **Estado:** ✅ COMPLETED

### Step 5: Verify contrato (alter→FAIL / restore→PASS) + OCR + commit

- **Archivos:** (verificación)
- **Acción:** mutar `vantadb-ts/package.json` a 0.7.0 → capturar output FAIL → restaurar → PASS; OCR review; commit local
- **Verify:** outputs documentados en §Resultado + `git diff` limpio (solo archivos de la tarea); commit local al cierre (hash en la recitation RESULTADO)
- **Estado:** ✅ COMPLETED

## Dependencias

- Wave 1 — sin dependencias bloqueantes. Próxima tarea: FIND-226 (W1 paralelo).

## Resultado (evidencia del contrato — outputs reales)

**FAIL (ts alterado a 0.7.0 → exit 1):**

```text
workspace version: 0.8.0 (Cargo.toml)
FAIL vantadb-ts/package.json: 0.7.0  (expected 0.8.0)
ok   vantadb-node/package.json: 0.7.0
GATE FAILED: vantadb-ts/package.json: version 0.7.0 != workspace version 0.8.0 (expected 0.8.0) — bump it in the same PR that bumps the workspace version (the release-plz Release PR), or the npm publish step will skip silently (see docs/dev/workflow/PUBLISH.md)
```

En CI (`GITHUB_ACTIONS=true`) emite además `::error file=vantadb-ts/package.json::…` (+ `::notice::` para la nota de node). `--json` emite el reporte estructurado (`pass:false`, `failures[0].path`).

**PASS (restaurado con `git checkout --` → exit 0):**

```text
ok   vantadb-ts/package.json: 0.8.0
ok   vantadb-node/package.json: 0.7.0
note: vantadb-node/package.json: 0.7.0 is a documented never-published version — allowed until its first release…
OK: 2 npm package(s) in sync with workspace 0.8.0.
```

`git diff --exit-code -- vantadb-ts/package.json` = 0 (byte-idéntico al original).

**Verificaciones:** `--self-test` 10/10 · `actionlint` (3 workflows) = 0 · `check-links`/`check-docs` = 0 · `gen-index --check` = 0 sobre el árbol de este changeset (1481 docs; el working tree compartido da 1 por `FIND-226.md` de un worker paralelo — no viaja en este commit) · markdownlint (3 MD) = 0 · OCR 0 Critical/High.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (leaf, contexto fresco — sesión `ses_f0049cdbcffeVEwXAY6FxgJDP5`, distinta al implementador). OCR delegation ejecutado antes del veredicto: 4 archivos vs sus Rule Groups, 0 Critical/High.
- **Enfoque:** approach vs alternativas (workflow nuevo / bash inline / derivar versión al publicar), regla node never-published, evidencia del contrato (re-ejecutó self-test, alter/restore, actionlint, docs gates), idempotencia de los `::warning::`, coherencia docs↔comportamiento. Round 1: `changes-required` (1 Required doc-only: PUBLISH.md no cubría que `release-plz-pr` cierra/reabre su PR sin commits humanos). Fix aplicado (paso 2 + cautela + nit) → Round 2: **`VERDICT: approve`**.
- **Cómo se probó:** contrato reproducido por el revisor con sus propios comandos (no auto-reporte): fail/pass del gate, annotations CI, exit 2 para mal uso, actionlint/docs gates en 0.
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (cada cifra con comando) · sin done sin verificar (contrato + gates re-corridos) · fallos parciales reportados (index race documentado; optional no absorbidos) · sin scope creep (optional quedaron como notas) · cobertura SDP declarada (abajo).
- **Veredicto:** ✅ **approve** (salvedad del revisor: integrar los optional — ruleset, index — fuera de este changeset).

## Herramientas necesarias

`campaign_verify_cmd` (contrato/selftest/actionlint/docs) · `campaign_validate_scope`/`validate_command` · node + `scripts/docs/*.mjs` · `actionlint` · `gh` (evidencia de runs).

**Skills cargadas (SDP v3):** base `campaign-executor`/`progreso` (auto) + pins `ci-cd-and-automation` y `git-workflow-and-versioning` + `documentation-skill` (MUST: edita `docs/`) + `test-driven-development` (self-test del script). Descartadas con justificación: `source-driven-development` (sin APIs externas nuevas; patrón in-repo), `incremental-implementation`/`context-engineering` (flujo estándar).

## Notas

- Run del skip: `37045932895`; backfill: `e62e0f62` + dispatches `37082050596`/`37082820490`.
- `release-ci.md` §4 quedó stale ("hoy 0.7.0"; menciona `web/package.json` que ya no existe) — NO se toca (`.opencode/` es repo separado + Gate H); anotado para el lead/harness.
- Decisión de no incluir `web/package.json` en el gate: no publica a npm (sin riesgo de skip) y el path no existe hoy.
- El gate sigue el patrón de `check-api-docs.mjs`: self-test puro (fixtures sintéticos), exit codes explícitos y nunca "pasa" si no puede evaluar.
- Stop condition del plan (>2 falsos positivos → warning-only) no disparada: el diseño elegido no bloquea el flujo legítimo (bump viaja en el Release PR).
- **Review round 1 → fix (doc-only):** PUBLISH.md no cubría que `release-plz-pr` (`release.yml` L44-71, corre en cada push a main) cierra/reabre su PR sin commits humanos → cautela agregada (mergear sin demora, verificar el diff justo antes de mergear, re-aplicar bump si fue cerrado/reabierto). Round 2: approve.
- **Riesgo de índice (working tree compartido):** `gen-index --check` reporta 1 en el árbol vivo porque `docs/dev/tasks/FIND-226.md` (worker W1 paralelo, untracked) apareció después del regen; este changeset queda en 1481 (consistente sin FIND-226) — quien commitee FIND-226 debe regenerar. No se absorbe acá.
- **Follow-ups del reviewer (fuera de scope, anotados para el orquestador):** (a) el ruleset de `main` no requiere `Check npm package versions` — acción de owner (rel. FIND-174a); (b) defense-in-depth evaluado y no exigido: `npm pkg set version=$VER` antes del check en `release-npm-61.yml`.
