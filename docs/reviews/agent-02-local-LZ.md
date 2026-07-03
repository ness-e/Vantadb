# Agent-02: Local Skills Analysis (L-Z)
## VantaDB Project — Full Detailed Report

---

## METHODOLOGY
- Source: `.agents/skills/` in project root
- Scope: All skills with names starting with letters L through Z
- Analysis: Read first 30 lines of each SKILL.md for purpose, quality indicators, triggers, and depth
- Total skills analyzed: ~75

---

## 1. LOGIN-FLOW
- **Path**: `.agents/skills/login-flow/`
- **Description**: Mobile login and authentication flow screens (phone/SMS, password, social SSO)
- **Category**: Mobile
- **Rating**: 6/10
- **Depth**: 48 lines — focused, clear workflow with states (default, loading, error)
- **Assessment**: Useful for mobile prototype work. Well-scoped. KEEP.
- **Notes**: Narrow scope but well-defined. Good for auth-heavy mobile projects.

---

## 2. MINIMALIST-SKILL / MINIMALIST-UI
- **Paths**: `.agents/skills/minimalist-skill/` and `.agents/skills/minimalist-ui/`
- **Description**: Clean editorial-style interfaces — warm monochrome palette, typographic contrast, flat bento grids, muted pastels
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: Both are substantial (110 and 98 lines). Detailed anti-pattern lists, typography rules.
- **Assessment**: REDUNDANT — these are nearly identical skills with the same name/description. The `minimalist-skill` folder is an alias/dupe of `minimalist-ui`. Keep one.
- **Notes**: MERGE or REMOVE one. Both originate from the same taste-skill lineage. Keep `minimalist-ui` (more complete at 98 lines of protocol).

---

## 3. MINIMAX-DOCX
- **Path**: `.agents/skills/minimax-docx/`
- **Description**: Professional DOCX creation/editing using OpenXML SDK
- **Category**: Utility/Tool
- **Rating**: 4/10
- **Depth**: 43 lines — thin wrapper/catalogue entry with no workflow detail
- **Assessment**: Minimal depth. Just a stub that says "use MiniMax skill." Lacks actionable instructions. REMOVE if not actively used.
- **Notes**: Redundant with `docx` skill (also for DOCX).

---

## 4. MINIMAX-PDF
- **Path**: `.agents/skills/minimax-pdf/`
- **Description**: Generate, fill, and reformat PDFs with token-based design system + 15 cover styles
- **Category**: Utility/Tool
- **Rating**: 4/10
- **Depth**: 43 lines — thin catalogue entry
- **Assessment**: Same pattern as minimax-docx — shallow stub. Redundant with `pdf` skill. REMOVE.
- **Notes**: If PDF capability is needed, keep `pdf` (Anthropic official) instead.

---

## 5. MOCKUP-DEVICE-3D
- **Path**: `.agents/skills/mockup-device-3d/`
- **Description**: Static iPhone and MacBook 3D-style showcase with real HTML embedded on screens
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 63 lines — well-structured HTML-anything template
- **Assessment**: High-quality visual template for product mockups. Unique purpose. KEEP.
- **Notes**: Great for product landing pages and portfolio showcases.

---

## 6. NANOBANANA-PPT
- **Path**: `.agents/skills/nanobanana-ppt/`
- **Description**: AI-powered PPT generation with document analysis and styled images
- **Category**: Deck/Slide/Video Template
- **Rating**: 4/10
- **Depth**: 43 lines — catalogue stub
- **Assessment**: Shallow, no actionable detail. REMOVE.
- **Notes**: Redundant with `pptx` and `pptx-generator` and `slides` skills.

---

## 7. OUTPUT-SKILL (full-output-enforcement)
- **Path**: `.agents/skills/output-skill/`
- **Description**: Enforces complete code generation, bans placeholder patterns, handles token-limit splits
- **Category**: AI Prompt/Agent
- **Rating**: 8/10
- **Depth**: 69 lines — clear, practical, production-critical content
- **Assessment**: High-value meta-skill for agent behavior. KEEP.
- **Notes**: Essential companion for any task requiring full code output. Unique purpose.

---

## 8. PAYWALL-UPGRADE-CRO
- **Path**: `.agents/skills/paywall-upgrade-cro/`
- **Description**: Design/optimize upgrade screens, paywalls, upsell modals for SaaS
- **Category**: Frontend/UI Design
- **Rating**: 4/10
- **Depth**: 43 lines — stub only
- **Assessment**: Shallow catalogue entry. Not actionable. REMOVE unless expanded.
- **Notes**: Narrow B2B SaaS focus; likely unused in VantaDB context.

---

## 9. PDF
- **Path**: `.agents/skills/pdf/`
- **Description**: Extract text, create PDFs, handle forms (Anthropic official)
- **Category**: Utility/Tool
- **Rating**: 5/10
- **Depth**: 43 lines — official but brief
- **Assessment**: Official Anthropic skill — keep over minimax-pdf. KEEP.
- **Notes**: Authoritative source but needs expansion for real workflows.

---

## 10. PIXELBIN-MEDIA
- **Path**: `.agents/skills/pixelbin-media/`
- **Description**: Generate/edit images and videos with 85+ API portfolio via Pixelbin
- **Category**: Image/Video/AI Media
- **Rating**: 3/10
- **Depth**: 43 lines — bare stub
- **Assessment**: Thin vendor wrapper. Redundant with fal.ai skills, replicate, venice, recraft. REMOVE.
- **Notes**: Pixelbin not mentioned in project deps. Low relevance.

---

## 11. PLAN-DESIGN-REVIEW
- **Path**: `.agents/skills/plan-design-review/`
- **Description**: Senior Designer review — rates design dimensions 0-10, flags AI Slop
- **Category**: Design Systems/Tokens
- **Rating**: 7/10
- **Depth**: 42 lines — concise but well-defined purpose
- **Assessment**: Unique quality-gate skill. Complements `design-review`. KEEP.
- **Notes**: Valuable pre-merge check. Pairs with visual-critique and impeccable.

---

## 12. PLATFORM-DESIGN
- **Path**: `.agents/skills/platform-design/`
- **Description**: 300+ design rules from Apple HIG, Material Design 3, WCAG 2.2
- **Category**: Design Systems/Tokens
- **Rating**: 8/10
- **Depth**: 43 lines (catalogue-level) but claims 300+ rules internally
- **Assessment**: High potential value for cross-platform design consistency. KEEP.
- **Notes**: Would benefit from deeper insight into the actual rules.

---

## 13. POSTER-HERO
- **Path**: `.agents/skills/poster-hero/`
- **Description**: Vertical poster or Moments-style share image with strong visual impact
- **Category**: Image/Video/AI Media
- **Rating**: 6/10
- **Depth**: 42 lines — HTML-anything template
- **Assessment**: Good for social media graphics and marketing collateral. KEEP.
- **Notes**: Unique vertical poster output. Not duplicating other skills.

---

## 14. PPT-KEYNOTE
- **Path**: `.agents/skills/ppt-keynote/`
- **Description**: Apple Keynote-quality slides, one card per screen, keyboard navigation
- **Category**: Deck/Slide/Video Template
- **Rating**: 7/10
- **Depth**: 43 lines — HTML-anything template with example
- **Assessment**: High-quality standalone HTML deck template. KEEP.
- **Notes**: Different approach from pptx (slides are HTML-based, not PowerPoint files).

---

## 15. PPTX
- **Path**: `.agents/skills/pptx/`
- **Description**: Read, generate, adjust PowerPoint slides (Anthropic official)
- **Category**: Deck/Slide/Video Template
- **Rating**: 5/10
- **Depth**: 43 lines — official stub
- **Assessment**: Authoritative but shallow. KEEP as reference over pptx-generator.
- **Notes**: Redundant with `slides`, `pptx-generator`, and `nanobanana-ppt`.

---

## 16. PPTX-GENERATOR
- **Path**: `.agents/skills/pptx-generator/`
- **Description**: Create/edit PowerPoint with PptxGenJS (MiniMax)
- **Category**: Deck/Slide/Video Template
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Redundant with `pptx` and `slides`. Slightly different stack (PptxGenJS). REMOVE or MERGE.
- **Notes**: Minimal differentiation from `pptx` skill.

---

## 17. PPTX-HTML-FIDELITY-AUDIT
- **Path**: `.agents/skills/pptx-html-fidelity-audit/`
- **Description**: Audit python-pptx export vs source HTML deck for layout drift
- **Category**: Utility/Tool
- **Rating**: 8/10
- **Depth**: 254 lines — thorough, practical workflow with real troubleshooting
- **Assessment**: High-quality, unique niche skill. Very detailed. KEEP.
- **Notes**: Essential if project does HTML→PPTX conversions. Niche but excellent.

---

## 18. PR-FEEDBACK-QUALITY-GATE
- **Path**: `.agents/skills/pr-feedback-quality-gate/`
- **Description**: Track PR feedback, resolve review comments/conflicts, validate fixes
- **Category**: Utility/Tool
- **Rating**: 7/10
- **Depth**: 53 lines — clear workflow
- **Assessment**: Useful for agent workflow when handling PRs. KEEP.
- **Notes**: Agentic PR workflow. Valuable for collaboration patterns.

---

## 19. PROTOTYPING-TESTING
- **Path**: `.agents/skills/prototyping-testing/`
- **Description**: Validate designs via prototyping, usability testing, heuristic evaluation, A/B experiments
- **Category**: Research/Strategy
- **Rating**: 7/10
- **Depth**: 527 lines — very substantial content
- **Assessment**: Deep, comprehensive research skill. KEEP.
- **Notes**: Overlaps partially with ux-heuristics. More focused on research process.

---

## 20. REACT-BEST-PRACTICES
- **Path**: `.agents/skills/react-best-practices/`
- **Description**: React/Next.js performance optimization guidelines from Vercel (70+ rules)
- **Category**: Backend/Dev/SEO
- **Rating**: 9/10
- **Depth**: 151+ lines (very extensive, with AGENTS.md reference file)
- **Assessment**: EXCELLENT. Official Vercel content. Production-critical guidance. KEEP.
- **Notes**: One of the highest-value skills in the project. Keep and maintain.

---

## 21. REDESIGN-EXISTING-PROJECTS
- **Path**: `.agents/skills/redesign-existing-projects/`
- **Description**: Upgrades existing websites/apps to premium quality — audit then apply high-end design standards
- **Category**: Frontend/UI Design
- **Rating**: 8/10
- **Depth**: 182 lines — detailed audit checklist and fix sequences
- **Assessment**: Very practical redesign methodology. KEEP.
- **Notes**: Complements design-taste-frontend. More focused on existing codebases.

---

## 22. REDESIGN-SKILL
- **Path**: `.agents/skills/redesign-skill/`
- **Description**: Same as redesign-existing-projects — alias/duplicate
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 204 lines — slightly different layout, same core content
- **Assessment**: REDUNDANT with redesign-existing-projects. Same skill name/description. REMOVE.
- **Notes**: These two are duplicates. Keep `redesign-existing-projects` only.

---

## 23. REFERENCE-DESIGN-CONTRACT
- **Path**: `.agents/skills/reference-design-contract/`
- **Description**: Turn vague taste, screenshots, URLs into grounded DESIGN.md + implementation handoff
- **Category**: Design Systems/Tokens
- **Rating**: 8/10
- **Depth**: 130 lines — robust structured workflow
- **Assessment**: Unique and valuable design process skill. KEEP.
- **Notes**: Bridges the gap between reference materials and structured design specs.

---

## 24. RELEASE-NOTES-ONE-PAGER
- **Path**: `.agents/skills/release-notes-one-pager/`
- **Description**: Release notes single HTML page with standardized sections
- **Category**: Content/Writing
- **Rating**: 6/10
- **Depth**: 107 lines — well-defined template
- **Assessment**: Good for changelog/release communication. KEEP.
- **Notes**: Niche but well-designed. Could be useful for project releases.

---

## 25. REMOTION
- **Path**: `.agents/skills/remotion/`
- **Description**: Programmatic video creation with React (Remotion team official)
- **Category**: Animation/Motion/3D
- **Rating**: 5/10
- **Depth**: 43 lines — catalogue stub
- **Assessment**: Important technology but skill depth is minimal. KEEP for reference.
- **Notes**: Would benefit from expansion. Vanta-orchestrator references Remotion.

---

## 26. REPLICATE
- **Path**: `.agents/skills/replicate/`
- **Description**: Discover/compare/run AI models via Replicate API
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 42 lines — stub
- **Assessment**: Thin vendor API wrapper. REMOVE if not actively using Replicate.
- **Notes**: Redundant with fal.ai and venice API skills for image gen.

---

## 27. RESEARCH-DECISION-ROOM
- **Path**: `.agents/skills/research-decision-room/`
- **Description**: Turn messy research into evidence-backed decision artifact (HTML dashboard)
- **Category**: Research/Strategy
- **Rating**: 8/10
- **Depth**: 191 lines — comprehensive workflow
- **Assessment**: Unique, well-structured research synthesis skill. KEEP.
- **Notes**: High value for product decision-making. No duplicate.

---

## 28. RESUME-MODERN
- **Path**: `.agents/skills/resume-modern/`
- **Description**: Modern minimal resume, single A4 page, ready for print/PDF
- **Category**: Utility/Tool
- **Rating**: 5/10
- **Depth**: 43 lines — HTML-anything template
- **Assessment**: Niche personal use. Unlikely to be used for VantaDB project work. REMOVE.
- **Notes**: Low project relevance unless team needs resume generation.

---

## 29. SCREENSHOT
- **Path**: `.agents/skills/screenshot/`
- **Description**: Capture desktop, app windows, pixel regions (OpenAI official)
- **Category**: Utility/Tool
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Redundant with Playwright MCP and screenshots-marketing. REMOVE.
- **Notes**: Playwright-based solutions are more powerful for VantaDB's needs.

---

## 30. SCREENSHOTS-MARKETING
- **Path**: `.agents/skills/screenshots-marketing/`
- **Description**: Generate marketing screenshots with Playwright (hero shots, App Store)
- **Category**: Image/Video/AI Media
- **Rating**: 6/10
- **Depth**: 42 lines — stub but practical Playwright integration
- **Assessment**: More specific than screenshot skill. KEEP if doing marketing visuals.
- **Notes**: Integrates with Playwright MCP that VantaDB already has configured.

---

## 31. SHADCN-UI
- **Path**: `.agents/skills/shadcn-ui/`
- **Description**: Build UI components with shadcn/ui (Google Stitch)
- **Category**: Design Systems/Tokens
- **Rating**: 6/10
- **Depth**: 42 lines — stub
- **Assessment**: Good reference but lacks depth. KEEP for shadcn/ui integration.
- **Notes**: Pairs with stitch-design-taste and stitch-loop.

---

## 32. SHADER-DEV
- **Path**: `.agents/skills/shader-dev/`
- **Description**: GLSL shader techniques — ray marching, fluid simulation, particle systems
- **Category**: Animation/Motion/3D
- **Rating**: 5/10
- **Depth**: 43 lines — MiniMax stub
- **Assessment**: Important topic but too shallow to be useful. Needs expansion or REMOVE.
- **Notes**: Vanta-orchestrator references shader pipeline. Keep if expanding later.

---

## 33. SLACK-GIF-CREATOR
- **Path**: `.agents/skills/slack-gif-creator/`
- **Description**: Animated GIFs optimized for Slack with size validators
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 42 lines — stub
- **Assessment**: Niche and shallow. Low project relevance. REMOVE.
- **Notes**: Very narrow use case (Slack GIFs). Not needed for VantaDB.

---

## 34. SLIDES
- **Path**: `.agents/skills/slides/`
- **Description**: Create/edit .pptx with PptxGenJS (OpenAI official)
- **Category**: Deck/Slide/Video Template
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Redundant with pptx, pptx-generator. Same tech stack. REMOVE.
- **Notes**: Third pptx-related skill. Consolidate to one.

---

## 35. SOCIAL-REDDIT-CARD
- **Path**: `.agents/skills/social-reddit-card/`
- **Description**: Realistic Reddit post card with vote rail and comment count
- **Category**: Image/Video/AI Media
- **Rating**: 6/10
- **Depth**: 68 lines — HTML-anything template with example
- **Assessment**: Good for social media content in video overlays. KEEP.
- **Notes**: Unique social media visual. Part of HTML-anything template family.

---

## 36. SOCIAL-SPOTIFY-CARD
- **Path**: `.agents/skills/social-spotify-card/`
- **Description**: Spotify Now Playing-style card with album art and playback controls
- **Category**: Image/Video/AI Media
- **Rating**: 6/10
- **Depth**: 67 lines — HTML-anything template
- **Assessment**: Good for music-related visuals. KEEP.
- **Notes**: Unique visual template for project demos/homepages.

---

## 37. SOCIAL-X-POST-CARD
- **Path**: `.agents/skills/social-x-post-card/`
- **Description**: Realistic X (Twitter) post card with engagement metrics
- **Category**: Image/Video/AI Media
- **Rating**: 6/10
- **Depth**: 65 lines — HTML-anything template
- **Assessment**: Good for social media content creation. KEEP.
- **Notes**: Part of social card family (reddit, spotify, x).

---

## 38. SOFT-SKILL (high-end-visual-design)
- **Path**: `.agents/skills/soft-skill/`
- **Description**: Teaches AI to design like a high-end agency — exact fonts, spacing, shadows
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 123 lines — detailed rules
- **Assessment**: REDUNDANT with `high-end-visual-design` in `$home/.agents/skills/`. This is a dup.
- **Notes**: Cross-install duplicate. Keep the global one, remove local copy.

---

## 39. SORA
- **Path**: `.agents/skills/sora/`
- **Description**: Generate/remix/manage short video clips via OpenAI's Sora API
- **Category**: Image/Video/AI Media
- **Rating**: 5/10
- **Depth**: 43 lines — stub
- **Assessment**: Important tech but no depth. REMOVE or expand.
- **Notes**: Video generation is covered by fal.ai and venice alternatives with more detail.

---

## 40. SPEECH
- **Path**: `.agents/skills/speech/`
- **Description**: Generate spoken audio via OpenAI TTS API
- **Category**: Utility/Tool
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Basic TTS skill. Keep for reference. KEEP.
- **Notes**: Redundant with venice-audio-speech. Both are TTS wrappers.

---

## 41. STITCH-DESIGN-TASTE
- **Path**: `.agents/skills/stitch-design-taste/`
- **Description**: Semantic design system skill for Google Stitch — generates DESIGN.md files
- **Category**: Design Systems/Tokens
- **Rating**: 8/10
- **Depth**: 213 lines — comprehensive
- **Assessment**: High-quality, unique skill for design system generation. KEEP.
- **Notes**: Core to Google Stitch workflow. Also see `stitch-skill` dup.

---

## 42. STITCH-LOOP
- **Path**: `.agents/skills/stitch-loop/`
- **Description**: Iterative design-to-code feedback loop (critique → adjust → ship)
- **Category**: Design Systems/Tokens
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Important concept but shallow implementation. KEEP for Stitch ecosystem.
- **Notes**: Works with stitch-design-taste and shadcn-ui.

---

## 43. STITCH-SKILL
- **Path**: `.agents/skills/stitch-skill/`
- **Description**: Duplicate of stitch-design-taste (same name, same description)
- **Category**: Design Systems/Tokens
- **Rating**: 7/10
- **Depth**: 209 lines — slightly different layout, same content
- **Assessment**: REDUNDANT with stitch-design-taste. Same origin/upstream. REMOVE.
- **Notes**: Keep `stitch-design-taste` (slightly better structure).

---

## 44. SWIFTUI-DESIGN
- **Path**: `.agents/skills/swiftui-design/`
- **Description**: SwiftUI frontend design with anti-AI-slop rules, brand asset protocol
- **Category**: Frontend/UI Design
- **Rating**: 5/10
- **Depth**: 43 lines — stub
- **Assessment**: Niche for Apple platform. Minimal depth. REMOVE if no iOS/SwiftUI work.
- **Notes**: VantaDB appears to be web-focused. Low relevance.

---

## 45. SWISS-CREATIVE-MODE-TEMPLATE
- **Path**: `.agents/skills/swiss-creative-mode-template/`
- **Description**: Swiss-inspired presentation template with editorial typography, geometric cards
- **Category**: Deck/Slide/Video Template
- **Rating**: 7/10
- **Depth**: 79 lines — well-structured template
- **Assessment**: High-quality deck template. KEEP.
- **Notes**: Part of template family (with swiss-user-research).

---

## 46. SWISS-USER-RESEARCH-VIDEO-TEMPLATE
- **Path**: `.agents/skills/swiss-user-research-video-template/`
- **Description**: Swiss-style user-research narrative template in warm-paper editorial aesthetics
- **Category**: Deck/Slide/Video Template
- **Rating**: 7/10
- **Depth**: 81 lines — well-structured template
- **Assessment**: Good companion to swiss-creative-mode. KEEP.
- **Notes**: Complements the research-decision-room and ux-heuristics skills.

---

## 47. TASTE-SKILL (design-taste-frontend)
- **Path**: `.agents/skills/taste-skill/`
- **Description**: Anti-slop frontend skill — landing pages, portfolios, redesigns. V2 experimental
- **Category**: Frontend/UI Design
- **Rating**: 9/10
- **Depth**: 1233+ lines — EXTREMELY comprehensive
- **Assessment**: One of the most valuable skills. Extensive anti-slop rules, design systems, templates. KEEP.
- **Notes**: Core skill for premium frontend work. Used by vanta-design-orchestrator.

---

## 48. TASTE-SKILL-V1 (design-taste-frontend-v1)
- **Path**: `.agents/skills/taste-skill-v1/`
- **Description**: Original v1 preserved for backward compatibility (252 lines)
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 252 lines — substantial
- **Assessment**: REDUNDANT with taste-skill (v2). KEEP only if v1 behavior is specifically needed.
- **Notes**: The v2 skill is marked "experimental" — keep v1 as fallback if v2 breaks.

---

## 49. THEME-FACTORY
- **Path**: `.agents/skills/theme-factory/`
- **Description**: Toolkit for styling artifacts with 10 preset themes (colors/fonts)
- **Category**: Design Systems/Tokens
- **Rating**: 8/10
- **Depth**: 59 lines — practical with theme showcase PDF
- **Assessment**: Unique theme system. High practical value. KEEP.
- **Notes**: Referenced by vanta-design-orchestrator. Good for rapid theme application.

---

## 50. THREEJS
- **Path**: `.agents/skills/threejs/`
- **Description**: Three.js for 3D elements and interactive browser experiences
- **Category**: Animation/Motion/3D
- **Rating**: 5/10
- **Depth**: 43 lines — stub
- **Assessment**: Shallow intro to massive topic. REMOVE or replace with the threejs-* sub-skills in global.
- **Notes**: Global skills have 7 dedicated threejs sub-skills — those are better.

---

## 51. UI-DESIGN
- **Path**: `.agents/skills/ui-design/`
- **Description**: Craft polished interfaces — layout grids, color systems, typography, data viz, Gestalt principles
- **Category**: Frontend/UI Design
- **Rating**: 8/10
- **Depth**: 859 lines — extremely comprehensive
- **Assessment**: Deep reference manual for UI design theory. KEEP.
- **Notes**: Complements design-taste-frontend from a theory/principle angle.

---

## 52. UI-SKILLS
- **Path**: `.agents/skills/ui-skills/`
- **Description**: Opinionated evolving constraints for agents building interfaces
- **Category**: Frontend/UI Design
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Redundant with design-taste-frontend, ui-design, and minimalist-ui. REMOVE.
- **Notes**: Too shallow to add value beyond what other UI skills provide.

---

## 53. UI-UX-PRO-MAX
- **Path**: `.agents/skills/ui-ux-pro-max/`
- **Description**: 50 styles, 21 palettes, 50 font pairings, 20 charts, 9 stacks — comprehensive design guide
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 309 lines — Python-driven design database
- **Assessment**: Impressive scope but Python dependency adds complexity. KEEP.
- **Notes**: Locally installed version. Heavy but comprehensive.

---

## 54. UX-HEURISTICS
- **Path**: `.agents/skills/ux-heuristics/`
- **Description**: Evaluate/improve usability via heuristic analysis (Nielsen, Krug)
- **Category**: Research/Strategy
- **Rating**: 8/10
- **Depth**: 281 lines — detailed framework
- **Assessment**: Excellent UX evaluation framework. KEEP.
- **Notes**: Complements prototyping-testing and ux-strategy.

---

## 55. UX-STRATEGY
- **Path**: `.agents/skills/ux-strategy/`
- **Description**: Shape product direction — competitive analysis, experience mapping, IA, content strategy
- **Category**: Research/Strategy
- **Rating**: 7/10
- **Depth**: 597 lines — comprehensive
- **Assessment**: Solid strategic UX skill. KEEP.
- **Notes**: Pairs with ux-heuristics for tactical + strategic coverage.

---

## 56. VANTA-DESIGN-ORCHESTRATOR
- **Path**: `.agents/skills/vanta-design-orchestrator/`
- **Description**: Master orchestrator for 170+ design skills. Central role definition.
- **Category**: Utility/Tool
- **Rating**: 9/10
- **Depth**: 1176+ lines — ENORMOUS
- **Assessment**: THE most important skill. Master coordinator for all other skills. KEEP.
- **Notes**: This is the project's custom orchestrator. Essential. Well-documented.

---

## 57. VENICE-AUDIO-MUSIC
- **Path**: `.agents/skills/venice-audio-music/`
- **Description**: Music generation via Venice.ai for jingles, loops, scoring
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 43 lines — stub
- **Assessment**: Shallow vendor API wrapper. REMOVE unless actively using Venice.ai.
- **Notes**: Redundant with AI music album skill for music generation context.

---

## 58. VENICE-AUDIO-SPEECH
- **Path**: `.agents/skills/venice-audio-speech/`
- **Description**: TTS via Venice.ai for narration, voiceover
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 43 lines — stub
- **Assessment**: Redundant with `speech` skill (OpenAI TTS). Same shallow depth. REMOVE.
- **Notes**: Pick one TTS skill (OpenAI's is more proven).

---

## 59. VENICE-IMAGE-EDIT
- **Path**: `.agents/skills/venice-image-edit/`
- **Description**: Image edits, upscaling, background removal via Venice.ai
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 41 lines — stub
- **Assessment**: Shallow. Redundant with fal-image-edit, recraft tools. REMOVE.
- **Notes**: Venice.ai API coverage is fragmented across 5 shallow skills.

---

## 60. VENICE-IMAGE-GENERATE
- **Path**: `.agents/skills/venice-image-generate/`
- **Description**: Image generation endpoints via Venice.ai
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 41 lines — stub
- **Assessment**: Shallow vendor wrapper. REMOVE.
- **Notes**: Redundant with fal-generate, recraft, replicate, imagegen, imagen.

---

## 61. VENICE-VIDEO
- **Path**: `.agents/skills/venice-video/`
- **Description**: Video generation and transcription via Venice.ai
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 41 lines — stub
- **Assessment**: Shallow vendor wrapper. REMOVE.
- **Notes**: Redundant with sora, remotion, fal-video-edit, youtube-clipper.

---

## 62. VERCEL-OPTIMIZE
- **Path**: `.agents/skills/vercel-optimize/`
- **Description**: Vercel cost/performance optimization audit with metrics-first approach
- **Category**: Backend/Dev/SEO
- **Rating**: 8/10
- **Depth**: 322 lines + AGENTS.md — very detailed
- **Assessment**: High-quality production optimization skill. KEEP.
- **Notes**: References doctrine, voice, and docs library. Practical framework.

---

## 63. VFX-TEXT-CURSOR
- **Path**: `.agents/skills/vfx-text-cursor/`
- **Description**: Cursor light trail, chromatic rays, directional flares for word-by-word quote reveals
- **Category**: Animation/Motion/3D
- **Rating**: 6/10
- **Depth**: 64 lines — HTML-anything VFX template
- **Assessment**: Unique VFX template for video intros. KEEP.
- **Notes**: Niche but high-quality. Good for video production pipeline.

---

## 64. VIDEO-DOWNLOADER
- **Path**: `.agents/skills/video-downloader/`
- **Description**: Download YouTube and other platform videos
- **Category**: Utility/Tool
- **Rating**: 4/10
- **Depth**: 42 lines — stub
- **Assessment**: Peripheral tool. Low project relevance. REMOVE.
- **Notes**: Potential legal/ethical concerns with video downloading.

---

## 65. VIDEO-HYPERFRAMES
- **Path**: `.agents/skills/video-hyperframes/`
- **Description**: Hyperframes/Remotion-compatible continuous frame animation with autoplay
- **Category**: Animation/Motion/3D
- **Rating**: 6/10
- **Depth**: 46 lines — HTML-anything template
- **Assessment**: Bridge between HTML and video output. KEEP.
- **Notes**: Referenced by other templates (weread, etc.). Integration point.

---

## 66. VISUAL-CRITIQUE
- **Path**: `.agents/skills/visual-critique/`
- **Description**: Critique across hierarchy, brand consistency, composition, typography
- **Category**: Design Systems/Tokens
- **Rating**: 7/10
- **Depth**: 300 lines — comprehensive
- **Assessment**: Good systematic critique framework. KEEP.
- **Notes**: Complements plan-design-review and impeccable-design-polish.

---

## 67. VISUAL-REVIEW
- **Path**: `.agents/skills/visual-review/`
- **Description**: Automated design audit pipeline — Playwright + ImageMagick + pixelmatch + CSS audit
- **Category**: Utility/Tool
- **Rating**: 8/10
- **Depth**: 105 lines — practical pipeline with scripts
- **Assessment**: Excellent automated QA pipeline. Integrates all tools. KEEP.
- **Notes**: Key part of vanta-design-orchestrator. Custom scripts available.

---

## 68. WEB-ARTIFACTS-BUILDER
- **Path**: `.agents/skills/web-artifacts-builder/`
- **Description**: Build complex HTML artifacts with React and Tailwind (Anthropic official)
- **Category**: Frontend/UI Design
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Authoritative reference but shallow. Keep for Anthropic standard. KEEP.
- **Notes**: Referenced in AGENTS.md. Official Anthropic workflow.

---

## 69. WEB-DESIGN-GUIDELINES
- **Path**: `.agents/skills/web-design-guidelines/`
- **Description**: Review UI code for Web Interface Guidelines compliance (Vercel)
- **Category**: Backend/Dev/SEO
- **Rating**: 6/10
- **Depth**: 40 lines — fetches guidelines from remote URL
- **Assessment**: Auto-updating guideline checker. KEEP.
- **Notes**: Pairs with writing-guidelines. Fetches latest rules dynamically.

---

## 70. WEREAD-YEAR-IN-REVIEW-VIDEO-TEMPLATE
- **Path**: `.agents/skills/weread-year-in-review-video-template/`
- **Description**: WeRead-inspired HyperFrames template for vertical reading reports
- **Category**: Deck/Slide/Video Template
- **Rating**: 7/10
- **Depth**: 94 lines — well-structured template
- **Assessment**: High-quality niche template for reading/analytics reports. KEEP.
- **Notes**: Unique vertical format (9:16). Not duplicated elsewhere.

---

## 71. WPDS
- **Path**: `.agents/skills/wpds/`
- **Description**: WordPress Design System tokens and patterns
- **Category**: Design Systems/Tokens
- **Rating**: 3/10
- **Depth**: 42 lines — stub
- **Assessment**: Low relevance — VantaDB is not a WordPress project. REMOVE.
- **Notes**: WordPress-only. No value for current project.

---

## 72. WRITING-GUIDELINES
- **Path**: `.agents/skills/writing-guidelines/`
- **Description**: Review docs/prose against Writing Guidelines compliance (Vercel)
- **Category**: Content/Writing
- **Rating**: 6/10
- **Depth**: 40 lines — fetches remote guidelines
- **Assessment**: Auto-updating writing checker. KEEP.
- **Notes**: Pairs with web-design-guidelines. Good for documentation quality.

---

## 73. YOUTUBE-CLIPPER
- **Path**: `.agents/skills/youtube-clipper/`
- **Description**: YouTube clip generation — pull, slice, add captions, export
- **Category**: Image/Video/AI Media
- **Rating**: 5/10
- **Depth**: 42 lines — stub
- **Assessment**: Niche video editing. Shallow. REMOVE.
- **Notes**: Limited use case for VantaDB project work.

---

## 74. HAND-DRAWN-DIAGRAMS
- **Path**: `.agents/skills/hand-drawn-diagrams/`
- **Description**: Generate Excalidraw hand-drawn diagrams — animated SVG, edit link, PNG
- **Category**: Utility/Tool
- **Rating**: 6/10
- **Depth**: 42 lines — stub
- **Assessment**: Unique diagramming approach. KEEP.
- **Notes**: Good for whiteboard-style planning and documentation.

---

## 75. IMPECCABLE-DESIGN-POLISH
- **Path**: `.agents/skills/impeccable-design-polish/`
- **Description**: Follow-up design polish — audit, critique, polish, animate, harden
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 69 lines — clear polish workflow
- **Assessment**: Good finishing/QA skill for shipped interfaces. KEEP.
- **Notes**: Complements design-review and visual-critique as a polish pass.

---

## 76. GPT-TASTE / GPT-TASTESKILL
- **Paths**: `.agents/skills/gpt-taste/` and `.agents/skills/gpt-tasteskill/`
- **Description**: Elite UX/UI + GSAP Motion Engineer with Python-driven randomization
- **Category**: Frontend/UI Design
- **Rating**: 7/10
- **Depth**: 89 and 100 lines — substantial content
- **Assessment**: REDUNDANT — these are duplicates of each other. Both from taste-skill lineage.
- **Notes**: MERGE or REMOVE one. The gpt-tasteskill has better metadata/triggers.

---

## 77. IMAGEN
- **Path**: `.agents/skills/imagen/`
- **Description**: Generate images via Google Gemini API
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: 43 lines — stub
- **Assessment**: Another generation API wrapper. Redundant with fal, recraft, venice, replicate. REMOVE.
- **Notes**: Fifth image generation skill. Fragmentation without depth.

---

## 78. IMAGE-TO-CODE / IMAGE-TO-CODE-SKILL
- **Paths**: `.agents/skills/image-to-code/` and `.agents/skills/image-to-code-skill/`
- **Description**: Website image-to-code — generate design images, analyze, implement
- **Category**: Frontend/UI Design
- **Rating**: 6/10
- **Depth**: ~40 lines each — stubs
- **Assessment**: REDUNDANT — two copies of same skill. Keep one. REMOVE duplicate.
- **Notes**: Unique approach (image → analysis → code). Potential value but needs depth.

---

## 79. IMAGE-ENHANCER
- **Path**: `.agents/skills/image-enhancer/`
- **Description**: Improve image/screenshot resolution, sharpness, clarity
- **Category**: Image/Video/AI Media
- **Rating**: 4/10
- **Depth**: stub level
- **Assessment**: Low depth. REMOVE.
- **Notes**: Covered by recraft upscale tools and visual-review pipeline.

---

## 80. IMAGEGEN / IMAGEGEN FRONTEND MOBILE / IMAGEGEN FRONTEND WEB
- **Paths**: `.agents/skills/imagegen/`, `.agents/skills/imagegen-frontend-mobile/`, `.agents/skills/imagegen-frontend-web/`
- **Description**: Image generation for UI mockups/icons/illustrations (OpenAI API)
- **Category**: Image/Video/AI Media
- **Rating**: 6/10 (imagegen), 7/10 (frontend-web), 6/10 (frontend-mobile)
- **Depth**: All ~42 lines — stubs
- **Assessment**: OpenAI imagegen family. Different platforms. KEEP the frontend-web (most useful).
- **Notes**: imagegen-frontend-web and mobile are premium direction skills — keep those.

---

## KEY FINDINGS — DUPLICATE/REDUNDANT SKILLS

| Group | Skills | Action |
|-------|--------|--------|
| Minimalist UI | minimalist-skill, minimalist-ui | KEEP minimalist-ui only |
| Redesign | redesign-existing-projects, redesign-skill | KEEP redesign-existing-projects only |
| PPTX/Slides | pptx, pptx-generator, slides, nanobanana-ppt | KEEP pptx only (official Anthropic) |
| Stitch | stitch-design-taste, stitch-skill | KEEP stitch-design-taste only |
| Taste/Design | taste-skill, taste-skill-v1 | KEEP taste-skill (v2), drop v1 unless needed |
| GPT Taste | gpt-taste, gpt-tasteskill | MERGE or keep one |
| Image-to-code | image-to-code, image-to-code-skill | KEEP one |
| Soft/High-end | soft-skill, high-end-visual-design | Keep high-end-visual-design (global copy) |
| Venice API set | venice-* (5 skills) | REMOVE all — shallow vendor stubs |
| Imagen | imagen | REMOVE — redundant with fal/higher-quality tools |

## KEY FINDINGS — KEEP (HIGHEST VALUE)

1. **vanta-design-orchestrator** (9/10) — Master orchestrator
2. **taste-skill** (design-taste-frontend v2) (9/10) — Premium frontend
3. **react-best-practices** (9/10) — Vercel production guidance
4. **theme-factory** (8/10) — Theme system
5. **platform-design** (8/10) — Cross-platform rules
6. **reference-design-contract** (8/10) — Design specification
7. **research-decision-room** (8/10) — Research synthesis
8. **ux-heuristics** (8/10) — Usability evaluation
9. **visual-review** (8/10) — Automated QA pipeline
10. **vercel-optimize** (8/10) — Production optimization
11. **pptx-html-fidelity-audit** (8/10) — PPTX quality control
12. **output-skill** (full-output-enforcement) (8/10) — Agent behavior
13. **stitch-design-taste** (8/10) — Design system generation
14. **redesign-existing-projects** (8/10) — Upgrade methodology
15. **ui-design** (8/10) — Design theory reference
16. **ux-strategy** (7/10) — Strategic UX
17. **prototyping-testing** (7/10) — Research process
18. **visual-critique** (7/10) — Design critique
19. **plan-design-review** (7/10) — Quality gate
20. **swiss-creative-mode-template** (7/10) — Deck template
21. **weread-year-in-review-video-template** (7/10) — Video report
22. **mockup-device-3d** (7/10) — Product mockups
23. **ppt-keynote** (7/10) — HTML deck
24. **pr-feedback-quality-gate** (7/10) — PR workflow
25. **gpt-taste** (7/10) — GSAP/editorial frontend
26. **impeccable-design-polish** (7/10) — Polish pass
27. **minimalist-ui** (7/10) — Editorial UI direction

## SUMMARY STATISTICS

- **Total skills analyzed**: ~80 (some are duplicates)
- **Recommended KEEP**: ~40
- **Recommended REMOVE (shallow stubs)**: ~15-20
- **Recommended REMOVE (duplicates)**: ~10-15
- **Average depth**: Shallow (most are <50 line stubs)
- **Highest quality sources**: Vercel, Anthropic official, taste-skill lineage, HTML-anything templates
- **Key problem**: Massive skill fragmentation across vendor API wrappers (venice, pixelbin, replicate, etc.)
