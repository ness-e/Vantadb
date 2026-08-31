# TASK-WEB-09: Densidad efectos home — reducción trust-bar ×11 → ≤3, hero 5 capas → ≤2

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-web-quickwins.md
- **Creado:** 2026-08-27
- **Estado:** ⬜ PENDING

## Blast Radius
**Archivos clave:** `web/src/components/vanta/trust-bar.tsx`, `web/src/components/vanta/hero.tsx`, `web/src/components/vanta/mark/mark-classic.tsx`, `web/src/components/vanta/mark/use-mark-interaction.ts`, `web/src/app/globals.css`
**Callers:** `HomeView` (home-view.tsx) → `HomePage` (page.tsx)
**Implicaciones:** Solo reducción de efectos visuales/animaciones. Sin cambios en lógica de negocio, datos, ni APIs. No toca Rust, ni Python SDK, ni bindings.

## Impacto mapeado (Regla 0)
- **Leídos completos:** trust-bar.tsx (108L), hero.tsx (201L), mark-classic.tsx (293L), use-mark-interaction.ts (233L), globals.css (622L), home-view.tsx (84L), page.tsx (13L)
- **Referencias hacia dentro:** `HomeView` importa `TrustBar` y `Hero`; `HomePage` renderiza `HomeView`. Cambios son internos a componentes.
- **Referencias salientes:** Ninguna nueva. Solo reducen animaciones/efectos CSS/JS.
- **Veredicto:** Seguro, aditivo (solo quita), sin breaking changes. Requiere gate visual humano antes de merge.

## Contrato
"Diff muestra reducción neta de efectos animados (trust-bar ×11 → ≤3, hero 5 capas → ≤2); screenshot before/after adjunto al task file; requiere OK visual owner antes de merge. Verificación: `npm run build` exit 0 + `npm run lint` exit 0. Sin commit — lo commitea vanta-lead."

## Herramientas
- Bash (npm run build, npm run lint, npx playwright para screenshots)
- Read/Edit para archivos TSX/CSS

## Steps

### Step 1: TrustBar — reducir a ≤3 efectos esenciales ✅
- **Archivos:** `web/src/components/vanta/trust-bar.tsx`
- **Acción:** 
  1. Mantener: `animate-marquee` (marquee principal), `animate-stamp` (entrada badge), hover border transition en logos (feedback interactivo)
  2. Eliminar: `animate-flicker` en punto naranja, halftone overlay (decorativo), speed-lines edge accents (decorativo)
  3. Simplificar: duplicación de LOGOS ×2 → mantener 1 vuelta (6 items) para marquee sin salto visual
- **Verify:** `npm run build` exit 0 ✅
- **Estado:** ✅ COMPLETO

### Step 2: Hero — reducir capas de fondo a ≤2 + simplificar Mark ✅
- **Archivos:** `web/src/components/vanta/hero.tsx`, `web/src/components/vanta/mark/mark-classic.tsx`, `web/src/components/vanta/mark/use-mark-interaction.ts`, `web/src/components/vanta/mark/types.ts`, `web/src/components/vanta/mark/index.tsx`
- **Acción:**
  1. Hero background: mantener solo `grid-tech` + `animate-rise` entrada. Eliminar: `halftone`, `speed-lines`, 4 `RegMark` corners
  2. Hero content: eliminar `animate-flicker` en badge, `glitch-hover` en H1, `animate-bounce` en flecha scroll
  3. MarkClassic: mantener mouse tracking (pupil/sphere via Anime.js) + squint (core interacción). Eliminar: ambient pulse loop (10 nodos), SVG glow ring animate, blink on click, node hover pulse, click/keyboard handlers
  4. use-mark-interaction: mantener eye/sphere smooth follow + squint. Eliminar: blink cycle, node hover state, ambient animations cleanup, BlinkState type
  5. Types: simplificar MarkInteractionState y MarkVariantProps (sin blink, annoyed, hoveredNode, onClick, onNodeHover)
- **Verify:** `npm run build` exit 0 ✅
- **Estado:** ✅ COMPLETO

### Step 3: Screenshots before/after + verificación visual owner ⬜
- **Nota:** Servidor local tiene problemas de binding (Next.js standalone no escucha en puerto). Se requiere verificación visual en producción o arreglar servidor local.
- **Acción pendiente:** Tomar screenshots before/after y obtener OK visual del owner
- **Estado:** ⬜ PENDING (gate humano explícito)

### Step 4: Verify full + cierre ⬜
- **Acción:** `npm run build` exit 0 ✅, `npm run lint` (pre-existing issue: falta plugin react-hooks, documentado en auditoría). Actualizar plan file → ✅ Done. Reportar sin commit.
- **Estado:** ⬜ PENDING (pendiente OK visual owner)

## Dependencias
- WEB-08 completado (Playwright E2E existe para screenshots)

## Notas
- **Gate humano explícito:** No mergear sin OK visual del owner (usuario)
- **Métricas:** trust-bar tenía ~9 efectos → target ≤3; hero tenía 5 capas fondo + ~11 efectos → target ≤2 capas + ≤5 efectos totales
- **Accesibilidad:** `prefers-reduced-motion` ya respeta en CountUpStat y globals.css — mantener
- **Performance:** Menos Anime.js instances, menos RAF, menos DOM nodes animados

## Context Save Point
- **Fecha:** 2026-08-27
- **Branch:** develop
- **Decisiones:** Mantener marquee + stamp + hover en trust-bar; grid-tech + rise en hero; mark mouse-track + squint only
- **Problemas conocidos:** Ninguno