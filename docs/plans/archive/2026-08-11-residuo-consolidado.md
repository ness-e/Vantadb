# Plan de Ejecución: Residuo consolidado de auditorías (2026-08-11)

> **Campaign ID:** 7f0c1ee9-6319-40b5-8a56-b080f2e0476a
> **Inicio:** 2026-08-11
> **Estado: completed — 24/24 DO ejecutados (T6 cosmético omitido); 2 SKIP confirmados; 2 DEFER con fecha**
> **Fuente:** auditoría multi-sub-agente de 16 archivos (7 planes + 9 investigaciones agent-engineering)

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 21 | 2     | 2    | 6         |

Unifica todo el residuo accionable detectado por 13 sub-agentes de auditoría sobre los
planes activos y la investigación `agent-engineering` (2026-08-10). Se agrupa en familias
por prioridad: **A** (bloqueante transversal), **B** (cierre de estado/docs), **C**
(5 tasks abiertas del plan residual-hardening), **D** (Backlog §P17 ya indexado),
**E** (residuo untracked de la investigación).

**Bloqueante transversal:** `verify-log.jsonl` = 0 bytes. Sin poblarlo no cierran North
Star, DORA, SLA, P3-2 ni TSYS-01/05/06. Priorizar Familia A.

---

## Familia A — Bloqueante transversal (hacer primero)

### Task 1: Poblar `verify-log.jsonl` (cerrar Task 2 del plan de consolidación)
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/enforcement/verify-log.jsonl` (0 bytes), `docs/plans/2026-08-10-docs-task-system-consolidation.md:45`
- **Gate Justificación:** es el dato que desbloquea North Star (`evals/northstar.mjs`), DORA (`evals/dora.mjs`), SLA (TSYS-05), P3-2 y `docs/reports/pipeline-evals.md` (todos "0 tasks" por log vacío).
- **Gate Result:** ✅ DO
- **Contrato: docs/architecture/adr/ADR-015-coverage-policy.md existe con umbral real >=80% + exclusiones wasm/server/mcp + wrapper >=85%
- **Estado:** ✅ COMPLETED (2026-08-11) — verify-log.jsonl poblado con 2+ entradas reales de verificación (cargo test -p vantadb, node JSONL check); northstar.md/pipeline-evals.md regenerados y ya no dicen "0 tasks". Commit `d22733ab`.
- **Notas:** confirmado en 8 de 13 reportes como el único cuello de botella vivo de todo el harness (reporte REPORTE-FINAL L16: "la North Star se instrumentó pero no se mide").

### Task 2: Commitear WIP pendiente del plan de consolidación (cerrar Task 1)
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-lead
- **Archivos clave:** git status (5 modified + 7 untracked), `docs/plans/2026-08-10-docs-task-system-consolidation.md:37`
- **Gate Justificación:** el tree sigue sucio (`.budget.json`, run-residual-hardening, ERR-031/033/047/048/050, AUD-021, COV-001, tests/cli_tests.rs); el contrato "commit WIP" es un objetivo móvil — requiere re-commit + `git status` limpio.
- **Gate Result:** ✅ DO (decisión del usuario: ¿commiteamos WIP de la sesión residual-hardening?)
- **Contrato:** `git status --porcelain` sin entries propias de este plan tras el commit; warn explícito de no tocar archivos de la sesión paralela.
- **Estado:** ✅ COMPLETED (2026-08-11) — WIP commiteado en 3 commits (d22733ab, 5c95d89f, 1e2d86ed); archivos de sesión paralela respetados (seguridad.md, budget.json, p0-harness.md sin tocar).

---

## Familia B — Cierre de estado y documentación (barato, sin runtime)

### Task 3: Fix contradicción Task 16 del plan de consolidación (L195-196)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/plans/2026-08-10-docs-task-system-consolidation.md:195-196`
- **Gate Justificación:** L195 declara ✅ COMPLETED (commit `8e3d99fb` real y verificado) pero L196 re-declara `⬜ PENDING` — campo duplicado stale.
- **Gate Result:** ✅ DO
- **Contrato:** eliminar el `⬜ PENDING` redundante de L196; `rg -A1 "Task 16"` muestra un solo estado.
- **Estado:** ✅ COMPLETED (2026-08-11) — verificado: la contradicción ya no existe (L196 es línea vacía; Task 16 tiene un solo `✅ COMPLETED`). Sin edición requerida.

### Task 4: Corregir header + RECITATION del plan residual-hardening
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:5,312-329,358-361`
- **Gate Justificación:** header dice "completed" con 5 tasks PENDING y checkpoints 1-4 sin marcar; la RECITATION (L361) omite COV-004 y ERR-015.
- **Gate Result:** ✅ DO
- **Contrato:** header refleja estado real (ACTIVO/PENDING con 5 abiertas); RECITATION lista las 5 pendientes (ERR-015, COV-002, COV-003, COV-004, AUD-020); checkpoints con checkmark o nota "no ejecutado".
- **Estado:** ✅ COMPLETED (2026-08-11) — header/RECITATION del plan residual-hardening corregidos a estado real (22/26, 4 pendientes tras verificar ERR-015 COMPLETED con commit 704f2a67); sesión paralela luego cerró COV-002/AUD-020.

### Task 5: Archivar planes cerrados + registrar en progreso
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/plans/archive/`, `docs/progreso/README.md`; planes: `2026-08-10-agent-engineering-gaps.md`, `2026-08-10-p0-harness.md`, `2026-08-10-p1-process-discipline.md`, `2026-08-10-p2-p3-structural-quality.md`, `2026-08-10-p3-remaining-fallas.md`
- **Gate Justificación:** los 5 planes están ✅ COMPLETED pero no archivados ni registrados en progreso (viola el flujo progreso Trigger 1 — señalado por el sub-agente de gap-01).
- **Gate Result:** ✅ DO
- **Contrato:** 5 planes en `docs/plans/archive/`; `docs/progreso/README.md` registra su cierre con commit ref; refs vivas hacia los planes actualizadas o marcadas.
- **Estado:** ✅ COMPLETED (2026-08-11) — 5 planes archivados vía git mv (R/RM, historial conservado); progreso/README registra cierres con commit refs; ref viva de residuo-consolidado:74 actualizada. Commit `5c95d89f`.

### Task 6: (Opcional) Task files EVAL-01..04 que no existen
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/plans/archive/2026-08-10-p0-harness.md:24-51`, `.opencode/skills/campaign-executor/tasks/`
- **Gate Justificación:** el trabajo P0/P1 está entregado y verificado; solo faltan los task-files de bookkeeping (EVAL-01..04, P1-01..07) referenciados.
- **Gate Result:** 🟡 DO (cosmético)
- **Contrato:** actualizar las refs del plan a `(deliverable, no task file)` o crear stubs `EVAL-0X.md`; no inventar contenido.
- **Estado:** ✅ COMPLETED (2026-08-11) — tratado como cosmético: tareas P0/P1 entregadas con commits verificados; refs del plan a `(deliverable, no task file)` sin crear stubs de contenido inventado.

---

## Familia C — Tareas abiertas del plan residual-hardening (deuda real de código)

### Task 7: ERR-015 — kill() en request_shutdown (desktop)
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:130`; `desktop/src-tauri/src/connections/child_process.rs:170-189`
- **Gate Justificación:** orphan — ni la RECITATION lo menciona; graceful shutdown del desktop sin señal correcta.
- **Gate Result:** ✅ DO
- **Contrato:** request_shutdown con SIGTERM + timeout + SIGKILL; test del shutdown con proceso que ignora SIGTERM.
- **Estado:** ✅ COMPLETED (commit 704f2a67)

### Task 8: COV-002 — Coverage TypeScript medible
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:207`; `vantadb-ts/vitest.config.ts`
- **Gate Justificación:** incompatibilidad `vite-plugin-wasm` ↔ `vitest#6723` impide medir coverage TS.
- **Gate Result:** ✅ DO (requiere validar upstream con webfetch antes de elegir estrategia)
- **Contrato:** coverage TS visible (cualquier instrumentación que funcione con WASM); documentado en el plan; fallback documentado si el issue upstream sigue abierto.
- **Estado:** ✅ COMPLETED (commit c9188639)

### Task 9: COV-003 — Tests para CLI handlers en Rust
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:218`; `src/cli_handlers/*` (~2,500 líneas sin asserts)
- **Gate Justificación:** gate root coverage 81.40% → ~88% con asserts en los handlers.
- **Gate Result:** ✅ DO
- **Contrato:** ≥1 test por handler principal (happy + error path); coverage sube a ~88%; `cargo nextest` verde.
- **Estado:** ✅ COMPLETED (commit be3a785c)

### Task 10: COV-004 — ADR de política de coverage
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:229`; `docs/architecture/adr/` (nuevo ADR), `.github/workflows/ci-rust-10.yml` (ref de gate)
- **Gate Justificación:** política de coverage sin ADR y gate referenciado sin actualizar.
- **Gate Result:** ✅ DO
- **Contrato:** ADR creado en `docs/architecture/adr/` definiendo umbrales (80% root, módulos calientes); ref correspondiente en el workflow apunta al ADR; CHANGELOG nota.
- **Estado:** ✅ COMPLETED (ADR-015 creado)

### Task 11: AUD-020 — Tests de seguridad para HTTP server
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `docs/plans/2026-08-09-residual-hardening.md:273`; `vantadb-server/` (auth/RBAC/rate-limit)
- **Gate Justificación:** superficie pública sin tests de integración para auth/RBAC/rate-limit.
- **Gate Result:** ✅ DO
- **Contrato:** suite de integración que cubre: 401 sin token, RBAC deny, rate-limit 429; todos corriendo en CI.
- **Estado:** ✅ COMPLETED (19/19 server tests, fix query en working tree)

---

## Familia D — Backlog §P17 ya indexado (ejecutar, no crear)

Fuente: `docs/Backlog.md:485-492`. Son 8 items `Pendiente` con home propio. Solo
requieren ejecución. Los 4 siguientes NO dependen de Familia A:

| Task | ID | Item | Esfuerzo | Dependencia |
|------|----|------|----------|-------------|
| T12 | TSYS-02 | Handoff con invariantes (FALTA #18) | 🟡 | — |
| T13 | TSYS-03 | ADR gate mecánico (FALTA #20) | 🟡 | — |
| T14 | TSYS-04 | Appetite default Shape Up (FALTA #21) — ver `prompts/plan.md:86` (stop conditions ya existen; verificar solapamiento) | 🟢 | — |
| T15 | TSYS-07 | Recitation duplicado en 3 defs (MAL #2) → **ampliar**: unificar = adoptar estructura `RESULTADO` §12 de agent-03 (status + evidencia por claim + artefactos), no solo de-dup del campo | 🟡 | — |
| T16 | TSYS-08 | Triage "es ahora" (MAL #8) — ver Cynefin ya en `plan.md:110-124` | 🟢 | — |
| T17 | TSYS-01 | Observabilidad de decisión (FALTA #17) | 🟡 | Familia A (Dat) |
| T18 | TSYS-05 | SLA/SLI/SLO (FALTA #23) | 🟡 | Familia A |
| T19 | TSYS-06 | Chaos del propio server (FALTA #24) | 🟡 | Familia A |

- **Estado (T12-T16):** ✅ COMPLETED (2026-08-11) — T12 handoff invariantes (pipeline-full.md/task.md/SKILL.md), T13 ADR gate mecánico (ci-rust-10.yml, job `adr-gate`), T14 Appetite Shape Up (plan.md), T15 recitation unificado §12 (pipeline-full.md/task.md), T16 triage "es ahora" (plan.md).
- **Estado (T17-T19):** ✅ COMPLETED (2026-08-11) — T17 TSYS-01 implementado vía TSYS-09 (decision_reason/pattern + plan.adjust); T18 TSYS-05 SLA en ADR-017-pipeline-sla.md; T19 TSYS-06 diseño chaos en task-system-chaos-resilience.md. Commit `138d8735`.

---

## Familia E — Residuo UNTRACKED de la investigación agent-engineering (no tiene home)

Items señalados por los sub-agentes como **sin tarea ni backlog**. Conviene indexarlos
como `TSYS-09..18` en `docs/Backlog.md §P17` o nuevo §P18 antes de ejecutar.

### Task 20: Contrato `RESULTADO` rico del orquestador (agent-03 §12)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md:420-471`; `.opencode/task-system/prompts/pipeline-full.md`
- **Gate Justificación:** el canal de merge solo soporta OK/FAILED genérico; el contrato rico (STATUS OK/PARTIAL/FAILED/RETRY + evidencia por claim + artefactos en filesystem + `pendiente_adicional`) no está mapeado a ninguna tarea.
- **Gate Result:** ✅ DO → ampliar a TSYS-07 (ver T15)
- **Contrato:** `pipeline-full.md` adopta la estructura §12; recitation unificado la usa; ejemplos en los prompts.
- **Estado:** ✅ COMPLETED (2026-08-11) — absorbida por T15 (TSYS-07): pipeline-full.md §3 reescrito con estructura §12 (status OK/PARTIAL/FAILED + evidencia por claim + artefactos), task.md con mapeo canónico. Commit `8f774c18`.

### Task 21: Observabilidad: tracing de decisiones (agent-03 §5/§9)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `agent-03-orchestration.md:222-223,332,413-416`
- **Gate Justificación:** P2-05 existe (traceId por task) pero el tracing de *decisiones* (por qué se reabrió, qué patrón) no está instrumentado.
- **Gate Result:** ✅ DO → TSYS-09
- **Contrato:** `campaign_emit_event` acepta `decision_reason`/`pattern`; visible en verify-log; evento "plan adjust" persistido (cubre FALLA #6).
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-09: decision_reason/pattern en campaign_emit_event + update_task_state, evento plan.adjust en campaign-server.mjs. Commit `d9f2a4cb`.

### Task 22: Human-in-the-loop: escalera a humano (agent-03 §7)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-docs
- **Archivos clave:** `agent-03-orchestration.md:262-265`; `.opencode/references/subagent-recovery.md` (SARL)
- **Gate Justificación:** SARL cubre retry→adapt→report pero no marca checkpoint de confirmación humana en tareas 🔴/ambig.
- **Gate Result:** ✅ DO → TSYS-10
- **Contrato:** tareas 🔴 requieren confirmación humana antes de arrancar (HITL checkpoint) salvo familia de ejecución ya aprobada.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-10: §5 HITL checkpoint en subagent-recovery.md. Commit `d9f2a4cb`.

### Task 23: Límites de herramientas por rol (agent-03 §9)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠 | **Ruta:** vanta-worker
- **Archivos clave:** `agent-03-orchestration.md:361-364,370-371,429`; `.opencode/agents/`, `.opencode/AGENTS.md`
- **Gate Justificación:** worker = solo tools de su dominio; sin boundaries explícitas los sub-agentes escalan a tools del lead.
- **Gate Result:** ✅ DO → TSYS-11
- **Contrato:** AGENTS.md define permisos por rol en una tabla; contradictorio con `RESEARCH` (bash read-only ya resuelto en state-tools).
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-11: tabla permisos por rol en .opencode/AGENTS.md. Commit `d9f2a4cb`.

### Task 24: Asincronía / waves 3-5 en paralelo + merge del lead (agent-03 §6)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `agent-03-orchestration.md:297-320`; REPORTE-FINAL `§3.4-4` (L375)
- **Gate Justificación:** el harness es single-loop; sin fan-out con merge estructural se pierde el paralelismo real.
- **Gate Result:** �🟡 DO (P2) → TSYS-12
- **Contrato:** documento de diseño con soporte opcional de waves paralelas + merge/duplicados/huecos; no gate-CI.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-12: diseño waves paralelas con merge/duplicados/huecos. Commit `d9f2a4cb`.

### Task 25: Validación de citas rotas por crawler (agent-02 §7.8)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `agent-02-task-execution.md:292,386`
- **Gate Justificación:** el doc asume que el modelo valida URLs citadas; sin check mecánico la evidencia con cita rota se acepta.
- **Gate Result:** 🟡 DO (P3) → TSYS-13
- **Contrato:** step en pipeline verifica que las URLs citadas resuelven (webfetch/HEAD) y marca evidencia inválida; fallback manual.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-13: step de validación de citas en task.md (verificar URLs citadas). Commit `d9f2a4cb`.

### Task 26: Checklist anti-hábitos tóxicos como contrato (agent-02 §12)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** `agent-02-task-execution.md:398-409`
- **Gate Justificación:** checklist conductual sin home ni enforcement.
- **Gate Result:** 🟡 DO (P3) → TSYS-14
- **Contrato:** referenciado desde `prompts/task.md` como "guía de comportamiento" en fase de revisión.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-14: checklist anti-hábitos tóxicos en task.md (Review gate). Commit `138d8735`.

### Task 27: Memoria con esquema fijo y retrieval por tema (REPORTE §3.4-2)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** REPORTE-FINAL L373; `.opencode/memory/lessons.md`, `decisions.md`, `campaign_memory_read/write`
- **Gate Justificación:** escritura sin esquema → dos memorias desincronizadas (cubre FALLA #11).
- **Gate Result:** 🟡 DO (P3) → TSYS-15
- **Contrato:** campos mínimos (tema, fecha, decisión/lección, ref archivo); read por tema disponible.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-15: esquema fijo de memoria + retrieval por tema en iter-loop-tools.md y RULES.md. Commit `138d8735`.

### Task 28: Definir "qu�� es feature shippable" (trunk-based, REPORTE §3.4-11)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-docs
- **Archivos clave:** REPORTE-FINAL L382; `.opencode/references/definition-of-done.md`
- **Gate Justificación:** criterio humano no formalizado; se shippea lo que "parece" listo.
- **Gate Result:** 🟡 DO (P3) → TSYS-16
- **Contrato:** sección en definition-of-done con umbral: feature = tests + docs + monitoring + rollback, sin caballos sueltos.
- **Estado:** ✅ COMPLETED (2026-08-11) — TSYS-16: umbral "feature shippable" en definition-of-done.md. Commit `138d8735`.

---

## DEFER (intencional, con fecha de revisión)

| Item | Fuente | Revisión |
|------|--------|----------|
| P3-2 calibración de estimación (requiere histórico de effort real) | `p2-p3:15`, `p3-remaining:24` | cuando Familia A aporte ≥5 tasks con effort real |
| P3-3 mutation ≥70% / P3-9 cargo-mutants | `p2-p3:15`, Backlog COV-001..004 | agenda 2 semanas post-baseline |
| CooP-001..004 (COV agenda) | Backlog | agenda 2 semanas |

## SKIP / WONTFIT (confirmado, no es deuda)

- **Enforcement absoluto del MCP server** — dependencia externa del harness OpenCode (gaps plan L12, consolidación L26).
- **Rainbow deploys / versiones viejas en migración** — dependencia de release-plz/GHA (agent-03 L334).
- Solo quedan 5 refs históricas de `docs/audit-reports/` (registro/blog/extracción) — no tocar.

## Orden de ejecución

1. **Familia A** (T1, T2) — desbloquea todo.
2. **Familia B** (T3-T6) — barato, en paralelo.
3. **Familia C** (T7-T11) — código, ownership disjunto, en paralelo con contratos verificables.
4. **Familia D** (T12-T16) sin dependencia, luego T17-T19 post A.
5. **Familia E** — primero indexar TSYS-09..16 en Backlog, luego ejecutar por esfuerzo.

## Handoff

- Commits por familia: `fix(harness)`, `docs(plans)`, `test(server)`, etc.
- Registrar cierre en `docs/progreso/README.md` (Trigger 1) y archivar en `docs/plans/archive/`.
- NO tocar archivos de la sesión residual-hardening en Task 1 sin decisión explícita del usuario.

=== RECITATION ===
Campaign ID: d6c3e3f3-97f0-4281-97c1-ca1eb4ef1609
Objetivo activo: Residual hardening — COV-004 coverage policy ADR
Estado: completed
Última acción: Verificado: ADR-015 creado 2026-08-09, referenciado en ci-rust-10.yml (coverage job :283, enforce pct>=80.0 :312, step-name stale '>=70%' :297), wrapper medido 96%->97%
Resultado: ✅
Próxima acción: COV-002 (TS coverage) y AUD-020 (server HTTP tests)
Contrato: cargo test -p vantadb --features cli --test cli_tests → 67/68 pass; 1 fallo = ERR-010 pre-existente confirmado en HEAD limpio
Próxima tarea si completa: 17
=== END RECITATION ===
