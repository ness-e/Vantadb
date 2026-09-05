# ADMIN-06: SOP operational panels (WAL replay, reindex, health) — desktop UI

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/components/SopPanel.tsx` — NUEVO: 3 paneles accionables (WAL Replay, Reindex, Health Check).
- `desktop/src/App.tsx` — montaje `<SopPanel />` entre KpiCards y el grid (no rompe MetricsGrid/KpiCards).
- `desktop/src/App.css` — estilos `.sop*` con tokens del design system (cream/ink/neon, border 2px, sombra dura 3px).
- `desktop/src/vanta.ts` — 4 campos nuevos en `OperationalMetrics` (startup_ms, ann_rebuild_ms, derived_rebuild_ms, text_index_rebuild_ms). Sin cambios de comportamiento: Rust serializa todos los u64 siempre; los consumidores existentes (KpiCards/MetricsGrid) no los leen.

## Contrato
"`npm run build` en desktop/ pasa (tsc + vite)."

## Pasos
### Step 1: Investigar comandos disponibles — ✅
- `vanta_health` existe (`commands/connection.rs:49`), `vanta_metrics` existe (`commands/metrics.rs`).
- **NO existen triggers de replay/reindex** en el core expuestos vía Tauri. Confirmado en `VantaOperationalMetrics` (src/sdk/types.rs:314): solo expone contadores/duraciones, no métodos de trigger. Los paneles Replay/Reindex muestran el ÚLTIMO valor del snapshot + botón Refresh (re-poll `vanta_metrics`). Sin inventar commands Rust nuevos (scope creep evitado).
- Core `VantaOperationalMetrics` tiene los campos de rebuild (`ann_rebuild_ms`, `derived_rebuild_ms`, `text_index_rebuild_ms`, `startup_ms`) pero el bridge TS solo declaraba un subconjunto → se agregaron 4 campos a la interfaz.

### Step 2: Implementar SopPanel.tsx — ✅
- `SopCard` (sub-componente: título, descripción, botón, resultado ok/err/idle) + default export `SopPanel` (3 paneles).
- Health: llama `health()` (vanta_health) en vivo en mount + botón "Run check" → estado ok/err + backend + latency + message.
- WAL Replay: muestra `wal_records_replayed` + `wal_replay_ms` del snapshot. Botón "Refresh" re-polla.
- Reindex: muestra `ann_rebuild_ms` + `derived_rebuild_ms` + `text_index_rebuild_ms`. Botón "Refresh" re-polla.
- Resultado: `data-status` ok/err/idle → color ink/neon/muted. Fallo de polling → err con `vantaErrorMessage`.
- Comentario ponytail: cuando el core exponga triggers, reemplazar las acciones Refresh por triggers reales.

### Step 3: Montar + estilos — ✅
- App.tsx: import + `<SopPanel />` después de `<KpiCards />`.
- App.css: `.sop`, `.sop-grid` (3 col → 1 col ≤900px), `.sop-panel`, `.sop-desc`, `.sop-foot`, `.sop-result` — mismos tokens que `.results li` (paper/cream, 2px ink border, sombra 3px).

### Step 4: Verify — ✅
- `npm run build` en desktop/ → `tsc && vite build` ✅ (2 runs, determinístico). 41 modules, dist OK.
- Error intermedio propio: duplicado de nombre `SopPanel` (sub-componente vs default export) → renombrado a `SopCard`. Corregido.
- Nota: `DataExplorer.tsx` (untracked, línea de otro trabajo) dio error TS18047 `r.score` en el 1er build; desapareció en builds posteriores sin tocarlo (probablemente modificado concurrentemente). No es de ADMIN-06.

### Step 5: Commit — ✅
- `git add` SOLO: SopPanel.tsx, App.tsx, App.css, vanta.ts. Working tree tenía ~40 archivos pre-existentes de otras líneas — NO tocados.
- Pre-commit hook: ✅ (no staged Rust; actionlint ok). Commit: `f20d67a4`.

## Dependencias
- `vanta_metrics` (ADMIN-01) y `vanta_health` (DESK-03) ya existentes — 0 cambios Rust.

## Notas
- **Scope creep evitado:** NO se crearon commands Tauri de trigger (p.ej. `vanta_reindex`). Trigger faltante documentado: el core (`VantaEmbedded`) no expone método público para disparar WAL replay o rebuild de índices; solo lectura de métricas. Cuando exista, agregar command + cambiar la acción del panel correspondiente.
- Diseño coherente con ADMIN-04/05 (tokens idénticos; paneles anidados tipo `.results li`).

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** f20d67a4f0a44977b1e9e3174d1ec6e3de7d58c4 — `feat(ADMIN-06): SOP operational panels (WAL replay, reindex, health)`
- **CI pendiente:** no (npm run build local pasado, determinístico x2)
- **Decisiones:** Replay/Reindex muestran último valor del snapshot (sin trigger en core) en lugar de inventar commands Rust (scope creep). Health sí llama vanta_health en vivo. Sub-componente `SopCard` para no colisionar nombre con el export default.
- **Problemas conocidos:** `desktop/src/components/DataExplorer.tsx` (untracked, otra línea de trabajo) tuvo error TS flaky en el primer build; desapareció sin intervención. No bloquea ADMIN-06.
- **Próxima tarea:** — (task única ejecutada)
