# Plan de Ejecución: Task System Hardening II — paralelismo real + spec-first

> **Campaign ID:** a5fe091a-e95c-4362-bc17-d36f72aeecf8
> **Inicio:** 2026-08-23
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** Segunda ronda de auditoría + decisiones Gate P del usuario

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 12 | 0     | 0    | 0         |

**Decisiones Gate P (2026-08-23):** versionar memoria · eliminar statewright · manual→índice · alcance TODO.

---

### Task 1: R1 — Recitation por-tarea (paralelo-safe)
- **Archivos clave:** .opencode/task-system/mcp/parsers.mjs, mcp/campaign-server.mjs 🟡
- **Contrato:** tests: updateRecitation escribe `=== RECITATION <ID> ===` sin pisar bloques de otras tareas; parseRecitation(content, taskId) devuelve la del taskId; compat: bloque viejo sin ID sigue parseando
- **Estado:** ✅ COMPLETED

### Task 2: R2 — Claim temprano de tarea (anti doble-Discovery)
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟡
- **Contrato:** test: get_next_task con claim:true marca IN PROGRESS atómicamente y la segunda llamada claim no devuelve la misma tarea
- **Estado:** ✅ COMPLETED

### Task 3: R3 — Locks para session_track y pipeline-state.json
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** test: dos updates de sesión consecutivos conservan ambos contextos (merge serializado); snapshot RMW bajo lock genérico withFileLock
- **Estado:** ✅ COMPLETED

### Task 4: R4 — Lock wait cap + diagnóstico + EPERM retry
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** lock ajeno → error con pid/ts del holder tras máx ~1s (no 5s de freeze); renameSync reintentado 3× ante EPERM
- **Estado:** ✅ COMPLETED

### Task 5: R5 — Guard: >1 plan activo exige planFile explícito
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** test: resolvePlan sin planFile y 2 planes modificados <24h → error claro pidiendo ruta explícita
- **Estado:** ✅ COMPLETED

### Task 6: R6 — Rotación verify-log + campaign_eval_summary + campaign_plan_lock_info
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟡
- **Contrato:** tests: log >5MB rota a verify-log-<ts>.jsonl; eval_summary calcula first-try rate por skill desde filas del log; lock_info reporta holder/ts o lista vacía
- **Estado:** ✅ COMPLETED

### Task 7: R7 — classify_workflow robusto + autonomous flag (server)
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs, parsers.mjs 🟢
- **Contrato:** keywords ultra-genéricos ("add","new","find") removidos o exigen score≥2; extractAutonomous lee `> **Autonomous:** true` y get_next_task lo expone
- **Estado:** ✅ COMPLETED

### Task 8: S1 — Spec-driven profundo
- **Archivos clave:** prompts/spec-template.md (NUEVO), prompts/pipeline-full.md, prompts/question-gates.md, skills/campaign-executor/templates/task-definition.md 🟢
- **Contrato:** feature-add/lógica nueva exige bloque `## Spec` en el task file antes de ACT (gate mecánico grep); plantilla versionada con secciones mínimas
- **Estado:** ✅ COMPLETED

### Task 9: D1 — Question tool availability + autonomous flag (prompts)
- **Archivos clave:** .opencode/agents/*.md, prompts/question-gates.md 🟢
- **Contrato:** tabla de qué agentes tienen `question` documentada en question-gates.md; anti-abuso referencia `> **Autonomous:**` del plan
- **Estado:** ✅ COMPLETED

### Task 10: D2 — Trim escaleras duplicadas
- **Archivos clave:** skills/campaign-executor/SKILL.md 🟢
- **Contrato:** §Escalation ladder y §SARL reducidos a resumen 5 líneas + puntero a subagent-recovery.md; cero números contradictorios
- **Estado:** ✅ COMPLETED

### Task 11: D3 — Manual → índice
- **Archivos clave:** .opencode/VANTADB-OPERATING-MANUAL.md 🟡
- **Contrato:** banner deprecado + índice apuntando a fuentes canónicas; ≤200 líneas
- **Estado:** ✅ COMPLETED

### Task 12: D4 — Limpieza decidida: statewright fuera, memoria versionada, AGENTS.md raíz
- **Archivos clave:** .opencode/references/statewright/, .gitignore, AGENTS.md (raíz) 🟢
- **Contrato:** statewright eliminado; memory/ versionada (lessons+decisions commiteados); AGENTS.md raíz verificado como stub (sin divergencia)
- **Estado:** ✅ COMPLETED

---

## Protocolo
Igual que hardening I: skills base al inicio, MCP tools por paso, C0 PLAN→ACT→VERIFY, Question Gates aplicables, commit Conventional por fase, gate de verificación entre fases (node --test + greps).

=== RECITATION ===
Campaign ID: b9c2d2c8-1819-40f9-8471-3c0fe16e9c10
Objetivo activo: S1 spec-driven profundo + D1-D4
Estado: completed
Última acción: spec-template.md creado y cableado; routing de questions documentado; SKILL trim; manual→índice; submodule statewright fuera; lessons/decisions commiteados
Resultado: OK
Próxima acción: Retrospectiva + commit cierre
Contrato: rg -l spec-template=2 prompts; ## Spec en template; manual 37L; statewright eliminado; memoria versionada; 42/42 tests
Próxima tarea si completa:

## Retrospectiva de cierre (Start / Stop / Continue)

**Start:**
- Ejecutar por fases con gate mecánico entre fases (detectó el desbalance de paréntesis y la regresión del mensaje de lock al instante).
- Dogfooding del MCP durante la ejecución: reveló que updateTaskStateCore descartaba el Campaign ID insertado (fix incluido).

**Stop:**
- Editar archivos línea-a-línea con pipelines PS frágiles (-NoNewline corrompió .gitignore a medio aplicar) — usar reemplazo regex explícito sobre raw.
- Asumir que el server en memoria refleja el código en disco: los fixes R1 no estaban vivos durante esta campaña; verificar versión con uptime/commit al arrancar.

**Continue:**
- Question gates previos a decisiones estructurales (las 4 respuestas definieron todo el alcance).
- Fuente única canónica + punteros: subagent-recovery, spec-template, state-tools.mjs.

**Una acción medible:** adoptar claim:true en todos los flujos /pipeline run (pipeline-run.md paso 5) para que el doble-Discovery desaparezca — métrica: tareas con 2+ task.started events en traces; baseline actual >0, objetivo 0.

> **Nota operativa:** esta campaña corrió contra un server cargado ANTES de R1-R7
> (recitations sin tag, campaignId regenerado por call). El código en disco está
> testeado 42/42 — **reiniciar OpenCode** para activarlo. El fix adicional de
> persistencia de Campaign ID en updateTaskStateCore quedó incluido.