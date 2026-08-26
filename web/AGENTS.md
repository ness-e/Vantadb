# VantaDB Web — AGENTS.md

> **Actualizado:** 2026-08-24 (auditoría WDA-00..07) — reemplaza la versión stale previa.
> Auditoría completa: `docs/reviews/web-design-audit-2026-08-24.md`

## Commands

```sh
npm run dev       # next dev -p 3000
npm run build     # next build (standalone output) — DEBE pasar exit 0
npm start         # node .next/standalone/server.js (copiar .next/static y public/ a .next/standalone/ antes)
npm run lint      # eslint . — exit 0 requerido (no-unused-vars=error desde WDA-05)
```

## Config (verificado)

- `next.config.ts`: `output: "standalone"` + `reactStrictMode: true` + `turbopack.root` (REVIEW-18). NO hay `ignoreBuildErrors`.
- Tailwind v4 CSS-first: tema en `src/app/globals.css` (`@theme inline`); `tailwind.config.ts` inerte.
- Path alias `@/*` → `./src/*`
- `lib/site-config.ts` exporta `SITE_URL` (= `https://vantadb.vercel.app`) — única fuente para sitemap/metadataBase/canonicals. Si se compra otro dominio, cambiar solo ahí.

## Architecture

- App Router, **32 rutas** (31 estáticas + docs-api redirect). `/quickstart` NO existe — el quickstart vive en `/docs`.
- Casi todas `"use client"`; layouts server para metadata. SiteShell (`site-shell.tsx`) envuelve todo desde root layout.
- `not-found.tsx` propio (estilizado, hereda shell); el catch-all `[...slug]` solo llama `notFound()`.
- Contenido estático en `src/components/vanta/vanta-data.ts`. Claims numéricos DEBEN salir de `docs/operations/BENCHMARKS.md` y README del repo (Regla 11).
- i18n custom: `LanguageProvider` + `dictionaries.ts` (~1.2k claves ES/EN simétricas). Helper compartido `createTt` en `lib/i18n-utils.ts`; componentes usan `const { t, tt } = useLanguage()`.

## Reglas vigentes (`.opencode/rules/frontend-web.md`)

- **R-FE-4:** light-only por diseño — NUNCA re-introducir dark mode / next-themes.
- **R-FE-2:** cero dependencias sin imports verificados.
- **R-FE-5:** touch targets ≥44px táctil (mínimo absoluto 24px inline secundario).
- **R-FE-6:** motion tokens en globals.css, no easings hardcodeados.

## Dead code eliminado (2026-08-24, WDA-05)

Borrados y NO restaurar: `components/ui/*` (46 wrappers shadcn, queda solo `sonner.tsx`), `hooks/use-toast.ts`, `vanta/navbar.tsx`, `ecosystem.tsx`, `metrics-bar.tsx`, 13 deps zombies (lista en tasks/WDA-05.md). **OJO:** `vs-table.tsx` y `react-markdown` PARECEN muertos pero NO lo son (usados por /why-vantadb y /docs/guides).

## Animaciones

- animejs solo en mark components — cleanup obligatorio en unmount + `prefers-reduced-motion` (fixed en WDA-01; no revertir).
- Timers (setTimeout/setInterval) siempre con clearTimeout/clearInterval en unmount.
- command-palette cargado lazy vía next/dynamic (arrastra vanta-data fuera del bundle global) — no volver a import estático.

## Conocido-pendiente (backlog candidatos)

- Assets faltantes en `public/assets/` (mascota_gato.png, avatar_gato.png) — 4 referencias usan fallbacks.
- Densidad de efectos decorativos en home (73 usos) — decisión de diseño fino pendiente.
- Playground usa `new Function` (self-XSS documentado en código; sandbox iframe solo si se expone a terceros).

## Verificación Lighthouse (WEB-05, INV-DECIDE)

Claim previo "perf 95-96" (WDA-05) quedó sin medición fresca tras lazy command-palette (EPERM ambiental persistente en CI).
Comando para re-medir localmente:
```sh
cd web && npx lighthouse http://localhost:3000 --output=json --output-path=./lighthouse-report.json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo
```
Workaround EPERM: correr `lighthouse` contra producción (`https://vantadb.vercel.app`) o en máquina sin contenedor. Si EPERM persiste, documentar resultado y entorno en `docs/operations/BENCHMARKS.md` (Regla 11: claim de performance sin número reproducible no existe).
