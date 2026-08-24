# DESKTOP-28: Unificar paneles legacy al design system Studio + limpieza

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ⏳ IN PROGRESS (steps 1-4 ✅ — pendiente cierre/commit por lead)

## Blast Radius
Callers: desktop/src/components/*.tsx, desktop/src/App.css, desktop/src/index.css
Callees: Tailwind manga/linocut tokens (index.css)
Implicaciones: Migración completa de estética mixta legacy/Studio a design system unificado

## Spec
N/A — refactor UI con contrato mecánico

## Impacto mapeado (Regla 0)
- **Leídos completos:** los 7 componentes, App.css (383L), index.css (803L), App.tsx, WorkspaceShell.tsx (§690-789).
- **Referencias entrantes:** los 7 componentes se montan SOLO en `WorkspaceShell.tsx` (720-751 resumen, 773-782 memorias). Sin imports en tests (`*.test.*` grep = 0 hits).
- **Huérfanos verificados (grep `SearchBar|ProcessPanel` en desktop/src):** cero imports — solo definición propia + comentarios en WorkspaceShell.tsx:11,204. Seguro borrar.
- **Hacia afuera:** componentes consumen bridge `../vanta` (sin cambios). Clases App.css que consumían: `.panel/.panel-head/.row/.stack/.muted/.tag/.ghost/.health-badge/.conn-*/.dot/.kpi*/.spark*/.metrics*/.tile*/.trend-*/.sop*/.export*/.polled(no existe)`.
- **Implicación token:** App.css `:root` pisa `--muted:#3A3A3A` sobre el token del DS (`#ECE6D8` light / `#1A1A1A` dark). Al limpiarlo, `hover:bg-muted` (Timeline/ActivityPanel/ConsolidateLens) vuelve al valor correcto del DS. `text-muted` pelado solo existe en DataExplorer (794, 881) → migrado a `text-muted-foreground` (AA).
- **Step 4 scope:** estilos globales de elementos (`input/textarea/button`, `:root` font stack, `body`) SE CONSERVAN — ~25 inputs/buttons pelados en ImportPaste/Inspector/RetrievalLens/WorkspaceShell dependen de ellos. Solo se borran clases huérfanas post-migración.
- **Veredicto:** refactor seguro, blast radius contenido a desktop/src.

## Contrato
`cd desktop && npm run build` pasa; 0 componentes huérfanos; SopPanel sin acciones muertas; RESUMEN sin estética mixta

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: ✅ Migrar componentes legacy a Tailwind manga/linocut
- **Hecho:** los 7 componentes migrados + `ResultsList.tsx` (consumidor de `.muted/.tag/.results/.row-between/.score` detectado en sweep). Tokens: `border-foreground/bg-card/text-muted-foreground/font-tech/press/shadow-[Npx_Npx_0_0_#000]` + `dark:shadow-...#FBF9F5`. Health-badge conserva estados idle/ok/warn|err. `text-muted` pelado (DataExplorer 794,881) → `text-muted-foreground`.
- **Verify:** `npm run build` ✅

### Step 2: ✅ Eliminar componentes huérfanos nunca montados
- **Hecho:** grep previo = cero imports (solo autodefs + comentarios en WorkspaceShell:11,204). Borrados `SearchBar.tsx`, `ProcessPanel.tsx`.
- **Verify:** `npm run build` sin errores de import ✅

### Step 3: ✅ SopPanel — botones falsos → solo lectura
- **Hecho:** WAL Replay y Reindex sin botón; tag "solo lectura" con tooltip ADMIN-06; solo leen última snapshot de metrics. Health Check conserva su acción real (`vanta_health`). Eliminado `metricsBusy` del action path.
- **Verify:** `npm run build` ✅

### Step 4: ✅ Limpiar App.css legacy
- **Hecho:** 383L → 41L. Borradas todas las clases de componentes migrados. CONSERVADOS los estilos globales de elementos (`input/textarea/button` base+press+focus, `:root` font stack, `body`) — ~25 inputs/buttons pelados en ImportPaste/ImportDrop/Inspector/RetrievalLens dependen de ellos. Eliminados overrides de palette `:root` (restaura tokens DS de index.css: `--muted` vuelve a #ECE6D8/#1A1A1A — afecta `hover:bg-muted`/`bg-muted` en Timeline/ActivityPanel/ConsolidateLens/HomeOverview/WorkspaceShell: valor correcto del DS).
- **Verify:** `npm run build` ✅ · sweep final: 0 usos residuales de clases borradas

## Dependencias
- DESKTOP-01 (Tailwind v4 + tokens) — ya completada (F0)

## Notas
- DoD: RESUMEN sin estética mixta; 0 componentes huérfanos; SopPanel sin acciones muertas
- Tokens en `desktop/src/index.css` replican `@theme inline` + `:root` COMPLETOS de `web/src/app/globals.css`
- `ink-corner` NO existe en la web → no replicar