# Detalle INV — web SEO/auditorías

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Detalle de Tareas Completadas (desde Backlog)

<!-- 6 entradas movidas a ARCHIVO_HISTORICO.md (fase 1, 2026-08-03) -->
### INV-013: JSON-LD structured data — auditoría
- **Fuente:** Backlog (Investigaciones — Web Frontend)
- **Fecha:** 2026-08-03
- **Objetivo:** Auditar si el sitio web tiene JSON-LD structured data para SEO. Sin implementación — solo auditoría + propuesta.
- **Resultado:** ✅ Doc: `docs/research/INV-013-jsonld-structured-data.md`. **JSON-LD AUSENTE.** `layout.tsx` exporta metadata rico (title, description, keywords, authors, OG, twitter, icons, manifest) pero cero `<script type="application/ld+json">`/`jsonLd`. `page.tsx` sin metadata. Veredicto: Next.js 16 Metadata API NO genera JSON-LD — solo tags `<head>`; no existe campo `jsonLd`; hay que emitirlo manualmente en Server Component. Propuesta: schema.org/SoftwareApplication (VantaDB, DatabaseApplication, version 0.2.0, offers price 0, author ness-e, keywords). Validación: Google Rich Results Test / validator.schema.org. Cero cambios de código.
- **Superseded:** **NOTA CORREGIDA 2026-08-04** — la entrada anterior afirmaba que el JSON-LD se implementó posteriormente (commits `afe79fef` y `03b435c8`, "ver WEB-13"). **Es FALSA**: WEB-13 fue sobre `web/src/routes/` (Pages Router) que ya no existe tras la migración a App Router; los commits citados son de OG/canonical/sitemap y de un layout W9 previo a la migración — el JSON-LD NUNCA se implementó. Re-verificado 2026-08-04: sigue ausente en `web/src/app/layout.tsx` (solo Metadata API, que no genera JSON-LD). Pendiente en Backlog.
- **Ids:** `INV-013`

### INV-014: Light mode (CSS muerto) — auditoría
- **Fuente:** Backlog (Investigaciones Postg-Frontend)
- **Fecha:** 2026-08-03
- **Objetivo:** Auditar si el tema oscuro-only tiene CSS light-mode no usado. Sin implementación — solo auditoría + propuesta.
- **Resultado:** ✅ Doc: `docs/research/INV-014-light-mode-css.md`. Premisa invertida — **NO existe CSS light muerto; el sitio es LIGHT-ONLY por diseño** (paleta manga/linocut cream #FBF9F5 / ink #000000 / neon #FF5500). globals.css define SOLO tokens light (`@theme inline` + `:root`); cero `.dark`, cero `light:`, cero `prefers-color-scheme`. Wiring: `ThemeProvider` (next-themes) NO montado en layout; `ThemeToggle` consumer único es `navbar.tsx` (código muerto, reemplazado por site-navbar); next-themes solo importado por 2 componentes huérfanos. Recomendación: eliminar plomería DARK inerte (`theme-provider.tsx` + `theme-toggle.tsx` + dep next-themes de package.json, YAGNI). NO reactivar dark mode (contradice estética manga). Corregir nota stale en web/AGENTS.md.
- **Ids:** `INV-014`

### INV-015: Touch targets < 44px — auditoría
- **Fuente:** Backlog (Investigaciones Frontend)
- **Fecha:** 2026-08-03
- **Objetivo:** Auditar accesibilidad mobile: botones/enlaces con mínimo 44×44px WCAG 2.5.8. Sin implementación — solo auditoría + propuesta.
- **Resultado:** ✅ Doc: `docs/research/INV-015-touch-targets-44px.md`. ~**23 componentes no cumplen 44×44** (todos pasan 24px mínimo salvo 2 icon buttons 14px clear-search < 24px → severo). Inventario priorizado: P0 navbar (hamburger 36×36, search ⌘K 36×36, lang-toggle ~32×40, theme-toggle 36×36), P1 modales close (28-32px), P2 copy buttons (28px/hover-only), P3 nav links text-only (16-18px, 31 en footer), P4 clear/filter icon (14px). Fix: `size-11` (=44px) para icon-only, `min-h-[44px] min-w-[44px]` para texto. Ya cumplen: back-to-top 48×48, CTAs 48-52px, FAQ. Cero cambios de código.
- **Ids:** `INV-015`

### INV-016: Motion-duration tokens — auditoría
- **Fuente:** Backlog (Investigaciones Frontend)
- **Fecha:** 2026-08-03
- **Objetivo:** Auditar si existe sistema de tokens de animación consistente. Sin implementación — solo auditoría + propuesta.
- **Resultado:** ✅ Doc: `docs/research/INV-016-motion-duration-tokens.md`. **NO existen tokens de duración/easing.** globals.css solo colores+fuentes; easing `cubic-bezier(0.2,0.8,0.2,1)` hardcodeado en 4 lugares. Duraciones hardcodeadas: framer-motion (page-transition .28s, latency .4/.5s), Reveal (NOTE: NO framer-motion — CSS transition vía IntersectionObserver; default 600ms + delays 40-240), animejs (marks), Tailwind duration-* (75-1000). Propuesta: CSS vars `--duration-fast:150ms; --duration-normal:300ms; --duration-slow:500ms` + `--ease-default`; mapa JS `web/src/lib/motion.ts` `MOTION={duration:{fast:.15,normal:.3,slow:.5},ease:[.2,.8,.2,1]}` (framer-motion/animejs no leen CSS vars en `duration`); Reveal consume CSS vars vía transitionDuration. Cero cambios de código.
- **Ids:** `INV-016`
