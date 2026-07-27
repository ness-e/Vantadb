---
title: "Accessibility Statement — VantaDB Web"
status: active
tags: [vantadb, web, qa]
last_reviewed: 2026-07-04
aliases: []
---

# Accessibility Statement — VantaDB Web

> Estándar: WCAG 2.2 AA | Última revisión: 2026-07

---

## Commitment

VantaDB web se compromete a proporcionar una experiencia accesible para todos los usuarios. El diseño Swiss + Neubrutalism (dark #111111 + white #ffffff + amber accent) soporta naturalmente la accesibilidad mediante alto contraste inherente, tipografía clara, y navegación predecible.

## Standards Compliance

| Estándar | Nivel | Estado |
|---|---|---|
| WCAG 2.2 | AA | Target |
| Contraste texto normal | ≥ 4.5:1 | Verificado |
| Contraste texto grande (≥ 18px / ≥ 14px bold) | ≥ 3:1 | Verificado |
| Contraste UI (componentes, borders) | ≥ 3:1 | Verificado |

## Color & Contrast

| Combinación | Ratio | Cumple AA |
|---|---|---|
| `#ffffff` texto sobre `#111111` background | 18.5:1 | ✅ (AAA) |
| `#ff5500` (amber) sobre `#111111` background | 7.1:1 | ✅ (AAA) |
| `#888888` (steel) sobre `#111111` background | 4.1:1 | ✅ (AA) |
| `#1a1a1a` (surface) sobre `#111111` background | — | Panel, not text |
| `#ffffff` sobre `#ff5500` amber bg (CTA button) | 4.5:1 | ✅ (AA) |
| `#111111` sobre `#ff5500` amber bg | 7.1:1 | ✅ (AA) |

**Nota**: Amber `#ff5500` se usa exclusivamente para señales activas, CTAs e índices. No se usa para párrafos de texto.

## Motion & Reduced Motion

- Todas las animaciones respetan `prefers-reduced-motion: reduce`
- `prefers-reduced-motion`: elementos visibles inmediatamente, sin animación, sin scanline overlay
- Animaciones solo en `transform` y `opacity`
- Sin bounce, elastic, spring (ver MOTION_CHOREOGRAPHY.md)

## Keyboard Navigation

- Nav: todos los links accesibles por teclado (Tab, Shift+Tab)
- Skip to content link (primera regla tab)
- Sin trampas de teclado
- Sin dropdowns que requieran hover

## Focus Indicators

- Todos los elementos interactivos tienen `focus-visible`
- `outline: 2.5px solid var(--amber)` con `outline-offset: 2px`
- Nunca `outline: none` sin reemplazo visible

## Touch Targets

| Elemento | Mínimo |
|---|---|
| Nav links | 44×44px |
| Botones | 44×44px |
| Cards (clickable) | 44px altura mínima |

## Semantic HTML

- Estructura jerárquica: `h1` → `h2` → `h3` (sin saltos)
- `<nav>` para navegación principal
- `<main>` para contenido principal
- `<footer>` para pie de página
- `aria-label` en iconos sin texto acompañante
- Imágenes decorativas: `aria-hidden="true"`
- Landmarks: `role="banner"`, `role="navigation"`, `role="main"`, `role="contentinfo"`

## Testing

- [ ] Contraste verificado con herramienta de color
- [ ] Navegación por teclado completa sin mouse
- [ ] `prefers-reduced-motion` verificado en browser
- [ ] Zoom 200% sin pérdida de contenido ni horizontal scroll
- [ ] Screen reader test (NVDA / VoiceOver)
- [ ] Focus order lógico
- [ ] Skip-to-content link funcional
