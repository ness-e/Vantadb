# Backlog Campaign Pipeline

Pipeline completo para ejecutar campañas de tareas desde backlog en loop automático.

## Arquitectura

```
/USER                         /AGENT                        /LOOP PLUGIN
┌──────────────────┐         ┌──────────────────────┐      ┌──────────────────┐
│ /campaign        │──────▶  │ Leer backlog          │      │ .opencode/task-system/harness/harness-executor.ps1 │
│ docs/Backlog.md  │         │ Triage gate           │      │ monitorea idle   │
│                  │         │ Crear plan file       │      │ re-inyecta cada  │
│                  │◀──────  │ Mostrar comando loop  │      │ vez que para     │
└──────────────────┘         └──────────────────────┘      └──────────────────┘
                                      │                              │
                                      ▼                              │
                             ┌──────────────────────┐                │
                             │ hybrid-prompt.md     │◀────────────────┘
                             │                      │
                             │ Por turno:           │
                             │ 1. Buscar plan file  │
                             │ 2. Leer recitation   │
                             │ 3. Una acción atómica│
                             │ 4. Recitation + stop │
                             └──────────────────────┘
```

## Skills involucrados

| Skill | Rol | Cuándo |
|-------|-----|--------|
| `backlog-executor` | Recitation block, iteration harness, plan file format | Cada turno del loop |
| `task-executor` | Discovery → Definition → Execution phases, evaluator-optimizer, self-harness gate | Cada tarea nueva |
| `ponytail (full)` | Escalera: ya existe > stdlib > dependency > mínimo código | Siempre |
| `progreso` | Migrar tareas completadas de Backlog.md a docs/progreso/ | Al completar cada tarea |
| `incremental-implementation` | Pasos atómicos de ~100 líneas | Fase de implementación |
| `source-driven-development` | Código Rust con docs oficiales | Si la tarea es Rust |
| `writing-plans` | Descomponer tareas multi-paso | Al planificar |
| `doubt-driven-development` | Verificación adversarial en contexto fresco | Código sensible (stakes altos) |
| `code-review-and-quality` | Review multi-eje antes de commit | Pre-commit gate |

## Pipeline híbrido (agente único, 200k contexto)

### Map-Reduce determinista

| Qué | Cómo | Costo |
|-----|------|-------|
| Código fuente (.rs, .ts, .py) | `codegraph_explore` (AST index, sub-ms) | 0 tokens, 0 alucinación |
| Prosa no indexada (plan files, skills) | Sub-agentes via task tool, resumen 3-5 líneas | Solo lo necesario |

### Capa determinista (barrera infranqueable)

- `cargo clippy --all-targets -- -D warnings` — cero advertencias
- `cargo nextest run --profile audit`
- `cargo fmt --check`
- Si el código usa `unsafe` o concurrencia: `cargo +nightly miri test` (UB detection)
- Si el componente es crítico (parser, serializador, WAL): fuzzing + proptest

### Reglas Rust (motor DB)

- `unsafe` prohibido por defecto. Si necesario: safety invariant documentado en `// SAFETY: ...`
- `Rc<T>` prohibido en multi-hilo. Siempre `Arc<T>`.

### Control de contexto

| Regla | Detalle |
|-------|---------|
| Uso inicial | < 20% (~40k tokens) |
| Parches | `edit` con oldString/newString, no reescribir archivos completos |
| Sesiones | Una tarea completa + commitea → sesión cerrada. Siguiente arranca limpia |
| MCPs | No cargar Argus/Discord/Pencil si no aplican a la tarea |

### Ciclo interno por iteración

```
1. Buscar plan file más reciente en docs/plans/ con ⬜ PENDING
2. Leer recitation block (o primera tarea si no hay)
3. Según estado:
   a. ⬜ PENDING sin task definition → discovery + definition + task/<ID>.md
   b. ⏳ IN PROGRESS → implementar un paso atómico (PLAN → ACT → VERIFY)
   c. Verify pasa → capa determinista + pivotaje cognitivo + evaluator-optimizer
   d. Gate pasa → commit + progreso + actualizar plan file
4. Escribir recitation block
5. DETENER
```

### Auto-revisión (pivotaje cognitivo)

Antes de evaluar, inyectar: *"Detené la implementación. Ahora asumí el rol de Ingeniero de Sistemas Senior ultra-crítico. Encontrá 1-3 problemas de seguridad, memoria, ineficiencia o errores lógicos ocultos en el código que acabas de escribir. Corregilos de inmediato."*

### Agente de diagnóstico (verify falla)

Si verify falla, no pasar el error crudo al implementador. Procesar: leer el error, identificar causa raíz (archivo, línea, mensaje), y devolver instrucción técnica precisa:

> "El compilador falló en la línea 45: error de lifetime. Reestructurá la función para evitar devolver una referencia local."

### Retry ladder

| Escalón | Acción |
|---------|--------|
| 1 | Mismo enfoque + feedback específico del error |
| 2 | Si mismo error (archivo+línea+mensaje) 2 veces → ❌ FAILED |
| Stagnation | 3 intentos mismo error → ❌ FAILED |

## Comandos

### Crear plan desde backlog

```
/campaign                            # usa docs/Backlog.md
/campaign docs/mi-backlog.md         # backlog custom
```

Aplica triage gate, crea `docs/plans/<FECHA>-<nombre>.md`, muestra el comando de loop.

### Ejecutar campaña

```
.opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\<plan>.md -Interval 10
```

El harness invoca `opencode run` con el prompt completo de `hybrid-prompt.md`, espera a que termine, y recién ahí avanza a la siguiente iteración.

### Revisar estado del loop

```
/loop-goal
```

Muestra turno actual, si está activo/pausado/bloqueado.

## Archivos clave

| Archivo | Propósito |
|---------|-----------|
| `.opencode/skills/backlog-executor/hybrid-prompt.md` | Pipeline híbrido (prompt principal del loop) |
| `.opencode/skills/backlog-executor/SKILL.md` | Full spec del backlog-executor (Prompt 0, 1, harness) |
| `.opencode/skills/task-executor/SKILL.md` | Full spec del task-executor (fases 0-6) |
| `.opencode/skills/task-executor/VISION.md` | North star del executor (principios invariantes) |
| `.opencode/commands/campaign.md` | Custom command `/campaign` |
| `docs/plans/*.md` | Plan files (creados por triage gate) |
| `.opencode/skills/task-executor/tasks/*.md` | Task definitions (generadas por tarea) |

## Referencias

- Loop plugin: `@bybrawe/opencode-loop` (v0.5.15+, `npx -y @bybrawe/opencode-loop@latest`)
- Agent skills: addyosmani/agent-skills (`.opencode/skills/`)
- OpenCode custom commands: https://opencode.ai/docs/commands/
