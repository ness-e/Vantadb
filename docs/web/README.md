# VantaDB Web — Design System & Documentation

> Estilo: **Swiss High-Contrast Minimal (Neon Precision)**
> Documento maestro: `design/DiseñoNuevo.md`
> Skills: `vanta-design-orchestrator`, `industrial-brutalist-ui`, `design-taste-frontend`

---

## Estructura

```
docs/web/
├── README.md                          ← Este archivo — índice maestro
├── brand/                             ← Identidad de marca y verbal
│   ├── BRAND_PLATFORM.md              ← BMC, purpose, vision, values, archetypes
│   ├── VERBAL_IDENTITY.md             ← Voice, tone, writing principles, glossary
│   ├── LOGO_USAGE.md                  ← Logo especificaciones y usos
│   └── VISUAL_IDENTITY.md             ← Resumen visual ejecutivo (color, tipografía, grid)
├── design/                            ← Especificación visual y técnica
│   ├── DiseñoNuevo.md                 ← MASTER — 582 líneas de especificación completa
│   ├── COMPONENT_LIBRARY.md           ← Catálogo de componentes Swiss
│   ├── ICON_SYSTEM.md                 ← Sistema de iconografía monoline
│   ├── MOTION_CHOREOGRAPHY.md         ← Animación, easing, microinteracciones
│   └── SUB_PAGE_PATTERNS.md           ← Patrón común de subpáginas
├── product/                           ← Producto y contenido
│   ├── PRODUCT.md                     ← Product purpose, users, personality
│   └── SITE_MAP.md                    ← Inventario completo de 30+ rutas con estado
├── qa/                                ← Calidad, auditoría, accesibilidad
│   ├── REPORTE-DE-REVISION.md         ← Auditoría visual vs DiseñoNuevo (2026-06-23)
│   ├── SWISS_CHECKLIST.md             ← Pre-Flight Checklist portable (DiseñoNuevo §13)
│   └── ACCESSIBILITY_STATEMENT.md     ← Declaración de accesibilidad y cumplimiento
├── strategy/                          ← Plan de implementación y fases
│   ├── implementation_plan.md         ← MASTER — índice y decisiones resueltas
│   ├── PHASE_0_1_FOUNDATIONS.md       ← Tokens, grid, Nav, Footer
│   ├── PHASE_2_INDEX.md               ← Landing page completa (8 secciones)
│   ├── PHASE_3_SUBPAGES.md            ← Subpáginas técnicas (Engine, Arch, Integrations)
│   └── PHASE_4_5_REMAINING.md         ← Métricas, Solutions, Docs, Pricing, About, Blog
└── tools/                             ← Herramientas de desarrollo y QA
    └── PLAYWRIGHT_CLI.md              ← Playwright CLI para revisión visual
```

## Convenciones de Nomenclatura

| Tipo | Formato | Ejemplo |
|:---|:---|:---|
| Documentos fuente | `UPPER_SNAKE_CASE.md` | `BRAND_PLATFORM.md` |
| Documentos técnicos | `UPPER_SNAKE_CASE.md` | `COMPONENT_LIBRARY.md` |
| Planes/fases | `PHASE_N_N_NAME.md` | `PHASE_0_1_FOUNDATIONS.md` |
| Este archivo | `README.md` | — |

## Documentos Clave

| Documento | Propósito |
|:---|:---|
| `design/DiseñoNuevo.md` | **Fuente de verdad** — toda decisión visual se origina aquí |
| `strategy/implementation_plan.md` | **Plan de implementación** — decisiones resueltas y fases |
| `qa/REPORTE-DE-REVISION.md` | **Auditoría** — estado actual vs especificación |
| `qa/SWISS_CHECKLIST.md` | **Pre-flight** — verificación obligatoria antes de merge |

## Estándares Transversales

- **Sin sombras**: `box-shadow: none` en todo el sistema (DiseñoNuevo §5.1)
- **Sin border-radius > 6px**: máximo absoluto (DiseñoNuevo §5.2)
- **Sin gradientes decorativos**: cero permitido
- **Sin ilustraciones 3D de plástico**: wireframes geométricos únicamente
- **Color funcional**: Safety Orange `#ff5500` SOLO para señales activas y CTAs (regla 95/5)
- **Tipografía**: Space Grotesk (display), Outfit (body), JetBrains Mono (código/datos)
- **Animaciones**: Solo `transform` y `opacity`, easing Swiss cortante, ≤ 300ms
