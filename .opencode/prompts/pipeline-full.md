Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — UNA TAREA COMPLETA POR ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA TAREA COMPLETA
por iteración y te detenés. No intentes continuar ni loopear. El loop externo lo maneja `/loop-goal`.

Las reglas detalladas están en `skills/campaign-executor/SKILL.md` (339L)
y `skills/campaign-executor/RULES.md` (167L). Seguilas exactamente.

## Flujo pipeline (una tarea completa por iteración)

### 1. OBTENER SIGUIENTE TAREA

Llamá a `campaign_get_task` (sin args — detecta automáticamente el plan más reciente).

Casos:
- `hasTask == false` → **campaña completada**. Ejecutá `skill progreso` y detenete. No hay más que hacer.
- Error → informalo y detenete.
- Task devuelta → tenés `id`, `name`, `files`, `contract`, `state`, `summary`, `recitation`.

### 2. EJECUTAR TAREA COMPLETA SEGÚN ESTADO

#### ⬜ PENDING → Discovery + Implementación + Cierre

**Discovery (task definition):**
- Auto-detectá tipo (Rust/Frontend/Python/TS/Docs/Mixto) según `Archivos clave`
- Cargá skills adicionales según tipo (RULES.md §10):
  - Rust → `source-driven-development`
  - Frontend → `frontend-ui-engineering`
  - Bug reportado → `systematic-debugging`
  - API pública → `api-and-interface-design`
- `codegraph_explore` para blast radius (nombrando los `Archivos clave` de la task)
- Web research (MetaSearchMCP/Argus) si hay ambigüedad en APIs externas
- Descomponé en steps atómicos usando template de `skills/campaign-executor/templates/task-definition.md`
- Creá task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`

**Implementación:**
- Llamá `campaign_update_task` con `"in-progress"` y recitation
- State machine: PLAN → ACT → VERIFY por cada step (~100 líneas por step)
- Si verify falla: retry ladder (RULES.md §9):
  1. Retry con feedback procesado
  2. Contexto fresco (~200 tokens resumen)
  3. Estrategia materialmente distinta
  4. ❌ FAILED → escalar a humano
- Evaluator-Optimizer: correctitud, simplicidad, consistencia
- Self-Harness Gate: propose → evaluate → accept
- Pre-commit Gate: Definition of Done + checklists por tipo
- Budget: máx 5 iteraciones por tarea (de estas iteraciones de loop, no steps)

**Cierre:**
- Verify full:
  1. `campaign_verify command="cargo fmt --check"`
  2. `campaign_verify command="cargo clippy --workspace --all-targets --all-features -- -D warnings"`
  3. `campaign_verify command="cargo nextest run --profile audit --workspace --build-jobs 2"`
  4. `campaign_verify command="scripts/validate-docs-coverage.ps1"`
- Si todo pasa: `git add -A && git commit -m "feat: <ID> — <name>"`
- Llamá `campaign_update_task` con `"completed"` y recitation
- Auto-mejora (RULES.md §10): evaluá qué fue más difícil de lo esperado

**Progreso:**
- Ejecutá `skill progreso` — mueve la tarea de docs/Backlog.md → docs/progreso/README.md

#### ⏳ IN PROGRESS → Continuar hasta completar

- Leé la recitation de `campaign_get_task` para saber dónde quedó
- Continuá con el próximo step (PLAN → ACT → VERIFY)
- Si verify falla: retry ladder (mismo que arriba)
- Errores colaterales: rápido (<30min) se arregla, lento se difiere a Backlog
- Budget: 5 iteraciones máximas por tarea, 2 stalls consecutivos → ❌ FAILED

**Cuando el último step esté completo + verificado + commiteado:**
- Llamá `campaign_update_task` con `"completed"` y recitation
- Ejecutá `skill progreso`

#### ❌ FAILED

- Anotá por qué falló y qué se intentó (los 4 escalones si aplica)
- Llamá `campaign_update_task` con `"failed"`
- Ejecutá `skill progreso` para registrar en docs/progreso/
- Detenete. No sigas a la siguiente tarea.

### 3. ACTUALIZAR RECITATION

Después de cada acción, llamá `campaign_update_task` con:
- `taskId`: ID de la tarea
- `newState`: `"completed"` | `"failed"` | `"in-progress"`
- `recitation`:
  - `activeGoal`: qué se estaba haciendo
  - `lastAction`: qué se hizo en esta iteración
  - `result`: ✅ o ❌
  - `nextAction`: próximo paso concreto (archivo + comando)
  - `contract`: qué comando verifica que está bien
  - `nextTask`: ID de la próxima tarea a ejecutar si completa

Sync el task file si aplica.

### 4. HANDOFF

Después de completar una tarea, dejá la recitation apuntando a la siguiente tarea.
El `/loop-goal` externo recogerá la próxima iteración.

### 5. EJECUCIÓN MULTI-TAREA

Si el usuario quiere ejecutar MÁS de una tarea, usá `/pipeline run` que llama este
mismo prompt por cada tarea vía sub-agentes. No intentes loopear vos mismo.

```
/pipeline run [plan]
```

### 6. REFERENCIA RÁPIDA

| Modo | Comando | Qué hace |
|------|---------|----------|
| Una tarea | `/pipeline task ID` o `/loop-goal "..."` | Este prompt: una tarea completa |
| Todas | `/pipeline run` | Usa sub-agentes, llama a este prompt por tarea |
| Plan | `/pipeline plan backlog.md` | Crea plan desde backlog |

REGLAS (del campaign-executor RULES.md):
- NO leas el plan file directamente — usá `campaign_get_task`
- El contrato es ley — si no se cumple, la tarea no está completa
- Verificación mecánica, nunca auto-reporte
- Si verify falla 2 veces con mismo error → ❌ FAILED
- Ponytail ladder: existe > stdlib > dependency > mínimo código
- ~100 líneas por step, un step por turno, cada step reversible
- No cambies scope. Rápido se arregla, lento se anota en Backlog
- Stagnation = stop: 3 vueltas sin progreso → ❌ FAILED
- Budget: 5 iteraciones máximas por tarea, 2 stalls consecutivos → FAILED
- La recitation es el handoff entre iteraciones
- Después de completar una tarea, DETENETE. No sigas a la siguiente.
