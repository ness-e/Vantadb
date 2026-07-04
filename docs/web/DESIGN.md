# DESIGN.md — VantaDB Website Redesign

> **Single source of truth** for all visual, motion, and layout decisions.
> Supersedes `design/DiseñoNuevo.md` as the master document.
> Version: 1.0 | 2026-07-04
> Style: **Amber on Near-Black — Warm Technical Minimal**

---

## Table of Contents

1. [Design Philosophy & Principles](#1-design-philosophy--principles)
2. [Color System](#2-color-system)
3. [Typography System](#3-typography-system)
4. [Layout System](#4-layout-system)
5. [Component Design Patterns](#5-component-design-patterns)
6. [Motion Principles](#6-motion-principles)
7. [Section-by-Section Specifications](#7-section-by-section-specifications)
8. [Anti-Slop Rules & Pre-Flight Checklist](#8-anti-slop-rules--pre-flight-checklist)
9. [Accessibility Requirements](#9-accessibility-requirements)

---

## 1. Design Philosophy & Principles

### 1.1 Positioning Summary

| Attribute | Value |
|---|---|
| **Product** | Embedded vector database. Rust core. 2MB binary. HNSW + BM25 hybrid search. WAL durability. PyO3 Python bridge. Zero servers. |
| **Target audience** | AI agent developers, local RAG builders, edge/embedded developers, Rust/Python devs |
| **Competitive differentiation** | vs Chroma: no server needed. vs LanceDB: hybrid search. vs Pinecone: embedded, free. |
| **Unique value** | Embedded-first, zero infrastructure, 1.2ms latency, 2MB binary |

### 1.2 Color Psychology

- **Amber #ff5500** on near-black #111111 creates a **"warm technical"** aesthetic.
- Orange is underrepresented in SaaS (~5%), differentiating VantaDB from the blue/purple/teal cluster of competitors.
- Amber signals **energy, action, approachable power** — not cold infrastructure, but tooling that moves.
- Near-black (#111111) conveys **depth, seriousness, premium quality** (70% of dev tools use dark).
- White (#ffffff) provides **breathing room and contrast** — used sparingly for content sections that need to feel open.
- Muted (#5a5a5a) handles secondary text, metadata, labels without competing for attention.

### 1.3 Design Principles

| # | Principle | Manifestation |
|---|---|---|
| 1 | **Editorial confidence** | Typography leads. Headings are large but never shout. Whitespace is structural, not decorative. The page reads like a premium technology monograph. |
| 2 | **Motion as language** | Every animation serves comprehension — scroll reveals show relationships, micro-interactions acknowledge intent. If a motion doesn't help the user understand faster, cut it. |
| 3 | **Developer-first, always** | The install command is the hero CTA. Code snippets are real and runnable. Technical depth is a feature. Show the API, don't describe it. |
| 4 | **Dark with purpose** | Dark isn't default because "tools look cool dark." It's dark because AI engineers live in dark terminals, dark IDEs, dark docs. The amber accent echoes a terminal cursor, a compile glow, a signal in the dark. |
| 5 | **Differentiation through restraint** | VantaDB's advantage is simplicity (one engine, one binary). The design embodies that: fewer sections than competitors, clearer copy, faster path to install. Every element that doesn't help a developer decide or install is removed. |

### 1.4 Design Persona

VantaDB is a **senior infrastructure engineer who also teaches at a code bootcamp**. Quietly brilliant. Generous with knowledge. Intolerant of unnecessary complexity. Excited about AI, skeptical of hype. They answer questions on Discord at 11pm on a Saturday.

The site must make visitors feel:
- **Trust**: built by people who understand databases and AI at a systems level
- **Curiosity**: I want to see how this works under the hood
- **Clarity**: I immediately understand the value proposition and install path

---

## 2. Color System

### 2.1 Brand Palette

| Token | Hex | OKLCH | Usage | Contrast (on bg) |
|---|---|---|---|---|
| `--amber` | `#ff5500` | `oklch(0.62 0.22 40)` | CTAs, hover states, data highlights, active indicators, code syntax strings | 4.5:1 on #111111 (AA) |
| `--near-black` | `#111111` | `oklch(0.13 0.005 265)` | Primary background (70% of page area) | — |
| `--white` | `#ffffff` | `oklch(1 0 0)` | Text on dark, card backgrounds on light sections | 15:1 on #111111 |
| `--muted` | `#5a5a5a` | `oklch(0.42 0.01 265)` | Secondary text, metadata, labels, inactive states | 4.6:1 on #111111 (AA) |

### 2.2 Dark Section Tokens (70% of page)

| Token | Value | Usage |
|---|---|---|
| `--bg-dark` | `#111111` | Primary dark section background |
| `--bg-dark-alt` | `#1a1a1a` | Alternate dark section (slightly lifted) |
| `--text-dark` | `#f5f5f5` | Primary text on dark backgrounds |
| `--text-dark-muted` | `#808080` | Secondary text on dark backgrounds |
| `--border-dark` | `rgba(255,255,255,0.08)` | Borders on dark backgrounds |
| `--amber-dark-dim` | `rgba(255,85,0,0.10)` | Very subtle amber tint for hover states on dark |

### 2.3 Light Section Tokens (30% of page)

| Token | Value | Usage |
|---|---|---|
| `--bg-light` | `#ffffff` | Light section background (breathing sections) |
| `--text-light` | `#111111` | Primary text on light backgrounds |
| `--text-light-muted` | `#5a5a5a` | Secondary text on light backgrounds |
| `--border-light` | `rgba(0,0,0,0.10)` | Borders on light backgrounds |
| `--amber-light-dim` | `rgba(255,85,0,0.06)` | Very subtle amber tint for hover states on light |

### 2.4 Surface & Overlay Tokens

| Token | Value | Usage |
|---|---|---|
| `--surface-glass` | `rgba(17,17,17,0.85)` | Nav background |
| `--surface-card-dark` | `rgba(255,255,255,0.03)` | Card resting on dark sections |
| `--surface-card-dark-hover` | `rgba(255,255,255,0.06)` | Card hover on dark sections |
| `--terminal-bg` | `#0d0d0d` | Terminal/code block background |
| `--success` | `#00c853` | Positive indicators, benchmarks where VantaDB wins |
| `--danger` | `#ff1744` | Negative indicators, competitor wins in comparisons |

### 2.5 Usage Rules

1. **Amber is the single accent color.** No secondary accent. No purple, blue, teal, or green for decorative purposes.
2. **Amber used exclusively for**: CTAs (primary buttons), hover/focus states, active navigation links, data highlight values, code syntax strings, terminal output accents, index labels on hover, icon hover states.
3. **Amber never used for**: body text, secondary text, background fills, decorative elements, generic borders.
4. **95/5 Rule**: 95% of the page is near-black, white, and muted. 5% is amber.
5. **Dark-dominant rhythm**: sections are grouped 2-3 dark, then 1-2 light. Never 1:1 alternation.
6. **Consistency lock**: amber tone (#ff5500) is used across the entire page. No mid-page accent drift.

### 2.6 Contrast Verification

| Combination | Ratio | WCAG |
|---|---|---|
| Amber `#ff5500` on Near-black `#111111` | 4.5:1 | AA (large text) |
| White `#ffffff` on Near-black `#111111` | 15:1 | AAA |
| Muted `#5a5a5a` on Near-black `#111111` | 4.6:1 | AA |
| Amber `#ff5500` on White `#ffffff` | 3.3:1 | AA large text only |
| Near-black `#111111` on White `#ffffff` | 15:1 | AAA |

---

## 3. Typography System

### 3.1 Font Families

| Role | Font | Fallback | Weight Used | Prohibited Alternatives |
|---|---|---|---|---|
| **Display** | Space Grotesk | sans-serif | 700 (bold) only | Inter, Roboto, Arial, Open Sans, Helvetica |
| **Body** | Outfit | sans-serif | 400 (regular), 600 (semibold) | Inter, Roboto, Arial, Open Sans, Helvetica |
| **Code/Label** | JetBrains Mono | monospace | 400 (regular), 600 (bold) | Fira Code, Cascadia Code, Menlo |

### 3.2 Type Scale

| Token | Size | Weight | Letter-spacing | Line-height | Usage |
|---|---|---|---|---|---|
| `--text-hero` | `clamp(3.5rem, 7vw, 6.5rem)` | 700 Space Grotesk | `-0.05em` | `0.95` | Primary hero headline |
| `--text-display` | `clamp(2.2rem, 4vw, 3.5rem)` | 700 Space Grotesk | `-0.04em` | `1.05` | Section titles |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | 600 Outfit | `-0.02em` | `1.2` | Card titles, feature names |
| `--text-body` | `1.05rem` | 400 Outfit | `-0.01em` | `1.65` | Running text, descriptions |
| `--text-small` | `0.875rem` | 400 Outfit | `normal` | `1.5` | Secondary descriptions |
| `--text-label` | `0.72rem` | 600 JetBrains Mono | `0.14em` | `1.2` | Labels, section headers, ALL CAPS |
| `--text-code` | `0.88rem` | 400 JetBrains Mono | `normal` | `1.5` | Code snippets, terminal output |
| `--text-metric` | `clamp(2.5rem, 5vw, 4rem)` | 700 Space Grotesk | `-0.03em` | `1` | Benchmark numbers, data highlights |

### 3.3 Typography Rules

1. **Left-aligned always** — exceptions only for isolated CTA blocks (Monolith section, max 1 per page).
2. `font-variant-numeric: tabular-nums` on ALL numeric data (benchmark numbers, metrics, prices).
3. Labels are ALL CAPS with tracking `0.14em` in JetBrains Mono 600 — never in body fonts.
4. Long descriptions rendered in `--muted` or `--text-dark-muted`, **never** in amber.
5. Code is always JetBrains Mono, never body fonts.
6. Headings never exceed 2 lines on desktop. Subtext never exceeds 20 words.
7. No serif fonts anywhere. Space Grotesk and Outfit are the only proportional families.

---

## 4. Layout System

### 4.1 Grid

```css
.vanta-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 0;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 24px;
}
```

- **12 columns**, no gap between columns (use padding inside cells).
- **Max-width**: 1200px. Full-width bleed sections (Monolith, Nav) exceed this.
- **Grid lines**: 1px borders serve as visible structural elements. Column borders are not decorative — they delineate information zones.

### 4.2 Spacing

| Token | Value | Usage |
|---|---|---|
| `--space-xs` | `8px` | Micro-spacing, icon padding |
| `--space-sm` | `16px` | Card padding (tight) |
| `--space-md` | `24px` | Card padding (default), button padding |
| `--space-lg` | `48px` | Section internal spacing |
| `--space-xl` | `96px` | Between major sections |
| `--space-2xl` | `160px` | Monolith CTA padding |
| `--space-3xl` | `240px` | Maximum vertical separation |

### 4.3 Section Rhythm

Dark and light sections are **grouped**, never alternated 1:1:

```
[DARK]     Hero
[DARK]     Social Proof / Metrics Strip
[DARK]     Features / Why Vanta
[LIGHT]    How It Works / Quickstart
[DARK]     Architecture
[DARK]     Performance / Benchmarks
[LIGHT]    Use Cases
[DARK]     Ecosystem
[DARK]     Final CTA + Footer
```

Rule: group 2-3 dark, then 1-2 light. Dark sections are dominant (70% of page height). Light sections are "breathing rooms."

### 4.4 Asymmetry Rules

1. **Every grid must have varying cell sizes.** No identical card grids anywhere.
2. One anchor cell per bento grid (a 2x2 or 3x1 cell that dominates).
3. No centered texts on asymmetric sections. Centered text is **only** for the Monolith CTA.
4. Section titles start at column 1-7 or 1-8, leaving intentional negative space on the right.
5. Max 1 eyebrow per 3 sections.

---

## 5. Component Design Patterns

### 5.1 Navigation

| Property | Value |
|---|---|
| Position | Fixed top |
| Height | 64px (max 80px desktop) |
| Background | `--surface-glass` + `backdrop-filter: blur(12px)` |
| Bottom border | `1px solid var(--border-dark)` |
| Layout | Logo left \| Links center \| CTA right |
| Link font | `--text-label` (JetBrains Mono 600, 0.72rem, ALL CAPS, tracking 0.14em) |
| Link color resting | `--text-dark-muted` (#808080) |
| Link color hover | `--white` (#ffffff), 100ms transition |
| Link color active | `--amber` (#ff5500) |
| CTA | Primary amber button "Get Started" |
| Mobile | Hamburger → panel with 1px border, links vertical stack |

### 5.2 Buttons

**Primary Button:**
| State | Style |
|---|---|
| Resting | `background: var(--amber)`, `color: #ffffff`, `border-radius: 0` |
| Hover | `background: #ffffff`, `color: #111111` |
| Active | `scale(0.97)` — 50ms, mechanical feel |
| Padding | `10px 24px`, text single-line |

**Ghost Button (on dark):**
| State | Style |
|---|---|
| Resting | `background: transparent`, `border: 1px solid var(--border-dark)` |
| Hover | `background: rgba(255,255,255,0.1)`, `border-color: #ffffff` |

**Ghost Button (on light):**
| State | Style |
|---|---|
| Resting | `background: transparent`, `border: 1px solid var(--border-light)` |
| Hover | `background: #111111`, `color: #ffffff` |

**All buttons:**
- `border-radius: 0` — enforced globally
- `box-shadow: none` — enforced globally
- Transition: `150ms var(--ease-swiss)`
- Must fit 1 line (design-taste §4.5)
- No duplicate CTA intent per page (design-taste §4.5)

### 5.3 Cards

| Property | Value |
|---|---|
| Background (dark section) | `var(--surface-card-dark)` — `rgba(255,255,255,0.03)` |
| Background (light section) | `#ffffff` |
| Border (dark) | `1px solid var(--border-dark)` |
| Border (light) | `1px solid var(--border-light)` |
| Hover (dark) | Background `var(--surface-card-dark-hover)`, border `rgba(255,255,255,0.2)` |
| Hover (light) | Border `#111111` |
| Padding | `24px` default |
| Border radius | `0` (max `4px` for terminal blocks) |
| Shadow | `box-shadow: none` — **never** |
| Index label | `[01]` format, `--text-label`, resting `--muted` → hover `--amber` |

### 5.4 Terminal / Code Block

| Property | Value |
|---|---|
| Background | `--terminal-bg` (#0d0d0d) |
| Border | `1px solid var(--border-dark)` |
| Border radius | max `4px` |
| Header | Three dots (unfilled circles) in muted gray + title in `--text-label` |
| Body font | JetBrains Mono, `0.88rem` |
| Syntax | Keywords `#ffffff`, strings `--amber`, comments `--muted` |
| Output indicator | `border-left: 2px solid var(--amber)` |
| No shadow | `box-shadow: none` |
| Typewriter speed | 30ms per character |

### 5.5 Footer

| Property | Value |
|---|---|
| Background | `#111111` |
| Grid | 5 columns |
| Link color resting | `#808080` |
| Link color hover | `#ffffff` |
| Column titles | `--text-label` (JetBrains Mono, ALL CAPS, 0.72rem, tracking 0.14em) |
| Dividers | `1px solid var(--border-dark)` |
| Bottom bar | Logo mark + copyright + GitHub link |

**Columns**: Product | Solutions | Developers | Resources | Company — full page list per `SUB_PAGE_PATTERNS.md`.

### 5.6 Index Labels

Used on cards, benchmark cells, feature items, and use cases. Format: `[01]`, `[02]`, etc. Always positioned at top-left of the card. Rendered in `--text-label` (JetBrains Mono 600, 0.72rem, ALL CAPS, tracking 0.14em). Color `--muted` resting, `--amber` on parent hover.

---

## 6. Motion Principles

### 6.1 Decision Framework (emil-design-eng)

Before animating **any** element, answer in order:

1. **Should it animate?** If the user sees it 100+ times/day → No. If occasional → Yes.
2. **What is the purpose?** Feedback, spatial consistency, state indication, preventing jarring changes.
3. **Which easing?** Entering → `ease-out`. Moving → `ease-in-out`. Hover → `ease`. Constant → `linear`.
4. **How fast?** UI < 300ms. Buttons 100-160ms. Tooltips 125-200ms. Modals 200-500ms.

### 6.2 Easing Curves (Mandatory — never CSS defaults)

```css
--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);         /* Mechanical, sharp */
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);           /* Strong ease-out for UI */
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);       /* For on-screen movement */
```

**Prohibited**: `ease-in` for UI (feels sluggish). `linear` or `ease-in-out` CSS defaults. bounce, elastic, soft spring.

### 6.3 Scroll Animations

| Animation | Technique | Duration | Stagger |
|---|---|---|---|
| Hero title reveal | `clip-path: inset(0 0 100% 0)` → `inset(0)` | 400ms | — |
| Benchmark cells | Expand from grid edge (scale 0→1) | 300ms | 60ms |
| Count-up numbers | `gsap.utils.interpolate` | 200ms | — |
| Core Engine features | `clip-path: inset(100% 0 0 0)` → `inset(0)` | 250ms | 100ms |
| SVG stroke draw | `stroke-dashoffset` → 0 | 800ms | — |
| Architecture layers | TranslateY separation (exploded view) | 400ms | Pin during scroll |
| Ecosystem grid | Expand from grid lines | 200ms | 60ms |
| Use case cards | Clip-path mask from left | 300ms | — |
| Monolith CTA | Clip-path reveal | 400ms | — |

### 6.4 Micro-interactions

| Element | Interaction |
|---|---|
| Active button | `scale(0.97)` — mechanical feel, 50ms |
| Card hover | Border `--border-light` → `#111111` (or dark equivalent), 100ms |
| Index label | Color `--muted` → `--amber` on parent hover, 150ms |
| Icon hover | Color `--muted` → `--amber`, 150ms |
| Nav link hover | Color `#808080` → `#ffffff`, 100ms |

### 6.5 Performance (Mandatory)

- Animate **only** `transform` and `opacity` — never `top`, `left`, `width`, `height`.
- `backdrop-blur` only on fixed/sticky elements (nav).
- `will-change: transform` only on actively animating elements.
- `prefers-reduced-motion`: all animations off, elements visible immediately.

---

## 7. Section-by-Section Specifications

### 7.1 NAV

| Property | Value |
|---|---|
| Background | `rgba(17,17,17,0.85)` + `backdrop-filter: blur(12px)` |
| Height | 64px |
| Border | `1px solid var(--border-dark)` bottom |
| Layout | Logo (left) \| Docs / Engine / Architecture / Pricing (center) \| "Get Started" CTA (right) |
| Links | JetBrains Mono 600, 0.72rem, ALL CAPS, tracking 0.14em, `#808080` resting |
| Hover | `#ffffff`, 100ms |
| Active/current | `--amber` |
| Mobile | Hamburger → right-side panel, 280px, `#111111` bg, 1px left border |

### 7.2 HERO

**Background**: `#111111` (dark)
**Eyebrow**: None (no eyebrow in hero — first eyebrow appears in section 4)
**Layout**: Grid 12 cols — headline + CTAs in cols 1-8, abstract visual/terminal in cols 9-12

**Elements:**
- **Headline**: "Embedded Vector Database for AI Agents." in `--text-hero` (Space Grotesk 700, `clamp(3.5rem, 7vw, 6.5rem)`). Max 2 lines.
- **Subtext**: "1.2ms queries. 2MB binary. Zero servers. One `pip install`." in `--text-body`, `--text-dark-muted`. Max 20 words.
- **Primary CTA**: "pip install vantadb" — amber button.
- **Ghost CTA**: "Read the Docs" — ghost button.
- **Visual**: Terminal window or code block preview in cols 9-12 (not a 3D graphic).
- **Input animation**: None (no typewriter, no particles, no 3D wireframe).

**Anti-slop verification:**
- Hero fits initial viewport (no scroll needed to see CTA)
- H1 max 2 lines
- Subtext max 20 words
- Max 4 text elements (headline + subtext + 2 CTAs)
- No "used by" logos inside hero
- Top padding max 6rem
- No feature list, no pricing teaser, no eyebrow

### 7.3 SOCIAL PROOF / METRICS STRIP

**Background**: `#111111` (dark) or `#1a1a1a` (slight lift)
**Layout**: Full-width strip with 3-4 metric blobs, inline (no cards)

**Metrics:**
```
1.2ms       2MB         0         99.8%
p50 query   binary      servers   Recall@10
latency     footprint              (HNSW)
```

**Style:**
- Numbers: `--text-metric` (Space Grotesk 700, `clamp(2.5rem, 5vw, 4rem)`), `--white`
- Labels: `--text-label` (JetBrains Mono 600, 0.72rem, ALL CAPS, tracking 0.14em), `--text-dark-muted`
- Separators: `1px` vertical lines between metrics, `var(--border-dark)`
- No cards, no containers — metrics float on the dark background
- Count-up animation: 200ms, `tabula-r-nums`

### 7.4 FEATURES / WHY VANTA

**Background**: `#111111` (dark)
**Section label**: `[WHY VANTA]` in `--text-label`, `--amber`
**Title**: "Everything you need, nothing you don't." in `--text-display`, `--white`
**Subtitle**: One embedded engine replacing the multi-database stack. in `--text-body`, `--text-dark-muted`

**Grid**: 6 features, varying cell sizes:

```
┌─────────────┬─────────────┐
│  2×1        │  1×1        │
│  Hybrid     │  Rust       │
│  Search     │  Core       │
├─────────────┼──────┬──────┤
│  1×1        │ 1×1  │ 1×1  │
│  WAL Crash  │ Zero │ PyO3 │
│  Recovery   │ Copy │ Bind │
└─────────────┴──────┴──────┘
```

**Feature cells content:**
| # | Feature | Metric | Description |
|---|---|---|---|
| 01 | Hybrid Search (HNSW + BM25) | 0.998 Recall@10 | Vector + full-text in a single query with RRF fusion |
| 02 | Rust Core | 2MB binary | Memory-safe, zero-cost abstractions, compiled performance |
| 03 | WAL Durability | Zero data loss | Write-Ahead Log with CRC32C auto-healing crash recovery |
| 04 | Zero-Copy deserialization | No allocation overhead | Serde-backed, mmap-friendly, allocation-free reads |
| 05 | PyO3 Native Bridge | <1µs overhead | Native Python bindings with zero serialization overhead |
| 06 | No Server Process | `pip install` → running | Embed directly in your application process |

**Each cell**: `--surface-card-dark` bg, `1px solid var(--border-dark)` border, 24px padding, index label `[01]`–`[06]`.

**No eyebrow on this section** (eyebrow was on section 4). Max 1 eyebrow per 3 sections.

### 7.5 HOW IT WORKS / QUICKSTART

**Background**: `#ffffff` (light — breathing section)
**Section label**: `[QUICKSTART]` in `--text-label`, `--amber`
**Title**: "From zero to query in 60 seconds." in `--text-display`, `--text-light`
**Layout**: 2-column grid — steps list (4 cols) + terminal (8 cols)

**Steps (left column):**
| Step | Code |
|---|---|
| `[01] Install` | `pip install vantadb-py` |
| `[02] Connect` | `db = vanta.connect("./memory.vdb")` |
| `[03] Store` | `db.store("key", embedding, metadata)` |
| `[04] Query` | `db.query("find similar", top_k=5)` |

- Active step: number `--amber`, left border `2px solid var(--amber)`
- Inactive steps: number `--muted`, no border
- Click step → terminal jumps to that step

**Terminal (right column):**
- Background: `--terminal-bg`, border `1px solid var(--border-light)`
- Header: 3 terminal dots + "terminal" label
- Code typewriter 30ms/char
- Output appears instantly with `border-left: 2px solid var(--amber)`
- Syntax: keywords `#ffffff`, strings `--amber`, comments `#5a5a5a`

**Auto-play**: Sequential. Step 01 types → output appears → step 02 activates → ... → after step 04, restart after 3s delay.

### 7.6 ARCHITECTURE

**Background**: `#111111` (dark)
**Section label**: `[ARCHITECTURE]` in `--text-label`, `--amber`
**Title**: "Inside the Engine." in `--text-display`, `--white`
**Layout**: 6-stage pipeline with numbered steps, horizontal flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Python  │ ──→ │   PyO3   │ ──→ │   Rust   │ ──→ │   Query  │ ──→ │  Storage │ ──→ │   Disk   │
│   App    │     │  Bridge  │     │  Engine  │     │  Engine  │     │   Layer  │     │  (mmap)  │
└──────────┘     └──────────┘     └────┬─────┘     └──────────┘     └──────────┘     └──────────┘
                                       │
                                ┌──────┴──────┐
                                │  HNSW · BM25 │
                                │  · WAL · MCP │
                                └─────────────┘
```

- Each stage is a card with `--surface-card-dark` bg, `1px solid var(--border-dark)` border
- Arrows: SVG monoline 1.5px stroke, `--muted`
- Stage labels: JetBrains Mono 600, 0.72rem, ALL CAPS
- Sub-modules in the Rust Engine: HNSW, BM25, WAL, MCP Server
- Hover on any stage: border → `rgba(255,255,255,0.2)`, rest dim to `opacity: 0.3`
- Scroll: exploded SVG view (layers separate vertically as user scrolls through)

### 7.7 PERFORMANCE / BENCHMARKS

**Background**: `#1a1a1a` or `#111111` (dark)
**Section label**: `[BENCHMARKS]` in `--text-label`, `--amber`
**Title**: "Numbers don't lie." in `--text-display`, `--white`
**Layout**: Comparison table + benchmark comparison callout

**Benchmark table:**
| Metric | VantaDB | Traditional | Delta |
|---|---|---|---|
| p99 Query Latency | 0.8ms | 12ms (Pinecone) | `↓ 15×` amber |
| Binary Size | 2MB | 180MB (Chroma) | `↓ 90×` amber |
| External Dependencies | 0 | 12+ (Pinecone) | `↓ Zero` amber |
| Setup Time | `pip install` | 45 min config (Weaviate) | `↓ Instant` amber |
| Search Types | Hybrid (BM25+HNSW) | Single mode | `↓ Full-spectrum` |
| Crash Recovery | WAL auto | Manual backup | `↓ Zero-loss` |
| Cost (100M vectors/mo) | $0 (embedded) | $1,200+ (Pinecone) | `↓ 100%` amber |

**Style:**
- `--text-metric` on VantaDB column numbers
- Delta indicators: `↓` in `--amber` (VantaDB wins), `↑` in `--danger` (competitor wins)
- Table borders: `1px solid var(--border-dark)`
- Header row: JetBrains Mono 600, 0.72rem, ALL CAPS, `--text-dark-muted`
- Numeric cells: `font-variant-numeric: tabular-nums`

**Eyebrow**: None (no eyebrow here — max 1 per 3 sections).

### 7.8 USE CASES

**Background**: `#ffffff` (light — breathing section)
**Section label**: `[USE CASES]` in `--text-label`, `--amber`
**Title**: "Built for real AI workflows." in `--text-display`, `--text-light`
**Layout**: Bento grid with varying cell sizes:

```
┌───────────────────────┬────────────────┐
│  2×1                  │  1×1           │
│  AI Agent Memory      │  Local RAG     │
│  "Persistent memory   │  Pipeline      │
│   for LLM agents"     │                │
├──────────┬────────────┴────────────────┤
│  1×1     │  2×1                        │
│  IDE     │  Offline Knowledge Base     │
│  Code    │  "Edge-deployed semantic     │
│  Intel.  │   search without internet"   │
└──────────┴─────────────────────────────┘
```

**Each cell:**
- Background: `#ffffff`, border `1px solid var(--border-light)`
- Title: `--text-title` (Outfit 600)
- Description: `--text-body`, `--text-light-muted`
- Index label: `[01]`–`[04]`, `--muted` → `--amber` on hover
- Hover: border → `#111111`
- Maximum 4 use cases. No identical cell sizes.

### 7.9 ECOSYSTEM

**Background**: `#111111` (dark)
**Section label**: `[ECOSYSTEM]` in `--text-label`, `--amber`
**Title**: "Works with everything you use." in `--text-display`, `--white`
**Layout**: Grid of integration badges (24 total, arranged in category groups)

**Categories:**
| Category | Integrations |
|---|---|
| **Frameworks** | LangChain, LlamaIndex, CrewAI, AutoGen, Haystack, DSPy |
| **LLM Providers** | OpenAI, Anthropic, Google, Ollama, Mistral, Together |
| **Memory/Agents** | Mem0, Letta, GraphRAG, memGPT, RAGFlow, LightRAG |
| **Deployment** | Docker, Kubernetes, Vercel, AWS Lambda, Railway, Fly.io |

**Each badge:**
- Icon (monoline SVG 1.5px) + name in `--text-label`
- Resting: icon `--muted`, name `--text-dark-muted`
- Hover: icon `--amber`, background `var(--amber-dark-dim)`, border `rgba(255,255,255,0.2)`
- Hover transition: 150ms `var(--ease-swiss)`
- No cards — badges float on the dark background with `1px` border separation

### 7.10 FINAL CTA (The Monolith)

**Background**: Full-width dark gradient: `#111111` → `#0d0d0d` (subtle darkening)
**Layout**: Centered (exception to left-align rule — isolated CTA)
**Padding**: `160px` vertical

**Elements:**
- Command: `pip install vantadb-py` in `--text-hero` (Space Grotesk 700, `#ffffff`)
- Tagline: "Zero servers. One line. Infinite context." in `--text-body`, `#808080`
- CTA button: "Get Started" — primary amber, centered
- Secondary link: "Read Documentation →" in `#808080`, hover `#ffffff`

**Interaction:**
- Reveal: clip-path mask, 400ms, on viewport enter
- Cursor blink (CSS animation, 500ms) after the command text
- No eyebrow, no feature list, no metrics — just the command

### 7.11 FOOTER

See [Component Design Patterns §5.5](#55-footer) above and detailed spec in `design/COMPONENT_LIBRARY.md`.

---

## 8. Anti-Slop Rules & Pre-Flight Checklist

### 8.1 Anti-Slop Rules

| # | Rule | Source |
|---|---|---|
| 1 | **No identical card grids** — every grid has varying cell sizes | design-taste §4.7 |
| 2 | **Max 1 eyebrow per 3 sections** — do not overuse section labels | design-taste §4.7 |
| 3 | **No centered text on asymmetric sections** — only Monolith CTA exception | design-taste §4.3 |
| 4 | **No purple/blue gradient** — zero gradient backgrounds allowed | design-taste §4.2 |
| 5 | **No generic stock imagery** — no photos, no 3D renders, no illustrations | high-end §2 |
| 6 | **Cards only when elevation communicates hierarchy** — no decorative cards | impeccable §4 |
| 7 | **One accent color (amber)** — used sparingly on CTAs and data highlights | design-taste §4.2 |
| 8 | **No split-header as default** (left H + right P) — vary the layout | design-taste §4.7 |
| 9 | **No generic AI copy** — never "Elevate", "Seamless", "Unleash", "Next-Gen", "Revolutionize" | minimalist §2 |
| 10 | **No 3-card feature rows** — every feature grid has varying cell sizes | impeccable §4 |
| 11 | **Each layout family max 1 time per page** — bento, terminal, table, badges, cards each used once | design-taste §4.7 |
| 12 | **Bento cells = exact content count** — no empty filler cells | design-taste §4.7 |
| 13 | **No bounce, elastic, or soft spring animations** — mechanical easing only | emil-design-eng §3 |
| 14 | **No box-shadow anywhere** — depth via background/border contrast only | industrial-brutalist §4 |
| 15 | **No border-radius > 0 on buttons** — 0px enforced globally | industrial-brutalist §5 |
| 16 | **No typewriter effect in hero** — typewriter reserved for quickstart terminal only | design-taste §4.7 |
| 17 | **No ball/animated particles** — no floating dots, no ambient particle systems | design-taste §4.7 |
| 18 | **Nav single-line on desktop** — must never wrap | design-taste §4.7 |
| 19 | **No duplicate CTA intent per page** — each page has one primary action | design-taste §4.5 |
| 20 | **All motion justifies its existence** — apply the emil decision framework before animating | emil-design-eng §2 |

### 8.2 Pre-Flight Checklist

Run this against every page before shipping:

**Typography:**
- [ ] All display text uses Space Grotesk 700
- [ ] All body text uses Outfit 400 or 600
- [ ] All code/labels use JetBrains Mono
- [ ] `font-variant-numeric: tabular-nums` on numeric data
- [ ] Labels are ALL CAPS with 0.14em tracking
- [ ] No serif fonts present
- [ ] Text left-aligned (except Monolith CTA)
- [ ] No Inter, Roboto, Arial, Open Sans, Helvetica

**Color:**
- [ ] Amber (#ff5500) is the only accent color
- [ ] No purple, blue, teal, green decorative elements
- [ ] Amber used only for CTAs, hovers, active states, data highlights
- [ ] Contrast: body text ≥ 4.5:1, large text ≥ 3:1
- [ ] No gradients anywhere
- [ ] Box-shadow: none on every element

**Layout:**
- [ ] 12-column grid respected
- [ ] Max-width 1200px on content sections
- [ ] Section spacing ≥ 96px between major sections
- [ ] No identical cell sizes in any grid
- [ ] One anchor cell per bento grid
- [ ] Dark/light grouped 2-3 dark → 1-2 light
- [ ] Hero fits viewport without scroll
- [ ] H1 max 2 lines, subtext max 20 words

**Motion:**
- [ ] All easing uses custom curves (not CSS defaults)
- [ ] Animations ≤ 300ms for UI elements
- [ ] Only `transform` and `opacity` animated
- [ ] `prefers-reduced-motion` respected
- [ ] No bounce, elastic, spring
- [ ] No `ease-in` for UI
- [ ] Stagger 30-80ms max
- [ ] `backdrop-blur` only on fixed elements

**Components:**
- [ ] Border-radius: 0 on buttons
- [ ] Border-radius max 4px on terminal blocks
- [ ] No decorative cards — every card has a function
- [ ] Button text fits 1 line
- [ ] Touch targets ≥ 44x44px on mobile
- [ ] Nav single-line on desktop
- [ ] Navigation active state on current page

---

## 9. Accessibility Requirements

### 9.1 WCAG Compliance Targets

| Criterion | Target | Notes |
|---|---|---|
| Contrast ratio (body text) | ≥ 4.5:1 (AA) | Verified for all text elements |
| Contrast ratio (large text) | ≥ 3:1 (AA) | ≥ 18px bold or ≥ 24px regular |
| Contrast ratio (UI components) | ≥ 3:1 (AA) | Buttons, borders, focus indicators |
| Keyboard navigation | Full | All interactive elements reachable and operable |
| Focus indicators | Visible | `outline: 2px solid var(--amber)` or similar |
| Touch targets | ≥ 44x44px | All buttons, links, interactive cards |
| Screen reader support | Semantic HTML | Proper landmarks, headings hierarchy, ARIA where needed |

### 9.2 Keyboard Navigation

- All interactive elements reachable via Tab in logical order.
- Visible focus indicator on all focusable elements: amber outline or background change.
- Skip-to-content link as first focusable element.
- Dropdown menus open/close with Enter/Space/Escape.
- No keyboard traps.

### 9.3 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  .reveal-mask, .reveal-expand, .reveal-draw {
    animation: none;
    opacity: 1;
    clip-path: none;
  }
  .count-up {
    opacity: 1;
  }
  .terminal-typewriter {
    animation: none;
  }
}
```

- All GSAP animations check `prefers-reduced-motion` before initializing.
- Three.js wireframe (if used): rotation off, static visible.
- Typewriter effect: full text visible immediately.
- Count-up animations: final number visible immediately.

### 9.4 Screen Reader Considerations

- Section labels (eyebrows) use visible text, not decorative — they are meaningful landmarks.
- Benchmark numbers use visible text, not image-based numerals.
- Icon-only links/buttons have `aria-label`.
- Code blocks have `role="code"` and `aria-label="Code example"`.
- Navigation has `aria-label="Main navigation"`.
- Footer links organized by category headings.

### 9.5 Color Blindness

- Amber on near-black is distinguishable for all common color vision deficiencies (protanopia, deuteranopia, tritanopia).
- Information is never conveyed by color alone — labels, icons, and text accompany color-coded signals.
- Amber `#ff5500` maintains its distinct luminance even in grayscale rendering.

### 9.6 Focus Order

Logical reading order: Nav → Hero title → Subtext → CTAs → Metrics strip → Features → Quickstart → Architecture → Benchmarks → Use Cases → Ecosystem → Final CTA → Footer.

Tab order follows visual order. No focus jumps or unexpected tab sequences.

---

## Appendix A: Related Documents

| Document | Purpose | Path |
|---|---|---|
| Component Library | Detailed component specs | `design/COMPONENT_LIBRARY.md` |
| Motion Choreography | Easing, timing, animation details | `design/MOTION_CHOREOGRAPHY.md` |
| Icon System | Monoline SVG icon specs | `design/ICON_SYSTEM.md` |
| Subpage Patterns | Reusable page templates | `design/SUB_PAGE_PATTERNS.md` |
| Brand Platform | Business model, archetypes, positioning | `brand/BRAND_PLATFORM.md` |
| Visual Identity | Resumen ejecutivo visual | `brand/VISUAL_IDENTITY.md` |
| Verbal Identity | Voice, tone, writing principles | `brand/VERBAL_IDENTITY.md` |
| Product Overview | Product purpose, users, personality | `product/PRODUCT.md` |
| Site Map | Complete route inventory | `product/SITE_MAP.md` |
| Implementation Plan | Phase breakdown, tasks | `strategy/implementation_plan.md` |
| Swiss Checklist | Portable pre-flight checklist | `qa/SWISS_CHECKLIST.md` |
| Accessibility Statement | Compliance declaration | `qa/ACCESSIBILITY_STATEMENT.md` |

## Appendix B: Design Tokens Summary

```css
--amber: #ff5500;
--near-black: #111111;
--white: #ffffff;
--muted: #5a5a5a;

--bg-dark: #111111;
--bg-dark-alt: #1a1a1a;
--text-dark: #f5f5f5;
--text-dark-muted: #808080;
--border-dark: rgba(255,255,255,0.08);
--surface-card-dark: rgba(255,255,255,0.03);
--surface-card-dark-hover: rgba(255,255,255,0.06);

--bg-light: #ffffff;
--text-light: #111111;
--text-light-muted: #5a5a5a;
--border-light: rgba(0,0,0,0.10);
--amber-light-dim: rgba(255,85,0,0.06);

--surface-glass: rgba(17,17,17,0.85);
--terminal-bg: #0d0d0d;
--success: #00c853;
--danger: #ff1744;

--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);

--text-hero: clamp(3.5rem, 7vw, 6.5rem);
--text-display: clamp(2.2rem, 4vw, 3.5rem);
--text-title: clamp(1.3rem, 2.2vw, 1.7rem);
--text-body: 1.05rem;
--text-small: 0.875rem;
--text-label: 0.72rem;
--text-code: 0.88rem;
--text-metric: clamp(2.5rem, 5vw, 4rem);

--space-xs: 8px;
--space-sm: 16px;
--space-md: 24px;
--space-lg: 48px;
--space-xl: 96px;
--space-2xl: 160px;
--space-3xl: 240px;
```
