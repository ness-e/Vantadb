---
title: "FIND-264: Recitation blocks break MD007 — fix the format at the source and drop the gate exclusion"
kind: task
description: "Fence the machine-appended recitation block in updateRecitation so plans stop emitting MD007, and remove the master-plan carve-out from docs-lint-regression"
---

# FIND-264: Recitation blocks break MD007 — fix the format at the source and drop the gate exclusion

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§5, Fase 1, wave 1.1)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-264` (Origen: DOCS-F1 master-plan 0.9.0, review R-2)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Docs / harness (`.opencode/` es repo separado, no trackeado por VantaDB)
- **Turns estimados:** 6
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `campaign-server.mjs:983` (`updateTaskStateCore`) → `updateRecitation` en cada cierre de tarea de **todo** plan |
| Callees | `node:crypto` (sin cambios); consumidor del plan: `parseRecitation` (`recitationBlocks`, regex marker-based, agnóstica a fences) |
| Implicaciones | El contrato de parseo (`=== RECITATION [ID] ===` … `=== END RECITATION ===`) no cambia. El render del plan pasa a un fence `~~~text`, que markdownlint no evalúa como lista (MD007) y que `check-links.mjs` salta (escanea sólo `proseOf()`). No toca `src/**`, ni CI de Rust. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `.opencode/task-system/prompts/recitation-template.md` (42 L)
  - `.opencode/task-system/mcp/parsers.mjs` (178 L)
  - `.opencode/task-system/mcp/parsers.test.mjs:175-191`, `.opencode/task-system/mcp/hardening.test.mjs:1-110`
  - `.github/workflows/docs-gates.yml:285-330`
  - `.github/markdownlint-cli2.yaml`, `scripts/docs/check-links.mjs` (doc de cabecera: `proseOf()` strippea fences y code spans)
  - `.opencode/task-system/prompts/pipeline-full.md` (§3, §7)
- **Referencias hacia dentro (quién importa lo editado):** `campaign-server.mjs:12` importa `updateRecitation`; tests `parsers.test.mjs` y `hardening.test.mjs`; `recitation-template.md` citado por `pipeline-full.md` §3/§7, `campaign-executor/SKILL.md` y `AGENTS.md` (path resolution).
- **Referencias entrantes al gate:** `.githooks/pre-push:126`, `.githooks/pre-commit-config.yaml:69`, `dev-tools/verify_staged.ps1:53` corren el mismo config; ninguno filtra el master plan.
- **Veredicto impacto:** **bajo-medio**. El único riesgo real es romper el parseo de las recitations existentes; mitigado porque `recitationBlocks()` ancla en los marcadores y no en el fence, y porque los tests del parser se corren antes y después.

## Contrato

"`npx --no-install markdownlint-cli2 --config .github/markdownlint-cli2.yaml` sobre el plan maestro reporta 0 errores MD007; `rg 'master-plan-0.9.0' .github/workflows/docs-gates.yml .github/markdownlint-cli2.yaml` devuelve 0 líneas; `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` en verde; `updateRecitation` emite el bloque en fence `~~~text` y los 34 tests de parser siguen pasando."

## Spec

No aplica feature-add: no hay símbolos públicos nuevos (el emisor es interno del harness en un repo separado) ni contrato de API de VantaDB. Decisiones tomadas por evidencia:

| # | Decisión | Opciones | Resuelto |
|---|----------|----------|----------|
| 1 | Fence vs dedent en el emisor | dedent (el caller sigue libre de indentar) / fence `~~~text` | ✅ fence: dedent no impide que el próximo `contract` traiga listas anidadas; el fence hace la clase de bug inexpresable. `~~~` y no ```` ``` ```` porque el payload suele traer backticks. |
| 2 | Tocar el corpus (master plan) | dedent de bloques existentes / nada | ✅ nada: `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` fue **borrado** en `4e831d3e1` (archivado); `rg '^ - `claim' docs/dev/plans/` = 0. El corpus ya está limpio (`a5da35529`, MD007 50→0). |
| 3 | Alcance del retiro de la exclusión | borrar sólo el `grep -v` / limpiar también el comentario justificativo | ✅ ambas: dejar el comentario que justifica una excepción inexistente es cómo la excepción reaparece. |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el contrato de marcadores de la recitation (`=== RECITATION [<ID>] ===` / `=== END RECITATION ===`) y el reemplazo per-tarea (waves paralelas no se pisan). El modelo `taskId …` por tarea en `recitationBlocks()` no cambia.
- **Comandos de verificación:** `cd .opencode && node --test task-system/mcp/parsers.test.mjs task-system/mcp/hardening.test.mjs` (34/34) · `npx --no-install markdownlint-cli2 --config .github/markdownlint-cli2.yaml "docs/**/*.md"` (0 errores) · `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs`
- **Deuda pendiente:** los bloques de recitation **ya escritos** en planes no archivados conservan su forma legacy hasta que `updateRecitation` los reescriba (el replace normaliza a fenceado al primer update de esa tarea). Ningún plan vivo tiene bloques hoy.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda (se elimina una excepción permanente en un gate required).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato de arriba, verificado mecánicamente |
| **Commit** | Atómico, `docs(harness): …`, sólo rutas de VantaDB |
| **Release** | N/A (docs/harness; justificado en Notas) |

## Herramientas necesarias
- `npx markdownlint-cli2` (repo pin v0.24.2)
- `node scripts/docs/{check-links,check-docs}.mjs`
- `rg`
- `node --test` (harness, en `.opencode/`)

**Skills cargadas (SDP):** `documentation-skill` (obligatoria: edita `.md` bajo `docs/`) · `campaign-executor` (base) · `progreso` (base) · `doubt-driven-development` (pinned: harness) · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development`

## Investigation Notes

- El template **no** contenía `  - claim:`; el patrón indented vive en `pipeline-full.md` §3 y `task.md` (bloque `contract:`), y el emisor real que lo aplana en el plan es `updateRecitation` (`Contrato: ${data.contract}`). Arreglar el template solo habría documentado un formato que nada aplicaba.
- `docs/dev/plans/**` está en `ignores` del config de markdownlint, así que los planes ni se lintean en el gate: la exclusión del master plan era un filtro sobre un fichero que ya no se emitía. Igual se retira: una exclusión muerta es una invitación a reusarla.
- `check-links.mjs` escanea `proseOf()`, que elimina fences y code spans → fencear las recitations también cierra OBS-1 (links vivos en evidencia).

## Steps

### Step 1: Fencear el bloque en el emisor
- **Archivos:** `.opencode/task-system/mcp/parsers.mjs`
- **Acción:** `updateRecitation` emite `~~~text` … `~~~` alrededor del bloque; el replace normaliza bloques legacy a fenceado.
- **Verify:** `cd .opencode && node --test task-system/mcp/parsers.test.mjs task-system/mcp/hardening.test.mjs` → 34/34 pass (antes y después)
- **Estado:** ✅ COMPLETED

### Step 2: Documentar el formato canónico en el template
- **Archivos:** `.opencode/task-system/prompts/recitation-template.md`
- **Acción:** declarar que el bloque se emite fenceado y qué va dentro del `contract` (sin listas anidadas vivas, links code-spaneados).
- **Verify:** `rg -n 'emite FENCEADO' .opencode/task-system/prompts/recitation-template.md` → 1
- **Estado:** ✅ COMPLETED

### Step 3: Retirar la exclusión del gate
- **Archivos:** `.github/workflows/docs-gates.yml`
- **Acción:** quitar el `grep -v 'master-plan-0.9.0.md'` (líneas 316, 321) y el comentario que justificaba la excepción (305-307); el conteo pasa a ser el árbol completo + guard `test -s` del log.
- **Verify:** `rg -n 'master-plan-0.9.0' .github/ scripts/ dev-tools/ .githooks/` → 0 líneas · `actionlint .github/workflows/docs-gates.yml` → 0
- **Estado:** ✅ COMPLETED

### Step 4: Verify del contrato + cierre
- **Archivos:** `docs/dev/tasks/FIND-264.md`
- **Acción:** correr el contrato completo, commit local.
- **Verify:** markdownlint `docs/**` 0 errores (342 files) · check-links 0 broken · check-docs `all clear` · plan sintético con `  - claim:` → 0 MD007
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (wave 1.1, en paralelo con `FIND-328`, que toca `heavy-certification.yml`, no `docs-gates.yml`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `doubt-driven-development` **degradado** (el sub-agente no puede spawnear reviewers).
- **Enfoque:** ¿fencear en el emisor es el arreglo de causa raíz, o sólo esconde el síntoma? ¿Retirar la exclusión antes de que el corpus esté en 0 reintroduce ruido?
- **Cómo se probó:** tests de parser antes/después (34/34), markdownlint sobre `docs/**` (0), grep de la exclusión (0), `check-links`/`check-docs`.
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas; sin `done` sin correr el contrato; sin degradar gates.
- **Veredicto:** ✅ approve (degradado, marcado como tal)

## Notas
- `.opencode/` es repo Git separado: los cambios ahí **no** entran al commit de VantaDB y se reportan aparte.
- La fila `FIND-264` del Backlog queda para el orquestador (`progreso`); este agente no toca `docs/dev/Backlog.md`.
- Deuda detectada y **no** tocada (fuera de scope, índice generado y compartido): `node scripts/docs/gen-index.mjs --check` falla ya en HEAD (docs/index.md, docs/specs/index.md, docs/user/index.md, llms.txt desactualizados; verificado quitando este task file del árbol). Requiere `gen-index.mjs --write` en un cambio propio.
- Tests del harness corridos: `parsers` + `hardening` 35/35, `state-persistence` + `review-gate` 20/20.
- `check-docs.mjs` marca este task file como huérfano (sin inbound link): es *reporting only*, no gatea.
