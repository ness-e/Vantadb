# CAPA 3 — DISEÑO VISUAL Y COMPOSICIÓN

---

## 6. `ui-design` — Diseño de Interfaces Pulidas

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 13 sub-skills visuales: layout grids, sistemas de color (con compliance WCAG), escalas tipográficas modulares, responsive design, data visualization, sistemas de espaciado, diseño dark mode, sistemas de ilustración, jerarquía visual, medida legible (45-75 caracteres), y principios Gestalt (proximidad, región común, efecto Von Restorff, efecto aesthetic-usability). |
| **¿Para qué es?** | Craft visual: convertir wireframes en interfaces pulidas con fundamento teórico en percepción visual. |
| **¿Para qué se usa?** | Generar paletas de color con escalas tonales completas (50-950) y mappings semánticos. Crear escalas tipográficas con ratio modular (1.25 major third). Definir grids responsivos (4/8/12 columnas). Diseñar data visualizations accesibles. Aplicar dark mode con desaturación y elevación por luminosidad. |
| **¿Cómo se usa?** | Consultando la sub-skill específica: `color-system` para paletas, `typography-scale` para tipografía, `layout-grid` para grids, `responsive-design` para breakpoints, `visual-hierarchy` para jerarquía, `spacing-system` para espaciado, `dark-mode-design` para modo oscuro. |
| **¿Cómo debería usarse?** | `color-system` → genera la paleta completa y verifica contraste AA en cada combinación fondo/texto. `typography-scale` → define con ratio matemático, mínimo 16px para body. `spacing-system` → usa base de 4px o 8px con escala nombrada (xs/sm/md/lg/xl). |
| **¿Cuándo debería usarse?** | **Fase 2** — Después de definir la estructura, para aplicar identidad visual con fundamento perceptual. |
| **Workflows disponibles** | `/ui-design:color-palette`, `/ui-design:design-screen`, `/ui-design:responsive-audit`, `/ui-design:type-system` |
| **Dependencias** | Skill de proyecto (`.agent/skills/ui-design/`). No requiere instalación — es conocimiento de diseño visual con 13 sub-skills. |
| **Requerimientos** | Ninguno. Funciona sobre cualquier stack. Opcional: herramientas de color (Coolors, OKLCH Chrome) para verificar paletas. |

## 7. `visual-critique` — Crítica Visual Estructurada

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 4 sub-skills de crítica: jerarquía visual (entry point, eye flow, weight, emphasis), consistencia de marca (mood.md, voice.md, tokens.md), composición (balance, whitespace, ritmo, gestalt), y tipografía (escala, legibilidad, consistencia, compliance de tokens). |
| **¿Para qué es?** | Evaluar una pantalla existente de forma estructurada y producir una lista priorizada de correcciones. |
| **¿Para qué se usa?** | Auditar cada dimensión con rating `pass` / `minor issue` / `major issue`. Identificar: puntos de entrada ambiguos, flujo ocular roto, peso visual mal distribuido, énfasis falso, inconsistencias tipográficas, desvíos de tokens, composición desequilibrada. |
| **¿Cómo se usa?** | Ejecutando el workflow `/visual-critique:critique-screen` que corre las 4 críticas y consolida hallazgos. Cada crítica sigue el formato: Observación → Problema → Fix. |
| **¿Cómo debería usarse?** | Comparando contra archivos de referencia del proyecto (`mood.md`, `voice.md`, `tokens.md`). Si no existen, crear primero la referencia de brand con `design-systems`. |
| **¿Cuándo debería usarse?** | **Fase 4** — Después de implementar, como auditoría de calidad visual antes de producción. |
| **Workflows disponibles** | `/visual-critique:critique-screen` |
| **Dependencias** | Skill de proyecto (`.agent/skills/visual-critique/`). No requiere instalación — es suite de crítica visual estructurada. |
| **Requerimientos** | Ninguno. Solo acceso a la pantalla/diseño a evaluar. Opcional: archivos de referencia del proyecto (`mood.md`, `voice.md`, `tokens.md`). |

## 8. `awesome-claude-design` — Anti-Slop y Familias Estéticas

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Base cognitiva contra la monotonía visual ("AI slop") con familias estéticas predefinidas, directrices WebGL/Shaders y recetas de diseño avanzado. Incluye el "Slop Test" y guías para Three.js/R3F. |
| **¿Para qué es?** | Alinear el proyecto con una familia estética refinada (ej. _Cinematic Dark_, _Organic Minimal_, _Editorial Mono_) y evitar los patrones visuales genéricos que delatan código generado por IA. |
| **¿Para qué se usa?** | Diseñar shaders WebGL optimizados sin loops dinámicos pesados. Aplicar el slop-test a cada pantalla (¿tiene tarjetas genéricas? ¿gradientes morado-azul? ¿tipografía Inter en todo?). Elegir y aplicar una familia estética con coherencia. |
| **¿Cómo se usa?** | Consultando las guías de familias estéticas y los checklists anti-slop. Para WebGL: verificando que fragment shaders no usen branching dinámico pesado y que el render sea 60fps constante. |
| **¿Cómo debería usarse?** | Restringiendo shaders y escenas 3D a un presupuesto de frame (<16.6ms). Aplicando fallbacks estáticos con `prefers-reduced-motion`. Evitando `ease-in` en transiciones UI. |
| **¿Cuándo debería usarse?** | **Fase 2-3** — Al definir identidad visual y al implementar elementos 3D/WebGL. |
| **Dependencias** | Skill de proyecto (`.agent/skills/awesome-claude-design/`). No requiere instalación — es base cognitiva anti-slop con guías de estética y WebGL. |
| **Requerimientos** | Ninguno. Para WebGL: navegador con soporte WebGL2. Opcional: Three.js si se usan las guías 3D. |
