# Task WDA-02 — F2 Estructura

**Plan:** docs/plans/2026-08-19-web-design-audit.md §6 Task 3 · **Ruta:** vanta-worker · **Estado:** ⏳ IN PROGRESS

## Objetivo
not-found.tsx migración + sitemap +7 rutas + dominio real en URLs base + extraer tt() ×36 a lib.

## Impacto mapeado (Regla 0)
- `web/src/app/[...slug]/page.tsx` — catch-all actual (404 estilizado) → se reemplaza por `notFound()`
- `web/src/app/not-found.tsx` — NUEVO
- `web/src/app/sitemap.ts` — 29 URLs → 36
- `web/src/app/layout.tsx` — metadataBase (~línea 36)
- `web/src/lib/i18n-utils.ts` (o lib existente) — helper tt único
- ~36 archivos con tt() duplicado (21 páginas + 15 componentes)
- Veredicto: mecánico, sin cambios de comportamiento público salvo 404 vía notFound()

## Steps

### S1 — not-found.tsx ✅ DONE
- Portar diseño del catch-all → `web/src/app/not-found.tsx`; catch-all llama `notFound()`.
- Docs verificadas: https://nextjs.org/docs/app/api-reference/functions/not-found (Next 16.3.2) — notFound() lanza NEXT_HTTP_ERROR_FALLBACK;404, nearest not-found boundary lo captura, root app/not-found.tsx se envuelve en layout raíz (SiteShell preservado) e inyecta noindex automático.

### S2 — Sitemap +7 rutas ✅ DONE
- Agregadas /demo /showcase /config /storage /latency /integrations /docs-api → 36 URLs totales.

### S3 — Dominio real ✅ DONE
- Creada `web/src/lib/site-config.ts` con `SITE_URL = "https://vantadb.vercel.app"` (única fuente).
- layout.tsx metadataBase + OG url → SITE_URL (antes apuntaban al repo GitHub); sitemap.ts y robots.ts importan SITE_URL.
- ~30 canonicals/OG por-ruta → vantadb.vercel.app vía replace masivo.
- Email muerto `maintainers@vantadb.dev` (dominio sin MX) → GitHub Security Advisories (canal real), label "Email"→"Security".

### S4 — Extraer tt() ×36 a lib ✅ DONE
- `web/src/lib/i18n-utils.ts`: `createTt(t)` (lógica única).
- `language-provider.tsx` expone `tt` en el contexto (default SSR: fallback directo).
- 36 archivos: definición local borrada, destructure `const { t, tt } = useLanguage()`.
- Incidente S4: primer script con `-replace x y 1` (sintaxis inválida) vació 35 archivos → restaurados vía git checkout y rehecho con .Replace() seguro. Sin pérdidas (verificado).

## Contrato verificable
- [x] Existe `web/src/app/not-found.tsx` (diseño estilizado porteado); catch-all llama `notFound()`; build genera /_not-found prerenderizada — `npm run build` exit 0
- [x] `rg -c "vantadb.dev" web/src` = **0**
- [x] sitemap = **36 URLs** (29 estáticas incl. las 7 nuevas + 4 blog + 3 casos)
- [x] `rg "const tt = \(key: string" web/src -g "*.tsx"` = **0 hits**; lógica única en `lib/i18n-utils.ts`; 36 archivos usan `const { t, tt } = useLanguage()`
- [x] `npm run build` exit 0 (Next 16.3.0 Turbopack, TypeScript OK, 36 rutas)

## Estado final: ✅ COMPLETADO (sin commit — instrucción explícita)

## Context Save Point
- S1-S4 completos y verificados con build doble (post-S4 y post-reparación contacto).
- Archivos tocados: not-found.tsx (nuevo), [...slug]/page.tsx, sitemap.ts, robots.ts, layout.tsx, lib/site-config.ts (nuevo), lib/i18n-utils.ts (nuevo), lib/language-provider.tsx, about/contact/page.tsx, competitive-table.tsx + ~30 layouts (dominio) + 35 archivos (tt).
- Deuda menor documentada: canonicals por-ruta quedan hardcodeados a vercel.app (no importan SITE_URL) — reversión futura requiere replace global o refactor de layouts (notado en site-config.ts).
- Lecciones registradas en memory/lessons.md (wildcard PowerShell + patrón restore-after-script).

## Reglas duras
NO tocar web/remotion/, desktop/. Sin git commit. NO modificar el plan file.
Incidente: al inicio escribí por error el task file encima del catch-all → restaurado vía git checkout (sin daño).

## Context Save Point
(inicio — sin trabajo previo)
