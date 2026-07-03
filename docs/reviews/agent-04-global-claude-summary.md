# Agent-04: Global Claude Skills — Summary Table

**Source:** `C:\Users\Eros\.claude\skills\` (36 dirs, 27 with content)

| # | Skill Name | Category | Rating | Keep/Remove | Notes |
|---|---|---|---|---|---|
| 1 | **ai-seo** | Backend/Dev/SEO | 9 | KEEP | Modern AI search optimization, v2.1.0, 489 lines |
| 2 | **animejs** | Animation/Motion/3D | 8 | KEEP | Good GSAP alternative for SVG-heavy work, 525 lines |
| 3 | **design-motion-principles** | Animation/Motion/3D | 7 | KEEP | Emil/Jakub/Jhey perspective, create + audit modes |
| 4 | **gsap-core** | Animation/Motion/3D | 9 | KEEP | Official GSAP core, MIT license |
| 5 | **gsap-frameworks** | Animation/Motion/3D | 8 | KEEP | Vue/Svelte GSAP integration |
| 6 | **gsap-performance** | Animation/Motion/3D | 7 | KEEP | Short (79 lines) — could merge into core |
| 7 | **gsap-plugins** | Animation/Motion/3D | 9 | KEEP | Comprehensive plugin docs, 433 lines |
| 8 | **gsap-react** | Animation/Motion/3D | 8 | KEEP | useGSAP hook, React cleanup patterns |
| 9 | **gsap-scrolltrigger** | Animation/Motion/3D | 9 | KEEP | Thorough scroll animation reference |
| 10 | **gsap-timeline** | Animation/Motion/3D | 7 | KEEP | Short (107 lines) — could merge into core |
| 11 | **gsap-utils** | Animation/Motion/3D | 8 | KEEP | clamp, mapRange, random, snap, etc. |
| 12 | **hyperframes** | Deck/Slide/Video Template | 9 | KEEP | Core video composition, 490 lines |
| 13 | **hyperframes-animation** | Animation/Motion/3D | 8 | KEEP | 7 runtime adapters, companion to hyperframes |
| 14 | **image-edit** | Image/Video/AI Media | 7 | KEEP | RunComfy smart router, requires CLI setup |
| 15 | **impeccable** | Frontend/UI Design | **10** | KEEP | **EXCELLENT** — v3.7.1, production-grade, active |
| 16 | **interaction-design** | Animation/Motion/3D | 6 | **REMOVE** | Redundant — covered by impeccable |
| 17 | **motion** | Animation/Motion/3D | 9 | KEEP | motion.dev v12.29.2, up-to-date, modern |
| 18 | **remotion-best-practices** | Deck/Slide/Video Template | 7 | KEEP | React-native video creation |
| 19 | **roier-seo** | Backend/Dev/SEO | 8 | KEEP | Lighthouse auto-audit + auto-fix |
| 20 | **seo** | Backend/Dev/SEO | 8 | KEEP | General SEO, 527 lines, comprehensive |
| 21 | **sleek-design-mobile-apps** | Mobile | 6 | CONDITIONAL | Requires paid Sleek Pro subscription |
| 22 | **threejs-animation** | Animation/Motion/3D | 8 | KEEP | Keyframe, skeletal, morph, GLTF |
| 23 | **threejs-fundamentals** | Animation/Motion/3D | 8 | KEEP | Scene, camera, renderer setup |
| 24 | **threejs-geometry** | Animation/Motion/3D | 8 | KEEP | Built-in + custom BufferGeometry |
| 25 | **threejs-interaction** | Animation/Motion/3D | 9 | KEEP | Raycasting, controls, input, 660 lines |
| 26 | **threejs-materials** | Animation/Motion/3D | 8 | KEEP | PBR, textures, material types |
| 27 | **threejs-shaders** | Animation/Motion/3D | 9 | KEEP | GLSL, ShaderMaterial, effects, 642 lines |

## Empty / Non-Functional (no SKILL.md)

| # | Skill Name | Category | Rating | Keep/Remove | Notes |
|---|---|---|---|---|---|
| 28 | cargo-nextest | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 29 | github-repo-management | Utility/Tool | 0 | REMOVE | Empty dir — no content |
| 30 | m10-performance | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 31 | markdown-documentation | Content/Writing | 0 | REMOVE | Empty dir — no content |
| 32 | python-packaging | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 33 | rust-ffi | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 34 | rust-write-tests | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 35 | test-reporting | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |
| 36 | vector-database-engineer | Backend/Dev/SEO | 0 | REMOVE | Empty dir — no content |

## By Category

| Category | Count | Skills |
|----------|-------|--------|
| Animation/Motion/3D | 18 | animejs, design-motion-principles, gsap-* (7), hyperframes-animation, motion, threejs-* (6), interaction-design |
| Backend/Dev/SEO | 4 (+9 empty) | ai-seo, roier-seo, seo + 9 empty (cargo-nextest, etc.) |
| Frontend/UI Design | 1 | impeccable |
| Deck/Slide/Video Template | 2 | hyperframes, remotion-best-practices |
| Image/Video/AI Media | 1 | image-edit |
| Mobile | 1 | sleek-design-mobile-apps |

## Key Actions

1. **REMOVE** `interaction-design` — redundant with impeccable
2. **REMOVE or INSTALL** the 9 empty directories
3. **CONSIDER MERGING** `gsap-performance` + `gsap-timeline` into `gsap-core`
4. **KEEP AS CORE** impeccable (10/10) — primary design authority
5. **CONDITIONAL** sleek-design-mobile-apps — keep only if Sleek Pro is active
