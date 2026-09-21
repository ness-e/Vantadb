# ADMIN-05: KPIs derivados — tarjetas KPI con sparklines CSS

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/components/KpiCards.tsx` — NUEVO: poll 5s del snapshot `vanta_metrics`, ring buffer de 12 snapshots, 5 KPIs derivados (guard div-by-zero), sparklines de barras CSS normalizadas por ventana. Sin dependencias.
- `desktop/src/vanta.ts` — consolidado el bridge de métricas: interfaz `OperationalMetrics` única (incluye los campos del snapshot ADMIN-01/04/05) + `metrics()` único. **Fix de compilación**: el working tree tenía DOS interfaces de métricas duplicadas (`OperationalMetrics` 17 campos y `MetricsSnapshot` 6 campos) y DOS funciones `metrics()` → error TS2393 duplicate implementation. Se consolidó a una sola.
- `desktop/src/App.tsx` + `desktop/src/App.css` — `<KpiCards />` renderizado + estilos `.kpi-grid`/`.kpi-card`/`.sparkline` (tokens cream/ink/neon, borde 3px ink, sombra dura 6px — mismos tokens del design system ADMIN-03).

## Contrato
"`npm run build` en `desktop/` ✅; panel de KPIs con tarjetas y sparkline."

## Pasos
### Step 1: Leer fuentes — ✅
- `App.tsx` (55 líneas original), `App.css` (tokens: --cream/--ink/--neon/--paper, `.panel` con borde 3px + sombra 6px), `vanta.ts` (bridge typed), `metrics.rs` (command `vanta_metrics` devuelve `VantaOperationalMetrics`).
- Campos del snapshot en `src/sdk/types.rs:314` — `VantaOperationalMetrics` (~40 campos u64 + `mmap_resident_bytes: Option<u64>`).

### Step 2: Zero-code planning — ✅
- Extender bridge con campos faltantes; componente nuevo KpiCards con poll autónomo + fórmulas guardadas; CSS append en App.css.

### Step 3: Implementar — ✅
- **KpiCards.tsx**: 5 KPIs derivados, todos con guard `ratio(num, den) = den > 0 ? num/den : 0`:
  1. Memory efficiency: `mmap_resident_bytes ?? 0 / process_rss_bytes` (pct)
  2. Hybrid query share: `planner_hybrid_queries / (hybrid + text_only)` (pct)
  3. Import error rate: `import_errors / records_imported` (pct)
  4. WAL efficiency: `wal_records_replayed / wal_replay_ms` (rec/ms)
  5. HNSW bytes/node: `hnsw_logical_bytes / hnsw_nodes_count` (bytes)
  - Sparkline: div con `<span class="spark-bar">` por punto, altura normalizada a max de la ventana (min 4px stub), eje base con `border-bottom: 2px ink`. Sin librerías.
  - Poll `POLL_MS=5000`, `WINDOW=12`, cleanup en unmount (`alive` flag + clearInterval).
  - Estado vacío: panel "Waiting for metrics…" o error del bridge (reusa `vantaErrorMessage`).
- **vanta.ts**: consolidación de las 2 interfaces duplicadas en `OperationalMetrics` (17 campos previos + `mmap_resident_bytes: number | null`, `hnsw_logical_bytes`, `hnsw_nodes_count`). `metrics()` único.
- **App.tsx**: `import KpiCards` + `<KpiCards />` tras el notice.
- **App.css**: `.kpi-grid` (auto-fit minmax 150px), `.kpi-card` (mismo lenguaje visual que `.panel`), `.kpi-value`/`.kpi-label`, `.sparkline`/`.spark-bar`.

### Step 4: Verify — ✅
- `npm run build` (workdir desktop): `tsc && vite build` → **built in 1.49s**, 39 modules, sin errores TS.

### Step 5: Commit — ✅
- Commit `4dcf268e` — `feat(ADMIN-05): derived KPI cards with CSS sparklines` — 2 archivos, 144 insertions: `desktop/src/components/KpiCards.tsx` (nuevo, 113) + `desktop/src/vanta.ts` (31).
- Pre-commit hook: sin archivos Rust staged → skip cargo; actionlint ok.

## Dependencias
- ADMIN-01 (`vanta_metrics` command) — commit d77559f3.
- ADMIN-03 (design system light mode) — commit 847ab080.
- ADMIN-04 (MetricsGrid) — commit b62fff7c.

## Notas
- **Colisión con línea paralela ADMIN-04/02:** el working tree fue mutado concurrentemente durante la sesión (aparecieron `MetricsGrid.tsx`, `MetricsPanel.tsx`, `useMetrics.ts` y ediciones a App.tsx/vanta.ts de otra sesión). Resolución:
  - Mi primer `npm run build` falló por 2 errores TS: (a) `App.tsx` renderizaba `<MetricsGrid>` sin import (WIP de ADMIN-04), (b) mi extensión de `OperationalMetrics` rompió el literal `ZERO` en `MetricsGrid.tsx` (faltaban los 3 campos nuevos).
  - La sesión ADMIN-04 commiteó `b62fff7c` **antes** de que yo commitease, barriendo mis ediciones de App.tsx/App.css junto con su MetricsGrid. Esos archivos quedaron en su commit, no en el mío.
  - Mi commit final (`4dcf268e`) contiene SOLO mis 2 archivos pendientes (`vanta.ts` + `KpiCards.tsx`) — los que completan el árbol (HEAD importaba `KpiCards` y `OperationalMetrics` sin tenerlos). App.tsx/App.css ya estaban en HEAD vía b62fff7c.
  - `MetricsPanel.tsx`/`useMetrics.ts` (WIP ADMIN-02) desaparecieron del disco durante la sesión — no eran míos, no los toqué.
- **Decisión:** la interfaz `OperationalMetrics` es compartida (la usan KpiCards, MetricsGrid, useMetrics). Consolidar era obligatorio — el estado pre-existente no compilaba (dos `metrics()` idénticos = TS2393).
- **Nota de diseño:** los KPIs con denominador acumulado (memory efficiency, import error rate) tienden a 0 con pocos datos; el sparkline normaliza por ventana para mostrar tendencia, el valor absoluto va en la etiqueta. Si se quiere ratio vs pico histórico, cambiar la normalización en `Sparkline` a un máximo fijo.

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** 4dcf268eb014e3974683ab835b72343d124c4999
- **CI pendiente:** no (npm run build local ✅; sin Rust en el commit)
- **Decisiones:** Consolidar las 2 interfaces duplicadas de métricas en `OperationalMetrics` única (era la única forma de compilar). KpiCards con poll autónomo 5s + ring buffer 12 (ADMIN-02/04 aún no commitean un hook compartido; cuando exista `useMetrics`, KpiCards puede consumirlo). Sparklines con CSS puro (barras) sin librerías, normalización por ventana.
- **Problemas conocidos:** ejecución paralela de sesiones sobre los mismos archivos compartidos (App.tsx, vanta.ts, App.css) — mi commit NO incluye los cambios de ADMIN-04 (commit separado b62fff7c) ni el WIP de ADMIN-02. Si el hook `useMetrics` aterriza después, migrar el poll de KpiCards a él. El estado del plan file (docs/plans/2026-08-08-admin-console.md) lo actualiza la sesión orquestadora.
- **Próxima tarea:** ADMIN-06 (SOP panels) / ADMIN-02 (hook de métricas compartido).
