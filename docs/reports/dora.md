# DORA Flow Metrics Report

> Generado por `evals/dora.mjs` (P3-07) — 2026-08-10T21:41:08.127Z
> Fuentes: `docs/plans/*.md` (50 tareas en 4 planes) + task files en `.opencode/skills/campaign-executor/tasks/` (180) + `*.budget.json` (3 con timestamps) + `verify-log.jsonl` (0 intentos de verify)
> ⚠️ **Fechas derivadas best-effort, NO normalizadas**. Prioridad: markers escritos (`**Inicio:**`, `**Estado:** COMPLETADO (fecha)`, `**Fecha:**`, `**Creado:**`, fechas en bloque de tarea) -> budget epoch ms (`startTime`/`lastActivity`) -> **file mtime**. Donde se usó mtime se marca `(mtime)`. Esto es exactamente lo que P2-05 (traceId por tarea) va a resolver: con traceId real, cada task tendrá timestamps estructurados.
> ⚠️ **verify-log.jsonl está vacío (0 líneas)** — CFR reportado en 0% como baseline; no hay intentos registrados todavía.


## 1. Cycle / Lead time

| Métrica | Definición pragmática |
|---|---|
| **Lead time** | completado − request. Request = plan `**Inicio:**` del plan que la contiene, fallback `**Creado:**` del task file, fallback budget `startTime`, fallback mtime. |
| **Cycle time** | completado − startOfWork. Start = budget `startTime` (timestamps reales) → `**Creado:**`/`**Inicio:**` del task file → plan inicio. Sin budget ni Creado, cycle == lead (mismo origen). |

### Por tipo de tarea

| Tipo | Tasks | Completed | Lead avg (días) | Cycle avg (días) |
|---|---|---|---|---|
| frontend | 2 | 1 | 0.0 | 0.0 |
| other | 158 | 85 | 0.8 | 0.8 |
| rust | 64 | 26 | 0.8 | 0.6 |

### Por tarea (completadas con fechas derivables)

| Task | Tipo | Plan | Request | Start | Completed | Lead | Cycle |
|---|---|---|---|---|---|---|---|
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
| AUD-031 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDIT-02 | rust | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| AUDIT-03 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-01 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-04 | rust | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| AUDREP-12 | rust | task | 2026-08-06 | 2026-08-06 | 2026-08-06 | 0.0 | 0.0 |
| CLI-01 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| COMP-006 | other | task | 2026-07-27 | 2026-07-27 | 2026-07-27 | 0.0 | 0.0 |
| COMP-008 | other | task | 2026-07-27 | 2026-07-27 | 2026-07-27 | 0.0 | 0.0 |
| COMP-021 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| COMP-028 | other | task | 2026-08-02 | 2026-08-02 | 2026-07-28 | -5.0 | -5.0 |
| COMP-029 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
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
| ERR-026 | rust | 2026-08-09-residual-hardening.md | 2026-08-09 (mtime) | 2026-08-09 | 2026-08-10 (mtime) | 1.0 | 1.0 |
| ERR-036 | rust | 2026-08-09-residual-hardening.md | 2026-08-09 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 1.0 | 0.0 |
| ERR-037 | rust | 2026-08-09-residual-hardening.md | 2026-08-09 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 1.0 | 0.0 |
| ERR-042 | rust | 2026-08-09-residual-hardening.md | 2026-08-09 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 1.0 | 0.0 |
| ERR-044 | rust | 2026-08-09-residual-hardening.md | 2026-08-09 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 1.0 | 0.0 |
| EVAL-01 | other | 2026-08-10-p0-harness.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| EVAL-02 | other | 2026-08-10-p0-harness.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| EVAL-03 | other | 2026-08-10-p0-harness.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| EVAL-04 | other | 2026-08-10-p0-harness.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| GH-119 | other | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |
| GH-123 | other | task | 2026-08-04 (mtime) | 2026-08-04 | 2026-08-05 (mtime) | 1.0 | 1.0 |
| GH-124 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-127 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-142 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| GH-143 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
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
| P1-01 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-02 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-03 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-04 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-05 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-06 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| P1-07 | other | 2026-08-10-p1-process-discipline.md | 2026-08-10 (mtime) | 2026-08-10 | 2026-08-10 (mtime) | 0.0 | 0.0 |
| REV-012 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| TECH-06 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| TECH-07 | other | task | 2026-08-05 | 2026-08-05 | 2026-08-05 | 0.0 | 0.0 |
| TEST-11 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| TEST-12 | other | task | 2026-07-31 (mtime) | 2026-07-31 | 2026-07-31 (mtime) | 0.0 | 0.0 |
| TSK-104 | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| TSK-107b | other | task | 2026-08-02 | 2026-08-02 | 2026-08-02 | 0.0 | 0.0 |
| VFY-011 | other | task | 2026-07-26 | 2026-07-26 | 2026-07-26 | 0.0 | 0.0 |
| WEB-18 | frontend | task | 2026-08-04 | 2026-08-04 | 2026-08-04 | 0.0 | 0.0 |

> Nota: 0 tareas completadas sin fechas derivables quedan fuera de la tabla (estado ✅ sin fecha escrita ni mtime utilizable).

## 2. CFR (Change Failure Rate)

| Intentos de verify | Fallos | CFR |
|---|---|---|
| 0 | 0 | **0.0%** |

> ⚠️ Sin intentos registrados en `verify-log.jsonl` — CFR 0% es **baseline sin datos**, no un resultado real. El log se alimenta desde `campaign_verify_cmd`.

## 3. Throughput

| Periodo (días) | Tareas completadas |
|---|---|
| Últimos 7 | 65 |
| Últimos 30 | 112 |



## 4. Flow table

### Por tipo de tarea

| Tipo | Total | Pending | In-progress | Completed | Failed | Unknown |
|---|---|---|---|---|---|---|
| frontend | 2 | 0 | 1 | 1 | 0 | 0 |
| other | 158 | 25 | 1 | 85 | 0 | 47 |
| rust | 64 | 22 | 0 | 26 | 0 | 16 |

### Por plan file

| Plan | Total | Pending | In-progress | Completed | Failed | Lead avg (días) |
|---|---|---|---|---|---|---|
| 2026-08-09-residual-hardening.md (inicio 2026-08-09) | 26 | 21 | 0 | 5 | 0 | 1.0 |
| 2026-08-10-p0-harness.md (inicio 2026-08-10) | 4 | 0 | 0 | 4 | 0 | 0.0 |
| 2026-08-10-p1-process-discipline.md (inicio 2026-08-10) | 7 | 0 | 0 | 7 | 0 | 0.0 |
| 2026-08-10-p2-p3-structural-quality.md (inicio 2026-08-10) | 13 | 13 | 0 | 0 | 0 | — |

### Sin plan file (task files sueltos)

| Bucket | Total | No completadas | Completed |
|---|---|---|---|
| task:flat | 0 | 0 | 0 |
| task:complete | 0 | 0 | 0 |
| task:closed | 0 | 0 | 0 |

## 5. Limitaciones

- **Fechas no estructuradas**: los plan files no tienen un campo de timestamp por tarea normalizado; las fechas se extraen de markers ad-hoc (varían entre `✅ COMPLETADO (2026-08-10)`, `**Estado: completed`, `**Fecha:**`, ISO en task files, epoch ms en budget). Donde no hay marker se usó **file mtime** → esas filas son aproximaciones del día en que el archivo se tocó, no el día real del evento.
- **Cycle vs Lead**: sin budget `startTime` ni `**Creado:**`, startOfWork cae al plan `**Inicio:**` y cycle == lead. Los avgs por tipo mezclan ambas calidades.
- **CFR**: con `verify-log.jsonl` vacío (0 líneas) el 0% es baseline sin telemetría; no es evidencia de ausencia de fallos.
- **Throughput**: cuenta tareas ✅ COMPLETED con fecha de completado derivada; tareas completadas sin fecha quedan fuera.
- **Numeric budget keys**: claves numéricas de budget.json se resolvieron vía mapeo `Task N → id` del plan; si no hay match quedan como id numérico (tarea desconocida, cuenta como `unknown`).
- **P2-05 traceId**: con traceId por tarea, plan files deberán persistir `createdAt`/`completedAt` estructurados; este reporte se recalculará sin fallback mtime.
- **Cobertura de task files**: los plan files P0/P1/P2/P3 no tienen todavía task files propios en `tasks/` (referencian `skills/campaign-executor/tasks/P*-NN.md` que aún no existen) — sus estados vienen del plan; los task files existentes (AUD/ERR/INV/COMP/DESKTOP/…) alimentan la vista de detalle.
