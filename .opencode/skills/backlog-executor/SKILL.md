---
name: backlog-executor
description: >
  Task execution harness for backlog-driven development.
  Harness-driven (no loop en prompt): externo ejecuta una iteración por turno.
  Cada iteración: triage → eval gate → implement → verify → commit → update.
  Incluye context preservation, stall detection, budget management, recitation.
compatibility: opencode
---

# Backlog Executor — Harness-Driven Task Execution

> **Arquitectura:** Harness externo (PowerShell script o opencode-loop plugin) maneja
> el while loop. El agente ejecuta EXACTAMENTE UNA acción atómica por turno.
>
> Esto no es opcional: OpenCode opera por turnos (Request-Response). Un LLM no puede
> mantener un loop nativo. Todo framework converge en el mismo while — la diferencia
> está en el harness alrededor del loop. (Steve Kinney, "Anatomy of an Agent Loop")
>
> Ver referencias en sección 10.

---

## Core Loop Architecture (Vista General)

El loop completo:

```
  HARNESS (externo)                         AGENTE (por turno)
  ┌────────────────────┐                    ┌──────────────────┐
  │ while (tasks > 0)  │──── llama ────────▶│ Leer plan file   │
  │                    │                    │ Hacer 1 acción   │
  │ 1. task = next()   │◀─── devuelve ──────│ Escribir estado  │
  │ 2. invoke opencode │                    │ Yield (stop)     │
  │ 3. leer resultado  │                    └──────────────────┘
  │ 4. detectar stall  │
  │ 5. loop o break    │
  └────────────────────┘
```

Cada invocación del agente ejecuta **una iteración del loop interno**:

```
       ┌──────────────────────────────────────┐
       │  PLAN: leer plan file, blast radius  │
       └─────────────┬────────────────────────┘
                     ▼
       ┌──────────────────────────────────────┐
       │  ACT: ~100 líneas de código máximo   │
       └─────────────┬────────────────────────┘
                     ▼
       ┌──────────────────────────────────────┐
       │  VERIFY: comando mecánico real       │
       │  (cargo check, nextest, tsc, etc.)   │
       └─────────────┬────────────────────────┘
                     │
            ┌────────┴────────┐
            ▼                 ▼
          PASA              FALLA
            │                 │
            ▼                 ▼
       ┌─────────┐    ┌──────────────┐
       │ COMMIT  │    │ RETRY LADDER │
       │ update  │    │ 1-4 escalones│
       └─────────┘    └──────────────┘
            │                 │
            ▼                 ▼
       Yield al harness → SIGUIENTE ITERACIÓN
```

---

## Cómo se usa (3 formas)

### Forma 1: PowerShell Harness (recomendada, sin dependencias)

```powershell
.\harness-executor.ps1 -PlanFile docs\plans\2026-07-13-campaign.md
```

El script:
1. Lee el plan file, encuentra próxima tarea ❌
2. Invoca `opencode run` con el prompt de una iteración
3. Espera a que termine
4. Lee el plan file actualizado
5. Detecta stalls (misma tarea sin progreso 2 veces)
6. Repite o termina

### Forma 2: opencode-loop plugin

Requiere: `oc plugin install opencode-loop`

```bash
# Goal Mode: el plugin verifica avance real
/loop-goal "implementar todas las tareas ❌ en docs/plans/campaign.md"
```

El plugin monitorea idle events y re-inyecta el prompt cuando el agente termina.

### Forma 3: Manual (una iteración a la vez)

Invocar el Prompt 1 manualmente, una vez por tarea/iteración.

---

## 1. Contrato por Tarea

Cada tarea necesita un **contrato observable** escrito en el plan file ANTES de
empezar: una definición de "completado" que código pueda verificar mecánicamente.

| ❌ Contrato vago | ✅ Contrato verificable |
|-----------------|------------------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, `cargo machete` 0 warnings, `cargo nextest run --profile audit` pasa" |
| "Mejorar la web" | "npm run typecheck 0 errors, npm run lint 0 errors, vitest run --pass" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |

---

## 2. Task Triage & Evaluation Gate

Antes de implementar, cada tarea pasa por este gate. El resultado se registra
en el plan file.

```
Tarea: [ID] — [Nombre]
Archivos: [paths]

[ ] 1. ¿RELEVANCIA — Sigue siendo necesaria?
       codegraph_explore "código a modificar"
       → Si el bug ya no existe → ❌ SKIP

[ ] 2. ¿IMPACTO REAL?
       → Data integrity, seguridad, CI, usuarios?
       → Cosmético sin queja → 🟡 DEFER

[ ] 3. ¿COSTO/BENEFICIO?
       → Si impacto << esfuerzo → 🟡 DEFER o ❌ SKIP

[ ] 4. ¿DEPENDENCIAS?
       → ¿Tareas bloqueantes sin completar?
       → Si depende de algo no hecho → 🔴 BLOQUEADO

[ ] 5. ¿RIESGO?
       → ¿Migración? ¿API pública? ¿Serialización?
       → Alto riesgo → cargar `doubt-driven-development`

[ ] 6. ¿SCOPE?
       → Si mezcla código + infra + CI + docs, separar

Resultado: ✅ DO | 🟡 DEFER | ❌ SKIP | 🔴 BLOQUEADO
```

---

## 3. Campaign Plan File Format

El plan file en `docs/plans/YYYY-MM-DD-<campaign>.md` es la **fuente de verdad
del estado**. El harness lo lee antes de cada invocación y el agente lo escribe
después de cada acción.

```markdown
# Plan de Ejecución: [Nombre]

> **Inicio:** YYYY-MM-DDTHH:MM
> **Estado:** ⏳ EN PROGRESO | ✅ COMPLETADO | ❌ ABORTADO
> **Harness PID:** [PID del harness ejecutando]

## Tasks

### Task 1: [ID] — [Nombre]

- **Fuente:** Backlog.md línea N
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🔴 2-3d
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** `path/to/file.rs`
- **Gate Result:** ✅ DO | 🟡 DEFER | ❌ SKIP | 🔴 BLOQUEADO
- **Contrato:** "qué significa 'completado'"
- **Estado:** ⬜ PENDING | ⏳ IN PROGRESS | ✅ COMPLETED | ❌ FAILED
- **Branch:** `fix/code-xxx`
- **Commit:** `abc1234`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Implementar | cargo check ✅ | codegraph |
  | 2 | — | — | — |

  **Notas:**
  - Contexto aprendido, decisiones, problemas

  **Check post-CI:**
  - [ ] GitHub workflow pasa
  - [ ] `skill progreso` ejecutado
```

### Recitation Block

Se escribe al final del plan file después de CADA acción del agente:

```
=== RECITATION ===
Objetivo activo: Task N — [ID]
Estado: [implementado / verificando / CI]
Última acción: [qué se hizo]
Resultado: [✅ pasa / ❌ falla]
Próxima acción concreta: [el siguiente comando o paso]
Contrato: "just verify pasa y CI workflow es green"
Próxima tarea si completa: Task N+1 — [ID]
=== END RECITATION ===
```

**Regla:** el harness parsea la recitation para decidir qué hacer después.
Si la recitation no existe o está corrupta, el harness aborta (no reintenta).

---

## 4. Execution Loop — UNA Iteración por Invocación

**IMPORTANTE:** OpenCode es reactivo por turnos. NO intentes procesar más de
una iteración o tarea por invocación. Cada mensaje del agente ejecuta
exactamente UN paso y devuelve el control.

### Preparación del Harness (se ejecuta una vez al arrancar)

```powershell
# harness-executor.ps1 hace esto:
# 1. Validar que el plan file existe y tiene tasks
# 2. Verificar que no hay cambios sin commit en el repo
# 3. Identificar el task runner (cargo, npm, just, etc.)
# 4. Arrancar el while loop
```

### Fases por Iteración de Agente

Cada invocación del agente ejecuta estas fases EN ORDEN y se detiene:

#### Fase 0: Skills

| Fase | Skills a cargar |
|------|----------------|
| Antes de implementar | `writing-plans`, `incremental-implementation`, `ponytail` (full) |
| Si es bug | `systematic-debugging` |
| Si es API pública | `api-and-interface-design` |
| Si es frontend | `frontend-ui-engineering` |
| Para verificar | `code-review-and-quality` |
| Al completar | `git-workflow-and-versioning` |

**Siempre:** `skill progreso` al inicio de cada nueva tarea.

#### Fase 1: Leer Plan File + Recitation

```
1. Leer docs/plans/<campaign>.md COMPLETO
2. Buscar:
   a. Recitation block → saber dónde se quedó
   b. Si no hay recitation: buscar primer ⬜ PENDING o ⏳ IN PROGRESS
3. Si la tarea actual está ⏳ → retomar desde Fase 3
4. Si la tarea actual está ⬜ PENDING → ejecutar Fase 2
```

#### Fase 2: Blast Radius + Contexto

```
skill systematic-debugging     # si es bug
codegraph_explore "query"

Identificar:
- Callers: qué módulos usan estos archivos
- Callees: dependencias de estos archivos
- Blast radius: ¿API pública? ¿serialización? ¿migración?
```

#### Fase 3: Implementar (Plan → Act → Verify)

```
PLAN:
  - Decidir el cambio atómico (~100 líneas)
  - Ponytail ladder: stdlib > reusar > dependency > desde cero

ACT:
  - Editar el/los archivos

VERIFY:
  - Mecánico, no auto-reporte
  - Rust:  cargo check -p <crate> (rápido)
  - Web:   npm run typecheck
  - Tests: cargo nextest run <test_name> (si aplica)

Stall Detection:
  Si el mismo error (archivo+línea+mensaje) aparece 2 veces → escalar al escalón 2
```

#### Fase 4: Pre-Commit Gate

```
[ ] ¿Ponytail ladder aplicada? (mínimo código que funciona)
[ ] ¿Tests pasan? (just verify local)
[ ] ¿Documentación afectada actualizada?
[ ] ¿Commit message sigue Conventional Commits?
[ ] ¿Cambio atómico (~100 líneas)?
```

#### Fase 5: Commit (si pasa verify)

```bash
git add -p
git commit -m "tipo(scope): ID — descripción breve"
```

#### Fase 6: Actualizar Plan File + Recitation

| Evento | Campo a actualizar |
|--------|-------------------|
| Gate evaluado | `Gate Result:`, `Estado: ⏳ IN PROGRESS` |
| Iteración hecha | Agregar fila en tabla de iteraciones |
| Commit hecho | `Commit: abc1234`, `Estado: ✅ COMPLETED` |
| Falla permanente | `Estado: ❌ FAILED`, notas del error |

**SIEMPRE:** Reescribir el bloque RECITATION al final del archivo.

#### Fase 7: Yield

Después de actualizar el plan file, el agente se detiene. El harness externo
lee el plan file, detecta si hay más tareas, y decide si invocar de nuevo.

---

## 5. Escalation Ladder (Retry Strategy)

```
ESCALÓN 1: Retry con feedback
  - Mismo enfoque, con el error específico como input
  -> Fix: ~80% de errores

ESCALÓN 2: Contexto fresco
  - Resumir lo aprendido (~200 tokens)
  - Arrancar con resumen + error
  -> Fix: contexto contaminado

ESCALÓN 3: Estrategia diferente
  - Approach materialmente distinto
  -> Fix: approach original no funcionaba

ESCALÓN 4: Escalar a humano
  - Documentar qué se intentó y qué falló
  - Commit del WIP a la branch
  - Marcar ❌ FAILED
  - Seguir con la siguiente tarea
```

**El harness detecta stall.** Si ve 2 iteraciones consecutivas con el mismo
error (misma tarea, mismo resultado de verify), NOTIFICA al usuario y pregunta
si abortar.

---

## 6. Budget Management

| Budget | Default | Acción al alcanzarlo |
|--------|---------|---------------------|
| **Iteraciones por tarea** | 5 | El harness marca ❌ FAILED y sigue |
| **Stall consecutivo** | 2 iguales | Harness pausa, pregunta al usuario |
| **Tokens por invocación** | N/A (contexto fresco cada turno) | OpenCode maneja internamente |

El harness lleva el ledger en la tabla de iteraciones del plan file.

---

## 7. Context Preservation

En un modelo por-turnos, el contexto se resetea en cada invocación. El plan file
es el único estado persistente.

### Técnicas

1. **Recitation block** al final del plan file (ver sección 3)
2. **Save point** cuando se completa una tarea:

```
=== CONTEXT SAVE POINT ===
Fecha: YYYY-MM-DDTHH:MM
Branch: fix/code-xxx
CI pendiente: no
Próxima tarea: TASK-N — <nombre>

Decisiones:
- Se eligió X sobre Y porque [razón breve]

Problemas conocidos:
- [issue]
=== END CONTEXT SAVE ===
```

3. **Compaction en el plan file**: Resumir iteraciones viejas, mantener la
   recitation actual, errores activos, y decisiones.

---

## 8. Probes de Integridad del Harness

El harness ejecuta estas verificaciones:

| Check | Falla | Acción |
|-------|-------|--------|
| Plan file no existe | ❌ | `Write-Error "Plan file not found"` |
| Plan file sin tasks ❌ | ✅ | Mostrar "todas completadas", exit 0 |
| Recitation corrupta | ❌ | Abortar, pedir revisión manual |
| Misma tarea 2× sin progreso | ⚠️ | Preguntar al usuario |
| Harness PID en plan file | ℹ️ | Prevenir doble ejecución |
| git status sucio al empezar | ⚠️ | Preguntar si commitear o stash |

---

## 9. Completion & Finalization

Cuando el harness detecta que todas las tasks están ✅ o ❌:

1. Ejecutar `skill progreso` (migrar backlog)
2. Verificar plan file actualizado
3. Push final: `git add -A && git commit -m "campaign: complete" && git up`
4. Marcar plan file como `Estado: ✅ COMPLETADO`

Resumen final:
```
Total: N
✅ Completadas: N
❌ Fallidas: N
🟡 Deferidas: N
❌ Skipped: N
🔴 Bloqueadas: N
```

---

## 10. Referencias de Loop Engineering

| Fuente | Concepto clave |
|--------|---------------|
| Steve Kinney, "Anatomy of an Agent Loop" (2026) | Todo framework converge en el mismo `while`. La diferencia está en el harness. |
| Addy Osmani, "Loop Engineering" (2026) | Verificación mecánica, stall detection, plan/act/verify. |
| Boris Cherny, Claude Code Lead (2026) | "I don't prompt anymore. I have loops running that prompt Claude." |
| Anthropic, "Harness Design for Long-Running Apps" (2025) | Context resets, plan/execute/review, durable artifacts. |
| Anthropic, "Building Effective AI Agents" (2024) | Workflows vs agents. "Start with a single loop." |
| Manus, "Recitation Pattern" (2025) | todo.md rewrite for goal preservation at end of context. |
| redreamality, "Agent Harness Pattern" (2026) | 5 layers: context, streaming, recovery, termination, state. |
| niv0, "Claude Code from Scratch" (2026) | 23-phase reproduction of Claude Code harness. |
| d3vr, "OCLoop" (2026) | OpenCode loop harness with dashboard, plan-based iteration. |
| ByBrawe, "opencode-loop" (2026) | `/loop` and `/loop-goal` plugin for OpenCode TUI. |
| Loop Engineering Guide (2026) | Circuit breaker: iterations, stagnation, no-progress, tokens. |
| StackOne, "Agent Suicide by Context" (2026) | 67.6% tokens son tool outputs. Sub-agents como defensa. |
| Inngest Blog, "Agent Loop Architecture" (2026) | Loop + skill + orchestrator. Durable execution. |
| Oracle Developers (2026) | 3 niveles: Level 1 (tools), Level 2 (memory), Level 3 (harness). |

---

## 11. Prompt Templates

### Prompt 0: Iniciar Campaña (TRIAGE + GATE + CREAR PLAN)

```
Cargá las skills brainstorming, writing-plans, idea-refine, y ponytail (full).

Tengo tareas para evaluar desde <ruta-backlog>. Aplicá el Task Triage Gate
del backlog-executor para CADA tarea: ✅ DO, 🟡 DEFER, ❌ SKIP, 🔴 BLOQUEADO.

Reglas:
1. Bug ya inexistente o feature ya implementada → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden

Después del gate, creá docs/plans/YYYY-MM-DD-<campaign>.md con:
- Solo tareas ✅ DO, ordenadas por prioridad real
- Gate result y contract para cada una
- Estado inicial ⬜ PENDING
```

### Prompt 1: Ejecutar UNA Iteración (uso con harness)

**Este es el prompt que el harness inyecta en cada `opencode run`:**

```
Cargá las skills writing-plans, incremental-implementation, ponytail (full).

Plan file: <ruta-al-plan-file>

INSTRUCCIONES DE UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

1. Leé el plan file COMPLETO.
2. Buscá la recitation block o la primera tarea ⬜ PENDING / ⏳ IN PROGRESS.
3. Determiná la PRÓXIMA ACCIÓN CONCRETA:
   a. ¿Gate no evaluado? → Evaluar gate
   b. ¿Blast radius no hecho? → codegraph_explore
   c. ¿Código no escrito? → Implementar (~100 líneas)
   d. ¿Verify no corrido? → Ejecutar comando mecánico
   e. ¿Verify pasa? → Commit + actualizar plan
   f. ¿Verify falla? → Aplicar escalón 1 de retry
4. Ejecutá SOLO esa acción.
5. Actualizá el plan file: estado, iteraciones, notas.
6. **ESCRIBÍ EL RECITATION BLOCK al final del archivo.**
7. Detenete. No sigas a la siguiente tarea.

REGLAS:
- Sin excepción: después de CADA acción → actualizar plan file + recitation
- Ponytail activo: stdlib > reusar > dependency > desde cero
- Si verify falla 2 veces con mismo error → marcar ❌ FAILED, escribir por qué
- No cambies scope. Si encuentras algo extra, anotalo pero no lo implementes
```

### Prompt 2: Arrancar el Harness (para el usuario)

```
# Abrir PowerShell como administrador NO es necesario.
# Ejecutar desde la raíz del proyecto:

.\harness-executor.ps1 -PlanFile docs\plans\2026-07-13-code-task-execution-campaign.md

# Alternativa con opencode-loop (si está instalado):
/loop-goal "implementar todas las tareas ❌ del plan file docs/plans/campaign.md"
```

---

## Apéndice A: Harness PowerShell

Ver `harness-executor.ps1` en la raíz del proyecto.

```powershell
# Uso básico
.\harness-executor.ps1 -PlanFile docs\plans\mi-plan.md

# Con modo debug (no ejecuta opencode, solo muestra qué haría)
.\harness-executor.ps1 -PlanFile docs\plans\mi-plan.md -DryRun

# Con intervalo entre iteraciones (segundos)
.\harness-executor.ps1 -PlanFile docs\plans\mi-plan.md -Interval 30
```

---

## Apéndice B: Integración con opencode-loop

Si instalaste el plugin `opencode-loop`:

```bash
oc plugin install opencode-loop
```

Usar Goal Mode para ejecución autónoma:

```
/loop-goal "Cargá backlog-executor. Ejecutá el plan file docs/plans/campaign.md una tarea a la vez. Después de cada tarea, actualizá el plan file y escribí la recitation. No avances a la siguiente tarea sin haber completado la actual (verified, committed, plan updated)."
```

El plugin monitorea idle events y re-inyecta el prompt automáticamente.

---

## Apéndice C: Troubleshooting

| Síntoma | Causa | Solución |
|---------|-------|----------|
| El agente hace 2+ tareas en un turno | Ignoró la regla "una iteración" | Usar el harness, no el prompt directo |
| El harness no detecta progreso | Recitation block faltante o corrupto | Verificar que el agente escribió `=== RECITATION ===` |
| opencode-loop no arranca | Plugin no instalado | `oc plugin install opencode-loop` |
| El agente se detiene a mitad de verify | Tool output muy grande | Verificar que el comando verify no excede límites de tool output |
| Misma tarea reprocesada infinitamente | Harness stall detection mal configurado | Verificar que `$stallThreshold` es >= 2 |

---

## Apéndice D: Comandos Rápidos (VantaDB)

| Comando | Propósito |
|---------|-----------|
| `cargo check -p vantadb` | Build rápido, solo crate core |
| `cargo nextest run --profile audit --workspace --build-jobs 2` | Tests completos |
| `cargo fmt --check` | Formato |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lints |
| `just verify` | Pre-flight completo |
| `codegraph_explore "query"` | Blast radius antes de editar |
| `rust-analyzer-mcp rust_analyzer_diagnostics file_path` | Errores de compilación |
| `skill progreso` | Migrar backlog → progreso |
