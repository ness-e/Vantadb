---
name: awesome-claude-design
description: Design systems, aesthetic families, anti-slop guidelines, and recipes for advanced UI/UX work (including Three.js 3D and shaders).
---

# Awesome Claude Design Guidelines

Guidelines for building modern web applications, avoiding generic AI-generated frontend slop, and implementing advanced design systems.

## The 9 Aesthetic Families

Use these families to establish clear, professional visual directions:

| Family                       | Vibe & Focus                                 | Key Tokens & Colors                                                       | Typical Brands     |
| ---------------------------- | -------------------------------------------- | ------------------------------------------------------------------------- | ------------------ |
| **1. Editorial Minimalism**  | Clean, dark, technical, highly focused       | Dark background (`#0f0f14`), slate/white text, indigo accents (`#5e6ad2`) | Linear             |
| **2. Terminal-Core**         | Monospaced, high-contrast, structural        | Black/White, raw borders, mono fonts                                      | Ollama             |
| **3. Warm Editorial**        | Textured, sand-like warm tones, organic      | Cream background (`#f4f3ee`), warm charcoal, rust accent (`#c96442`)      | Anthropic / Claude |
| **4. Data-Dense Pro**        | Information-rich, developer-focused          | Dark gray (`#181818`), neon yellow (`#faff69`), magenta accents           | ClickHouse         |
| **5. Cinematic Dark**        | Immersive, deep canvas, neon details         | True black (`#000000`), cyan/magenta glow, blur backdrops                 | RunwayML           |
| **6. Playful Color**         | Vibrant, rounded, collaborative              | Bright green (`#0acf83`), orange (`#f24e1e`), purple (`#a259ff`)          | Figma              |
| **7. Glass / Soft-Futurism** | Pastel, radial gradients, soft glass layers  | Transparent white/black borders, radial backdrop-blur                     | Arc Browser        |
| **8. Neon Brutalist**        | High contrast, bold headers, stark UI        | Saturated orange/neon, heavy borders (`#000`), raw typography             | The Verge          |
| **9. Cult / Indie**          | Warm glass, nostalgic textures, fine borders | Warm paper background (`#faf8f2`), subtle glows                           | Granola            |

---

## Anti-Slop Frontend Kit

Avoid common AI-generated design patterns that look cheap or templated:

### 1. Typography & Hierarchy

- **Don't** use Inter/system-sans for absolutely everything.
- **Do** select distinct typography configurations:
  - For minimal: Inter, Outfit, or Geist.
  - For warm: Georgia, Playfair Display, or elegant serifs.
  - For terminal: JetBrains Mono, Fira Code, SF Mono.
- **Do** use tabular numbers (`tabular-nums`) for numeric interfaces, counts, and clocks.

### 2. Cards & Nesting

- **Don't** wrap every section in a rounded card, and never nest cards inside cards (e.g. card container -> card lists -> card items).
- **Do** use borderless layouts, subtle separator lines, or background shifts to organize hierarchy.

### 3. Contrast & Colors

- **Don't** use generic purple-to-blue gradients for landing heroes.
- **Don't** use medium gray text on dark or colored backgrounds (insufficient contrast).
- **Do** use HSL color palettes tailored to the brand. Tint your blacks and grays with a touch of the primary brand color to make them feel organic and cohesive.

---

## Advanced 3D & Shader Guidelines

When implementing Three.js elements, interactive 3D objects, or GLSL Shaders:

### 1. Geometry & Rendering Constraints

- **Limit Accent Materials**: Busy/overloaded 3D scenes look unprofessional. Stick to a clean primary surface texture and at most one accent material (e.g., glass refraction or metallic glow).
- **Performance Budget**: Shaders and WebGL renders must run smoothly at 60fps on mid-tier hardware. Avoid dynamic nested loops inside fragment shaders. Pre-compute noise fields when possible.

### 2. Motion and Easing

- **Don't** use default `ease-in-out` transitions for everything.
- **Do** use purposeful, physical easing (e.g., cubic-bezier springs, slow dollies, inertia).
- **Accessibility**: Respect `prefers-reduced-motion`. Always provide a static fallback image or canvas state if the user has motion preferences enabled.
