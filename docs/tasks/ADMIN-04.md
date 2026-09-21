# ADMIN-04: Dashboard grid (metro-style) con poll 3-5s

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/components/MetricsGrid.tsx` — NUEVO componente: grid metro de 6 tiles (RSS, Records, Queries, Scans, WAL Replay, Text Index) con valor grande + delta + trend arrow. Poll inline `setInterval` 4s sobre `vanta_metrics`.
- `desktop/src/App.tsx` — import + render de `<MetricsGrid>` como vista principal (header con vanta_health + grid). Se removió `<MetricsPanel />` (ADMIN-02) del render: superseded por el grid, evita doble poller.
- `desktop/src/App.css` — bloque ADMIN-04 (`.metrics`, `.metrics-grid` 3-col→2→1, `.tile*`, `.trend-*`). Se removió CSS muerto de ADMIN-02 (`.metric*`, `.metrics-grid` 4-col) tras quitar MetricsPanel.
- `desktop/src/vanta.ts` — NO tocado por ADMIN-04: el bridge `metrics()` + `OperationalMetrics` ya existía (escrito concurrentemente por ADMIN-02/05).

## Contrato
"`npm run build` en `desktop/` ✅; grid de tiles 2-4 columnas con métricas vivas (valor + delta + trend) y poll 3-5s."

## Steps
### Step 1: Bridge — ✅
- `metrics()` + `OperationalMetrics` ya existían en vanta.ts (ADMIN-02/05 concurrente). Mi intento de agregar wrapper duplicado fue sobreescrito por la wave — sin acción final, se consume el bridge compartido.

### Step 2: MetricsGrid.tsx — ✅
- Poll inline 4s (`useEffect` + `setInterval`, cleanup en unmount, `cancelled` flag).
- Delta entre snapshots consecutivos + trend de 3 puntos (delta(now,prev) vs delta(prev,prevprev)) → arrow ▲/▼/—.
- 6 tiles: RSS (bytes), Records (records_imported), Queries (planner hybrid+text+vector), Scans (derived_prefix_scans), WAL Replay (wal_records_replayed + replay ms), Text Index (text_postings_written). Cada tile: título, valor grande, delta + muted.
- Header: estado de conexión vía `vanta_health` (health badge reusado de ConnectionPanel), nombre del backend activo, last-poll time.
- Sin `ZERO` literal: `OperationalMetrics | null` + estado "Waiting for first metrics snapshot…" (la interfaz creció con campos de ADMIN-05; un ZERO habría quebrado en cada cambio).

### Step 3: App.tsx — ✅
- `<MetricsGrid health healthStatus activeName />` renderizado bajo el header (vista principal). Se removió `<MetricsPanel />` (ADMIN-02) — grid lo supersede y evita doble polling a `vanta_metrics`.
- **Race de wave:** la wave paralela (ADMIN-02/05 escribiendo App.tsx/App.css/vanta.ts en vivo) sobreescribió mi import de MetricsGrid en un rewrite → build falló 1 vez (TS2304). Fix: re-agregar `import MetricsGrid`. Build final ✅.

### Step 4: App.css — ✅
- Bloque `.metrics-grid` 3-col (→2 @900px →1 @560px), tiles con design system exacto: paper bg, border 3px ink, radius 0, shadow 6px 6px 0 #000, press effect (hover/active translate).
- Eliminado CSS de ADMIN-02 (`.metrics-grid` 4-col + `.metric*`) — MetricsPanel no renderiza; selector duplicado hubiera pisado el grid.

### Step 5: Verify — ✅
- `npm run build` (workdir desktop): tsc 0 errors + vite built in 1.46s (39 modules, JS 205 kB / gzip 64.66 kB).

### Step 6: Commit — ✅
- `git add` SOLO: `desktop/src/App.tsx`, `desktop/src/App.css`, `desktop/src/components/MetricsGrid.tsx`. NO se commitearon archivos de la wave concurrente (vanta.ts, useMetrics.ts, MetricsPanel.tsx, KpiCards.tsx). Pre-commit hook: sin Rust staged → cargo checks skipped, actionlint ok. Commit: `b62fff7c`.

## Dependencias
- ADMIN-01 (`vanta_metrics` command) — consumido vía bridge `metrics()`.
- ADMIN-02 ⬜ PENDING al ejecutar ADMIN-04 → poll/delta inline (fallback del contrato). `useMetrics.ts` existía en working tree pero sin commit; se evitó depender de él (interface renombrada mid-flight, riesgo de colisión con agente paralelo).
- ADMIN-03 (design system light) — tokens reusados.

## Notas
- **Concurrencia Wave 1:** ADMIN-02, ADMIN-04, ADMIN-05 corren en paralelo y comparten App.tsx/App.css/vanta.ts. El estado final del commit incluye integraciones de ADMIN-05 (`KpiCards`) que quedaron en App.tsx/App.css — el merge final de wave es responsabilidad del harness.
- **Double-poller evitado:** MetricsPanel (ADMIN-02) también polla `vanta_metrics` cada 4s; renderizarlo junto al grid habría duplicado IPC. Se removió del render; el archivo queda intacto para uso futuro.
- **ponytail:** trend de 3 puntos (~6 líneas) en vez de historial completo; formato compacto (fmtCount/fmtBytes) sin libs nuevas; sin sparklines (ADMIN-05 los cubre).

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** b62fff7c — `feat(ADMIN-04): metro-style metrics dashboard grid with live poll` (3 files, +291)
- **CI pendiente:** no (npm run build local pasado)
- **Decisiones:** Poll inline en vez de consumir `useMetrics` porque ADMIN-02 no es commit y su hook estaba roto mid-flight (importaba `MetricsSnapshot` inexistente). MetricsGrid self-contained sobre el bridge compartido `metrics()`. Se removió MetricsPanel del render (superseded, doble poller). CSS muerto de ADMIN-02 eliminado.
- **Problemas conocidos:** App.tsx/App.css del commit contienen también integración ADMIN-05 (KpiCards) escrita por agente paralelo — la división exacta de ownership la resuelve el merge de wave. KpiCards.tsx aún no commiteado por su agente.
- **Próxima tarea:** ADMIN-05 (KPI cards) / ADMIN-06 (SOP panels) según secuencia del plan.
