> **ACTIVE INSTRUCTION — Una Iteración del Ciclo**
> Cargado por `/loop-goal` cuando hay tareas en ejecución.
> Path resolution: `skills/X` → `.opencode/skills/X/`
> Procesar EXACTAMENTE UNA iteración (no una tarea completa).
> Usar MCP tools (`campaign_get_next_task`, `campaign_verify_cmd`, etc.) para estado.
> Al finalizar: recitation + STOP.

Las skills base (campaign-executor, progreso, ponytail) se cargan automáticamente al invocar `campaign_load_skills` (MCP) — no las cargues dos veces. Usá `campaign_load_skills` para obtener las skills específicas del tipo de tarea.

### Step 0: Auto-cargar skills vía MCP

1. Llamá `campaign_get_next_task` (MCP) para obtener la próxima tarea
2. Con los `Archivos clave` de la tarea, llamá `campaign_load_skills` (MCP) para obtener:
   - Tipo de tarea detectado
   - Lista de skills a cargar
   - Comandos de verificación
3. Cargá CADA skill devuelta con `skill <nombre>` (no te saltees ninguna)
4. Llamá `campaign_get_workflow` (MCP) con el tipo detectado para cargar el workflow JSON (bug-fix/feature-add/refactor/research/nine-second-saloon). El workflow define el template de fases (estados, instrucciones y transiciones por tipo de tarea) **como guía para el agente (classification-output)** — estos estados NO pasan por `campaign_enforce_state`. El enforcement de herramientas en runtime usa SIEMPRE la state machine genérica C0 (`STATE_TOOLS` en `config/state-tools.mjs`: PLAN/ACT/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE/STALL). No confundir los dos conjuntos de estados.
5. Si es bug → cargá `systematic-debugging` además
6. Si es lógica nueva/compleja → cargá `test-driven-development` además
7. Si es security-sensitive → cargá `doubt-driven-development` además

### Step 1: Leer estado actual

1. Llamá `campaign_get_next_task` (MCP) para obtener la tarea activa + recitation + resumen
   (lee el plan file indirectamente — no necesitás leerlo con Read tool)
2. Si la tarea existe y tiene recitation → la recitation dice el estado exacto
3. Si no hay recitation o la tarea es nueva → estado = ⬜ PENDING

### Step 2: Determinar próxima acción

| Estado del task file | Acción |
|----------------------|--------|
| No existe (⬜ PENDING) | **MODO DISCOVERY** — investigar, crear task file, arrancar primer step |
| Existe con steps ⬜ PENDING | **MODO EJECUCIÓN** — ejecutar próximo step, state machine PLAN→ACT→VERIFY |
| Existe con todos ✅ | **MODO CIERRE** — verificación full, commit, progreso |
| ❌ FAILED | Documentar, progreso, detenerse |

---

## MODO DISCOVERY

```
skill progreso

1. Auto-detectar tipo de tarea con MCP:
   Llamá `campaign_detect_task_type` (MCP) con los `Archivos clave` de la tarea.
   Devuelve: type, skills, checks, estimate.
   Cargá los skills devueltos con `skill <nombre>`.

2. Clasificar workflow:
   Llamá `campaign_classify_workflow` con taskName + descripción de la tarea.
   Devuelve el workflow template matching (bug-fix, feature-add, refactor, research).
   **Los estados del workflow son GUÍA de fases (classification-output), NO la
   state machine de enforcement**: el runtime siempre enforcea la C0 genérica
   (`STATE_TOOLS` en `config/state-tools.mjs`). Usá el workflow para saber qué
   fases aplicar y en qué orden; las transiciones y tools permitidas son C0.

3. Auto-estimar turns con `campaign_detect_task_type`:
   El MCP devuelve estimate: { turns, label }

4. codegraph_explore "símbolos/archivos de la tarea"

5. Zero-code planning: antes de escribir código o crear el task file, describí
   la solución en ≤3 viñetas de pseudocódigo. Sin tocar archivos todavía.
   Identificá: qué archivos cambiar, qué funciones modificar, qué firma tendrá,
   qué tests escribir. Si hay ambigüedad → web research antes de continuar.
   Validá que el enfoque es correcto antes de comprometerte.

5b. **Gate D** (`question-gates.md`): si blast radius >10 archivos / hot path /
    API pública / contrato ambiguo / feature-add sin spec → `question` al usuario
    (GO / ajustar / dividir) ANTES de escribir el task file.

6. Llamá `campaign_update_task_state` (MCP) con `"in-progress"` y recitation
   que apunte al próximo step.

7. Web research si hay ambigüedad (API externa, patrón no familiar):
   MetaSearchMCP.search_web("consulta") + Argus.extract_content(url)
   → Documentar en Investigation Notes del task file

8. Documentar en el task file:
   - CALLERS: qué módulos llaman
   - CALLEES: de qué depende
   - IMPLICACIONES: contratos, API, performance, migración
   - RIESGO: alto / medio / bajo
   - Contrato verificable (NO vago — ver tabla al final)
   - Herramientas necesarias (cargo-mcp, rust-analyzer-mcp, etc.)
   - Solución planeada (de step 5: zero-code planning)
   - Descomponer en steps atómicos (cada uno: archivo + acción + verify)

9. Escribir task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`
   Usando el template de `skills/campaign-executor/templates/task-definition.md`.
   Agregar last-synced en ambos archivos (plan + task).

10. Implementá el primer step (~100 líneas)
    Verificá con `campaign_verify_cmd` (comando del campo `contract`)
```

---

## MODO EJECUCIÓN — State Machine (C0)

Cada paso sigue esta state machine. **No se permite saltar estados.**
En cada estado, las tools permitidas/denegadas están definidas abajo.

```
Estados válidos (C0 — Statewright pattern):

  PLAN     → ACT
  ACT      → VERIFY
  VERIFY   → PLAN      (falló → reintentar)
  VERIFY   → STALL     (3 same-error → bloqueo)
  VERIFY   → COLLATERAL (pasó → errores colaterales)
  COLLATERAL → RESEARCH (ambigüedad → investigar)
  RESEARCH → ACT       (investigado → implementar)
  COLLATERAL → EVALUATE (sin errores → evaluar)
  EVALUATE → REVIEW    (auto-evaluación pasa → revisión)
  EVALUATE → ACT       (auto-evaluación falla → re-implementar)
  REVIEW   → VERIFY    (review encuentra issues → re-verificar)
  REVIEW   → ACCEPT    (review pasa → aceptar)
  ACCEPT   → CLOSE     (aceptado → commit)

Transiciones inválidas (NO permitidas):
  PLAN → EVALUATE      ❌ no implementado
  ACT  → ACCEPT        ❌ no verificado
  ACT  → CLOSE         ❌ no revisado
  ACT  → REVIEW        ❌ no evaluado
```

### Per-state tool enforcement

Antes de cada tool call, verificá con `campaign_validate_action`. Usá `campaign_get_state_allowed_tools` para saber qué tools están permitidas en cada estado.

| Estado | Tools permitidas | Tools denegadas |
|--------|-----------------|-----------------|
| PLAN   | read, grep, glob, codegraph_explore, campaign_*, skill, bash, websearch, webfetch, argus_*, metasearchmcp_* | edit, write, campaign_verify_cmd, cargo-mcp_*, rust-analyzer-mcp_* |
| ACT    | edit, write, bash, campaign_*, read, grep, glob, codegraph_explore, skill, cargo-mcp_*, rust-analyzer-mcp_* | (ninguna) |
| VERIFY | bash, campaign_verify_cmd, cargo-mcp_*, campaign_*, read, grep | edit, write |
| COLLATERAL | bash, read, grep, glob, codegraph_explore, campaign_* | edit, write |
| RESEARCH | read, grep, glob, codegraph_explore, websearch, webfetch, argus_*, metasearchmcp_*, campaign_*, **bash read-only** | edit, write (writes via bash bloqueados por classifyBashWrite) |
| EVALUATE | read, grep, codegraph_explore, campaign_* | edit, write, bash |
| REVIEW | read, grep, codegraph_explore, campaign_*, skill | edit, write, bash |
| ACCEPT | campaign_*, skill, read, bash | edit, write |
| CLOSE  | bash, campaign_*, skill, read | edit, write |
| STALL  | campaign_*, read | edit, write, bash, cargo-mcp_* |

> Fuente canónica de esta tabla: `config/state-tools.mjs` (el enforcement
> runtime usa SIEMPRE ese archivo). Si esta tabla diverge, manda el .mjs —
> corregir acá, no inventar una tercera versión.

Usá `campaign_validate_action state=<ESTADO> toolName=<TOOL>` para verificar antes de llamar tools que puedan estar en el límite. Si una tool está denegada, NO la llames — cambiá de estado primero vía `campaign_update_task_state`.

### PLAN
- Leer el próximo step del task file
- Consultar memoria: `campaign_memory_read lessons` y `decisions` para contexto
  - **Esquema fijo por línea** (TSYS-15; los campos van separados por ` | `):
    `- <fecha-auto> | <tema> | <decisión|lección> | ref: <ruta:línea>`
  - `campaign_memory_write` recibe SOLO `entry="<tema> | <decisión|lección> | ref: <ruta:línea>"` — **NO incluir la fecha**: el server la antepone (`- YYYY-MM-DD | `); duplicarla es la FALLA #11 (desync entre líneas)
  - Ej: `campaign_memory_write(file="decisions", entry="pyo3 | PyBytes owned en vez de raw pointer (AUDIT-01) | ref: vantadb-python/src/vector.rs:59")`
  - **Read por tema** (el server NO filtra): `rg -n "<tema>" .opencode/task-system/memory/*.md` (o Grep con pattern `^\- .*\| <tema> \|`)
- Decidir el cambio atómico (~100 líneas máx)
- Ponytail ladder: ya existe > stdlib > dependency > mínimo código

### ACT
- Editar archivos (preferir `edit` con oldString/newString sobre reescribir completos)
- Para comandos destructivos (rm, format, DDL, scripts generados):
  usar `campaign_run_sandboxed` para ejecutar en staging aislado

### VERIFY
- Comando mecánico real, nunca auto-reporte
- Rust: `cargo check -p <crate>`
- Web: `npx tsc --noEmit`
- Tests: `cargo nextest run <test_name>`
- Usar `campaign_verify_cmd command="..."` (MCP) — nunca auto-reporte

**Agente de Diagnóstico (si verify falla):**
No pasar el error crudo al implementador. Procesá el error del compilador/test/lint, identificá la causa raíz (archivo, línea, mensaje), y sintetizá una instrucción técnica precisa:
"El compilador falló en la línea 45: error de lifetime. Reestructurá la función para evitar devolver una referencia local."
Recién ahí → retry.

### MoM Ladder + Gate V

Umbral único de fallo mismo-error = **2**. Cada reintento sube un tier vía `campaign_mom_escalate`. No reintentes con el mismo modelo más de una vez.

| Falla | Acción |
|-------|--------|
| 1ª falla | corregir con feedback del error (Agente de Diagnóstico) + tier up (`campaign_mom_escalate`) |
| 2ª falla mismo error (archivo+línea+mensaje) | **Gate V** (`question-gates.md`): `question` al usuario — reintentar fresh / cambiar estrategia (deepseek-v4, tier 2) / ❌ FAILED |

Nunca marcar FAILED unilateralmente al agotar el umbral: sin respuesta del usuario → STOP.

### Stagnation Detection

Gate previo a errores colaterales. Umbral único de fallo mismo-error = **2** (igual que MoM ladder y SARL — un solo número en todo el sistema). Si ALGUNA condición se cumple → llamá `campaign_update_task_state` (MCP) con `"failed"`, anotá la causa en recitation, y detenete:
- ¿2 fallas de verify con el mismo error (archivo+línea+mensaje)? → FAILED
- ¿5+ iteraciones sin cambiar de step?
- ¿Mismos archivos tocados en últimas 3 iteraciones?

Usá `campaign_stalled_tasks` (MCP) para revisar tareas estancadas existentes.

### Fork/Join

Tareas independientes en paralelo vía sub-agentes. CIERRE steps que NO dependen entre sí → fork a sub-agentes:
- Grupo 1 (independiente): `cargo fmt --check`, `cargo machete`
- Grupo 2 (depende de build): `cargo nextest`, `cargo clippy`
- Usá `task` tool para spawn sub-agentes; join all antes de avanzar
- Máximo 3 sub-agentes simultáneos (RAM en Windows)
- Si un sub-agente de fork/join devuelve resultado INCOMPLETO, vacío o se detuvo solo →
  aplicá `prompts/subagent-recovery.md` (RESUME misma sesión con `task_id` → RETRY → STRATEGY → ESCALATE)

### MODO EJECUCIÓN — Flujo paso a paso

1. Continuá desde donde quedó (usá recitation de `campaign_get_next_task`)
2. State machine: PLAN → ACT → VERIFY
   - Antes de ACT → `campaign_validate_command` (MCP) para validar el comando
   - Si el comando es riesgoso (rm, format, dangerous) → `campaign_run_sandboxed` (MCP)
   - En cada transición de estado → `campaign_enforce_state` (MCP) para pre-call checks
3. Si verify falla: retry ladder (4 escalones MoM arriba)
4. Errores colaterales: rápido se arregla (<30min), lento se difiere a Backlog
5. Evaluator-Optimizer: correctitud, simplicidad, consistencia
6. Self-Harness Gate: propose → evaluate → accept
7. Pre-commit Gate: Definition of Done + checklists por tipo
8. Verificá con `campaign_verify_cmd`
9. Budget: máx 5 iteraciones por tarea, 2 stalls consecutivos → ❌ FAILED

---

## MODO CIERRE

```
1. Verificación full del contrato (fork/join — grupos independientes en paralelo):
   - Grupo 1 (inmediato, sin deps): cargo fmt --check, cargo machete
   - Build (dependencia): cargo build --workspace (o warm cache si Windows da error)
   - Grupo 2 (post-build): cargo nextest + cargo clippy, fork a sub-agentes
   - (si frontend) npx tsc --noEmit
   - Si el código contiene unsafe o concurrencia:
     Si nightly disponible: cargo +nightly miri test (UB detection)
     Marcar para ThreadSanitizer / AddressSanitizer en CI
   - Si el componente es crítico (parser, serializador, WAL):
     Marcar para fuzzing en CI
     Escribir test de propiedad básico (quickcheck/proptest)

2. Pivotaje cognitivo (auto-revisión):
   "Detené la implementación. Ahora asumí el rol de Ingeniero de Sistemas
   Senior ultra-crítico. Encontrá 1-3 problemas de seguridad, memoria,
   ineficiencia o errores lógicos ocultos en el código que acabas de
   escribir. Corregilos de inmediato."

3. Evaluator-optimizer: auto-crítica 3 ejes:
   a) CORRECTITUD: ¿edge cases cubiertos? ¿input vacío? ¿límites? ¿nulls?
      ¿colecciones vacías? ¿acceso concurrente?
   b) SIMPLICIDAD: revisar con ponytail ladder. ¿algo se puede acortar?
      ¿stdlib lo hace? ¿dependency ya instalada lo cubre?
   c) CONSISTENCIA: ¿sigue el mismo patrón que el código existente?
      ¿misma convención de nombres? ¿mismo estilo de error handling?
   codegraph_explore post-implement para verificar impacto completo.
   Máximo 2 iteraciones de evaluator-optimizer.

4. Errores colaterales (encontrados durante verify/review):
   Para cada error colateral:
     - Anotarlo
     - 🟢 RÁPIDO (<30min, mismo archivo): arreglar y commitear junto
     - 🟡 LENTO (>30min, módulo diferente): crear entrada en Backlog.md
     - NO perder foco de la tarea principal

5. Self-Harness Gate (propose → evaluate → accept):
   1. PROPOSE: leer git diff, resumir en 3 líneas: qué cambió, por qué,
      qué contrato cumple
   2. EVALUATE (5 condiciones booleanas):
      [ ] ¿SATISFACE el contrato? (sí/no — booleano, sin matices)
      [ ] ¿OUTPUT validado? (campaign_validate_output para shell/cmd/paths)
      [ ] ¿ROMPE algo fuera del blast radius? (codegraph_explore check)
      [ ] ¿INTRODUCE deuda técnica nueva? (ponytail-review)
      [ ] ¿ESTÁ documentado si cambió API pública?
   3. ACCEPT: todas ✅ → continuar
      REJECT: alguna ❌ → volver a EJECUCIÓN con lista de issues
   4. Si 2 rejections consecutivas → bloquear, escalar a humano

6. Pre-commit gate:
   [ ] Definition of Done aplicado (ver RULES.md §6)
   [ ] Security checklist (si toca datos/auth)
   [ ] Performance checklist (si es camino crítico)
   [ ] Testing checklist (si es lógica nueva)
   [ ] Ponytail ladder aplicada
   [ ] Tests pasan
   [ ] Documentación afectada actualizada

7. git add -p + git commit (solo archivos tocados en esta tarea — nunca `git add -A`)
   con mensaje Conventional Commits:
    tipo(scope): ID — descripción breve

    Blast radius: [módulos afectados]
    Skills: [skills usadas]
    Contrato: [condición cumplida]
    Errores colaterales: [ninguno | lista con destino]

8. skill progreso (Trigger 1)

9. Context Save Point: registrá decisiones y estado al final del task file:
    - Fecha, Branch, CI pendiente, Decisiones, Problemas conocidos, Próxima tarea

10. Auto-mejora (RULES.md §10): evaluá qué fue más difícil de lo esperado
    Llamá `campaign_diagnose_pipeline` (MCP) para diagnosticar performance
    y obtener sugerencias de mejora
```

---

## MODO FAILED

Anotá por qué falló y qué se intentó (los 4 escalones MoM si aplica), llamá `campaign_update_task_state` con `"failed"`, ejecutá `skill progreso`, y detenete. No sigas a la siguiente tarea.

---

## Step 3: Verificar con MCP

Usá `campaign_verify_cmd` (MCP) — nunca auto-reporte, el compilador/test runner/linter deciden:

```
campaign_verify_cmd command="cargo check -p vantadb"
campaign_verify_cmd command="cargo fmt --check"
campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"
campaign_verify_cmd command="cargo clippy --workspace --all-targets --all-features -- -D warnings"
```

Si verify falla → MoM ladder (4 escalones, ver arriba). Si pasa → continuar.

## Step 4: Actualizar estado vía MCP

Después de la acción, actualizá SIEMPRE con `campaign_update_task_state` (MCP):
- `"in-progress"` cuando arrancás un step (con recitation apuntando al próximo)
- `"completed"` cuando todo el step está verificado y commiteado
- `"failed"` cuando el retry ladder se agotó

La recitation se escribe automáticamente en el plan file por el MCP server.
No modificés el plan file manualmente — usá siempre el MCP tool.

**Task file (si existe):**
- Step marcado como ✅ o ❌
- `last-synced` actualizado

## Step 5: Recitation (handoff entre iteraciones)

Después de cada acción, llamá `campaign_update_task_state` (MCP) con recitation estructurada:

```
Objetivo activo: TASK-N — ID
lastAction: qué se acaba de hacer
result: ✅ / ❌
nextAction: el PRÓXIMO paso concreto (archivo + comando)
contract: "condición verificable"
nextTask: TASK-N+1 — ID
```

El MCP server escribe el bloque RECITATION en el plan file automáticamente.
Sin recitation, la próxima iteración arranca perdida.

## Step 6: STOP

No sigas a la siguiente tarea ni iteración.

---

## Apéndice: Tabla de contratos

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "npx tsc --noEmit 0 errors, npm run lint 0 errors" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |

---

## REGLAS (del campaign-executor RULES.md)

- Usá `campaign_get_next_task` (MCP) o leé el plan file directamente
- **Reglas de contexto:**
  - Código fuente → CodeGraph (determinista, 0 alucinación). NO leas archivos .rs/.ts/.py directamente.
  - Prosa no indexada (plan files, skills) → sub-agentes vía `task` tool
  - Context Budget: uso inicial < 20%. Si estás cerca del límite, usá sub-agentes.
  - Preferir `edit` con oldString/newString sobre reescribir completos
  - No cargar MCPs que no uses para esta tarea
  - No prosa defensiva — si hay un problema, expresalo modificando código
  - No cambiar scope — anotá hallazgos extra en Notas, no los implementes
- El contrato es ley — si el contrato no se cumple, la tarea no está completa
- Verificación mecánica, nunca auto-reporte
- ~100 líneas por paso, un paso por turno, cada paso reversible independientemente
- No cambies scope. Rápido se arregla (<30min), lento se anota en Backlog
- Stagnation = stop: 3 vueltas sin progreso → ❌ FAILED
- Budget: 5 iteraciones máximas por tarea
- La recitation es el handoff entre iteraciones — sé específico
- Después de actualizar, DETENETE. No sigas a la siguiente tarea ni iteración.
