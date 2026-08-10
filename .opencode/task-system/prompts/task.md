> **ACTIVE INSTRUCTION — Define Task**
> Cargado por `commands/pipeline.md` (modo TASK) cuando el task file no existe.
> Path resolution: skills por nombre → `.opencode/skills/<nombre>/`
> Ejecutar las 4 fases SECUENCIALMENTE. No saltar pasos.
> Al finalizar: crear task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`,
> actualizar plan file, mostrar próximo comando recomendado.

Cargá las skills source-driven-development, progreso, ponytail (full).
Si la tarea toca API pública → cargá api-and-interface-design.

Task file target: {{TASK_PATH}}
Plan file: {{PLAN_FILE}}
Backlog: {{BACKLOG_PATH}}

## INSTRUCCIONES — DEFINIR TAREA A PROFUNDIDAD

Esta es una tarea individual. Investigá, definí y escribí el task file completo.

### Phase 1: Auto-detectar tipo de tarea

Según los archivos involucrados:

| Archivos | Tipo | Skills a cargar | Checks |
|----------|------|-----------------|--------|
| `src/**` (Rust core) | Rust | source-driven-development, doubt-driven-development | cargo check, nextest, fmt, clippy |
| `web/src/**` | Frontend | frontend-ui-engineering | npx tsc --noEmit, npm run lint |
| `vantadb-python/**` | Python SDK | source-driven-development | pytest -v |
| `vantadb-ts/**` | TypeScript SDK | source-driven-development | npx tsc, npm test |
| `docs/**` | Documentation | writing-guidelines, writing-plans | scripts/validate-docs-coverage |
| `*.md` (plan/backlog) | Planning | writing-plans, planning-and-task-breakdown | — |
| Mixto | Multiple | TODOS aplicables | TODOS los checks |

Si hay archivos de múltiples tipos, cargar skills de todos los tipos aplicables.

> **¿Tipo Bug (`fix:`)?** Cargá systematic-debugging (Iron Law: no hay fixes
> sin investigación de causa raíz primero) y exigí en el task file la sección
> "Fase 1 — Evidencia de Debugging" (ver formato) ANTES de escribir el fix.

### Phase 2: Discovery + Blast Radius

```
codegraph_explore "IDs, archivos, símbolos de la tarea"

Documentar en el task file:
- CALLERS: qué módulos llaman a estos archivos
- CALLEES: de qué dependen estos archivos
- IMPLICACIONES:
  · ¿Se rompen contratos existentes?
  · ¿Cambia comportamiento público (API, CLI, SDK)?
  · ¿Afecta performance, memoria, serialización?
  · ¿Requiere migración de datos o re-indexación?
  · ¿Afecta tests existentes?
- RIESGO: alto / medio / bajo
- CONTRATO: "completado = [condición verificable por comando]"
  (NO usar contratos vagos — ver tabla al final)
```

### Phase 3: Web research (si hay ambigüedad)

Si la tarea involucra APIs/librerías externas cuya doc no está en el código,
patrones de diseño no familiares, o decisiones técnicas con múltiples enfoques:

```
MetaSearchMCP.search_web("patrón o API específica")
Argus.extract_content(url_del_resultado)
→ Documentar en Investigation Notes del task file
```

### Phase 4: Descomponer en pasos atómicos

Cada paso debe ser:
- **Una sola acción** (editar un archivo, ejecutar un comando, correr un test)
- **≤100 líneas de código** por paso
- **Verificable mecánicamente** (cargo check, nextest, tsc, etc.)

Auto-estimar turns totales:

| Esfuerzo | Turns estimados |
|----------|----------------|
| 🟢 Bajo (1h) | 5-10 |
| 🟡 Medio (1d) | 15-30 |
| 🔴 Alto (2-3d) | 30-60 |

### Formato del task file

```markdown
# TASK-ID: Descripción

## Metadata
- **Plan file:** [ruta al plan file]
- **Fuente:** [backlog línea / plan file task N]
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🔴 2-3d
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Tipo:** Rust | Frontend | Python | TypeScript | Docs | Mixto
- **Turns estimados:** N
- **Creado:** YYYY-MM-DDTHH:MM
- **last-synced:** YYYY-MM-DDTHH:MM
- **Estado:** ⬜ PENDING | ⏳ IN PROGRESS | ✅ COMPLETED | ❌ FAILED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | módulo A, módulo B |
| Callees | módulo C, crate D |
| Implicaciones | contrato X no cambia, performance Y mejora |

## Contrato
"cargo nextest run --profile audit --workspace --build-jobs 2 pasa y el comportamiento específico es [condición]"

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt)
- rust-analyzer-mcp (diagnostics, goto def)
- codegraph_explore (blast radius)

## Investigation Notes
- Hallazgos de web research, si aplica

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

> Obligatoria para tareas tipo Bug (`fix:`). El fix requiere método correcto,
> no solo test verde: **Iron Law** de systematic-debugging — sin investigación
> de causa raíz no hay fix. Sin esta sección poblada, NO se escribe ni se
> ejecuta el step de fix.

- **Repro:** reproducción determinística del bug (pasos exactos / comando)
- **Hipótesis:** causa raíz probable, escrita ANTES de tocar código
- **1 variable controlada:** exactamente UNA variable cambiada por intento

**Gate:** los steps de fix y sus Verify se definen solo DESPUÉS de completar
esta sección con `repro`, `hipótesis` y `1 variable controlada`.
Grafías aceptadas del campo: `hipótesis|hipotesis`.

## Steps

### Step 1: [Nombre corto]
- **Archivos:** `path/to/file.rs`
- **Acción:** describir qué hacer
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: [Nombre corto]
- **Archivos:** `path/to/file.rs`
- **Acción:** describir qué hacer
- **Verify:** `cargo nextest run test_xxx`
- **Estado:** ⬜ PENDING

## Dependencias
- Task N-1: [ID] — [descripción] (debe completarse antes)

## Notas
- Decisiones de diseño, contexto aprendido, problemas conocidos
```

### Apéndice: Contrato vago vs verificable

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "cd web && npx tsc --noEmit 0 errors, npm run lint 0 errors, npm run build éxito" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |
| "Funciona bien" | "cargo build && cargo nextest run pasa, y [comportamiento específico] funciona" |
