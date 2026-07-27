# Components Guide

## File Organization

- **`src/components/vanta/`** — Custom page components (44 active). Every component is `"use client"`.
- **`src/components/vanta/mark/`** — Animated SVG product mark (5 files).
- **`src/components/ui/`** — shadcn/ui primitives (30+ stubs, only 3 actually consumed: `toaster.tsx`, `sonner.tsx`, `toast.tsx`).

## Component Architecture

Most pages follow this pattern:

```
page.tsx (thin — Pattern B)
└── PageHeader (reusable hero)
    └── PageSection (reusable content wrapper)
        └── [section-specific components]
```

**PageHeader** — Hero with black panel background, neon accent line, rigid shadow below. Props: `title`, `description`, optional `badge`, `cta`.

**PageSection** — Content wrapper with 3 background variants: `cream` (default), `paper` (muted), `ink` (dark). Props: `variant`, `className`, `children`.

## Shell Structure

```
SiteShell
├── SiteNavbar     # Main navigation (460 lines) — marquee, 4 dropdowns, mobile menu, ⌘K
├── Footer          # Links + i18n (39 lines)
├── PageTransition  # AnimatePresence wrapper
├── ScrollProgress  # Reading progress bar
├── BackToTop       # Scroll FAB
└── 4 modals        # CommandPalette, ShortcutOverlay, EasterEgg, TutorialModal
```

## How to Create a New Page

1. Create `src/app/route-name/page.tsx` + `src/app/route-name/layout.tsx`
2. Add `"use client"` directive
3. Use Pattern B template unless specialized behavior needed
4. Import `useLanguage()` for i18n text
5. Compose PageHeader + PageSection + content components
6. Add dictionary keys to `src/lib/dictionaries.ts` (both ES and EN)

## How to Create a New Component

1. Create file in `src/components/vanta/` with `"use client"`
2. Import `useLanguage()` if text is needed
3. Use Tailwind v4 utility classes and `cn()` from `@/lib/utils`
4. Use CSS tokens: `bg-cream text-ink border-neon`
5. Add `data-` attributes for any test hooks
6. Export as default

## Component Conventions

- All functional components
- Default exports preferred
- Props typed with TypeScript interfaces (named `XxxProps`)
- Animation via CSS classes first, framer-motion for transitions, anime.js only for SVG mark
- i18n text via `t()` with `tt()` fallback pattern
