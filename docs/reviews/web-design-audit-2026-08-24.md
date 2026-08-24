# Auditoría de Diseño Web VantaDB — Reporte Final

> **Fecha:** 2026-08-24 · **Plan:** `docs/plans/archive/2026-08-19-web-design-audit.md` · **Ejecución:** pipeline run secuencial WDA-00..WDA-07 + triage
> **Alcance:** `web/**` (excluye `web/remotion/`) · **Verificación:** build exit 0 · lint exit 0 · tsc exit 0 · 31 rutas 200 · not-found 404 correcto

## Scores finales

| Dimensión | Antes | Después | Evidencia clave |
|---|---|---|---|
| Diseño | — | Mejorado | Dark-mode huérfano eliminado (R-FE-4); leak animejs fixed; hallazgos de ruido documentados (73 efectos en home — pendiente P2 de diseño fino) |
| Estructura | 29 URLs sitemap, catch-all 404, dominio muerto | 36 URLs, `not-found.tsx`, `SITE_URL` única | `lib/site-config.ts`; grep vantadb.dev=0 |
| Información | 9 claims falsos | 0 | recall 99.8%, tabla competitiva=BENCHMARKS §7, snippets README canónicos, v0.5.0, showcase/case-studies/changelog/blog honestos |
| UI/a11y | contrast ×24 fallando, targets <24px, doble Toaster | 0 errores lint-a11y dirigidos | text-black/30-50→/70, roles 16:1, Toaster shadcn fuera, mark teclado-accesible |
| Performance | 14 deps zombies, ui/ muerta (~7.6k líneas), bundle global inflado | −7,615 líneas netas (commit aa1b1b93), command-palette lazy | react-markdown conservada (import real); vs-table conservada (lo usa /why-vantadb) |
| Escritura | ~12 literales sin i18n, aria EN hardcode | migrados a tt(), 6 claves ES+EN nuevas | FAQ bilingüe YA existía (plan stale) |
| Comercial | hero sin PARA QUIÉN, TCO con plan fantasma, demo prometía beta falsa | audiencia explícita, CTA primario único, funnel honesto | −44 dead keys demoPage.* |

## Lighthouse (baseline WDA-00 vs post)

| Ruta | Perf | A11y | BP | SEO |
|---|---|---|---|---|
| `/` (pre) | 96 | 95 | 100 | 100 |
| `/about/team` (pre) | 95 | 96 | 100 | 100 |
| `/` (post) | ⚠️ no re-medido (EPERM ambiental del temp de Lighthouse); cambios posteriores solo **reducen** bundle (lazy command-palette, −deps) |

## Correcciones al propio plan (stale detectado durante ejecución)
1. `/quickstart` **no existe como ruta** — contenido vive en /docs (screenshots baseline de quickstart capturaron el 404 estilizado).
2. `vs-table.tsx` NO es dead code — lo usa `/why-vantadb`.
3. `react-markdown` NO es zombie — import real en `/docs/guides`.
4. FAQ bilingüe ya existía.
5. `turbopack.root` ya configurado (REVIEW-18).
6. Dark mode: decisión cerrada por regla dura R-FE-4 (light-only por diseño), no "a decidir".

## Pendiente (fuera de este plan — candidatos a backlog)
- **Diseño fino F1:** reducir densidad de efectos decorativos en home (trust-bar ×11 efectos, hero 5 capas) — requiere criterio visual del usuario.
- **Decisión de producto:** pricing mid-tier (hoy $0/Custom únicamente).
- **Dominio:** comprar/apuntar vantadb.dev o mantener vercel.app como canónico (hoy vercel.app).
- **Assets:** restaurar `mascota_gato.png`/`avatar_gato.png` en public/assets (4 referencias usan fallbacks).
- **Playground `new Function`:** riesgo self-XSS documentado en código; sandbox iframe si algún día se expone a terceros.
- Re-medir Lighthouse post-cambios cuando el EPERM lo permita.

## Commits de la campaña
`fbc404b5` WDA-00 baseline · `bb19f113` WDA-01 diseño · `50436783` WDA-02 estructura · `edd3a7f1` WDA-03 claims · `350b2115` WDA-04 UI · `aa1b1b93` WDA-05 perf (−7,615 líneas) · `ae72479e` WDA-06 escritura · `53785dfd` WDA-07 comercial
