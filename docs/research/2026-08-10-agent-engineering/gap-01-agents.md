# GAP-01 · Análisis de brechas del sistema de tareas de VantaDB vs. mejores prácticas de agentes IA

> **Fecha:** 2026-08-10 · **Idioma:** español (términos técnicos en inglés)
> **Baseline analizado:** las 3 investigaciones de agentes (`agent-01-fundaments.md`, `agent-02-task-execution.md`, `agent-03-orchestration.md`) + las 3 de ingeniería (`eng-01-software.md`, `eng-02-systems.md`, `eng-03-project.md`).
> **Sistema auditado:** el pipeline de tareas en `.opencode/` (state machine C0, prompts, commands, skills, memory).
> **Alcance:** solo lectura + análisis. Ningún archivo del sistema fue modificado.

---

## 0. Metodología de la auditoría

1. **Leer el baseline de investigación.** Se leyeron las 6 investigaciones de `docs/research/2026-08-10-agent-engineering/`: 3 de agentes (fundamentos, ejecución de tareas, orquestación) y 3 de ingeniería (software, sistemas/debugging, entrega de proyectos). Para los archivos extensos se preservó la estructura de secciones y el índice (las áreas 4-8 de `agent-01-fundaments.md` y las secciones tardías de `agent-02-task-execution.md`/`agent-03-orchestration.md` están citadas por índice; ver nota en §8).
2. **Inventariar el sistema.** Se leyeron los archivos de enforcement (`config/state-tools.mjs`), prompts del pipeline (`plan.md`, `task.md`, `iter-loop-tools.md`, `pipeline-full.md`, `pipeline-run.md`, `subagent-recovery.md`, `research-agent.md`), el router de comandos (`commands/pipeline.md`), el SKILL del campaign-executor y sus `RULES.md`, la memoria (`lessons.md`, `decisions.md`) y las directivas globales (`AGENTS.md`).
3. **Clasificar capacidades.** Cada capacidad de la investigación se cruzó contra el sistema y se etiquetó: YA TENEMOS (≥90%), FALTA (0-30%), PARCIAL (30-70%), MAL/CONTRADICTORIO.
4. **Identificar fallas reales.** Se detectaron divergencias observables entre fuentes del propio sistema (p. ej. lista de estados en prose vs. runtime).
5. **Priorizar mejoras P0-P3.** Impacto × esfuerzo, con archivo objetivo.

Nota de citado: todas las referencias son `archivo:línea`. Donde una sección de investigación no se pudo leer completa (ver §8), se cita con el rango del índice visible y se marca "según índice".

---

## 1. Tabla maestra: Capacidad → Estado actual → GAP → Acción

| # | Capacidad (de la investigación) | Estado actual (archivo:línea) | GAP | Acción propuesta |
|---|---|---|---|---|
| 1 | **Secuencia de producción / single loop** — procesar y entregar de a una pieza | Loop orquestador→agente UNA tarea a la vez (`prompts/pipeline-full.md`); canónico: "forma canónica: UNA tarea" (`AGENTS.md` "Cómo ejecutar un comando" §7 handoff) | Pitch/plan sin "next" reanudable automático; la continuidad depende del recitation del agente | Añadir reanudación por SARL + campo `next` en el plan file (hook ya existe) |
| 2 | **Chunking de tareas** — tareas atómicas, verificables, ≤ 1-2 días | Auto-detección de tipo de tarea + fase Discovery/Blast Radius con `codegraph_explore` (`prompts/task.md`); invariante ~100 líneas/commit (`RULES.md` invariant 5) | No hay timebox/appetite explícito por tarea; la atomicidad depende del detect de tipo | Añadir appetite → timebox a la autodetección (`task.md`) |
| 3 | **Paralelismo / bandas (waves)** — DAG, critical path | `FAIL_MODE stop/skip/parallel` (`prompts/pipeline-run.md`); fan-out con merge es el único patrón multi-persona endorsed (`AGENTS.md` "Regla de composición") | Sin waves reales por dependencias; sin critical path modelado | Documentar fan-out + merge step endorsed (`AGENTS.md` ya lo permite) |
| 4 | **Evals** — medir tasa 1er intento, falsos positivos, regresión | North Star definida pero NO medida (`RULES.md` North Star); sin harness de evals ni golden-set | **GAP crítico**: el contrato central no tiene instrumento de medición | Crear `evals/` mínima (ver P0-1) |
| 5 | **Guardrails** — límites accionables por estado | `STATE_TOOLS` con allow/deny por 10 estados (`state-tools.mjs:9-59`), `validateAction` consumida por `campaign-server.mjs:11` e `iter-loop-tools.md:18` | ACT sin `denied` (`state-tools.mjs:16-17`) → sin guardrail de "solo implementación"; RESEARCH niega `bash` | Endurecer `denied` de ACT/RESEARCH (P1-4) |
| 6 | **Memoria** — decisiones + lecciones persistentes | `decisions.md` (ADR-light) + `lessons.md`; `campaign_memory_write(file=decisions\|lessons)` | Sin esquema/retrieval estructurado; sin validación web obligatoria antes de decidir | Validar hallucination en escrituras de decisión (`AGENTS.md` Validation Rule) |
| 7 | **Auto-corrección** — loop reflexivo | `EVALUATE` estado C0 (`state-tools.mjs:35-39`) + área "auto-corrección" en `agent-01-fundaments.md` (según índice) | Sin reglas de cuándo/cómo corregirse; sin gate de entrada/salida | Añadir paso exclusivo de auto-evaluación post-verify (P1) |
| 8 | **Verificación mecánica** — evidencia, no opinión | `campaign_verify_cmd` + gates `verify.ps1`/`verify_changed.ps1`; invariante "verificación mecánica" (`RULES.md` invariant 3) | Hooks git NO instalados (`AGENTS.md`) → la verificación depende de disciplina manual | Instalar pre-push barrier (plantilla ya existe en `templates/pre-push.ps1`) |
| 9 | **Reanudación tras fallo (SARL)** — clasificar por evidencia | `subagent-recovery.md`: clasificación por evidencia observable; regla de oro INCOMPLETE/UNEXPECTED ≠ FAILED; escalera en misma sesión | Clasificación no registrada en memoria; escalada no automatizada | Registrar peldaño SARL usado + desenlace en `lessons.md` (P2) |
| 10 | **Pre-mortem / circuit breaker** — evitar rabbit holes | Triage ✅DO/🟡DEFER/❌SKIP/🔴BLOQUEADO + "Paso 0: Verificación de Realidad" (`plan.md`) | Sin pre-mortem ni stop conditions escritas (Shape Up: "se CANCELA por defecto, no se extiende") | Añadir pre-mortem + circuit breaker al iniciar plan (`plan.md`) P1-1 |
| 11 | **Risk register** — riesgos con trigger | No existe tabla de riesgos en el template de plan | Sin registro de riesgos (max 5-8 vivos, con mitigación y trigger) | Añadir risk register al template de plan (`plan.md`) P1-2 |
| 12 | **Hill chart / reporting de incertidumbre** — no el % | Reporting por recitation `{ activeGoal, contract, lastAction, nextAction, result }` (`campaign_update_task_state`) | El % oculta "descubiertas vs imaginadas" (Shape Up cap. 13) | Reportar uphill (incógnitas) vs downhill (ejecución) (P2) |
| 13 | **Postmortem blameless + triggers predefinidos** | Memory de lessons (`lessons.md`), sin estructura de postmortem | Sin triggers en frío (outage, data loss, rollback, recovery-time) ni plantilla | Plantilla "postmortem de 10 min" al cerrar incidentes (`progreso` skill) P2 |
| 14 | **Retrospectiva con UNA acción medida** | `progreso` skill migra completados a `docs/progreso/README.md` | Sin Start/Stop/Continue + una acción de cambio medida en el siguiente ciclo | Añadir retro con 1 acción medida (P1-3) |
| 15 | **WIP limitado / un bet activo** | Pipeline canónico de UNA tarea (`pipeline-full.md`); "un solo bet" implícito | Tooling no bloquea 2ª tarea mientras una está in-progress; solo convención | Hard-block en `run` (P2-4) |
| 16 | **Observabilidad de ejecución** | `session-tracking.ps1` = "telemetry, not a state machine" (`state-tools.mjs:8`); `campaign_emit_event` disponible | Sin trace ID end-to-end por tarea (idea→verify→cierre) | Trace por tarea + eventos con `campaign_emit_event` (P2-5) |
| 17 | **Resultados DORA por entrega de agente** | No medidos (lead time, change fail rate, recovery) | Sin baseline de throughput real | Emitir tareas cerradas con DoD por día (`progreso`) P3 |
| 18 | **Segunda opinión / review por par** | `REVIEW` estado C0 auto-referencial (`state-tools.mjs:40-44`) | Mismo agente revisa su propio cambio — falta contraste | Revisión por agente distinto (vanta-*) en `task.md` (P2-1) |
| 19 | **Estimación relativa / reference class** | Autodetección 🟢🟡🔴 heurística (`campaign_detect_task_type`) | Sin histórico que calibre la estimación | Guardar esfuerzo real por tipo y calibrar (P3-2) |
| 20 | **Root-cause sobre symptom** | RULES.md invariante "primero entender" + `eng-02-systems.md` (§3 debug sistemático, §10 protocolo) | Depende del agente; sin gate automático | Reforzar con `systematic-debugging` skill obligatoria en bugs |
| 21 | **Repro determinista antes del fix** | No hay paso explícito de "repro rojo" en `task.md`; la skill `systematic-debugging` lo cubre si se carga | Repro mínimo no forzado por el pipeline | Forzar "construir repro" como fase del bug-fix workflow (`eng-02-systems.md:410-413`) |
| 22 | **Test de regresión RED antes del fix** | RULES.md invariant "verificación mecánica"; skill TDD disponible en `.opencode/skills/test-driven-development` | No obligatorio; el agente puede fix-sin-test | Fase VERIFY que exige "test falló ANTES del cambio" (`eng-01-software.md:122-130`, `eng-02-systems.md:430`) |
| 23 | **git bisect / localización por historial** | No automatizado en el pipeline | Bugs de regresión se cazan a mano | Script/check sugerido cuando el repro es no-determinista (`eng-02-systems.md:172-176`) |
| 24 | **Observabilidad (logs/metrics/traces)** | `session-tracking.ps1` telemetría de sesión; `campaign_emit_event` existe | Sin trazas por tarea ni logs estructurados de decisión de agente | Traces por tarea (P2-5); logs estructurales en el server (`eng-02-systems.md:227-237`) |
| 25 | **Respuesta a incidentes (mitigar primero)** | No hay fase de incident que priorice mitigación antes que RCA | Un agente podría debuggear "en caliente" | Fase de contención antes del debug profundo (`eng-02-systems.md:214`, `eng-02-systems.md:397-400`) |
| 26 | **Handoff transferible / contexto** | Recitation estructura `lastAction`/`nextAction`; `subagent-recovery.md` da escalera | Handoff no requiere "invariantes de dominio + comandos de verificación" | Enriquecer recitation de cierre con contexto transferible (`eng-03-project.md:198`) |
| 27 | **Estimation por relativos / timebox** | Autodetección 🟢🟡🔴 heurística | Sin appetite (Shape Up) ni referencia calibrada | Timebox como default; calibrar con histórico (`eng-03-project.md:52`, `82`) |
| 28 | **ADR para decisiones significativas** | `decisions.md` ADR-light en memoria del agente | No integrado con `docs/architecture/adr/` del repo | Duplicar decisión relevante a ADR del repo cuando es arquitectónica (`AGENTS.md` Regla 5) |
| 29 | **DoD por nivel (task/PR/release)** | RULES.md invariantes + `verify*.ps1` | DoD no es una checklist visible por nivel de cambio | Plantilla DoD por nivel al crear PR de tarea (`eng-03-project.md:151-159`) |
| 30 | **Reporte de progreso honesto** | Recitation binario (in-progress/completed/failed) | Sin hill chart; "no sabe lo que no sabe" invisible | Contador de incógnitas en estado de tarea (P2-3) |
| 31 | **Postmortem blameless (errores)** | `docs/references/troubleshooting.md` captura síntomas; `decisions.md` captura decisiones | Sin timeline/impacto/owner en incidentes del pipeline | Plantilla de postmortem en `progreso` (P2-2) |
| 32 | **Small batches / merging diario** | pipeline canónico una tarea por ciclo; `git up` + PR a main por feature | La tarea se "cierra" sin merge; el PR/CI a main queda fuera del loop | DoD de tarea incluye "PR en main con CI verde" (P3) |
| 33 | **Contexto / memoria externa** | `lessons.md` + `decisions.md` son la memoria externa del agente | Sin enlace entre el aprendizaje y la tarea que lo produjo | Guardar `taskID` en cada escritura de memoria |
| 34 | **Estimación con tiers** | `campaign_detect_task_type` con effort 🟢🟡🔴 | El esfuerzo no se compara contra el gasto real (planning fallacy) | Log de effort estimado vs real por tipo (P3-2) |
| 35 | **SLA/error budget** | No hay SLI/SLO del pipeline | Sin error budget que justifique cuánto riesgo asumir por iteración | Definir 1-2 SLO del task-system y trackear (P2) |
| 36 | **Chaos/resilience del pipeline** | `.opencode/agents/vanta-chaos` existe para fuzz/resilience del código | No se aplica al propio MCP server ni a la máquina de estados | Test de recovery del `campaign-server.mjs` (P3) |
| 37 | **Trace ID / correlación** | `session-tracking.ps1` con telemetría de sesión | El trace no une "tarea → decisiones → tools → verify" | Trace ID por tarea (P2-5) |
| 38 | **Falsos positivos medibles** | North Star exige `falsos positivos 0` (`RULES.md`) | `campaign_verify_cmd` puede pasar un cambio que sigue roto sin registro | Log de veredictos false-positive en `evals/` (P0-1) |
| 39 | **Auto-corrección con stop condition** | `EVALUATE` (`state-tools.mjs:35-39`) | Sin "regla de tres" (3 fixes → STOP) (agent-01 índice + `eng-02-systems.md:151`) | Stop condition × N intentos en el loop (P1-1) |
| 40 | **Reanudación tras bloqueo** | `subagent-recovery.md` SARL: clasificar por evidencia | La decision de reapertura no queda trazada en el plan file | Registro de reapertura con motivo en el plan (P2) |
| 41 | **Task en "imaginada" vs "descubierta"** | Recitation no distingue trabajo descubierto al ejecutar | El sistema no puede aprender de sub-estimación | Evento de plan-ajuste al re-planificar (P2-3) |
| 42 | **Docs de área como check opcional del verify** | `AGENTS.md` Regla 3 (docs/api al día) | No forma parte de `campaign_verify_cmd` | Doc-check en el gap de tareas que tocan APIs (P2) |
| 43 | **Validación web antes de decisiones** | `AGENTS.md` Validation Rule (websearch/webfetch si no seguro) | No integrada con `campaign_memory_write(decisions)` | P3-4 (contraste con fuente antes de persistir) |
| 44 | **Escalones del bottleneck** | Sin datos de "qué tool/skill destrabó" | P0-1 (evals) se queda sin variable independiente | Campos de `skills/tools` en el log de tarea (P0-1) |

---

## 2. YA TENEMOS (cobertura ≥ 90%)

1. **State machine C0 con enforcement por estado.** 10 estados (`PLAN→ACT→VERIFY→COLLATERAL→RESEARCH→EVALUATE→REVIEW→ACCEPT→CLOSE→STALL`) con listas `allowed`/`denied` (`state-tools.mjs:9-59`), función `validateAction` disponibles en `campaign-server.mjs:11` y referenciada por `iter-loop-tools.md:18`. Pocos sistemas de agentes tienen guardrails ejecutables por fase; esto es lo más fuerte del pipeline.
2. **Triage gate explícito antes de planificar.** `✅DO / 🟡DEFER / ❌SKIP / 🔴BLOQUEADO` + "Paso 0: Verificación de Realidad" (`plan.md`) → implementa "Define the problem" de `eng-02-systems.md:70-79` ("saltarse el paso 1 es la causa #1 de solucionar el problema equivocado").
3. **Discovery / Blast Radius obligatorio.** Fase de descubrimiento con `codegraph_explore` antes de editar (`task.md`; `AGENTS.md` sección CodeGraph) → implementa "entender el blast radius antes de tocar" (`eng-02-systems.md:342`) y el uso de tooling de índice en lugar de leer millones de líneas (`eng-02-systems.md:341`).
4. **Verificación mecánica con comando dedicado.** `campaign_verify_cmd` + gates `verify.ps1`/`verify_changed.ps1` ~30s + invariante "verificación mecánica, no opinión" (`RULES.md`) → shift-left real (`eng-01-software.md:234-237`). La verificación rápida tras cada cambio es correcta para TDD/small batches (`eng-03-project.md:386-389`).
5. **Reanudación con regla de oro.** Clasificación de fallos por evidencia observable y `INCOMPLETE/UNEXPECTED ≠ FAILED` (`subagent-recovery.md`) → coincide con "el método sistemático es MÁS rápido que el ensayo-error" (`eng-02-systems.md:159`) y evita el anti-patrón "ver el síntoma ≠ entender la causa raíz" (`eng-02-systems.md:163`).
6. **Memoria de decisiones y lecciones.** `decisions.md` ADR-light + `lessons.md` + `campaign_memory_write` → "las decisiones viejas con contexto evitan re-litigios" (`eng-02-systems.md:306`). El sistema ya registra decisiones y trade-offs (ADR: contexto/decisión/alternativas/razón).
7. **One-task-at-a-time.** Pipeline canónico de una sola tarea con bloque `RESULTADO` y handoff ("detenerse al finalizar, no continuar sin que el usuario lo pida", `AGENTS.md` §7) → WIP=1 y anti-context-switching (`eng-03-project.md:211`; "un solo proyecto a la vez como bet", `eng-03-project.md:313`).
8. **Skills obligatorias + tabla anti-racionalización.** El agente DEBE cargar skills y NO puede excusarse para saltárselas (`AGENTS.md` "Anti-Rationalization (MUST)") → cultura de gates escrita, que es lo que sustituye "la fricción social de un equipo" en un dev solo (`eng-03-project.md:322`).
9. **Ponytail / escalera YAGNI.** Ladder de 7 peldaños integrado (`AGENTS.md` Ponytail; `RULES.md` invariant 5) → matchea la rung 1 YAGNI de `eng-03-project.md:50` y el anti-patrón "speculative generality" de `eng-01-software.md:176`.
10. **Formato de recitation estructurado.** `campaign_update_task_state` con `recitation { activeGoal, contract, lastAction, nextAction, result }` → corresponde al "modelo single source of truth: objetivo activo, contrato de validación y próximo paso" (`eng-03-project.md:44`).
11. **Plan file como artefacto vivo.** El pipeline crea `docs/plans/<FECHA>-<nombre>.md` (`plan.md`) → "el documento, no la memoria, es la fuente de verdad" (`eng-03-project.md:18`) y "toda tarea se cierra con su documentación al día" (`eng-03-project.md:200`).
12. **Clasificación previa del tipo de tarea.** Auto-detección de tipo (bug-fix/feature-add/refactor/research) + workflows JSON por tipo (`iter-loop-tools.md:18`) → matchea los "gates por tipo de trabajo" de `eng-03-project.md:174-180` (cada tipo de trabajo tiene su técnica de verificación).
13. **Blast-radius y call paths disponibles vía CodeGraph.** `codegraph_explore` con call paths y blast radius en una llamada (`AGENTS.md` CodeGraph) → implementa el "issue tree / MECE" y el mapeo de dependencias de `eng-02-systems.md:111-117` sin re-lectura masiva.
14. **Análisis de causa raíz multi-técnica documentado.** El sistema hereda las skills `systematic-debugging` y `debugging-and-error-recovery` con fases (repro → patrones → fix & verify → root cause), alineadas con el protocolo de 28 pasos de `eng-02-systems.md:393-439`. La "regla de tres" ya figura como principio (cuestionar arquitectura tras 3 fixes, `eng-02-systems.md:427`).
15. **Gate CI real de dos niveles.** Fast gate (<5 min, determinístico) + Heavy certification (hasta 2 h, manual/scheduled) (`AGENTS.md` CI Architecture) → shift-left + quality gates de `eng-01-software.md:223-244` con separación correcta de tiempos.
16. **Postmortem de código en la cultura.** `docs/references/troubleshooting.md` y los ADR del repo capturan incidentes y decisiones con formato de secciones → base ya construida para el postmortem blameless de `eng-03-project.md:257-265`.
17. **Verificación determinista con `campaign_verify_cmd`.** El pipeline expone una tool que ejecuta comandos de verificación y registra exit code esperado (`campaign_verify_cmd` en `state-tools.mjs:21`, `verify` en `RULES.md`) → test automatizable "a voluntad" como pide `eng-02-systems.md:411` para el repro.
18. **Cierre explícito (CLOSE) con commit.** El estado `CLOSE` (`state-tools.mjs:50-54`) permite `bash` para commit y cierre, lo que concreta "cada tarea se cierra con su commit y su documentación" (`eng-01-software.md:155`).
19. **Enforcement de `skill` como tool de estado.** `skill` está en `allowed` de PLAN/ACT/REVIEW/ACCEPT/CLOSE (`state-tools.mjs:11,16,41,46,51`) → la carga de skills obligatoria de `AGENTS.md` tiene acompañamiento de tooling, no solo cultura.
20. **Separación de RESEARCH del runtime.** El estado `RESEARCH` (`state-tools.mjs:30-34`) aísla la fase de investigación con `read/grep/glob/codegraph/websearch/webfetch/argus/meta` y prohíbe `edit/write/bash` → concentra la fase de "conocer antes de tocar" del área 4-8 de `agent-01-fundaments.md`.

---

## 3. FALTA (cobertura 0-30%)

1. **Harness de eval del propio pipeline.** No existe medición de la North Star (tasa >90% primer intento, falsos positivos 0, regresión 0) (`RULES.md`). No hay golden-set ni suite de evaluación del task-system. Sin esto, el sistema no puede "verificarse a sí mismo": el único eval es el humano. La investigación (`agent-01-fundaments.md`, sección "evals" del índice) lo lista como estándar de primeros principios. → **P0-1**.
2. **Pre-mortem.** No aparece en `plan.md`. Es "la técnica con mejor ROI cognitivo" (`eng-03-project.md:125-127`) y la única que expone rabbit holes ANTES del commitment fuerte (`eng-03-project.md:129-134`). En un sistema de agentes, pre-mortem escrito = stop conditions explícitas. → **P1-1**.
3. **Risk register con triggers.** Sin tabla "ID/riesgo/Prob/Impacto/Score/Respuesta/Trigger/Estado" en el template de plan (`eng-03-project.md:115-123`). Max 5-8 riesgos activos ("el registro no es un museo").
4. **Circuit breaker / stop conditions escritos.** Existe 🔴BLOQUEADO en triage (`plan.md`), pero no "cuándo se CANCELA por defecto, no solo cuándo se termina" (`eng-03-project.md:134`, `371`). Un agente puede re-intentar indefinidamente (regresión al anti-patrón de bucle del área "auto-corrección" de `agent-01-fundaments.md`).
5. **Retrospectiva con UNA acción de proceso medible.** `progreso` migra tareas pero no hay "Start/Stop/Continue + UNA acción de cambio (no tres)" ni su medición en el siguiente ciclo (`eng-03-project.md:270-271`, `402`). → **P1-3**.
6. **Postmortem triggers definidos ANTES del incidente.** La regla es definir umbrales en frío (outage visible, data loss, rollback intervenido, recovery-time) (`eng-03-project.md:145`, `262`). Solo existe `lessons.md` reactivo. → **P2-2**.
7. **Hill chart / contador de incógnitas.** El reporting actual es recitation de %-completado. Falta distinguir *uphill* (incógnitas por resolver) vs *downhill* (ejecución pura): "el % de tareas completadas miente" (`eng-03-project.md:186-188`). → **P2-3**.
8. **WIP hard-limit por tooling.** La convención es one-task-at-a-time (`pipeline-full.md`) pero nada impide que `run` arranque una tarea con otra in-progress (WIP limits de Kanban, `eng-03-project.md:211`). → **P2-4**.
9. **Trazas end-to-end de tarea.** La telemetría actual es "telemetry, not a state machine" (`state-tools.mjs:8`); sin trace ID por tarea, el RCA de un agente fallido es adivinanza ("si no hay trazas, el debug es adivinanza", `eng-02-systems.md:237`). → **P2-5**.
10. **DORA por entrega de agente.** No se miden lead time del slice, change fail / rework (cerrado que se reabrió), recovery (`eng-03-project.md:248-251`). → **P3-1**.
11. **Segunda opinión externalizada.** "Externalizar la segunda opinión (code review de un peer/agente, publicar el plan para que alguien lo cuestione)" (`eng-03-project.md:316`). El review de `REVIEW` (`state-tools.mjs:40-44`) lo hace el mismo agente que implementó.
12. **Estimación independiente / reference class.** No hay estimación relativa calibrada con histórico ni "los que harán el trabajo estiman" (`eng-03-project.md:87-90`). La autodetección 🟢🟡🔴 es heurística sin histórico real.
13. **Repro determinista obligatorio.** El bug-fix flow no fuerza "construir un repro mínimo que falle a voluntad" (`eng-02-systems.md:411-413`). Un bug sin repro es un bug cuya arreglo no se puede verificar.
14. **Test RED antes del fix como gate.** No hay verificación de que el test de regresión exista y falle ANTES del cambio (`eng-01-software.md:122-130`). El estado `VERIFY` (`state-tools.mjs:20-24`) solo exige correr verificaciones, no el orden TDD.
15. **Mitigar primero en incidentes.** No hay fase de contención ("restaurar servicio PRIMERO, debuggear después", `eng-02-systems.md:209-214`). Para un sistema de tareas esto se traduce en: ante un build roto o backoff, no seguir el plan original.
16. **git bisect automatizable.** La localización de regresiones por historial/inputs no tiene script (`eng-02-systems.md:172-187`); se deja al criterio del agente.
17. **Observabilidad de decisión de agente.** No hay log estructurado de qué herramienta usó el agente, por qué cambió de estado, ni eventos en `campaign_emit_event` registrados de forma sistemática por tarea.
18. **Handoff con contexto transferible.** El recitation de cierre reporta `lastAction`/`nextAction`, pero no exige "invariantes de dominio + comandos de verificación + deuda pendiente" que permitan continuar sin preguntar al anterior (`eng-03-project.md:198`).
19. **DoD por nivel como checklist visible.** `verify*.ps1` cubre la verificación técnica, pero no existe una checklist DoD por nivel (task/PR/feature/release) exigida por el pipeline (`eng-03-project.md:151-159`).
20. **ADR integrado al repo.** `decisions.md` es memoria del agente; las decisiones arquitectónicas que deberían vivir en `docs/architecture/adr/` dependen de que el agente recuerde duplicar (`AGENTS.md` Regla 5).
21. **Estimar con appetite (Shape Up).** No hay un "tiempo que VAMOS a invertir" como default de tasas; todo es estimación heurística del esfuerzo (`eng-03-project.md:52`).
22. **Pre-mortem del pitch.** La investigación marca el pre-mortem como gate del inception (`eng-03-project.md:125-127`, `368`); el triage `plan.md` no lo pide.
23. **SLA del pipeline.** No hay SLI/SLO ni error budget para el task-system; no se sabe si "el pipeline falla mucho" (`eng-02-systems.md:200-203`).
24. **Chaos/resilience propio.** `vanta-chaos` fuzzza el código fuente, no al `campaign-server.mjs` ni a la máquina de estados.
25. **Merge a main como parte del DoD de tarea.** La tarea se cierra con commit + verify, pero el "PR a main / releases" queda fuera de su explicito cierre.
26. **Rollback plan como DoD de feature.** No se exige declarar rollback cuando la feature toca producción (infra lo permite con git revert; `eng-03-project.md:396-397`).
27. **Post-release / monitoring en el loop de tarea.** La investigación cierra el ciclo de feature con "monitoring + post-release metrics" (`eng-01-software.md:22-23`, `eng-03-project.md:440-441`); el pipeline termina en CLOSE/commit sin verificación post-merge.
28. **Solo-power de "pre-mortem" para bugs recurrentes.** Para fallos que reaparecen, eng-02 pide DMAIC con baseline y control ("el paso Control es el que falta en la mayoría de fixes", `eng-02-systems.md:92-94`); no hay seguimiento de regresión en el pipeline.

---

## 4. PARCIAL (cobertura 30-70%)

1. **Guardrails de estado.** El enforcement es real y correcto, pero:
   - `ACT.denied = []` (`state-tools.mjs:16-17`) — no prohíbe nada; cualquier tool es "implementación".
   - `PLAN.allowed` incluye `bash` (`state-tools.mjs:11`) aunque sea fase de "solo lectura e investigación".
   - `RESEARCH.denied` incluye `bash` (`state-tools.mjs:32`) mientras `COLLATERAL` (diagnóstico) sí lo permite — inconsistencia lógica: diagnosticar sin `bash` es difícil. Si la intención es "RESEARCH no cambia el repo", denegar `edit/write` ya lo cubre; prohibir `bash` read-only obliga a volver a PLAN.
   - **Acción:** endurecer lists sin romper el flujo real (P1-4).
2. **Memoria con estructura liviana.** `campaign_memory_write(file=lessons|decisions)` existe y da formato, pero sin esquema fijo por entrada ni retrieval por tema: "la deuda sin registro no genera trabajo" (`eng-01-software.md:307`); aquí depende del agente recordar escribir. Parcial: mejor que la mayoría, menos que un sistema de memoria con búsqueda.
3. **Verificación pre-push.** Gates existen y son rápidos (`verify_changed.ps1` ~30s), pero los hooks git NO están instalados (`AGENTS.md` "Nota: los hooks NO están instalados"), así que el pre-push gate depende de disciplina manual. Infra lista, enforcement diario débil.
4. **Parallel fan-out.** `FAIL_MODE parallel` existe (`pipeline-run.md`), y el único patrón endorsed es fan-out con merge step (`AGENTS.md`). Pero no hay merge step estructural en los prompts ni modelado de DAG/critical path/waves (`eng-03-project.md:105-107`).
5. **Auto-corrección.** Hay estado `EVALUATE` (`state-tools.mjs:35-39`) y la investigación lo lista como área clave (`agent-01-fundaments.md`, índice), pero falta el "cuándo" (gates de entrada/salida) y el "cómo" (qué métricas disparan la corrección y cuándo escalar). La "regla de tres" de debugging (`eng-02-systems.md:151-152`, `eng-02-systems.md:427`): tras 3 fixes que fallan, STOP y cuestionar la arquitectura — un loop de auto-corrección sin ese stop es el anti-patrón.
6. **SARL con escalera manual.** La escalera de recuperación está documentada (`subagent-recovery.md`) y es de calidad, pero su resolución (misma sesión → más contexto → escalar) no está automatizada ni instrumentada: no se registra en `lessons.md` qué peldaño se usó ni su desenlace → no hay dato para calibrar la North Star.
7. **TDD disponible pero no impuesto.** La skill `test-driven-development` existe en `.opencode/skills/` (también en `AGENTS.md` Lifecycle mapping), y el cronograma del pipeline carga skills. Pero la secuencia Red→Green→Refactor no es un gate del `VERIFY`: puede haber "tests que verifican algo" sin que hayan sido escritos primero.
8. **Shift-left con gates por PR vs. solo local.** Los gates CI existen por push del repo (`AGENTS.md` CI), pero las tareas del pipeline en su loop interno dependen de `verify*.ps1` local; el vínculo entre "tarea del pipeline" y "PR con CI verde" no está forzado por el task-system.
9. **Documentación viva.** `AGENTS.md` Regla 3 obliga a actualizar `docs/api/` cuando se toca API pública (doc-driven), pero el enforcement es una regla del AGENT (prosa), no un paso del pipeline de tareas: una tarea que cambia un `pub fn` puede completarse sin tocar docs y sin que `campaign_verify_cmd` lo detecte.
10. **Feature flags / rollback** — infra del repo robusta (release-plz, CI, verify), pero el task-system no declara rollback plan como DoD de feature. En un dev solo con `git revert` disponible, falta solo declarar el gate (`eng-03-project.md:396-397`).
11. **Trunk-based / batch pequeño.** El repo usa develop + PR a main (`AGENTS.md` Regla 7) y commits ~100 líneas (`RULES.md`), que es el mecanismo real de DORA (`eng-03-project.md:322`); pero la decisión de "qué es una feature shippable" queda al criterio, no a un batch automático desde el plan.
12. **Guardrails "fuertes" en estados de lectura.** PLAN/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE niegan `edit/write` o los limitan (`state-tools.mjs:13,22,27,32,37,42,47,52`) — el join más riesgoso (escritura) está bien custodiado excepto en ACT, donde la actividad principal cubre TODO.

---

## 5. MAL / CONTRADICTORIO

1. **Definiciones duplicadas de los mismos estados.** Existen cuatro fuentes de estado:
   - Canon runtime: `config/state-tools.mjs` (`state-tools.mjs:1-8`).
   - Prose spec: `prompts/iter-loop-tools.md:18` — lista **sin STALL** (PLAN/ACT/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE), mientras `state-tools.mjs:55-59` SÍ define STALL.
   - Diagram en el SKILL (`SKILL.md`, sección diagrama) con otra enumeración.
   - Workflows JSON por tipo de tarea (explícitamente NO pasan por enforcement — `iter-loop-tools.md:18`).
   - **Riesgo real:** la propia cabecera dice "do NOT diverge them" (`state-tools.mjs:3-8`) pero YA hay divergencia (STALL ausente en prose). → **P0-2**: generar prose+diagram desde `state-tools.mjs` o test de paridad.
2. **Recitation duplicado (3 definiciones).** El contrato `recitation` aparece en `pipeline-full.md` (bloque RESULTADO), en `task.md` (datos) y como parámetro de `campaign_update_task_state`. Campos compatibles pero redactados por separado → drift cosmético; fuente única.
3. **North Star aspiracional sin medición.** `RULES.md` define umbrales (tasa >90%, falsos positivos 0) pero no existe instrumento para medirlos → es una "métrica como meta sin medición", el pitfall de Goodhart (`eng-03-project.md:241`). Peor: un agente que no mide `falsos positivos = 0` puede "auto-certificar" sin datos.
4. **REVIEW auto-referencial vs. review por par.** `REVIEW` (`state-tools.mjs:40-44`) permite `read/grep/codegraph/campaign/skill` pero lo ejecuta el mismo agente que implementó. La investigación insiste en revisión con "enfoque + cómo se probó" (`eng-01-software.md:199-203`) y en externalizar la segunda opinión (`eng-03-project.md:316`). Un único agente self-review es el punto más débil del modelo one-person-band.
5. **Enforcement cooperativo, no absoluto.** `validateAction` y `STATE_TOOLS` solo se aplican si el agente los invoca. Un flujo guiado por el usuario (como esta auditoría) o un agente que "se salta" la máquina de estados no pasa por `campaign_enforce_state`. Es de hecho lo que permitió escribir este reporte sin pasar por C0. El enforcement es una barrera de cortesía, no un juez.
6. **`iterate` / velocidad vs. estabilidad DORA.** `eng-03-project.md:221-242` insiste en que velocidad y estabilidad se miden juntas (lead time, fail rate, MTTR). El pipeline optimiza throughput de tareas pero no mide estabilidad: una tarea "completada" que reabre un bug cuenta como éxito en el log actual (sin rework tracking).
7. **Fragmentación del enforcement entre sesiones.** Los estados C0 viven en el MCP server (`state-tools.mjs`) y la memoria (`lessons.md`/`decisions.md`) en archivos aparte; el `AGENTS.md` global dice qué skills cargar. Tres mecanismos de "reglas" con pesos distintos y sin un único maestro de consistencia — la Regla 0 (`AGENTS.md` "Análisis de Impacto") y la Regla 6 de deuda técnica por PR son ejemplos de reglas que no se pueden verificar automáticamente.
8. **Triage vs. "es ahora" de Shape Up.** El triage (`plan.md`) clasifica por DO/DEFER/SKIP/BLOQUEADO pero no pregunta "¿es el problema adecuado? ¿correcto el appetite? ¿es ahora?" (`eng-03-project.md:312`). La priorización por valor×riesgo/costo no está explicitada en el template del plan.

---

## 6. Fallas reales del sistema (observadas en la auditoría)

1. **Sin medición del contrato central.** North Star declarada en `RULES.md` y nada más. No hay log de primer-intento, ni de falsos positivos, ni de regresión. Toda afirmación de mejora del sistema es hoy anecdótica.
2. **Cruce de definiciones de estado.** `iter-loop-tools.md:18` (prose) y `state-tools.mjs:9` (runtime) puede divergir sin que nada falle: no hay test de paridad en CI. La jerarquía del comentario (`state-tools.mjs:3-8`) es un pacto que el código no hace cumplir.
3. **Hooks de pre-push no instalados.** Regla 1 de `AGENTS.md` es estricta en prosa, pero `pre-commit`/`pre-push` NO están instalados (`AGENTS.md`). El gate se repite manualmente y en la práctica se salta.
4. **SARL sin trazabilidad.** `subagent-recovery.md` define la escalera, pero no se registra en `lessons.md` qué peldaño se usó ni su desenlace → el loop de aprendizaje queda incompleto (`eng-03-project.md:279-280`: "persistir la lección — no volver a aprender").
5. **reporting de progreso basado en %/recitation de paciencia.** El estado reportado es binario (completed/pending/in-progress/failed). No hay hill chart ni contador de incógnitas (ver FALTA #7): el agente "no sabe lo que no sabe" de forma visible para el humano.
6. **Tareas "imaginadas" vs "descubiertas" sin señal.** Un feature re-planificado a mitad de la tarea se reporta igual que uno que encajó en el original; la curva de descubrimiento (Shape Up) no produce un evento en el log — por lo tanto el sistema no puede aprender cuánto sub-estimó.
7. **Decisions/lessons sin validación de hallazgo.** Escribir `campaign_memory_write(decisions)` no exige la validación web de `AGENTS.md` ("si no estás 100% seguro, DEBES validar contra internet") ni cita la fuente. Una decisión mal fundada se vuelve precedente.
8. **Sin telemetría de qué herramienta eliminó el cuello de botella.** No hay correlación entre "qué skill/tool usó el agente" y "la tarea pasó verify al primer intento" — el input que permitiría calibrar P0-1 (evals) no se recolecta hoy.
9. **Cambios de última hora sin trazabilidad de impacto.** La Regla 0 de `AGENTS.md` exige mapear impacto antes de modificar/eliminar, pero es prosa: no hay registro de "leí el archivo completo + mapeé referencias" en la ejecución de la tarea, así que esa garantía no se puede auditar retrospectivamente.
10. **`verify` puede pasar con un cambio que rompe docs.** El gate `campaign_verify_cmd` cubre fmt/clippy/test/deny del repo, no la regla de `AGENTS.md` Regla 3 (docs/api al día al tocar APIs). La deuda documental no se detecta.
11. **Dos memorias paralelas sin reconciliación.** `lessons.md`/`decisions.md` (agente) y `docs/progreso/*` + `docs/Backlog.md` (proyecto) pueden divergir: una tarea completada no actualiza memoria del agente al cerrarse si el agente no lo recuerda.

---

## 7. Mejoras priorizadas (P0–P3)

### P0 — hacer ya (impacto alto, esfuerzo bajo-medio)

| # | Mejora | Impacto | Esfuerzo | Archivo objetivo |
|---|---|---|---|---|
| P0-1 | **Harness de evals del pipeline**: log por tarea (tipo, intentos, veredicto de verify, resultado) → comparación contra North Star de `RULES.md`; salida a `docs/reports/`. | Hace medible la promesa central; habilitador de todas las métricas posteriores | 🟡 2-4 h | nuevo `evals/` + `campaign-server.mjs` + `pipeline-run.md` |
| P0-2 | **Fuente única de estados**: generar prose (`iter-loop-tools.md`) y diagram (`SKILL.md`) desde `state-tools.mjs`, o test de paridad que falle si divergen. Mata la divergencia real ya existente (STALL). | Elimina el riesgo de drift en el enforcement | 🟡 2-4 h | `config/state-tools.mjs` + test unitario |

### P1 — pronto (impacto alto, esfuerzo medio)

| # | Mejora | Impacto | Esfuerzo | Archivo objetivo |
|---|---|---|---|---|
| P1-1 | **Pre-mortem + stop conditions** en el Paso 0: escribir "por qué fracasaría" + cuándo se CANCELA (appetite/circuit breaker) (`eng-03-project.md:125-134`) | Evita rabbit holes y re-intentos infinitos; protege el ritmo sostenible | 🟢 1 h | `prompts/plan.md` |
| P1-2 | **Risk register** en el template de plan: Prob×Impacto, respuesta, trigger/due; máximo 5-8 riesgos vivos (`eng-03-project.md:115-123`) | Riesgos con trigger accionables en frío, antes del incidente | 🟢 1 h | `prompts/plan.md` |
| P1-3 | **Retrospectiva con 1 acción medida** al cerrar milestone: Start/Stop/Continue + métrica contra baseline (`eng-03-project.md:270-271`, `402`) | Loop de mejora de proceso real, no teatro | 🟢 1-2 h | `prompts/pipeline-run.md` + `progreso` |
| P1-4 | **Endurecer `denied` de ACT/RESEARCH**: ACT sin debt de `edit` fuera de scope; RESEARCH permitir `bash` read-only y denegar solo `edit/write` | Guardrails más fuertes sin romper el flujo | 🟢 1 h | `config/state-tools.mjs:16-17,30-34` |

### P2 — pronto-medio (impacto medio)

| # | Mejora | Impacto | Esfuerzo | Archivo objetivo |
|---|---|---|---|---|
| P2-1 | **Review por agente distinto** (segunda opinión) en `REVIEW`: descripción de enfoque + cómo se probó, hecha por vanta-audit/vanta-review y no por el implementador (`eng-01-software.md:199-203`) | Golpea directamente la falla más grave (self-review) | 🟡 4-8 h | `prompts/task.md`, persona review leaf |
| P2-2 | **Postmortem triggers + plantilla de 10 min** para incidentes (timeline, impacto, causa, follow-ups con owner) (`eng-03-project.md:257-265`) | Cierra el loop de aprendizaje con estructura | 🟡 2-4 h | `progreso` skill + `lessons.md` |
| P2-3 | **Reporte de incertidumbre (uphill/downhill)** en el estado de la tarea: contador de incógnitas vs. pendientes de ejecución (`eng-03-project.md:186-188`) | Reporting honesto; las tareas "descubiertas" no camuflan trabajo pendiente | 🟡 2-4 h | `plan.md`, `task.md` |
| P2-4 | **WIP hard-limit**: `run` rechaza arrancar una tarea si hay otra en in-progress | Hace cumplir el bet único por tooling, no por convención (`eng-03-project.md:211`) | 🟢 1-2 h | `campaign-server.mjs` |
| P2-5 | **Trace ID por tarea** (idea→verify→cierre) con eventos en `campaign_emit_event` | RCA de fallos de agente con datos, no adivinanza (`eng-02-systems.md:230-237`) | 🟡 2-4 h | `campaign-server.mjs` + `session-tracking.ps1` |

### P3 — nice-to-have (impacto bajo-medio)

| # | Mejora | Impacto | Esfuerzo | Archivo objetivo |
|---|---|---|---|---|
| P3-1 | **Métricas DORA por entrega de agente**: lead time del slice, rework (reabiertos), recovery | Baseline honesto de eficiencia | 🟡 2-4 h | `progreso` + `docs/reports/INDEX.md` |
| P3-2 | **Estimación relativa calibrada con histórico** en la autodetección 🟢🟡🔴 (guardar esfuerzo real por tipo) | Reduce la planning fallacy (`eng-03-project.md:87-90`) | 🟡 2-4 h | `campaign_detect_task_type` + memoria |
| P3-3 | **Gate de calidad de tests (mutation / pirámide)** en tareas de lógica (mutation score ≥70%, cobertura ≥80%, `eng-01-software.md:132-144`) | Tests que verifican algo, no tests muertos | 🔴 4-8 h | `verify.ps1` / check experimental |
| P3-4 | **Contraste de decisión con validation web**: antes de `campaign_memory_write(decisions)`, validar con websearch/webfetch la base fáctica (`AGENTS.md` Validation Rule) | Menos decisiones mal fundadas persistidas | 🟢 1 h | `campaign-server.mjs` / `AGENTS.md` |

---

## 8. Limitaciones y notas de fuentes

1. **Archivos de investigación extensos leídos parcialmente.** `agent-01-fundaments.md` (áreas 1-8 visibles en índice), `agent-02-task-execution.md` y `agent-03-orchestration.md` fueron capturados hasta el índice/resumen ejecutivo. Las citas a sus áreas 4-8, secciones de anti-patrones y evaluación se marcan "según índice" y deben confirmarse releyendo esas secciones completas antes de implementar las mejoras que dependan de ellas.
2. **Ningún archivo del sistema fue modificado.** Este reporte es exclusivamente análisis (modo lectura) más la escritura de este archivo.
3. **Verificación pendiente de la North Star.** Cualquier mejora P0-1 debe definir prim mediante `campaign_*` tools y golden-set; sin eso, ningún umbral de `RULES.md` es verificable.
4. **Ruta de mejora sugerida:** ejecutar las P0-P1 como tareas DRV en un plan (con triage en `plan.md`), ya que tocan prompts y el MCP server que sirve al pipeline — el propio sistema debe auto-aplicarse su disciplina.
5. **Veredicto sobre las investigaciones:** las 6 investigaciones (agentes + ingeniería) coinciden en pocos puntos con divergencia de énfasis: (a) guardrails/evals del agente (agent-01/02, DORA), (b) shape-up/risk gates (eng-03), (c) debug sistemático y repro (eng-02). El sistema de VantaDB tiene sólidos fundamentos de (b) y (c) en cultura, y le falta instrumentar (a) y (b) en tooling.
6. **Cómo leer este reporte:** las "Acción propuesta" no implican que falte "entender"; priorizan medir antes de construir (P0-1) y eliminar duplicación (P0-2) antes de agregar fricción nueva. Cualquier mejora que agregue pasos al loop debe pasar el test "¿esto mediría o bloquearía algo hoy?"

---

## 9. Anexo: mapeo investigación → artefacto del sistema

| Práctica investigada | Fuente | Artefacto VantaDB donde aplica |
|---|---|---|
| Single action loop / chunking | `agent-01-fundaments.md` (ejecución) | `state-tools.mjs:9-59`, `pipeline-full.md` |
| Guardrails / boundaries | `agent-01-fundaments.md` (guardrails) | `config/state-tools.mjs` (runtime), `iter-loop-tools.md:18` |
| Evals de 1er intento | `agent-02-task-execution.md` (diagnóstico) | `RULES.md` North Star → P0-1 |
| Memoria / aprendizajes | `agent-01-fundaments.md` (memoria) | `lessons.md`, `decisions.md`, `campaign_memory_write` |
| Orquestación / SARL | `agent-03-orchestration.md` (fallos/orquestador) | `subagent-recovery.md`, `campaign_get_workflow` |
| TDD + pirámide de tests | `eng-01-software.md:118-154` | `.opencode/skills/test-driven-development`, `verify*.ps1` |
| Quality gates / shift-left | `eng-01-software.md:223-252` | `AGENTS.md` CI two-tier, `verify_changed.ps1` |
| Debug sistémico / repro | `eng-02-systems.md:393-439` | skill `systematic-debugging`, `debugging-and-error-recovery` |
| RCA / postmortem | `eng-02-systems.md:247-276` | `troubleshooting.md`, propuesta P2-2 |
| Observabilidad / traces | `eng-02-systems.md:227-237` | `session-tracking.ps1`, `campaign_emit_event` → P2-5 |
| Risk register / pre-mortem | `eng-03-project.md:113-146` | triage de `plan.md` → P1-1, P1-2 |
| DoD por nivel | `eng-03-project.md:149-181` | RULES.md invariantes, `verify*.ps1` → P3 |
| WIP / one bet | `eng-03-project.md:211`, `313` | `pipeline-full.md` → P2-4 |
| Retro / learning loop | `eng-03-project.md:267-279` | `progreso` skill → P1-3 |

---

## Resumen ejecutivo

- **Fuerte:** enforcement por estado (C0) con 10 estados y allow/deny por fase (`state-tools.mjs`), triage gate, discovery con blast radius (`task.md`), verificación mecánica (`verify*.ps1`), SARL (`subagent-recovery.md`), memoria de decisiones/lecciones, one-task-at-a-time y recitation estructurado. En guardrails y verificación, el sistema está por encima del promedio.
- **Gap crítico:** nadie mide si el sistema cumple su North Star (`RULES.md`). Sin eval extrínseco, el loop de mejora es anecdótico.
- **Fallos reales:** duplicación/divergencia de definiciones de estado (STALL falta en `iter-loop-tools.md:18`), enforcement cooperativo (saltable), self-review sin segunda opinión, hooks no instalados, SARL sin trazabilidad.
- **Prioridad inmediata:** P0-1 (evals) + P0-2 (fuente única de estados). Luego P1-1 a P1-4 (pre-mortem, risk register, retrospectiva medible, endurecer denied) ≈ 4-5 h totales.
- **Lo que NO falta:** fricción o burocracia. El sistema ya tiene rigor en verificación, triage y memoria; las mejoras P0-P1 instrumentan lo que ya se declara, no agregan pasos vacíos.
- **Paso siguiente propuesto:** crear un plan con las tareas DRV de P0-P2, arrancando por P0-1, y re-ejecutar este gap como eval de primer intento del propio pipeline (objetivo: la tasa de éxito del task-system se mide por primera vez).
- Este reporte es solo lectura; las mejoras P0-P3 se implementarían como tareas nuevas (DRV) en un plan, no con cambios directos al sistema.