---
title: Component Library
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [components, ui, cards, buttons, bento]
---

# Component Library — Nb System

```
[ VANTADB ] >> components.reference
────────────────────────────────────────────────────
  PREFIX:    nb- (all classes and components)
  SOURCE:    web/src/components/nb/
  STYLES:    web/src/styles/nb-components.css
────────────────────────────────────────────────────
```

Every component follows the same interaction model:

```
REST  →  HOVER  →  FOCUS  →  ACTIVE  →  DISABLED
        shadow-sm    amber      shadow-none
      translate(2,2) outline    translate(4,4)
```

## NbCard

Purpose: Content containers for features, data, and sections.

| Variant | Class | Use |
|---------|-------|-----|
| Default | `.nb-card` | Standard content card |
| Amber | `.nb-card--amber` | Featured/promoted content |
| Strong | `.nb-card--strong` | High-emphasis structural card |
| Inset | `.nb-card--inset` | Subtle, low-emphasis containers |
| Offset | `.nb-card--offset` | Cards with MD shadow |
| Offset Amber | `.nb-card--offset-amber` | Featured with amber shadow |

```
States:
  Rest:     2px border-visible, no shadow
  Hover:    amber border, shadow-sm, translate(2px, 2px)
  Focus:    amber outline +2px offset
  Active:   no shadow, translate(4px, 4px)
```

## NbButton

Purpose: Primary call-to-action and interactive actions.

| Variant | Class | Use |
|---------|-------|-----|
| Primary | `.nb-btn` | CTAs, main actions |
| Ghost | `.nb-btn--ghost` | Secondary actions |
| Ghost Light | `.nb-btn--ghost-light` | Actions on dark backgrounds |
| Install | `.nb-btn--install` | Code install commands |
| Small | `.nb-btn--sm` | Compact contexts |
| Large | `.nb-btn--lg` | Hero CTAs, featured actions |

```
Structure:
  Mono type (0.72rem / bold / 0.14em tracking / uppercase)
  2px solid border (amber for primary, border-visible for ghost)
  28px horizontal padding standard
  Offset shadow-md (4px 4px) for primary

States:
  Rest:     amber bg + border, shadow-md
  Hover:    amber-light bg, shadow-sm, translate(2px, 2px)
  Focus:    amber outline +2px offset
  Active:   no shadow, translate(4px, 4px)
```

## NbCardFrame

Purpose: Alternative card with stronger structural borders.

| Variant | Class | Use |
|---------|-------|-----|
| Default | `.nb-card-frame` | Standard framed card |
| Featured | `.nb-card-frame--featured` | Amber border + shadow-amber-lg |
| Transparent | `.nb-card-frame--transparent` | No background fill |

## NbBento

Purpose: Multi-cell grid layouts with visible seams (1px gap).

```
.nb-bento + .nb-bento-cell
```

| Span Class | Effect |
|------------|--------|
| `.nb-bento-cell--featured` | 2 col x 2 row |
| `.nb-bento-cell--span2` | 2 column span |
| `.nb-bento-cell--span3` | 3 column span |
| `.nb-bento-cell--row2` | 2 row span |

## NbAccordion / NbAccordionItem

Purpose: Expandable content sections for FAQs and specs.

```
Structure:
  Item:    2px border-strong between items
  Button:  full width, left-aligned, amber on hover
  Toggle:  + symbol, rotates 45° when open
  Content: revealed below, muted body text
```

## NbMetric

Purpose: Data display — benchmarks, stats, KPIs.

```
Structure:
  Value:  mono, clamp(2rem, 5vw, 3.5rem), bold, tabular-nums
  Label:  micro (0.6rem), uppercase, 0.14em tracking, steel color
  Unit:   0.5em, muted, vertical-align super
```

## NbCopyCommand

Purpose: Install commands and CLI snippets with one-click copy.

```
Structure:
  Block:      2px amber border, terminal-bg, shadow-md
  Prompt ($): steel color
  Command:    amber color, bold
  Cursor:     amber blink animation
  Copy btn:   ghost button, amber text, copies on click
```

## NbCodeBlock

Purpose: Multi-line code samples.

```
Structure:
  Background: terminal-bg (#080808)
  Border:     2px border-strong
  Padding:    space-md
  Font:       JetBrains Mono, 0.88rem
```

## NbBlockAmber

Purpose: High-emphasis callout or CTA area.

```
Structure:
  Background: amber
  Text:       text-on-amber (#0a0a0a)
  Padding:    space-lg
  All child headings inherit text-on-amber
```

## NbBlockWarning

Purpose: Warnings, important notices.

```
Structure:
  Surface bg, 3px amber border
  ⚠ WARNING label positioned at top-left edge overlap
```

## NbIconBox

Purpose: Icon containers with consistent framing.

```
Structure:
  2.5rem x 2.5rem box, 2px border-visible, amber icon color
  Icon stroke-width: 2
```

## NbLogLine

Purpose: Terminal log-style data display.

```
Structure:
  Mono code font, muted color
  Prefix:  attr(data-level) — amber for default
  Variants: info (steel), warn (amber), ok (success), error (danger)
```

## NbArrow

Purpose: Navigational links and "learn more" indicators.

```
Structure:
  Mono label, uppercase, 0.14em tracking, amber
  >>> suffix indicator
  Hover: gap increases (attraction effect)
```

## NbMetaTag

Purpose: Small metadata labels, tags, categories.

```
Structure:
  Mono, micro, uppercase, 0.14em tracking, steel
  > prefix in amber
  1px border-visible
```

## NbList

Purpose: Styled lists with amber chevron markers.

```
Structure:
  Removes default list styling
  Each item: flex row with amber > prefix
  Border-bottom divides items (last item has no border)
```

## NbTable

Purpose: Data tables with technical aesthetic.

```
Structure:
  Mono type, 0.88rem
  1px border-visible on all cells
  Header: micro uppercase, steel, surface background
  Row hover: surface-alt background
```

## NbTactileCard / NbTactileInput / NbTactileBtn

Purpose: High-tactility alternative components for maximum
physical feedback.

```
Tactile Card:   2px border, amber on hover
Tactile Input:  2px border-strong, amber on focus
Tactile Button: amber bg, invert to white on hover
```

## NbNumMarker

Purpose: Large decorative number for step sequences or indices.

```
Structure:
  Space Grotesk, clamp(3rem, 6vw, 5rem), weight 900
  -0.06em tracking, border-strong color
  Amber variant available
```

## NbDitherImage

Purpose: Team photos and editorial images with 1-bit dithering filter.

```
Structure:
  SVG filter: dither (discrete color matrix, 2 values)
  image-rendering: pixelated
```

## Section Header Patterns

### Headers

```
.nb-section-header — standard
.nb-section-header--bordered — 2px bottom border
.nb-section-header--hairline — 1px bottom border
.nb-section-header--right-border — 2px right border
.nb-section-header--amber — amber bottom border
.nb-section-header--center — center-aligned
```

### Typographic Header Elements

```
.nb-section-headline — display heading
.nb-section-sub — muted body beneath headline
.nb-mono-label — mono uppercase label (micro)
.nb-amber-title — 1.25rem amber display title
.nb-footer-heading — 0.65rem mono amber footer titles
```

---

```
[ END COMPONENTS ]
>> next: 07-animation.md
```
