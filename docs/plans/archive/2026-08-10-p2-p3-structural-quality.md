# Plan de Ejecución: P2 calidad estructural + P3 seleccionado

> **Inicio:** 2026-08-10
> **Estado:** ✅ COMPLETADO (2026-08-10)
> **Fuente:** docs/research/2026-08-10-agent-engineering/REPORTE-FINAL.md (§3.7 P2-1..P2-8, P3 selectos)

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 12 | 3     | 0    | 0         |

Ejecuta el paquete **P2** completo + **P3** ligeros. Cada tarea es independiente y vive en
archivos exclusivos → se distribuye en sub-agentes paralelos sin conflictos de edición.
DEFER: P3-3 (mutation score/Rust infra), P3-9 (cargo-mutants CI Heavy, 8-16h), P3-2 (requiere
histórico de datos real — se habilita cuando haya datos).

### Task 1: P2-01 — Review por agente distinto (segunda opinión)
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `prompts/task.md`, `RULES.md`, `.opencode/agents/vanta-audit.md`
- **Gate Justificación:** P2-1 es la falla más grave (REPORTE §4): el REVIEW lo ejecuta el mismo contexto que implementó.
- **Gate Result:** ✅ DO
- **Contrato:** task.md exige REVIEW por agente distinto (vanta-audit) con enfoque+cómo se probó para tareas 🔴; RULES.md lo norma; persona vanta-audit es leaf node con task:* deny.
- **Task file:** `skills/campaign-executor/tasks/P2-01.md`
- **Estado:** ✅ COMPLETED

### Task 2: P2-02 — Postmortem triggers + plantilla 10 min
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `.opencode/skills/progreso/SKILL.md`, `.opencode/memory/lessons.md`
- **Gate Justificación:** Solo existe `lessons.md` reactivo (REPORTE §3.3 punto 6).
- **Gate Result:** ✅ DO
- **Contrato:** progreso define triggers de postmortem (task failed/❌, verify repetido, incidente) + plantilla 10 min (timeline, impacto, causa, follow-ups con owner). `rg -c "postmortem" progreso/SKILL.md` ≥1.
- **Task file:** `skills/campaign-executor/tasks/P2-02.md`
- **Estado:** ✅ COMPLETED

### Task 3: P2-03 — Reporte de incertidumbre uphill/downhill
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/plan.md`, `prompts/task.md`
- **Gate Justificación:** Reporting es %-completado; falta uphill (incógnitas) vs downhill (ejecución) (REPORTE §3.3 punto 7).
- **Gate Result:** ✅ DO
- **Contrato:** task.md template agrega contador de incógnitas (uphill) vs pendientes de ejecución (downhill); plan.md documenta el evento "plan adjust" al re-planificar. `rg -c "uphill\|incógnita\|incognita" prompt/*.md` ≥1.
- **Task file:** `skills/campaign-executor/tasks/P2-03.md`
- **Estado:** ✅ COMPLETED

### Task 4: P2-04 — WIP hard-limit
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`
- **Gate Justificación:** Convención one-task-at-a-time pero nada impide que `run` arranque con otra in-progress (REPORTE §3.3 punto 8).
- **Gate Result:** ✅ DO
- **Contrato:** campaign-server rechaza arrancar (o marcar in-progress) una tarea si hay otra in-progress activa; mensaje claro; test del tool.
- **Task file:** `skills/campaign-executor/tasks/P2-04.md`
- **Estado:** ✅ COMPLETED

### Task 5: P2-05 — Trace ID por tarea
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/enforcement/session-tracking.ps1`
- **Gate Justificación:** Sin trace ID por tarea, el RCA de un agente fallido es adivinanza (REPORTE §3.3 punto 9).
- **Gate Result:** ✅ DO
- **Contrato:** cada task in-progress recibe traceId (uuid); eventos de `campaign_emit_event` y verify log incluyen traceId; session-tracking persiste.
- **Task file:** `skills/campaign-executor/tasks/P2-05.md`
- **Estado:** ✅ COMPLETED

### Task 6: P2-06 — Coverage mínimo mecanizado (gate documentado)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-lead
- **Archivos clave:** `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1`
- **Gate Justificación:** PR puede bajar cobertura del módulo caliente sin que nada falle (REPORTE gap-02 punto 3).
- **Gate Result:** ✅ DO (documentar + wiring guard)
- **Contrato:** CI_POLICY documenta `cargo llvm-cov --fail-under <umbral>` como gate en Heavy con umbral inicial prudente + subida gradual; verify.ps1 corre llvm-cov --fail-under solo si la tool está disponible (guard). NO bloquear el verify default si no está instalado.
- **Task file:** `skills/campaign-executor/tasks/P2-06.md`
- **Estado:** ✅ COMPLETED

### Task 7: P2-07 — Security/Performance como fase explícita
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `prompts/pipeline-full.md`, `prompts/task.md`
- **Gate Justificación:** Skills existen pero no son paso mandatorio en pipeline-full (dependen de "REVIEW si corresponde") (REPORTE gap-02 punto 4).
- **Gate Result:** ✅ DO
- **Contrato:** pipeline-full define fases explícitas SECURITY (si toca trust boundaries/deps) y PERFORMANCE (si toca hot path) que activan skills correspondientes; task.md template agrega checkboxes.
- **Task file:** `skills/campaign-executor/tasks/P2-07.md`
- **Estado:** ✅ COMPLETED

### Task 8: P2-08 — DoD multi-nivel explícito
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/task.md`, `prompts/plan.md`, `.opencode/references/definition-of-done.md`
- **Gate Justificación:** DoD existe pero no está referenciada como contrato en task.md ni aplica por nivel (REPORTE gap-02 punto 5).
- **Gate Result:** ✅ DO
- **Contrato:** task.md referencia `definition-of-done.md` y diferencia DoD por nivel (task/commit/release); plan.md incluye bloque DoD multi-nivel en el template.
- **Task file:** `skills/campaign-executor/tasks/P2-08.md`
- **Estado:** ✅ COMPLETED

### Task 9: P3-04 — Contraste de decisión con validation web
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/AGENTS.md`
- **Gate Justificación:** Las decisiones se registran sin validación externa (REPORTE P3-4).
- **Gate Result:** ✅ DO
- **Contrato:** `campaign_memory_write(file="decisions")` aconseja/require validation web (Regla AGENTS.md) cuando la decisión es técnica ambiguous; documentado en AGENTS.md.
- **Task file:** `skills/campaign-executor/tasks/P3-04.md`
- **Estado:** ✅ COMPLETED

### Task 10: P3-05 — Cynefin + top-3 riesgos en triaje
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/plan.md`
- **Gate Justificación:** El triaje clasifica por tipo pero nunca por dominio de complejidad (REPORTE gap-02 punto 1).
- **Gate Result:** ✅ DO
- **Contrato:** plan.md agrega al triaje de tareas 🔴/ambiguas: clasificación Cynefin (obvio/complicado/complejo/caótico) + "top 3 riesgos" obligatorio.
- **Task file:** `skills/campaign-executor/tasks/P3-05.md`
- **Estado:** ✅ COMPLETED

### Task 11: P3-06 — Refactoring 2-sombreros
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `.opencode/skills/code-simplification/SKILL.md`, `RULES.md`
- **Gate Justificación:** Ninguna skill documenta la separación comportamiento/estructura en commits distintos con tests entre ambos (REPORTE gap-02 punto 8).
- **Gate Result:** ✅ DO
- **Contrato:** code-simplification documenta 2-sombreros Fowler (comportamiento vs estructura en commits distintos, tests entre ambos); RULES.md agrega contrato "commit de refactor no cambia comportamiento".
- **Task file:** `skills/campaign-executor/tasks/P3-06.md`
- **Estado:** ✅ COMPLETED

### Task 12: P3-07 — Métricas DORA de flujo (script)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `evals/` (script nuevo), `docs/reports/`
- **Gate Justificación:** Sin fechas estructuradas no se calculan cycle/lead time ni CFR (REPORTE gap-02 punto 6).
- **Gate Result:** ✅ DO
- **Contrato:** `evals/dora.mjs` deriva cycle/lead time y CFD de plan files + tasks; emite `docs/reports/dora.md`; `node evals/dora.mjs` exit 0; degrade graceful si faltan fechas.
- **Task file:** `skills/campaign-executor/tasks/P3-07.md`
- **Estado:** ✅ COMPLETED

### Task 13: P3-08 — Auditar conteo de skills y SKILLS-MANIFEST
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `SKILLS-MANIFEST.md`, `.opencode/VANTADB-OPERATING-MANUAL.md`
- **Gate Justificación:** Cifras de skills inconsistentes entre manifest/manual/AGENTS (gap-02 punto 3).
- **Gate Result:** ✅ DO
- **Contrato:** conteos de skills en SKILLS-MANIFEST, manual y AGENTS verificados contra el disco (`.opencode/skills/`, `.agents/skills/`, `~/.agents/skills/`) y unificados.
- **Task file:** `skills/campaign-executor/tasks/P3-08.md`
- **Estado:** ✅ COMPLETED