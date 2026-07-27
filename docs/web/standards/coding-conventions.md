# Coding Conventions — VantaDB Web Frontend

## File Organization

- **Page files** go in `src/app/<route-name>/page.tsx` — each file is a thin "use client" wrapper that imports a single view component from `src/components/vanta/`
- **Layouts** — root layout at `src/app/layout.tsx`, route-specific metadata layouts at `src/app/<route-name>/layout.tsx` (currently 21 route layouts)
- **Components** go in `src/components/vanta/` — one file per component
- **Hooks** go in `src/hooks/` — one file per hook
- **Utilities** go in `src/lib/` (e.g. `utils.ts`, `dictionaries.ts`, `language-provider.tsx`)
- **shadcn/ui primitives** stay in `src/components/ui/` — do not edit these manually

## Imports

- Use `@/` path alias (e.g. `@/components/vanta/Hero`, `@/lib/utils`)
- Group in this order: React → Next.js → third-party → local
- No barrel files (`index.ts`) — import directly from the source file (exception: `mark/index.tsx` which re-exports `Mark`, `MarkClassic`, and `MARK_VARIANTS`)

## Component Rules

- **All components are `"use client"`** — add the directive as the first line of every file. No React Server Components are used anywhere
- Use TypeScript interfaces for props: `interface XxxProps { ... }`
- **Default exports** for page components, **named exports** for utility components
- Use `cn()` from `@/lib/utils` for conditional Tailwind class merging
- Use `useLanguage()` for any user-facing text — never hardcode strings
- Use `tt(key, fallback)` pattern for defensive fallback (duplicate inline until extracted to dictionaries.ts):
  ```ts
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  ```

## CSS Rules

- Use **Tailwind v4** utility classes in JSX — theme is defined in `globals.css` via `@theme inline {}`
- `tailwind.config.ts` is **inert** — all theme values live in `@theme inline`
- Add custom CSS in `globals.css` within `@layer utilities`
- Use design tokens: `bg-cream`, `text-ink`, `border-neon`, `text-neon`, `bg-paper`, `bg-smoke`, `text-cream`, `border-cream`
- **No dark mode overrides** — not implemented
- Use shadcn/ui semantic tokens for primitives: `bg-background`, `text-foreground`, `border-border`
- Rigid border style: `border-4 border-black` with `shadow-[6px_6px_0_0_#000]` for card-like elements

## Animation Rules

- **Page entrances**: `animate-rise` class on root `<div>` of each page
- **Scroll reveals**: `<Reveal direction="up" delay={n}>` wrapping sections (uses framer-motion)
- **Count-up numbers**: `<CountUpStat value="1.2ms" />` for animated metrics
- **Hover effects**: CSS utility classes (`.glitch-hover`, `.neon-underline`, `.btn-neon-glow`, `.press`, `.press-lg`)
- **Complex SVG animations**: anime.js in `mark/` components only (MarkClassic, MarkCta, useMarkInteraction)
- **Always respect `prefers-reduced-motion`** — every animation hook checks `window.matchMedia("(prefers-reduced-motion: reduce)")` and short-circuits
- Page transitions: `<PageTransition viewKey={pathname}>` wrapping children in layout

## i18n Rules

- Add EVERY user-facing string to `src/lib/dictionaries.ts` (both ES and EN, ~880 keys)
- Use namespace-dot-key convention: `section.camelCaseName`
- Use single braces `{param}` for interpolation — never double braces `{{param}}`
- Never delete a dictionary key that might be referenced — `grep` first
- The `next-intl` package is installed but **unused** — the custom `LanguageProvider` context is the active system

## Routing Patterns

- **Pattern B (standard)**: Page imports one view component + optional extras (e.g., `/benchmarks` imports `BenchmarksView` + `BenchmarkRace`). Layout exports route-specific `metadata`
- **Catch-all**: `[...slug]/page.tsx` handles 404 with suggested routes grid
- Navigation: `useVantaNavigate()` bridges legacy `onNavigate(view: View)` → `router.push()`

## Performance

- No external state library — `useState`/`useEffect` + `LanguageProvider` context is sufficient
- No data fetching — all content is static from `vanta-data.ts`
- Fonts: Geist (body via `--font-geist-sans`), Anton (display via `--font-anton`), Space Mono (tech via `--font-space-mono`)
