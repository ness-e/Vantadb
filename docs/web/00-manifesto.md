---
title: VantaDB Design Manifesto
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [vantadb, design, manifesto, swiss, neubrutalism]
---

# VantaDB Design Manifesto

```
[ VANTADB ] >> design.philosophy
────────────────────────────────────────────────────
  STATUS: ENFORCED
  SCOPE:  ALL web surfaces
  UPDATED: 2026-07-05
────────────────────────────────────────────────────
```

## Core Beliefs

VantaDB's visual identity is built on a deliberate friction between two
traditions: the **International Typographic Style (Swiss Design)** and
**Neubrutalism**. One brings clarity, precision, and objective hierarchy.
The other brings raw energy, tactile presence, and defiant personality.

Neither dominates. They amplify each other.

- **Swiss gives us structure.** The grid is law. Typography is communication,
  not decoration. Hierarchy is established through scale and weight alone.
  Color is economical — every hue must earn its place.

- **Neubrutalism gives us presence.** Borders are thick and unapologetic.
  Shadows are hard offset, zero blur. Surfaces are flat. The interface
  declares itself — it does not fade into "delightful" obscurity.

- **Together they forge a third thing.** A developer tools brand that is
  precise but not cold, bold but not chaotic, functional but not boring.

## The Tenets

01 **Function dictates form.** Every visual decision must serve clarity,
   readability, or hierarchy. If it doesn't, cut it.

02 **Structure is visible.** Grids are not hidden scaffolding. Borders are
   not shy. The skeleton of the page is part of the design.

03 **One accent, maximum signal.** Amber (#ff5500) is the single voice of
   emphasis. It marks interactive elements, highlights data, and guides
   navigation. No secondary accent competes.

04 **Zero radius. Zero gradient. Zero blur.** Corners are always hard.
   Fills are always flat. Depth comes from offset shadows alone.

05 **Typography as architecture.** Type is the primary visual material.
   Three faces: Space Grotesk (display), Outfit (body), JetBrains Mono
   (code/data). Never mix more. Never use a fourth.

06 **Dark is default.** The canvas is #0a0a0a. This is not a theme — it
   is the material. Light mode is not supported. The interface lives in
   the terminal tradition.

07 **Motion is motivated.** Every animation must answer: what does this
   communicate? Hierarchy, sequence, feedback, or state change. Not
   decoration. Not "delight."

08 **Accessibility is structural.** 4.5:1 contrast minimum. Reduced motion
   is not optional. Keyboard navigation works without broken paths.
   WCAG AA is the floor, not the ceiling.

09 **Anti-slop by default.** Reject defaults: no glassmorphism, no purple
   gradients, no nested cards, no tiny uppercase eyebrows above every
   section, no centered hero with three feature card grid below.

10 **The interface is honest.** What you see is what the stack is.
   No fake screenshots rendered in divs. No simulated dashboards.
   No decorative illustrations that add nothing.

## Decision Hierarchy

When styles conflict, resolve in this order:

```
1. ACCESSIBILITY   → WCAG AA is non-negotiable
2. READABILITY     → If it's hard to read, change it
3. HIERARCHY       → Information priority must be obvious
4. CONSISTENCY     → Follow the token system
5. SWISS PRECISION → Grid alignment, typographic rhythm
6. NEUBRUTALIST    → Borders, shadows, tactile presence
   TACTILITY
7. PERFORMANCE     → 60fps, Core Web Vitals, minimal DOM cost
```

## The Name

`Nb` — short for Neubrutalism — is the prefix for every class and
component in the system. It signals intent: this is not a default
interface. It is constructed, opinionated, and deliberate.

---

```
[ END MANIFESTO ]
>> next: 01-swiss-design.md
```
