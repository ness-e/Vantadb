# Design Token System — VantaDB

> Versión: 1.0 | 2026-07 | Modos: Light / Dark

---

## 1. Color Tokens

### 1.1 Light Mode

| Token | Value | Usage |
|:---|:---|:---|
| `--amber` | `#ff5500` | Primary accent — CTAs, highlights, selected states |
| `--ink` | `#111111` | Near-black text, dark surfaces |
| `--paper` | `#ffffff` | White backgrounds |
| `--elevated` | `#f5f5f5` | Light section backgrounds |
| `--muted` | `#5a5a5a` | Body text, secondary info |
| `--subtle` | `#808080` | Metadata, captions |
| `--line` | `#e0e0e0` | Borders, dividers |
| `--amber-glow` | `rgba(255, 85, 0, 0.35)` | CTA shadow glow |
| `--amber-on-light` | `#111111` | Text on amber buttons |

### 1.2 Dark Mode

| Token | Value | Usage |
|:---|:---|:---|
| `--amber-on-dark` | `#ff5500` | Accent on dark backgrounds |
| `--ink-on-dark` | `#ffffff` | White text on dark surfaces |
| `--muted-on-dark` | `#808080` | Gray text on dark surfaces |
| `--line-strong` | `#333333` | Dark mode borders |
| `--surface-dark` | `#1a1a1a` | Dark section surfaces |
| `--surface-dark-alt` | `#111111` | Dark section background (OLED) |

### 1.3 Token Aliases

```css
--color-accent: var(--amber);
--color-text-primary: var(--ink);
--color-text-body: var(--muted);
--color-text-subtle: var(--subtle);
--color-bg-primary: var(--paper);
--color-bg-elevated: var(--elevated);
--color-border: var(--line);
--color-border-strong: var(--line-strong);
--color-bg-dark: var(--surface-dark);
--color-bg-dark-alt: var(--surface-dark-alt);
--color-text-on-dark: var(--ink-on-dark);
--color-text-muted-on-dark: var(--muted-on-dark);
--color-accent-on-dark: var(--amber-on-dark);
--color-accent-glow: var(--amber-glow);
--color-text-on-accent: var(--amber-on-light);
```

---

## 2. Typography Tokens

### 2.1 Font Families

| Token | Value | Usage |
|:---|:---|:---|
| `--font-sans` | `'Space Grotesk', system-ui, sans-serif` | Headings, display, UI labels |
| `--font-mono` | `'JetBrains Mono', 'Fira Code', monospace` | Code, terminal, data, metrics |

### 2.2 Font Size Scale

Base unit: 1rem = 16px. All sizes use `clamp()` for responsive scaling between mobile (375px) and desktop (1440px).

| Token | Value | Mobile | Desktop | Usage |
|:---|:---|:---|:---|:---|
| `--text-micro` | `clamp(0.5rem, 0.45rem + 0.15vw, 0.625rem)` | 8px | 10px | Legal, timestamps |
| `--text-label` | `clamp(0.625rem, 0.55rem + 0.2vw, 0.6875rem)` | 10px | 11px | ALL CAPS labels, tag text |
| `--text-caption` | `clamp(0.6875rem, 0.6rem + 0.25vw, 0.75rem)` | 11px | 12px | Captions, helper text |
| `--text-body` | `clamp(0.8125rem, 0.7rem + 0.35vw, 0.875rem)` | 13px | 14px | Body copy, descriptions |
| `--text-lead` | `clamp(0.9375rem, 0.8rem + 0.5vw, 1rem)` | 15px | 16px | Lead paragraphs, larger body |
| `--text-h3` | `clamp(1.125rem, 0.9rem + 0.75vw, 1.25rem)` | 18px | 20px | Section sub-headings |
| `--text-h2` | `clamp(1.5rem, 1.1rem + 1.5vw, 1.75rem)` | 24px | 28px | Section headings |
| `--text-h1` | `clamp(1.75rem, 1.25rem + 2vw, 2.25rem)` | 28px | 36px | Page headings |
| `--text-display` | `clamp(2rem, 1.5rem + 2.5vw, 2.5rem)` | 32px | 40px | Hero subtitle, display text |
| `--text-hero` | `clamp(2.5rem, 1.75rem + 4vw, 4.5rem)` | 40px | 72px | Hero title |

### 2.3 Font Weights

| Token | Value | Usage |
|:---|:---|:---|
| `--weight-regular` | `400` | Body, lead |
| `--weight-medium` | `500` | Captions, labels (sans) |
| `--weight-semibold` | `600` | H3, H2, buttons |
| `--weight-bold` | `700` | H1, display, hero |

### 2.4 Line Heights

| Token | Value | Usage |
|:---|:---|:---|
| `--leading-tight` | `1.1` | Hero, display, H1 |
| `--leading-snug` | `1.2` | H2, H3 |
| `--leading-normal` | `1.5` | Body, lead |
| `--leading-relaxed` | `1.6` | Captions, micro |

### 2.5 Letter Spacing

| Token | Value | Usage |
|:---|:---|:---|
| `--tracking-tight` | `-0.02em` | Hero, display |
| `--tracking-normal` | `0em` | Body, headings |
| `--tracking-wide` | `0.04em` | Labels, captions |
| `--tracking-wider` | `0.08em` | ALL CAPS labels |
| `--tracking-widest` | `0.14em` | Small uppercase tags |

### 2.6 Typography Map

```css
--typo-hero: var(--weight-bold) var(--text-hero)/var(--leading-tight) var(--font-sans);
--typo-display: var(--weight-bold) var(--text-display)/var(--leading-tight) var(--font-sans);
--typo-h1: var(--weight-bold) var(--text-h1)/var(--leading-tight) var(--font-sans);
--typo-h2: var(--weight-semibold) var(--text-h2)/var(--leading-snug) var(--font-sans);
--typo-h3: var(--weight-semibold) var(--text-h3)/var(--leading-snug) var(--font-sans);
--typo-lead: var(--weight-regular) var(--text-lead)/var(--leading-normal) var(--font-sans);
--typo-body: var(--weight-regular) var(--text-body)/var(--leading-normal) var(--font-sans);
--typo-caption: var(--weight-medium) var(--text-caption)/var(--leading-relaxed) var(--font-sans);
--typo-label: var(--weight-medium) var(--text-label)/1 var(--font-sans);
--typo-micro: var(--weight-regular) var(--text-micro)/var(--leading-normal) var(--font-mono);
--typo-code: var(--weight-regular) var(--text-body)/var(--leading-normal) var(--font-mono);
--typo-mono-label: var(--weight-medium) var(--text-label)/1 var(--font-mono);
```

---

## 3. Spacing Tokens

Base unit: 4px. Namespaced `--space-*`.

| Token | Value | Rem | Usage |
|:---|:---|:---|:---|
| `--space-3xs` | `4px` | `0.25rem` | Icon gaps, dot spacing |
| `--space-2xs` | `8px` | `0.5rem` | Inline padding, small gaps |
| `--space-xs` | `12px` | `0.75rem` | Button padding, tag padding |
| `--space-sm` | `16px` | `1rem` | Card padding, section padding |
| `--space-md` | `24px` | `1.5rem` | Between sections, grid gap |
| `--space-lg` | `32px` | `2rem` | Large section gaps |
| `--space-xl` | `48px` | `3rem` | Section padding top/bottom |
| `--space-2xl` | `64px` | `4rem` | Hero padding, major sections |
| `--space-3xl` | `80px` | `5rem` | Page section separation |
| `--space-4xl` | `96px` | `6rem` | CTA section padding |

### Spacing Aliases

```css
--gap-icon: var(--space-3xs);
--gap-inline: var(--space-2xs);
--pad-tag: var(--space-xs);
--pad-card: var(--space-sm);
--pad-section: var(--space-xl);
--pad-hero: var(--space-2xl);
--pad-cta: var(--space-4xl);
--grid-gap: var(--space-md);
```

---

## 4. Border Radius Tokens

| Token | Value | Usage |
|:---|:---|:---|
| `--radius-none` | `0px` | Buttons (primary), cards (Swiss style) |
| `--radius-sm` | `4px` | Terminal blocks, code blocks |
| `--radius-md` | `6px` | Larger terminal panels |
| `--radius-lg` | `10px` | Input fields, search bars |
| `--radius-xl` | `12px` | Modals, dropdowns |
| `--radius-full` | `9999px` | Pill tags, badges |

### Radius Map

```css
--radius-button: var(--radius-none);
--radius-card: var(--radius-none);
--radius-terminal: var(--radius-sm);
--radius-badge: var(--radius-full);
--radius-input: var(--radius-lg);
--radius-modal: var(--radius-xl);
```

---

## 5. Shadow Tokens

| Token | Value | Usage |
|:---|:---|:---|
| `--shadow-sm` | `0 1px 3px rgba(0, 0, 0, 0.06)` | Cards, subtle elevation |
| `--shadow-md` | `0 4px 12px rgba(0, 0, 0, 0.08)` | Dropdowns, raised cards |
| `--shadow-lg` | `0 8px 30px rgba(0, 0, 0, 0.12)` | Modals, sticky elements |
| `--shadow-amber` | `0 4px 16px rgba(255, 85, 0, 0.35)` | Primary CTA glow |

### Dark Mode Shadows

```css
--shadow-sm-dark: 0 1px 3px rgba(0, 0, 0, 0.3);
--shadow-md-dark: 0 4px 12px rgba(0, 0, 0, 0.4);
--shadow-lg-dark: 0 8px 30px rgba(0, 0, 0, 0.5);
```

---

## 6. Z-Index Scale

| Token | Value | Usage |
|:---|:---|:---|
| `--z-base` | `0` | Page content |
| `--z-dropdown` | `100` | Dropdown menus, tooltips |
| `--z-sticky` | `200` | Sticky nav, side panels |
| `--z-overlay` | `300` | Mobile nav panel, backdrop |
| `--z-modal` | `400` | Modals, dialogs |
| `--z-toast` | `500` | Toast notifications |
| `--z-tooltip` | `600` | Tooltips (above everything) |

### Z-Index Map

```css
--z-nav: var(--z-sticky);
--z-nav-overlay: var(--z-overlay);
--z-backdrop: calc(var(--z-overlay) - 1);
```

---

## 7. Breakpoints

| Token | Value | Media Query | Device |
|:---|:---|:---|:---|
| `--bp-mobile` | `375px` | `@media (min-width: 375px)` | Small mobile |
| `--bp-tablet` | `768px` | `@media (min-width: 768px)` | Tablet portrait |
| `--bp-tablet-lg` | `1024px` | `@media (min-width: 1024px)` | Tablet landscape |
| `--bp-desktop` | `1280px` | `@media (min-width: 1280px)` | Desktop |
| `--bp-wide` | `1440px` | `@media (min-width: 1440px)` | Wide desktop |

### Breakpoint Aliases

```css
--bp-sm: var(--bp-mobile);
--bp-md: var(--bp-tablet);
--bp-lg: var(--bp-desktop);
--bp-xl: var(--bp-wide);
```

### Usage Convention

```css
/* Mobile-first — base styles are mobile */
/* Tablet */
@media (min-width: 768px) { }
/* Desktop */
@media (min-width: 1280px) { }
/* Wide */
@media (min-width: 1440px) { }
```

---

## 8. Easing Tokens

| Token | Value | Usage |
|:---|:---|:---|
| `--ease-swiss` | `cubic-bezier(0.25, 1, 0.5, 1)` | Primary UI — mechanical cut |
| `--ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Strong ease-out for revealed elements |
| `--ease-in-out` | `cubic-bezier(0.77, 0, 0.175, 1)` | In-out for on-screen transitions |
| `--ease-emil` | `cubic-bezier(0.16, 1, 0.3, 1)` | Emil Kowalski style — optimistic over-shoot |
| `--ease-snappy` | `cubic-bezier(0.5, 0, 0, 1)` | Fast micro-interactions, button press |

### Easing Map

```css
--duration-instant: 50ms;
--duration-fast: 100ms;
--duration-normal: 150ms;
--duration-slow: 200ms;
--duration-reveal: 300ms;
--duration-enter: 400ms;
--duration-stagger: 80ms;
```

---

## 9. Layout Tokens

| Token | Value | Usage |
|:---|:---|:---|
| `--nav-height` | `64px` | Desktop nav bar |
| `--nav-height-mobile` | `56px` | Mobile nav bar |
| `--content-max` | `1200px` | Max content width |
| `--content-wide` | `1400px` | Wide content sections |
| `--grid-cols` | `12` | Base grid column count |
| `--grid-gap` | `var(--space-md)` | Default grid gap |

---

## 10. Token Index (Quick Reference)

```css
:root {
  /* Color */
  --amber: #ff5500;
  --ink: #111111;
  --paper: #ffffff;
  --elevated: #f5f5f5;
  --muted: #5a5a5a;
  --subtle: #808080;
  --line: #e0e0e0;
  --amber-glow: rgba(255, 85, 0, 0.35);
  --amber-on-light: #111111;

  /* Typography */
  --font-sans: 'Space Grotesk', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  --text-micro: clamp(0.5rem, 0.45rem + 0.15vw, 0.625rem);
  --text-label: clamp(0.625rem, 0.55rem + 0.2vw, 0.6875rem);
  --text-caption: clamp(0.6875rem, 0.6rem + 0.25vw, 0.75rem);
  --text-body: clamp(0.8125rem, 0.7rem + 0.35vw, 0.875rem);
  --text-lead: clamp(0.9375rem, 0.8rem + 0.5vw, 1rem);
  --text-h3: clamp(1.125rem, 0.9rem + 0.75vw, 1.25rem);
  --text-h2: clamp(1.5rem, 1.1rem + 1.5vw, 1.75rem);
  --text-h1: clamp(1.75rem, 1.25rem + 2vw, 2.25rem);
  --text-display: clamp(2rem, 1.5rem + 2.5vw, 2.5rem);
  --text-hero: clamp(2.5rem, 1.75rem + 4vw, 4.5rem);
  --weight-regular: 400;
  --weight-medium: 500;
  --weight-semibold: 600;
  --weight-bold: 700;
  --leading-tight: 1.1;
  --leading-snug: 1.2;
  --leading-normal: 1.5;
  --leading-relaxed: 1.6;
  --tracking-tight: -0.02em;
  --tracking-normal: 0em;
  --tracking-wide: 0.04em;
  --tracking-wider: 0.08em;
  --tracking-widest: 0.14em;

  /* Spacing */
  --space-3xs: 4px;
  --space-2xs: 8px;
  --space-xs: 12px;
  --space-sm: 16px;
  --space-md: 24px;
  --space-lg: 32px;
  --space-xl: 48px;
  --space-2xl: 64px;
  --space-3xl: 80px;
  --space-4xl: 96px;

  /* Radius */
  --radius-none: 0px;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 10px;
  --radius-xl: 12px;
  --radius-full: 9999px;

  /* Shadows */
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.06);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 8px 30px rgba(0, 0, 0, 0.12);
  --shadow-amber: 0 4px 16px rgba(255, 85, 0, 0.35);

  /* Z-index */
  --z-base: 0;
  --z-dropdown: 100;
  --z-sticky: 200;
  --z-overlay: 300;
  --z-modal: 400;
  --z-toast: 500;
  --z-tooltip: 600;

  /* Easing */
  --ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);
  --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  --ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);
  --ease-emil: cubic-bezier(0.16, 1, 0.3, 1);
  --ease-snappy: cubic-bezier(0.5, 0, 0, 1);

  /* Layout */
  --nav-height: 64px;
  --nav-height-mobile: 56px;
  --content-max: 1200px;
  --content-wide: 1400px;
  --grid-cols: 12;
}

[data-theme="dark"] {
  --amber-on-dark: #ff5500;
  --ink-on-dark: #ffffff;
  --muted-on-dark: #808080;
  --line-strong: #333333;
  --surface-dark: #1a1a1a;
  --surface-dark-alt: #111111;
  --line: #333333;
  --muted: #808080;
  --subtle: #666666;
  --elevated: #1a1a1a;
  --ink: #e5e5e5;
  --paper: #111111;
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 8px 30px rgba(0, 0, 0, 0.5);
}
```

---

## 11. Usage Rules

1. **Never** use raw color values — always reference tokens.
2. **Never** hardcode font stacks — use `var(--font-sans)` and `var(--font-mono)`.
3. **Never** hardcode spacing — compose from `var(--space-*)` scale.
4. **Never** use CSS `ease-in`, `ease-out`, `linear` defaults — always use easing tokens.
5. **Always** use `clamp()` for font sizes — never fixed `px` or `rem` alone.
6. **Mobile-first** — define base styles for mobile, override upward with `min-width` queries.
7. **Shadows** in dark mode: use dark-specific shadow tokens for depth on dark surfaces.
