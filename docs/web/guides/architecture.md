# Architecture

## Project Structure

```
web/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── layout.tsx          # Root layout (server component)
│   │   ├── globals.css         # Tailwind v4 theme (817 lines)
│   │   ├── page.tsx            # Home (/) — Pattern A
│   │   ├── opengraph-image.tsx # Dynamic OG image
│   │   ├── robots.ts           # /robots.txt
│   │   ├── sitemap.ts          # /sitemap.xml (22 static + 4 blog + 3 case studies)
│   │   ├── [...slug]/          # Catch-all 404 (Pattern B)
│   │   ├── about/              # company, team, community, contact
│   │   ├── architecture/       # Pattern A
│   │   ├── benchmarks/         # Pattern A
│   │   ├── blog/               # + [slug]
│   │   ├── case-studies/       # + [slug]
│   │   ├── changelog/          # Pattern C — ChangelogSection
│   │   ├── config/             # Pattern B
│   │   ├── cost/               # Pattern B
│   │   ├── demo/               # Server redirect → /playground
│   │   ├── docs/               # Pattern A
│   │   ├── docs-api/           # Server redirect → /docs
│   │   ├── engine/             # Pattern C — CoreEngine + WalSimulator
│   │   ├── integrations/       # Pattern B
│   │   ├── latency/            # Pattern B
│   │   ├── maint/              # Pattern B
│   │   ├── playground/         # Pattern C — CodePlayground
│   │   ├── pricing/            # Pattern B
│   │   ├── security/           # Pattern B
│   │   ├── showcase/           # Pattern B
│   │   ├── solutions/          # ai-agents, ai-ide-tooling, local-rag
│   │   ├── storage/            # Pattern B
│   │   ├── use-cases/          # Pattern B
│   │   └── why-vantadb/        # (may not exist)
│   ├── components/
│   │   ├── ui/                 # shadcn/ui primitives (30+ stubs, only 3 consumed)
│   │   └── vanta/              # Custom page components (44 active + 5 dead)
│   ├── hooks/                  # 8 custom hooks
│   └── lib/                    # utils, language-provider, dictionaries
├── public/                     # logo.svg, manifest.json, storage-illustration.webp
├── next.config.ts
├── components.json
├── package.json
├── Caddyfile
├── AUDIT.md
├── AGENTS.md
```

## Data Flow

- Static data flows from `vanta-data.ts` → page components → children
- i18n flows from `LanguageProvider` → `useLanguage()` hook → 48 consumers
- No backend API calls, no data fetching
- Images served from external URLs (remote patterns in `next.config`)

## Architectural Decisions

- **All pages are `"use client"`** — no React Server Components used for rendering
- **Custom i18n** instead of next-intl (next-intl is installed but unused)
- **Each page has its own layout.tsx** — 29 nested + 1 root = 30 total
- **No route groups** — flat route structure
- **No middleware** — no locale negotiation, redirects, or auth
- **No error/loading/not-found boundaries** — errors bubble to default Next.js overlay
- **Tailwind v4 with CSS-based config** — `tailwind.config.ts` is inert
- **Build ignores all TS errors** (`ignoreBuildErrors: true`)
- **Strict mode disabled** (`reactStrictMode: false`)

## Component Tree (Shell)

```
RootLayout (layout.tsx — server)
└── LanguageProvider
    └── SiteShell
        ├── ScrollProgress
        ├── SiteNavbar
        │   ├── LogoMark
        │   ├── LangToggle
        │   ├── Navigation groups (dropdowns)
        │   ├── Marquee strip
        │   └── CmdK trigger
        ├── main > {children} (page content)
        ├── Footer
        └── 4 modals: CommandPalette, ShortcutOverlay, EasterEgg, TutorialModal
    ├── Toaster (shadcn)
    └── Sonner (notifications)
```
