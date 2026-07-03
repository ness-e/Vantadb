# Subpage Design Patterns — Swiss High-Contrast Minimal

> Fuente: `design/DiseñoNuevo.md §12` | Versión: 1.0 | 2026-07

---

## Patrón Común de Subpágina

Toda subpágina sigue esta estructura:

```
┌─────────────────────────────────────────┐
│  [SwissSubpageHero]                     │
│  label: "ENGINE"                        │
│  title: "The Rust Core"                 │
│  breadcrumb: "Home / Engine"           │
│  description: "..."                     │
│  ───────────────────────────────────    │
│                                         │
│  Sección 1 (warm paper)                 │
│  ┌───────┬───────────────┐             │
│  │ SVG   │ Content       │             │
│  │ diagr.│ text          │             │
│  └───────┴───────────────┘             │
│                                         │
│  ───────────────────────────────────    │
│  Sección 2 (OLED oscuro)               │
│  ┌───────────────┬───────┐             │
│  │ Content text  │ SVG   │             │
│  └───────────────┴───────┘             │
│                                         │
│  ... (alternar warm/OLED)               │
│                                         │
│  [SwissMonolith] — CTA final           │
└─────────────────────────────────────────┘
```

### Elementos obligatorios

1. **SwissSubpageHero** — Hero compacto compartido (ver COMPONENT_LIBRARY.md)
2. **Secciones alternadas** — warm paper ↔ OLED `#0a0a0a`
3. **Diagramas SVG monoline** — con líneas de cota y etiquetas técnicas
4. **SwissMonolith** — CTA final en bloque OLED
5. **Borde 1px** — entre todas las secciones y tarjetas
6. **Macro-spacing** — `96px` entre secciones

### Layout de sección

Cada sección de contenido usa grid `5fr 7fr` (SVG izquierda + texto derecha) o invertido `7fr 5fr`. Alternar para evitar monotonía.

---

## SwissSubpageHero — Especificación

| Elemento | Estilo |
|:---|:---|
| Label | `[SECTION]` en `--text-label`, `--amber`, ALL CAPS |
| Breadcrumb | `Home / Section` en `--text-label`, `--steel` |
| Título | `--text-display`, Space Grotesk 700, cols 1-8 |
| Descripción | `--text-body`, `--muted`, max 20 words, cols 1-8 |
| Layout | Grid 12 columnas, asimetría intencional |
| Borde inferior | `1px solid var(--border)` full-width |

---

## Mapa de Subpáginas

### Fase 3 — Técnicas

| Ruta | Label | Título | Secciones |
|:---|:---|:---|:---|
| `/engine` | `ENGINE` | The Rust Core | HNSW, BM25, WAL, PyO3, Zero-Copy, SIMD |
| `/architecture` | `ARCHITECTURE` | Inside the Engine | Capas SVG interactivas |
| `/integrations` | `INTEGRATIONS` | Works With Everything | Grid por categoría |
| `/use-cases` | `USE CASES` | Real-World Applications | Casos expandidos |

### Fase 4 — Métricas

| Ruta | Label | Título | Secciones |
|:---|:---|:---|:---|
| `/cost` | `COST` | Total Cost of Ownership | Grid Bento, tabla comparativa |
| `/latency` | `PERFORMANCE` | Latency Benchmarks | Barras SVG, tabla, comparativa |
| `/storage` | `STORAGE` | Storage Architecture | Diagramas SVG, métricas |
| `/config` | `CONFIG` | Configuration Reference | Tablas + terminal |
| `/maint` | `MAINTENANCE` | Operations Guide | Pasos numerados + diagrama |
| `/changelog` | `CHANGELOG` | Release Notes | Timeline vertical |

### Fase 5 — Solutions, Docs, About

| Ruta | Label | Título |
|:---|:---|:---|
| `/solutions/ai-agents` | `SOLUTION` | AI Agent Memory |
| `/solutions/local-rag` | `SOLUTION` | Local RAG Pipeline |
| `/solutions/ai-ide-tooling` | `SOLUTION` | AI IDE Tooling |
| `/docs` | — | Documentation (2-col layout) |
| `/pricing` | `PRICING` | Simple, Transparent Pricing |
| `/about` | `ABOUT` | About VantaDB |
| `/about/company` | `ABOUT` | Company |
| `/about/community` | `ABOUT` | Community |
| `/about/contact` | `ABOUT` | Contact |
| `/blog` | — | Blog |
| `/blog/$slug` | — | Post detail |

---

## Reglas de Implementación

- **Hero**: Siempre usar `<SwissSubpageHero>` — nunca hero custom
- **Alternancia**: warm paper → OLED → warm paper (nunca dos OLED consecutivos)
- **Diagramas SVG**: Monoline 1.5px, solo contornos, geometría 90°
- **Benchmarks**: Barras horizontales SVG minimalistas, `tabular-nums`
- **Código**: Terminal con fondo `--void`, syntax highlighting mínimo
- **Formularios** (contact): Inputs rectangulares radius 0px, focus borde `--amber`
- **Blog list**: Grid 2 columnas desktop, cada post card con borde 1px
- **Pricing**: Plan destacado con borde `--amber`, feature table con iconos monoline ✓/✗
