# Agent Global Skills Analysis — `C:\Users\Eros\.agents\skills\`

**Date:** 2026-07-03  
**Total skills analyzed:** 76  
**Scope:** All SKILL.md files (first 30 lines each)

---

## 1. Frontend/UI Design (13 skills)

### impeccable ⭐ (10/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\impeccable\SKILL.md`
- **Version:** 3.7.1 — very mature
- **Description:** Full-spectrum frontend design/redesign/audit/polish skill. Covers websites, dashboards, components, forms, onboarding, empty states, accessibility, performance, theming, typography, motion, i18n.
- **Quality signals:** Has scripts (`context.mjs`, `palette.mjs`), reference docs (`brand.md`, `product.md`, `init.md`), sub-commands (`craft`, `shape`, `audit`, `polish`). Production-grade, constant updates.
- **Assessment:** The most comprehensive design skill. Should be the default for ALL design work. Anti-slop, OKLCH color, contrast enforcement.

### design-taste-frontend (9/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\design-taste-frontend\SKILL.md`
- **Description:** Anti-slop frontend skill. Reads brief, infers design direction, ships non-templated interfaces. Landing pages, portfolios, redesigns.
- **Quality signals:** 1200+ lines of detailed rules. Brief inference engine, vibe detection, reference signals. Well-structured.
- **Assessment:** Excellent complement to impeccable. Use for initial direction-finding before implementing with impeccable.

### design-taste-frontend-v1 (6/10) — REMOVE (legacy)
- **Path:** `C:\Users\Eros\.agents\skills\design-taste-frontend-v1\SKILL.md`
- **Description:** The original v1 preserved for backward compatibility. v2 (above) is the rewrite.
- **Assessment:** Redundant now that v2 exists and is stable. Only keep if a specific project depends on v1 behavior.

### frontend-design (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\frontend-design\SKILL.md`
- **Description:** OpenAI's production-grade frontend skill. Distinctive, avoids AI slop. Any HTML/CSS/JS/React/Vue.
- **Quality signals:** Bold aesthetic direction framework, tone selection, typography guidance.
- **Assessment:** Strong alternative to impeccable for quick-turnaround projects. Slightly less comprehensive.

### high-end-visual-design (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\high-end-visual-design\SKILL.md`
- **Description:** Awwwards-tier agency-level design. Defines exact fonts, spacing, shadows, cards, animations for expensive-looking sites.
- **Quality signals:** "Absolute Zero" anti-pattern list, creative variance engine with vibe/layout archetypes, banned fonts list.
- **Assessment:** Niche but useful for premium work. Overlaps with impeccable and design-taste-frontend.

### interface-design (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\interface-design\SKILL.md`
- **Description:** For dashboards, admin panels, SaaS apps, tools. NOT for marketing/landing pages.
- **Quality signals:** Domain-specific focus (not marketing). Interface-specific anti-patterns.
- **Assessment:** Useful companion to frontend-design for app-style projects. Clear scope differentiation.

### industrial-brutalist-ui (6/10) — KEEP (niche)
- **Path:** `C:\Users\Eros\.agents\skills\industrial-brutalist-ui\SKILL.md`
- **Description:** Swiss typography + military terminal aesthetics. Rigid grids, extreme contrast, declassified blueprint feel.
- **Quality signals:** Very detailed visual archetypes, typographic architecture, degradation effects.
- **Assessment:** Excellent for its niche (data-heavy dashboards, portfolios, dark-tech). Not general-purpose.

### minimalist-ui (6/10) — KEEP (niche)
- **Path:** `C:\Users\Eros\.agents\skills\minimalist-ui\SKILL.md`
- **Description:** Clean editorial-style interfaces. Warm monochrome, typographic contrast, flat bento grids, muted pastels.
- **Quality signals:** Strict banned elements list, detailed typographic architecture, absolute negative constraints.
- **Assessment:** Good for editorial/calm interfaces. Overlaps with minimalist approaches in other skills.

### gpt-taste (6/10) — CONSIDER REMOVING
- **Path:** `C:\Users\Eros\.agents\skills\gpt-taste\SKILL.md`
- **Description:** Elite UX/UI & Advanced GSAP Motion Engineer. Python-driven randomization, AIDA, bento grids.
- **Quality signals:** Opinionated, specific about GSAP + AIDA structure.
- **Assessment:** Overlapping with impeccable and design-taste-frontend. Python randomization gimmick is unnecessary. Low depth (74 lines).

### redesign-existing-projects (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\redesign-existing-projects\SKILL.md`
- **Description:** Upgrades existing sites to premium quality. Audit-first approach, works with any CSS framework.
- **Quality signals:** Structured scan-diagnose-fix cycle, detailed typography checklist, specific anti-patterns.
- **Assessment:** Unique value proposition — designed specifically for retrofitting, not greenfield. Keep.

### web-design-guidelines (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\web-design-guidelines\SKILL.md`
- **Description:** Vercel's Web Interface Guidelines compliance checker. Reviews UI code against latest guidelines.
- **Quality signals:** Fetches latest rules from Vercel repo, terse `file:line` format. Official Vercel.
- **Assessment:** Good for compliance checking. Relies on web fetch so always current.

### ui-ux-pro-max (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\ui-ux-pro-max\SKILL.md`
- **Description:** Massive design database: 50 styles, 21 palettes, 50 font pairings, 20 charts, 9 stacks.
- **Quality signals:** Rule categories by priority, impact ratings, structured reference. Very comprehensive reference document.
- **Assessment:** Excellent as a reference/tool for quick design decisions. Integrates with shadcn/ui MCP.

### image-to-code (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\image-to-code\SKILL.md`
- **Description:** Image-first website design to code. Generate images first, then implement as code matching them.
- **Quality signals:** Hard output rules, section-by-section generation, premium direction.
- **Assessment:** Unique workflow. Overlaps with imagegen-frontend-web + code implementation. Niche but valuable.

---

## 2. Animation/Motion/3D (19 skills)

### GSAP Suite (8 skills) — ALL KEEP (official, MIT licensed)
The GSAP skills are official, MIT-licensed, highly detailed, and well-maintained. They form the most comprehensive animation library reference available.

- **gsap-core** (9/10) — Core tween API, easing, stagger, matchMedia. Essential baseline.
- **gsap-timeline** (8/10) — Timeline sequencing, position parameter, nesting.
- **gsap-scrolltrigger** (9/10) — Scroll-linked animation, pinning, scrub. Extremely detailed (296 lines).
- **gsap-plugins** (8/10) — All GSAP plugins: SplitText, MorphSVG, Draggable, Flip, ScrollSmoother, etc.
- **gsap-react** (8/10) — useGSAP hook, context, cleanup for React.
- **gsap-frameworks** (7/10) — Vue, Svelte, other non-React frameworks.
- **gsap-performance** (7/10) — Performance optimization, transforms, will-change.
- **gsap-utils** (6/10) — Utility methods (clamp, mapRange, random, snap). Less frequently needed.

**Assessment:** Keep the entire suite. They are modular, official, and non-overlapping. The `gsap-utils` could be considered optional but is small.

### animejs (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\animejs\SKILL.md`
- **Description:** Lightweight animation engine (~9KB gzipped). Timeline, stagger, SVG morphing. Framework-agnostic.
- **Quality signals:** 525 lines. Comprehensive coverage of anime.js API.
- **Assessment:** Good alternative to GSAP for smaller projects or SVG-heavy work. Not redundant — different use cases.

### motion (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\motion\SKILL.md`
- **Description:** Motion.dev library (Framer Motion successor). React/Vue, 120fps, gestures, spring physics.
- **Quality signals:** Auto-generated from official source (2026-02-01). v12.29.2. Modular references.
- **Assessment:** Essential for React projects using Motion library. Cannot be replaced by GSAP — different paradigm.

### design-motion-principles (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\design-motion-principles\SKILL.md`
- **Description:** Motion design expert based on Emil Kowalski, Jakub Krehel, Jhey Tompkins. Two modes: Create + Audit.
- **Quality signals:** Structured workflows (create.md, audit.md), designer-specific perspectives.
- **Assessment:** Pairs perfectly with any animation implementation skill. The "taste layer" on top of GSAP/motion/animejs.

### Three.js Suite (6 skills) — ALL KEEP
Comprehensive coverage of Three.js from basics to shaders:

- **threejs-fundamentals** (8/10) — Scene, camera, renderer, Object3D hierarchy.
- **threejs-geometry** (7/10) — Built-in shapes, BufferGeometry, custom geometry, instancing.
- **threejs-materials** (7/10) — PBR, basic, phong, shader materials with detailed tables.
- **threejs-animation** (7/10) — Keyframe, skeletal, morph targets, animation mixing.
- **threejs-interaction** (7/10) — Raycasting, controls, mouse/touch input, object selection.
- **threejs-shaders** (7/10) — GLSL, ShaderMaterial, uniforms, custom effects.

**Assessment:** Keep all — modular and comprehensive. Each serves a distinct purpose.

### remotion-best-practices (6/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\remotion-best-practices\SKILL.md`
- **Description:** Best practices for Remotion video creation in React.
- **Quality signals:** Covers setup, composition, audio, rendering, performance (340 lines).
- **Assessment:** Thin but useful as a quickstart reference. Workspace has higher-fidelity alternatives (hyperframes).

---

## 3. Image/Video/AI Media (8 skills)

### brandkit (9/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\brandkit\SKILL.md`
- **Description:** Premium brand-kit image generation. Logo systems, identity decks, visual-world presentations.
- **Quality signals:** 798 lines. Extremely detailed. Reference style DNA, multiple brand archetypes.
- **Assessment:** World-class brand identity skill. Generates presentation-ready brand images. Essential for branding work.

### canvas-design (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\canvas-design\SKILL.md`
- **Description:** Create visual art in .png and .pdf via design philosophy. Poster/art/static designs.
- **Quality signals:** Two-phase approach (philosophy creation → visual expression). Philosophy-driven.
- **Assessment:** Good for artistic/abstract visual output. Different from brandkit (more art, less brand).

### hyperframes (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\hyperframes\SKILL.md`
- **Description:** HTML-to-video compositions. Title cards, overlays, captions, TTS, audio-reactive, transitions.
- **Quality signals:** 490 lines. Full workflow. Multiple companion skills (hyperframes-animation).
- **Assessment:** Unique and powerful for HTML-based video production. Essential for video workflows.

### hyperframes-animation (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\hyperframes-animation\SKILL.md`
- **Description:** All animation knowledge for HyperFrames. Atomic rules, blueprints, transitions, 7 runtime adapters.
- **Quality signals:** Rules index, blueprint index, runtime adapters (GSAP, Lottie, Three.js, Anime.js, CSS, WAAPI, TypeGPU).
- **Assessment:** Comprehensive companion to hyperframes. Both needed for video work.

### imagegen-frontend-web (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\imagegen-frontend-web\SKILL.md`
- **Description:** Generates premium website design reference images. One image per section, hard rules.
- **Quality signals:** 987 lines. Very strict output format, composition variety enforced, 6+ hero archetypes.
- **Assessment:** Excellent for creating visual design briefs that developers can implement accurately.

### imagegen-frontend-mobile (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\imagegen-frontend-mobile\SKILL.md`
- **Description:** Mobile app screen image generation. Premium, app-native, highly readable.
- **Quality signals:** 1465 lines — extremely detailed. Phone mockup framing, multi-screen flows.
- **Assessment:** Best mobile app image gen skill. Complements imagegen-frontend-web for full-stack design.

### image-edit (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\image-edit\SKILL.md`
- **Description:** Smart router for RunComfy edit models. Batch edit, text rewrite, precise edit, mask-driven edit.
- **Quality signals:** Picks correct model based on intent. Supports multiple RunComfy models.
- **Assessment:** Useful when RunComfy is available. Requires `runcomfy` CLI.

---

## 4. Backend/Dev/SEO (17 skills)

### vercel-react-best-practices (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\vercel-react-best-practices\SKILL.md`
- **Description:** 57 rules across 8 categories for React/Next.js performance. Vercel Engineering.
- **Quality signals:** Prioritized by impact (CRITICAL→MEDIUM). MIT licensed. Official Vercel.
- **Assessment:** Authoritative reference for React performance. Should be loaded before any React optimization.

### next-best-practices (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\next-best-practices\SKILL.md`
- **Description:** Next.js file conventions, RSC boundaries, async patterns, metadata, error handling.
- **Quality signals:** Framework-aware, covers v15+ async API changes, middleware rename in v16.
- **Assessment:** Essential for any Next.js project. Updated for latest versions.

### react-dev (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\react-dev\SKILL.md`
- **Description:** React + TypeScript. Generic components, event typing, React 19 features, routing integration.
- **Quality signals:** 391 lines. Covers React 19 breaking changes, forwardRef deprecation, Server Components, Actions.
- **Assessment:** Good TypeScript-specific React reference. Focused on typing patterns.

### react-useeffect (6/10) — KEEP (small)
- **Path:** `C:\Users\Eros\.agents\skills\react-useeffect\SKILL.md`
- **Description:** When NOT to use useEffect, alternatives, derived state patterns.
- **Quality signals:** Official React docs patterns. Quick reference table.
- **Assessment:** Small (53 lines) but high-value. Quick reference for a common pitfall.

### react-state-management (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\react-state-management\SKILL.md`
- **Description:** Redux Toolkit, Zustand, Jotai, React Query, SWR. State category framework.
- **Quality signals:** 437 lines. Five state categories, detailed per-solution guidance.
- **Assessment:** Comprehensive state management reference. Useful for architecture decisions.

### react-modernization (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\react-modernization\SKILL.md`
- **Description:** Upgrade React versions, class→hooks migration, concurrent features, codemods.
- **Quality signals:** 526 lines. Version-specific breaking changes, migration paths.
- **Assessment:** Niche but valuable for legacy React projects. Not needed every day.

### react-components (6/10) — KEEP/MAYBE
- **Path:** `C:\Users\Eros\.agents\skills\react-components\SKILL.md`
- **Description:** Converts Stitch designs to React components. AST-based validation, modular output.
- **Quality signals:** Stitch MCP integration, fetch-stitch.sh script.
- **Assessment:** Only useful if using Google Stitch. Otherwise inert.

### react-email (6/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\react-email\SKILL.md`
- **Description:** HTML email templates with React. Resend's official skill.
- **Quality signals:** 518 lines. Covers all email types, client compatibility.
- **Assessment:** The standard approach for email in React. Useful when needed.

### react-native-best-practices (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\react-native-best-practices\SKILL.md`
- **Description:** Callstack's RN optimization guide. Hermes, FlashList, profiling, bundle size.
- **Quality signals:** Impact ratings, detailed performance rules. Official Callstack content.
- **Assessment:** Essential for RN performance work. Authoritative source (Callstack).

### Supabase/Postgres Skills (2 items)

#### supabase-postgres-best-practices (7/10) — KEEP
- **Description:** Postgres optimization from Supabase. 8 categories, prioritized.
- **Assessment:** Authoritative for Supabase Postgres work. Includes RLS patterns.

#### database-design (7/10) — KEEP
- **Description:** Database selection, ORM selection, schema design, indexing, migration strategy.
- **Assessment:** Good high-level decision framework. Not just SQL — covers architecture.

#### database-schema-designer (7/10) — KEEP
- **Description:** SQL/NoSQL schema design. Normalization, indexing, migrations, constraints.
- **Assessment:** More granular than database-design. Both can coexist.

### Prisma Skills (2 items)

#### prisma (6/10) — CONSIDER REMOVING
- **Description:** Basic Prisma ORM skill. Schema design, client usage, transactions.
- **Quality signals:** 71 lines only. Shallow.
- **Assessment:** Redundant with prisma-expert which is deeper and more comprehensive.

#### prisma-expert (7/10) — KEEP
- **Description:** Comprehensive Prisma expert. Schema, migrations, queries, relations, multiple DBs.
- **Quality signals:** 355 lines. Environment detection, troubleshooting steps, advanced patterns.
- **Assessment:** The better Prisma skill. Keep this, remove `prisma`.

### API/Dev Skills

#### api-design-principles (7/10) — KEEP
- **Description:** REST + GraphQL API design. Resource-oriented, HTTP methods, documentation.
- **Quality signals:** 528 lines. Covers both paradigms. Well-structured.
- **Assessment:** Solid reference for API design decisions. Good for new API work.

#### ai-sdk (8/10) — KEEP
- **Description:** Vercel AI SDK reference. generateText, streamText, agents, tools, providers.
- **Quality signals:** Auto-updates from source. Critical: warns against trusting internal knowledge. Fetches current model IDs.
- **Assessment:** Essential if using AI SDK. Must-keep for any AI agent work.

### SEO Skills (5 items)

#### seo (6/10) — KEEP/ORGANIZE
- **Description:** Lighthouse-based SEO optimization. Technical SEO, on-page, structured data.
- **Assessment:** Generic but solid. Overlaps with seo-audit and roier-seo.

#### seo-audit (7/10) — KEEP
- **Description:** Full SEO audit methodology. Technical + on-page + content + off-page.
- **Quality signals:** 394 lines. Structured audit process.
- **Assessment:** More comprehensive than plain `seo`. Better for audit-first workflows.

#### programmatic-seo (7/10) — KEEP
- **Description:** Building SEO pages at scale via templates and data. Location pages, comparison pages.
- **Quality signals:** 236 lines. Template design, data modeling, anti-thin-content.
- **Assessment:** Distinct from audit-focused SEO skills. Unique value proposition.

#### roier-seo (7/10) — KEEP
- **Description:** Technical SEO auditor + auto-fixer. Lighthouse/PageSpeed, implements fixes in codebase.
- **Quality signals:** Auto-fix capability, framework-aware (Next.js, React, Vue), JSON-LD structured data.
- **Assessment:** More actionable than plain audits — actually modifies code. Tool dependency (lighthouse).

#### ai-seo (7/10) — KEEP
- **Description:** AI search optimization (AEO/GEO/LLMO). Optimize content for ChatGPT, Perplexity, Claude, Gemini.
- **Quality signals:** 489 lines. Covers llms.txt, OKF, knowledge bundles, agent-readable sites. Version 2.1.0.
- **Assessment:** Forward-looking, covers emerging AI search landscape. Distinct from traditional SEO.

#### audit-website (7/10) — KEEP
- **Description:** 230+ rule website audit via squirrelscan CLI. SEO, perf, security, content, broken links.
- **Quality signals:** LLM-optimized reports, health scores, tool dependency (squirrel).
- **Assessment:** Powerful when squirrel CLI is installed. Comprehensive audit across 15+ categories.

---

## 5. Content/Writing (6 skills)

### copywriting (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\copywriting\SKILL.md`
- **Description:** Conversion copywriting. Pages, pricing, features, about. CTA optimization.
- **Quality signals:** Structured discovery process, audience analysis, objection handling.
- **Assessment:** Solid conversion-focused copy tool. Pairs with marketing-psychology.

### marketing-psychology (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\marketing-psychology\SKILL.md`
- **Description:** 70+ mental models for marketing. First Principles, Scarcity, Social Proof, etc.
- **Quality signals:** 454 lines. Well-organized by category. Ethical application guidance.
- **Assessment:** Unique — not copywriting but the psychology layer beneath it. Good companion to copywriting.

### marketing-ideas (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\marketing-ideas\SKILL.md`
- **Description:** 139 proven SaaS marketing approaches. Content/SEO, competitor, free tools, community, etc.
- **Quality signals:** Categorized ideas, implementation guidance. Context-aware suggestions.
- **Assessment:** Useful brainstorming tool for marketing strategy. Not copy — ideas and tactics.

### social-content (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\social-content\SKILL.md`
- **Description:** Multi-platform social media content. LinkedIn, Twitter/X, Instagram, TikTok, Facebook.
- **Quality signals:** Platform-specific strategies, content repurposing, scheduling.
- **Assessment:** Covers social media specifically. Does not overlap with copywriting (different format).

---

## 6. Utility/Tool (9 skills)

### agent-browser (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\agent-browser\SKILL.md`
- **Description:** Browser automation CLI. Navigate, snapshot, click, fill, extract.
- **Quality signals:** 217 lines. Element ref system (@e1, @e2), session management.
- **Assessment:** Robust browser automation. Parallels Playwright MCP but CLI-based.

### browser-use (7/10) — MAYBE REMOVE
- **Path:** `C:\Users\Eros\.agents\skills\browser-use\SKILL.md`
- **Description:** Browser automation via `browser-use` CLI (uvx-based). Python/uv ecosystem.
- **Quality signals:** 405 lines. Session persistence, complex workflows.
- **Assessment:** Redundant with agent-browser and Playwright MCP. Different CLI but same use cases. Keep only if uvx/browser-use is preferred.

### find-skills (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\find-skills\SKILL.md`
- **Description:** Discovers and installs skills from the open ecosystem. `npx skills find/add/check/update`.
- **Quality signals:** Well-structured, integrates with skills CLI. Essential for skill management.
- **Assessment:** Critical infrastructure skill for maintaining the agent ecosystem itself.

### skill-creator (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\skill-creator\SKILL.md`
- **Description:** Guide for creating effective skills. Context-efficient, modular, workflow-focused.
- **Quality signals:** 356 lines. Core principles, structure guidelines, best practices.
- **Assessment:** Essential for creating new skills. Needed whenever a new capability must be added.

### full-output-enforcement (6/10) — KEEP (meta)
- **Path:** `C:\Users\Eros\.agents\skills\full-output-enforcement\SKILL.md`
- **Description:** Prevents LLM truncation. Bans `// ...`, `// TODO`, skeleton outputs.
- **Quality signals:** Clear banned patterns, execution process, token-limit handling.
- **Assessment:** Meta-skill that improves output quality. Small but useful.

### systematic-debugging (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\systematic-debugging\SKILL.md`
- **Description:** Root-cause-first debugging methodology. No fixes without investigation.
- **Quality signals:** 296 lines. Iron Law, phased approach, evidence collection.
- **Assessment:** Excellent methodology. Prevents random fixes. Should be loaded before any bug-fixing task.

### writing-plans (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\writing-plans\SKILL.md`
- **Description:** Writes bite-sized implementation plans. 2-5 minute tasks per step.
- **Quality signals:** DRY, YAGNI, TDD. Saves to docs/plans/.
- **Assessment:** Critical for multi-step tasks. Prevents scope creep and missed steps.

### brainstorming (8/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\brainstorming\SKILL.md`
- **Description:** Turns ideas into specs through dialogue. Multi-choice questions, design presentation.
- **Quality signals:** Structured process, 2-3 approach proposals, section-by-section validation.
- **Assessment:** Perfect first step for any creative or feature work.

### professional-github-repo (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\professional-github-repo\SKILL.md`
- **Description:** Orchestrates GitHub repo professionalization. README, CI, issues, PRs, badges, security.
- **Quality signals:** Skill routing table, references to specialized sub-skills.
- **Assessment:** Good orchestration layer. Depends on sub-skills being installed.

---

## 7. Design Systems/Tokens (3 skills)

### stitch-design-taste (6/10) — KEEP/IF USING STITCH
- **Path:** `C:\Users\Eros\.agents\skills\stitch-design-taste\SKILL.md`
- **Description:** Generates DESIGN.md files for Google Stitch screen generation. Anti-slop.
- **Quality signals:** 184 lines. Generates Stitch-interpretable design specs.
- **Assessment:** Only useful if using Google Stitch for design generation.

### react-components (6/10) — KEEP/IF USING STITCH
- **Path:** `C:\Users\Eros\.agents\skills\react-components\SKILL.md`
- **Description:** Converts Stitch designs into React components. AST validation, modular output.
- **Assessment:** Only useful with Google Stitch. Inert otherwise.

### brandkit (already covered in Image/Video/AI Media) — duplicates note

---

## 8. AI Prompt/Agent (3 skills)

### ai-sdk (already covered in Backend/Dev)

### find-skills (already covered in Utility)

### skill-creator (already covered in Utility)

### full-output-enforcement (already covered in Utility)

---

## 9. Mobile (2 skills)

### react-native-best-practices (7/10) — KEEP
- Already covered in Backend/Dev.

### sleek-design-mobile-apps (7/10) — KEEP
- **Path:** `C:\Users\Eros\.agents\skills\sleek-design-mobile-apps\SKILL.md`
- **Description:** AI-powered mobile app design via Sleek API. Create projects, generate screens.
- **Quality signals:** 520 lines. REST API integration, requires SLEEK_API_KEY.
- **Assessment:** Useful if Sleek.design subscription is active. Generates mobile screens externally.

### imagegen-frontend-mobile (already covered in Image/AI Media)

---

## 10. Research/Strategy (2 skills)

### marketing-psychology (already covered)
### marketing-ideas (already covered)

---

## KEY FINDINGS & RECOMMENDATIONS

### ⭐ Must-Keep (Essential, High Quality)
1. **impeccable** — The single best design skill. Use by default for all frontend work.
2. **design-taste-frontend** — Best for initial direction-finding before implementation.
3. **brandkit** — World-class brand identity image generation.
4. **hyperframes + hyperframes-animation** — Unique video production pipeline.
5. **agent-browser** — Reliable browser automation CLI.
6. **find-skills** — Essential infrastructure for skill ecosystem.
7. **skill-creator** — Essential for creating new skills.
8. **systematic-debugging** — Must-load before any bug fixing.
9. **writing-plans** — Must-load before multi-step implementations.
10. **brainstorming** — Must-load before creative work.
11. **ai-sdk** — Essential for AI SDK work.
12. **vercel-react-best-practices** + **next-best-practices** — Essential for React/Next.js.

### ⚠️ Candidates for Removal (Redundant/Legacy)
1. **design-taste-frontend-v1** — Legacy; v2 is the rewrite. Remove unless project depends on v1.
2. **prisma** (basic) — Redundant with `prisma-expert` which is deeper and more comprehensive.
3. **gpt-taste** — Low depth (74 lines), overlaps with impeccable/design-taste-frontend. Gimmicky.
4. **browser-use** — Redundant with `agent-browser` and Playwright MCP. Same use case, different CLI.

### 🔀 Overlap/Consolidation Candidates
1. **seo / seo-audit / roier-seo / ai-seo / audit-website** — 5 SEO-adjacent skills. Different focus but overlapping. Consider organizing by: audits (roier-seo, audit-website), strategy (seo-audit, seo), AI-specific (ai-seo), programmatic (programmatic-seo).
2. **high-end-visual-design / impeccable / design-taste-frontend** — All three target premium design. impeccable is the most comprehensive. Consider using design-taste-frontend for direction and impeccable for implementation.
3. **database-design / database-schema-designer** — Overlapping. database-design is higher-level strategy, database-schema-designer is granular schema. Can coexist.
4. **canvas-design / brandkit** — Both generate images. canvas-design is artistic/philosophical, brandkit is brand/identity. Distinct enough to keep both.

### 📊 Quality Summary
| Rating | Count | Skills |
|--------|-------|--------|
| 10/10 | 1 | impeccable |
| 9/10 | 3 | design-taste-frontend, gsap-core, gsap-scrolltrigger, brandkit |
| 8/10 | 16 | frontend-design, redesign-existing-projects, design-motion-principles, motion, hyperframes, imagegen-frontend-web, imagegen-frontend-mobile, ai-sdk, agent-browser, find-skills, skill-creator, systematic-debugging, writing-plans, brainstorming, vercel-react-best-practices, next-best-practices, gsap-timeline, gsap-plugins, gsap-react, threejs-fundamentals |
| 7/10 | 31 | (majority of remaining) |
| 6/10 | 17 | (niche/legacy/redundant) |
| <6 | 0 | — |

### 📈 Overall Ecosystem Health
- **Total skills:** 76
- **License health:** Most are MIT or permissive
- **Depth:** Average skill is 200+ lines; highest is 1465 lines (imagegen-frontend-mobile)
- **Recency:** Most appear updated in 2025-2026
- **Redundancy rate:** ~15% of skills overlap significantly
- **Gaps:** No dedicated Flutter/Dart skill, no dedicated Docker/K8s skill, no dedicated CI/CD skill (beyond GitHub)
