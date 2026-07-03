# DiseñoNuevo.md — Swiss High-Contrast Minimal (Neon Precision) para VantaDB

> Documento maestro de especificación visual, interactiva y técnica.
> Versión: 3.0 | Fecha: 2026-07-02
> Sintetiza: `industrial-brutalist-ui`, `high-end-visual-design`, `emil-design-eng`,
> `design-taste-frontend`, `minimalist-ui`, `impeccable-design-polish`

---

## 0. Manifiesto

**"La precisión es el lujo. El espacio vacío es la confianza. El neón es la señal, no la decoración."**

VantaDB es un motor de base de datos embebido escrito en Rust. Su interfaz web comunica
**velocidad mecánica**, **rigor ingenieril** y **transparencia técnica**. No vendemos
promesas; mostramos datos, arquitectura y rendimiento real.

El estilo **Swiss High-Contrast Minimal (Neon Precision)** fusiona la disciplina de la
Escuela Suiza de Diseño (Müller-Brockmann, Hofmann, Ruder) con la estética de interfaces
de telemetría de alta precisión y las reglas anti-slop de producción de
`design-taste-frontend` y `impeccable-design-polish`.

### Design Read (Contexto del Agente)

> "Reading this as: B2B technical infrastructure landing for AI engineers,
> with a Swiss-industrial typographic language, leaning toward vanilla CSS +
> Space Grotesk + GSAP ScrollTrigger + Three.js wireframe."

### Dials (design-taste-frontend §1)

| Dial | Valor | Justificación |
|:---|:---|:---|
| `DESIGN_VARIANCE` | **7** | Asimetría Swiss, nunca caótico |
| `MOTION_INTENSITY` | **5** | Mecánico y cortante, nunca cinematic |
| `VISUAL_DENSITY` | **6** | Densidad tipo telemetría, respira con macro-spacing |

---

## 1. Principios Fundamentales

### 1.1 Del Swiss Clásico (1950-1970)

| Principio | Implementación |
|:---|:---|
| **Grid rígido** | CSS Grid 12 columnas, gap 1px sólido `var(--border)`. Grid tracks visibles como elementos de diseño (industrial-brutalist §5) |
| **Tipografía sans-serif** | Space Grotesk (display), Outfit (body), JetBrains Mono (datos/código). Nunca Inter, Roboto, Arial, Open Sans, Helvetica (high-end §2, minimalist §2) |
| **Asimetría** | Títulos arrancan en columna 1-8/12. Bloques desplazados del centro. ANTI-CENTER BIAS activo cuando `DESIGN_VARIANCE > 4` (design-taste §4.3) |
| **Jerarquía escalar** | Contraste de peso 400 vs 700 en la misma pantalla. Tamaños de `0.72rem` a `7.5rem` con `clamp()` (industrial-brutalist §3.1) |
| **Bandera izquierda** | `text-align: left` por defecto. Nunca justify. Centro SOLO en CTAs aislados (design-taste §4.3) |
| **Espacio negativo** | Macro-spacing `96px`–`160px` entre secciones. Columnas enteras vacías. Secciones usan `py-24` mínimo (high-end §4.C) |
| **Color funcional** | Safety Orange `#ff5500` SOLO para: hover/focus, datos críticos, CTAs principales. Regla 95/5 |
| **Sin decoración** | Cero ornamentos, cero `border-radius > 6px`, cero `box-shadow`, cero gradientes difusos |
| **Geometría 90°** | Proporciones basadas en múltiplos de 8px. Ángulos de 90° exclusivamente. `border-radius: 0px` en botones (industrial-brutalist §5) |

### 1.2 Anti-Slop Layer (Reglas que el agente DEBE verificar)

| Regla | Fuente |
|:---|:---|
| Nunca generar el mismo layout dos veces consecutivas | high-end §2 |
| Nunca usar 3-card feature rows genéricas | impeccable §4 |
| Nunca usar gradientes purple/blue sin razón de producto | design-taste §4.2 |
| Nunca usar copys genéricos: "Elevate", "Seamless", "Unleash", "Next-Gen" | minimalist §2 |
| Max 1 eyebrow por cada 3 secciones | design-taste §4.7 |
| Split-header (left big H + right small P) PROHIBIDO como default | design-taste §4.7 |
| Zigzag alternation max 2 secciones consecutivas | design-taste §4.7 |
| Section-Layout-Repetition Ban: cada layout family max 1 vez por página | design-taste §4.7 |
| Bento cells = exact content count, nunca celdas vacías | design-taste §4.7 |
| Hero MUST fit initial viewport: H1 max 2 líneas, subtext max 20 words | design-taste §4.7 |
| Hero top padding max `pt-24` (6rem) | design-taste §4.7 |
| Hero stack max 4 text elements (eyebrow + H1 + subtext + CTAs) | design-taste §4.7 |
| Nav single-line en desktop, height max 80px | design-taste §4.7 |
| Button text MUST fit 1 line en desktop | design-taste §4.5 |
| No duplicate CTA intent en la misma página | design-taste §4.5 |

---

## 2. Paleta de Color

### 2.1 Primarios

```
--background:       #f9f8f6          /* Warm paper (lienzo) */
--foreground:       #000000          /* Negro absoluto */
--amber:            #ff5500          /* Safety Orange — ÚNICO acento */
```

### 2.2 Superficies

```
--surface:          #ffffff          /* Tarjetas resting */
--surface-raised:   oklch(92% 0.003 85) /* Tarjetas hover */
--surface-glass:    rgba(249,248,246,0.85) /* Nav + blur */
--deep-space:       oklch(96% 0.003 85) /* Fondo alternativo */
--void:             oklch(94% 0.004 85) /* Terminales */
```

### 2.3 Bloques Invertidos (Secciones OLED)

```
--block-dark-bg:    #0a0a0a
--block-dark-text:  #f0f0f0
--block-dark-muted: #808080
--block-dark-border: rgba(255,255,255,0.08)
```

### 2.4 Estados

```
--amber-light:      #ff3300          /* Hover naranja */
--amber-dim:        rgba(255,85,0,0.08) /* Fondo activo sutil */
--success:          #00aa30
--danger:           #cc1100
```

### 2.5 Texto y Bordes

```
--muted:            oklch(40% 0.01 80)  /* Texto secundario */
--steel:            oklch(35% 0.01 240) /* Labels, metadatos */
--border:           oklch(15% 0.008 265) /* Líneas 1px */
--border-strong:    #000000             /* Bordes hover/acción */
--subtle:           oklch(88% 0.004 85) /* Guías secundarias */
```

### 2.6 Regla del 95/5

- **95%** monocromática (negro, blancos, grises)
- **5%** naranja Safety Orange para señales activas
- COLOR CONSISTENCY LOCK: el naranja se usa en TODA la página. No cambiar de accent mid-page (design-taste §4.2)

---

## 3. Tipografía

### 3.1 Familias

| Rol | Familia | Prohibidas |
|:---|:---|:---|
| **Display** | Space Grotesk 700 | Inter, Roboto, Arial, Open Sans, Helvetica |
| **Body** | Outfit 400 | Mismas prohibidas |
| **Mono/Label** | JetBrains Mono 600 | — |

### 3.2 Escala Tipográfica

| Token | Tamaño | Peso | Spacing | Height |
|:---|:---|:---|:---|:---|
| `--text-hero` | `clamp(3.8rem, 8vw, 7.5rem)` | 700 | `-0.05em` | 0.95 |
| `--text-display` | `clamp(2.2rem, 5vw, 4rem)` | 700 | `-0.04em` | 1.05 |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | 600 | `-0.02em` | 1.2 |
| `--text-body` | `1.05rem` | 400 | `-0.01em` | 1.65 |
| `--text-label` | `0.72rem` | 600 | `0.14em` | 1.2 (ALL CAPS) |
| `--text-code` | `0.88rem` | 400 | `normal` | 1.5 |

### 3.3 Reglas Estrictas

- `text-align: left` always. Centro SOLO en CTA Monolith
- `font-variant-numeric: tabular-nums` en datos numéricos
- Textos largos en `--muted`, NUNCA en naranja
- Labels ALL CAPS con tracking `0.14em` — estilo industrial-brutalist §3.2
- Etiquetas de datos en JetBrains Mono para estabilidad numérica
- SERIF PROHIBIDO como default. Solo si la marca lo exige explícitamente (design-taste §4.1)

---

## 4. Sistema de Grid

### 4.1 Grid Principal

```css
.swiss-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 1px;
  max-width: 1200px;
  margin: 0 auto;
}
```

### 4.2 Líneas Visibles (Elemento de diseño, no guías)

```css
.grid-line-v { width: 1px; background: var(--border); position: absolute; }
.grid-line-h { height: 1px; background: var(--subtle); }
```

### 4.3 Compartimentalización (industrial-brutalist §5)

- Bordes `1px solid var(--border)` delinean zonas de información
- Líneas horizontales `<hr>` span full container para segregar unidades
- Densidad bimodal: datos compactos monospace + macro-whitespace masivo

---

## 5. Bordes, Elevación y Profundidad

### 5.1 Sin Sombras — ABSOLUTO

- `box-shadow: none` en TODO el sistema (industrial-brutalist §4, high-end §2)
- Profundidad SOLO con:
  - Cambio de fondo (resting → hover)
  - Cambio de borde `--border` → `--border-strong`
  - Contraste secciones claras/oscuras

### 5.2 Border Radius

```
--radius-sm: 0px   /* Botones, inputs, tarjetas — RECTANGULARES */
--radius-md: 2px   /* Mínimo suavizado */
--radius-lg: 4px   /* Terminales */
--radius-xl: 6px   /* MÁXIMO del sistema — NUNCA > 6px */
```

**SHAPE CONSISTENCY LOCK**: todo-sharp (radius 0) es el default del sistema.
Nunca mezclar border-radius entre componentes (design-taste §4.4).

---

## 6. Motion & Animación

### 6.1 Decision Framework (emil-design-eng §2)

Antes de animar CUALQUIER elemento, responder en orden:

1. **¿Debe animarse?** Si el usuario lo ve 100+ veces/día → NO. Si es ocasional → sí.
2. **¿Cuál es el propósito?** Feedback, spatial consistency, state indication, o preventing jarring changes.
3. **¿Qué easing?** Entrando → `ease-out`. Moviéndose → `ease-in-out`. Hover → `ease`. Constante → `linear`.
4. **¿Cuán rápido?** UI < 300ms siempre. Buttons 100-160ms. Tooltips 125-200ms. Modals 200-500ms.

### 6.2 Custom Easing Curves (OBLIGATORIAS — nunca CSS defaults)

```css
/* Swiss mechanical — cortante y rápido */
--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);

/* Strong ease-out for UI (emil) */
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);

/* Strong ease-in-out for on-screen movement */
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);
```

**PROHIBIDO**: `ease-in` para UI (se siente sluggish — emil §3).
**PROHIBIDO**: `linear` o `ease-in-out` genéricos de CSS (high-end §2).
**PROHIBIDO**: bounce, elastic, spring suave.

### 6.3 Scroll Animations (GSAP + ScrollTrigger)

- Revelado por máscara: `clip-path: inset()` (emil §clip-path)
- Expansión desde líneas de grid: elementos que crecen desde bordes 1px
- Contadores numéricos: count-up rápido (200ms) al entrar en viewport
- Líneas SVG: `stroke-dashoffset` animado
- Stagger: 30-80ms entre items, nunca bloquear interacción (emil §stagger)

### 6.4 Microinteracciones

- Hover botones: fondo 150ms, `scale(0.97)` en `:active` (emil §buttons)
- Hover tarjetas: borde `--border` → `--border-strong` en 100ms
- Labels `[01]` cambian de `--steel` a `--amber` al hover del padre
- Nunca `scale(0)` para entrada — empezar desde `scale(0.95)` + `opacity: 0` (emil §never-scale-0)

### 6.5 Performance (OBLIGATORIO)

- Animar SOLO `transform` y `opacity` — nunca `top`, `left`, `width`, `height` (emil §performance, high-end §6)
- `backdrop-blur` SOLO en elementos fixed/sticky (nav) — nunca en scrolling containers
- `will-change: transform` solo en elementos activamente animándose
- `prefers-reduced-motion`: todas las animaciones OFF, elementos visibles inmediatamente

---

## 7. Elemento 3D — Logo VantaDB Interactivo

### 7.1 Concepto

El elemento 3D del hero es una representación tridimensional del logo VantaDB:
**torus negro (anillo exterior) + esfera naranja sólida (core central)**.

### 7.2 Estilo Visual — Swiss Wireframe (NO LISO)

El 3D DEBE sentirse Swiss-industrial, no plástico-render:

- **Torus exterior**: wireframe negro `#0a0a0a`, `MeshBasicMaterial` con `wireframe: true`
  - Grosor visual: radio de tubo fino (0.12-0.18)
  - Segmentos: suficientes para que las líneas del wireframe sean visibles (24-32)
  - El wireframe ES el diseño — líneas de 1px como los bordes del grid system
- **Esfera central**: `MeshBasicMaterial` naranja `#ff5500`, `wireframe: true`
  - Geometría: `IcosahedronGeometry` (facetas geométricas, no esfera perfecta)
  - El wireframe naranja brilla sutilmente sobre fondo claro
- **Nodos de red**: puntos flotantes `#ff5500` alrededor de la estructura
  - Tamaño: 0.03-0.05 (pequeños, tipo datos en telemetría)
  - Conexiones: `LineBasicMaterial` negro `#0a0a0a`, opacidad 0.06
- **Sin luces**: `MeshBasicMaterial` no requiere luces → render más limpio y Swiss
- **Sin sombras**: coherente con `box-shadow: none` del sistema

### 7.3 Interacción

- Rotación vinculada al mouse con interpolación suave (lerp 0.05)
- Rotación base lenta: `0.001` rad/frame en Y, `0.0005` en X
- NO rotación en saltos de 15° — suave pero con inercia mecánica
- `prefers-reduced-motion`: rotación OFF, wireframe estático visible

### 7.4 Rendimiento

- Geometría mínima (<1500 polígonos total)
- Sin post-processing (bloom, DOF, SSAO)
- Canvas `opacity: 0.4` en mobile para no saturar
- Lazy-load del chunk Three.js vía `import()` dinámico

---

## 8. Componentes del Sistema

### 8.1 Navegación (Nav)

- Fijo superior, `--surface-glass` + `backdrop-filter: blur(12px)`
- Altura: 64px, max 80px (design-taste §4.7)
- Borde inferior: `1px solid var(--border)`
- Links en `--text-label` (ALL CAPS, 0.72rem, 0.14em)
- Hover: `--steel` → `--foreground` en 100ms
- MUST render single-line en desktop (design-taste §4.7)

### 8.2 Botones

- **Primary**: fondo `--amber`, texto `#ffffff`, `border-radius: 0px`, padding `10px 24px`
  - Hover: fondo `#000000`, texto `#ffffff`
  - Active: `scale(0.97)` (emil §buttons)
- **Ghost**: fondo transparente, borde `1px solid var(--border)`, texto `--foreground`
  - Hover: fondo `var(--border)`, texto `#ffffff`
- **Link**: texto `--amber`, underline animado left→right 200ms
- Button text MUST fit 1 line (design-taste §4.5)
- No duplicate CTA intent (design-taste §4.5)

### 8.3 Tarjetas / Bloques

- Fondo `--surface`, borde `1px solid var(--border)`
- Padding `24px`
- Hover: borde → `--border-strong` en 100ms
- Index label esquina superior izquierda: `[01]` en `--text-label`
- Sin sombras. Sin `border-radius > 4px`

### 8.4 Terminal / Code Block

- Fondo `--void`, borde `1px solid var(--border)`
- Header: 3 dots en `--subtle` + título en `--text-label`
- JetBrains Mono
- Syntax: keywords `--foreground`, strings `--amber`, comments `--muted`
- Nunca `box-shadow`. Nunca `border-radius > 4px`

### 8.5 Footer

- Fondo `#0a0a0a` (OLED invertido)
- Grid 4 columnas, links `--block-dark-muted`
- Hover: links → `#ffffff`
- Bordes `rgba(255,255,255,0.08)`

---

## 9. Diseño del Index (Landing Page)

### 9.1 HERO — "Typographic Grid + 3D Logo Wireframe"

**Concepto:** Tipografía masiva asimétrica con elemento 3D wireframe del logo VantaDB.

**Layout:**
- Fondo: `--background` (warm paper `#f9f8f6`)
- Texto: columnas 1-7 (izquierda)
- 3D wireframe: columnas 8-12 (derecha), `position: absolute`, `z-index: 1`
- Título "VantaDB" en `--text-hero`, peso 700, left-aligned
- Tagline: "Embedded cognitive memory for AI agents" en `--text-title`, peso 600
- Descripción: max 20 words en `--muted`
- Labels: `[RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS]` en `--text-label`, `--foreground`
- CTAs: "pip install vantadb" (primary) + "Read Docs" (ghost)

**Interacción:**
- Labels flash naranja → foreground al cargar (GSAP stagger 80ms)
- Título: mask reveal con `clip-path: inset(0 0 100% 0)` → `inset(0)` (emil §clip-path)
- Tagline/desc/CTAs: fade secuencial con `opacity` + `translateY(8px)` (minimalist §7)
- 3D wireframe: rotación con mouse

**Lo que NO tiene (Anti-Slop):**
- Sin estadísticas de downloads/stars
- Sin partículas flotantes
- Sin gradientes difusos
- Sin animación typewriter
- Sin "Used by" logos en el hero (→ sección separada abajo per design-taste §4.7)
- Sin bounce/elastic en ninguna animación

### 9.2 COMPARATIVA — "Swiss Benchmark Grid"

**Concepto:** Bento asimétrico VantaDB vs cliente-servidor.

- Grid Bento con tamaños variados y bordes 1px
- Números gigantes Space Grotesk 700 display
- Indicadores `↓` naranja (mejor) / `↑` rojo (peor)
- Count-up 200ms al entrar viewport
- Hover: borde → `--amber`, label `[01]` iluminado
- Celdas se expanden desde líneas del grid (no fade-in)
- Exact content count = exact cell count (design-taste §4.7)

### 9.3 QUICKSTART — "Precision Terminal"

- Grid 2 columnas: izquierda = pasos `[01]-[04]`, derecha = terminal
- Paso activo: número `--amber`, borde izquierdo 2px `--amber`
- Terminal: `--void`, JetBrains Mono
- Typewriter rápido, output instantáneo con borde `--amber`
- Layout DIFERENTE a secciones anteriores (section-layout-repetition ban)

### 9.4 CORE ENGINE — "Exploded Architecture"

- Fondo invertido `#0a0a0a` (OLED)
- Grid 3 columnas con features
- Iconos monoline 1px naranja
- GSAP ScrollTrigger: pin + reveal secuencial
- Líneas SVG `stroke-dashoffset` animado

### 9.5 ARCHITECTURE — "Blueprint Cross-Section"

- SVG capas apiladas con bordes 1px
- Labels con líneas de cota y coordenadas monoespaciadas
- Scroll: exploded view (capas se separan)
- Hover: borde `--amber`, demás `opacity: 0.3`

### 9.6 USE CASES — "Case Study Cards"

- Stack vertical de tarjetas horizontales full-width
- Grid `3fr 9fr`: número display + contenido
- Hover: número `--subtle` → `--amber`
- Borde superior 1px separa tarjetas
- Layout DIFERENTE a benchmark grid y quickstart

### 9.7 CTA FINAL — "The Monolith"

- Bloque OLED full-width, texto centrado masivo (EXCEPCIÓN al left-align rule)
- `"pip install vantadb"` en `--text-hero`, `#ffffff`
- `"Zero servers. One line. Infinite context."` en `--block-dark-muted`
- Un botón primary centrado naranja
- Sin estadísticas. Sin feature list. Solo el comando.

---

## 10. Contraste Invertido por Secciones

```
[Warm Paper #f9f8f6]  → Hero
[Warm Paper]          → Comparativa
[Warm Paper]          → Quickstart
[OLED #0a0a0a]        → Core Engine
[Warm Paper]          → Architecture
[Warm Paper]          → Use Cases
[OLED #0a0a0a]        → CTA Monolith + Footer
```

**Reglas de inversión:**
- Secciones oscuras: texto `#f0f0f0`, muted `#808080`
- Bordes: `rgba(255,255,255,0.08)`
- Naranja `#ff5500` mantiene valor en AMBOS modos
- Botones ghost invierten: borde blanco, texto blanco

---

## 11. Iconografía — "Planos Técnicos"

- **Monoline**: trazo 1.5px constante
- **Sin relleno**: solo contornos
- **Color**: `--steel` resting → `--amber` hover/activo
- **Geometría**: 90° exclusivamente (industrial-brutalist §6)
- **Diagramas**: flechas ortogonales, líneas de cota, coordenadas técnicas
- **Nunca**: Lucide, FontAwesome, Material Icons genéricos (high-end §2, minimalist §2)
- **Permitidos**: Phosphor Bold, Radix UI Icons, o SVG monoline custom

---

## 12. Diseño de Subpáginas

### 12.1 Patrón Común

1. Hero compacto: título masivo asimétrico + breadcrumb label + descripción
2. Secciones alternando warm paper / OLED con grid visible
3. Diagramas SVG monoline con etiquetas técnicas
4. CTA bottom: bloque OLED con comando de terminal

### 12.2 Páginas

| Ruta | Hero | Contenido principal |
|:---|:---|:---|
| `/engine` | "The Rust Core" | HNSW, BM25, WAL, PyO3 con benchmarks |
| `/architecture` | "Architecture" | SVG interactivo de capas |
| `/pricing` | "Pricing" | Grid de planes, borde `--amber` en destacado |

---

## 13. Pre-Flight Checklist (OBLIGATORIO antes de merge)

Cada componente y cada página DEBE pasar esta verificación completa:

### 13.1 Swiss System

- [ ] `border-radius` ≤ 6px en todo
- [ ] Sin `box-shadow` en ningún elemento
- [ ] Sin gradientes decorativos
- [ ] Naranja usado SOLO para señales activas/CTAs (regla 95/5)
- [ ] `text-align: left` en bloques de contenido
- [ ] Tipografía del sistema: Space Grotesk / Outfit / JetBrains Mono
- [ ] Bordes de 1px presentes como elementos de diseño
- [ ] Grid asimétrico (nunca 3 columnas iguales genéricas)
- [ ] `font-variant-numeric: tabular-nums` en datos numéricos
- [ ] Macro-spacing ≥ 96px entre secciones

### 13.2 Anti-Slop (design-taste + impeccable)

- [ ] Sin purple/blue AI glow gradients
- [ ] Sin copys genéricos: "Elevate", "Seamless", "Unleash"
- [ ] Sin 3-card feature rows idénticas
- [ ] Max 1 eyebrow por 3 secciones
- [ ] Sin split-header (left H + right P) como default
- [ ] Max 2 zigzag secciones consecutivas
- [ ] Cada layout family aparece max 1 vez por página
- [ ] Bento cells = exact content count
- [ ] Sin ilustraciones 3D de plástico brillante
- [ ] Sin emojis en código ni markup

### 13.3 Hero Específico

- [ ] Hero cabe en viewport inicial
- [ ] H1 max 2 líneas desktop
- [ ] Subtext max 20 words
- [ ] Max 4 text elements (eyebrow + H1 + subtext + CTAs)
- [ ] "Used by" / trust logos DEBAJO del hero, nunca dentro
- [ ] Top padding max `pt-24`
- [ ] Sin feature list ni pricing teaser dentro del hero

### 13.4 Motion (emil + industrial-brutalist)

- [ ] Animaciones ≤ 300ms para UI
- [ ] Custom easing curves (nunca `linear` o `ease-in-out` genéricos)
- [ ] Nunca `ease-in` para UI
- [ ] Nunca `scale(0)` para entrada — empezar desde `scale(0.95)`
- [ ] Solo `transform` y `opacity` animados
- [ ] `backdrop-blur` solo en fixed/sticky elements
- [ ] `prefers-reduced-motion` respetado
- [ ] Stagger 30-80ms, nunca bloquea interacción
- [ ] Sin bounce, elastic, ni spring suave

### 13.5 Accesibilidad

- [ ] Contraste WCAG AA en todos los textos
- [ ] Button text contraste verificado contra background
- [ ] Form inputs con label, helper, error text
- [ ] Focus states visibles
- [ ] Touch targets ≥ 44px
- [ ] Nav single-line en desktop

---

## 14. Stack Tecnológico

| Capa | Tecnología |
|:---|:---|
| Framework | React 19 + TypeScript 5 |
| Routing | TanStack Router (file-based) |
| Bundler | Vite 8 + Rolldown |
| CSS | Vanilla CSS + CSS custom properties |
| Animaciones | GSAP + ScrollTrigger |
| 3D | Three.js (wireframe, MeshBasicMaterial) |
| Fonts | Google Fonts: Space Grotesk, Outfit, JetBrains Mono |
| Hosting | Vercel SPA |

---

## 15. Skills y Herramientas del Agente

| Skill | Uso | Fase |
|:---|:---|:---|
| `design-taste-frontend` | Brief inference, anti-slop rules, layout diversification | Pre-diseño |
| `industrial-brutalist-ui` | Grids rígidos, escala tipográfica, compartimentalización | Diseño |
| `high-end-visual-design` | Anti-patterns, variance engine, motion choreography | Diseño |
| `minimalist-ui` | Composición editorial, warm monochrome, bento flat | Diseño |
| `emil-design-eng` | Animation decision framework, clip-path, performance | Motion |
| `emilkowalski-motion` | Motion restraint, microinteracciones | Motion |
| `impeccable-design-polish` | Audit → Critique → Polish → Animate → Harden → Live | Post-diseño |
| `plan-design-review` | Gate de calidad 0-10 por dimensión | Pre-merge |
| `threejs` | Wireframe 3D del logo | Implementación |
| `color-expert` | Verificación OKLCH, contraste WCAG | QA |
