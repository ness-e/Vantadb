---
title: Fuentes Vivas — Catálogo de Carpetas Externas
kind: review
status: active
description: "Propietario del pipeline: campaign-server.mjs (MCP), prompts pipeline-full.md/plan.md. Los plan files son el estado de ejecución vivo del sistema de tareas; pipeline-run.md busca el más reciente por defecto. NO mover"
aliases: [docs/dev/avance/fuentes-vivas]
tags: [vantadb, avance, index, live-sources, plans, audit-reports, reviews]
---

# Fuentes Vivas — Carpetas Externas Referenciadas

> Estas carpetas **NO viven dentro de `docs/dev/avance`**: son escritas por pipelines activos que las buscan por ruta fija (task system MCP, scripts de audit, skills de review). Moverlas físicamente rompería esos flujos. Aquí se catalogan por referencia: índice de archivos + estado + ruta directa.

## 1. `docs/dev/plans/` — Plan files (sistema de tareas)

**Propietario del pipeline:** campaign-server.mjs (MCP), prompts pipeline-full.md/plan.md. Los plan files son el **estado de ejecución vivo** del sistema de tareas; `pipeline-run.md` busca el más reciente por defecto. NO mover.

| Plan | Líneas | Estado |
|---|---|---|
| `2026-07-28-recovery-plan.md` | 622 | ⚠️ Superado (post-recovery) |
| `2026-07-29-index-rebuild-execution.md` | 146 | ✅ Completado |
| `2026-08-04-launch-web-campaign.md` | 120 | 📦 Migrado 2026-09-22 a Vantadb-web `docs/history/` (WEB-DOCS-MOVE) |
| `2026-08-05-backlog-validation-actions.md` | 427 | ✅ Completado (sync Backlog 2026-08-06) |
| `2026-08-06-desktop-mvp.md` | 268 | 🟡 Activo (Fase 12 Tauri) |
| `2026-08-06-oc-vantadb-pro.md` | 224 | 🟡 Activo |
| `PROMPT-MAESTRO-FREEZE.md` | 193 | ✅ Freeze completado |
| `archive/2026-07-25-p4-drv130-t3-node-reordering.md` | 32 | ✅ Archivado |
| `*.budget.json` | — | Estado de presupuesto de la campaña correspondiente |

## 2. `docs/dev/reviews/` — Reportes de auditoría y review

**Propietario del pipeline:** `dev-tools/audit-all.ps1` (`$ReportDir`), prompt `audit-full.md` (escribe `audit-<mode>-<timestamp>.md`), skill `unified-review` (`/review`, reportes `review-<mode>-<timestamp>.md`). Unificado: `/audit` ≡ `/review` (alias legacy) escriben en `docs/dev/reviews/`. `audit-full-2025-07-27.md` (antes `vantadb-audit-report.md`) es el reporte estático multi-agente principal — archivado 2026-08-09 al estar 100% procesado (P13 AUDREP).

| Fichero | Líneas | Tipo |
|---|---|---|
| Notion › [«Reviews 2026»](https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd) | 45 | Reports del pipeline migrados a Notion (uno por review, con el análisis de vigencia a 2026-10-22) |
| `2026-10-20-auditoria-profunda-v0.9.0.md` | 407 | **Auditoría exhaustiva del árbol v0.9.0** (informe consolidado: veredicto por área, P0/P1 con fix, cobertura) |
| `2026-10-20-auditoria-profunda-v0.9.0-ledger.md` | 165 | **Ledger de los 141 hallazgos** de esa campaña (id, severidad, área, ubicación) |
| `2026-10-20-release-v0.9.0-postmortem.md` | 302 | **Post-mortem del release v0.9.0** (7 fallos, causa raíz, checklist de pre-flight) |
| `audit-deep-w0..w10-*.md` + `audit-deep-sintesis-*.md` | 13 | Campaña 11 waves: informe por wave + síntesis global |
| `audit-subagent-s1..s7-*.md` | 7 | Informes crudos de los 7 subagentes (evidencia por área) |
| `INDEX.md` | — | Índice de esta carpeta: qué informe es cuál |
| Notion › [«Reviews 2026»](https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd) | 45 | Los reports archivados (incluido `review.md` y `audit-full-2025-07-27.md`) ya no viven en el repo: se conservan en Notion |

## 4. `docs/dev/research/` — Investigaciones

**Ya catalogada 1:1 en `docs/dev/avance/investigaciones.md`** (incluye todos los INV-003..020, DESKTOP-01/01b, vectara, meta-001, ACID). No duplicar aquí el índice — ver `investigaciones.md`.