---
title: "Fuentes Vivas — Catálogo de Carpetas Externas"
type: index
status: active
tags: [vantadb, avance, index, live-sources, plans, audit-reports, reviews]
last_reviewed: 2026-08-07
aliases: [docs/avance/fuentes-vivas]
---

# Fuentes Vivas — Carpetas Externas Referenciadas

> Estas carpetas **NO viven dentro de `docs/avance`**: son escritas por pipelines activos que las buscan por ruta fija (task system MCP, scripts de audit, skills de review). Moverlas físicamente rompería esos flujos. Aquí se catalogan por referencia: índice de archivos + estado + ruta directa.

## 1. `docs/plans/` — Plan files (sistema de tareas)

**Propietario del pipeline:** campaign-server.mjs (MCP), prompts pipeline-full.md/plan.md. Los plan files son el **estado de ejecución vivo** del sistema de tareas; `pipeline-run.md` busca el más reciente por defecto. NO mover.

| Plan | Líneas | Estado |
|---|---|---|
| `2026-07-28-recovery-plan.md` | 622 | ⚠️ Superado (post-recovery) |
| `2026-07-29-index-rebuild-execution.md` | 146 | ✅ Completado |
| `2026-08-04-launch-web-campaign.md` | 120 | ⚠️ Paralelo a web-release |
| `2026-08-05-backlog-validation-actions.md` | 427 | ✅ Completado (sync Backlog 2026-08-06) |
| `2026-08-06-desktop-mvp.md` | 268 | 🟡 Activo (Fase 12 Tauri) |
| `2026-08-06-oc-vantadb-pro.md` | 224 | 🟡 Activo |
| `PROMPT-MAESTRO-FREEZE.md` | 193 | ✅ Freeze completado |
| `archive/2026-07-25-p4-drv130-t3-node-reordering.md` | 32 | ✅ Archivado |
| `*.budget.json` | — | Estado de presupuesto de la campaña correspondiente |

## 2. `docs/reviews/` — Reportes de auditoría y review

**Propietario del pipeline:** `dev-tools/audit-all.ps1` (`$ReportDir`), prompt `audit-full.md` (escribe `audit-<mode>-<timestamp>.md`), skill `unified-review` (`/review`, reportes `review-<mode>-<timestamp>.md`). Unificado: `/audit` ≡ `/review` (alias legacy) escriben en `docs/reviews/`. `audit-full-2025-07-27.md` (antes `vantadb-audit-report.md`) es el reporte estático multi-agente principal — archivado 2026-08-09 al estar 100% procesado (P13 AUDREP).

| Archivo | Líneas | Tipo |
|---|---|---|
| `review-full-2026-07-27-0309.md` | 457 | Review completo |
| `review-certify-2026-08-05-2025.md` | 74 | Certify reciente |
| `review-full-2026-08-05-t1545.md` | 53 | Review rápido |
| `audit-full-20260808-002617.md` | — | Audit completo vigente |
| `archive/review.md` | 61 | Archivado |
| `archive/` (14 files) | — | Audits/reviews superados pre-unificación + `audit-full-2025-07-27.md` |

## 4. `docs/research/` — Investigaciones

**Ya catalogada 1:1 en `docs/avance/investigaciones.md`** (incluye todos los INV-003..020, DESKTOP-01/01b, vectara, meta-001, ACID). No duplicar aquí el índice — ver `investigaciones.md`.