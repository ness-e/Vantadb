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

**Skill Discovery (SDP — obligatorio, fuente canónica:
`.opencode/references/skills-engineering.md` §"Skill Discovery Protocol"):**
la tabla de arriba es solo la base. Macheá la fase de trabajo contra la tabla
Lifecycle mapping (skills-engineering.md) y grepeá `SKILLS-MANIFEST.md` por
keywords del título/contrato → cargá las candidatas relevantes (≤8 total,
justificadas en 1 línea) y listalas en "Herramientas necesarias" del task file.
Si no hay candidatas nuevas, registrá `SDP: sin candidatos adicionales`.

> **¿Tipo Bug (`fix:`)?** Cargá systematic-debugging (Iron Law: no hay fixes
> sin investigación de causa raíz primero) y exigí en el task file la sección
> "Fase 1 — Evidencia de Debugging" (ver formato) ANTES de escribir el fix.

### Phase 1b: Detección de feature-add (SDD — Gate P/D)

La etiqueta de tipo auto-detectada NO decide si la tarea es feature-add.
Después del discovery, verificá mecánicamente si la solución planeada
**agrega símbolos/contratos públicos nuevos**:

| Señal (cualquiera aplica) | Ejemplos |
|---------------------------|----------|
| `pub fn`/struct/enum público nuevo o firma pública nueva | SDK, engine, storage |
| Tool MCP nueva / endpoint HTTP/CLI nuevo | tools.rs, cli_server.rs, cli_handlers |
| Método expuesto en binding PyO3/WASM/TS/NAPI | lib.rs, wasm lib.rs, vantadb-ts |
| Componente/página consumible nuevo | web/, desktop/src/components |
| Capability de usuario final que no existía | dashboards, comandos, integraciones |

**Si alguna señal aplica → es feature-add para efectos de SDD**, aunque los
archivos y keywords sugieran "Rust"/"wrapper"/"refactor". Consecuencias:
1. El task file DEBE llevar sección `## Spec` LLENA según la definición canónica
   (question-gates.md §"Contenido válido"): tabla de decisiones de
   spec-template §5 O justificación por-evidencia por ítem. `N/A` NO es válido.
2. Gate P/D dispara UNA ronda de `question` con las decisiones abiertas
   (opciones + `(Recomendado)` primero) — incluso dentro de una familia aprobada.
3. Sin spec válida no se entra a ACT (gate mecánico, pipeline-full §Discovery).

### Phase 2: Discovery + Blast Radius

> **DISCOVERY híbrido (R3):** tareas 🟡/🔴 con DISCOVERY pesado (web research
> multi-doc, extracción de contenido, blast radius amplio) → fork a
> `vanta-research` (`task(subagent_type="vanta-research", ...)` — read-only,
> digest ≤500 palabras + bloque RESULTADO) y usá su digest para poblar esta
> fase. 🟢 con DISCOVERY liviano → ejecutá inline (abajo). NO fragmentar por
> step: la delegación cubre la fase pesada completa (R4: SARL RESUME ya
> conserva contexto del ejecutor).

**SDP Automatizado (OBLIGATORIO — CORE-005):** ANTES de documentar blast radius, llamá:
```
campaign_discover_skills archivosClave="<archivos clave de la task>" phase="BUILD" contractKeywords=["<keywords del contrato/título>"] maxSkills=8
```
El tool devuelve skills con justificaciones (base type + lifecycle BUILD + manifest grep). Cargá cada skill con `skill <nombre>` y registrá `SDP: <skills cargadas>` en el task file.

```
codegraph_explore "IDs, archivos, símbolos de la tarea"
codebase-memory-mcp_detect_changes scope="impact" direction="inbound" depth=3
codebase-memory-mcp_get_architecture aspects="['overview','clusters','hotspots','boundaries']"
codebase-memory-mcp_check_index_coverage paths=["<archivos clave>"]

Documentar en el task file:
- CALLERS: qué módulos llaman a estos archivos (CodeGraph + codebase-memory-mcp_trace_path inbound)
- CALLEES: de qué dependen estos archivos (CodeGraph + codebase-memory-mcp_trace_path outbound)
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

> **Pesado (multi-doc, extracción de contenido) → fork a `vanta-research`**
> (read-only, digest ≤500 palabras + bloque RESULTADO); el lead documenta el
> digest en Investigation Notes. **Liviano (búsqueda puntual) → inline:**

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

### Phase 5: REVIEW por agente distinto (GATE — P2-01)

> **Esta es la falla más grave del reporte P2:** el REVIEW lo ejecutaba el
> mismo contexto que implementó. Desde acá, para CUALQUIER tarea (y con
> prioridad máxima en 🔴), el review lo hace un agente DISTINTO al
> implementador. Nunca el mismo contexto.

- **Persona de review:** `vanta-audit` (persona leaf, `task: * deny` en
  `.opencode/agents/vanta-audit.md` — no puede implementar, solo revisa:
  seguridad + code review). Para review de approach/diseño: persona
  `vanta-review` si existe en el entorno, o la skill `review-deep`
  (`.opencode/skills/review-deep/`) como pipeline de revisión profunda.
- **Alcance obligatorio del review:**
  1. **Enfoque** — ¿el approach elegido es el correcto? ¿hay alternativas
     mejores que no se evaluaron?
  2. **Cómo se probó** — ¿la evidencia de verificación es real y suficiente?
     No alcanza con "test verde" auto-reportado.
- **Gate:** sin review de agente distinto registrado en el task file, la
  tarea NO se marca COMPLETED, aunque el contrato pase.

**Fallback si no hay agente distinto disponible:** `doubt-driven-development`
como gate mandatorio para 🔴 — revisión adversarial en contexto fresco
(nuevo sub-agente o sesión nueva) antes de marcar ✅.

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
- **Incógnitas (uphill):** N abiertas (indicador independiente del % — debe bajar a 0 para ✅)
- **Pendientes (downhill):** N steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | módulo A, módulo B |
| Callees | módulo C, crate D |
| Implicaciones | contrato X no cambia, performance Y mejora |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos. Como orquestador, exigí su
> cumplimiento en cada task file.

- **Archivos leídos (completos):** [paths]
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** [grep]
- **Archivos que referencian a los editados (referencias entrantes):** [grep por nombre del archivo]
- **Veredicto impacto:** [bajo/medio/alto — qué se rompe si cambio/elimino]

## Contrato
"cargo nextest run --profile audit --workspace --build-jobs 2 pasa y el comportamiento específico es [condición]"

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)

> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`".
> Tabla de decisiones (spec-template §5) O justificación por-evidencia por ítem.
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

> **Fuente única de verdad:** plantilla `RESULTADO` §12.3 de
> `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md`
> (SOLO LECTURA). Este task file es la vista de "datos" de la recitation: sus
> secciones Metadata/Steps/Invariantes se sincronizan a `campaign_update_task_state`
> con la MISMA estructura canónica que define `prompts/pipeline-full.md` § 3.
> Los campos MCP reales son 6 (schema campaign-server.mjs); la estructura §12 se
> embeberá dentro de `contract` y `result`:

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# TASK-ID: Descripción` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS con steps pendientes · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos (formato abajo) |
| `nextTask` | Siguiente tarea del plan file |

`contract` (idéntica a `prompts/pipeline-full.md` § 3; sub-campos §12.3):

    contract:
      verificacion: <comando EXACTO + resultado obtenido>   # del task file
      evidencia:
        - claim: <afirmación concreta>
          evidencia: <URL | file path | tool result>
          confianza: alta | media | baja
      artefactos:
        - <path persistido en filesystem>
      invariantes: <qué NO se puede romper — de "Invariantes de dominio">   # si nada: "ninguna"
      deuda: <lo que queda incompleto — de "Invariantes de dominio">        # si nada: "ninguna"
      queda_pendiente: <pendiente_adicional §12 — qué debe delegar/validar el orquestador>

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

**Skills cargadas (SDP):** [lista + justificación 1 línea cada una, o `base-only + SDP sin candidatos`]

## Investigation Notes
- Hallazgos de web research, si aplica

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
```

### Apéndice: Contrato vago vs verificable

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "cd web && npx tsc --noEmit 0 errors, npm run lint 0 errors, npm run build éxito" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |
| "Funciona bien" | "cargo build && cargo nextest run pasa, y [comportamiento específico] funciona" |
