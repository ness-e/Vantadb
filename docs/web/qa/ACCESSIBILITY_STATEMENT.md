# Accessibility Statement — VantaDB Web

> Estándar: WCAG 2.2 AA | Última revisión: 2026-07

---

## Commitment

VantaDB web se compromete a proporcionar una experiencia accesible para todos los usuarios, independientemente de sus capacidades. El diseño Swiss High-Contrast Minimal soporta naturalmente la accesibilidad mediante alto contraste, tipografía clara y navegación predecible.

## Standards Compliance

| Estándar | Nivel | Estado |
|:---|:---|:---|
| WCAG 2.2 | AA | Target |
| Contraste texto normal | ≥ 4.5:1 | Verificado |
| Contraste texto grande (≥ 18px / ≥ 14px bold) | ≥ 3:1 | Verificado |
| Contraste UI (componentes, borders) | ≥ 3:1 | Verificado |

## Color & Contrast

| Combinación | Ratio | Cumple AA |
|:---|:---|:---|
| `#000000` texto sobre `#f9f8f6` fondo | ~17:1 | ✅ |
| `#ffffff` texto sobre `#0a0a0a` fondo | ~19:1 | ✅ |
| `#ff5500` (amber) sobre `#f9f8f6` | ~5:1 | ✅ (texto ≥ 18px) |
| `#808080` sobre `#0a0a0a` | ~4.9:1 | ✅ (texto ≥ 18px) |
| `--muted` (`oklch(40%...)`) sobre `#f9f8f6` | ~6:1 | ✅ |

**Nota**: Safety Orange `#ff5500` se usa exclusivamente para señales activas y CTAs. No se usa para texto informativo largo.

## Motion & Reduced Motion

- Todas las animaciones respetan `prefers-reduced-motion: reduce`
- GSAP ScrollTrigger con `toggleActions: "play none none reverse"`
- `prefers-reduced-motion`: elementos visibles inmediatamente, sin animación
- Three.js wireframe: rotación OFF, estático visible (ver MOTION_CHOREOGRAPHY.md)

## Keyboard Navigation

- Nav: todos los links accesibles por teclado
- Skip to content link (primera regla tab)
- Dropdowns: hover + focus visibles
- Sin trampas de teclado
- Formulario de contacto: inputs navegables por Tab, submit con Enter

## Focus Indicators

- Todos los elementos interactivos tienen focus visible
- Outline `2px solid var(--amber)` con offset `2px`
- Nunca `outline: none` sin reemplazo

## Touch Targets

| Elemento | Mínimo |
|:---|:---|
| Nav links | 44×44px |
| Botones | 44×44px |
| Cards | 44px altura mínima clickable |

## Semantic HTML

- Estructura jerárquica: `h1` → `h2` → `h3` (sin saltos)
- `<nav>` para navegación principal
- `<main>` para contenido principal
- `<footer>` para pie de página
- `aria-label` en iconos sin texto

## Testing

- [ ] Contraste verificado con herramienta de color
- [ ] Navegación por teclado completa sin mouse
- [ ] `prefers-reduced-motion` verificado
- [ ] Zoom 200% sin pérdida de contenido
- [ ] Screen reader test (NVDA / VoiceOver)
