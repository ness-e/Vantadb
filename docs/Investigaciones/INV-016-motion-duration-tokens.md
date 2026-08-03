# INV-016: Motion-Duration Tokens — Auditoría

> **Estado:** ✅ COMPLETADA 2026-08-03 · **Fuente:** docs/Backlog.md INV-016 · **Tipo:** Web Frontend (tokens de animación) — auditoría + propuesta, sin implementación

## Resumen Ejecutivo

**NO existe un sistema de tokens de duración/easing.** `globals.css` `@theme inline{}` solo define colores + fuentes; no hay `--duration-*` ni `--ease-*`. El easing `cubic-bezier(0.2,0.8,0.2,1)` está hardcodeado en 4 lugares — candidato a token `--ease-default`.

## Corrección al backlog

`Reveal` **NO usa framer-motion** — es una CSS transition disparada por IntersectionObserver (`useReveal`). framer-motion solo vive en `page-transition.tsx` y `latency-comparator.tsx`. Animejs en los marks.

## Inventario de duraciones hardcodeadas

| Componente → valor | Stack | Token propuesto |
|---|---|---|
| `page-transition.tsx:26` → `0.28` | framer-motion | `normal` (.3) |
| `latency-comparator.tsx:323` → `0.4` | framer-motion | `slow` (.5) |
| `latency-comparator.tsx:367` → `0.5` | framer-motion | `slow` (.5) |
| `reveal.tsx` default → `600ms` + delays 40–240 | CSS transition (useReveal) | `transitionDuration` vía CSS var |
| `mark-classic` 400 / 2400 loop; `mark-cta` 250,300,350,400,400,500,500 | animejs | settle→`slow`/`normal`; loop queda |
| Tailwind `duration-*`: 200×13, 300×6, 500, 150, 75, 1000 | CSS utility | `fast`/`normal`/`slow` (300 ya matchea) |

Detalle del easing hardcodeado: `cubic-bezier(0.2,0.8,0.2,1)` en `page-transition`, `reveal`, `latency-comparator`, `faq`.

## Esquema propuesto

### 1. CSS vars en `@theme` (globals.css)

```css
--duration-fast: 150ms;
--duration-normal: 300ms;
--duration-slow: 500ms;
--ease-default: cubic-bezier(0.2, 0.8, 0.2, 1);
```

### 2. Mapa JS para framer-motion/animejs

framer-motion y animejs NO consumen CSS vars en `duration` — exportar `web/src/lib/motion.ts`:

```ts
export const MOTION = {
  duration: { fast: 0.15, normal: 0.3, slow: 0.5 },
  ease: [0.2, 0.8, 0.2, 1],
};
```

### 3. Reveal consume CSS vars directamente

`Reveal` (CSS transition) SÍ puede leerlas vía `transitionDuration` / `transitionDelay`.

## Tabla de reemplazo

| Actual | Token |
|---|---|
| `page-transition` .28 | → `normal` (.3) |
| `latency` .4 / .5 | → `slow` (.5) |
| `Reveal` delay | → `--duration-fast` |
| stagger `i*80` | → `motion.delay(i)` con base fast |
| Tailwind `duration-200` (13) | → `duration-fast` |
| Tailwind `duration-300` (6) | → `duration-normal` (matchean token) |
| Tailwind `duration-500` | → `duration-slow` |
| animejs settle ms | → `.slow` / `.normal` |
| animejs loop 2400 | queda (ambient intencional) |

## Notas

- Recomendación: centralizar en `motion.ts`; dejar CSS vars solo para `Reveal` y utilities Tailwind.
- Solo auditoría + propuesta (alcance del backlog). Cero cambios de código.
