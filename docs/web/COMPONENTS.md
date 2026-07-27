# Component Catalog — VantaDB Web

## Custom Components (`src/components/vanta/`)

47 custom components. All are `"use client"`.

### Shell & Layout

| Component | File | Role |
|---|---|---|
| `SiteShell` | `site-shell.tsx` | Root layout wrapper — renders ScrollProgress + SiteNavbar + main + Footer + 4 modals |
| `SiteNavbar` | `site-navbar.tsx` | Main navigation — marquee strip, 4 dropdown groups, mobile menu, ⌘K trigger (460 lines) |
| `Footer` | `footer.tsx` | Site footer — links, distribution commands, i18n (39 lines) |
| `PageTransition` | `page-transition.tsx` | Framer-motion AnimatePresence wrapper — slide+fade (34 lines) |
| `SkipLink` | `skip-link.tsx` | Skip-to-content link (rendered in layout.tsx) |
| `BackToTop` | `back-to-top.tsx` | Scroll-to-top FAB |
| `ScrollProgress` | `scroll-progress.tsx` | Reading progress bar (top of page) |

### Page Views

| Component | File | Route |
|---|---|---|
| `HomeView` | `home-view.tsx` | `/` — 11-section narrative arc |
| `BenchmarksView` | `benchmarks-view.tsx` | `/benchmarks` |
| `BenchmarkRace` | `benchmark-race.tsx` | `/benchmarks` — animated head-to-head race |
| `DocsView` | `docs-view.tsx` | `/docs` |
| `Architecture` | `architecture.tsx` | `/architecture` |
| `SearchSemantics` | `search-semantics.tsx` | `/architecture` — search types diagram |

### Home Sections

| Component | File | Position | Purpose |
|---|---|---|---|
| `Hero` | `hero.tsx` | §01 | Hook — headline stats + install + CTAs (317 lines) |
| `TrustBar` | `trust-bar.tsx` | §02 | Social proof — ecosystem logos |
| `Features` | `features.tsx` | §03 | 6 Core Capabilities cards |
| `CoreEngine` | `core-engine.tsx` | §04 | Engine pipeline visualization |
| `CodeTerminal` | `code-terminal.tsx` | §05 | Typed quickstart demo |
| `LatencyComparator` | `latency-comparator.tsx` | §06 | Live latency comparison chart |
| `UseCases` | `use-cases.tsx` | §07 | Target use cases |
| `TutorialsSection` | `tutorials-section.tsx` | §08 | Tutorial cards |
| `FaqSection` | `faq-section.tsx` | §09 | Accordion FAQ |
| `TrustSection` | `trust-section.tsx` | §10 | Final credibility — metrics + endorsements |
| `CtaFinal` | `cta-final.tsx` | §11 | Final call-to-action |
| `InkDivider` | `ink-divider.tsx` | — | Section separator |

### Visual & Interactive

| Component | File | Purpose |
|---|---|---|
| `Mark` (dir) | `mark/` | Animated product mark — see submodule below |
| `MarkClassic` | `mark/mark-classic.tsx` | SVG interactive graph with anime.js (274 lines) |
| `MarkCta` | `mark/mark-cta.tsx` | SVG mark for CTA section — eyes follow hovered button, unique click reactions per button (191 lines) |
| `useMarkInteraction` | `mark/use-mark-interaction.ts` | Reusable hook: mouse tracking, eye/blink animation via anime.js, squint effect (225 lines) |
| `MarkVariantName` | `mark/types.ts` | Types: `BlinkState`, `MarkInteractionState`, `MarkVariantProps`, `MARK_VARIANTS` |
| `LogoMark` | `logo-mark.tsx` | VantaDB logotype/anchor |
| `PageHeader` | `page-header.tsx` | Reusable page hero header — black panel, neon accent, rigid shadow |
| `PageSection` | `page-header.tsx` | Reusable content wrapper — 3 variants: cream/paper/ink backgrounds |
| `Reveal` | `reveal.tsx` | Scroll-triggered entrance animation (6 directions, 76 lines) |
| `CountUpStat` | `count-up.tsx` | Animated number counter (134 lines, hook + component in same file) |
| `WalSimulator` | `wal-simulator.tsx` | WAL visualization (interactive) |
| `CodePlayground` | `code-playground.tsx` | Interactive code editor |
| `VsTable` | `vs-table.tsx` | Comparison table (VantaDB vs cloud) |
| `TutorialModal` | `tutorial-modal.tsx` | Tutorial walkthrough modal |
| `ChangelogSection` | `changelog-section.tsx` | Changelog timeline |

### Utility & Global UI

| Component | File | Purpose |
|---|---|---|
| `CommandPalette` | `command-palette.tsx` | ⌘K search modal (filters nav items + blog posts) |
| `ShortcutOverlay` | `shortcut-overlay.tsx` | `?` keyboard shortcuts modal |
| `EasterEgg` | `easter-egg.tsx` | Type "vanta" sequence → hidden content |
| `LangToggle` | `lang-toggle.tsx` | ES/EN language switcher |
| `ThemeToggle` | `theme-toggle.tsx` | Dark/light toggle (rendered in dead navbar only) |
| `ThemeProvider` | `theme-provider.tsx` | next-themes ThemeProvider wrapper |
| `Toast` | `toast.tsx` | Custom toast component (for site-navbar) |
| `CopyUtils` | `copy-utils.ts` | Clipboard copy helper |
| `InkDivider` | `ink-divider.tsx` | Section divider |

### Infrastructure

| File | Purpose |
|---|---|
| `vanta-data.ts` | Centralized static content (1109 lines) — product data, nav structure, blog posts, case studies, FAQ |
| `page-transition.tsx` | Framer-motion AnimatePresence wrapper |
| `skip-link.tsx` | Accessible skip link |

## Hooks (`src/hooks/`)

| Hook | File | Purpose |
|---|---|---|
| `useVantaNavigate` | `use-vanta-navigate.ts` | Legacy `View` → `router.push()` adapter (92 lines) |
| `useCountUp` | `count-up.tsx` | IntersectionObserver + rAF number animation |
| `useReveal` | `use-reveal.ts` | IntersectionObserver for scroll reveal |
| `useMobile` | `use-mobile.ts` | Mobile detect hook |
| `useParallax` | `use-parallax.ts` | Scroll parallax |
| `useFocusTrap` | `use-focus-trap.ts` | Focus trap for modals |
| `useTypingLines` | `use-typing-lines.ts` | Typewriter animation for code terminal |
| `useToast` | `use-toast.ts` | shadcn/ui toast hook |

## Dead Components (Imported by Nothing)

| Component | Lines | Replaced By |
|---|---|---|
| `navbar.tsx` | 546 | `site-navbar.tsx` |
| `hero-mark-interactive.tsx` | 312 | `mark-classic.tsx` |
| `ecosystem.tsx` | 168 | Removed from HomeView |
| `metrics-bar.tsx` | 131 | Removed from HomeView |
| **Total** | **1,157** | |

## shadcn/ui Primitives (`src/components/ui/`)

Standard set (not documented here — refer to [shadcn/ui docs](https://ui.shadcn.com/docs/components)).
