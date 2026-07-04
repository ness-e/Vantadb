---
title: "Visual Identity — VantaDB Web"
status: active
tags: [vantadb, web, brand]
last_reviewed: 2026-07-03
aliases: []
---

# Visual Identity — VantaDB Web

> Resumen ejecutivo visual. Para especificación completa, ver `design/DiseñoNuevo.md`.

---

## Color System

| Token | Valor | Uso |
|:---|:---|:---|
| `--background` | `#f9f8f6` | Lienzo warm paper |
| `--foreground` | `#000000` | Negro absoluto |
| `--amber` | `#ff5500` | Safety Orange — ÚNICO acento |
| `--surface` | `#ffffff` | Tarjetas resting |
| `--border` | `oklch(15% 0.008 265)` | Líneas 1px |
| `--block-dark-bg` | `#0a0a0a` | Bloques invertidos OLED |

**Regla 95/5**: 95% monocromático, 5% naranja para señales activas.

## Typography

| Rol | Familia | Peso | Escala |
|:---|:---|:---|:---|
| Display | Space Grotesk | 700 | `clamp(3.8rem, 8vw, 7.5rem)` |
| Body | Outfit | 400 | `1.05rem` |
| Mono/Label | JetBrains Mono | 600 | `0.72rem` (ALL CAPS) |

**Prohibidas**: Inter, Roboto, Arial, Open Sans, Helvetica.

## Grid System

- **12 columnas** CSS Grid, gap `1px`
- Líneas de grid visibles como elemento de diseño
- Asimetría obligatoria cuando `DESIGN_VARIANCE > 4`
- Macro-spacing: `96px`–`160px` entre secciones

## Layout Principles

- `text-align: left` por defecto (excepción: CTA Monolith centrado)
- Sin sombras (`box-shadow: none`)
- Sin border-radius > 6px (botones: 0px)
- Geometría 90° — proporciones basadas en múltiplos de 8px
- Bordes 1px delinean zonas de información
- Contraste invertido: warm paper ↔ OLED `#0a0a0a`
