# DESKTOP-34: Polish UX global — CommandPalette 9 superficies, hints F1/F2, unificar ES/EN

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETED (2026-08-24)

## Blast Radius
Callers: desktop/src/components/palette/CommandPalette.tsx, desktop/src/components/layout/WorkspaceShell.tsx, componentes varios
Callees: cmdk, desktop/src/lib/i18n.ts (nuevo o simple map)
Implicaciones: Ctrl+K alcanza 9 superficies; tooltips F1/F2 en sidebar; todo label visible en ES consistente

## Spec
N/A — polish UX con contrato mecánico

## Contrato
`cd desktop && npm run build`; Ctrl+K alcanza 9 superficies; todo label visible en ES consistente; hints F1/F2 con tooltip

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Extender CommandPalette a 9 superficies — ✅ DONE
- **Archivos:** `desktop/src/components/palette/CommandPalette.tsx`
- **Acción:** Añadidas nav entries para PAPELERA, BÚSQUEDA (retrieval), CONSOLIDAR, ESPACIO; "ACTIVITY" → "ACTIVIDAD". PaletteSurface extendido al union completo de Surface. Nota: DESKTOP-31 (AJUSTES) se implementó en paralelo y añadió la 10ª superficie — fusionado sin conflicto.
- **Verify:** ✅ `npm run build` verde (incluye tsc); palette navega a las 10 superficies del shell.

### Step 2: Hints F1/F2 en Sidebar con tooltip — ✅ DONE
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx`
- **Acción:** `SideButton` ahora acepta `title` → tooltip nativo + aria-label en los 9 botones del sidebar. Corrección sobre el spec original: F1/F2 NO son atajos alternar-sidebar/palette (eso es Ctrl+K) sino fases documentadas en HelpPanel ("F1: ÍNDICES/ACTIVIDAD", "F2: Consola IQL") — los tooltips describen lo que cada botón realmente hace.
- **Verify:** ✅ build verde; title/aria-label presentes.

### Step 3: Unificar ES/EN → ES consistente (sin i18n framework) — ✅ DONE
- **Archivos:** IngestForm, ExportPanel, MetricsGrid, ConnectionPanel, DataExplorer, Timeline, KpiCards, SopPanel, HomeOverview, App.tsx
- **Acción:** Traducidos labels visibles + aria-labels + placeholders EN→ES (sin framework i18n). Los componentes que mencionaba el spec (site-navbar, easter-egg, tutorial-modal, back-to-top) no existen en desktop — eran del prototipo web.
- **Verify:** ✅ build verde; grep sin labels EN residuales en UI (queda statusReport.ts, ver deuda).

## Dependencias
- DESKTOP-09 (CommandPalette base) — ya completada
- DESKTOP-08 (undo + papelera) — ya completada
- DESKTOP-28 (design system unificado) — complementaria

## Notas
- DoD: Ctrl+K alcanza las 9 superficies; todo label visible en ES consistente
- Sin framework i18n (YAGNI) — simple replace hardcoded
- Idioma default del proyecto: ES