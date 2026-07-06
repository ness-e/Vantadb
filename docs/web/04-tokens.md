---
title: Design Tokens
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [tokens, colors, typography, spacing, shadows]
---

# Design Tokens

```
[ VANTADB ] >> tokens.reference
────────────────────────────────────────────────────
  SOURCE:  web/src/styles/tokens.css
  FORMAT:  CSS custom properties
────────────────────────────────────────────────────
```

All values below are defined in `tokens.css` and consumed throughout
the component system. Never hardcode raw values. Always use the token.

## Color System

### Core Palette

| Token | Value | Usage |
|-------|-------|-------|
| `--background` | `#0a0a0a` | Page background, dark substrate |
| `--foreground` | `#ffffff` | Primary text, primary element color |
| `--surface` | `#111111` | Card and component surfaces |
| `--surface-alt` | `#282828` | Elevated surfaces, hover states |
| `--surface-raised` | `#1a1a1a` | Interactive raised surfaces |

### Amber (Primary Accent)

| Token | Value | Usage |
|-------|-------|-------|
| `--amber` | `#ff5500` | Primary accent, CTAs, highlights |
| `--amber-light` | `#ff7733` | Amber hover state |
| `--amber-dim` | `rgba(255,85,0,0.12)` | Subtle amber backgrounds |
| `--amber-glow` | `rgba(255,85,0,0.3)` | Amber glow effects |
| `--text-on-amber` | `#0a0a0a` | Text on amber backgrounds |

### Neutrals

| Token | Value | Usage |
|-------|-------|-------|
| `--steel` | `oklch(53% 0 0)` | Secondary text, metadata |
| `--frost` | `oklch(60% 0 0)` | Tertiary text, muted elements |
| `--muted` | `oklch(63% 0 0)` | Body text on dark, descriptions |
| `--subtle` | `oklch(48% 0 0)` | Subtle text, captions |

### Status

| Token | Value | Usage |
|-------|-------|-------|
| `--success` | `#00c853` | Positive states, benchmarks |
| `--danger` | `#ff1744` | Errors, destructive actions |

### Terminal

| Token | Value | Usage |
|-------|-------|-------|
| `--terminal-bg` | `#080808` | Code block backgrounds |
| `--terminal-text` | `#e0e0e0` | Terminal/console text |

### Border

| Token | Value | Usage |
|-------|-------|-------|
| `--border` | `rgba(255,255,255,0.12)` | Default hairline borders |
| `--border-hover` | `rgba(255,255,255,0.25)` | Border hover state |
| `--border-strong` | `oklch(38% 0 0)` | Strong structural borders |
| `--border-visible` | `rgba(255,255,255,0.2)` | Visible component borders |
| `--border-faint` | `rgba(255,255,255,0.15)` | Subtle borders |

## Typography

### Font Stack

| Role | Font | Variable | Weight |
|------|------|----------|--------|
| Display | Space Grotesk | `--font-display` | 400-700 |
| Body | Outfit | `--font-sans` | 400-700 |
| Code/Data | JetBrains Mono | `--font-mono` | 400-700 |

### Type Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--text-micro` | `0.6rem` | Tiny labels, legal |
| `--text-label` | `0.72rem` | Mono labels, metadata |
| `--text-sm` | `0.875rem` | Small text, captions |
| `--text-body` | `1.05rem` | Paragraph body text |
| `--text-code` | `0.88rem` | Inline code, code blocks |
| `--text-lead` | `1.2rem` | Lead paragraphs |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | Section titles |
| `--text-display` | `clamp(2.2rem, 4vw, 3.5rem)` | Section headings |
| `--text-hero` | `clamp(3.5rem, 7vw, 6.5rem)` | Hero headlines |
| `--text-metric` | `clamp(2.5rem, 5vw, 4rem)` | Data metrics |

### Weights

| Token | Value |
|-------|-------|
| `--weight-regular` | `400` |
| `--weight-medium` | `500` |
| `--weight-semibold` | `600` |
| `--weight-bold` | `700` |

### Tracking (Letter-Spacing)

| Token | Value | Usage |
|-------|-------|-------|
| `--tracking-tight` | `-0.05em` | Hero headlines |
| `--tracking-display` | `-0.04em` | Display text |
| `--tracking-wide` | `0.14em` | Mono labels, uppercase |
| `--tracking-mega` | `-0.06em` | Large numeric markers |

### Leading (Line-Height)

| Token | Value |
|-------|-------|
| `--leading-tight` | `0.95` |
| `--leading-snug` | `1.05` |
| `--leading-normal` | `1.65` |

## Spacing

Based on a 4px unit scale. All spacing uses these tokens.

| Token | Value | Rem |
|-------|-------|-----|
| `--space-3xs` | `0.25rem` | 4px |
| `--space-2xs` | `0.5rem` | 8px |
| `--space-xs` | `0.75rem` | 12px |
| `--space-sm` | `1rem` | 16px |
| `--space-md` | `1.5rem` | 24px |
| `--space-lg` | `2rem` | 32px |
| `--space-xl` | `3rem` | 48px |
| `--space-2xl` | `4rem` | 64px |
| `--space-3xl` | `6rem` | 96px |
| `--space-4xl` | `8rem` | 128px |

### Section Gaps

| Token | Value |
|-------|-------|
| `--section-gap` | `96px` |
| `--section-gap-lg` | `160px` |

## Border Radius

**All radii are 0px.** No exceptions.

```
--radius-sm: 0px
--radius-md: 0px
--radius-lg: 0px
--radius-xl: 0px
```

## Shadow System

Offset shadows with zero blur and full opacity. The offset distance
creates the illusion of physical depth.

| Token | Value | Usage |
|-------|-------|-------|
| `--shadow-sm` | `2px 2px 0 0 #fff` | Card hover, button rest |
| `--shadow-md` | `4px 4px 0 0 #fff` | Default button, elevated cards |
| `--shadow-lg` | `6px 6px 0 0 #ff5500` | Featured elements, amber shadow |
| `--shadow-amber` | `4px 4px 0 0 #ff7733` | Amber interactive states |
| `--shadow-amber-lg` | `8px 8px 0 0 #ff5500` | Hero cards, featured CTAs |
| `--shadow-none` | `none` | No shadow |

## Z-Index Scale

| Token | Value | Context |
|-------|-------|---------|
| `--z-base` | `1` | Base stacking |
| `--z-dropdown` | `100` | Dropdown menus |
| `--z-sticky` | `200` | Sticky nav |
| `--z-overlay` | `300` | Overlays, backdrops |
| `--z-modal` | `400` | Modal dialogs |
| `--z-toast` | `500` | Toast notifications |
| `--z-noise` | `9999` | Noise overlay |

## Easing Curves

| Token | Curve | Usage |
|-------|-------|-------|
| `--ease-swiss` | `cubic-bezier(0.25, 1, 0.5, 1)` | Standard UI transitions |
| `--ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Exit animations |
| `--ease-in-out` | `cubic-bezier(0.77, 0, 0.175, 1)` | Attention animations |
| `--ease-brutal` | `cubic-bezier(0.05, 0.95, 0.3, 1)` | Neubrutalist snaps |
| `--ease-cut` | `cubic-bezier(0.25, 0, 0.6, 1)` | Sharp cuts |

## Grid

| Token | Value |
|-------|-------|
| `--grid-cols` | `12` |
| `--grid-gap` | `0px` |
| `--grid-max` | `1200px` |

## Navigation

| Token | Value |
|-------|-------|
| `--nav-height` | `64px` |

---

```
[ END TOKENS ]
>> next: 05-grid.md
```
