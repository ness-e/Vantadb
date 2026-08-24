# Plan — Auditoría de Diseño Web VantaDB (2026-08-19) — v2 CORREGIDA

> **v2:** Plan corregido tras investigación multi-agente (6 sub-agentes en paralelo + verificación directa) del estado **REAL** del código. Los docs `web/AGENTS.md`, `web/AUDIT.md`, `web/worklog.md` estaban desactualizados; cada afirmación de v1 que cambió está marcada **[CORREGIDO]**.

**Objetivo:** Auditar, limpiar y mejorar `web/` en 7 dimensiones: **diseño, estructura, información, UI, performance, escritura, diseño comercial**. Hallazgos con evidencia → triage → remediación.

**Alcance:** `web/src/**` · `web/package.json` · `web/next.config.ts` · `web/public` · `web/AGENTS.md`. No tocar `web/remotion/`.

---

## 1. Estado REAL verificado (2026-08-19, contra el código)

### Arquitectura
- **32 `page.tsx`** en App Router. **Todas** pasan por SiteShell (root `layout.tsx:134`). **[CORREGIDO] vs "SPA 3 vistas"** (ya migrado, AGENTS.md parcialmente stale).
- `docs-api/page.tsx` es la **única server component** (redirect 307 → /docs). Los 31 route layouts son server (metadata). **[CORREGIDO] vs AGENTS.md "no RSC anywhere"**.
- `next.config.ts` real: `output: "standalone"` + `reactStrictMode: true`. **NO** hay `ignoreBuildErrors`, NO hay turbopack config, NO headers/redirects. **[CORREGIDO] vs AGENTS.md/AUDIT.md**.
- `not-found.tsx` **NO existe** → el 404 es el catch-all `[...slug]/page.tsx` (estilizado, con SiteShell, i18n y metadata noindex). El impacto del AUDIT #4 (404 sin navbar) era falso — el shell SÍ renderiza — pero la estructura catch-all persiste.
- `tailwind.config.ts` sigue **inerte** (v3 syntax, v4 CSS-first). `globals.css` = 622 líneas/13.9KB, ~100 líneas en blanco, clases plain-CSS en `@layer utilities` (se emiten siempre).

### Tema / dark mode — **[CORREGIDO] más grave que AUDIT**
- **Dark mode NO existe** (no "roto"): `globals.css` solo define `:root` (sin `.dark`/`prefers-color-scheme`), **no hay** `theme-provider.tsx`, `theme-toggle.tsx`, ni `next-themes` en package.json. `site-navbar.tsx:354` tiene un comentario "search, lang, theme, github" pero solo renderiza LangToggle. Quedan claves i18n huérfanas (`theme.light/dark` dictionaries.ts:561-562, `shortcuts.toggleTheme` :591).
- **Decisión requerida:** ¿implementar dark mode bien (P2) o descartar y limpiar huérfanas? (El design system inviolable dice "tema claro por defecto" — light-only es válido).

### i18n
- Custom `LanguageProvider` + `dictionaries.ts`: **1,220 claves ES = 1,220 EN**, simétricas, ~85 namespaces. `next-intl` instalado, **0 imports**.
- `lang` dinámico **[CORREGIDO #2 FIXED]**: `layout.tsx:120` `<html lang={DEFAULT_LANG}>` ("es" solo SSR), `language-provider.tsx:38,44,53` sincroniza `document.documentElement.lang` en runtime.
- Helper `tt()` duplicado **36 veces** (21 páginas + 15 componentes) — peor que el AUDIT (3).

### Rutas / navegación — **[CORREGIDO]**
- **`/why-vantadb` y `/solutions/local-rag` SÍ EXISTEN en disco** (page.tsx verificados) — 2 agentes reportaron "rutas fantasma" pero era falso; están en LIVE_ROUTES, navbar, footer y sitemap correctamente.
- LIVE_ROUTES = 36 entradas (32 estáticas + 4 blog slugs + 3 case slugs + prefix-checks).
- Sitemap = 29 URLs; **7 rutas en disco faltan del sitemap**: `/demo`, `/showcase`, `/config`, `/storage`, `/latency`, `/integrations`, `/docs-api`.
- Navbar: `site-navbar.tsx` activo (4 dropdowns + Security/Pricing flat + logo/⌘K/LangToggle/GitHub). Navbar viejo `navbar.tsx` = dead code.

### Dead code / dependencias
- **5 componentes muertos**: `navbar.tsx` (546), `hero-mark-interactive.tsx`, `metrics-bar.tsx`, `ecosystem.tsx`, `vs-table.tsx` (~1,500 líneas). AGENTS.md documenta 4 (falta vs-table).
- **Carpeta `ui/` completa (~40 wrappers shadcn) muerta**: solo `toaster/sonner/toast` son alcanzables desde layout. `recharts`, `react-hook-form`, `cmdk`, `vaul`, `embla-carousel-react`, `input-otp`, `react-day-picker`, `react-resizable-panels` y ~20 paquetes radix viven tras wrappers muertos (no llegan al bundle, pero son ~26 archivos basura).
- **14 deps zombies** (0 imports, eliminables): `@dnd-kit/{core,sortable,utilities}`, `@hookform/resolvers`, `@reactuses/core`, `@tanstack/{react-query,react-table}`, `date-fns`, `next-intl`, `react-markdown`, `uuid`, `z-ai-web-dev-sdk`, `zod`, `zustand`. **[CORREGIDO] vs 17 del AUDIT** — ya se quitaron @mdxeditor/editor, next-auth, react-syntax-highlighter, sharp.
- **Doble Toaster**: `layout.tsx:135` shadcn `<Toaster/>` (nunca se dispara) + `:136` `<Sonner>` (activo). Render muerto en cada página.

### Assets — **NUEVO hallazgo crítico**
- `public/assets/` **existe pero está VACÍO** → 4 referencias rotas:
  1. `about/team/page.tsx:53` — avatares `/assets/avatar_gato.png` + `/assets/mascota_gato.png` (vanta-data.ts:1077,1084) → **imágenes rotas visibles**
  2. `easter-egg.tsx:76` — `<img src="/assets/mascota_gato.png">` → **404**
  3. `opengraph-image.tsx:15` — lee `public/assets/mascota_gato.png` → **fallback 🐱** (OG degradado)
  4. `layout.tsx:96` — JSON-LD logo → raw.githubusercontent URL (probable 404)
- `public/vanta-wasm/` = 1.18MB wasm + glue (playground). `favicon.png` 12.8KB. **0 uso de next/image** (solo `<img>`).

### Animaciones / perf
- **Leak real**: `mark-classic.tsx:76-94` — bucle animejs `loop:true` sin cleanup (comentario falso "Cleanup handled by anime.js on unmount"); además no respeta prefers-reduced-motion. `use-mark-interaction.ts:143-149` SÍ tiene `.pause()` parcial. +3 `setTimeout` sin gestionar (`use-mark-interaction.ts:193,205`, `mark-cta.tsx:110`) y `latency-comparator.tsx:137` setInterval sin cleanup.
- **Bundle global**: `framer-motion` (PageTransition en SiteShell, ~35KB gz todas las rutas) + `vanta-data.ts` (1,136 líneas, vía command-palette en SiteShell) + doble toast. **0 `next/dynamic`** en todo el proyecto.
- `eslint.config.mjs` desactiva **30+ reglas** (no-unused-vars, no-explicit-any, exhaustive-deps…) → todo el dead code pasa silencioso. **[NUEVO]**

### Contenido — claims verificados vs README/BENCHMARKS del repo
- ✅ Reales: 1.2ms p50 (BENCHMARKS.md:33), 3,636 QPS/2.80x, BENCH01 (13.2ms/2.0ms/3.1ms), SIFT1M, WAL CRC32C, RRF, BM25, HNSW, PyO3, **versión real v0.5.0** (no v0.1.x).
- ❌ **"100% Recall@10"** cherry-pick: real = 0.9560 (Block 1) / 0.9980 (10K-100K); solo 50K = 1.0000.
- ❌ **Tabla competitiva se contradice a sí misma**: sitio muestra VantaDB p50 4.124ms/241 QPS/recall 1.0 (competitive-benchmark.json) pero el propio BENCHMARKS.md §7 (mismo dataset) = **39.74ms/24.3 QPS/recall 24.50%**. Y /latency cita 39.74ms → **3 p50 distintos para lo mismo** (2.024 / 4.124 / 39.74).
- ❌ **Snippet QUICKSTART no coincide con README**: sitio usa `import vantadb_py` + `db.get/search`; README canónico = `import vantadb` + `get_memory/search_memory` → **dev que copie el snippet falla**.
- ❌ **SHOWCASE**: 6 URLs son archivos first-party reales (langgraph_checkpoint.py, crewai_memory.py…) pero se venden como "comunidad" con author `@ness-e` y los títulos del diccionario no coinciden con los repos linkeados.
- ❌ **CASE_STUDIES**: 3 historias anónimas ("Indie AI Studio"…) **sin disclaimer de composite** → fabricadas sin aviso.
- ❌ **BLOG**: fechas 2025-01-15… predatan el primer commit del repo (2026-04-02) → fechas inventadas.
- ❌ **CHANGELOG del sitio**: 3 versiones inventadas (0.1.0-0.1.2, fechas 2025) que no existen; el repo real solo tiene 0.4.0/0.5.0.
- ⚠️ **Versión inconsistente**: navbar "v0.1 · embedded rust" (site-navbar.tsx:311) vs `PRODUCT.versions.vantadb = "v0.5.0"` (vanta-data.ts:42) vs hero "0.5.0 · MVP".
- ⚠️ TEAM_MEMBERS: ness-e real + 3/4 no son personas (Vanta Cat, "Community", "Open Source").
- ⚠️ PRICING: 2 planes ($0/Custom), "Contact Sales" → GitHub repo; TCO menciona "Team plan" inexistente; $1,800 Pinecone/$600 Weaviate sin fuente; "∞ egress" retórico.
- ⚠️ Integraciones "native" (OpenAI/Ollama/CrewAI/Haystack/LiteLLM) contradicen Product Boundary del README ("Experimental"). Trust "0 deps — Zero external deps" falso (Fjall/RocksDB/PyO3).

### SEO / dominio — **[NUEVO]**
- **Sitemap + canonicals apuntan a `vantadb.dev` que NO resuelve** (transport error); el sitio real sirve en `vantadb.vercel.app` (verificado). Todos los canonicals/OG muertos. `metadataBase` = URL del repo GitHub (layout.tsx:36).
- **Sin og:image en ningún layout** (solo favicon); twitter card "summary_large_image" sin imagen = cards rotas.
- Default `lang="es"` + copy español para audiencia dev anglófona; FAQ 100% español → poco citable por LLMs EN.
- `turbopack.root` sin configurar (lockfile duplicado en `~` y proyecto).

### i18n residual / textos literales — **[NUEVO]**
- `code-playground.tsx` 100% EN sin i18n (Run/Reset/aria-labels, "playground.js", "// press Run") + **`new Function`** ejecuta código arbitrario (playground.tsx:319).
- `site-navbar.tsx` aria-labels EN hardcoded ("Main navigation", "Mobile navigation", "Toggle menu", "GitHub", "v0.1 · embedded rust").
- `mark-classic.tsx:244` labels EN ("◆ blink", "◆ click me · move mouse"); `:165` aria-label EN.
- ~12+ textos literales sin tt(): code-terminal.tsx:83-90, benchmarks-view.tsx:203,423, competitive-table.tsx:153, code-playground.tsx:563, latency-comparator.tsx:473, docs-view.tsx, command-palette.tsx:225.
- aria-labels ES hardcoded ×6 (back-to-top:43, command-palette, easter-egg:100, tutorial-modal:109).

---

## 2. Skills aplicables — [ACTUALIZADO 2026-08-19] revisión contra las 193 skills del catálogo (SKILLS-MANIFEST, verificado en disco)

> Revisión completa del catálogo: se agregaron las 6 faltantes de ingeniería + 16 más que aplican; se quitó `ai-seo` duplicada (queda solo en Medición).

**Core/Orquestación:** impeccable (10/10) · vanta-design-orchestrator (9, cargar primero para diseño UI) · design-taste-frontend (9) · visual-critique (7) · plan-design-review (7) · planning-and-task-breakdown (8).
**Ingeniería:** source-driven-development (8) · doubt-driven-development (8) · frontend-ui-engineering (7) · systematic-debugging (8) · incremental-implementation (8) · test-driven-development (8).
**Medición:** visual-review (8) · playwright-cli (7, canónico web/) · webapp-testing (7) · browser-testing-with-devtools (7) · a11y-accessibility-scan (8) · a11y-accessibility-audit (8) · a11y-accessibility-fix (8) · a11y-accessibility-inspect (8) · a11y-accessibility-diff (8) · audit-website (7) · seo-audit (7) · ai-seo (8) · performance-optimization (8) · vercel-react-best-practices (9) · next-best-practices (8).
**UX/UI:** ux-heuristics (8) · web-design-guidelines (7) · high-end-visual-design (8) · industrial-brutalist-ui (7) · redesign-existing-projects (8) · ui-design (8) · frontend-designer (8) · responsive-craft (8) · design-systems (8).
**Motion:** design-motion-principles (8, modo AUDIT) · emil-design-eng (9) · emilkowalski-motion (7) · interaction-design (8) · motion (9, solo si se reimplementa animación del mark sin animejs).
**A11y inclusiva (subset incl-*):** incl-inclusive-interaction-touch-target-design (R-FE-5, ≥44px) · incl-inclusive-interaction-motion-sensitivity (reduced-motion) · incl-inclusive-interaction-keyboard-review (F4) · incl-accessible-content-heading-structure (h1, F2) · incl-cognitive-accessibility-cognitive-load-assessment (ruido hero, F1).
**Escritura/comercial:** writing-guidelines (6) · copywriting (7, global).

**Descartadas (revisadas, no aplican a este run):** brandkit/imagegen-frontend-web (no se crean assets nuevos — se restauran faltantes) · remotion/hyperframes (video) · threejs-* (3D) · sleek-design-mobile-apps (mobile) · ux-strategy/prototyping-testing (no UX research) · understand-* (CodeGraph cubre introspección) · unified-review (gate de release, no auditoría web) · vercel-optimize (requiere métricas de cuenta Vercel) · gsap-* (no se reimplementa con GSAP) · animejs (REMOVED del catálogo — redundante con motion) · react-state-management (no se rediseña estado) · las 53 incl-* restantes (cobertura de diseño inclusivo general, no fases concretas del plan).

---

## 3. Fases de auditoría (actualizado con hallazgos reales)

### Fase 0 — Baseline medible
- `npm run build` + `npm start` → Lighthouse desktop/mobile (`audit-website`).
- `visual-review`: screenshots 1440×900 + 390×844 de 15 rutas (incl. about/team para ver las imágenes rotas).
- Escaneo a11y en rutas clave (`a11y-accessibility-scan`).
- Verificación de rutas: **`/why-vantadb` y `/solutions/local-rag` existen** (confirmado) — no re-auditar como fantasma.

### Fase 1 — Diseño
- **Dark mode: decidir** (implementar P2 vs descartar + limpiar huérfanas theme.*). Ya NO es "arreglar toggle roto".
- Jerarquía hero → §01-§09; ruido por exceso de efectos (speed-lines, halftone, SfxLabel, glitch, marquee, easter egg).
- Tipografía (Anton/Space Mono), contrastes cream/paper/ink/neon, consistencia entre las 32 rutas (PageHeader uniforme).
- Verificar el mark interactivo (animejs): movimiento, labels EN, nodos SVG.

### Fase 2 — Estructura
- Migrar catch-all → `not-found.tsx` + `notFound()` (mantener diseño estilizado actual).
- **7 rutas fuera del sitemap** → añadir (demo, showcase, config, storage, latency, integrations, docs-api).
- **Sitemap/canonicals/metadataBase → dominio real** (vantadb.vercel.app o comprar vantadb.dev; hoy todos muertos).
- **h1 en 4 thin wrappers**: /engine, /architecture, /changelog, /playground usan h1 sr-only (`page.tsx:13-16`) — evaluar si es suficiente SEO o restaurar h1 visible.
- AGENTS.md actualizar (config real, 32 rutas, dead code incl. vs-table, ui/ muerta).
- Extraer `tt()` (36 copias) a `src/lib/i18n-utils.ts` — quitar duplicación.

### Fase 3 — Información
- **Corregir claims**: recall 100% → 0.998 con contexto; **resolver 3 p50 distintos** (2.024/4.124/39.74) eligiendo una fuente coherente por contexto; tabla competitiva vs BENCHMARKS.md §7.
- **Corregir snippet QUICKSTART** → `import vantadb` + `get_memory/search_memory` (README canónico).
- **Decidir sobre contenido dudoso**: showcase (marcar como "ejemplos oficiales" o renombrar), case studies (añadir disclaimer composite), blog (fechas reales o quitar), changelog (usar versiones reales 0.4.0/0.5.0 con fechas 2026), versión navbar → v0.5.0.
- Completar i18n residual: code-playground completo, site-navbar aria-labels, ~12 textos literales.
- Añadir faltantes: link LICENSE, `cargo install`, roadmap, docs API reales (o honesto "en GitHub").

### Fase 4 — UI
- Componentes interactivos: command palette, tutorial modal, playground (`new Function` → evaluar sandbox/simulador), latency comparator, benchmark race, WAL simulator, vs-table.
- Estados hover/active/focus/empty; forms (demo no tiene form — redirige a /playground).
- **Quitar Toaster shadcn muerto** (layout.tsx:135) — queda Sonner.
- Responsive 390px en 15 rutas (tablas, navbar 4 dropdowns).
- A11y fina: aria-hidden decorativos (mark-classic:223-234), hit-circles con nombre (:140-152), eventos sintéticos (3× dispatchEvent KeyboardEvent).

### Fase 5 — Performance
- **Quitar 14 deps zombies** + carpeta `ui/` muerta (~26 archivos, ~8 paquetes tras wrappers) → ahorro >100MB node_modules.
- **Restaurar assets faltantes** (mascota_gato.png, avatar_gato.png) o quitar referencias — 4 puntos rotos (team, easter-egg, OG, JSON-LD).
- **Fix leak animejs** mark-classic.tsx:76-94 (cleanup + prefers-reduced-motion) + setTimeouts/setInterval sin gestión.
- Code-splitting: dynamic import framer-motion/PageTransition (o limitar a /), command-palette + vanta-data fuera del bundle global, animejs solo en /.
- next/image para assets (webp/avif) cuando existan.
- `eslint.config.mjs`: reactivar reglas clave (no-unused-vars, no-explicit-any) — el dead code pasó silencioso.
- turbopack.root fix (lockfile raíz).

### Fase 6 — Escritura
- Copy por ruta (tono, claridad ES/EN), microcopy (toasts "coming soon", "prefers nap time", "you found the shadow cat").
- Textos literales sin i18n → mover a dictionaries (Fase 3).
- SEO copy ES/EN: FAQ bilingüe (o al menos EN) para citabilidad LLM.

### Fase 7 — Diseño comercial
- **Dominio**: decidir vantadb.dev (comprar/resolver) vs vercel.app — canonical + OG image reales (hoy rotos).
- **Value prop**: H1 es solo marca "VantaDB"; subhead sí comunica QUÉ/POR QUÉ, falta PARA QUIÉN (devs de agentes/RAG) en el primer pliegue. 5 CTAs compitiendo en hero → jerarquizar (primario: install o quickstart).
- **Pricing**: 2 planes sin mid-tier; "Contact Sales" → GitHub; decidir plan Team $49 (existía en v2) o eliminar menciones (TCO row "Team plan").
- **Social proof**: case studies con disclaimer o quitar; showcase "community" → "official examples"; trust "0 deps" corregir; integrar "native/experimental" coherente con README.
- **Funnel demo**: /demo promete beta/WASM pero redirige a /playground — alinear copy (o implementar waitlist real, o quitar promesa).
- **SEO comercial**: titles/descriptions cubren intención dev ("vector database rust", "local rag", "agent memory"); JSON-LD (Organization, SoftwareApplication) presente pero con logo roto.

### Fase 8 — Triage y remediación
**P0 (roto visible):** assets faltantes (4 refs) · dominio canonicals/OG · claims falsos (recall, p50, snippet, versión navbar).
**P1 (limpieza):** 14 deps zombies · ui/ muerta · Toaster shadcn · 5 componentes muertos · tt() ×36 → lib · catch-all → not-found.
**P2 (polish):** dark mode (decidir) · sitemap +7 · i18n residual · a11y fina · animejs leak · eslint reglas · code-splitting.
**P3 (comercial):** value prop hero · pricing mid-tier · showcase/case-studies etiquetado · demo funnel · FAQ EN · SEO/LLM.
Slices verticales, cada uno: lint + build + visual-review. Commits atómicos (conventional commits).

---

## 4. Entregables
1. `docs/reviews/web-design-audit-2026-08-19.md` — reporte con evidencia por dimensión + scores (gate `plan-design-review`).
2. Cambios aplicados P0-P3 con commits atómicos.
3. `AGENTS.md` + `AUDIT.md` actualizados al cierre.

## 5. Verificación final
- `npm run lint` 0/0 (con reglas reactivadas) · `npm run build` sin warnings nuevos · Lighthouse ≥90 (desktop) · a11y sin críticas · 32 rutas 200 · visual-review sin regresiones.

---

## 6. Tareas registradas (formato campaign — para /pipeline run)

> Registradas 2026-08-19 por vanta-lead. Ejecución secuencial por fases (las fases comparten `web/src` — no paralelizar para evitar conflictos de merge). R-FE-4 ya decide dark mode = NO.

### Task 1: WDA-00 — F0 Baseline medible
- **Archivos clave:** `web/package.json`, `web/next.config.ts`, `web/src/app/layout.tsx`
- **Gate Justificación:** medición inicial; genera screenshots/lighthouse/a11y que informan F1-F8
- **Contrato:** `npm run build` en `web/` pasa (exit 0) + 15 rutas screenshot 1440×900 y 390×844 en `web/audit-baseline/` + lighthouse desktop + escaneo a11y en rutas clave
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-00.md`
- **Estado:** ✅ COMPLETED (2026-08-24) — build exit 0 (35 rutas) · 30 screenshots 15 rutas ×2 viewports · Lighthouse: home perf96/a11y95/bp100/seo100, team perf95/a11y96/bp100/seo100 · a11y axe: home color-contrast×19+heading-order+label-mismatch, team color-contrast×5+label-mismatch · lint limpio hoy. Detalles en `tasks/WDA-00.md`. Artefactos locales en web/audit-baseline/ (gitignored).

=== RECITATION ===
Objetivo activo: RUN WDA-00..WDA-08 — Auditoría Diseño Web
Estado: act
Última acción: plan registrado (§2 skills completado, §6 tareas WDA-00..08); WDA-00 delegada a vanta-worker
Resultado: ⬜
Próxima acción: esperar RESULTADO de WDA-00 → verify contrato → commit scoped
Invariantes: NO tocar web/remotion/ · NO tocar cambios desktop sin commit (Cargo.lock, completions/, desktop/, src/cli_server.rs) · commits solo web/ + docs/plans/ + tasks/WDA-* · R-FE-4 light-only (no dark mode)
Comandos de verificación: por tarea (ver Contrato en §6)
Deuda: tracking manual (MCP con 29 tareas fantasma corruptas — no usar campaign_update_task_state)
Próxima tarea si completa: WDA-01 — F1 Diseño
last-synced: 2026-08-24T00:00
=== END RECITATION ===

### Task 2: WDA-01 — F1 Diseño
- **Archivos clave:** `web/src/components/vanta/site-navbar.tsx`, `web/src/components/vanta/mark-classic.tsx`, `web/src/lib/dictionaries.ts`, `web/src/app/globals.css`
- **Gate Justificación:** dark mode decisión (R-FE-4 = NO) + jerarquía hero + tipografía + limpiar huérfanas theme.* + fix leak animejs (cleanup + prefers-reduced-motion)
- **Contrato:** `grep -c "theme.dark\|theme.light\|toggleTheme" web/src/lib/dictionaries.ts` = 0; comentario "theme" removido de site-navbar; mark-classic sin `loop:true` sin cleanup; `npm run build` en `web/` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-01.md`
- **Estado:** ⬜ PENDING

### Task 3: WDA-02 — F2 Estructura
- **Archivos clave:** `web/src/app/[...slug]/page.tsx`, `web/src/app/sitemap.ts`, `web/src/app/layout.tsx`, `web/src/lib/`
- **Gate Justificación:** catch-all → not-found.tsx; sitemap +7 rutas; metadataBase dominio real; extraer tt() ×36 a lib
- **Contrato:** existe `web/src/app/not-found.tsx`; sitemap ≥36 URLs; `grep -rn "const tt = \|function tt(" web/src/lib/i18n-utils.ts` existe y páginas lo importan; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-02.md`
- **Estado:** ⬜ PENDING

### Task 4: WDA-03 — F3 Información (claims)
- **Archivos clave:** `web/src/components/vanta/vanta-data.ts`, `web/src/components/vanta/site-navbar.tsx`, `web/src/app/quickstart/page.tsx`
- **Gate Justificación:** claims falsos P0 (recall 100%, 3 p50, snippet QUICKSTART, versión navbar) + etiquetado showcase/case-studies
- **Contrato:** sin "100% Recall" en `web/src`; snippet QUICKSTART = `import vantadb` + `get_memory/search_memory`; versión navbar = v0.5.0; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-03.md`
- **Estado:** ⬜ PENDING

### Task 5: WDA-04 — F4 UI
- **Archivos clave:** `web/src/app/layout.tsx`, `web/src/components/vanta/*`, `web/src/components/ui/`
- **Gate Justificación:** quitar Toaster shadcn muerto (layout:135); componentes interactivos; a11y fina (targets ≥44px R-FE-5); responsive 390px
- **Contrato:** layout.tsx sin `<Toaster/>` shadcn (queda Sonner); `grep -c "w-\[14px\]\|h-\[14px\]" web/src` sin icon-buttons <24px; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-04.md`
- **Estado:** ⬜ PENDING

### Task 6: WDA-05 — F5 Performance
- **Archivos clave:** `web/package.json`, `web/src/components/ui/`, `web/eslint.config.mjs`, `web/next.config.ts`
- **Gate Justificación:** 14 deps zombies + ui/ muerta + reactivar eslint (no-unused-vars, no-explicit-any) + turbopack.root
- **Contrato:** `npm ls --depth=0` en `web/` sin @dnd-kit/@hookform/@reactuses/@tanstack/date-fns/next-intl/react-markdown/uuid/z-ai-web-dev-sdk/zod/zustand; `npm run lint` 0 errores; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-05.md`
- **Estado:** ⬜ PENDING

### Task 7: WDA-06 — F6 Escritura
- **Archivos clave:** `web/src/components/vanta/code-playground.tsx`, `web/src/lib/dictionaries.ts`, `web/src/app/faq/page.tsx`
- **Gate Justificación:** textos literales EN sin i18n → dictionaries (code-playground, aria-labels navbar, ~12 literales); FAQ bilingüe para citabilidad LLM
- **Contrato:** `grep -c "press Run\|Main navigation\|click me · move mouse" web/src` = 0 (o traducidos vía tt()); FAQ tiene versión EN; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-06.md`
- **Estado:** ⬜ PENDING

### Task 8: WDA-07 — F7 Diseño comercial
- **Archivos clave:** `web/src/app/layout.tsx`, `web/src/app/page.tsx` (hero), `web/src/components/vanta/vanta-data.ts` (PRICING/TEAM)
- **Gate Justificación:** dominio canonical/OG real; value prop hero (PARA QUIÉN + CTA primario); pricing mid-tier decisión; social proof etiquetado; funnel demo
- **Contrato:** `metadataBase` = dominio real (vantadb.vercel.app o decidido); hero incluye público objetivo; `npm run build` pasa
- **Ruta:** vanta-worker
- **Task file:** `tasks/WDA-07.md`
- **Estado:** ⬜ PENDING

### Task 9: WDA-08 — F8 Triage + Reporte
- **Archivos clave:** `docs/reviews/web-design-audit-2026-08-19.md`, `web/AGENTS.md`, `web/AUDIT.md`
- **Gate Justificación:** consolidar hallazgos de WDA-00..07 en reporte con scores; actualizar AGENTS.md/AUDIT.md al cierre
- **Contrato:** existe `docs/reviews/web-design-audit-2026-08-19.md` con evidencia por dimensión; AGENTS.md/AUDIT.md reflejan estado real (32 rutas, dead code, config real); `npm run lint` 0/0 + `npm run build` sin warnings
- **Ruta:** vanta-lead
- **Task file:** `tasks/WDA-08.md`
- **Estado:** ⬜ PENDING