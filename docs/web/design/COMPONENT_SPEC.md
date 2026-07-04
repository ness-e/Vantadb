# Component Specification — VantaDB Landing Page

> Versión: 1.0 | 2026-07 | Modos: Light / Dark

---

## 1. Logo Mark

### Purpose
Brand identity mark used in nav, footer, and favicon. Represents the VantaDB brand with a technical, precision-instrument aesthetic.

### Anatomy

```
        ╭─────────────╮
        │   ┌─────┐   │   Outer ring: circular stroke
        │   │  ●  │   │   Inner dot: filled amber circle
        │   └─────┘   │
        ╰─────────────╯
```

| Element | Spec |
|:---|:---|
| Outer ring | Circle, stroke-width `2px`, color `var(--ink)` |
| Inner dot | Filled circle, `--amber`, centered inside ring |
| Canvas | `40×40px` (nav), `48×48px` (footer), `32×32px` (mobile) |
| Responsive | SVG viewBox `0 0 40 40` — scale via `width`/`height` |

### States

| Context | Outer Ring | Inner Dot |
|:---|:---|:---|
| Light (nav) | `var(--ink)` | `var(--amber)` |
| Dark (footer) | `var(--ink-on-dark)` | `var(--amber-on-dark)` |
| Hover | Opacity `0.8` | Opacity `1` |

### Tokens Used
`--ink`, `--amber`, `--ink-on-dark`, `--amber-on-dark`

---

## 2. Nav

### Purpose
Primary navigation bar — persistent across all pages. Provides brand identity, page links, and primary CTA access.

### Anatomy

```
┌─────────────────────────────────────────────────────────────┐
│  [◆ VantaDB]         Docs  Engine  Benchmarks  Pricing  [Get Started →]  │
│  ──────────────────────────────────────────────────────────  │  ← 1px border
└─────────────────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Height | `var(--nav-height)` (64px desktop), 56px mobile |
| Background light | `var(--paper)` with `rgba(255,255,255,0.85)` + `backdrop-filter: blur(12px)` |
| Background dark | `var(--surface-dark-alt)` with `rgba(17,17,17,0.9)` + `backdrop-filter: blur(12px)` |
| Bottom border | `1px solid var(--line)` (light), `1px solid var(--line-strong)` (dark) |
| Position | `fixed` top, `z-index: var(--z-sticky)` |
| Layout | Logo left \| Links center \| CTA right |
| Max content | `var(--content-max)`, centered with `padding: 0 var(--space-sm)` |
| Padding inline | `var(--space-sm)` left/right |

### Link Styles

| Property | Value |
|:---|:---|
| Font | `var(--typo-label)` |
| Letter spacing | `var(--tracking-widest)` |
| Text transform | `uppercase` |
| Color resting | `var(--muted)` |
| Color hover | `var(--ink)` (light) / `var(--ink-on-dark)` (dark) |
| Color active | `var(--amber)` |
| Color active route | `var(--amber)` |
| Padding | `var(--space-2xs) var(--space-xs)` |
| Transition | `color var(--duration-fast) var(--ease-swiss)` |

### CTA Button in Nav

| Property | Value |
|:---|:---|
| Background | `var(--amber)` |
| Color | `var(--amber-on-light)` |
| Font | `var(--typo-label)` |
| Letter spacing | `var(--tracking-wider)` |
| Text transform | `uppercase` |
| Padding | `var(--space-2xs) var(--space-sm)` |
| Border radius | `var(--radius-none)` |
| Hover background | `#e64a00` |
| Active | `scale(0.97)` |

### Mobile Behavior

| Property | Value |
|:---|:---|
| Breakpoint | Below `768px` |
| Trigger | Hamburger icon (3-line stack, `var(--ink)` color) |
| Panel | Slide-in from right, width `280px`, `var(--paper)` bg |
| Panel z-index | `var(--z-overlay)` |
| Backdrop | `rgba(0,0,0,0.4)`, `z-index: var(--z-backdrop)` |
| Links | Stacked vertically, full-width tap targets |
| CTA | Full-width button at bottom of panel |
| Transition | `transform 200ms var(--ease-out)` |
| Body scroll | `overflow: hidden` when open |

### Tokens Used
`--paper`, `--surface-dark-alt`, `--line`, `--line-strong`, `--ink`, `--ink-on-dark`, `--muted`, `--amber`, `--amber-on-light`, `--font-sans`, `--font-mono`, `--text-label`, `--tracking-widest`, `--tracking-wider`, `--space-sm`, `--space-xs`, `--space-2xs`, `--radius-none`, `--z-sticky`, `--z-overlay`, `--z-base`, `--ease-swiss`, `--ease-out`

---

## 3. Tag Badges

### Purpose
Contextual labels for classification — "RUST-NATIVE", "IN-PROCESS", "ZERO-SERVERS". Used in hero and feature headers.

### Anatomy

```
●  RUST-NATIVE
[─── pill ───]
```

| Element | Spec |
|:---|:---|
| Shape | Pill — `border-radius: var(--radius-full)` |
| Dot | `4×4px` filled circle, `2px` right of text |
| Text | ALL CAPS, `var(--typo-label)`, `var(--tracking-widest)` |
| Padding | `var(--space-3xs) var(--space-xs)` |
| Border | `1px solid currentColor` (subtle default) |

### Variants

| Variant | Dot Color | Text Color | Border | Background |
|:---|:---|:---|:---|:---|
| Amber | `var(--amber)` | `var(--amber)` | `var(--amber)` light | `rgba(255,85,0,0.08)` |
| Neutral | `var(--muted)` | `var(--muted)` | `var(--line)` | `var(--elevated)` |
| Dark | `var(--amber-on-dark)` | `var(--muted-on-dark)` | `var(--line-strong)` | `transparent` |

### States

| State | Change |
|:---|:---|
| Resting | As above |
| Hover | Opacity `1` → slightly brighter variant |

### Tokens Used
`--amber`, `--muted`, `--muted-on-dark`, `--amber-on-dark`, `--line`, `--line-strong`, `--elevated`, `--text-label`, `--tracking-widest`, `--radius-full`, `--space-3xs`, `--space-xs`

---

## 4. Primary Button

### Purpose
Primary call-to-action — amber fill, black text, amber glow shadow. Used in hero, CTA section, and nav.

### Anatomy

```
┌──────────────────────┐
│  Get Started    →    │
└──────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background resting | `var(--amber)` |
| Color | `var(--amber-on-light)` |
| Font | `var(--typo-label)` |
| Letter spacing | `var(--tracking-wider)` |
| Text transform | `uppercase` |
| Padding | `var(--space-xs) var(--space-md)` |
| Border | `none` |
| Border radius | `var(--radius-none)` |
| Shadow | `var(--shadow-amber)` |
| Gap (icon to text) | `var(--space-2xs)` |
| Cursor | `pointer` |
| White-space | `nowrap` |
| Transition | `all var(--duration-normal) var(--ease-swiss)` |

### States

| State | Background | Shadow | Transform |
|:---|:---|:---|:---|
| Resting | `var(--amber)` | `var(--shadow-amber)` | — |
| Hover | `#e64a00` | `0 6px 24px rgba(255,85,0,0.45)` | `translateY(-1px)` |
| Active | `#cc4200` | `var(--shadow-amber)` | `scale(0.97)` |
| Focus-visible | — | — | Outline `2px solid var(--amber)`, `outline-offset: 2px` |
| Disabled | `var(--line)` | `none` | `cursor: not-allowed`, `opacity: 0.5` |

### Tokens Used
`--amber`, `--amber-on-light`, `--shadow-amber`, `--text-label`, `--tracking-wider`, `--radius-none`, `--space-xs`, `--space-md`, `--space-2xs`, `--ease-swiss`, `--line`

---

## 5. Secondary Button

### Purpose
Secondary CTA — outline stroke, transparent fill. Used alongside primary buttons for paired CTAs.

### Anatomy

```
┌──────────────────────┐
│  View on GitHub  →   │
└──────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background resting | `transparent` |
| Color | `var(--ink)` |
| Border | `1px solid var(--line)` |
| Font | `var(--typo-label)` |
| Letter spacing | `var(--tracking-wider)` |
| Text transform | `uppercase` |
| Padding | `var(--space-xs) var(--space-md)` |
| Border radius | `var(--radius-none)` |
| Gap (icon to text) | `var(--space-2xs)` |
| Cursor | `pointer` |
| White-space | `nowrap` |
| Transition | `all var(--duration-normal) var(--ease-swiss)` |

### States

| State | Background | Border | Color | Transform |
|:---|:---|:---|:---|:---|
| Resting | `transparent` | `var(--line)` | `var(--ink)` | — |
| Hover | `var(--elevated)` | `var(--ink)` | `var(--ink)` | `translateY(-1px)` |
| Active | `var(--line)` | `var(--ink)` | `var(--ink)` | `scale(0.97)` |
| Focus-visible | — | — | — | Outline `2px solid var(--ink)` |

#### Dark Surface Variant

| State | Background | Border | Color |
|:---|:---|:---|:---|
| Resting | `transparent` | `rgba(255,255,255,0.2)` | `var(--ink-on-dark)` |
| Hover | `rgba(255,255,255,0.1)` | `var(--ink-on-dark)` | `var(--ink-on-dark)` |

### Tokens Used
`--ink`, `--ink-on-dark`, `--line`, `--elevated`, `--text-label`, `--tracking-wider`, `--radius-none`, `--space-xs`, `--space-md`, `--space-2xs`, `--ease-swiss`

---

## 6. Code Block / Terminal Window

### Purpose
Display code snippets and terminal output. Used in the hero quickstart and feature demos. Mimics a macOS terminal window.

### Anatomy

```
┌─────────────────────────────────────────┐
│  ● ● ●          Quick Start            │  ← macOS dots + title
├─────────────────────────────────────────┤
│  $ cargo add vantadb               ⏎   │  ← monospace commands
│  $ vantadb init my-project         ⏎   │
│    ✓ Project created in 0.3s           │  ← output with amber accent
│  $ vantadb start                   ⏎   │
│    ┃ Engine running on port 7171       │  ← amber left-border indicator
└─────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | `var(--surface-dark-alt)` |
| Border | `1px solid var(--line-strong)` |
| Border radius | `var(--radius-sm)` |
| Padding | `0` (header + body) |
| Shadow | `var(--shadow-md)` (light), `var(--shadow-lg)` (dark) |
| Min height | `200px` |
| Max width | `var(--content-max)` / 2 (typically `560px`) |

#### Header Bar

| Property | Value |
|:---|:---|
| Height | `36px` |
| Padding | `0 var(--space-sm)` |
| Background | `var(--surface-dark)` |
| Border bottom | `1px solid var(--line-strong)` |
| Display | `flex`, `align-items: center`, `gap: var(--space-2xs)` |

##### macOS Dots

| Dot | Color | Size |
|:---|:---|:---|
| Close (left) | `#ff5f57` | `12×12px` circle |
| Minimize (center) | `#febc2e` | `12×12px` circle |
| Maximize (right) | `#28c840` | `12×12px` circle |

##### Header Title

| Property | Value |
|:---|:---|
| Font | `var(--typo-mono-label)` |
| Color | `var(--muted-on-dark)` |
| Text | "Quick Start" or file name |
| Position | Centered in remaining space |

#### Body

| Property | Value |
|:---|:---|
| Padding | `var(--space-sm)` |
| Font | `var(--typo-code)` |
| Color | `var(--ink-on-dark)` |
| Line height | `var(--leading-relaxed)` |

### Syntax Highlighting

| Token | Color | Usage |
|:---|:---|:---|
| Prompt (`$`) | `var(--muted-on-dark)` | Shell prompt prefix |
| Command | `var(--ink-on-dark)` | User-entered command |
| Keyword | `var(--amber-on-dark)` | Language keywords, flags |
| String | `#f5a623` | String literals |
| Comment | `var(--subtle)` | Code comments |
| Output | `var(--muted-on-dark)` | Terminal output text |
| Success | `#28c840` | Success markers (`✓`) |
| Amber line | `border-left: 2px solid var(--amber-on-dark)` | Important output lines |

### States

| State | Change |
|:---|:---|
| Resting | As above |
| Typewriter active | Characters appear at `30ms/char` |
| Output reveal | Appears instantly after command |
| Auto-restart | After 3s idle, sequence resets |

### Tokens Used
`--surface-dark-alt`, `--surface-dark`, `--line-strong`, `--ink-on-dark`, `--muted-on-dark`, `--amber-on-dark`, `--subtle`, `--radius-sm`, `--shadow-md`, `--shadow-lg`, `--font-mono`, `--text-body`, `--text-label`, `--space-sm`, `--space-2xs`

---

## 7. Metrics Strip

### Purpose
Showcase key performance statistics — throughput, latency, adoption. Organized as labeled stat pairs with dividers.

### Anatomy

```
┌──────────┐│┌──────────┐│┌──────────┐
│   1.2M   │││   <5ms   │││  100%    │
│  ops/sec │││  p99     │││  uptime  │
└──────────┘│└──────────┘│└──────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Layout | Horizontal flex row, `justify-content: center` |
| Gap | `var(--space-lg)` |
| Padding | `var(--space-xl) var(--space-sm)` |
| Background | `var(--paper)` (light) / `var(--surface-dark-alt)` (dark) |
| Border top | `1px solid var(--line)` |
| Border bottom | `1px solid var(--line)` |
| Max width | `var(--content-wide)` |

#### Stat Block

| Element | Property | Value |
|:---|:---|:---|
| Value | Font | `var(--weight-bold) var(--text-display)/var(--leading-tight) var(--font-sans)` |
| Value | Color | `var(--ink)` (light) / `var(--ink-on-dark)` (dark) |
| Value | Margin bottom | `var(--space-3xs)` |
| Label | Font | `var(--typo-label)` |
| Label | Letter spacing | `var(--tracking-wider)` |
| Label | Text transform | `uppercase` |
| Label | Color | `var(--muted)` (light) / `var(--muted-on-dark)` (dark) |
| Alignment | Text | `center` |

#### Divider

| Property | Value |
|:---|:---|
| Width | `1px` |
| Height | `48px` |
| Background | `var(--line)` (light) / `var(--line-strong)` (dark) |
| Align | `align-self: center` |

### States

| State | Change |
|:---|:---|
| Enter viewport | Count-up animation: `0` → target value, `200ms`, `var(--ease-out)` |

### Tokens Used
`--paper`, `--surface-dark-alt`, `--line`, `--line-strong`, `--ink`, `--ink-on-dark`, `--muted`, `--muted-on-dark`, `--font-sans`, `--text-display`, `--text-label`, `--tracking-wider`, `--space-xl`, `--space-sm`, `--space-lg`, `--space-3xs`, `--ease-out`

---

## 8. Benchmark Table

### Purpose
Comparative performance data — VantaDB vs competitors. Label + value + comparison + diff format.

### Anatomy

```
┌──────────────────────────────────────────────────┐
│  Metric              VantaDB     SQLite    ±%     │
│  ──────────────────────────────────────────────── │
│  Write throughput    1,200,000   85,000    ↓ 93%  │  ← amber (faster)
│  Read latency (p99)  4.2ms       12.8ms    ↓ 67%  │  ← amber (faster)
│  Memory per op       128KB       2.1MB     ↓ 94%  │  ← amber (faster)
│  Startup time        0.3s        2.1s      ↓ 86%  │  ← amber (faster)
└──────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | `var(--paper)` (light) / `var(--surface-dark-alt)` (dark) |
| Border | `1px solid var(--line)` (light) / `1px solid var(--line-strong)` (dark) |
| Border radius | `var(--radius-none)` |
| Width | `100%`, max `var(--content-max)` |
| Font | `var(--font-mono)` for all data cells |
| Shadow | `var(--shadow-sm)` |

#### Header Row

| Property | Value |
|:---|:---|
| Background | `var(--elevated)` (light) / `var(--surface-dark)` (dark) |
| Border bottom | `2px solid var(--line)` (light) / `2px solid var(--line-strong)` (dark) |
| Font | `var(--typo-label)` |
| Letter spacing | `var(--tracking-wider)` |
| Text transform | `uppercase` |
| Color | `var(--muted)` (light) / `var(--muted-on-dark)` (dark) |
| Padding | `var(--space-xs) var(--space-sm)` |

#### Data Rows

| Property | Value |
|:---|:---|
| Padding | `var(--space-sm)` |
| Border bottom | `1px solid var(--line)` (light) / `1px solid var(--line-strong)` (dark) |
| Font | `var(--typo-code)` |
| Row height | `48px` |
| Hover | Background `var(--elevated)` (light) / `rgba(255,255,255,0.03)` (dark) |

#### Column Alignment

| Column | Align | Width |
|:---|:---|:---|
| Metric name | `left` | `40%` |
| VantaDB value | `right` | `20%` |
| Competitor value | `right` | `20%` |
| Diff indicator | `right` | `20%` |

#### Diff Indicator

| Value | Display | Color |
|:---|:---|:---|
| Better (faster) | `↓ XX%` | `var(--amber)` |
| Better (higher) | `↑ XX%` | `var(--amber)` |

### States

| State | Change |
|:---|:---|
| Resting | As above |
| Row hover | Background `var(--elevated)`, `transition: 100ms` |
| Enter viewport | Rows stagger in `60ms` apart, `scale(0)` → `scale(1)` from grid edge |

### Tokens Used
`--paper`, `--surface-dark-alt`, `--surface-dark`, `--elevated`, `--line`, `--line-strong`, `--ink`, `--ink-on-dark`, `--muted`, `--muted-on-dark`, `--amber`, `--font-mono`, `--font-sans`, `--text-label`, `--text-body`, `--tracking-wider`, `--space-sm`, `--space-xs`, `--radius-none`, `--shadow-sm`, `--ease-swiss`

---

## 9. Feature Card

### Purpose
Highlight product features in a bento-grid layout. Cards have varied sizes — some span 2 columns, some 1. Each card has an icon/bullet, title, and description.

### Anatomy

```
┌─────────────────────────────────────────┐
│  ●  Title                                │
│                                         │
│  Description text explaining the         │
│  feature in concise sentences.           │
│                                         │
│  [Learn more →]                         │
└─────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | `var(--paper)` (light) / `var(--surface-dark)` (dark) |
| Border | `1px solid var(--line)` (light) / `1px solid var(--line-strong)` (dark) |
| Border radius | `var(--radius-none)` |
| Padding | `var(--space-md)` |
| Shadow | `var(--shadow-sm)` |
| Transition | `all var(--duration-normal) var(--ease-swiss)` |

#### Icon / Bullet Area

| Property | Value |
|:---|:---|
| Size | `20×20px` SVG icon |
| Color resting | `var(--ink)` (light) / `var(--ink-on-dark)` (dark) |
| Color hover | `var(--amber)` |
| Margin bottom | `var(--space-sm)` |
| Stroke | `1.5px`, `stroke-linecap: square` |

#### Title

| Property | Value |
|:---|:---|
| Font | `var(--typo-h3)` |
| Color | `var(--ink)` (light) / `var(--ink-on-dark)` (dark) |
| Margin bottom | `var(--space-2xs)` |

#### Description

| Property | Value |
|:---|:---|
| Font | `var(--typo-body)` |
| Color | `var(--muted)` (light) / `var(--muted-on-dark)` (dark) |
| Line height | `var(--leading-normal)` |

### Grid Placement

| Size | Columns (12-col grid) | Aspect hint |
|:---|:---|:---|
| Wide | `span 2` (6 cols on desktop) | Horizontal emphasis |
| Standard | `span 1` (3 cols on desktop) | Square emphasis |
| Tall | `span 1`, `rows: span 2` | Vertical emphasis |

### States

| State | Background | Border | Icon Color | Shadow |
|:---|:---|:---|:---|:---|
| Resting | `var(--paper)` | `var(--line)` | `var(--ink)` | `var(--shadow-sm)` |
| Hover | `var(--elevated)` | `var(--ink)` | `var(--amber)` | `var(--shadow-md)` |
| Active | `var(--elevated)` | `var(--ink)` | `var(--amber)` | `var(--shadow-sm)` |

### Tokens Used
`--paper`, `--surface-dark`, `--elevated`, `--line`, `--line-strong`, `--ink`, `--ink-on-dark`, `--muted`, `--muted-on-dark`, `--amber`, `--font-sans`, `--text-h3`, `--text-body`, `--space-md`, `--space-sm`, `--space-2xs`, `--radius-none`, `--shadow-sm`, `--shadow-md`, `--ease-swiss`

---

## 10. Architecture Pipeline

### Purpose
Visual diagram showing VantaDB's internal architecture as numbered stages with alternating tint backgrounds. Communicates the processing pipeline.

### Anatomy

```
┌─────────────────────────────────────────────────────────────┐
│  [STAGE 1]         [STAGE 2]         [STAGE 3]             │
│  ┌──────────┐      ┌──────────┐      ┌──────────┐          │
│  │  Ingest  │ ──→  │  Index   │ ──→  │  Query   │          │
│  │          │      │          │      │          │          │
│  │  Raw     │      │  Vector  │      │  Graph   │          │
│  │  input   │      │  store   │      │  search  │          │
│  └──────────┘      └──────────┘      └──────────┘          │
│                                                             │
│  [STAGE 4]         [STAGE 5]                                │
│  ┌──────────┐      ┌──────────┐                             │
│  │  Rank    │ ──→  │  Return  │                             │
│  │          │      │          │                             │
│  │  Score   │      │  Result  │                             │
│  │  + sort  │      │  set     │                             │
│  └──────────┘      └──────────┘                             │
└─────────────────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Layout | Grid or flex row, wrapping. Stages numbered with `[01]`, `[02]` format |
| Background | Alternating row tints: `var(--paper)` / `var(--elevated)` |
| Border radius | `var(--radius-none)` |
| Padding | `var(--space-2xl)` vertical |
| Max width | `var(--content-wide)` |

#### Stage Card

| Property | Value |
|:---|:---|
| Background | `var(--paper)` (light) / `var(--surface-dark)` (dark) |
| Border | `1px solid var(--line)` |
| Border radius | `var(--radius-sm)` |
| Padding | `var(--space-md)` |
| Min width | `180px` |
| Shadow | `var(--shadow-sm)` |

#### Stage Number

| Property | Value |
|:---|:---|
| Font | `var(--typo-mono-label)` |
| Color | `var(--muted)` |
| Letter spacing | `var(--tracking-wider)` |
| Margin bottom | `var(--space-2xs)` |

#### Stage Title

| Property | Value |
|:---|:---|
| Font | `var(--weight-semibold) var(--text-h3)/var(--leading-snug) var(--font-sans)` |
| Color | `var(--ink)` |
| Margin bottom | `var(--space-xs)` |

#### Stage Description

| Property | Value |
|:---|:---|
| Font | `var(--typo-caption)` |
| Color | `var(--muted)` |
| Line height | `var(--leading-normal)` |

#### Pipeline Arrow

| Property | Value |
|:---|:---|
| Style | `→` character or SVG arrow |
| Color | `var(--amber)` |
| Size | `20px` |
| Margin | `0 var(--space-sm)` |

### States

| State | Change |
|:---|:---|
| Resting | As above |
| Stage hover | Border `var(--amber)`, `transition: 100ms` |
| Non-hovered siblings | Opacity `0.4` (dim effect, `200ms` `--ease-out`) |

### Tokens Used
`--paper`, `--elevated`, `--surface-dark`, `--line`, `--line-strong`, `--ink`, `--muted`, `--amber`, `--font-sans`, `--font-mono`, `--text-h3`, `--text-label`, `--text-caption`, `--tracking-wider`, `--space-2xl`, `--space-md`, `--space-sm`, `--space-xs`, `--space-2xs`, `--radius-sm`, `--shadow-sm`, `--ease-out`

---

## 11. Use Case Card

### Purpose
Showcase real-world application scenarios in a bento cell with shadow. Used in the use cases section. Icon/bullet + title + description.

### Anatomy

```
┌──────────────────────────────────────┐
│ ●                                    │
│                                      │
│ Use Case Title                       │
│                                      │
│ Description of the use case in       │
│ one or two sentences explaining      │
│ the scenario and benefits.           │
│                                      │
│ [Explore →]                          │
└──────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | `var(--paper)` (light) / `var(--surface-dark)` (dark) |
| Border | `1px solid var(--line)` (light) / `1px solid var(--line-strong)` (dark) |
| Border radius | `var(--radius-none)` |
| Padding | `var(--space-md)` |
| Shadow | `var(--shadow-md)` (more depth than feature card) |
| Layout | Flex column, `gap: var(--space-xs)` |
| Transition | `all var(--duration-normal) var(--ease-swiss)` |

#### Icon

| Property | Value |
|:---|:---|
| Size | `24×24px` |
| Color resting | `var(--amber)` |
| Stroke | `1.5px`, monoline style |
| Margin bottom | `var(--space-2xs)` |

#### Title

| Property | Value |
|:---|:---|
| Font | `var(--weight-semibold) var(--text-h3)/var(--leading-snug) var(--font-sans)` |
| Color | `var(--ink)` (light) / `var(--ink-on-dark)` (dark) |

#### Description

| Property | Value |
|:---|:---|
| Font | `var(--typo-body)` |
| Color | `var(--muted)` (light) / `var(--muted-on-dark)` (dark) |
| Line height | `var(--leading-normal)` |

#### Link

| Property | Value |
|:---|:---|
| Font | `var(--typo-label)` |
| Color | `var(--amber)` |
| Letter spacing | `var(--tracking-wider)` |
| Text transform | `uppercase` |
| Margin top | `var(--space-xs)` |
| Hover | Underline or arrow slide-right |

### Grid Placement

| Size | Columns | Notes |
|:---|:---|:---|
| Standard | `span 1` (4 cols of 12) | Default |
| Wide | `span 2` (8 cols) | Featured use case |

### States

| State | Background | Border | Shadow | Transform |
|:---|:---|:---|:---|:---|
| Resting | `var(--paper)` | `var(--line)` | `var(--shadow-md)` | — |
| Hover | `var(--elevated)` | `var(--ink)` | `var(--shadow-lg)` | `translateY(-2px)` |
| Active | `var(--elevated)` | `var(--ink)` | `var(--shadow-md)` | `scale(0.98)` |

### Tokens Used
`--paper`, `--surface-dark`, `--elevated`, `--line`, `--line-strong`, `--ink`, `--ink-on-dark`, `--muted`, `--muted-on-dark`, `--amber`, `--font-sans`, `--text-h3`, `--text-body`, `--text-label`, `--tracking-wider`, `--space-md`, `--space-xs`, `--space-2xs`, `--radius-none`, `--shadow-md`, `--shadow-lg`, `--ease-swiss`

---

## 12. Ecosystem Badge

### Purpose
Integration badges for the ecosystem section — frameworks, LLM providers, deployment targets. Pill-shaped with dot, dark surface, subtle border.

### Anatomy

```
●  LangChain
[───────────]
```

### Spec

| Property | Value |
|:---|:---|
| Shape | Pill — `border-radius: var(--radius-full)` |
| Background | `var(--surface-dark)` (both modes typically dark) |
| Border | `1px solid var(--line-strong)` |
| Padding | `var(--space-2xs) var(--space-sm)` |
| Display | `inline-flex`, `align-items: center` |
| Gap | `var(--space-2xs)` |
| Font | `var(--typo-label)` |
| Color | `var(--ink-on-dark)` |
| Letter spacing | `var(--tracking-wide)` |
| Height | `32px` |
| Max width | `fit-content` |
| White-space | `nowrap` |

#### Dot

| Property | Value |
|:---|:---|
| Size | `6×6px` |
| Border radius | `50%` |
| Color | `var(--amber-on-dark)` |

### States

| State | Background | Border | Dot Color |
|:---|:---|:---|:---|
| Resting | `var(--surface-dark)` | `var(--line-strong)` | `var(--amber-on-dark)` |
| Hover | `var(--surface-dark-alt)` | `var(--amber-on-dark)` | `var(--amber-on-dark)` |
| Active | `var(--surface-dark-alt)` | `var(--amber-on-dark)` | `var(--amber-on-dark)` |

### Tokens Used
`--surface-dark`, `--surface-dark-alt`, `--line-strong`, `--ink-on-dark`, `--amber-on-dark`, `--text-label`, `--tracking-wide`, `--radius-full`, `--space-2xs`, `--space-sm`

---

## 13. CTA Section

### Purpose
Final call-to-action section — full-width gradient background, centered content, prominent amber CTA with glow. Drives conversion.

### Anatomy

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                                                             │
│      Build something great with VantaDB                     │
│                                                             │
│      Start building in minutes — zero servers required.     │
│                                                             │
│           [ Get Started    →  ]                             │
│                                                             │
│                                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | Gradient: `linear-gradient(135deg, var(--surface-dark-alt) 0%, var(--surface-dark) 50%, #0d0d0d 100%)` |
| Padding | `var(--space-4xl) var(--space-sm)` |
| Layout | `flex`, `flex-direction: column`, `align-items: center`, `text-align: center` |
| Gap | `var(--space-md)` |
| Border top | `1px solid var(--line-strong)` |
| Border bottom | `1px solid var(--line-strong)` |

#### Title

| Property | Value |
|:---|:---|
| Font | `var(--weight-bold) var(--text-display)/var(--leading-tight) var(--font-sans)` |
| Color | `var(--ink-on-dark)` |
| Max width | `640px` |

#### Description

| Property | Value |
|:---|:---|
| Font | `var(--typo-lead)` |
| Color | `var(--muted-on-dark)` |
| Max width | `480px` |
| Line height | `var(--leading-normal)` |

#### CTA Button

| Property | Value |
|:---|:---|
| Spec | Same as Primary Button (Section 4) |
| Font size | `var(--text-lead)` (slightly larger) |
| Padding | `var(--space-sm) var(--space-xl)` |
| Shadow | `var(--shadow-amber)` — glow on dark surface is more visible |

### States

| State | Change |
|:---|:---|
| Resting | As above |
| Hover | Button glow intensifies: `0 8px 30px var(--amber-glow)` |
| Enter viewport | Clip-path mask reveal: `400ms`, `var(--ease-out)` |

### Tokens Used
`--surface-dark-alt`, `--surface-dark`, `--line-strong`, `--ink-on-dark`, `--muted-on-dark`, `--amber`, `--amber-on-light`, `--amber-glow`, `--shadow-amber`, `--font-sans`, `--text-display`, `--text-lead`, `--tracking-wider`, `--space-4xl`, `--space-sm`, `--space-md`, `--space-xl`, `--space-sm`, `--radius-none`, `--ease-swiss`, `--ease-out`

---

## 14. Footer

### Purpose
Site footer — brand identity, navigation links, social links, and copyright. Dark OLED background.

### Anatomy

```
┌─────────────────────────────────────────────────────────────┐
│  ◆ VantaDB         Docs     Engine     Resources            │
│  [tagline]         Get      Benchmarks Blog                 │
│                    Started  Pricing    GitHub                │
│                                         Discord             │
│  ─────────────────────────────────────────────────────────  │
│  © 2026 VantaDB     MIT License          GitHub ★           │
└─────────────────────────────────────────────────────────────┘
```

### Spec

| Property | Value |
|:---|:---|
| Background | `var(--surface-dark-alt)` (OLED `#111111`) |
| Padding | `var(--space-2xl) var(--space-sm) var(--space-lg)` |
| Max width | `var(--content-wide)`, centered |
| Layout | Grid — 4 columns on desktop, 2 on tablet, 1 on mobile |

#### Brand Column

| Property | Value |
|:---|:---|
| Logo | `48×48px` (dark variant) |
| Tagline | `var(--typo-caption)`, `var(--muted-on-dark)`, margin-top `var(--space-2xs)` |
| Max width | `240px` |

#### Link Columns

| Property | Value |
|:---|:---|
| Column title | `var(--typo-label)`, `var(--tracking-wider)`, ALL CAPS, `var(--ink-on-dark)` |
| Column title margin | `0 0 var(--space-sm)` |
| Links | `var(--typo-body)`, `var(--muted-on-dark)` |
| Link padding | `var(--space-3xs) 0` |
| Link hover | `var(--ink-on-dark)`, `var(--duration-fast) var(--ease-swiss)` |
| Link gap | `var(--space-2xs)` |

#### Divider

| Property | Value |
|:---|:---|
| Height | `1px` |
| Background | `rgba(255, 255, 255, 0.08)` |
| Margin | `var(--space-lg) 0 var(--space-md)` |

#### Bottom Bar

| Property | Value |
|:---|:---|
| Layout | Flex row, `justify-content: space-between`, `align-items: center` |
| Copyright | `var(--typo-caption)`, `var(--muted-on-dark)` |
| License link | `var(--typo-caption)`, `var(--muted-on-dark)`, hover → `var(--ink-on-dark)` |
| GitHub star | Inline badge/link |

### Grid Layout

| Breakpoint | Columns |
|:---|:---|
| ≥ 1024px | 4 columns (Brand + 3 link groups) |
| 768px – 1023px | 2 columns (Brand + links wrap) |
| < 768px | 1 column, stacked |

### States

| Element | Resting | Hover |
|:---|:---|:---|
| Brand logo | `opacity: 0.9` | `opacity: 1` |
| Link text | `var(--muted-on-dark)` | `var(--ink-on-dark)` |
| Social icon | `var(--muted-on-dark)` | `var(--ink-on-dark)` |

### Tokens Used
`--surface-dark-alt`, `--ink-on-dark`, `--muted-on-dark`, `--amber-on-dark`, `--font-sans`, `--font-mono`, `--text-body`, `--text-caption`, `--text-label`, `--tracking-wider`, `--space-2xl`, `--space-lg`, `--space-md`, `--space-sm`, `--space-2xs`, `--space-3xs`, `--ease-swiss`

---

## 15. Implementation Rules

1. **No hardcoded values** — every color, font-size, radius, shadow, and spacing must reference a token from `TOKEN_SYSTEM.md`.
2. **Mobile-first CSS** — base styles target `< 768px`, then override upward with `min-width` breakpoints.
3. **Dark mode** — use `[data-theme="dark"]` selector to override color tokens; component structure stays identical.
4. **Motion** — transitions use `var(--ease-swiss)` for UI, `var(--ease-out)` for reveals. See `MOTION_CHOREOGRAPHY.md`.
5. **Icons** — monoline SVG, `1.5px` stroke, `stroke-linecap: square`. See `ICON_SYSTEM.md`.
6. **Accessibility** — all interactive elements must have `focus-visible` outlines. Minimum contrast ratio `4.5:1`.
7. **No generic selects** — all `<select>` elements must be custom-styled to match the design system.
