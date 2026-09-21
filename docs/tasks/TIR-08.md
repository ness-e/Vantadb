# TIR-08: Criterios de investigación (saturación + re-enfoque) en research-agent

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-desktop-ux-core.md (Task 7)
- **Fuente:** docs/Backlog.md P18 TIR-08 — decisión "IMPLEMENTAR parcial (criterios 1 y 2 en research-agent.md ~6 líneas)"
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Docs (prompts del task-system)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-25T10:00
- **last-synced:** 2026-08-25T10:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `commands/pipeline.md` (path resolution `prompts/X.md`), `prompts/pipeline-full.md` (Paso 0 sub-agentes research), `subagent-recovery.md` (SARL) |
| Callees | ninguno — archivo standalone, sin imports |
| Implicaciones | ninguna — adición de 5 líneas de criterios accionables en un prompt; no toca código, API, ni contratos de bindings |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición.

- **Archivos leídos (completos):** `.opencode/task-system/prompts/research-agent.md` (33 líneas, leído completo), `.opencode/task-system/prompts/pipeline-full.md` (274 líneas), `.opencode/task-system/prompts/task.md` (376 líneas), `docs/plans/2026-08-25-batch-desktop-ux-core.md` (231 líneas)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** ninguno — prompt standalone
- **Archivos que referencian a los editados (referencias entrantes):** grep `research-agent` en `.opencode/` → `commands/pipeline.md` (resolución `prompts/X.md`), `prompts/pipeline-full.md` (Paso 0), `prompts/subagent-recovery.md`; `.opencode/task-system/agents/research-agent.md` NO existe (Test-Path false)
- **Veredicto impacto:** ⚠️ HALLAZGO — el archivo YA contiene los criterios TIR-08 (líneas 30-33), commiteados en `1c7660dc` (2026-08-22, "TIR-08c criterios research"). `git diff HEAD` = vacío. **NO hay nada que editar** — duplicar sería un error. La tarea se cierra por verificación, no por edición.

## Contrato
"research-agent.md contiene los criterios 1-2; verificación con rg"
→ `rg -n "saturación|Re-enfoque|narrowing|broadening|TIR-08" .opencode/task-system/prompts/research-agent.md` → líneas 30-33 presentes.

## Spec (SDD)

> N/A justificado: tarea 100% docs/prompt sin decisiones técnicas nuevas — la
> decisión fue tomada y registrada en P18 (backlog) y ya implementada en
> `1c7660dc`. No hay símbolos públicos, endpoints ni bindings involucrados
> (Phase 1b: sin señales de feature-add).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no modificar otros prompts del task-system (regla del lead: otra sesión editó `*.md`, commit d8bf0e2a); formato de `research-agent.md` intacto; sin duplicar los criterios ya presentes.
- **Comandos de verificación:** `rg -n "saturación|Re-enfoque|narrowing|broadening|TIR-08" .opencode/task-system/prompts/research-agent.md` → 4 matches (líneas 30-33)
- **Deuda pendiente:** ninguna (implementación completa en commit previo)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | Implementar criterios de investigación TIR-08 (stop por saturación <20% + broadening/narrowing) en research-agent.md |
| `lastAction` | DISCOVERY: lectura completa de research-agent.md + plan file; hallazgo — criterios ya implementados y commiteados en 1c7660dc (líneas 30-33); diff vs HEAD vacío; task file creado |
| `result` | `OK` |
| `nextAction` | Lead: verifica `rg` (contrato), NO hay commit pendiente (cambio ya en 1c7660dc), marca TIR-08 ✅ en plan |
| `contract` | ver abajo |
| `nextTask` | FIND-11 (Task 8, mismo plan) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (cero ediciones — tarea de verificación/cierre)

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato verificable (`rg` criterios 1-2 en research-agent.md) | ✅ pasa |
| **Commit** | Sin diff propio (cambio ya commiteado en `1c7660dc`); convención cumplida en commit previo | ✅ no aplica diff nuevo |
| **Release** | `dev-tools/verify.ps1` — NO aplica (docs/prompt, sin código; justificado en Notas) | N/A |

## Herramientas necesarias
- git (log/show/diff read-only — verificación de estado)
- rg (verificación del contrato)

**Skills cargadas (SDP):** base-only (campaign-executor, progreso, ponytail — auto-cargadas vía MCP) + SDP sin candidatos adicionales — tarea de verificación/cierre sin edición de contenido; decisión TIR-08 ya registrada en backlog/prompt; sin prosa nueva ni ADR que documentar.

## Investigation Notes
- `git log --oneline -8 -- .opencode/task-system/prompts/research-agent.md` → último commit que tocó el archivo: `1c7660dc` "perf(task-system): TIR-02a dora recovery time + TIR-04b contenedor tasks/closed + TIR-08c criterios research (GOV-T01..03)"
- `git show 1c7660dc -- .opencode/task-system/prompts/research-agent.md` → +5 líneas exactamente: header "Criterios de investigación (TIR-08):", criterio 1 (STOP saturación <20%), criterio 2 (Re-enfoque narrowing/broadening), + línea jitter WONTFIT con ref TIR-08
- `.opencode/task-system/agents/research-agent.md` NO existe → el archivo canónico es `prompts/research-agent.md` (confirmado)
- Task file TIR-08.md no existía en tasks/, tasks/complete/ ni tasks/closed/ (glob 0 resultados)

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — implementación verificada en commit 1c7660dc |
| Pendientes de ejecución (downhill) | 1 — formalizar cierre (task file + estado) |
| % completado | 90% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: no toca trust boundaries, input, auth, datos ni dependencias (edición de prompt interno, y además sin edición).
- [ ] **PERFORMANCE** — NO aplica: no toca hot paths (búsqueda, indexación, serialización).

## Steps

### Step 1: Verificar contrato con rg (criterios 1-2 presentes)
- **Archivos:** `.opencode/task-system/prompts/research-agent.md`
- **Acción:** ejecutar verificación mecánica del contrato del plan; NO editar (contenido ya existe en commit 1c7660dc)
- **Verify:** `rg -n "saturación|Re-enfoque|narrowing|broadening|TIR-08" .opencode/task-system/prompts/research-agent.md` → 4 matches (líneas 30-33) ✅ ejecutado: línea 30 header, 31 STOP saturación, 32 Re-enfoque, 33 jitter WONTFIT
- **Estado:** ✅ COMPLETED

### Step 2: Formalizar cierre (task file + estado plan)
- **Archivos:** task file TIR-08.md, estado de tarea en plan
- **Acción:** documentar evidencia + actualizar `campaign_update_task_state` a completed con recitation
- **Verify:** RESULTADO estructurado devuelto al orquestador
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0, sin dependencias previas)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead (verificación mecánica final del contrato `rg`; verificación de que NO hay diff pendiente que commitear)
- **Enfoque:** el approach correcto es NO editar — el contenido ya está implementado y commiteado; duplicar habría introducido ruido y roto el formato.
- **Cómo se probó:** `git diff HEAD -- research-agent.md` vacío + `git show 1c7660dc` muestra el diff de +5 líneas con los criterios exactos + `rg` confirma líneas 30-33.
- **Checklist anti-hábitos tóxicos:** verificado — sin salidas inventadas (todas las tool results reales), sin "ya sé qué quiere" (lectura completa previa), sin done sin verificar contra acceptance criteria (rg ejecutado), sin fallos ignorados, sin búsqueda única (git log + git show + glob + Test-Path), sin copiar sin citar (commit hash citado), sin reintentos en bucle, sin huérfanos (steps conectados al objetivo), sin degradar errores, sin gasto infinito (1 round de verificación).
- **Veredicto:** ✅ approve (pendiente confirmación final del lead)

## Notas
- Regla 6: sin deuda nueva; el plan ya fue modificado por otra sesión (git status M en plan) — no tocar ese diff.
- Context Save Point: cero ediciones realizadas; único artefacto nuevo = task file.