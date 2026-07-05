# Index Redesign — Swiss+Neubrutalism

**Goal:** Redesign all 13 sections of the index page applying DESIGN_RULES.md (Swiss grid + Neubrutalism hard edges).

**Architecture:** CSS-only changes for most components (borders, shadows, spacing, typography). HTML/JSX changes for MetricsBar (hero-metric template → metric blocks) and section header patterns.

**Design rules applied:**
- Grid visible: 2px section borders, grid background
- Typography: text-display → text-title → text-body scale, nb-label for metadata
- Borders: 2px solid, 0px border-radius
- Shadows: hard-offset (6px 6px 0px 0px)
- Color: 95% neutral, 5% amber accent
- Whitespace: space-3xl/4xl between sections
- Asymmetry: grids 2fr 1fr, 8fr 4fr
- No colored left borders, no blur shadows, no numbered section markers

**Excluded (per user request):** Glitch effects + colored macOS dots in NbTerminalHero

---

### Phase 1: CSS Infrastructure (nb-components.css)

**Files:**
- Modify: `web/src/styles/nb-components.css`
- Reference: `web/src/styles/tokens.css`

**Changes:**
- Update `.nb-section` with `border-top: 2px solid var(--border-strong)` and `padding: var(--space-4xl) 0`
- Update `.nb-divider` as 2px solid separator
- Add `.nb-grid-bg` pattern (subtle grid background)
- Add `.nb-section-header` with proper typography
- Define `.nb-label` for metadata
- Add `.nb-metric-block` for metrics
- Add `.nb-hero-grid`, `.nb-asymmetric-grid` patterns
- Add `.nb-btn` base styles with hard-offset shadows

---

### Phase 2: Component Redesigns

#### Group 1: TerminalHero + TrustBar + MetricsBar
- NbTerminalHero: keep glitch/dots, add 2px border to terminal window, hard shadow
- NbTrustBar: clean up, add 2px section top border
- NbMetricsBar: FULL RESTRUCTURE — remove hero-metric template, use nb-metric-block pattern from DESIGN_RULES.md

#### Group 2: FeatureGrid + CoreEngine
- NbFeatureGrid: remove colored left borders, use 2px borders + hard shadows on cards, hover effects
- NbCoreEngine: apply section standards, cleaner typography

#### Group 3: Quickstart + ArchPreview + BenchmarkRace
- NbQuickstart: fix blur shadow in quickstart.css, apply section standards
- NbArchPreview: remove border-left patterns, use 2px borders + hard shadows
- NbBenchmarkRace: apply section standards, fix bar styles

#### Group 4: UseCases + Ecosystem + FaqAccordion
- NbUseCases: remove colored left borders, use 2px borders + hard shadows
- NbEcosystem: apply section standards, fix card patterns
- NbFaqAccordion: apply section standards

#### Group 5: PricingPreview + Monolith
- NbPricingPreview: apply section standards, fix card patterns
- NbMonolith: fix hardcoded colors in monolith.css, apply section standards

---

### Phase 3: Polish & Audit
- Run `npx tsc --noEmit`
- Run `pwsh scripts/audit-tokens.ps1`
- Visual check with Playwright screenshots
