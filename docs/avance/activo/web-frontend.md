---
title: "Avance — Web Frontend"
type: domain-log
status: active
tags: [vantadb, avance, web, frontend, seo, docs-site]
last_reviewed: 2026-08-07
aliases: []
---

# Avance — Web Frontend

> Registro consolidado del trabajo en el frontend web (docs site, landing, cálculo de memoria) y su credibilidad (SEO/UX). IDs originales conservados.

## Frontend & Webapp

### WEB-01: ViteJs + React starter
- **Fecha:** 2026-07-01
- **Resultado:** ✅ Landing + MemoryScreen + EditorScreen, routing SPA. Monorepo `web/`.

### WEB-02: Sistema de plugins web
- **Fecha:** 2026-07-01
- **Resultado:** ✅ PluginRegistry, themes, ExtensionPoints, usePlugin hook, microbundle para npm publish.

### WEB-05: Cloud indexing para demo
- **Fecha:** 2026-07-11
- **Resultado:** ✅ Deploy en gh-pages (live). Imágenes/CPU alimentadas por llamada directa Node.js a `index.js` vía RPC; frontend struct URL a `data/api` (blind route) que se resuelve a "static/intersection-observer.md". Cloud indexing completo de 2.4M records con corpus.

### WEB-06: Reto: diseño responsive + mobile nav
- **Fecha:** 2026-07-11
- **Resultado:** ✅ Hereda token de color.

### WEB-11: Docs cultura open-source
- **Fecha:** 2026-07-14
- **Resultado:** ✅ 3 tabs (Fixes/Research/Feature), Community page, docs SEO (og tags), nav per-page.

### WEB-12: Docs site SEO
- **Fecha:** 2026-07-14
- **Resultado:** ✅ 1866 palabras fixes...; sitemap + robots.txt; LCP 1.1s.

### WEB-13: Analogías
- **Fecha:** 2026-07-14
- **Resultado:** ✅ Analogías explicativas.

### WEB-07: Frontend entropy — ReDoS risks (security audit)
- **Fecha:** 2026-07-11
- **Resultado:** ✅ — copiar desde **Web-07** (security audit) y retiró runtime para fomentar adoption.

### WEB-08: React HMR page guard
- **Fecha:** 2026-07-11
- **Resultado:** ✅ React fast-refresh preserves scroll? — guard `usePageVisibility` retry reset de scroll con `setTimeout(0)` en route change.

### WEB-09: Astro docs + rehype
- **Fecha:** 2026-07-14
- **Resultado:** ✅ SEO score >90, 43 pages, RSS. `rehype-pretty-code` build.

### WEB-10: i18n UI texts
- **Fecha:** 2026-07-14
- **Resultado:** ✅ en.py/en.js dict + text components.

### DOC-11 (multicolor reorg): two-column menu web
- **Resultado:** ✅ 14 sub-links (docs categories).

### DOC-12: Docs contributors section
- **Resultado:** ✅.

### DOC-16: Type "Page" outreach page src/pages/outreach.mdx
- **Resultado:** ✅ .

### DOC-21: GH Pages URL docs
- **Resultado:** ✅ .

### MKT-01 (frontenv): site package after GH pages
- **Resultado:** ✅.

---

## SEO / Peorance web

### seo-01: Sitemap XML + StructuredData
- **Resultado:** ✅.

### seo-02: OpenGraph/ Twitter cards
- **Resultado:** ✅ OG + TwitterCard según metadata.

### seo-03: hreflang
- **Resultado:** ✅ EN/ES.

### UX-01: Detect mobile vs desktop
- **Resultado:** ✅ (componente `useMediaQuery` custom).

### UX-02: SKIP linker
- **Resultado:** ✅ (componente QuickNav).

### UX-Х por lib.

---

## Deploy & credibilidad

### WEB-03 / WEB-04 (build infra)
- WEB-03: `c59e0f80` — 25/25 tests wal_sharded (WAL batching fsyncs). Se registra en core-engine.
- WEB-04: `21432104` — storage format versioning (`VantaHeader::validate_compat()`), revisa bindings.

### DOC-11 (web nav reorg)
- **Resultado:** ✅ Two-column menu con 12 sub-links (docs categories) — 2026-07-14.

### REVIEW-18: Warning Next.js package-lock stray (turbopack root)
- **Fecha:** 2026-08-23
- **Resultado:** ✅ `6ea5e545` — `turbopack: { root: __dirname }` en `web/next.config.ts`. `npm run build` sin warning (Next 16.3.0 Turbopack), exit 0, 35/35 páginas. Causa: lockfile stray en `C:\Users\Eros\` fuera del repo git. Fuente: review-full-20260822 H03-CODE-001.

> **Cruce:** el código frontend vive en `web/` + `docs/`; SEO/docs siguen reglas en `docs/avance/activo/operaciones.md` (docs site governance).
### FIND-23: `vanta-http-map.ts` DEFAULT_NS en ingest/get (2026-08-25)
- **Fecha:** 2026-08-25 — **Objetivo:** `desktop/src/vanta-http-map.ts:93` `namespace: item.namespace ?? ""` → `DEFAULT_NS`, alinear con mapping WASM (inconsistencia WEB-04), `IngestForm` vacío ya no rechaza — **Resultado:** ✅ + test — **Commit:** `ae03cc7d` (follow-up `b8e89585`/`f865ddde` cleanup, `480935a7` E2E-VISUAL)

## WDA-00..08 — Auditoría de diseño web completa (2026-08-24)

- **Fuente:** Plan `docs/plans/archive/2026-08-19-web-design-audit.md` · Reporte: `docs/reviews/web-design-audit-2026-08-24.md`
- **Resultado:** ✅ 8/8 tareas, 8 commits (`fbc404b5..a328ca64`) — baseline medible, dark-mode huérfano fuera (R-FE-4), not-found.tsx + sitemap 36 URLs + SITE_URL centralizada, 9 claims falsos corregidos, contrastes/touch-targets/ARIA, −7,615 líneas (13 deps zombies + ui/ muerta), i18n residual migrado, hero value prop + funnel demo honesto.
- **Pendiente:** Lighthouse post (EPERM ambiental), densidad de efectos home, pricing mid-tier (decisión producto), assets public/assets.

## ERR-WEB-01 — Toast por código de error + catch sin silenciar (2026-09-02)

- **Fuente:** Plan `docs/plans/2026-09-02-error-observability-excellence.md` Task 7 (Wave 3) · Task file `.opencode/skills/campaign-executor/tasks/ERR-WEB-01.md`
- **Resultado:** ✅ `toastError(error)` en `web/src/components/vanta/toast.tsx` mapea `error.code` → claves `errors.VANTADB_*` (10 ES + 10 EN en `dictionaries.ts`, códigos canónicos de vantadb-ts) con fallback `toast.error` + message en dev; duck-type porque la app no importa `vantadb` (WASM vive en iframe sandbox). `catch {}` de `code-playground.tsx:174` → `console.error` + `toastError` en el catch de `run()`; `copy-utils.ts` → `console.warn` con justificación de degradación intencional. Contrato: `catch {}`=0 · `VANTADB_`=22 · build exit 0 · lint 0 errors · Playwright 2 passed. **Commit:** `10cc9671`

## RES-12 — Touch targets ≥44px con hit-area invisible (2026-09-03)

- **Fuente:** Plan `docs/plans/2026-09-03-quality-gtm-wave.md` Task 7 (Wave 2) · WCAG 2.5.5 pre-Show-HN
- **Resultado:** ✅ 8 `<button>` de 28-36px en 5 componentes (`docs-view` ×2 copiar-código/comando, `tutorial-modal` copiar, `command-palette` cerrar, `shortcut-overlay` cerrar + trigger `?`, `site-navbar` buscar + hamburguesa) pasan a target ≥44px vía pseudo-elemento hit-area sin tocar el borde brutalista: `after:absolute after:-inset-2` (+4px laterales `after:-inset-1` en los h-9 → exactamente 44px todos); gap navbar `gap-1.5 sm:gap-2`→`gap-2.5` para mantener separación entre targets expandidos (WCAG 2.5.8). Cero cambios de layout (pseudo fuera de flujo — sin `-m-*` que colapse flex), focus ring intacto, verificado con screenshots 1440×900 before/after de /docs y home. Decorativos h-7 (badges span, ticker div) fuera de scope. Contrato: `rg -U '<button[^>]*h-(7|9)[^>]*' web/src -g '*.tsx' | rg -v "inset|p-2 -m-2|hit"` = 0 líneas · `npm run build` exit 0 (36 rutas app) · lint 0 errors · `npx playwright test` 2 passed (WEB-08/WEB-09) · sin harness unit web → guard E2E es la verificación. **Commit:** `539dfa41` (−38 líneas netas)

