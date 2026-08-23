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
- **Gate D (question-gates.md):** tras zero-code planning y ANTES de escribir el
  task file — si blast radius >10 archivos/hot path/API pública, contrato ambiguo,
  o feature-add sin spec → `question` al usuario (GO / ajustar / dividir).
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
- **Gate Regla 0 (MUST):** antes de la PRIMERA edición de cualquier archivo,
  el task file debe tener llena la sección **"Impacto mapeado (Regla 0)"**
  (formato en `prompts/task.md`): archivos leídos completos, referencias
  hacia dentro, referencias entrantes y veredicto de impacto. Si el task file
  no la tiene → volver a DISCOVERY y poblarla antes de editar.
- Llamá `campaign_update_task_state` con `"in-progress"` y recitation
- State machine: PLAN → ACT → VERIFY por cada step (~100 líneas por step)
  * Antes de ACT → `campaign_validate_command` (MCP) para validar el comando
  * Si el comando es riesgoso → `campaign_run_sandboxed` (MCP)
  * En cada transición de estado → `campaign_enforce_state` (MCP) para pre-call checks
- Si verify falla: retry ladder:
  1. Retry con feedback procesado
  2. Contexto fresco (~200 tokens resumen)
  **Umbral único (2 fallas mismo-error): Gate V (question-gates.md) → `question`
  al usuario (reintentar fresh / cambiar estrategia / FAILED). Sin respuesta → STOP.**
- Evaluator-Optimizer: correctitud, simplicidad, consistencia
- Self-Harness Gate: propose → evaluate → accept
- Pre-commit Gate: Definition of Done + checklists por tipo
- **FASE SECURITY** (obligatoria cuando el cambio toca trust boundaries):
  * Condición: input de usuario, auth/sesiones, dependencias (nuevas o bump),
    storage/persistencia, FFI (PyO3/WASM/Node), red (server/MCP/HTTP)
  * Skill: `security-and-hardening` — seguí su checklist completa (la skill tiene la suya propia)
  * Gate de verificación: checklist de `security-and-hardening` ✅ +
    `cargo audit` SI hubo cambios de dependencias
- **FASE PERFORMANCE** (obligatoria cuando el cambio toca hot paths):
  * Condición: `vector/` (HNSW, métricas de distancia), `engine.rs`, loops de
    search/ingestión, serialización (ver AGENTS.md Regla 4 y deuda P2)
  * Skill: `performance-optimization`
  * Gate de verificación: comparación contra baseline — medí antes/después
    (bench Criterion o timing simple); sin regresión o regresión documentada
- **Pre-commit: skill code-review-and-quality** antes del commit final
- Budget: `BUDGET_LIMITS` (campaign-server.mjs). **Si se agota el budget sin completar → devolvé
  `RESULTADO: 🟡 INCOMPLETO` con el próximo step ⬜ PENDING; NO lo marques FAILED solo por budget.**
  El orquestador decide RESUME/RETRY vía subagent-recovery.md.

**Cierre:**
- **GATE CITAS (TSYS-13, solo research/evidencia):** si la tarea produce evidencia con URLs citadas
  (campo `fuentes`/URLs del task file o `contract.evidencia` de la recitation):
  * Extraé cada URL citada.
  * Check mecánico: resolvé cada URL con `webfetch`/HEAD (o `argus_extract_content` si aplica).
    URL que NO resuelve (404 / dead / timeout) → evidencia **INVALIDA**: reemplazá la fuente
    o descartá el claim — no la presentes como verificada.
  * SIN RED (runner offline): fallback manual documentado — marcá cada cita como
    `[cita NO VERIFICADA — sin red]` en la evidencia y anotá la verificación pendiente
    en `contract.deuda` de la recitation. Nunca la des por verificada.
- Verify full:
  1. `campaign_verify_cmd command="cargo fmt --check"`
  2. `campaign_verify_cmd command="cargo clippy --workspace --all-targets --all-features -- -D warnings"`
  3. `campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"`
  4. `campaign_verify_cmd command="scripts/validate-docs-coverage.ps1"`
- Si todo pasa: `git add <solo los archivos tocados en esta tarea> && git commit -m "feat: <ID> — <name>"` (el commit SIEMPRE está precedido por el verify full de arriba — nunca commitear un cambio sin verificación mecánica)
- **Learnings (memoria única):** documentá 1-2 aprendizajes vía
  `campaign_memory_write(file="lessons", entry="<tema> | <lección> | ref: <ruta:línea>")`
  — NO editar AGENTS.md manualmente (schema TSYS-15; el server antepone la fecha).
- Llamá `campaign_update_task_state` con `"completed"` y recitation
- Auto-mejora: evaluá qué fue más difícil de lo esperado
- Llamá `campaign_diagnose_pipeline` (MCP) para diagnosticar performance y obtener sugerencias de mejora

**Progreso:**
- Ejecutá `skill progreso`

#### ⏳ IN PROGRESS

- Leé la recitation del plan file para saber dónde quedó
- Continuá con el próximo step (PLAN → ACT → VERIFY)
- Si verify falla: retry ladder (mismo que arriba, con Gate V al agotar el umbral)
- Errores colaterales: **Gate C (question-gates.md)** — `question` al usuario:
  arreglar ahora (<30min) / mandar a Backlog / incluir en commit.
  Si `git status` muestra archivos fuera del blast radius declarado → confirmar
  alcance del commit antes de `git add`.
- Budget: límites en `BUDGET_LIMITS` (campaign-server.mjs). 2 stalls consecutivos → ❌ FAILED.
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
- `recitation` — **estructura canónica única** (fuente única de verdad: §12.3 —
  plantilla `RESULTADO` — de `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md`,
  SOLO LECTURA). Los campos reales del MCP son 6 (schema campaign-server.mjs); la
  estructura §12 se embeberá DENTRO de `contract` y `result`:
  - `activeGoal`: echo del objetivo (≈ §12 `objective`)
  - `lastAction`: qué se hizo en esta iteración (≈ §12 `resumen`, máx ~200 tokens)
  - `result`: `OK` | `PARTIAL` | `FAILED` — el §12 `status`; estado real, nunca fabricado
  - `nextAction`: próximo paso concreto (archivo + comando)
  - `contract`: CONTRATO §12 (texto — incluye lo que gap-01 §3.3-18 llamaba `invariants`/`debt`):
    - `verificacion`: comando de verificación EXACTO + resultado obtenido (p.ej. `cargo nextest run --profile audit --workspace --build-jobs 2` ✅)
    - `evidencia` (obligatoria por claim):
      - `claim`: <afirmación concreta>
        `evidencia`: <URL | file path | tool result>
        `confianza`: alta | media | baja
    - `artefactos`: <paths persistidos en filesystem> — outputs grandes NO en el mensaje
    - `invariantes`: qué NO se puede romper al continuar (dominio/seguridad; del task file) — si nada, "ninguna"
    - `deuda`: deuda pendiente / lo que queda incompleto al cerrar esta iteración — si nada, "ninguna"
    - `queda_pendiente`: <pendiente_adicional §12 — qué debe delegar/validar el orquestador>
  - `nextTask`: ID de la próxima tarea a ejecutar si completa

> La recitation debe dejar al próximo agente en capacidad de continuar SIN
> preguntar al anterior: invariantes, verificación y deuda (eng-03-project.md:198).
> El orquestador valida `result` + evidencia por claim (§12.3); si el bloque no es
> parseable → `⚠️ SIN-FORMATO` (ver § 7). El server persiste SOLO las 6 claves MCP —
> claves top-level como `invariants`/`debt` no existen en el schema
> (campaign-server.mjs:673-680), van dentro de `contract`.

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
- Mapeo con la recitation canónica (§ 3 / §12): `✅ COMPLETO` ↔ `result: OK`,
  `🟡 INCOMPLETO` ↔ `result: PARTIAL`, `❌ FALLIDO` ↔ `result: FAILED`,
  `⚠️ SIN-FORMATO` = bloque no parseable (ningún status §12 válido).

**Nunca** devuelvas resultados vacíos, "lista", o silencio. Si no pudiste terminar,
la información del bloque es el handoff para que el siguiente intento continúe.

REGLAS (del campaign-executor RULES.md):
- Usá `campaign_get_next_task` (MCP) o leé el plan file directamente
- El contrato es ley — si no se cumple, la tarea no está completa
- Verificación mecánica, nunca auto-reporte
- Ponytail ladder: existe > stdlib > dependency > mínimo código
- ~100 líneas por step, un step por turno, cada step reversible
- No cambies scope. Rápido se arregla, lento se anota en Backlog
- Stagnation = stop: 3 vueltas sin progreso → ❌ FAILED
- Budget: límites en `BUDGET_LIMITS` (campaign-server.mjs), 2 stalls consecutivos → FAILED
- 2 fallas de verify con mismo error → Gate V (`question-gates.md`): preguntar al usuario antes de FAILED
- La recitation es el handoff entre iteraciones
- Después de completar una tarea, DETENETE. No sigas a la siguiente.
