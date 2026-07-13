---
name: backlog-executor
description: >
  Autonomous task execution loop for backlog-driven development.
  One task per iteration: triage → eval gate → implement → verify → commit → update → next.
  Includes context preservation protocol, stall detection, budget management,
  and the recitation pattern to survive long sessions without goal drift.
compatibility: opencode
---

# Backlog Executor Skill — Loop Engineering for Task Execution

> Basado en investigación de patrones de loop engineering (2025-2026):
> Ralph Loop, ReAct, Plan-Execute-Verify, Anthropic Harness Design,
> Addy Osmani / Boris Cherny / Steve Kinney — loop engineering.
> Ver referencias completas en sección 10.

---

## Core Loop Architecture

Cada tarea ejecuta un ciclo **Plan → Act → Verify**, donde la verificación es
**mecánica** (código, no opinión del modelo):

```
      ┌──────────────────────────────────────────────────────┐
      │                    PLAN                              │
      │  Leer plan file → código existente → blast radius   │
      └─────────────┬────────────────────────────────────────┘
                    │
                    ▼
      ┌──────────────────────────────────────────────────────┐
      │                    ACT                               │
      │  Implementar cambio atómico (~100 líneas max)        │
      └─────────────┬────────────────────────────────────────┘
                    │
                    ▼
      ┌──────────────────────────────────────────────────────┐
      │                    VERIFY                            │
      │  just verify / nextest / typecheck / lint — CODIGO   │
      │  (nunca auto-reporte del modelo)                     │
      └─────────────┬────────────────────────────────────────┘
                    │
           ┌────────┴────────┐
           ▼                 ▼
         PASA              FALLA
           │                 │
           ▼                 ▼
      ┌─────────┐    ┌──────────────────┐
      │ COMMIT  │    │ RETRY LADDER     │
      │ push +  │    │ 1. con feedback  │
      │ CI      │    │ 2. contexto limpio│
      └─────────┘    │ 3. nueva estrategia│
           │         │ 4. escalar humano │
           ▼         └──────────────────┘
       SIGUIENTE
       TAREA
```

### El Contrato del Loop

Cada tarea necesita un **contrato observable** antes de empezar: una definición
de "completado" que código pueda verificar mecánicamente.

| ❌ Contrato vago | ✅ Contrato verificable |
|-----------------|------------------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, `cargo machete` 0 warnings, `cargo nextest run --profile audit` pasa" |
| "Mejorar la web" | "npm run typecheck 0 errors, npm run lint 0 errors, vitest run --pass" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |

**El contrato se escribe en el plan file antes de empezar a implementar.**

---

## 1. Task Triage & Evaluation Gate

Antes de implementar nada, cada tarea debe pasar por esta evaluación. El resultado
se registra en el plan file y determina si se ejecuta, aplaza o descarta.

### Gate Checklist

```
Tarea: [ID] — [Nombre]
Archivos: [paths]

[ ] 1. ¿RELEVANCIA — Sigue siendo necesaria?
       codegraph_explore "código a modificar" para verificar estado actual
       → Si el bug ya no existe o la feature ya está implementada → ❌ SKIP

[ ] 2. ¿IMPACTO REAL — A quién afecta?
       → Data integrity, seguridad, CI, usuarios, release blocker?
       → Si es cosmético sin queja de usuario → 🟡 DEFER

[ ] 3. ¿COSTO/BENEFICIO — Justifica el esfuerzo?
       → Esfuerzo estimado vs. impacto real
       → Si impacto << esfuerzo → 🟡 DEFER o ❌ SKIP

[ ] 4. ¿DEPENDENCIAS — Todo lo necesario está listo?
       → ¿Hay tareas bloqueantes sin completar?
       → Si depende de algo no hecho → 🔴 BLOQUEADO

[ ] 5. ¿RIESGO — Puede romper algo existente?
       → ¿Requiere migración? ¿Cambia API pública? ¿Afecta serialización?
       → Alto riesgo → cargar `doubt-driven-development`

[ ] 6. ¿SCOPE — Es código puro o mezcla tipos?
       → Si mezcla código + infra + CI + docs, separar en sub-tareas

Resultado: ✅ DO | 🟡 DEFER (cuándo retomar) | ❌ SKIP (por qué) | 🔴 BLOQUEADO (dependencia)
```

---

## 2. Campaign Plan File Format

El archivo de plan vive en `docs/plans/YYYY-MM-DD-<campaign>.md` y es la
**fuente de verdad del estado**. Se lee antes de cada acción y se escribe
después de cada acción.

```markdown
# Plan de Ejecución: [Nombre]

> **Inicio:** YYYY-MM-DDTHH:MM
> **Fuente:** docs/Backlog.md / docs/bitacora.md
> **Estado:** ⏳ EN PROGRESO | ✅ COMPLETADO | ❌ ABORTADO
> **Contrato del loop:** [condición única que detiene el loop]

## Tasks

### Task 1: [ID] — [Nombre]

- **Fuente:** Backlog.md línea N
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🔴 2-3d
- **Prioridad original:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** `path/to/file.rs`
- **Gate Result:** ✅ DO | 🟡 DEFER: razón | ❌ SKIP: razón | 🔴 BLOQUEADO: dependencia
- **Contrato:** "qué significa 'completado' en términos verificables"
- **Estado:** ⬜ PENDING | ⏳ IN PROGRESS | ✅ COMPLETED | ❌ FAILED
- **Branch:** `fix/code-xxx-descripcion`
- **Commit:** `abc1234`

  **Iteraciones:**
  | # | Acción | Resultado | Tokens |
  |---|--------|-----------|--------|
  | 1 | Implementar fix | cargo nextest pasa ✅ | ~8K |
  | 2 | CI run | ✅ pasa | ~2K |
  | 3 | — | — | — |

  **Notas:**
  - Contexto aprendido durante implementación
  - Problemas encontrados y cómo se resolvieron
  - Decisiones tomadas (para futuros ADRs)

  **Check post-CI:**
  - [ ] GitHub workflow principal pasa
  - [ ] No se rompieron otros workflows
  - [ ] `skill progreso` ejecutado

---

### Task 2: ...
```

### Recitation Pattern (Anti-Goal-Drift)

Cada vez que el plan file se actualiza (después de cada acción), se **reescribe**
el objetivo al final del archivo para combatir el "lost in the middle" del
contexto del modelo:

```
=== RECITATION ===
Objetivo activo: Task 1 — Implementar CODE-012
Estado actual: Implementación completa, esperando CI
Próxima acción: Monitorear CI run #12345
Contrato del loop: "just verify pasa y CI workflow es green"
Próxima tarea si esta completa: Task 2 — CODE-015
=== END RECITATION ===
```

---

## 3. Execution Loop (por tarea)

Para cada tarea en estado `⬜ PENDING`:

### Fase 0: Cargar Skills

| Fase | Skills |
|------|--------|
| **TRIAGE** | `brainstorming` (ambiguo), `interview-me` (faltan requisitos), `idea-refine` |
| **DEFINE** | `spec-driven-development`, `writing-plans` |
| **PLAN** | `planning-and-task-breakdown` |
| **BUILD** | `incremental-implementation`, `test-driven-development`, `doubt-driven-development` (stakes altos) |
| **BUILD (Rust)** | `source-driven-development` |
| **BUILD (Frontend)** | `frontend-ui-engineering` |
| **VERIFY** | `debugging-and-error-recovery`, `browser-testing-with-devtools` (web) |
| **REVIEW** | `code-review-and-quality`, `code-simplification`, `security-and-hardening`, `performance-optimization` |
| **SHIP** | `git-workflow-and-versioning`, `ci-cd-and-automation`, `documentation-and-adrs` |

**Siempre:** `skill progreso` al inicio y al completar cada tarea.
**Siempre:** Ponytail `full` activo durante BUILD (escalera de eficiencia).

### Fase 1: Leer Estado + Recitation

```markdown
1. Leer docs/plans/<campaign-file>.md
2. Buscar última recitation o tarea ⏳/⬜ PENDING
3. Si hay WIP (⏳), retomar desde Fase 3
4. Leer fuente original (Backlog.md) para contexto completo
```

### Fase 2: Blast Radius + Contexto

```
skill systematic-debugging     # si es bug
codegraph_explore "query sobre archivos a modificar"

Responder:
- Callers: qué módulos llaman a estos archivos
- Callees: de qué dependen
- Blast radius: ¿contratos rotos? ¿cambia API pública?
  ¿performance? ¿serialización? ¿seguridad? ¿migración necesaria?
```

### Fase 3: Implementar (Plan → Act → Verify Loop Interno)

Cada iteración interna sigue:

```
LOOP:
  1. PLAN: decidir el cambio atómico (~100 líneas)
  2. ACT: editar código
  3. VERIFY: verificación MECÁNICA (no auto-reporte)
     - Rust:  cargo check (o parcial con -p <crate>)
     - Web:   npm run typecheck
     - Tests: cargo nextest run <test_name>
  4. Si VERIFY pasa → OK, salir del loop interno
  5. Si VERIFY falla → aplicar RETRY LADDER (ver sección 6)
```

**Stall Detection Interna:** Si dos iteraciones consecutivas producen el mismo
error (mismo archivo, misma línea, mismo mensaje), NO reintentar — escalar.

### Fase 4: Pre-Commit Gate

```
[ ] ¿Código mínimo que funciona? (Ponytail ladder aplicada)
[ ] ¿Tests pasan? (just verify local)
[ ] ¿Documentación afectada actualizada?
[ ] ¿Commit message sigue Conventional Commits?
[ ] ¿Cambio atómico (~100 líneas)?
```

### Fase 5: Commit + Push

```bash
git add -p                    # revisar cada cambio
git commit -m "tipo(scope): descripción breve"
git up                        # push -u origin HEAD
```

### Fase 6: Monitoreo CI

```bash
# Esperar al workflow principal
RUN_ID=$(gh run list --branch $(git branch --show-current) --limit 1 \
  --json databaseId --jq '.[0].databaseId')
gh run watch $RUN_ID

# Si falla:
if [ "$(gh run view $RUN_ID --json conclusion --jq '.conclusion')" = "failure" ]; then
  gh run view $RUN_ID --log-failed > logs/${RUN_ID}-failed.log
  # Extraer errores clave
  echo "=== ERRORES ==="
  rg -i "error\[|error:|FAILED|test.*FAILED" logs/${RUN_ID}-failed.log | head -10
fi
```

### Fase 7: Actualizar Plan File + Recitation

| Acción | Actualizar |
|--------|-----------|
| Gate eval hecho | `Gate Result:`, `Gate Reason:` |
| Empezar implementación | `Estado: ⏳ IN PROGRESS`, crear branch |
| Iteración completada | Agregar fila en tabla de iteraciones |
| Commit hecho | `Commit: abc1234` |
| CI pasa | `Estado: ✅ COMPLETED`, checklist post-CI |
| CI falla | `Estado: ❌ FAILED`, notas + error |
| Pendiente de próxima sesión | **Recitation block** al final del archivo |

**Después de cada actualización**, agregar o refrescar el bloque de recitation.

---

## 4. Context Preservation Protocol

La muerte por contexto es el failure mode #1 de loops largos.
Investigación de 2026 muestra que el 67.6% de los tokens en un loop largo
son resultados de herramientas (tool outputs), no razonamiento del modelo.

### Cuándo ejecutar

| Señal | Acción |
|-------|--------|
| ~70% del contexto ocupado | Compactar activamente |
| ~85% del contexto ocupado | Ejecutar protocolo + preparar handoff |
| Tool call devuelve >5000 tokens | Offload a sub-agente o pointer |

### Técnicas de Preservación

#### A. Offloading de Tool Outputs

Cuando una herramienta devuelve muchos tokens (>3000), no los pases completos
al contexto. En vez de:

```
Resultado de grep: [5000 líneas de código]
```

Pon:

```
Resultado de grep: resumen: "se encontraron 42 matches en 15 archivos.
Archivos clave: src/core.rs (líneas 120-145 con patrón X)
Ver detalle: grep guardado en logs/grep-result-001.log"
```

#### B. Compaction Selectivo

Cuando el contexto está ~70% lleno:

```
1. Resumir las iteraciones pasadas (no borrar, comprimir)
2. Mantener: la recitation actual, errores activos, decisiones
3. Descartar: stack traces viejos, tool outputs ya procesados,
   logs de compilación exitosa, intentos fallidos ya descartados
```

#### C. Recitation Pattern (contra goal drift)

Cada actualización del plan file incluye un bloque de recitation al final.
Esto empuja el objetivo actual al final del contexto (donde la atención del
modelo es más fuerte).

```
=== RECITATION ===
Objetivo activo: Task 1 — CODE-012
Estado actual: Implementación completa, esperando CI
Próxima acción: Monitorear CI run #12345
Contrato del loop: "just verify pasa y CI workflow es green"
Próxima tarea: Task 2 — CODE-015
=== END RECITATION ===
```

#### D. Save Point (handoff entre sesiones)

Cuando el contexto está ~85% lleno o después de cada tarea (lo que ocurra
primero), escribir un save point al final del plan file:

```
=== CONTEXT SAVE POINT ===
Fecha: YYYY-MM-DDTHH:MM
Tokens aprox usados: ~140K / 200K

Estado del workspace:
- Cambios sin commit: (ninguno | archivos X,Y,Z)
- Branch actual: <nombre>
- CI pendiente: no (último run: ✅ / ❌)

Próxima sesión:
1. Leer docs/plans/<campaign-file>.md
2. Buscar último ⏳ o próximo ⬜ PENDING
3. Cargar skills correspondientes a la fase
4. Continuar desde Fase 1 del Execution Loop

Decisiones registradas:
- Se eligió enfoque X sobre Y porque [razón breve]
- Se difirió [algo] para no expandir scope

Problemas conocidos sin resolver:
- [issue 1]
- [issue 2]
=== END CONTEXT SAVE ===
```

**Al retomar:** leer plan file, encontrar última recitation o save point,
continuar desde la tarea ⏳ correspondiente.

---

## 5. Ciclo Completo: El Prompt Reutilizable

Este es el prompt para arrancar UNA campaña completa:

### Prompt 0: Iniciar Campaña (TRIAGE + GATE)

```
Cargá las skills brainstorming, writing-plans, idea-refine, y ponytail (full).

Tengo una lista de tareas para evaluar. Necesito que para CADA tarea apliques
el Task Triage & Evaluation Gate del skill backlog-executor y determines:

✅ DO — hacer ahora
🟡 DEFER — posponer (razón + condición para retomar)
❌ SKIP — no hacer (razón)
🔴 BLOQUEADO — depende de algo no listo

Reglas:
1. Si el bug ya no existe o la feature ya está implementada → SKIP
2. Si es cosmético sin queja de usuario → DEFER
3. Si el esfuerzo supera al impacto → DEFER o SKIP
4. Si depende de algo no completado → BLOQUEADO (anotar dependencia)
5. La prioridad original en el backlog es sugerencia, no orden

Después del gate, creá docs/plans/YYYY-MM-DD-<campaign>.md con:
- Solo tareas ✅ DO
- Gate result y gate reason para cada una
- Estado inicial ⬜ PENDING
- Tareas ordenadas por prioridad real (no orden original)
- Tasks bloqueadas listadas por separado
- Cada tarea con su contrato verificable

Fuente de tareas: <ruta-al-backlog>
```

### Prompt 1: Ejecutar una Tarea Individual

```
Cargá las skills <skills-según-fase> y ponytail (full).

Tarea: <ID> — <Nombre>
Plan file: docs/plans/<campaign-file>.md

ANTES:
1. Leé el plan file completo
2. Si hay cambios sin commit, preguntá antes de continuar
3. Si la tarea ya está ⏳, retomá donde se dejó

PASOS (backlog-executor Execution Loop):

1. BLAST RADIUS:
   codegraph_explore "query sobre archivos"
   Identificar: callers, callees, implicaciones

2. IMPLEMENTAR (Plan → Act → Verify loop interno):
   - Bug? → systematic-debugging primero
   - Feature? → spec-driven-development
   - Ponytail ladder: stdlib > reutilizar > dependency > desde cero
   - VERIFY: just verify (mecánico, no opinión)
   - Stall detection: mismo error 2× seguidas = escalar

3. PRE-COMMIT GATE:
   ¿Mínimo cambio? ¿Tests? ¿Docs? ¿Conventional Commit?

4. COMMIT + PUSH:
   git add -p && git commit -m "tipo(scope): descripción"
   git up

5. CI MONITOR:
   gh run watch <run_id>
   Si falla: gh run view <run_id> --log-failed > log.txt
   2 intentos de fix, después ❌ FAILED

6. ACTUALIZAR PLAN:
   - Commit SHA, estado, iteraciones, notas
   - Recitation block al final
   - Si contexto >70%, compactar

DESPUÉS DE CADA ACCIÓN → actualizar plan file. Sin excepción.
```

### Prompt 2: Loop Automático (Todas las Tareas)

```
Cargá las skills writing-plans, incremental-implementation, progreso, y ponytail (full).

Plan file: docs/plans/<campaign-file>.md

INSTRUCCIONES:

1. Leé el plan file. Identificá todas las tareas ⬜ PENDING o la ⏳ si hay WIP.

2. Para cada tarea, en orden:
   a. Ejecutá el Execution Loop del backlog-executor skill (fases 1-7)
   b. skill progreso (migrar backlog)
   c. Actualizá el plan file + recitation
   d. Si contexto > ~70% → compactar
   e. Si contexto > ~85% → Context Preservation Protocol + avisar

3. Si una tarea falla (❌ FAILED después de escalation ladder):
   - Documentá por qué
   - NO te detengas — seguí con la siguiente
   - Al final, listá todas las fallidas

4. REPETÍ hasta que no queden tareas pendientes.

5. AL FINAL:
   - Resumí: N ✅ completadas, N ❌ fallidas, N 🟡 deferidas
   - skill progreso (batch final)
   - Push final si hay cambios pendientes
   - Marcá el plan file como Estado: ✅ COMPLETADO

REGLAS DE ORO:
- Sin excepción: después de CADA acción → actualizar plan file + recitation
- Si el contexto se llena → Context Preservation Protocol + aviso
- Si un CI falla → 2 intentos de fix, si sigue → ❌ FAILED
- No cambiar scope — si encuentras algo extra, anotalo en Notas pero no lo implementes
```

---

## 6. Escalation Ladder (Retry Strategy)

Cuando un paso falla, no reintentar ciegamente. Subir la escalera:

```
ESCALÓN 1: Retry con feedback
  - Mismo enfoque, pero con el error específico como input
  - Fix: la mayoría de errores transitorios

ESCALÓN 2: Contexto fresco
  - Resumir lo aprendido (máximo 200 tokens)
  - Descartar el contexto contaminado
  - Arrancar con el resumen + el error
  - Fix: contextos contaminados con stack traces largos

ESCALÓN 3: Estrategia diferente
  - El plan step debe proponer un approach materialmente distinto
  - NO más de lo mismo con distinto nombre
  - Fix: el approach original no funcionaba

ESCALÓN 4: Escalar a humano
  - Escribir: qué se intentó, qué falló, por qué se cree que no funcionará
  - Commit del WIP a la branch (no perder el trabajo)
  - Marcar tarea como ❌ FAILED
  - Seguir con la siguiente tarea
```

**Stall Detection:** Si dos iteraciones consecutivas en el escalón 1 producen
exactamente el mismo error (mismo mensaje, mismo archivo), pasar directamente
al escalón 2 o 3. No quemar tokens en reintentos idénticos.

---

## 7. Budget Management

El loop necesita techos en **tres dimensiones**, no solo una:

| Budget | Default | Qué pasa al alcanzarlo |
|--------|---------|------------------------|
| **Iteraciones** | 25 por tarea | Última llamada: "sintetizá tu mejor respuesta con lo que hay" |
| **Tokens** | Sin límite fijo | ~170K tokens → ejecutar Context Preservation Protocol |
| **Pasos sin progreso** | 3 intentos idénticos | Escalar directamente a escalón 3 o 4 |

Implementación: el archivo de plan lleva la tabla de iteraciones por tarea,
que funciona como ledger para detectar estancamiento.

---

## 8. Recovery & Error Handling

| Situación | Acción |
|-----------|--------|
| **Compilación falla** | `rust-analyzer-mcp diagnostics` para error exacto. Arreglar. Si el fix cambia el approach, actualizar plan file. |
| **Tests existentes se rompen** | El cambio tiene un bug. Revertir y re-evaluar approach. |
| **CI falla post-push** | `gh run view <id> --log-failed > log.txt`. Error del cambio → arreglar. Infraestructura (toolchain, cache, timeout) → anotar y continuar. |
| **Stall detectado** (mismo error 2×) | Saltar al escalón 2 o 3. No reintentar con el mismo contexto. |
| **Contexto >85%** | Ejecutar Context Preservation Protocol y avisar al usuario. |
| **Merge conflict** | `git pull --rebase`, resolver, reintentar push. |
| **git push rechazado** (branch out of sync) | `git pull --rebase`, resolver conflictos, reintentar. |
| **Tarea depende de otra no hecha** | Marcar `🔴 BLOQUEADO`, mover al final, continuar con tareas independientes. |
| **Bug más crítico descubierto durante ejecución** | Anotar como nueva tarea en el plan. NO implementar — completar la actual primero. |
| **Task imposible** (3 intentos, escalera completa) | `❌ FAILED`, documentar causa raíz, seguir a la siguiente. |

---

## 9. Completion & Finalization

Cuando todas las tareas del plan estén procesadas:

```markdown
1. Ejecutar skill progreso (migrar backlog — batch final)
2. Ejecutar ponytail-review (over-engineering residual)
3. Verificar plan file actualizado
4. Push final
5. Marcar plan file como Estado: ✅ COMPLETADO

Resumen final:
- Total: N
  ✅ Completadas: N
  ❌ Fallidas: N
  🟡 Deferidas: N
  ❌ Skipped: N
  🔴 Bloqueadas: N
- Workflows CI afectados: [lista de verificación]
```

---

## 10. Referencias de Loop Engineering (Investigación 2026)

| Fuente | Concepto clave |
|--------|---------------|
| Steve Kinney, "Anatomy of an Agent Loop" (2026) | Todo framework converge en el mismo `while`. La diferencia está en el harness alrededor del loop. |
| Addy Osmani, "Loop Engineering" (2026) | Verificación mecánica, stall detection, plan/act/verify shape. |
| Boris Cherny, Claude Code Lead @ Scale (2026) | "I don't prompt anymore. I have loops running that prompt Claude." Sub-agentes, `/goal`, `/loop`. |
| Anthropic, "Building Effective AI Agents" (2024) | Workflows vs agents. "Start with a single loop." |
| Anthropic, "Harness Design for Long-Running Apps" (2025) | Context resets, plan/execute/review structure, durable artifacts. |
| Manus, "Recitation Pattern" (2025) | `todo.md` rewrite for goal preservation at end of context. |
| Loop Engineering Guide, Loop Context Tool (2026) | Circuit breaker con 4 ejes (iterations, stagnation, no-progress, tokens). |
| Anthropic, "Context Engineering Guide" (2025) | Sub-agent isolation, compaction at 80%, tool output offloading. |
| StackOne, "Agent Suicide by Context" (2026) | 67.6% tokens son tool outputs. Sub-agents y code mode como defensa. |
| Oracle Developers, "The Agent Loop Decoded" (2026) | 3 niveles de loop: Level 1 (tools), Level 2 (memory), Level 3 (harness). |

---

## Apéndice A: Guía Rápida de Herramientas (VantaDB)

| Comando | Propósito |
|---------|-----------|
| `just verify` | Pre-flight: fmt + clippy + test + deny |
| `just check` | `cargo check --workspace` (rápido pre-verify) |
| `cargo check -p vantadb` | Solo crate core (mucho más rápido que workspace) |
| `cargo nextest run --profile audit -p <crate>` | Tests específicos |
| `codegraph_explore "query"` | Blast radius antes de editar |
| `rust-analyzer-mcp rust_analyzer_diagnostics file_path` | Errores de compilación |
| `cargo-mcp cargo_clippy` | Lints |
| `gh run watch <id>` | Monitorear CI |
| `skill progreso` | Migrar backlog → progreso |

### Rust Build Optimization

```bash
# Más rápido que workspace completo
cargo check -p vantadb
cargo check -p vantadb --no-default-features -F "fjall,cli"
cargo check -p vantadb -p vantadb-server -p vantadb-mcp
```
