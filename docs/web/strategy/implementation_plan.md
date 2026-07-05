---
title: "Rediseño Completo VantaDB — Swiss High-Contrast Minimal (Neon Precision)"
status: active
tags: [vantadb, web, strategy]
last_reviewed: 2026-07-03
aliases: []
---

<!-- markdownlint-disable-file MD013 MD022 MD025 MD026 MD031 MD040 MD058 -->

# Rediseño Completo VantaDB — Swiss High-Contrast Minimal (Neon Precision)

## Descripción del Objetivo

Rediseñar **todo** el sitio web de VantaDB (index + todas las subpáginas + Nav + Footer) bajo el estilo **Swiss High-Contrast Minimal (Neon Precision)**. El documento de referencia técnica completo se encuentra en [`../design/DiseñoNuevo.md`](../design/DiseñoNuevo.md).

---

## Decisiones Resueltas

| Decisión | Resolución |
|:---|:---|
| **Última sección del Index** | **Opción A: "The Monolith"** — Bloque OLED negro full-width con `pip install vantadb` en tipografía hero masiva + CTA naranja. Es la opción más coherente con Swiss Minimal: máximo impacto con mínimos elementos. |
| **Hero 3D vs Tipográfico** | **100% tipográfico con fondo animado** — Sin wireframe Three.js. El fondo será un grid de hairlines que se dibujan/animan sutilmente (stroke-dashoffset + ambient drift). |
| **Singularity Black Hole** | **ELIMINADA** — Se remueve `SingularityHero.tsx` y el directorio `singularity-master/` deja de usarse en el hero. |
| **`/changelog`** | **Rediseñar** con estilo Swiss Minimal (timeline vertical con labels `[v0.6.0]`, bordes de 1px, tipos de cambio con badges monoline). |
| **`/product/*`** | Directorio vacío actualmente — **no aplica**. |
| **Footer links** | **Todas las páginas** del sitio excepto artículos individuales del blog. Agrupación mejorada en 5 columnas. |
| **`/about/roadmap`** | **ELIMINAR** completamente. |

---

## Proposed Changes

La implementación se divide en **6 fases** ejecutadas secuencialmente. Cada fase es autocontenida y verificable.

---

### FASE 0 — Fundaciones y Tokens

Actualización del sistema de diseño base. Todo lo demás depende de esta fase.

---

#### [MODIFY] [tokens.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/tokens.css)
**Cambios específicos:**
- Añadir variables de bloques oscuros invertidos:
  ```css
  --block-dark-bg: #0a0a0a;
  --block-dark-text: #f0f0f0;
  --block-dark-muted: #808080;
  --block-dark-border: rgba(255,255,255,0.08);
  ```
- Añadir easing cortante: `--ease-cut: cubic-bezier(0.25, 1, 0.5, 1)`
- Añadir macro-spacing: `--section-gap: 96px; --section-gap-lg: 160px`
- Verificar sincronización con DESIGN.md

#### [MODIFY] [DESIGN.md](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/DESIGN.md)
- Sincronizar con DiseñoNuevo.md: añadir bloques invertidos, anti-slop checklist, iconografía monoline

#### [NEW] [swiss-grid.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-grid.css)
**Sistema de grid visible de 12 columnas:**
```css
.swiss-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 1px;
  max-width: 1200px;
  margin: 0 auto;
}
```
- Clases utilitarias: `.swiss-section`, `.swiss-section--dark` (fondo `#0a0a0a` con texto invertido), `.swiss-section--inverted`
- Clases de span: `.col-span-1` a `.col-span-12`, `.col-start-*`
- Asimetría: `.asymmetric-left` (cols 1-8 contenido, 9-12 vacío)
- `.grid-line-v`, `.grid-line-h` para hairlines decorativas visibles
- `.section-divider` — línea horizontal de 1px full-width entre secciones

#### [MODIFY] [layout.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/layout.css)
- `.section`: padding `var(--section-gap) 0`, max-width `1200px`
- `.section-dark`: fondo `#0a0a0a`, `color: var(--block-dark-text)`
- Purgar estilos del diseño anterior (gradientes, sombras difusas)

#### [MODIFY] [buttons.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/buttons.css)
- `border-radius: 0px` forzado en todos los botones
- Primary: fondo `--amber` → hover `#000000`
- Ghost: borde `1px solid var(--border)` → hover fondo `--border` + texto blanco
- Ghost invertido (para secciones oscuras): borde blanco, texto blanco → hover fondo blanco + texto negro
- Eliminar todo `box-shadow`, todo gradiente
- Transición: `150ms var(--ease-cut)`

#### [MODIFY] [animations.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/animations.css)
- Eliminar animaciones de bounce, elastic, spring
- Nuevas clases:
  - `.reveal-mask` — clip-path reveal desde abajo (`clip-path: inset(100% 0 0 0)` → `inset(0)`)
  - `.reveal-expand` — expand desde líneas del grid (scale 0 → 1 desde borde)
  - `.reveal-draw` — stroke-dashoffset animado para líneas SVG
- Todas las duraciones ≤ 250ms
- `@media (prefers-reduced-motion: reduce) { .reveal-mask, .reveal-expand, .reveal-draw { animation: none; opacity: 1; } }`

---

### FASE 1 — Nav + Footer (Componentes Globales)

Estos componentes se renderizan en todas las páginas. Se hacen primero para establecer la estructura visual global.

---

#### [MODIFY] [Nav.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/Nav.tsx)
**Rediseño completo de la navegación:**
- Barra fija superior, altura 64px
- Fondo: `var(--surface-glass)` + `backdrop-filter: blur(12px)`
- Borde inferior: `1px solid var(--border)`
- **Layout:** Logo izquierda | Links centro | CTA derecha
- Links en JetBrains Mono ALL CAPS, 0.72rem, letter-spacing 0.14em, color `--steel`
- Hover: `--steel` → `--foreground` en 100ms (sin underline, sin background)
- Link activo: color `--amber`
- CTA: botón primary rectangular "Get Started"
- **Eliminar:** link a `/about/roadmap`
- **Mobile:** hamburger icon → panel lateral fondo `--surface-overlay`, lista vertical de links
- **Simplificar dropdowns:** eliminar `NavDropdown.tsx` complejo, usar links directos o mega-menu ultra-simple con borde 1px

#### [MODIFY] [nav.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/nav.css)
- Reescribir: `border-radius: 0px`, `box-shadow: none`, sin gradientes
- `.nav-link`: font JetBrains Mono, uppercase, tracking 0.14em
- `.nav-link:hover`: color `--foreground`, transition 100ms
- `.nav-link--active`: color `--amber`
- `.nav-mobile-panel`: fondo blanco, borde izquierdo 1px `--border`

#### [NEW] [SwissFooter.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissFooter.tsx)
**Footer OLED negro con todas las páginas:**
- Fondo: `#0a0a0a`
- **Grid de 5 columnas** para agrupar todas las páginas:

| Product | Solutions | Developers | Resources | Company |
|:---|:---|:---|:---|:---|
| Engine | AI Agents | Docs | Blog | About |
| Architecture | Local RAG | Config | Changelog | Community |
| Integrations | AI IDE Tooling | Pricing | — | Contact |
| Use Cases | — | — | — | — |
| Cost | — | — | — | — |
| Latency | — | — | — | — |
| Storage | — | — | — | — |
| Maintenance | — | — | — | — |

- Links en `--block-dark-muted` (#808080)
- Hover: `#ffffff`
- Títulos de columna en `--text-label` ALL CAPS, color `--block-dark-text`
- Líneas divisorias: `1px solid rgba(255,255,255,0.08)`
- Bottom bar: logo VantaDB miniatura + copyright en label + links sociales (GitHub)
- **NO incluir:** artículos individuales del blog

#### [NEW] [footer.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/footer.css)
- Estilos del footer OLED

#### [MODIFY] [__root.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/__root.tsx)
- Reemplazar footer inline por `<SwissFooter />`
- Verificar estructura de shell global

---

### FASE 2 — Index / Landing Page (Secciones del Index)

El corazón del rediseño. Se reescribe `index.tsx` completamente.

---

#### [MODIFY] [index.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/index.tsx)
**Reescritura completa del componente `Landing`:**

**Orden de secciones (8 secciones, no 10):**
1. `<SwissHero />` — Hero tipográfico con fondo animado
2. `<SwissBenchmarkGrid />` — Comparativa Bento
3. ~~Estadísticas~~ — **ELIMINADAS**
4. `<SwissQuickstart />` — Terminal + pasos 01-04
5. `<SwissCoreEngine />` — Features del motor con scroll pin (sección oscura)
6. `<SwissArchSection />` — Diagrama blueprint
7. `<SwissEcosystem />` — Grid de integraciones
8. `<SwissUseCases />` — Tarjetas editoriales
9. `<SwissMonolith />` — CTA final (The Monolith)

**Actualizar `useGSAPReveal()`:**
- ScrollTrigger.batch para `.swiss-reveal` elements
- Pin para sección Core Engine
- Count-up triggers para números del benchmark
- Stroke-dashoffset draws para líneas SVG
- `prefers-reduced-motion` respetado

**Eliminar imports obsoletos:**
- `SingularityHero`, `AmberParticles`, `CodeGridBackground`, `ComparisonTable`, `ScrollStory`

---

#### [NEW] [SwissHero.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissHero.tsx)
**Hero 100% tipográfico con fondo animado:**

**Layout (Grid 12 columnas):**
- Fondo `--background` con grid de hairlines visibles que se animan
- **Fondo animado:** Las líneas verticales del grid de 12 columnas se dibujan con `stroke-dashoffset` de arriba a abajo al cargar (duración ~800ms staggered). Después, ambient drift muy sutil — las líneas tienen una micro-oscilación de opacidad (0.3 → 0.5 → 0.3) en bucle lento (8s), dando sensación de "datos fluyendo"
- **Label superior** (cols 1-8): `[RUST-NATIVE]` · `[IN-PROCESS]` · `[ZERO-SERVERS]` en JetBrains Mono ALL CAPS, color `--amber`
- **Título** (cols 1-8): "VantaDB" en Space Grotesk 700, `clamp(3.8rem, 8vw, 7.5rem)`, color `#000000`
- **Subtítulo** (cols 1-8): "Embedded cognitive memory for AI agents" en Outfit 400, `clamp(1.3rem, 2.2vw, 1.7rem)`, color `--muted`
- **CTAs** (cols 1-8): `pip install vantadb` (primary naranja) + "Read Docs" (ghost con borde negro)
- **Columnas 9-12:** vacías intencionalmente (espacio negativo Swiss)

**Animación de entrada:**
1. Grid lines se dibujan (stroke-dashoffset, 800ms staggered)
2. Labels `[RUST-NATIVE]` parpadean una vez en naranja (flash 200ms) y quedan estáticos
3. Título se revela por clip-path mask desde abajo (400ms, ease-cut)
4. Subtítulo y CTAs aparecen con opacity 0→1 (300ms delay after title)

#### [NEW] [swiss-hero.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-hero.css)

---

#### [NEW] [SwissBenchmarkGrid.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissBenchmarkGrid.tsx)
**Grilla Bento asimétrica de benchmarks:**

**Layout:**
- Sección label: `[BENCHMARKS]` en `--text-label`
- Título: "VantaDB vs. The Stack" en `--text-display`
- Grid Bento de tamaños variados con bordes de 1px `var(--border)`
  ```
  ┌──────────┬─────┬──────────┐
  │  2×2     │ 1×1 │  2×1     │
  │ LATENCY  │DEPS │ MEMORY   │
  │          │     │          │
  ├──────────┼─────┼──────────┤
  │  1×1     │  2×1          │
  │ SETUP    │  SEARCH TYPE  │
  ├──────────┼───────────────┤
  │  3×1  CRASH RECOVERY     │
  └──────────────────────────┘
  ```
- Cada celda contiene:
  - Index label: `[01]` en esquina superior izquierda, `--text-label`
  - Métrica VantaDB: número gigante en Space Grotesk 700, `--text-display`
  - Unidad/label debajo en JetBrains Mono, `--text-label`
  - Comparativa: "vs 12ms traditional" en `--muted`, tamaño pequeño
  - Indicador: `↓ 15×` en `--amber` (mejor) o `↑` en `--danger` (peor)

**Métricas:**
| # | Métrica | VantaDB | Traditional | Delta |
|:---|:---|:---|:---|:---|
| 01 | Query Latency (p99) | 0.8ms | 12ms | ↓ 15× faster |
| 02 | External Dependencies | 0 | 12+ | ↓ Zero deps |
| 03 | Memory Overhead | 2MB | 180MB | ↓ 90× less |
| 04 | Setup Time | 1 line | 45 min config | ↓ Instant |
| 05 | Search Type | Hybrid BM25+HNSW | Single mode | Full-spectrum |
| 06 | Crash Recovery | WAL automatic | Manual backup | Zero-loss |

**Interacción:**
- Al entrar en viewport, celdas se expanden desde las líneas de grid (GSAP: scaleX/Y 0→1 desde el borde, stagger 60ms)
- Números hacen count-up rápido (200ms) con `font-variant-numeric: tabular-nums`
- Hover: borde de celda → `--amber`, index label `[01]` → `--amber`

#### [NEW] [swiss-benchmark.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-benchmark.css)

---

#### [NEW] [SwissQuickstart.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissQuickstart.tsx)
**Terminal + pasos 01-04 estilo Swiss:**

**Layout (Grid 2 columnas: 4col pasos | 8col terminal):**
- Sección label: `[QUICKSTART]`
- **Columna izquierda** — Lista de pasos:
  - `[01] Install` → `pip install vantadb`
  - `[02] Import` → `from vantadb import VantaDB`
  - `[03] Store` → `db.store("memory", vector, metadata)`
  - `[04] Query` → `db.query("find similar", top_k=5)`
  - Paso activo: número en `--amber`, borde izquierdo `2px solid var(--amber)`
  - Pasos inactivos: número en `--steel`, sin borde

- **Columna derecha** — Terminal:
  - Fondo `--void`, borde `1px solid var(--border)`
  - Header: `⬤ ⬤ ⬤` en `--subtle` (sin color) + título `terminal` en `--text-label`
  - Código con typewriter rápido (30ms/char)
  - Output aparece instantáneamente (no fade) con `border-left: 2px solid var(--amber)`
  - Syntax: keywords `--foreground`, strings `--amber`, comments `--muted`

**Interacción:**
- Auto-play secuencial: paso 01 se escribe, output aparece, paso 02 se activa...
- Click en paso: salta directamente a ese paso
- Al completar 04: loop restart después de 3s

#### [NEW] [swiss-quickstart.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-quickstart.css)

---

#### [NEW] [SwissCoreEngine.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissCoreEngine.tsx)
**Features del motor con GSAP ScrollTrigger pin — sección OLED oscura:**

**Layout:**
- **Sección con fondo `#0a0a0a`** (contraste invertido)
- Label: `[CORE ENGINE]` en naranja sobre fondo negro
- Título: "Built for Speed. Written in Rust." en `#f0f0f0`
- **Grid 3×2** de features, cada una revelada secuencialmente con scroll pin

**6 Features:**
| # | Feature | Icono (monoline) | Descripción |
|:---|:---|:---|:---|
| 01 | Rust Core | Engranaje monoline | Memory-safe, zero-cost abstractions, compiled performance |
| 02 | HNSW Index | Grafo de nodos monoline | Hierarchical Navigable Small World for vector similarity |
| 03 | BM25 Engine | Documento con lupa monoline | Full-text search with TF-IDF ranking |
| 04 | WAL Durability | Disco con checkmark monoline | Write-Ahead Log for crash-safe persistence |
| 05 | PyO3 Bridge | Puente de dos bloques monoline | Native Python bindings, zero serialization overhead |
| 06 | Zero-Copy Serde | Flecha bidireccional monoline | Deserialization without memory allocation |

**Cada feature card:**
- Fondo: `transparent` → hover `rgba(255,255,255,0.03)`
- Borde: `1px solid var(--block-dark-border)`
- Icono: SVG monoline 1.5px stroke, color `--amber`
- Título: `#f0f0f0`, Outfit 600
- Descripción: `--block-dark-muted` (#808080)
- Index: `[01]` en esquina, `--block-dark-muted`

**Animación GSAP ScrollTrigger:**
- La sección entera se **pinea** durante el scroll
- Las 6 features se revelan una a una con stagger al scrollear
- Cada feature entra con `clipPath: inset(100% 0 0 0)` → `inset(0)` + `opacity 0→1`
- Icono SVG: stroke se dibuja con `stroke-dashoffset` (200ms)
- Al completar las 6, la sección se des-pinea y el scroll continúa

#### [NEW] [swiss-core-engine.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-core-engine.css)

---

#### [NEW] [SwissArchSection.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissArchSection.tsx)
**Diagrama de arquitectura tipo blueprint/plano industrial:**

**Layout:**
- Label: `[ARCHITECTURE]`
- Título: "Inside the Engine" en `--text-display`
- Diagrama SVG central de capas apiladas:
  ```
  ┌─────────────────────────────┐
  │  Python Application         │ ← Label con línea de cota
  ├─────────────────────────────┤
  │  PyO3 FFI Bridge            │ ← Flechas ortogonales de flujo
  ├─────────────────────────────┤
  │  Rust Core Engine           │
  │  ┌──────┬──────┬──────┐    │
  │  │ HNSW │ BM25 │ WAL  │    │ ← Sub-módulos con labels
  │  └──────┴──────┴──────┘    │
  ├─────────────────────────────┤
  │  Storage Layer (mmap)       │
  └─────────────────────────────┘
  ```
- Cada capa: rectángulo con borde 1px, relleno semi-transparente
- Labels con líneas de cota (flechas de medición) y coordenadas

**Interacción:**
- Scroll: las capas se separan verticalmente (exploded view) con GSAP
- Hover en capa: borde → `--amber`, resto `opacity: 0.3`
- Labels técnicos aparecen con clip-path al hacer scroll

#### [NEW] [swiss-architecture.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-architecture.css)

---

#### [NEW] [SwissEcosystem.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissEcosystem.tsx)
**Grid limpio de integraciones:**

**Layout:**
- Label: `[ECOSYSTEM]`
- Título: "Works With Everything" en `--text-display`
- Grid 4×3 con bordes de 1px en todas las celdas
- Cada celda:
  - Icono SVG monoline (1.5px stroke, sin relleno), color `--steel`
  - Nombre de la integración en `--text-label` ALL CAPS debajo

**Categorías (columnas):**
- `[FRAMEWORKS]`: LangChain, LlamaIndex, CrewAI, AutoGen
- `[LLM PROVIDERS]`: OpenAI, Anthropic, Google, Ollama
- `[DEPLOYMENT]`: Docker, Kubernetes, Vercel, AWS Lambda

**Interacción:**
- Hover en celda: icono `--steel` → `--amber`, fondo → `--amber-dim`, borde → `--amber`
- Scroll: celdas aparecen con stagger desde las líneas del grid (GSAP batch, 60ms stagger)

#### [NEW] [swiss-ecosystem.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-ecosystem.css)

---

#### [NEW] [SwissUseCases.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissUseCases.tsx)
**Tarjetas editoriales horizontales:**

**Layout:**
- Label: `[USE CASES]`
- Stack vertical de tarjetas full-width, separadas por `1px solid var(--border)` horizontal
- Cada tarjeta: grid `3fr 9fr`
  - **Izquierda:** Número del caso en `--text-display`, color `--subtle` (gris muy claro)
  - **Derecha:** Título en `--text-title` + descripción en `--text-body` color `--muted` + label de industria en `--text-label`

**Casos:**
| # | Caso | Industria |
|:---|:---|:---|
| 01 | AI Agent Memory | AI / ML |
| 02 | Local RAG Pipeline | Enterprise |
| 03 | IDE Code Intelligence | Developer Tools |
| 04 | Offline Knowledge Base | Edge / IoT |

**Interacción:**
- Hover: número `--subtle` → `--amber` (150ms)
- Scroll: tarjetas revelan con clip-path mask desde izquierda

#### [NEW] [swiss-use-cases.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-use-cases.css)

---

#### [NEW] [SwissMonolith.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissMonolith.tsx)
**CTA final "The Monolith" — Opción A:**

**Layout:**
- **Bloque OLED negro full-width** (fondo `#0a0a0a`)
- Padding vertical masivo: `160px`
- Contenido centrado (excepción al alineamiento izquierdo, justificado por ser CTA aislado):
  - Texto principal: `pip install vantadb` en Space Grotesk 700, `--text-hero`, color `#ffffff`
  - Subtítulo: "Zero servers. One line. Infinite context." en Outfit 400, `--block-dark-muted`
  - Botón primary centrado: "Get Started" en naranja
  - Link secundario: "Read Documentation →" en `--block-dark-muted`, hover `#ffffff`

**Interacción:**
- Al entrar viewport: texto se revela por clip-path mask (400ms)
- El comando `pip install vantadb` tiene un cursor parpadeante al final (CSS blink `500ms`)

#### [NEW] [swiss-cta.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-cta.css)

---

### FASE 3 — Subpáginas Técnicas

Todas las subpáginas siguen este **patrón Hero común** (componente compartido):

#### [NEW] [SwissSubpageHero.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SwissSubpageHero.tsx)
**Hero reutilizable para subpáginas:**
- Props: `label`, `title`, `description`, `breadcrumb`
- Layout: grid 12 columnas, título en cols 1-8 (asimetría)
- Label en `--text-label` ALL CAPS naranja: `[ENGINE]`, `[ARCHITECTURE]`, etc.
- Breadcrumb: `Home / Engine` en `--text-label` color `--steel`
- Título en `--text-display`, descripción en `--text-body` color `--muted`
- Borde inferior: `1px solid var(--border)` full-width

#### [NEW] [swiss-subpage.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/swiss-subpage.css)
- Estilos compartidos para todas las subpáginas

---

#### [MODIFY] [engine.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/engine.tsx)
**Reescritura completa:**
- Usar `<SwissSubpageHero label="ENGINE" title="The Rust Core" />`
- 6 secciones (una por subsistema): HNSW, BM25, WAL, PyO3, Zero-Copy Serde, SIMD
- Cada sección: grid `5fr 7fr` (diagrama SVG izquierda + texto derecha) o invertido
- Fondo alternado: warm paper → OLED negro → warm paper
- Diagramas SVG monoline con líneas de cota
- Benchmarks con barras horizontales SVG minimalistas
- **GSAP ScrollTrigger funcional:** arreglar animaciones rotas, usar pin por sección

#### [MODIFY] [engine.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/engine.css)
- Purgar completamente y reescribir con tokens Swiss

---

#### [MODIFY] [architecture.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/architecture.tsx)
- Usar `<SwissSubpageHero label="ARCHITECTURE" />`
- Diagrama principal SVG interactivo con exploded view al scroll
- Secciones de detalle por capa con código ejemplo en terminal
- Labels con líneas de cota y coordenadas técnicas

#### [MODIFY] [architecture.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/architecture.css)
- Reescribir con tokens Swiss

---

#### [MODIFY] [integrations.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/integrations.tsx)
- **Rediseño completo** — eliminar colores del diseño anterior
- Usar `<SwissSubpageHero label="INTEGRATIONS" />`
- Grid matrix ampliado de integraciones por categoría
- Cada integración: card con icono monoline + ejemplo de código en terminal
- Borde `1px solid var(--border)` en todas las cards

#### [MODIFY] [integrations.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/integrations.css)
- Purgar y reescribir

---

#### [MODIFY] [use-cases.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/use-cases.tsx)
- Usar `<SwissSubpageHero label="USE CASES" />`
- Versión expandida de SwissUseCases con más detalle por caso
- Cada caso: sección completa con diagrama de arquitectura + stack técnico + código
- Fondo alternado claro/oscuro

#### [MODIFY] [use-cases.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/use-cases.css)
- Reescribir

---

### FASE 4 — Subpáginas de Métricas + Changelog

---

#### [MODIFY] [cost.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/cost.tsx)
- `<SwissSubpageHero label="COST" title="Total Cost of Ownership" />`
- Grid Bento de costos: VantaDB ($0/mo embedded) vs Pinecone vs Weaviate vs Chroma
- Números gigantes Space Grotesk para precios
- Barras SVG horizontales de ahorro porcentual
- Tabla comparativa con bordes 1px

#### [MODIFY] [latency.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/latency.tsx)
- `<SwissSubpageHero label="PERFORMANCE" title="Latency Benchmarks" />`
- Gráficos de barras horizontales SVG (p50/p95/p99)
- Tabla monoespaciada JetBrains Mono con `tabular-nums`
- Comparativa con competidores: grid de cards con bordes

#### [MODIFY] [storage.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/storage.tsx)
- `<SwissSubpageHero label="STORAGE" title="Storage Architecture" />`
- Diagramas SVG: WAL structure, HNSW graph, BM25 inverted index
- Métricas de compresión en grid
- Crash recovery flow diagram con flechas ortogonales

#### [MODIFY] [config.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/config.tsx)
- `<SwissSubpageHero label="CONFIG" title="Configuration Reference" />`
- Tabla de opciones: `name | type | default | description` con bordes 1px
- Labels de sección: `[INDEXING]` `[STORAGE]` `[SEARCH]` `[RUNTIME]`
- Código ejemplo en terminal por sección

#### [MODIFY] [maint.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/maint.tsx)
- `<SwissSubpageHero label="MAINTENANCE" title="Operations Guide" />`
- Pasos numerados `[01]`–`[N]` con borde izquierdo activo
- Diagrama de flujo con flechas ortogonales
- Secciones: Backup, Upgrade, Monitoring, Troubleshooting

#### [MODIFY] [changelog.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/changelog.tsx)
- `<SwissSubpageHero label="CHANGELOG" title="Release Notes" />`
- **Timeline vertical** con línea de 1px izquierda
- Cada release: nodo punto en la línea + card con borde 1px
  - Header: `[v0.6.0]` en `--text-label` naranja + fecha en `--steel`
  - Tag badge: `Hybrid RRF GA` en label con borde 1px naranja
  - Changes list con iconos monoline por tipo:
    - `[FEATURE]` → icono `+` naranja
    - `[PERF]` → icono `↑` verde
    - `[FIX]` → icono `✓` azul steel
    - `[BREAKING]` → icono `!` rojo

---

### FASE 5 — Solutions, Docs, Pricing, About, Blog, Community, Contact

---

#### [MODIFY] [solutions/ai-agents.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/ai-agents.tsx)
- `<SwissSubpageHero label="SOLUTION" title="AI Agent Memory" />`
- Diagrama de arquitectura: Agent → VantaDB (embedded) → Memory/Retrieval
- Grid de features para agentes
- Código ejemplo: `agent.remember()` / `agent.recall()`
- CTA: "Build Your Agent"

#### [MODIFY] [solutions/local-rag.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/local-rag.tsx)
- `<SwissSubpageHero label="SOLUTION" title="Local RAG Pipeline" />`
- Diagrama de pipeline: Documents → Embed → VantaDB → Query → LLM
- Comparativa local vs cloud RAG en grid Bento

#### [MODIFY] [solutions/ai-ide-tooling.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/solutions/ai-ide-tooling.tsx)
- `<SwissSubpageHero label="SOLUTION" title="AI IDE Tooling" />`
- Diagrama de integración IDE → VantaDB → Code Intelligence
- Features para desarrolladores

#### [MODIFY] [docs.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/docs.tsx)
- Layout 2 columnas: sidebar (3col) + contenido (9col)
- Sidebar: lista de secciones en `--text-label`, link activo con borde izquierdo `--amber`
- Contenido: código con syntax highlighting mínimo, bordes 1px entre secciones
- Terminal examples con fondo `--void`

#### [MODIFY] [docs.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/docs.css)
- Reescribir

#### [MODIFY] [pricing.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/pricing.tsx)
- `<SwissSubpageHero label="PRICING" title="Simple, Transparent Pricing" />`
- Grid de planes con borde 1px, plan destacado con borde `--amber`
- Precios en `--text-display` Space Grotesk
- Feature comparison table con iconos monoline ✓/✗
- FAQ accordion al final (opcional)

#### [MODIFY] [pricing.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/pricing.css)
- Reescribir

#### [MODIFY] [about/index.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/index.tsx)
- `<SwissSubpageHero label="ABOUT" title="About VantaDB" />`
- Layout editorial: mission statement + values grid + timeline de hitos
- Eliminar colores/estructuras del diseño anterior

#### [MODIFY] [about/company.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/company.tsx)
- Rediseño Swiss con grid visible y tipografía correcta

#### [DELETE] [about/roadmap.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/roadmap.tsx)
- **ELIMINAR** completamente
- Eliminar links en Nav y Footer

#### [MODIFY] blog/ pages
- Lista de posts con grid 2 columnas desktop
- Cada post card: fecha `--text-label` + categoría badge + título `--text-title`
- Borde 1px en cada card
- Post detalle: layout editorial con tipografía Swiss

#### [MODIFY] [about/community.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/community.tsx)
- Grid de canales con iconos monoline: GitHub, Discord, Twitter
- Cards con borde 1px, hover → borde `--amber`

#### [MODIFY] [about/contact.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/routes/about/contact.tsx)
- Grid 2 columnas: info izquierda + formulario derecha
- Inputs rectangulares: `border-radius: 0px`, borde `1px solid var(--border)`
- Labels en `--text-label` ALL CAPS
- Focus en input: borde → `--amber`
- Botón submit: primary naranja

---

### FASE 6 — Limpieza, Purga y Polish Final

---

#### Componentes a ELIMINAR:
| Componente | Razón |
|:---|:---|
| [SingularityHero.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/SingularityHero.tsx) | Reemplazado por SwissHero — eliminación confirmada |
| [AmberParticles.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/AmberParticles.tsx) | Partículas no son Swiss Minimal |
| [TypewriterHero.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/TypewriterHero.tsx) | Reemplazado por SwissHero |
| [CodeGridBackground.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/CodeGridBackground.tsx) | Grid se maneja con CSS puro |
| [ComparisonTable.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/ComparisonTable.tsx) | Reemplazado por SwissBenchmarkGrid |
| [AnimeMorphLogo.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/AnimeMorphLogo.tsx) | No Swiss |
| [AnimeStaggerGrid.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/AnimeStaggerGrid.tsx) | No Swiss |
| [ScrollStory.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/ScrollStory.tsx) | Reemplazado por SwissCoreEngine scroll pin |
| [CtaSection.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/CtaSection.tsx) | Reemplazado por SwissMonolith |
| [HeroSubpage.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/HeroSubpage.tsx) | Reemplazado por SwissSubpageHero |

#### Componentes a EVALUAR:
| Componente | Acción |
|:---|:---|
| [NavDropdown.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/NavDropdown.tsx) | Simplificar drásticamente o eliminar |
| [TypewriterTitle.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/TypewriterTitle.tsx) | Reutilizar lógica de typewriter en SwissQuickstart, eliminar componente |
| [ScrambleText.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/ScrambleText.tsx) | Evaluar si encaja en Swiss (el scramble puede ser compatible), mantener si es útil |
| [Reveal.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/Reveal.tsx) | Refactorizar para usar clip-path mask en lugar de opacity fade |
| [VantaDBLogo.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/VantaDBLogo.tsx) | Mantener, simplificar si tiene efectos no-Swiss |
| [InteractiveQuickstart.tsx](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/components/InteractiveQuickstart.tsx) | Extraer lógica útil para SwissQuickstart, luego eliminar |
| Componentes SVG (HNSW, Flamegraph, etc.) | Refactorizar a monoline Swiss o eliminar si se reemplazan |

#### CSS a PURGAR/REEMPLAZAR:
| Archivo Viejo | Reemplazo |
|:---|:---|
| [hero.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/hero.css) | `swiss-hero.css` |
| [comparison.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/comparison.css) | `swiss-benchmark.css` |
| [cards.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/cards.css) | Estilos unificados en `swiss-grid.css` |
| [effects.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/effects.css) | Eliminar, no hay efectos difusos en Swiss |
| [split-playground.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/split-playground.css) | Eliminar si no hay playground |
| [visualizations.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/visualizations.css) | Reescribir para diagramas SVG monoline |
| [logo.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/logo.css) | Simplificar para Swiss |
| [terminal.css](file:///c:/Users/Eros/VantaDB%20Proyect/vantadb.github.io/src/styles/terminal.css) | Reescribir: `border-radius: 0px`, sin sombras, borde 1px |

#### Anti-Slop Audit Final:
Ejecutar verificación global en todo el CSS/JSX:
- [ ] `border-radius` ≤ 6px en todo el proyecto
- [ ] `box-shadow: none` en todo el proyecto
- [ ] Sin gradientes decorativos
- [ ] Naranja solo en señales activas/CTAs (regla 95/5)
- [ ] Texto alineado a izquierda (excepto SwissMonolith CTA)
- [ ] Tipografía correcta: Space Grotesk / Outfit / JetBrains Mono
- [ ] Bordes de 1px presentes en todas las tarjetas y contenedores
- [ ] Animaciones ≤ 250ms, easing cortante
- [ ] `prefers-reduced-motion` respetado en todo GSAP
- [ ] `font-variant-numeric: tabular-nums` en datos numéricos
- [ ] Touch targets ≥ 44×44px en mobile
- [ ] Contraste WCAG AA: ≥ 4.5:1 texto, ≥ 3:1 UI

---

## Skills a Utilizar por Fase

| Fase | Skills de Diseño/Código | Skills de Auditoría |
|:---|:---|:---|
| **0: Tokens** | `color-expert`, `awesome-claude-design` | — |
| **1: Nav/Footer** | `frontend-design`, `emil-design-eng`, `high-end-visual-design` | `design-review` |
| **2: Index** | `gpt-taste` (GSAP ScrollTriggers, bento grids), `industrial-brutalist-ui` (grids rígidos), `high-end-visual-design` (typography) | `plan-design-review`, `impeccable-design-polish` |
| **3: Subpáginas Técnicas** | `frontend-design`, `d3-visualization` (SVG charts), `minimalist-ui` | `design-review` |
| **4: Métricas** | `frontend-design`, `d3-visualization` | `design-review` |
| **5: Solutions/About** | `frontend-design`, `design-taste-frontend` | `plan-design-review` |
| **6: Polish** | `impeccable-design-polish`, `emilkowalski-motion`, `visual-review` | `plan-design-review` (gate final) |

---

## Verification Plan

### Automated Tests
```powershell
# TypeScript compilation — zero errors
npx tsc --noEmit

# Linting — zero errors
npx eslint .

# Production build — success
npm run build
```

### Visual Verification (por fase)
- [ ] **Fase 0:** Tokens correctos, sin errores de compilación
- [ ] **Fase 1:** Nav funcional desktop/mobile, footer con todas las páginas,
  links correctos
- [ ] **Fase 2:** Hero tipográfico con fondo animado, benchmark grid con
  count-up, quickstart terminal, core engine pin funcional, architecture
  exploded view, ecosystem grid, use cases cards, monolith CTA
- [ ] **Fase 3:** Engine/Architecture/Integrations/UseCases con Swiss style,
  SVG diagramas funcionales
- [ ] **Fase 4:** Cost/Latency/Storage/Config/Maint con métricas legibles,
  changelog timeline
- [ ] **Fase 5:** Solutions/Docs/Pricing/About/Blog/Community/Contact
  redesigned, roadmap 404
- [ ] **Fase 6:** Anti-slop audit passed, responsive en
  375px/768px/1280px/1920px

### Git Commits (al final de cada fase)

```bash
git add -A && git commit -m "feat(design): phase 0 — design tokens + grid"
git add -A && git commit -m "feat(nav): phase 1 — swiss nav and OLED footer"
git add -A && git commit -m "feat(index): phase 2 — landing page redesign"
git add -A && git commit -m "feat(pages): phase 3 — technical subpages"
git add -A && git commit -m "feat(pages): phase 4 — metrics + changelog"
git add -A && git commit -m "feat(pages): phase 5 — solutions, about, blog"
git add -A && git commit -m "chore(cleanup): phase 6 — purge + anti-slop"
```
