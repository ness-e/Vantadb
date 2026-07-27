# Reglas de Diseño — Frontend Web

> Diseño: Manga / Linocut / Neo-Brutalist con Tailwind v4.
> Este documento define las reglas de CSS, utilidades, animaciones, componentes y anti-patrones del frontend web de VantaDB.
> Basado exclusivamente en el código de `web/`. No cubre brand identity ni coding conventions.

---

## 1. Arquitectura CSS

Todo el CSS global vive en `globals.css`. No hay archivos CSS separados por componente — los componentes usan Tailwind utility classes o estilos inline.

```
@import "tailwindcss";           ← v4 engine
@import "tw-animate-css";        ← shadcn animaciones base
@theme inline { ... }            ← Tokens de diseño (paleta, fuentes, semántica)
:root { ... }                    ← CSS variables: radius, shadcn colores
@layer base { ... }              ← Resets globales, border-black, selection neon
@layer utilities { ... }         ← 40+ utility classes propias
@keyframes ...                   ← 13 animaciones con prefijo vanta-
```

**Regla:** Cualquier cambio de paleta, tipografía, o animación global se hace únicamente en `globals.css`. No duplicar tokens en archivos de componentes.

---

## 2. Utility Classes

Todas definidas en `@layer utilities {}` en `globals.css`. Son específicas de VantaDB — no reemplazan Tailwind, lo extienden.

### 2.1 Press Effects (física de papel)

Simulan un objeto físico presionado. La transición de 90ms es clave para la sensación táctil.

| Class | Sombra | Hover translate | Active translate |
|-------|--------|-----------------|------------------|
| `.press` | `6px` | `2px` | `6px` |
| `.press-lg` | `8px` | `3px` | `8px` |
| `.press-neon` | `6px` | `2px` | `6px` (+ bg transition) |

```html
<button class="press bg-paper text-ink font-bold">Press me</button>
```

### 2.2 Texture Effects

Capa visual distintiva del estilo manga/linocut. Se aplican como `::before` pseudo-elementos o clases directas en contenedores.

| Class | Efecto | Uso |
|-------|--------|-----|
| `.paper-bg` | Radial-gradient dots | Fondo de secciones cream |
| `.paper-grain` | SVG feTurbulence noise | Overlay sobre fondos sólidos, mix-blend-mode multiply |
| `.halftone` | Puntos negros regulares (12px) | Efecto impresión retro |
| `.halftone-neon` | Puntos neon naranja | Versión neón |
| `.hatch` | Líneas diagonales 45° | Linocut / cincel |
| `.speed-lines` | Líneas verticales | Acción manga |
| `.scanlines` | Líneas CRT (2px spacing) | Impresión / digital |
| `.grid-tech` | Crosshatch engineering grid | Fondos técnicos |

**Regla:** No más de una textura por contenedor. Combinar texturas requiere verificación visual.

### 2.3 Shadow System

Brutalista — hard offset, sin blur. La sombra es un bloque sólido de color.

| Class | Valor |
|-------|-------|
| `.shadow-brutal` | `6px 6px 0 0 #000` |
| `.shadow-brutal-lg` | `8px 8px 0 0 #000` |
| `.shadow-brutal-neon` | `6px 6px 0 0 #FF5500, 6px 6px 0 2px #000` |
| `.shadow-brutal-sm` | `4px 4px 0 0 #000` |
| `.shadow-throw` | `6px 6px 0 0 #000, 12px 12px 0 0 rgba(0,0,0,0.15)` |

**⚠️ Código muerto:** Estas clases existen en `globals.css` pero ningún componente las importa. Todos los componentes usan valores inline como `shadow-[6px_6px_0_0_#000]`. No agregar nuevos usos a estas clases — decidir si eliminarlas o migrar los componentes a usarlas.

### 2.4 Text Treatments

| Class | Efecto |
|-------|--------|
| `.text-stencil` | Tracking tight + line-height 0.9 |
| `.text-outline` | Contorno negro 2px, fill transparente |
| `.text-outline-neon` | Contorno neon 2px |
| `.marker-neon` | Subrayado gradient highlight |

### 2.5 Manga Effects

| Class | Descripción |
|-------|-------------|
| `.manga-frame` | Clip-path octagonal (12px corner cuts) |
| `.scroll-manga` | Scrollbar thin black/cream |
| `.tape` | Washi tape vía `::after` |
| `.glitch-hover` | Text-shadow split cyan/neon en hover |
| `.neon-underline` | `::after` underline que crece 0→100% en hover |

---

## 3. Sistema de Animaciones

13 keyframes con prefijo `vanta-` en `globals.css`. Convención de naming: `vanta-<efecto>` para evitar colisiones.

### 3.1 Catálogo

| Keyframe | Duración | Propósito |
|----------|----------|-----------|
| `vanta-marquee` | 28s | Scrolling infinito horizontal |
| `vanta-blink` | 1.1s | Parpadeo cursor/indicador |
| `vanta-flicker` | 4.5s | Flicker neón inestable |
| `vanta-shake` | 180ms | Sacudida impacto/error |
| `vanta-rise` | 500ms | Slide up + fade entrada |
| `vanta-stamp` | 500ms | Sello con squash/stretch |
| `vanta-scan` | 3.2s | Barrido tipo radar/SCR |
| `vanta-pulse-ring` | 2s | Anillo expansivo |
| `vanta-glow-pulse` | 2.5s | Pulso de brillo neón |
| `vanta-glitch` | 300ms | Glitch digital |
| `vanta-kinetic-type` | 4s | Tipografía cinética |
| `vanta-gradient-border` | 3s | Borde gradient animado |
| `vanta-float` | 3s | Flotación suave |

### 3.2 Reglas

- Usar `motion-safe:animate-vanta-*` para respetar `prefers-reduced-motion`.
- No crear keyframes fuera de `globals.css`. Si un componente necesita una animación única, usar Tailwind `animate-[...]` inline.
- Preferir `vanta-` sobre `animate-` de Tailwind para animaciones propias.

---

## 4. Component Patterns

### 4.1 PageHeader

Hero consistente para páginas Tier 2/3 (features, benchmarks, etc.). Patrón:

```
Panel black background + neon shadow
├── "◆" decorative element
├── Badge (tag)
├── h1 title
├── p subtitle
└── CTA buttons
```

### 4.2 PageSection

Content wrapper con variantes de color:

| Variant | bg | text |
|---------|-----|------|
| `cream` | `bg-cream` | `text-ink` |
| `paper` | `bg-paper` | `text-ink` |
| `ink` | `bg-ink` | `text-cream` |

### 4.3 SiteShell

Raíz del layout. Orden de render:
1. `ScrollProgress` (barra de progreso superior)
2. `Navbar`
3. `main`
4. `Footer`
5. Modales superpuestos (CommandPalette, Tutorial)

### 4.4 Reveal

Scroll reveal con 6 direcciones: `up`, `down`, `left`, `right`, `scale`, `fade`.
Props: `direction`, `delay`, `className`.

### 4.5 InkDivider

Divisor decorativo con ◆ central. Separación visual entre secciones.

---

## 5. Anti-Patrones Críticos

### 5.1 🔴 Hardcoded Colors (~350 instancias en total)

El `@theme inline {}` define tokens semánticos (`cream`, `ink`, `neon`, `paper`, `surface`) pero ningún componente los usa.

| Hardcoded | Token equivalente | Instancias |
|-----------|-------------------|------------|
| `bg-[#FBF9F5]` | `bg-cream` | ~200 |
| `text-[#FF5500]` | `text-neon` | ~100 |
| `text-[#FBF9F5]` | `text-primary-foreground` | ~50 |

**Impacto:** Cambiar la paleta requiere buscar y reemplazar 200+ valores.

**Solución:** Migrar progresivamente: cada vez que se toque un componente, reemplazar hardcoded colors por tokens. Priorizar componentes en páginas públicas (landing, features, benchmarks).

### 5.2 🔴 SectionHeader Duplicado (~15 copias)

El patrón `tag badge → h2 → subtitle` está copiado manualmente en:
- features.tsx
- trust-section.tsx
- latency-comparator.tsx
- benchmarks-view.tsx (definición local en líneas 417-444)
- Y ~11 componentes más

Cada copia tiene ligeras variantes de spacing, tipografía, y animación.

**Solución:** Extraer a `PageSection.Header` como subcomponente de `PageSection`. Props: `tag`, `title`, `description`, `align` (center/left).

### 5.3 🟡 Z-Index Ad-Hoc (sin sistema de capas)

```
z-0   → paper grain background
z-1   → scanlines overlay
z-10  → copy button (code blocks)
z-20  → SFX labels
z-30  → register marks
z-50  → navbar, back-to-top
z-[60] → scroll progress bar
z-[70] → command palette
z-[75] → tutorial modal
z-[80] → easter egg
z-100 → skip link
```

**Problema:** Los valores `[60]`, `[75]`, `[80]` son frágiles. Agregar una nueva capa entre 70 y 75 requiere adivinar.

**Solución:** Definir capas en `@theme inline {}`:

| Token | Valor | Uso |
|-------|-------|-----|
| `--z-base` | `0` | Fondos, texturas |
| `--z-decorative` | `10` | Elementos decorativos |
| `--z-floating` | `30` | Tooltips, badges flotantes |
| `--z-navigation` | `50` | Navbar, header fijo |
| `--z-overlay` | `60` | Backdrops semi-transparentes |
| `--z-modal` | `70` | Modales, diálogos |
| `--z-toast` | `80` | Notificaciones |
| `--z-max` | `100` | Skip link, emergency |

---

## 6. Anti-Patrones Medios

### 6.1 🟡 Shadow Classes Muertas

5 shadow utility classes en `globals.css` que nadie usa. Todos los componentes aplican shadow via inline Tailwind `shadow-[6px_6px_0_0_#000]`.

**Decisión pendiente:** Eliminar las clases muertas o migrar componentes a usarlas. Preferir eliminación (YAGNI) si no hay plan de adopción.

### 6.2 🟡 Dark Mode Stubbed

- `next-themes` instalado y `ThemeToggle` funcional
- `enableSystem: false`
- **Cero** CSS `.dark` en cualquier archivo
- El toggle no produce ningún cambio visual

**Decisión:** No implementar dark mode. Si se requiere en el futuro, el diseño neo-brutalist con paleta manga no tiene equivalencia directa oscura — requeriría una paleta completa separada.

---

## 7. Reglas de Diseño

### 7.1 Lo que SÍ se hace

- Bordes sharp: `--radius: 0.125rem` (2px)
- Sombras hard offset sin blur
- Fondos texturados (paper, halftone, grain, hatch)
- Paleta de alto contraste: cream + ink + neon accent
- Tipografía expresiva: letter-spacing negativo, outline, stencil
- Animaciones con personalidad: flicker, glitch, shake, stamp
- Layout asimétrico, bento grids

### 7.2 Lo que NO se hace

| Técnica | Estado | Razón |
|---------|--------|-------|
| Dark mode | No implementado | Sin paleta oscura definida |
| Glassmorphism | No usado | Contradice el ethos brutalist (sin blur) |
| Border radius suave | No usado | `radius: 0.125rem` es casi sharp |
| Gradient backgrounds | Solo decorativo | No como fondo de sección |
| Box shadow con blur | Prohibido | Solo hard offset |
| React Server Components | No usado | Todo "use client" |
| Server-side data fetching | No usado | Todo static/build-time |
| External state library | No usado | zustand instalado pero sin imports |

### 7.3 Convenciones

- `"use client"` en todos los componentes (SSR deshabilitado explícitamente)
- `motion-safe` para animaciones que puedan causar molestias
- Preferir CSS classes sobre Tailwind inline para efectos propietarios (press, texture)
- No agregar dependencias de UI framework — el diseño es artesanal, no component-library
- Los componentes que definen animaciones deben incluir `prefers-reduced-motion` respect
- No mezclar `@theme` tokens con valores hardcoded en un mismo componente
