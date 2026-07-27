# Presupuesto de Performance — Frontend Web

> Estado actual: **Sin optimizar** — sin bundle analyzer, sin code splitting, sin lazy loading.
> Próxima revisión: post-MVP.

---

## 1. Visión General

El frontend web de VantaDB se entrega actualmente sin ninguna estrategia de
optimización de performance. Corre sobre Next.js 16 + React 19 + Tailwind v4,
pero toda página es `"use client"` (sin React Server Components), no hay
`dynamic()`, no hay `<Suspense>`, no hay bundle analyzer. `node_modules` pesa
~230MB+, con ~150MB en dependencias muertas.

El objetivo de este documento es establecer un presupuesto de performance
claro y una hoja de ruta priorizada para alcanzarlo.

---

## 2. Bundle Analysis

### Dependencias activas (~10-15 paquetes)

| Paquete | Tamaño est. | Propósito |
|---------|-------------|-----------|
| next + react + react-dom | ~25MB | Framework core |
| framer-motion | ~1.2MB | Page transitions, Reveal |
| animejs | ~500KB | Mark animations |
| lucide-react | ~1MB | Iconos |
| tailwindcss (dev) | ~200MB+ | CSS framework |
| tw-animate-css | ~50KB | Utilidades de animación |

### Dependencias zombie (17 paquetes, ~150MB+ desperdicio)

`@dnd-kit/*`, `@tanstack/react-query`, `@tanstack/react-table`, `date-fns`,
`next-auth`, `next-intl`, `react-syntax-highlighter`, `recharts`, `sharp`,
`react-hook-form`, `@hookform/resolvers`, `zod`, `zustand`, `uuid`,
`@mdxeditor/editor`, `cmdk`, `@reactuses/core`, `z-ai-web-dev-sdk`,
`embla-carousel-react`.

**Ninguna tiene imports activos en el bundle real.** Ver
`reference/deps-audit.md` para el análisis completo.

### Código muerto (1,157 líneas)

| Archivo | Líneas | Reemplazado por |
|---------|--------|-----------------|
| `navbar.tsx` | 546 | `site-navbar.tsx` |
| `hero-mark-interactive.tsx` | 312 | `mark/` directory |
| `metrics-bar.tsx` | 131 | Eliminado de HomeView |
| `ecosystem.tsx` | 168 | Eliminado de HomeView |
| Bloque `{false && ...}` en hero.tsx | ~30 | Nunca renderiza |

### Toast systems duplicados

`Toaster` (shadcn/ui) y `Sonner` están ambos montados. Solo uno es necesario.

---

## 3. Code Splitting — Estado Actual

**No existe.** Cero instancias de:

- `dynamic()` de `next/dynamic`
- `React.lazy()`
- `<Suspense>`
- `import()` dinámico

Toda página es un solo bundle monolítico. Componentes pesados (blog content,
WAL simulator, editor MDX) se cargan en el bundle crítico sin necesidad.

---

## 4. Build Config — Issues Conocidos

```ts
// next.config.ts — actual
output: "standalone"
typescript: { ignoreBuildErrors: true }  // ← oculta errores, no los resuelve
reactStrictMode: false                   // ← deshabilita detección de problemas
// Sin images.metadataBase               // ← SEO: previene canonical correcta
// Sin images.remotePatterns             // ← seguridad: permite cualquier host
// Sin compression                       // ← servidor no comprime respuestas
// Sin experimental features             // ← no aprovecha optimizaciones Nx
```

---

## 5. Recomendaciones Priorizadas

### P0 — Fácil, alto impacto inmediato

| # | Acción | Impacto | Esfuerzo |
|---|--------|---------|----------|
| 1 | Eliminar 17 zombie deps | ~150MB menos en node_modules, ~30s menos en install | 1h |
| 2 | Eliminar archivos muertos | ~1,157 líneas menos | 30min |
| 3 | Consolidar toast system (elegir Sonner) | ~1 bundle menos | 15min |
| 4 | Configurar `metadataBase` | SEO 80→90+ | 5min |

### P1 — Moderado, impacto alto en carga

| # | Acción | Impacto | Esfuerzo |
|---|--------|---------|----------|
| 5 | Agregar `@next/bundle-analyzer` | Monitoreo visible del bundle | 30min |
| 6 | Envolver blog content, WAL simulator con `dynamic()` | Reduce bundle crítico ~30% | 1h |
| 7 | Agregar `loading.tsx` por segmento de ruta | Mejora LCP percibido | 1h |
| 8 | Configurar `images.remotePatterns` | Seguridad + rendimiento imágenes | 30min |

### P2 — Arquitectural, requiere planificación

| # | Acción | Impacto | Esfuerzo |
|---|--------|---------|----------|
| 9 | Migrar páginas estáticas a RSC | Menos JS en cliente para landing, docs | 2-3d |
| 10 | ISR para blog y case-studies | Páginas estáticas + datos frescos | 1-2d |
| 11 | Lighthouse CI en GitHub Actions | Prevención de regresiones | 1d |

---

## 6. Targets de Performance

### Lighthouse v12

| Métrica | Actual | Target Mínimo | Target Ideal |
|---------|--------|---------------|--------------|
| Performance | ? | 85 | 95+ |
| Accessibility | ? | 90 | 95+ |
| Best Practices | ? | 85 | 90+ |
| SEO | 80-85 | 90 | 98+ |

### Core Web Vitals

| Métrica | Actual | Target | Notas |
|---------|--------|--------|-------|
| LCP | ? | <2.5s | Hero section debe cargar primero |
| TBT | ? | <200ms | Sin bloqueo de main thread |
| CLS | ? | <0.1 | Layout shifts en hero + animaciones |
| FCP | ? | <1.8s | Primer render con contenido |
| SI | ? | <3.4s | Speed Index en 3G simulado |
| TTI | ? | <3.8s | Time to Interactive |

### Presupuesto de bundle

| Recurso | Target | Notas |
|---------|--------|-------|
| JS crítico (rutas públicas) | <150KB gzip | Landing, docs, pricing |
| JS crítico (app) | <200KB gzip | Dashboard post-auth |
| CSS crítico | <30KB | Tailwind purgado + animations |
| Imagen hero | <200KB | WebP, 1600px máximo |
| Tamaño total página | <400KB | Sin contar imágenes externas |

---

*Este documento se actualiza tras cada auditoría de performance.
Ver `reference/deps-audit.md` para dependencias, `QA.md` para calidad,
`guides/build-deploy.md` para build/deploy.*
