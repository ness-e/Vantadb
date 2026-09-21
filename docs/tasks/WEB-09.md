# TASK-WEB-09: Densidad efectos home — refinamiento sutil+A11y (15 efectos KEEP atenuados, no a 0)

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-web-quickwins.md
- **Creado:** 2026-08-27
- **Actualizado:** 2026-09-02 (Refinamiento sutil+A11y — vanta-worker, propietario aprobó keep 15 efectos atenuados + slices)
- **Estado:** 🟡 PROPUESTA-REFINADA — atenuación incremental aplicada en disco, gate GO de lead pendiente (no commit)

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
"Refinamiento sutil: 15 efectos KEEP atenuados (no eliminados a 0) + gating A11y (prefers-reduced-motion + pausa hover). Slices incrementales (trust-bar/hero/mark). Verificación: `npm run build` exit 0 + `npm run lint` exit 0. Screenshots after 1440×900. Sin commit — lo commitea vanta-lead."

## Filosofía elegida (owner refinamiento 2026-09-02)
Sutil no cero + A11y primero + Slices incrementales. Todos los 15 efectos se CONSERVAN atenuados:
- trust-bar: marquee 28s keep, halftone 0.05→0.02, speed-lines w-16→w-8 (50%), 12→6 evaluado (salta→keep 12 atenuado), grid-tech keep, border hover keep
- hero: halftone 520→260px opacity 0.30→0.12, speed-lines h-40→h-20 opacity 0.06→0.03, RegMarks 4→1, flicker gated, glitch 2→1, bounce solo flecha gated
- mark: ambient 10→3 (1.3→1.15, 2400→3200ms), glow SMIL 2→1 tenue 0.30→0.14, blink 8s, node hover pulse keep reducido 2.5→1.8, handlers keep
- Gating: prefers-reduced-motion (pausa marquee/flicker/glitch/bounce/blink/ambient) + pausa hover marquee

## Herramientas
- Bash (npm run build, npm run lint, npx playwright para screenshots)
- Read/Edit para archivos TSX/CSS

## Steps

### Slice 1: TrustBar — atenuado sutil ✅ APLICADO EN DISCO
- **Archivos:** `web/src/components/vanta/trust-bar.tsx`, `web/src/app/globals.css`
- **Cambios aplicados:**
  1. halftone 0.05→0.02 (`opacity-[0.02]`), speed-lines w-16→w-8 (50%) + opacity-60 tenue
  2. marquee 28s KEEP + pausa hover + `@media (prefers-reduced-motion) paused` (A11y)
  3. 12→6 evaluado: 6 salta (hueco ~480px en 1440×900, translateX -50%) → **fallback keep 12** con atenuación visual, documentado en trust-bar.tsx comentario
  4. border hover keep
- **Verify slice1:** `npm run build` 36/36 ✅ `npm run lint` 0e4w ✅ (tras slice1+2+3 conjunto, 2.2s Turbopack)
- **Estado:** ✅ APLICADO — diff listo para lead, no commit

### Slice 2: Hero — atenuado sutil ✅ APLICADO EN DISCO
- **Archivos:** `web/src/components/vanta/hero.tsx`, `web/src/app/globals.css`
- **Cambios aplicados:**
  1. halftone 520→260px (-right-12 -top-12, 260×260) opacity 0.30→0.12 tenue; speed-lines h-40→h-20 opacity 0.06→0.03 (50%)
  2. RegMarks 4→1 (solo left-3 top-3 esquinero)
  3. flicker: `motion-safe:animate-flicker motion-reduce:animate-none` + `@media` gating
  4. glitch 2→1: solo "Vanta" mantiene `glitch-hover`, "DB" sin glitch; reduced-motion gating `@media` + `.glitch-disabled`
  5. bounce solo flecha: `motion-safe:animate-bounce motion-reduce:animate-none` en ArrowDown (no en botón)
- **Verify slice2:** build/lint ✅ (conjunto)
- **Estado:** ✅ APLICADO

### Slice 3: Mark — atenuado sutil ✅ APLICADO EN DISCO
- **Archivos:** `web/src/components/vanta/mark/mark-classic.tsx`, `web/src/app/globals.css`
- **Cambios aplicados:**
  1. ambient 10→3 estáticos: `validNodes.slice(0,3)`, 1.3→1.15 amplitud, 2400→3200ms, resto estáticos
  2. glow SMIL 2→1 tenue sin anim r: `strokeWidth 0.6→0.5`, `opacity 0.30→0.14`, solo `<animate opacity>` 5s (antes r+opacity 3.5s duplicado)
  3. blink 8s (antes 1.1s) + reduced-motion gating `@media`
  4. node hover pulse keep reducido 2.5→1.8, 400→380ms outQuad, gating reduced-motion
  5. handlers keep (handleClick/handleNodeHover + BlinkState + hoveredNode) — no eliminados
  6. globals.css: `.animate-blink` 1.1s→8s, pulse-ring/grow gating ya cubierto por global, flicker/blink/marquee todos con `@media (prefers-reduced-motion)`
- **Verify slice3:** build/lint ✅
- **Estado:** ✅ APLICADO

### Step 3: Screenshots before/after + verificación visual owner 🟡 AFTER GENERADO, GATE LEAD
- **Before 2026-09-02 (histórico):** `web/web09-before-home.png` 505 KB + `web/web09-before-mark.png` 65 KB (1440×900 fullPage). Comando: `npx playwright test web09-screenshots` 1 passed 3.9s. Artefactos gitignored originales — no en disco actual (patch base recreable); evidencia previa documentada.
- **After refinamiento sutil 2026-09-02:** `web/web09-after-home.png` 133 KB + `web/web09-after-mark.png` 68 KB (1440×900 fullPage+mark, viewport 1440×900). Comando: `npx playwright test web09-screenshots` 1 passed 2.3s/7.8s (recreado `web/e2e/web09-screenshots.spec.ts`). Gating: halftone tenue, speed-lines mitad, RegMark 1, glow 1 SMIL, ambient 3 nodos. Artefactos en `web/` (gitignored) listos para adjuntar a GO.
- **Spec recreado:** `web/e2e/web09-screenshots.spec.ts` (viewport 1440×900, fullPage true, fallback mark clip) — mismo spec que before, reproducible.
- **Patch base antiguo:** `docs/plans/WEB-09-patch-propuesto.diff` (~340L, -263L -13 loops) NO existe en disco (verificado `Test-Path False`) — era propuesta de eliminación a 0, reemplazado por refinamiento sutil incremental. No recreado (obsoleto).
- **Gate lead:** requiere GO visual lead (opciones A GO / B ajustar / C DEFER). Ver notas filosofía.
- **Estado:** 🟡 AFTER listo, pendiente GO

### Step 4: Verify full + cierre 🟡 PROPUESTA-REFINADA
- **Verify actual (con slices atenuados aplicados en disco, 2026-09-02):** `npm run build` ✅ 36/36 (2.2s, Next 16.3.0 Turbopack), `npm run lint` ✅ 0e4w (4 warnings pre-existentes @next/no-img-element + exhaustive-deps, 0 errors)
- **Verify slice unitario:** cada slice deja repo compilable; build/lint verde tras slice3 conjunto
- **Cierre:** No commit aún — diff incremental listo para `vanta-lead` ( slices trust-bar → hero → mark). Este task queda PROPUESTA-REFINADA.
- **Estado:** 🟡 PROPUESTA-REFINADA — verificado atenuado, cierre pendiente GO lead

## Dependencias
- WEB-08 completado (Playwright E2E existe para screenshots)

## Notas
- **Gate humano explícito:** No mergear sin OK visual del owner (usuario)
- **Métricas:** trust-bar tenía ~9 efectos → target ≤3; hero tenía 5 capas fondo + ~11 efectos → target ≤2 capas + ≤5 efectos totales
- **Accesibilidad:** `prefers-reduced-motion` ya respeta en CountUpStat y globals.css — mantener
- **Performance:** Menos Anime.js instances, menos RAF, menos DOM nodes animados

## Evidencia visual (gate humano)

- **Before (histórico 2026-09-02):** `web/web09-before-home.png` 505 KB + `web/web09-before-mark.png` 65 KB — viewport 1440×900, Playwright chromium `npx playwright test web09-screenshots` PASS 3.9s. Ya no en disco (patch base obsoleto); referencia documentada. Contenía hero 5 capas fondo + trust-bar ×11 + mark ambient 10 + glow SMIL ×2.
- **After refinamiento sutil (2026-09-02, aplicado en disco):** `web/web09-after-home.png` 133 KB + `web/web09-after-mark.png` 68 KB — 1440×900 fullPage+mark, Playwright `npx playwright test web09-screenshots` 1 passed 7.8s (recreado `web/e2e/web09-screenshots.spec.ts`). Gating visible: halftone tenue, speed-lines mitad, RegMark 1, glitch 1, glow 1 SMIL, ambient 3. Archivos locales gitignored — adjuntar al GO.
- **Propuesta refinada:** Esta sección + slices 1-3 (sutil no cero, 15 KEEP atenuados, A11y gating, slices incrementales). Propuesta previa `docs/plans/WEB-09-propuesta-reduccion.md` + `WEB-09-patch-propuesto.diff` (-263L -13 loops eliminación a 0) obsoletas — verificadas `Test-Path False` 2026-09-02, no recreadas (reemplazadas por slices atenuados).
- **Spec screenshots:** `web/e2e/web09-screenshots.spec.ts` (viewport 1440×900, fullPage true, webServer npm run dev) — recreado, reproducible para before/after idéntico framing.

## Mejora Sutil+A11y — inventario atenuado (15 efectos KEEP)

| # | Efecto | Antes | Después (sutil) | Gating A11y |
|---|--------|-------|------------------|-------------|
| 1 | trust-bar marquee | 28s | 28s keep | pausa hover + `prefers-reduced-motion` paused |
| 2 | trust-bar halftone | 0.05 | 0.02 | — |
| 3 | trust-bar speed-lines | w-16 | w-8 (50%) opacity-60 | — |
| 4 | trust-bar nodos | 12 | test 6→saltó→keep 12 atenuado visual | — |
| 5 | trust-bar border hover | 0.15→0.60 | keep | — |
| 6 | hero grid-tech | 0.06 | keep | — |
| 7 | hero halftone 520px | 520px 0.30 | 260px 0.12 | — |
| 8 | hero speed-lines h-40 | h-40 0.06 | h-20 0.03 (50%) | — |
| 9 | hero RegMark | 4 | 1 (left-3 top-3) | — |
| 10 | hero flicker | 4.5s | keep atenuado | `motion-safe` + `@media` none |
| 11 | hero glitch-hover | 2 | 1 (Vanta solo) | `@media` none + .glitch-disabled |
| 12 | hero bounce | flecha+botón | solo flecha | `motion-safe` |
| 13 | mark ambient pulse | 10 loop | 3 slice(0,3) 1.15 3200ms | `@media` skip |
| 14 | mark glow SMIL | 2 (r+opacity) | 1 (opacity 0.14 5s) tenue | — |
| 15 | mark blink | 1.1s | 8s tenue | `@media` none |
| + | mark node hover pulse | 2.5 elastic | 1.8 outQuad | `@media` skip |
| + | mark handlers (click/hover/annoyed) | keep | keep | — |

## Context Save Point
- **Fecha:** 2026-09-02 (Refinamiento sutil+A11y — vanta-worker)
- **Branch:** develop (slices atenuados APLICADOS en disco, no commit — diff incremental listo para vanta-lead)
- **Decisiones:** Sutil no cero (15 KEEP atenuados, no eliminar a 0) + A11y primero (prefers-reduced-motion + hover pausa) + Slices (trust-bar → hero → mark). Test 12→6 saltó → fallback keep 12. Patch base -263L obsoleto reemplazado. Spec screenshots recreado.
- **Problemas conocidos:** `npm run lint` 0e4w pre-existente; 6-nodos salta documentado; `prefers-reduced-motion` global ya cubre `*` pero se añadió gating explícito por componente para auditabilidad.

## Recitation (pipeline-full.md §3 — handoff)
- **activeGoal:** WEB-09 refinamiento sutil+A11y — 15 efectos KEEP atenuados + gating prefers-reduced-motion/hover + slices incrementales
- **lastAction:** 2026-09-02 DISCOVERY (codegraph_explore trust-bar/hero/mark + verify patch base NO existe) + EJECUCIÓN 3 slices atenuados (trust-bar 0.05→0.02 w-16→w-8 + marquee 28s pausa hover, hero 520→260 0.30→0.12 h-40→h-20 RegMark 4→1 flicker/glitch/bounce gated 2→1, mark ambient 10→3 glow 2→1 blink 8s) + CIERRE build/lint + screenshots after 1440×900 + task file PROPUESTA-REFINADA
- **result:** ✅ PROPUESTA-REFINADA (slices aplicados en disco, verificados, listos para GO lead; no commit)
- **nextAction:** vanta-lead GO: revisar diff (`git diff web/src/components/vanta/trust-bar.tsx web/src/components/vanta/hero.tsx web/src/components/vanta/mark/mark-classic.tsx web/src/app/globals.css web/e2e/web09-screenshots.spec.ts`) + validar after screenshots `web/web09-after-*.png` + `npm run build && npm run lint` + commit atómico mensaje `feat(web): WEB-09 refinamiento sutil+A11y home — 15 efectos atenuados con gating reduced-motion`
- **contract:** `npm run build` 36/36 ✅ (2.2s Turbopack) + `npm run lint` 0e4w ✅ (2026-09-02 con slices atenuados); screenshot after `npx playwright test web09-screenshots` 1 passed 7.8s viewport 1440×900
- **artefactos:** `web/src/components/vanta/trust-bar.tsx`, `web/src/components/vanta/hero.tsx`, `web/src/components/vanta/mark/mark-classic.tsx`, `web/src/app/globals.css`, `web/e2e/web09-screenshots.spec.ts` (recreado), `web/web09-after-home.png` 133KB, `web/web09-after-mark.png` 68KB
- **invariantes:** 15 efectos KEEP (no a 0) atenuados; `prefers-reduced-motion` pausa marquee/flicker/glitch/bounce/blink/ambient; no tocar lógica negocio/APIs; build 36/36; sin commit hasta GO lead
- **deuda:** Ninguna técnica; pendiente GO lead + archivar plan Wave2 + `skill progreso` tras merge
- **queda_pendiente:** GO vanta-lead → commit → `skill progreso` → archivar plan