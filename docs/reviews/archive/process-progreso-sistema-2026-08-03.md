---
title: "Auditoría del Sistema de Tareas — Integración Backlog / Progreso"
type: audit-report
status: completed
tags: [vantadb, audit, task-system, backlog, progreso, docs]
audit_date: 2026-08-03
auditor: vanta-docs
scope: .opencode/ (commands, skills, prompts, config, mcp) vs docs/Backlog.md + docs/progreso/
---

# Auditoría del Sistema de Tareas — Integración Backlog / Progreso

**Fecha:** 2026-08-03
**Alcance:** Cómo el sistema `.opencode/` usa `docs/Backlog.md` y `docs/progreso/README.md`, tras la limpieza del backlog (346→321 líneas), la creación de `docs/progreso/BACKLOG_HISTORY.md`, `docs/progreso/2026-07-28-sdk-gap-audit.md`, `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md` y `.opencode/references/definition-of-done.md`.

## Archivos auditados (leídos completos)

| # | Archivo | Líneas reales | Estado |
|---|---------|---------------|--------|
| 1 | `.opencode/skills/progreso/SKILL.md` | 157 | ✅ alineado con la nueva convención |
| 2 | `.opencode/commands/pipeline.md` | 272 | ✅ pasos 6, 118-123, 168, 186 alineados |
| 3 | `.opencode/commands/status.md` | 56 | ✅ línea 5 correcta (no recarga progreso) |
| 4 | `.opencode/AGENTS.md` | 937 | ✅ líneas 58, 527, 545, 597-599, 621, 818 alineadas; ❌ línea 9 (conteo stale) |
| 5 | `.opencode/VANTADB-OPERATING-MANUAL.md` | 917 | ❌ §7.1/7.4/7.5/7.6 y Apéndice B con conteos stale |
| 6 | `.opencode/task-system/prompts/*.md` (7 archivos) | — | ⚠️ plan.md, task.md, iter-loop-tools.md, pipeline-full.md, pipeline-run.md, audit-full.md |
| 7 | `.opencode/task-system/config/state-tools.mjs` | 89 | ✅ sin referencias a Backlog/progreso (enforcement por estado solo) |
| 8 | `.opencode/task-system/mcp/campaign-server.mjs` | 1241 | ⚠️ líneas 767-768: auto-add de skills base |
| 9 | `.opencode/skills/campaign-executor/SKILL.md` + `RULES.md` | 420 / 413 | ⚠️ RULES.md:199 "Siempre" base-3 |
| 10 | `docs/Backlog.md`, `docs/progreso/README.md`, `BACKLOG_HISTORY.md`, `2026-07-28-sdk-gap-audit.md`, `ADR DRV-014`, `definition-of-done.md` | 321 / 3320 / 68 / 42 / 29 / 90 | ✅ todos existen y reflejan la limpieza |

---

## 1. Desalineaciones con la limpieza (delete/borrar vs tachar + archivar)

**Veredicto general:** el sistema de instrucciones está **alineado** con la nueva convención (tachar `~~…~~` + archivar a `BACKLOG_HISTORY.md`). No se encontró lenguaje "delete row" / "borrar la fila" en archivos de instrucción del sistema. La desalineación residual es mínima y está en archivos históricos o documentos de referencia.

| Archivo | Línea | Problema | Severidad | Fix propuesto |
|---------|-------|----------|-----------|---------------|
| `.opencode/skills/progreso/SKILL.md` | 16, 68, 121 | Usa "táchalo" + "archived to BACKLOG_HISTORY.md" — **correcto**, sin cambio | ✅ OK | — |
| `.opencode/commands/pipeline.md` | 118-123 | Paso 6 "Progreso": "Tachar la tarea como ✅ … items removidos → BACKLOG_HISTORY.md" — **correcto** | ✅ OK | — |
| `.opencode/AGENTS.md` | 597-599 | Progreso Skill (MUST USE): "tacha la fila … items removidos van a BACKLOG_HISTORY.md" — **correcto** | ✅ OK | — |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 462 | §7.5: "se archivan … (no se borran en silencio)" — **correcto** | ✅ OK | — |
| `docs/Backlog.md` | 7, 37 | Header referencia `BACKLOG_HISTORY.md` como historial de verificación — **correcto** | ✅ OK | — |
| `.opencode/skills/campaign-executor/tasks/complete/GH-124.md` | 54 | "borrar al final del test si usa temp dir" — se refiere a archivos temp de ejemplo, **no** a filas de backlog. Histórico | 🟢 Baja | Sin acción (task file histórico completado) |
| `.opencode/skills/campaign-executor/tasks/complete/TSK-104.md` | 38, 53 | "borrar `langchain_rag.py`" — se refiere a un sketch de ejemplo, **no** a filas de backlog. Histórico | 🟢 Baja | Sin acción |
| `docs/progreso/README.md` | 38-44 | Legend lista `✅ Completed / 🟡 In progress / 🔴 Blocked` pero no documenta la semántica de **tachado** (`~~…~~`) ni el archivo `BACKLOG_HISTORY.md` como destino de removidos (el link existe en línea 14) | 🟢 Baja | Agregar fila al Legend: "~~Tachado~~ = completado y migrado (ver BACKLOG_HISTORY.md)" |
| `.opencode/skills/progreso/SKILL.md` | 22 | Invariante "No task exists in both Backlog.md and progreso/README.md" — solo cubre Backlog↔progreso; no menciona explícitamente la triple relación Backlog↔BACKLOG_HISTORY↔progreso | 🟢 Baja | Opcional: extender la invariante para incluir BACKLOG_HISTORY.md |

---

## 2. Conteos stale (números de líneas / cantidades que ya no coinciden)

| Archivo | Línea | Problema | Real | Severidad | Fix propuesto |
|---------|-------|----------|------|-----------|---------------|
| `.opencode/AGENTS.md` | 9 | "VANTADB-OPERATING-MANUAL.md (948 líneas)" | **917** | 🟡 Media | Actualizar a 917 (o eliminar el número; ver Recomendación R2) |
| `.opencode/AGENTS.md` | 11 | "104 skills del proyecto" | 82 (.agents) + 32 (.opencode) = **114** | 🟢 Baja | Recalcular o decir "82+32 skills" |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 49 | "`task-system/prompts/` (8 prompts)" | **7** archivos | 🟢 Baja | Cambiar a "7 prompts" |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 51 | "31 skills engineering + 4 skills VantaDB" | **32** dirs en `.opencode/skills/` (25 engineering + 7 VantaDB) | 🟢 Baja | Actualizar conteo |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 339 | "Las 31 skills de ingeniería…" | 25 engineering | 🟢 Baja | Actualizar conteo |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 396 | "unified-review/SKILL.md (1084 líneas)" | **1198** | 🟡 Media | Actualizar o quitar conteo |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 436 | "review-deep/SKILL.md (370 líneas) + loop-prompt.md (71 líneas)" | **474 / 98** | 🟡 Media | Actualizar o quitar conteo |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 464 | "progreso/SKILL.md (154 líneas)" | **157** | 🟢 Baja | Actualizar a 157 |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 484 | "campaign-executor/SKILL.md (334 líneas) + RULES.md (228 líneas)" | **420 / 413** | 🟡 Media | Actualizar o quitar conteo |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 873-874 (Apéndice B) | "SKILL.md (334L)" / "RULES.md (228L)" | **420 / 413** | 🟡 Media | Actualizar o quitar conteo |
| `.opencode/task-system/prompts/pipeline-full.md` | 27-28 | "campaign-executor/SKILL.md (339L)" / "RULES.md (167L)" | **420 / 413** | 🟡 Media | Actualizar o quitar conteo — RULES.md creció 2.5× |
| `.opencode/skills/progreso/SKILL.md` | 74, 113 | "README.md … 3K+ líneas ~60K tokens" | 3320 líneas / 250 KB ≈ 62K tokens | ✅ OK | Sin cambio (sigue siendo precisa) |
| `.opencode/AGENTS.md` | Skills Manifest | ".agents/skills/ (82 skills) + .opencode/skills/ (32 skills)" | 82 / 32 dirs | ✅ OK | Sin cambio |
| `.opencode/AGENTS.md` | 458 | "references/ (12 total)" | 12 entradas | ✅ OK | Sin cambio |
| `.opencode/VANTADB-OPERATING-MANUAL.md` | 52 | "8 vanta-* agents" | 8 archivos | ✅ OK | Sin cambio |

---

## 3. Referencias rotas (paths que apuntan a archivos inexistentes)

| Path verificado | Estado | Dónde se referencia | Severidad | Fix propuesto |
|-----------------|--------|---------------------|-----------|---------------|
| `docs/Investigaciones.md` | **MISSING** (no existe) | Sin referencias literales en el repo; el sistema usa el directorio `docs/Investigaciones/` (15 archivos) | 🟢 Baja | No hay fix requerido; el directorio es la forma correcta. Si alguien escribe la forma `.md`, se rompe — considerar agregar nota en Path Resolution |
| `tasks/INV-010.md` | **MISSING** literal | No referenciado por commands/prompts. La fila de Backlog está tachada y migrada; el research existe en `docs/Investigaciones/ACID_ROLLBACK_DESIGN.md` y `docs/progreso/README.md:1592` | 🟢 Baja | Resolver siempre vía tabla Path Resolution: `tasks/<ID>.md` → `.opencode/skills/campaign-executor/tasks/<ID>.md` (que **existe**) |
| `docs/plans/2026-07-28-recovery-plan.md` | **EXISTS** ✅ | `docs/progreso/README.md:355`, `docs/progreso/2026-07-28-sdk-gap-audit.md:22`, `tasks/complete/REC-007.md:3` | ✅ OK | Sin cambio |
| `docs/progreso/2026-07-28-sdk-gap-audit.md` | **EXISTS** ✅ (42 líneas) | Referenciado por README/progreso | ✅ OK | Sin cambio |
| `docs/audit-reports/backlog-validation-2026-07-28.md` | **EXISTS** ✅ | `docs/progreso/BACKLOG_HISTORY.md:67` | ✅ OK | Sin cambio |
| `docs/pipeline-state.json` | **MISSING** (runtime) | `pipeline.md:221`, `status.md:18` | 🟢 Baja | No es roto — es un state file runtime con graceful degradation (status.md:51). Sin acción |
| `docs/last-build-state.json` | **MISSING** (runtime) | `status.md:17` | 🟢 Baja | Idem — graceful degradation. Sin acción |
| `docs/last-ship-state.json` | **MISSING** (runtime) | `status.md:29` | 🟢 Baja | Idem. Sin acción |

**Conclusión punto 3:** no hay referencias rotas reales en archivos del sistema. Los tres paths pedidos en el scope se resuelven correctamente (2 vía directorio/tabla de resolución, 1 existe).

---

## 4. Cargas redundantes (skills cargadas donde no hace falta)

| Archivo | Línea | Problema | Severidad | Fix propuesto |
|---------|-------|----------|-----------|---------------|
| `.opencode/task-system/mcp/campaign-server.mjs` | 767-768 | `campaign_load_skills` auto-agrega **siempre** `campaign-executor`, `progreso`, `ponytail (full)` a toda respuesta | 🟡 Media | Es el origen de la redundancia (ver abajo). Mantener acá (dedupe por `Set` ya existe) y **quitar** la tripleta de los prompts que la piden explícitamente |
| `.opencode/task-system/prompts/iter-loop-tools.md` | 8 | "Cargá las skills campaign-executor, progreso, ponytail (full)" **y** luego Step 0 llama `campaign_load_skills` que las vuelve a devolver → doble carga | 🟡 Media | Eliminar la tripleta del prompt; el MCP la garantiza |
| `.opencode/task-system/prompts/pipeline-full.md` | 9 | Idem: tripleta explícita + `campaign_load_skills` la devuelve | 🟡 Media | Idem |
| `.opencode/task-system/prompts/pipeline-run.md` | 9 | Idem | 🟡 Media | Idem |
| `.opencode/skills/campaign-executor/RULES.md` | 199 | Tabla "Siempre: campaign-executor, progreso, ponytail (full)" — cuarta fuente que manda lo mismo | 🟢 Baja | Dejar como fuente canónica (RULES.md), documentar que el MCP la replica |
| `.opencode/commands/pipeline.md` | 13 | Carga 6 skills incondicionalmente para **todos** los modos (incluye `brainstorming`, `writing-plans`, `planning-and-task-breakdown` para `/pipeline run`/`task` donde no aplican) | 🟡 Media | Mover la carga de brainstorming/writing-plans/planning al modo plan (plan.md ya las carga) |
| `.opencode/task-system/prompts/plan.md` | 8 | Carga `brainstorming`, `writing-plans`, `idea-refine` — y `pipeline.md:13` ya pidió brainstorming + writing-plans → doble instrucción para `/pipeline plan` | 🟡 Media | Mantener solo la del prompt (plan.md); quitar de pipeline.md:13 las duplicadas |
| `.opencode/task-system/prompts/audit-full.md` | 3 | Carga `vanta-design-orchestrator` (skill de **diseño UI**) en un pipeline de **auditoría** — irrelevante; además `audit.md:16` ya carga progreso+ponytail → doble | 🟡 Media | Eliminar `vanta-design-orchestrator` (error de copy-paste); dejar solo progreso+ponytail |
| `.opencode/commands/status.md` | 5 | "La skill `progreso` ya se carga en el ritual de inicio — no recargarla acá" | ✅ OK | **Modelo a seguir** — replicar este patrón en los demás |
| `.opencode/AGENTS.md` | 525-530, 543-548 | progreso al inicio y al final de sesión | ✅ OK | Por diseño (Trigger 1/2) |

---

## 5. Recomendaciones (lista priorizada de edits concretos)

### P1 — Alto impacto (hacer primero)

| # | Archivo | Línea | Cambio propuesto |
|---|---------|-------|------------------|
| R1 | `.opencode/task-system/prompts/iter-loop-tools.md` | 8 | Quitar "campaign-executor, progreso, ponytail (full)" del encabezado de carga — `campaign_load_skills` (MCP, campaign-server.mjs:767-768) ya las devuelve y deduplica. Evita doble carga por tarea |
| R2 | `.opencode/task-system/prompts/pipeline-full.md` | 9 | Ídem R1 |
| R3 | `.opencode/task-system/prompts/pipeline-run.md` | 9 | Ídem R1 |
| R4 | `.opencode/task-system/prompts/audit-full.md` | 3 | Eliminar `vanta-design-orchestrator` (skill de diseño irrelevante en auditoría). Dejar: "Cargá las skills `progreso`, `ponytail` (full)." |
| R5 | `.opencode/commands/pipeline.md` | 13, 15 | Simplificar la carga base a "campaign-executor, progreso, ponytail (full)" + `spec-driven-development` solo en modo plan. Dejar brainstorming/writing-plans/planning a cargo de `prompts/plan.md` y `prompts/task.md` |

### P2 — Conteos stale (un batch de ediciones de texto)

| # | Archivo | Línea | Cambio propuesto |
|---|---------|-------|------------------|
| R6 | `.opencode/VANTADB-OPERATING-MANUAL.md` | 396, 436, 464, 484, 873-874 | Actualizar conteos: unified-review 1198, review-deep 474 + loop-prompt 98, progreso 157, campaign-executor 420 + RULES 413. **Alternativa más robusta:** eliminar los números y dejar solo la ruta (ver R12) |
| R7 | `.opencode/task-system/prompts/pipeline-full.md` | 27-28 | Actualizar (339L→420 / 167L→413) o quitar los números — RULES.md duplicó tamaño desde que se escribió |
| R8 | `.opencode/AGENTS.md` | 9 | "948 líneas" → "917 líneas" (o quitar el número) |
| R9 | `.opencode/AGENTS.md` | 11 | "104 skills" → "82 + 32 skills" o recalcular contra SKILLS-MANIFEST.md |
| R10 | `.opencode/VANTADB-OPERATING-MANUAL.md` | 49, 51, 339 | "8 prompts" → "7 prompts"; "31 + 4 skills" → conteo real (32 dirs: 25 engineering + 7 VantaDB) |
| R11 | `.opencode/AGENTS.md` | 458, 621, 818 | Verificados correctos — sin cambio |

### P3 — Baja prioridad / higiene

| # | Archivo | Línea | Cambio propuesto |
|---|---------|-------|------------------|
| R12 | Todos los archivos con conteos de líneas | — | **Regla de oro:** los conteos exactos de líneas son frágiles y se desactualizan con cada PR. Preferir rangos ("~400 líneas") o eliminar el número. Considerar agregar un check en `scripts/validate-docs-coverage.ps1` que verifique los conteos declarados vs reales |
| R13 | `docs/progreso/README.md` | 38-44 | Agregar al Legend: "~~Tachado~~ = completado y migrado (ver `BACKLOG_HISTORY.md` para removidos)" |
| R14 | `.opencode/skills/progreso/SKILL.md` | 22 | Opcional: ampliar la invariante a "No task exists in Backlog.md y progreso/README.md simultáneamente; todo item removido de Backlog queda registrado en BACKLOG_HISTORY.md" |
| R15 | `.opencode/skills/campaign-executor/tasks/complete/GH-124.md`, `TSK-104.md` | 54 / 38,53 | Opcional: nada que hacer (históricos); si se tocan, renombrar "borrar" → "eliminar archivo de ejemplo" para evitar confusión con la convención de backlog |
| R16 | `.opencode/AGENTS.md` | 599 | Verificado correcto — mantener la redacción "tacha + archiva" como fuente canónica del comportamiento |

---

## Resumen ejecutivo

- **Limpieza bien aplicada:** la convención "tachar + archivar a BACKLOG_HISTORY.md" está correctamente implementada en los 4 archivos de instrucción clave (progreso SKILL.md, pipeline.md, AGENTS.md, Operating Manual §7.5) y en los artefactos nuevos (Backlog.md header, BACKLOG_HISTORY.md, definition-of-done.md).
- **Deuda principal = conteos stale:** 12 ocurrencias de números de líneas/cantidades desactualizados, con drift grande en campaign-executor (334→420) y RULES.md (228→413 / 167→413 según la fuente).
- **Redundancia de carga de skills:** el MCP (`campaign-server.mjs:767-768`) y 3+ prompts piden la misma tripleta base; `audit-full.md` carga una skill de diseño en un flujo de auditoría.
- **Sin referencias rotas:** los paths verificados se resuelven correctamente.
- **Modelo positivo:** `status.md:5` ya evita la recarga redundante de progreso — replicar ese patrón.
