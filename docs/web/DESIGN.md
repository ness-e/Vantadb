# DESIGN.md — VantaDB Website Redesign

> **Single source of truth** for all visual, motion, and layout decisions.
> Version: 2.0 | 2026-07-04
> Style: **Swiss + Neubrutalism — "orden matemático + carácter agresivo"**

---

## Table of Contents

1. [Design Philosophy & Principles](#1-design-philosophy--principles)
2. [Color System](#2-color-system)
3. [Typography System](#3-typography-system)
4. [Layout System](#4-layout-system)
5. [Component Design Patterns](#5-component-design-patterns)
6. [Motion Principles](#6-motion-principles)
7. [Section-by-Section Specifications](#7-section-by-section-specifications)
8. [Anti-Slop Rules & Pre-Flight Checklist](#8-anti-slop-rules--pre-flight-checklist)
9. [Accessibility Requirements](#9-accessibility-requirements)
10. [Brand Platform](#10-brand-platform)
11. [Verbal Identity](#11-verbal-identity)
12. [Logo System](#12-logo-system)

---

## 1. Design Philosophy & Principles

### 1.1 The Manifesto

**"Si el diseño corporativo es un sedán familiar, VantaDB es un coche de rally Grupo B."**

Swiss + Neubrutalism fusiona la disciplina matemática de la escuela tipográfica suiza (Müller-Brockmann, Hofmann) con la agresividad visual del neubrutalism web — bordes duros, sombras de offset sin blur, tipografía masiva, y una estética que rechaza activamente el diseño SaaS genérico.

| Atributo | Swiss (orden) | Neubrutalism (carácter) |
|---|---|---|
| Grid | 12-columnas rígido, líneas visibles | Ruptura intencional del grid, celdas bento asimétricas |
| Bordes | 1px funcionales | 2px gruesos visibles como elemento de diseño |
| Sombras | Ninguna (diseño clásico) | Hard offset `8px 8px 0px 0px #000` — NUNCA difusas |
| Botones | Planos, funcionales | Mecánicos — hover reduce sombra, translate(3px,3px) |
| Tipografía | Space Grotesk 700 display | ALL CAPS monoespaciado como elemento agresivo |
| Animaciones | ≤ 300ms, cubic-bezier suave | 50-150ms, snap-fast, sin bounce/elastic |

### 1.2 Color Psychology

**Black (#111111) — Command:**
El negro suizo no es un negro genérico. Es un negro **matemático** — ligeramente levantado de `#000000` para sugerir tinta sobre papel, manteniendo autoridad absoluta. Comunica "esto funciona, no necesito distraerte". Las palabras blancas sobre `#111111` crean el ratio de contraste más alto posible (19:1+). El negro profundo canaliza atención como un visor. Anti-clickbait: no hay fondos brillantes compitiendo.

**White (#ffffff) — Precision:**
El blanco no es "limpio" — es **preciso**. Es el blanco de un blueprint, de especificaciones técnicas, de ingeniería. Texto blanco sobre negro para legibilidad máxima. Sin grises corporativos — blanco puro comunica "no ocultamos nada".

**Amber (#ff5500) — Activation:**
El único color saturado (95/5 rule). No es "amigable" como el azul corporativo — es un color de **alerta y energía**. El naranja-rojizo estimula el sistema nervioso simpático: incrementa atención y urgencia. Al tener un solo color de acento, cada elemento ámbar significa **acción**: CTA, link, alerta, dato importante. La consistencia del acento único construye reconocimiento de marca en <1 segundo. Asociación técnica: naranja en terminales/AMBER en logs = sistemas técnicos.

**Muted #808080 / Steel #5a5a5a** — jerarquía secundaria sin competir.

### 1.3 Design Principles

| # | Principle | Manifestation |
|---|---|---|
| 1 | **Mecánico sobre orgánico** | Botones presionan (no flotan). Sombras son sólidas (no difusas). Bordes son cortantes (no redondeados). |
| 2 | **Grid como ley, no como sugerencia** | 12 columnas, líneas visibles, gap 0px. El grid ES el diseño. |
| 3 | **Telemetría visible** | Prefijos `>`, corchetes `[ ]`, labels monospace ALL CAPS. La interfaz parece un panel de control. |
| 4 | **Anti-corporativo deliberado** | Sin rounded corners. Sin pasteles. Sin glassmorphism. Sin gradientes suaves. |
| 5 | **Ritmo agresivo** | 50-150ms animaciones. Stagger mínimo (30-60ms). Cada milisegundo cuenta. |
| 6 | **Transparencia estructural** | Las líneas del grid son visibles. Las celdas tienen bordes. No se oculta la estructura. |

---

## 2. Color System

### 2.1 Brand Palette

| Token | Hex | OKLCH | Usage | Contrast (on bg) |
|---|---|---|---|---|
| `--amber` | `#ff5500` | `oklch(0.62 0.22 40)` | CTAs, hover states, data highlights, active indicators | 4.5:1 on #111111 (AA) |
| `--black` | `#000000` | `oklch(0 0 0)` | Extreme contrast elements | — |
| `--white` | `#ffffff` | `oklch(1 0 0)` | Text on dark, backgrounds | 15:1 on #111111 |
| `--background` | `#111111` | `oklch(0.13 0.005 265)` | Primary background | — |
| `--foreground` | `#f5f5f5` | `oklch(0.95 0 0)` | Primary text | 15:1 on #111111 |
| `--muted` | `#808080` | `oklch(0.55 0 0)` | Secondary text, metadata | 4.6:1 on #111111 |
| `--steel` | `#5a5a5a` | `oklch(0.42 0.01 265)` | Labels, metadata, inactive | 3.5:1 on #111111 |

### 2.2 Surface Tokens

| Token | Value | Usage |
|---|---|---|
| `--surface` | `#1a1a1a` | Card/section surfaces |
| `--surface-alt` | `#222222` | Hover state for surfaces |
| `--surface-glass` | `rgba(17,17,17,0.85)` | Nav background with blur |
| `--terminal-bg` | `#0d0d0d` | Terminal/code block background |
| `--surface-card-dark` | `rgba(255,255,255,0.03)` | Card resting on dark |
| `--surface-card-dark-hover` | `rgba(255,255,255,0.06)` | Card hover on dark |

### 2.3 Border Tokens

| Token | Value | Usage |
|---|---|---|
| `--border` | `rgba(255,255,255,0.08)` | Subtle 1px borders |
| `--border-hover` | `rgba(255,255,255,0.2)` | Hover state borders |
| `--border-strong` | `#333333` | Strong borders |
| `--border-visible` | `rgba(255,255,255,0.15)` | Visible 2px neubrutalist borders |

### 2.4 Shadow Tokens (Neubrutalism — Hard Offset)

| Token | Value | Usage |
|---|---|---|
| `--shadow-sm` | `4px 4px 0px 0px #000000` | Cards, subtle elevation |
| `--shadow-md` | `6px 6px 0px 0px #000000` | Default elevation |
| `--shadow-lg` | `8px 8px 0px 0px #000000` | Modals, emphasis |
| `--shadow-amber` | `4px 4px 0px 0px var(--amber)` | Amber-tinted hard shadow |
| `--shadow-brutal` | `8px 8px 0px 0px #111111` | Neubrutalism signature |
| `--shadow-brutal-hover` | `2px 2px 0px 0px #111111` | Button hover (shadow reduces) |

**Rule:** ALL shadows are hard offset (`Xpx Ypx 0px 0px color`). Zero blur, zero spread. `box-shadow: none` is prohibited — hard shadows define the Neubrutalism style.

### 2.5 Noise & Texture

| Token | Value | Usage |
|---|---|---|
| `.scanline` | `repeating-linear-gradient` 2px/4px | CRT scanline overlay |
| `.noise-overlay` | SVG fractal noise, opacity 0.035 | Subtle grain texture |
| `.nb-bg-dot` | radial-gradient 1px dots on 24px grid | Dot grid background |
| `.nb-bg-cross` | cross-grid lines on 24px grid | Engineering blueprints |
| `.nb-bg-cross--faint` | Cross-grid 48px at 4% opacity | Subtle background texture |

### 2.6 Usage Rules

1. **Amber is the single accent color.** No secondary accent.
2. **Amber used exclusively for**: CTAs, hover/focus states, active navigation, data highlights, code syntax strings.
3. **95/5 Rule**: 95% of the page is black, white, gray. 5% is amber.
4. **Dark-dominant rhythm**: `#111111` background throughout, with `--surface` (#1a1a1a) for cards.

### 2.7 Contrast Verification

| Combination | Ratio | WCAG |
|---|---|---|
| Amber `#ff5500` on `#111111` | 4.5:1 | AA (large text) |
| White `#ffffff` on `#111111` | 15:1 | AAA |
| Muted `#808080` on `#111111` | 4.6:1 | AA |
| Foreground `#f5f5f5` on `#111111` | 15:1 | AAA |

---

## 3. Typography System

### 3.1 Font Families

| Role | Font | Fallback | Weight Used | Prohibited Alternatives |
|---|---|---|---|---|
| **Display** | Space Grotesk | sans-serif | 700 (bold) only | Inter, Roboto, Arial, Open Sans, Helvetica |
| **Body** | Outfit | sans-serif | 400 (regular), 600 (semibold) | Inter, Roboto, Arial, Open Sans, Helvetica |
| **Code/Label** | JetBrains Mono | monospace | 400 (regular), 600 (bold) | Fira Code, Cascadia Code, Menlo |

### 3.2 Type Scale

| Token | Size | Weight | Tracking | Line-height | Usage |
|---|---|---|---|---|---|
| `--text-hero` | `clamp(3.5rem, 7vw, 6.5rem)` | 700 SG | `-0.05em` | `0.95` | Primary hero headline |
| `--text-display` | `clamp(2.2rem, 4vw, 3.5rem)` | 700 SG | `-0.04em` | `1.05` | Section titles |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | 600 Outfit | `-0.02em` | `1.2` | Card titles, feature names |
| `--text-body` | `1.05rem` | 400 Outfit | `normal` | `1.65` | Running text, descriptions |
| `--text-label` | `0.72rem` | 600 JBM | `0.14em` | `1.2` | Labels, section headers, ALL CAPS |
| `--text-code` | `0.88rem` | 400 JBM | `normal` | `1.5` | Code snippets, terminal output |
| `--text-metric` | `clamp(2.5rem, 5vw, 4rem)` | 700 SG | `-0.03em` | `1` | Benchmark numbers |
| `--text-micro` | `0.6rem` | 600 JBM | `0.14em` | `1` | Timestamps, legal |

### 3.3 Typography Rules

1. **Left-aligned always** — exceptions only for isolated CTA blocks (Monolith section).
2. `font-variant-numeric: tabular-nums` on ALL numeric data.
3. Labels are ALL CAPS with tracking `0.14em` in JetBrains Mono 600 — never in body fonts.
4. Code is always JetBrains Mono, never body fonts.
5. No serif fonts anywhere. Space Grotesk and Outfit are the only proportional families.

---

## 4. Layout System

### 4.1 Grid

```css
.nb-grid {
  display: grid;
  gap: 1px;
  background: var(--border-visible);
}
```

- **12-column mental model**, implemented via `nb-grid--cols-*` variants
- **Gap: 1px** — the gap IS the grid line (visible using background color)
- **Max-width**: 1200px (`--grid-max`)
- **Grid lines are visible** — the `gap: 1px` with `background: var(--border-visible)` means every cell is separated by a visible line

### 4.2 Layout Variants

| Class | Columns | Usage |
|---|---|---|
| `.nb-grid--cols-2` | 2 | Split layouts |
| `.nb-grid--cols-3` | 3 | Feature grids |
| `.nb-grid--cols-4` | 4 | Ecosystem grids |
| `.nb-grid--cols-6` | 6 | Data-dense grids |
| `.nb-bento` | Varied | Asymmetric bento layouts |
| `.nb-asymmetric` | 8fr 4fr | Hero/section headers |
| `.nb-split-7-5` | 7fr 5fr | Content + visual split |
| `.nb-split-5-7` | 5fr 7fr | Visual + content split |

### 4.3 Spacing

| Token | Value | Usage |
|---|---|---|
| `--space-3xs` | `0.25rem` (4px) | Icon gaps, dot spacing |
| `--space-2xs` | `0.5rem` (8px) | Inline padding, micro-spacing |
| `--space-xs` | `0.75rem` (12px) | Button padding, tag padding |
| `--space-sm` | `1rem` (16px) | Card padding (tight) |
| `--space-md` | `1.5rem` (24px) | Card padding (default) |
| `--space-lg` | `2rem` (32px) | Section internal spacing |
| `--space-xl` | `3rem` (48px) | Section spacing |
| `--space-2xl` | `4rem` (64px) | Hero padding |
| `--space-3xl` | `6rem` (96px) | Major section separation |
| `--space-4xl` | `8rem` (128px) | CTA section padding |

### 4.4 Asymmetry Rules

1. Every grid must have varying cell sizes. No identical card grids.
2. One anchor cell per bento grid (`.nb-bento-cell--featured` — spans 2x2).
3. Section titles use `.nb-asymmetric` (8fr 4fr) leaving intentional space.
4. Labels use `.nb-label` with brackets: `[SECTION]`.

---

## 5. Component Design Patterns

### 5.1 Navigation

| Property | Value |
|---|---|
| Position | Fixed top |
| Height | 64px |
| Background | `--surface-glass` + `backdrop-filter: blur(12px)` |
| Bottom border | `2px solid var(--border-visible)` |
| Layout | Logo left \| Links center \| CTA right |
| Link font | `--text-label` (JetBrains Mono 600, 0.72rem, ALL CAPS, tracking 0.14em) |
| Link color resting | `--steel` |
| Link color hover | `--foreground`, 80ms |
| Link color active | `--amber` |
| CTA | Primary amber button with hard shadow |
| Mobile | Hamburger → panel with border, links vertical |

### 5.2 Buttons (Neubrutalist Mechanical)

**Primary Button (`.btn-primary`):**
| State | Style |
|---|---|
| Resting | `background: var(--amber)`, `color: var(--text-on-amber)`, `box-shadow: var(--shadow-md)`, `border: 2px solid #cc4400` |
| Hover | `background: var(--amber-light)`, `box-shadow: var(--shadow-sm)`, `transform: translate(3px, 3px)` |
| Active | `box-shadow: none`, `transform: translate(6px, 6px)` |
| Padding | `12px 28px`, text single-line |

**Ghost Button (`.btn-ghost`):**
| State | Style |
|---|---|
| Resting | `background: transparent`, `border: 2px solid var(--border-visible)`, `box-shadow: var(--shadow-sm)` |
| Hover | `background: var(--foreground)`, `color: var(--background)`, `border-color: var(--foreground)`, `transform: translate(3px, 3px)` |
| Active | `box-shadow: none`, `transform: translate(6px, 6px)` |

**All buttons:**
- `border-radius: 0` — enforced globally
- Hard shadow offset — never blurred
- Transition: `80ms var(--ease-brutal)` (snap-fast)
- Mechanical press: shadow reduces on hover, disappears on active
- Button translates (3px, 3px) on hover — simulates physical press

### 5.3 Cards (`.nb-card`)

| Property | Value |
|---|---|
| Background | `--surface` (#1a1a1a) |
| Border | `2px solid var(--border-visible)` |
| Border radius | `0` |
| Padding | `var(--space-lg)` (32px) |
| Shadow | `box-shadow: var(--shadow-sm)` (on hover: `var(--shadow-amber)`) |
| Hover | Border → `--amber`, shadow → `--shadow-amber` |
| Active | `transform: translate(2px, 2px)` |
| Transition | `border-color 80ms var(--ease-brutal)` |

### 5.4 Terminal / Code Block

| Property | Value |
|---|---|
| Background | `--terminal-bg` (#0d0d0d) |
| Border | `2px solid var(--border-visible)` |
| Border radius | `0` |
| Font | JetBrains Mono, `0.88rem` |
| Syntax | Keywords `#ffffff`, strings `--amber`, comments `--muted` |
| Output indicator | `border-left: 2px solid var(--amber)` |
| Header | Three dots (unfilled circles) in muted gray + title label |

### 5.5 Footer

| Property | Value |
|---|---|
| Background | `#111111` |
| Grid | 5 columns |
| Link color resting | `#808080` |
| Link color hover | `#ffffff` |
| Column titles | `--text-label` ALL CAPS |
| Dividers | `2px solid var(--border-visible)` |
| Bottom bar | Logo mark + copyright + GitHub link |

### 5.6 Telemetry Elements

| Element | Style |
|---|---|
| Label brackets | `.nb-bracket` — `[ ` prefijo, ` ]` sufijo en monospace |
| Telemetry row | `.nb-telemetry` — prefijo `>` en amber |
| Index labels | `.nb-index` — monospace, micro size, steel → amber on hover |
| Arrows | `.nb-arrow` — `>>>` suffix, gap expands on hover |
| Pills | `.nb-pill-status` — monospace, border, 4px dot indicator |

### 5.7 Frames

| Element | Style |
|---|---|
| `.nb-frame` | `2px` border, label `[ label ]` floating above top-left |
| `.nb-block-warning` | `3px solid #ff5500`, `⚠ WARNING` floating label |
| `.nb-icon-box` | `2.5rem` square, `2px` border, centered amber icon |

### 5.8 Texture Backgrounds

| Class | Description |
|---|---|
| `.nb-bg-dot` | Dot grid (24px spacing) |
| `.nb-bg-cross` | Cross grid (24px spacing) |
| `.nb-bg-cross--faint` | Subtle cross grid (48px, 4%) |
| `.scanline` | CRT scanline overlay |
| `.noise-overlay` | Fractal noise grain |

---

## 6. Motion Principles

### 6.1 Timing Philosophy

**"50-150ms. Snap-fast. No bounce, no elastic, no spring."**

En Neubrutalism, las animaciones no son "suaves" — son **mecánicas**. Cada transición debe sentirse como un interruptor físico, no como un fade acuoso.

### 6.2 Timing Scale

| Duración | Nombre | Uso |
|---|---|---|
| **50ms** | Instant | Button active, ticker toggle, estado binario |
| **80ms** | Snap-fast | Button hover, card border, icon color, nav hover — **MÁS COMÚN** |
| **100ms** | Fast | Section reveals, label transitions |
| **150ms** | Normal | Hover estados complejos (background + border + shadow) |
| **200ms** | Slow | Count-up numbers, panel slide-in |
| **300ms** | Reveal | Section entry animations (GSAP) |

### 6.3 Easing Curves (OBLIGATORIAS)

```css
--ease-brutal: cubic-bezier(0.05, 0.95, 0.3, 1);   /* PRIMARY — snap-fast */
--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);         /* Mechanical, sharp */
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);           /* Strong ease-out for reveals */
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);       /* On-screen movement */
```

**PROHIBIDO:**
- ❌ `ease-in`, `ease-out`, `ease-in-out` CSS defaults
- ❌ `linear` para cualquier propiedad que no sea `color`
- ❌ bounce, elastic, spring, soft cubic (nunca `cubic-bezier` con overshoot > 1)
- ❌ `transition: all` — siempre propiedades específicas

### 6.4 Micro-interactions

| Elemento | Disparador | Duración | Easing | Propiedad |
|---|---|---|---|---|
| Button primary | Hover | 80ms | `--ease-brutal` | `box-shadow`, `transform: translate(3px,3px)` |
| Button primary | Active | 50ms | `--ease-brutal` | `box-shadow: none`, `translate(6px,6px)` |
| Button ghost | Hover | 80ms | `linear` | `background`, `color`, `border-color`, `box-shadow`, `transform` |
| Card | Hover | 80ms | `--ease-brutal` | `border-color`, `box-shadow` |
| Card | Active | 50ms | `--ease-brutal` | `transform: translate(2px,2px)` |
| Nav link | Hover | 80ms | `linear` | `color` |
| Index label | Parent hover | 80ms | `--ease-brutal` | `color: steel → amber` |
| Arrow link | Hover | 150ms | `--ease-brutal` | `gap: 0.25rem → 0.5rem` |
| Ecosystem cell | Hover | 80ms | `--ease-brutal` | `background`, `border-color` |

### 6.5 Animated Components

**Ticker (`.nb-ticker`):**
```css
@keyframes nb-ticker {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
.nb-ticker { animation: nb-ticker 0.8s steps(1) infinite; }
```
Usado para: indicadores de carga, estado "live", métricas en tiempo real, estado de grabación.

**Cursor (`.nb-cursor`):**
```css
@keyframes nb-cursor-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}
.nb-cursor { animation: nb-cursor-blink 1s step-end infinite; }
```
Usado para: cursor al final de comandos en Monolith, terminal prompts.

**Split Flip (`.nb-split`):**
```css
@keyframes nb-split-flip {
  0% { transform: translateY(0); }
  50% { transform: translateY(-50%); }
  100% { transform: translateY(-100%); }
}
.nb-split-inner { animation: nb-split-flip 2s steps(10) infinite; }
```
Usado para: display de números tipo slot machine (métrica rotante).

### 6.6 GSAP ScrollTrigger

| Animación | Técnica | Duración | Stagger |
|---|---|---|---|
| Hero title | `clip-path: inset(0 0 100% 0)` → `inset(0)` | 300ms | — |
| Section reveals | Fade + translateY(8px) | 200ms | 60ms |
| Benchmark count-up | `gsap.utils.interpolate` | 200ms | — |
| Feature cards | `clip-path: inset(100% 0 0 0)` → `inset(0)` | 200ms | 80ms |
| Ecosystem grid | Expand from grid lines | 150ms | 50ms |
| Architecture layers | TranslateY separation | 200ms | — |
| CTA (Monolith) | Clip-path reveal | 300ms | — |

**Reglas GSAP:**
- Solo animar `transform` y `opacity` — nunca `top`, `left`, `width`, `height`
- Stagger nunca > 80ms
- No pin sections (excepto Core Engine si es necesario)
- `toggleActions: "play none none reverse"`
- `prefers-reduced-motion`: elementos visibles inmediatamente

### 6.7 Quickstart Terminal

| Propiedad | Valor |
|---|---|
| Typewriter | `30ms/char` |
| Output aparece | instantáneo (no fade) |
| Auto-play | secuencial, restart después de 3s |
| Click en paso | salta inmediatamente |

### 6.8 No-bounce Policy

- **CERO** `cubic-bezier` con valores > 1.0 (no overshoot)
- **CERO** `transform: scale()` superior a 1.0
- **CERO** `keyframes` que retornen con bounce
- **CERO** animaciones tipo "swing" o "pendulum"
- **CERO** spring(), elastic() en GSAP

### 6.9 Performance (Mandatory)

- Animar SOLO `transform` y `opacity`
- `backdrop-blur` SOLO en nav (fixed/sticky)
- `will-change: transform` solo en elementos animándose activamente
- Sin post-processing. Sin Three.js. Sin canvas animado.
- `prefers-reduced-motion`: todas las animaciones desactivadas, elementos visibles inmediatamente

---

## 7. Section-by-Section Specifications

### 7.1 NAV

```tsx
<nav className="navbar">
  <SwissLogo />
  <div className="nav-links">
    <Link to="/docs">[DOCS]</Link>
    <Link to="/engine">[ENGINE]</Link>
    <Link to="/architecture">[ARCH]</Link>
    <Link to="/pricing">[PRICING]</Link>
  </div>
  <button className="btn-primary">Get Started</button>
</nav>
```

### 7.2 HERO

```tsx
<section className="nb-section">
  <div className="nb-inner nb-asymmetric">
    <div>
      <span className="nb-label nb-label--amber">[RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS]</span>
      <h1 className="text-hero">VantaDB</h1>
      <p>The database that thinks with you.</p>
      <div className="hero-ctas">
        <button className="btn-primary">pip install vantadb</button>
        <button className="btn-ghost">Read the Docs</button>
      </div>
    </div>
    {/* Right column intentionally sparse — Swiss asymmetry */}
  </div>
</section>
```

### 7.3 METRICS STRIP

```tsx
<div className="nb-grid nb-grid--cols-4">
  <div className="nb-cell"><span className="text-metric">1.2ms</span><span className="nb-label">p50 Query</span></div>
  <div className="nb-cell"><span className="text-metric">2MB</span><span className="nb-label">Binary</span></div>
  <div className="nb-cell"><span className="text-metric">0</span><span className="nb-label">Servers</span></div>
  <div className="nb-cell"><span className="text-metric">99.8%</span><span className="nb-label">Recall@10</span></div>
</div>
```

### 7.4 FEATURES

```tsx
<section className="nb-section nb-section--dark">
  <div className="nb-inner nb-section-header">
    <span className="nb-label nb-label--amber">[WHY VANTA]</span>
    <h2 className="text-display">Everything you need, nothing you don't.</h2>
  </div>
  <div className="nb-bento nb-bento--3col">
    <div className="nb-bento-cell nb-bento-cell--featured">
      <span className="nb-index">[01]</span>
      <div className="nb-icon-box">🔍</div>
      <h3>Hybrid Search</h3>
      <p>HNSW + BM25 in a single query with RRF fusion.</p>
    </div>
    <div className="nb-bento-cell"><span className="nb-index">[02]</span>...</div>
    <div className="nb-bento-cell"><span className="nb-index">[03]</span>...</div>
    <div className="nb-bento-cell"><span className="nb-index">[04]</span>...</div>
    <div className="nb-bento-cell nb-bento-cell--span2"><span className="nb-index">[05]</span>...</div>
  </div>
</section>
```

### 7.5 QUICKSTART

```tsx
<section className="nb-section">
  <div className="nb-inner nb-section-header">
    <span className="nb-label nb-label--amber">[QUICKSTART]</span>
    <h2>From zero to query in 60 seconds.</h2>
  </div>
  <div className="nb-split-5-7">
    <div className="quickstart-steps">
      <div className="step step--active"><span className="nb-index nb-index--amber">[01]</span> Install</div>
      <div className="step"><span className="nb-index">[02]</span> Connect</div>
      <div className="step"><span className="nb-index">[03]</span> Store</div>
      <div className="step"><span className="nb-index">[04]</span> Query</div>
    </div>
    <div className="terminal">
      <div className="terminal-header">⬤ ⬤ ⬤ terminal</div>
      <pre className="terminal-body">$ pip install vantadb</pre>
    </div>
  </div>
</section>
```

### 7.6 FINAL CTA (The Monolith)

```tsx
<section className="nb-section nb-section--lg" style={{ textAlign: 'center' }}>
  <div className="nb-inner">
    <span className="text-hero">pip install vantadb-py</span>
    <span className="nb-cursor" />
    <p>Zero servers. One line. Infinite context.</p>
    <button className="btn-primary">Get Started</button>
    <a className="nb-arrow">Read Documentation</a>
  </div>
</section>
```

---

## 8. Anti-Slop Rules & Pre-Flight Checklist

### 8.1 Anti-Slop Rules

| # | Rule | Enforcement |
|---|---|---|
| 1 | **No rounded corners** — border-radius: 0px on everything | Visual inspection |
| 2 | **No soft shadows** — hard offset only | CSS audit |
| 3 | **No gradients** — zero gradient backgrounds allowed | CSS audit |
| 4 | **No glassmorphism** — backdrop-blur only on nav | Component audit |
| 5 | **No identical card grids** — every grid has varying cell sizes | Visual inspection |
| 6 | **No bounce/elastic animations** — snap-fast only | Code review |
| 7 | **One accent color (amber)** — used sparingly on CTAs and data highlights | Color audit |
| 8 | **No generic AI copy** — never "Elevate", "Seamless", "Unleash" | Copy review |
| 9 | **No Inter, Roboto, Arial, Open Sans, Helvetica** — Space Grotesk/Outfit/JBM only | CSS audit |
| 10 | **No decorative illustrations or photography** — technical, functional visuals only | Visual inspection |
| 11 | **No duplicate CTA intent per page** — each page has one primary action | UX review |
| 12 | **Nav single-line on desktop** — must never wrap | Responsive check |

### 8.2 Neubrutalist Pre-Flight Checklist

**Typography:**
- [ ] All display text uses Space Grotesk 700
- [ ] All body text uses Outfit 400 or 600
- [ ] All code/labels use JetBrains Mono
- [ ] `font-variant-numeric: tabular-nums` on numeric data
- [ ] Labels are ALL CAPS with 0.14em tracking
- [ ] Text left-aligned (except Monolith CTA)

**Color:**
- [ ] Amber (#ff5500) is the only accent color
- [ ] No purple, blue, teal, green decorative elements
- [ ] Contrast: body text ≥ 4.5:1, large text ≥ 3:1
- [ ] No gradients anywhere

**Neubrutalism (hard edges):**
- [ ] `border-radius: 0` on every element
- [ ] Hard offset shadows only (`Xpx Ypx 0px 0px color`) — no blur
- [ ] Button hover: shadow reduces + translate(3px,3px)
- [ ] Button active: shadow = none + translate(6px,6px)
- [ ] Borders are 2px on cards, frames, inputs
- [ ] 1px gap between grid cells (visible hairline grid)
- [ ] Noise texture or dot grid background present

**Layout:**
- [ ] 12-column grid system respected
- [ ] Max-width 1200px on content sections
- [ ] No identical cell sizes in any grid
- [ ] One anchor cell per bento grid
- [ ] Hero fits viewport without scroll
- [ ] H1 max 2 lines, subtext max 20 words

**Motion:**
- [ ] All easing uses `--ease-brutal` (primary) or `--ease-swiss`
- [ ] Animations ≤ 150ms for UI elements
- [ ] Only `transform` and `opacity` animated
- [ ] `prefers-reduced-motion` respected
- [ ] No bounce, elastic, spring

**Components:**
- [ ] Telemetry elements present: `>` prefixed rows, bracket labels
- [ ] Button text fits 1 line
- [ ] Touch targets ≥ 44x44px on mobile
- [ ] Nav single-line on desktop

---

## 9. Accessibility Requirements

### 9.1 WCAG Compliance Targets

| Criterion | Target | Notes |
|---|---|---|
| Contrast ratio (body text) | ≥ 4.5:1 (AA) | Verified for all text elements |
| Contrast ratio (large text) | ≥ 3:1 (AA) | ≥ 18px bold or ≥ 24px regular |
| Keyboard navigation | Full | All interactive elements reachable |
| Focus indicators | Visible | `outline: 2.5px solid var(--amber)` |
| Touch targets | ≥ 44x44px | All buttons, links, interactive cards |

### 9.2 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

### 9.3 Focus Indicators

- All interactive elements: `outline: 2.5px solid var(--amber)`, `outline-offset: 2px`
- Never `outline: none` without replacement
- Visible on both dark and amber backgrounds

---

## 10. Brand Platform

### 10.1 Purpose (Why we exist)
Make vector-native data infrastructure invisible. Every AI agent, every RAG pipeline, every intelligent application deserves a database that embeds as easily as SQLite but understands vectors, text, and SQL — without requiring a dedicated infrastructure team.

### 10.2 Vision & Mission
**Vision:** A future where any AI application — from a weekend prototype to an enterprise agent mesh — runs on self-contained, zero-ops data infrastructure. **Mission:** Build the fastest, most embeddable converged database engine — unifying SQL, vector search, and full-text search in a single Rust binary.

### 10.3 Values

| Value | Manifestation |
|---|---|
| **Radical Simplicity** | One binary, one `pip install`, zero servers. Complexity is the enemy. |
| **Performance Without Compromise** | Sub-millisecond queries at 0.998 Recall@10. Every microsecond matters. |
| **Developer Empathy First** | SDKs, docs, and APIs built by developers for developers. |
| **Open by Default** | Open core, open benchmarks, open roadmap. We show receipts. |
| **AI-Native by Design** | Every architectural decision starts with "how does this serve an AI agent?" |

### 10.4 Brand Territory

```
  Industrial precision ◄━━━━━━━━━━━━━━━━━► Developer warmth
  Enterprise rigor    ◄━━━━━━━━━━━━━━━━━► Indie hacker energy
```

VantaDB lives at the intersection of **industrial precision** and **developer warmth** — reliable enough for production, approachable enough for a hackathon.

### 10.5 Archetypes

| Primary | Secondary |
|---|---|
| **The Magician** — "Your data stack disappears." VantaDB makes infrastructure invisible. | **The Creator** — "Build what you imagine." Unconstrained by ops. |

### 10.6 Decision Hierarchy

```
Business > Brand > Marketing > Design
```
A decision blocked at any lower layer must be re-escalated. Design cannot override marketing. Marketing cannot override brand.

### 10.7 Tagline System

| Context | Tagline |
|---|---|
| **Hero / Above fold** | *The database that thinks with you.* |
| **Technical / SDK docs** | *One binary. Three query engines. Zero ops.* |
| **Community / Open source** | *SQLite-for-AI-Agents, MIT open core.* |
| **Enterprise / Compliance** | *Vector-native. Air-gapped. Auditable.* |
| **GTM: AI Agents** | *Memory, state, and search — embedded.* |
| **GTM: Local RAG** | *Your data never leaves your laptop.* |
| **GTM: IDE Tooling** | *Your codebase, searchable by meaning.* |

---

## 11. Verbal Identity

### 11.1 Voice Dimensions

| Dimension | Position | Meaning |
|---|---|---|
| **Precision vs. Warmth** | 60/40 | Technically accurate but never cold. |
| **Confidence vs. Humility** | 70/30 | We own our benchmarks but credit the community. |
| **Visionary vs. Practical** | 40/60 | Point toward the future, show how to install today. |
| **Direct vs. Decorative** | 80/20 | Say what it does. No fluff, no jargon traps. |
| **Serious vs. Playful** | 75/25 | We take performance seriously, not ourselves. |
| **Expert vs. Peer** | 50/50 | Lead with authority, sit next to developers. |

### 11.2 Tone Matrix

| Context | Precision/Warmth | Confidence/Humility | Style |
|---|---|---|---|
| **Docs & SDK** | 80/20 | 60/40 | Clear, complete, neutral. Let the code speak. |
| **Landing / Hero** | 40/60 | 80/20 | Bold claim + immediate proof. "One install. Zero servers." |
| **Social / Community** | 30/70 | 50/50 | Short, human, generous. |
| **Enterprise / Sales** | 70/30 | 90/10 | Compliance, SLAs, audit trails. |
| **Error messages** | 60/40 | 60/40 | "Connection failed: check path. Here's the fix." |
| **Changelog** | 50/50 | 80/20 | "v0.4.2: 22% faster hybrid queries. Just upgrade." |

### 11.3 Writing Principles

1. **Show the number** — "p50 latency: 1.2ms" not "extremely fast"
2. **Lead with the verb** — "Query with SQL, vectors, or plain text" not passive constructions
3. **One binary, zero jargon** — never "leveraging our unified multi-model converged architecture"
4. **Infrastructure is invisible** — "`pip install vantadb-py`. You're running."
5. **Reframe the comparison** — name the old way, not competitors: "No server process to manage."

### 11.4 Editorial Glossary

| Always Use | Never Use |
|---|---|
| Embedded, Converged, Hybrid query, Zero-infrastructure, Agent memory | Revolutionize, Leverage, Game-changing, Best-in-class, Disrupt, Paradigm shift, Synergy |

### 11.5 Brand Personality
VantaDB is a **senior infrastructure engineer who also teaches at a code bootcamp** — quietly brilliant, generous with knowledge, intolerant of unnecessary complexity, excited about AI but skeptical of hype, approachable.

---

## 12. Logo System

### 12.1 Versions

| Versión | Descripción | Uso |
|---|---|---|
| **Flat SVG Mark** | Torus (anillo) + esfera ámbar — 2D monocromático | Nav, Footer, favicon |
| **Wordmark** | "VantaDB" en Space Grotesk 700 | Alternativa cuando el mark no aplica |

### 12.2 Colors

| Elemento | Dark Mode |
|---|---|
| Torus (anillo) | `#f0f0f0` |
| Core (esfera) | `#ff5500` |
| Wordmark | `#ffffff` |

### 12.3 Sizes

| Contexto | Mark mínimo | Wordmark mínimo |
|---|---|---|
| Nav | 24×24px | 16px font-size |
| Footer | 20×20px | 14px font-size |
| Favicon | 16×16px (SVG) | — |

### 12.4 Prohibiciones

- ❌ No recolorear el core naranja (siempre `#ff5500`)
- ❌ No aplicar sombras, gradientes o glow al logo
- ❌ No border-radius en contenedores del logo
- ❌ No usar en fondos con patrón o fotografía

---

## Appendix A: Related Documents

| Document | Purpose | Path |
|---|---|---|
| Token System | All CSS tokens | `design/TOKEN_SYSTEM.md` |
| Component Spec | Detailed component specs | `design/COMPONENT_SPEC.md` |
| Component Library | Component catalog | `design/COMPONENT_LIBRARY.md` |
| Icon System | Monoline SVG icon specs | `design/ICON_SYSTEM.md` |
| Site Map | Complete route inventory | `product/SITE_MAP.md` |
| Product | Product description | `product/PRODUCT.md` |
| Neubrutalist Checklist | Portable pre-flight checklist | `qa/NEUBRUTALIST_CHECKLIST.md` |
| Accessibility Statement | Compliance declaration | `qa/ACCESSIBILITY_STATEMENT.md` |
| Playwright CLI | Visual review tooling | `tools/PLAYWRIGHT_CLI.md` |

## Appendix B: Design Tokens Summary

```css
--amber: #ff5500;
--black: #000000;
--white: #ffffff;
--background: #111111;
--foreground: #f5f5f5;
--muted: #808080;
--steel: #5a5a5a;
--surface: #1a1a1a;
--surface-alt: #222222;
--surface-glass: rgba(17,17,17,0.85);
--terminal-bg: #0d0d0d;

--border: rgba(255,255,255,0.08);
--border-hover: rgba(255,255,255,0.2);
--border-strong: #333333;
--border-visible: rgba(255,255,255,0.15);

--shadow-sm: 4px 4px 0px 0px #000000;
--shadow-md: 6px 6px 0px 0px #000000;
--shadow-lg: 8px 8px 0px 0px #000000;
--shadow-amber: 4px 4px 0px 0px var(--amber);
--shadow-brutal: 8px 8px 0px 0px #111111;
--shadow-brutal-hover: 2px 2px 0px 0px #111111;

--radius-sm: 0px;
--radius-md: 0px;
--radius-lg: 0px;
--radius-xl: 0px;
--radius-pill: 0px;

--font-sans: 'Outfit', sans-serif;
--font-display: 'Space Grotesk', sans-serif;
--font-mono: 'JetBrains Mono', monospace;

--ease-brutal: cubic-bezier(0.05, 0.95, 0.3, 1);
--ease-swiss: cubic-bezier(0.25, 1, 0.5, 1);

--section-gap: 96px;
--section-gap-lg: 160px;
--grid-max: 1200px;
```
