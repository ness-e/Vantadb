---
title: "Neubrutalist Pre-Flight Checklist"
status: active
tags: [vantadb, web, qa]
last_reviewed: 2026-07-04
aliases: []
---

# Neubrutalist Pre-Flight Checklist

> Obligatorio antes de mergear cualquier cambio visual.

---

## §1 Border-Radius

- [ ] `border-radius: 0` en TODOS los elementos
- [ ] Sin excepciones — ni 2px, ni 4px, ni 6px
- [ ] Botones, cards, inputs, badges, imágenes — todo 0px
- [ ] Sin `border-radius` en pseudo-elementos

## §2 Shadows (Hard Offset)

- [ ] Solo sombras hard offset: `Xpx Ypx 0px 0px color`
- [ ] Blur siempre `0` — sin `box-shadow: ... blur(...)` 
- [ ] Sin `box-shadow: none` ni `box-shadow: 0 0 0 transparent` en resting state
- [ ] Button shadow pattern: resting → `--shadow-md`, hover → `--shadow-sm`, active → `none`
- [ ] Card shadow: `--shadow-sm` resting, `--shadow-amber` on hover

## §3 Color System

- [ ] Background dominante: `#111111` (--background)
- [ ] Texto principal: `#ffffff` (--foreground)
- [ ] Sin color de acento secundario — solo amber `#ff5500` (95/5 rule)
- [ ] Amber SOLO para: CTAs, labels activos, índices, arrows, alerts
- [ ] Sin gradientes decorativos — flat colors everywhere
- [ ] Sin grises cálidos — todos los grises son neutros/fríos (#888, #2a2a2a, #333)

## §4 Typography

- [ ] Space Grotesk para display/titles
- [ ] Outfit para body text
- [ ] JetBrains Mono para labels, code, telemetry, metadata
- [ ] ALL LABELS UPPERCASE con `letter-spacing: 0.14em` 
- [ ] `font-variant-numeric: tabular-nums` en datos numéricos
- [ ] `text-align: left` en bloques de contenido
- [ ] Títulos hero max 2 líneas desktop

## §5 Borders

- [ ] Bordes visibles en cards, frames, icon boxes (2px solid)
- [ ] Nav bottom border: `2px solid var(--border-visible)`
- [ ] Grid separators usan `gap: 1px` con background del borde
- [ ] Sin bordes invisibles ni basados en padding/spacing

## §6 Layout

- [ ] Macro-spacing ≥ 96px entre secciones
- [ ] Grid asimétrico — preferir 7fr/5fr, 8fr/4fr sobre columnas iguales
- [ ] Sin 3-card feature rows idénticas
- [ ] Cada layout family aparece max 1 vez por página
- [ ] Max 2 zigzag consecutivos
- [ ] Bento cells = exact content count

## §7 Anti-Slop

- [ ] Sin purple/blue AI glow gradients
- [ ] Sin copys genéricos: "Elevate", "Seamless", "Unleash", "Supercharge"
- [ ] Sin ilustraciones 3D de plástico brillante
- [ ] Sin fotos de stock
- [ ] Sin emojis en código ni markup
- [ ] Sin emojis en texto visible
- [ ] Sin customer logos como imágenes — solo ASCII list o nb-list

## §8 Hero

- [ ] Hero cabe en viewport inicial
- [ ] Subtext max 20 words
- [ ] Max 4 text elements (eyebrow + H1 + subtext + CTAs)
- [ ] Sin feature list ni pricing teaser dentro del hero
- [ ] Fondo: noise overlay + scanline + dark bg

## §9 Motion

- [ ] Animaciones ≤ 150ms para UI (80ms default)
- [ ] Easing: `--ease-brutal` (cubic-bezier(0.05, 0.95, 0.3, 1))
- [ ] Sin `ease-in`, `ease-out`, `ease-in-out` CSS defaults
- [ ] Sin bounce, elastic, spring, ni overshoot
- [ ] Solo `transform` y `opacity` animados
- [ ] `backdrop-blur` solo en nav (fixed)
- [ ] `prefers-reduced-motion` respetado
- [ ] Stagger 50-80ms máximo

## §10 Accesibilidad

- [ ] Contraste WCAG AA en todos los textos (4.5:1 body, 3:1 large)
- [ ] Button text contraste verificado contra background
- [ ] `focus-visible` amber outline, 2.5px, offset 2px
- [ ] Touch targets ≥ 44px en mobile
- [ ] Nav single-line en desktop
- [ ] Skip-to-content link presente
- [ ] Form inputs con label, helper, error text

## §11 Telemetry Elements

- [ ] `>` prefix en data rows y lists
- [ ] `[ LABEL ]` bracket notation en section headers
- [ ] Mono labels para metadata
- [ ] Índices numéricos `[01]`, `[02]` en cards

## §12 Background Textures

- [ ] `.scanline` overlay presente (fixed, z-index 9999)
- [ ] `.noise-overlay` presente (3.5% opacity)
- [ ] Dark sections usan `.nb-bg-cross` o `.nb-bg-dot`
- [ ] Texturas no rompen legibilidad del contenido
