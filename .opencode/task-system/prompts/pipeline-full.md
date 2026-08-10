> **ACTIVE INSTRUCTION — Execute Complete Task**
> Cargado por `commands/pipeline.md` (modo TASK, ejecución NOW) o por `/pipeline run` vía sub-agente.
> Path resolution: `skills/X` → `.opencode/skills/X/`, `tasks/ID.md` → `.opencode/skills/campaign-executor/tasks/ID.md`
> Ejecutar UNA TAREA COMPLETA por invocación: discovery → implementación → cierre.
> Seguir el flujo según estado (PENDING / IN PROGRESS / FAILED).
> Al finalizar: commit, actualizar plan file, ejecutar skill progreso, handoff y STOP.
> NO continuar a la siguiente tarea — el loop externo (pipeline-run / sub-agentes) lo maneja.
> **PROFUNDIDAD UNIFICADA:** este prompt es la forma canónica de ejecutar UNA tarea
> (usado por `/pipeline task`, `/pipeline run` y vanta-lead). Incluye el DISCOVERY completo
> (crea el task file si no existe) y el cierre completo. Si no podés terminar, devolvé
> el bloque `RESULTADO` del § Resultado con el trabajo hecho — **nunca te detengas en silencio**;
> el orquestador reanuda vía `subagent-recovery.md` (SARL).

Las skills base (campaign-executor, progreso, ponytail) se cargan automáticamente vía MCP — no las cargues manualmente.

Paso 0 — Auto-cargar skills según tipo de tarea:
   Llamá `campaign_get_next_task` (MCP) para obtener la tarea activa.
   Con los `Archivos clave`, llamá `campaign_load_skills` (MCP) que devuelve
   skills + checks exactos. Ejecutá `skill <nombre>` para CADA skill.
    Si es bug → además `systematic-debugging`. Si es lógica nueva →
    `test-driven-development`. Si es security-sensitive → `doubt-driven-development`.
    Llamá `campaign_get_workflow` (MCP) con el tipo detectado para cargar el
    workflow JSON (bug-fix/feature-add/refactor/research/nine-second-saloon).
    El workflow define estados, allowed_tools y transiciones específicas.

INSTRUCCIONES — UNA TAREA COMPLETA POR ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA TAREA COMPLETA
por invocación y te detenés. El loop externo lo maneja el agente que te invocó
(/pipeline run via sub-agentes, o /loop-goal si usás el approach manual).

Las reglas detalladas están en `skills/campaign-executor/SKILL.md` (420L)
y `skills/campaign-executor/RULES.md` (413L). Seguilas exactamente.

## Flujo

### 1. LEER plan file directamente

Usá `campaign_get_next_task` (MCP) para obtener la tarea, o leé el plan file si ya lo tenés. Si se te pasó el plan file por
argumento, leelo con Read tool. Si no, buscá el más reciente en `docs/plans/`.

Buscá la tarea con el ID que te pasaron. Si está ⬜ PENDING o ⏳ IN PROGRESS,
ejecutala. Si está ✅ o ❌, informalo y detenete.

### 2. EJECUTAR TAREA COMPLETA SEGÚN ESTADO

#### ⬜ PENDING

**Discovery:**
- Llamá `campaign_detect_task_type` (MCP) con `Archivos clave` → type, skills, checks
- Cargá skills devueltos con `skill <nombre>`
- Si es bug → además `systematic-debugging`
- Si es security-sensitive → `doubt-driven-development`
- Si es lógica nueva/compleja → `test-driven-development`
- `codegraph_explore` para blast radius (nombrando los `Archivos clave` de la task)
- Web research (MetaSearchMCP/Argus) si hay ambigüedad en APIs externas
- Descomponé en steps atómicos
- Creá task file en `.opencode/skills/campaign-executor/tasks/<ID>.md` SI NO EXISTE.
  **Si ya existe** (tarea reanudada tras un intento previo), LEELO y continuá desde el primer
  step ⬜ PENDING — **no re-hagas steps ya ✅ ni pisés el trabajo hecho.**
  El trabajo parcial vive en el worktree (git diff) y en el task file: respetalo.

**Implementación:**
- Llamá `campaign_update_task_state` con `"in-progress"` y recitation
- State machine: PLAN → ACT → VERIFY por cada step (~100 líneas por step)
  * Antes de ACT → `campaign_validate_command` (MCP) para validar el comando
  * Si el comando es riesgoso → `campaign_run_sandboxed` (MCP)
  * En cada transición de estado → `campaign_enforce_state` (MCP) para pre-call checks
- Si verify falla: retry ladder:
  1. Retry con feedback procesado
  2. Contexto fresco (~200 tokens resumen)
  3. Estrategia materialmente distinta
  4. ❌ FAILED → escalar a humano
- Evaluator-Optimizer: correctitud, simplicidad, consistencia
- Self-Harness Gate: propose → evaluate → accept
- Pre-commit Gate: Definition of Done + checklists por tipo
- **Pre-commit: skill code-review-and-quality** antes del commit final
- Budget: máx 5 iteraciones por tarea. **Si se agota el budget sin completar → devolvé
  `RESULTADO: 🟡 INCOMPLETO` con el próximo step ⬜ PENDING; NO lo marques FAILED solo por budget.**
  El orquestador decide RESUME/RETRY vía subagent-recovery.md.

**Cierre:**
- Verify full:
  1. `campaign_verify_cmd command="cargo fmt --check"`
  2. `campaign_verify_cmd command="cargo clippy --workspace --all-targets --all-features -- -D warnings"`
  3. `campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"`
  4. `campaign_verify_cmd command="scripts/validate-docs-coverage.ps1"`
- Si todo pasa: `git add <solo los archivos tocados en esta tarea> && git commit -m "feat: <ID> — <name>"` (el commit SIEMPRE está precedido por el verify full de arriba — nunca commitear un cambio sin verificación mecánica)
- **AGENTS.md learnings:** documentá 1-2 aprendizajes de la tarea en una entrada al final de `.opencode/AGENTS.md`:
  ```markdown
  <!-- Learnings: TASK-ID — fecha -->
  - <qué fue más difícil de lo esperado>
  - <qué patrón o técnica funcionó bien>
  ```
- Llamá `campaign_update_task_state` con `"completed"` y recitation
- Auto-mejora: evaluá qué fue más difícil de lo esperado
- Llamá `campaign_diagnose_pipeline` (MCP) para diagnosticar performance y obtener sugerencias de mejora

**Progreso:**
- Ejecutá `skill progreso`

#### ⏳ IN PROGRESS

- Leé la recitation del plan file para saber dónde quedó
- Continuá con el próximo step (PLAN → ACT → VERIFY)
- Si verify falla: retry ladder (mismo que arriba)
- Errores colaterales: rápido (<30min) se arregla, lento se difiere a Backlog
- Budget: 5 iteraciones máximas por tarea, 2 stalls consecutivos → ❌ FAILED.
  Si el presupuesto se agota sin terminar → devolvé `🟡 INCOMPLETO` con próximo step,
  no te cierres en silencio.

**Cuando el último step esté completo + verificado + commiteado:**
- Llamá `campaign_update_task_state` con `"completed"` y recitation
- Ejecutá `skill progreso`

#### ❌ FAILED

- Anotá por qué falló y qué se intentó (los 4 escalones si aplica)
- Llamá `campaign_update_task_state` con `"failed"`
- Ejecutá `skill progreso` para registrar en docs/progreso/
- Detenete. No sigas a la siguiente tarea.

### 3. ACTUALIZAR RECITATION

Después de cada acción, llamá `campaign_update_task_state` con:
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
El agente que te invocó recogerá la próxima iteración.

### 5. EJECUCIÓN MULTI-TAREA

Si el usuario quiere ejecutar MÁS de una tarea, usá `/pipeline run` que invoca
este mismo prompt por cada tarea vía sub-agentes con contexto fresco. No intentes
loope vos mismo.

```
/pipeline run [plan]
```

### 6. REFERENCIA RÁPIDA

| Modo | Comando | Qué hace |
|------|---------|----------|
| Una tarea | `/pipeline task ID` o `/loop-goal "./prompts/pipeline-full.md"` | Este prompt: una tarea completa |
| Todas | `/pipeline run` | Usa sub-agentes, invoca este prompt por tarea |
| Plan | `/pipeline plan backlog.md` | Crea plan desde backlog |
| Interactivo | `/pipeline` | Detecta estado y sugiere próximo paso |

### 7. RESULTADO — contrato de retorno obligatorio

Al final de CADA invocación devolvé ESTE bloque (el orquestador lo parsea para
decidir si la tarea está terminada o requiere reintentar/reanudar):

```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO | ⚠️ SIN-FORMATO
STEPS_OK: <n>/<M> total steps
PROXIMO_STEP: <nombre del próximo step pendiente, o "ninguno">
COMMIT_HASH: <hash o "ninguno">
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | qué impidió terminar>
```

- `✅ COMPLETO`: todos los steps ✅ + `campaign_verify_cmd` del contrato pasa + commit hecho.
- `🟡 INCOMPLETO`: hay trabajo parcial (steps ✅ y ⬜ restantes). Decime el próximo step.
  Ocurre cuando: se agotó el budget, te detuviste a pedir aclaración, o el sub-proceso fue
  interrumpido. **SIEMPRE** actualizá el task file (steps ✅ + Context Save Point) antes de
  devolver INCOMPLETO — es lo que permite reanudar sin perder nada.
- `❌ FALLIDO`: agotaste el retry ladder interno (4 escalones) en VERIFY.
- `⚠️ SIN-FORMATO`: no devolviste el bloque — el orquestador va a re-invocarte pidiéndolo.

**Nunca** devuelvas resultados vacíos, "lista", o silencio. Si no pudiste terminar,
la información del bloque es el handoff para que el siguiente intento continúe.

REGLAS (del campaign-executor RULES.md):
- Usá `campaign_get_next_task` (MCP) o leé el plan file directamente
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
