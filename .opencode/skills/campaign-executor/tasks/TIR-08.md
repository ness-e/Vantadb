# TIR-08: Saturación <20% + Broadening/Narrowing + jitter en retry

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 454)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-08-saturacion-broadening-jitter.md` (IMPLEMENTAR parcial en research-agent.md; WONTFIT jitter)
- **Incógnitas (uphill):** 1 — ¿formalizar criterios en `iter-loop-tools.md`/`task.md` o son guías tácitas?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | prompts de iteración/retry del task-system |
| Callees | `agent-02-task-execution.md §7.2/§7.6/§8.1`, `RULES.md:453` (exponential backoff determinista) |
| Implicaciones | Decisión de formalización de prompts — sin cambios de código del producto |

## Contrato
"`docs/Investigaciones/TIR-08-saturacion-broadening-jitter.md` existe con: (1) inventario de los 3 criterios (saturación <20% como stop, broadening/narrowing como re-enfoque, jitter en retry) y dónde viven hoy (solo en investigación, no en prompts); (2) análisis formalizar-en-prompts vs guías tácitas (riesgo de ruido vs costo de mantenimiento); (3) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep

## Steps
### Step 1: Leer fuentes
- **Archivos:** `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md` (§7.2/§7.6/§8.1), `.opencode/skills/campaign-executor/RULES.md:453`, `.opencode/task-system/prompts/iter-loop-tools.md`
- **Acción:** leer los 3 criterios en la investigación; verificar qué está ya en prompts vs solo en la investigación
- **Verify:** inventario con líneas exactas
- **Estado:** ⬜ PENDING

### Step 2: Analizar formalización
- **Acción:** costo/beneficio de formalizar en iter-loop-tools.md/task.md (consistencia, enforceability) vs dejarlo tácito (flexibilidad, menos ruido de prompts)
- **Verify:** análisis documentado
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-08-saturacion-broadening-jitter.md`
- **Acción:** doc con análisis + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación. NO editar prompts. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica — justificado.
- Review: lead (orquestador).