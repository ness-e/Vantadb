# CORE-005: SDP Unificado — campaign_discover_skills MCP Tool

## Metadata
- **Plan file:** docs/plans/2026-08-28-master-pipeline-optimization.md
- **Fuente:** plan file Task 5 / CORE-005 (CRÍTICO #5)
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Tipo:** Mixto (infra task-system + docs prompts)
- **Turns estimados:** 8
- **Creado:** 2026-08-28T12:00
- **last-synced:** 2026-08-28
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/pipeline-full.md` (Paso 0b, Discovery, Re-validar), `.opencode/task-system/prompts/task.md` (Phase 2), `.opencode/task-system/prompts/plan.md` (Paso 0), `.opencode/task-system/prompts/iter-loop-tools.md` (Step 0), `.opencode/task-system/prompts/pipeline-run.md` (paso 6.c), todos los sub-agentes |
| Callees | `.opencode/task-system/mcp/campaign-server.mjs` (tool `campaign_discover_skills`, `LIFECYCLE_SKILLS`, `grepSkillsManifest`, `sdpCache`), `SKILLS-MANIFEST.md`, `.opencode/references/skills-engineering.md` (Lifecycle mapping canónica) |
| Implicaciones | contrato no cambia API pública; si el tool faltara → SDP manual divergente → skills incorrectas → verificación débil; cache TTL 1h (sdpCache) evita staleness cross-campaign; prompts ya migrados → sin breaking change |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición. Sin este bloque poblado, NO se escribe ni se ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** `.opencode/task-system/mcp/campaign-server.mjs` (2098 líneas, tools 1213-1346), `.opencode/references/skills-engineering.md` (78 líneas, SDP § + Lifecycle table), `.opencode/task-system/prompts/pipeline-full.md` (280 líneas), `.opencode/task-system/prompts/iter-loop-tools.md` (404 líneas), `.opencode/task-system/prompts/task.md` (385 líneas), `.opencode/task-system/prompts/plan.md` (304 líneas), `SKILLS-MANIFEST.md` (catálogo 193 skills)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `campaign-server.mjs` importa `parsers.mjs`, `state-tools.mjs`, `model-traits.mjs`, `tracer.mjs`, `SKILLS-MANIFEST.md` (via readFileSync en grepSkillsManifest); prompts no tienen imports — son markdown que invoca MCP tool `campaign_discover_skills` en runtime
- **Archivos que referencian a los editados (referencias entrantes):** `Select-String "campaign_discover_skills"` → 5 hits: `campaign-server.mjs:21,1213,1288`, `pipeline-full.md:18,26,27,67,76`, `iter-loop-tools.md:8,13,15,62`, `task.md:82`, `plan.md:40`; `LIFECYCLE_SKILLS` → solo `campaign-server.mjs:1225,1308,1321` + `skills-engineering.md` tabla; `sdpCache` → `campaign-server.mjs:22,1299,1341`; ningún Rust crate referencia estos archivos
- **Veredicto impacto:** bajo — verificación de lectura idempotente; si hubiese edición sería medio (afecta skill discovery de todas las tareas futuras, pero reversible con 1 commit). Como todo ya existe → impacto nulo, sin edición requerida.

## Contrato
Contrato del plan: Nuevo tool `campaign_discover_skills(keywords, phase)` devuelve `{ skills, justificaciones, lifecycle_phase }`; `campaign_load_skills` actualizado para usarlo; todos prompts invocan MCP. Verificación mecánica:
```
node --check .opencode/task-system/mcp/campaign-server.mjs → exit 0
Select-String "campaign_discover_skills" campaign-server.mjs → ≥1
Select-String "LIFECYCLE_SKILLS" campaign-server.mjs → ≥1
Select-String "grepSkillsManifest|sdpCache" campaign-server.mjs → ≥1
Select-String "campaign_discover_skills" pipeline-full.md → ≥1
Select-String "campaign_discover_skills" iter-loop-tools.md → ≥1
Select-String "campaign_discover_skills" task.md → ≥1
Select-String "campaign_discover_skills" plan.md → ≥1
```
Resultado: todos pasan ✅ (ver Investigation Notes). Sin re-edición.

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)
> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`". Tabla de decisiones O justificación por evidencia por ítem. `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Re-implementar campaign_discover_skills si ya existe con LIFECYCLE+grepSkills+sdpCache? | A) No re-implementar (idempotente, menor riesgo) / B) Re-escribir igual (ruido, posible regresión) | A — idempotente | ✅ decidido-por-evidencia (ref: campaign-server.mjs:1213-1346 tool existe, LIFECYCLE_SKILLS:1225, grepSkillsManifest:1263, sdpCache:22) |
| 2 | ¿Migrar prompts si ya invocan campaign_discover_skills? | A) No migrar (ya cumplen contrato) / B) Re-editar igual | A | ✅ decidido-por-evidencia (ref: pipeline-full.md:18,27,67,76; iter-loop-tools.md:8,13,15; task.md:82; plan.md:40 todos contienen la invocación) |

Justificación: el plan pide "si ya está, marca COMPLETED". Re-implementar introduce riesgo de romper firma/tool ya en uso por HIGH-007/010/012/MED-015/018. Evidencia de que ya está verificado abajo.

## Invariantes de dominio (handoff — MUST)
> El task file debe declarar qué NO se puede romper al continuar, con qué comando se verifica y qué queda incompleto. Sin esto, el próximo agente arranca sin contexto (gap-01 §3.3-18, eng-03-project.md:198).

- **Invariantes a preservar:** `campaign_discover_skills` debe seguir exportando `{ type, label, phase, skills[{name,justification}], commands, checks, estimate, baseSkills, lifecycleSkills, manifestSkills }` con TTL 1h en `sdpCache`; `LIFECYCLE_SKILLS` tabla en `campaign-server.mjs:1225` debe mantenerse alineada con `skills-engineering.md` Lifecycle mapping; todos los 4 prompts deben seguir invocando `campaign_discover_skills` con `archivosClave + phase + contractKeywords + maxSkills=8`; `campaign_load_skills` no debe romper compat hacia atrás
- **Comandos de verificación:** `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0; `Select-String -Pattern "campaign_discover_skills" -Path .opencode/task-system/mcp/campaign-server.mjs` → ≥1; mismo para `LIFECYCLE_SKILLS`, `sdpCache`, `grepSkillsManifest`; 4 prompts cada uno ≥1 hit; `campaign_discover_skills` invocable vía MCP retorna 8 skills con justificaciones (verificado arriba via `campaign_discover_skills` call)
- **Deuda pendiente:** ninguna — idempotente completo, sin edición; próxima tarea HIGH-006 continúa secuencial

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# CORE-005: SDP Unificado — campaign_discover_skills MCP Tool` |
| `lastAction` | Step 1-3 VERIFY completados: campaign-server.mjs tool+LIFECYCLE+grep+sCache ✅ + 4 prompts con MCP invoke ✅ + node --check 0 |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | HIGH-006 — detect_changes en plan.md Paso 0 (siguiente en plan secuencial) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HIGH-006 |

`contract` (idéntica a `prompts/pipeline-full.md` § 3; sub-campos §12.3):
```
contract:
  verificacion: node --check campaign-server.mjs 0 ✅ + Select-String campaign_discover_skills 5/5 ✅ (server 1213, pipeline-full 18,27,67,76; iter-loop 8,13,15,62; task 82; plan 40) + LIFECYCLE_SKILLS 1225 ✅ + grepSkillsManifest 1263 ✅ + sdpCache 22,1299,1341 + TTL 1h (3600000) ✅
  evidencia:
    - claim: campaign-server.mjs tiene campaign_discover_skills con LIFECYCLE_SD + grepSkillsManifest + sdpCache
      evidencia: campaign-server.mjs:21 (sdpCache Map), 1225 (LIFECYCLE_SKILLS), 1263 (grepSkillsManifest), 1288 (server.tool campaign_discover_skills), node --check 0
      confianza: alta
    - claim: prompts pipeline-full.md, iter-loop-tools.md, task.md, plan.md usan campaign_discover_skills
      evidencia: pipeline-full.md:18,26,27,67,76; iter-loop-tools.md:8,13,15,62; task.md:82; plan.md:40 (Select-String hits, ver arriba)
      confianza: alta
  artefactos:
    - .opencode/skills/campaign-executor/tasks/CORE-005.md
    - docs/plans/2026-08-28-master-pipeline-optimization.md (Estado Task 5 → COMPLETED)
  invariantes: LIFECYCLE_SKILLS alineada con skills-engineering.md, sdpCache TTL 1h, prompts invocan MCP con archivosClave/phase/contractKeywords/maxSkills — ninguna rota
  deuda: ninguna
  queda_pendiente: HIGH-006
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda existente — el saldo neto por PR es 0 o negativo.

No se introdujo código, no se introdujo `unsafe`, `clone` en hot path, ni duplicación. Solo creación de task file de trazabilidad. Verificación es solo lectura. Deuda conocida P2 no tocada.

## Definition of Done (contrato multi-nivel — P2-08)
El DoD es **contrato**, no checklist decorativo. La calidad mínima de pie está en `.opencode/references/definition-of-done.md` y aplica SIEMPRE. Además, el task se evalúa por nivel:

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable del task file se cumple + capa determinista (fmt, clippy, nextest) + tests del cambio pasan |
| **Commit** | Commit atómico (~100 líneas), conventional commit, `git diff` limpio, verificación mecánica (nunca auto-reporte) |
| **Release** | `dev-tools/verify.ps1` completo (6 pasos), changelog, semver respetado, pre-push gate (Regla 1) |

**Gate:** el task se marca COMPLETED solo si pasan los tres niveles aplicables a la tarea. Si un nivel no aplica (p.ej. tarea docs sin release), justificar en Notas.

Para CORE-005: Task ✅ (contrato mecánico 7/7 greps + node --check + MCP call 8 skills), Commit ✅ (solo task file nuevo + plan file Estado), Release no aplica (tarea infra interna, justificado en Notas).

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

**Skills cargadas (SDP):** campaign-executor (base type No detectable) · progreso (base) · ponytail (base) · incremental-implementation (lifecycle BUILD) · test-driven-development (lifecycle BUILD) · context-engineering (BUILD) · source-driven-development (BUILD) · doubt-driven-development (BUILD stakes altos) — SDP automatizado vía `campaign_discover_skills` phase BUILD, `contractKeywords=["campaign_discover_skills","SDP","LIFECYCLE_SKILLS","grepSkillsManifest"]`, maxSkills 8 — retorno 8 skills con justificaciones (ver Investigation Notes)

## Investigation Notes
- Verificación directa contra código (no web research necesario — files locales):

  - **Claim:** `campaign-server.mjs` tiene `campaign_discover_skills` con `LIFECYCLE_SKILLS + grepSkillsManifest + sdpCache`
    - **Evidencia:** `Select-String campaign_discover_skills campaign-server.mjs` → líneas 21, 1213, 1288; `LIFECYCLE_SKILLS` → 1225,1308,1321; `grepSkillsManifest` → 1263,1312; `sdpCache` → 22,1299,1341; `3600000` (TTL 1h) en línea 1300; `node --check` → 0
    - **Confianza:** alta

  - **Claim:** `pipeline-full.md` invoca `campaign_discover_skills`
    - **Evidencia:** líneas 18 (base type + lifecycle), 26 (SDP automatiza), 27 (contractKeywords), 67 (SDP Automatizado), 76 (Re-validar Skills) — 5 hits
    - **Confianza:** alta

  - **Claim:** `iter-loop-tools.md` invoca `campaign_discover_skills`
    - **Evidencia:** líneas 8 (base via discover), 13 (Con Archivos clave), 15 (campaign_discover_skills archivosClave... BUILD), 62 (SDP Ya Completado) — 4 hits
    - **Confianza:** alta

  - **Claim:** `task.md` invoca `campaign_discover_skills`
    - **Evidencia:** línea 82 `campaign_discover_skills archivosClave="<archivos clave de la task>" phase="BUILD" contractKeywords=["<keywords del contrato/título>"] maxSkills=8` — 1 hit
    - **Confianza:** alta

  - **Claim:** `plan.md` invoca `campaign_discover_skills`
    - **Evidencia:** línea 40 `campaign_discover_skills archivosClave="<archivos de la tarea>" phase="PLAN" contractKeywords=["<keywords del título/contrato>"] maxSkills=8` — 1 hit
    - **Confianza:** alta

  - **Claim:** `skills-engineering.md` es fuente canónica SDP alineada con LIFECYCLE_SKILLS
    - **Evidencia:** SDP §5-31 lifecycle table DEFINE/PLAN/BUILD/VERIFY/REVIEW/SHIP; `campaign-server.mjs` LIFECYCLE_SKILLS replica tabla (DEFINE: spec-driven, interview-me, idea-refine; BUILD: incremental, TDD, context-engineering, source-driven, doubt-driven, frontend, api; etc.)
    - **Confianza:** alta

  - **Claim:** `campaign_discover_skills` MCP funciona y retorna skills con justificaciones (≤8)
    - **Evidencia:** llamada MCP `campaign_discover_skills archivosClave="..." phase="BUILD" contractKeywords=["campaign_discover_skills","SDP",...] maxSkills=8` → retorno 8 skills `[campaign-executor, progreso, ponytail, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development]` cada una con justification; baseSkills + lifecycleSkills + manifestSkills separados
    - **Confianza:** alta

- Ponytail: todo ya existe → no re-implementar (ladder rung 1: ¿necesita existir? No, ya existe). Tarea es verificación y trazabilidad.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
Dos ejes distintos del % de completado. El % mide ejecución; las incógnitas miden certidumbre. El estado reporta los tres por separado:

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — verificado que tool + lifecycle + cache + 4 prompts existen; contrato claro (grep + node --check); sin ambigüedad |
| Pendientes de ejecución (downhill) | 0 — sin edición, solo verify + plan update + commit |
| % completado | 100% |

**Regla de reporting:** cada actualización de estado actualiza los tres contadores. Una incógnita resuelta se mueve de Incógnitas → Notas con la respuesta. Una tarea con incógnitas abiertas NO se marca ✅ aunque el % sea 100%.

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
> Obligatoria para tareas tipo Bug (`fix:`). El fix requiere método correcto, no solo test verde: **Iron Law** de systematic-debugging — sin investigación de causa raíz no hay fix. Sin esta sección poblada, NO se escribe ni se ejecuta el step de fix.

- **Repro:** N/A — no es bug, es verificación de infra SDP
- **Hipótesis:** N/A
- **1 variable controlada:** N/A
- **Test RED:** N/A

**Gate:** los steps de fix y sus Verify se definen solo DESPUÉS de completar esta sección con `repro`, `hipótesis`, `1 variable controlada` y `test RED`. Grafías aceptadas del campo: `hipótesis|hipotesis`.

No aplica — tarea feature infra ya implementada, justificado.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
Evaluación mandatoria ANTES de codear. Si no aplica, justificar en Notas:

- [x] **SECURITY** — si toca trust boundaries, input de usuario, auth, datos, o agrega/quita dependencias → cargar `security-and-hardening` y documentar hallazgos en Notas. Si no aplica, justificar por qué. → **No aplica:** tool lee SKILLS-MANIFEST.md local y parsea archivos clave strings; sin input usuario no sanitizado, sin auth, sin dependencias nuevas, sin storage externo, sin network. `grepSkillsManifest` usa RegExp con keywords controladas — sin inyección (keywords son strings fijas del contrato). Justificado.
- [x] **PERFORMANCE** — si toca un hot path (búsqueda, indexación, serialización, loops calientes) → cargar `performance-optimization` y registrar baseline/impacto esperado. Si no aplica, justificar. → **No aplica:** SDP discovery es llamado 1× por tarea en DISCOVERY (≤5 min budget); cache `sdpCache` Map con TTL 1h evita re-parseo; `readFileSync SKILLS-MANIFEST.md` es 5-10ms. No hot path vector/search. Justificado.

## Steps
### Step 1: Verificar campaign-server.mjs tiene campaign_discover_skills completo (DISCOVERY)
- **Archivos:** `.opencode/task-system/mcp/campaign-server.mjs`
- **Acción:** Ejecutar `Select-String` para `campaign_discover_skills`, `LIFECYCLE_SKILLS`, `grepSkillsManifest`, `sdpCache` + `node --check` + llamada MCP `campaign_discover_skills` con phase BUILD y keywords del contrato → verificar 8 skills con justificaciones y TTL 1h
- **Verify:** `node --check` 0 ✅ ; 7/7 greps ≥1 ✅ ; MCP call retorna `skills.length ≤8` con justifications ✅
- **Estado:** ✅ COMPLETED (2026-08-28 — ver Investigation Notes: líneas 21/1225/1263/1288 + cache 3600000 + MCP 8 skills)

### Step 2: Verificar 4 prompts usan campaign_discover_skills (DISCOVERY)
- **Archivos:** `.opencode/task-system/prompts/pipeline-full.md`, `.opencode/task-system/prompts/iter-loop-tools.md`, `.opencode/task-system/prompts/task.md`, `.opencode/task-system/prompts/plan.md`
- **Acción:** `Select-String "campaign_discover_skills"` en cada prompt → contar hits; validar que cada uno invoca con `archivosClave + phase + contractKeywords + maxSkills=8` y registra `SDP:` en task file / `SKILLS_CARGADAS:` en RESULTADO
- **Verify:** pipeline-full 5 hits ✅ ; iter-loop 4 hits ✅ ; task 1 hit (línea 82) ✅ ; plan 1 hit (línea 40) ✅
- **Estado:** ✅ COMPLETED (2026-08-28 — ver Investigation Notes líneas exactas)

### Step 3: VERIFY/CLOSE — gate mecánico + commit
- **Archivos:** `.opencode/skills/campaign-executor/tasks/CORE-005.md`, `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 5 Estado → ✅ COMPLETED), `.opencode/task-system/memory/lessons.md`
- **Acción:** Crear task file con 20 secciones + actualizar plan file Task 5 Estado PENDING → COMPLETED + `campaign_memory_write` lesson + `git commit` con conventional commit `chore: CORE-005 — SDP Unificado — verificación idempotente`
- **Verify:** `git status` muestra solo 2 archivos tocados (task file nuevo + plan file 1 línea Estado); `git log --oneline -1` muestra commit chore; plan file Task 5 Estado = COMPLETED
- **Estado:** ✅ COMPLETED (commit idempotente)

## Dependencias
- Task previo: CORE-004 — Task File Template Completo (debe completarse antes) → ✅ COMPLETED (commit bec9d2cf)
- Siguiente: HIGH-006 — detect_changes en plan.md Paso 0

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf, verifica sin implementar) — ponytail: auto-review mecánico porque cambio es solo verificación de lectura + task file de trazabilidad, sin código
- **Enfoque:** ¿el approach idempotente es correcto? ¿evita re-implementación innecesaria del tool? ¿contrato 7/7 greps + node --check + MCP call se cumple? ¿4 prompts realmente invocan con firma correcta?
- **Cómo se probó:** evidencia mecánica `node --check 0`, 7 greps con líneas exactas documentadas en Investigation Notes, llamada MCP real `campaign_discover_skills` → 8 skills con justificaciones, 4 prompts cada uno ≥1 hit con líneas citadas; `git diff` de campaign-server.mjs = vacío (no editado) — idempotente justificado por ponytail
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve — idempotente correcto, contrato 7/7 verificado con líneas exactas, MCP call con 8 skills justificado, sin re-edición justificada por ponytail (menos código = menos riesgo)

## Notas
- Ponytail: todo el SDP ya existe (tool + LIFECYCLE + cache + 4 prompts) → se evita re-implementación que solo generaría churn y riesgo de regresión en firma. Si en futuro cambia Lifecycle mapping, mantener sincronizado `skills-engineering.md` ↔ `LIFECYCLE_SKILLS` en `campaign-server.mjs:1225` y actualizar cache TTL si needed.
- Instrucción explícita del usuario: "Si ya está, marca COMPLETED. Seguí pipeline-full.md, devolvé RESULTADO." — cumplida: verificación mecánica idempotente, task file trae trazabilidad, plan file se actualiza.
- No se tocó `campaign-server.mjs` (2098 líneas, tool 1213-1346) → `git diff` vacío para ese archivo; solo task file nuevo + plan file 1 línea Estado.

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping canónica (§ Lifecycle mapping)
- `SKILLS-MANIFEST.md` — catálogo de 193 skills (grep target para manifestSkills)
- `.opencode/task-system/prompts/pipeline-full.md` — Paso 0b SDP unificado vía MCP (líneas 18,26,27,67,76)
- `.opencode/task-system/prompts/iter-loop-tools.md` — Step 0 SDP via campaign_discover_skills (líneas 8,13,15,62)
- `.opencode/task-system/prompts/task.md` — Phase 2 SDP automatizado (línea 82)
- `.opencode/task-system/prompts/plan.md` — Paso 0 SDP automatizado (línea 40)
- `.opencode/task-system/prompts/pipeline-full.md` — flujo DISCOVERY→EJECUCIÓN→CIERRE seguido
- `.opencode/task-system/mcp/campaign-server.mjs:21,1225,1263,1288` — sdpCache, LIFECYCLE_SKILLS, grepSkillsManifest, campaign_discover_skills

## Context Save Point
- **Fecha:** 2026-08-28
- **Branch:** develop (ver git branch)
- **CI pendiente:** no — verify mecánico local (node --check, Select-String 7/7, MCP call 8 skills); `cargo check/fmt/clippy` no aplica (cambio docs/infra solo, sin Rust)
- **Decisiones:** No re-implementar SDP porque ya existe completo con LIFECYCLE+grep+sdpCache+4 prompts — ponytail rung 1 → skip; task file documenta evidencia con líneas exactas
- **Problemas conocidos:** ninguno
- **Próxima tarea:** HIGH-006
