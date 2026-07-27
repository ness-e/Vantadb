# Tech Stack Reference

## Framework Layer

| Layer | Choice | Version | Notes |
|---|---|---|---|
| Framework | Next.js | ^16.1.1 | App Router, standalone output |
| Runtime | React | ^19.0.0 | All pages "use client" |
| Language | TypeScript | ^5 | Strict mode (noImplicitAny: false) |
| Styling | Tailwind CSS | ^4 | CSS-based config via `@theme inline {}` |
| Package Manager | npm | — | Lockfile in package-lock.json |

## Core Dependencies (Active)

| Package | Purpose | Used In |
|---|---|---|
| framer-motion | Page transitions, section reveals | PageTransition, Reveal |
| animejs | Interactive SVG mark animations | mark-classic, mark-cta, use-mark-interaction |
| next-themes | Theme provider (installed, dark mode broken) | ThemeProvider (no dark CSS vars exist) |
| lucide-react | Icons | Navbar, sections, CTAs |
| sonner | Toast notifications | SiteShell (bottom-right) |
| class-variance-authority | Component variants (shadcn dep) | ui/ stubs |
| clsx + tailwind-merge | cn() utility | lib/utils.ts |
| tailwindcss-animate | Animation utilities (v3 plugin, redundant) | — |
| tw-animate-css | CSS-level animations (v4 native) | globals.css import |

## Config Files

| File | Purpose | Active? |
|---|---|---|
| next.config.ts | Standalone output, ignoreBuildErrors | ✅ Yes |
| tailwind.config.ts | v3 syntax — ignored by Tailwind v4 | ❌ Inert |
| components.json | shadcn/ui config (New York, neutral) | ✅ For CLI |
| postcss.config.mjs | @tailwindcss/postcss (single plugin) | ✅ Yes |
| tsconfig.json | Strict mode, path aliases @/* | ✅ Yes |
| Caddyfile | Reverse proxy :81 → localhost:3000 | ✅ Yes |

## Dead Dependencies (16 packages, 0 imports)

@dnd-kit/* (3), @mdxeditor/editor (~500KB), @reactuses/core, @tanstack/react-query, @tanstack/react-table, date-fns (~300KB), next-auth (~200KB), next-intl (~80KB), react-markdown, react-syntax-highlighter (~800KB), sharp (~30MB native), uuid, z-ai-web-dev-sdk, zustand.

## Zombie Dependencies (28 packages)

All @radix-ui/* primitives (only imported by dead ui/ stubs), recharts (only in ui/chart.tsx), react-hook-form (only in ui/form.tsx).

## shadcn/ui Consumption

30+ component stubs installed. Only 3 are actually consumed:

- toaster.tsx → SiteShell
- sonner.tsx → SiteShell
- toast.tsx → use-toast.ts hook

The remaining 27+ stubs (button, card, dialog, dropdown-menu, form, chart, etc.) compile but are never rendered.
