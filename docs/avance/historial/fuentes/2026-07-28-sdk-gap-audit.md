---
title: "SDK Gap Audit + Recovery Plan — Recitation"
type: session-report
status: completed
tags: [vantadb, audit, sdk, cli, recitation]
session_date: 2026-07-28
aliases: []
---

# Session: 2026-07-28 — SDK Gap Audit + Recovery Plan

Campaign: `auditoria-sdk-2026-07-28`

**Objetivo:** Investigar 12 puntos sobre SDK/CLI missing features contra código real y git history.

**Estado:** ✅ COMPLETED

**Hallazgo clave:** `delete_by_filter()`, `count()`, `similar_to_key()` NUNCA existieron como SDK — solo CLI handlers eliminados en AUD-09 (`e9371ea8`). Multi-namespace solo en tipos de output report.

## Outputs

- `docs/plans/2026-07-28-recovery-plan.md` — plan detallado (37KB, 11 tasks, 4 fases)
- `docs/Backlog.md` Phase 8 — SDK-01 a SDK-05 existentes + REC-001, REC-007–010, REC-999 agregados (filas SDK-01..05, REC-001, REC-007, REC-999 — actualizadas 2026-08-03, todas ✅ tachadas)

## Corrección al plan

SDK-01/02/03/04/05 YA estaban en backlog (no REC-002/003/004/005/006).

## IDs finales

| ID | Tarea |
|----|-------|
| REC-001 | Foundation types |
| REC-007 | WAL CLI |
| REC-008 | Backup design |
| REC-009 | PQ analysis |
| REC-010 | py.typed |
| REC-999 | progreso fix |

**Próxima acción sugerida (en su momento):** ejecutar REC-010 primero (🟢 30min), después REC-001 (foundation types).

**Contrato:** Plan recuperación + backlog actualizados. Plan referencias REC IDs internas, backlog usa SDK-XX + REC-XX.
