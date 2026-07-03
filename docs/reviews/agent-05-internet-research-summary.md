# AI Agent Skills: Research Summary

| # | Skill | Source | Verified? | Consensus | Trust (1-10) | Notes |
|---|-------|--------|-----------|-----------|--------------|-------|
| 1 | impeccable | Paul Bakaus | Y | Gold standard design skill. 23 commands, 27 anti-pattern rules. 28.6K★ | 9 | Apache 2.0. Ex-Google author. #1 trending on GitHub. Must-install for frontend work. |
| 2 | design-taste-frontend | leonxlnx | Y | Go-to anti-slop skill. Three Dials system, strict ban list. 55.5K★ | 8 | Anti-slop rules are deterministic. V2 experimental. Not for dashboards. |
| 3 | brandkit | leonxlnx | Y | Premium brand identity boards. 119.9K installs. 3x3 panel system. | 8 | Most-installed image-gen skill. Quality depends on underlying model. |
| 4 | emil-design-eng | emilkowalski | Y | Authoritative animation/motion skill. 4.4K★. Ex-Vercel/Linear author. | 9 | Narrow scope (animation only). High-authority author. Required Before/After table. |
| 5 | canvas-design | Anthropic | Y | Static art via design philosophy. Official Anthropic. | 7 | Niche, quality depends on agent's render capability. Needs local fonts. |
| 6 | algorithmic-art | Anthropic | Y | p5.js generative art with seeded randomness. Official. | 7 | Limited to canvas 2D. Has Anthropic branding in template. |
| 7 | hyperframes | HeyGen | Y | HTML-to-video framework. 20 skills. Real rendering pipeline. | 8 | Apache 2.0. Complex ecosystem (20 skills). Cloud rendering costs. |
| 8 | GSAP skills | GreenSock | Y | 8 official skills. Must-have for animation work. All plugins free. | 9 | Officially by GSAP creators. Fixes janky AI animation code. |
| 9 | threejs skills | Multiple | Partial | Competing packages. No single authoritative source. | 6 | Best: CloudAI-X (11 skills) or Impertio-Studio (24 skills). |
| 10 | Motion.dev AI Kit | Motion.dev | Y | Official. MCP server + MotionScore audit. 10M+ downloads. | 7 | Requires Motion+ subscription for full access. Free tier limited. |
| 11 | imagegen-frontend-web | leonxlnx | Y | One image per section rule. 87.8K installs. | 7 | Hard rule is enforceable. Multiple API calls per page (expensive). |
| 12 | fal.ai skills | fal-ai-community | Y | Bash scripts for 50+ models via fal API. 319 installs. | 6 | Thin wrappers. Requires FAL_KEY. Multiple competing packages. |
| 13 | image-to-code | leonxlnx | Partial | Image-first pipeline. Codex-specific. | 5 | Heavy token usage. "Deep analysis" is subjective. Competitor exists. |
| 14 | ai-seo | coreyhaines31 | Y | AI search optimization. Three-pillar framework. 35.1K★ parent. | 7 | GEO/AEO field is emerging. Hard to verify 30-40% visibility claim. |
| 15 | audit-website | squirrelscan | Y | 245+ rules. LLM-optimized reports. Free local audits. | 7 | Free tier limited. Cloud features cost credits. Real CLI. |
| 16 | copywriting | coreyhaines31 | Y | Conversion copywriting. 136.8K installs. AI-tell sweep. | 8 | Most-installed marketing skill. Practical rules. Needs product context setup. |
| 17 | ai-sdk | Vercel | Y | Official AI SDK skill. 107K+★. HarnessAgent in v7. | 9 | Massive adoption. Best if using AI SDK. HarnessAgent experimental. |
| 18 | writing-plans | obra/superpowers | Y | Bite-sized TDD plans. Banned placeholders. | 8 | Requires discipline. Verbose but thorough. Needs executing-plans companion. |
| 19 | brainstorming | obra/superpowers | Y | Structured design exploration. No-code-before-approval gate. | 8 | Can feel bureaucratic for small tasks. Best for medium-large features. |
| 20 | systematic-debugging | obra/superpowers | Partial | 4-phase methodology. 95% fix rate claim (unverified). | 7 | Sound methodology. 95% claim uncited. Can slow simple fixes. |
| 21 | agent-browser | Vercel Labs | Y | Rust browser automation. 37K★. CDP-native. @eN ref system. | 8 | Fast. Token-efficient (@eN refs). Requires Chrome/Chromium installed. |
| 22 | skill-creator | Anthropic | Y | Skill creation + eval + A/B comparison. Official Anthropic. | 8 | Essential for skill builders. Subagent evals consume tokens. |
| 23 | find-skills | Vercel Labs | Y | Meta-skill for skill discovery. | 7 | Limited utility without skills CLI. Just a markdown instruction file. |
| 24 | vanta-design-orchestrator | Custom | N/A | Internal VantaDB orchestrator. No public presence. | N/A | Custom skill. Unclear if it effectively orchestrates or creates redundancy. |
| 25 | next-best-practices | Vercel Labs | Y | Official Next.js best practices. File conventions, RSC, data. | 9 | Authoritative. Vercel-platform-specific in places. |
| 25b | vercel-react-best-practices | Vercel Labs | Y | 64 rules, 8 categories, impact-prioritized. Official Vercel. | 9 | Covered by InfoQ. Some overlap with next-best-practices. |

---

## Key Takeaways for VantaDB

### Must-Install (High Value)
1. **impeccable** (9/10) — Gold standard design skill. Install immediately.
2. **GSAP skills** (9/10) — Official from GreenSock. Essential for animation.
3. **ai-sdk** (9/10) — If using AI SDK. Vercel-backed.
4. **next-best-practices** + **vercel-react-best-practices** (9/10 each) — Official Vercel.
5. **emil-design-eng** (9/10) — Premium animation philosophy.

### Should Install (Good Value)
6. **copywriting** (8/10) — 136.8K installs. Strong conversion copy.
7. **agent-browser** (8/10) — 37K★ Rust browser automation.
8. **design-taste-frontend** (8/10) — Anti-slop frontend rules.
9. **writing-plans** + **brainstorming** (8/10 each) — Structured dev methodology.
10. **skill-creator** (8/10) — For building/improving your own skills.
11. **brandkit** (8/10) — Brand identity images.
12. **hyperframes** (8/10) — HTML-to-video.

### Consider for Specific Needs
13. **audit-website** (7/10) — SEO audits.
14. **ai-seo** (7/10) — AI search optimization.
15. **Motion.dev AI Kit** (7/10) — If using Motion library.
16. **canvas-design** (7/10) — Static art generation.
17. **algorithmic-art** (7/10) — Generative art.
18. **systematic-debugging** (7/10) — Debugging discipline.
19. **find-skills** (7/10) — Skill discovery.
20. **imagegen-frontend-web** (7/10) — Section-by-section images.

### Low Priority
21. **threejs skills** (6/10) — No single authoritative package.
22. **fal.ai skills** (6/10) — Thin wrappers, low installs.
23. **image-to-code** (5/10) — Codex-specific, heavy token usage.
24. **vanta-design-orchestrator** (N/A) — Custom. Review for overlap/redundancy.
