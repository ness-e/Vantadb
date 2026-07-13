# AI Agent Skills: Internet Research Findings

## Research Date: 2026-07-03
## Project: VantaDB — Embedded Vector Database for AI Agents
## Site: vantadb.dev

---

## Group 1 — Design Skills

### 1. impeccable (pbakaus/impeccable)

**Source:** Paul Bakaus (ex-Google, creator of jQuery UI and Diavlo)  
**License:** Apache 2.0  
**GitHub Stars:** ~28.6K ★ (v3.1.1, May 2026)  
**Installs (skills.sh):** Not directly tracked (standalone repo)

**What it does:** Design language skill system — 23 slash commands (/polish, /audit, /typeset, /craft, etc.) and 27 deterministic anti-pattern rules that inject professional design vocabulary into AI coding agents. Designed to combat "AI slop" in frontend generation.

**Community reception:** Extremely positive. Went #1 trending on GitHub within days of launch. Widely considered the gold standard design skill for Claude Code. Multiple blog reviews rate it 8.4-9.0/10.

**Key quotes from reviews:**
- "Impeccable is the first tool that directly attacks this problem — a structured design language that plugs into your AI agent and rewires how it thinks about visual design." (computertech.co)
- "The skill pointed at five specific things — three of them were decisions I had personally made, defended in design reviews, and shipped with confidence." (mejba.me)
- "If you use Claude Code for any frontend work, Impeccable improves your output quality at zero cost with zero workflow changes." (emelia.io)

**Verified claims:** Yes. The anti-pattern rules are deterministic and auditable. 23 commands are real and documented at impeccable.style. Works with Claude Code, Cursor, Codex CLI, Gemini CLI, VS Code Copilot, Kiro, OpenCode, Pi.

**Criticisms:** Some token overhead from loading 7 domain-specific reference files. Best results reported with Claude Code specifically. Some users report it can be overly prescriptive for creative/experimental design.

**Trust Score:** 9/10

---

### 2. design-taste-frontend (leonxlnx/taste-skill)

**Source:** Leonxlnx community project  
**License:** MIT  
**GitHub Stars:** ~55.5K ★ (taste-skill repo overall)  
**Installs (skills.sh):** ~87.8K for imagegen-frontend-web (same repo)

**What it does:** Anti-slop frontend skill with Three Dials system (DESIGN_VARIANCE, MOTION_INTENSITY, VISUAL_DENSITY), brief-to-design-system mapping, strict forbidden patterns (em-dashes, hero version labels, scroll cues, AI-purple gradients), and redesign-audit protocol.

**Community reception:** Very strong. Considered the go-to "anti-slop" skill for frontend. V2 (2026) is a substantial rewrite with contextual brief inference.

**Key quotes from reviews:**
- "Reads the brief, infers the right design direction, and ships interfaces that do not look templated." (tasteskill.dev)
- "Section 9 ships a strict ban list. The agent enforces these on every generation." (documentation)

**Verified claims:** Yes. The Three Dials system, brief-to-DS mapping, and ban list are fully documented. Active development with v2 experimental.

**Criticisms:** Can be opinionated to the point of rigidity. Some users find the ban list too strict for non-landing-page work. Not suitable for dashboards, data tables, or multi-step product UI (skill explicitly disclaims these).

**Trust Score:** 8/10

---

### 3. brandkit (leonxlnx/taste-skill)

**Source:** Leonxlnx (same repo as taste-skill)  
**Installs (skills.sh):** ~119.9K  
**GitHub Stars (parent repo):** ~55.5K total

**What it does:** Premium brand-kit image generation for identity systems, logo concepts, and visual-world presentations. 3x3 panel system, 5 logo concept methods, 10+ visual modes.

**Community reception:** Strong adoption. 119.9K installs makes it one of the most installed image-gen skills.

**Key quotes from reviews:**
- "If you need to mock up brand identity systems that don't look like AI slop, this is the move." (claudemarketplaces.com)
- "Generates presentation-ready brand kit boards with proper grid layouts, restrained typography, and actual symbolic thinking." (same source)

**Verified claims:** Yes. 3x3 panel system, 10 visual modes, 5 logo methods all documented in SKILL.md. Real output quality depends on underlying image model.

**Criticisms:** Output quality is limited by the image generation model being used. Prompts can be long and consume tokens. Some users find the default 3x3 panel layout repetitive.

**Trust Score:** 8/10

---

### 4. emil-design-eng (emilkowalski/skill)

**Source:** Emil Kowalski (ex-Vercel, ex-Linear, creator of Sonner, animations.dev)  
**License:** MIT  
**GitHub Stars:** 4.4K ★ (emilkowalski/skills repo)  
**Installs (skills.sh):** Not separately tracked

**What it does:** Encodes Emil Kowalski's design engineering philosophy — animation decision framework (frequency, purpose, easing, duration), component patterns (button press feedback, origin-aware popovers, tooltip delays), CSS transform mastery, gesture principles, and a required Before/After review table format.

**Community reception:** Excellent. Highly respected due to Emil's reputation from Vercel/Linear. Considered the authoritative source for tasteful UI motion.

**Key quotes from reviews:**
- "I have seen plenty of times that agents don't pick the right ingredients for an animation. An ease-in easing for an enter animation when it's supposed to be ease-out." (emilkowal.ski)
- "All these little things compound and make your interface either amazing, or just... not that great." (same source)

**Verified claims:** Yes. Emil is a real design engineer with proven track record. The skill accurately reflects his published articles and animations.dev course material.

**Criticisms:** Smaller scope than impeccable — focused specifically on animation/motion decisions, not full design systems. Requires familiarity with animation vocabulary to get full value.

**Trust Score:** 9/10

---

### 5. canvas-design (anthropics/skills)

**Source:** Anthropic (official)  
**License:** See LICENSE in repo  
**GitHub Stars (anthropics/skills):** Not separately tracked but official Anthropic repo

**What it does:** Creates beautiful visual art in .png and .pdf using design philosophy. Two-step process: design philosophy creation (aesthetic manifesto), then visual expression on canvas. Emphasizes museum/magazine quality, sparse typography, limited palettes.

**Community reception:** Positive but niche. Well-received by solo builders who need one-off graphics without a designer.

**Key quotes from reviews:**
- "Turns text briefs into finished canvas artwork (PNG/PDF-style deliverables)" (skillselion.com)
- "Claude generates visuals that are technically correct and aesthetically generic without guidance." (theskills.directory)

**Verified claims:** Yes. Official Anthropic skill with clear documentation. Two-step creative process is well-defined.

**Criticisms:** Output heavily depends on the rendering capability of the underlying agent (some can't actually produce valid PNG/PDF). The "design philosophy" step can feel overly abstract. Requires local font files (canvas-fonts directory).

**Trust Score:** 7/10

---

### 6. algorithmic-art (anthropics/skills)

**Source:** Anthropic (official)  
**Installs (skills.sh):** Not tracked (official repo)

**What it does:** Creates generative art using p5.js with seeded randomness (Art Blocks pattern), interactive parameter exploration, and self-contained HTML artifacts. Includes seed navigation, sliders, and canvas export.

**Community reception:** Positive. Appreciated for its Art Blocks-inspired approach and reproducible output via seeded randomness.

**Key quotes from reviews:**
- "This skill creates living algorithms, not static images with randomness." (mintlify docs)
- "Seed navigation controls (previous/next/random/jump) and parameter sliders with live updates." (SKILL.md)

**Verified claims:** Yes. Official Anthropic skill. Uses templates/viewer.html as starting point. Seeded randomness pattern is well-documented.

**Criticisms:** Output quality limited to p5.js (canvas 2D, not 3D). The template has Anthropic branding that must be replaced. Not suitable for production video or high-end motion graphics.

**Trust Score:** 7/10

---

## Group 2 — Animation/Video

### 7. hyperframes (heygen-com/hyperframes)

**Source:** HeyGen (commercial video platform, open-sourced framework)  
**License:** Apache 2.0  
**GitHub Stars:** Not directly tracked

**What it does:** Open-source framework + 20 skills for rendering video from HTML. Compositions use data-* attributes for timing, seekable animation via GSAP/Lottie/Three.js/Anime.js/CSS/WAAPI/TypeGPU adapters. Includes CLI (init, lint, preview, render, doctor).

**Community reception:** Very positive. Seen as the most viable open-source approach to HTML-to-video for AI agents. Backed by a real company (HeyGen).

**Key quotes from reviews:**
- "HyperFrames lets AI agents compose videos by writing HTML, CSS & JS." (hyperframes.heygen.com)
- "Now Claude Code can edit videos — install HyperFrames skill." (same source)

**Verified claims:** Yes. Real CLI, real rendering pipeline, 20 specialized skills. Active development.

**Criticisms:** Requires understanding of the data-* attribute contract and timeline model. The skill ecosystem is complex (20 skills to manage). Rendering quality depends on the runtime adapter chosen. Cloud rendering via Lambda may incur costs.

**Trust Score:** 8/10

---

### 8. GSAP skills (greensock/gsap-skills)

**Source:** GreenSock (official, acquired by Webflow Oct 2024)  
**License:** Standard GSAP license (all plugins now free)  
**GitHub Stars:** Not directly tracked

**What it does:** 8 official skills covering core API, timelines, ScrollTrigger, plugins, React/Vue/Svelte integration, and performance. Teaches agents correct GSAP patterns (use x/y not left/top, useGSAP hook, autoAlpha, context cleanup).

**Community reception:** Extremely positive. Considered a must-have for any frontend developer using AI coding agents. Called "the fix" for janky AI-generated animations.

**Key quotes from reviews:**
- "Your AI coding agent writes janky GSAP code. GreenSock just shipped the fix." (Medium)
- "Most libraries won't [do this]. So the takeaway isn't only 'install gsap-skills,' though you should." (same source)
- "Low-effort, high-return addition to the workflow of any frontend developer who uses AI coding assistants." (thnkandgrow.com)

**Verified claims:** Yes. Official GSAP skills from the platform creators. Works with 40+ agents. Covers all 8 skill domains.

**Criticisms:** Some skills overlap (gsap-react vs gsap-frameworks). The massive number of skills can be confusing to install selectively. Some users report the ScrollTrigger skill still misses edge cases.

**Trust Score:** 9/10

---

### 9. threejs skills (CloudAI-X/threejs-skills, Impertio-Studio, anthropics)

**Source:** Multiple (CloudAI-X has 11 skills, Impertio-Studio has 24, Anthropic has official)

**Community reception:** Mixed. Multiple competing packages cause confusion. The CloudAI-X set has ~11 skills with solid coverage. The Impertio-Studio set covers 24 skills including R3F, Drei, WebGPU.

**Key quotes from reviews:**
- "A curated collection of Three.js skill files that provide Claude Code with foundational knowledge" (CloudAI-X README)
- "24 deterministic skills for Three.js 3D web development with Claude Code" (Impertio-Studio)

**Verified claims:** Partially. Skills exist and are documented, but quality varies. The official Anthropic threejs skills are more conservative; community ones are more comprehensive but less vetted.

**Criticisms:** No single official "authoritative" Three.js skill package. Multiple sources competing. Version tracking is inconsistent.

**Trust Score:** 6/10

---

### 10. Motion.dev AI Kit (motion.dev)

**Source:** Motion.dev (Matt Perry, former Framer Motion author)  
**License:** Proprietary (Motion+ subscription for full features)

**What it does:** Official Motion AI Kit — MCP server + skills for 120fps GPU-accelerated animations, spring physics, scroll effects, gesture interactions. Includes MotionScore performance audit, CSS spring generation, transition editor.

**Community reception:** Positive among serious animation users. Motion.dev has 10M+ monthly downloads. The AI Kit is well-regarded but requires Motion+ subscription for full access.

**Key quotes from reviews:**
- "AI agents tend to guess at animation code, often from outdated or low-quality sources. The Motion AI Kit grounds yours in current, performance-checked Motion knowledge." (motion.dev docs)
- "The most important: only GPU-accelerated properties (transform, opacity, filter). No layout thrashing."

**Verified claims:** Yes. Real product with real MCP server. MotionScore audit is functional. The kit is actively maintained by the Motion team.

**Criticisms:** Requires Motion+ subscription for premium examples and full features. The free tier is limited. Some users prefer GSAP's more generous free model.

**Trust Score:** 7/10

---

## Group 3 — Image/AI Generation

### 11. imagegen-frontend-web (leonxlnx/taste-skill)

**Source:** Leonxlnx (taste-skill repo)  
**Installs (skills.sh):** ~87.8K  
**GitHub Stars (parent repo):** ~55.5K total

**What it does:** Elite frontend image-direction skill. Hard rule: one separate horizontal image per section. Enforces composition variety, full-bleed backgrounds, varied hero scales, consistent palette. Defaults: 6 sections for landing page, 8 for full website.

**Community reception:** Strong. Popular for the "one image per section" rule which solves a real problem with AI image generation collapsing multi-section pages.

**Key quotes from reviews:**
- "The biggest differentiator is the hard rule to generate one separate horizontal image for every section." (agentskillsfinder.com)
- "Generate one horizontal reference image per landing section so you or your coding agent can recreate premium, conversion-aware marketing UI." (skillselion.com)

**Verified claims:** Yes. The hard output rule is well-documented and enforceable. Composition variety guidelines are comprehensive.

**Criticisms:** Only works with image models capable of section-level generation. Requires multiple API calls per page (expensive). Not suitable for single illustrations or non-web imagery.

**Trust Score:** 7/10

---

### 12. fal.ai skills (fal-ai-community/skills)

**Source:** fal.ai (official community skills repository)  
**License:** Community standard  
**GitHub Stars:** ~198  
**Installs (skills.sh):** ~319 (fal-generate)

**What it does:** Collection of bash scripts for generating images, videos, audio, 3D models via fal.ai API. Includes queue-based generation, model discovery, schema inspection. Supports 50+ models (Flux, SDXL, Ideogram, Kling, Veo, etc.).

**Community reception:** Positive but niche compared to direct API usage. Useful for AI agents that need programmatic access to fal.ai.

**Key quotes from reviews:**
- "Agent skills for fal.ai — ready-to-use bash scripts that let AI agents generate images, videos, audio, 3D models, and more." (github repo)
- "Compatible with Claude.ai Projects, Claude Code, and any agent platform."

**Verified claims:** Yes. Real bash scripts that work. The skills are thin wrappers around the fal API queue system.

**Criticisms:** Requires FAL_KEY setup. Skills are relatively simple bash scripts (not deep knowledge integration). Installation numbers are modest compared to other skills. Multiple competing fal.ai skill packages exist (karamble/claude-skill-falai, analyticalmonk/fal-ai-skill).

**Trust Score:** 6/10

---

### 13. image-to-code (leonxlnx/taste-skill)

**Source:** Leonxlnx (taste-skill repo)

**What it does:** Elite website image-to-code skill for Codex. Mandatory image-first pipeline: generate design images → deep analyze → extract design system → implement frontend. Forces one image per section, fresh regeneration, no cropping old images.

**Community reception:** Niche but appreciated for the disciplined workflow it enforces. Popular among Codex users.

**Key quotes from reviews:**
- "For visually important web tasks, it must first generate the design image(s) itself, deeply analyze them, then implement the website to match them as closely as possible." (SKILL.md)
- "It forces an image first workflow: generate the design, analyze what was actually created, then code it faithfully." (claudemarketplaces.com)

**Verified claims:** Partially. The workflow is documented but execution depends on the agent's image generation capabilities. Codex-specific optimizations may not transfer to other agents.

**Criticisms:** Very Codex-specific. Heavy token usage from image generation + analysis + implementation. The "deep analysis" step is subjective — agents may not analyze effectively. Alternative: Ixe1/ui-from-image is a simpler competitor.

**Trust Score:** 5/10

---

## Group 4 — SEO/Marketing/Dev

### 14. ai-seo (coreyhaines31/marketingskills)

**Source:** Corey Haines (marketing-skills repo)  
**GitHub Stars (parent repo):** ~35.1K

**What it does:** AI search optimization skill — optimizes content for Google AI Overviews, ChatGPT, Perplexity, Claude, Gemini, Copilot. Three-pillar approach: structure (extractable answer blocks, FAQ schema), authority (citations, statistics), presence (llms.txt, robots.txt, third-party citations).

**Community reception:** Strong. Considered the authoritative skill for AI SEO/GEO. Built by a known marketing figure (Corey Haines runs Swipe Files, Conversion Factory).

**Key quotes from reviews:**
- "Traditional SEO gets you ranked. AI SEO gets you cited." (AI SEO skill doc)
- "Content with proper schema shows 30-40% higher AI visibility on non-Google AI engines." (same source)

**Verified claims:** Yes. Strategies align with industry best practices. The three-pillar framework is practical and well-documented. Realistic about Google's stance (structured data not required for AI Overviews but helps).

**Criticisms:** Some claims (30-40% higher visibility) are hard to independently verify. The field (GEO/AEO) is still emerging with rapidly changing best practices. Competes with Claude SEO (7K+ GitHub stars) which is free and open-source.

**Trust Score:** 7/10

---

### 15. audit-website (squirrelscan/skills)

**Source:** squirrelscan (commercial + free CLI)  
**GitHub Stars:** Not tracked separately

**What it does:** Website auditing via squirrelscan CLI — 245+ rules across SEO, performance, security, accessibility, content, and 15+ other categories. LLM-optimized report format. Coverage modes: quick (25 pages), surface (100), full (500).

**Community reception:** Positive for its agent-friendly design. Built specifically for AI workflows with LLM-optimized output format.

**Key quotes from reviews:**
- "240+ SEO, performance & security rules — run them from the CLI, inside your coding agent, in the cloud, or over MCP." (squirrelscan.com)
- "squirrelscan is built for autonomous AI workflows." (docs)

**Verified claims:** Yes. Real CLI, real audits. Free for local use. Cloud features require credits (paid).

**Criticisms:** Free tier limited to quick coverage (25 pages). Cloud features (rendering, AI analysis, dead-link checks) cost credits. Competes with Claude SEO (free, 7K+ GitHub stars) and other SEO tools.

**Trust Score:** 7/10

---

### 16. copywriting (coreyhaines31/marketingskills)

**Source:** Corey Haines (marketing-skills repo)  
**Installs (skills.sh):** ~136.8K  
**GitHub Stars (parent repo):** ~35.1K

**What it does:** Conversion-focused marketing copywriting — homepages, landing pages, pricing pages, feature pages. Includes page structure framework, CTA formulas, voice/tone guidance, AI-tell sweep (removes "leverage", "best-in-class", "unlock", "seamless", "robust").

**Community reception:** Very popular. 136.8K installs makes it one of the most installed marketing skills. Well-regarded for its practical, conversion-oriented approach.

**Key quotes from reviews:**
- "Specific over vague — avoid 'streamline,' 'optimize,' 'innovative.' Confidence over qualified — remove 'almost,' 'very,' 'really.'" (SKILL.md)
- "'Save time' is invisible. 'Cut your weekly reporting from 6 hours to 20 minutes' earns the click." (agentcookbooks.com)

**Verified claims:** Yes. Strong, opinionated rules. The AI-tell sweep addresses a real problem. Well-integrated with other marketing skills.

**Criticisms:** Best results require populating product-marketing-context.md first (extra setup). The skill's voice/tone guidance is generic — needs customization for specific brands.

**Trust Score:** 8/10

---

### 17. ai-sdk (vercel/ai)

**Source:** Vercel (official)  
**GitHub Stars:** 107K+ (vercel/ai monorepo)

**What it does:** Official AI SDK skill for coding agents. Includes full AI SDK documentation, ToolLoopAgent, streamText, generateText, embed, and provider integrations. Recently added HarnessAgent for running Claude Code, Codex, Pi programmatically.

**Community reception:** Extremely strong. 107K+ GitHub stars makes it one of the most popular AI libraries. The skill is actively maintained by Vercel.

**Key quotes from reviews:**
- "The AI Toolkit for TypeScript. From the creators of Next.js." (github)
- "HarnessAgent lets you switch the harness the same way you switch models." (vercel.com/changelog)
- "Program Claude Code, Codex, Pi and other agent harnesses with AI SDK." (same source)

**Verified claims:** Yes. Official Vercel product with massive adoption. HarnessAgent is a real feature in AI SDK 7. The skill is genuinely useful for AI SDK users.

**Criticisms:** The skill is most useful if you're already using the AI SDK. HarnessAgent is experimental and may change. Some providers require API keys through Vercel AI Gateway.

**Trust Score:** 9/10

---

## Group 5 — Utility

### 18. writing-plans (obra/superpowers)

**Source:** obra/superpowers  
**Installs (skills.sh):** Not directly tracked (part of superpowers collection)

**What it does:** Creates detailed bite-sized implementation plans with TDD structure. Each step 2-5 minutes. Exact file paths, complete code, test commands. Scope check, task structure, no-placeholder policy, execution handoff (subagent-driven or inline).

**Community reception:** Very positive among heavy Claude Code users. Part of the popular "superpowers" collection by Jesse Vincent.

**Key quotes from reviews:**
- "Write comprehensive implementation plans assuming the engineer has zero context for our codebase and questionable taste." (SKILL.md)
- "Bite-sized 2-5 minute steps enable frequent commits, easier code review, and safer parallel execution by subagents." (skillstore.io)

**Verified claims:** Yes. Well-structured approach with clear output. The no-placeholder policy is strict and enforced.

**Criticisms:** Requires discipline to use properly. Plans can be verbose. Works best when paired with executing-plans skill. Some users find the zero-context assumption overly cautious.

**Trust Score:** 8/10

---

### 19. brainstorming (obra/superpowers)

**Source:** obra/superpowers  
**Installs (skills.sh):** Part of superpowers collection

**What it does:** Turns rough ideas into implementation-ready designs through structured codebase-grounded research. Multi-phase: explore context, ask clarifying questions, propose 2-3 approaches, present design, write spec, transition to writing-plans.

**Community reception:** Strong. Popular for its disciplined approach to preventing premature implementation. The "no code before design approval" gate is widely appreciated.

**Key quotes from reviews:**
- "You MUST use this before any creative work — creating features, building components, adding functionality, or modifying behavior." (SKILL.md)
- "Do NOT invoke any implementation skill, write any code, scaffold any project, or take any implementation action until you have presented a design and the user has approved it." (same source)

**Verified claims:** Yes. Multi-phase approach is well-documented. Transitions to writing-plans skill as designed.

**Criticisms:** Can feel overly bureaucratic for small changes. The strict "no implementation before approval" rule can slow down rapid prototyping. Works best for medium-to-large features.

**Trust Score:** 8/10

---

### 20. systematic-debugging (obra/superpowers)

**Source:** obra/superpowers (Jesse Vincent)  
**Installs (skills.sh):** Widely distributed (multiple forks)

**What it does:** Four-phase deterministic debugging methodology — Phase 1: Root Cause Investigation, Phase 2: Pattern Analysis, Phase 3: Hypothesis and Testing, Phase 4: Implementation. Core principle: "NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST."

**Community reception:** Very positive. Considered essential for preventing the "guess and check" debugging pattern common in AI agents.

**Key quotes from reviews:**
- "Systematic debugging achieves ~95% first-time fix rate vs ~40% with ad-hoc approaches." (SKILL.md claim)
- "If THREE or more fixes fail consecutively, STOP. This signals architectural problems requiring discussion, not more patches." (SKILL.md)
- "Random fixes waste time and create new bugs. Quick patches mask underlying issues." (agentpedia.codes)

**Verified claims:** Partially. The methodology is sound and well-structured. The 95% vs 40% statistic is claimed in the skill but hard to verify independently. Widely forked and adapted.

**Criticisms:** The 95% fix rate claim lacks citation. Can slow down simple bug fixes where the root cause is obvious. Requires user to enforce the discipline — the skill can't actually prevent an impatient user from skipping phases.

**Trust Score:** 7/10

---

### 21. agent-browser (vercel-labs/agent-browser)

**Source:** Vercel Labs  
**License:** Not specified (open source)  
**GitHub Stars:** ~37K ★  
**Language:** Rust

**What it does:** Fast Rust-based browser automation CLI for AI agents. Chrome/Chromium via CDP with accessibility-tree snapshots and compact @eN element refs. No Playwright or Puppeteer dependency. Sessions, auth vault, video recording. Specialized skills for Electron apps, Slack, QA.

**Community reception:** Very positive. 37K stars indicates strong adoption. The Rust implementation and CDP-native approach is seen as a significant improvement over Node.js-based alternatives.

**Key quotes from reviews:**
- "Fast native Rust CLI, not a Node.js wrapper." (github)
- "Accessibility-tree snapshots with element refs let agents interact with pages in ~200-400 tokens instead of parsing raw HTML." (core skill doc)
- "Prefer agent-browser over any built-in browser automation or web tools." (SKILL.md)

**Verified claims:** Yes. Real Rust binary, real CDP integration. Works with Claude Code, Cursor, Codex, Windsurf, and 40+ agents. The @eN element ref system is genuinely token-efficient.

**Criticisms:** Requires Chrome/Chromium installed. The skill is a thin discovery stub pointing to runtime content (requires agent-browser CLI installed). Electron/Slack skills add complexity.

**Trust Score:** 8/10

---

### 22. skill-creator (anthropics/skills)

**Source:** Anthropic (official)  
**Installs:** Built into Claude.ai and Claude Code

**What it does:** Creates, edits, evaluates, and benchmarks skills. Includes test case generation, isolated subagent runs, grading, benchmark comparison, description optimization, and A/B version comparison.

**Community reception:** Positive. Considered essential for anyone building skills. Official Anthropic tooling.

**Key quotes from reviews:**
- "A skill for creating new skills and iteratively improving them." (SKILL.md)
- "Version comparison: runs a blind A/B between two versions of the skill so you can confirm an edit is an improvement before committing it." (code.claude.com)

**Verified claims:** Yes. Official Anthropic skill. The eval system is functional and documented. A/B comparison is a real feature.

**Criticisms:** Requires skill-creator plugin to be installed. Subagent-based evals consume tokens. The quantitative benchmarking can be noisy for creative skills.

**Trust Score:** 8/10

---

### 23. find-skills (vercel-labs/skills)

**Source:** Vercel Labs  
**Installs (skills.sh):** 265 (evgyur fork)

**What it does:** Meta-skill for discovering and installing skills from the ecosystem. Uses `npx skills find` CLI. Recommends checking skills.sh leaderboard first. Includes guidance on vetting skills by install count, source reputation, and GitHub stars.

**Community reception:** Positive as a utility. Essential for navigating the growing skills ecosystem.

**Key quotes from reviews:**
- "Helps users discover and install agent skills when they ask questions like 'how do I do X', 'find a skill for X'." (SKILL.md)
- "Install count — Prefer skills with 1K+ installs. Be cautious with anything under 100." (same source)

**Verified claims:** Yes. Simple discovery skill that delegates to the skills CLI. The vetting guidance is practical.

**Criticisms:** Limited utility if the skills CLI isn't installed. The skill itself is just a markdown file with instructions — it doesn't actually perform searches autonomously.

**Trust Score:** 7/10

---

## Group 6 — Specific / VantaDB

### 24. vanta-design-orchestrator (custom VantaDB skill)

**Source:** Custom skill for VantaDB project  
**Status:** Installed locally at `.agents/skills/vanta-design-orchestrator/SKILL.md`

**What it does:** Master orchestrator that integrates all local UI/UX design tools, brand strategy, and skills. References business model design, brand platform, Krug UX, Impeccable CLI, Emil animation philosophy, Motion library, Anime.js, GSAP (8 skills), Three.js/Shader pipeline, SEO audit, Visual Review pipeline, and multiple image-gen/design skills.

**Community reception:** N/A — custom/internal skill. No public presence.

**Key quotes from reviews:**
- "Master orchestrator and role definition for local UI/UX design tools, brand strategy, and skills." (SKILL.md)

**Verified claims:** N/A — internal tool. The skill references a comprehensive set of external skills and tools.

**Criticisms:** Unclear whether it effectively orchestrates all referenced skills or creates redundancy. No public validation.

**Trust Score:** N/A (custom/internal)

---

### 25. next-best-practices (vercel-labs/next-skills) & vercel-react-best-practices (vercel-labs/agent-skills)

**Source:** Vercel Labs (official)  
**License:** MIT

**What it does (next-best-practices):** Next.js best practices — file conventions, RSC boundaries, data patterns, async APIs, metadata, error handling, route handlers, image/font optimization, bundling.

**What it does (vercel-react-best-practices):** 64+ rules across 8 categories (waterfalls CRITICAL, bundle size CRITICAL, server-side HIGH, client-side MEDIUM-HIGH, re-render MEDIUM, rendering MEDIUM, JS performance LOW-MEDIUM, advanced LOW).

**Community reception:** Very positive. Considered authoritative since they come directly from Vercel engineering. The InfoQ article (Feb 2026) highlighted the structured, impact-prioritized approach.

**Key quotes from reviews:**
- "React and Next.js performance optimization guidelines from Vercel Engineering. Contains 64 rules across 8 categories, prioritized by impact." (SKILL.md)
- "The two highest-priority categories focus on eliminating async waterfalls and reducing bundle size." (InfoQ)

**Verified claims:** Yes. Official Vercel-released skills. Well-structured with clear impact levels. The repository includes build scripts and test cases.

**Criticisms:** Some overlap between next-best-practices and react-best-practices. Rules can be Vercel-platform-specific (e.g., preferring Vercel AI Gateway). Large compiled AGENTS.md document can be verbose.

**Trust Score:** 9/10

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Skills researched | 25 |
| GitHub Stars (highest) | 107K+ (ai-sdk) |
| GitHub Stars (design category highest) | ~55.5K (taste-skill) |
| Installs highest | ~136.8K (copywriting) |
| Average Trust Score | 7.5/10 |
| Official Anthropic skills | 4 (canvas-design, algorithmic-art, skill-creator, frontend-design) |
| Official Vercel skills | 4 (ai-sdk, agent-browser, next-best-practices, react-best-practices) |
| Official GSAP/GreenSock skills | 8 (entire gsap-skills suite) |
| Community skills | ~9 |
| Custom/internal skills | 1 (vanta-design-orchestrator) |
