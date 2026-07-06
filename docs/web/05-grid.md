---
title: Grid System
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [grid, layout, columns, responsive]
---

# Grid System

```
[ VANTADB ] >> grid.architecture
────────────────────────────────────────────────────
  COLUMNS:  12
  MAX-WIDTH: 1200px
  GAP:      0px (hairline borders provide visual separation)
────────────────────────────────────────────────────
```

## Philosophy

The grid is the single source of layout truth. Every component, every
section, every page aligns to it. In the Swiss tradition, the grid is
not a constraint — it is the framework that enables freedom.

In the VantaDB interpretation, grid lines are **visible**. Hairline
borders at grid edges and between columns make the structure apparent.

## Base Grid

```css
.nb-grid--cols-12 {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  column-gap: clamp(1.5rem, 3vw, 4rem);
}
```

### Column Spans

| Class | Span |
|-------|------|
| `.nb-col-1` through `.nb-col-12` | Spans 1-12 columns |
| `.nb-start-1` through `.nb-start-12` | Start at column N |

## Asymmetric Splits

These are the preferred layout patterns. They create the dynamic
tension that defines Swiss composition.

| Class | Ratio | Usage |
|-------|-------|-------|
| `.nb-asymmetric` | `8fr 4fr` | Main content + sidebar |
| `.nb-asymmetric--right` | `4fr 8fr` | Sidebar + main content |
| `.nb-split-7-5` | `7fr 5fr` | Slightly uneven content pair |
| `.nb-split-5-7` | `5fr 7fr` | Reverse of above |
| `.nb-split-3-9` | `3fr 9fr` | Narrow navigation + content |
| `.nb-split-2-10` | `2fr 10fr` | Minimal index + full content |

## Equal Column Grids

For structured data, feature grids, and bento layouts.

| Class | Columns | Usage |
|-------|---------|-------|
| `.nb-grid--cols-2` | `repeat(2, 1fr)` | Comparison, split content |
| `.nb-grid--cols-3` | `repeat(3, 1fr)` | Feature cards, stats |
| `.nb-grid--cols-4` | `repeat(4, 1fr)` | Metrics, compact grids |
| `.nb-grid--cols-6` | `repeat(6, 1fr)` | Dense data, tiny tiles |

## Bento Grid

The bento grid uses 1px gaps with contrasting parent background to
create hairline borders between cells. This is the Neubrutalism
contribution — visible seams in the layout.

```css
.nb-bento {
  display: grid;
  gap: 1px;
  background: var(--border-visible);
}
```

### Bento Variants

| Class | Columns | Span Support |
|-------|---------|--------------|
| `.nb-bento--col3` | 3 | Yes |
| `.nb-bento--col4` | 4 | Yes |

### Bento Cell Spans

| Class | Behavior |
|-------|----------|
| `.nb-bento-cell--featured` | Span 2 cols, 2 rows |
| `.nb-bento-cell--span2` | Span 2 cols |
| `.nb-bento-cell--span3` | Span 3 cols |
| `.nb-bento-cell--row2` | Span 2 rows |

## Section Layout

### Standard Sections

```css
.nb-section {
  padding: var(--section-gap) clamp(1.5rem, 5vw, 4rem);
}

.nb-section--sm {
  padding: calc(var(--section-gap) * 0.5) clamp(1.5rem, 5vw, 4rem);
}

.nb-section--lg {
  padding: var(--section-gap-lg) clamp(1.5rem, 5vw, 4rem);
}
```

### Inner Containers

| Class | Max Width | Usage |
|-------|-----------|-------|
| `.nb-inner` | `1200px` | Default content width |
| `.nb-inner--wide` | `1440px` | Full-width sections |
| `.nb-inner--narrow` | `720px` | Reading content |

## Dividers

### Structural Dividers (2px)

| Class | Style |
|-------|-------|
| `.nb-divider` | `2px solid --border-visible` |
| `.nb-divider--amber` | `2px solid --amber` |
| `.nb-divider--strong` | `2px solid --border-strong` |

### Hairline Dividers (1px)

| Class | Style |
|-------|-------|
| `.nb-hairline` | `1px solid --border` |
| `.nb-hairline--amber` | `1px solid --amber` |
| `.nb-hairline--strong` | `1px solid --border-strong` |

## Responsive Behavior

### Desktop (960px+)

All grids render as defined. 12-column grid, asymmetric splits, bento
layouts all function at full resolution.

### Tablet (768px - 960px)

- 4-col and 6-col grids collapse to 2-col
- Asymmetric splits collapse to single column
- Right border on section headers removed

### Mobile (< 768px)

- 12-column grid collapses to single column
- All multi-column grids collapse to single column
- Section padding reduced
- Title and hero type scales reduced via clamp
- Cell spans (span2, span3) collapse to span1

### Small Mobile (< 640px)

- Section padding: `3.5rem 1.25rem`
- All grids collapse to 1 column
- Bento grids collapse to single column

---

```
[ END GRID ]
>> next: 06-components.md
```
