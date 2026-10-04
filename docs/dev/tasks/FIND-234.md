---
title: "TASK FIND-234: check-avance-coverage.ps1 apunta a docs/avance (inexistente) — reporte 0/237 engañoso"
kind: task
description: "Fix de path stale tras la migración 2026-08-23 — el script ahora lee docs/dev/avance (reporte real 1034/1034)"
---

# TASK FIND-234: check-avance-coverage.ps1 apunta a docs/avance (inexistente)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 5, F0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-234; origen PROC-04)
- **Esfuerzo:** 🟢 1h | **Appetite:** max 1h | **Prioridad:** 🟢
- **Tipo:** tooling/task-system (script PowerShell)
- **Creado:** 2026-10-04T06:40Z | **last-synced:** 2026-10-04T06:45Z
- **Estado:** ⏳ IN PROGRESS (review P2-01 pendiente)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | skill `progreso` (lo referencia como check de cierre, L143/L209) + cierres de campaña del orquestador |
| Callees | `docs/dev/avance/**` (lectura), `docs/dev/avance/historial/fuentes/**` (lectura) |
| Implicaciones | Solo lectura + reporte; ningún consumidor automatizado parsea su salida (advisory). El fix corrige la lectura falsa de cobertura |

## Impacto mapeado (Regla 0)

- **Archivo leído (completo):** `scripts/check-avance-coverage.ps1` (89 líneas).
- **Referencias entrantes:** skill `progreso` (check de cierre); ningún workflow lo invoca.
- **Veredicto impacto:** NULO fuera del script — única línea stale (L10); L15-17 ya usaban `docs/dev/avance`.
- **Verificado:** `docs/dev/avance/historial/campanas` EXISTE (el guard adicional no es necesario); `docs/avance` NO existe (path roto confirmado).

## Contrato

1. `pwsh scripts/check-avance-coverage.ps1` → sin errores de ruta + reporte real (≠ `0/237`): ✅ **1034/1034 (100.0%)**
2. exit 0: ✅

## Steps

- [x] **Step 1 — Repro "antes":** `FINAL: 0/237 (0.0%)` con el path stale `docs/avance` — ✅ (exit 0, reporte engañoso)
- [x] **Step 2 — Fix:** L10 `"docs/avance"` → `"docs/dev/avance"` — ✅ (1 línea; el resto del script ya era correcto)
- [x] **Step 3 — Verify "después":** `FINAL: 1034/1034 IDs cubiertos en dominio (100.0%)`, sin errores, exit 0 — ✅
- [ ] **Step 4 — Review P2-01 (vanta-review fresco)** — ⏳ en curso

## Evidencia before/after

| Métrica | Antes | Después |
|---------|-------|---------|
| IDs detectados/cubiertos | 0/237 (0.0%) | **1034/1034 (100.0%)** |
| Errores de ruta | sí (`docs/avance` inexistente) | ninguno |
| Exit code | 0 | 0 |

## Review (P2-01)

| Campo | Valor |
|-------|-------|
| Reviewer | (pendiente — vanta-review fresco) |
| reviewer_context | (pendiente) |
| Verdict | (pendiente) |

## DoD (3 niveles)

- **task:** contrato ✅ (before/after documentados)
- **commit:** `fix(scripts):` + verify · release: n/a

## Deuda técnica (Regla 6)

Ninguna introducida. Nota: el salto 237→1034 IDs confirma que el árbol canónico (`docs/dev/avance`) es la fuente real de cobertura.
