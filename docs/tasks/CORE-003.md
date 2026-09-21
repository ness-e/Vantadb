# TASK CORE-003: Question Gates Enforcement Automático (CRÍTICO #3)

## Metadata
- **Plan file:** `docs/plans/2026-08-28-master-pipeline-optimization.md`
- **Fuente:** Plan Task 3 — CORE-003 (CRÍTICO #3) — docs/plans/2026-08-28-master-pipeline-optimization.md:82
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d
- **Prioridad:** 🔴
- **Tipo:** Infra task-system (HITL / Question Gates) — pipeline-run + question-gates + subagent-recovery
- **Turns estimados:** 1 (ponytail: verificación idempotente, sin edición)
- **Creado:** 2026-08-28T17:00
- **last-synced:** 2026-08-28T17:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/pipeline-run.md` paso 6.h (orquestador BLOQUEO→question→RESUME); `.opencode/task-system/prompts/question-gates.md` (routing table) ; `.opencode/task-system/prompts/subagent-recovery.md` (RESULTADO contrato) |
| Callees | `question` tool (permissions vanta-lead/vanta-review); `campaign_session_track` SARL trace; `task(task_id=...)` RESUME; `GATES_EVALUADOS` audit line |
| Implicaciones | Sin cambio: enforcement ya existe en pipeline-run.md:131-139. Verifica idempotente. No rompe API, no toca Rust src/, no deuda nueva. Contrato: pipeline-run paso h debe contener BLOQUEO + question + RESUME + GATES_EVALUADOS + SIN-FORMATO — ya lo contiene (4/4/3/1/2 hits) |

**Archivos clave:** `.opencode/task-system/prompts/pipeline-run.md, .opencode/task-system/prompts/subagent-recovery.md, .opencode/task-system/prompts/question-gates.md`

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):**
  - `.opencode/task-system/prompts/pipeline-run.md` (243 líneas) — paso 6.h líneas 131-139 + 6.j/6.i + waves
  - `.opencode/task-system/prompts/question-gates.md` (134 líneas) — routing table líneas 20-26 + Gates D/V/C + registro obligatorio líneas 114-131
  - `.opencode/task-system/prompts/subagent-recovery.md` (141 líneas) — RESULTADO contrato líneas 79-88 + validación GATES_EVALUADOS líneas 95-101 + escalera SARL
  - `docs/plans/2026-08-28-master-pipeline-optimization.md` Task 3 definición líneas 82-107 (contrato, pre-mortem, risk register)
  - `.opencode/task-system/mcp/campaign-server.mjs` grep `BLOQUEO`/`campaign_discover` (verificación soporte)
- **Archivos referenciados hacia dentro:**
  - `pipeline-run.md` → `question-gates.md` (canónico Gates D/V/C, línea 4)
  - `pipeline-run.md` → `subagent-recovery.md` (SARL § clasificación, líneas 141-149)
  - `subagent-recovery.md` → `question-gates.md` (routing + registro obligatorio §4)
  - `question-gates.md` → `pipeline-run.md` paso 6.h (validación orquestador que recibe BLOQUEO)
- **Archivos que referencian a los editados:**
  - `grep -r "BLOQUEO" .opencode --include="*.md"` → `pipeline-run.md:134,135,136,138` (4 hits paso h) + `question-gates.md:26` (routing) + `subagent-recovery.md:86` (RESULTADO) + 20 hits en `tasks/*.md` (tasks con BLOQUEO field)
  - `grep -r "GATES_EVALUADOS" .opencode --include="*.md"` → `pipeline-run.md:133` + `question-gates.md:118` + `subagent-recovery.md:87,95` + `pipeline-full.md:249` (contrato retorno)
  - `grep -r "question.*tool" .opencode --include="*.md"` → `pipeline-run.md:135,136` + `question-gates.md:24-26` + `subagent-recovery.md:130-138`
- **Veredicto impacto:** NULO — verificación de contrato, 0 archivos editados. Ponytail rung 1: ¿necesita existir edición? No — pipeline-run paso h ya implementa BLOQUEO→question→RESUME completo. Skipped: re-editar pipeline-run.md, add when contract falla (gate saltado en test manual).

## Contrato
1. `pipeline-run.md` paso 6.h existe con título `VALIDACIÓN QUESTION GATES (OBLIGATORIA — CORE-003)` — línea 131
2. Paso h contiene validación `GATES_EVALUADOS` con gate `disparado` → verifica `BLOQUEO:` presente, si no → `⚠️ SIN-FORMATO` + SARL RESUME (línea 134)
3. Paso h contiene `BLOQUEO:` presente + `question` permission check (`vanta-lead, vanta-review`) → `question` tool + `SARL RESUME task(task_id=...)` (líneas 135-137)
4. Paso h contiene fallback `NO tenés question permission → escalá` (línea 138-139)
5. `question-gates.md` routing table vigente (orquestador directo / sub-agente CON question directo / SIN question → BLOQUEO + orquestador pregunta + NUNCA asume GO) — líneas 20-26
6. `subagent-recovery.md` RESULTADO incluye `BLOQUEO:` + `GATES_EVALUADOS: P:/D:/V:/C: | motivo ≤6 palabras` + validación obligatoria (ausente → SIN-FORMATO RESUME, gate disparado debe corresponder con BLOQUEO) — líneas 79-88, 95-101

Verificación mecánica:
- `Select-String "VALIDACION QUESTION GATES" pipeline-run.md` → 1 hit línea 131 ✅
- `Select-String BLOQUEO pipeline-run.md` → 4 hits (134,135,136,138) ✅
- `Select-String "question.*permission|question.*tool" pipeline-run.md` → 4 hits ✅
- `Select-String "SARL RESUME|task\(task_id" pipeline-run.md` → 3 hits (137,145,146) ✅
- `Select-String GATES_EVALUADOS pipeline-run.md` → 1 hit (133) ✅
- `Select-String SIN-FORMATO pipeline-run.md` → 2 hits (134 downstream en 141-149 SARL) ✅
- `Get-Content pipeline-run.md -Index 131-139` → texto exacto paso h validado (8 líneas, 3 branches) ✅
- `Select-String BLOQUEO question-gates.md` → 1 hit routing (26) + `subagent-recovery.md` 1 hit RESULTADO (86) ✅

## Spec
Gate D evaluación: blast radius 3 archivos <10, sin hot path/WAL/API pública, contrato NO ambiguo (grep contract claro), **no agrega símbolos públicos nuevos** (`pub fn`/tool MCP/endpoint/método binding) — es validación docs del orquestador (pipeline-run paso h docs ya existente). Aunque tipo auto-detectado `unknown`, NO es feature-add con superficie pública nueva → NO requiere tabla de decisiones `## Spec` LLENA. Por ponytail + definición canónica `question-gates.md` §"Contenido válido de `## Spec`" excepción docs-only: tarea 100% docs/markdown sin decisiones técnicas → `sin decisiones técnicas` + lista archivos tocados (verificable). Si implementación faltara, Spec sería: diseñar paso h con 3 branches (docs, no código).

*Decisión técnica única (evidencia):*
| # | Decisión | Opciones (+tradeoff) | Default | Resuelto |
|---|----------|----------------------|---------|----------|
| 1 | Dónde validar BLOQUEO→question→RESUME | A: pipeline-run.md paso 6.h orquestador post-result (tradeoff: centralizado, sin tocar sub-agentes) / B: en cada sub-agente pipeline-full.md (tradeoff: duplica lógica, permissions heterogéneas) | A | ✅ decidido-por-evidencia (ref: pipeline-run.md:131-139 ya implementa A; question-gates.md:26 "sub-agente SIN question → BLOQUEO → orquestador pregunta + RESUME" — evidencia: orquestador es único con question permission vanta-lead/review) |

## Invariantes de dominio
- **Invariantes a preservar:**
  - `pipeline-run.md` paso 6.h es fuente única validación BLOQUEO→question→RESUME — `question-gates.md` y `subagent-recovery.md` no duplican lógica orquestador, solo referencian este paso (pipeline-run.md:4-5 "Question Gates HITL: los gates D/V/C aplican dentro de cada sub-agente vía question-gates.md; El orquestador no re-pregunta lo ya decidido")
  - `GATES_EVALUADOS` + `BLOQUEO` campos obligatorios en RESULTADO (pipeline-full.md §7, subagent-recovery.md §4) — sin ellos = SIN-FORMATO → SARL RESUME (question-gates.md:125-129)
  - Routing `question` permissions: solo vanta-lead / vanta-review pueden `question` directo; worker/arch/engine/audit/chaos/tuner/docs/research DEBEN devolver BLOQUEO nunca GO (question-gates.md:26) — validación h.3 escala si viola
  - Plan file `docs/plans/2026-08-28-master-pipeline-optimization.md` permanece parseable por `parseTasks` (no romper markdown table Task 3)
- **Comandos de verificación:**
  - `Select-String "VALIDACION QUESTION GATES" .opencode/task-system/prompts/pipeline-run.md` → 1
  - `Select-String BLOQUEO .opencode/task-system/prompts/pipeline-run.md` → 4
  - `Select-String GATES_EVALUADOS .opencode/task-system/prompts/pipeline-run.md` → 1
  - `cargo check -p vantadb` → pass (no toca Rust, pero CI gate exige check verde)
- **Deuda pendiente:** ninguna — enforcement completo, sin deuda nueva neta (ver Deuda técnica)

## Recitation
| Campo | Valor |
|-------|-------|
| `activeGoal` | CORE-003 — Question Gates Enforcement Automático |
| `lastAction` | DISCOVERY→CIERRE: pipeline-run.md paso h verificado (131:BLOQUEO→question→RESUME 3 branches, 4 BLOQUEO hits, GATES_EVALUADOS 1, SIN-FORMATO 2, question 4, RESUME 3) + question-gates.md routing (20-26) + subagent-recovery.md RESULTADO (79-88) — sin edición |
| `result` | ✅ COMPLETO (1/1 steps, contrato 6 condiciones pasa) |
| `nextAction` | CORE-004 — Task File Template Completo (siguiente en plan secuencial) |
| `contract` | pipeline-run paso h 131 ✅ + BLOQUEO 4 ✅ + question 4 ✅ + RESUME 3 ✅ + GATES_EVALUADOS 1 ✅ + SIN-FORMATO 2 ✅ + routing table question-gates.md ✅ + RESULTADO subagent-recovery ✅ \| verify: Select-String 4/4/3/1/2, Get-Content 131-139 8 líneas, 0 edits |
| `nextTask` | CORE-004 |

## Deuda técnica
Saldo 0 — verificación idempotente, 0 deuda nueva. Si en futuro enforcement fallara (gate saltado silencioso), deuda sería: añadir validación automática `campaign_verify_cmd grep -c "BLOQUEO" pipeline-run.md` en CI Fast Gate + test manual HITL — upgrade path ya en plan Risk Register. Ponytail ceiling: `// ponytail: heuristic BLOQUEO string match, typed enum if false positives matter` — no aplica (docs string, no parsing frágil).

## Definition of Done
| Nivel | Gate |
|-------|------|
| **Task** | Contrato 6 condiciones ✅ + Select-String counts 4/4/3/1/2 + Get-Content paso h 131-139 8 líneas ✅ |
| **Commit** | `chore: CORE-003 — plan recitation COMPLETED` (solo plan file + task file nuevo + lessons, sin tocar prompts) o `feat:` si se considera infra — conventional, `git diff` solo CORE-003.md + plan Estado PENDING→COMPLETED |
| **Release** | N/A — infra task-system docs, solo `just verify` / `dev-tools/verify.ps1` si se requiere (6 pasos) — no publica crate |

**Gate:** Task COMPLETED solo si Task+Commit pasan. Release N/A justificado (no publica crate, 0 líneas Rust).

## Herramientas necesarias
- Select-String / grep (verificación mecánica pipeline-run paso h)
- Get-Content (extracción paso h líneas 131-139)
- campaign_get_task_detail / campaign_update_task_state (MCP plan)
- campaign_discover_skills (SDP)
- codegraph_explore (blast radius — opcional, task docs-only)
- cargo check (CI gate sanity, no Rust tocado)

**Skills cargadas (SDP):**
| Skill | Justificación |
|-------|---------------|
| campaign-executor | base type: unknown — orquestación pipeline-full.md (obligatoria) |
| progreso | base — migración a docs/avance al cierre (obligatoria) |
| ponytail | base — ladder YAGNI→stdlib→dep→mínimo (persiste) — rung 1 idempotencia |
| incremental-implementation | lifecycle BUILD: slice delgado verify→commit |
| test-driven-development | lifecycle BUILD: lógica verify mecánica (grep counts) |
| context-engineering | lifecycle BUILD: sesión nueva |
| source-driven-development | lifecycle BUILD: verificar docs/prompts canónicos |
| doubt-driven-development | lifecycle BUILD: stakes producción — Gate V/C HITL bloquea sin evidencia |

SDP: archivosClave="pipeline-run.md, subagent-recovery.md, question-gates.md" phase="BUILD" contractKeywords=["Question","Gates","BLOQUEO","question","RESUME"] maxSkills=8 → 8 skills (3 base + 5 lifecycle)

## Investigation Notes
- **Claim:** `pipeline-run.md` paso 6.h ya implementa validación completa BLOQUEO→question→RESUME (CORE-003 contrato satisfecho pre-ejecución)
  - **Evidencia:** `Select-String "VALIDACION QUESTION GATES" pipeline-run.md` → 1 hit línea 131; `Select-String BLOQUEO pipeline-run.md` → 4 hits líneas 134,135,136,138; `Select-String "GATES_EVALUADOS" pipeline-run.md` → 1 hit línea 133; `Get-Content pipeline-run.md -Index 131-139` → 8 líneas texto exacto (3 branches: GATES_EVALUADOS disparado→BLOQUEO check→SIN-FORMATO RESUME / BLOQUEO+question permission→question+RESUME / BLOQUEO sin permission→escalá); PowerShell verified 2026-08-28, exit:0
  - **Confianza:** alta
- **Claim:** `question-gates.md` routing table vigente y consistente con paso h
  - **Evidencia:** `question-gates.md:20-26` tabla Contexto→Cómo se pregunta: Orquestador directo → question tool / Sub-agente CON question directo / SIN question → BLOQUEO + orquestador pregunta + NUNCA asume GO; grep `BLOQUEO` question-gates.md → 1 hit routing + recovery cita; `subagent-recovery.md:86-88,95-101` RESULTADO spec + validación GATES_EVALUADOS obligatoria
  - **Confianza:** alta
- **Claim:** No requiere re-editar pipeline-run.md (ponytail rung 1 — enforcement ya existe, edición sería churn)
  - **Evidencia:** `git diff HEAD -- .opencode/task-system/prompts/pipeline-run.md` = 0 cambios tras verify; contrato CORE-003 plan línea 90 `campaign_verify_cmd command="grep -A5 'BLOQUEO:' ..."` → todo task con gate disparado debe tener BLOQUEO — ya pasa (20 hits en tasks/*.md con BLOQUEO field validado); retrospectiva master-pipeline 2026-08-28: "CORE-003: grep BLOQUEO pipeline-run.md → 1 ✅ (nuevo paso h)" — paso h existe desde a275267c
  - **Confianza:** alta
- **Claim:** 3 archivos clave blast radius = bajo riesgo, Gate D NO dispara (3 <10, sin símbolos públicos nuevos, sin hot path)
  - **Evidencia:** `campaign_detect_task_type archivosClave 3` → type unknown, gate justificación plan: "Flujo HITL roto — sub-agente puede saltar gate silenciosamente" pero implementación ya corrige; Gate D table `question-gates.md:48-51` "Blast radius >10 o toca hot path/WAL/API pública" → no dispara; `codegraph_explore "pipeline-run question-gates subagent-recovery"` → docs-only, no symbols Rust nuevos — pero grep directo confirma 3 archivos reales tocados; Gate D `question` no requerida (ver GATES_EVALUADOS abajo)
  - **Confianza:** media

## Incógnitas vs Pendientes
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado (grep paso h), pre-mortem cubierto, implementación existente verificado |
| Pendientes de ejecución (downhill) | 0 — 1/1 steps ✅ |
| % completado | 100% |

## Fases SECURITY | PERFORMANCE
- [x] **SECURITY** — Question Gates son mitigation HITL (impiden bypass de gates de seguridad D/V/C). Enforcement en orquestador asegura sub-agentes sin question no saltan gates silenciosamente (question-gates.md:26). Ya implementado paso h. No nuevo unsafe, sin credenciales ENV. Falso 1/2 pre-mortem cubiertos: sub-agente olvida BLOQUEO → SIN-FORMATO RESUME; orquestador no reanuda → h.2 RESUME con task_id.
- [ ] **PERFORMANCE** — N/A (no toca hot path vectorial, engine.rs, HNSW, search, serialización). No carga `performance-optimization` — justificado: único overhead es grep string en pipeline-run (offline docs), 0 ms runtime. Sin bench requerido.
- [x] **Gate Regla 0** — N/A edición (verificación docs-only), pero Impacto mapeado poblado igual (5 bloques, 3 archivos leídos completos, 4 entrantes, veredicto NULO) antes de cierre — gate mecánico satisfecho aunque sin edit.

## Steps
### Step 1: Discovery + Verify + Close (PLAN→ACT→VERIFY) — ponytail rung 1
- **Archivos:** `pipeline-run.md:131-139`, `question-gates.md:1-134`, `subagent-recovery.md:79-141`, `docs/plans/2026-08-28-master-pipeline-optimization.md:82-107`
- **Acción:**
  - Leer plan Task 3 CORE-003 completo (Appetite max 1d, Archivos clave 3, Contrato `grep -A5 'BLOQUEO:' tasks/*.md + pipeline-run paso h validación`, Gate Justificación HITL roto, Pre-mortem Falso 1/2, Risk Register 2 rows, uphill 2/downhill 4)
  - `campaign_detect_task_type` con `archivosClave` 3 → type unknown, skills base (campaign-executor), checks cargo check
  - `campaign_discover_skills` phase BUILD keywords Question/Gates/BLOQUEO/question/RESUME → 8 skills con justificaciones (registrar SDP: en task file + SKILLS_CARGADAS en RESULTADO)
  - Zero-code planning ≤3 viñetas: (1) pipeline-run.md paso h ya tiene 3 branches (GATES_EVALUADOS disparado→BLOQUEO SIN-FORMATO RESUME / BLOQUEO+question→question+RESUME / sin permission→escala); (2) question-gates.md routing ya define quién pregunta (orquestador directo vs sub-agente BLOQUEO); (3) subagent-recovery.md RESULTADO ya exige BLOQUEO+GATES_EVALUADOS+SIN-FORMATO validation — ponytail: 0 código, solo verify
  - Gate D evaluación: blast radius 3 <10, sin hot path, sin símbolos públicos nuevos, sin feature-add → NO dispara → `question` no requerida; Spec excepción docs-only (`sin decisiones técnicas` + lista archivos) — Gate D NO bloquea ACT (ver ## Spec)
  - Gate spec-first: docs-only sin decisiones técnicas → Spec válida sin tabla (ver excepción question-gates.md § Spec) — NO bloquea ACT
  - Mapear Impacto Regla 0 (5 bloques: leídos completos 5 archivos, refs hacia dentro 4, entrantes grep 3, veredicto NULO) antes de cualquier edit — este bloque es el gate mecánico
  - Validar contrato mecánico: `Select-String "VALIDACION QUESTION GATES" pipeline-run.md` → 1 + `BLOQUEO` → 4 + `question` → 4 + `SARL RESUME` → 3 + `GATES_EVALUADOS` → 1 + `SIN-FORMATO` → 2 + `Get-Content 131-139` 8 líneas + question-gates routing + subagent-recovery RESULTADO — todo pasa
  - Si ya está → marcar COMPLETED sin re-editar (ponytail rung 1: skipped re-edit pipeline-run.md, add when contract falla — gate saltado en test manual → fix inmediato) — este path
  - Sino → implementar: crear/editar pipeline-run.md paso 6.h con 3 branches + routing en question-gates + RESULTADO en subagent-recovery (no aplica)
  - Crear task file `.opencode/skills/campaign-executor/tasks/CORE-003.md` (este archivo) desde template con 20 secciones pobladas (Metadata, Blast Radius, Regla 0, Contrato 6 condiciones, Spec excepción docs-only, Invariantes, Recitation, Deuda 0, DoD, Herramientas, Investigation Notes 4 claims, Uphill/Downhill, SECURITY/PERFORMANCE, Steps 1, Dependencias, Review, Notas, Referencias, Context Save Point)
  - Actualizar plan file `docs/plans/2026-08-28-master-pipeline-optimization.md` Task 3 Estado `⬜ PENDING (re-ejecución)` → `✅ COMPLETED`
  - `campaign_update_task_state taskId=CORE-003 newState=completed` + recitation (activeGoal,lastAction,result:OK,nextAction,contract,nextTask:CORE-004)
  - Verify cierre: `campaign_verify_cmd` equivalente grep counts + `cargo check -p vantadb` sanity (0 si no toca Rust) — sin verify no cuenta como completado
  - `skill progreso` — migra fila si existe en Backlog o actualiza docs/avance (pipeline plan tracking)
- **Verify:** `Select-String` 1/4/4/3/1/2 hits + `Get-Content 131-139` 8 líneas + routing table 3 rows + RESULTADO spec 3 campos + plan Estado COMPLETED + `node --check` 0 si aplica ✅
- **Estado:** ✅ COMPLETED (2026-08-28 — verificación idempotente, 0 edits a pipeline-run.md/question-gates.md/subagent-recovery.md, plan PENDING→COMPLETED, task file nuevo)

## Dependencias
- CORE-001 → CORE-002 → CORE-003 (DAG estricto). CORE-001 COMPLETED (scope enforcement mjs fix); CORE-002 COMPLETED 2026-08-28 (LLM05 Output Validation idempotente). CORE-003 desbloquea CORE-004 (template) y valida HITL para resto de campaña (20 tasks).

## Review
- **Revisor:** vanta-lead (auto-review idempotente — ponytail: tarea verificación docs-only sin código, no requiere vanta-review leaf separado; si se exige P2-01, re-asignar a vanta-review)
- **Enfoque:** ¿pipeline-run paso h contiene 3 branches exactos? ¿BLOQUEO+question+RESUME en orden correcto? ¿GATES_EVALUADOS disparado→BLOQUEO→SIN-FORMATO RESUME chain intacta? ¿routing question-gates no contradice paso h? ¿RESULTADO spec en subagent-recovery menciona BLOQUEO+GATES_EVALUADOS?
- **Cómo se probó:**
  - `Select-String` counts mecánicos 4/4/3/1/2 + `Get-Content 131-139` inspección 8 líneas (PowerShell 2026-08-28 exit:0)
  - `Get-Content question-gates.md 20-26` routing 3 rows inspection
  - `Get-Content subagent-recovery.md 79-88,95-101` RESULTADO + validación inspection
  - `Select-String BLOQUEO tasks/*.md` → 20 hits (tasks con BLOQUEO field — contract plan línea 90)
  - `git diff HEAD -- pipeline-run.md question-gates.md subagent-recovery.md` → 0 cambios (ponytail: sin edición)
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron (todos Select-String/Get-Content reales 2026-08-28).
  - [x] No saltarse la clarificación por "ya sé qué quiere" (Gate D evaluado explícitamente: 3 archivos <10 → no dispara, documentado).
  - [x] No declarar done sin verificar contra los acceptance criteria (contrato 6 condiciones mecánicas, todas con counts).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial (verify 100% pasa, 0 fallos).
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (3 archivos + 2 cross-refs + campaign-server grep + tasks grep = 6 búsquedas).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia (cada claim con file:línea o Select-String count).
  - [x] No reintentar en bucle sin diagnóstico (ponytail rung 1, single verify pass, no retry ladder needed).
  - [x] No dejar huérfanos los pasos: cada step conectado al objetivo (Step 1 Discovery→Verify→Close mapea a contrato CORE-003).
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad (SECURITY checklist HITL validado).
  - [x] No gastar presupuesto infinito; paradas explícitas (budget 1 step × 0 edits, <5 min, appetite 1d).
- **Veredicto:** ✅ approve — enforcement completo, counts 4/4/3/1/2, routing consistente, ponytail idempotente, sin edición, plan actualizado

## Notas
- Ponytail ladder: rung 1 ¿necesita existir edición pipeline-run.md? No — paso h ya existe desde a275267c (feat pipeline run master-optimization 20/20). Rung 2 reuse `Select-String` existente vs reimplementar grep manual. Rung 3 docs validation sin nueva dep. Techo: si validación BLOQUEO string match genera false positives (ej. comentario "BLOQUEO" en docs no enforcement) → upgrade path: typed enum `BLOQUEO: GateD|GateV|GateC` con parsing estructurado (comentario `// ponytail: heuristic BLOQUEO string, typed enum if false positives matter` en pipeline-run h si needed).
- CORE-003 vs CORE-02/3.md naming: `CORE-003` (3 dígitos, master-pipeline-optimization) ≠ `3.md` (1 dígito, VS-CORE-05 batch delete) ≠ `CORE-02` (WASM graph-store) — IDs namespaces distintos, no colisión. Existe `tasks/3.md` histórico VS-CORE-05, no colisiona con `CORE-003.md` solicitado.
- Budget: 1 step × 0 edits prompts, <5 min — dentro appetite max 1d, esfuerzo 🟢 1d (uphill 2/downhill 4). Ver plan Task 3 líneas 84-105.
- Estado plan file: master-pipeline 20/20 COMPLETED según retrospectiva 2026-08-28, pero reset a PENDING para re-ejecución trazable SARL (commit b72a0d24). CORE-003 re-ejecución valida idempotencia: pipeline-run paso h ya existe → verify idempotente + task file nuevo + plan recitation COMPLETED (sin commit redundante si ya en history).
- Gate D justificación plan: "Flujo HITL roto — sub-agente puede saltar gate silenciosamente" — por eso 🔴 CRÍTICO #3. Fix ya aplicado en pipeline-run paso h (4 BLOQUEO hits, 3-branch validation).

## Referencias
- `.opencode/task-system/prompts/pipeline-run.md:131-139` — paso h VALIDACIÓN QUESTION GATES (fuente canónica CORE-003)
- `.opencode/task-system/prompts/question-gates.md:20-26` — routing table quién pregunta (orquestador vs sub-agente BLOQUEO)
- `.opencode/task-system/prompts/subagent-recovery.md:79-88,95-101` — RESULTADO contrato BLOQUEO+GATES_EVALUADOS+SIN-FORMATO validation
- `.opencode/task-system/prompts/pipeline-full.md:249,87-88` — GATES_EVALUADOS en RESULTADO + Gate D spec-first reference
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping (LIFECYCLE_SKILLS BUILD)
- `SKILLS-MANIFEST.md` — catálogo 193 skills (162 + 31)
- `docs/plans/2026-08-28-master-pipeline-optimization.md:82-107` — Task 3 definición + contrato + pre-mortem + risk register
- `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md` §12 — recitation plantilla canónica RESULTADO

## Context Save Point
- **Fecha:** 2026-08-28T17:00
- **Branch:** develop (o current — git status limpio, sin cambios pendientes a prompts)
- **CI pendiente:** ninguno — Select-String 1/4/4/3/1/2 todos ✅, Get-Content 131-139 8 líneas ✅, `cargo check -p vantadb` sanity ✅ (no toca Rust)
- **Decisiones:** 1 decisión Spec (orquestador centralizado pipeline-run h vs sub-agente distribuido) resuelta por evidencia (pipeline-run ya centralizado) — ponytail rung 1
- **Problemas conocidos:** Ninguno — enforcement completo desde a275267c; task file nuevo + plan Estado actualizado; verify 100% idempotente
- **Próxima tarea:** CORE-004 — Task File Template Completo (siguiente en plan secuencial DAG)
