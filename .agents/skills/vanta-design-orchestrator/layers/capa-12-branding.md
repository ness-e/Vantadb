# CAPA 12 (BIS) — BRANDING, ARTE GENERATIVO Y TEMATIZACIÓN

---

## `brandkit` — Brand-Kit Image Generation (Premium Identity)

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Generación de boards de brand-kit premium: sistemas de logo, identity decks, presentaciones visuales de marca. 50 estilos, layouts 3×3/2×3/2×2/1×3/4×2. |
| **¿Para qué es?** | Crear identidad de marca completa desde cero: logo, paleta, tipografía, aplicaciones. |
| **¿Cómo se usa?** | Estrategia de marca primero (categoría, audiencia, metáfora). Luego generar board con 9 paneles. |
| **¿Cómo debería usarse?** | Como fase de discovery visual para definir dirección de marca antes de implementar código. Combinar con `canvas-design` para assets finales. |
| **¿Cuándo debería usarse?** | **Pre-Fase 1 / Fase 1** — Al iniciar un nuevo proyecto o rediseñar la identidad visual completa. |
| **Dependencias** | Skill de proyecto (`.agent/skills/brandkit/`). No requiere instalación. |
| **Requerimientos** | API de generación de imágenes disponible. |

## `canvas-design` — Visual Art + Design Philosophy (Static)

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Creación de arte visual en .png y .pdf usando filosofía de diseño. Output: filosofía (.md) + canvas (.png/.pdf). |
| **¿Para qué es?** | Pósters, piezas de arte, diseños estáticos, boards de dirección visual, arte impreso. |
| **¿Cómo se usa?** | 1) Crear filosofía de diseño (manifiesto estético). 2) Deducir referencia conceptual. 3) Expresar visualmente en canvas vía código. |
| **¿Cómo debería usarse?** | Para piezas que requieren un manifiesto estético primero, implementación visual después. |
| **¿Cuándo debería usarse?** | **Fase 2-3** — Cuando se necesita una pieza visual de alta calidad (logo asset, poster, board de dirección). |
| **Dependencias** | Skill de proyecto (`.agent/skills/canvas-design/`). Incluye `canvas-fonts/` con fuentes preinstaladas. |
| **Requerimientos** | Para output PDF: biblioteca de generación PDF. Para output PNG: canvas API. Fuentes en `./canvas-fonts/`. |

## `algorithmic-art` — Generative Art + p5.js (Interactive)

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Creación de arte generativo algorítmico con p5.js. Output: filosofía algorítmica (.md) + HTML interactivo autónomo. Seeded randomness. |
| **¿Para qué es?** | Arte generativo interactivo, flow fields, particle systems, animaciones algorítmicas. |
| **¿Cómo se usa?** | 1) Crear filosofía algorítmica. 2) Deducir semilla conceptual. 3) Implementar en p5.js usando `templates/viewer.html`. |
| **¿Cómo debería usarse?** | Para piezas interactivas que se exploran en navegador. Seed navigation, parámetros ajustables, regenerate button. |
| **¿Cuándo debería usarse?** | **Fase 2-3** — Cuando el proyecto necesita arte generativo interactivo o background animado único. |
| **Dependencias** | Skill de proyecto (`.agent/skills/algorithmic-art/`). Templates en `templates/viewer.html`. |
| **Requerimientos** | p5.js vía CDN. Navegador moderno. No requiere build step. |
