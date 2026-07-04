---
title: "Swiss High-Contrast Minimal Redesign"
status: review
tags: [vantadb, plans, design]
last_reviewed: 2026-07-03
aliases: []
---

# Swiss High-Contrast Minimal Redesign

> **Design Read**: B2B developer-tool landing for technical buyers, with Swiss High-Contrast Minimal language, leaning toward Swiss International Typographic Style + monochrome + single accent.

**Goal**: Redesign VantaDB website following Swiss High-Contrast Minimal principles — strict grids, high contrast, one accent, typographic hierarchy, no decoration.

**Architecture**: CSS tokens → component-by-component refactor → animation audit. Each component independently updated. Maintains existing Routing/lazy loading.

**Tech Stack**: React + TanStack Router + Tailwind v4 + CSS tokens + motion/react + GSAP.

**Dials**: DESIGN_VARIANCE=7, MOTION_INTENSITY=5, VISUAL_DENSITY=4

---

## Swiss High-Contrast Minimal Rules (from research)

1. **Grid as law**: 12-column grid, every element snaps
2. **One sans-serif family**: hierarchy via weight/size, not variety
3. **Spacing scale**: 4px base unit (4, 8, 12, 16, 24, 32, 48, 64, 96, 128)
4. **High contrast**: near-black text on off-white bg, 4.5:1+ body, 3:1+ large text
5. **Monochrome + single accent**: black/white/gray + one color (amber #ff5500)
6. **No shadows, no rounded corners, no gradients**
7. **Asymmetric layout**: flush-left text, ragged-right, no centers
8. **Typography carries structure**: remove boxes, use alignment + spacing
9. **Empty space is structural**: don't fill it
10. **Flush-left, ragged-right**: standard for readability
