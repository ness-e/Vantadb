# VantaDB Web Frontend

**Stack:** Next.js 16 (App Router) · React 19 · TypeScript · Tailwind CSS v4 · shadcn/ui (New York) · Framer Motion 12 · Anime.js 4

**Status:** Active development — 30+ real routes, 47 custom components, 817 lines of CSS design system.

## Purpose

Landing/marketing site for [VantaDB](https://github.com/ness-e/Vantadb).
Showcases the engine's hybrid search (BM25 + HNSW via RRF), WAL durability, and in-process architecture.
All pages are static — no backend data fetching.

## Quick Start

```sh
npm install     # install dependencies
npm run dev     # next dev -p 3000
npm run build   # next build (standalone output)
npm start       # node .next/standalone/server.js
```

Build ignores TypeScript errors (`ignoreBuildErrors: true`) and runs with `reactStrictMode: false`.

## Docs Index

| Document | Purpose |
|---|---|
| `README.md` | This file — overview, quick start, index |
| `STACK.md` | Full tech stack — packages, versions, rationale |
| `ARCHITECTURE.md` | Project structure, routing, component tree, data flow |
| `DESIGN.md` | Design system — palette, typography, effects, CSS utilities |
| `COMPONENTS.md` | Component catalog — every file in `src/components/vanta/` |
| `ANIMATION.md` | Animation conventions — framer-motion, anime.js, CSS animations |
| `QA.md` | Quality audit — known issues, dead code, unused deps, todos |

## History

This frontend was rebuilt from an earlier SPA prototype (Swiss + Neubrutalism design docs archived in `docs/web_old/`).
The active codebase uses Next.js 16 App Router with client-side rendering (`"use client"` everywhere) and a
manga/linocut inspired aesthetic — rigid shadows, neon accents, ink textures.

## Key Files

| File | Role |
|---|---|
| `src/app/layout.tsx` | Root layout — fonts, metadata, providers, SiteShell |
| `src/app/globals.css` | Tailwind v4 theme — palette, effects, animations (~817 lines) |
| `src/app/page.tsx` | Home route (`/`) |
| `src/components/vanta/vanta-data.ts` | Centralized static content store (~1109 lines) |
| `src/components/vanta/site-shell.tsx` | Shared layout wrapper — navbar, footer, modals |
| `src/lib/language-provider.tsx` | Custom i18n context (ES/EN, not next-intl) |
| `src/lib/dictionaries.ts` | ~880 ES/EN i18n keys |
| `src/hooks/use-vanta-navigate.ts` | Legacy `View` → `router.push()` adapter |
| `next.config.ts` | Standalone output, build config (12 lines) |
| `Caddyfile` | Reverse proxy :81 → localhost:3000 |
| `AUDIT.md` | 24-item quality audit (auto-generated) |
| `AGENTS.md` | Agent instructions for this project |
