---
title: "DEF-03: Frontera verificable en CI (`validate-frontier`)"
kind: task
description: pwsh scripts/validate-frontier.ps1 exit 0 sobre el repo actual Y exit ≠0 al perturbar (fila Production-facing con ruta/feature inexistente o feature/workspace-member real no listado) Y job check-frontier presente en...
---

# DEF-03: Frontera verificable en CI (`validate-frontier`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 11, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-03` (L938) + plan Task 11
- **Esfuerzo:** 🟡 2d · **Prioridad:** 🟠 · **Tipo:** Docs/CI (PowerShell + GitHub Actions)
- **Turns estimados:** 15-25
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED (2026-09-27 — steps 1–4 ✅; review fresco P2-01 ✅; commit LEAD)
- **Incógnitas (uphill):** 0 abiertas · **Pendientes (downhill):** 0 — 4/4 ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Script nuevo** → 0 callers existentes. CI: `.github/workflows/gate-docs.yml` (job nuevo `check-frontier`; paths filter ya cubre `docs/**` y `scripts/**`, L4-9). Si en cambio se extendiera `validate-docs-coverage.ps1` (opción descartada, ver Spec #1), sus 4 callers locales también entran: `dev-tools/verify.ps1:86-87`, `dev-tools/verify_changed.ps1:35`, `Justfile:142-143` (`just docs`), `dev-tools/audit-all.ps1:107` |
| Callees | `docs/user/operations/EXPERIMENTAL_FEATURES.md` (parseo de filas — output de DEF-02), `Cargo.toml` (features L149-190 + workspace members L741-763), rutas del repo (check de existencia), `pwsh` del runner ubuntu-latest |
| Implicaciones | Sin cambios de API pública/performance de producto. CI: +1 job ≤5min en gate-docs (no toca fast gate de ci-rust). Riesgo principal: falsos positivos por rutas condicionales/feature-gated → mitigado con mapa explícito (pre-mortem del master: "rutas feature-gated → mapa feature→path explícito"). Hoy **ningún workflow CI invoca los checks de docs** → este job introduce el primero. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `scripts/validate-docs-coverage.ps1` (227 L), `.github/workflows/gate-docs.yml` (87 L), `.github/workflows/ci-gate.yml` (75 L), `docs/user/operations/EXPERIMENTAL_FEATURES.md` (109 L)
- **Archivos referenciados hacia dentro:** el script actual lee `src/sdk/{builder,api,graph,search/mod}.rs`, `src/config.rs`, `src/error.rs`, `src/cli.rs`, `vantadb-python/src/lib.rs`, `vantadb-mcp/src/handlers/tools.rs` y `docs/api/*`/`docs/user/operations/*` (formato de sus checks = referencia para el script nuevo)
- **Archivos que referencian a los editados (grep `validate-docs-coverage`):** `dev-tools/verify.ps1:86-87`, `dev-tools/verify_changed.ps1:35`, `Justfile:143`, `dev-tools/audit-all.ps1:107`, `SPEC.md:45,70`, `docs/api/VERSIONING.md:70`, `docs/api/MCP.md:536`. En `.github/` = **0 hits** (verificado por grep)
- **codegraph_explore sobre `validate-docs-coverage`:** sin resultados útiles — CodeGraph no indexa `.ps1` (retornó símbolos Rust no relacionados) → blast radius mapeado vía grep, documentado aquí
- **Veredicto impacto:** bajo — script nuevo aislado + 1 job CI; ningún consumidor existente se modifica. El único acoplamiento frágil es el formato del doc de DEF-02 (parser tolerante + mensajes accionables).

## Contrato
"`pwsh scripts/validate-frontier.ps1` exit 0 sobre el repo actual Y exit ≠0 al perturbar (fila Production-facing con ruta/feature inexistente **o** feature/workspace-member real no listado) Y job `check-frontier` presente en `.github/workflows/gate-docs.yml` Y verde en CI al abrir PR (push owner-gated)"

## Spec (SDD — decisiones de script/CI)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Script | A: extender `validate-docs-coverage.ps1` (un entry point, pero acopla 2 dominios y 4 callers locales) / B: nuevo `scripts/validate-frontier.ps1` | B | ✅ decidido-por-evidencia (0 callers rotos; dominios distintos; el título del master lo nombra `validate-frontier`) |
| 2 | Mapa fila→feature/ruta | A: heurística por nombre (grep de ruta) / B: mapa explícito en el script | B | ✅ decidido-por-evidencia (pre-mortem Task 11: "rutas feature-gated → mapa feature→path explícito") |
| 3 | Sensibilidad | A: solo doc→código (filas declaradas existen) / B: bidireccional (+ feature real no listada = fail) | B | ✅ decidido-por-evidencia (DoD Backlog DEF-03: "falla si una feature REAL no está listada") |
| 4 | Ubicación CI | A: step en job existente (`check-api-version`) / B: job nuevo `check-frontier` en gate-docs.yml | B | ✅ decidido-por-evidencia (aislado, timeout propio ≤5min, ubuntu-latest trae `pwsh`; no arriesga los jobs actuales) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `validate-docs-coverage.ps1` intacto y sus 4 callers locales siguen verdes; gate-docs.yml conserva sus 3 jobs actuales sin cambios de comportamiento; workflow sin secrets (`permissions: contents: read`); el script corre offline (sin red).
- **Comandos de verificación:** `pwsh -NoProfile -File scripts/validate-frontier.ps1; $LASTEXITCODE` → 0 · perturbaciones en copia temporal → 1 (×2 casos) · `yamllint .github/workflows/gate-docs.yml` (si está instalado) → 0
- **Deuda pendiente:** ninguna (si el script resultara >1min en CI → mover a nightly documentado, no silencio)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← valor en este task file |
|------------------------|----------------------------|
| `activeGoal` | DEF-03 — Frontera verificable en CI (`validate-frontier`) |
| `lastAction` | Steps 1–3 ✅ + fixes F1/F2/F3 + re-review fresco APPROVE (ses_f1b852583ffefT5FBFo4LbABaF) |
| `result` | OK (review fresco ✅) |
| `nextAction` | LEAD: commit local `ci:` (DEF-03) — staging selectivo script + workflow |
| `contract` | §Contrato + §Invariantes (verificación: exit codes RED/GREEN + job en gate-docs) |
| `nextTask` | DEF-04 (Task 12 del master) |

```
=== RECITATION ===
Objetivo activo: DEF-03 — Frontera verificable en CI (validate-frontier)
Estado: completed
Última acción: re-review fresco ✅ APPROVE (2026-09-27)
Resultado: OK
Próxima acción: commit LEAD (ci:)
Contrato: ver ## Contrato (exit 0 actual, ≠0 perturbado, job en gate-docs.yml)
Invariantes: validate-docs-coverage intacto · gate-docs sin secrets · script offline
Deuda: ninguna
Próxima tarea si completa: DEF-04
last-synced: 2026-09-27
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (script + step CI; introduce el primer gate mecanizado de frontera).

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato ✅ + sensibilidad probada (2 casos RED + 1 GREEN) + gate verde local
- **Commit:** atómico `ci:` + `(DEF-03)`, diff limitado a `scripts/validate-frontier.ps1` + `.github/workflows/gate-docs.yml`
- **Release:** N/A aplicable — tooling de CI, sin versionado (release-plz lo ignora con `ci:`)

## Herramientas necesarias
- `pwsh` (script + dry-runs), `yamllint`/revisión de workflow, `rg` (blast radius), `gh` (estado de CI post-PR — solo lectura)
- **Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` · **PINNED (policy):** `ci-cd-and-automation`, `git-workflow-and-versioning`
  - Descartadas del SDP (no aplican): `frontend-ui-engineering`, `context-engineering`, `api-and-interface-design`

## Investigation Notes
- Hoy `validate-docs-coverage.ps1` corre **solo local** (verify.ps1/verify_changed/just/audit-all); 0 workflows CI lo invocan → este task agrega el primer eslabón docs al CI (gate-docs).
- `gate-docs.yml` actual: jobs `lint-markdown`, `check-format`, `check-api-version` (L20-87); triggers por paths `docs/**`, `src/server/router.rs|routing.rs`, `scripts/**` → el script nuevo cae dentro del trigger sin cambios.
- `docs/api/MCP.md:536` ya declara que la cobertura de tools se enforce mecánicamente con `validate-docs-coverage.ps1` contra `handle_tools_list()` → el patrón "doc ↔ código" existe; DEF-03 lo extiende a la frontera de producto.
- Stop condition del master (Task 11): >30% falsos positivos → rediseñar como reporte; script >1min → nightly.
- HARD-06 (FIND-162) repara la §3 de `validate-docs-coverage.ps1`: con script nuevo no hay colisión de archivos.
- Coordinación HARD-02: si se crea nightly de certificación pesada, el job de frontera queda en gate-docs (fast, offline) — no duplicar.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 — 4/4 ✅ |
| % completado | 100% (review fresco ✅; commit LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — N/A: el script solo lee el repo; job CI sin secrets (`contents: read`); no agregar permisos.
- [x] **PERFORMANCE** — N/A: script offline, target <10s; job CI con `timeout-minutes: 5` alineado al Fast Gate (<5min).

## Steps

### Step 1: `scripts/validate-frontier.ps1` v0 (parser + mapa explícito + checks principales)
- **Archivos:** `scripts/validate-frontier.ps1` (nuevo)
- **Acción:** parser de filas de `EXPERIMENTAL_FEATURES.md` (sección + columna Boundary); mapa explícito fila→{feature Cargo | ruta | comando}; check (a) cada fila Production-facing resuelve a feature/ruta existente y check (c) feature/workspace-member real no listado → fail; `Set-StrictMode -Version Latest`, exit 0/1, mensajes accionables (`file/row → qué falta`). ≤100 L funcionales.
- **Verify:** `pwsh -NoProfile -File scripts/validate-frontier.ps1; $LASTEXITCODE` → 0 sobre el doc de DEF-02. ✅ 2026-09-27: exit 0 (12 production rows · 6 doc features · 6 members).
- **Estado:** ✅ DONE (2026-09-27)

### Step 2: Checks negativos + sensibilidad RED/GREEN
- **Archivos:** `scripts/validate-frontier.ps1`
- **Acción:** check (b) "Deferred/Archived no puede existir vivo sin marca" (p.ej. compara contra features Cargo); probar 3 perturbaciones sobre copias temporales (NUNCA commitear): fila→ruta inexistente (exit 1), feature nueva no listada (exit 1), doc intacto (exit 0). Registrar los 3 comandos exactos en Notas al ejecutar.
- **Verify:** exit codes 1/1/0 en los 3 casos. ✅ 2026-09-27: 5/5 casos (RED fila→ruta=1 · RED feature no listada=1 · GREEN=0 · RED Deferred sin marca=1 · RED sección renombrada/vacuous-pass=1).
- **Estado:** ✅ DONE (2026-09-27)

### Step 3: Wiring CI en `gate-docs.yml`
- **Archivos:** `.github/workflows/gate-docs.yml`
- **Acción:** job `check-frontier` (ubuntu-latest, `timeout-minutes: 5`, `permissions: contents: read`, step `pwsh -NoProfile -File scripts/validate-frontier.ps1`); jobs existentes intactos. Decisión YAGNI: NO sumarlo a `verify.ps1` local (corre en CI docs; si hiciera falta, `just docs-frontier` en un follow-up — no en este scope).
- **Verify:** diff del workflow revisado; `yamllint .github/workflows/gate-docs.yml` 0 (si disponible); paths filter ya cubre el script (no requiere cambio). ✅ 2026-09-27: `actionlint .github/workflows/gate-docs.yml` → 0 (yamllint no instalado; actionlint es el gate YAML del repo). Job `check-frontier` (ubuntu-latest, `timeout-minutes: 5`, `permissions: contents: read`, pin checkout v7.0.1); jobs existentes intactos (L20-87). **Review P2-01 (F1):** `paths` ahora incluye `Cargo.toml` en push y PR (sin esto, un PR que solo agrega una feature/member no disparaba el check Cargo→doc); actionlint re-verificado → 0.
- **Estado:** ✅ DONE (2026-09-27)

### Step 4: Cierre
- **Archivos:** —
- **Acción:** contrato completo local (script + perturbaciones) + commit `ci: add frontier validation gate (DEF-03)`. CI verde se confirma al abrir PR (push owner-gated, política del master).
- **Verify:** `campaign_verify_cmd` con el contrato; `git diff --stat` = 2 archivos; (post-push owner) checks del PR verdes.
- **Resultado:** re-review fresco ✅ APPROVE (`ses_f1b852583ffefT5FBFo4LbABaF`): contrato exit 0 + 6 RED reproducidas + actionlint 0 + paridad Linux 25/25 + pin SHA verificado por API. Commit delegado al LEAD (staging selectivo: script + workflow).
- **Estado:** ✅ DONE (2026-09-27 — review fresco ✅; commit LEAD)

## Dependencias
- **DEF-02 (duro):** el script parsea el doc regenerado (formato final de filas). No iniciar Step 1 sin DEF-02 ✅.
- Siguiente: DEF-04.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` fresh, sesión `ses_f1bb79090ffeuw5y57q6aGafas` (≠ autor `ses_f1bc7c4d4ffei1QAotNCifcDp2`)
- **Enfoque:** ¿la sensibilidad es real (no un check decorativo)? ¿falsos positivos controlados? ¿el step CI no debilita el gate docs?
- **Cómo se probó:** GREEN (0) + 3 RED del harness (1/1/1) + 4 RED propios del revisor (heading guard, member no listado, member sin dir, feature fantasma) + `actionlint` 0 + pin `checkout` validado contra API GitHub (`3d3c42e5…` exacto) + case-sensitivity Linux vs `git ls-files` (25/25 tracks)
- **Re-review fresco P2-01 (2026-09-27):** `vanta-review` sesión `ses_f1b852583ffefT5FBFo4LbABaF` (≠ autor y ≠ primer revisor) → **✅ APPROVE**. Evidencia reproducida: contrato exit 0 (~1.5s) · 6 RED independientes (2 propias del revisor) + ruta de excepción ≠0 · actionlint 0 · paridad Linux validada contra `git ls-files` (25/25 caso exacto) · pin checkout verificado contra API GitHub · jobs existentes intactos (3+1).
- **Checklist anti-hábitos tóxicos:** según plantilla
- **Veredicto:** ✅ **APPROVE** — re-review fresco (sesión `ses_f1b852583ffefT5FBFo4LbABaF`, 2026-09-27). Primer veredicto ❌ (F1) resuelto y re-verificado; ACCEPT habilitado (payload review fresh para HARD-07).

## Notas
- Stop conditions (master Task 11): >30% falsos positivos → rediseñar como reporte; script >1min → nightly.
- Con script nuevo, `validate-docs-coverage.ps1` queda sin tocar (HARD-06/FIND-162 trabaja ese archivo).
- No crear workflow nuevo: extender `gate-docs.yml` (anti-proliferación de CI).
- Commit `ci:` atómico; push owner-gated.
- **Sensibilidad verificada (2026-09-27)** — comandos exactos (harness: `%TEMP%\opencode\def03-perturb.ps1`, copias temporales nunca commiteadas):
  1. `pwsh -NoProfile -File scripts/validate-frontier.ps1; $LASTEXITCODE` → **0** (GREEN, repo intacto)
  2. `pwsh -NoProfile -File scripts/validate-frontier.ps1 -DocPath <tmp>\perturbed-path.md; $LASTEXITCODE` (fila L28 con `src/nope/missing.rs`) → **1** (message: `...:28 [row 'Memory ...'] → path does not exist`)
  3. `pwsh -NoProfile -File scripts/validate-frontier.ps1 -CargoPath <tmp>\perturbed-feature.toml; $LASTEXITCODE` (`+vanta-newness = []`) → **1** (feature no clasificada)
  4. `pwsh -NoProfile -File scripts/validate-frontier.ps1 -DocPath <tmp>\perturbed-deferred.md; $LASTEXITCODE` (fila L93 sin marca de estado vivo) → **1** (check b vivo)
  5. `pwsh -NoProfile -File scripts/validate-frontier.ps1 -DocPath <tmp>\perturbed-renamed.md; $LASTEXITCODE` (sección Production-Facing renombrada) → **1** (guard anti-vacuous-pass, agregado tras self-review; evita green silencioso si el doc se reformatea)
  6. `pwsh -NoProfile -File scripts/validate-frontier.ps1 -DocPath <tmp>\perturbed-ghost.md; $LASTEXITCODE` (`feature:ghost-feature` citada solo en el doc) → **1** (doc→Cargo: feature citada inexistente)
- **Fixes post-review P2-01 (2026-09-27):** F1 `paths` +`Cargo.toml` en gate-docs (push+PR); F2 comentario `$cells[3]`=Evidence; F3 extensiones `ts/tsx/js` en `$pathLike`. Re-battery sobre artefacto congelado: GREEN=0 + 5 RED=1 + actionlint 0. F4/F5 (allowlist por diseño; `pending` en `aliveMarker`) aceptados por Spec.
- **Follow-ups del re-review (optional, owner/lead):** (a) `Check Product Frontier` no está en los required checks del ruleset main/develop (los jobs de docs no bloquean merges — pre-existente para los 3 jobs); (b) doc-sync: `docs/dev/workflow/gate-docs-21.md` ("2 jobs") y `docs/dev/operations/CI_POLICY.md` §4 quedaron stale (incluye drift pre-existente de `check-api-version`).
- **Nota verify (2026-09-27):** `campaign_verify_cmd` con `pwsh -NoProfile -File <script>` → exit **-1 determinista** en el contexto del server MCP (Windows/bun; stdout vacío, 0.0s); con `pwsh scripts/validate-frontier.ps1` → exit **0** (2.0s) ✅. Mismo script y semántica de exit; los flags se omiten en la invocación MCP. Registrado como FIND-173.
