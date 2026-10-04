---
title: "TASK FIND-234: check-avance-coverage.ps1 apunta a docs/avance (inexistente) — reporte 0/237 engañoso"
kind: task
description: "Fix de path stale (ruptura real 2026-09-23, b764d703) — el script ahora lee docs/dev/avance (reporte real 1034/1034)"
---

# TASK FIND-234: check-avance-coverage.ps1 apunta a docs/avance (inexistente)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 5, F0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-234; origen PROC-04)
- **Esfuerzo:** 🟢 1h | **Appetite:** max 1h | **Prioridad:** 🟢
- **Tipo:** tooling/task-system (script PowerShell)
- **Creado:** 2026-10-04T06:40Z | **last-synced:** 2026-10-04T07:00Z
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE — `ses_efa538f07ffe7SP6Rfi9oy5FNu`)
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
- **Corrección de narrativa (review):** la ruptura real es **2026-09-23** (`b764d703` movió `docs/avance → docs/dev/avance`; `3d143cd9`/`fffbf5a6` corrigieron `$srcDir`/`$live` pero omitieron L10). Ventana rota: 09-23 → 10-04.

## Contrato

1. `pwsh scripts/check-avance-coverage.ps1` → sin errores de ruta + reporte real (≠ `0/237`): ✅ **1034/1034 (100.0%)**
2. exit 0: ✅

## Steps

- [x] **Step 1 — Repro "antes":** `FINAL: 0/237 (0.0%)` con el path stale `docs/avance` — ✅ (exit 0, reporte engañoso; reproducido por el reviewer desde el blob `HEAD~1`)
- [x] **Step 2 — Fix:** L10 `"docs/avance"` → `"docs/dev/avance"` — ✅ (1 línea; el resto del script ya era correcto)
- [x] **Step 3 — Verify "después":** `FINAL: 1034/1034 IDs cubiertos en dominio (100.0%)`, sin errores, exit 0 — ✅
- [x] **Step 4 — Review P2-01 (vanta-review fresco):** ✅ APPROVE

## Evidencia before/after

| Métrica | Antes | Después |
|---------|-------|---------|
| IDs detectados/cubiertos | 0/237 (0.0%) | **1034/1034 (100.0%)** |
| Errores de ruta | sí (`docs/avance` inexistente) | ninguno |
| Exit code | 0 | 0 |

**Plausibilidad validada por el reviewer:** conteo independiente `srcDir=237 · campanas=907 · union=1034 · overlap=110` (237+907−1034=110 ✓); matcher estricto también da 1034/1034; sin side-effects (solo lectura).

## Review (P2-01)

| Campo | Valor |
|-------|-------|
| Reviewer | vanta-review (contexto fresco) |
| reviewer_context | `ses_efa538f07ffe7SP6Rfi9oy5FNu` |
| Verdict | ✅ **APPROVE** — diff = solo L10; after re-ejecutado (hash worktree == blob `6a92bb4d`); before reproducido desde `HEAD~1`; adversarial sin side-effects ni rutas stale restantes |

## DoD (3 niveles)

- **task:** contrato ✅ (before/after documentados)
- **commit:** `35cbd2e1` — `fix(scripts):` + verify · release: n/a

## Deuda técnica (Regla 6)

Ninguna introducida. Notas del reviewer (opcionales, fuera de scope): (1) comentarios L3/L13 aún dicen `docs/avance` (solo texto — próximo touch); (2) hardening futuro si algún día se vuelve gate: `exit 1` cuando `idProcessed < idAll` (hoy advisory por diseño).
