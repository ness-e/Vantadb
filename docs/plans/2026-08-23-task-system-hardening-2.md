# Plan de Ejecución: Task System Hardening II — paralelismo real + spec-first

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
- **Estado:** ⬜ PENDING

### Task 2: R2 — Claim temprano de tarea (anti doble-Discovery)
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟡
- **Contrato:** test: get_next_task con claim:true marca IN PROGRESS atómicamente y la segunda llamada claim no devuelve la misma tarea
- **Estado:** ⬜ PENDING

### Task 3: R3 — Locks para session_track y pipeline-state.json
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** test: dos updates de sesión consecutivos conservan ambos contextos (merge serializado); snapshot RMW bajo lock genérico withFileLock
- **Estado:** ⬜ PENDING

### Task 4: R4 — Lock wait cap + diagnóstico + EPERM retry
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** lock ajeno → error con pid/ts del holder tras máx ~1s (no 5s de freeze); renameSync reintentado 3× ante EPERM
- **Estado:** ⬜ PENDING

### Task 5: R5 — Guard: >1 plan activo exige planFile explícito
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** test: resolvePlan sin planFile y 2 planes modificados <24h → error claro pidiendo ruta explícita
- **Estado:** ⬜ PENDING

### Task 6: R6 — Rotación verify-log + campaign_eval_summary + campaign_plan_lock_info
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟡
- **Contrato:** tests: log >5MB rota a verify-log-<ts>.jsonl; eval_summary calcula first-try rate por skill desde filas del log; lock_info reporta holder/ts o lista vacía
- **Estado:** ⬜ PENDING

### Task 7: R7 — classify_workflow robusto + autonomous flag (server)
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs, parsers.mjs 🟢
- **Contrato:** keywords ultra-genéricos ("add","new","find") removidos o exigen score≥2; extractAutonomous lee `> **Autonomous:** true` y get_next_task lo expone
- **Estado:** ⬜ PENDING

### Task 8: S1 — Spec-driven profundo
- **Archivos clave:** prompts/spec-template.md (NUEVO), prompts/pipeline-full.md, prompts/question-gates.md, skills/campaign-executor/templates/task-definition.md 🟢
- **Contrato:** feature-add/lógica nueva exige bloque `## Spec` en el task file antes de ACT (gate mecánico grep); plantilla versionada con secciones mínimas
- **Estado:** ⬜ PENDING

### Task 9: D1 — Question tool availability + autonomous flag (prompts)
- **Archivos clave:** .opencode/agents/*.md, prompts/question-gates.md 🟢
- **Contrato:** tabla de qué agentes tienen `question` documentada en question-gates.md; anti-abuso referencia `> **Autonomous:**` del plan
- **Estado:** ⬜ PENDING

### Task 10: D2 — Trim escaleras duplicadas
- **Archivos clave:** skills/campaign-executor/SKILL.md 🟢
- **Contrato:** §Escalation ladder y §SARL reducidos a resumen 5 líneas + puntero a subagent-recovery.md; cero números contradictorios
- **Estado:** ⬜ PENDING

### Task 11: D3 — Manual → índice
- **Archivos clave:** .opencode/VANTADB-OPERATING-MANUAL.md 🟡
- **Contrato:** banner deprecado + índice apuntando a fuentes canónicas; ≤200 líneas
- **Estado:** ⬜ PENDING

### Task 12: D4 — Limpieza decidida: statewright fuera, memoria versionada, AGENTS.md raíz
- **Archivos clave:** .opencode/references/statewright/, .gitignore, AGENTS.md (raíz) 🟢
- **Contrato:** statewright eliminado; memory/ versionada (lessons+decisions commiteados); AGENTS.md raíz verificado como stub (sin divergencia)
- **Estado:** ⬜ PENDING

---

## Protocolo
Igual que hardening I: skills base al inicio, MCP tools por paso, C0 PLAN→ACT→VERIFY, Question Gates aplicables, commit Conventional por fase, gate de verificación entre fases (node --test + greps).
