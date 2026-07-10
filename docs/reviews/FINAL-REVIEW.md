---
title: "Final Review — VantaDB Skills Ecosystem"
type: review
status: active
tags: [vantadb, review, skills]
last_reviewed: 2026-07-10
language: en
---

# FINAL REVIEW — VantaDB Skills Ecosystem

**Date:** 2026-07-03  
**Sources analyzed:** 4 agent reports + raw data  
**Locations covered:**
1. `VantaDB\.agents\skills\` (project-local, A-Z) — agent-01 & agent-02
2. `C:\Users\Eros\.agents\skills\` (global user) — agent-03
3. `C:\Users\Eros\.claude\skills\` (Claude-specific) — agent-04

---

## 1. Executive Summary

The VantaDB skill ecosystem spans three physical directories with an estimated 260+ total skill directories, of which roughly 190+ contain actual SKILL.md files. Across all agents, approximately 60% of skills are recommended to Keep and 40% to Remove, though consensus varies by category.

**Design/UI skills** dominate the ecosystem (frontend design, animation, deck templates). The strongest skills are `impeccable` (rated 10/10 by agent-03/04), `design-taste-frontend` (9/10), `brandkit` (9/10), and the GSAP suite (7-9/10). The ecosystem has a solid core of ~50 high-value skills.

**Key problems:** Massive fragmentation across vendor API wrappers (14+ venice/fal/pixelbin/replicate skills that are shallow stubs), 10-15 duplicate/alias directories, 9 empty directories in `.claude/skills/`, and significant overlap between the three physical locations. Many skills share the same name across locations (e.g., `impeccable` exists in all three locations).

**The recommended cleanup** removes ~40% of total directories (shallow stubs, duplicates, empties, irrelevant skills), producing a lean "Core 50" set of unique, high-value skills for the VantaDB project. The three locations should be consolidated into a single canonical set with no cross-location duplication.

---

## 2. Total Inventory

| Location | Total Directories | With SKILL.md | Empty/Broken |
|----------|:-:|:-:|:-:|
| `VantaDB\.agents\skills\` (A-Z) | ~160 | ~160 | 0 |
| `C:\Users\Eros\.agents\skills\` (global) | 76 | 76 | 0 |
| `C:\Users\Eros\.claude\skills\` (claude) | 36 | 27 | 9 |
| **Total raw directories** | **~272** | **~263** | **9** |
| **Estimated unique skills (after dedup)** | **~190** | **~183** | **~7 empty** |

| Metric | Count |
|--------|:-----:|
| Total raw SKILL.md files analyzed across all agents | 263 |
| Estimated unique skills | ~190 |
| Skills recommended to KEEP (consensus) | ~110 |
| Skills recommended to REMOVE (merged) | ~80 |
| **Core 50 recommended (after full cleanup)** | **~50-55** |
| Empty/non-functional directories | 9 |

---

## 3. Skills by Category (Merged from All Agents)

### 3.1 Frontend / UI Design

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| impeccable | .agents, .claude | 10 | KEEP | Best-in-class. v3.7.1. Primary design authority. |
| design-taste-frontend | .agents, .claude? | 9 | KEEP | Anti-slop direction finder. 1200+ lines. |
| design-taste-frontend-v1 | .agents | 6 | REMOVE | Legacy. v2 is the active rewrite. |
| frontend-design | .agents | 8 | KEEP | OpenAI production playbook. |
| high-end-visual-design | .agents | 8 | KEEP | Agency-level design rules. Overlaps but unique. |
| awesome-claude-design | .agents (local) | 7 | KEEP | 9 aesthetic families reference. |
| industrial-brutalist-ui | .agents (both) | 7 | KEEP | Niche but excellent for data-heavy UI. |
| minimalist-ui | .agents (both) | 7 | KEEP | Keep — remove minimalist-skill (duplicate). |
| minimalist-skill | .agents (local) | 7 | REMOVE | Duplicate of minimalist-ui. |
| gpt-taste | .agents (both) | 7 | KEEP | GSAP/editorial. Remove gpt-tasteskill. |
| gpt-tasteskill | .agents (local) | 5 | REMOVE | Duplicate of gpt-taste. |
| interface-design | .agents | 7 | KEEP | App/dashboard focused. Clear scope. |
| redesign-existing-projects | .agents (both) | 8 | KEEP | Keep — remove redesign-skill. |
| redesign-skill | .agents (local) | 7 | REMOVE | Duplicate of redesign-existing-projects. |
| web-design-guidelines | .agents, .claude? | 7 | KEEP | Auto-updating compliance checker. |
| ui-ux-pro-max | .agents | 7 | KEEP | Design reference DB (50 styles, 21 palettes). |
| image-to-code | .agents (both) | 7 | KEEP | Keep one — remove image-to-code-skill. |
| image-to-code-skill | .agents (local) | 6 | REMOVE | Duplicate of image-to-code. |
| article-magazine | .agents (local) | 6 | KEEP | Magazine article HTML layout. |
| card-twitter | .agents (local) | 5 | KEEP | Twitter card template. |
| card-xiaohongshu | .agents (local) | 5 | KEEP | Chinese social card templates. |
| faq-page | .agents (local) | 6 | KEEP | FAQ accordion template. |
| impeccable-design-polish | .agents (local) | 7 | KEEP | Lightweight polish companion. |
| web-artifacts-builder | .agents (local) | 5 | KEEP | Official Anthropic reference. |
| artifacts-builder | .agents (local) | 3 | REMOVE | Stub. Replaceable by web-artifacts-builder. |
| frontend-dev | .agents (local) | 3 | REMOVE | Stub, superseded. |
| frontend-skill | .agents (local) | 3 | REMOVE | Stub, superseded. |
| mockup-device-3d | .agents (local) | 7 | KEEP | iPhone/MacBook 3D showcase. Unique. |
| doc-kami-parchment | .agents (local) | 6 | KEEP | Editorial one-pager template. |
| paywall-upgrade-cro | .agents (local) | 4 | REMOVE | Shallow B2B stub. Low relevance. |
| ui-design | .agents (local) | 8 | KEEP | 859 lines. UI theory reference. |
| ui-skills | .agents (local) | 5 | REMOVE | Shallow. Redundant. |
| taste-skill | .agents (local) | 9 | KEEP | design-taste-frontend v2 alias. |
| taste-skill-v1 | .agents (local) | 7 | REMOVE | Legacy fallback. Only if v2 breaks. |
| soft-skill | .agents (local) | 7 | REMOVE | Duplicate of high-end-visual-design. |
| brutalist-skill | .agents (local) | 6 | REMOVE | Duplicate of industrial-brutalist-ui. |
| swiftui-design | .agents (local) | 5 | REMOVE | iOS-only. Low relevance. |

### 3.2 Animation / Motion / 3D

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| gsap-core | .agents, .claude | 9 | KEEP | Official GSAP. Core API. |
| gsap-timeline | .agents, .claude | 8 | KEEP | Timeline sequencing. |
| gsap-scrolltrigger | .agents, .claude | 9 | KEEP | Scroll-linked. Pinning, scrub. |
| gsap-plugins | .agents, .claude | 8 | KEEP | SplitText, MorphSVG, Draggable, etc. |
| gsap-react | .agents, .claude | 8 | KEEP | useGSAP hook, React cleanup. |
| gsap-frameworks | .agents, .claude | 7 | KEEP | Vue/Svelte GSAP integration. |
| gsap-performance | .agents, .claude | 7 | KEEP | Optimize for 60fps. |
| gsap-utils | .agents, .claude | 7 | KEEP | Utility methods. Optional but useful. |
| motion | .agents, .claude | 9 | KEEP | motion.dev v12.29.2. Modern. |
| animejs | .agents, .claude | 8 | KEEP | Lightweight GSAP alternative. |
| design-motion-principles | .agents, .claude | 8 | KEEP | Create + Audit modes. Taste layer. |
| emil-design-eng | .agents (local) | 9 | KEEP | UI polish philosophy. 685 lines. |
| emilkowalski-motion | .agents (local) | 7 | KEEP | Micro-interaction follow-up. |
| interaction-design | .agents, .claude | 8 | KEEP | Comprehensive (1097 lines) — KEEP despite agent-04 suggesting remove. High unique value. |
| threejs-fundamentals | .agents, .claude | 8 | KEEP | Scene/camera/renderer. |
| threejs-geometry | .agents, .claude | 8 | KEEP | Geometry creation. |
| threejs-materials | .agents, .claude | 8 | KEEP | PBR, textures, materials. |
| threejs-animation | .agents, .claude | 8 | KEEP | Animation techniques. |
| threejs-interaction | .agents, .claude | 9 | KEEP | Raycasting, controls. |
| threejs-shaders | .agents, .claude | 9 | KEEP | GLSL, ShaderMaterial. |
| threejs (shallow) | .agents (local) | 5 | REMOVE | Shallow. Keep global threejs-* suite instead. |
| algorithmic-art | .agents (local) | 8 | KEEP | p5.js generative art. |
| shader-dev | .agents (local) | 5 | KEEP (conditional) | Keep if expanded. Important topic. |
| video-hyperframes | .agents (local) | 6 | KEEP | Hyperframes bridge template. |
| vfx-text-cursor | .agents (local) | 6 | KEEP | VFX text reveal. |
| remotion | .agents (local) | 5 | KEEP | Shallow but important tech. |
| flutter-animating-apps | .agents (local) | 2 | REMOVE | Flutter not used. |

### 3.3 Deck / Slide / Video Templates

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| hyperframes | .agents, .claude | 9 | KEEP | Core video composition. 490 lines. |
| hyperframes-animation | .agents, .claude | 8 | KEEP | 7 runtime adapters. |
| deck-open-slide-canvas | .agents (local) | 7 | KEEP | Freeform 1920x1080 React canvas. |
| deck-swiss-international | .agents (local) | 7 | KEEP | 16-column Swiss grid. |
| deck-guizang-editorial | .agents (local) | 7 | KEEP | Editorial e-ink deck. |
| field-notes-editorial-template | .agents (local) | 7 | KEEP | Magazine-style business report. |
| digits-fintech-swiss-template | .agents (local) | 7 | KEEP | Fintech data deck. |
| editorial-burgundy-principles-template | .agents (local) | 7 | KEEP | Burgundy/blush manifesto deck. |
| html-ppt-retro-quarterly-review | .agents (local) | 7 | KEEP | Retro quarterly deck. |
| after-hours-editorial-template | .agents (local) | 7 | KEEP | Luxury fashion HyperFrames deck. |
| swiss-creative-mode-template | .agents (local) | 7 | KEEP | Swiss editorial deck. |
| swiss-user-research-video-template | .agents (local) | 7 | KEEP | Research deck template. |
| weread-year-in-review-video-template | .agents (local) | 7 | KEEP | Vertical 9:16 report. |
| ppt-keynote | .agents (local) | 7 | KEEP | Apple Keynote-quality HTML deck. |
| pptx | .agents (local) | 5 | KEEP | Official Anthropic. Keep over pptx-generator. |
| pptx-generator | .agents (local) | 5 | REMOVE | Redundant with pptx. |
| pptx-html-fidelity-audit | .agents (local) | 8 | KEEP | Detailed audit workflow. High value. |
| slides | .agents (local) | 5 | REMOVE | Redundant with pptx suite. |
| nanobanana-ppt | .agents (local) | 4 | REMOVE | Shallow stub. |
| remotion-best-practices | .agents, .claude | 7 | KEEP | React native video creation. |
| frame-data-chart-nyt | .agents (local) | 6 | KEEP | NYT-style chart frame. |
| frame-flowchart-sticky | .agents (local) | 6 | KEEP | Sticky note flowchart. |
| frame-glitch-title | .agents (local) | 6 | KEEP | Glitch title frame. |
| frame-light-leak-cinema | .agents (local) | 6 | KEEP | Cinematic frame. |
| frame-liquid-bg-hero | .agents (local) | 6 | KEEP | Fluid background hero. |
| frame-logo-outro | .agents (local) | 6 | KEEP | Logo outro brand frame. |
| frame-macos-notification | .agents (local) | 6 | KEEP | macOS notification overlay. |
| frontend-slides | .agents (local) | 3 | REMOVE | Redundant with deck-* suite. |
| 8-bit-orbit-video-template | .agents (local) | ? | KEEP | Not analyzed by agents but exists. |

### 3.4 Image / Video / AI Media

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| brandkit | .agents (both) | 9 | KEEP | World-class brand identity gen. |
| imagegen-frontend-web | .agents (both) | 9 | KEEP | Premium web section images. |
| imagegen-frontend-mobile | .agents (both) | 8 | KEEP | Mobile app screen images. |
| imagegen | .agents (both) | 3 | REMOVE | Stub. Redundant with frontend-* variants. |
| canvas-design | .agents (both) | 7 | KEEP | Artistic visual output. |
| ecommerce-image-workflow | .agents (local) | 7 | KEEP | Product image pipeline. |
| poster-hero | .agents (local) | 6 | KEEP | Vertical poster template. |
| screenshots-marketing | .agents (local) | 6 | KEEP | Playwright marketing screenshots. |
| social-reddit-card | .agents (local) | 6 | KEEP | Reddit card template. |
| social-spotify-card | .agents (local) | 6 | KEEP | Spotify card template. |
| social-x-post-card | .agents (local) | 6 | KEEP | X/Twitter card template. |
| image-edit | .agents, .claude | 7 | KEEP | RunComfy smart router. |
| gif-sticker-maker | .agents (local) | 2 | REMOVE | Niche. Not relevant. |
| slack-gif-creator | .agents (local) | 4 | REMOVE | Niche. Low project relevance. |
| image-enhancer | .agents (local) | 2 | REMOVE | Stub. Covered by recraft/visual-review. |
| imagen | .agents (local) | 2 | REMOVE | Stub. Redundant. |
| pixelbin-media | .agents (local) | 3 | REMOVE | Thin vendor stub. |
| sora | .agents (local) | 5 | REMOVE | Shallow. Covered by fal/venice. |
| youtube-clipper | .agents (local) | 5 | REMOVE | Shallow. Low relevance. |
| hatch-pet | .agents (local) | 7 | REMOVE | Codex-specific. Not relevant. |
| ai-music-album | .agents (local) | 2 | REMOVE | Not relevant. |

### 3.5 fal.ai Vendor Suite (14 skills)

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| fal-generate | .agents (local) | 3 | CONDITIONAL | Keep ONLY if fal.ai API actively used. |
| fal-image-edit | .agents (local) | 3 | CONDITIONAL | Same. |
| fal-upscale | .agents (local) | 3 | CONDITIONAL | Same. |
| fal-vision | .agents (local) | 3 | CONDITIONAL | Same. |
| fal-3d | .agents (local) | 3 | REMOVE | Stub. |
| fal-kling-o3 | .agents (local) | 3 | REMOVE | Stub. |
| fal-lip-sync | .agents (local) | 3 | REMOVE | Stub. |
| fal-realtime | .agents (local) | 3 | REMOVE | Stub. |
| fal-restore | .agents (local) | 3 | REMOVE | Stub. |
| fal-train | .agents (local) | 3 | REMOVE | Stub. |
| fal-tryon | .agents (local) | 3 | REMOVE | Stub. |
| fal-video-edit | .agents (local) | 3 | REMOVE | Stub. |
| **Core 4 to possibly keep** | | **3** | **CONDITIONAL** | generate, image-edit, upscale, vision. Remove rest. |

### 3.6 venice.ai Vendor Suite (5 skills)

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| venice-audio-music | .agents (local) | 4 | REMOVE | Shallow vendor stub. |
| venice-audio-speech | .agents (local) | 4 | REMOVE | Redundant with speech skill. |
| venice-image-edit | .agents (local) | 4 | REMOVE | Shallow vendor stub. |
| venice-image-generate | .agents (local) | 4 | REMOVE | Shallow vendor stub. |
| venice-video | .agents (local) | 4 | REMOVE | Shallow vendor stub. |

### 3.7 Design Systems / Tokens

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| design-systems | .agents (local) | 8 | KEEP | Complete design system management. |
| color-expert | .agents (local) | 8 | KEEP | 286K words color science. |
| theme-factory | .agents (local) | 8 | KEEP | 10 preset themes. High practical value. |
| platform-design | .agents (local) | 8 | KEEP | 300+ HIG/Material/WCAG rules. |
| stitch-design-taste | .agents (both) | 8 | KEEP | Stitch DESIGN.md generator. |
| stitch-loop | .agents (local) | 5 | KEEP | Design-to-code iteration. Stub. |
| stitch-skill | .agents (local) | 7 | REMOVE | Duplicate of stitch-design-taste. |
| reference-design-contract | .agents (local) | 8 | KEEP | Reference-to-spec workflow. |
| visual-critique | .agents (local) | 7 | KEEP | Systematic design critique. |
| plan-design-review | .agents (local) | 7 | KEEP | Quality gate. |
| shadcn-ui | .agents (local) | 6 | KEEP | shadcn/ui component building reference. |
| figma-use | .agents (local) | 4 | CONDITIONAL | Keep if Figma MCP active. |
| figma-code-connect-components | .agents (local) | 4 | CONDITIONAL | Same. |
| figma-create-design-system-rules | .agents (local) | 4 | CONDITIONAL | Same. |
| figma-create-new-file | .agents (local) | 4 | CONDITIONAL | Same. |
| figma-generate-design | .agents (local) | 4 | CONDITIONAL | Same. |
| figma-generate-library | .agents (local) | 4 | CONDITIONAL | Same. |
| figma-implement-design | .agents (local) | 4 | CONDITIONAL | Same. |
| design-consultation | .agents (local) | 3 | REMOVE | Thin stub. |
| apple-hig | .agents (local) | 2 | REMOVE | Stub only. No actual HIG content. |
| brand-guidelines | .agents (local) | 2 | REMOVE | Anthropic-specific stub. |
| wpds | .agents (local) | 3 | REMOVE | WordPress-only. Zero relevance. |

### 3.8 Backend / Dev / SEO

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| vercel-react-best-practices (react-best-practices) | .agents (both) | 9 | KEEP | 70+ React/Next.js perf rules. |
| next-best-practices | .agents | 8 | KEEP | Next.js v15+ conventions. |
| react-dev | .agents | 7 | KEEP | React + TypeScript patterns. |
| react-useeffect | .agents | 6 | KEEP | When NOT to useEffect. |
| react-state-management | .agents | 7 | KEEP | Zustand, Redux Toolkit, Jotai. |
| react-modernization | .agents | 7 | KEEP | Version upgrades, class→hooks. |
| react-components | .agents | 6 | KEEP | Stitch→React converter. |
| react-email | .agents | 6 | KEEP | HTML email templates. |
| react-native-best-practices | .agents | 7 | KEEP | Hermes, FlashList, profiling. |
| supabase-postgres-best-practices | .agents | 7 | KEEP | Postgres optimization. |
| database-design | .agents | 7 | KEEP | DB selection, schema strategy. |
| database-schema-designer | .agents | 7 | KEEP | Granular schema design. |
| prisma | .agents | 6 | REMOVE | Redundant with prisma-expert. |
| prisma-expert | .agents | 7 | KEEP | Comprehensive Prisma reference. |
| api-design-principles | .agents | 7 | KEEP | REST + GraphQL. 528 lines. |
| ai-sdk | .agents | 8 | KEEP | Vercel AI SDK. Auto-updates. |
| ai-seo | .agents, .claude | 8 | KEEP | AI search optimization. |
| seo | .agents, .claude | 7 | KEEP | General SEO reference. |
| seo-audit | .agents | 7 | KEEP | Full SEO audit. |
| programmatic-seo | .agents | 7 | KEEP | Pages-at-scale. |
| roier-seo | .agents, .claude | 7 | KEEP | Auto-fixes SEO. |
| audit-website | .agents | 7 | KEEP | 230+ rules via squirrelscan. |
| vercel-optimize | .agents (local) | 8 | KEEP | Production optimization. Detailed. |

### 3.9 Content / Writing

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| copywriting | .agents | 7 | KEEP | Conversion copy. Structured. |
| marketing-psychology | .agents | 7 | KEEP | 70+ mental models. |
| marketing-ideas | .agents | 7 | KEEP | 139 SaaS approaches. |
| social-content | .agents | 7 | KEEP | Multi-platform social media. |
| release-notes-one-pager | .agents (local) | 6 | KEEP | Release notes template. |
| writing-guidelines | .agents (local) | 6 | KEEP | Auto-updating writing checker. |
| ad-creative | .agents (local) | 3 | REMOVE | Not relevant. Thin stub. |
| competitive-ads-extractor | .agents (local) | 2 | REMOVE | Not relevant. |

### 3.10 Research / Strategy

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| research-decision-room | .agents (local) | 8 | KEEP | Research synthesis dashboard. |
| ux-heuristics | .agents (local) | 8 | KEEP | Nielsen/Krug framework. |
| ux-strategy | .agents (local) | 7 | KEEP | Strategic UX planning. 597 lines. |
| prototyping-testing | .agents (local) | 7 | KEEP | 527 lines. Comprehensive. |
| creative-director | .agents (local) | 6 | KEEP | Creative methodologies. |
| design-ops | .agents (local) | 7 | KEEP | Critique frameworks, sprint planning. |
| design-research | .agents (local) | 7 | KEEP | UX research methodologies. |

### 3.11 Utility / Tool

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| vanta-design-orchestrator | .agents (local) | 9 | KEEP | Master orchestrator. 1176+ lines. ESSENTIAL. |
| full-output-enforcement | .agents (both) | 8 | KEEP | Anti-truncation. Essential. |
| output-skill | .agents (local) | 8 | KEEP | Alias of full-output-enforcement (same dir?). Verify. |
| systematic-debugging | .agents | 8 | KEEP | Root-cause-first methodology. |
| writing-plans | .agents | 8 | KEEP | Bite-sized implementation plans. |
| brainstorming | .agents | 8 | KEEP | Idea-to-spec dialogue. |
| find-skills | .agents | 8 | KEEP | Skill ecosystem CLI. Essential. |
| skill-creator | .agents | 8 | KEEP | Skill creation guide. Essential. |
| agent-browser | .agents | 8 | KEEP | Browser automation CLI. |
| browser-use | .agents | 7 | CONSIDER REMOVE | Redundant with agent-browser + Playwright MCP. |
| professional-github-repo | .agents | 7 | KEEP | GitHub repo orchestration. |
| export-download-debugging | .agents (local) | 7 | KEEP | Debug export/download failures. |
| visual-review | .agents (local) | 8 | KEEP | Automated QA pipeline. Includes scripts. |
| design-brief | .agents (local) | 8 | KEEP | I-Lang brief parser. 252 lines. |
| design-md | .agents (local) | 4 | KEEP | DESIGN.md file management. |
| designer-toolkit | .agents (local) | 7 | KEEP | Case studies, UX writing. |
| design-review | .agents (local) | 3 | REMOVE | Stub. Replaceable by impeccable. |
| pptx-html-fidelity-audit | .agents (local) | 8 | KEEP | PPTX quality control. |
| pr-feedback-quality-gate | .agents (local) | 7 | KEEP | PR workflow agent skill. |
| docx | .agents (local) | 4 | KEEP | Word docs with tracked changes. |
| doc | .agents (local) | 3 | REMOVE | Stub. Redundant with docx. |
| pdf | .agents (local) | 5 | KEEP | Official Anthropic. Keep over minimax-pdf. |
| minimax-docx | .agents (local) | 4 | REMOVE | Shallow stub. Redundant with docx. |
| minimax-pdf | .agents (local) | 4 | REMOVE | Shallow stub. Keep pdf instead. |
| speech | .agents (local) | 5 | KEEP | Basic TTS via OpenAI. |
| hand-drawn-diagrams | .agents (local) | 6 | KEEP | Excalidraw diagram generation. |
| d3-visualization | .agents (local) | 5 | KEEP | D3.js charts (stub, but valuable). |
| data-report | .agents (local) | 5 | KEEP | CSV→visual report template. |
| domain-name-brainstormer | .agents (local) | 2 | REMOVE | Not relevant. |
| resume-modern | .agents (local) | 5 | REMOVE | Personal use only. |
| screenshot | .agents (local) | 5 | REMOVE | Redundant with Playwright MCP. |
| full-page-screenshot | .agents (local) | 3 | REMOVE | Redundant with Playwright MCP. |
| video-downloader | .agents (local) | 4 | REMOVE | Low relevance. Ethical concerns. |
| enhance-prompt | .agents (local) | 3 | REMOVE | Thin stub. |
| replicate | .agents (local) | 4 | REMOVE | Thin API wrapper. Redundant with fal. |
| sleek-design-mobile-apps | .agents, .claude | 6 | CONDITIONAL | Requires paid Sleek Pro. |

### 3.12 Mobile

| Skill | Location(s) | Rating | Verdict | Notes |
|-------|-------------|:------:|:-------:|-------|
| login-flow | .agents (local) | 6 | KEEP | Mobile auth workflow. |
| react-native-best-practices | .agents | 7 | KEEP | (listed in Backend/Dev) |
| flutter-animating-apps | .agents (local) | 2 | REMOVE | (listed in Animation) |

### 3.13 Empty / Non-Functional (`.claude/skills/`)

| Skill | Location | Rating | Verdict | Notes |
|-------|----------|:------:|:-------:|-------|
| cargo-nextest | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| github-repo-management | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| m10-performance | .claude/skills | 0 | REMOVE | Empty dir — unknown purpose. |
| markdown-documentation | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| python-packaging | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| rust-ffi | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| rust-write-tests | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| test-reporting | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |
| vector-database-engineer | .claude/skills | 0 | REMOVE | Empty dir — install or remove. |

---

## 4. Top 20 Most Valuable Skills (Ranked)

These are the highest-rated, highest-impact skills for the VantaDB project specifically:

| # | Skill | Rating | Why Valuable for VantaDB |
|:-:|-------|:------:|--------------------------|
| 1 | **impeccable** | 10/10 | Primary design authority. v3.7.1. Covers every UI dimension. Sub-commands, scripts, auto-updating. Use by default for all frontend work. |
| 2 | **vanta-design-orchestrator** | 9/10 | Master orchestrator. 1176+ lines. Integrates every other design skill into a unified workflow. ESSENTIAL infrastructure. |
| 3 | **design-taste-frontend** | 9/10 | Anti-slop direction finder. 1200+ lines of rules. Brief inference engine. Use before implementation to establish design direction. |
| 4 | **brandkit** | 9/10 | World-class brand identity generation (798 lines). Essential for any branding work, pitch decks, or visual identity tasks. |
| 5 | **emil-design-eng** | 9/10 | Emil Kowalski's UI polish philosophy (685 lines). The "invisible details" that make software feel premium. |
| 6 | **gsap-core** | 9/10 | Official GSAP core. MIT license. Foundation for all animation work. Pairs with design skills. |
| 7 | **gsap-scrolltrigger** | 9/10 | Scroll-linked animation, pinning, scrub. Extremely detailed (296 lines). Critical for modern web experiences. |
| 8 | **motion (motion.dev)** | 9/10 | Modern animation library (v12.29.2). 120fps, gestures. Preferred by AGENTS.md over CSS. Up-to-date. |
| 9 | **hyperframes** | 9/10 | HTML-to-video composition (490 lines). TTS, captions, transitions, audio-reactive. Unique video pipeline. |
| 10 | **imagegen-frontend-web** | 9/10 | Premium web section images. Hard output rules, one per section. Critical for design-to-code workflow. |
| 11 | **image-to-code** | 9/10 | Image-first design-to-code. Generate premium design images, then implement from them. Unique workflow. |
| 12 | **react-best-practices (vercel-react-best-practices)** | 9/10 | 70+ production rules from Vercel Engineering. Essential for the React/Next.js stack. |
| 13 | **color-expert** | 8/10 | 286K words of color science. OKLCH/OKLAB, palette generation, accessibility. Critical for design quality. |
| 14 | **design-systems** | 8/10 | Complete design system management — tokens, components, accessibility, theming, motion, governance. |
| 15 | **high-end-visual-design** | 8/10 | Agency-level design rules. "Absolute Zero" anti-pattern list. Makes sites look expensive. |
| 16 | **platform-design** | 8/10 | 300+ design rules from Apple HIG, Material Design 3, WCAG 2.2. Cross-platform authority. |
| 17 | **theme-factory** | 8/10 | 10 preset themes + custom generator. Practical, reusable. Great for rapid theming. |
| 18 | **interaction-design** | 8/10 | 1097 lines of interaction design frameworks. Micro-animations, state machines, cognitive laws. |
| 19 | **ai-sdk** | 8/10 | Vercel AI SDK. Auto-updates with current model IDs. Essential for AI-powered features. |
| 20 | **systematic-debugging** | 8/10 | Root-cause-first debugging methodology. Load before ANY bug fix. Practical and essential. |

---

## 5. Redundancies & Duplicates (Complete List)

| Group | Duplicate Skills | Keep | Remove | Notes |
|-------|-----------------|:----:|:------:|-------|
| **Taste/Design Frontend** | design-taste-frontend, design-taste-frontend-v1, taste-skill, taste-skill-v1 | design-taste-frontend (v2) | design-taste-frontend-v1, taste-skill-v1 | taste-skill is an alias of design-taste-frontend — verify if separate dir. |
| **Minimalist UI** | minimalist-ui, minimalist-skill | minimalist-ui | minimalist-skill | Nearly identical. |
| **Redesign** | redesign-existing-projects, redesign-skill | redesign-existing-projects | redesign-skill | Same methodology. |
| **GPT Taste** | gpt-taste, gpt-tasteskill | gpt-taste | gpt-tasteskill | Duplicates. |
| **Image-to-Code** | image-to-code, image-to-code-skill | image-to-code | image-to-code-skill | Same skill, two copies. |
| **Stitch** | stitch-design-taste, stitch-skill | stitch-design-taste | stitch-skill | Same origin. |
| **Brutalist** | industrial-brutalist-ui, brutalist-skill | industrial-brutalist-ui | brutalist-skill | Same content. |
| **Soft/High-End** | high-end-visual-design, soft-skill | high-end-visual-design | soft-skill | soft-skill is a copy. |
| **PPTX/Slides** | pptx, pptx-generator, slides, nanobanana-ppt | pptx | pptx-generator, slides, nanobanana-ppt | Keep official Anthropic pptx. |
| **Docx** | docx, doc | docx | doc | doc (OpenAI) is a stub. |
| **PDF** | pdf, minimax-pdf | pdf | minimax-pdf | Keep Anthropic pdf. |
| **GSAP Granularity** | gsap-performance, gsap-timeline could merge into gsap-core | All KEEP | — | Keep separate; merging optional. |
| **SEO Suite** | seo, seo-audit, roier-seo, ai-seo, audit-website | All KEEP | — | Different focuses — keep all. |
| **Impeccable + interaction-design** | impeccable covers interaction-design territory | Both KEEP | — | interaction-design (1097 lines) has enough unique value. |
| **Prisma** | prisma, prisma-expert | prisma-expert | prisma | prisma is 71 lines vs prisma-expert 355 lines. |
| **Image Generation** | imagegen, imagegen-frontend-web, imagegen-frontend-mobile, imagen | imagegen-frontend-web, imagegen-frontend-mobile | imagegen, imagen | Keep the premium direction skills. |
| **Impeccable cross-location** | impeccable in `.agents/skills/` AND `.claude/skills/` | Keep one | Remove duplicate location | Same skill in two locations. Keep in `.agents/skills/`. |
| **Output/Full Output** | full-output-enforcement, output-skill | full-output-enforcement | output-skill (if duplicate dir) | Verify if same skill or alias. |
| **Figma Suite** | figma-* (7 skills) similar across locations | Keep suite | — | All conditional on Figma MCP. |
| **Three.js** | threejs (local, shallow) vs threejs-* (global, comprehensive) | threejs-* global suite | threejs (local) | Local threejs is shallow. Global suite is comprehensive. |
| **Hyperframes cross-location** | hyperframes in `.agents/skills/` AND `.claude/skills/` | Keep one | — | Same skill in two places. Deduplicate. |

---

## 6. Shallow/Vendor Stubs to Remove

These skills are thin wrappers (≤50 lines) around third-party APIs with no actionable content:

### fal.ai Suite (keep only 4 if API active)
- fal-3d, fal-kling-o3, fal-lip-sync, fal-realtime, fal-restore, fal-train, fal-tryon, fal-video-edit — **REMOVE**
- fal-generate, fal-image-edit, fal-upscale, fal-vision — **KEEP CONDITIONAL** (only if fal.ai API actively used)

### venice.ai Suite (remove all 5)
- venice-audio-music, venice-audio-speech, venice-image-edit, venice-image-generate, venice-video — **REMOVE**

### Other Vendor Wrappers
- pixelbin-media — **REMOVE** (thin vendor stub, not in project deps)
- replicate — **REMOVE** (thin API wrapper, redundant with fal)
- sora — **REMOVE** (shallow, covered by fal/venice)
- slick-design-mobile-apps — **REMOVE** (requires paid Sleek Pro)
- imagen — **REMOVE** (5th image gen skill, no unique value)

### Stubs with No Depth
- artifacts-builder — **REMOVE** (stub, replaceable by web-artifacts-builder)
- apple-hig — **REMOVE** (stub only, no actual HIG content)
- brand-guidelines — **REMOVE** (Anthropic-specific stub)
- frontend-dev — **REMOVE** (stub, superseded)
- frontend-skill — **REMOVE** (stub, superseded)
- frontend-slides — **REMOVE** (redundant with deck-* suite)
- design-consultation — **REMOVE** (thin stub)
- design-review — **REMOVE** (stub, replaceable by impeccable)
- doc (OpenAI) — **REMOVE** (stub, redundant with docx)
- enhance-prompt — **REMOVE** (thin stub)
- image-enhancer — **REMOVE** (stub, covered by recraft/visual-review)
- paywall-upgrade-cro — **REMOVE** (shallow B2B stub)
- ui-skills — **REMOVE** (shallow, redundant)

### Irrelevant to VantaDB
- ad-creative — **REMOVE** (not relevant to database project)
- ai-music-album — **REMOVE** (not relevant)
- competitive-ads-extractor — **REMOVE** (not relevant)
- domain-name-brainstormer — **REMOVE** (not relevant)
- flutter-animating-apps — **REMOVE** (Flutter not used)
- gif-sticker-maker — **REMOVE** (niche toy, not relevant)
- hatch-pet — **REMOVE** (Codex-specificed, not relevant)
- resume-modern — **REMOVE** (personal use only)
- video-downloader — **REMOVE** (low relevance, ethical concerns)
- wpds — **REMOVE** (WordPress-only)
- slack-gif-creator — **REMOVE** (niche)
- youtube-clipper — **REMOVE** (shallow, low relevance)
- swiftui-design — **REMOVE** (iOS-only)

---

## 7. Empty/Non-functional Directories

### 9 empty directories in `C:\Users\Eros\.claude\skills\`

| Directory | Inferred Purpose | Status |
|-----------|-----------------|:------:|
| `cargo-nextest/` | Rust test runner (nextest) | Empty — no SKILL.md |
| `github-repo-management/` | GitHub repository management | Empty — no SKILL.md |
| `m10-performance/` | Unknown — possibly M10 framework | Empty — no SKILL.md |
| `markdown-documentation/` | Markdown documentation guidance | Empty — no SKILL.md |
| `python-packaging/` | Python packaging (pip/setuptools) | Empty — no SKILL.md |
| `rust-ffi/` | Rust FFI (Foreign Function Interface) | Empty — no SKILL.md |
| `rust-write-tests/` | Rust test writing | Empty — no SKILL.md |
| `test-reporting/` | Test reporting tools | Empty — no SKILL.md |
| `vector-database-engineer/` | Vector DB (Pinecone, Qdrant, etc.) | Empty — no SKILL.md |

**Recommendation:** Either install proper SKILL.md content for these or delete the empty directories. The `vector-database-engineer` directory is ironically most relevant to VantaDB (a vector database project) but has no content.

---

## 8. Recommendations by Priority

### P0 — Remove Immediately (Duplicates, Broken, Dangerous)

These are clear-cut: duplicates that cause confusion, broken directories, or skills that misdirect the agent.

| Skill | Reason |
|-------|--------|
| design-taste-frontend-v1 | Legacy. v2 is the active, stable version. |
| minimalist-skill | Duplicate of minimalist-ui (same content). |
| redesign-skill | Duplicate of redesign-existing-projects. |
| gpt-tasteskill | Duplicate of gpt-taste. |
| image-to-code-skill | Duplicate of image-to-code. |
| stitch-skill | Duplicate of stitch-design-taste. |
| brutalist-skill | Duplicate of industrial-brutalist-ui. |
| soft-skill | Duplicate of high-end-visual-design. |
| pptx-generator | Redundant with pptx (official Anthropic). |
| slides | Redundant with pptx suite. |
| nanobanana-ppt | Redundant with pptx suite. |
| taste-skill-v1 | Legacy fallback. Remove unless v2 breaks. |
| threejs (local shallow) | Redundant with global threejs-* comprehensive suite. |
| output-skill | Verify — may be duplicate of full-output-enforcement. |
| prisma (basic) | Redundant with prisma-expert (which is deeper). |
| All 9 empty dirs in .claude/skills/ | No content. Install or delete. |

### P1 — Clean Up (Shallow Stubs, Vendor Wrappers with Low Value)

| Skill | Reason |
|-------|--------|
| fal-3d, fal-kling-o3, fal-lip-sync, fal-realtime, fal-restore, fal-train, fal-tryon, fal-video-edit | 8 of 14 fal.ai skills are stubs. Remove. |
| venice-audio-music, venice-audio-speech, venice-image-edit, venice-image-generate, venice-video | All 5 venice skills are shallow stubs. Remove. |
| pixelbin-media | Thin vendor stub. Remove. |
| replicate | Thin API wrapper. Remove. |
| sora | Shallow. Covered by fal. Remove. |
| artifacts-builder | Stub. Use web-artifacts-builder instead. |
| apple-hig | No actual HIG content. |
| brand-guidelines | Anthropic-specific stub. |
| frontend-dev | Stub, superseded by better skills. |
| frontend-skill | Stub, superseded by better skills. |
| frontend-slides | Redundant with deck-* suite. |
| design-consultation | Thin stub. |
| design-review | Stub. Use impeccable instead. |
| doc (OpenAI) | Stub. Use docx instead. |
| enhance-prompt | Thin stub. |
| image-enhancer | Covered by recraft/visual-review. |
| imagen | 5th image gen skill. No unique value. |
| imagegen (bare) | Stub. Keep frontend-web and frontend-mobile. |
| paywall-upgrade-cro | Shallow B2B stub. |
| ui-skills | Shallow. Redundant. |
| ad-creative | Not relevant. |
| ai-music-album | Not relevant. |
| competitive-ads-extractor | Not relevant. |
| domain-name-brainstormer | Not relevant. |
| flutter-animating-apps | Flutter not used. |
| gif-sticker-maker | Niche. |
| hatch-pet | Codex-specific. |
| resume-modern | Personal use only. |
| video-downloader | Low relevance. |
| wpds | WordPress-only. |
| slack-gif-creator | Niche. |
| youtube-clipper | Shallow. |
| swiftui-design | iOS-only. |
| screenshot | Redundant with Playwright MCP. |
| full-page-screenshot | Redundant with Playwright MCP. |
| minimax-docx | Shallow stub. Use docx. |
| minimax-pdf | Shallow stub. Use pdf. |

### P2 — Consider Keeping (Borderline Useful)

| Skill | Reason |
|-------|--------|
| web-artifacts-builder | Official Anthropic but shallow. Keep as reference. |
| docx | Low depth (4/10). Keep for tracked changes capability. |
| pdf | Low depth (5/10). Keep as official Anthropic. |
| speech | Low depth (5/10). Keep for basic TTS. |
| d3-visualization | Low depth (5/10). Keep for chart reference. |
| data-report | Low depth (5/10). Keep for template. |
| stitch-loop | Stub, but part of Stitch ecosystem. Keep if Stitch active. |
| react-components | Only useful with Google Stitch. |
| react-email | Niche use case. Keep for email templates. |
| design-md | 4/10. Useful for documenting design decisions. |
| remotion | Shallow (5/10) but important React video tech. |
| shader-dev | 5/10. Important topic but shallow. Keep if expanded. |
| browser-use | Redundant with agent-browser + Playwright MCP. Remove if not using. |
| gpt-taste | 6/10. Overlapping. Consider removing if impeccable + design-taste-frontend cover enough. |
| login-flow | Mobile auth. Niche but well-scoped. |
| hand-drawn-diagrams | 6/10. Excalidraw is useful but skill is a stub. |
| card-twitter, card-xiaohongshu | Niche templates. Low depth. |

### P3 — Definitely Keep (Core 50+)

See section 4 (Top 20) and the comprehensive "Core 50" list below. These skills form the essential toolset for the VantaDB project.

---

## 9. Final Summary

### Recommended Count Summary

| Category | Raw Count | Keep | Remove | Conditional |
|----------|:---------:|:----:|:------:|:-----------:|
| Frontend/UI Design | ~50 | 25 | 25 | 0 |
| Animation/Motion/3D | ~26 | 24 | 2 | 0 |
| Deck/Slide/Video Templates | ~30 | 23 | 4 | 3 |
| Image/Video/AI Media | ~25 | 12 | 10 | 3 |
| fal.ai Suite | 14 | 0 | 10 | 4 |
| venice.ai Suite | 5 | 0 | 5 | 0 |
| Design Systems/Tokens | ~18 | 10 | 5 | 3 |
| Backend/Dev/SEO | ~18 | 17 | 1 | 0 |
| Content/Writing | ~7 | 5 | 2 | 0 |
| Research/Strategy | ~7 | 7 | 0 | 0 |
| Utility/Tool | ~25 | 16 | 7 | 2 |
| Mobile | ~3 | 1 | 1 | 1 |
| Empty/Broken | 9 | 0 | 9 | 0 |
| **TOTAL** | **~272 raw / ~190 unique** | **~140** | **~81** | **~16** |

### Core 50 — Recommended Essential Skills

These 50 skills form the complete, lean toolset for VantaDB:

**Frontend/UI (10):**
impeccable, design-taste-frontend, frontend-design, high-end-visual-design, awesome-claude-design, industrial-brutalist-ui, minimalist-ui, interface-design, redesign-existing-projects, ui-ux-pro-max, web-design-guidelines, image-to-code

**Animation/Motion (10):**
gsap-core, gsap-scrolltrigger, gsap-timeline, gsap-plugins, gsap-react, motion, animejs, design-motion-principles, emil-design-eng, interaction-design

**Three.js (6):**
threejs-fundamentals, threejs-geometry, threejs-materials, threejs-animation, threejs-interaction, threejs-shaders

**Video/Deck (8):**
hyperframes, hyperframes-animation, deck-open-slide-canvas, deck-swiss-international, field-notes-editorial-template, after-hours-editorial-template, remotion-best-practices, pptx

**Image/Brand (4):**
brandkit, imagegen-frontend-web, imagegen-frontend-mobile, canvas-design

**Design Systems (4):**
design-systems, color-expert, theme-factory, platform-design

**Backend/Dev/SEO (8):**
react-best-practices, next-best-practices, ai-sdk, database-schema-designer, prisma-expert, api-design-principles, ai-seo, roier-seo

**Utility/Infrastructure (6):**
vanta-design-orchestrator, systematic-debugging, writing-plans, brainstorming, find-skills, skill-creator

**Research/Strategy (3):**
ux-heuristics, research-decision-room, ux-strategy

**Other (1):**
agent-browser

**Total Core 50: ~50-55 skills** (with some optional add-ons like GPT suite, figma suite, fal suite based on active API usage).

### Final Verdict

The VantaDB skill ecosystem is rich but bloated. Roughly **40% of skills** should be removed:
- **~80 duplicates and stubs** to delete
- **~16 conditional** (keep only if API/services active)
- **~9 empty directories** to either install or delete
- **~140 skills to keep** (raw), consolidating to **~50-55 unique core skills**

The three physical locations should be consolidated into a single canonical directory with no cross-location duplication. The `vanta-design-orchestrator` skill should be maintained as the master entry point.
