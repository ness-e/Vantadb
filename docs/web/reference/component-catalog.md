# Component Catalog — `src/components/vanta/`

## Overview

- **47 active components** + **5 dead components** (1,264 lines orphaned)
- All are `"use client"`
- **5 additional files** in `vanta/mark/`
- **2 static/utility files**: `vanta-data.ts` (1,057 lines, 25 named exports) + `copy-utils.ts` (39 lines)

---

## Shell & Layout (7)

| Component | Lines | Description | Key Props |
|---|---|---|---|
| `SiteShell` | 48 | Root layout wrapper, renders navbar + main + footer | `{ children }` |
| `SiteNavbar` | 432 | Main navigation bar with animated hamburger menu, dropdowns, route state | — |
| `Footer` | 174 | Site footer with distribution commands, links, social | — |
| `PageTransition` | 32 | Framer Motion `AnimatePresence` wrapper, keyed by `viewKey` | `{ viewKey, children }` |
| `BackToTop` | 56 | Floating button visible after scroll threshold | — |
| `ScrollProgress` | 57 | Top-of-page progress bar indicating scroll depth | — |
| `SkipLink` | 10 | **Dead** — accessible skip-to-content link, rendered in layout but non-functional since SPA→Next migration |

---

## Page Views (6)

| Component | Lines | Route | Description |
|---|---|---|---|
| `HomeView` | 72 | `/` | Orchestrator that composes all 11 home sections in order |
| `BenchmarksView` | 429 | `/benchmarks` | BENCH-01 + SIFT1M tables, latency explorer |
| `BenchmarkRace` | 251 | `/benchmarks` | Animated head-to-head VantaDB vs cloud DB bar race (append below BenchmarksView) |
| `DocsView` | 626 | `/docs` | Full documentation viewer with sidebar nav, syntax-highlighted code blocks |
| `Architecture` | 239 | `/architecture` | Architecture diagram + engine layers description |
| `SearchSemantics` | 116 | `/docs` | Search semantics explanation section |

---

## Home Sections (11)

| § | Component | Lines | Purpose | Interactivity |
|---|---|---|---|---|
| 01 | `Hero` | 297 | Value prop + animated mark + headline stats | Interactive SVG mark, scroll-down indicator |
| 02 | `TrustBar` | 101 | Ecosystem logos (Ollama, LangChain, etc.) | — |
| 03 | `Features` | 97 | 3-column feature cards | — |
| 04 | `CoreEngine` | 380 | Engine capabilities (6 vignette cards). Also used on `/engine` page | — |
| 05 | `CodeTerminal` | 271 | Typewriter-effect Python snippet | `useTypingLines` typewriter animation |
| 06 | `LatencyComparator` | 558 | Animated latency bar chart with cloud DB comparison | Interactive bars |
| 07 | `UseCases` | 171 | Use case cards grid | — |
| 08 | `TutorialsSection` | 195 | Tutorial cards with step count | — |
| 09 | `FaqSection` | 126 | Accordion FAQ (ES content) | Expand/collapse |
| 10 | `TrustSection` | 122 | Security pillars cards | — |
| 11 | `CtaFinal` | 173 | Final CTA with MarkCta interactive SVG | `MarkCta` eyes follow hovered button |

Also: `InkDivider` (3 lines) — decorative ink-splash section divider.

---

## Visual & Interactive (9 + 5 mark files)

### vanta/ directory

| Component | Lines | Description |
|---|---|---|
| `LogoMark` | 34 | Static VantaDB logo SVG |
| `PageHeader` | 97 | Page hero with badge, title, subtitle, optional tag. Exports `PageHeader` + `PageSection` |
| `Reveal` | 71 | Scroll-triggered reveal wrapper (framer motion). Props: `direction`, `delay`, `as` |
| `CountUpStat` | 134 | Renders animated counter. Exported w/ `useCountUp` hook from `src/hooks/count-up.tsx` |
| `WalSimulator` | 218 | Interactive WAL crash-recovery visualizer |
| `CodePlayground` | 363 | Interactive Python code editor with run output |
| `TutorialModal` | 253 | Step-by-step tutorial overlay modal |
| `ChangelogSection` | 190 | Release history timeline |

### mark/ directory (5 files)

| File | Lines | Description |
|---|---|---|
| `index.tsx` | 31 | Entry point — renders `MarkClassic` (future variant switcher) |
| `mark-classic.tsx` | 274 | SVG interactive graph + cat mark with anime.js-driven eyes, sphere, graph nodes. **Known bug: `loop: true` ambient pulse animation (line 80-93) has no cleanup ref — orphaned instances persist on unmount** |
| `mark-cta.tsx` | 191 | Smaller SVG mark for CTA section. Eyes track hovered button (`install`/`docs`/`github`). Click triggers unique reaction per button (sphere pulse, spin, or bounce). No `loop: true` — safe |
| `use-mark-interaction.ts` | 225 | Reusable hook: mouse tracking (rAF-throttled), anime.js smooth interpolation, blink cycle, squint effect. ✅ has cleanup on unmount |
| `types.ts` | 31 | `BlinkState`, `MarkInteractionState`, `MarkVariantProps`, `MARK_VARIANTS`, `MarkVariantName` |

---

## Utility & Global UI (8)

| Component | Lines | Description |
|---|---|---|
| `CommandPalette` | 319 | ⌘K command palette with fuzzy search |
| `ShortcutOverlay` | 161 | `?` keyboard shortcut reference overlay |
| `EasterEgg` | 96 | Typing "vanta" sequence triggers hidden animation |
| `LangToggle` | 30 | Language switch (ES/EN) |
| `ThemeProvider` | 15 | `next-themes` wrapper |
| `Toast` | 30 | shadcn/ui toast re-export (`useToast`) |
| `CopyUtils` | 39 | `copyToClipboard` utility function |
| `InkDivider` | 3 | Decorative section divider |

---

## Dead Components (5 — imported by nothing)

| Component | Lines | Replaced By |
|---|---|---|
| `navbar.tsx` | 507 | `site-navbar.tsx` |
| `hero-mark-interactive.tsx` | 288 | `mark-classic.tsx` |
| `ecosystem.tsx` | 155 | Removed from `HomeView` |
| `vs-table.tsx` | 196 | No longer rendered |
| `metrics-bar.tsx` | 118 | Removed from `HomeView` |
| **Total** | **1,264** | |

---

## Static Data

`vanta-data.ts` (1,057 lines, 25 named exports) is the centralized content store:

- `VANTA` — project metadata (name, repo, discord, versions)
- `PRODUCT` — core metrics, versions, tech stack, ecosystem, distribution, hardware
- `View` — legacy type `"home" | "benchmarks" | "docs"`
- `HERO_STATS` — 4 headline metric cards
- `CORE_CAPABILITIES` — 6 engine capability vignettes
- `BENCH01` / `SIFT1M` — benchmark tables
- `QUICKSTART_PYTHON` — 28-line Python snippet
- `CLI_COMMANDS` — 6 CLI reference commands
- `SEARCH_SEMANTICS` — 4 search explanation cards
- `PRODUCT_BOUNDARY` — production vs deferred classification
- `DOC_LINKS` — 8 documentation links
- `FAQ` — 8 bilingual FAQ items (ES)
- `TUTORIALS` — 4 tutorials (01–04)
- `CHANGELOG` — 3 releases (0.1.0–0.1.2)
- `PRICING_PLANS` — 3 tiers (Community/Team/Enterprise)
- `SECURITY_PILLARS` — 6 security feature cards
- `USE_CASES_DETAIL` — 3 use cases (AI Agents, Local RAG, IDE Tooling)
- `TCO_COMPARISON` — 4-row cost comparison table
- `MAINTENANCE_PILLARS` — 4 zero-maintenance argument cards
- `WHY_VANTADB` — benefits (4) + comparison table
- `BLOG_POSTS` — 4 articles with full content
- `CASE_STUDIES` — 3 customer stories
- `TEAM_MEMBERS` — 4 team entries
- `COMPANY_INFO` — company data + principles
