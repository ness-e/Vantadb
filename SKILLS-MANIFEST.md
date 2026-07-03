# VantaDB Skills Manifest

**Location:** `.agents/skills/` (project-local, 154 skills)
**Updated:** 2026-07-03
**Purpose:** Reference for AI agents to know which skills are available and when to use them.

---

## Quick Navigation

- [Core 50 — Essential Skillset](#core-50--essential-skillset)
- [All Skills by Category](#all-skills-by-category)
- [Skill Loading Guide](#skill-loading-guide)

---

## Core 50 — Essential Skillset

These 50 skills form the complete, lean toolset for VantaDB work. Load the relevant ones based on task type.

### Frontend/UI Design (12)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `impeccable` | 10/10 | **Default for all frontend work.** Design audit, critique, polish. v3.7.1. |
| `design-taste-frontend` | 9/10 | Anti-slop direction finder. Load BEFORE implementing to establish design direction. |
| `frontend-design` | 8/10 | OpenAI production playbook. Landing pages, components, artifacts. |
| `high-end-visual-design` | 8/10 | Agency-level design rules. "Absolute Zero" anti-pattern list. Makes sites look expensive. |
| `awesome-claude-design` | 7/10 | 9 aesthetic families reference. Use for design inspiration and direction. |
| `industrial-brutalist-ui` | 7/10 | Data-heavy dashboards, portfolios, declassified-blueprint aesthetics. |
| `minimalist-ui` | 7/10 | Clean editorial-style interfaces. Warm monochrome palette. |
| `interface-design` | 7/10 | App/dashboard focused. Not for marketing pages. |
| `redesign-existing-projects` | 8/10 | Upgrade existing websites to premium quality. Audit-first approach. |
| `ui-ux-pro-max` | 7/10 | Design reference database (50 styles, 21 palettes, 50 font pairings). |
| `web-design-guidelines` | 7/10 | Auto-updating compliance checker. Use for accessibility and standards reviews. |
| `image-to-code` | 7/10 | Generate design images then implement from them. Image-first workflow. |

### Animation/Motion (10)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `gsap-core` | 9/10 | **Foundation for all GSAP animation.** MIT license. |
| `gsap-scrolltrigger` | 9/10 | Scroll-linked animation, pinning, scrub. Extremely detailed. |
| `gsap-timeline` | 8/10 | Timeline sequencing, position parameter, nesting. |
| `gsap-plugins` | 8/10 | SplitText, MorphSVG, Draggable, ScrollSmoother, CustomEase. |
| `gsap-react` | 8/10 | useGSAP hook, React cleanup, gsap.context(). |
| `motion` | 9/10 | **Preferred over CSS for animations per AGENTS.md.** v12.29.2, 120fps, gestures. |
| `animejs` | 8/10 | Lightweight GSAP alternative. SVGs, timelines, stagger. |
| `design-motion-principles` | 8/10 | Create + Audit modes. Emil/Jhey/Jakub techniques. |
| `emil-design-eng` | 9/10 | Emil Kowalski UI polish philosophy. "Invisible details." |
| `interaction-design` | 8/10 | 1097 lines. Micro-animations, state machines, cognitive laws. |

### Three.js / 3D (6)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `threejs-fundamentals` | 8/10 | Scene, camera, renderer setup. Start here for any 3D work. |
| `threejs-geometry` | 8/10 | Custom geometry, BufferGeometry, instancing. |
| `threejs-materials` | 8/10 | PBR, textures, ShaderMaterial properties. |
| `threejs-animation` | 8/10 | Keyframe, skeletal, morph targets, animation mixer. |
| `threejs-interaction` | 9/10 | Raycasting, controls, object selection. |
| `threejs-shaders` | 9/10 | GLSL, ShaderMaterial, uniforms, custom effects. |

### Video / Deck (8)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `hyperframes` | 9/10 | HTML-to-video composition. TTS, captions, transitions. |
| `hyperframes-animation` | 8/10 | 7 runtime adapters. GSAP/Lottie/Three.js/CSS/WAAPI/TypeGPU. |
| `deck-open-slide-canvas` | 7/10 | Freeform 1920x1080 React canvas deck. |
| `deck-swiss-international` | 7/10 | 16-column Swiss grid, saturated accent, 22 locked layouts. |
| `field-notes-editorial-template` | 7/10 | Magazine-style business report with charts. |
| `after-hours-editorial-template` | 7/10 | Luxury fashion HyperFrames deck. Cinematic storyboard. |
| `remotion` | 5/10 | React native video creation. Shallow but important tech. |
| `pptx` | 5/10 | Official Anthropic PPTX generation. Keep over pptx-generator. |

### Image / Brand (4)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `brandkit` | 9/10 | Premium brand identity generation. Logo systems, identity decks. |
| `imagegen-frontend-web` | 9/10 | Premium web section images. One image per section. Hard output rules. |
| `imagegen-frontend-mobile` | 8/10 | Mobile app screen images with phone mockup framing. |
| `canvas-design` | 7/10 | Artistic visual output. .png/.pdf with design philosophy. |

### Design Systems (4)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `design-systems` | 8/10 | Complete design system management — tokens, components, accessibility. |
| `color-expert` | 8/10 | 286K words of color science. OKLCH/OKLAB, palette generation. |
| `theme-factory` | 8/10 | 10 preset themes + custom generator. Colors/typography for slides/docs. |
| `platform-design` | 8/10 | 300+ Apple HIG, Material Design 3, WCAG 2.2 rules. Cross-platform. |

### Backend / Dev / SEO (8)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `vercel-react-best-practices` | 9/10 | 70+ React/Next.js production rules from Vercel Engineering. |
| `next-best-practices` | 8/10 | Next.js v15+ conventions, RSC boundaries, data patterns. |
| `ai-sdk` | 8/10 | Vercel AI SDK. Auto-updates with current model IDs. |
| `database-schema-designer` | 7/10 | Schema design, normalization, indexing, migration patterns. |
| `prisma-expert` | 7/10 | Comprehensive Prisma ORM reference. Schema, queries, relations. |
| `api-design-principles` | 7/10 | REST + GraphQL API design. 528 lines. |
| `ai-seo` | 8/10 | AI search optimization. Optimize for LLM citation. |
| `seo-audit` | 7/10 | Full SEO audit. Technical, on-page, content issues. |

### Utility / Infrastructure (6)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `vanta-design-orchestrator` | 9/10 | **Load FIRST for any design task.** Orchestrates all other design skills. |
| `systematic-debugging` | 8/10 | **Load BEFORE any bug fix.** Root-cause-first methodology. |
| `writing-plans` | 8/10 | **Load before multi-step implementation.** Bite-sized plans. |
| `brainstorming` | 8/10 | **Load before creative work.** Idea-to-spec dialogue. |
| `find-skills` | 8/10 | Discover and install new skills. Essential for ecosystem growth. |
| `skill-creator` | 8/10 | Create or update skills. Essential for ecosystem maintenance. |

### Research / Strategy (3)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `ux-heuristics` | 8/10 | Nielsen/Krug framework. Usability audits, cognitive walkthroughs. |
| `research-decision-room` | 8/10 | Research synthesis dashboard. Evidence ledger, decision memo. |
| `ux-strategy` | 7/10 | Competitive analysis, experience mapping, IA, content strategy. |

---

## All Skills by Category

### Frontend / UI Design

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| impeccable | 10 | KEEP | Primary design authority. v3.7.1. |
| design-taste-frontend | 9 | KEEP | Anti-slop direction finder. |
| frontend-design | 8 | KEEP | OpenAI production playbook. |
| high-end-visual-design | 8 | KEEP | Agency-level rules. |
| awesome-claude-design | 7 | KEEP | 9 aesthetic families. |
| industrial-brutalist-ui | 7 | KEEP | Data-heavy UI. |
| minimalist-ui | 7 | KEEP | Editorial style. |
| interface-design | 7 | KEEP | App/dashboard focused. |
| redesign-existing-projects | 8 | KEEP | Upgrade existing sites. |
| ui-ux-pro-max | 7 | KEEP | Design reference DB. |
| web-design-guidelines | 7 | KEEP | Compliance checker. |
| image-to-code | 7 | KEEP | Image-first workflow. |
| article-magazine | 6 | KEEP | Magazine layout. |
| card-twitter | 5 | KEEP | Twitter card. |
| card-xiaohongshu | 5 | KEEP | Xiaohongshu card. |
| faq-page | 6 | KEEP | FAQ template. |
| impeccable-design-polish | 7 | KEEP | Polish companion. |
| web-artifacts-builder | 5 | KEEP | Official Anthropic ref. |
| mockup-device-3d | 7 | KEEP | iPhone/MacBook showcase. |
| doc-kami-parchment | 6 | KEEP | Editorial one-pager. |
| ui-design | 8 | KEEP | 859 lines UI theory. |
| taste-skill | 9 | KEEP | Alias for design-taste-frontend. |
| gpt-taste | 7 | KEEP | Editorial/GSAP taste. |
| shadcn-ui | 6 | KEEP | shadcn/ui reference. |
| stitch-design-taste | 8 | KEEP | Stitch DESIGN.md gen. |
| stitch-loop | 5 | KEEP | Design-to-code iteration. |

### Animation / Motion / 3D

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| gsap-core | 9 | KEEP | Official GSAP core. |
| gsap-scrolltrigger | 9 | KEEP | Scroll-linked. |
| gsap-timeline | 8 | KEEP | Sequencing. |
| gsap-plugins | 8 | KEEP | SplitText, Draggable. |
| gsap-react | 8 | KEEP | useGSAP hook. |
| gsap-frameworks | 7 | KEEP | Vue/Svelte. |
| gsap-performance | 7 | KEEP | 60fps optimization. |
| gsap-utils | 7 | KEEP | Utility methods. |
| motion | 9 | KEEP | motion.dev v12. |
| animejs | 8 | KEEP | Lightweight alternative. |
| design-motion-principles | 8 | KEEP | Create + Audit modes. |
| emil-design-eng | 9 | KEEP | UI polish philosophy. |
| emilkowalski-motion | 7 | KEEP | Micro-interaction follow-up. |
| interaction-design | 8 | KEEP | 1097 lines. |
| threejs-fundamentals | 8 | KEEP | Scene/camera/renderer. |
| threejs-geometry | 8 | KEEP | Geometry creation. |
| threejs-materials | 8 | KEEP | PBR/textures. |
| threejs-animation | 8 | KEEP | Animation techniques. |
| threejs-interaction | 9 | KEEP | Raycasting/controls. |
| threejs-shaders | 9 | KEEP | GLSL/ShaderMaterial. |
| algorithmic-art | 8 | KEEP | p5.js generative. |
| shader-dev | 5 | KEEP | GLSL reference. |
| video-hyperframes | 6 | KEEP | Hyperframes bridge. |
| vfx-text-cursor | 6 | KEEP | VFX text reveal. |
| remotion | 5 | KEEP | React video. |

### Deck / Slide / Video Templates

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| hyperframes | 9 | KEEP | Core video composition. |
| hyperframes-animation | 8 | KEEP | 7 runtime adapters. |
| deck-open-slide-canvas | 7 | KEEP | Freeform React canvas. |
| deck-swiss-international | 7 | KEEP | 16-column Swiss grid. |
| deck-guizang-editorial | 7 | KEEP | Editorial e-ink deck. |
| field-notes-editorial-template | 7 | KEEP | Magazine report. |
| digits-fintech-swiss-template | 7 | KEEP | Fintech data deck. |
| editorial-burgundy-principles | 7 | KEEP | Manifesto deck. |
| html-ppt-retro-quarterly-review | 7 | KEEP | Retro quarterly deck. |
| after-hours-editorial-template | 7 | KEEP | Luxury fashion deck. |
| swiss-creative-mode-template | 7 | KEEP | Swiss editorial deck. |
| swiss-user-research-video-template | 7 | KEEP | Research deck. |
| weread-year-in-review-video-template | 7 | KEEP | 9:16 report. |
| ppt-keynote | 7 | KEEP | Keynote-quality HTML. |
| pptx | 5 | KEEP | Official Anthropic. |
| pptx-html-fidelity-audit | 8 | KEEP | PPTX quality control. |
| frame-data-chart-nyt | 6 | KEEP | NYT-style chart. |
| frame-flowchart-sticky | 6 | KEEP | Flowchart template. |
| frame-glitch-title | 6 | KEEP | Glitch title. |
| frame-light-leak-cinema | 6 | KEEP | Cinematic frame. |
| frame-liquid-bg-hero | 6 | KEEP | Fluid background. |
| frame-logo-outro | 6 | KEEP | Logo outro. |
| frame-macos-notification | 6 | KEEP | macOS overlay. |
| 8-bit-orbit-video-template | 6 | KEEP | Retro pixel deck. |
| frontend-slides | 3 | REMOVED | Redundant. |

### Image / Video / AI Media

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| brandkit | 9 | KEEP | Brand identity gen. |
| imagegen-frontend-web | 9 | KEEP | Premium web images. |
| imagegen-frontend-mobile | 8 | KEEP | Mobile screen images. |
| canvas-design | 7 | KEEP | Artistic visual output. |
| ecommerce-image-workflow | 7 | KEEP | Product image pipeline. |
| poster-hero | 6 | KEEP | Vertical poster. |
| screenshots-marketing | 6 | KEEP | Marketing screenshots. |
| social-reddit-card | 6 | KEEP | Reddit card. |
| social-spotify-card | 6 | KEEP | Spotify card. |
| social-x-post-card | 6 | KEEP | X/Twitter card. |
| image-edit | 7 | KEEP | RunComfy smart router. |
| fal-generate | 3 | CONDITIONAL | Keep if fal.ai active. |
| fal-image-edit | 3 | CONDITIONAL | Keep if fal.ai active. |
| fal-upscale | 3 | CONDITIONAL | Keep if fal.ai active. |
| fal-vision | 3 | CONDITIONAL | Keep if fal.ai active. |

### Design Systems / Tokens

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| design-systems | 8 | KEEP | Complete DS management. |
| color-expert | 8 | KEEP | 286K words color science. |
| theme-factory | 8 | KEEP | 10 preset themes. |
| platform-design | 8 | KEEP | 300+ HIG/Material/WCAG. |
| stitch-design-taste | 8 | KEEP | Stitch DESIGN.md gen. |
| stitch-loop | 5 | KEEP | Design iteration. |
| reference-design-contract | 8 | KEEP | Reference→spec workflow. |
| visual-critique | 7 | KEEP | Systematic critique. |
| plan-design-review | 7 | KEEP | Quality gate. |
| shadcn-ui | 6 | KEEP | shadcn/ui reference. |
| figma-use | 4 | CONDITIONAL | Keep if Figma MCP. |
| figma-code-connect-components | 4 | CONDITIONAL | Same. |
| figma-create-design-system-rules | 4 | CONDITIONAL | Same. |
| figma-create-new-file | 4 | CONDITIONAL | Same. |
| figma-generate-design | 4 | CONDITIONAL | Same. |
| figma-generate-library | 4 | CONDITIONAL | Same. |
| figma-implement-design | 4 | CONDITIONAL | Same. |

### Backend / Dev / SEO

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| vercel-react-best-practices | 9 | KEEP | 70+ perf rules. |
| next-best-practices | 8 | KEEP | Next.js v15+ conventions. |
| react-dev | 7 | KEEP | React + TS patterns. |
| react-useeffect | 6 | KEEP | Effect best practices. |
| react-state-management | 7 | KEEP | Zustand, Redux, Jotai. |
| react-modernization | 7 | KEEP | Version upgrades. |
| react-components | 6 | KEEP | Stitch→React converter. |
| react-email | 6 | KEEP | Email templates. |
| react-native-best-practices | 7 | KEEP | Hermes, FlashList. |
| supabase-postgres | 7 | KEEP | Postgres optimization. |
| database-design | 7 | KEEP | DB selection. |
| database-schema-designer | 7 | KEEP | Schema design. |
| prisma-expert | 7 | KEEP | Prisma reference. |
| api-design-principles | 7 | KEEP | REST + GraphQL. |
| ai-sdk | 8 | KEEP | Vercel AI SDK. |
| ai-seo | 8 | KEEP | AI search optimization. |
| seo-audit | 7 | KEEP | Full SEO audit. |
| programmatic-seo | 7 | KEEP | Pages-at-scale. |
| audit-website | 7 | KEEP | 230+ rules. |
| vercel-optimize | 8 | KEEP | Production optimization. |
| typescript-expert | 7 | KEEP | TS best practices. |

### Content / Writing

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| copywriting | 7 | KEEP | Conversion copy. |
| marketing-psychology | 7 | KEEP | 70+ mental models. |
| marketing-ideas | 7 | KEEP | 139 SaaS approaches. |
| social-content | 7 | KEEP | Multi-platform social. |
| release-notes-one-pager | 6 | KEEP | Release notes template. |
| writing-guidelines | 6 | KEEP | Writing compliance. |

### Research / Strategy

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| research-decision-room | 8 | KEEP | Research synthesis. |
| ux-heuristics | 8 | KEEP | Nielsen/Krug framework. |
| ux-strategy | 7 | KEEP | Strategic UX planning. |
| prototyping-testing | 7 | KEEP | 527 lines. |
| creative-director | 6 | KEEP | Creative methodologies. |
| design-ops | 7 | KEEP | Critique frameworks. |
| design-research | 7 | KEEP | UX research methods. |

### Utility / Tool

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| vanta-design-orchestrator | 9 | KEEP | Master orchestrator. |
| full-output-enforcement | 8 | KEEP | Anti-truncation. |
| systematic-debugging | 8 | KEEP | Root-cause methodology. |
| writing-plans | 8 | KEEP | Implementation plans. |
| brainstorming | 8 | KEEP | Idea-to-spec dialogue. |
| find-skills | 8 | KEEP | Skill ecosystem CLI. |
| skill-creator | 8 | KEEP | Skill creation guide. |
| agent-browser | 8 | KEEP | Browser automation. |
| professional-github-repo | 7 | KEEP | GitHub orchestration. |
| export-download-debugging | 7 | KEEP | Debug exports. |
| visual-review | 8 | KEEP | Automated QA pipeline. |
| design-brief | 8 | KEEP | I-Lang brief parser. |
| design-md | 4 | KEEP | DESIGN.md management. |
| designer-toolkit | 7 | KEEP | Case studies, UX writing. |
| pptx-html-fidelity-audit | 8 | KEEP | PPTX quality control. |
| pr-feedback-quality-gate | 7 | KEEP | PR workflow. |
| docx | 4 | KEEP | Word docs. |
| pdf | 5 | KEEP | Anthropic PDF. |
| speech | 5 | KEEP | TTS via OpenAI. |
| hand-drawn-diagrams | 6 | KEEP | Excalidraw diagrams. |
| d3-visualization | 5 | KEEP | D3.js charts. |
| data-report | 5 | KEEP | CSV→visual report. |

---

## Skill Loading Guide

### For Design Tasks
```
1. vanta-design-orchestrator  (orchestrator - load FIRST)
2. brainstorming              (if requirements unclear)
3. impeccable                 (default design authority)
4. imagegen-frontend-web      (for image generation)
5. brandkit                   (for brand identity)
```

### For Animation Tasks
```
1. motion (motion.dev)  (preferred per AGENTS.md)
2. gsap-core            (if GSAP needed)
3. gsap-scrolltrigger   (for scroll animations)
4. emil-design-eng      (for polish)
```

### For Bug Fixes
```
1. systematic-debugging  (root cause first)
2. writing-plans         (plan before implementing)
```

### For Multi-step Features
```
1. writing-plans    (plan first)
2. brainstorming    (if creative/ambiguous)
3. relevant skills  (implement)
```

### For Video/Deck Work
```
1. hyperframes           (HTML-to-video)
2. after-hours-editorial (luxury deck)
3. field-notes           (business report)
```

### For SEO
```
1. ai-seo        (AI search optimization)
2. seo-audit     (technical audit)
3. audit-website (230+ rules)
```

---

## What Was Removed

The following 62 skills were removed during cleanup (duplicates, stubs, irrelevant):

**Duplicates (14):** brutalist-skill, design-taste-frontend-v1, gpt-tasteskill, image-to-code-skill, minimalist-skill, redesign-skill, stitch-skill, soft-skill, taste-skill-v1, threejs (shallow), pptx-generator, slides, nanobanana-ppt, output-skill

**Shallow Stubs (48):** all 8 fal stubs (fal-3d, kling-o3, lip-sync, realtime, restore, train, tryon, video-edit), all 5 venice stubs, pixelbin-media, replicate, sora, imagen, artifacts-builder, apple-hig, brand-guidelines, frontend-dev, frontend-skill, frontend-slides, design-consultation, design-review, enhance-prompt, image-enhancer, paywall-upgrade-cro, ui-skills, ad-creative, ai-music-album, competitive-ads-extractor, domain-name-brainstormer, flutter-animating-apps, gif-sticker-maker, hatch-pet, resume-modern, video-downloader, wpds, slack-gif-creator, youtube-clipper, swiftui-design, screenshot, full-page-screenshot, minimax-docx, minimax-pdf, doc (OpenAI), imagegen (bare)

**Empty dirs (9):** cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer

**Cross-location (1):** interaction-design (from .claude/skills/, redundant with impeccable)

---

## Source Locations

All skills are now consolidated in `.agents/skills/` (project-local, 154 skills).
The global `.agents/skills/` and `.claude/skills/` locations still exist but are secondary — prefer the project-local copy.
