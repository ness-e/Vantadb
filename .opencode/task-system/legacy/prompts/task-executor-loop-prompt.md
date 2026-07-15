# Single-Task Executor Loop

> **Modo:** Tarea única compleja. Ejecutás esto cuando querés dedicar atención completa
> a una sola tarea: implementación pesada, feature nueva, refactor grande.
>
> **Anchor:** `.opencode/skills/task-executor/VISION.md` — north star. Leer antes de empezar.

## Entrada

Variables de entorno / argumento del loop:
- `TASK` = ID de la tarea (ej: `NUEVO-13`)
- `DESC` = Descripción corta (ej: `HNSW PID loop: auto-tuning ef_construction y M`)

## Flujo General

```
FASE A: DISCOVERY — analizar, investigar, determinar todo
FASE B: DEFINITION — crear artifact de tarea con plan atómico
FASE C: EXECUTION — implementar, verificar, iterar, cerrar
```

---

## FASE A: Discovery

### A1. Cargar skills base
```
skill progreso
```

### A2. Obtener contexto de la tarea
Leer el plan file activo (`docs/plans/*.md`) si existe, y buscar la tarea por ID para obtener:
- Descripción y esfuerzo
- Archivos involucrados
- Estado actual

### A3. Mapear blast radius vía CodeGraph
```
codegraph_explore "símbolos/archivos de la tarea ${TASK}"
```
Responder en el task log:
- **CALLERS:** qué módulos llaman a los archivos que vamos a tocar
- **CALLEES:** de qué dependen estos archivos
- **API CHANGES:** ¿contratos públicos rotos? ¿serialización? ¿migración?
- **TEST IMPACT:** qué tests existentes cubren esta área

### A4. Auto-detectar tipo de tarea
Según los archivos que aparecen en codegraph_explore:

| Archivos | Tipo | Skills a cargar |
|----------|------|-----------------|
| `src/**` (Rust core) | Rust | `source-driven-development`, `doubt-driven-development`, `ponytail full` |
| `web/src/**` | Frontend | `frontend-ui-engineering`, `design-taste-frontend` |
| `vantadb-python/**` | Python SDK | `source-driven-development` |
| `vantadb-ts/**` | TypeScript SDK | `source-driven-development` |
| `docs/**` | Documentation | `writing-plans` |
| `*.md` plan/backlog | Planning | `writing-plans`, `planning-and-task-breakdown` |

Si hay archivos de **múltiples tipos**, cargar skills de todos los tipos aplicables.

### A5. Auto-detectar checks según tipo
| Tipo | Checks |
|------|--------|
| Rust | `cargo build`, `cargo nextest run --profile audit --workspace --build-jobs 2`, `cargo fmt --check` |
| Frontend (web/) | `npx tsc --noEmit`, `npm run lint` |
| Python | `python -m pytest vantadb-python/tests/ -v` |
| TypeScript | `npx tsc --noEmit`, `npm test` |
| Docs sólamente | `ruby scripts/validate-docs-coverage.ps1` |
| Mixto | TODOS los checks aplicables |

### A6. Auto-estimar turns necesarios
Basado en esfuerzo del plan file y complejidad:

| Esfuerzo | Turns estimados |
|----------|----------------|
| 🟢 Bajo | 5-10 |
| 🟡 Medio | 15-30 |
| 🔴 Alto | 30-60 |

Guardar estimación para usarla como `--max-turns`.

### A7. Investigación web si hay ambigüedad
Si la tarea involucra:
- APIs/librerías externas cuya doc no está en el código
- Patrones de diseño no familiares
- Decisiones técnicas donde hay múltiples enfoques

→ Usar MetaSearchMCP.search_web + Argus.extract_content para investigar.
→ Documentar findings en la task definition (Fase B).

Registrar progreso:
```
opencode_loop_goal_progress summary:"Fase A completa — blast radius mapeado, tipo=${TIPO}, skills=[...], checks=[...]" next:"Fase B: crear task definition"
```

---

## FASE B: Definition

### B1. Crear task definition
Crear archivo `.opencode/skills/task-executor/tasks/${TASK}.md` con:

```markdown
---
id: "${TASK}"
name: "${DESC}"
created: "$(date +%Y-%m-%d)"
module: "$(basename $(dirname $(codegraph_explore files principales)))"
status: "ready"
estimate: "N turns"
---

## Contract
{condición verificable y observable}

## Atomic Steps
1. {primer cambio atómico, ~100 líneas}
2. {segundo cambio atómico}
3. ...

## Skills
{lista de skills detectadas en Fase A}

## Checks
{comandos de verificación}

## Blast Radius
{CALLERS + CALLEES + API CHANGES de Fase A}

## Investigation Notes
{hallazgos de investigación web, si aplica}
```

### B2. Definir contrato verificable
El contrato debe ser algo como:
- "cargo build && cargo nextest run pasa, y [comportamiento específico] funciona"
- "npx tsc --noEmit pasa, y el componente renderiza [condición visual]"

NO usar contratos vagos como "funciona bien" o "está completo".

### B3. Descomponer en pasos atómicos
Cada paso = ~100 líneas de código, verificable individualmente.
Si un paso es >100 líneas, dividirlo.

```
codegraph_explore para verificar que los pasos cubren todos los archivos del blast radius.
```

Registrar progreso:
```
opencode_loop_goal_progress summary:"Fase B completa — task definition creada en tasks/${TASK}.md" next:"Fase C: implementar paso atómico 1/N"
```

---

## FASE C: Execution

### C0. State Machine (Statewright pattern)

Cada sección de FASE C tiene un estado formal. El loop registra el estado actual
y solo transiciona si la anterior está ✅. Esto evita saltos inválidos.

```
Estados válidos y transiciones:

  PLAN     → ACT       (diseñar → implementar)
  ACT      → VERIFY    (implementar → verificar)
  VERIFY   → PLAN      (falló → reintentar)
  VERIFY   → STALL     (3 same-error → bloqueo)
  VERIFY   → COLLATERAL (pasó → errores colaterales)
  COLLATERAL → RESEARCH (ambigüedad → investigar)
  RESEARCH → ACT       (investigado → implementar)
  COLLATERAL → EVALUATE (sin más errores → auto-evaluar)
  EVALUATE → REVIEW    (auto-evaluación pasa → revisión)
  EVALUATE → ACT       (auto-evaluación falla → re-implementar)
  REVIEW   → VERIFY    (review encuentra issues → re-verificar)
  REVIEW   → ACCEPT    (review pasa → aceptación final)
  ACCEPT   → CLOSE     (aceptado → commit)

Transiciones inválidas (saltarse pasos):
  PLAN → EVALUATE      ❌ no implementado
  ACT  → ACCEPT        ❌ no verificado
  ACT  → CLOSE         ❌ no revisado
  ACT  → REVIEW        ❌ no evaluado
```

Registrar estado después de cada transición:
```
state: ${ESTADO_ACTUAL} (desde: ${ESTADO_ANTERIOR})
```

Referencia: [Statewright](https://github.com/statewright/statewright) (415⭐, 2026) —
state machine guardrails redujeron errores 80% shrinking tool space.

### C1. Loop de implementación
Para cada paso atómico en orden:

```
1. PLAN: leer el paso y decidir el cambio exacto
2. ACT: editar código siguiendo skills cargadas + ponytail ladder
3. VERIFY: correr los checks correspondientes
4. SI PASA:
   - Registrar progreso
   - Siguiente paso
5. SI FALLA → RETRY LADDER:
   a) Leer el error exacto
   b) Corregir con feedback del error
   c) 2 intentos consecutivos = MISMO error → ESCALAR:
      - codegraph_explore para más contexto
      - Si sigue: opencode_loop_goal_blocked
```

### C2. Ponytail Ladder (aplica en CADA paso)
```
1. ¿Ya existe en el codebase? → reusar
2. ¿Stdlib lo hace? → stdlib
3. ¿Platform feature? → platform
4. ¿Dependency instalada? → usarla
5. ¿Una línea? → una línea
6. Recién acá: código mínimo
```

### C3. Stagnation Detection Middleware

Después de cada intento fallido en C1-C2, ejecutar:

```
CHECK:
  - ¿Es la 3ra vez consecutiva con el MISMO error?
  - ¿Es la 5ta iteración total sin avanzar de paso?
  - ¿Se tocaron los mismos archivos en las últimas 3 iteraciones?

  Si ALGUNA condición es true → STAGNATION DETECTED:
    1. opencode_loop_goal_blocked
       reason:"Stagnation: {N} intentos sin progreso en {archivos}"
       needed:"Revisión manual del error recurrente"
    2. NO seguir iterando — el loop no se auto-corrige solo

  Si ninguna es true → continuar normalmente
```

Referencia: patrón evaluator-optimizer (Lilian Weng, 2025) + harness engineering
no-progress detection (Anthropic, 2026). Ver VISION.md principio #8.

### C4. Errores colaterales (Fase 5 del skill)
Mientras implementás, si encontrás OTRO error:
- **Rápido** (🟢 <30min, mismo archivo): arreglar y commitear junto
- **Lento** (🟡 >30min, módulo diferente): crear entrada en Backlog.md y seguir
- **No perder foco de la tarea principal**

### C5. Refinar skills vía web research
Si durante la implementación surge algo ambiguo (API desconocida, patrón no familiar):
```
MetaSearchMCP.search_web("patrón o API específica")
Argus.extract_content(url_del_resultado)
→ Actualizar Investigation Notes en tasks/${TASK}.md
```

### C6. Evaluator-Optimizer Loop

Antes de la review final, ejecutar auto-evaluación:

```
1. codegraph_explore "task_id=${TASK} post-implement"
   → confirmar que todos los archivos del blast radius fueron tocados
   → confirmar que no hay callers rotos

2. Evaluar contra el contrato:
   - ¿El contrato se cumple AHORA? (no "cuando termine el review")
   - Si NO → identificar qué falta exactamente y volver a C1
   - Si SÍ → continuar

3. Auto-crítica en 3 ejes:
   a) CORRECTITUD: ¿edge cases cubiertos? ¿input vacío? ¿límites?
   b) SIMPLICIDAD: revisar con ponytail ladder. ¿algo se puede acortar?
   c) CONSISTENCIA: ¿sigue el mismo patrón que el código existente?

4. Si la auto-crítica encuentra issues → volver a C1 con lista concreta
5. Si pasa → continuar a C6 (review post-implementación)
```

Este loop evita que el agente se auto-apruebe cambios incompletos.
Máximo 2 iteraciones de evaluator-optimizer. Si en la 3ra sigue sin pasar,
bloquear con `opencode_loop_goal_blocked`.

### C6.5 Self-Harness: Propose → Evaluate → Accept Gate

Después del evaluator-optimizer y ANTES de la review formal, ejecutar
el gate de aceptación (patrón Self-Harness propose-evaluate-accept):

```
1. PROPOSE:
   El cambio actual se "propone" como candidato a completo.
   - Leer el diff completo: git diff
   - Resumir en 3 líneas: qué cambió, por qué, qué contrato cumple

2. EVALUATE:
   Evaluar la propuesta contra el contrato y la tarea:
   - ¿SATISFACE el contrato? (sí/no — booleano, sin matices)
   - ¿ROMPE algo fuera del blast radius? (codegraph_explore check)
   - ¿INTRODUCE deuda técnica nueva? (ponytail-review)
   - ¿ESTÁ documentado si cambió API pública?

3. ACCEPT / REJECT:
   Si todas las condiciones son ✅ → ACCEPT → continuar a C7
   Si ALGUNA es ❌ → REJECT → volver a C1 con lista de issues
   Si 2 rejections consecutivas → opencode_loop_goal_blocked
     reason:"Self-Harness: 2 rejections en ${TASK}"
     needed:"Revisar si el enfoque de la tarea es correcto"
```

Referencia: Self-Harness propose-evaluate-accept loop (cierra el círculo
evaluator-optimizer + quality gate).

### C7. Review post-implementación
Después del último paso:
```
1. codegraph_explore para verificar impacto completo
2. skill code-review-and-quality (auto-review del diff)
3. skill doubt-driven-development (si stakes altos: datos, seguridad, dinero)
4. ¿Tests cubren edge cases? ¿Documentación actualizada?
5. Actualizar AGENTS.md con learnings
```

### C8. Verificación full
```
[ ] cargo build --workspace (o warm cache)
[ ] cargo nextest run --profile audit --workspace --build-jobs 2
[ ] npx tsc --noEmit (frontend)
[ ] cargo fmt --check
[ ] Contract check: verificar condición del contrato explícitamente
```

### C9. Commit + Cierre
```
1. Última verificación pasa
2. git add -p (revisar cada cambio)
3. git commit -m "feat(${TASK}): ${DESC}

   Blast radius: {módulos afectados}
   Skills: {skills usadas}
   Contract: {condición cumplida}
   Errores colaterales: {ninguno | lista}"

4. skill progreso (Trigger 1: migrar a progreso, Backlog, plan, CHANGELOG)
5. Actualizar AGENTS.md con learnings de la tarea
6. opencode_loop_goal_complete
   summary:"${TASK} — ${DESC} completo"
   evidence:"cargo nextest pasa (N/N), contrato: ${CONTRATO}"
```

---

## Diagrama de flujo completo

```
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=NUEVO-13"
  │
  ├─ FASE A: DISCOVERY
  │   ├─ codegraph_explore → blast radius
  │   ├─ auto-detectar tipo → skills + checks
  │   ├─ auto-estimar turns
  │   └─ web research si ambigüedad
  │
  ├─ FASE B: DEFINITION
  │   ├─ crear tasks/NUEVO-13.md
  │   ├─ definir contrato
  │   └─ descomponer en pasos atómicos
  │
  └─ FASE C: EXECUTION (State Machine C0)
      ├─ C1: loop de implementación (1 paso a la vez)
      ├─ C2: ponytail ladder en cada paso
      ├─ C3: stagnation detection (3 same-error = block)
      ├─ C4: errores colaterales atrapados
      ├─ C5: refinar skills vía web research
      ├─ C6: evaluator-optimizer loop (auto-crítica 3 ejes)
      ├─ C6.5: Self-Harness propose-evaluate-accept gate
      ├─ C7: review + doubt-driven
      ├─ C8: verificación full
      └─ C9: skill progreso → commit → complete
```
