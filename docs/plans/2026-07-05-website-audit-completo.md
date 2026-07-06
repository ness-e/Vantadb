# Auditoría Completa — VantaDB Website V1

> Fecha: 2026-07-05
> Alcance: Código, diseño, contenido, herramientas, animaciones, arquitectura

---

## 1. VERIFICACIÓN DE CONTENIDO vs PROYECTO RUST

### 1.1 ¿Qué es VantaDB realmente?

VantaDB NO es una base de datos cliente-servidor. Es un **motor de memoria persistente y recuperación vectorial embebido** para aplicaciones AI local-first. Se linkea como librería en la aplicación host. Zero dependencias de red.

### 1.2 Errores GRAVES en la web

| Error | Dónde | Realidad | Severidad |
|-------|-------|----------|-----------|
| **"Sled" storage backend** | `docs.lazy.tsx` Migration Guide | No existe. Los backends reales son **Fjall, RocksDB, InMemory**. | 🔴 ALTA |
| **"Any ONNX model"** | `why-vantadb.lazy.tsx` | **CERO soporte ONNX.** No hay runtime, dependencia, ni código ONNX en el codebase. | 🔴 ALTA |
| **"LangChain + LlamaIndex" integraciones** | `pricing.lazy.tsx` | No existen como crates. Solo hay ejemplos Python sueltos. | 🔴 ALTA |
| **softwareVersion "0.1.5"** | `__root.tsx` (Schema.org) | Versión real: **0.2.0** | 🟡 MEDIA |
| **"Rust 1.75+"** | `docs.lazy.tsx` | Real: **1.94.1** | 🟡 MEDIA |
| **"Python 3.8+"** | README badges | Real: **3.11+** (abi3-py311) | 🟡 MEDIA |
| **Benchmarks inconsistentes** | `product/benchmarks` vs `latency` | Números diferentes en distintas páginas | 🟡 MEDIA |
| **Recall 24.5% no divulgado** | `product/benchmarks.lazy.tsx` | Benchmark competitivo muestra recall bajo vs LanceDB/ChromaDB | 🔴 ALTA |

### 1.3 Gap: Lo que el proyecto TIENE pero la web NO muestra

- 🔴 **Graph edges** (BFS, DFS, topological sort, DAG check) — no se menciona en ninguna página
- 🔴 **TurboQuant (3-bit) + RaBitQ (1-bit)** quantization — más avanzado que SQ8, no documentado
- 🔴 **Predictive kernel prefetching** (madvise / PrefetchVirtualMemory) — diferencial técnico importante
- 🔴 **Schema evolution tests** — no se menciona compatibilidad hacia adelante
- 🔴 **Multi-process file locking** — diferencial vs otras embedded DBs
- 🔴 **Tantivy tokenizer** (feature-gated) — búsqueda de texto avanzada no documentada
- 🔴 **Mem0, Letta, CrewAI, DSPy, Haystack, LiteLLM** — son 6 adapters AI que existen como workspace members pero apenas se mencionan

### Acción Requerida: Corrección de Contenido

1. Eliminar todas las claims falsas (ONNX, Sled, LangChain/LlamaIndex)
2. Actualizar versión a 0.2.0 en schema.org
3. Corregir Rust 1.75+ → 1.94.1
4. Corregir Python 3.8+ → 3.11+
5. Unificar benchmarks entre páginas (usar una sola fuente de verdad)
6. Agregar transparencia en benchmarks competitivos
7. Agregar features faltantes: graph edges, quantization avanzada, prefetching, adapters AI

---

## 2. ARQUITECTURA DE CÓDIGO

### 2.1 Estructura Actual

```
web/src/
├── components/
│   ├── nb/            (18 primitives de design system)
│   └── *.tsx          (8 componentes compuestos + layout)
├── styles/            (46 archivos CSS — demasiados)
├── hooks/             (7 hooks)
├── lib/               (5 archivos + dir api/ vacío)
├── routes/            (27 rutas enflat routing tree)
└── SourceDesign/      (41 imágenes de referencia — MAL UBICADO)
```

### 2.2 Problemas Críticos

| Problema | Detalle |
|----------|---------|
| **CSS duplicado** | `.nb-section-header` definido en 2 archivos, keyframes de animación duplicados |
| **CSS no utilizado** | ~15 clases definidas pero nunca referenciadas (.nb-tactile-*, .nb-step-*, etc.) |
| **Directorio plano de componentes** | 26 archivos en `components/` sin subdirectorios — no escala |
| **Sin orden de imports** | No hay regla de lint para import ordering |
| **SourceDesign/ en src/** | 41 imágenes de referencia mezcladas con código fuente |
| **Sin tipos compartidos** | No hay directorio `types/` — interfaces duplicadas |
| **Sin layout routes** | Las 27 rutas son hijas planas de `__root.tsx` |
| **Tailwind infrautilizado** | Todo el styling es CSS manual, Tailwind solo para `@theme` |

### 2.3 Nombres y Organización

✅ PascalCase para componentes, camelCase para hooks/libs, kebab-case para CSS
✅ Barrel exports desde `nb/index.ts`
✅ Route definition + lazy component pattern (TanStack Router)
❌ `VantaDBLogo` export default (único default entre 26 componentes)
❌ Sin `@/` path alias usado consistentemente (mezcla de relativo y alias)
❌ `lib/api/` directorio vacío

### 2.4 Dependencias

| Paquete | Versión | Bundle | Estado |
|---------|---------|--------|--------|
| React | 19.2.0 | ~40KB | ✅ Latest |
| TailwindCSS | 4.3.2 | Build-time | ✅ Latest |
| GSAP | 3.15.0 | ~100KB gzip | ✅ Latest, pero es el mayor contribuyente |
| TanStack Router | 1.168.25 | ~40KB | ✅ Latest |
| TanStack Query | 5.101.2 | ~15KB | ✅ Latest |
| Vite | 8.1.3 | Build-time | ✅ Latest |
| @fontsource fonts | 3 fonts | ~500KB total | ⚠️ Peso alto, considerar subsetting |
| `rollup-plugin-visualizer` | 7.0.1 | Build-time | ❌ En `dependencies`, debe ser `devDependencies` |
| `tw-animate-css` | 1.3.4 | ~5KB | ⚠️ Valor limitado (mayoría animaciones son custom) |

### Acción Requerida: Refactor de Código

1. **Mover SourceDesign/** a `docs/references/` o raíz del proyecto
2. **Eliminar CSS duplicado y no utilizado**
3. **Decidir: Tailwind vs CSS puro** — o se usa Tailwind para todo o se elimina
4. **Agregar regla de import ordering** (eslint-plugin-import)
5. **Agregar directorio `types/`** para interfaces compartidas
6. **Consolidar 46 CSS → ~20** (agrupar por sección)
7. **Mover `rollup-plugin-visualizer`** a devDependencies
8. **Evaluar si GSAP puede reducirse** (~100KB es mucho para un site marketing)

---

## 3. ANIMACIONES Y EFECTOS VISUALES

### 3.1 Lo que YA existe (bien implementado)

| Feature | Implementación | Calidad |
|---------|---------------|---------|
| ScrollTrigger section reveals | useGSAP + gsap.context() | ✅ |
| Terminal boot sequence | setTimeout chain | ✅ |
| Benchmark bar fill | GSAP to() width | ✅ |
| Count-up numbers | useCountUp (RAF) | ✅ |
| Text scramble | useTextScramble (GSAP) | ✅ |
| Cursor blink, marquee, glitch | CSS keyframes | ✅ |
| Noise overlay, scanlines | SVG feTurbulence + CSS | ✅ |
| prefers-reduced-motion | gsap.matchMedia() en useAnimationSafe | ✅🌟 |

### 3.2 Lo que NO existe (pero GSAP ya está instalado — GRATIS)

| Técnica | Plugin | Impacto | Dificultad |
|---------|--------|---------|------------|
| **Text splitting** (word/char reveal) | split-type (~5KB) o fetta (~3KB) | 🔥 Alto visual | Baja |
| **DrawSVG** (line drawing en SVG grid) | GSAP DrawSVGPlugin (gratis) | 🔥 Alto | Baja |
| **ScrollTrigger pin + scrub** (horizontal scroll) | GSAP ScrollTrigger (ya instalado) | 🔥 Alto | Media |
| **Parallax sutil** (hero grid) | GSAP yPercent scrollTrigger | 🔥 Alto | Baja |
| **MorphSVG** (logo transitions) | GSAP MorphSVGPlugin (gratis) | 🟡 Medio | Media |
| **Flip** (layout animations) | GSAP FlipPlugin (gratis) | 🟡 Medio | Media |
| **MotionPath** (data flow animation) | GSAP MotionPathPlugin (gratis) | 🟡 Medio | Media |

**Costo adicional: $0**. Todos los plugins GSAP son gratuitos desde 2024.

### 3.3 SVG y Gráficos

| Técnica | Estado | Recomendación |
|---------|--------|---------------|
| SVG grid overlay en hero | ✅ Existe (estático) | Agregar DrawSVG + stroke-dasharray reveal |
| Logo VantaDB | ✅ Estático | Agregar SMIL animate en load |
| Iconos decorativos | ❌ No existen | Crear SVG inline geométricos (brand-aligned) |
| Force-directed graph | ❌ No existe | d3-force (~8KB) para /architecture |
| Benchmark charts | ⚠️ Barras CSS básicas | observable-plot (~45KB) o D3 |
| Diagramas interactivos | ❌ No existen | Mermaid.js (~55KB, lazy-load) |
| Logos de tecnologías reales | ❌ Text labels | simple-icons (CC0, tree-shakeable, ~0.5KB/icono) |

### 3.4 Referencias de Ilustración con HTML/CSS/JS

| Categoría | Librería | Bundle | Uso |
|-----------|----------|--------|-----|
| CSS Puro | — | 0KB | Formas geométricas, gradientes, pseudo-elementos |
| SVG Inline | — | 0KB | Logos, iconos, diagramas técnicos |
| Canvas API | — | 0KB | Partículas ligeras, datos en tiempo real |
| Pseudo-3D | Zdog | ~7KB | Efecto 2.5D para hero/logo (opcional) |
| Escena gráfica 2D | Two.js | ~27KB | Animaciones vectoriales complejas (si se necesitan) |
| Partículas | Canvas API custom | 0KB | Sistema propio — sin librería, control total |
| Charts | observable-plot | ~45KB | Benchmarks, latency, cost comparison |
| Force graph | d3-force | ~8KB | Arquitectura VantaDB como grafo interactivo |
| Diagramas | Mermaid.js | ~55KB | Diagramas de flujo en docs (lazy-load) |

### Acción Requerida: Plan de Animaciones

**Fase 1 (Alto impacto, 0-1 día):**
1. Agregar `split-type` para text reveals en Hero + headers
2. Activar DrawSVG en grid overlay del hero
3. Agregar parallax sutil en hero (ScrollTrigger scrub, ya instalado)
4. Agregar `@observablehq/plot` para benchmarks

**Fase 2 (Medio, 2-3 días):**
5. Force-directed graph (d3-force) en /architecture
6. Diagramas Mermaid en documentación
7. MorphSVG en logo (transiciones entre variantes)
8. Logos reales de tecnologías (simple-icons)

**Fase 3 (Bajo, futuro):**
9. Horizontal scroll section (benchmarks)
10. Interactive terminal demo
11. Page transitions con TanStack Router + GSAP

---

## 4. DISEÑO Y COMPONENTES

### 4.1 Design System Score

| Componente | Score | Fortaleza | Debilidad |
|------------|-------|-----------|-----------|
| NbSection | 9/10 | forwardRef, semantic | — |
| NbNoise | 10/10 | Perfect scope | — |
| NbCursor | 10/10 | Perfect scope | — |
| NbBlockAmber | 9/10 | Minimal | — |
| NbIconBox | 9/10 | Focused | — |
| NbArrow | 8/10 | Polymorphic | Under-documented |
| NbBento | 8/10 | Grid abstraction | Limited columns |
| NbCard | 8/10 | Semantic variants | — |
| NbCopyCommand | 8/10 | Clipboard UX | i18n limit |
| NbLogLine | 8/10 | Data-attr styling | Presentational |
| NbMetric | 8/10 | Clean display | — |
| NbSectionHeader | 8/10 | Semantic h2 | — |
| NbAccordion | 7/10 | Generic render-prop | No keyboard nav |
| NbButton | 7/10 | Polymorphic | No disabled/loading |
| NbDitherImage | 7/10 | alt + lazy | CLS risk |
| NbSplitFlap | 7/10 | Decorative | Sin animación real |
| NbMarquee | 5/10 | Duplicate technique | No reduced-motion |
| **Promedio** | **7.9/10** | | |

### 4.2 Recomendación: Framework de Componentes

**Decisión: Agregar shadcn/ui sobre el stack actual**

| Factor | shadcn/ui | Radix UI solo | Headless UI |
|--------|-----------|---------------|-------------|
| Bundle | 0KB (código en tu repo) | ~3-5KB/componente | ~4KB/componente |
| Accesibilidad | ★★★★★ (Radix) | ★★★★★ | ★★★★☆ |
| Tailwind | Nativo | Compatible | Nativo |
| Componentes | 50+ | 35+ | 15 |

**Por qué shadcn/ui:**
- Zero runtime dependencies (copias el código, es tuyo)
- Accesibilidad Radix sin pagar el bundle de Radix importado
- Componentes pre-estructurados que puedes recolorear a tus tokens
- Ya usas Tailwind v4 + clsx + tailwind-merge — shadcn usa el mismo stack

**Instalación recomendada:**
```bash
npx shadcn@latest init
npx shadcn@latest add button dialog dropdown-menu tabs tooltip sheet accordion navigation-menu
```

### 4.3 Documentación de Componentes

| Herramienta | Bundle | Cold Start | Veredicto |
|-------------|--------|------------|-----------|
| **Ladle** | ~2MB | **1.2s** | ✅ Empezar aquí |
| Storybook 8 | ~20MB | 8-20s | ⏳ Agregar cuando >50 componentes |
| Histoire | ~3MB | ~2s | Vue-first (irrelevante) |

### 4.4 Nuevos Patrones de Diseño

| Patrón | Estado | Prioridad |
|--------|--------|-----------|
| Micro-interacciones mecánicas | ✅ En CSS | — |
| Terminal aesthetic | ✅ Hero | Mejorar con boot secuencia animada |
| Swiss grid con líneas visibles | ✅ En diseño | — |
| Data viz dark mode | ❌ No existe | 🔥 Alta (benchmarks) |
| Scroll-linked storytelling | ⚠️ Básico | 🟡 Medio (pin + scrub) |
| Carga con personalidad (skeleton) | ❌ No existe | 🟡 Medio |
| Transiciones de página | ❌ No existe | 🟡 Medio |
| Hero interactivo (terminal real) | ❌ No existe | 🔵 Bajo (opcional) |

---

## 5. HERRAMIENTAS Y WORKFLOW

### 5.1 MCP Servers Actuales (6)

| MCP | Propósito |
|-----|-----------|
| CodeGraph | Índice de código (7.3K símbolos) |
| Pencil | Editor de archivos .pen (diseño UI) |
| Playwright | Automatización de navegador |
| Recraft | Generación de imágenes AI |
| cargo-mcp | Operaciones Cargo |
| rust-analyzer-mcp | LSP Rust |

### 5.2 MCPs Recomendados para Agregar

| MCP | Comando | Propósito | Prioridad |
|-----|---------|-----------|-----------|
| **lighthouse-mcp** | `npx @danielsogl/lighthouse-mcp` | Auditoría performance + accesibilidad | 🔥 Alta |
| **shadcn-ui-mcp** | `npx @jpisnice/shadcn-ui-mcp-server` | Componentes shadcn (source + install) | 🔥 Alta |
| **seo-mcp** | `npx mcp-seo` | SEO técnico + validación structured data | 🔥 Alta |
| tailwindcss-mcp | `npx clarity-contrib/tailwindcss-mcp-server` | Sugerencia/validación clases Tailwind | 🟡 Media |
| DesignMCP | `npx cdej-lgtm/designmcp-server` | Generación tokens + temas shadcn | 🟡 Media |

### 5.3 Skills a Usar por Tipo de Tarea

| Tarea | Skills Chain |
|-------|-------------|
| **Nueva sección landing** | `vanta-design-orchestrator` → `brainstorming` → `writing-plans` → `design-taste-frontend` → `impeccable` → `motion` |
| **Corregir bug UI** | `systematic-debugging` → `writing-plans` → `impeccable` → fix |
| **Auditoría SEO** | `ai-seo` → `seo-audit` → `audit-website` (230+ reglas) |
| **Rediseño visual** | `brandkit` → `color-expert` → `theme-factory` → `platform-design` → `impeccable` |
| **Performance** | `vercel-react-best-practices` → `vercel-optimize` |
| **Animación** | `motion` (preferido) o `gsap-core` + `gsap-scrolltrigger` + `emil-design-eng` |
| **Review diseño** | `impeccable` (UI) + `react-dev` (código) + `web-design-guidelines` (accesibilidad) |
| **Artículo blog** | `writing-guidelines` → `article-magazine` |

### 5.4 Flujo de Sub-agentes Recomendado

**Patrón Generation + Review:**
```
Agent 1 (Diseño):   vanta-design-orchestrator → design-taste-frontend → impeccable
                     → Genera componente/página → Output: código

Agent 2 (Review):    impeccable → web-design-guidelines → plan-design-review
                     → Rate 0-10, flag AI slop → Output: critique

Agent 3 (Polish):    emil-design-eng → interaction-design → motion
                     → Micro-interacciones, transiciones → Output: código final
```

**Patrón Bug Fix:**
```
Agent 1 (Debug):     systematic-debugging → Root cause
Agent 2 (Fix):       writing-plans → Implementar fix
Agent 3 (Verify):    cargo test + Playwright e2e → Green CI
```

### 5.5 Estado de Configuración Actual

| Archivo | Propósito | Estado |
|---------|-----------|--------|
| `opencode.json` | Config MCP + skills | ✅ Existe |
| `.opencode/AGENTS.md` | Instrucciones para agente | ❌ Solo cubre Rust backend, falta sección web |
| `.github/` | CI/CD | No verificado |
| `vercel.json` | Deploy config | ✅ En web/ |
| `.prettierrc` | Formatter | ✅ |
| `eslint.config.js` | Linter | ⚠️ unused vars rule OFF |

### 5.6 Skills Específicas para Features de Diseño

- **`design-brief`**: Convertir requerimientos vagos en spec concreto (I-Lang protocol)
- **`design-md`**: Crear DESIGN.md como fuente de verdad visual
- **`reference-design-contract`**: Convertir referencias visuales en spec grounded
- **`color-expert`**: Ciencia del color OKLCH (286K words de referencia)
- **`visual-review`**: Pipeline de auditoría visual con Playwright + pixelmatch
- **`screenshots-marketing`**: Generar screenshots para landing pages con Playwright
- **`imagegen-frontend-web`**: 1 imagen por sección, composición variada

---

## 6. PLAN DE ACCIÓN PRIORIZADO

### 🔴 Prioridad Alta (Corregir ahora)

| # | Acción | Archivos | Tiempo |
|---|--------|----------|--------|
| 1 | Eliminar claims falsas: ONNX, Sled, LangChain | `why-vantadb.lazy.tsx`, `docs.lazy.tsx`, `pricing.lazy.tsx` | 1h |
| 2 | Actualizar versión 0.1.5 → 0.2.0 en schema.org | `__root.tsx` | 5min |
| 3 | Corregir Rust 1.75+ → 1.94.1 | `docs.lazy.tsx` | 5min |
| 4 | Corregir Python 3.8+ → 3.11+ | README, docs | 5min |
| 5 | Agregar features faltantes: graph, quantization, prefetching, adapters AI | Routes relevantes | 4h |
| 6 | Agregar `split-type` para text reveals | Hero + section headers | 2h |
| 7 | Activar DrawSVP para grid reveal | Hero SVG | 1h |
| 8 | Agregar logos reales con simple-icons | NbEcosystem, NbTrustBar | 2h |
| 9 | Eliminar CSS duplicado y no utilizado | `nb-base.css`, `nb-components.css` | 2h |
| 10 | Mover `rollup-plugin-visualizer` a devDependencies | `package.json` | 5min |

### 🟡 Prioridad Media (Siguiente sprint)

| # | Acción | Tiempo |
|---|--------|--------|
| 11 | Agregar shadcn/ui + migrar NbAccordion, NbButton | 4h |
| 12 | Configurar Ladle para documentación de componentes | 1h |
| 13 | Agregar lighthouse-mcp, shadcn-ui-mcp, seo-mcp a `opencode.json` | 30min |
| 14 | Agregar sección web en `.opencode/AGENTS.md` | 1h |
| 15 | Mover SourceDesign/ a `docs/references/` | 15min |
| 16 | Unificar benchmarks entre páginas | 2h |
| 17 | Agregar ScrollTrigger pin + scrub para feature walkthrough | 3h |
| 18 | Agregar `@observablehq/plot` para charts de benchmarks | 4h |
| 19 | Agregar d3-force graph para /architecture | 4h |
| 20 | Agregar tipos compartidos en `types/` | 2h |
| 21 | Agregar regla de import ordering | 30min |

### 🔵 Prioridad Baja (Futuro)

| # | Acción | Tiempo |
|---|--------|--------|
| 22 | Page transitions con TanStack Router + GSAP | 4h |
| 23 | Mermaid.js diagramas en documentación | 3h |
| 24 | Style Dictionary pipeline (tokens → CSS + TS) | 3h |
| 25 | Agregar axe-core a Playwright tests | 2h |
| 26 | Agregar Chromatic para visual regression (cuando >50 componentes) | 4h |
| 27 | Font subsetting para reducir ~500KB | 2h |
| 28 | Interactive terminal demo en hero | 8h |
| 29 | Light mode (contradice brand actual, evaluar) | — |

---

## 7. RESUMEN EJECUTIVO

### Lo que está BIEN
- Diseño Swiss + Neubrutalism maduro y bien documentado (DESIGN.md de 862 líneas)
- Accesibilidad sólida (skip-link, aria, keyboard trap, reduced-motion)
- Arquitectura de animaciones correcta (GSAP matchMedia en useAnimationSafe)
- SEO completo (schema.org, sitemap, robots, OG, Twitter cards)
- Build optimization (manualChunks, Vite 8 config madura)
- Testing infrastructure (Vitest + Playwright configurados)

### Lo que está MAL
- **5 errores de contenido** (ONNX, Sled, LangChain, versiones, benchmarks)
- **~15 clases CSS no utilizadas** + duplicación de keyframes
- **Componentes sin accesibilidad completa** (NbAccordion sin keyboard nav)
- **Sin documentación de componentes** (no hay Ladle/Storybook)
- **SourceDesign/ mezclado en src/**
- **Sin orden de imports** (no hay lint rule)
- **Tailwind infrautilizado** (solo @theme, todo el resto es CSS manual)
- **GSAP subutilizado** (plugins gratis no activados: DrawSVG, MorphSVG, Flip)
- **Sin text splitting** (headers estáticos)
- **Sin logos reales de tecnologías** (solo text labels)
- **Sin gráficos reales** (benchmarks son CSS bars)

### Stack Correcto para 2026
✅ React 19 + Vite 8 + Tailwind v4 + GSAP 3.15 — **excelente stack**
✅ Se mantiene: Tailwind v4 con CSS variables (no migrar a CSS-in-JS)
✅ Se agrega: shadcn/ui para primitivas accesibles
✅ Se agrega: observable-plot / d3-force para data viz
✅ Se agrega: split-type para text animations
✅ Se mantiene: CSS manual para diseño visual (Swiss grid, texturas)

### Costo de las Mejoras
- **shadcn/ui**: ~15-25KB gzip (5 componentes)
- **observable-plot**: ~45KB gzip (lazy-load en páginas de benchmarks)
- **d3-force**: ~8KB gzip (standalone, no full D3)
- **split-type**: ~5KB gzip (0 deps)
- **simple-icons**: ~0.5KB por icono (tree-shakeable)
- **GSAP plugins**: $0 (todos gratuitos desde 2024)
- **Total estimado adicional**: ~80KB gzip (lazy-loaded por página, no en bundle inicial)
