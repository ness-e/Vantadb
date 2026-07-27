# Design Token System — VantaDB

> Versión: 2.0 | 2026-07 | Estilo: Swiss + Neubrutalism
> Modo único: Dark

---

## 1. Color Tokens

### 1.1 Core Palette

| Token | Value | OKLCH | Usage |
|---|---|---|---|
| `--black` | `#000000` | `oklch(0 0 0)` | Extreme contrast, text on light |
| `--white` | `#ffffff` | `oklch(1 0 0)` | Text on dark, backgrounds |
| `--amber` | `#ff5500` | `oklch(0.62 0.22 40)` | Primary accent — CTAs, highlights, selected states |
| `--amber-light` | `#ff7733` | `oklch(0.68 0.22 45)` | Hover state for amber elements |
| `--amber-dim` | `rgba(255, 85, 0, 0.12)` | — | Subtle amber tint for hover states |
| `--amber-glow` | `rgba(255, 85, 0, 0.3)` | — | Amber shadow tint |
| `--success` | `#00c853` | — | Positive indicators, benchmarks |
| `--danger` | `#ff1744` | — | Negative indicators |

### 1.2 Surface Tokens

| Token | Value | Usage |
|---|---|---|
| `--background` | `#111111` | Primary background |
| `--foreground` | `#f5f5f5` | Primary text |
| `--surface` | `#1a1a1a` | Card/section surfaces |
| `--surface-alt` | `#222222` | Hover state for surfaces |
| `--surface-glass` | `rgba(17, 17, 17, 0.85)` | Nav background + backdrop-blur |
| `--terminal-bg` | `#0d0d0d` | Terminal/code block background |
| `--surface-card-dark` | `rgba(255, 255, 255, 0.03)` | Card resting on dark |
| `--surface-card-dark-hover` | `rgba(255, 255, 255, 0.06)` | Card hover on dark |
| `--text-on-amber` | `#111111` | Text on amber backgrounds |

### 1.3 Text & Border Tokens

| Token | Value | Usage |
|---|---|---|
| `--muted` | `#808080` | Secondary text, metadata |
| `--steel` | `oklch(45% 0 0)` | Labels, metadata, inactive states |
| `--subtle` | `#5a5a5a` | Subtle text, captions |
| `--frost` | `oklch(55% 0 0)` | Light metadata |
| `--border` | `rgba(255, 255, 255, 0.08)` | Subtle borders |
| `--border-hover` | `rgba(255, 255, 255, 0.2)` | Hover state borders |
| `--border-strong` | `#333333` | Strong borders |
| `--border-visible` | `rgba(255, 255, 255, 0.15)` | Visible 2px neubrutalist borders |

### 1.4 Block Tokens (Dark Sections)

| Token | Value | Usage |
|---|---|---|
| `--block-dark-bg` | `#111111` | Dark section backgrounds |
| `--block-dark-text` | `#f5f5f5` | Text on dark sections |
| `--block-dark-muted` | `#808080` | Muted text on dark sections |
| `--block-dark-border` | `rgba(255, 255, 255, 0.08)` | Borders on dark sections |
| `--block-dark-amber` | `rgba(255, 85, 0, 0.12)` | Amber tint on dark sections |

### 1.5 Grid Tokens

| Token | Value | Usage |
|---|---|---|
| `--grid-cols` | `12` | Base grid column count |
| `--grid-gap` | `0px` | Grid column gap |
| `--grid-hairline` | `rgba(255, 255, 255, 0.06)` | Subtle grid lines |
| `--grid-visible` | `rgba(255, 255, 255, 0.12)` | Visible grid lines |
| `--grid-max` | `1200px` | Max content width |

### 1.6 Usage Rules

1. **Amber is the single accent color.** No secondary accent.
2. **95/5 Rule**: 95% of the page is monochrome. 5% is amber.
3. Amber used for: CTAs, hover states, active nav, data highlights, code strings.
4. Amber never used for: body text, secondary text, background fills, decorative elements.

---

## 2. Shadow Tokens (Neubrutalism — Hard Offset)

### 2.1 Core Shadows

| Token | Value | Usage |
|---|---|---|
| `--shadow-sm` | `4px 4px 0px 0px #000000` | Cards, subtle elevation |
| `--shadow-md` | `6px 6px 0px 0px #000000` | Default elevation, buttons |
| `--shadow-lg` | `8px 8px 0px 0px #000000` | Modals, emphasis |
| `--shadow-amber` | `4px 4px 0px 0px var(--amber)` | Amber-tinted hard shadow |
| `--shadow-brutal` | `8px 8px 0px 0px #111111` | Neubrutalism signature shadow |
| `--shadow-brutal-hover` | `2px 2px 0px 0px #111111` | Button hover (shadow reduces) |
| `--shadow-glow` | `0 0 20px rgba(255, 85, 0, 0.15)` | Rare amber glow for hero only |
| `--shadow-none` | `none` | Explicit no-shadow |

### 2.2 Shadow Rules

- ALL shadows must be hard offset: `Xpx Ypx 0px 0px color` — zero blur, zero spread
- `box-shadow: none` is prohibited in normal component states (use `--shadow-none` explicitly if needed)
- Button hover: shadow reduces (`--shadow-md` → `--shadow-brutal-hover`)
- Button active: no shadow + translate(6px, 6px) — mechanical press
- Card hover: shadow upgrades to `--shadow-amber`

---

## 3. Typography Tokens

### 3.1 Font Families

| Token | Value | Usage |
|---|---|---|
| `--font-sans` | `'Outfit', ui-sans-serif, system-ui, sans-serif` | Body text |
| `--font-display` | `'Space Grotesk', ui-sans-serif, sans-serif` | Headings, display |
| `--font-mono` | `'JetBrains Mono', ui-monospace, monospace` | Code, labels, data |

### 3.2 Font Size Scale

| Token | Value | Mobile | Desktop | Usage |
|---|---|---|---|---|
| `--text-micro` | `0.6rem` | 9.6px | 9.6px | Legal, timestamps |
| `--text-label` | `0.72rem` | 11.5px | 11.5px | ALL CAPS labels, tag text |
| `--text-sm` | `0.875rem` | 14px | 14px | Small text, captions |
| `--text-body` | `1.05rem` | 16.8px | 16.8px | Body copy, descriptions |
| `--text-code` | `0.88rem` | 14px | 14px | Code, terminal |
| `--text-lead` | `1.2rem` | 19.2px | 19.2px | Lead paragraphs |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | 20.8px | 27.2px | Card titles, feature names |
| `--text-display` | `clamp(2.2rem, 4vw, 3.5rem)` | 35.2px | 56px | Section titles |
| `--text-hero` | `clamp(3.5rem, 7vw, 6.5rem)` | 56px | 104px | Hero title |
| `--text-metric` | `clamp(2.5rem, 5vw, 4rem)` | 40px | 64px | Benchmark numbers |

### 3.3 Font Weights

| Token | Value | Usage |
|---|---|---|
| `--weight-regular` | `400` | Body, lead |
| `--weight-medium` | `500` | — |
| `--weight-semibold` | `600` | Labels, buttons |
| `--weight-bold` | `700` | Display, hero, metrics |

### 3.4 Tracking

| Token | Value | Usage |
|---|---|---|
| `--tracking-tight` | `-0.05em` | Hero |
| `--tracking-display` | `-0.04em` | Section titles |
| `--tracking-wide` | `0.14em` | Labels, ALL CAPS |
| `--tracking-mega` | `-0.06em` | Hero (tightest) |

### 3.5 Line Heights

| Token | Value | Usage |
|---|---|---|
| `--leading-tight` | `0.95` | Hero |
| `--leading-snug` | `1.05` | Display |
| `--leading-normal` | `1.65` | Body |

### 3.6 Typography Rules

1. `font-variant-numeric: tabular-nums` on ALL numeric data.
2. Labels are ALL CAPS with tracking `0.14em` in JetBrains Mono.
3. Code is always JetBrains Mono, never body fonts.
4. No serif fonts anywhere.

---

## 4. Spacing Tokens

Base unit: 4px. Namespaced `--space-*`.

| Token | Value | Rem | Usage |
|---|---|---|---|
| `--space-3xs` | `0.25rem` | 4px | Icon gaps, dot spacing |
| `--space-2xs` | `0.5rem` | 8px | Inline padding, small gaps |
| `--space-xs` | `0.75rem` | 12px | Button padding, tag padding |
| `--space-sm` | `1rem` | 16px | Card padding (tight) |
| `--space-md` | `1.5rem` | 24px | Card padding (default) |
| `--space-lg` | `2rem` | 32px | Section internal spacing |
| `--space-xl` | `3rem` | 48px | Section spacing |
| `--space-2xl` | `4rem` | 64px | Hero padding, major sections |
| `--space-3xl` | `6rem` | 96px | Major section separation |
| `--space-4xl` | `8rem` | 128px | CTA section padding |

## 5. Easing Tokens

| Token | Value | Usage |
|---|---|---|
| `--ease-brutal` | `cubic-bezier(0.05, 0.95, 0.3, 1)` | **PRIMARY** — snap-fast, neubrutalist |
| `--ease-swiss` | `cubic-bezier(0.25, 1, 0.5, 1)` | Secondary — mechanical, sharp |
| `--ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Reveals, entering elements |
| `--ease-in-out` | `cubic-bezier(0.77, 0, 0.175, 1)` | On-screen movement |

**Duration map:**
```css
--duration-instant: 50ms;   /* Button active, ticker */
--duration-snap: 80ms;       /* Button hover, card border */
--duration-fast: 100ms;      /* Nav links, labels */
--duration-normal: 150ms;    /* Card hover, reveals */
--duration-slow: 200ms;      /* Count-up, panel slide */
--duration-reveal: 300ms;    /* Section reveals */
```

**Prohibited:** bounce, elastic, spring, CSS `ease-in-out` default, `linear`.

---

## 6. Border Radius Tokens

| Token | Value | Usage |
|---|---|---|
| `--radius-sm` | `0px` | Buttons (all) |
| `--radius-md` | `0px` | Cards (all) |
| `--radius-lg` | `0px` | Terminal blocks |
| `--radius-xl` | `0px` | Modals |
| `--radius-pill` | `0px` | Badges |

**Rule:** `border-radius: 0` on EVERY element. No exceptions. No 4px, no 6px, no 9999px.

---

## 7. Z-Index Scale

| Token | Value | Usage |
|---|---|---|
| `--z-base` | `1` | Page content |
| `--z-dropdown` | `100` | Dropdown menus, tooltips |
| `--z-sticky` | `200` | Sticky nav, side panels |
| `--z-overlay` | `300` | Mobile nav panel, backdrop |
| `--z-modal` | `400` | Modals, dialogs |
| `--z-toast` | `500` | Toast notifications |
| `--z-noise` | `9999` | Scanline/noise overlays |

---

## 8. Breakpoints

| Token | Value | Media Query | Device |
|---|---|---|---|
| `--bp-mobile` | `375px` | `@media (min-width: 375px)` | Small mobile |
| `--bp-tablet` | `768px` | `@media (min-width: 768px)` | Tablet portrait |
| `--bp-desktop` | `1024px` | `@media (min-width: 1024px)` | Desktop |
| `--bp-wide` | `1440px` | `@media (min-width: 1440px)` | Wide desktop |

Mobile-first: base styles are mobile, override upward with `min-width`.

---

## 9. Section Tokens

| Token | Value | Usage |
|---|---|---|
| `--nav-height` | `64px` | Desktop nav bar |
| `--section-gap` | `96px` | Vertical spacing between sections |
| `--section-gap-lg` | `160px` | Large vertical spacing |
| `--grid-max` | `1200px` | Max content width |

---

## 10. Quick Reference

```css
:root {
  /* Colors */
  --amber: #ff5500;
  --amber-light: #ff7733;
  --amber-dim: rgba(255, 85, 0, 0.12);
  --amber-glow: rgba(255, 85, 0, 0.3);
  --black: #000000;
  --white: #ffffff;
  --success: #00c853;
  --danger: #ff1744;

  --background: #111111;
  --foreground: #f5f5f5;
  --surface: #1a1a1a;
  --surface-alt: #222222;
  --surface-glass: rgba(17, 17, 17, 0.85);
  --terminal-bg: #0d0d0d;
  --text-on-amber: #111111;

  --muted: #808080;
  --steel: oklch(45% 0 0);
  --subtle: #5a5a5a;

  --border: rgba(255, 255, 255, 0.08);
  --border-hover: rgba(255, 255, 255, 0.2);
  --border-strong: #333333;
  --border-visible: rgba(255, 255, 255, 0.15);

  /* Shadows — HARD OFFSET only */
  --shadow-sm: 4px 4px 0px 0px #000000;
  --shadow-md: 6px 6px 0px 0px #000000;
  --shadow-lg: 8px 8px 0px 0px #000000;
  --shadow-amber: 4px 4px 0px 0px var(--amber);
  --shadow-brutal: 8px 8px 0px 0px #111111;
  --shadow-brutal-hover: 2px 2px 0px 0px #111111;
  --shadow-glow: 0 0 20px rgba(255, 85, 0, 0.15);

  /* Radius — ALL ZERO */
  --radius-sm: 0px;
  --radius-md: 0px;
  --radius-lg: 0px;
  --radius-xl: 0px;
  --radius-pill: 0px;

  /* Typography */
  --font-sans: 'Outfit', sans-serif;
  --font-display: 'Space Grotesk', sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* Easing */
  --ease-brutal: cubic-bezier(0.05, 0.95, 0.3, 1);
  --ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);
  --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  --ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);

  /* Layout */
  --nav-height: 64px;
  --section-gap: 96px;
  --section-gap-lg: 160px;
  --grid-max: 1200px;

  /* Spacing */
  --space-3xs: 0.25rem;
  --space-2xs: 0.5rem;
  --space-xs: 0.75rem;
  --space-sm: 1rem;
  --space-md: 1.5rem;
  --space-lg: 2rem;
  --space-xl: 3rem;
  --space-2xl: 4rem;
  --space-3xl: 6rem;
  --space-4xl: 8rem;

  /* Z-index */
  --z-base: 1;
  --z-dropdown: 100;
  --z-sticky: 200;
  --z-overlay: 300;
  --z-modal: 400;
  --z-toast: 500;
  --z-noise: 9999;
}
```

---

## 11. Usage Rules

1. **Never** use raw color values — always reference tokens.
2. **Never** hardcode font stacks — use `var(--font-sans)`, `var(--font-display)`, `var(--font-mono)`.
3. **Never** hardcode spacing — compose from `var(--space-*)` scale.
4. **Never** use CSS default easing — always use easing tokens.
5. **Never** use `box-shadow` with blur — hard offset only.
6. **Always** use `border-radius: 0` — tokens exist for visibility, not for variation.
7. **Mobile-first** — define base styles for mobile, override upward with `min-width`.
