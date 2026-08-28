# TASK-ID: Descripción

## Metadata
- **Plan file:** 
- **Fuente:** [backlog línea / plan file task N]
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🟡 2-3d
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Tipo:** Rust | Frontend | Python | TypeScript | Docs | Mixto
- **Turns estimados:** N
- **Creado:** YYYY-MM-DDTHH:MM
- **last-synced:** YYYY-MM-DDTHH:MM
- **Estado:** ⬜ PENDING | ⏳ IN PROGRESS | ✅ COMPLETED | ❌ FAILED
- **Incógnitas (uphill):** N abiertas (indicador independiente del % — debe bajar a 0 para ✅)
- **Pendientes (downhill):** N steps de ejecución restantes
- **Campaign ID:** <uuid>

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | módulo A, módulo B |
| Callees | módulo C, crate D |
| Implicaciones | contrato X no cambia, performance Y mejora |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** [paths]
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** [grep]
- **Archivos que referencian a los editados (referencias entrantes):** [grep por nombre del archivo]
- **Veredicto impacto:** [bajo/medio/alto — qué se rompe si cambio/elimino]

## Contrato
"cargo nextest run --profile audit --workspace --build-jobs 2 pasa y el comportamiento específico es [condición]"

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)

> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`".
> Tabla de decisiones (spec-template §5) O justificación por evidencia por ítem.
> `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | [decisión técnica abierta] | A (tradeoff) / B (tradeoff) | A | ✅ pregunta / ✅ decidido-por-evidencia (ref: archivo:línea) |

## Invariantes de dominio (handoff — MUST)

> El task file debe declarar qué NO se puede romper al continuar, con qué
> comando se verifica y qué queda incompleto. Sin esto, el próximo agente
> arranca sin contexto (gap-01 §3.3-18, eng-03-project.md:198).

- **Invariantes a preservar:** [qué condición de dominio/seguridad no puede violar el próximo agente]
- **Comandos de verificación:** [comando exacto + resultado esperado, p.ej. `cargo nextest run --profile audit --workspace --build-jobs 2`]
- **Deuda pendiente:** [lo que queda incompleto al cerrar esta iteración, o "ninguna"]

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# TASK-ID: Descripción` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS con steps pendientes · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | Siguiente tarea del plan file |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Deuda registrada (≤0, justificada en Notas) | Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda
> existente — el saldo neto por PR es 0 o negativo. Si hay deuda nueva,
> completar el campo `Deuda registrada` con el ID de la deuda y su moneda de
> pago (ver tabla P2 en AGENTS.md).

## Definition of Done (contrato multi-nivel — P2-08)

El DoD es **contrato**, no checklist decorativo. La calidad mínima de pie está
en `.opencode/references/definition-of-done.md` y aplica SIEMPRE. Además, el
task se evalúa por nivel:

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable del task file se cumple + capa determinista (fmt, clippy, nextest) + tests del cambio pasan |
| **Commit** | Commit atómico (~100 líneas), conventional commit, `git diff` limpio, verificación mecánica (nunca auto-reporte) |
| **Release** | `dev-tools/verify.ps1` completo (6 pasos), changelog, semver respetado, pre-push gate (Regla 1) |

**Gate:** el task se marca COMPLETED solo si pasan los tres niveles
aplicables a la tarea. Si un nivel no aplica (p.ej. tarea docs sin release),
justificar en Notas.

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt)
- rust-analyzer-mcp (diagnostics, goto def)
- codegraph_explore (blast radius)
- codebase-memory-mcp_detect_changes (blast radius transitivo, impacto de cambios — ANTES de commit)
- codebase-memory-mcp_get_architecture (overview, clusters, hotspots, boundaries)
- codebase-memory-mcp_query_graph (Cypher: complejidad, ciclos, hot paths)
- codebase-memory-mcp_search_graph (semantic search, bridge vocabulary)
- codebase-memory-mcp_trace_path (calls/data_flow/cross_service con risk_labels)
- codebase-memory-mcp_check_index_coverage (verifica cobertura del índice en archivos a tocar)
- codebase-memory-mcp_index_status (health check, parse_partial/skipped files)

**Skills cargadas (SDP):** [lista + justificación 1 línea cada una, o `base-only + SDP sin candidatos`]

## Investigation Notes
- Hallazgos de web research, si aplica
- Formato estándar por hallazgo:
  - **Claim:** [afirmación concreta]
  - **Evidencia:** [URL | file path | tool result]
  - **Confianza:** alta | media | baja

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

Dos ejes distintos del % de completado. El % mide ejecución; las incógnitas
miden certidumbre. El estado reporta los tres por separado:

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | N — qué NO se sabe todavía: approach a validar, dependencia a investigar, decisión abierta |
| Pendientes de ejecución (downhill) | N — steps de ejecución restantes (trabajo conocido) |
| % completado | N% |

**Regla de reporting:** cada actualización de estado actualiza los tres
contadores. Una incógnita resuelta se mueve de Incógnitas → Notas con la
respuesta. Una tarea con incógnitas abiertas NO se marca ✅ aunque el % sea
100%.

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

> Obligatoria para tareas tipo Bug (`fix:`). El fix requiere método correcto,
> no solo test verde: **Iron Law** de systematic-debugging — sin investigación
> de causa raíz no hay fix. Sin esta sección poblada, NO se escribe ni se
> ejecuta el step de fix.

- **Repro:** reproducción determinística del bug (pasos exactos / comando)
- **Hipótesis:** causa raíz probable, escrita ANTES de tocar código
- **1 variable controlada:** exactamente UNA variable cambiada por intento
- **Test RED:** test de regresión que reproduce el bug — verificado como FALLO
  (RED) antes del fix; pasa a GREEN solo con el fix aplicado

**Gate:** los steps de fix y sus Verify se definen solo DESPUÉS de completar
esta sección con `repro`, `hipótesis`, `1 variable controlada` y `test RED`.
Grafías aceptadas del campo: `hipótesis|hipotesis`.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

Evaluación mandatoria ANTES de codear. Si no aplica, justificar en Notas:

- [ ] **SECURITY** — si toca trust boundaries, input de usuario, auth, datos,
      o agrega/quita dependencias → cargar `security-and-hardening` y
      documentar hallazgos en Notas. Si no aplica, justificar por qué.
- [ ] **PERFORMANCE** — si toca un hot path (búsqueda, indexación,
      serialización, loops calientes) → cargar `performance-optimization` y
      registrar baseline/impacto esperado. Si no aplica, justificar.

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

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la
> tarea no está COMPLETED.

- **Revisor:** [vanta-audit | vanta-review | review-deep | doubt-driven-development]
- **Enfoque:** [¿el approach es correcto? ¿alternativas mejores?]
- **Cómo se probó:** [evidencia de verificación real, no auto-reporte]
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor
  verifica que el implementador NO haya incurrido en ninguno antes de aprobar;
  fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve | ❌ cambios requeridos (volver a Steps)

## Notas
- Decisiones de diseño, contexto aprendido, problemas conocidos

## Context Save Point
- **Fecha:** ISO
- **Branch:** nombre
- **CI pendiente:** sí/no
- **Decisiones:** X sobre Y porque [razón breve]
- **Problemas conocidos:** [ninguno | lista]
- **Próxima tarea:** TASK-N+1