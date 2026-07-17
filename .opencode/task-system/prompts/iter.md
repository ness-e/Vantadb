> **ACTIVE INSTRUCTION — One Harness Iteration**
> Cargado por el harness loop (legacy) cuando hay una tarea ⬜ PENDING o ⏳ IN PROGRESS.
> Path resolution: `skills/X` → `.opencode/skills/X/`
> Procesar EXACTAMENTE UNA iteración (no una tarea completa).
> Usar MCP tools (`campaign_get_next_task`, `campaign_verify_cmd`, etc.) para estado.
> Al finalizar: recitation + STOP. El harness externo maneja el loop.

Cargá las skills campaign-executor, progreso, ponytail (full). Después de determinar la tarea activa, usá `campaign_load_skills` (MCP) para cargar skills específicas del tipo de tarea.

Plan file: {{PLAN_FILE}}
Single task: {{SINGLE_TASK}} (vacío = todas las tareas)
{{SUMMARY}}

## INSTRUCCIONES — UNA SOLA ITERACIÓN

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

**Reglas de contexto:**
- **Map-Reduce determinista:** código fuente → CodeGraph (determinista, 0 tokens,
  0 alucinación). `codegraph_explore` devuelve source verbatim + call paths + blast
  radius en una llamada. NO leas archivos .rs, .ts, .py directamente como contexto.
  Prosa no indexada (plan files, skills) → sub-agentes via `task` tool. Cada
  sub-agente lee un archivo y devuelve resumen enfocado en 3-5 líneas. Stale cache:
  si codegraph muestra "edited since last sync", leé SOLO esos archivos.
- **Context Budget:** uso inicial < 20% del contexto (~40k tokens en 200k). Si estás cerca del límite, usá sub-agentes para tareas largas. No leas archivos completos si solo necesitás 5 líneas — usá `grep` o `codegraph_explore`.
- Preferir `edit` con oldString/newString sobre reescribir archivos completos
- No cargar MCPs que no uses para esta tarea
- **No prosa defensiva:** no expliques código ni justifiques decisiones en comentarios.
  Si hay un problema, expresalo modificando código o tests.
- **No cambiar scope:** si encontrás algo extra (bug no relacionado, feature faltante),
  anotalo en Notas del task file, no lo implementes
- **Skills de ingeniería:** cargá los skills que devuelva `campaign_load_skills` según
  el tipo de tarea detectado. No saltees pasos (ver agent-skills lifecycle en AGENTS.md).

### Paso 0: Auto-cargar skills vía MCP

1. Llamá `campaign_get_next_task` (MCP) para obtener la próxima tarea
2. Con los `Archivos clave` de la tarea, llamá `campaign_load_skills` (MCP) para obtener:
   - Tipo de tarea detectado
   - Lista de skills a cargar
   - Comandos de verificación
3. Cargá CADA skill devuelta con `skill <nombre>` (no te saltees ninguna)
4. Si es bug → cargá `systematic-debugging` además
5. Si es lógica nueva/compleja → cargá `test-driven-development` además
6. Si es security-sensitive → cargá `doubt-driven-development` además

### Paso 1: Leer estado actual

1. Llamá `campaign_get_next_task` (MCP) para obtener la tarea activa + recitation + resumen
   (lee el plan file indirectamente — no necesitás leerlo con Read tool)
2. Si la tarea existe y tiene recitation → la recitation dice el estado exacto
3. Si no hay recitation o la tarea es nueva → estado = ⬜ PENDING
4. Si `{{SINGLE_TASK}}` no está vacío → usá `campaign_get_task_detail` (MCP) para
   ver el bloque completo de esa tarea específica

### Paso 2: Determinar próxima acción

| Si el task file... | Entonces... |
|-------------------|-------------|
| No existe | **MODO DISCOVERY:** codegraph_explore → blast radius → detectar tipo de tarea → crear task file con steps atómicos. Actualizá plan file: Estado → ⏳ IN PROGRESS. |
| Existe y tiene steps ⬜ PENDING | **MODO EJECUCIÓN:** ejecutá el próximo step pendiente del task file. State machine: PLAN → ACT → VERIFY. |
| Existe y todos los steps ✅ | **MODO CIERRE:** verificación full → evaluator-optimizer → self-harness gate → pre-commit → commit → skill progreso. State machine: REVIEW → ACCEPT → CLOSE. |

### Paso 3: Ejecutar (MODO DISCOVERY)

```
skill progreso

1. Auto-detectar tipo de tarea con MCP:
   Llamá `campaign_detect_task_type` (MCP) con los `Archivos clave` de la tarea.
   Devuelve: type, skills, checks, estimate.
   Cargá los skills devueltos con `skill <nombre>`.

2. Clasificar workflow:
   Llamá `campaign_classify_workflow` con taskName + descripción de la tarea.
   Devuelve el workflow template matching (bug-fix, feature-add, refactor, research).
   Usá los estados del workflow como state machine específica para esta tarea.
   Si no hay template matching → usar C0 genérica de iter.md como fallback.

3. Auto-estimar turns con `campaign_detect_task_type`:
   El MCP devuelve estimate: { turns, label }

4. codegraph_explore "símbolos/archivos de la tarea"

5. Zero-code planning: antes de escribir código o crear el task file, describí
   la solución en ≤3 viñetas de pseudocódigo. Sin tocar archivos todavía.
   Identificá: qué archivos cambiar, qué funciones modificar, qué firma tendrá,
   qué tests escribir. Si hay ambigüedad → web research antes de continuar.
   Validá que el enfoque es correcto antes de comprometerte.

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
   - Contrato verificable (NO vago — ver tabla abajo)
   - Herramientas necesarias (cargo-mcp, rust-analyzer-mcp, etc.)
   - Solución planeada (de step 5: zero-code planning)
   - Descomponer en steps atómicos (cada uno: archivo + acción + verify)

9. Escribir task file en {{TASK_BASE}}<ID>.md
   Agregar last-synced en ambos archivos (plan + task).
```

### Paso 3: Ejecutar (MODO EJECUCIÓN) — State Machine

Cada paso sigue esta state machine. No se permite saltar estados.
En cada estado, consultá `campaign_get_state_allowed_tools` para saber qué tools
están permitidas. No uses tools denegadas para el estado actual.

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

Per-state tool enforcement — antes de cada tool call, verificá con `campaign_validate_action`:

  | Estado | Tools permitidas | Tools denegadas |
  |--------|-----------------|-----------------|
  | PLAN   | read, grep, glob, codegraph_explore, campaign_*, skill, bash, websearch, webfetch, argus_*, metasearchmcp_* | edit, write, campaign_verify_cmd, cargo-mcp_*, rust-analyzer-mcp_* |
  | ACT    | edit, write, bash, campaign_*, read, grep, glob, codegraph_explore, skill, cargo-mcp_*, rust-analyzer-mcp_* | (ninguna) |
  | VERIFY | bash, campaign_verify_cmd, cargo-mcp_*, campaign_*, read, grep | edit, write |
  | COLLATERAL | bash, read, grep, glob, codegraph_explore, campaign_* | edit, write |
  | RESEARCH | read, grep, glob, codegraph_explore, websearch, webfetch, argus_*, metasearchmcp_*, campaign_* | edit, write, bash |
  | EVALUATE | read, grep, codegraph_explore, campaign_* | edit, write, bash |
  | REVIEW | read, grep, codegraph_explore, campaign_*, skill | edit, write, bash |
  | ACCEPT | campaign_*, skill, read, bash | edit, write |
  | CLOSE  | bash, campaign_*, skill, read | edit, write |
  | STALL  | campaign_*, read | edit, write, bash, cargo-mcp_* |

  Usá `campaign_validate_action state=<ESTADO> toolName=<TOOL>` para verificar
  antes de llamar tools que puedan estar en el límite. Si una tool está denegada,
  NO la llames — cambiá de estado primero vía `campaign_update_task_state`.

```

```
PLAN:
  - Leer el próximo step del task file
  - Consultar memoria: `campaign_memory_read lessons` y `decisions` para contexto
  - Decidir el cambio atómico (~100 líneas máx)
  - Ponytail ladder: ya existe > stdlib > dependency > mínimo código

ACT:
  - Editar archivos (preferir edit con oldString/newString)
  - Para comandos destructivos (rm, format, DDL, scripts generados):
    usar `campaign_run_sandboxed` para ejecutar en staging aislado

VERIFY:
  - Comando mecánico real, nunca auto-reporte
  - Rust: cargo check -p <crate>
  - Web: npx tsc --noEmit
  - Tests: cargo nextest run <test_name>

  Agente de Diagnóstico (si verify falla):
    No pasar el error crudo al implementador. Procesá el error del compilador/
    test/lint, identificá la causa raíz (archivo, línea, mensaje), y sintetizá
    una instrucción técnica precisa:
    "El compilador falló en la línea 45: error de lifetime. Reestructurá la
    función para evitar devolver una referencia local."
    Recién ahí → retry.

MoM ladder — cada escalón usa un modelo más potente (cambia vía `campaign_mom_escalate`):

  | Escalón | Modelo | Costo | Acción |
  |---------|--------|-------|--------|
  | 1ª falla | haiku (tier 0) | low | corregir con feedback del error (Agente de Diagnóstico) |
  | 2ª falla mismo error | sonnet/gpt-4o (tier 1) | medium | contexto fresco + resumen ~200 tokens |
  | 3ª falla mismo error | deepseek-v4 (tier 2) | high | estrategia materialmente distinta |
  | 4ª falla mismo error | humano (tier 3) | human | documentar intentos, commit WIP, ❌ FAILED |

  En cada falla: llamá `campaign_mom_escalate` con `currentModel` + `retryCount` para
  obtener el próximo modelo. No reintentes con el mismo modelo más de una vez.

Stagnation Detection (gate previo a errores colaterales):
  - ¿3+ iteraciones con el mismo error?
  - ¿5+ iteraciones sin cambiar de step?
  - ¿Mismos archivos tocados en últimas 3 iteraciones?
  Si ALGUNA → llamá `campaign_update_task_state` (MCP) con `"failed"`, anotá
  la causa en recitation, y detenete. Si necesitás revisar tareas estancadas
  usá `campaign_stalled_tasks` (MCP).

Fork/Join — tareas independientes en paralelo vía sub-agentes:
  - CIERRE steps que NO dependen entre sí → fork a sub-agentes
  - Grupo 1 (independiente): cargo fmt --check, cargo machete
  - Grupo 2 (depende de build): cargo nextest, cargo clippy
  - Usá `task` tool para spawn sub-agentes; join all antes de avanzar
  - Máximo 3 sub-agentes simultáneos (RAM en Windows)
  ```

### Paso 3: Ejecutar (MODO CIERRE)

```
1. Verificación full del contrato (fork/join — correr grupos independientes en paralelo):
   - **Grupo 1 (inmediato, sin deps):** cargo fmt --check, cargo machete
   - **Build (dependencia):** cargo build --workspace (o warm cache si Windows da page file error)
   - **Grupo 2 (post-build):** cargo nextest + cargo clippy, fork a sub-agentes
   - (si frontend) npx tsc --noEmit
   - Si el código contiene `unsafe` o concurrencia:
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
   Máximo 2 iteraciones de evaluator-optimizer. Si en la 3ra sigue sin
   pasar → bloquear.

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
      [ ] ¿OUTPUT validado? (campaign_validate_output para shell/cmd/paths antes de write)
      [ ] ¿ROMPE algo fuera del blast radius? (codegraph_explore check)
      [ ] ¿INTRODUCE deuda técnica nueva? (ponytail-review)
      [ ] ¿ESTÁ documentado si cambió API pública?
   3. ACCEPT: todas ✅ → continuar
      REJECT: alguna ❌ → volver a MODO EJECUCIÓN con lista de issues
   4. Si 2 rejections consecutivas → bloquear, escalar a humano

6. Pre-commit gate:
   [ ] Definition of Done aplicado (DoD v1 baseline — ver RULES.md §6)
   [ ] Security checklist (si toca datos/auth) — ver security-check-list.md
   [ ] Performance checklist (si es camino crítico) — ver performance-checklist.md
   [ ] Testing checklist (si es lógica nueva) — ver testing-patterns.md
   [ ] Ponytail ladder aplicada
   [ ] Tests pasan
   [ ] Documentación afectada actualizada

7. git add -p + git commit con mensaje Conventional Commits:
    tipo(scope): ID — descripción breve

    Blast radius: [módulos afectados]
    Skills: [skills usadas]
    Contrato: [condición cumplida]
    Errores colaterales: [ninguno | lista con destino]

8. skill progreso (Trigger 1)

9. Context Save Point: registrá decisiones y estado en la sección ## Context Save
   del task file (al final, después de ## Notas):
   - **Fecha:** ISO
   - **Branch:** nombre
   - **CI pendiente:** sí/no
   - **Decisiones:** X sobre Y porque [razón breve]
   - **Problemas conocidos:** [ninguno | lista]
   - **Próxima tarea:** TASK-N+1
```

### Paso 4: Verificar con MCP

Cada verify debe usar `campaign_verify_cmd` (MCP) — nunca auto-reporte:

```
campaign_verify_cmd command="cargo check -p vantadb"
campaign_verify_cmd command="cargo fmt --check"
campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"
campaign_verify_cmd command="cargo clippy --workspace --all-targets --all-features -- -D warnings"
```

Si verify falla → retry ladder (4 escalones, ver arriba).
Si verify pasa → continuar.

### Paso 5: Actualizar estado vía MCP

Después de la acción, actualizá SIEMPRE con `campaign_update_task_state` (MCP):

- `"in-progress"` cuando arrancás un step (con recitation apuntando al próximo)
- `"completed"` cuando todo el step está verificado y commiteado
- `"failed"` cuando el retry ladder se agotó

La recitation se escribe automáticamente en el plan file por el MCP server.
No modificés el plan file manualmente — usá siempre el MCP tool.

**Task file (si existe):**
- Step marcado como ✅ o ❌
- `last-synced` actualizado

### Paso 6: Recitation (handoff entre iteraciones)

Después de cada acción, llamá `campaign_update_task_state` (MCP) con recitation:

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

### Paso 6: STOP

No sigas a la siguiente tarea ni iteración.

### Apéndice: Tabla de contratos

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "npx tsc --noEmit 0 errors, npm run lint 0 errors" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |
