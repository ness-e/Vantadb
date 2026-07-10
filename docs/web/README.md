---
title: "VantaDB Web — Design System & Documentation"
status: active
tags: [vantadb, web, design-system]
last_reviewed: 2026-07-04
aliases: []
---

# VantaDB Web — Design System & Documentation

> Estilo: **Swiss + Neubrutalism**
> Filosofía: "Si el diseño corporativo es un sedán familiar, VantaDB es un coche de rally Grupo B"
> Skills: `vanta-design-orchestrator`, `industrial-brutalist-ui`, `design-taste-frontend`

---

## Estructura

```
docs/web/
├── README.md                          ← Este archivo — índice maestro
├── brand/                             ← ⏳ Planificado — Identidad de marca y verbal
│   ├── BRAND_PLATFORM.md              ← ⏳ BMC, purpose, vision, values, archetypes
│   ├── VERBAL_IDENTITY.md             ← ⏳ Voice, tone, writing principles, glossary
│   ├── LOGO_USAGE.md                  ← ⏳ Logo especificaciones y usos
│   └── VISUAL_IDENTITY.md             ← ⏳ Resumen visual ejecutivo (color, tipografía, grid)
├── design/                            ← Especificación visual y técnica
│   ├── TOKEN_SYSTEM.md                ← TODOS los tokens CSS del sistema
│   ├── COMPONENT_SPEC.md              ← Especificación detallada de cada componente
│   ├── ICON_SYSTEM.md                 ← Sistema de iconografía (nb-icon-box)
│   ├── MOTION_CHOREOGRAPHY.md         ← Animación, easing, snap-fast mechanics
│   └── SUB_PAGE_PATTERNS.md           ← Patrón común de subpáginas
├── product/                           ← Producto y contenido
│   ├── PRODUCT.md                     ← Product purpose, users, personality
│   └── SITE_MAP.md                    ← Inventario completo de rutas con estado
├── qa/                                ← Calidad, auditoría, accesibilidad
│   ├── REPORTE-DE-REVISION.md         ← Auditoría visual vs diseño actual
│   ├── NEUBRUTALIST_CHECKLIST.md      ← Pre-Flight Checklist neubrutalista
│   └── ACCESSIBILITY_STATEMENT.md     ← Declaración de accesibilidad y cumplimiento
└── strategy/                          ← Análisis competitivo
    └── COMPETITIVE_ANALYSIS.md        ← Análisis competitivo
```

## Convenciones de Nomenclatura

| Tipo | Formato | Ejemplo |
|:---|:---|:---|
| Documentos fuente | `UPPER_SNAKE_CASE.md` | `BRAND_PLATFORM.md` |
| Documentos técnicos | `UPPER_SNAKE_CASE.md` | `TOKEN_SYSTEM.md` |
| Este archivo | `README.md` | — |

## Documentos Clave

| Documento | Propósito |
|:---|:---|
| `design/TOKEN_SYSTEM.md` | **Fuente de verdad** — todos los tokens CSS |
| `design/COMPONENT_SPEC.md` | **Especificación de componentes** |
| `qa/NEUBRUTALIST_CHECKLIST.md` | **Pre-flight** — verificación obligatoria antes de merge |
| `qa/REPORTE-DE-REVISION.md` | **Auditoría** — estado actual vs especificación |

## Estándares Transversales

- **Hard offset shadows**: `--shadow-sm: 4px 4px 0px 0px #000000` — NUNCA sombras difusas
- **Border-radius: 0px** en TODO el sistema — sin excepciones
- **Sin gradientes decorativos**: cero permitido
- **Bordes visibles de 2px**: `border: 2px solid var(--border-visible)`
- **Color funcional**: Accent Rust `#ff5500` SOLO para señales activas y CTAs (regla 95/5)
- **Tipografía**: Space Grotesk (display), Outfit (body), JetBrains Mono (código/datos)
- **Animaciones**: 50-150ms, snap-fast, `--ease-brutal`, sin bounce/elastic
- **Noise texture y dot grid** como fondos de textura
- **Telemetría**: prefijos `>`, brackets ASCII, labels monospace
