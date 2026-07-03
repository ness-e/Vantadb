# Swiss Pre-Flight Checklist

> Portable desde `design/DiseñoNuevo.md §13`
> Obligatorio antes de mergear cualquier cambio visual.

---

## §13.1 Swiss System

- [ ] `border-radius` ≤ 6px en todo el componente/página
- [ ] Sin `box-shadow` en ningún elemento
- [ ] Sin gradientes decorativos
- [ ] Naranja usado SOLO para señales activas/CTAs (regla 95/5)
- [ ] `text-align: left` en bloques de contenido (excepción: Monolith)
- [ ] Tipografía del sistema: Space Grotesk / Outfit / JetBrains Mono
- [ ] Bordes de 1px presentes como elementos de diseño
- [ ] Grid asimétrico (nunca 3 columnas iguales genéricas)
- [ ] `font-variant-numeric: tabular-nums` en datos numéricos
- [ ] Macro-spacing ≥ 96px entre secciones

## §13.2 Anti-Slop (design-taste + impeccable)

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

## §13.3 Hero Específico

- [ ] Hero cabe en viewport inicial
- [ ] H1 max 2 líneas desktop
- [ ] Subtext max 20 words
- [ ] Max 4 text elements (eyebrow + H1 + subtext + CTAs)
- [ ] "Used by" / trust logos DEBAJO del hero, nunca dentro
- [ ] Top padding max `pt-24`
- [ ] Sin feature list ni pricing teaser dentro del hero

## §13.4 Motion (emil + industrial-brutalist)

- [ ] Animaciones ≤ 300ms para UI
- [ ] Custom easing curves (nunca `linear` o `ease-in-out` genéricos)
- [ ] Nunca `ease-in` para UI
- [ ] Nunca `scale(0)` para entrada — empezar desde `scale(0.95)`
- [ ] Solo `transform` y `opacity` animados
- [ ] `backdrop-blur` solo en fixed/sticky elements
- [ ] `prefers-reduced-motion` respetado
- [ ] Stagger 30-80ms, nunca bloquea interacción
- [ ] Sin bounce, elastic, ni spring suave

## §13.5 Accesibilidad

- [ ] Contraste WCAG AA en todos los textos
- [ ] Button text contraste verificado contra background
- [ ] Form inputs con label, helper, error text
- [ ] Focus states visibles
- [ ] Touch targets ≥ 44px
- [ ] Nav single-line en desktop
