# VantaDB Web Frontend

**Stack:** Next.js 16 (App Router) · React 19 · TypeScript · Tailwind CSS v4 · shadcn/ui (New York) · Framer Motion 12 · Anime.js 4

**Status:** Active development — 30 `page.tsx` routes + 3 special server files, 44 active components, 817 lines of custom CSS design system.

## Quick Start

```sh
npm install
npm run dev       # next dev -p 3000
npm run build     # next build (standalone output)
npm start         # node .next/standalone/server.js
```

Build ignores TypeScript errors (`ignoreBuildErrors: true`) and runs with `reactStrictMode: false`.

## Docs Index

| Document | Purpose |
|---|---|
| `README.md` | This file — overview, quick start, doc index |
| `guides/architecture.md` | Project structure, data flow, key architectural decisions |
| `guides/routing.md` | 3 page patterns, server redirects, layouts, catch-all 404 |
| `guides/components.md` | How to create and modify components |
| `guides/i18n.md` | Custom LanguageProvider, dictionaries, translation workflow |
| `guides/animation.md` | Animation conventions — framer-motion, anime.js, CSS |
| `reference/stack.md` | Tech stack with versions and rationale |
| `reference/design-system.md` | Design tokens, palette, typography, CSS utilities |
| `reference/component-catalog.md` | Every vanta/ component with props and usage |
| `reference/hooks.md` | All 8 custom hooks with signatures |
| `reference/deps-audit.md` | Dependency audit — dead, zombie, and active packages |
| `standards/coding-conventions.md` | Naming, imports, file organization |
| `standards/review-checklist.md` | Pre-merge quality checklist |

## History

The site was rebuilt from an earlier SPA prototype (archived in `docs/web_old/`). Current codebase uses Next.js 16 App Router with client-side rendering everywhere and a manga/linocut inspired aesthetic — rigid shadows, neon accent, ink textures. Focused on the VantaDB engine's hybrid search (BM25 + HNSW via RRF), WAL durability, and in-process architecture.

## Key Files

| File | Role |
|---|---|
| `src/app/layout.tsx` | Root layout — fonts, metadata, LanguageProvider, SiteShell |
| `src/app/globals.css` | Tailwind v4 theme — palette, effects, animations (~817 lines) |
| `src/app/page.tsx` | Home route (`/`) |
| `src/components/vanta/vanta-data.ts` | Centralized static content (~1109 lines) |
| `src/components/vanta/site-shell.tsx` | Shared layout wrapper — navbar, footer, modals |
| `src/lib/language-provider.tsx` | Custom i18n context (ES/EN, not next-intl) |
| `src/lib/dictionaries.ts` | ~1228 ES/EN i18n keys per language |
| `src/hooks/use-vanta-navigate.ts` | Legacy View → `router.push()` adapter |
| `next.config.ts` | Standalone output, ignoreBuildErrors (12 lines) |
| `Caddyfile` | Reverse proxy :81 → localhost:3000 |
| `AUDIT.md` | 24-item quality audit (auto-generated) |
