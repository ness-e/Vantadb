# Tech Stack — VantaDB Web Frontend

## Framework

| Layer | Choice | Version | Notes |
|---|---|---|---|
| Framework | Next.js | ^16.1.1 | App Router, standalone output |
| Runtime | React | ^19.0.0 | All pages `"use client"` |
| Language | TypeScript | ^5 | Strict mode |
| Styling | Tailwind CSS | ^4 | CSS-based config via `@theme inline {}` |
| UI Library | shadcn/ui | — | New York style, neutral base, lucide icons |
| Package Manager | npm | — | Lockfile in `package-lock.json` |

## Core Dependencies (Active)

| Package | Purpose |
|---|---|
| `framer-motion` | Page transitions (`PageTransition`), section reveals |
| `animejs` | SVG/Canvas interactive graph animation (`mark-classic.tsx`) |
| `next-themes` | Theme provider (installed, dark mode broken — no dark CSS vars) |
| `lucide-react` | Icons |
| `sonner` | Toast notifications (bottom-right) |
| `class-variance-authority` | Component variant system (shadcn/ui dep) |
| `clsx` + `tailwind-merge` | `cn()` utility |
| `tailwindcss-animate` | Animation utilities for Tailwind |
| `tw-animate-css` | CSS-level Tailwind animation utilities |
| `react-hook-form` | Form handling (unused — 0 imports) |
| `recharts` | Charts (`benchmark-race.tsx`) |
| `z-ai-web-dev-sdk` | AI SDK (unused) |
| `zustand` | State management (installed, unused) |

## shadcn/ui Components Installed (`src/components/ui/`)

All standard shadcn/ui primitives from `components.json` (New York style, neutral base, RSC enabled):
accordion, alert-dialog, aspect-ratio, avatar, button, card, checkbox, collapsible, command,
context-menu, dialog, dropdown-menu, hover-card, input, label, menubar, navigation-menu,
popover, progress, radio-group, scroll-area, select, separator, slider, sonner, switch, tabs,
textarea, toast, toggle, toggle-group, tooltip.

## Styling Architecture

- **Tailwind v4**: Theme defined in `globals.css` via `@theme inline {}` — NOT in `tailwind.config.ts` (that file is inert v3 syntax).
- **`tailwind.config.ts`**: Ignored by Tailwind v4. Colors defined as `hsl(var(--...))` which resolve to hex values but have no effect.
- **shadcn/ui CSS variables**: Defined in `:root` block in `globals.css` as hex values (not HSL).
- **Custom utility classes**: ~700 lines in `@layer utilities` — press, paper-bg, halftone, speed-lines, grid-tech, shadow-brutal, glitch-hover, scanlines, etc.

## Fonts

| Font | Variable | Usage | Weight |
|---|---|---|---|
| Geist Sans | `--font-geist-sans` | Body text | Variable |
| Geist Mono | `--font-geist-mono` | Code/mono | Variable |
| Anton | `--font-anton` | Display/headlines | 400 |
| Space Mono | `--font-space-mono` | Tech labels, stencil | 400, 700 |

Tailwind aliases: `font-sans` → Geist, `font-mono` → Geist Mono, `font-display` → Anton, `font-tech` → Space Mono.

## Palette

| Token | Value | Usage |
|---|---|---|
| `cream` | `#FBF9F5` | Page background, card backgrounds |
| `ink` | `#000000` | Text, borders, rigid shadows |
| `neon` | `#FF5500` | Accent — CTAs, highlights, active states (95/5 rule) |
| `paper` | `#F2EDE2` | Secondary surfaces, muted sections |
| `smoke` | `#1A1A1A` | Dark variant elements (near-black, not pure ink) |

## i18n

Custom `LanguageProvider` context (NOT `next-intl`):
- Handles ES/EN via `useLanguage()` → `{ t, lang, setLang }`
- ~880 keys in `src/lib/dictionaries.ts`
- Persists to `localStorage("vantadb-lang")`, auto-detects browser language
- Some components duplicate a local `tt(key, fallback)` helper (3 copies)

## Known Discrepancies

| Package | Status |
|---|---|
| `next-intl` | Installed but unused (custom i18n instead) |
| `@dnd-kit/*` | Installed, 0 imports |
| `@tanstack/react-query` | Installed, 0 imports |
| `@tanstack/react-table` | Installed, 0 imports |
| `@mdxeditor/editor` | Installed, 0 imports (~500KB) |
| `next-auth` | Installed, 0 imports (~200KB) |
| `date-fns` | Installed, 0 imports (~300KB) |
| `react-markdown` | Installed, 0 imports |
| `react-syntax-highlighter` | Installed, 0 imports (~800KB) |
| `sharp` | Installed, 0 imports (~30MB native) |
| `uuid` | Installed, 0 imports |
| `zustand` | Installed, 0 imports |
| `taillwind.config.ts` | Inert — v3 syntax in v4 project |

17 dead dependencies total. See `docs/web/QA.md` or `web/AUDIT.md`.
