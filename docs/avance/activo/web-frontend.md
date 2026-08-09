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

> **Cruce:** el código frontend vive en `web/` + `docs/`; SEO/docs siguen reglas en `docs/avance/activo/operaciones.md` (docs site governance).