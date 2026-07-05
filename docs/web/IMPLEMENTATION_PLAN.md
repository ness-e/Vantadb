# Implementation Plan — VantaDB Landing Page

> Versión: 2.0 | 2026-07 | Estilo: Swiss + Neubrutalism

---

## Phase 0: Foundation (COMPLETED)

| Task | Status | Notes |
|---|---|---|
| CSS tokens (`tokens.css`) | ✅ Done | Colors, spacing, shadows (hard offset), radii (0px), easing |
| Base styles (`index.css`) | ✅ Done | Reset, utilities, redwood classes |
| Neubrutalism classes (`neubrutalism.css`) | ✅ Done | Layout, backgrounds, cards, labels, frames |
| Button system (`buttons.css`) | ✅ Done | Primary, ghost, ghost inverted, install, mechanical press |
| Nav/index.astro | ✅ Done | Responsive nav with backdrop blur |
| Hero/index.astro | ✅ Done | Hero with title, subtitle, telemetry row |
| Footer/index.astro | ✅ Done | Footer layout |

## Phase 1: Home Page Sections (COMPLETED)

| Task | Status | Notes |
|---|---|---|
| Hero section | ✅ Done | Noise overlay, code block, telemetry |
| Features section | ✅ Done | 3-card layout |
| Benchmark / Metrics | ✅ Done | Telemetry metrics |
| Ecosystem integration | ✅ Done | Bento grid with 8 cells |
| CTA section | ✅ Done | Primary CTA + ghost |

## Phase 2: Interaction & Animation (IN PROGRESS)

| Task | Status | Notes |
|---|---|---|
| Button mechanical press | ✅ Done | In buttons.css |
| Card hover effects | ✅ Done | In neubrutalism.css |
| Nav scroll state | 🟡 Pending | Add bg opacity on scroll |
| Tick Counter animation | 🟡 Pending | GSAP count-up |
| Section reveals | 🟡 Pending | GSAP ScrollTrigger |

## Phase 3: Mobile Responsive (IN PROGRESS)

| Task | Status | Notes |
|---|---|---|
| Nav hamburger | ✅ Done | In Nav/index.astro |
| Card stack on mobile | 🟡 Pending | Single column < 768px |
| Font scaling | 🟡 Pending | Clamp adjustments |
| Touch target sizes | 🟡 Pending | > 44×44px |

## Phase 4: Subpages (PENDING)

| Task | Status | Notes |
|---|---|---|
| Pricing page | 🟡 Pending | Pricing tiers, feature comparison |
| Docs hub | 🟡 Pending | Docs search, sidebar navigation |
| Integration pages | 🟡 Pending | Per-integration detail pages |
| Legal pages | 🟡 Pending | Privacy, Terms |
| Engine page | 🔵 Future | Dedicated page |

## Phase 5: SEO & Performance (PENDING)

| Task | Status | Notes |
|---|---|---|
| Meta tags | 🟡 Pending | Open Graph, Twitter Cards |
| Schema markup | 🟡 Pending | Product, FAQ structured data |
| Lighthouse audit | 🟡 Pending | Target 95+ all categories |
| Bundle optimization | 🟡 Pending | Code splitting, critical CSS |
| Sitemap | 🟡 Pending | XML sitemap |
| Robots.txt | 🟡 Pending | |

## Phase 6: Polish & QA (PENDING)

| Task | Status | Notes |
|---|---|---|
| Reduced motion | 🟡 Pending | Media query |
| Focus-visible | 🟡 Pending | Amber outline, 2.5px |
| Contrast audit | 🟡 Pending | WCAG AA / AAA |
| Cross-browser | 🟡 Pending | Chrome, Firefox, Safari |
| 404 page | 🟡 Pending | |
| Loading states | 🟡 Pending | Skeleton screens |
