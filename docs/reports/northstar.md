# North Star Report

> Generado por `evals/northstar.mjs` (P1-06) — 2026-08-11T20:24:42.570Z
> Datos: `.opencode/task-system/enforcement/verify-log.jsonl` (2 invocaciones de verify) + `docs/plans/*.md` (51 tareas) + `docs/plans/*.budget.json` (6 tareas trackeadas)


## Definiciones (documentadas en este header)

| Métrica | Definición pragmática |
|---|---|
| **Primer intento** | Tarea con estado ✅ COMPLETED sin evidencia de fallo antes de completarse: ninguna invocación verify fallida (`passed:false`), ni `consecutiveFails>0` en budget, ni primer verify fallido. Sin datos de verify ni budget se asume primer intento (best-effort). |
| **Falso positivo** | Registro donde una verificación no fue confiable: (a) **verified-then-rerun** = la tarea se verificó (`passed:true`) y luego se volvió a invocar verify (>1 invocación); (b) **verified-then-failed** = pasó una verify y falló en otra posterior, o quedó ✅ COMPLETED con verify fallido; (c) **budget fail** = `consecutiveFails>0` en budget para una tarea. Headline = unión de tareas en (a)+(b)+(c). |
| **Regresión** | Tarea que pasó verify al menos una vez y luego falló en una verify posterior (mismo `taskId`: patrón passed→failed). Equivale a la "regresión silenciosa" de RULES.md (romper tests que antes pasaban). |

## 1. Tasa completado primer intento

| Métrica | Valor |
|---|---|
| Tareas ✅ COMPLETED | 37 |
| Completadas en primer intento (sin fallos registrados) | 37 |
| **Tasa primer intento** | **100.0%** |

## 2. Falsos positivos

| Componente | Count |
|---|---|
| COMPLETED con verify fallido | 0 |
| Verified-then-rerun (>1 invocación verify) | 1 |
| Budget fails (consecutiveFails > 0) | 0 |
| **Falsos positivos (unión)** | **1** |

## 3. Regresión

| Métrica | Valor |
|---|---|
| Tareas con patrón passed→failed (verify) | 0 |

## 4. Comparación contra North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | 100.0% | ✅ |
| Falsos positivos | 0 | 1 | 🚩 |
| Regresión silenciosa | 0 | 0 | ✅ |

## 5. Calibración de telemetría (P3-rem)

| Métrica | Valor |
|---|---|
| Tareas con skills registradas en verify-log | 1 |
| Cobertura de telemetría (tareas con skills / tareas con verify) | 100.0% |

> Este indicador mide cuánto input de calibración (skill/tool → primer intento) se recolecta para el harness de evals (P0-1). ✅ Recolectando.



## Por tipo de tarea

| Tipo | Tareas |
|---|---|
| other | 32 |
| rust | 21 |

## Detalle por tarea

| Task | Plan | Estado | Verify ok | Verify fail | Primer intento | Presencia regresión | Budget fails |
|---|---|---|---|---|---|---|---|
| Archivar | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Arreglar | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Asincron | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| AUD-016 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| AUD-018 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| AUD-020 | 2026-08-11-residuo-consolidado.md | completed | 0 | 0 | ✅ | — | 0 |
| AUD-021 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| Checklist | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| CI-01 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| Commitear | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Contrato | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Correcci | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Corregir | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| COV-001 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| COV-002 | 2026-08-11-residuo-consolidado.md | completed | 0 | 0 | ✅ | — | 0 |
| COV-003 | 2026-08-11-residuo-consolidado.md | completed | 0 | 0 | ✅ | — | 0 |
| COV-004 | 2026-08-11-residuo-consolidado.md | completed | 0 | 0 | ✅ | — | 0 |
| Crear | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Definir | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Eliminar | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-006 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-008 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-015 | 2026-08-11-residuo-consolidado.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-026 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-031 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-032 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-033 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-036 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-037 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-042 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-043 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-044 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-045 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-047 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| ERR-048 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| Estado | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Fix | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Human-in-the-loop | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Items | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| L | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Memoria | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Mover | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Observabilidad | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| PERF-07 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| PERF-09 | 2026-08-09-residual-hardening.md | completed | 0 | 0 | ✅ | — | 0 |
| Poblar | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |
| Resolver | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Restaurar | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Sincronizar | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| T1-residuo | 2026-08-11-residuo-consolidado.budget.json | sin-plan | 0 | 0 | ✅ | — | 0 |
| T1-residuo-consolidado | 2026-08-11-residuo-consolidado.budget.json | sin-plan | 1 | 1 | ❌ | — | 0 |
| Unificar | 2026-08-10-docs-task-system-consolidation.md | completed | 0 | 0 | ✅ | — | 0 |
| Validaci | 2026-08-11-residuo-consolidado.md | pending | 0 | 0 | ✅ | — | 0 |

## Notas
- "Primer intento" se infiere de plan + budget + verify-log; sin telemetría de verify la tasa es best-effort (asume primer intento cuando no hay evidencia de fallo).
- Falsos positivos y regresión se solapan por diseño: una tarea COMPLETED con patrón passed→failed cuenta en ambas — el headline de FP es unión de tareas, la regresión es el patrón de verify.
- El log se alimenta automáticamente desde `campaign_verify_cmd` (campaign-server.mjs); los budget.json se alimentan desde `consumeBudget`. Este reporte es la referencia del threshold de RULES.md.
- Fuente de planes: `docs/plans/*.md` raíz (el subdirectorio `archive/` no se incluye).
