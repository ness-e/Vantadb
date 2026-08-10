# North Star Report

> Generado por `evals/northstar.mjs` (P1-06) — 2026-08-10T20:29:30.066Z
> Datos: `.opencode/task-system/enforcement/verify-log.jsonl` (0 invocaciones de verify) + `docs/plans/*.md` (37 tareas) + `docs/plans/*.budget.json` (3 tareas trackeadas)
> ⚠️ **verify-log.jsonl está vacío** — sin telemetría de verificación, las métricas se reportan en 0 y los thresholds no pueden evaluarse aún.


## Definiciones (documentadas en este header)

| Métrica | Definición pragmática |
|---|---|
| **Primer intento** | Tarea con estado ✅ COMPLETED sin evidencia de fallo antes de completarse: ninguna invocación verify fallida (`passed:false`), ni `consecutiveFails>0` en budget, ni primer verify fallido. Sin datos de verify ni budget se asume primer intento (best-effort). |
| **Falso positivo** | Registro donde una verificación no fue confiable: (a) **verified-then-rerun** = la tarea se verificó (`passed:true`) y luego se volvió a invocar verify (>1 invocación); (b) **verified-then-failed** = pasó una verify y falló en otra posterior, o quedó ✅ COMPLETED con verify fallido; (c) **budget fail** = `consecutiveFails>0` en budget para una tarea. Headline = unión de tareas en (a)+(b)+(c). |
| **Regresión** | Tarea que pasó verify al menos una vez y luego falló en una verify posterior (mismo `taskId`: patrón passed→failed). Equivale a la "regresión silenciosa" de RULES.md (romper tests que antes pasaban). |

## 1. Tasa completado primer intento

| Métrica | Valor |
|---|---|
| Tareas ✅ COMPLETED | 9 |
| Completadas en primer intento (sin fallos registrados) | 9 |
| **Tasa primer intento** | **100.0%** |

## 2. Falsos positivos

| Componente | Count |
|---|---|
| COMPLETED con verify fallido | 0 |
| Verified-then-rerun (>1 invocación verify) | 0 |
| Budget fails (consecutiveFails > 0) | 0 |
| **Falsos positivos (unión)** | **0** |

## 3. Regresión

| Métrica | Valor |
|---|---|
| Tareas con patrón passed→failed (verify) | 0 |

## 4. Comparación contra North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | 100.0% | ⚠️ |
| Falsos positivos | 0 | 0 | ✅ |
| Regresión silenciosa | 0 | 0 | ⚠️ |

> ⚠️ Sin telemetría de verify — los thresholds de primer intento y regresión **no pueden evaluarse aún** (baseline pendiente); con budget solo, falsos positivos es parcialmente evaluable.


## Por tipo de tarea

| Tipo | Tareas |
|---|---|
| rust | 21 |
| other | 16 |

## Detalle por tarea

| Task | Plan | Estado | Verify ok | Verify fail | Primer intento | Presencia regresión | Budget fails |
|---|---|---|---|---|---|---|---|
| AUD-016 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| AUD-018 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| AUD-020 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| AUD-021 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| CI-01 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| COV-001 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| COV-002 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| COV-003 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| COV-004 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-006 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-008 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-015 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-026 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-031 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-032 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-033 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-036 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-037 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-042 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-043 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-044 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-045 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-047 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| ERR-048 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| EVAL-01 | 2026-08-10-p0-harness.md | completed | 0 | 0 | ✅ | — | 0 |
| EVAL-02 | 2026-08-10-p0-harness.md | completed | 0 | 0 | ✅ | — | 0 |
| EVAL-03 | 2026-08-10-p0-harness.md | completed | 0 | 0 | ✅ | — | 0 |
| EVAL-04 | 2026-08-10-p0-harness.md | completed | 0 | 0 | ✅ | — | 0 |
| P1-01 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-02 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-03 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-04 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-05 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-06 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| P1-07 | 2026-08-10-p1-process-discipline.md | pending | 0 | 0 | ✅ | — | 0 |
| PERF-07 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |
| PERF-09 | 2026-08-09-residual-hardening.md | pending | 0 | 0 | ✅ | — | 0 |

## Notas
- "Primer intento" se infiere de plan + budget + verify-log; sin telemetría de verify la tasa es best-effort (asume primer intento cuando no hay evidencia de fallo).
- Falsos positivos y regresión se solapan por diseño: una tarea COMPLETED con patrón passed→failed cuenta en ambas — el headline de FP es unión de tareas, la regresión es el patrón de verify.
- El log se alimenta automáticamente desde `campaign_verify_cmd` (campaign-server.mjs); los budget.json se alimentan desde `consumeBudget`. Este reporte es la referencia del threshold de RULES.md.
- Fuente de planes: `docs/plans/*.md` raíz (el subdirectorio `archive/` no se incluye).
