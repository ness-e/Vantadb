---
title: Animation & Motion
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [animation, motion, gsap, transitions, scroll]
---

# Animation & Motion System

```
[ VANTADB ] >> motion.rules
────────────────────────────────────────────────────
  ENGINE:   GSAP + ScrollTrigger
  FALLBACK: IntersectionObserver + CSS transitions
  RULE:     Only transform and opacity
────────────────────────────────────────────────────
```

## Philosophy

Motion is motivated. Every animation must answer: what does this
communicate? Acceptable answers:

- **Hierarchy** — drawing attention to the right thing at the right time
- **Sequence** — revealing content in narrative order
- **Feedback** — acknowledging a user action (hover, click, press)
- **State change** — showing something has transitioned

Unacceptable answer: "it looked cool."

## Easing Curves

All easing curves are engineered, not defaulted.

| Token | Curve | Character |
|-------|-------|-----------|
| `--ease-swiss` | `cubic-bezier(0.25, 1, 0.5, 1)` | Standard — snappy entrance |
| `--ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Deceleration — natural rest |
| `--ease-in-out` | `cubic-bezier(0.77, 0, 0.175, 1)` | Emphasis — dramatic arc |
| `--ease-brutal` | `cubic-bezier(0.05, 0.95, 0.3, 1)` | Neubrutalism — fast snap |
| `--ease-cut` | `cubic-bezier(0.25, 0, 0.6, 1)` | Sharp cut — instant stop |

## Duration

| Scope | Duration | Notes |
|-------|----------|-------|
| Micro-interaction | 80ms | Hover, focus, active states |
| Element transition | 150ms | Card hover, button transform |
| Reveal animation | 250-300ms | Enter animations, fade-in |
| Page transition | 400ms | Route changes |
| Maximum | 500ms | Never exceed for UI motion |

## What to Animate

**Only `transform` and `opacity`.** Never:
- `top`, `left`, `width`, `height` (causes layout reflow)
- `margin`, `padding` (causes layout reflow)
- `color` transitions (cheap but visually muddy on dark backgrounds)
- `background-color` at scale (fine for isolated interactions)

## Reveal System

VantaDB uses two reveal mechanisms:

### CSS-Based (Default)

For simple fade-in-up on scroll:

```css
.nb-fade-up {
  opacity: 0;
  transform: translateY(16px);
  transition: opacity 0.25s var(--ease-brutal),
              transform 0.25s var(--ease-brutal);
}

.nb-fade-up.is-visible {
  opacity: 1;
  transform: translateY(0);
}
```

### Masked Reveal (Space Grotesk visual)

For typographic reveal animations:

```css
.nb-reveal-mask {
  overflow: hidden;
}

.nb-reveal-mask-inner {
  transform: translateY(110%);
  transition: transform 300ms var(--ease-brutal);
}

.nb-reveal-mask-inner.is-visible {
  transform: translateY(0);
}
```

### SVG Draw (Architecture diagrams)

For line-based illustrations:

```css
.nb-reveal-draw {
  stroke-dashoffset: var(--dash-total, 1000);
  stroke-dasharray: var(--dash-total, 1000);
}

.nb-reveal-draw.is-drawn {
  stroke-dashoffset: 0;
  transition: stroke-dashoffset 250ms var(--ease-brutal);
}
```

### Line Reveal

For horizontal accent lines:

```css
.nb-reveal-line {
  transform: scaleX(0);
  transform-origin: left center;
}

.nb-reveal-line.is-visible {
  transform: scaleX(1);
  transition: transform 250ms var(--ease-brutal);
}
```

## GSAP (Advanced Use)

For scroll-triggered stacks and horizontal pans, GSAP with
ScrollTrigger is the engine. Basic reveal should use CSS.

### Canonical Sticky Stack

```typescript
// Each card pinned at viewport top as next card pushes it
ScrollTrigger.create({
  trigger: card,
  start: "top top",
  endTrigger: lastCard,
  end: "top top",
  pin: true,
  pinSpacing: false,
});
```

### Canonical Horizontal Pan

```typescript
// Pinned section, inner track scrolls horizontally
const distance = track.scrollWidth - window.innerWidth;
gsap.to(track, {
  x: -distance,
  ease: "none",
  scrollTrigger: {
    trigger: wrap,
    start: "top top",
    end: () => `+=${distance}`,
    pin: true,
    scrub: 1,
  },
});
```

## Interactive States

All interactive elements follow the same tactile model:

| State | Transform | Shadow | Timing |
|-------|-----------|--------|--------|
| Rest | none | shadow-md | — |
| Hover | translate(2px, 2px) | shadow-sm | 80ms |
| Active | translate(4px, 4px) | none | 80ms |
| Focus | + amber outline 2px | shadow-sm | 80ms |

This creates a physical "press" metaphor. The element moves down-right
as if pushed, and the shadow shrinks (hover) or disappears (active).

## Micro-Interactions

### Cursor Blink

```css
@keyframes nb-cursor-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.nb-cursor {
  animation: nb-cursor-blink 1s step-end infinite;
}
```

### Split-Flip Counter

```css
@keyframes nb-split-flip {
  0% { transform: translateY(0); }
  50% { transform: translateY(-50%); }
  100% { transform: translateY(-100%); }
}
```

### Ticker

```css
@keyframes nb-ticker {
  0% { opacity: 1; }
  50% { opacity: 0.3; }
  100% { opacity: 1; }
}

.nb-ticker {
  animation: nb-ticker 0.8s steps(1) infinite;
}
```

## Reduced Motion

**Mandatory.** Every animation must have a `prefers-reduced-motion`
fallback. The system disables all animated reveals, cursor blinks,
tickers, split-flips, and GSAP interactions when reduced motion is
preferred.

```css
@media (prefers-reduced-motion: reduce) {
  .nb-reveal,
  .nb-fade-up,
  .nb-fade-up.is-visible,
  .nb-cursor,
  .nb-ticker,
  .nb-split-inner {
    opacity: 1 !important;
    transform: none !important;
    animation: none !important;
    transition: none !important;
  }
}
```

In GSAP: wrap all animations in a check for `useReducedMotion()`
from Motion library, and return early if true.

## Forbidden Animation Patterns

| Pattern | Why | Instead |
|---------|-----|---------|
| `window.addEventListener("scroll")` | Jank, no batching | GSAP ScrollTrigger, IntersectionObserver, CSS scroll-driven |
| `useState` for mouse/scroll values | Re-renders on every frame | Motion's `useMotionValue` / `useTransform` |
| `requestAnimationFrame` + React state | Same issue | Motion values |
| Animating `<img>` transform on hover | Adds no information, is a known AI tell | Animate card border/shadow instead |
| Infinite loop on every card | Cognitive noise | Reserve loops for status indicators |
| `ease-in` on UI | Feels slow and unnatural | `ease-out` for entrances, `ease-brutal` for feedback |
| CSS `transition: all` | Performance, unpredictable behavior | Target specific properties |

---

```
[ END ANIMATION ]
>> next: 08-limits.md
```
