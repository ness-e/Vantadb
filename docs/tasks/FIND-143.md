# FIND-143: docs/workflow/ — inventario, triggers, publish, runbook, FAQ

## Metadata

- **Plan file:** docs/plans/2026-09-21-workflows-repair.md (Wave 4)
- **Fuente:** auditoria 28/28 + FIND-128 matriz + FIND-137/139/140/141/146 (estado post-waves)
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢 Baja (docs, tras codigo estable)
- **Tipo:** Documentation (docs-only, sin YAML, sin Rust)
- **Turns estimados:** 6
- **Creado:** 2026-09-22
- **last-synced:** 2026-09-22
- **Estado:** ⬜ PENDING → IN PROGRESS (esta iteracion)
- **In cognitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 (S1 task file, S2-S4 docs, S5 verify, S6 commit)

## Blast Radius

| Direccion | Modulos |
|-----------|---------|
| Callers | Humano/CI que lea `docs/workflow/`; FIND-144 (`RULES.md`) consume este inventario |
| Callees | `.github/workflows/*.yml` solo LECTURA (`on:`, concurrency, environments); `docs/tasks/FIND-128.md` matriz; `docs/workflow/*.md` vecinos (formato frontmatter) |
| Implicaciones | Cero cambio comportamiento CI; solo docs nuevas; links relativos internos |
| Riesgo | minimo (docs-only; prohibido YAML/src/web/desktop/locks/plans/Backlog) |

## Impacto mapeado (Regla 0)

- **Archivos leidos (completos):** `docs/plans/2026-09-21-workflows-repair.md` (plan Wave 4), `docs/tasks/FIND-128.md` (matriz 28 + propuesta dedup), `.opencode/rules/README.md` (formato reglas), `.opencode/rules/release-ci.md` (completo), `docs/workflow/ci-gate.md` + `release-wheels-60.md` + `release-npm-61.md` (formato vecino/frontmatter), `.markdownlint-cli2.yaml` (reglas), `scripts/validate-docs-coverage.ps1` (alcance)
- **Triggers `on:` de 27 workflows:** extraidos via script python (read-only, sin edits YAML)
- **Archivos que referencian a los creados:** ninguno aun (nuevos); FIND-144 referenciara despues
- **Veredicto impacto:** nulo en runtime — 5 markdown nuevos + este task file; `docs/workflow/*.md` vecinos intactos

## Contrato

"Crear `docs/workflow/README.md` (inventario 27 + nota 28→27), `TRIGGERS.md` (matriz push/PR/schedule/dispatch/tags), `PUBLISH.md` (flujo por registro + namespaces tags), `RUNBOOK.md` (re-run, approve environments, [no-adr]), `FAQ.md` (duplicados, cancel-in-progress, skipped-vs-required); `npx markdownlint-cli2 docs/workflow/**/*.md` 0 issues + 0 links rotos + commit `docs: FIND-143 — ...` selectivo sin push"

## Spec

N/A — docs-only, Phase 1b negativa: no agrega `pub fn`, tools, endpoints ni bindings. Sin Gate P/D.

## Invariantes de dominio (handoff — MUST)

- **Invariantes:** (1) cero edits `.github/workflows/`; (2) no tocar `src/`, `web/src/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, `docs/Backlog.md`, plan file (solo recitation via MCP), versionados (`docs/CHANGELOG.md`, `docs/api/openapi.yaml`, `docs/api/MCP.md`); (3) ingles tecnico en docs, espanol solo aqui; (4) citas GitHub con URL o `[NO VERIFICADA]`; (5) NO PUSH
- **Verificacion:** `npx markdownlint-cli2 "docs/workflow/**/*.md"` exit 0 + grep links rotos 0 + `git status --short` selectivo
- **Deuda:** `validate-docs-coverage.ps1` N/A (no toca producto); `actionlint` N/A (solo docs)

## Deuda tecnica (Regla 6 — MUST)

Sin deuda nueva (docs-only, elimina deuda doc Wave 4). No aplica moneda P2.

## Definition of Done

- **Task:** contrato arriba (5 docs + lint 0 + links 0)
- **Commit:** atomico, `docs:` conventional + ID, solo 6 archivos (5 docs + task file), verify mecanico previo
- **Release:** N/A (docs sin versionado)

## Herramientas necesarias

- `read`/`glob` (lectura triggers y vecinos)
- `npx markdownlint-cli2` (lint contrato)
- `git` (diff selectivo, commit sin push)
- `campaign_verify_cmd` (contrato; BUG exit -1 → bash directa + mencion)

**Skills cargadas (SDP):** documentation-and-adrs (estructura docs + decisiones escritas) · writing-guidelines (voz/tono) · base campaign-executor/progreso/ponytail. SDP v2 sugirio lifecycle genericas (incremental/test-driven/context/source-driven/doubt/api-design/frontend-ui) — descartadas: docs markdown sin logica nueva ni UI ni API (ponytail: no cargar por cargar).

## Steps

### Step 1: Discovery + task file

- **Archivos:** plan Wave 4, FIND-128, triggers `on:` 27 workflows, vecinos frontmatter
- **Accion:** extraer triggers, confirmar 27 (28 menos rustdoc-70 eliminado FIND-137), crear este task file
- **Verify:** existe + Regla 0/contrato poblados
- **Estado:** ✅ COMPLETED

### Step 2: README + TRIGGERS

- **Archivos:** `docs/workflow/README.md`, `docs/workflow/TRIGGERS.md`
- **Accion:** inventario 27 (nombre, proposito 1 linea, triggers corto) + matriz completa push/PR/schedule/dispatch/tags/release/call/comment
- **Verify:** existen + frontmatter espejo vecino
- **Estado:** ⬜ PENDING

### Step 3: PUBLISH + RUNBOOK

- **Archivos:** `docs/workflow/PUBLISH.md`, `docs/workflow/RUNBOOK.md`
- **Accion:** flujo por registro (crates.io release-plz, wheels PyPI, npm wasm+ts, node, adapters, binaries, sbom) + namespaces tags + runbook (gh rerun, approve environments API, [no-adr])
- **Verify:** comandos `gh` con sintaxis exacta de workflows leidos
- **Estado:** ⬜ PENDING

### Step 4: FAQ

- **Archivos:** `docs/workflow/FAQ.md`
- **Accion:** duplicados push+PR, cancel-in-progress, skipped vs required (fail-closed FIND-139, informational continue-on-error)
- **Verify:** cada respuesta cita workflow/fuente concreta
- **Estado:** ⬜ PENDING

### Step 5: Verify contrato

- **Archivos:** `docs/workflow/*.md` (5 nuevos)
- **Accion:** `npx markdownlint-cli2 "docs/workflow/**/*.md"` + grep links `](docs/workflow` 0 + grep links relativos rotos
- **Verify:** exit 0 + 0 hits
- **Estado:** ⬜ PENDING

### Step 6: Commit selectivo + RESULTADO

- **Archivos:** 5 docs + este task file (6 total)
- **Accion:** `git add` SOLO esos 6 + `git commit -m "docs: FIND-143 — ..."` (SIN push); bloque RESULTADO §7 + Gates
- **Verify:** `git status --short` sin WIP ajeno; `git log --oneline -1`
- **Estado:** ⬜ PENDING

## Dependencias

- Requiere Waves 0-2 verdes (triggers post-FIND-134/139/140/141/146 — cumplido, verificado por lectura directa `on:`).
- FIND-142 (renombres) pendiente: si renombra workflows, este inventario queda con nombres viejos → deuda anotada en README ("pre-rename").
- NextTask: FIND-145 (orquestador).

## Review

- **Revisor:** pendiente (self-gate mecanico: lint + links + diff selectivo; P2-01 batch en cierre plan)
- **Enfoque:** ¿triggers copiados fieles del YAML? ¿publish namespaces exactos? ¿cero edits prohibidos?
- **Veredicto:** pendiente
