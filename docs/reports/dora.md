# DORA Flow Metrics Report

> Generado por `evals/dora.mjs` (P3-07) — 2026-08-22T09:16:24.251Z
> Fuentes: `docs/plans/*.md` (23 tareas en 9 planes) + task files en `.opencode/skills/campaign-executor/tasks/` (412) + `*.budget.json` (33 con timestamps) + `verify-log.jsonl` (124 intentos de verify)
> ⚠️ **Fechas derivadas best-effort, NO normalizadas**. Prioridad: markers escritos (`**Inicio:**`, `**Estado:** COMPLETADO (fecha)`, `**Fecha:**`, `**Creado:**`, fechas en bloque de tarea) -> budget epoch ms (`startTime`/`lastActivity`) -> **file mtime**. Donde se usó mtime se marca `(mtime)`. Esto es exactamente lo que P2-05 (traceId por tarea) va a resolver: con traceId real, cada task tendrá timestamps estructurados.


## 1. Cycle / Lead time

| Métrica | Definición pragmática |
|---|---|
| **Lead time** | completado − request. Request = plan `**Inicio:**` del plan que la contiene, fallback `**Creado:**` del task file, fallback budget `startTime`, fallback mtime. |
| **Cycle time** | completado − startOfWork. Start = budget `startTime` (timestamps reales) → `**Creado:**`/`**Inicio:**` del task file → plan inicio. Sin budget ni Creado, cycle == lead (mismo origen). |

### Por tipo de tarea

| Tipo | Tasks | Completed | Lead avg (días) | Cycle avg (días) |
|---|---|---|---|---|
| frontend | 8 | 3 | -0.3 | -0.3 |
| other | 339 | 209 | 0.4 | 0.4 |
| rust | 91 | 54 | 0.9 | 0.9 |

### Por tarea (completadas con fechas derivables)

| Task | Tipo | Plan | Request | Start | Completed | Lead | Cycle |
|---|---|---|---|---|---|---|---|
| 1 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 10 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 2 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 3 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 4 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 5 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 6 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 7 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 8 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| 9 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| ADMIN-01 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-02 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-03 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-04 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-05 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-06 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-07 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-08 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| ADMIN-09 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| AUD-002 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUD-004 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUD-006 | rust | task | 2026-07-21 | 2026-07-21 | 2026-08-05 | 15.0 | 15.0 |
| AUD-009 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUD-022 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-023 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-024 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-025 | rust | task | 2026-08-14 (mtime) | 2026-08-14 | 2026-08-14 (mtime) | 0.0 | 0.0 |
| AUD-026 | rust | task | 2026-08-14 | 2026-08-14 | 2026-08-14 | 0.0 | 0.0 |
| AUD-028 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-029 | rust | task | 2026-08-14 | 2026-08-14 | 2026-08-14 | 0.0 | 0.0 |
| AUD-030 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-031 | rust | task | 2026-08-13 | 2026-08-13 | 2026-08-13 | 0.0 | 0.0 |
| AUD-032 | rust | task | 2026-08-14 (mtime) | 2026-08-14 | 2026-08-14 (mtime) | 0.0 | 0.0 |
| AUD-034 | rust | task | 2026-08-14 | 2026-08-14 | 2026-08-14 | 0.0 | 0.0 |
| AUD-037 | rust | task | 2026-08-14 | 2026-08-14 | 2026-08-14 | 0.0 | 0.0 |
| AUD-038 | rust | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| AUD-040 | rust | task | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-16 (mtime) | 0.0 | 0.0 |
| AUD-041 | rust | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| AUD-043 | rust | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| AUD-044 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| AUD-045 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| AUD-046 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| AUD-047 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-20 | 3.0 | 3.0 |
| AUD-048 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-20 | 3.0 | 3.0 |
| AUD-049 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-20 | 3.0 | 3.0 |
| AUD-050 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-20 | 3.0 | 3.0 |
| AUD-051 | rust | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| AUD-E2E | rust | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| AUDIT-02 | rust | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| AUDIT-03 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-01 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-04 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-12 | rust | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| CI-04 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| CI-05 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| CLI-01 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| COMP-006 | other | task | 2026-07-27 | 2026-07-27 | 2026-07-27 | 0.0 | 0.0 |
| COMP-008 | other | task | 2026-07-27 | 2026-07-27 | 2026-07-27 | 0.0 | 0.0 |
| COMP-021 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| COMP-028 | other | task | 2026-08-02 | 2026-08-02 | 2026-07-28 | -5.0 | -5.0 |
| COMP-029 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| COV-001 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| COV-002 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| COV-003 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| COV-004 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| DESKTOP-01 | other | task | 2026-08-04 (mtime) | 2026-08-04 | 2026-08-19 (mtime) | 15.0 | 15.0 |
| DESKTOP-02 | other | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| DESKTOP-03 | other | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| DESKTOP-04 | other | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| DESKTOP-05 | other | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| DESKTOP-08 | other | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| DESKTOP-09 | other | task | 2026-08-07 | 2026-08-07 | 2026-08-07 | 0.0 | 0.0 |
| DESKTOP-11 | other | task | 2026-08-07 | 2026-08-07 | 2026-08-07 | 0.0 | 0.0 |
| DESKTOP-20 | other | task | 2026-08-08 | 2026-08-08 | 2026-08-08 | 0.0 | 0.0 |
| DESKTOP-MVP-54 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| DEVEX-DEMO | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| DEVEX-EXAMPLES | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| DEVOPS-HOMEBREW | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| DEVOPS-PY313 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| DRV-001 | rust | task | 2026-07-23 | 2026-07-23 | 2026-07-23 | 0.0 | 0.0 |
| DRV-013 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| DRV-014 | rust | task | 2026-07-31 | 2026-07-31 | 2026-07-31 | 0.0 | 0.0 |
| DRV-015 | rust | task | 2026-07-24 | 2026-07-24 | 2026-07-24 | 0.0 | 0.0 |
| DRV-017 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| DRV-027 | rust | task | 2026-07-24 | 2026-07-24 | 2026-07-24 | 0.0 | 0.0 |
| DRV-054 | rust | task | 2026-07-25 | 2026-07-25 | 2026-07-25 | 0.0 | 0.0 |
| DRV-061 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| DRV-067 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| DRV-073 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| DRV-136 | rust | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| ECO-001 | other | task | 2026-07-23 | 2026-07-23 | 2026-07-23 | 0.0 | 0.0 |
| ECO-002 | other | task | 2026-07-23 | 2026-07-23 | 2026-07-23 | 0.0 | 0.0 |
| ECO-003 | other | task | 2026-07-23 | 2026-07-23 | 2026-07-28 | 5.0 | 5.0 |
| ECO-004 | other | task | 2026-07-23 | 2026-07-23 | 2026-07-28 | 5.0 | 5.0 |
| ENT-04 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| ERR-026 | rust | task | 2026-08-11 | 2026-08-11 | 2026-08-20 | 9.0 | 9.0 |
| ERR-036 | rust | task | 2026-08-11 (mtime) | 2026-08-11 | 2026-08-11 (mtime) | 0.0 | 0.0 |
| ERR-037 | rust | task | 2026-08-10 | 2026-08-10 | 2026-08-11 | 1.0 | 1.0 |
| ERR-043 | rust | task | 2026-08-10 | 2026-08-10 | 2026-08-11 | 1.0 | 1.0 |
| ERR-044 | rust | task | 2026-08-09 | 2026-08-09 | 2026-08-11 | 2.0 | 2.0 |
| FND-01-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-02 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-02-M2 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-02-M3 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-03 | other | 2026-08-16-wave-p20-tsys.budget.json | 2026-08-16 | 2026-08-17 | 2026-08-16 | 0.0 | -1.0 |
| FND-04-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-05-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-06 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-06-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-08 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-09 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-11 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-12 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-13-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-13-F2 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| FND-14 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-15 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-17 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-18 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-19 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-20 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| FND-21 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-20 | 2026-08-17 (mtime) | 1.0 | -3.0 |
| FND-22 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-23 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| FND-23-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-17 | 1.0 | 1.0 |
| GH-119 | other | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |
| GH-123 | other | task | 2026-08-04 (mtime) | 2026-08-04 | 2026-08-05 (mtime) | 1.0 | 1.0 |
| GH-124 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-127 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-142 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-143 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GOV-T01 | other | 2026-08-22-vantadb-bindings-sdk.budget.json | 2026-08-22 | 2026-08-22 | 2026-08-22 | 0.0 | 0.0 |
| INV-002 | other | task | 2026-07-30 | 2026-07-30 | 2026-07-30 | 0.0 | 0.0 |
| INV-005-A | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| INV-006 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| INV-007 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-007-B | other | task | 2026-06-06 | 2026-06-06 | 2026-08-05 | 60.0 | 60.0 |
| INV-008 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-009 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-010 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-013 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-014 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-015 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-015-B | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| INV-016 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| INV-016-B | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| INV-017 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| MCP-03 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-04 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-05 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-06 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-07 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-08 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-09 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-10 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-11 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-12 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-13 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-14 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MCP-15 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| MEM-02 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-03 | other | 2026-08-18-vanta-memory.budget.json | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-04 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-05 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-06 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-07 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-08a | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-08b | other | task | 2026-08-20 | 2026-08-20 | 2026-08-18 | -2.0 | -2.0 |
| MEM-09 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-18 | -2.0 | -2.0 |
| MEM-10 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-18 | -2.0 | -2.0 |
| MEM-11 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-18 | -2.0 | -2.0 |
| MEM-12 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-13 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-14 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-15 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-16 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-17 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-18 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-19 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-20 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-21 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-22 | other | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| MEM-23 | other | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| MEM-25 | other | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| MEM-26 | other | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| MEM-34 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-35 | other | task | 2026-08-20 | 2026-08-20 | 2026-08-20 | 0.0 | 0.0 |
| MEM-38 | other | task | 2026-08-21 | 2026-08-21 | 2026-08-21 | 0.0 | 0.0 |
| MEM-45 | other | task | 2026-08-22 | 2026-08-22 | 2026-08-22 | 0.0 | 0.0 |
| MKT-05 | other | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |
| MKT-10 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| NUEVO-07 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| NUEVO-08 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| NUEVO-10 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| NUEVO-13 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| NUEVO-17 | other | task | 2026-08-02 (mtime) | 2026-08-02 | 2026-08-03 (mtime) | 1.0 | 1.0 |
| OLD-02 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| OLD-14 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| OLD-16 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| OLD-20 | other | task | 2026-07-28 | 2026-07-28 | 2026-07-28 | 0.0 | 0.0 |
| OLD-21 | other | task | 2026-08-03 | 2026-08-03 | 2026-08-03 | 0.0 | 0.0 |
| P2R-01 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-20 | 4.0 | 4.0 |
| PERF-02 | rust | task | 2026-08-12 | 2026-08-12 | 2026-08-20 | 8.0 | 8.0 |
| PERF-03 | rust | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| PERF-05 | rust | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| PERF-08 | rust | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| R1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R10 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R2 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| R3 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R5 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R6 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R7 | other | 2026-08-16-wave-r2-r7-fnd.md | 2026-08-16 (mtime) | 2026-08-16 | 2026-08-17 (mtime) | 1.0 | 1.0 |
| R8 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| R9 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| REST-03 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| REST-04 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| REST-05 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| REV-012 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| REVIEW-04 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| REVIEW-05 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| SDKB-01 | other | 2026-08-22-vantadb-bindings-sdk.md | 2026-08-22 (mtime) | 2026-08-22 | 2026-08-22 (mtime) | 0.0 | 0.0 |
| SDKB-02 | other | 2026-08-22-vantadb-bindings-sdk.md | 2026-08-22 (mtime) | 2026-08-22 | 2026-08-22 (mtime) | 0.0 | 0.0 |
| SKL-01 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| SKL-02 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| SKL-03 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| SKL-04 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| TECH-06 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| TECH-07 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| TEST-11 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| TEST-12 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| TIR-01 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TIR-02 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TIR-03 | other | task | 2026-08-12 | 2026-08-12 | 2026-08-12 | 0.0 | 0.0 |
| TIR-04 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-17 | 0.0 | 0.0 |
| TIR-05 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TIR-06 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TIR-07 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TIR-08 | other | task | 2026-08-17 | 2026-08-17 | 2026-08-12 | -5.0 | -5.0 |
| TSK-104 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| TSK-107b | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| TSYS-06 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| TSYS-06-F1 | other | task | 2026-08-16 | 2026-08-16 | 2026-08-16 | 0.0 | 0.0 |
| VFY-011 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| VS-02 | other | 2026-08-18-vanta-studio-fase0.budget.json | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-03 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-04 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-05 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-06 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-07 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-08 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-09 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-10 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-11 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-16 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-18 | other | 2026-08-18-vanta-studio-fase1.budget.json | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-CORE-01 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-CORE-02 | other | 2026-08-18-vanta-studio-fase0.budget.json | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| VS-CORE-07 | other | task | 2026-08-18 | 2026-08-18 | 2026-08-18 | 0.0 | 0.0 |
| WASM-01 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| WASM-02 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| WASM-03 | other | task | 2026-08-19 | 2026-08-19 | 2026-08-20 | 1.0 | 1.0 |
| WASM-PLAYGROUND-001 | other | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |
| WEB-01 | frontend | 2026-08-18-vanta-studio-fase3.budget.json | 2026-08-19 | 2026-08-19 | 2026-08-18 | -1.0 | -1.0 |
| WEB-04 | frontend | task | 2026-08-19 | 2026-08-19 | 2026-08-19 | 0.0 | 0.0 |
| WEB-18 | frontend | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |

> Nota: 0 tareas completadas sin fechas derivables quedan fuera de la tabla (estado ✅ sin fecha escrita ni mtime utilizable).

## 2. CFR (Change Failure Rate)

| Intentos de verify | Fallos | CFR |
|---|---|---|
| 124 | 60 | **48.4%** |

> Detalle de fallos por tarea: T1-residuo-consolidado(101), ERR-036(1), ERR-043(-1), CI-05(-1), AUD-033(-1), ?(-1), AUD-049(-1), AUD-049(1), AUD-049(-1), AUD-044(-1), AUD-051(-1), AUD-051(-1), AUD-051(-1), AUD-045(1), AUD-045(1), AUD-045(1), AUD-045(1), AUD-045(1), AUD-046(-1), AUD-048(-1), AUD-050(-1), VS-02(1), VS-18(-1), 1(-1), 3(1), 3(1), 3(1), 3(1), 3(1), 5(-1), 5(-1), 6(-1), 8(-1), 8(1), 8(1), WEB-00(1), WEB-02(-1), 2(-1), MEM-03(-1), 24(1), 24(1), 24(1), 24(1), 2(-1), 2(1), 3(-1), 3(-1), 3(1), 8(-1), 1(-1), ?(1), 4(1), 4(1), 4(1), 4(1), 4(1), 5(-1), 1(-1), GOV-T01(-1), 1(-1)

## 3. Recovery Time

| Task | Fallo (exit) | Comando | Clasificación | Δt |
|---|---|---|---|---|
| T1-residuo-consolidado | 101 | `cargo test -p vantadb` | fallo real | 12.56h |
| CI-05 | -1 | `python -c "import json; json.load(open('benchmarks/python_ba` | no-ejecutado (espurio) | 28.59h |
| AUD-033 | -1 | `cargo nextest run --profile audit -p vantadb-server --build-` | no-ejecutado (espurio) | 16.8s |
| AUD-049 | -1 | `& .venv\Scripts\python.exe -c "import vantadb; import vantad` | no-ejecutado (espurio) | 10min |
| AUD-049 | 1 | `& .venv\Scripts\python.exe -m pytest vantadb-python/tests/te` | fallo real | 4min |
| AUD-049 | -1 | `& .venv\Scripts\python.exe -c "import vantadb; import vantad` | no-ejecutado (espurio) | 8.2s |
| AUD-051 | -1 | `cargo check -p vantadb` | no-ejecutado (espurio) | 20.3s |
| AUD-051 | -1 | `cargo test -p vantadb --test cli_tests test_put_ 2>&1 | Sele` | no-ejecutado (espurio) | 10min |
| AUD-051 | -1 | `cargo nextest run --profile audit -p vantadb -E 'not test(te` | no-ejecutado (espurio) | 9.2s |
| AUD-045 | 1 | `cargo nextest run --profile audit -p vantadb-mcp --test mcp_` | fallo real | 1min |
| AUD-045 | 1 | `cargo nextest run -p vantadb-mcp --test mcp_tests` | fallo real | 1min |
| AUD-045 | 1 | `cargo nextest run -p vantadb-mcp -E 'binary(mcp_tests)'` | fallo real | 41.0s |
| AUD-045 | 1 | `cargo test -p vantadb-mcp --test mcp_tests` | fallo real | 21.4s |
| AUD-045 | 1 | `cargo fmt --check -p vantadb-mcp` | fallo real | 10.7s |
| VS-18 | -1 | `npm run build` | no-ejecutado (espurio) | 17.8s |
| 1 | -1 | `cargo check -p vantadb 2>&1 | Select-Object -Last 5` | no-ejecutado (espurio) | 29.52h |
| 3 | 1 | `npm run build` | fallo real | 2min |
| 3 | 1 | `npm --prefix vantadb-ts run build` | fallo real | 1min |
| 3 | 1 | `npm --prefix vantadb-ts test -- --run src/__tests__/vanta.te` | fallo real | 1min |
| 3 | 1 | `cargo check -p vantadb-desktop` | fallo real | 22.0s |
| 3 | 1 | `cargo nextest run --profile audit --workspace --build-jobs 2` | fallo real | 4min |
| 5 | -1 | `npm run build` | no-ejecutado (espurio) | 18.6s |
| 5 | -1 | `npm --prefix desktop run build` | no-ejecutado (espurio) | 15.3s |
| 8 | -1 | `npx vitest run src/store/undo.test.ts` | no-ejecutado (espurio) | 14.9s |
| 2 | -1 | `cargo check -p vantadb-mcp` | no-ejecutado (espurio) | 21.12h |
| 24 | 1 | `cargo nextest run -p vanta-memory` | fallo real | 1min |
| 24 | 1 | `cargo fmt --check` | fallo real | 1min |
| 24 | 1 | `cargo clippy -p vanta-memory --all-targets --no-deps -- -D w` | fallo real | 1min |
| 2 | -1 | `cargo check -p vanta-memory` | no-ejecutado (espurio) | 8.7s |
| 2 | 1 | `pwsh -NoProfile -Command "cargo clippy -p vanta-memory --all` | fallo real | 2min |
| 3 | -1 | `cargo nextest run -p vanta-memory memory_generation_log 2>&1` | no-ejecutado (espurio) | 13.71h |
| 3 | -1 | `cargo check -p vantadb 2>&1 | tail -20` | no-ejecutado (espurio) | 26.7s |
| 4 | 1 | `cargo check -p vanta-memory 2>&1 | Select-Object -Last 15` | fallo real | 44.7s |
| 4 | 1 | `cargo nextest run -p vanta-memory 2>&1 | Select-Object -Last` | fallo real | 10min |
| 4 | 1 | `cargo nextest run -p vanta-memory 2>&1 | Select-Object -Last` | fallo real | 10min |
| 4 | 1 | `cargo nextest run -p vanta-memory 2>&1 | Select-Object -Last` | fallo real | 10min |
| 4 | 1 | `cargo fmt --check && cargo clippy -p vanta-memory --all-targ` | fallo real | 1min |
| GOV-T01 | -1 | `node evals/dora.mjs` | no-ejecutado (espurio) | 1min |

**Δt promedio (solo fallos reales, exit ≠ -1):** 0.6h sobre 21 pares (17 espurios excluidos del promedio).


> Caveat: entradas con `taskId:null` (10 en el log actual) no son pareables — quedan fuera de esta métrica.

## 4. Throughput

| Periodo (días) | Tareas completadas |
|---|---|
| Últimos 7 | 136 |
| Últimos 30 | 266 |



## 5. Flow table

### Por tipo de tarea

| Tipo | Total | Pending | In-progress | Completed | Failed | Unknown |
|---|---|---|---|---|---|---|
| frontend | 8 | 0 | 0 | 3 | 0 | 5 |
| other | 339 | 23 | 1 | 209 | 0 | 106 |
| rust | 91 | 6 | 0 | 54 | 0 | 31 |

### Por plan file

| Plan | Total | Pending | In-progress | Completed | Failed | Lead avg (días) |
|---|---|---|---|---|---|---|
| 2026-08-16-wave-p20-tsys.budget.json (inicio —) | 2 | 0 | 0 | 1 | 0 | 0.0 |
| 2026-08-16-wave-r2-r7-fnd.md (inicio 2026-08-16) | 10 | 0 | 0 | 10 | 0 | 1.0 |
| 2026-08-18-vanta-memory.budget.json (inicio —) | 13 | 0 | 0 | 1 | 0 | 0.0 |
| 2026-08-18-vanta-studio-fase0.budget.json (inicio —) | 2 | 0 | 0 | 2 | 0 | 0.0 |
| 2026-08-18-vanta-studio-fase1.budget.json (inicio —) | 1 | 0 | 0 | 1 | 0 | 0.0 |
| 2026-08-18-vanta-studio-fase3.budget.json (inicio —) | 3 | 0 | 0 | 1 | 0 | -1.0 |
| 2026-08-19-web-design-audit.md (inicio 2026-08-19) | 9 | 9 | 0 | 0 | 0 | — |
| 2026-08-22-vantadb-bindings-sdk.budget.json (inicio —) | 1 | 0 | 0 | 1 | 0 | 0.0 |
| 2026-08-22-vantadb-bindings-sdk.md (inicio 2026-08-22) | 4 | 2 | 0 | 2 | 0 | 0.0 |

### Sin plan file (task files sueltos)

| Bucket | Total | No completadas | Completed |
|---|---|---|---|
| task:flat | 0 | 0 | 0 |
| task:complete | 0 | 0 | 0 |
| task:closed | 0 | 0 | 0 |

## 6. Limitaciones

- **Fechas no estructuradas**: los plan files no tienen un campo de timestamp por tarea normalizado; las fechas se extraen de markers ad-hoc (varían entre `✅ COMPLETADO (2026-08-10)`, `**Estado: completed`, `**Fecha:**`, ISO en task files, epoch ms en budget). Donde no hay marker se usó **file mtime** → esas filas son aproximaciones del día en que el archivo se tocó, no el día real del evento.
- **Cycle vs Lead**: sin budget `startTime` ni `**Creado:**`, startOfWork cae al plan `**Inicio:**` y cycle == lead. Los avgs por tipo mezclan ambas calidades.
- **CFR**: con `verify-log.jsonl` vacío (0 líneas) el 0% es baseline sin telemetría; no es evidencia de ausencia de fallos.
- **Throughput**: cuenta tareas ✅ COMPLETED con fecha de completado derivada; tareas completadas sin fecha quedan fuera.
- **Numeric budget keys**: claves numéricas de budget.json se resolvieron vía mapeo `Task N → id` del plan; si no hay match quedan como id numérico (tarea desconocida, cuenta como `unknown`).
- **Recovery time**: el emparejamiento fail→pass usa solo `taskId` (no `taskId+command`) porque los reintentos cambian flags entre intentos; un par con fallo `exitCode:-1` es "no-ejecutado" (timeout/kill, espurio), no evidencia de recovery real. Entradas sin taskId no son pareables.
- **P2-05 traceId**: con traceId por tarea, plan files deberán persistir `createdAt`/`completedAt` estructurados; este reporte se recalculará sin fallback mtime.
- **Cobertura de task files**: los plan files P0/P1/P2/P3 no tienen todavía task files propios en `tasks/` (referencian `skills/campaign-executor/tasks/P*-NN.md` que aún no existen) — sus estados vienen del plan; los task files existentes (AUD/ERR/INV/COMP/DESKTOP/…) alimentan la vista de detalle.
