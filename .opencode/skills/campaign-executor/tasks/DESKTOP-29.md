# DESKTOP-29: Coordinar polling de métricas — 1 hook useMetricsPoll compartido

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** `desktop/src/components/MetricsGrid.tsx`, `KpiCards.tsx`, `ExportPanel.tsx`, `components/indices/IndicesLens.tsx`, `src/vanta.ts` (`metrics()`:496, `OperationalMetrics`:469, `vantaErrorMessage`:191), `vitest.config.ts`, `package.json`
- **Referencias entrantes:** los 4 componentes los montan shells existentes (WorkspaceShell/home) — no cambian sus Props ni exports default
- **Referencias salientes:** `vanta.metrics()`, `OperationalMetrics`, `vantaErrorMessage` — sin cambios
- **Pre-existentes sin commit (DESKTOP-23/24/26/28):** Tailwind styling en los 4 componentes se preserva; solo se reemplaza la lógica de polling
- **Veredicto:** refactor interno de `desktop/src`; sin API pública afectada; riesgo bajo

## Blast Radius
Callers: MetricsGrid (4s), KpiCards (5s), IndicesLens (4s), ExportPanel (oneshot)
Callees: desktop/src/hooks/useMetrics.ts, desktop/src/vanta.ts (vanta_metrics)
Implicaciones: Un solo poller activo para `vanta_metrics`; deltas/trend intactos; reduce llamadas redundantes

## Spec
N/A — refactor polling con contrato mecánico

## Contrato
`cd desktop && npm run build`; 1 solo `setInterval` activo para métricas; deltas/trend funcionan en MetricsGrid, KpiCards, IndicesLens

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear hook useMetricsPoll compartido — ✅ DONE
- **Archivos:** `desktop/src/hooks/useMetricsPoll.ts` (nuevo), `useMetricsPoll.test.tsx` (nuevo)
- **Acción:** Store module-level + `useSyncExternalStore` (React nativo, sin deps). 1 `setInterval` 4s que arranca con el primer subscriber y se detiene con el último; `history` newest-last capped a 12, `error`, `polledAt`; guard `inFlight` para no pisar ticks lentos; cleanup `clearInterval` al desuscribir.
- **Verify:** `cd desktop && npm run build` ✅

### Step 2: Migrar MetricsGrid, KpiCards, IndicesLens al hook compartido — ✅ DONE
- **Archivos:** `MetricsGrid.tsx`, `KpiCards.tsx`, `components/indices/IndicesLens.tsx`
- **Acción:** Reemplazados los `useEffect`+`setInterval` propios por `useMetricsPoll()`. MetricsGrid: delta/trend sobre `poll.history` (últimos 3). KpiCards: la history compartida (cap 12) es la serie del sparkline. IndicesLens: `snapshot = last(history)`; su poll de namespaceStats (comando distinto) queda intacto. Styling Tailwind de DESKTOP-28 preservado.
- **Verify:** `npm run build` ✅; deltas/trend lógica intacta (`deltaAndTrend` sin cambios semánticos)

### Step 3: Adaptar ExportPanel (oneshot) al hook — ✅ DONE
- **Acción:** Lee el último snapshot del poller compartido (no crea poller propio); persiste a localStorage en cada snapshot nuevo vía `useEffect [live]`.
- **Verify:** build ✅; export usa blob download intacto

### Step 4: Verificar 1 solo poller activo — ✅ DONE
- **Acción:** Test automatizado `useMetricsPoll.test.tsx`: (1) 3 consumidores → 1 call por tick, (2) interval se detiene tras el último unmount, (3) history cap 12. Verificación manual DevTools/Network queda para QA visual (requiere app Tauri corriendo).
- **Verify:** `cd desktop && npm test` ✅ — 9 files / 48 tests passed

## Context Save Point
- Sin commit (instrucción explícita): los cambios viven en worktree junto a DESKTOP-23/24/26/28.
- Nota: primer `npm run build` reportó TS6133 falso en `useConnectionState.ts:7` (import sí usado en :142) — desapareció en el segundo run (caché tsc); ese archivo NO fue modificado.

## Dependencias
- ADMIN-01/02/04 (vanta_metrics command + deltas) — ya completadas

## Notas
- DoD: 1 solo poller activo para `vanta_metrics`; deltas/trend intactos
- Hook debe manejar cleanup correcto (clearInterval en unmount)
- Evitar memory leaks con useRef para interval ID