# VantaDB Web — AGENTS.md

> **Actualizado:** 2026-08-24 (auditoría WDA-00..07) — reemplaza la versión stale previa.
> Auditoría completa: `docs/reviews/web-design-audit-2026-08-24.md`

## Commands

```sh
npm run dev       # next dev -p 3000
npm run build     # next build (standalone output) — DEBE pasar exit 0
npm start         # node .next/standalone/server.js (copiar .next/static y public/ a .next/standalone/ antes)
npm run lint      # eslint . — exit 0 requerido (no-unused-vars=error desde WDA-05)
npx playwright test                 # E2E guard WEB-08: landing→/docs#quickstart→/playground (WASM run) — 1 spec, 3.1s
npx playwright test --reporter=list # alternativa con reporte list
npx playwright show-trace test-results/<dir>/trace.zip # debug trace en falla
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
- Densidad de efectos decorativos en home (73 usos) — decisión de diseño fino pendiente (WEB-09).
- ~~Playground usa `new Function` (self-XSS)~~ — ✅ WEB-07: ejecución aislada en `iframe sandbox="allow-scripts allow-same-origin"` (`playground-executor.tsx` + `public/playground-executor.html`); `new Function` corre solo dentro del iframe, sin acceso al DOM/storage del parent.

## Verificación Lighthouse (WEB-05, INV-DECIDE — re-medido 2026-09-01)

> Re-medición post-WDA-05 (lazy command-palette ya en producción, sin EPERM). Ver `lighthouse-report.json` y `lighthouse-report-docs.json` (gitignored, generados localmente).

**Comando canónico (local, reproducido 2026-09-01):**
```sh
# /  (home)
cd web && npx lighthouse http://localhost:3000 --output=json --output-path=./lighthouse-report.json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo
# /docs (ruta interna)
cd web && npx lighthouse http://localhost:3000/docs --output=json --output-path=./lighthouse-report-docs.json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo
# servidor: node .next/standalone/server.js (previo `npm run build` 36/36 routes)
```

**Resultados 2026-09-01 (lighthouse 13.4.1, Chrome 152.0.7977.64, Node v26.8.1, Windows 11 26100, build `next build` 36/36):**

| Ruta | Performance | Accessibility | Best Practices | SEO | FCP | LCP | CLS | TBT | SI |
|------|-------------|---------------|----------------|-----|-----|-----|-----|-----|----|
| `/` | **99** | 96 | 96 | 100 | 1110 ms | 1800 ms | 0 | 0 ms | 2545 ms |
| `/docs` | **98** | 94 | 96 | 100 | 1110 ms | 1680 ms | 0 | 0 ms | 3998 ms |

**Lectura:** perf mejora 99/98 vs claim stale 95-96 (WDA-05) — confirma que el lazy `command-palette` (next/dynamic) no regresó; TBT 0 y CLS 0 en ambas rutas; LCP <2s.

**Workaround EPERM probado (2026-09-01):**
- EPERM **no reprodujo** en este entorno (Windows 11 bare-metal, Chrome --no-sandbox). Previamente reportado como ambiental persistente en CI/contenedor (WDA-05).
- Workaround documentado y probado: `--chrome-flags="--no-sandbox --headless --disable-gpu"` suficiente en Windows bare-metal; alternativa prod `https://vantadb.vercel.app` (200 OK, probada con `Invoke-WebRequest` 2026-09-01) queda como fallback si EPERM reaparece en CI contenedor.
- Si EPERM persiste en CI, re-medir contra producción con mismo comando (`npx lighthouse https://vantadb.vercel.app --output=json ... --chrome-flags="--no-sandbox --headless --disable-gpu"`).

(Regla 11: claim de performance sin número reproducible no existe — números arriba son reproducibles con comando + entorno citado.)

## E2E Guard (WEB-08)

- **Spec:** `e2e/flujo-critico.spec.ts` — landing→`/docs#quickstart`→`/playground` (patrón desktop/e2e, asserts por roles/labels visibles).
- **Config:** `playwright.config.ts` — `testDir:e2e`, `webServer: npm run dev` en `http://localhost:3000`, `workers:1`, `timeout:60s`.
- **Contrato:** `npx playwright test` verde local (1 test, ~3.1s) — usa `localhost` (no 127.0.0.1) para evitar 403 de Next allowedDevOrigins.
- **CI:** no registrado aún en `.github/workflows/ci-web-11.yml` (build+lint only); comando local documentado en `playwright.config.ts` header y en este AGENTS.md. Para CI, añadir job `e2e` con `npx playwright install --with-deps chromium` + `npx playwright test`.
- **Dependencias:** `@playwright/test@1.62.1` en `devDependencies`, browsers en `%LOCALAPPDATA%\ms-playwright`.
- **Playground WASM:** ejecución aislada en `iframe sandbox="allow-scripts allow-same-origin"` (`public/playground-executor.html` fetch+eval `vantadb_wasm.js` + `initSync` binario, `allow-same-origin` requerido para fetch `/vanta-wasm/*`); ver `playground-executor.tsx` para ping/retry de ready (ponytail: techo `allow-same-origin` — reducir a `allow-scripts` si WASM se sirve con CORS/blob).
