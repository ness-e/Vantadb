# VantaDB Skills Manifest

**Location:** **193 skills en `.opencode/skills/`** (git submodule → `configOpencode` private repo); total **193** (project-local; curated essential list below)
**Updated:** 2026-09-01

> **AUDIT 2026-09-01 — Reestructuración completada:**
> - `.opencode/skills/` = **193** skills (submodule `configOpencode` — todas las skills del proyecto viven aquí)
> - `.agents/skills/` = **ELIMINADO** (2026-09-01) — contenido migrado a submodule
> - **Total proyecto = 193** (todas en `.opencode/skills/`)
> - Global `~/.agents/skills/` = 160 dirs (139 con SKILL.md, 14 sin); `~/.claude/skills/` = 33 dirs
> - **87 skills nuevas documentadas 2026-08-19**: 63 de accesibilidad (5 `a11y-*` + 58 `incl-*`), 17 de diseño/UX/utilidad, 7 de pipeline/VantaDB core
> - Stub detectado: `debugging-and-error-recovery` (`.opencode/skills/`) está DEPRECADO — delega en `systematic-debugging` (EVAL-03). Se mantiene listado porque existe en disk, pero NO se debe cargar.
> - `impeccable` (Essential, KEEP) ahora existe en `.opencode/skills/` (submodule) — sync completado.

---

## Quick Navigation

- [Essential Skillset (37)](#essential-skillset-37)
- [All Skills by Category](#all-skills-by-category)
- [Accessibility / Inclusive Design (63)](#accessibility--inclusive-design-63)
- [Pipeline / VantaDB Core (7)](#pipeline--vantadb-core-7)
- [Skill Loading Guide](#skill-loading-guide)

---

## Essential Skillset (37)

These 37 skills form the lean VantaDB toolset. Load the relevant ones based on task type.

### Frontend/UI Design (8)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `impeccable` | 10/10 | **Default for all frontend work.** Design audit, critique, polish. v3.7.1. |
| `design-taste-frontend` | 9/10 | Anti-slop direction finder. Load BEFORE implementing to establish design direction. |
| `interface-design` | 7/10 | App/dashboard focused. Not for marketing pages. |
| `high-end-visual-design` | 8/10 | Agency-level design rules. "Absolute Zero" anti-pattern list. Makes sites look expensive. |
| `ui-ux-pro-max` | 7/10 | Design reference database (50 styles, 21 palettes, 50 font pairings). |
| `web-design-guidelines` | 7/10 | Auto-updating compliance checker. Use for accessibility and standards reviews. |
| `redesign-existing-projects` | 8/10 | Upgrade existing websites to premium quality. Audit-first approach. |
| `platform-design` | 8/10 | 300+ Apple HIG, Material Design 3, WCAG 2.2 rules. Cross-platform. |

### Animation/Motion (8)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `gsap-core` | 9/10 | **Foundation for all GSAP animation.** MIT license. |
| `gsap-scrolltrigger` | 9/10 | Scroll-linked animation, pinning, scrub. Extremely detailed. |
| `gsap-timeline` | 8/10 | Timeline sequencing, position parameter, nesting. |
| `gsap-plugins` | 8/10 | SplitText, MorphSVG, Draggable, ScrollSmoother, CustomEase. |
| `gsap-react` | 8/10 | useGSAP hook, React cleanup, gsap.context(). |
| `motion` | 9/10 | **Preferred over CSS for animations per AGENTS.md.** v12.29.2, 120fps, gestures. |
| `design-motion-principles` | 8/10 | Create + Audit modes. Emil/Jhey/Jakub techniques. |
| `emil-design-eng` | 9/10 | Emil Kowalski UI polish philosophy. "Invisible details." |

### Design Systems (4)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `design-systems` | 8/10 | Complete design system management — tokens, components, accessibility. |
| `color-expert` | 8/10 | 286K words of color science. OKLCH/OKLAB, palette generation. |
| `theme-factory` | 8/10 | 10 preset themes + custom generator. Colors/typography for slides/docs. |
| `platform-design` | 8/10 | 300+ Apple HIG, Material Design 3, WCAG 2.2 rules. Cross-platform. |

### Backend / Dev / SEO (5)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `vercel-react-best-practices` | 9/10 | 70+ React/Next.js production rules from Vercel Engineering. |
| `ai-sdk` | 8/10 | Vercel AI SDK. Auto-updates with current model IDs. |
| `api-design-principles` | 7/10 | REST + GraphQL API design. 528 lines. |
| `ai-seo` | 8/10 | AI search optimization. Optimize for LLM citation. |
| `seo-audit` | 7/10 | Full SEO audit. Technical, on-page, content issues. |

### Engineering Lifecycle (12)

| Skill | Rating | When to Use |
|-------|:------:|-------------|
| `systematic-debugging` | 8/10 | **Load BEFORE any bug fix.** Root-cause-first methodology. |
| `writing-plans` | 8/10 | **Load before multi-step implementation.** Bite-sized plans. |
| `brainstorming` | 8/10 | **Load before creative work.** Idea-to-spec dialogue. |
| `spec-driven-development` | 8/10 | Write spec/PRD before coding. |
| `test-driven-development` | 8/10 | Red-Green-Refactor. |
| `source-driven-development` | 8/10 | Verify docs before implementing. |
| `doubt-driven-development` | 8/10 | Adversarial review for high-stakes. |
| `code-review-and-quality` | 9/10 | Multi-axis review before merge. |
| `incremental-implementation` | 8/10 | Thin vertical slices. |
| `ci-cd-and-automation` | 7/10 | CI/CD pipelines, Shift Left. |
| `git-workflow-and-versioning` | 8/10 | Atomic commits, trunk-based. |
| `documentation-and-adrs` | 7/10 | ADRs, API docs, feature docs. |

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
| impeccable-design-polish | 7 | KEEP | Polish companion. |
| web-artifacts-builder | 5 | KEEP | Official Anthropic ref. |
| article-magazine | 6 | REMOVED | Magazine layout — no blog/article design. |
| card-twitter | 5 | REMOVED | Twitter card — no social media. |
| card-xiaohongshu | 5 | REMOVED | Xiaohongshu card — no social media. |
| faq-page | 6 | REMOVED | FAQ template — usar HTML directo. |
| mockup-device-3d | 7 | REMOVED | iPhone/MacBook showcase — no need. |
| doc-kami-parchment | 6 | REMOVED | Editorial one-pager — no docs branding. |
| ui-design | 8 | KEEP | 859 lines UI theory. |
| taste-skill | 9 | KEEP | Alias for design-taste-frontend. |
| gpt-taste | 7 | KEEP | Editorial/GSAP taste. |
| shadcn-ui | 6 | KEEP | shadcn/ui reference. |
| stitch-design-taste | 8 | KEEP | Stitch DESIGN.md gen. |
| stitch-loop | 5 | KEEP | Design-to-code iteration. |
| design-taste-frontend-v1 | 6 | KEEP | v1 legacy — usar design-taste-frontend (v2) para nuevo trabajo. |
| sleek-design-mobile-apps | 6 | REMOVED | VantaDB no es mobile app. |

### Animation / Motion / 3D

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| gsap-core | 9 | KEEP | Official GSAP core. |
| gsap-scrolltrigger | 9 | KEEP | Scroll-linked. |
| gsap-timeline | 8 | KEEP | Sequencing. |
| gsap-plugins | 8 | KEEP | SplitText, Draggable. |
| gsap-react | 8 | KEEP | useGSAP hook. |
| gsap-frameworks | 7 | REMOVED | Vue/Svelte — VantaDB usa React + gsap-react. |
| gsap-performance | 7 | KEEP | 60fps optimization. |
| gsap-utils | 7 | KEEP | Utility methods. |
| motion | 9 | KEEP | motion.dev v12. |
| animejs | 8 | REMOVED | Redundante con motion + GSAP. |
| design-motion-principles | 8 | KEEP | Create + Audit modes. |
| emil-design-eng | 9 | KEEP | UI polish philosophy. |
| emilkowalski-motion | 7 | KEEP | Micro-interaction follow-up. |
| interaction-design | 8 | KEEP | 1097 lines. |
| threejs-fundamentals | 8 | REMOVED | No 3D rendering in VantaDB. |
| threejs-geometry | 8 | REMOVED | No 3D rendering in VantaDB. |
| threejs-materials | 8 | REMOVED | No 3D rendering in VantaDB. |
| threejs-animation | 8 | REMOVED | No 3D rendering in VantaDB. |
| threejs-interaction | 9 | REMOVED | No 3D rendering in VantaDB. |
| threejs-shaders | 9 | REMOVED | No 3D rendering in VantaDB. |
| algorithmic-art | 8 | REMOVED | p5.js generative — no arte generativo. |
| shader-dev | 5 | REMOVED | GLSL reference — no shaders. |
| video-hyperframes | 6 | REMOVED | Hyperframes bridge — no video. |
| vfx-text-cursor | 6 | REMOVED | VFX text reveal — no video. |
| remotion | 5 | REMOVED | Stub catálogo — usar `remotion-best-practices` (canónica). |
| remotion-best-practices | 6 | KEEP | **Canónica video (2026-08-19):** reglas reales de API Remotion (`rules/timing.md`, `transitions.md`, `text-animations.md`, `gifs.md`). Usada en `web/remotion/` para los GIFs del README. |

### Deck / Slide / Video Templates

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| frontend-slides | 3 | REMOVED | Redundant. |
| hyperframes | 9 | REMOVED | No video production in VantaDB. |
| hyperframes-animation | 8 | REMOVED | No video production in VantaDB. |
| deck-open-slide-canvas | 7 | REMOVED | No deck/slide production. |
| deck-swiss-international | 7 | REMOVED | No deck/slide production. |
| deck-guizang-editorial | 7 | REMOVED | No deck/slide production. |
| field-notes-editorial-template | 7 | REMOVED | No deck/slide production. |
| digits-fintech-swiss-template | 7 | REMOVED | No deck/slide production. |
| editorial-burgundy-principles-template | 7 | REMOVED | No deck/slide production. |
| html-ppt-retro-quarterly-review | 7 | REMOVED | No deck/slide production. |
| after-hours-editorial-template | 7 | REMOVED | No deck/slide production. |
| swiss-creative-mode-template | 7 | REMOVED | No deck/slide production. |
| swiss-user-research-video-template | 7 | REMOVED | No deck/slide production. |
| weread-year-in-review-video-template | 7 | REMOVED | No deck/slide production. |
| ppt-keynote | 7 | REMOVED | No deck/slide production. |
| pptx | 5 | REMOVED | No deck/slide production. |
| pptx-html-fidelity-audit | 8 | REMOVED | No deck/slide production. |
| frame-data-chart-nyt | 6 | REMOVED | No video production. |
| frame-flowchart-sticky | 6 | REMOVED | No video production. |
| frame-glitch-title | 6 | REMOVED | No video production. |
| frame-light-leak-cinema | 6 | REMOVED | No video production. |
| frame-liquid-bg-hero | 6 | REMOVED | No video production. |
| frame-logo-outro | 6 | REMOVED | No video production. |
| frame-macos-notification | 6 | REMOVED | No video production. |
| 8-bit-orbit-video-template | 6 | REMOVED | No video production. |

### Image / Video / AI Media

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| brandkit | 9 | KEEP | Brand identity gen. Useful for web visuals. |
| imagegen-frontend-web | 9 | KEEP | Premium web images. Useful for web visuals. |
| screenshots-marketing | 6 | KEEP | Marketing screenshots. Useful for docs/landing. |
| imagegen-frontend-mobile | 8 | REMOVED | Mobile screen images — no mobile app. |
| canvas-design | 7 | REMOVED | Artistic visual output — no need. |
| ecommerce-image-workflow | 7 | REMOVED | Ecommerce — no ecommerce. |
| poster-hero | 6 | REMOVED | Vertical poster — no need. |
| social-reddit-card | 6 | REMOVED | Social card — no social media. |
| social-spotify-card | 6 | REMOVED | Social card — no social media. |
| social-x-post-card | 6 | REMOVED | Social card — no social media. |
| image-edit | 7 | REMOVED | RunComfy smart router — no image editing. |

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
| supabase-postgres-best-practices | 7 | KEEP | Postgres optimization. |
| database-design | 7 | KEEP | DB selection. |
| database-schema-designer | 7 | KEEP | Schema design. |
| prisma-expert | 7 | KEEP | Prisma reference. |
| api-design-principles | 7 | KEEP | REST + GraphQL. |
| ai-sdk | 8 | KEEP | Vercel AI SDK. |
| ai-seo | 8 | KEEP | AI search optimization. |
| seo-audit | 7 | KEEP | Full SEO audit. |
| audit-website | 7 | KEEP | 230+ rules. |
| vercel-optimize | 8 | KEEP | Production optimization. |
| react-email | 6 | REMOVED | No transactional emails. |
| react-native-best-practices | 7 | REMOVED | No mobile app. |
| programmatic-seo | 7 | REMOVED | No pages-at-scale. |
| seo | 6 | REMOVED | Redundant con ai-seo + seo-audit. |
| roier-seo | 6 | REMOVED | Redundant con seo-audit + audit-website. |
| browser-use | 6 | REMOVED | Redundant con Playwright MCP. |
| prisma | 7 | REMOVED | Redundant con prisma-expert. |
| typescript-expert | 7 | REMOVED | Dangling — no SKILL.md on disk. |

### Engineering Lifecycle

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| spec-driven-development | 8 | KEEP | Write spec/PRD before code. |
| interview-me | 7 | KEEP | Extract real intent from vague asks. |
| idea-refine | 7 | KEEP | Vague concept → concrete proposal. |
| planning-and-task-breakdown | 8 | KEEP | Spec → small verifiable tasks. |
| incremental-implementation | 8 | KEEP | Thin vertical slices. |
| test-driven-development | 8 | KEEP | Red-Green-Refactor. |
| context-engineering | 7 | KEEP | Pack relevant context for agents. |
| source-driven-development | 8 | KEEP | Verify docs before implementing. |
| doubt-driven-development | 8 | KEEP | Adversarial review for high-stakes. |
| frontend-ui-engineering | 7 | KEEP | Production UI in web/. |
| api-and-interface-design | 8 | KEEP | Stable public interfaces. |
| debugging-and-error-recovery | — | DEPRECATED | Stub (EVAL-03, 2026-08-10) — unificada con `systematic-debugging`. NO cargar. |
| browser-testing-with-devtools | 7 | KEEP | Real browser via CDP. |
| code-review-and-quality | 9 | KEEP | Multi-axis review before merge. |
| code-simplification | 7 | KEEP | Reduce complexity. |
| security-and-hardening | 8 | KEEP | Input/auth/data hardening. |
| performance-optimization | 8 | KEEP | Vitals, bottlenecks. |
| git-workflow-and-versioning | 8 | KEEP | Atomic commits, trunk-based. |
| ci-cd-and-automation | 7 | KEEP | CI/CD pipelines, Shift Left. |
| shipping-and-launch | 7 | KEEP | Pre-deploy checklists. |
| documentation-and-adrs | 7 | KEEP | ADRs, API docs, feature docs. |
| deprecation-and-migration | 7 | KEEP | Sunset old systems. |
| observability-and-instrumentation | 7 | KEEP | Logging, metrics, tracing. |
| using-agent-skills | 8 | KEEP | Meta-skill for skill discovery. |
| design-audit-orchestrator | 7 | KEEP | Audit-first design review pipeline. |
| vantadb-full-review | 10 | REMOVED | Eliminado (2026-08-10) — reemplazado por `unified-review --profile vantadb`. Directorio físico eliminado 2026-08-23 (campaña avance-canónico). Referencias históricas en `docs/plans/archive/`, `docs/reviews/archive/`. |
| backlog-executor | — | SUPERSEDED | Reemplazada por `campaign-executor` (unificado backlog-executor + task-executor). NO cargar. |

### Content / Writing

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| writing-guidelines | 6 | KEEP | Writing compliance. Useful for docs. |
| release-notes-one-pager | 6 | KEEP | Release notes template. |
| copywriting | 7 | REMOVED | No marketing copy work. |
| marketing-psychology | 7 | REMOVED | No marketing activities. |
| marketing-ideas | 7 | REMOVED | No marketing activities. |
| social-content | 7 | REMOVED | No social media management. |

### Research / Strategy

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| ux-heuristics | 8 | KEEP | Nielsen/Krug framework. Útil para web/. |
| research-decision-room | 8 | REMOVED | No user research. |
| ux-strategy | 7 | REMOVED | No UX research program. |
| prototyping-testing | 7 | REMOVED | No prototyping. |
| creative-director | 6 | REMOVED | No creative campaigns. |
| design-ops | 7 | REMOVED | No design critiques. |
| design-research | 7 | REMOVED | No UX research. |

### Utility / Tool

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| vanta-design-orchestrator | 9 | KEEP | Master orchestrator. |
| full-output-enforcement | 8 | KEEP | Anti-truncation. |
| systematic-debugging | 8 | KEEP | Root-cause methodology. |
| writing-plans | 8 | KEEP | Implementation plans. |
| brainstorming | 8 | KEEP | Idea-to-spec dialogue. |
| visual-review | 8 | KEEP | Automated QA pipeline. |
| pr-feedback-quality-gate | 7 | KEEP | PR workflow. |
| find-skills | 8 | REMOVED | Meta — no skill creation activa. |
| skill-creator | 8 | REMOVED | Meta — no skill creation activa. |
| agent-browser | 8 | REMOVED | Redundant con Playwright MCP. |
| professional-github-repo | 7 | REMOVED | Meta — no administración de repo. |
| export-download-debugging | 7 | REMOVED | No export/download workflows. |
| design-brief | 8 | REMOVED | No diseño desde briefs formales. |
| design-md | 4 | REMOVED | No DESIGN.md management. |
| designer-toolkit | 7 | REMOVED | No case studies. |
| pptx-html-fidelity-audit | 8 | REMOVED | No PPTX. |
| docx | 4 | REMOVED | No Word docs. |
| pdf | 5 | REMOVED | No PDF generation. |
| speech | 5 | REMOVED | No TTS/audio. |
| hand-drawn-diagrams | 6 | REMOVED | No diagramas. |
| d3-visualization | 5 | REMOVED | No D3 charts — usar @observablehq/plot. |
| data-report | 5 | REMOVED | No data report generation. |
| understand | 7 | KEEP | Full pipeline: scan → graph → analyze. |
| understand-chat | 7 | KEEP | Contextual chat sobre codebase. |
| understand-dashboard | 6 | KEEP | Visor web interactivo del grafo. |
| understand-diff | 7 | KEEP | Analiza git diff contra grafo. |
| understand-domain | 7 | KEEP | Extrae dominio de negocio del grafo. |
| understand-explain | 7 | KEEP | Análisis narrativo de archivo/módulo. |
| understand-knowledge | 6 | KEEP | Analiza wikis Markdown → grafo. |
| understand-onboard | 6 | KEEP | Guía interactiva de onboarding. |
| coordinated-web-search | 7 | KEEP | Orchestrates MetaSearchMCP + Argus for coordinated web search and validation across multiple providers. |
| web-research | 6 | KEEP | Orchestrates MCP search servers for comprehensive web research and content extraction. |

---

### Accessibility / Inclusive Design (63)

Batch 2026-08-19 (5 `a11y-*` pipeline WCAG 2.2 + 58 `incl-*` inclusive design). Para todo trabajo de accesibilidad: usar `a11y-*` para el pipeline automatizado/manual WCAG, `incl-*` para diseño inclusivo por plugin.

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `a11y-accessibility-audit` | 8 | KEEP | Auditoría a11y completa de sitio/producto (WCAG 2.2, WCAG-EM). Usar para «audita mi sitio por accesibilidad» o informes de conformidad. |
| `a11y-accessibility-diff` | 8 | KEEP | Diff de violaciones a11y contra baseline; detecta regresiones por PR o rama. Usar como gate de CI o «¿mi cambio rompió accesibilidad?». |
| `a11y-accessibility-fix` | 8 | KEEP | Remediación de violaciones WCAG 2.2 (baseline → editar → verificar). Usar para «arregla los issues de a11y de X». |
| `a11y-accessibility-inspect` | 8 | KEEP | Tier manual por página: teclado, foco, screen-reader, zoom 200%. Usar para operabilidad real, no solo lint-clean. |
| `a11y-accessibility-scan` | 8 | KEEP | Tier automatizado: rule engine WCAG 2.2 sobre DOM vivo. Usar para «¿esta página es accesible?» o verificar un cambio de UI. |
| `incl-accessibility-decisions-accessibility-debt-tracking` | 7 | KEEP | Gestiona deuda de accesibilidad diferida y backlog de remediación. |
| `incl-accessibility-decisions-accessibility-testing-strategy` | 7 | KEEP | Plan de testing a11y: qué, cómo y quién debe testear. |
| `incl-accessibility-decisions-compliance-mapping` | 7 | KEEP | Mapea decisiones a estándares y requisitos legales (WCAG, VPAT, ADA, EAA). |
| `incl-accessibility-decisions-decision-documentation` | 7 | KEEP | Documenta decisiones de accesibilidad y su razonamiento (ADR). |
| `incl-accessibility-decisions-document` | 7 | KEEP | Captura decisiones y tradeoffs de accesibilidad por feature. |
| `incl-accessibility-decisions-handoff` | 7 | KEEP | Genera handoff de requisitos de accesibilidad para ingeniería. |
| `incl-accessibility-decisions-review` | 7 | KEEP | Expone supuestos de accesibilidad no documentados en producto existente. |
| `incl-accessibility-decisions-stakeholder-communication` | 7 | KEEP | Comunica valor y requisitos de a11y a stakeholders no técnicos. |
| `incl-accessibility-decisions-tradeoff-analysis` | 7 | KEEP | Analiza tradeoffs cuando la accesibilidad compite con otros requisitos. |
| `incl-accessible-content-alt-text-design` | 7 | KEEP | Redacta alt text significativo para imágenes, charts y diagramas. |
| `incl-accessible-content-form-labelling` | 7 | KEEP | Diseña labels, instrucciones y agrupación de formularios accesibles. |
| `incl-accessible-content-heading-structure` | 7 | KEEP | Diseña jerarquías de headings y estructura semántica de contenido. |
| `incl-accessible-content-link-text-design` | 7 | KEEP | Escribe texto de enlace comprensible fuera de contexto. |
| `incl-accessible-content-multimedia-accessibility` | 7 | KEEP | Diseña video/audio accesibles con captions, transcripts y audio descriptions. |
| `incl-accessible-content-readable-content` | 7 | KEEP | Escribe contenido legible para diversas habilidades de lectura. |
| `incl-accessible-content-review` | 7 | KEEP | Revisa contenido por accesibilidad cognitiva y de alfabetización. |
| `incl-accessible-content-rewrite` | 7 | KEEP | Reescribe contenido en lenguaje claro preservando el significado. |
| `incl-accessible-content-structure` | 7 | KEEP | Estructura contenido para lectores de pantalla y tecnología de asistencia. |
| `incl-accessible-content-table-accessibility` | 7 | KEEP | Diseña tablas de datos que funcionen con screen readers. |
| `incl-adaptive-interfaces-colour-independence` | 7 | KEEP | Diseña sin que el color sea el único canal de información. |
| `incl-adaptive-interfaces-flexible-typography` | 7 | KEEP | Diseña tipografía adaptable a escala, spacing y preferencias de fuente. |
| `incl-adaptive-interfaces-information-density` | 7 | KEEP | Diseña densidad de información ajustable a necesidades cognitivas. |
| `incl-adaptive-interfaces-preference-audit` | 7 | KEEP | Audita respeto de preferencias de usuario (motion, contraste, color, escala). |
| `incl-adaptive-interfaces-responsive-accessibility` | 7 | KEEP | Diseña layouts responsive accesibles en tamaños, zoom y orientaciones. |
| `incl-adaptive-interfaces-responsive-review` | 7 | KEEP | Revisa layout responsive por accesibilidad en dispositivos y niveles de zoom. |
| `incl-adaptive-interfaces-simplified-views` | 7 | KEEP | Diseña vistas simplificadas con menos ruido visual y opciones. |
| `incl-adaptive-interfaces-specify` | 7 | KEEP | Especifica comportamiento adaptativo de la interfaz. |
| `incl-adaptive-interfaces-user-preference-respect` | 7 | KEEP | Diseña interfaces que responden a preferencias del sistema (dark mode, reduced motion). |
| `incl-cognitive-accessibility-adaptive-personalisation` | 7 | KEEP | Diseña sistemas de ayuda y soporte para discapacidad cognitiva. |
| `incl-cognitive-accessibility-assess-load` | 7 | KEEP | Evalúa carga cognitiva de procesos multi-paso y genera load map. |
| `incl-cognitive-accessibility-cognitive-load-assessment` | 7 | KEEP | Evalúa y reduce carga cognitiva en interfaces, flows y contenido. |
| `incl-cognitive-accessibility-contextual-help-design` | 7 | KEEP | Diseña ayuda contextual (tooltips, onboarding, FAQs) para discapacidad cognitiva. |
| `incl-cognitive-accessibility-error-prevention-recovery` | 7 | KEEP | Diseña prevención de errores, mensajes y flujos de recuperación. |
| `incl-cognitive-accessibility-focus-attention-design` | 7 | KEEP | Diseña interfaces que sostienen el foco y reducen distracciones. |
| `incl-cognitive-accessibility-memory-load-reduction` | 7 | KEEP | Minimiza demandas de memoria de trabajo en flows multi-paso. |
| `incl-cognitive-accessibility-plain-language-design` | 7 | KEEP | Escribe contenido de interfaz en lenguaje claro. |
| `incl-cognitive-accessibility-review` | 7 | KEEP | Ejecuta revisión completa de accesibilidad cognitiva de un flow o pantalla. |
| `incl-cognitive-accessibility-simplify` | 7 | KEEP | Simplifica contenido e interacciones para reducir carga cognitiva. |
| `incl-cognitive-accessibility-wayfinding-navigation` | 7 | KEEP | Diseña navegación y arquitectura de información para accesibilidad cognitiva. |
| `incl-inclusive-interaction-audit` | 7 | KEEP | Audita soporte de interacción multi-modal en interfaz existente. |
| `incl-inclusive-interaction-design-flow` | 7 | KEEP | Diseña flujos de interacción inclusivos desde el inicio (input/output). |
| `incl-inclusive-interaction-feedback-and-status` | 7 | KEEP | Diseña feedback y estado multi-sensorial: visual, auditivo y háptico. |
| `incl-inclusive-interaction-gesture-alternatives` | 7 | KEEP | Diseña alternativas a gestos (swipe, pinch, shake, tilt). |
| `incl-inclusive-interaction-keyboard-navigation` | 7 | KEEP | Diseña navegación por teclado y gestión de foco. |
| `incl-inclusive-interaction-keyboard-review` | 7 | KEEP | Revisa navegación por teclado y gestión de foco en interfaz existente. |
| `incl-inclusive-interaction-motion-sensitivity` | 7 | KEEP | Diseña para sensibilidad al movimiento, vértigo y convulsiones. |
| `incl-inclusive-interaction-multi-modal-input` | 7 | KEEP | Diseña múltiples métodos de entrada seleccionables por habilidad. |
| `incl-inclusive-interaction-touch-target-design` | 7 | KEEP | Diseña touch targets y pointer interactions para dificultad motora. |
| `incl-inclusive-interaction-voice-interaction` | 7 | KEEP | Diseña voice interactions para patrones diversos de habla y acentos. |
| `incl-inclusive-personas-ability-spectrum-mapping` | 7 | KEEP | Mapea el producto al espectro completo de habilidad humana. |
| `incl-inclusive-personas-assistive-technology-scenarios` | 7 | KEEP | Escribe escenarios de uso con tecnología de asistencia. |
| `incl-inclusive-personas-audit-stories` | 7 | KEEP | Audita user stories por inclusión de discapacidad. |
| `incl-inclusive-personas-disability-inclusive-personas` | 7 | KEEP | Crea personas con discapacidad como dimensión natural de diversidad. |
| `incl-inclusive-personas-edge-case-identification` | 7 | KEEP | Identifica edge cases que afectan a usuarios con discapacidad. |
| `incl-inclusive-personas-generate` | 7 | KEEP | Genera un set de personas diverso e inclusivo. |
| `incl-inclusive-personas-inclusive-user-stories` | 7 | KEEP | Escribe user stories que consideran discapacidad desde el inicio. |
| `incl-inclusive-personas-scenario-map` | 7 | KEEP | Mapea escenarios inclusivos a través de espectros de habilidad. |
| `incl-inclusive-personas-situational-impairment-mapping` | 7 | KEEP | Mapea deterioros situacionales (entorno, contexto, multitasking). |

### Adiciones 2026-08-19 — 17 skills (batch UX/Design)

#### Frontend / UI Design

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `frontend-designer` | 8 | KEEP | Sistemas de diseño UI producción: CSS architecture, tokens, responsive, a11y, motion. |
| `responsive-craft` | 8 | KEEP | Responsive mobile-first y patrones complejos (sticky, scroll, tablas, dashboards). 3 modos: audit/build/preview. |
| `wireframer` | 7 | KEEP | Wireframes low-fi clickeables estilo sketch (React/Vue/HTML). |
| `ux-flow-designer` | 7 | KEEP | User flows, wireframes HTML y diagramas Mermaid; puente PRD→diseño visual. |
| `figma-implement-design` | 6 | REMOVED | Requiere Figma MCP no configurado (ya removida en Phase 1 por la misma razón). |

#### Animation / Motion / 3D

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `motion-design` | 7 | KEEP | Principios de motion (timing, easing, choreografía, Disney) para UI; CSS/Framer/GSAP/Lottie. Complementa design-motion-principles. |

#### Image / Video / AI Media

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `icon-generator` | 6 | REMOVED | Iconos app iOS/Android — VantaDB no es mobile app. |
| `slack-gif-creator` | 5 | REMOVED | GIFs para Slack — VantaDB no produce GIFs/video. |

#### Design Systems / Tokens

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `brand-guidelines` | 5 | REMOVED | Marca Anthropic (colores/tipografía) — no aplica: VantaDB tiene su propia marca. |

#### Content / Writing

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `doc-coauthoring` | 5 | KEEP | Workflow de co-autoría de docs (specs, propuestas, RFCs) en 3 etapas. |
| `internal-comms` | 4 | REMOVED | Comms internas formato Anthropic (3P, newsletters) — no aplica. |
| `academy-guide` | 4 | REMOVED | Recomienda cursos de Claude Academy — no aplica. |

#### Utility / Tool

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `webapp-testing` | 7 | KEEP | Testing de webapps locales con Playwright: verificar UI, screenshots, logs. |
| `mcp-builder` | 6 | KEEP | Guía para crear servidores MCP (FastMCP / MCP SDK TS). Útil para vantadb-mcp. |
| `xlsx` | 6 | KEEP | Crear/editar/limpiar .xlsx/.xlsm/.csv/.tsv (openpyxl, pandas). |
| `discernment-nudge` | 6 | KEEP | Nudge post-respuesta: 2-3 preguntas para verificar hechos/razonamiento/contexto. |
| `claude-api` | 7 | REMOVED | Referencia API Claude/Anthropic (alta calidad, 553 líneas) — no aplica: uso solo en `.opencode/references/`, no en producto. |

### Pipeline / VantaDB Core (7)

| Skill | Rating | Keep | Notes |
|-------|:------:|:----:|-------|
| `campaign-executor` | 9 | KEEP | Núcleo del task system; unifica backlog-executor (SUPERSEDED) + task-executor. Entry point `/pipeline plan\|task\|run`, states PLAN/ACT/VERIFY, budget, SARL. |
| `unified-review` | 9 | KEEP | Gate universal de review/certify; reemplaza vantadb-full-review, vantadb-certify y vantadb-audit. Modos quick/certify/review/full, fan-out paralelo, perfil `vantadb`. |
| `vantadb` | 8 | KEEP | Guía experta del producto: put/get/search/list, hybrid search (BM25 + HNSW + RRF), Python SDK, integraciones LangChain/LlamaIndex, benchmarks. |
| `vantadb-mcp` | 8 | KEEP | Referencia del MCP server propio (15 tools, 2 resources, 4 prompts); uso de VantaDB como memoria desde agentes OpenCode/Claude/Cursor. |
| `review-deep` | 7 | KEEP | Loop de revisión profunda módulo por módulo (análisis → web research → competidores → triage → Backlog). Complementa unified-review (one-shot). |
| `progreso` | 7 | KEEP | Migración Backlog → docs/progreso + reconciliación de memorias + sync de reportes. MUST al inicio y cierre de sesión (AGENTS.md). |
| `playwright-cli` | 7 | KEEP | Automatización de browser por CLI (snapshots, forms, screenshots, sessions, DevTools). Canónico para testing de web/; reemplaza a agent-browser/browser-use (removidas). |

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



### For SEO
```
1. ai-seo        (AI search optimization)
2. seo-audit     (technical audit)
3. audit-website (230+ rules)
```

### For Accessibility
```
1. a11y-accessibility-scan  (automated check of a live page)
2. a11y-accessibility-audit (full WCAG 2.2 site audit)
3. a11y-accessibility-fix   (remediate violations)
4. incl-inclusive-interaction-keyboard-review (manual keyboard/focus pass)
```
Ejemplos: «audita la accesibilidad de la landing» → a11y-accessibility-audit; «¿mi cambio rompió la accesibilidad del dashboard?» → a11y-accessibility-diff; «revisa la navegación por teclado antes del release» → incl-inclusive-interaction-keyboard-review.

### For Wireframing / UX Flow
```
1. wireframer      (low-fi clickable prototype)
2. ux-flow-designer (flows + screens + use cases in Mermaid/HTML)
3. frontend-design  (then implement)
```
Ejemplos: «hazme un wireframe de la dashboard» → wireframer; «dibujá el flujo de onboarding del usuario» → ux-flow-designer.

### For Pipeline / VantaDB Core
```
1. campaign-executor (task system — /pipeline plan|task|run)
2. unified-review    (certify/review/audit gates)
3. progreso          (backlog → progreso at session start/end)
4. vantadb-mcp       (MCP server usage)
```
Ejemplos: «/pipeline task DRV-012» → campaign-executor; «/review certify --profile vantadb» → unified-review; «¿cómo uso VantaDB desde MCP?» → vantadb-mcp.

---

## What Was Removed

The following skills were removed during cleanup (158 total):

### Phase 1 — Original Cleanup (75 skills)

**Duplicates (14):** brutalist-skill, gpt-tasteskill, image-to-code-skill, minimalist-skill, redesign-skill, stitch-skill, soft-skill, taste-skill-v1, threejs (shallow), pptx-generator, slides, nanobanana-ppt, output-skill, react-best-practices

**Shallow Stubs (48):** all 8 fal stubs (fal-3d, kling-o3, lip-sync, realtime, restore, train, tryon, video-edit), all 5 venice stubs, pixelbin-media, replicate, sora, imagen, artifacts-builder, apple-hig, brand-guidelines, frontend-dev, frontend-skill, frontend-slides, design-consultation, design-review, enhance-prompt, image-enhancer, paywall-upgrade-cro, ui-skills, ad-creative, ai-music-album, competitive-ads-extractor, domain-name-brainstormer, flutter-animating-apps, gif-sticker-maker, hatch-pet, resume-modern, video-downloader, wpds, slack-gif-creator, youtube-clipper, swiftui-design, screenshot, full-page-screenshot, minimax-docx, minimax-pdf, doc (OpenAI), imagegen (bare)

**Empty dirs (9):** cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer

**Conditional removed (11):** fal-generate, fal-image-edit, fal-upscale, fal-vision, figma-code-connect-components, figma-create-design-system-rules, figma-create-new-file, figma-generate-design, figma-generate-library, figma-implement-design, figma-use (no MCP/no API key configured for fal.ai or Figma)

**Cross-location (1):** interaction-design (from .claude/skills/, redundant with impeccable)

### Phase 2 — Irrelevant to VantaDB (85 skills, 2026-07-10)

**Video/Deck/Slide Production (28):** hyperframes, hyperframes-animation, deck-open-slide-canvas, deck-swiss-international, deck-guizang-editorial, field-notes-editorial-template, digits-fintech-swiss-template, editorial-burgundy-principles-template, html-ppt-retro-quarterly-review, after-hours-editorial-template, swiss-creative-mode-template, swiss-user-research-video-template, weread-year-in-review-video-template, ppt-keynote, pptx, pptx-html-fidelity-audit, frame-data-chart-nyt, frame-flowchart-sticky, frame-glitch-title, frame-light-leak-cinema, frame-liquid-bg-hero, frame-logo-outro, frame-macos-notification, 8-bit-orbit-video-template, video-hyperframes, vfx-text-cursor, remotion, speech

**Video canónica (1):** remotion-best-practices — reglas oficiales para `web/remotion/` (GIFs README).

**3D/Shaders/Art (8):** threejs-fundamentals, threejs-geometry, threejs-materials, threejs-animation, threejs-interaction, threejs-shaders, shader-dev, algorithmic-art

**Mobile (4):** sleek-design-mobile-apps, imagegen-frontend-mobile, login-flow, react-native-best-practices

**Social Media/Cards (7):** card-twitter, card-xiaohongshu, poster-hero, social-reddit-card, social-spotify-card, social-x-post-card, social-content

**Media/Image (4):** ecommerce-image-workflow, image-edit, mockup-device-3d, canvas-design

**Marketing/Copywriting (5):** copywriting, marketing-psychology, marketing-ideas, creative-director, research-decision-room

**Redundant Tools (9):** agent-browser, browser-use, roier-seo, seo, find-skills, skill-creator, professional-github-repo, gsap-frameworks, animejs

**UX/Design Process (6):** ux-strategy, prototyping-testing, design-ops, design-research, design-brief, design-md

**Documents/Print (5):** docx, pdf, article-magazine, faq-page, doc-kami-parchment

**Other (6):** react-email, programmatic-seo, prisma, designer-toolkit, export-download-debugging, data-report, d3-visualization, hand-drawn-diagrams

---

## Source Locations

All skills are now consolidated in the project: **193 skills en `.opencode/skills/`** (git submodule → `configOpencode` private repo); total 193 (audited 2026-09-01). Skills not relevant to VantaDB live in `~/.agents/skills/` (160 dirs global) and `~/.claude/skills/` (33 dirs).
The global locations still exist but are secondary — prefer the project-local copy. Regla de resolución (AGENTS.md): `.opencode/skills/` (submodule) → global, usar la primera que exista. **Nota:** `.agents/` fue eliminado 2026-09-01.

### Removidas en batch 2026-08-19 (7)

`figma-implement-design` (sin Figma MCP configurado), `icon-generator` (assets mobile — VantaDB no es app móvil), `slack-gif-creator` (GIFs Slack), `brand-guidelines` (marca Anthropic), `academy-guide` (cursos Claude Academy), `claude-api` (referencia API Claude — no aplica al producto), `internal-comms` (comms internas Anthropic). **Nota:** Estas existían en `.agents/skills/` (eliminado 2026-09-01); se documentan para registro histórico.
