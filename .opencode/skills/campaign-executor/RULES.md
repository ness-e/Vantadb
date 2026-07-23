# Campaign Executor — North Star + Reglas Invariantes

> **Este archivo no cambia.** Todo el pipeline referencia esta visión como anchor.
> Si una iteración se desvía, vuelve acá. No se edita durante ejecución.

---

## VISIÓN: North Star del Campaign Executor

### Propósito

Automatizar la ejecución de campañas de tareas desde backlog con **calidad
consistente** y **cero supervision overhead**: que un dev pueda dejar N tareas
encargadas y volver a encontrar todo hecho, verificado y comiteado.

### Criterios de éxito

| Dimensión | Target |
|-----------|--------|
| **Tasa de completado** | >90% de tareas en 1er intento |
| **Falsos positivos** | 0 — no marcar complete si algo falla |
| **Regresión silenciosa** | 0 — no romper tests que antes pasaban |
| **Deuda técnica** | no introducir más de la que se resuelve |
| **Tiempo dev** | 100% del foco en código, 0% en coordinar el loop |

### Principios invariantes

1. **El contrato es ley** — cada tarea tiene una condición booleana verificable.
   Si el contrato no se cumple, la tarea no está completa. Punto.

2. **Primero entender, después tocar** — Fase Discovery no es opcional.
   codegraph_explore antes de la primera línea de código.

3. **Verificación mecánica, nunca auto-reporte** — el compilador, el test runner
   y el linter son los únicos que pueden decir "pasa". No confiar en resúmenes
   escritos por el agente.

4. **Un paso a la vez** — ~100 líneas por commit. Si un cambio es más grande,
   dividirlo. Cada paso debe poder revertirse individualmente.

5. **Ponytail: el mínimo que funciona** — subir la escalera antes de cada
   bloque de código: ya existe > stdlib > platform > dependency > una línea > mínimo.

6. **Errores colaterales se atrapan, no se ignoran** — si durante una tarea
   encontrás otro bug: rápido se arregla (<30min), lento se difiere a Backlog.
   Nunca se deja pasar sin registro.

7. **Progreso visible siempre** — después de cada paso, plan file actualizado
   + recitation. El harness nunca debe estar más de 3-5 iteraciones sin reportar
   progreso.

8. **Stagnation = stop** — si el loop da 3 vueltas sin avanzar (mismo error,
   mismo archivo, mismo contrato insatisfecho), se detiene y pide ayuda. No
   seguir dando vueltas.

9. **Presupuesto finito** — cada ejecución tiene un tope de iteraciones por
   tarea (default 5), stall consecutivo (default 2). Pasado el tope, FAILED.

10. **Auto-mejora** — después de cada tarea, evaluar: ¿qué fue más difícil de
    lo esperado? ¿el proceso mejoró o empeoró? Actualizar discoverys en el
    proceso.

### Árbol de decisión (antes de empezar)

```
¿Querés ejecutar tareas desde un backlog?
  ├─ Sí → /pipeline plan docs/Backlog.md
  │       (crea plan file + muestra próximo paso)
  │
  └─ No → ¿Querés definir una tarea a profundidad?
       ├─ Sí → /pipeline task DRV-NN
       │       (investiga, crea task file con steps atómicos)
       │
       └─ No → ¿Querés ejecutar un plan existente?
├─ Completo → .opencode\task-system\harness\harness-executor.ps1 -PlanFile ...
├─ Una tarea → .opencode\task-system\harness\harness-executor.ps1 -PlanFile ... -SingleTask DRV-NN
└─ En paralelo → .opencode\task-system\harness\harness-executor.ps1 -PlanFile ... -Parallel
```

### Relación con archivos

```
RULES.md / VISION.md          ← north star (este archivo, no se modifica)
.opencode/task-system/prompts/plan.md               ← crear plan desde backlog (triage gate)
.opencode/task-system/prompts/task.md               ← definir tarea a profundidad
.opencode/task-system/prompts/iter-loop-tools.md               ← ejecutar una iteración del harness
.opencode/commands/pipeline.md                      ← entry point: plan | task | run | interactive
.opencode/task-system/harness/harness-executor.ps1   ← loop externo PowerShell
SKILL.md                      ← referencia completa del skill
tasks/<ID>.md               ← auto-generated task definitions (resuelve a .opencode/skills/campaign-executor/tasks/<ID>.md)
.opencode/references/           ← repos clonados (awesome-harness-engineering, statewright, ...)
```

---

## Reglas Invariantes (operativas)

### 1. Un paso por turno

OpenCode opera por turnos (Request-Response). Cada invocación ejecuta
EXACTAMENTE UNA acción atómica. El harness itera por vos.

### 2. Estado en archivos, no en contexto

El contexto se resetea en cada invocación. El plan file y el task file
son la única fuente de verdad. Siempre leer antes de actuar, siempre
escribir después.

### 3. La recitation es el handoff

Después de cada acción, escribir el bloque RECITATION al final del plan
file. Es lo único que persiste entre iteraciones. Sin recitation, la
próxima iteración arranca perdida.

### 4. Verificación mecánica siempre

Nunca auto-reportar "anda". Siempre ejecutar un comando real:
- `cargo check -p vantadb`
- `cargo nextest run`
- `npx tsc --noEmit`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`

### Rust Safety Rules (motor DB)

- `unsafe` prohibido por defecto. Si es indispensable: `// SAFETY:` invariant documentado explicando por qué es seguro.
- `Rc<T>` prohibido en contextos multi-hilo. Siempre `Arc<T>`.
- Sin `#[allow(unsafe_code)]` sin aprobación explícita en code review.
- Sin `unwrap()` en código de producción que toque datos de usuario o E/S — usar `?` o `expect("contexto del error")`.

### Capa Determinista (barreras infranqueables)

Estas verificaciones NO se saltan bajo ninguna circunstancia:

0. **Output Validation (LLM05)**: Antes de escribir cualquier archivo que contenga shell commands, SQL, Python code, HTML o file paths, validar con `campaign_validate_output` MCP tool. Output del agente NO es confiable — sanitizar antes de write.
1. `cargo clippy --all-targets -- -D warnings` — cero advertencias
2. `cargo fmt --check` — formato correcto
3. `cargo nextest run --profile audit --workspace --build-jobs 2` — tests pasan
4. Si el diff contiene `unsafe` → `cargo +nightly miri test` (detección de UB)
5. Si el componente es crítico (parser, serializador, WAL, protocolo de red) → marcar para fuzzing + quickcheck/proptest en CI

### 6. Versioned DoD Thresholds (ratchet — solo sube, nunca baja)

Current: **DoD v1** (baseline)
Next: bump `NEXT_DOD_VERSION` en este archivo cuando se cumplan todas las condiciones de la versión actual.

| Versión | Nuevos checks (suman a los anteriores) |
|---------|----------------------------------------|
| v1 (baseline) | Capa determinista (0-5) + Pre-commit gate (7 items) |
| v2 (coverage) | `cargo nextest run --coverage` mínimo 70% en módulos nuevos. Security checklist obligatorio (no condicional). |
| v3 (hardening) | Fuzzing obligatorio en todo parser/serializer. `cargo audit` sin warnings. Miri en todo `unsafe`. |
| v4 (enterprise) | `cargo deny check` sin advisories. 90% coverage mín. Review externo obligatorio antes de merge. |

Regla: **No se puede saltar una versión.** Si NEXT_DOD_VERSION = v2, todos los checks de v1 + v2 aplican. Para pasar a v3, v2 debe estar estable por 5 tareas consecutivas.

### 5. Ponytail ladder

1. ¿Ya existe en el codebase? → reusar
2. ¿Stdlib lo hace? → stdlib
3. ¿Feature nativa del platform? → usarla
4. ¿Dependency ya instalada? → usarla
5. ¿Una línea? → una línea
6. Recién acá: código mínimo que funciona

### 6. Atomicidad

Cada cambio: ~100 líneas máximo. Un paso del task file = una acción =
un commit. Si el cambio es más grande, partilo en más steps.

### 7. No cambiar scope

Si encontrás algo extra (bug no relacionado, feature faltante) durante
la ejecución: anotalo en Notas, no lo implementes. Seguí con la tarea
actual.

### 8. Sync bidireccional plan ↔ task

- Plan file y task file se referencian mutuamente
- Ambos tienen `last-synced: <fecha>`
- Después de cada acción, actualizar ambos
- El harness valida sync antes de cada iteración

### 9. Stagnation detection

- 2 intentos consecutivos con el mismo error (archivo+línea+mensaje) → ❌ FAILED
- 3 intentos sin cambiar de step → ❌ FAILED
- El harness detecta stall y pregunta al usuario

### 10. Skills según tipo de tarea

| Tipo | Skills a cargar |
|------|-----------------|
| Rust | source-driven-development, campaign-executor |
| Frontend | frontend-ui-engineering |
| API pública | api-and-interface-design |
| Bug | systematic-debugging |
| Review | code-review-and-quality, doubt-driven-development |
| Docs | writing-guidelines |
| Siempre | campaign-executor, progreso, ponytail (full) |

## Apéndice A: HarnessCard (CAR Decomposition)

| Capa | Dimensión | Implementación |
|------|-----------|----------------|
| **Control** | State machine | C0 en iter-loop-tools.md: 10 estados, guards, per-state tool enforcement |
| | Budgets | 15 tool calls, 40 sub-agents, 5 fails, 120min por tarea |
| | DoD | 4 versiones ratcheted (v1 baseline → v4 enterprise) |
| | Capa determinista | 6 barreras infranqueables (clippy, fmt, tests, miri, fuzz, output validation) |
| | MoM ladder | 4 tiers (haiku → sonnet/gpt-4o → deepseek-v4 → humano) |
| | Pre-commit gate | 7 checks: DoD, security, perf, testing, ponytail, tests, docs |
| | Stagnation detection | 3 mismo error, 5 sin cambiar step → FAILED |
| **Agency** | Step ordering | Task file con steps atómicos, zero-code planning antes de código |
| | File edits | ~100 líneas/commit, edit con oldString/newString |
| | Verify strategy | Mecánico (cargo, npx), Agente de Diagnóstico en falla |
| | Sub-agent spawning | `task` tool para research isolation, fork/join paralelo |
| | Self-Harness Gate | Propose → Evaluate (5 condiciones) → Accept/Reject |
| **Runtime** | Execution | MCP server (campaign-* tools), cargo-mcp, rust-analyzer-mcp |
| | Sub-agents | `task` tool, research isolation pattern, fork/join groups |
| | Sandbox | `campaign_run_sandboxed` vía PowerShell aislado |
| | Memory | `memory/lessons.md`, `memory/decisions.md` + `campaign_memory_read/write` |
| | Tracing | JSONL events a `traces/<campaign-id>.jsonl` via tracer.mjs |
| | Plan files | `docs/plans/<plan>.md` + `docs/plans/<plan>.budget.json` |

### Rule 11 — Session lifecycle

Una tarea completa + commiteada → sesión cerrada mentalmente.
La siguiente tarea arranca con contexto fresco. No arrastres estado entre tareas.
Si necesitás continuar algo, dejalo en la recitation o en Context Save Point.

### Rule 12 — Bounded Memory (Context Budget)

Fuente: awesome-harness-engineering (OpenHands, Anthropic context engineering)

El contexto no es un dumping ground. Solo se preserva entre iteraciones:

| Qué se guarda | Qué se descarta |
|---------------|-----------------|
| Goals activos | Tool outputs cerrados |
| Progreso del step actual | Logs de builds pasados |
| Archivos críticos modificados | Diffs completos (git los tiene) |
| Tests que fallan | Mensajes de éxito |
| Recitation block | Conversación previa |

Reglas:
1. Si el contexto crece >20% del límite del modelo, usar sub-agentes vía task tool para aislar investigación
2. Antes de cada iteración, evaluar: "¿esto es necesario para el próximo paso?"
3. Useful failures se mantienen en contexto (lo que salió mal es diagnóstico, no ruido)
4. Backpressure on low-value work: si una tarea genera ruido sin progreso >3 iteraciones, abortar

### Rule 13 — 12-Factor Agents (Production Discipline)

Fuente: awesome-harness-engineering (HumanLayer: 12-Factor Agents)

| Factor | Aplicación en Campaign Executor |
|--------|---------------------------------|
| 1. Explicit prompts | Cada tarea tiene un prompt completo (plan.md, task.md, iter-loop-tools.md) |
| 2. State ownership | Plan file + task file son la única fuente de verdad. No confiar en contexto de sesión. |
| 3. Clean pause-resume | Recitation block permite retomar exactamente donde se quedó |
| 4. Logs as event streams | harness.ps1 emite eventos JSONL a traces/<campaign-id>.jsonl |
| 5. Disposable agents | Cada iteración arranca contexto fresco. Sin estado en memoria del agente. |
| 6. Backward compatibility | Nunca romper el formato de plan file — otros scripts lo leen |
| 7. Fail fast, fail visibly | Stagnation detection + MoM ladder. No reintentar con el mismo modelo. |
| 8. Runtime verification | campaign_verify_cmd — nunca auto-reporte |
| 9. Bounded resources | Budget: 15 tool calls, 40 sub-agents, 5 fails, 120min por tarea |
| 10. Observable | JSONL logs + correlation ID + structured recitation |

### Rule 14 — Correlation ID Tracing

Fuente: REFERENCE-SYNTHESIS.md (prioridad #1: alta)

Cada campaña genera un UUID al inicio (CampaignId). Este ID se propaga a:

| Destino | Dónde se escribe |
|---------|------------------|
| Plan file | `> **Campaign ID:** <uuid>` en el header |
| JSONL log | Cada línea: `{"event":"...", "campaign_id":"<uuid>", ...}` |
| Recitation | Bloque RECITATION en cada iteración |
| Git commits | `Campaign: <uuid>` en el footer del commit message |
| Task files | `campaign_id: <uuid>` en el header |

El correlation ID permite conectar:
- Una iteración de harness → su log JSONL → el commit → el task file
- Sin correlation ID, cada componente es un silo aislado

### Rule 15 — init.sh Pattern (Bootstrap Harness)

Fuente: Anthropic — "Effective harnesses for long-running agents"

Cada tarea importante debería tener un `init.sh` (o `init.ps1` en Windows) que:

1. Verifica el entorno (herramientas instaladas, versiones, variables de entorno)
2. Limpia estado residual de ejecuciones anteriores
3. Prepara directorios temporales si es necesario
4. Establece el correlation ID de la ejecución
5. Imprime un resumen del estado inicial

```powershell
# Ejemplo de init.ps1 para tarea de campaña
param([string]$CampaignId)
Write-Host "=== init.ps1 — Campaign $CampaignId ==="
# 1. Verificar herramientas
@("cargo", "git", "python") | ForEach-Object {
    if (-not (Get-Command $_ -ErrorAction SilentlyContinue)) {
        Write-Error "Missing: $_"; exit 1
    }
}
# 2. Limpiar estado residual
Remove-Item -Path ".campaign/temp/*" -Recurse -ErrorAction SilentlyContinue
# 3. Correlation ID
$env:CAMPAIGN_ID = $CampaignId
Write-Host "Ready: Campaign $CampaignId"
```

Reglas:
- `init.sh`/`init.ps1` se ejecuta UNA vez al inicio de la campaña, no por iteración
- Si falla, la campaña no arranca — error es mejor que ejecución en entorno roto
- Debe ser idempotente (ejecutar dos veces no rompe nada)

### Rule 16 — While-Loop Harness (Ralph Pattern)

Fuente: Geoffrey Huntley — "Ralph Wiggum as a Software Engineer"

El patrón más simple de harness que funciona:

```bash
while :; do cat PROMPT.md | agent; done
```

En nuestro contexto (OpenCode + campaign system):

| Elemento | Cómo se aplica |
|----------|---------------|
| **PROMPT.md** | `iter-loop-tools.md` — el prompt de iteración |
| **agent** | OpenCode con los MCP tools de campaign |
| **loop externo** | `harness-executor.ps1` — iteración PowerShell |
| **determinismo** | Cada iteración arranca contexto fresco (Disposable Agents) |
| **handoff** | Recitation block en el plan file |

La versión VantaDB del while-loop:

```powershell
# harness-executor.ps1 -IterationCount $count
while ($true) {
    $task = Get-NextTask
    if (-not $task) { break }
    Invoke-OpenCode -Prompt "iter-loop-tools.md" -Task $task
    if (-not (Test-Contract $task)) { break }
}
```

Este patrón es la base conceptual de todo el campaign executor — no agregar complejidad innecesaria al loop.

### Rule 17 — Lurkr Scanner (CI Gate for Agent Risks)

Fuente: [agentveil-protocol/lurkr](https://github.com/agentveil-protocol/lurkr)

El Lurkr scanner se ejecuta en CI y detecta riesgos de agentes de IA:

| Riesgo | Detecta |
|--------|---------|
| Shadow capabilities | Código no explicitado en el prompt del agente |
| Credentials en contexto LLM | API keys, tokens, secrets en archivos que el agente leería |
| eval/subprocess en @tool | Código que ejecuta comandos sin sanitización |
| Prompt interpolation directa | Strings armadas con ${} en prompts |
| MCP endpoints no verificados | Llamadas a MCP URLs sin validación |

**Gate de CI** (agregar al workflow de CI):
```yaml
- name: Lurkr scanner
  run: npx lurkr scan .opencode/ --report-format json
```

Por ahora: no bloqueante, solo informativo. Cuando el ecosistema madure, el Lurkr gate será obligatorio antes de deploy.

### Rule 18 — Context Condensation (OpenHands Pattern)

Fuente: OpenHands — "Context Condensation for More Efficient AI Agents"

Entre iteraciones del harness, condensar el contexto preservando solo:

| Preservar | Descartar |
|-----------|----------|
| Goals activos | Logs de builds pasados |
| Archivos modificados en este step | Output de tool calls previas |
| Tests que fallan actualmente | Mensajes de éxito |
| Recitation block | Conversación completa previa |
| Correlation ID | Errores ya resueltos |

En la práctica:
1. El harness escribe el recitation block al final de cada iteración
2. La próxima iteración LEE el plan file + recitation — eso ES la condensación
3. No necesita lógica extra: el recitation block es nuestro mecanismo de condensación
4. Si una tarea excede 15 tool calls sin progreso, fork a sub-agente vía `task` tool para aislar investigación

Límite práctico: si el plan file supera 200 líneas, hay que archivarlo y empezar uno nuevo (clean state).

### Rule 19 — AgentKit Patterns (Event-Driven Durable Agents)

Fuente: Inngest AgentKit

Para tareas que requieren durabilidad (continuar después de crash):

| Patrón | Descripción |
|--------|-------------|
| **Workflow-aware** | Cada paso registra estado en JSON durable (`.campaign/budget.json`) |
| **Event-driven** | Las transiciones del state machine C0 son eventos |
| **Idempotency** | Cada paso puede re-ejecutarse sin side effects |
| **Retry with backoff** | Si un paso falla, esperar 2^retry segundos antes de reintentar |

No implementar como dependency — implementar como convention en el harness existente.
- Budget tracking via JSON es el estado durable
- C0 state machine en iter-loop-tools.md maneja transiciones
- Retry con backoff: `Start-Sleep -Seconds [Math]::Pow(2, $retryCount)`
