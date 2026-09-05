# ADMIN-03: Migrar UI del desktop al design system web (modo claro)

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/App.css` — reemplazo completo del tema oscuro por tokens del design system web (modo claro).
- `desktop/src/components/ConnectionSelector.tsx` — ELIMINADO (componente muerto, solo self-references).
- Componentes vivos (ConnectionPanel, IngestForm, SearchBar, ResultsList) usan clases semánticas — cubiertas por el nuevo App.css, sin cambios TSX.

## Contrato
"`npm run build` en desktop/ pasa; la app abre en modo claro con tokens web; ConnectionSelector.tsx eliminado sin referencias rotas."

## Pasos
### Step 1: Migrar tema claro — ✅
- **Archivos:** `desktop/src/App.css`
- **Acción:** Tokens del web design system copiados de `web/src/app/globals.css`: cream `#FBF9F5`, ink `#000000`, neon `#FF5500`, paper `#F2EDE2`, smoke `#1A1A1A`, muted `#3A3A3A`. Estética linocut/neo-brutalist: bordes 2-3px `solid #000`, sombras duras `6px 6px 0 #000` (press effect), radius 0. Mantiene TODOS los nombres de clase usados por componentes vivos (panel, panel-head, row, narrow, stack, muted, tag, health-badge, conn-list, conn-name, dot, ghost, error, results, row-between, score).
- **Verify:** `npm run build` ✅ (tsc + vite build, 1.32s)

### Step 2: Eliminar ConnectionSelector.tsx — ✅
- **Archivos:** `desktop/src/components/ConnectionSelector.tsx` (deleted)
- **Acción:** grep confirmó solo self-references; `ConnectionInfo`/`HealthReport` viven en `src/vanta.ts` (imports vivos intactos). Post-delete grep: 0 matches.

### Step 3: Adaptar ConnectionPanel.tsx — ✅ (sin cambios)
- No requirió cambios: todas sus clases cubiertas por el nuevo App.css. No hay `tailwind.config.js` en desktop (no existe Tailwind en el proyecto desktop) — nada que adaptar allí.

## Dependencias
- Ninguna.

## Notas
- **Discrepancia de token:** el prompt decía neon `#FF5A45`, pero el source de truth real es `web/src/app/globals.css` → neon `#FF5500`. Se usó `#FF5500`.
- `desktop/tailwind.config.js` listado en la tarea NO existe (desktop no tiene Tailwind instalado). Se omitió; no se instaló Tailwind para 3 componentes (ponytail).

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** 847ab080 — `feat(ADMIN-03): migrate desktop UI to web design system light mode, drop dead ConnectionSelector`
- **CI pendiente:** no (build local pasado)
- **Decisiones:** Usar `#FF5500` (token real de globals.css) sobre `#FF5A45` del prompt porque la instrucción manda "copiar los tokens del design system web" y globals.css es la fuente. No tocar la web — solo copia.
- **Problemas conocidos:** El working tree tenía cambios pre-existentes de otras líneas; NO se tocaron ni commitean (solo staged App.css + ConnectionSelector.tsx).
- **Próxima tarea:** — (task única ejecutada)
