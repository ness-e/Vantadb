# Agent-04: Global Claude Skills Analysis

**Directory:** `C:\Users\Eros\.claude\skills\`
**Date:** 2026-07-03
**Total directories found:** 36
**Directories with SKILL.md:** 27
**Empty directories (no content):** 9

---

## EMPTY DIRECTORIES (NO SKILL.MD — No Content)

These directories exist but contain ZERO files. They are placeholders at best:

| Directory | Purpose (inferred from name) |
|-----------|------------------------------|
| `cargo-nextest` | Rust test runner (nextest) |
| `github-repo-management` | GitHub repo management |
| `m10-performance` | Unknown — possibly M10 framework |
| `markdown-documentation` | Markdown documentation |
| `python-packaging` | Python packaging |
| `rust-ffi` | Rust FFI |
| `rust-write-tests` | Rust test writing |
| `test-reporting` | Test reporting |
| `vector-database-engineer` | Vector database (e.g., Pinecone, Qdrant) |

**Verdict:** All 9 are non-functional. Either the skill installation failed, or they require manual setup. Remove or install properly.

---

## DETAILED SKILL-BY-SKILL ANALYSIS

### 1. ai-seo
- **Path:** `C:\Users\Eros\.claude\skills\ai-seo\`
- **Version:** 2.1.0
- **Lines:** 489
- **Category:** Backend/Dev/SEO
- **Description:** Optimizes content for AI search engines (ChatGPT, Perplexity, Google AI Overviews, Claude, Gemini). Covers AI citations, llms.txt, OKF, knowledge bundles, and agent-readable sites.
- **Quality:** Very thorough with structured diagnostic workflow. Includes product-marketing context integration.
- **Rating:** 9/10
- **Keep/Remove:** KEEP — Excellent, covers a modern and growing SEO niche that traditional SEO skills don't.

### 2. animejs
- **Path:** `C:\Users\Eros\.claude\skills\animejs\`
- **Lines:** 525
- **Category:** Animation/Motion/3D
- **Description:** Lightweight JS animation library for DOM, CSS, SVG, and JS objects. Covers timelines, stagger, SVG morphing, keyframes, spring physics.
- **Quality:** Very comprehensive with practical examples. Well-organized reference.
- **Rating:** 8/10
- **Keep/Remove:** KEEP — Good alternative to GSAP for SVG-heavy/framework-agnostic projects. Compliments the GSAP suite.

### 3. design-motion-principles
- **Path:** `C:\Users\Eros\.claude\skills\design-motion-principles\`
- **Lines:** 122
- **Category:** Animation/Motion/3D
- **Description:** Motion/interaction design expert based on Emil Kowalski, Jakub Krehel, and Jhey Tompkins. Two modes: create (build interactive components) and audit (review existing motion design).
- **Quality:** Well-structured with clear mode detection. References external workflow files.
- **Rating:** 7/10
- **Keep/Remove:** KEEP — Unique perspective from known designers. Usefully distinct from generic motion skills.

### 4–10. GSAP Suite (7 skills)
All official, MIT-licensed, well-structured skills from the GSAP team. They cover the full GSAP ecosystem in granular detail.

| Skill | Lines | Rating | Notes |
|-------|-------|--------|-------|
| **gsap-core** | 254 | 9/10 | Core API — essential |
| **gsap-frameworks** | 266 | 8/10 | Vue/Svelte lifecycle integration |
| **gsap-performance** | 79 | 7/10 | Focused but brief — could merge into core |
| **gsap-plugins** | 433 | 9/10 | Comprehensive plugin coverage |
| **gsap-react** | 136 | 8/10 | useGSAP hook, cleanup patterns |
| **gsap-scrolltrigger** | 296 | 9/10 | Thorough scroll animation docs |
| **gsap-timeline** | 107 | 7/10 | Short — could merge into core |
| **gsap-utils** | 284 | 8/10 | Utility helpers (clamp, etc.) |

- **Keep:** All 7. The GSAP suite is comprehensive and official.
- **Consolidation opportunity:** `gsap-performance`, `gsap-timeline`, and `gsap-frameworks` could theoretically be merged into `gsap-core` to reduce granularity.

### 11. hyperframes
- **Path:** `C:\Users\Eros\.claude\skills\hyperframes\`
- **Lines:** 490
- **Category:** Deck/Slide/Video Template
- **Description:** HTML-based video composition framework. Covers timing, media, animation, captions, TTS, audio-reactive visuals, transitions, text highlighting.
- **Quality:** Very comprehensive with clear workflows (discovery, design system, step-by-step composition).
- **Rating:** 9/10
- **Keep:** EXCELLENT — Core video production skill for VantaDB's video/embed needs.

### 12. hyperframes-animation
- **Path:** `C:\Users\Eros\.claude\skills\hyperframes-animation\`
- **Lines:** 82 (references rules, blueprints, transitions, 7 runtimes)
- **Category:** Animation/Motion/3D
- **Description:** All animation knowledge for HyperFrames: atomic rules, multi-phase scene templates, transitions, 7 runtime adapters (GSAP, Lottie, Three.js, Anime.js, CSS, WAAPI, TypeGPU).
- **Quality:** Compact but references extensive sub-files. Good companion to hyperframes.
- **Rating:** 8/10
- **Keep:** KEEP — Essential companion to hyperframes.

### 13. image-edit
- **Path:** `C:\Users\Eros\.claude\skills\image-edit\`
- **Lines:** 270
- **Category:** Image/Video/AI Media
- **Description:** Smart router for image editing via RunComfy CLI. Picks between Nano Banana Edit, GPT Image 2 Edit, Flux Kontext Pro, or Z-Image Turbo Inpaint based on intent.
- **Quality:** Well-designed intent-routing approach. Requires `runcomfy` CLI and network access.
- **Rating:** 7/10
- **Keep:** KEEP — Useful for AI-powered image editing workflows. Conditional on RunComfy setup.

### 14. impeccable
- **Path:** `C:\Users\Eros\.claude\skills\impeccable\`
- **Version:** 3.7.1
- **Lines:** 174+ (with reference files)
- **Category:** Frontend/UI Design
- **Description:** Production-grade frontend design/redesign/audit/polish. Covers UX review, visual hierarchy, accessibility, performance, responsive behavior, typography, color, motion, micro-interactions, design systems, tokens.
- **Quality:** Extremely well-structured with sub-commands (craft, shape, audit, polish), context scripts, palette generation, and detailed design guidance. Version 3.7.1 indicates active maintenance.
- **Rating:** 10/10
- **Keep:** EXCELLENT — Top-tier frontend design skill. Keep as primary design authority.

### 15. interaction-design
- **Path:** `C:\Users\Eros\.claude\skills\interaction-design\`
- **Lines:** 320
- **Category:** Animation/Motion/3D
- **Description:** Microinteractions, motion design, transitions, loading states, skeleton screens, gestures, drag-and-drop, scroll-triggered animations.
- **Quality:** Good principles but heavily overlaps with `impeccable` and `design-motion-principles`.
- **Rating:** 6/10
- **Keep/Remove:** REMOVE — Redundant with impeccable (which covers all this + more).

### 16. motion
- **Path:** `C:\Users\Eros\.claude\skills\motion\`
- **Version:** Based on Motion v12.29.2 (generated 2026-02-01)
- **Lines:** 105+ (with reference files)
- **Category:** Animation/Motion/3D
- **Description:** Motion animation library (motion.dev) for JS, React, Vue. Covers motion components, animate API, gestures, springs, layout transitions, scroll-linked effects, timelines.
- **Quality:** Up-to-date, based on latest Motion version. Auto-generated from official source. Well-structured with reference files.
- **Rating:** 9/10
- **Keep:** EXCELLENT — Modern, up-to-date animation library coverage. Preferred by AGENTS.md for motion.dev over CSS.

### 17. remotion-best-practices
- **Path:** `C:\Users\Eros\.claude\skills\remotion-best-practices\`
- **Lines:** 340
- **Category:** Deck/Slide/Video Template
- **Description:** Best practices for Remotion (React-based video creation). Covers project setup, animation (useCurrentFrame, interpolate), composition, rendering.
- **Quality:** Practical code examples. Good for React-native video creation.
- **Rating:** 7/10
- **Keep:** KEEP — Complementary to HyperFrames for React-native video workflow.

### 18. roier-seo
- **Path:** `C:\Users\Eros\.claude\skills\roier-seo\`
- **Version:** 1.0.0
- **Lines:** 458
- **Category:** Backend/Dev/SEO
- **Description:** Technical SEO auditor with Lighthouse/PageSpeed integration. Auto-fixes meta tags, structured data, Core Web Vitals, accessibility. Framework-aware (Next.js, React, Vue).
- **Quality:** Well-structured with dependency management (lighthouse, chrome-launcher). Auto-fix capability is practical.
- **Rating:** 8/10
- **Keep:** KEEP — Automated SEO auditing/ fixing is useful. Overlaps partially with `seo` but auto-fix adds value.

### 19. seo
- **Path:** `C:\Users\Eros\.claude\skills\seo\`
- **Version:** 1.0
- **Lines:** 527
- **Category:** Backend/Dev/SEO
- **Description:** General SEO optimization. Covers technical SEO, on-page optimization, crawlability, structured data, page experience.
- **Quality:** Comprehensive, well-structured with tables and references. Author: web-quality-skills.
- **Rating:** 8/10
- **Keep:** KEEP — Solid general SEO reference. Compliments roier-seo (manual vs automated).

### 20. sleek-design-mobile-apps
- **Path:** `C:\Users\Eros\.claude\skills\sleek-design-mobile-apps\`
- **Lines:** 520
- **Category:** Mobile
- **Description:** Mobile app design via Sleek API. Create projects, design screens, take screenshots. Requires SLEEK_API_KEY and Pro plan.
- **Quality:** Well-documented API integration. However, requires paid plan ($).
- **Rating:** 6/10
- **Keep:** REMOVE — Conditional on paid Sleek Pro subscription. If not actively using Sleek, remove.

### 21–26. Three.js Suite (6 skills)
Granular decomposition of Three.js into fundamentals, geometry, materials, animation, interaction, and shaders.

| Skill | Lines | Rating | Notes |
|-------|-------|--------|-------|
| **threejs-fundamentals** | 488 | 8/10 | Scene setup, cameras, renderer, transforms |
| **threejs-geometry** | 548 | 8/10 | Built-in shapes, BufferGeometry, instancing |
| **threejs-materials** | 520 | 8/10 | PBR, basic/phong/standard, textures |
| **threejs-animation** | 552 | 8/10 | Keyframe, skeletal, morph targets, GLTF |
| **threejs-interaction** | 660 | 9/10 | Raycasting, controls, mouse/touch input |
| **threejs-shaders** | 642 | 9/10 | GLSL, ShaderMaterial, uniforms, effects |

- **Keep:** All 6. Comprehensive Three.js coverage.
- **Rating rationale:** Each skill is thorough (488–660 lines) with practical code examples and good structure. They are slightly granular but well-defined boundaries make sense. Consolidation not recommended due to distinct technical domains.

---

## SUMMARY OF QUALITY INDICATORS

**Excellent (9-10):**
- impeccable (10) — Top-tier frontend design, active maintenance
- ai-seo (9) — Modern topic, thorough
- gsap-core (9), gsap-plugins (9), gsap-scrolltrigger (9) — Official, comprehensive
- hyperframes (9) — Core video production
- motion (9) — Up-to-date, modern animation
- threejs-interaction (9), threejs-shaders (9) — Very comprehensive

**Good (7-8):**
- animejs (8), gsap-frameworks (8), gsap-react (8), gsap-utils (8)
- hyperframes-animation (8)
- roier-seo (8), seo (8)
- remotion-best-practices (7)
- threejs-fundamentals/geometry/materials/animation (all 8)
- image-edit (7)

**Adequate (6):**
- interaction-design (6) — Redundant with impeccable
- sleek-design-mobile-apps (6) — Paywalled
- gsap-performance (7) — Short, could merge
- gsap-timeline (7) — Short, could merge

**Non-functional (0):**
- cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer

---

## REDUNDANCY DETECTION

| Redundant Group | Skills | Recommendation |
|----------------|--------|---------------|
| Motion Design | interaction-design, design-motion-principles, impeccable | Remove interaction-design (covered by impeccable) |
| SEO | seo, roier-seo, ai-seo | Keep all 3 — different focuses (general, automated, AI-specific) |
| GSAP Granularity | gsap-performance (79 lines), gsap-timeline (107 lines) | Consider merging into gsap-core |
| Animation Libraries | GSAP suite (7) + animejs + motion + Three.js (6) | All distinct use cases — keep |

---

## FINAL RECOMMENDATION

**Keep (24):** ai-seo, animejs, design-motion-principles, gsap-core, gsap-frameworks, gsap-performance, gsap-plugins, gsap-react, gsap-scrolltrigger, gsap-timeline, gsap-utils, hyperframes, hyperframes-animation, image-edit, impeccable, motion, remotion-best-practices, roier-seo, seo, threejs-animation, threejs-fundamentals, threejs-geometry, threejs-interaction, threejs-materials, threejs-shaders

**Remove (1):** interaction-design (redundant)

**Conditional (1):** sleek-design-mobile-apps (requires paid Sleek Pro)

**Non-functional (9 — install or remove):** cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer
