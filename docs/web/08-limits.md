---
title: Limits & Anti-Patterns
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [limits, anti-patterns, slop, banned]
---

# Limits & Anti-Patterns

```
[ VANTADB ] >> limits.enforced
────────────────────────────────────────────────────
  TYPE:  HARD BANS — do not violate
────────────────────────────────────────────────────
```

## Absolute Bans (Zero Tolerance)

These patterns are never acceptable. Any detection = redesign.

### 01. Gradient Text

```css
/* BANNED */
background: linear-gradient(...);
-webkit-background-clip: text;
color: transparent;
```

Accent emphasis uses solid amber. Always.

### 02. Any border-radius

```css
/* BANNED — any value > 0 */
border-radius: 8px;
border-radius: 999px;
```

All radii must be 0px. Hard edges are not negotiable.

### 03. Glassmorphism / Frosted Glass

```css
/* BANNED */
backdrop-filter: blur(12px);
background: rgba(255, 255, 255, 0.1);
```

Flat fills only. Transparency is limited to defined rgba values
in the token system.

### 04. Gradients of Any Kind

```css
/* BANNED */
background: linear-gradient(...);
background: radial-gradient(...);
background: conic-gradient(...);
```

Every fill is flat. Every surface is solid.

### 05. Nested Cards

```
<!-- BANNED -->
<div class="nb-card">
  <div class="nb-card">
    <div class="nb-card">
```

Cards never contain other cards. If you need hierarchy, use
border changes or background tint — never nesting.

### 06. Purple / Blue AI Gradients

The "AI aesthetic" (purple-to-blue glow, neon gradients) is
banned. VantaDB uses amber as its single accent. No neon purple,
no electric blue, no cyan-magenta blends.

### 07. Fake Screenshots in Divs

```html
<!-- BANNED -->
<div class="fake-browser">
  <div class="fake-toolbar"></div>
  <div class="fake-content"></div>
</div>
```

If you need to show a product screenshot, use a real image or
a real component preview. No div-based mockups.

### 08. Tiny Uppercase Eyebrows on Every Section

```
<!-- BANNED when used on every section -->
<p class="nb-mono-label">ABOUT</p>
<h2>Our Story</h2>
```

Maximum 1 eyebrow per 3 sections. Most sections get no eyebrow at all.

### 09. Centered Hero + Three Feature Cards

The default SaaS template (centered headline, centered subtext,
centered CTA, 3 equal cards below) is banned. Every hero must
use a different composition.

### 10. Side-Stripe Accent Borders

```css
/* BANNED */
border-left: 4px solid var(--amber);
```

Colored left/right borders on cards or callouts. Use full borders,
background tints, or nothing.

## Structural Limits

| Limit | Rule | Reason |
|-------|------|--------|
| Max font size | `clamp(3.5rem, 7vw, 6.5rem)` hero cap | Beyond this = shouting, not designing |
| Min tracking | `-0.06em` | Tighter and glyphs collide |
| Max body width | `65ch` | Readability ceiling |
| Max headline lines | 2 lines desktop | Hero must fit viewport |
| Max subtext words | 20 words | Hero value prop must be concise |
| Max CTAs in hero | 1 primary + 1 secondary | Decision clarity |
| Section layout reuse | Each layout max 1x per page | Avoid repetition |
| Zigzag alternation | Max 2 consecutive image/text splits | Break pattern by 3rd |
| Marquee per page | Max 1 | More than one = filler |
| Cards per row | 3 max (desktop) | Beyond = hard to scan |

## Color Limits

| Constraint | Rule |
|------------|------|
| Accent colors | 1 (amber only) |
| Saturation cap | Amber at full saturation |
| Palette type | Fixed, no hue rotation per page |
| Text on amber | Must use --text-on-amber (#0a0a0a) |
| Contrast minimum | Body: 4.5:1, Large: 3:1 (WCAG AA) |

## Typography Limits

| Constraint | Rule |
|------------|------|
| Font families | 3 max (Display, Body, Mono) |
| Serif usage | Banned. No serif fonts anywhere. |
| Inter font | Banned. The generic AI default. |
| Variable axes | Weight only. No width/optical axes. |
| Italic on display | Must check descender clearance (y/g/j/p/q) |

## Motion Limits

| Constraint | Rule |
|------------|------|
| Max duration | 500ms for any single animation |
| Animated properties | `transform` and `opacity` only |
| Infinite animations | Status indicators only |
| Reduced motion | Every animation must have a fallback |
| Scroll listeners | Never raw `scroll` — use GSAP/IO |

## Component Limits

| Constraint | Rule |
|------------|------|
| Shadow blur | Always 0. Always. |
| Button text wrap | Must fit 1 line at desktop |
| Nav height | Max 80px desktop, default 64px |
| Z-index values | Only from the defined scale |
| Card padding | Minimum --space-md (1.5rem) |

## Anti-Slop Checklist

Before shipping any page, audit for these signals:

- [ ] No gradient text anywhere
- [ ] No border-radius > 0
- [ ] No glass/blur effects
- [ ] No nested cards
- [ ] No purple/blue accent
- [ ] No centered hero with 3 cards
- [ ] No eyebrow on every section
- [ ] No fake screenshot divs
- [ ] No serif fonts
- [ ] No side-stripe borders
- [ ] No Inter font
- [ ] No `transition: all`
- [ ] No raw scroll listeners
- [ ] Image hover not animated
- [ ] Reduced motion works
- [ ] WCAG AA contrast on all text
- [ ] Max 1 marquee
- [ ] Button text fits 1 line
- [ ] Hero fits viewport
- [ ] Nav single line at 1024px+

---

```
[ END LIMITS ]
>> next: 09-rules.md
```
