# Component Specification — VantaDB Landing Page

> Versión: 2.0 | 2026-07 | Estilo: Swiss + Neubrutalism
> Modo único: Dark

---

## 1. Logo Mark

### Purpose
Brand identity mark used in nav, footer, and favicon.

### Anatomy
```
        ┌─────────────┐
        │   ┌─────┐   │   Outer ring: circular stroke
        │   │  ●  │   │   Inner dot: filled amber circle
        │   └─────┘   │
        └─────────────┘
```

| Element | Spec |
|---|---|
| Outer ring | Circle, stroke-width `2px`, color `var(--foreground)` |
| Inner dot | Filled circle, `--amber`, centered inside ring |
| Canvas | `40×40px` (nav), `48×48px` (footer), `32×32px` (mobile) |
| Responsive | SVG viewBox `0 0 40 40` — scale via `width`/`height` |

---

## 2. Nav (`.nav`)

### Purpose
Primary navigation bar — persistent across all pages.

### Anatomy
```
┌─────────────────────────────────────────────────────────────┐
│  [◆ VantaDB]         [DOCS]  [ENGINE]  [PRICING]  [Get Started →]  │
│  ═══════════════════════════════════════════════════════════  │  ← 2px border
└─────────────────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|---|---|
| Height | `var(--nav-height)` (64px) |
| Background | `var(--surface-glass)` + `backdrop-filter: blur(12px)` |
| Bottom border | `2px solid var(--border-visible)` |
| Position | `fixed` top, `z-index: var(--z-sticky)` |
| Layout | Logo left \| Links center \| CTA right |
| Max content | `var(--grid-max)`, centered with `padding: 0 var(--space-sm)` |

### Link Styles

| Property | Value |
|---|---|
| Font | `var(--font-mono)`, `0.72rem`, `600` |
| Letter spacing | `var(--tracking-wide)` (0.14em) |
| Text transform | `uppercase` |
| Color resting | `var(--steel)` |
| Color hover | `var(--foreground)`, 80ms transition |
| Color active | `var(--amber)` |
| Padding | `var(--space-2xs) var(--space-xs)` |
| Transition | `color 80ms linear` |

### CTA Button in Nav

| Property | Value |
|---|---|
| Class | `.btn-primary` |
| Padding | `12px 28px` |
| Shadow | `var(--shadow-md)` resting → `var(--shadow-brutal-hover)` hover |

### Mobile Behavior

| Property | Value |
|---|---|
| Breakpoint | Below `768px` |
| Trigger | Hamburger icon (3-line stack) |
| Panel | Slide-in from right, width `280px` |
| Backdrop | `rgba(0,0,0,0.4)` |

---

## 3. Buttons

### Primary (`.btn-primary`)

| Property | Value |
|---|---|
| Font | `var(--font-mono)`, `0.72rem`, `700`, `0.14em` tracking |
| Text transform | `uppercase` |
| Padding | `12px 28px` |
| Border | `2px solid #cc4400` |
| Border radius | `0` |
| Background resting | `var(--amber)` |
| Color | `var(--text-on-amber)` |
| Shadow resting | `var(--shadow-md)` (6px 6px 0px 0px #000) |
| Hover background | `var(--amber-light)` |
| Hover shadow | `var(--shadow-sm)` (4px 4px 0px 0px #000) |
| Hover transform | `translate(3px, 3px)` |
| Active shadow | `none` |
| Active transform | `translate(6px, 6px)` |
| Transition | `80ms var(--ease-brutal)` for shadow, transform, background |
| Focus-visible | `outline: 2.5px solid var(--amber)`, `outline-offset: 2px` |

**Mechanical press behavior:** Resting → shadow reduces + button moves right/down → active → shadow disappears + button presses deeper. Simulates a physical key press.

### Ghost (`.btn-ghost`)

| Property | Value |
|---|---|
| Background resting | `transparent` |
| Border | `2px solid var(--border-visible)` |
| Color | `var(--foreground)` |
| Shadow resting | `var(--shadow-sm)` |
| Hover background | `var(--foreground)` |
| Hover color | `var(--background)` |
| Hover border | `var(--foreground)` |
| Hover transform | `translate(3px, 3px)` |
| Hover shadow | `none` |
| Active transform | `translate(6px, 6px)` |
| Active shadow | `none` |
| Transition | Same as primary |

### Ghost Inverted (`.btn-ghost--inverted`)

For use on amber backgrounds or light surfaces:
| State | Border | Color | Hover bg | Hover color |
|---|---|---|---|---|
| Resting | `rgba(255,255,255,0.3)` | `var(--block-dark-text)` | — | — |
| Hover | `#ffffff` | `#000000` | `#ffffff` | `#000000` |

### Install Button (`.btn-install`)

| Property | Value |
|---|---|
| Font | `var(--font-mono)`, `0.8rem`, `600` |
| Border | `2px solid var(--border-visible)` |
| Background | `var(--surface)` |
| Color | `var(--foreground)` |
| Hover background | `var(--surface-alt)` |
| Hover border | `var(--border-strong)` |
| Transition | `80ms linear` |

### All buttons:
- `border-radius: 0` — enforced globally
- Must fit 1 line
- No duplicate CTA intent per page

---

## 4. Cards (`.nb-card`)

### Purpose
Content containers for features, use cases, data points.

### Anatomy
```
┌──────────────────────────────────────┐
│  [01]                                │  ← nb-index, top-left
│  ┌──────────┐                        │
│  │  icon    │  Title                  │  ← nb-icon-box
│  └──────────┘                        │
│  Description text explaining the      │
│  feature in concise sentences.       │
│                                      │
│  >>  Read more                       │  ← nb-arrow
└──────────────────────────────────────┘
```

### Spec

| Property | Value |
|---|---|
| Background | `var(--surface)` (#1a1a1a) |
| Border | `2px solid var(--border-visible)` |
| Border radius | `0` |
| Padding | `var(--space-lg)` (32px) |
| Shadow | `--shadow-sm` (4px 4px 0px 0px #000) — cards are slightly elevated |
| Hover border | `var(--amber)` |
| Hover shadow | `var(--shadow-amber)` (4px 4px 0px 0px #ff5500) |
| Active | `transform: translate(2px, 2px)` |
| Transition | `border-color 80ms var(--ease-brutal)`, `box-shadow 80ms var(--ease-brutal)` |

### States

| State | Border | Shadow | Background |
|---|---|---|---|
| Resting | `var(--border-visible)` | `var(--shadow-sm)` | `var(--surface)` |
| Hover | `var(--amber)` | `var(--shadow-amber)` | `var(--surface)` |
| Active | `var(--amber)` | `var(--shadow-amber)` | `var(--surface)` |

### Variants

| Variant | Modifier | Change |
|---|---|---|
| Amber highlight | `.nb-card--amber` | Border `var(--amber)` always |
| Strong border | `.nb-card--strong` | Border `var(--border-strong)` |
| Dark section | Uses transparent bg | Hover `rgba(255,255,255,0.03)` |

---

## 5. Frames (`.nb-frame`)

### Purpose
Bordered content areas with floating label — like engineering blueprints.

### Anatomy
```
┌──────────────────────────────────────┐
│  [ LABEL ]                            │  ← floating label
│                                      │
│  Content area with 2px border        │
│  around everything                   │
└──────────────────────────────────────┘
```

### Spec

| Property | Value |
|---|---|
| Border | `2px solid var(--border-visible)` |
| Border radius | `0` |
| Padding | `var(--space-lg)` |
| Label | `data-frame-label` attribute |
| Label font | `--font-mono`, `--text-micro`, `uppercase` |
| Label position | Absolute, top-left, offset -0.6rem |
| Label bg | `var(--background)` |
| Label color | `var(--steel)` |

---

## 6. Block Warning (`.nb-block-warning`)

### Purpose
High-priority warning/alert block — for deprecation notices, breaking changes, security alerts.

### Spec

| Property | Value |
|---|---|
| Background | `#111111` |
| Border | `3px solid #ff5500` |
| Color | `#ffffff` |
| Padding | `var(--space-lg)` |
| Float label | `⚠ WARNING` in `--font-mono`, `--text-micro`, `--amber` |
| Label position | Absolute, top-left, offset -0.7rem |

---

## 7. Icon Box (`.nb-icon-box`)

### Purpose
Square container for icons — replaces plain SVG icons with framed box.

### Spec

| Property | Value |
|---|---|
| Width/Height | `2.5rem` (40px) |
| Border | `2px solid var(--border-visible)` |
| Display | `inline-flex`, centered |
| Icon size | `1.25rem` (20px) |
| Icon stroke | `2` |
| Icon color | `var(--amber)` |

---

## 8. Telemetry Row (`.nb-telemetry`)

### Purpose
Data rows with amber `>` prefix — like live terminal output or log streams.

### Spec

| Property | Value |
|---|---|
| Font | `--font-mono` |
| Font size | `--text-micro` (0.6rem) |
| Color | `var(--steel)` |
| Layout | `flex`, `gap: var(--space-md)` |
| Prefix | `::before { content: ">" }`, color `var(--amber)` |

---

## 9. Labels (`.nb-label`, `.nb-index`)

### Purpose
Section labels with bracket notation: `[SECTION]`, `[01]`, `[02]`.

### Spec

| Property | `.nb-label` | `.nb-index` |
|---|---|---|
| Font | `--font-mono` | `--font-mono` |
| Size | `--text-micro` (0.6rem) | `--text-micro` (0.6rem) |
| Weight | `600` | `600` |
| Transform | `uppercase` | `uppercase` |
| Tracking | `--tracking-wide` (0.14em) | `--tracking-wide` (0.14em) |
| Color | `var(--steel)` | `var(--steel)` |
| Hover | — | `var(--amber)` on parent hover, 80ms |

### Variants

| Variant | Color |
|---|---|
| `.nb-label--amber` | `var(--amber)` |
| `.nb-index--amber` | `var(--amber)` (permanent) |

---

## 10. Brackets (`.nb-bracket`)

### Purpose
Auto-wraps content in `[ ]` monospace brackets — for telemetry-style labels.

```css
.nb-bracket::before { content: "[ "; font-family: var(--font-mono); color: var(--steel); }
.nb-bracket::after { content: " ]"; font-family: var(--font-mono); color: var(--steel); }
```

---

## 11. Arrows (`.nb-arrow`)

### Purpose
CTA links with animated `>>>` suffix. Used in cards and section links.

| Property | Value |
|---|---|
| Font | `--font-mono` |
| Size | `--text-label` |
| Weight | `700` |
| Transform | `uppercase` |
| Tracking | `--tracking-wide` |
| Color | `var(--amber)` |
| Suffix | `>>>` |
| Hover | `gap` expands: `0.25rem` → `0.5rem`, 150ms |

---

## 12. Pills (`.nb-pill-status`)

### Purpose
Status indicators — used for tags, version badges, integration categories.

| Property | Value |
|---|---|
| Font | `--font-mono`, `--text-micro`, `600` |
| Transform | `uppercase` |
| Tracking | `--tracking-wide` |
| Border | `1px solid var(--border-visible)` |
| Padding | `0.25rem 0.5rem` |
| Layout | `inline-flex`, `gap: 0.375rem` |
| Dot | `4×4px` `currentColor` circle before text |

### Variants

| Variant | Border Color | Text Color |
|---|---|---|
| Default | `var(--border-visible)` | `var(--steel)` |
| `.nb-pill-status--amber` | `var(--amber)` | `var(--amber)` |
| `.nb-pill-status--green` | `var(--success)` | `var(--success)` |

---

## 13. Table (`.nb-table`)

### Purpose
Data tables with monospace typography and visible borders.

| Property | Value |
|---|---|
| Font | `--font-mono` |
| Size | `--text-code` |
| Width | `100%` |
| Collapse | `collapse` |
| Cell padding | `var(--space-xs) var(--space-sm)` |
| Cell border | `1px solid var(--border-visible)` |
| Header font | `--text-micro`, `uppercase`, `--tracking-wide` |
| Header color | `var(--steel)` |
| Header bg | `var(--surface)` |
| Cell color | `var(--foreground)` |
| Row hover | Background `var(--surface-alt)` |

---

## 14. List (`.nb-list`)

### Purpose
Telemetry-style lists with `>` prefix.

| Property | Value |
|---|---|
| List style | `none` |
| Item layout | `flex`, `gap: var(--space-sm)` |
| Item padding | `var(--space-xs) 0` |
| Item border | `1px solid var(--border)` bottom |
| Item font | `--font-mono`, `--text-code` |
| Prefix | `::before { content: ">" }`, amber, bold |

---

## 15. Dividers (`.nb-divider`)

| Property | Value |
|---|---|
| Height | `2px` |
| Background | `var(--border-visible)` |
| Border | `none` |
| Margin | `0` |

### Variants

| Variant | Color |
|---|---|
| Default | `var(--border-visible)` |
| `.nb-divider--amber` | `var(--amber)` |
| `.nb-divider--strong` | `var(--border-strong)` |

---

## 16. Section Header (`.nb-section-header`)

### Purpose
Standard section title block — label + heading + optional border.

| Property | Value |
|---|---|
| Margin bottom | `clamp(2.5rem, 5vw, 4rem)` |
| Label | `.nb-label` above heading |
| Heading | `.text-display` or `.text-title` |

### Variants

| Variant | Addition |
|---|---|
| `.nb-section-header--bordered` | `2px` bottom border + padding |
| `.nb-section-header--right-border` | `2px` right border |

---

## 17. Section Layout (`.nb-section`)

| Property | Value |
|---|---|
| Padding | `var(--section-gap) clamp(1.5rem, 5vw, 4rem)` |
| Position | `relative` |

### Variants

| Variant | Property |
|---|---|
| `.nb-section--sm` | Half gap |
| `.nb-section--lg` | Large gap (`--section-gap-lg`) |
| `.nb-section--dark` | Dark bg + text |

---

## 18. Inner Content (`.nb-inner`)

| Property | Value |
|---|---|
| Max width | `var(--grid-max)` (1200px) |
| Margin | `0 auto` (centered) |
| Width | `100%` |

### Variant

| Variant | Max width |
|---|---|
| `.nb-inner--wide` | `1440px` |

---

## 19. Grid Layouts

### Grid (`.nb-grid`)

| Property | Value |
|---|---|
| Display | `grid` |
| Gap | `1px` |
| Background | `var(--border-visible)` — gap = visible grid line |

| Variant | Columns |
|---|---|
| `.nb-grid--cols-2` | `repeat(2, 1fr)` |
| `.nb-grid--cols-3` | `repeat(3, 1fr)` |
| `.nb-grid--cols-4` | `repeat(4, 1fr)` |
| `.nb-grid--cols-6` | `repeat(6, 1fr)` |

### Cell (`.nb-cell`)

| Property | Value |
|---|---|
| Background | `var(--background)` |
| Padding | `var(--space-lg)` |
| Transition | `background 150ms var(--ease-brutal)` |
| Hover | Background `var(--surface-alt)` |

### Span Variants

| Class | Property |
|---|---|
| `.nb-cell--span2` | `grid-column: span 2` |
| `.nb-cell--span3` | `grid-column: span 3` |
| `.nb-cell--row2` | `grid-row: span 2` |

### Asymmetric Layout (`.nb-asymmetric`)

| Variant | Columns |
|---|---|
| `.nb-asymmetric` | `8fr 4fr` |
| `.nb-asymmetric--right` | `4fr 8fr` |

### Split Layout (`.nb-split-7-5`, `.nb-split-5-7`)

| Variant | Columns | Usage |
|---|---|---|
| `.nb-split-7-5` | `7fr 5fr` | Content + visual |
| `.nb-split-5-7` | `5fr 7fr` | Visual + content |

### Bento Grid (`.nb-bento`)

| Property | Value |
|---|---|
| Display | `grid` |
| Gap | `1px` |
| Background | `var(--border-visible)` |

| Variant | Columns |
|---|---|
| `.nb-bento--3col` | `repeat(3, 1fr)` |
| `.nb-bento--4col` | `repeat(4, 1fr)` |

### Bento Cell (`.nb-bento-cell`)

| Property | Value |
|---|---|
| Background | `var(--background)` |
| Padding | `var(--space-lg)` |
| Transition | `background 150ms var(--ease-brutal)` |
| Hover | `var(--surface-alt)` |

| Variant | Span |
|---|---|
| `.nb-bento-cell--featured` | `grid-column: span 2`, `grid-row: span 2` |
| `.nb-bento-cell--span2` | `grid-column: span 2` |
| `.nb-bento-cell--span3` | `grid-column: span 3` |
| `.nb-bento-cell--row2` | `grid-row: span 2` |

---

## 20. Background Textures

### Scanline (`.scanline`)

| Property | Value |
|---|---|
| Position | `fixed`, full viewport |
| Z-index | `var(--z-noise)` (9999) |
| Pointer | `none` |
| Pattern | `repeating-linear-gradient` — 2px transparent, 2px black at 8% opacity |

### Noise Overlay (`.noise-overlay`)

| Property | Value |
|---|---|
| Position | `fixed`, full viewport |
| Z-index | `calc(var(--z-noise) - 1)` |
| Pointer | `none` |
| Opacity | `0.035` |
| Pattern | SVG fractal noise, 256px tiles |

### Dot Grid (`.nb-bg-dot`)

| Property | Value |
|---|---|
| Background | `radial-gradient(var(--border-visible) 1px, transparent 1px)` |
| Background size | `24px 24px` |

### Cross Grid (`.nb-bg-cross`)

| Property | Value |
|---|---|
| Background | Dual linear-gradient (horizontal + vertical) |
| Background size | `24px 24px` |
| Line | `1px`, `var(--border-visible)` |

### Faint Cross Grid (`.nb-bg-cross--faint`)

| Property | Value |
|---|---|
| Background size | `48px 48px` |
| Line opacity | `rgba(255,255,255,0.04)` |

---

## 21. Scrollbar

| Property | Value |
|---|---|
| Width | `6px` |
| Track | `var(--surface)` |
| Thumb | `var(--border-strong)` |
| Thumb radius | `0px` |
| Thumb hover | `var(--amber)` |

---

## 22. Selection

| Property | Value |
|---|---|
| Background | `var(--amber)` |
| Color | `var(--white)` |

---

## 23. Page Templates (Subpage Patterns)

### 23.1 Subpage Hero

```
┌──────────────────────────────────────────┐
│  [SECTION]                               │  nb-label
│                                          │
│  Page Title                              │  text-display
│                                          │
│  Supporting description of the           │
│  page content and what users can         │
│  expect to find here.                    │
│                                          │
│  ┌────────────────┐  ┌────────────────┐  │
│  │  btn-primary   │  │  btn-ghost     │  │
│  └────────────────┘  └────────────────┘  │
└──────────────────────────────────────────┘
```

| Property | Value |
|---|---|
| Padding top | `calc(var(--nav-height) + var(--space-lg))` (96px) |
| Padding bottom | `var(--space-xl)` |
| Content | Centered, max `720px` (text) |
| Background | `.nb-section--dark` with `.noise-overlay` |

**Variant `--sm`:** Padding top reduced, title smaller. For docs index, legal pages.
**Variant `--docs`:** Title in `--font-mono`, search input subtitle, `.nb-bg-cross` background.

### 23.2 Feature Page Template

```
┌──────────────────────────────────────────┐
│  .nb-subpage-hero                        │
├──────────────────────────────────────────┤
│  .nb-section                             │
│  ┌────────────────────────────────────┐  │
│  │  .nb-section-header               │  │
│  │  > .nb-label                       │  │
│  │  > .text-title                     │  │
│  │  Asymmetric layout with feature    │  │
│  │  details and code example          │  │
│  └────────────────────────────────────┘  │
│  ┌───────────┐ ┌─────────────────────┐  │
│  │ .nb-frame │ │ Code example /      │  │
│  │ Content   │ │ Visual / Metrics    │  │
│  └───────────┘ └─────────────────────┘  │
├──────────────────────────────────────────┤
│  .nb-section (dark) — benchmark section  │
│  ┌────────────────────────────────────┐  │
│  │  Telemetry metrics in nb-grid:     │  │
│  │  [ LATENCY ] [ RECALL ] [ INDEX ] │  │
│  └────────────────────────────────────┘  │
├──────────────────────────────────────────┤
│  .nb-section — related cards grid       │
│  ┌──────┐ ┌──────┐ ┌──────┐            │
│  │ Card │ │ Card │ │ Card │            │
│  └──────┘ └──────┘ └──────┘            │
├──────────────────────────────────────────┤
│  CTA Section (.nb-section--dark)        │
└──────────────────────────────────────────┘
```

### 23.3 Docs Page Template

```
┌──────────────────────────────────────────┐
│  Hero--docs                               │
├──────────────────────────────────────────┤
│  ┌──────────┬─────────────────────────┐  │
│  │  Sidebar  │  Content               │  │
│  │  .nb-list │  .nb-frame             │  │
│  │  [01]     │  ## Section Title      │  │
│  │  [02]     │  Body text with lists  │  │
│  │  [03]     │  .nb-block-warning     │  │
│  └──────────┴─────────────────────────┘  │
└──────────────────────────────────────────┘
```

| Element | Component |
|---|---|
| Sidebar | `.nb-list` with `.nb-index` items; active = amber bg + white text |
| Content | `.nb-frame` wrapping |
| Code blocks | `--font-mono`, `--text-code`, visible border |
| Inline code | `--amber` background |

### 23.4 CTA Section

```
┌──────────────────────────────────────────┐
│  [ GET STARTED ]  nb-label--amber         │
│  Ready to Build?                          │
│  ┌──────────────────────┐               │
│  │  GET STARTED         │  btn-primary   │
│  └──────────────────────┘               │
│  or read the docs →                      │
└──────────────────────────────────────────┘
```

| Property | Value |
|---|---|
| Section | `.nb-section--dark` |
| Background | `.nb-bg-cross--faint` overlay |
| Padding | `clamp(4rem, 8vw, 8rem)` vertical |

---

## 24. Implementation Rules

1. **No hardcoded values** — every color, size, radius, shadow must reference a token.
2. **Mobile-first CSS** — base styles target `< 768px`, override with `min-width`.
3. **Single dark mode** — no light/dark toggle. `#111111` is the only background.
4. **Motion** — all transitions use `--ease-brutal` (primary) at 50-150ms.
5. **Shadows** — hard offset only. No blur, no spread, no `box-shadow: none` in normal states.
6. **Border-radius** — zero everywhere. No exceptions.
7. **Icons** — use `.nb-icon-box` container. 2px stroke min.
8. **Accessibility** — `focus-visible: 2.5px solid var(--amber)` on all interactive elements.
