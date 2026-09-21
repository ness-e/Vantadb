---
title: "Mapa de Cobertura — Migration docs/progreso → docs/avance"
type: verification
status: active
date: 2026-08-07
generated_by: scripts/check-avance-coverage.ps1
tags: [vantadb, avance, migracion, cobertura, verificacion]
---

# Mapa de Cobertura (fuente → destino)

> Generado por `scripts/check-avance-coverage.ps1` (2026-08-07). Re-ejecutable con `pwsh scripts/check-avance-coverage.ps1 [-Detail]` tras cada cambio en las fuentes.

## Fuente → Destino

| Fuente (`docs/progreso/`) | Destino (`docs/avance/`) | Estado |
|---|---|---|
| `README.md` | `historial/snapshot-2026-08-07.md` (copia directa) + `activo/*` desglosado por dominio | ✅ |
| `BACKLOG_HISTORY.md` | `historial/backlog-history.md` (copia directa) | ✅ |
| `ARCHIVO_HISTORICO.md` | `historial/archivo-historico.md` (copia directa) + refs en `activo/*`, `auditoria/*`, `decisiones/*`, `meta.md` | ✅ |
| `bitacora.md` | `historial/sesiones/2026-07.md` + `historial/sesiones/2026-07-consolidacion.md` (sección pendientes, copia) + `activo/*` | ✅ |
| `2026-07-28-sdk-gap-audit.md` | `historial/sdk-gap-audit-2026-07-28.md` (link → canónico `docs/progreso/2026-07-28-sdk-gap-audit.md`) | ✅ |

## Cobertura de IDs

| Métrica | Valor |
|---|---|
| IDs únicos detectados en fuentes | 821 |
| procesados en archivos de dominio | 428 (52.1%) |
| solo en snapshot (espejo literal — sin pérdida) | 393 |

Los 393 IDs restantes viven en los snapshots espejo (copia íntegra verbatim de los fuentes) — no hay pérdida de información; son IDs que aún no tienen entrada redactada en un archivo de dominio (típicamente ítems detalle de registros históricos menores).

## Convención

- El snapshot NUNCA se edita (es la garantía de "0 info perdida").
- Los archivos de dominio (`activo/`, `auditoria/`, `decisiones/`, `meta.md`) son la versión reorganizada/navegable; si un ID solo existe en snapshot y es relevante, crear su entrada de dominio es la vía correcta (no editar el snapshot).

## Migración física (2026-08-23)

docs/progreso/ fue **eliminado**: todo su contenido vive ahora bajo este árbol.
- campanas/*.md → historial/campanas/
- README/bitácora/ARCHIVO_HISTORICO/sdk-gap-audit → historial/fuentes/
- BACKLOG_HISTORY.md → historial/backlog-history.md (**archivo vivo**, único destino de removidos del Backlog)
- Las citas docs/progreso/... dentro de archivos de dominio son evidencia histórica congelada (convención GOV-D2): apuntan a lo que hoy es historial/fuentes/.
- Registro vivo de tareas completadas: ctivo/*, uditoria/*, decisiones/, investigaciones.md.
