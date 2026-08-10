> **ACTIVE INSTRUCTION — Sub-Agent Recovery Protocol (SARL)**
> Cargado por el ORQUESTADOR (`/pipeline run`, `/pipeline task`, vanta-lead) cuando un
> sub-agente devuelve un resultado incompleto, fallido, detenido, vacío o inesperado.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Objetivo: **NO perder trabajo ya hecho**. Reanudar/reintentar hasta agotar la escalera,
> reutilizando la MISMA sesión del sub-agente cuando sea posible.
> Referencia canónica del ciclo de vida de una tarea: `pipeline-full.md` (DISCOVERY → EJECUCIÓN → CIERRE).

## 1. Clasificar el resultado del sub-agente

Cada sub-agente DEBE devolver el bloque `RESULTADO:` estructurado (ver `pipeline-full.md` § Resultado).
Si no lo devuelve, clasificá por evidencia observable (salida, git log, task file, worktree):

| Evidencia observable | Clasificación | Significado |
|---|---|---|
| `RESULTADO: ✅ COMPLETO` + commit + contrato pasa | **DONE** | Terminar: actualizar plan, progreso, siguiente tarea |
| `🟡 INCOMPLETO` (steps OK < total, sin commit) | **INCOMPLETE** | Trabajo parcial existe; continuar desde el próximo step |
| Salida vacía / se detuvo solo / "necesito X" / respuesta no trivial | **UNEXPECTED** | Re-invocar pidiendo resultado estructurado + feedback |
| `❌ FALLIDO` (verify falla tras su retry ladder interno) | **FAILED** | Cambiar estrategia o escalar |
| `⚠️ SIN-FORMATO` (sin bloque RESULTADO parseable) | **UNEXPECTED** | Idem UNEXPECTED |

**Regla de oro:** nunca tratés un INCOMPLETE/UNEXPECTED como FAILED. Casi siempre se
recupera reanudando la MISMA sesión, y el trabajo del worktree + task file ya existe.
El FAILED real es solo cuando la propia tarea reportó agotamiento de su retry ladder.

## 2. Escalera de recuperación (una vez por resultado no-DONE)

```
Nivel 1  RESUME — misma sesión del sub-agente
         task(task_id=<T>, subagent_type=<mismo>, prompt="<feedback procesado>")
         La sesión del sub-agente conserva su contexto y su worktree persiste.
         Indicá: qué se hizo (del Context Save Point / git log), el PRÓXIMO step ⬜ PENDING
         del task file, y que devuelva el bloque RESULTADO al final.
         NO re-ejecutés steps ya ✅ — continuá desde el primer step ⬜ PENDING.

Nivel 2  RETRY — sub-agente fresco del mismo tipo
         task(description, subagent_type=<mismo>,
              prompt="Digest ~200 tokens de lo aprendido + path del task file + feedback procesado")
         Se parte del estado durable (task file + worktree), no desde cero.

Nivel 3  STRATEGY — enfoque materialmente distinto
         Otro ángulo de solución (puede escalar de modelo con campaign_mom_escalate).
         Si la tarea es ambigua también podés forkear a un sub-agente de research.

Nivel 4  ESCALATE — a humano
         Documentar intentos (los 4 niveles), commit WIP si hay cambios, 
         campaign_update_task_state "failed", aplicar FAIL_MODE (stop/skip).
         Si FAIL_MODE=stop → detener el pipeline.
```

## 3. Reglas del protocolo

1. **INCOMPLETE/UNEXPECTED → 1 RESUME; si no completa → RETRY.** Rara vez llega a ESCALATE.
2. **FAILED → RETRY (1) → STRATEGY → ESCALATE.** Dos fallas de verify con el mismo error
   (archivo+línea+mensaje) son FAILED real; no consumir RESUME en eso.
3. **Preservación total del trabajo:**
   - El estado durable vive en el **task file** (steps ✅/⬜, Context Save Point) + **plan file** (vía MCP `campaign_update_task_state`).
   - Antes de cada intento, verificá si el sub-agente escribió el Context Save Point. Si no,
     mandá a RESUME a reconstruir dónde quedó con `git diff`/`git log` + lectura del task file.
   - Nunca `git checkout`/`reset --hard` sobre el trabajo del sub-agente. Solo commits atómicos de la tarea.
4. **Verify mecánico obligatorio entre intentos:** antes de aceptar un RESUME/RETRY como DONE,
   corré `campaign_verify_cmd` con el contrato del task file. Sin verify no cuenta como completado.
5. **Budgets:** cada recovery consume `campaign_budget_consume resource="fail"`. Si la tarea agota
   su presupuesto → ESCALATE sin más intentos. Max sub-agentes totales: ver SKILL.md (HARD STOP).
6. **Contra-stall:** si 2 recuperaciones consecutivas del mismo nivel no avanzan → subí de nivel.
   3 resultados no-DONE seguidos en la misma tarea → pausá y preguntá al usuario (NO_PROGRESS_LIMIT).
7. **Registro final:** SIEMPRE `campaign_update_task_state` + recitation indicando el nivel de la
   escalera alcanzado y la razón del resultado final.

## 4. Qué pedirle SIEMPRE al sub-agente (contrato de retorno)

Al final de cada invocación, el sub-agente debe devolver:

```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO | ⚠️ SIN-FORMATO
STEPS_OK: <n>/<M> total steps
PROXIMO_STEP: <nombre del próximo step pendiente, o "ninguno">
COMMIT_HASH: <hash o "ninguno">
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | qué impidió terminar>
```

Con este bloque el orquestador decide el nivel de recovery en 1 solo paso, sin adivinar.
Si un sub-agente "se detiene solo" sin devolver el bloque → tratarlo como UNEXPECTED,
RESUME pidiendo el bloque + feedback.