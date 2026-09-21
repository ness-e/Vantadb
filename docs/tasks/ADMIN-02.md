# ADMIN-02: Métricas vivas — deltas entre snapshots de `vanta_metrics` en desktop UI

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED (vía colisión resuelta — ver Notas)

## Blast Radius
- `desktop/src/App.tsx` — render del panel de métricas (MOVIDO por ADMIN-04)
- `desktop/src/App.css` — estilos de tiles métricas (MOVIDO por ADMIN-04)
- `desktop/src/vanta.ts` — tipo `OperationalMetrics` + wrapper `metrics()` (MOVIDO por ADMIN-05)
- `desktop/src/components/MetricsGrid.tsx` — grid metro-style (ADMIN-04)
- `desktop/src/components/KpiCards.tsx` — KPI cards con sparklines (ADMIN-05)
- `desktop/src/hooks/useMetrics.ts` — MI hook (creado y ELIMINADO, duplicado)
- `desktop/src/components/MetricsPanel.tsx` — MI panel (creado y ELIMINADO, duplicado)

## Contrato
"`npm run build` en desktop/ pasa; el frontend muestra deltas de contadores (imports, queries, scans), RSS actual, y poll interval."

## Pasos
### Step 1: Leer archivos — ✅
- `vanta.ts`, `useConnectionState.ts`, `App.tsx`, `commands/metrics.rs`, `lib.rs`, `sdk/types.rs` (struct `VantaOperationalMetrics`).
- El comando `vanta_metrics` ya devuelve el snapshot completo (ADMIN-01, commit d77559f3). Cero cambios Rust (Opción A).

### Step 2: Zero-code planning — ✅
- Tipo `MetricsSnapshot` + wrapper `metrics()` en vanta.ts (patrón `health()`).
- Hook `useMetrics` con poll 4s, snapshot previo en ref, delta + rate = delta/elapsed real.
- Panel con 4 tarjetas (RSS, imports/s, queries/s, scan fallbacks) siguiendo design system.

### Step 3: Implementar — ⚠️ COLISIÓN PARALELA
- Implementé `MetricsSnapshot` en vanta.ts, `useMetrics.ts` y `MetricsPanel.tsx`.
- **DETECCIÓN:** otro agente ejecutaba ADMIN-04/05 EN PARALELO sobre los MISMOS archivos con el MISMO feature (metro grid `MetricsGrid` + `KpiCards`, ya montados en App.tsx). El grid de ADMIN-04 es un superset del contrato ADMIN-02 (deltas imports/queries/scans + RSS + poll 4s + trends + WAL + text index).
- El grid de ellos tenía 2 errores de build (import `MetricsGrid` faltante en App.tsx, literal `ZERO` sin los 3 campos nuevos de `OperationalMetrics`) — EL OTRO AGENTE los arregló él mismo.

### Step 4: Convergencia (ponytail + colaboración) — ✅
- **Decisión:** no duplicar UI. Adoptar el grid de ADMIN-04 como implementación canónica del contrato ADMIN-02.
- Eliminé mi `MetricsPanel.tsx` + `useMetrics.ts` (duplicados, neto cero — eran untracked).
- Removí mis estilos `.metrics-grid`/`.metric` de App.css (el grid de ellos ya tiene el suyo, 3-col) — el otro agente también los removió por su lado.
- Reusé el tipo `OperationalMetrics` + `metrics()` de vanta.ts (ya existía) en vez de mi `MetricsSnapshot`.

### Step 5: Verify — ✅
- `npm run build` (tsc && vite build) → ✅ 39 modules, dist generado en 1.50s.
- `MetricsGrid.tsx` + `KpiCards.tsx` + vanta.ts compilan sin errores TS.

### Step 6: Commit — ⏸️ NO EJECUTADO (ver Notas)
- El otro agente commiteó `b62fff7c feat(ADMIN-04): metro-style metrics dashboard grid with live poll` (App.tsx + App.css + MetricsGrid.tsx) — este commit ENTREGA el contrato ADMIN-02.
- Quedaron sin commitear (ADMIN-05, en vuelo): `vanta.ts` (métricas) + `KpiCards.tsx`.
- NO commiteé archivos de otras líneas por instrucción explícita del usuario.

## Dependencias
- ADMIN-01 (d77559f3): comando `vanta_metrics` — ya entregado.
- ADMIN-04 (b62fff7c): grid metro — ya entregado.
- ADMIN-05: vanta.ts metrics bridge + KpiCards — PENDIENTE commit.

## Notas
- **COLISIÓN PARALELA REAL:** dos agentes implementaron el mismo feature (métricas vivas) simultáneamente sobre los mismos archivos. El agente ADMIN-04 commiteó primero con una implementación superior. Mi implementación fue descartada como duplicado (correcto: menos código, cero regresiones).
- **HEAD quedo transitoriamente roto** tras b62fff7c (App.tsx importa KpiCards que no está en HEAD) hasta que ADMIN-05 commitee vanta.ts + KpiCards.tsx. El árbol de trabajo COMPLETO compila (verificado).
- **Commit de ADMIN-02 propio: NO existe** — todo mi código fue superseded. El commit que entrega el contrato es b62fff7c (ADMIN-04). Reportado con transparencia al orquestador.
- Lección para el pipeline: ADMIN-02 y ADMIN-04/05 se solapan por diseño — en el plan file deberían ser la misma tarea o ejecutarse en secuencia estricta.

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** N/A (sin commit propio; feature entregado vía b62fff7c — ADMIN-04)
- **CI pendiente:** no (build local verde sobre árbol completo)
- **Decisiones:** Adoptar el grid ADMIN-04 (superset) sobre mi panel duplicado; no commitear archivos de otras líneas; reusar `OperationalMetrics`/`metrics()` existentes en vez de tipos propios.
- **Problemas conocidos:** HEAD roto sin los archivos sin commitear de ADMIN-05 (vanta.ts + KpiCards.tsx); colisión activa en desktop/src/ — esperar a que ADMIN-05 aterrice antes de nuevos cambios en ese directorio.
- **Próxima tarea:** ADMIN-05 (commit de vanta.ts metrics + KpiCards) — no tocar.
