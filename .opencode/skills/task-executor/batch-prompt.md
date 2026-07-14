# Batch Task Loop — Multi-Tarea en Secuencia

> **Modo:** Ejecución en lote. Usá esto cuando tenés N tareas pendientes en un plan file
> y querés que se ejecuten una tras otra (o en paralelo) automáticamente.
>
> Cada tarea se delega al single-task executor (`loop-prompt.md`) individualmente.
> Tareas sin dependencias entre sí pueden ejecutarse en **paralelo** (Fase 1.5).
> Si una falla, decidís si seguir o parar.
>
> **Anchor:** `.opencode/skills/task-executor/VISION.md` — leer antes de empezar.
> **Patrón 2026:** orchestrator workers — el batch actúa como orchestrator que
> delega a N sub-agentes worker. Parallel cuando es posible, secuencial cuando
> hay dependencias.

## Entrada

Argumentos del loop:
- `PLAN_FILE` = ruta al plan file (ej: `docs/plans/2026-07-13-verified-backlog-execution.md`)
- `FAIL_MODE` = `stop` | `skip` | `parallel` (default: `stop`)
  - `stop`: se para ante la primera falla
  - `skip`: registra fallo y sigue
  - `parallel`: ejecuta tareas independientes en paralelo vía sub-agentes concurrentes
- `FILTER` = regex opcional para filtrar tareas por ID (ej: `NUEVO-|VFY-`)

## Flujo General

```
ITERACIÓN PRINCIPAL (una por tarea):
  1. LEER plan file → encontrar próxima tarea ❌ o 🟡
  2. VERIFICAR filtro (si FILTER está seteado)
  3. EJECUTAR single-task para esa tarea
  4. ACTUALIZAR plan file (marcar ✅ o ❌)
  5. ACTUALIZAR AGENTS.md con metadata de la ejecución
  6. Si FAIL_MODE=stop y falló → DETENER
  7. Si FAIL_MODE=skip y falló → LOG y CONTINUAR
  8. Repetir hasta que no queden tareas
```

---

## Fase 0: Inicialización

### 0.1 Anchor
Leer `.opencode/skills/task-executor/VISION.md` — principios invariantes #7 (progreso visible)
y #9 (presupuesto finito) aplican directamente al batch.

### 0.2 Cargar skills base
```
skill progreso
skill planning-and-task-breakdown
```

### 0.3 Budget Controls

Al iniciar, definir y registrar presupuesto para TODO el batch:

| Control | Default | Límite duro |
|---------|---------|-------------|
| `MAX_ITER` | 50 | 200 |
| `MAX_CONSECUTIVE_FAIL` | 3 | 10 |
| `NO_PROGRESS_LIMIT` | 5 tareas seguidas fallando | 10 |
| `MAX_TURNS_PER_TASK` | 30 | 60 |
| `MAX_COST` (reservado) | 0.00 (sin medir aún) | — |

```
openCode loops may set these via --max-turns on the goal, but batch enforces its own ceilings:
- Si MAX_ITER se alcanza → stop ordenado con reporte parcial
- Si MAX_CONSECUTIVE_FAIL se alcanza → FAIL_MODE pasa a "stop" automáticamente
- Si NO_PROGRESS_LIMIT se alcanza → opencode_loop_goal_blocked
  reason:"Batch stagnation: {N} tareas consecutivas fallando"
  needed:"Revisar plan file y decidir si continuar"
```

Referencia: explainx.ai "3 hard stops" — token/iteration/cost ceilings (2026).

### 0.4 Leer plan file
Leer `$PLAN_FILE`. Extraer la tabla de tareas global.

Localizar TODAS las filas donde `Estado` no es ✅ ni ❌ SKIP:
- `❌` → tarea pendiente
- `🟡` → tarea diferida (si FILTER la incluye, intentar)

### 0.5 Armar cola de ejecución
Orden de ejecución:
1. Por prioridad (Tier 0 > 1 > 2 > 3)
2. Por dependencias (si una tarea dice "depende de X", X va antes)

Registrar:
```
Tareas encontradas: N
Primera tarea: ${ID}
Filtro activo: ${FILTER:-ninguno}
Modo fallo: ${FAIL_MODE}
```

```
opencode_loop_goal_progress summary:"Batch init: N tareas pendientes, primera=${ID}" next:"Ejecutar ${ID}"
```

---

## Fase 1: Ejecutar una tarea

### 1.1 Cargar single-task executor
Invocar un sub-agente (`task`) con el prompt del single-task executor:

```
Sub-agent prompt:
  1. Leer tasks/$ID.md si existe (definición previa)
  2. Si no existe: correr FASE A (Discovery) + FASE B (Definition) del loop-prompt.md
  3. Ejecutar FASE C (Execution) completa
  4. Reportar: ✅ | ❌ + evidencia
```

Ejecutar el sub-agente con los checks correspondientes según el tipo de tarea.

### 1.2 Esperar resultado
El sub-agente devuelve:
- `status`: `completed` o `failed`
- `evidence`: resumen de verificación
- `contract_met`: true/false
- `error_colateral`: lista de issues encontrados (si aplica)

### 1.3 Si completó (✅)
```
1. Actualizar contadores de budget:
   - CONSECUTIVE_FAIL = 0 (reset)
   - CONSECUTIVE_SUCCESS++
   - TOTAL_DONE++

2. Actualizar $PLAN_FILE:
   - Marcar la fila como ✅
   - Actualizar recitation block con "Última tarea completada: ${ID}"
   - Si la fila decía 🟡 DEFER, cambiar a ✅ y quitar DEFER

3. skill progreso (Trigger 1: migrar a progreso, Backlog, bitácora, CHANGELOG)

3. AGENTS.md — agregar entry:
   ## Batch Learnings
   - ${ID}: completada en $(date +%Y-%m-%d)
   - Contrato: ${contract_met}
   - Archivos tocados: (lista)
   - Skills usadas: (lista)

4. Registrar progreso:
   opencode_loop_goal_progress summary:"✅ ${ID} completada" next:"Siguiente tarea: ${NEXT_ID} (N restantes)"
```

### 1.4 Si falló (❌)
```
1. Actualizar $PLAN_FILE:
   - Marcar la fila como ❌ ERROR
   - Agregar nota con el error

2. Actualizar contadores de budget:
   - CONSECUTIVE_FAIL++
   - TOTAL_FAIL++
   - CONSECUTIVE_SUCCESS = 0

3. Registrar progreso:
   opencode_loop_goal_progress summary:"❌ ${ID} falló: ${error}" next:"Decidir continuación"

4. Budget enforcement (before deciding stop/skip):
   Si CONSECUTIVE_FAIL >= MAX_CONSECUTIVE_FAIL:
     - FAIL_MODE pasa a "stop" forzosamente
     - opencode_loop_goal_blocked
       reason:"${MAX_CONSECUTIVE_FAIL} fallos consecutivos — límite alcanzado"
       needed:"Revisar plan file y errores"

5. Si FAIL_MODE=stop:
   - Informar al usuario y detener
   - opencode_loop_goal_blocked reason:"${ID} falló" needed:"Revisar error manualmente"

6. Si FAIL_MODE=skip:
   - Registrar en log de fallos
   - Si TOTAL_FAIL >= NO_PROGRESS_LIMIT → stop igual (stagnation)
   - CONTINUAR con siguiente tarea
```

---

## Fase 1.5: Ejecución Paralela (FAIL_MODE=parallel)

> **Patrón orchestrator workers (2026):** el batch actúa como orchestrator que
> delega tareas independientes a N sub-agentes worker en paralelo, luego
> mergea resultados.

### 1.5.1 Identificar tareas paralelizables

```
Analizar cola de tareas:
  1. Construir grafo de dependencias entre tareas:
     - Tarea A dice "depende de X" → arista X → A
     - codegraph_explore en archivos de cada tarea para detectar conflictos
       compartidos (mismo archivo, mismo módulo)
     - Si dos tareas tocan archivos DIFERENTES y no hay arista de dep → paralelizable

  2. Agrupar por "wave" (nivel de profundidad en DAG):
     - Wave 0: tareas sin dependencias (ejecutar primero, en paralelo)
     - Wave 1: tareas que dependen de Wave 0
     - Wave 2: tareas que dependen de Wave 1
     - ...
     - Cada wave espera a que la anterior termine

  3. Límites de paralelismo:
     - MAX_CONCURRENT = min(4, tareas_en_wave)  (default: 4 workers)
     - No paralelizar si wave tiene menos de 2 tareas
```

### 1.5.2 Ejecutar wave en paralelo

```
Para cada wave (0, 1, 2, ...):
  1. Lanzar N sub-agentes concurrentes (task), uno por tarea de la wave
  2. Cada sub-agente recibe el mismo prompt que Fase 1.1
  3. ESPERAR a que TODOS terminen (join)
  4. Recolectar resultados:

     task-agent-$ID-1 → { status, evidence, contract_met, error_colateral }
     task-agent-$ID-2 → { status, evidence, contract_met, error_colateral }
     ...

  5. Procesar resultados individualmente (igual que Fase 1.3 / 1.4):
     - ✅: actualizar plan, reset contadores
     - ❌: incrementar contadores, budget check

  6. Si alguna tarea falló y FAIL_MODE=parallel:
     - Las demás tareas de la wave que ya arrancaron TERMINAN (no se matan)
     - Las waves siguientes NO arrancan
     - Reporte parcial con fallos
     - opencode_loop_goal_blocked
       reason:"Parallel wave ${WAVE}: ${N_FAIL} tareas fallaron"
       needed:"Revisar errores y decidir si re-ejecutar"
```

### 1.5.3 Merge de resultados

```
Después de completar todas las waves:
  1. Mergear cambios: cada sub-agente ya commiteó individualmente
  2. Verificar que no hay conflictos entre tareas paralelas:
     - git log --oneline después del último commit secuencial
     - Si hay conflictos: informar, no resolver automáticamente
  3. Reporte consolidado:

     Wave 0: ${OK_W0}/${TOTAL_W0} tareas (paralelo)
     Wave 1: ${OK_W1}/${TOTAL_W1} tareas (paralelo)
     ...
     Total: ${OK}/${TOTAL} tareas
```

Referencia: patrón orchestrator workers — Anthropic building effective agents (2026),
TaskWeaver (Microsoft, 9k⭐) plan-and-execute concurrent planner.

---

## Fase 2: Siguiente tarea o finalizar

### 2.1 Buscar siguiente tarea
```
1. Releer $PLAN_FILE
2. Buscar próxima ❌ o 🟡 que pase FILTER
3. Si existe → volver a Fase 1 con NEXT_ID
4. Si no existe → FIN
```

### 2.2 Reporte final
Cuando se completaron todas las tareas:

```
Reporte Batch:
├─ Total: N
├─ ✅ Completadas: M
├─ ❌ Falladas: K
├─ ⏭️ Skipeadas: 0
├─ Tiempo total: (estimado)
└─ Errores colaterales encontrados: L
   - Rápidos (arreglados): X
   - Diferidos a Backlog: Y
```

```
opencode_loop_goal_complete
  summary:"Batch completo: ${M}/${N} tareas ejecutadas"
  evidence:"plan file actualizado, progreso migrado"
```

---

## Diagrama de flujo

```
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/plan.md" "FAIL_MODE=skip"
  │
  ├─ Fase 0: Leer plan → presupuesto (MAX_ITER, budget) → cola
  │
  ├─ Fase 1: [SECUENCIAL] Ejecutar UNA tarea vía sub-agente
  │   ├── ¿✅? → reset consecutivo, actualizar plan, progreso, AGENTS.md
  │   └── ¿❌? → incrementar contadores → budget check → stop/skip
  │
  ├─ Fase 1.5: [PARALELO] Si FAIL_MODE=parallel
  │   ├── 1.5.1: Identificar waves (DAG de dependencias)
  │   ├── 1.5.2: Ejecutar wave en N sub-agentes concurrentes
  │   └── 1.5.3: Mergear resultados y verificar conflictos
  │
  ├─ Fase 2: ¿Quedan tareas?
  │   ├── Sí → volver a Fase 1 o 1.5 según FAIL_MODE
  │   └── No → reporte final + complete
  │
  └── Fin
```

## Modo de uso

```bash
# Ejecutar todas las tareas pendientes de un plan, parar si falla
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/2026-07-13-verified-backlog-execution.md"

# Ejecutar solo tareas NUEVO-*, skipeando fallos
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/2026-07-13-verified-backlog-execution.md" \
  "FILTER=NUEVO-" "FAIL_MODE=skip"

# Ejecutar tareas independientes en paralelo (orchestrator workers)
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/2026-07-13-verified-backlog-execution.md" \
  "FAIL_MODE=parallel"
```
```
