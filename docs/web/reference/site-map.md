# Site Map — Frontend Web VantaDB

> **Propósito:** Mapeo completo ruta → componente → datos del frontend web.
> **Framework:** Next.js 16 App Router.
> **Convención:** Todas las `page.tsx` son `"use client"` excepto redirect pages.

---

## 1. Visión general

El frontend expone **30 rutas `page.tsx`**, **2 redirect pages** (server components), y **1 catch-all `[...slug]`** para 404.

Tres patrones de página dominan:

- **Componente standalone importado** (HomeView, BenchmarksView, DocsView, CoreEngine, ChangelogSection, CodePlayground, Architecture) — páginas que usan componentes extraídos en `@/components/vanta/`.
- **PageHeader + PageSection inline con `vanta-data.ts`** — la mayoría de las páginas secundarias siguen esta receta: importan `PageHeader` y `PageSection`, consumen un slice de `vanta-data.ts` y renderizan cards/grids/tablas.
- **PageHeader + PageSection inline con datos locales** — /latency, /storage, /integrations, /showcase definen su propio array de datos en la misma `page.tsx`.

Todas las páginas usan `useLanguage()` + helper `tt(key, fallback)` para i18n local. El hook `useVantaNavigate()` se usa en HomeView, BenchmarksView y DocsView para navegación programática.

---

## 2. Tabla completa ruta → componente → datos

| Ruta | Componente(s) | Datos (`vanta-data.ts`) | Layout metadata | Notas |
|------|--------------|------------------------|----------------|-------|
| `/` | `HomeView` (11 secciones) | `HERO_STATS`, `CORE_CAPABILITIES`, `BENCH01`, `SEARCH_SEMANTICS`, `FAQ`, `TUTORIALS`, `QUICKSTART_PYTHON` | metadata raíz | `useVantaNavigate` |
| `/benchmarks` | `BenchmarksView` + `BenchmarkRace` | `BENCH01`, `SIFT1M`, `PRODUCT.metrics` | "Benchmarks · VantaDB" | `useVantaNavigate` |
| `/docs` | `DocsView` | `DOC_LINKS`, `CLI_COMMANDS`, `QUICKSTART_PYTHON`, `PRODUCT_BOUNDARY`, `FAQ` | "Quickstart · VantaDB" | `useVantaNavigate` |
| `/engine` | `CoreEngine` + `WalSimulator` | `PRODUCT.techStack` | "Core Engine · VantaDB" | h1 oculto sr-only |
| `/architecture` | `Architecture` + `SearchSemantics` | `SEARCH_SEMANTICS`, `PRODUCT.metrics` | "Architecture · VantaDB" | h1 oculto sr-only |
| `/playground` | `CodePlayground` | `QUICKSTART_PYTHON`, `CLI_COMMANDS` | "Code Playground · VantaDB" | h1 oculto sr-only |
| `/changelog` | `ChangelogSection` | `CHANGELOG` | "Changelog · VantaDB" | h1 oculto sr-only |
| `/why-vantadb` | `PageHeader` + `PageSection` + `VsTable` | `WHY_VANTADB` | "Why VantaDB" | 4 benefit cards + vs table |
| `/pricing` | `PageHeader` + `PageSection` | `PRICING_PLANS` | "Pricing · VantaDB" | `TAG_STYLES` local, highlight card |
| `/security` | `PageHeader` + `PageSection` | `SECURITY_PILLARS` | "Security · VantaDB" | 6 pillars |
| `/use-cases` | `PageHeader` + `PageSection` | `USE_CASES_DETAIL` | "Use Cases · VantaDB" | 3 cards grid |
| `/cost` | `PageHeader` + `PageSection` | `TCO_COMPARISON`, `VANTA` | "Cost · VantaDB" | 3 stat cards + tabla |
| `/maint` | `PageHeader` + `PageSection` | `MAINTENANCE_PILLARS`, `VANTA` | "Maintenance · VantaDB" | 4 pillars |
| `/config` | `PageHeader` + `PageSection` | `VANTA` | "Zero Configuration · VantaDB" | `ICONS` map local, terminal flow |
| `/latency` | `PageHeader` + `PageSection` | datos locales (`rows` array) | "Sub-Millisecond Latency · VantaDB" | 6-row table + CSS bar chart |
| `/storage` | `PageHeader` + `PageSection` | `VANTA` | "Single-Binary Storage · VantaDB" | 4 arch layers + components grid |
| `/integrations` | `PageHeader` + `PageSection` | datos locales (`integrations` array) | "Ecosystem & Integrations · VantaDB" | 9 integration cards |
| `/showcase` | `PageHeader` + `PageSection` | datos locales (`items` array) | "Community Showcase · VantaDB" | 6 project cards |
| `/blog` | `PageHeader` + `PageSection` | `BLOG_POSTS` | "Blog · VantaDB" | `TAG_STYLES` local, card grid |
| `/blog/[slug]` | `PageHeader` + `PageSection` | `BLOG_POSTS.findIndex` | generateMetadata() dinámico | 404 inline, next suggested post |
| `/case-studies` | `PageHeader` + `PageSection` | `CASE_STUDIES` | "Case Studies · VantaDB" | 3-card grid |
| `/case-studies/[slug]` | `PageHeader` + `PageSection` | `CASE_STUDIES.findIndex` | generateMetadata() dinámico | 404 inline |
| `/solutions/ai-agents` | `PageHeader` + `PageSection` | `USE_CASES_DETAIL[0]`, `VANTA` | "AI Agents · VantaDB" | pain/solution/flow/metrics/code |
| `/solutions/local-rag` | `PageHeader` + `PageSection` | `USE_CASES_DETAIL[1]`, `VANTA` | "Local RAG · VantaDB" | mismo patrón |
| `/solutions/ai-ide-tooling` | `PageHeader` + `PageSection` | `USE_CASES_DETAIL[2]`, `VANTA` | "IDE Tooling · VantaDB" | mismo patrón |
| `/about/company` | `PageHeader` + `PageSection` | `COMPANY_INFO` | "Company · VantaDB" | stats + principles |
| `/about/team` | `PageHeader` + `PageSection` | `TEAM_MEMBERS`, `VANTA` | "Team · VantaDB" | 4 member cards |
| `/about/community` | `PageHeader` + `PageSection` | `VANTA` | "Community · VantaDB" | 4-step guide |
| `/about/contact` | `PageHeader` + `PageSection` | `VANTA` | "Contact · VantaDB" | 4 channels |
| `/demo` | — (redirect) | — | "Browser Demo · VantaDB" | server component, `redirect()` |
| `/docs-api` | — (redirect) | — | "VantaDB API Docs → /docs" | server component, `redirect()` |
| `/[...slug]` | `CatchAllNotFoundPage` | `SUGGESTED` (local) | "Not Found · VantaDB", noindex | 404 con 5 suggested links |

---

## 3. Layouts

Tres tipos de layout:

**Root layout** (`src/app/layout.tsx`):
- Providers: `LanguageProvider` + `SiteShell` + `Toaster` (shadcn) + `Sonner`
- Fuentes: Geist (body), Anton (display), Space Mono (tech)
- HTML `lang="es"` hardcodeado
- Metadata global con OG y Twitter cards
- Viewport: `themeColor: "#FF5500"`

**Sub-layouts passthrough** (27 layouts): exportan `metadata` estática y devuelven `{children}`. Sin providers adicionales. Ejemplo: `/benchmarks/layout.tsx`, `/pricing/layout.tsx`, `/engine/layout.tsx`.

**Sub-layouts dinámicos** (2 layouts):
- `blog/[slug]/layout.tsx`: `generateMetadata()` con `BLOG_POSTS.find()` — si no encuentra slug, devuelve 404 metadata
- `case-studies/[slug]/layout.tsx`: mismo patrón con `CASE_STUDIES`
- `[...slug]/layout.tsx`: metadata 404 estática con `noindex`, `canonical: /404`

Los layouts dinámicos son server components (no tienen `"use client"`).

---

## 4. Archivos especiales

| Archivo | Ruta generada | Runtime | Propósito |
|---------|--------------|---------|-----------|
| `opengraph-image.tsx` | `/opengraph-image.png` (1200×630) | nodejs | OG image dinámica con mascota |
| `sitemap.ts` | `/sitemap.xml` | — | 29 entradas: 22 estáticas + 4 blog posts + 3 case studies |
| `robots.ts` | `/robots.txt` | — | Allow `/`, disallow `/api/` y `/_next/` |
| `api/route.ts` | `/api` | — | Placeholder, devuelve `{ message: "Hello, world!" }` |

`sitemap.ts` incluye rutas dinámicas hardcodeadas (4 blog slugs, 3 case study slugs). No hay generación dinámica desde `vanta-data.ts`.

---

## 5. Páginas redirect

Dos server components (sin `"use client"`) que ejecutan `redirect()` de Next.js:

| Ruta | Destino | Layout metadata |
|------|---------|----------------|
| `/demo` | `/playground` | "Browser Demo · VantaDB" |
| `/docs-api` | `/docs` | "VantaDB API Docs → /docs" |

Ambos son páginas de transición desde la versión anterior del diseño (diseño2). Se mantienen para compatibilidad de enlaces externos y SEO.

---

## 6. Catch-all 404 (`/[...slug]`)

`[...slug]/page.tsx` captura cualquier ruta no definida. Renderiza:

- `PageHeader` con badge "§404" y título "Not Found" (traducible)
- Botones "Back to Home" y "Go back"
- Grid de 5 suggested links: Home, Docs, Benchmarks, Pricing, Blog
- Nota para reportar URLs rotas en GitHub

El layout asociado (`[...slug]/layout.tsx`) setea `robots: { index: false, follow: false }` y `canonical: /404`.

---

## 7. Páginas faltantes y anomalías

Tras inspeccionar el sistema de archivos, todas las rutas declaradas en el sitemap tienen su `page.tsx` y `layout.tsx` correspondientes. No hay páginas faltantes.

/`solutions/local-rag/` sí existe con su `page.tsx` y `layout.tsx`. La página sigue exactamente el mismo patrón que `/solutions/ai-agents/` y `/solutions/ai-ide-tooling/`.

---

## 8. Convenciones

1. **`"use client"`** en todas las páginas excepto redirects y layouts dinámicos.
2. **Import path:** `@/components/vanta/page-header` para `PageHeader` + `PageSection`.
3. **i18n local:** `const { t } = useLanguage()` + helper `tt(key, fallback)`.
4. **Animación:** `<div className="animate-rise">` wrapping el contenido, `<Reveal direction="up" delay={n}>` por sección.
5. **Navegación programática:** `useVantaNavigate()` → `router.push()` en las 3 páginas principales.
6. **Datos estáticos:** importados de `@/components/vanta/vanta-data.ts` o definidos inline en la page.
7. **No data fetching:** cero llamadas fetch/async — todo es contenido estático de `vanta-data.ts`.
8. **Hidden h1:** en /engine, /architecture, /playground, /changelog usan `<h1 className="sr-only">` para SEO.
