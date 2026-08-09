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

**Propietario del pipeline:** campaign-server.mjs (MCP), harness-executor.ps1, prompts pipeline-full.md/plan.md. Los plan files son el **estado de ejecución vivo** del sistema de tareas; el harness busca el más reciente por defecto. NO mover.

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

## 2. `docs/audit-reports/` — Reportes de auditoría

**Propietario del pipeline:** `dev-tools/audit-all.ps1` (`$ReportDir`), prompt `audit-full.md` (escribe `audit-<mode>-<timestamp>.md`). `vantadb-audit-report.md` es el reporte estático multi-agente principal.

| Archivo | Líneas | Tipo |
|---|---|---|
| `vantadb-audit-report.md` | 915 | Auditoría estática multi-agente (principal) |
| `audit-full-2026-07-18.md` | 165 | Audit completo |
| `audit-full-2026-07-24.md` | 135 | Audit completo |
| `audit-full-2026-07-24T1751Z.md` | 52 | Audit (resumen) |
| `audit-full-2026-08-04T174544.md` | 87 | Audit reciente (IDs AUDIT-01..08) |
| `deps-01-duplicadas-2026-08-05.md` | 31 | Investigación dependencias |
| `inv-001-rustsec-2026-07-29.md` | 52 | Investigación rustsec |
| `inv-024-unsafe-audit-2026-07-30.md` | 148 | Investigación unsafe blocks |
| `archive/` (5 files) | — | Intermedios superados / backups pre-2026-08-05 |

## 3. `docs/reviews/` — Reportes de review

**Propietario pipeline:** skill `unified-review` (`/review`), reportes `review-<mode>-<timestamp>.json`.

| Archivo | Estado |
|---|---|
| `review-full-2026-07-27-0309.md` | 457 | Review completo |
| `review-certify-2026-08-05-2025.md` | 74 | Certify reciente |
| `review-full-2026-08-05-t1545.md` | 53 | Review rápido |
| `archive/review.md` | 61 | Archivado |

## 4. `docs/Investigaciones/` — Investigaciones

**Ya catalogada 1:1 en `docs/avance/investigaciones.md`** (incluye todos los INV-003..020, DESKTOP-01/01b, vectara, meta-001, ACID). No duplicar aquí el índice — ver `investigaciones.md`.