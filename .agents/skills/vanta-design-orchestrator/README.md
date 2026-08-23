# VantaDB Design Toolkit — Manual de Referencia

> **170+ habilidades de diseño** integradas en un único ecosistema operativo.
> Desde tokens de color hasta auditorías de producción. Todo documentado, todo orquestado.

---

## Índice

1. [Visión General](#visión-general)
2. [Estructura de Carpetas](#estructura-de-carpetas)
3. [Mapa de Skills por Capa](#mapa-de-skills-por-capa)
4. [Guía de Uso por Skill](#guía-de-uso-por-skill)
   - [Capa 1 — Fundaciones y Tokens](#capa-1--fundaciones-y-tokens)
   - [Capa 2 — Estructura y Usabilidad](#capa-2--estructura-y-usabilidad)
   - [Capa 3 — Diseño Visual y Composición](#capa-3--diseño-visual-y-composición)
   - [Capa 4 — Interacciones y Movimiento](#capa-4--interacciones-y-movimiento)
   - [Capa 5 — Auditoría y Refinamiento](#capa-5--auditoría-y-refinamiento)
   - [Capa 6 — Rendimiento y Optimización](#capa-6--rendimiento-y-optimización)
    - [Capa 7 — Investigación y Metodología](#capa-7--investigación-y-metodología)
    - [Capa 8 — Operaciones y Herramientas de Equipo](#capa-8--operaciones-y-herramientas-de-equipo)
4. [Capas Opcionales](#capas-opcionales)
    - [Capa 9 — Video](#capa-9--video-opcional)
    - [Capa 10 — 3D Avanzado](#capa-10--3d-avanzado-opcional)
    - [Capa 11 — SEO + Mobile](#capa-11--seo--mobile-opcional)
    - [Capa 12 — Branding, Arte, Temas](#capa-12--branding-arte-temas)
    - [Capa 13 — Open Design Skills](#capa-13--open-design-skills)
5. [Ciclo de Orquestación](#ciclo-de-orquestación)
6. [Workflows Disponibles](#workflows-disponibles)
7. [Reglas de Resolución de Conflictos](#reglas-de-resolución-de-conflictos)
8. [Referencia Rápida](#referencia-rápida)

---

## Visión General

El toolkit de diseño de VantaDB es una colección de 170+ habilidades (skills) que cubren el ciclo completo de diseño de producto digital:

```
Estrategia → Research → Estructura → Visuals → Interacción → Auditoría → Producción
```

Cada skill es un archivo `SKILL.md` con instrucciones operativas que el agente (o el diseñador) puede seguir. Están organizadas en **13 capas funcionales (8 base + 5 opcionales)** que se ejecutan en secuencia para evitar contradicciones.

**Principio rector**: Las decisiones se fundamentan en evidencia (research, métricas, heurísticas), no en preferencia personal. El "AI slop" (diseño genérico de IA) se rechaza activamente.

---

## Estructura de Carpetas

```
.agent/skills/
├── vanta-design-orchestrator/   ← Orquestador (este README + SKILL.md)
│
├── # CAPA 1 — Fundaciones
├── ui-ux-pro-max/               ← Motor de estilos, tokens, paletas HSL
├── design-systems/              ← Tokens, componentes, accesibilidad, temas, motion
│
├── # CAPA 2 — Estructura
├── ux-heuristics/               ← Nielsen 10, Krug, Trunk Test
├── frontend-design/             ← HTML semántico, CSS limpio, anti-divitis
├── ux-strategy/                 ← IA, competitive analysis, principles, metrics
│
├── # CAPA 3 — Visuals
├── ui-design/                   ← Color, tipografía, grid, responsive, dark mode, gestalt
├── visual-critique/             ← Crítica: jerarquía, brand, composición, tipografía
├── awesome-claude-design/       ← Anti-slop, familias estéticas, WebGL/Shaders
│
├── # CAPA 4 — Interacción
├── emil-design-eng/             ← Springs, micro-detalles, filosofía de movimiento
├── animejs/                     ← Anime.js: timelines, SVG morphing, stagger grid
├── interaction-design/          ← Leyes cognitivas, forms, loading, FSM, nav, search
│
├── # CAPA 5 — Auditoría
├── impeccable/                  ← CLI: audit, polish, slop-test (41 reglas)
├── web-design-guidelines/       ← Compliance web (Vercel guidelines)
├── writing-guidelines/          ← Compliance de prosa y docs
│
├── # CAPA 6 — Rendimiento
├── react-best-practices/        ← 70 reglas React/Next.js (Vercel Engineering)
├── vercel-optimize/             ← Pipeline de auditoría costos/performance Vercel
│
├── # CAPA 7 — Research
├── design-research/             ← Personas, journeys, entrevistas, JTBD, usability
├── prototyping-testing/         ← Prototipos, A/B tests, heurísticas, accessibility tests
│
├── # CAPA 8 — Operaciones
├── design-ops/                  ← Critique, QA, sprints, handoff, team workflow
├── designer-toolkit/            ← Rationale, UX writing, case studies, presentations
│
├── # CAPA 9 — Video (opcional)
├── hyperframes/                 ← Composición de video HTML
├── hyperframes-animation/       ← Animación para HyperFrames
├── remotion-best-practices/     ← Video con React
│
├── # CAPA 10 — 3D Avanzado (opcional)
├── threejs-fundamentals/        ← Escenas Three.js
├── threejs-geometry/            ← Geometría 3D
├── threejs-materials/           ← Materiales PBR
├── threejs-interaction/         ← Raycasting, controles
├── threejs-animation/           ← Animación 3D
├── threejs-shaders/             ← Shaders GLSL
├── threejs/                     ← Three.js consolidado (open-design)
├── shader-dev/                  ← Desarrollo de shaders
├── vfx-text-cursor/             ← Cursor de texto con shader
│
├── # CAPA 11 — SEO + Mobile (opcional)
├── roier-seo/                   ← Auditoría técnica Lighthouse
├── ai-seo/                      ← AI visibility / LLM optimization
├── seo/                         ← Correcciones on-page
├── seo-audit/                   ← SEO health check
├── apple-hig/                   ← Apple Human Interface Guidelines
├── swiftui-design/              ← Diseño SwiftUI nativo
├── flutter-animating-apps/      ← Animaciones Flutter
│
├── # CAPA 12 — Branding, Arte, Temas
├── brandkit/                    ← Brand-kit boards premium
├── canvas-design/               ← Arte estático .png/.pdf
├── algorithmic-art/             ← Arte generativo p5.js
├── theme-factory/               ← 10 temas + custom generator
├── color-expert/                ← Teoría del color avanzada
├── creative-director/           ← Dirección creativa profesional
├── brand-guidelines/            ← Guías de marca
├── design-brief/                ← Briefs de diseño
├── design-consultation/         ← Consultoría de diseño
├── design-review/               ← Revisión profesional de diseños
├── design-md/                   ← Documentación de diseño
├── research-decision-room/      ← Sala de decisiones de investigación
│
├── # CAPA 13 — Open Design Skills (134 skills)
│   ├── FAL.ai Pipeline/         ← 11 skills de generación imagen/video/3D
│   ├── Figma Integration/       ← 7 skills de Figma
│   ├── Deck / Slides/           ← 5 skills de presentaciones
│   ├── PPT/PPTX/                ← 4 skills de PowerPoint
│   ├── Frame Effects/           ← 7 skills de efectos visuales
│   ├── Venice AI/               ← 5 skills de imagen/audio/video
│   ├── Social / Cards/          ← 6 skills de redes sociales
│   ├── Video Templates/         ← 7 templates de video
│   ├── Documentos/              ← 5 skills de documentos
│   ├── Apple / Mobile/          ← 3 skills de plataformas Apple
│   ├── Visual / Mockups/        ← 5 skills de visualización
│   ├── UI / Components/         ← 3 skills de componentes
│   ├── Imágenes / E-commerce/   ← 9 skills de imágenes
│   ├── Audio / Video/           ← 3 skills de audio/video
│   ├── Marketing / Ads/         ← 3 skills de marketing
│   └── Utility/                 ← 20+ skills de utilidad
```

> Ver `SKILL.md` → CAPA 13 para la lista completa de las 134 skills open-design.

---

## Mapa de Skills por Capa

| Capa                          | Skill                       | Sub-Skills | Fase      |
| :---------------------------- | :-------------------------- | ---------: | :-------- |
| **1. Fundaciones**            | `ui-ux-pro-max`             |          — | Fase 1    |
|                               | `design-systems`            |         10 | Fase 1-2  |
| **2. Estructura**             | `ux-heuristics`             |          — | Fase 1    |
|                               | `frontend-design`           |          — | Fase 1    |
|                               | `ux-strategy`               |         10 | Pre-Fase  |
| **3. Visuals**                | `ui-design`                 |         13 | Fase 2    |
|                               | `visual-critique`           |          4 | Fase 4    |
|                               | `awesome-claude-design`     |          — | Fase 2-3  |
| **4. Interacción**            | `emil-design-eng`           |          — | Fase 3    |
|                               | `animejs`                   |          — | Fase 3    |
|                               | `interaction-design`        |         13 | Fase 3    |
| **5. Auditoría**              | `impeccable`                |          — | Fase 4    |
|                               | `web-design-guidelines`     |          — | Fase 4    |
|                               | `writing-guidelines`        |          — | Fase 4    |
| **6. Rendimiento**            | `react-best-practices`      |          — | Fase 3-4  |
|                               | `vercel-optimize`           |          — | Post-prod |
| **7. Research**               | `design-research`           |         10 | Pre-Fase  |
|                               | `prototyping-testing`       |          8 | Fase 2-3  |
| **8. Operaciones**            | `design-ops`                |          7 | Fase 4+   |
|                               | `designer-toolkit`          |          7 | Cualquier |
| **9. Video (opcional)**       | `hyperframes` + `remotion` |          — | Fase 3    |
| **10. 3D Avanzado (opcional)**| `threejs-*` + `shader-dev` |          7 | Fase 3    |
| **11. SEO + Mobile (opcional)**| `roier-seo` + `apple-hig` |          — | Fase 4    |
| **12. Branding / Arte**       | `brandkit` + `canvas-design` + `algorithmic-art` + `theme-factory` | — | Pre-Fase→2 |
| **13. Open Design Skills**    | 134 skills (ver SKILL.md)  |          — | Variable  |
| **Meta**                      | `vanta-design-orchestrator` |          — | Siempre   |
|                               | **Total skills**            | **170+** |           |

---

## Guía de Uso por Skill

### CAPA 1 — Fundaciones y Tokens

#### `ui-ux-pro-max`

**Motor de estilos y tokens programático.**

Contiene 50 estilos de diseño, 21 paletas de color, 50 parejas tipográficas y scripts Python para búsqueda.

```bash
# Buscar un estilo de diseño
python skills/ui-ux-pro-max/scripts/search.py "cinematic dark database" --design-system

# Persistir como archivo maestro
python skills/ui-ux-pro-max/scripts/search.py "cinematic dark" --design-system --persist
```

**Cuándo usarlo**: Primera skill que se consulta al iniciar un proyecto o rediseño. Define la paleta, tipografías y estilo base.

---

#### `design-systems`

**Arquitectura completa de design systems.**

10 sub-skills: `design-token`, `component-spec`, `accessibility-audit`, `theming-system`, `motion-system`, `naming-convention`, `pattern-library`, `icon-system`, `documentation-template`, `localization-design`, `design-system-governance`.

**Jerarquía de tokens**:

```
Global tokens (raw)     →  blue-500: #3B82F6
  ↓
Alias tokens (semantic) →  color-action-primary: var(--blue-500)
  ↓
Component tokens        →  button-color-primary: var(--color-action-primary)
```

**Convención de nombres**: `{category}-{property}-{variant}-{state}`

**Cuándo usarlo**: Después de `ui-ux-pro-max`, para formalizar tokens en un sistema versionable con semver.

---

### CAPA 2 — Estructura y Usabilidad

#### `ux-heuristics`

**Las 10 de Nielsen + Krug.**

Checklist evaluable de severidad 0-4:

| Severidad | Significado                           |
| :-------- | :------------------------------------ |
| 0         | No es problema de usabilidad          |
| 1         | Cosmético — corregir si hay tiempo    |
| 2         | Menor — prioridad baja                |
| 3         | Mayor — importante corregir           |
| 4         | Catástrofe — corregir antes de lanzar |

**Trunk Test de Krug**: ¿El usuario puede identificar al instante?

1. ¿Qué sitio es? (identidad)
2. ¿En qué página/sección está?
3. ¿Cuáles son las opciones principales?
4. ¿Cómo busca?

**Regla de Krug**: Elimina la mitad de las palabras. Luego elimina la mitad de lo que queda.

**Cuándo usarlo**: Antes de pasar a la fase visual. Si no pasa el Trunk Test, se reestructura.

---

#### `frontend-design`

**Estructuración limpia de componentes.**

Anti-patrones que detecta y rechaza:

- Divitis (nesting excesivo de `<div>`)
- Layouts de 3 columnas iguales sin tensión visual
- Hero sections sin whitespace dramático
- Composiciones simétricas predecibles

Patrones que promueve:

- Bento grids asimétricos
- Composiciones 60/40 o 70/30
- Whitespace como elemento de diseño activo
- HTML5 semántico (`<article>`, `<section>`, `<aside>`, `<nav>`)

**Cuándo usarlo**: Al escribir la primera línea de HTML/CSS de cualquier componente.

---

#### `ux-strategy`

**10 sub-skills estratégicas.**

| Sub-skill                  | Propósito                                                        |
| :------------------------- | :--------------------------------------------------------------- |
| `competitive-analysis`     | Benchmark UX contra competidores                                 |
| `design-principles`        | Principios que resuelven debates                                 |
| `design-brief`             | Brief con problema, audiencia, constraints, criterios            |
| `information-architecture` | Sitemap, taxonomía, modelo de contenido                          |
| `content-strategy`         | Auditoría, modelo, voz/tono, governance                          |
| `experience-map`           | Mapa multi-canal de todo el ecosistema                           |
| `metrics-definition`       | KPIs con HEART framework                                         |
| `north-star-vision`        | Visión a 1/3/5 años                                              |
| `opportunity-framework`    | Priorización con RICE, Kano, Impact-Effort                       |
| `service-blueprint`        | 5 swim lanes: evidence → user → frontstage → backstage → support |

**Cuándo usarlo**: Antes de diseñar pixeles. Discovery puro.

---

### CAPA 3 — Diseño Visual y Composición

#### `ui-design`

**13 sub-skills de craft visual.**

| Sub-skill              | Regla clave                                                            |
| :--------------------- | :--------------------------------------------------------------------- |
| `color-system`         | Escalas tonales 50-950. Contraste 4.5:1 body, 3:1 large text           |
| `typography-scale`     | Ratio modular 1.25+. Mínimo 16px body. 4-5 tamaños en uso regular      |
| `spacing-system`       | Base 4px o 8px. Escala nombrada xs→3xl                                 |
| `layout-grid`          | 4/8/12 columnas. Gutters 16/24/32px                                    |
| `responsive-design`    | Mobile-first. Breakpoints: 375/640/1024/1440                           |
| `visual-hierarchy`     | Squint test: jerarquía visible al desenfocar                           |
| `dark-mode-design`     | No invertir — rediseñar superficies. Off-white `#E0E0E0` no pure white |
| `readable-measure`     | 45-75 caracteres por línea. `max-width: 65ch`                          |
| `data-visualization`   | Data-ink ratio máximo. Colorblind-safe palettes                        |
| `law-of-proximity`     | Espacio dentro < espacio entre                                         |
| `law-of-common-region` | Usar el contenedor más débil que funcione                              |
| `von-restorff-effect`  | Un solo elemento diferenciado por pantalla                             |
| `aesthetic-usability`  | Consistencia como señal de calidad                                     |

**Cuándo usarlo**: Fase 2 — Después de la estructura, para aplicar identidad visual.

---

#### `visual-critique`

**4 dimensiones de crítica.**

```
Jerarquía Visual → Entry point, eye flow, weight, emphasis
Consistencia de Marca → mood.md, voice.md, tokens.md
Composición → Balance, whitespace, ritmo, gestalt
Tipografía → Escala, legibilidad, consistencia, tokens
```

Cada hallazgo sigue: **Observación** → **Problema** → **Fix**
Rating por dimensión: `pass` / `minor issue` / `major issue`

**Cuándo usarlo**: Fase 4 — Auditoría post-implementación.

---

#### `awesome-claude-design`

**Anti-slop y familias estéticas.**

**Slop test — señales de alerta**:

- ❌ Gradientes morado→azul ubicuos
- ❌ Inter/Roboto como única fuente en todo
- ❌ Tarjetas sobre tarjetas sobre tarjetas
- ❌ Sombras `box-shadow: 0 4px 6px rgba(0,0,0,0.1)` en todo
- ❌ Border-radius `8px` universal sin variación
- ❌ Layouts simétricos de 3 columnas iguales
- ❌ CTAs genéricos "Get Started", "Learn More"

**Cuándo usarlo**: Fase 2-3 — Al definir identidad y al implementar WebGL.

---

### CAPA 4 — Interacciones y Movimiento

#### `emil-design-eng`

**Filosofía de microinteracciones de Emil Kowalski.**

**Reglas fundamentales**:

- Las buenas animaciones son invisibles — los usuarios solo notan las malas
- Respuesta física inmediata: <100ms visual feedback al input
- Duración estándar: 150-300ms. Máximo: 500ms para transiciones de página
- Easing: `cubic-bezier(0.2, 0, 0, 1)` (standard) o spring
- **Prohibido**: `ease-in` en UI (se siente lento). `scale(1.1)` en hover (agresivo)
- **Preferido**: `scale(1.02)` + shadow sutil para hover

**Cuándo usarlo**: Fase 3 — Al diseñar cualquier elemento interactivo.

---

#### `animejs`

**Animaciones de timeline, SVG morphing y stagger avanzado.**

Librería ligera (~9KB gzipped) para animaciones JavaScript sin dependencias de framework.

**Cuándo usar animejs vs motion (motion.dev):**

| Escenario | Librería recomendada |
| :-------- | :------------------- |
| Hover/layout/scroll animations UI | `motion` |
| Timelines multi-paso con posicionamiento relativo | `animejs` |
| SVG path morphing y draw animations | `animejs` |
| Stagger grid desde centro con easing por elemento | `animejs` |
| Easing spring físico avanzado | `animejs` |
| Animaciones declarativas en React | `motion` |
| `prefers-reduced-motion` automático | `motion` |

```bash
npm install animejs
```

**Importación**:
```javascript
import anime from 'animejs'
```

**Timeline básico**:
```javascript
anime.timeline({
  easing: 'easeOutExpo',
  duration: 500
})
.add({ targets: '.el', translateX: 250 })
.add({ targets: '.el', rotate: '1turn' }, '-=300')
.add({ targets: '.el', scale: 1.5 }, '-=200')
```

**Stagger desde centro**:
```javascript
anime({
  targets: '.grid-item',
  scale: [0, 1],
  delay: anime.stagger(50, { from: 'center' })
})
```

**Cuándo usarlo**: Fase 3 — Para animaciones que requieren coreografía temporal, SVG morphing, o stagger sobre colecciones.

---

#### `interaction-design`

**13 sub-skills con base en ciencia cognitiva.**

**Leyes cognitivas aplicadas**:

| Ley                       | Regla de diseño                                                                        |
| :------------------------ | :------------------------------------------------------------------------------------- |
| **Doherty** (<400ms)      | Feedback visual <100ms. Loading indicator >400ms. Progress >3s                         |
| **Fitts** (target sizing) | Touch: 44×44pt mínimo. Pointer: 24×24px mínimo. Bordes de pantalla = targets infinitos |
| **Hick** (decisión)       | Reducir opciones simultáneas. Agrupar antes de eliminar. Pricing: 3 tiers max          |
| **Miller** (chunking)     | Agrupar en bloques de 4±1. Forms: secciones lógicas. Phone: `555-867-5309`             |

**State machines para UI**:

```
Form:   idle → editing → validating → submitting → success/error → idle
Fetch:  idle → loading → success/error, error → retrying → success/error
Auth:   logged-out → authenticating → logged-in → logging-out → logged-out
```

**Cuándo usarlo**: Fase 3 — Al implementar comportamientos interactivos.

---

### CAPA 5 — Auditoría y Refinamiento

#### `impeccable`

**CLI de auditoría con 23 comandos y 41 reglas anti-slop.**

```bash
/impeccable craft <target>    # Construir un componente desde cero
/impeccable shape <target>    # Dar forma a un componente existente
/impeccable audit <target>    # Auditar contra las 41 reglas
/impeccable polish <target>   # Refinamiento final de detalles
```

**Flujo obligatorio antes de producción**: `audit` → fix → `polish` → deploy.

**Cuándo usarlo**: Fase 4 — Maquetación media y finalización.

---

#### `web-design-guidelines` + `writing-guidelines`

**Compliance checks de Vercel.**

Ambas skills descargan las guidelines actualizadas desde los repos oficiales y aplican reglas en formato `file:line`.

- `web-design-guidelines` → accesibilidad, performance, SEO, responsive
- `writing-guidelines` → voz, tono, claridad, consistencia de prosa

**Cuándo usarlos**: Fase 4 — Gate de calidad final.

---

### CAPA 6 — Rendimiento y Optimización

#### `react-best-practices`

**70 reglas en 8 categorías por prioridad.**

| Prioridad | Categoría            | Prefijo      | Impacto     |
| :-------- | :------------------- | :----------- | :---------- |
| 1         | Eliminar waterfalls  | `async-`     | CRITICAL    |
| 2         | Bundle size          | `bundle-`    | CRITICAL    |
| 3         | Server-side          | `server-`    | HIGH        |
| 4         | Client data fetching | `client-`    | MEDIUM-HIGH |
| 5         | Re-renders           | `rerender-`  | MEDIUM      |
| 6         | Rendering            | `rendering-` | MEDIUM      |
| 7         | JavaScript           | `js-`        | LOW-MEDIUM  |
| 8         | Advanced             | `advanced-`  | LOW         |

**Top 5 reglas de mayor impacto**:

1. `async-parallel` → `Promise.all()` para operaciones independientes
2. `bundle-barrel-imports` → Importar directo, evitar barrel files
3. `async-suspense-boundaries` → Usar Suspense para streaming
4. `bundle-dynamic-imports` → `next/dynamic` para componentes pesados
5. `server-cache-react` → `React.cache()` para deduplicación per-request

**Cuándo usarlo**: Fase 3-4 — Al escribir y revisar componentes.

---

#### `vercel-optimize`

**Pipeline de auditoría de producción.**

Requiere: Vercel CLI v53+, proyecto linkeado, Node.js 20+.

Pipeline: `collect → scan → merge → gate → deep-dive → reconcile → verify → render-report`

**Cuándo usarlo**: Post-producción — Solo sobre proyectos desplegados con tráfico real.

---

### CAPA 7 — Investigación y Metodología

#### `design-research`

**10 sub-skills de investigación de usuario.**

| Sub-skill             | Método                    | Artefacto                               |
| :-------------------- | :------------------------ | :-------------------------------------- |
| `user-persona`        | Entrevistas + analytics   | Persona profile                         |
| `empathy-map`         | Investigación cualitativa | 4 cuadrantes: Says/Thinks/Does/Feels    |
| `journey-map`         | Observación + entrevistas | Mapa 5-7 stages con emociones           |
| `interview-script`    | Preparación               | Script con warm-up/core/wrap-up         |
| `usability-test-plan` | Planificación             | Plan con tareas, métricas, facilitación |
| `card-sort-analysis`  | Análisis                  | Similarity matrix + IA recomendada      |
| `affinity-diagram`    | Síntesis                  | Clusters bottom-up con insights         |
| `jobs-to-be-done`     | Framework JTBD            | Mapa funcional/emocional/social         |
| `diary-study-plan`    | Longitudinal              | Plan 1-4 semanas                        |
| `survey-design`       | Cuantitativa              | Encuesta con scales validadas           |

**Cuándo usarlo**: Pre-Fase — Antes de cualquier diseño significativo.

---

#### `prototyping-testing`

**8 sub-skills de validación.**

| Sub-skill                 | Propósito                                  |
| :------------------------ | :----------------------------------------- |
| `prototype-strategy`      | Elegir fidelidad correcta para la pregunta |
| `heuristic-evaluation`    | Nielsen 10 con severidad 0-4               |
| `a-b-test-design`         | Hipótesis + variantes + sample size        |
| `accessibility-test-plan` | 4 capas: automated → manual → AT → user    |
| `click-test-plan`         | First-click success rate                   |
| `test-scenario`           | Tareas + criterios + guía de observación   |
| `user-flow-diagram`       | Happy path + error paths + exits           |
| `wireframe-spec`          | Layout anotado con comportamiento          |

**Cuándo usarlo**: Entre Fase 2 y Fase 3.

---

### CAPA 8 — Operaciones y Herramientas de Equipo

#### `design-ops`

**7 sub-skills operacionales.**

| Sub-skill                 | Entregable                                                  |
| :------------------------ | :---------------------------------------------------------- |
| `design-critique`         | Sesión estructurada con formato I notice/I wonder/What if   |
| `design-debt-audit`       | Registro con Severidad × Frecuencia / Esfuerzo              |
| `design-impact-reporting` | Before/After + A/B summary + scorecard                      |
| `design-qa-checklist`     | Visual/Layout/Interaction/Content/A11y/Cross-platform       |
| `design-review-process`   | 4 gates: Concept → Design → Pre-Handoff → Implementation QA |
| `handoff-spec`            | Specs: visual + interaction + content + assets + edge cases |
| `design-sprint-plan`      | 5 días: Understand → Diverge → Decide → Prototype → Test    |

**Cuándo usarlo**: Fase 4 y post-producción.

---

#### `designer-toolkit`

**7 sub-skills utilitarias.**

| Sub-skill                | Uso                                                              |
| :----------------------- | :--------------------------------------------------------------- |
| `design-rationale`       | Decision → Context → Options → Evidence → Reasoning → Trade-offs |
| `ux-writing`             | Microcopy, errors, empty states, CTAs, onboarding                |
| `case-study`             | Challenge → Process → Solution → Impact → Reflection             |
| `design-negotiation`     | Advocacy basada en evidencia, no en preferencia                  |
| `presentation-deck`      | Hook → Context → Journey → Solution → Evidence → Ask             |
| `design-system-adoption` | Awareness → Education → Enablement → Incentives                  |
| `design-token-audit`     | Inventory → Categorize → Map → Flag → Recommend                  |

**Cuándo usarlo**: Cualquier fase, según necesidad.

---

## Ciclo de Orquestación

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  PRE-FASE: Discovery & Research                                     │
│  ├─ design-research  (personas, JTBD, journeys)                    │
│  ├─ ux-strategy      (IA, competitive, principles)                  │
│  └─ prototyping-testing (prototype strategy)                        │
│                                                                     │
│  FASE 1: Estructura y Usabilidad                                    │
│  ├─ ux-heuristics    (Trunk Test, Nielsen 10)                       │
│  ├─ frontend-design  (HTML semántico, anti-divitis)                 │
│  └─ interaction-design/navigation-patterns, form-design             │
│                                                                     │
│  FASE 2: Identidad Visual y Tokens                                  │
│  ├─ ui-ux-pro-max    (paleta HSL, tipografías)                      │
│  ├─ design-systems   (tokens, temas, motion)                        │
│  ├─ ui-design        (color, type, grid, spacing)                   │
│  └─ awesome-claude-design (anti-slop, familia estética)             │
│                                                                     │
│  FASE 3: Interacciones y 3D                                         │
│  ├─ emil-design-eng  (springs, micro-detalles)                      │
│  ├─ animejs          (timelines, SVG morph, stagger)                │
│  ├─ interaction-design (leyes cognitivas, FSM, loading)             │
│  └─ awesome-claude-design (WebGL/Shaders 60fps)                     │
│                                                                     │
│  FASE 4: Auditoría y Refinamiento                                   │
│  ├─ impeccable       (audit → polish → slop test)                   │
│  ├─ visual-critique  (4 dimensiones de crítica)                     │
│  ├─ web-design-guidelines + writing-guidelines                      │
│  ├─ react-best-practices (70 reglas)                                │
│  ├─ design-ops/qa-checklist                                         │
│  └─ vercel-optimize  (solo si está desplegado)                      │
│                                                                     │
│  → PRODUCCIÓN                                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Workflows Disponibles

Workflows encadenados que combinan múltiples sub-skills:

| Skill                 | Workflow              | Descripción                              |
| :-------------------- | :-------------------- | :--------------------------------------- |
| `design-systems`      | `:audit-system`       | Auditoría completa de design system      |
| `design-systems`      | `:create-component`   | Scaffold de spec de componente           |
| `design-systems`      | `:tokenize`           | Extraer tokens de stylesheet existente   |
| `ui-design`           | `:color-palette`      | Paleta completa con accessibility checks |
| `ui-design`           | `:design-screen`      | Diseñar pantalla desde requirements      |
| `ui-design`           | `:responsive-audit`   | Auditar responsive en breakpoints        |
| `ui-design`           | `:type-system`        | Sistema tipográfico completo             |
| `visual-critique`     | `:critique-screen`    | 4 críticas + fix list priorizada         |
| `interaction-design`  | `:design-interaction` | Flujo de interacción completo            |
| `interaction-design`  | `:error-flow`         | Flujo de manejo de errores               |
| `interaction-design`  | `:map-states`         | State machine para UI complejo           |
| `design-research`     | `:discover`           | Ciclo completo de user research          |
| `design-research`     | `:interview`          | Script o resumen de entrevista           |
| `design-research`     | `:synthesize`         | Síntesis en affinity diagram             |
| `design-research`     | `:test-plan`          | Plan de usability test                   |
| `prototyping-testing` | `:evaluate`           | Evaluación heurística                    |
| `prototyping-testing` | `:experiment`         | Diseño de A/B test                       |
| `prototyping-testing` | `:prototype-plan`     | Plan de prototipado                      |
| `prototyping-testing` | `:test-plan`          | Plan de testing completo                 |
| `ux-strategy`         | `:benchmark`          | Benchmarking competitivo                 |
| `ux-strategy`         | `:frame-problem`      | Estructurar design challenge             |
| `ux-strategy`         | `:strategize`         | Estrategia UX completa                   |
| `design-ops`          | `:handoff`            | Package de handoff a dev                 |
| `design-ops`          | `:plan-sprint`        | Planificar design sprint                 |
| `design-ops`          | `:setup-workflow`     | Configurar workflow de equipo            |
| `designer-toolkit`    | `:build-presentation` | Presentación estructurada                |
| `designer-toolkit`    | `:write-case-study`   | Case study de portfolio                  |
| `designer-toolkit`    | `:write-rationale`    | Rationale de decisiones                  |

---

## Reglas de Resolución de Conflictos

Cuando dos skills dan directrices contradictorias, se resuelven en este orden:

| Prioridad  | Dominio           | Skills Rectoras                                       |
| :--------- | :---------------- | :---------------------------------------------------- |
| 1 (máxima) | **Accesibilidad** | `design-systems/accessibility-audit`, `ux-heuristics` |
| 2          | **Rendimiento**   | `react-best-practices`, `vercel-optimize`             |
| 3          | **Usabilidad**    | `ux-heuristics`, `interaction-design`                 |
| 4          | **Anti-Slop**     | `impeccable`, `awesome-claude-design`, `animejs`      |
| 5          | **Estética**      | `ui-design`, `emil-design-eng`, `visual-critique`     |

> Si es accesible pero lento → se optimiza rendimiento.
> Si es bonito pero confuso → se reestructura usabilidad.
> Si es usable pero genérico → se refina estética.
> La accesibilidad **nunca** se sacrifica.

---

## Referencia Rápida

### Uso de Workflows Predefinidos

Además del ciclo genérico, el orquestador incluye **3 pipelines completos** en `workflows/`:

| Pipeline | Fases | Skills totales | Tiempo estimado |
|:---------|:-----:|:--------------:|:---------------:|
| `landing-page-pipeline.json` | 12 | 25+ | 2-4h |
| `brand-identity-pipeline.json` | 11 | 20+ | 3-6h |
| `video-production-pipeline.json` | 9 | 16+ | 1-3h |

Y **20 presets de proyecto** en `configs/project-presets.json` para cargar configuración rápida:

```bash
# Ejemplo: cargar preset para landing page
.agent\skills\vanta-design-orchestrator\scripts\skill-bridge.ps1 -Route landing-page
```

### Valores por defecto del sistema

| Propiedad                  | Valor                                      |
| :------------------------- | :----------------------------------------- |
| Color space                | OKLCH o HSL (no hex/rgb sin justificación) |
| Contraste mínimo texto     | 4.5:1 (AA)                                 |
| Contraste mínimo UI        | 3:1                                        |
| Fuente body mínimo         | 16px                                       |
| Línea óptima (measure)     | 45-75 caracteres                           |
| Touch target mínimo        | 44×44pt                                    |
| Pointer target mínimo      | 24×24px                                    |
| Feedback visual máximo     | <100ms                                     |
| Loading indicator          | >400ms (Doherty)                           |
| Duración animación UI      | 150-300ms                                  |
| Duración máxima transición | 500ms                                      |
| WebGL target               | 60fps constante                            |
| Spacing base               | 4px o 8px                                  |
| Easing estándar            | `cubic-bezier(0.2, 0, 0, 1)`               |
| Easing de entrada          | `cubic-bezier(0, 0, 0.2, 1)`               |
| Easing de salida           | `cubic-bezier(0.3, 0, 1, 0.3)`             |
| Easing prohibido en UI     | `ease-in`                                  |
| `prefers-reduced-motion`   | Obligatorio en toda animación              |

---

_Documento generado y mantenido por el Vanta Design Orchestrator._
_Última actualización: 2026-06-19._

## Archivos del Orquestador

| Archivo | Descripción |
|:--------|:------------|
| `SKILL.md` | Orquestador principal con 13 capas, routing, conflictos |
| `README.md` | Este archivo — visión general y referencia rápida |
| `configs/project-presets.json` | 20 presets de proyecto (landing, brand, video, deck, etc.) |
| `routing/ROUTING.md` | Tabla maestra de 95+ combinaciones de skills |
| `examples/examples.md` | 20 ejemplos prácticos de uso combinado |
| `workflows/landing-page-pipeline.json` | Pipeline landing page (12 fases) |
| `workflows/brand-identity-pipeline.json` | Pipeline brand identity (11 fases) |
| `workflows/video-production-pipeline.json` | Pipeline video production (9 fases) |
| `scripts/skill-bridge.ps1` | CLI para listar skills, rutas y conflictos |
| `scripts/skill-inventory.json` | Inventario completo de 170+ skills |
| `strategy/` | 13 documentos de estrategia (business-model, brand-platform, sensory-identity, etc.) |
| `infrastructure/` | Documentación de herramientas, analytics, developer handoff |
| `strategy/SUMMARY.md` | Índice rápido de los 13 documentos estratégicos |
