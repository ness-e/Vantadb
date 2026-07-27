# Architecture — VantaDB Web Frontend

## Directory Layout

```
web/
├── src/
│   ├── app/                          # Next.js App Router (28+ route groups)
│   │   ├── layout.tsx                # Root layout — fonts, metadata, providers
│   │   ├── globals.css               # Tailwind v4 theme + manga design system
│   │   ├── page.tsx                  # Home (/)
│   │   ├── opengraph-image.tsx       # OG image
│   │   ├── robots.ts                 # Robots.txt
│   │   ├── sitemap.ts                # Sitemap.xml
│   │   ├── [...slug]/page.tsx        # Catch-all 404 (non-standard, no notFound())
│   │   ├── about/                    # Company, team, community, contact
│   │   ├── architecture/             # Retrieval pipeline + search semantics
│   │   ├── benchmarks/               # BENCH-01 + SIFT1M + BenchmarkRace
│   │   ├── blog/                     # Blog index + [slug] posts
│   │   ├── case-studies/             # Case study index + [slug]
│   │   ├── changelog/                # Changelog page
│   │   ├── config/                   # Configuration
│   │   ├── cost/                     # Pricing calculator / cost
│   │   ├── demo/                     # Interactive demo
│   │   ├── docs/                     # Documentation
│   │   ├── docs-api/                 # API docs
│   │   ├── engine/                   # Core engine deep-dive
│   │   ├── integrations/             # Ecosystem integrations
│   │   ├── latency/                  # Latency explorer
│   │   ├── maint/                    # Maintenance
│   │   ├── playground/               # Code playground
│   │   ├── pricing/                  # Pricing page
│   │   ├── security/                 # Security page
│   │   ├── showcase/                 # Showcase
│   │   ├── solutions/                # AI agents, local RAG, IDE tooling
│   │   ├── storage/                  # Storage backends
│   │   ├── use-cases/                # Use cases
│   │   └── why-vantadb/              # Why VantaDB
│   ├── components/
│   │   ├── ui/                       # shadcn/ui primitives (30+ components)
│   │   └── vanta/                    # Custom page components (47 files)
│   ├── hooks/                        # Custom hooks (8 files)
│   └── lib/                          # Utilities, i18n, data (3 files)
├── public/                           # Static assets
├── next.config.ts                    # Standalone output, ignoreBuildErrors
├── components.json                   # shadcn/ui configuration
├── tailwind.config.ts                # INERT — v3 syntax, no effect
├── Caddyfile                         # :81 → localhost:3000 reverse proxy
├── AGENTS.md                         # Agent instructions
├── AUDIT.md                          # 24-item quality audit
└── package.json
```

## Routing

All pages are `"use client"` — no React Server Components. Every `page.tsx` follows the pattern:

```tsx
"use client";
import { ViewComponent } from "@/components/vanta/view-component";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

export default function Page() {
  const navigate = useVantaNavigate();
  return <ViewComponent onNavigate={navigate} />;
}
```

### Route Tiers

| Tier | Routes | Status |
|---|---|---|
| F1 | `/`, `/benchmarks`, `/docs` | Live since prototype |
| F2 | `/engine`, `/architecture`, `/playground`, `/why-vantadb`, `/changelog`, `/pricing`, `/security`, `/use-cases`, `/cost`, `/maint`, `/solutions/*` | Live (F2 expansion) |
| F4 | `/blog`, `/blog/[slug]`, `/case-studies`, `/case-studies/[slug]`, `/about/*` | Live (blog/case study stubs) |
| Catch-all | `[...slug]/page.tsx` | Acts as 404 (non-standard) |

### Navigation Bridge

`/hooks/use-vanta-navigate.ts` bridges the legacy `onNavigate(view: View)` interface to Next.js App Router:

```tsx
// Legacy interface: onNavigate("home" | "benchmarks" | "docs")
// Adapter: View → router.push("/" | "/benchmarks" | "/docs")
```

`isLiveRoute(path)` controls whether nav links navigate or show "coming soon" toast.

## Component Tree (Shell)

```
RootLayout (layout.tsx)
├── LanguageProvider
│   ├── SiteShell
│   │   ├── ScrollProgress
│   │   ├── SiteNavbar
│   │   │   ├── LogoMark
│   │   │   ├── LangToggle
│   │   │   ├── Navigation groups (dropdowns)
│   │   │   ├── Marquee strip
│   │   │   └── CommandPalette trigger (⌘K)
│   │   ├── main
│   │   │   └── PageTransition (framer-motion AnimatePresence)
│   │   │       └── children (page content)
│   │   ├── Footer
│   │   ├── BackToTop
│   │   ├── CommandPalette (⌘K modal)
│   │   ├── ShortcutOverlay (? modal)
│   │   └── EasterEgg (type "vanta")
│   ├── Toaster (shadcn)
│   └── Sonner (bottom-right)
```

## Data Flow

- **All content is static** — no API calls, no data fetching, no server components
- **Centralized content store**: `src/components/vanta/vanta-data.ts` (~1109 lines)
  - `VANTA` — project metadata (repo, version, license)
  - `PRODUCT` — metrics, versions, ecosystem, distribution
  - `HERO_STATS`, `CORE_CAPABILITIES`, navigation arrays, FAQ data, blog posts
- **i18n**: separate `src/lib/dictionaries.ts` (~880 keys) consumed via `useLanguage().t(key)`
- **No external state library** — component-local `useState`/`useEffect` + `LanguageProvider` context

## Key Config Quirks

| Config | Value | Impact |
|---|---|---|
| `next.config.ts` → `ignoreBuildErrors` | `true` | TS errors won't block build |
| `next.config.ts` → `reactStrictMode` | `false` | No double-render in dev |
| `next.config.ts` → `output` | `"standalone"` | Self-contained build |
| `globals.css` | `@theme inline {}` | Tailwind v4 theme (not tailwind.config.ts) |
| `<html lang="es">` | Hardcoded | Always Spanish — ignores LanguageProvider |
| `components.json` → `rsc` | `true` | But no RSC used anywhere |
| Turbopack root | Wrong lockfile | Picks `~` package-lock instead of project |
| `metadataBase` | Not set | OG images use localhost URL |

## i18n Architecture

Custom implementation (not next-intl):

- `LanguageProvider` wraps the app, reads `localStorage("vantadb-lang")` on mount
- Falls back to browser `navigator.language` auto-detection
- `useLanguage()` → `{ t: (key, params?) => string, lang, setLang }`
- Supports template params: `t("hero.install", { version: "0.1.2" })`
- Default language: Spanish (`"es"`)

## Known Dead Code

4 components (1,157 lines) are imported by nothing:
- `src/components/vanta/navbar.tsx` (546 lines) — replaced by `site-navbar.tsx`
- `src/components/vanta/hero-mark-interactive.tsx` (312 lines) — replaced by `mark-classic.tsx`
- `src/components/vanta/ecosystem.tsx` (168 lines) — removed from HomeView
- `src/components/vanta/metrics-bar.tsx` (131 lines) — removed from HomeView

See `QA.md` or `web/AUDIT.md` for full dead code accounting.
