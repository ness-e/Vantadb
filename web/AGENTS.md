# VantaDB Web — AGENTS.md

## Commands

```sh
npm run dev       # next dev -p 3000
npm run build     # next build (standalone output)
npm start         # node .next/standalone/server.js
npm run lint      # eslint . (very permissive — 34 rules disabled)
```

No test infrastructure exists. No CI/CD.

## Config quirks

- `next.config.ts` — `ignoreBuildErrors: true` (TS errors won't block build), `reactStrictMode: false`
- Tailwind v4: theme defined in `src/app/globals.css` via `@theme inline {}`; `tailwind.config.ts` is **inert**
- shadcn/ui: New York style, neutral base, lucide icons, RSC enabled in `components.json`
- Path alias `@/*` → `./src/*`

## Architecture

- **All pages are `"use client"`** — no React Server Components used anywhere
- Every `page.tsx` imports a single named component from `src/components/vanta/`
- No data fetching — all content is static from `src/components/vanta/vanta-data.ts` (1109 lines)
- No external state library in use; component-local `useState`/`useEffect` + `LanguageProvider` context
- Fonts: Geist (body via `--font-geist-sans`), Anton (display via `--font-anton`), Space Mono (tech via `--font-space-mono`)

## i18n (custom, not next-intl)

Uses `LanguageProvider` context + `useLanguage()` hook returning `{ t, lang, setLang }`. Dictionary in `src/lib/dictionaries.ts` (~880 ES/EN keys). The `next-intl` package is installed but unused.

Many components use a local `tt(key, fallback)` helper — duplicate pattern that should be extracted:
```ts
const tt = (key: string, fallback: string) => {
  const v = t(key);
  return v === key ? fallback : v;
};
```

## Navigation quirks

- `src/components/vanta/vanta-data.ts` still exports a legacy `View` type (`"home" | "benchmarks" | "docs"`) from the old SPA
- `useVantaNavigate()` bridges `onNavigate(view: View)` → `router.push()`
- `isLiveRoute()` in `use-vanta-navigate.ts` controls which nav links navigate vs show "coming soon" toast

## Animation conventions

- Page transitions: `<PageTransition viewKey={pathname}>` wrapping children in layout (framer-motion AnimatePresence)
- Scroll reveal: `<Reveal direction="up" delay={n}>` wrapping sections
- Count-up: `<CountUpStat value={N} suffix="vec/s" />` for benchmark numbers
- SVG/Canvas: `animejs` used in `mark-classic.tsx` for the interactive graph

## Synthetic events for cross-component communication

- `⌘K` → `window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", metaKey: true }))` → CommandPalette
- `?` → same pattern → ShortcutOverlay
- Typing sequence "vanta" → EasterEgg (global keydown listener)

## Known dead code (documented in AUDIT.md)

- `src/components/vanta/navbar.tsx` (546 lines, replaced by site-navbar.tsx)
- `src/components/vanta/hero-mark-interactive.tsx`
- `src/components/vanta/ecosystem.tsx`
- `src/components/vanta/metrics-bar.tsx`

## Design system (from globals.css)

- Colors: cream `#FBF9F5`, ink `#000000`, neon `#FF5500`, paper `#F2EDE2`, smoke `#1A1A1A`
- Borders: `border-4 border-black` with rigid shadows `shadow-[6px_6px_0_0_#000]`
- Effects: press/press-lg/glow-neon/glitch-hover/scanlines/halftone/speed-lines/grid-tech etc.
- Theme switching via `next-themes` with `attribute="class"` (light default)

## Notable

- Two toast systems both rendered in layout: `Toaster` (shadcn) + `Sonner` (positioned bottom-right)
- `tailwindcss-animate` + `tw-animate-css` both installed (animation utilities)
- Caddyfile: reverse proxy `:81` → `localhost:3000`
- `.env` exists but is empty (0 bytes)
- 17 dead dependencies documented in AUDIT.md — installed packages with zero imports in source
