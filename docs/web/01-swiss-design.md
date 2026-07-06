---
title: Swiss Design
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [swiss, typography, grid, international-typographic-style]
---

# Swiss Design (International Typographic Style)

```
[ VANTADB ] >> swiss-design.principles
────────────────────────────────────────────────────
  ORIGIN:  1950s — Zurich & Basel, Switzerland
  KEY:     Clarity. Objectivity. Structure.
────────────────────────────────────────────────────
```

## What It Is

The International Typographic Style emerged in postwar Switzerland as
a rejection of decorative excess. Designers like Josef Müller-Brockmann,
Emil Ruder, and Armin Hofmann developed a discipline where communication
efficiency was the only measure of success.

For VantaDB, Swiss design is the **structural skeleton**. It governs
layout, hierarchy, typography, and the relationship between content
and space.

## Core Principles Applied

### 01. Grid-Based Layout

Every page aligns to a visible or implied grid. Content does not float.
Elements anchor to grid tracks and intersections.

- 12-column grid as the universal framework
- Asymmetric splits (8/4, 7/5, 5/7, 3/9) for deliberate tension
- Grid lines are visible as hairline borders — the structure is part
  of the aesthetic

### 02. Sans-Serif Typography as Primary Material

Type is not decoration. It is the interface.

- Space Grotesk: geometric, tight, architectural — for display and headings
- Outfit: clean, humanist, readable — for body text
- JetBrains Mono: technical, monospaced — for code and data

### 03. Asymmetric Composition

Balance is achieved through contrast, not symmetry.

- Left-aligned content with right-aligned visual assets
- Wide margins and deliberate empty zones
- Section headers that span partial columns

### 04. Objective Hierarchy

Size, weight, and position alone establish order. No color decoration,
no illustration, no ornamental differentiation.

```
H1: Space Grotesk Bold / clamp(3.5rem, 7vw, 6.5rem)
H2: Space Grotesk Bold / clamp(2.2rem, 4vw, 3.5rem)
H3: Space Grotesk Bold / clamp(1.3rem, 2.2vw, 1.7rem)
Body: Outfit Regular / 1.05rem
Code: JetBrains Mono / 0.88rem
Label: JetBrains Mono Bold / 0.72rem / uppercase / 0.14em tracking
```

### 05. Color Economy

One accent serves all emphasis. Swiss red in the original tradition;
VantaDB uses amber (#ff5500) as its single structural accent.

- Background: #0a0a0a (dark substrate)
- Foreground: #ffffff (white phosphor text)
- Accent: #ff5500 (amber — only accent)
- Neutral greys: oklch 48-63% at 0 chroma (pure achromatic)

### 06. Negative Space as Active Element

White (black) space is not empty — it is a design element that gives
content room to breathe and creates visual rhythm.

- Section gaps: 96px standard, 160px for major transitions
- Content max-width: 1200px
- Body text max-width: 65ch

## Swiss Figures Who Inform Our Approach

### Josef Müller-Brockmann

His grid systems codified the mathematical approach to layout. His concert
posters prove that strict grids produce dynamic, arresting work. Every
VantaDB grid descends from his methodology.

> "The grid system is an aid, not a guarantee. It permits a number of
> possible uses and each designer can look for a solution appropriate
> to his personal style."

### Emil Ruder

His philosophy that legibility and visual rhythm are inseparable guides
our typography. Word spacing, line height, and letter spacing are not
defaults — they are engineered.

### Armin Hofmann

His integration of photography into Swiss rigor shows how visual assets
can coexist with strict typographic discipline. Our hero images and
product screenshots follow this tradition.

## Rules for VantaDB

| Rule | Enforcement |
|------|-------------|
| Every element aligns to the grid | All components use CSS Grid or grid-derived spacing |
| Max 2 typefaces per page | Display + Body. Mono for code is separate. |
| No centered text on asymmetric layouts | Left-aligned is default. Center only for short messages. |
| One accent color per page | Amber only. No secondary accent. |
| No decorative images | Every visual asset serves information hierarchy. |

---

```
[ END SWISS DESIGN ]
>> next: 02-neubrutalism.md
```
