# TIR-07: Chaos runner del task-system (TSYS-06 runtime)

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 453)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🔴 2-3d (investigación: 🟢 — el effort 🔴 es si se implementa el runner)
- **Prioridad:** 🟢
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 5-10 (investigación)
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-07-chaos-runner.md` (DEFERIR runner; cerrar gaps C8/C9 con tests puntuales)
- **Incógnitas (uphill):** 1 — ¿runner que fuzzee `campaign-server.mjs`/máquina de estados vale frente a tests de inyección de fallos puntuales?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | robustez del MCP server (`campaign-server.mjs`, máquina de estados C0) |
| Callees | `docs/architecture/task-system-chaos-resilience.md` (T19), `TSYS-06`, `gap-01 §3.3-24` (en agent-engineering) |
| Implicaciones | Decisión de inversión en testing de robustez del harness — sin cambios de código |

## Contrato
"`docs/Investigaciones/TIR-07-chaos-runner.md` existe con: (1) análisis del diseño existente (`task-system-chaos-resilience.md`, T19) y qué cubriría un runner (fuzzing de campaign-server.mjs / máquina de estados / SARL); (2) comparación runner vs tests de inyección de fallos puntuales (costo/esfuerzo/valor); (3) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep/glob (verificar campaign-server.mjs), bash read-only

## Steps
### Step 1: Leer fuentes
- **Archivos:** `docs/architecture/task-system-chaos-resilience.md` (T19), `docs/Investigaciones/2026-08-10-agent-engineering/gap-01.md` (§3.3-24), `.opencode/task-system/mcp/campaign-server.mjs` (estructura, máquina de estados)
- **Acción:** leer el diseño existente y el alcance real de un runner (qué fuzzear, cómo detectar fallos)
- **Verify:** alcance documentado
- **Estado:** ⬜ PENDING

### Step 2: Comparar opciones
- **Acción:** runner de fuzzing dedicado vs tests de inyección de fallos puntuales (estados inválidos, transiciones ilegales, payloads malformados). Costo de construir/mantener runner vs valor
- **Verify:** comparación documentada
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-07-chaos-runner.md`
- **Acción:** doc con análisis + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación. NO implementar el runner. NO commitear (el lead commitea).
- SECURITY: no aplica (sin cambios de código) — justificado. PERFORMANCE: no aplica.
- Review: lead (orquestador).