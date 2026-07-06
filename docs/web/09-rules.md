---
title: Rules & Conventions
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [rules, conventions, naming, accessibility, responsive]
---

# Rules & Conventions

```
[ VANTADB ] >> rules.enforced
────────────────────────────────────────────────────
  SCOPE: All web surfaces
────────────────────────────────────────────────────
```

## Naming Conventions

### CSS Classes

All VantaDB design system classes use the `nb-` prefix.

```
nb-card          — component name
nb-card--amber   — variant modifier (double dash)
nb-card-frame    — compound name (no modifier)
nb-grid--cols-12 — parameter modifier
```

### React Components

Components use PascalCase with `Nb` prefix.

```
NbCard
NbButton
NbBento
NbAccordion
NbSectionHeader
```

### Files

Component files match their export name.

```
NbCard.tsx
NbButton.tsx
NbAccordion.tsx
```

### CSS Custom Properties

Use `--` prefix with kebab-case scoped names.

```
--amber
--space-md
--ease-brutal
--border-visible
--text-on-amber
```

## Accessibility Standards

### Contrast

| Requirement | Ratio | Applies To |
|-------------|-------|------------|
| WCAG AA Normal | 4.5:1 | Body text, captions, labels |
| WCAG AA Large | 3:1 | Headings, display text (18px+ / 14px bold+) |
| WCAG AAA Normal | 7:1 | Preferred for body text |

### Keyboard Navigation

- All interactive elements reachable via Tab
- Focus indicators: 3px solid amber outline, 3px offset
- Skip-to-content link present on every page
- No keyboard traps

### Screen Readers

- Semantic HTML (`<nav>`, `<main>`, `<section>`, `<article>`)
- ARIA labels where semantic HTML is insufficient
- Alt text on all images

### Reduced Motion

- `prefers-reduced-motion: reduce` must disable all animations
- GSAP animations check `useReducedMotion()` before initializing
- No content depends on animation to be understood

## Responsive Rules

### Breakpoints

| Name | Width | Target |
|------|-------|--------|
| Default | 0+ | Mobile |
| `sm` | 640px | Large phone |
| `md` | 768px | Tablet |
| `lg` | 1024px | Desktop |
| `xl` | 1280px | Wide desktop |
| `2xl` | 1536px | Ultrawide |

### Mobile First

All layouts are mobile-first. Multi-column grids collapse to single
column below `md` (768px).

### Content

- Touch targets: minimum 44x44px
- No horizontal overflow on any breakpoint
- Hero must render without scroll at 1024px+
- Navigation single line at 1024px+

## Code Quality

### CSS

- Use `@theme` for Tailwind v4 tokens
- Use `@layer base` for global resets
- No `!important` except in reduced-motion overrides
- Always use CSS custom properties from the token system
- Never hardcode values that have a token equivalent

### React/TypeScript

- Server Components by default (Next.js)
- Client Components only where interactivity is required
- Use `motion/react` for animation components
- Use `useMotionValue` / `useTransform` for continuous values
- Never `useState` for scroll position, mouse position, or physics

### Icons

- One icon library per project. Default: Lucide React.
- Never hand-roll SVG icons
- Standardized `strokeWidth` across all icons (default 2)

## Composition Rules

### Section Repetition

No layout pattern may be used more than once per page. If a 3-column
card grid appears for "features", it cannot appear again for "solutions."
Each section must use a different composition.

### Zigzag Alternation

Left-image/right-text alternating with right-image/left-text patterns
may appear at most 2 times consecutively. On the 3rd occurrence,
change to full-width, bento, or other layout.

### Eyebrow Restraint

Small uppercase labels above section headers: maximum 1 per 3 sections.
Hero counts as 1. Most sections should have no eyebrow at all.

### Bento Cell Count

A bento grid has exactly as many cells as there are content items.
No empty cells. No filler cells.

### Copy Self-Audit

Before shipping, re-read every visible string:
- No grammatical errors
- No unclear referents
- No AI-hallucinated metaphors
- No fake-precise numbers (92%, 4.1x, 48k — unless real data)
- One copy register per page (don't mix technical, editorial, marketing)

## Image Asset Rules

### Priority Order

1. Generate section-specific images via image generation tools
2. Use real photography sources
3. Leave clearly labeled placeholder slots

### Banned

- Div-based fake screenshots
- Hand-rolled SVG illustrations as default
- Plain text wordmarks for logo walls (use SVG or real logos)

### Logo Wall Rules

- Logos only — no industry labels below logos
- Must render in both light and dark mode
- Real SVG logo marks for real companies
- Generate inline SVG monograms for invented brand names

---

```
[ END RULES ]
>> next: 10-product.md
```
