# Motion & Choreography — Swiss Mechanical

> Fuente: `design/DiseñoNuevo.md §6` | Skills: `emil-design-eng`, `emilkowalski-motion`, `gsap-scrolltrigger`

---

## Decision Framework (emil-design-eng §2)

Antes de animar **cualquier** elemento, responder en orden:

1. **¿Debe animarse?** Si el usuario lo ve 100+ veces/día → NO
2. **¿Cuál es el propósito?** Feedback, spatial consistency, state indication, preventing jarring
3. **¿Qué easing?** Entrando → `ease-out`. Moviéndose → `ease-in-out`. Hover → `ease`. Constante → `linear`
4. **¿Cuán rápido?** UI < 300ms. Buttons 100-160ms. Tooltips 125-200ms. Modals 200-500ms

## Easing Curves (OBLIGATORIAS — nunca CSS defaults)

```css
--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);       /* Cortante mecánico */
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);         /* Strong ease-out UI */
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);     /* In-out on-screen */
```

### PROHIBIDO
- ❌ `ease-in` para UI (se siente sluggish)
- ❌ `linear` o `ease-in-out` genéricos de CSS
- ❌ bounce, elastic, spring suave

## Animaciones por Componente

### Nav

| Elemento | Duración | Easing | Propiedad |
|:---|:---|:---|:---|
| Link hover | 100ms | `--ease-swiss` | color |
| Mobile panel | 200ms | `--ease-out` | transform X |

### Botones

| Estado | Duración | Easing | Propiedad |
|:---|:---|:---|:---|
| Hover | 150ms | `--ease-swiss` | background |
| Active | 50ms | `--ease-swiss` | `scale(0.97)` |

### Cards

| Estado | Duración | Easing | Propiedad |
|:---|:---|:---|:---|
| Hover border | 100ms | `--ease-swiss` | border-color |
| Index label hover | 150ms | `--ease-swiss` | color `--steel` → `--amber` |
| Hover dim (architecture) | 200ms | `--ease-out` | opacity |

## GSAP ScrollTrigger (DiseñoNuevo §6.3)

| Animación | Técnica | Duración | Stagger |
|:---|:---|:---|:---|
| Hero title reveal | `clip-path: inset(0 0 100% 0)` → `inset(0)` | 400ms | — |
| Labels flash | Opacity/color flash | 200ms | 80ms |
| Benchmark cells | `scale(0)` → `scale(1)` desde grid edge | 300ms | 60ms |
| Count-up numbers | `gsap.utils.interpolate` | 200ms | — |
| Core Engine reveal | `clip-path: inset(100% 0 0 0)` → `inset(0)` + opacity | 250ms | 100ms |
| SVG stroke draw | `stroke-dashoffset` → 0 | 800ms | — |
| Architecture layers | `translateY` separación (exploded view) | 400ms | pin durante scroll |
| Ecosystem grid | Expand from grid lines, stagger | 200ms | 60ms |
| Use cases | Clip-path mask from left | 300ms | — |
| Monolith | Clip-path mask reveal | 400ms | — |

## Quickstart Terminal

- Typewriter: `30ms/char`
- Output appears instantly (no fade)
- Auto-play secuencial, restart after 3s
- Click en paso → salta a ese paso inmediatamente

## Microinteracciones (emil-design-eng §3)

| Elemento | Interacción |
|:---|:---|
| Active button | `scale(0.97)` — sensación mecánica |
| Card hover | Borde `--border` → `--border-strong` |
| Index label | Color `--steel` → `--amber` on parent hover |
| Icon hover | Color `--steel` → `--amber`, 150ms |
| Ecosystem cell hover | Fondo `--amber-dim`, borde `--amber` |

## Performance (OBLIGATORIO)

- Animar SOLO `transform` y `opacity` — nunca `top`, `left`, `width`, `height`
- `backdrop-blur` SOLO en elementos fixed/sticky (nav)
- `will-change: transform` solo en animación activa
- Geometría Three.js mínima (< 1500 polígonos)
- Sin post-processing (bloom, DOF, SSAO)
- Canvas `opacity: 0.4` en mobile

## Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  .reveal-mask, .reveal-expand, .reveal-draw {
    animation: none;
    opacity: 1;
  }
  .swiss-hero-grid-line {
    opacity: 0.4;
  }
}
```

- Todas las animaciones GSAP: `scrollTrigger { toggleActions: "play none none reverse" }`
- `prefers-reduced-motion`: elementos visibles inmediatamente sin animación
- Three.js: rotación OFF, wireframe estático visible
