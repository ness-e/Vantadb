# Implementation Plan — VantaDB Web Redesign

> Plan de implementación por fases para alinear `web/src/` con `docs/web/DESIGN.md`.
> Basado en el gap analysis del 2026-07-04.

---

## Phase 0 — Security & Code Health

**Goal:** Eliminate XSS vectors, remove dead code, harden deployment config.

| ID | Task | Priority | Files |
|----|------|----------|-------|
| 0.1 | **CSP: remove `unsafe-eval`** | 🔴 Critical | `vercel.json:18` |
| 0.2 | **Blog: add DOMPurify** — sanitize HTML before `dangerouslySetInnerHTML` | 🔴 Critical | `$slug.lazy.tsx:82`, install `dompurify` + `@types/dompurify` |
| 0.3 | **Remove dead Three.js** — no imports anywhere, 600KB+ in bundle | 🔴 Critical | `package.json:32,40`, `vite.config.ts:16` |
| 0.4 | **Bundle analysis** — add `vite-plugin-visualizer` to catch future bloat | 🟡 Medium | `vite.config.ts`, install dep |

---

## Phase 0.5 — Accessibility (a11y)

**Goal:** Meet WCAG 2.1 AA baseline.

| ID | Task | Priority | Files |
|----|------|----------|-------|
| 0.5 | **Add skip-link** — `href="#main-content"` as first focusable element | 🔴 High | `Nav.tsx`, `index.css` |
| 0.6 | **Nav focus trap** — trap Tab in mobile menu when open | 🟡 Medium | `Nav.tsx` |
| 0.7 | **GSAP ScrollTrigger cleanup** — prevent duplicate listeners on remount | 🟡 Medium | `SwissBackToTop.tsx:7-48` |
| 0.8 | **Array index → stable key** — replace `key={i}` with stable IDs | 🟡 Medium | Multiple `.lazy.tsx` files |

---

## Phase 1 — Token Overhaul (Global Redesign)

**Goal:** Flip from `docs/web/qa/REPORTE-DE-REVISION.md` spec (light Swiss) to `docs/web/DESIGN.md` spec (dark-dominant).

### 1A — Update `tokens.css` (light → dark dominant)

| Token | Old Value | New Value | Affects |
|-------|-----------|-----------|---------|
| `--background` | `#f9f8f6` | `#111111` | Every bg |
| `--foreground` | `#000000` | `#f0f0f0` | Every text |
| `--surface` | `#ffffff` | `#1a1a1a` | Cards, inputs |
| `--block-dark-bg` | `#0a0a0a` | `#111111` | Dark sections (unify) |
| `--muted` | `#666` | `#888` | Muted text on dark |
| `--text-hero` | `clamp(3.8rem, 10vw, 9rem)` | `clamp(3.5rem, 7vw, 6.5rem)` | Hero heading |
| `--grid-max` | `1280px` | `1200px` | Layout width |
| `--border` | `oklch(15% 0.008 265)` | `oklch(35% 0.02 265)` | Visible on dark |

- Remove unused tokens: `--steel`, `--frost`, `--crimson`, `--void`, `--block-accent`
- Add missing tokens from DESIGN.md: `--brand-accent`, `--code-bg`, `--tag-bg`
- Update `--nav-bg` to match section 7.1 spec

### 1B — Flip `index.css` and all section CSS files

All `@layer components` sections need color updates. Key files:
- `swiss-hero.css` — text/background colors, grid lines
- `swiss-benchmark.css` — card backgrounds, stat colors
- `swiss-core-engine.css` — feature cards
- `swiss-use-cases.css` — case cards
- `swiss-ecosystem.css` — ecosystem tiles
- `swiss-monolith.css` — CTA section
- `swiss-arch.css` — architecture layer colors
- `global.css` — base color resets
- `nav.css` — nav background, dropdown
- `footer.css` — footer colors (already close)

---

## Phase 2 — Homepage Restructure

**Goal:** Match section order and content from `DESIGN.md` sections 8.1–8.9.

### 2A — Reorder sections in `index.lazy.tsx`

| New Order | Section | Component | Notes |
|-----------|---------|-----------|-------|
| 1 | Hero | `SwissHero.tsx` | Still Hero, but redesigned |
| 2 | Metrics Bar | **New component** | `SwissMetricsBar.tsx` — not implemented |
| 3 | Features | `SwissCoreEngine.tsx` | Renamed from Core Engine |
| 4 | Quickstart | `SwissQuickstart.tsx` | Same position |
| 5 | Architecture | `SwissArchSection.tsx` | Same position |
| 6 | Benchmark Grid | `SwissBenchmarkGrid.tsx` | Moved from position 2 |
| 7 | Use Cases | `SwissUseCases.tsx` | Expanded to bento grid |
| 8 | Ecosystem | `SwissEcosystem.tsx` | Same position |
| 9 | CTA / Final | `SwissMonolith.tsx` | Same position |

### 2B — Create `SwissMetricsBar.tsx`

- Center-aligned stat row: 4–6 metrics (stars, downloads, latency, languages, etc.)
- Count-up animation via GSAP
- Placed between Hero and Features

### 2C — Redesign Hero (`SwissHero.tsx`)

Following DESIGN.md section 8.1:
- Remove stats from hero
- Add tech labels: `[RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS]`
- Asymmetric grid: title cols 1–8, wireframe/pattern cols 9–12
- Reduce `--text-hero` to new size (done in Phase 1)
- GSAP stroke-dashoffset grid line animation

---

## Phase 3 — Component Refinements

**Goal:** Bring each section into spec compliance.

| Section | Changes |
|---------|---------|
| **Hero (8.1)** | See 2C. Font-weight 800→700. Left-align text. No center. |
| **Benchmark Grid (8.2)** | Add `[VANTADB]` / `[TRADITIONAL]` column labels. Count-up animation. Left-align. |
| **Metrics Bar (8.3)** | New component (see 2B). |
| **Quickstart (8.4)** | GSAP typewriter animation on terminal code. Already 2-col grid OK. |
| **Features (8.5)** | Exploded architecture with GSAP pin + sequential reveal. |
| **Architecture (8.6)** | Layer animations OK. Verify labels follow spec. |
| **Use Cases (8.7)** | Expand from 3 hardcoded cards to 6+ bento grid. Dynamic content from data file. |
| **Ecosystem (8.8)** | Add stagger reveal. |
| **CTA (8.9)** | Add GSAP entrance animation. |

---

## Phase 4 — Subpage Implementation

**Goal:** Convert all 16 Legacy routes to Swiss design.

### SITE_MAP.md Priority Order

| Priority | Routes | Pattern |
|----------|--------|---------|
| 🔴 Critical | `/engine`, `/architecture`, `/integrations`, `/use-cases` | Swiss subpage hero + section grid |
| 🟠 High | `/cost`, `/latency`, `/storage`, `/config`, `/maint`, `/changelog` | Swiss subpage + data |
| 🟡 Medium | `/solutions/*`, `/docs`, `/pricing`, `/about/*`, `/blog/*`, `/security`, `/product/*` | Full Swiss |
| ⚫ Delete | `/about/roadmap` | Remove route + file |

### Subpage Pattern

All subpages should use `SwissSubpageHero.tsx` (new component):
```tsx
<SwissSubpageHero
  label="[ENGINE]"
  title="Engine"
  description="..."
  cta={{ text: "View docs", href: "/docs" }}
/>
```

Plus section-specific components per page.

---

## Phase 5 — Content & Copy

**Goal:** Write new section copy per DESIGN.md, update metadata.

- Update all `<title>` and `<meta name="description">` per route
- Add JSON-LD `SoftwareApplication` to every subpage
- Ensure OG tags on every route
- NEW: `/solutions/*` pages with unique content (currently empty)

---

## Phase 6 — SEO & Meta Tags

**Goal:** 100% meta coverage, structured data, sitemap.

- [ ] Add `<link rel="canonical">` to every route (use TanStack Router head)
- [ ] Add OG tags to every route
- [ ] Add JSON-LD breadcrumbs on subpages
- [ ] Generate `sitemap.xml`
- [ ] Add `robots.txt` to public/
- [ ] Verify all with structured data testing tool

---

## Phase 7 — Animation Layer

**Goal:** Implement all GSAP ScrollTrigger animations from DESIGN.md §6.

- [ ] Hero: stroke-dashoffset grid line reveal
- [ ] Metrics Bar: count-up numbers
- [ ] Benchmark Grid: count-up + stagger card reveal
- [ ] Features: exploded layer pin + sequential reveal
- [ ] Ecosystem: stagger tile reveal
- [ ] Quickstart: typewriter code effect
- [ ] CTA: entrance animation
- [ ] Nav: backdrop-blur transition on scroll
- [ ] All: respect `prefers-reduced-motion`

---

## Phase 8 — Polish & QA

- [ ] Visual regression: set up Playwright screenshot pipeline (per CODE-074)
- [ ] 1440×900 + 390×844 + 768×1024 viewport checks
- [ ] Fix text-align: center violations (0 center elements)
- [ ] Verify 0 CSS `border-radius` > 6px
- [ ] Verify 0 `box-shadow` usage
- [ ] Verify 0 gradient usage
- [ ] Verify `font-variant-numeric: tabular-nums` on all data
- [ ] Anti-slop checklist: 14/14 pass
- [ ] `npm run build` passes with 0 warnings
- [ ] Lighthouse audit: 90+ all categories

---

## Cross-Reference: Backlog Tasks

| Phase | Backlog ID | Description |
|-------|-----------|-------------|
| 0 | CODE-020 | CSP unsafe-eval |
| 0 | CODE-021 | DOMPurify blog |
| 0 | CODE-022 | Remove unused Three.js |
| 0.5 | CODE-048 | Skip-link a11y |
| 0.5 | CODE-049 | Nav focus trap |
| 0.5 | CODE-076 | GSAP ScrollTrigger cleanup |
| 0.5 | CODE-072 | Array index keys |
| 2 | MKT-13 | Homepage content redesign |
| 2 | MKT-14 | Benchmark section copy |
| 3 | MKT-17 | Use cases update |
| 4 | MKT-15 | Subpage content |
| 4 | CODE-023 | CI tests for web |
| 8 | CODE-074 | Visual regression tests |
| 4 | CODE-070 | Bundle analysis |
| 4 | CODE-073 | E2E tests |
| 4 | CODE-078 | Playwright install in CI |
