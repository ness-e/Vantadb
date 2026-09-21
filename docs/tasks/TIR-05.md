# TIR-05: LLM-as-judge (0.0-1.0)

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 451)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-05-llm-as-judge.md` (DEFERIR con triggers de reapertura)
- **Incógnitas (uphill):** 1 — ¿aplica judge de fabricación a salidas sintéticas del task-system (resúmenes, categorizaciones) vs costo por llamada?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | calidad de verificación de salidas no-deterministas |
| Callees | `agent-03-orchestration.md #35`, `evals/` (verificar existencia), mecanismo compare/verify primitivo |
| Implicaciones | Decisión de diseño del harness de evals — sin cambios de código del producto |

## Contrato
"`docs/Investigaciones/TIR-05-llm-as-judge.md` existe con: (1) inventario de qué salidas del task-system no tienen ground-truth determinista; (2) análisis de aplicabilidad LLM-as-judge vs costo por llamada vs alternativas (heurísticas, checks mecánicos); (3) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep/glob (verificar evals/), web search opcional (mejores prácticas LLM-as-judge)

## Steps
### Step 1: Leer fuentes + inventario
- **Archivos:** `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md` (#35), verificar `evals/` con glob/grep
- **Acción:** leer qué verificación mecánica existe hoy (compare/verify primitivo); identificar salidas sintéticas sin ground-truth
- **Verify:** inventario documentado
- **Estado:** ⬜ PENDING

### Step 2: Analizar aplicabilidad/costo
- **Acción:** LLM-as-judge (0.0-1.0) sobre resúmenes/categorizaciones: qué aporta, qué cuesta (costo por llamada), riesgos (judge sesgado, latencia), alternativas
- **Verify:** análisis documentado
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-05-llm-as-judge.md`
- **Acción:** doc con análisis + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica — justificado.
- Review: lead (orquestador).