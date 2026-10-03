---
title: "Pipeline Evaluation Report"
kind: report
description: "Métricas del pipeline vs North Star (RULES.md) — verify calls, primer intento, regresiones — generado por evals/eval-metrics.mjs"
---

# Pipeline Evaluation Report

> Generado por `evals/eval-metrics.mjs` (EVAL-01) — 2026-10-03T06:19:49.702Z
> Datos: `.opencode/task-system/enforcement/verify-log.jsonl` (505 invocaciones de verify) + `docs/dev/plans/*.md`

## North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | 45.1% | 🚩 |
| Falsos positivos (COMPLETED con verify fallido) | 0 | 0 | ✅ |
| Regresión silenciosa (verify falla tras pasar) | 0 | 32 | 🚩 |

## Por tipo de tarea

| Tipo | Tareas |
|---|---|
| other | 70 |
| rust | 1 |

## Skills → primer intento (P3-rem)

| Skill | Tareas con skill | Primer intento ✅ | Tasa |
|---|---|---|---|
| source-driven-development | 30 | 14 | 46.7% |
| campaign-executor | 23 | 9 | 39.1% |
| test-driven-development | 21 | 8 | 38.1% |
| incremental-implementation | 21 | 9 | 42.9% |
| progreso | 20 | 8 | 40.0% |
| doubt-driven-development | 17 | 6 | 35.3% |
| writing-guidelines | 13 | 6 | 46.2% |
| ci-cd-and-automation | 11 | 7 | 63.6% |
| api-and-interface-design | 11 | 4 | 36.4% |
| writing-plans | 11 | 5 | 45.5% |
| documentation-and-adrs | 10 | 5 | 50.0% |
| git-workflow-and-versioning | 10 | 6 | 60.0% |
| context-engineering | 9 | 2 | 22.2% |
| security-and-hardening | 8 | 5 | 62.5% |
| ponytail | 6 | 2 | 33.3% |
| systematic-debugging | 5 | 2 | 40.0% |
| documentation-skill | 5 | 3 | 60.0% |
| spec-driven-development | 4 | 1 | 25.0% |
| deprecation-and-migration | 4 | 2 | 50.0% |
| coordinated-web-search | 4 | 1 | 25.0% |
| rust-write-tests | 3 | 1 | 33.3% |
| frontend-ui-engineering | 2 | 0 | 0.0% |
| interview-me | 2 | 1 | 50.0% |
| performance-optimization | 1 | 0 | 0.0% |

> Tareas con skills registradas: 44 / 71 (el resto no tenía "Archivos clave" derivables o log previo a P3-rem).

## Detalle por tarea

| Task | Plan | Verify ok | Verify fail | Primer intento | Regresiones | Estado final |
|---|---|---|---|---|---|---|
| 1 | — | 4 | 0 | ✅ | 0 | — |
| 14 | — | 4 | 0 | ✅ | 0 | — |
| 3 | — | 0 | 1 | ❌ | 0 | — |
| 4 | — | 0 | 2 | ❌ | 0 | — |
| 7 | — | 1 | 0 | ❌ | 1 | — |
| 8 | — | 4 | 0 | ✅ | 0 | — |
| API-01 | — | 18 | 1 | ✅ | 13 | — |
| API-03 | — | 3 | 0 | ❌ | 1 | — |
| API-05 | — | 0 | 2 | ❌ | 0 | — |
| API-06 | — | 7 | 0 | ❌ | 1 | — |
| API-07 | — | 5 | 0 | ❌ | 1 | — |
| AST-001 | — | 2 | 0 | ❌ | 1 | — |
| AST-011 | — | 0 | 1 | ❌ | 0 | — |
| BND-12 | — | 0 | 1 | ❌ | 0 | — |
| C-07 | — | 2 | 0 | ❌ | 4 | — |
| C-10 | — | 2 | 0 | ❌ | 3 | — |
| DEF-01 | — | 5 | 1 | ✅ | 3 | — |
| DEF-02 | — | 13 | 0 | ❌ | 3 | — |
| DEF-03 | — | 3 | 0 | ✅ | 6 | — |
| DEF-04 | — | 4 | 0 | ✅ | 0 | — |
| DEF-05 | — | 5 | 0 | ❌ | 1 | — |
| DEF-07 | — | 1 | 0 | ❌ | 1 | — |
| DEF-08 | — | 6 | 0 | ❌ | 1 | — |
| EST-05 | — | 1 | 0 | ✅ | 0 | — |
| FIND-103 | — | 1 | 0 | ✅ | 0 | — |
| FIND-105 | — | 0 | 1 | ❌ | 0 | — |
| FIND-106 | — | 1 | 0 | ✅ | 0 | — |
| FIND-109 | — | 1 | 0 | ✅ | 0 | — |
| FIND-110 | — | 0 | 1 | ❌ | 0 | — |
| FIND-114 | — | 0 | 1 | ❌ | 0 | — |
| FIND-119 | — | 1 | 0 | ✅ | 0 | — |
| FIND-225 | — | 1 | 0 | ✅ | 0 | — |
| FIND-226 | — | 1 | 1 | ✅ | 1 | — |
| FIND-227 | — | 3 | 0 | ❌ | 1 | — |
| FIND-228 | — | 5 | 0 | ✅ | 0 | — |
| FIND-230 | — | 5 | 0 | ✅ | 0 | — |
| FIND-231 | — | 3 | 0 | ❌ | 3 | — |
| FIND-48 | — | 1 | 0 | ✅ | 0 | — |
| FIND-50 | — | 0 | 1 | ❌ | 0 | — |
| FIND-54 | — | 0 | 1 | ❌ | 0 | — |
| FIND-63 | — | 3 | 1 | ✅ | 1 | — |
| FIND-64 | — | 1 | 0 | ❌ | 1 | — |
| FIND-70 | — | 0 | 1 | ❌ | 0 | — |
| FIND-88 | — | 1 | 1 | ✅ | 1 | — |
| FIND-90 | — | 0 | 1 | ❌ | 0 | — |
| FIND-91 | — | 1 | 0 | ✅ | 0 | — |
| FIND-BND12-01 | — | 1 | 0 | ✅ | 0 | — |
| GOV-A5 | — | 1 | 0 | ✅ | 0 | — |
| HARD-01 | — | 6 | 0 | ❌ | 5 | — |
| HARD-03 | — | 1 | 0 | ✅ | 0 | — |
| HARD-04 | — | 3 | 0 | ❌ | 1 | — |
| HARD-05 | — | 13 | 0 | ❌ | 5 | — |
| HARD-06 | — | 9 | 0 | ❌ | 5 | — |
| HARD-07 | — | 27 | 0 | ✅ | 11 | — |
| IMPL-112-S2 | — | 0 | 1 | ❌ | 0 | — |
| MEM-66 | — | 0 | 1 | ❌ | 0 | — |
| MGR-13 | — | 0 | 2 | ❌ | 0 | — |
| MKT-18i | — | 2 | 0 | ❌ | 1 | — |
| PERF-BENCH-01 | — | 1 | 0 | ✅ | 0 | — |
| PROC-04 | — | 2 | 0 | ✅ | 0 | — |
| R-03 | — | 1 | 0 | ❌ | 1 | — |
| SCH-02 | — | 1 | 2 | ❌ | 3 | — |
| SHOW-02 | — | 1 | 0 | ✅ | 0 | — |
| SHOW-04 | — | 1 | 0 | ✅ | 0 | — |
| STABLE-04 | — | 2 | 0 | ✅ | 0 | — |
| UX-19 | — | 0 | 1 | ❌ | 0 | — |
| WIRE-02 | — | 2 | 0 | ✅ | 0 | — |
| WIRE-03 | — | 4 | 0 | ❌ | 1 | — |
| WIRE-07 | — | 25 | 1 | ✅ | 9 | — |
| WIRE-08 | — | 6 | 0 | ✅ | 2 | — |
| WIRE-10 | — | 8 | 0 | ❌ | 3 | — |

## Notas
- Un "Primer intento" = la primera invocación de verify de la tarea pasó.
- "Regresiones" cuenta verifies que fallaron después de haber pasado para la misma tarea.
- El log se alimenta automáticamente desde `campaign_verify_cmd`; este reporte es la referencia del threshold de RULES.md.
