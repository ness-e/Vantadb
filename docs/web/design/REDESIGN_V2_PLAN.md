# Redesign v2 — VantaDB Website: Complete Architectural Overhaul

> **NO** es una mejora de lo que existe. Es una reconstrucción completa desde cero.
> Fecha: 2026-07-04 | Estilo: Swiss + Neubrutalism (Anti-Slop)
> Manifiesto: "Si el diseño corporativo es un sedán familiar, VantaDB es un coche de rally Grupo B."

---

## 1. Diagnóstico: Problemas de la Arquitectura Actual

| Problema | Impacto | Solución |
|---|---|---|
| 28 CSS files sueltos (4,567 lines) | Mantenimiento imposible, estilos duplicados | CSS modular único + tokens + utility classes |
| Componentes nombrados `Swiss*` | Confusión semántica (Swiss ≠ Swiss+Neubrutalism) | Renombrar a `Nb*` (Neubrutalism) |
| Sin framework de documentación | Docs estáticos sin search, versioning, MDX | Integrar Docusaurus o VitePress |
| Sin playground interactivo | No se puede probar el producto sin instalar | WASM demo embed (SQLite → VantaDB) |
| Hero textual sin terminal | Bajo impacto visual para dev tool | Terminal-as-hero con live query output |
| Sin social proof bar | No hay señales de confianza | GitHub stars + "used by" logos |
| Sin FAQ / "Why VantaDB" page | El usuario no sabe cuándo usarlo vs alternativas | Página de posicionamiento dedicada |
| Sin interactive architecture diagram | Arquitectura explicada solo con texto | Diagrama de capas clickable (SVG/Canvas) |
| Benchmarks estáticos | No comunican dinamismo del producto | Benchmark dashboard animado con D3 |
| Sin animated ticker / marquee | El diseño se siente estático | Data tickers, marquees con métricas reales |
| Sin dithering/ASCII en assets | Las imágenes no refuerzan la estética raw-computing | Filtro dithering 1-bit en fotos de equipo |
| Blog con solo 4 posts | Contenido insuficiente | Estrategia de contenido + SEO |

---

## 2. Nueva Estructura del Sitio (Site Map v3)

```
vantadb.com/
│
├── /                               HOME — Landing Page completamente nueva
│   ├── Terminal Hero                Terminal animada con query en vivo
│   ├── Trust Bar                    GitHub stars + "used by" + benchmarks
│   ├── Features (Bento)             Grid asimétrico 12-col con celdas variables
│   ├── Quickstart                   3-step code snippet with copy button
│   ├── Architecture Preview         Capas del motor (clickable)
│   ├── Benchmark Strip              Carrusel horizontal de métricas clave
│   ├── Ecosystem                    Grid de integraciones (frameworks, LLMs, cloud)
│   ├── Pricing Preview              Free tier + Enterprise
│   ├── FAQ                          "When to use VantaDB" / "Why not XYZ"
│   └── CTA Monolith                 pip install + blinking cursor
│
├── /engine/                         Core Engine (deep-dive técnico)
│   ├── Hybrid Search (HNSW+BM25)
│   ├── WAL & Durability
│   ├── PyO3 Bindings
│   ├── Zero-Copy Design
│   └── Benchmark Dashboard          En vivo (animado con D3)
│
├── /architecture/                   System Architecture (interactive)
│   ├── 5-Layer Stack                Diagrama clickable SVG
│   ├── Storage Backends
│   └── Query Pipeline
│
├── /docs/                           → Docusaurus / VitePress
│   ├── /docs/quickstart             Getting started (5min)
│   ├── /docs/guides                 Deployment, config, tuning
│   ├── /docs/api                    API Reference (OpenAPI auto)
│   ├── /docs/sdk/                   Python SDK, Rust SDK
│   └── /docs/faq                    FAQ
│
├── /pricing/                        Pricing page
│   ├── Free Tier                    Single-node, 10M vectors
│   ├── Pro                          Multi-node, 100M vectors
│   └── Enterprise                   On-prem, SSO, audit, SLA
│
├── /benchmarks/                     Benchmark Hub (interactive)
│   ├── vs Chroma / Pinecone / LanceDB
│   ├── vs SQLite + Extensions
│   ├── Latency Distribution         Histograma animado
│   └── Recall@10 Chart              Línea de tiempo
│
├── /why-vantadb/                    Positioning / FAQ
│   ├── When to use VantaDB
│   ├── VantaDB vs Chroma
│   ├── VantaDB vs SQLite + vec0
│   ├── VantaDB vs LanceDB
│   └── VantaDB vs Weaviate/Qdrant
│
├── /playground/                     In-browser demo (WASM)
│   ├── Query Editor                 CodeMirror + Run button
│   ├── Results Panel                JSON table output
│   └── Benchmark Mode               Compare queries side-by-side
│
├── /integrations/                   Ecosystem
│   ├── Frameworks (LangChain, LlamaIndex, Haystack)
│   ├── LLM Providers (OpenAI, Anthropic, Ollama)
│   ├── Memory (Mem0, Letta, CrewAI)
│   └── Deployment (Docker, PyPI, Cargo)
│
├── /solutions/                      Use Cases
│   ├── /solutions/ai-agents
│   ├── /solutions/local-rag
│   ├── /solutions/ai-ide-tooling
│   └── /solutions/offline-knowledge-base
│
├── /about/                          Company
│   ├── /about/team                  Team (fotos con dithering 1-bit)
│   ├── /about/community             Discord, GitHub, Contributing
│   └── /about/contact               Contact
│
├── /blog/                           Engineering Blog
│   ├── Architecture deep-dives
│   ├── Benchmark analysis
│   ├── Release notes
│   └── Community spotlights
│
└── /changelog/                      Release History (v0.1.x → v0.5.x)
```

**Pages nuevas (vs. ahora):** `/benchmarks/`, `/why-vantadb/`, `/playground/`, `/about/team`, `/about/community` (refactor), FAQ section en Home + /why-vantadb/.

---

## 3. Nueva Arquitectura de Componentes (NB- System)

### 3.1 Principios

- **Cada componente es atómico** — un solo archivo `.tsx` + estilos inline o CSS module
- **Zero layout shift** — tamaños fijos, reserva de espacio para animaciones
- **Data-driven** — los componentes reciben props, no hardcodean contenido
- **Reusabilidad máxima** — patrones extraídos a componentes base

### 3.2 Componentes Base (Atómicos)

| Componente | Props | Descripción |
|---|---|---|
| `NbLabel` | `children, variant?: 'amber' \| 'steel'` | `[ LABEL ]` en JetBrains Mono ALL CAPS |
| `NbFrame` | `children, label?, variant?` | Caja con borde 2px visible + label flotante |
| `NbCard` | `children, href?, variant?` | Card con hard shadow mecánico |
| `NbButton` | `children, variant: 'primary' \| 'ghost' \| 'install'` | Botón prensa mecánica |
| `NbTelemetry` | `children, prefix?: '>' \| '#' \| '$'` | Fila de datos con prefijo ámbar |
| `NbTerminal` | `children, lines?: string[], title?: string` | Terminal CRT con scanline |
| `NbTicker` | `active?: boolean, label?: string` | Indicador parpadeante "live" |
| `NbCursor` | — | Cursor parpadeante (blink CSS) |
| `NbMetric` | `value: string, label: string, trend?: 'up' \| 'down'` | Número grande + label |
| `NbSplitFlap` | `value: number, digits?: number` | Display rotativo tipo aeropuerto |
| `NbMarquee` | `children, speed?: number` | Cinta horizontal continua |
| `NbBento` | `cells: BentoCell[]` | Grid asimétrico 12-col |
| `NbBentoCell` | `children, span?: string, featured?: boolean` | Celda individual bento |
| `NbPill` | `children, status?: 'ga' \| 'beta' \| 'alpha'` | Pill con dot indicador |
| `NbArrow` | `children` | Link con `>>>` que se expande en hover |
| `NbBlockAmber` | `children` | Bloque destacado con fondo ámbar |
| `NbIconBox` | `children` | Contenedor cuadrado 40px para iconos |
| `NbTable` | `headers, rows` | Tabla con bordes visibles 2px |
| `NbCode` | `children, lang?: string` | Bloque de código con copy button |
| `NbDitherImage` | `src, alt` | Imagen con filtro dithering 1-bit |
| `NbNoise` | — | Overlay de ruido SVG (fixed) |
| `NbScanline` | — | Overlay CRT scanline (fixed) |
| `NbBgPattern` | `pattern: 'dot' \| 'cross' \| 'cross-faint'` | Fondo con patrón |

### 3.3 Componentes de Página (Organismos)

| Componente | Descripción | Reutilizado en |
|---|---|---|
| `NbNav` | Navbar con frame label `[ NAV ]`, métricas de telemetría | Global |
| `NbFooter` | Footer 5-columnas con metadata técnica | Global |
| `NbBackToTop` | Botón flotante con hard shadow | Global |
| `NbPageHero` | Hero de subpágina (label + title + desc + CTAs) | Todas las subpáginas |
| `NbTerminalHero` | Hero principal con terminal animada | Home |
| `NbTrustBar` | GitHub stars + logos + benchmark numbers | Home, /pricing |
| `NbFeatureGrid` | Bento grid de features con celdas variables | Home, /engine |
| `NbQuickstart` | 3-step code blocks with copy | Home, /docs/quickstart |
| `NbArchDiagram` | SVG interactivo de capas del motor | Home, /architecture |
| `NbBenchmarkRace` | Horizontal bar chart animado (D3) | Home, /benchmarks |
| `NbEcosystemGrid` | Grid de integraciones con pills | Home, /integrations |
| `NbPricingTable` | Tabla de pricing Free vs Pro vs Enterprise | /pricing |
| `NbPricingCard` | Card de tier individual con features | /pricing |
| `NbFaqAccordion` | Acordeón FAQ (collapsible sections) | Home, /why-vantadb |
| `NbMonolithCta` | CTA final gigante con pip install + cursor | Home |
| `NbBlogCard` | Blog post card | /blog |
| `NbBlogPost` | Blog post full render (Markdown → HTML) | /blog/$slug |
| `NbChangelogEntry` | Release entry con pills de status | /changelog |
| `NbSolutionCard` | Solución type: title + desc + features + CTA | /solutions/* |
| `NbPlayground` | WASM demo editor + output | /playground |
| `NbTeamGrid` | Team members con dithering | /about/team |
| `NbCommunityMetrics` | GitHub stats, Discord count, contributors | /about/community |

---

## 4. Nueva Arquitectura CSS (Consolidada)

### 4.1 Problema Actual

28 archivos CSS separados (4,567 líneas) → **mantenimiento insostenible**.

### 4.2 Solución Propuesta

```
web/src/styles/
├── tokens.css              (170 lines) → KEEP
├── neubrutalism.css        (474 lines) → KEEP, expandir
├── buttons.css             (100 lines) → MERGE into neubrutalism.css
├── animations.css          (206 lines) → MERGE into neubrutalism.css
├── layout.css              (374 lines) → MERGE into neubrutalism.css
├── utilities.css           (263 lines) → MERGE into neubrutalism.css
├── index.css               (25 lines)  → MERGE into neubrutalism.css
├── swiss-grid.css          (311 lines) → RENAME to nb-grid.css
├── nav.css                 (304 lines) → MOVE to components/Nav.css
├── footer.css              (126 lines) → MOVE to components/NbFooter.css
├── hero.css                (295 lines) → DELETE, inline in NbTerminalHero
├── *.css (16 más)          (1,719 lines) → EVALUATE each for merge/delete
│
└── (target: 3-4 files + CSS modules per component)
```

**Target final:**
```
web/src/styles/
├── tokens.css              Design tokens (colores, spacing, shadows, easing)
├── nb-base.css             Reset + typography + layout + utilities + buttons
└── nb-components.css       Clases compartidas (frame, card, telemetry, ticker, etc.)
```

Cada componente puede tener su propio CSS module (`.module.css`) si necesita estilos específicos no compartidos.

### 4.3 Nuevos Elementos CSS (del research doc)

```css
/* Shadow más agresivo */
--shadow-brutal: 8px 8px 0px 0px #111111;

/* Split-flap animation */
@keyframes nb-split-flip {
  0% { transform: translateY(0); }
  50% { transform: translateY(-50%); }
  100% { transform: translateY(-100%); }
}

/* Marquee */
@keyframes nb-marquee {
  0% { transform: translateX(0); }
  100% { transform: translateX(-50%); }
}

/* Dithering filter */
.nb-dither {
  filter: url("data:image/svg+xml,...");
  image-rendering: pixelated;
}

/* Hover reveal blocks */
.nb-reveal {
  clip-path: inset(0 100% 0 0);
  transition: clip-path 150ms var(--ease-brutal);
}
.nb-reveal:hover {
  clip-path: inset(0 0 0 0);
}

/* Terminal scanline */
.scanline::after {
  content: '';
  position: fixed;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0,0,0,0.08) 2px,
    rgba(0,0,0,0.08) 4px
  );
  pointer-events: none;
  z-index: 9999;
}
```

---

## 5. Nueva Landing Page — Sección por Sección

### 5.1 Terminal Hero

```
┌──────────────────────────────────────────────────────────┐
│ [ NAV ]  VantaDB  ENGINE  BENCHMARKS  PRICING  [ DOCS ] │
├──────────────────────────────────────────────────────────┤
│ ┌─────────────────────────┐  ┌────────────────────────┐  │
│ │  ┌──────────────────┐  │  │  > VantaDB v0.4.2      │  │
│ │  │ The database     │  │  │  > Loading...           │  │
│ │  │ that thinks      │  │  │  > Connected to :memory │  │
│ │  │ with you.        │  │  │  > INSERT 10k vectors.. │  │
│ │  │                  │  │  │  > Done (1.2ms)         │  │
│ │  │ ┌──────────┐     │  │  │  > SELECT * WHERE       │  │
│ │  │ │ pip inst │     │  │  │  > similarity > 0.95    │  │
│ │  │ └──────────┘     │  │  │  ─────────────────────  │  │
│ │  │                  │  │  │  │ id │ score │ meta  │  │
│ │  │ Read the Docs →  │  │  │  ├────┼───────┼───────┤  │
│ │  └──────────────────┘  │  │  │ 001 │ 0.98  │ ...   │  │
│ └─────────────────────────┘  └────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

- **Left**: Headline + CTA + quickstart
- **Right**: Terminal emulator animado con query en vivo
- **Background**: Dot grid + noise overlay
- **Entry**: Terminal typewriter 30ms/char, results instant

### 5.2 Trust Bar

```
┌──────────────────────────────────────────────────────────┐
│ ⭐ 2,847 GitHub ★   │   1.2ms p50   │   0.998 R@10    │
│ Used by: [Logo] [Logo] [Logo] [Logo]                    │
└──────────────────────────────────────────────────────────┘
```

- Números reales con `NbTicker` animado
- Logos de Simple Icons (GitHub, Docker, PyPI)
- Sin labels debajo de logos

### 5.3 Features Bento

```
┌──────────────────┬─────────┬──────────────────┐
│    2×2           │  1×1    │     2×1          │
│ Hybrid Search    │  WAL    │  0-ops           │
│ HNSW + BM25      │        │  Zero Servers    │
│ in one query     │        │                  │
├──────────────────┼────────┼──────────────────┤
│    1×1           │  2×1        │   1×1        │
│ PyO3 Native      │  SQL + Vector + FTS       │
│ Python bindings  │  Converged query engine   │
├──────────────────┼───────────────────────────┤
│    3×1: Embed anywhere — 2MB binary          │
└──────────────────────────────────────────────┘
```

- 7 celdas asimétricas (nunca grid idéntico)
- Featured cell (2×2) para Hybrid Search
- Hover: border → amber, shadow → amber

### 5.4 Quickstart (3-Step Code Blocks)

```
┌──────────────────────────────────────────────────────────┐
│ [ QUICKSTART ]                                            │
│                                                          │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│ │ [01] INSTALL │  │ [02] CONNECT │  │ [03] QUERY   │    │
│ │ $ pip inst.. │  │ $ import..   │  │ $ results =  │    │
│ │ └────────────┘  │              │  │   db.query() │    │
│ │                  │              │  │              │    │
│ │ → Copy          │ → Copy       │  │ → Run        │    │
│ └────────────────┘ └────────────┘  └──────────────┘    │
└──────────────────────────────────────────────────────────┘
```

- 3 columnas con código real
- Copy button en cada bloque
- "Run" button abre playground
- NbFrame con label `[ INSTALL ]` etc.

### 5.5 Architecture Preview

```
┌──────────────────────────────────────────────────────────┐
│ [ ARCHITECTURE ] — 6 layers clickable                    │
│                                                          │
│ ┌────────────────────────────────────────────────────┐   │
│ │  Python SDK (PyO3)  Rust SDK  C API  │             │   │
│ ├────────────────────────────────────────────────────┤   │
│ │  SQL Parser    │  Vector Index (HNSW)  │  BM25 FTS │   │
│ ├────────────────────────────────────────────────────┤   │
│ │              Query Engine (Cost-based)              │   │
│ ├────────────────────────────────────────────────────┤   │
│ │  Write-Ahead Log (WAL)  │  In-Memory Store         │   │
│ ├────────────────────────────────────────────────────┤   │
│ │  Storage Engine (SQLite VFS + DuckDB + custom)     │   │
│ ├────────────────────────────────────────────────────┤   │
│ │  Disk I/O  │  Memory-mapped  │  Network (optional) │   │
│ └────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

- SVG-based, 6 capas horizontales
- Click en capa → expande detalle debajo
- Hover: capa se ilumina en amber, otras se atenúan
- Entrada: staggered reveal desde abajo (60ms gap)

### 5.6 Benchmark Race

```
┌──────────────────────────────────────────────────────────┐
│ [ BENCHMARKS ]                                           │
│                                                          │
│ Hybrid Query (p50 latency, lower is better)              │
│                                                          │
│ VantaDB    ████████████████████████████  1.2ms           │
│ Chroma     ████████████████████████████████████████  4.8ms│
│ Pinecone   █████████████████████████████████████████  7.3ms│
│ Qdrant     ████████████████████████████████  3.1ms       │
│                                                          │
│ Recall@10 (higher is better)                             │
│ VantaDB    ████████████████████████████████████████  0.998│
│ Chroma     ████████████████████████████████████  0.945   │
│ SQlite+vec ████████████████████████████████  0.890       │
└──────────────────────────────────────────────────────────┘
```

- Barras horizontales con D3 animado
- VantaDB en amber, resto en steel
- Count-up en entrada (200ms)
- Tooltip con detalle en hover (NbBlockAmber reveal)

### 5.7 Ecosystem Grid

```
┌──────────────────────────────────────────────────────────┐
│ [ ECOSYSTEM ]                                            │
│                                                          │
│ Frameworks        │  LLM Providers     │  Memory          │
│ ┌────┐ ┌────┐     │  ┌────┐ ┌────┐     │  ┌────┐ ┌────┐ │
│ │LC  │ │LI  │     │  │OA  │ │AN  │     │  │M0  │ │LT  │ │
│ └────┘ └────┘     │  └────┘ └────┘     │  └────┘ └────┘ │
│ Deployment        │  Cloud             │                  │
│ ┌────┐ ┌────┐     │  ┌────┐ ┌────┐     │                  │
│ │DKR │ │PYPI│     │  │AWS │ │GCP │     │                  │
│ └────┘ └────┘     │  └────┘ └────┘     │                  │
└──────────────────────────────────────────────────────────┘
```

- NbBento con celdas agrupadas por categoría
- Iconos + labels ALL CAPS
- Hover: icono → amber, border → amber

### 5.8 Pricing Preview

```
┌────────────────────────┬─────────────────────────┐
│  FREE                  │  ENTERPRISE              │
│  ∞                     │  Custom                  │
│  vectors               │                          │
│                        │  On-prem deploy          │
│  Single-node           │  SSO / SAML              │
│  10M vectors           │  Audit trails            │
│  Community support     │  Priority SLA            │
│                        │                          │
│  [ GET STARTED ]       │  [ CONTACT SALES ]       │
└────────────────────────┴─────────────────────────┘
```

- Solo 2 tiers en Home (Free + Enterprise)
- Pricing completo en /pricing con 3 tiers
- Hard shadow en cards, CTA amber

### 5.9 FAQ Section

```
┌──────────────────────────────────────────────────────────┐
│ [ FAQ ]                                                   │
│                                                          │
│ ▼ When should I use VantaDB vs Chroma?                   │
│   VantaDB is best when you need...                       │
│                                                          │
│ ▶ Can I use VantaDB in production?                       │
│                                                          │
│ ▶ How does VantaDB compare to SQLite + vec0?              │
│                                                          │
│ ▶ Do I need a server?                                    │
│                                                          │
│  [ View full FAQ → ]                                     │
└──────────────────────────────────────────────────────────┘
```

- NbFaqAccordion con máximo 4 preguntas
- "View full FAQ →" link a /why-vantadb
- Frame con label `[ FAQ ]`

### 5.10 Monolith CTA

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│              pip install vantadb-py                      │
│                    ▊ (blinking cursor)                     │
│                                                          │
│         Zero servers. One line. Infinite context.         │
│                                                          │
│            [ GET STARTED ]  Read Documentation →          │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

- Bloque gigante centrado
- `pip install` en hero font size
- Cursor parpadeante
- Background: cross grid pattern

---

## 6. Sistema de Documentación

### 6.1 Recomendación: Docusaurus

| Criterio | Peso | Docusaurus | VitePress | Starlight |
|---|---|---|---|---|
| Search nativo | Alto | ✅ (local search plugin) | ✅ (MiniSearch) | ✅ (PageFind) |
| Versioning | Alto | ✅ Built-in | ❌ Manual | ✅ Built-in |
| MDX | Alto | ✅ Nativo | ✅ Nativo | ✅ MDX/Markdoc |
| API Docs auto | Medio | ✅ Redoc plugin | ❌ Manual | ❌ Manual |
| React custom components | Alto | ✅ Nativo | ⚠️ Vue | ⚠️ Astro |
| i18n | Medio | ✅ Crowdin | ⚠️ Config | ✅ Built-in |
| Self-host | Alto | ✅ MIT | ✅ MIT | ✅ MIT |
| OSS adoption | Medio | ✅ Muy alto | ✅ Alto | ⚠️ Creciente |

**Veredicto:** Docusaurus porque VantaDB necesita componentes React interactivos (playground WASM, benchmark charts, architecture diagrams).

### 6.2 Estructura del Docs Site

```
docs.vantadb.com/
├── /                       Landing de docs
├── /quickstart             Getting Started (5min)
├── /guides/
│   ├── /guides/installation    PyPI, Cargo, Docker
│   ├── /guides/configuration   Environment, Feature flags
│   ├── /guides/deployment      Single-node, Multi-node
│   ├── /guides/migration       From Chroma/Pinecone/LanceDB
│   └── /guides/tuning          Performance, Memory, Storage
├── /sdk/
│   ├── /sdk/python             Python API reference
│   ├── /sdk/rust               Rust API reference
│   └── /sdk/examples           Cookbook
├── /api/                       Auto-generated from Rust code
├── /faq                        FAQ
└── /contributing               How to contribute

docs.vantadb.com/ → Docusaurus en subdominio
```

### 6.3 Integración con el Sitio Principal

- El site principal (vantadb.com) es Vite + React + TanStack Router
- Docs (docs.vantadb.com) es Docusaurus
- Nav link [DOCS] → docs.vantadb.com/quickstart
- Search unified via Algolia DocSearch (free for OSS)

---

## 7. Componentes Interactivos Clave

### 7.1 Playground WASM (Crítico)

```
┌──────────────────────────────────────────────────────┐
│  [ PLAYGROUND ]  Run VantaDB in your browser         │
│                                                      │
│  ┌──────────────────┐  ┌────────────────────────┐   │
│  │ // Query Editor │  │  Results                │   │
│  │                  │  │                        │   │
│  │ import { Vanta } │  │ ┌─────┬───────┬──────┐│   │
│  │ from 'vantadb'   │  │ │ id  │ score │ meta ││   │
│  │                  │  │ ├─────┼───────┼──────┤│   │
│  │ const db = new.. │  │ │ 001 │ 0.98  │ {...}││   │
│  │                  │  │ │ 002 │ 0.95  │ {...}││   │
│  │ [▶ RUN] [↺ RST] │  │ └─────┴───────┴──────┘│   │
│  └──────────────────┘  └────────────────────────┘   │
└──────────────────────────────────────────────────────┘
```

- **Implementación:** VantaDB compilado a WASM (via `wasm-pack`) + CodeMirror 6 editor
- **Latencia target:** < 100ms para queries simples
- **Benchmark mode:** Comparar HNSW vs brute force vs BM25

### 7.2 Architecture Diagram Interactivo

- SVG con 6 capas horizontales
- Click en capa → expand panel debajo con detalle técnico
- CSS transitions: opacidad 200ms, translateY 200ms
- Implementación: React + SVG + state local

### 7.3 Benchmark Dashboard Animado

- D3.js con barras horizontales
- Count-up animation en entrada
- Tooltip con detalle en hover
- Web Workers para cálculos pesados
- Respetar `prefers-reduced-motion`

### 7.4 Terminal Emulator (Hero)

- State machine: boot → connect → insert → query → done
- Typewriter effect: 30ms/char
- Output instantáneo (no fade)
- Timestamp en cada línea
- Auto-play on load, loop después de 5s pausa

---

## 8. Estrategia de Branding Técnico

### 8.1 Fotos del Equipo con Dithering

```css
.nb-team-img {
  filter: url("data:image/svg+xml,...floyd-steinberg...");
  image-rendering: pixelated;
  border: 2px solid var(--border-visible);
}
```

- Fotos convertidas a 1-bit dithering (Floyd-Steinberg)
- Borde 2px visible
- Sin border-radius
- Hover: revela imagen original (sin dither)

### 8.2 Split-Flap Metrics Display

```
┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐
│  1  ││  2  ││  .  ││  3  ││  4  ││  5  │
└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘
         p50 latency (ms)
```

- Para métricas en /engine, /benchmarks
- CSS animation con steps(10) — sin JS
- Cada dígito rota independientemente

### 8.3 Data Tickers (Marquee)

```
> INSERT 10k vectors... DONE (1.2ms) /// QUERY 1k results... DONE (0.8ms) /// ...
```

- Strip continuo en la parte inferior de sections oscuras
- Separador `///` en amber
- Velocidad: 30s por ciclo

### 8.4 ASCII Integration Logos

En lugar de logos de empresas reales (startup sin clientes conocidos):

```
┌──────────────────────────────────────────────────┐
│  BUILT WITH                                       │
│                                                    │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│  │ RUST │ │ PYTH │ │ DOCK │ │ GIT  │ │ VSX  │   │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
│                                                    │
│  RUST  PYTHON  DOCKER  GITHUB  VS CODE             │
└──────────────────────────────────────────────────┘
```

- Cajas ASCII 6×3 con texto ALL CAPS mono
- Hover: border → amber
- Link a GitHub / Docker Hub / VS Code Marketplace

---

## 9. Fases de Implementación

### Fase 0: Fundación (semana 1)
- [ ] CSS consolidation: `nb-base.css` + `nb-components.css`
- [ ] Rename components: `Swiss*` → `Nb*`
- [ ] Create atomic component library (NbLabel, NbFrame, NbCard, NbButton, etc.)
- [ ] Build NbPageHero + NbNav + NbFooter como componentes base
- [ ] Setup Docusaurus para docs.vantadb.com

### Fase 1: Home Page (semana 2)
- [ ] NbTerminalHero (terminal emulator)
- [ ] NbTrustBar (GitHub stars + metrics)
- [ ] NbFeatureGrid (bento cells)
- [ ] NbQuickstart (3-step code)
- [ ] NbArchPreview (architecture preview)
- [ ] NbBenchmarkRace (D3 bars)
- [ ] NbEcosystemGrid
- [ ] NbPricingCards (2 tiers)
- [ ] NbFaqAccordion
- [ ] NbMonolithCta

### Fase 2: Subpáginas Técnicas (semana 3)
- [ ] /engine — deep-dive con split-flap metrics
- [ ] /architecture — interactive diagram SVG
- [ ] /benchmarks — benchmark hub with D3
- [ ] /playground — WASM demo with CodeMirror
- [ ] /why-vantadb — FAQ + comparativas

### Fase 3: Docs (semana 3-4)
- [ ] Docusaurus setup + theming Swiss+Neubrutalism
- [ ] Quickstart guide
- [ ] Migration guides (from Chroma, Pinecone, LanceDB)
- [ ] SDK docs (Python + Rust)
- [ ] API reference auto-generated
- [ ] Search (Algolia DocSearch)

### Fase 4: Content Pages (semana 4)
- [ ] /pricing (3 tiers)
- [ ] /integrations (ecosystem grid)
- [ ] /solutions/* (3 use case pages)
- [ ] /about/team (team con dithering)
- [ ] /about/community (GitHub stats)
- [ ] /about/contact
- [ ] /blog (listing + slug)
- [ ] /changelog

### Fase 5: Polish + SEO (semana 5)
- [ ] Test responsive (375px, 768px, 1024px, 1440px)
- [ ] Accessibility audit (WCAG 2.2 AA)
- [ ] Performance audit (Lighthouse 90+)
- [ ] SEO audit (structured data, meta, OG)
- [ ] Reduced motion test
- [ ] SVG dithering for team photos
- [ ] Data tickers + marquees

---

## 10. Anti-Slop Checklist (Pre-Flight)

### Typography
- [ ] ALL display: Space Grotesk 700
- [ ] ALL body: Outfit 400/600
- [ ] ALL code/labels: JetBrains Mono
- [ ] tabular-nums en datos numéricos
- [ ] Labels ALL CAPS con 0.14em tracking
- [ ] Left-aligned (except Monolith CTA)

### Color
- [ ] Amber (#ff5500) único acento
- [ ] 95/5 rule: 95% B&W, 5% amber
- [ ] Sin purple, blue, teal, green decorative
- [ ] Contrast ≥ 4.5:1 body, ≥ 3:1 large text
- [ ] 0 gradients en toda la UI

### Neubrutalism (hard edges)
- [ ] 0px border-radius en TODA la UI
- [ ] Hard offset shadows (8px 8px 0px 0px)
- [ ] No blur/soft shadows
- [ ] Button press mechanics (hover reduce, active none)
- [ ] 2px borders visibles en cards, frames, inputs
- [ ] 1px gap en grids (hairline visible)
- [ ] Noise texture OR dot grid presente

### Layout
- [ ] 12-column grid, max-width 1200px
- [ ] Celdas variables en bento (nunca iguales)
- [ ] Una celda featured por bento
- [ ] Hero sin scroll (fit viewport)
- [ ] H1 ≤ 2 lines, subtext ≤ 20 words

### Motion
- [ ] Easing: `--ease-brutal` (primary) / `--ease-swiss`
- [ ] Animaciones ≤ 150ms UI, ≤ 300ms reveals
- [ ] Solo transform + opacity
- [ ] prefers-reduced-motion respetado
- [ ] 0 bounce, 0 elastic, 0 spring
- [ ] Marquees ≤ 1 por página

### Anti-Patterns Banned
- [ ] 0 side-stripe borders (accent bars on left)
- [ ] 0 gradient text (`background-clip: text`)
- [ ] 0 glassmorphism (except nav blur)
- [ ] 0 identical card grids
- [ ] 0 generic AI copy ("Elevate", "Seamless", "Unleash")
- [ ] 0 emoji as icons
- [ ] 0 stock photography
- [ ] 0 Inter, Roboto, Arial fonts

---

## 11. Métricas de Éxito

| Métrica | Target Actual | Target Post-Redesign |
|---|---|---|
| Lighthouse Performance | ~70 | ≥ 90 |
| Lighthouse Accessibility | ~80 | ≥ 95 |
| Lighthouse SEO | ~75 | ≥ 95 |
| Pagespeed LCP | ~3.5s | < 2.0s |
| Bundle size (gzip) | ~200KB | < 150KB |
| CSS files | 28 | 3-4 |
| Unique components | 20 (prod) | 30+ (reusable) |
| Blog posts | 4 | 12+ (en 3 meses) |
| Docs search | None | Algolia DocSearch |
| WASM playground | None | ✅ |
| Interactive diagrams | None | ✅ (architecture + benchmarks) |
| Mobile responsive | Partial | ✅ Full |
| WCAG compliance | Partial | ✅ AA minimum |
