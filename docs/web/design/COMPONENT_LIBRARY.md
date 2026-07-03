# Component Library — Swiss High-Contrast Minimal

> Fuente: `design/DiseñoNuevo.md §8` | Versión: 1.0 | 2026-07

---

## Convenciones

- **Sin sombras**: `box-shadow: none` en todos los componentes
- **Border-radius**: 0px (botones, inputs), max 6px (terminales)
- **Transiciones**: 100-150ms, `var(--ease-swiss)`
- **Tokens**: todas las variables desde `tokens.css`
- **Sin gradientes**, sin efectos decorativos

---

## Nav (`SwissNav`)

| Prop | Valor |
|:---|:---|
| Altura | 64px (max 80px desktop) |
| Fondo | `var(--surface-glass)` + `backdrop-filter: blur(12px)` |
| Borde inferior | `1px solid var(--border)` |
| Layout | Logo izquierda \| Links centro \| CTA derecha |
| Links | `--text-label` (JetBrains Mono, 0.72rem, ALL CAPS, tracking 0.14em) |
| Link color resting | `--steel` |
| Link color hover | `--foreground` (100ms) |
| Link color active | `--amber` |
| Mobile | Hamburger → panel lateral `--surface` con borde 1px |

### Estados

| Estado | Link | CTA |
|:---|:---|:---|
| Resting | `--steel` | Primary amber |
| Hover | `--foreground` (100ms) | `background: #000` |
| Active route | `--amber` | — |
| Focus | Outline visible (WCAG) | Outline visible |

---

## Buttons

### Primary

| Prop | Valor |
|:---|:---|
| Background resting | `--amber` |
| Text | `#ffffff` |
| Background hover | `#000000` |
| Border-radius | `0px` |
| Padding | `10px 24px` |
| Transition | `150ms var(--ease-swiss)` |
| Active | `scale(0.97)` |

### Ghost

| Prop | Valor |
|:---|:---|
| Background resting | `transparent` |
| Border | `1px solid var(--border)` |
| Text | `--foreground` |
| Hover | `background: var(--border)`, text `#ffffff` |
| Ghost inverted (dark sections) | Border white, text white, hover: bg white + text black |

### Link

| Prop | Valor |
|:---|:---|
| Text | `--amber` |
| Underline | Animado left→right, 200ms |
| Sin borde, sin bg |

### Reglas

- Button text MUST fit 1 line (design-taste §4.5)
- No duplicate CTA intent per page (design-taste §4.5)

---

## Cards / Blocks (`SwissCard`)

| Prop | Valor |
|:---|:---|
| Fondo | `--surface` |
| Borde | `1px solid var(--border)` |
| Hover borde | `--border-strong` (100ms) |
| Padding | `24px` |
| Index label | `[01]` en `--text-label`, esquina superior izquierda |
| Sin sombras | `box-shadow: none` |
| Border-radius | `0px` (max `4px`) |

### Variants

| Variant | Modificación |
|:---|:---|
| Dark section card | Fondo transparent, borde `var(--block-dark-border)`, hover `rgba(255,255,255,0.03)` |
| Benchmark cell | Tamaños variables (bento), números display + count-up |
| Ecosystem card | Icono monoline + label ALL CAPS, hover icon → `--amber` |
| Use case card | Layout grid `3fr 9fr`, número display `--subtle` → `--amber` hover |

---

## Terminal / Code Block (`SwissTerminal`)

| Prop | Valor |
|:---|:---|
| Fondo | `--void` |
| Borde | `1px solid var(--border)` |
| Border-radius | `max 4px` |
| Header | 3 dots `--subtle` + título en `--text-label` |
| Font | JetBrains Mono |
| Syntax | keywords `--foreground`, strings `--amber`, comments `--muted` |
| Output | `border-left: 2px solid var(--amber)` |

---

## Footer (`SwissFooter`)

| Prop | Valor |
|:---|:---|
| Fondo | `#0a0a0a` (OLED) |
| Grid | 5 columnas |
| Links resting | `--block-dark-muted` (#808080) |
| Links hover | `#ffffff` |
| Column titles | `--text-label` ALL CAPS, `--block-dark-text` |
| Dividers | `1px solid rgba(255,255,255,0.08)` |
| Bottom bar | Mark + copyright + GitHub link |

---

## Hero (`SwissHero`)

| Prop | Valor |
|:---|:---|
| Fondo | `--background` (warm paper) |
| Layout | Grid 12 cols: título 1-8, vacío 9-12 (asimetría) |
| Etiquetas | `[RUST-NATIVE]` `[IN-PROCESS]` `[ZERO-SERVERS]` en `--amber` |
| Título | "VantaDB" en `--text-hero`, Space Grotesk 700 |
| Subtítulo | "Embedded cognitive memory..." en `--text-body`, `--muted` |
| CTAs | Primary (amber) + Ghost (borde negro) |

---

## Monolith CTA (`SwissMonolith`)

| Prop | Valor |
|:---|:---|
| Fondo | `#0a0a0a` OLED full-width |
| Padding | `160px` vertical |
| Texto | Centrado (excepción: CTA aislado) |
| Comando | `--text-hero`, `#ffffff` |
| Subtítulo | `--block-dark-muted` |
| Botón | Primary amber centrado |

---

## Subpage Hero (`SwissSubpageHero`)

| Prop | Valor |
|:---|:---|
| Props | `label`, `title`, `description`, `breadcrumb` |
| Layout | Grid 12 cols: título 1-8, asimetría |
| Label | `[ENGINE]` en `--text-label` naranja |
| Breadcrumb | `Home / Engine` en `--text-label`, `--steel` |
| Título | `--text-display` |
| Descripción | `--text-body`, `--muted` |
| Borde inferior | `1px solid var(--border)` full-width |
