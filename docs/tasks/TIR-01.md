# TIR-01: Compaction de contexto runtime

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 448)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-01-contexto-compaction.md` (recomendación: micro-cambio de prompt, decisión del lead)
- **Incógnitas (uphill):** 1 — ¿resumen incremental por fase o suficiente el Context Save Point + escalera retry?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | tareas de ejecución larga / multi-turn en el loop del task-system |
| Callees | `iter-loop-tools.md:178` (Context Save Point), escalera retry (`subagent-recovery.md`), `RULES.md` |
| Implicaciones | Decisión de diseño del harness — si se implementa, toca prompts del task-system, no código del producto |

## Contrato
"`docs/Investigaciones/TIR-01-contexto-compaction.md` existe con: (1) análisis de mecanismos de compactación (resumen incremental por fase, qué se conserva, comparación contra claim original multi-turn compaction); (2) recomendación EXPLÍCITA (implementar / WONTFIT / deferir) con justificación."

## Herramientas
- Read (fuentes internas), grep, campaign MCP opcional

## Steps
### Step 1: Leer fuentes
- **Archivos:** `docs/Investigaciones/2026-08-10-agent-engineering/agent-01-fundaments.md` (#21/#40/#59), `.opencode/task-system/prompts/iter-loop-tools.md:178`, `.opencode/task-system/prompts/subagent-recovery.md`, `.opencode/skills/campaign-executor/RULES.md`
- **Acción:** leer las secciones citadas y el mecanismo actual de Context Save Point + escalera retry
- **Verify:** grep confirma que las líneas citadas existen
- **Estado:** ⬜ PENDING

### Step 2: Analizar alternativas
- **Acción:** evaluar (a) resumen incremental por fase, (b) compactación multi-turn, (c) estado actual (Context Save Point manual + retry fresco ~200 tokens). Costo/beneficio de cada una contra el claim original (multi-turn compaction)
- **Verify:** análisis documentado en el doc de salida
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-01-contexto-compaction.md`
- **Acción:** doc con análisis + recomendación explícita (implementar/WONTFIT/deferir)
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente de TIR-02..08)

## Notas
- Tarea read-only de investigación: NO editar prompts ni código. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica (sin cambios de código) — justificado.
- Review: lo hace el lead (orquestador) al recibir el doc; la decisión final se registra en memoria/backlog.