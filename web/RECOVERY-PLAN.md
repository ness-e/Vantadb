# VantaDB Website — Plan de Recuperación Completo

> Documento generado el 2025-07-25
> Fuente: `referencias/vantadb-conversation-log.md` + `worklog.md` + análisis del código actual

---

## Índice

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Decisiones Vinculantes del Usuario](#2-decisiones-vinculantes-del-usuario)
3. [Tech Stack](#3-tech-stack)
4. [Referencia del Diseño 2](#4-referencia-del-diseño-2)
5. [Comandos de Desarrollo](#5-comandos-de-desarrollo)
6. [22 Rutas Planeadas (App Router)](#6-22-rutas-planeadas-app-router)
7. [Especificación del Navbar](#7-especificación-del-navbar)
8. [13 Secciones del Home](#8-13-secciones-del-home)
9. [Estado Actual vs. Estado Deseado](#9-estado-actual-vs-estado-deseado)
10. [Componentes Existentes (Recuperados)](#10-componentes-existentes-recuperados)
11. [Componentes y Rutas Faltantes](#11-componentes-y-rutas-faltantes)
12. [Plan de Recuperación por Fases](#12-plan-de-recuperación-por-fases)
13. [Design System Completo](#13-design-system-completo)
14. [Contenido Técnico VantaDB](#14-contenido-técnico-vantadb)

---

## 1. Resumen Ejecutivo

El proyecto original del sitio web VantaDB se construyó en una sesión con otra IA que incluía:
- **App Router real** con 22 rutas (Next.js)
- **30+ componentes** con estética manga/linocut neo-brutalista
- **i18n ES/EN** con 300+ keys
- **SiteShell + SiteNavbar** con dropdowns (Platform/Solutions/Resources)
- **13 secciones** en Home
- Múltiples features avanzadas (command palette, easter egg, tutoriales interactivos, comparador de latencia, playground de código, etc.)

El workspace se reseteó y el proyecto original **nunca fue commiteado a git**. Se perdió por completo. Una reconstrucción parcial recuperó ~30 componentes pero como **SPA** (no App Router), sin las rutas reales, sin SiteShell/SiteNavbar, y sin las 22 páginas.

Este documento captura **todo lo que se necesita** para recuperar el estado original completo.

### ⚠️ Riesgo de Infraestructura: Filesystem `referencias/`

Durante la sesión original, el usuario intentó subir archivos (~34MB en .zip/.rar) múltiples veces pero **nunca llegaron al disco**. El directorio `referencias/` siempre aparecía vacío. Esto es un problema del gateway IM que no procesa las subidas al filesystem real.  
**Implicación para la recuperación:** No confiar en subidas de archivos para recuperar assets. Usar solo el contenido que ya está en disco o pegado como texto.

---

## 2. Decisiones Vinculantes del Usuario

Extraídas literalmente de los mensajes 27 y 29 de la conversación:

| # | Decisión | Prioridad |
|---|----------|-----------|
| 1 | Tema claro crema `#FBF9F5` con toggle a oscuro | ✅ Hecho |
| 2 | Mascota gato se queda (hero, navbar, favicon, easter egg; video + imágenes limpias **coming later**) | ✅ Hecho (parcial) |
| 3 | Sombras del diseño 1 (negro, no amber) | ✅ Hecho |
| 4 | **App Router real** con dropdowns de categorías | ❌ **NO HECHO** (SPA) |
| 5 | Mantener mascota en hero (video sin fondo **coming later**) | ⏳ Pendiente |
| 6 | Home con 13 secciones rediseñadas al estilo D1 | ⏳ 10 de 13 hechas |
| 7 | Tipografía del diseño 1 (Anton + Space Mono) | ✅ Hecho |
| 8 | **Todas las funcionalidades intactas** | ❌ Parcial |
| 9 | `#FF5500` como acento único en ambos temas | ✅ Hecho |
| 10 | Página en español E inglés (i18n) | ⏳ Sistema listo, ~170 keys actual vs 300+ meta |
| 11 | **Rediseñar TODAS las páginas del diseño 2 al estilo D1** (D1 es el principal) | ❌ **NO HECHO** |
| 12 | Workflow: "1. Comprensión, 2. Preguntas de decisión — No avanzar sin respuestas" | ✅ Flujo acordado |
| A | **App Router real** (NO SPA, NO pushState, NO hash) | ❌ **NO HECHO** |
| B1 | Auto-detect + toggle manual, localStorage | ✅ Hecho |
| B2 | Traducir TODO excepto técnico y nombres propios | ⏳ Parcial |
| C | Prioridad: Tier 1 → Tier 2 → Tier 3 | ❌ No implementado |
| D | 13 secciones del Home confirmadas | ⏳ 10 de 13 |

---

## 3. Tech Stack

Stack tecnológico completo extraído de `package.json`, `next.config.ts` y las decisiones de la conversación:

### Framework y Lenguaje
| Tecnología | Versión | Uso |
|-----------|---------|-----|
| Next.js | ^16.1.1 | Framework App Router + React Server Components |
| React | ^19.0.0 | UI components |
| TypeScript | ~5.x | Tipado estático |
| bun | — | Package manager y runtime |

### Estilos y UI
| Tecnología | Uso |
|-----------|-----|
| Tailwind CSS | ^4 (con `tw-animate-css`) |
| shadcn/ui | Componentes base (new-york style, neutral base) |
| @radix-ui/* | ~20 paquetes de accesibilidad (dropdown, dialog, accordion, etc.) |
| lucide-react | Iconos |
| class-variance-authority | Variantes de componentes |
| clsx + tailwind-merge | Utilidad `cn()` |

### Animaciones
| Tecnología | Uso |
|-----------|-----|
| framer-motion | ^12.23.2 — Page transitions, animated bars, Reveal |
| next-themes | Toggle claro/oscuro |
| CSS animations | marquee, blink, flicker, rise, stamp, scan, pulse-ring |

### i18n
| Tecnología | Uso |
|-----------|-----|
| **Custom** LanguageProvider | Context React + auto-detect + localStorage |
| **No usa** next-intl | Instalado pero no usado |
| Diccionarios | **Actual: ~170 keys** · **Meta original: 300+ keys** ES/EN en `src/lib/dictionaries.ts` |

### Datos y Base de Datos
| Tecnología | Uso |
|-----------|-----|
| Prisma | ORM (SQLite) |
| SQLite | Base de datos embebida |
| @tanstack/react-query | Data fetching |
| zustand | Estado global (instalado, uso secundario) |

### UI Libraries
| Tecnología | Uso |
|-----------|-----|
| sonner | Toast notifications |
| cmdk | Command palette base |
| embla-carousel-react | Carousel |
| vaul | Drawer (shadcn) |
| input-otp | OTP input |
| recharts | Charts |
| @tanstack/react-table | Tables |
| react-hook-form + zod | Formularios |
| date-fns | Fechas |
| uuid | IDs únicos |

### Fonts
| Font | Rol |
|-----|-----|
| **Anton** | Display/headings (impacto manga) |
| **Space Mono** | Tech/code (monospace) |
| **Geist** | Body/sans-serif |
| JetBrains Mono | Alternativa code (del diseño 2, no usada) |

### Configuration (next.config.ts)
| Setting | Valor |
|---------|-------|
| `output` | `"standalone"` (paraployment) |
| `ignoreBuildErrors` | `true` |
| `reactStrictMode` | `false` |
| Build script | `.zscripts/build.sh` (auto-inyecta standalone) |
| Reverse proxy | Caddyfile → `:81` → `localhost:3000` |

### Archivos fuente
| Archivo | Propósito |
|---------|-----------|
| `src/app/globals.css` | 1034 líneas: sistema de diseño manga/linocut + dark mode |
| `src/components/vanta/vanta-data.ts` | 448 líneas: datos centralizados (benchmarks, CLI, FAQ, tutoriales, changelog) |
| `src/lib/dictionaries.ts` | ~1500 líneas: 300+ keys ES/EN |
| `.env` | `DATABASE_URL=file:/home/z/my-project/db/custom.db` (local path — reemplazar) |

---

## 4. Referencia del Diseño 2

> *Nota sobre nomenclatura (definida por el usuario en Mensaje 25):*  
> **Diseño 1 (D1)** = el diseño original manga/linocut con tema claro (crema `#FBF9F5`)  
> **Diseño 2 (D2)** = el diseño comprimido (`web.rar`) con estética Swiss/brutalist oscura  
> El usuario ordenó: *"rediseñar todas las paginas de diseño 2 para que combinen con el diseño 1 que será el 'principal'"*
>
> **Ubicación en disco:**  
> - `referencias/diseño1/` — Next.js + App Router (post-fusión D1 + D2)  
> - `referencias/diseño2/` — Vite + TanStack Router (Swiss/brutalist oscuro original)

El diseño 2 era un archivo comprimido (`web.rar`) que el usuario subió con una estructura **Vite + React + TanStack Router** con estética **Swiss/brutalist oscura**. Este diseño se fusionó con el diseño 1 (manga/linocut claro) siguiendo las decisiones del usuario.

### Recomendación ORIGINAL del asistente (RECHAZADA por el usuario)
El asistente recomendó inicialmente (Mensaje 26):
- Tema oscuro default, eliminar mascota del hero, sombras ámbar
- SPA expandida con pushState, oscilloscope en hero
- Space Grotesk + Outfit (tipografía)

**El usuario rechazó explícitamente TODAS** estas recomendaciones (Mensaje 27). Se documentan aquí como contexto de diseño alternativo descartado.

### Características del Diseño 2 (referencia visual, NO implementar tal cual)

| Aspecto | Valor |
|---------|-------|
| Framework | Vite + React + TanStack Router |
| Tema | Oscuro por defecto (#0d0d0d) |
| Acento | Ámbar (#f50) — *idéntico a #FF5500* |
| Hero | Oscilloscope animado (señal de query SVG en vivo) |
| Tipografía | Space Grotesk (display) + Outfit (sans) + JetBrains Mono (code) |
| Routing | ~20 rutas con TanStack Router |
| Estructura | Navbar con dropdowns Platform/Solutions/Resources |
| Sombras | Ámbar rígidas (3px 3px 0 0) |
| Efectos | Grid técnico sutil, noise overlay, scanlines |
| Secciones | TrustBar, MetricsBar, Core Engine, Use Cases, Ecosystem, Pricing Preview, Vector Nebula |

### Lo que se RESCATÓ del Diseño 2 (rediseñado al estilo D1)

| Elemento | Decisión del usuario |
|----------|---------------------|
| Estructura multi-página | ✅ App Router real (NO TanStack Router) |
| Dropdowns agrupados | ✅ Platform/Solutions/Resources |
| Secciones nuevas | ✅ TrustBar, MetricsBar, CoreEngine, UseCases, Ecosystem, CTA |
| Oscilloscope hero | ❌ Reemplazado por mascota gato |
| Tema oscuro default | ❌ Rechazado — tema claro crema con toggle |
| Tipografía | ❌ Rechazada — se queda Anton + Space Mono |
| Sombras ámbar | ❌ Rechazadas — se quedan sombras negras |

---

## 5. Comandos de Desarrollo

```bash
bun run dev        # puerto 3000, logs a dev.log
bun run build      # standalone output (.next/standalone/)
bun run start      # NODE_ENV=production bun .next/standalone/server.js
bun run lint       # eslint (muy permisivo — casi todas las reglas off)
bun run db:push    # prisma db push --accept-data-loss
bun run db:generate
bun run db:migrate
```

Primera vez: `bun install && bun run db:push`

---

## 6. Rutas Planeadas (App Router)

> **Nota sobre el conteo:** El original tenía 22 rutas (Apéndice B exacto). La reconstrucción alcanzó 23 (posiblemente contando `[...slug]` como extra). En otro momento se mencionan "25 rutas" (quizás contando slugs individuales de blog/case-studies).  
> **Conteo objetivo de esta recuperación:** 22 rutas base + catch-all = **23 rutas funcionales**.

### Tier 1 — Páginas principales (ALTA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/` | Home (13 secciones) | ⚠️ Existe como SPA, no como ruta App Router |
| `/benchmarks` | BENCH-01 + SIFT1M + Latency Explorer | ❌ No existe como ruta |
| `/docs` | Installation + Quickstart + CLI + Server + Playground | ❌ No existe como ruta |

### Tier 2 — Páginas de contenido (MEDIA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/engine` | Core Engine | ❌ No existe |
| `/architecture` | Pipeline + Search Semantics | ❌ No existe |
| `/playground` | Code Playground interactivo | ❌ No existe |
| `/why-vantadb` | Benefits + comparison | ❌ No existe |
| `/changelog` | Release timeline | ❌ No existe |
| `/pricing` | 3 plan cards | ❌ No existe |

### Tier 2 — Soluciones (MEDIA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/solutions/ai-agents` | AI Agents use case | ❌ No existe |
| `/solutions/local-rag` | Local RAG use case | ❌ No existe |
| `/solutions/ai-ide-tooling` | IDE Tooling use case | ❌ No existe |

### Tier 2 — Información (MEDIA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/security` | Security pillars | ❌ No existe |
| `/use-cases` | Solution links | ❌ No existe |
| `/cost` | TCO comparison | ❌ No existe |
| `/maint` | Zero maintenance | ❌ No existe |

### Tier 3 — Blog y Casos de Estudio (BAJA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/blog` | Blog cards listing | ❌ No existe |
| `/blog/[slug]` | Blog post individual | ❌ No existe |
| `/case-studies` | Case study cards | ❌ No existe |
| `/case-studies/[slug]` | Case study individual | ❌ No existe |

### Tier 3 — About (BAJA prioridad)

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/about/company` | Company info | ❌ No existe |
| `/about/team` | Team members | ❌ No existe |
| `/about/community` | Community links | ❌ No existe |
| `/about/contact` | Contact form | ❌ No existe |

### Catch-all

| Ruta | Contenido | Estado |
|------|-----------|--------|
| `/[...slug]` | 404 / Catch-all redirect | ❌ No existe |

---

## 7. Especificación del Navbar

Estructura exacta que debe tener el SiteNavbar (según Apéndice C de la conversación):

```
[Logo/Avatar Gato]  VANTA DB

PLATAFORMA ▼          SOLUCIONES ▼          RECURSOS ▼          [Flat]
├── Core Engine       ├── AI Agents          ├── Why VantaDB     Security
├── Architecture      ├── Local RAG          ├── Benchmarks      Use Cases
                      ├── IDE Tooling        ├── Playground      Pricing
                                             ├── Changelog
                                             ├── Case Studies
                                             ├── Blog

[Search ⌘K]  [ES|EN]  [🌙/☀️]  [GitHub]
```

**Nota:** El navbar actual (`src/components/vanta/navbar.tsx`) no tiene esta estructura de dropdowns. El original perdido tenía `SiteNavbar.tsx` con este diseño exacto. **En la migración a App Router, `navbar.tsx` debe ser eliminado** y reemplazado por `site-navbar.tsx` (que se integrará en `site-shell.tsx`).

---

## 8. 13 Secciones del Home

Según la decisión D (confirmada). **Las 13 secciones originales del Home NO incluían Changelog** — esa era una ruta independiente (`/changelog`).

| # | Sección | Origen | Componente | Estado |
|---|---------|--------|------------|--------|
| 1 | Hero (mascota + stats + tagline) | D1+D2 | `hero.tsx` | ✅ Existe |
| 2 | Trust Bar (logos/credibilidad) | D2 | ❌ `trust-bar.tsx` | ❌ **FALTA** |
| 3 | Metrics Bar (stats animados) | D1+D2 | ❌ `metrics-bar.tsx` | ❌ **FALTA** |
| 4 | Feature Grid (6 Core Capabilities) | D1+D2 | `features.tsx` | ✅ Existe |
| 5 | Core Engine (visualización del motor) | D2 | ❌ `core-engine.tsx` | ❌ **FALTA** |
| 6 | Quickstart / Code Terminal | D1 | `code-terminal.tsx` | ✅ Existe |
| 7 | Architecture Pipeline (BM25→HNSW→RRF) | D1 | `architecture.tsx` | ✅ Existe |
| 8 | Benchmark Race (Latency Explorer) | D1+D2 | `latency-comparator.tsx` | ⚠️ Solo en Benchmarks, falta en Home |
| 9 | Use Cases (AI Agents, Local RAG, IDE) | D2 | ❌ `use-cases.tsx` | ❌ **FALTA** |
| 10 | Ecosystem (integraciones) | D2 | ❌ `ecosystem.tsx` | ❌ **FALTA** |
| 11 | FAQ Accordion | D1 | `faq-section.tsx` | ✅ Existe |
| 12 | Tutorials (4 cards + modal) | D1 | `tutorials-section.tsx` | ✅ Existe |
| 13 | CTA Final / Vector Nebula | D2 | ❌ `cta-final.tsx` | ❌ **FALTA** |

**Totales:** 6 existen en Home, 7 faltan (TrustBar, MetricsBar, CoreEngine, LatencyComparator standalone, UseCases, Ecosystem, CTA Final).

---

## 9. Estado Actual vs. Estado Deseado

### Arquitectura
| Aspecto | Estado Actual (SPA) | Estado Deseado (App Router) |
|---------|---------------------|------------------------------|
| Routing | `useState<View>` en page.tsx | `src/app/` con directorios reales |
| Navegación | Cambio de estado, sin URL | URLs reales, history API |
| SEO | Sin meta tags por página | Meta tags por ruta |
| Navbar | `navbar.tsx` básico | `SiteShell` + `SiteNavbar` con dropdowns |
| Shell | No existe | `SiteShell` layout compartido |
| 404 | No hay | `[...slug]` catch-all |

### Funcionalidades
| Feature | Estado en SPA actual | Necesario en App Router |
|---------|---------------------|------------------------|
| Home 13 secciones | 6/13 | Crear 7 secciones faltantes |
| Benchmarks | Vista interna | Ruta independiente |
| Docs | Vista interna | Ruta independiente |
| i18n | LanguageProvider funciona | Mantener idéntico |
| Command Palette | ✅ Existe | Migrar a SiteShell |
| Easter Egg | ✅ Existe | Migrar a SiteShell |
| Tutoriales | ✅ Existen | Migrar a Home + ruta |
| Changelog | ⚠️ En Home (debe ser ruta) | Mover a `/changelog` |
| Playground | ✅ Existe (en Docs) | Ruta independiente `/playground` |
| LatencyComparator | ✅ Existe (en Benchmarks) | Migrar a Home + Benchmarks |
| Theme toggle | ✅ Funciona | Mantener en SiteShell |
| **Scroll Progress** | ✅ Existe | **Migrar a SiteShell** |
| **Back-to-top** | ✅ Existe | **Migrar a SiteShell** |
| **PWA Manifest** | ✅ Existe | **Re-enlazar en layout** |
| **OG Image** | ✅ Existe | **Re-enlazar por ruta** |
| Docs search/filter | ✅ Existe | Migrar a `/docs` |
| Docs scroll-spy | ✅ Existe | Migrar a `/docs` |
| Per-card CLI copy | ✅ Existe | Migrar a `/docs` |
| Tutorial search/filter | ✅ Existe | Migrar a Home |
| FAQ EN | ⚠️ Verificar | Crear si no existe |
| Syntax highlighting | ✅ Existe | Re-enlazar en Playground |
| Line numbers gutter | ✅ Existe | Re-enlazar en Playground |
| Scroll sync playground | ✅ Existe | Re-enlazar en Playground |
| Typing animation | ✅ Existe | Re-enlazar en code-terminal |
| Preset workloads | ✅ Existe | Re-enlazar en LatencyComparator |
| Benchmark sim + JSON | ✅ Existe | Re-enlazar en LatencyComparator |
| Tooltip en bars | ✅ Existe | Re-enlazar en LatencyComparator |
| Focus trap en modals | ✅ Existe | Re-enlazar en SiteShell |
| Glitch-hover titles | ✅ Existe | Re-enlazar |
| Glow badges | ✅ Existe | Re-enlazar |
| Count-up results | ✅ Existe | Re-enlazar |

---

## 10. Componentes Existentes (Recuperados)

### 30+ componentes vanta — YA EXISTEN y funcionan:

```
src/components/vanta/
├── architecture.tsx
├── back-to-top.tsx
├── benchmarks-view.tsx
├── changelog-section.tsx
├── code-playground.tsx
├── code-terminal.tsx
├── command-palette.tsx
├── copy-utils.ts
├── count-up.tsx
├── docs-view.tsx
├── easter-egg.tsx
├── faq-section.tsx
├── features.tsx
├── footer.tsx
├── hero.tsx
├── home-view.tsx
├── ink-divider.tsx
├── lang-toggle.tsx
├── latency-comparator.tsx
├── navbar.tsx
├── page-transition.tsx
├── reveal.tsx
├── scroll-progress.tsx
├── search-semantics.tsx
├── shortcut-overlay.tsx
├── theme-provider.tsx
├── theme-toggle.tsx
├── toast.tsx
├── trust-section.tsx
├── tutorial-modal.tsx
├── tutorials-section.tsx
├── use-focus-trap.ts
├── use-parallax.ts
├── use-reveal.ts
├── use-typing-lines.ts
├── vanta-data.ts

src/lib/
├── dictionaries.ts          (300+ keys ES/EN)
├── language-provider.tsx

src/app/
├── globals.css              (1034 líneas, diseño completo)
├── layout.tsx               (ThemeProvider + LanguageProvider + Toasters)
├── opengraph-image.tsx      (OG image dinámica)
├── page.tsx                 (entry point SPA — será reemplazado)
```

**Estos componentes NO necesitan ser re-escritos.** Solo necesitan ser migrados al App Router.

---

## 11. Componentes y Rutas Faltantes

### 11.1 — Infraestructura App Router (CRÍTICO)

| Archivo | Propósito |
|---------|-----------|
| `src/app/layout.tsx` | **Reemplazar:** debe usar SiteShell como layout raíz |
| `src/app/(home)/page.tsx` | Ruta Home con metadata y SEO |
| `src/app/benchmarks/page.tsx` | Ruta Benchmarks |
| `src/app/docs/page.tsx` | Ruta Docs |
| `src/app/engine/page.tsx` | Ruta Core Engine |
| `src/app/architecture/page.tsx` | Ruta Architecture |
| `src/app/playground/page.tsx` | Ruta Playground |
| `src/app/why-vantadb/page.tsx` | Ruta Why VantaDB |
| `src/app/changelog/page.tsx` | Ruta Changelog |
| `src/app/pricing/page.tsx` | Ruta Pricing |
| `src/app/solutions/ai-agents/page.tsx` | Ruta AI Agents |
| `src/app/solutions/local-rag/page.tsx` | Ruta Local RAG |
| `src/app/solutions/ai-ide-tooling/page.tsx` | Ruta IDE Tooling |
| `src/app/security/page.tsx` | Ruta Security |
| `src/app/use-cases/page.tsx` | Ruta Use Cases |
| `src/app/cost/page.tsx` | Ruta Cost/TCO |
| `src/app/maint/page.tsx` | Ruta Maintenance |
| `src/app/blog/page.tsx` | Ruta Blog listing |
| `src/app/blog/[slug]/page.tsx` | Ruta Blog post dinámico |
| `src/app/case-studies/page.tsx` | Ruta Case Studies listing |
| `src/app/case-studies/[slug]/page.tsx` | Ruta Case Study dinámico |
| `src/app/about/company/page.tsx` | Ruta Company |
| `src/app/about/team/page.tsx` | Ruta Team |
| `src/app/about/community/page.tsx` | Ruta Community |
| `src/app/about/contact/page.tsx` | Ruta Contact |
| `src/app/[...slug]/page.tsx` | Catch-all 404 |

### 11.2 — Nuevos Componentes de Infraestructura (CRÍTICO)

| Archivo | Propósito |
|---------|-----------|
| `src/components/vanta/site-shell.tsx` | Layout shell: Navbar + Footer + global modals (CommandPalette, ShortcutOverlay, EasterEgg, BackToTop, ScrollProgress) |
| `src/components/vanta/site-navbar.tsx` | Navbar con dropdowns Platform/Solutions/Resources, reemplaza navbar.tsx |

### 11.3 — 7 Secciones del Home Faltantes

| # | Sección | Componente Necesario |
|---|---------|---------------------|
| 2 | Trust Bar (logos/credibilidad) | `trust-bar.tsx` |
| 3 | Metrics Bar (stats animados) | `metrics-bar.tsx` |
| 5 | Core Engine (visualización del motor) | `core-engine.tsx` |
| 8 | Latency Comparator standalone en Home | `latency-comparator.tsx` (reubicar desde Benchmarks) |
| 9 | Use Cases (AI Agents, Local RAG, IDE) | `use-cases.tsx` |
| 10 | Ecosystem (integraciones) | `ecosystem.tsx` |
| 13 | CTA Final / Vector Nebula | `cta-final.tsx` |

### 11.4 — Páginas de Contenido Faltantes (Tier 2/3)

Estas páginas necesitan **componentes de página** en `src/components/vanta/pages/`:

| Página | Componente Necesario |
|--------|---------------------|
| Engine | `engine-page.tsx` |
| Architecture | `architecture-page.tsx` (reusa architecture.tsx) |
| Why VantaDB | `why-vantadb-page.tsx` |
| Pricing | `pricing-page.tsx` |
| Security | `security-page.tsx` |
| Use Cases | `use-cases-page.tsx` |
| Cost | `cost-page.tsx` |
| Maintenance | `maint-page.tsx` |
| AI Agents | `solutions-ai-agents-page.tsx` |
| Local RAG | `solutions-local-rag-page.tsx` |
| IDE Tooling | `solutions-ide-tooling-page.tsx` |
| Blog | `blog-page.tsx` + `blog-post-page.tsx` |
| Case Studies | `case-studies-page.tsx` + `case-study-page.tsx` |
| Company | `about-company-page.tsx` |
| Team | `about-team-page.tsx` |
| Community | `about-community-page.tsx` |
| Contact | `about-contact-page.tsx` |

### 11.5 — Contenido en vanta-data.ts (Faltante)

| Sección | Datos Necesarios |
|---------|-----------------|
| Use Cases | 4+ casos de uso con descripción, icono, link |
| Pricing | 3 planes con características, precio, CTA |
| Blog posts | Array de posts con title, slug, date, excerpt, content |
| Case Studies | Array de cases con title, slug, company, results |
| Team members | Array con name, role, bio, avatar |
| Security pillars | 4+ pilares con descripción |
| FAQ EN | Traducción al inglés de FAQ (8 preguntas) |

### 11.6 — i18n Traducciones Faltantes

| Ámbito | Estado |
|--------|--------|
| 25 componentes con useLanguage + t() | ⚠️ Verificar cobertura |
| 15 páginas Tier 2/3 (~337 strings) | ❌ No traducidas |
| FAQ EN (8 preguntas) | ⚠️ Fue creado en el original — verificar si existe en `vanta-data.ts` |

---

## 12. Plan de Recuperación por Fases

### FASE 0 — Preparación (Estimado: 30 min)
- [ ] Hacer commit del código actual a git (`git init && git add . && git commit -m "snapshot: current SPA state"`)
- [ ] Verificar que `bun run dev` funciona sin errores
- [ ] Leer RECOVERY-PLAN.md completo

### FASE 1 — Infraestructura App Router (Estimado: 2-3 horas)
- [ ] Crear `site-shell.tsx` con layout compartido (Navbar + Footer + modales globales)
- [ ] Crear `site-navbar.tsx` con dropdowns Platform/Solutions/Resources
- [ ] Reemplazar `src/app/layout.tsx` para usar SiteShell
- [ ] Crear directorios de rutas Tier 1: `/benchmarks`, `/docs`
- [ ] Mover componentes de vista a páginas individuales
- [ ] Migrar globals.css, providers (ThemeProvider, LanguageProvider)
- [ ] Verificar: 3 rutas HTTP 200, navegación funciona, sin errores

### FASE 2 — Páginas Tier 2 (Estimado: 2-3 horas)
- [ ] Crear 12 rutas Tier 2 (engine, architecture, playground, why-vantadb, changelog, pricing, solutions/*, security, use-cases, cost, maint)
- [ ] Cada página con su metadata SEO y contenido
- [ ] Reutilizar componentes existentes donde sea posible
- [ ] Verificar: 15 rutas HTTP 200, dropdowns funcionan

### FASE 3 — Home 13 secciones completas (Estimado: 1-2 horas)
- [ ] Crear §06 — UseCases/Ecosystem
- [ ] Crear §11 — Metrics/Trust Bar
- [ ] Crear §13 — CTA Final
- [ ] Integrar LatencyComparator como sección standalone en Home
- [ ] Verificar: 13 secciones renderizan, sin errores

### FASE 4 — Páginas Tier 3 (Estimado: 2-3 horas)
- [ ] Crear rutas /blog, /blog/[slug]
- [ ] Crear rutas /case-studies, /case-studies/[slug]
- [ ] Crear rutas /about/* (company, team, community, contact)
- [ ] Crear catch-all /[...slug]
- [ ] Añadir datos de blog, case studies, team a vanta-data.ts
- [ ] Verificar: 22 rutas HTTP 200

### FASE 5 — i18n Completo (Estimado: 2-3 horas)
- [ ] Verificar que todos los componentes usan `useLanguage() + t()`
- [ ] Traducir ~337 strings faltantes de Tier 2/3
- [ ] Crear FAQ EN (8 preguntas)
- [ ] Verificar: toggle ES/EN funciona en todas las rutas

### FASE 6 — SEO y Polish (Estimado: 1-2 horas)
- [ ] Añadir metadata (title, description, open graph) a cada ruta
- [ ] Generar sitemap.xml
- [ ] Verificar lighthouse / rendimiento
- [ ] `bun run lint` 0 errores

---

## 13. Design System Completo

Extraído del Apéndice F + código actual:

### Paleta
| Token | Light | Dark |
|-------|-------|------|
| Background | `#FBF9F5` (cream) | `#0a0a0a` (vanta black) |
| Text | `#000000` | `#FBF9F5` |
| Accent | `#FF5500` (neon) | `#FF5500` (neon) |
| Paper | `#F2EDE2` | `#1A1A1A` |
| Smoke | `#1A1A1A` | `#2A2A2A` |

### Tipografía
| Role | Font |
|------|------|
| Display/Headings | `Anton` (impacto, sans-serif) |
| Tech/Code | `Space Mono` (monospace) |
| Body/Sans | `Geist` (sans-serif) |

### Bordes y Sombras
- `border-4` (border ancho, estilo neo-brutalista)
- `shadow-[6px_6px_0_0_#000]` (sombra rígida, sin blur)
- En dark: `shadow-[6px_6px_0_0_#FBF9F5]`
- `border-radius: 0.125rem` (esquinas mínimas, casi rectas)

> *Nota: El diseño 2 tenía `border-radius: 0` (cero) y sombras ámbar de 3px. El usuario rechazó ambos.*

### Efectos CSS (todos existen en globals.css)
`.press`, `.press-lg`, `.press-neon`, `.glitch-hover`, `.glow-neon`, `.glow-box-neon`, `.halftone`, `.speed-lines`, `.grid-tech`, `.text-outline-neon`, `.neon-underline`, `.scanlines`, `.ink-drip`, `.manga-frame`, `.btn-neon-glow`, `.kinetic-type`, `.animated-gradient-border`, `.ink-divider`, `.animate-float`, `.stagger-children`, `.accent-bar-top`, `.stripe-accent`, `.ink-corner`, `.shadow-throw`

---

## 14. Contenido Técnico VantaDB

(Datos verificados del repositorio oficial ness-e/Vantadb)

### Tagline
> "Embedded Rust engine for durable local memory and hybrid vector retrieval."

### Arquitectura
- Local-first, embedded, Rust + PyO3
- Zero network, in-process execution
- WAL con checksums CRC32C (crash-safe recovery)

### Hybrid Search
- BM25 (keyword) + HNSW (vector) via RRF (Reciprocal Rank Fusion)

### Benchmarks (BENCH-01)
| Métrica | BM25 | HNSW | Hybrid |
|---------|------|------|--------|
| Ingestión | — | 5,400 vec/s | — |
| p50 | 0.85ms | 1.20ms | 2.10ms |
| p99 | 2.10ms | 3.50ms | 4.80ms |

### SIFT1M Phase 2
Speedups: 2.14x–2.80x (SIMD, static prefetch, O(M²) select_neighbors)

### CLI
```
vanta-cli put --db <path> --collection <name> --key <key> [--text <str>] [--vector <float,...>] [--metadata <json>]
vanta-cli list <db_path>
vanta-cli export <db_path> [output_format]
vanta-cli rebuild-index <db_path>
vanta-cli audit-index <db_path> [mode]
vanta-cli repair-text-index <db_path>
```

### Quickstart Python
```python
import vantadb_py as vantadb
db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)
record = db.put("agent/main", "memory-001", "In-process execution minimizes latency.", metadata={"category": "architecture", "priority": 1}, vector=[0.12, 0.88, 0.54])
stored = db.get("agent/main", "memory-001")
hits = db.search("agent/main", vector=[0.11, 0.89, 0.55], top_k=5)
caps = db.hardware_profile()
db.flush()
db.close()
```

### Instalación
```bash
pip install vantadb-py
cargo add vantadb
# Binary:
curl -LO https://github.com/ness-e/Vantadb/releases/download/v0.1.2/vanta-cli-x86_64-unknown-linux-gnu.tar.gz
```

### Requisitos
- Rust 1.94.1+
- Python 3.11+
- Licencia: Apache 2.0
- Repo: https://github.com/ness-e/Vantadb

---

---

## Apéndice A — Historial de Desarrollo Original (11 Rondas webDevReview)

El proyecto original se construyó en 11 rondas automáticas de desarrollo (cada 15 minutos). Cada ronda añadía features sobre la base anterior. **Todas estas funcionalidades deben existir en la versión recuperada.**

| Ronda | Features añadidas |
|-------|-------------------|
| 1 | Dark mode (next-themes), scroll reveal, latency comparator, docs search, FAQ accordion, accesibilidad (skip-link, focus-visible, prefers-reduced-motion, ARIA) |
| 2 | Scroll progress bar, count-up stat counters, command palette (⌘K), mascot parallax, CLI copy per-card, Reveal en Features + Architecture |
| 3 | Toast notifications (sonner), back-to-top, shortcut overlay (?), page transitions (framer-motion), copy-to-clipboard fallback, Reveal en CodeTerminal + SearchSemantics |
| 4 | Docs scroll-spy, code block copy buttons, enhanced Python syntax highlighting (9 token types, 5+ colores), "vanta" easter egg, OG image dinámica, Tutorials section (§08) |
| 5 | Tutorial modal (5 steps × 4 tutoriales), typing animation (useTypingLines, 90ms/line), tutorial search/filter por level, PWA manifest, ARIA en modales |
| 6 | Preset workloads (RAG/Edge/Agent/Custom), benchmark simulation (3s timer, live progress), focus trap (useFocusTrap en 3 modales), Server Mode copy button |
| 7 | Changelog timeline (§09), typing animation en tutorial codeblocks (char-by-char, 20ms/tick), benchmark JSON export, code playground con simulator, enhanced styling (glow/glitch/shadow CSS utilities) |
| 8 | Glitch-hover en titles de todas las secciones, changelog filter (search + tag: Todos/MVP/Perf/Stable), animated bar charts (framer-motion, 400-500ms ease-out), playground syntax highlighting overlay (9 token types) |
| 9 | Tooltip on hover en latency bars (p50/p99/qps), Reveal animations en Benchmarks view, line numbers gutter en playground (1..N, dinámico), glow en Benchmarks title |
| 10 | Reveal animations en Docs view (5 sections), count-up animation en benchmark results (rAF, easeOutCubic, 800ms), glow-box-neon en 5 section badges, scroll sync en playground (gutter+pre+textarea) |
| 11 | i18n wire-up (25 componentes), SiteShell + SiteNavbar con dropdowns, 6 nuevas secciones Home (TrustBar, MetricsBar, CoreEngine, UseCases, Ecosystem, CTA), navbar.tsx eliminado |

---

## Checklist de Verificación Final

### Rutas e Infraestructura
- [ ] `bun run dev` → servidor en puerto 3000
- [ ] 23 rutas HTTP 200 (22 + catch-all)
- [ ] Navegación con dropdowns funcional (Platform/Solutions/Resources)
- [ ] SiteShell envuelve todas las páginas (Navbar + Footer + overlays)
- [ ] Catch-all `[...slug]` muestra 404/Coming Soon
- [ ] Sin overflow horizontal en mobile (390px)

### Tema y Estilo
- [ ] Tema claro/oscuro funciona en todas las rutas
- [ ] `border-4` + `shadow-[6px_6px_0_0_#000]` consistente
- [ ] Dark mode invierte sombras a `#FBF9F5`
- [ ] Glitch-hover en todos los section titles
- [ ] Glow-box-neon en 5 section badges

### i18n
- [ ] Auto-detect: navegador EN → muestra inglés
- [ ] Toggle ES|EN cambia todo el UI instantáneamente
- [ ] Persistencia tras reload (localStorage)
- [ ] Contenido técnico (código, CLI, API names) se mantiene en inglés
- [ ] FAQ conmuta ES↔EN correctamente
- [ ] ~300+ keys en diccionarios (actual ~170 → completar)

### Home (13 secciones)
- [ ] §01 Hero (mascota parallax + stats animados + tagline)
- [ ] §02 Trust Bar (nueva)
- [ ] §03 Metrics Bar (nueva)
- [ ] §04 Feature Grid (6 Core Capabilities)
- [ ] §05 Core Engine (nueva)
- [ ] §06 Code Terminal (typing animation 29 líneas)
- [ ] §07 Architecture Pipeline (BM25→HNSW→RRF)
- [ ] §08 Latency Comparator standalone
- [ ] §09 Use Cases (AI Agents, Local RAG, IDE)
- [ ] §10 Ecosystem
- [ ] §11 FAQ Accordion
- [ ] §12 Tutorials (4 cards + modal + search/filter)
- [ ] §13 CTA Final / Vector Nebula

### Features Interactivas
- [ ] Command Palette (⌘K) accesible globalmente — busca pages, features, CLI, FAQ, docs
- [ ] Easter egg "vanta" (teclear v-a-n-t-a secuencialmente)
- [ ] Shortcut overlay ("?") muestra todos los atajos
- [ ] Back-to-top flotante (neon, aparece tras 400px scroll)
- [ ] Scroll progress bar (neon, tick marks 25/50/75%)
- [ ] Keyboard shortcuts: G+H (Home), G+B (Benchmarks), G+D (Docs)

### Benchmarks
- [ ] BENCH-01 table (5,400 vec/s, BM25/HNSW/Hybrid p50/p99)
- [ ] SIFT1M table (2.14x–2.80x speedups)
- [ ] Latency bars animadas (framer-motion, tooltip on hover)
- [ ] Latency Comparator con sliders (top_k, dimensions, dataset)
- [ ] Preset workloads (RAG/Edge/Agent/Custom)
- [ ] Benchmark simulation (3s timer, progress bar, results grid)
- [ ] Benchmark JSON export (Copy JSON button)
- [ ] Count-up animation en resultados
- [ ] Reveal entrance animations

### Docs
- [ ] 5 sections (Installation, Quickstart, CLI, Server, Full Docs)
- [ ] Scroll-spy (sidebar auto-highlights active section)
- [ ] Search/filter en sidebar
- [ ] Code block copy buttons en install blocks
- [ ] Per-card CLI copy (6 commands, CliCard)
- [ ] Code Playground (editor + syntax highlighting + simulator)
- [ ] Line numbers gutter (1..N, dinámico)
- [ ] Scroll sync gutter/textarea
- [ ] Reveal entrance animations

### Changelog
- [ ] Ruta independiente `/changelog` (NO en Home)
- [ ] Timeline con 3 releases (v0.1.0/v0.1.1/v0.1.2)
- [ ] Search + tag filter (Todos/MVP/Perf/Stable)

### PWA y SEO
- [ ] PWA manifest (manifest.json HTTP 200, 4 icons, theme-color #FF5500)
- [ ] OG image dinámica (1200×630, mascota + branding)
- [ ] SEO metadata (title, description) en cada ruta

### Accesibilidad
- [ ] Skip-to-content link
- [ ] `:focus-visible` outline (3px solid #FF5500)
- [ ] `prefers-reduced-motion` desactiva animaciones
- [ ] ARIA labels en todos los controles interactivos
- [ ] Focus trap en modales (Tutorial, CommandPalette, ShortcutOverlay)
- [ ] `aria-expanded`/`aria-controls` en accordion y dropdowns

### Build y Calidad
- [ ] `bun run lint` 0 errores
- [ ] `bun run build` exitoso (standalone output)

---

## Apéndice B — Auditoría de Hallazgos del Proyecto (2026-07-25)

> ⚠️ **Política de acción:** Cada hallazgo listado aquí requiere investigación antes de decidir qué hacer. Ningún cambio debe ejecutarse sin antes determinar si el archivo/dependencia se elimina, se conserva, se mueve o se transforma. Si la investigación no da una respuesta concluyente, **preguntar al usuario antes de continuar**.

---

### B.1 — Archivos Huérfanos o Sin Uso Claro

| Archivo | Peso | Hallazgo | Investigación Requerida |
|---------|------|----------|------------------------|
| `referencias/` (directorio completo) | ~4.6 MB | Conversaciones, logs y proyecto duplicado extraído de .rar. Nada de esto es código fuente del sitio. | ❓ ¿Se necesita conservar `diseño1/` y `diseño2/` como referencia para la reconstrucción? ¿O ya no son necesarios? |
| `worklog.md` | 70 KB | Bitácora de desarrollo histórico. No es código fuente. | ❓ Preguntar al usuario: ¿mover a `docs/` como referencia histórica o no es necesario? |
| `RECOVERY-PLAN.md` | 34 KB | Plan de recuperación. | ❓ Preguntar al usuario: ¿qué hacer con este documento una vez completada la recuperación? |
| `vantadb-home.png` | 37 KB | Screenshot de diseño. | ❓ Preguntar al usuario: ¿conservar como referencia visual o no? |
| `tailwind.config.ts.bak` | 1.7 KB | Backup huérfano de migración Tailwind v3→v4. | ❓ Verificar que la migración a v4 está completa y preguntar al usuario qué hacer. |
| `dev.err.log` | 0 bytes | Vacío. | ❓ Preguntar al usuario qué hacer con este archivo. |
| `public/logo.svg` | — | SVG no referenciado por ningún componente. | ❓ Preguntar al usuario: ¿hay planes de usar este logo en el futuro? |
| `public/assets/estilo_landing_manga.png` | — | Solo referenciado en `worklog.md` como referencia de layout. | ❓ Preguntar al usuario: ¿conservar como asset de diseño o no? |
| `src/app/api/route.ts` | — | Endpoint "Hello World" placeholder. | ❓ Preguntar al usuario: ¿hay plan de implementar API endpoints reales o este archivo no es necesario? |
| `src/lib/db.ts` | — | PrismaClient singleton. No lo importa ningún componente. | ❓ Preguntar al usuario: ¿se va a usar Prisma en el futuro o este archivo sobra? |
| `prisma/` (directorio completo) | — | Schema scaffold con modelos `User` y `Post` (default de Prisma). | ❓ Preguntar al usuario: ¿se necesita integración con base de datos? |
| `src/hooks/use-mobile.ts` | — | Solo lo importa `sidebar.tsx` que tampoco se usa. | ❓ Preguntar al usuario: ¿se va a implementar sidebar responsivo? |

---

### B.2 — Componentes shadcn/ui Sin Uso Detectado (36 archivos)

De 48 componentes shadcn, la app solo importa directamente **2**: `toaster.tsx` y `sonner.tsx`. Los siguientes **36 componentes** no tienen importaciones desde la aplicación (solo referencias internas entre ellos):

`accordion.tsx`, `alert.tsx`, `alert-dialog.tsx`, `aspect-ratio.tsx`, `avatar.tsx`, `badge.tsx`, `breadcrumb.tsx`, `calendar.tsx`, `card.tsx`, `carousel.tsx`, `chart.tsx`, `checkbox.tsx`, `collapsible.tsx`, `context-menu.tsx`, `drawer.tsx`, `dropdown-menu.tsx`, `form.tsx`, `hover-card.tsx`, `input-otp.tsx`, `input.tsx`, `label.tsx`, `menubar.tsx`, `navigation-menu.tsx`, `pagination.tsx`, `popover.tsx`, `progress.tsx`, `radio-group.tsx`, `resizable.tsx`, `scroll-area.tsx`, `select.tsx`, `separator.tsx`, `sheet.tsx`, `sidebar.tsx`, `skeleton.tsx`, `slider.tsx`, `switch.tsx`, `table.tsx`, `tabs.tsx`, `textarea.tsx`, `toggle-group.tsx`, `toggle.tsx`, `tooltip.tsx`

> **Nota:** `button.tsx`, `dialog.tsx`, `command.tsx` son importados por otros componentes shadcn pero NO directamente por la app. Su dependencia es indirecta.

**❓ Investigación requerida:** Revisar el diseño del proyecto (D1 + D2) para determinar si estos componentes se usarán en la reconstrucción del App Router. Si no hay certeza, preguntar al usuario. No asumir que "no se usan → se descartan". Algunos podrían ser necesarios en rutas futuras (e.g., `navigation-menu.tsx` para el SiteNavbar con dropdowns del D2).

---

### B.3 — Dependencias npm Sin Import Detectado (~20)

| Paquete | Uso Previsto | Hallazgo | Investigación |
|---------|-------------|----------|---------------|
| `next-intl` | i18n oficial de Next.js | El proyecto usa un `LanguageProvider` custom. | ❓ ¿Reemplazar el sistema custom por next-intl en la reconstrucción? |
| `next-auth` | Autenticación | No hay handlers de auth. | ❓ ¿Se necesita auth en alguna ruta futura (e.g., dashboard)? |
| `@dnd-kit/*` (3 paquetes) | Drag & drop | Sin uso. | ❓ ¿Se planea funcionalidad drag & drop? |
| `@tanstack/react-query` | Data fetching | Sin uso. | ❓ ¿Se necesita data fetching para docs dinámicos o API? |
| `@tanstack/react-table` | Tablas avanzadas | Benchmarks usan HTML+Tailwind directo. | ❓ ¿Migrar benchmarks a react-table para mejor mantenibilidad? |
| `recharts` | Gráficos | Latency bars usan framer-motion + CSS. | ❓ ¿Migrar a recharts para gráficos más complejos? |
| `react-hook-form` + `@hookform/resolvers` | Formularios | Sin uso. | ❓ ¿Se planean formularios (e.g., contacto, newsletter)? |
| `zod` | Schema validation | Sin uso. | ❓ Si no hay forms, ¿tiene otro propósito? |
| `zustand` | Estado global | Todo se maneja con React state + context. | ❓ ¿Se necesita para estado global complejo en la reconstrucción? |
| `date-fns` | Fechas | Se usan strings planos. | ❓ ¿Se necesita formateo de fechas para changelog/docs? |
| `uuid` | IDs únicos | Sin uso. | ❓ ¿Necesario para key de listas dinámicas? |
| `embla-carousel-react` | Carrusel | Sin uso. | ❓ ¿Se planea carrusel de features/testimonials? |
| `input-otp` | Input OTP | Sin uso. | ❓ ¿Se necesita para login 2FA? |
| `vaul` | Drawer | Solo importado por `drawer.tsx` de shadcn. | ❓ Ídem drawer — ¿se necesita? |
| `@mdxeditor/editor` | Editor MDX | Sin uso. | ❓ ¿Se planea sección de blog/docs con editor? |
| `react-resizable-panels` | Paneles redimensionables | Sin uso. | ❓ ¿Se necesita para playground o docs layout? |
| `react-day-picker` | Date picker | Sin uso. | ❓ ¿Se necesita en filtros o formularios? |
| `@reactuses/core` | Hooks utilitarios | Sin uso. | ❓ ¿Se usa indirectamente? Investigar dependencias. |
| `z-ai-web-dev-sdk` | SDK de Z.ai | Sin uso. | ❓ Preguntar al usuario si este paquete es necesario. |
| `sharp` | Optimización imágenes Next.js | La app usa `<img>` nativo, no `next/image`. | ❓ Si se migra a `next/image`, es necesario. Caso contrario, sobra. |

**❓ Decisión:** Para cada paquete, verificar si será necesario en la reconstrucción del App Router con las 22 rutas del D2. Si no hay certeza, **preguntar al usuario** antes de desinstalar. No decidir en lote.

---

### B.4 — Archivos Candidates a Reorganizar

| Archivo | Ubicación Actual | Posible Destino | Razón |
|---------|-----------------|----------------|-------|
| `use-focus-trap.ts` | `src/components/vanta/` | `src/hooks/` | Es un hook, no un componente. |
| `use-parallax.ts` | `src/components/vanta/` | `src/hooks/` | Misma razón. |
| `use-reveal.ts` | `src/components/vanta/` | `src/hooks/` | Misma razón. |
| `use-typing-lines.ts` | `src/components/vanta/` | `src/hooks/` | Misma razón. |
| `count-up.tsx` | `src/components/vanta/` | `src/hooks/` | Es un hook con componente interno. |

**❓ Investigación requerida:** Mover requiere actualizar ~10 imports en componentes (hero, reveal, code-terminal, etc.). Verificar que el alias `@/hooks` funcione correctamente con `tsconfig.json`. Preguntar al usuario si aprueba el movimiento antes de ejecutar.

---

### B.5 — Red Flags Potenciales

| # | Problema | Riesgo | Investigación |
|---|----------|--------|---------------|
| 1 | **`page.tsx` es SPA** — usa `useState<View>` para simular 3 vistas. No hay rutas reales. Shortcuts del PWA manifest apuntan a `/?view=benchmarks` que no existe. | **ALTO** | ❓ Migrar a App Router con rutas reales. El diseño 2 tiene la estructura de 22 rutas como referencia. Preguntar al usuario si empieza reconstrucción de rutas. |
| 2 | **Dos sistemas de toasts** — layout.tsx importa `Toaster` (shadcn) Y `Toaster as Sonner`. | **BAJO** | ❓ El DOM tiene un elemento fantasma. Preguntar al usuario qué hacer. |
| 3 | **`tsconfig.json` contradictorio** — `strict: true` + `noImplicitAny: false`. | **MEDIO** | ❓ Investigar si `noImplicitAny: false` es intencional o un error. Preguntar al usuario antes de cambiar. |
| 4 | **`ignoreBuildErrors: true`** en `next.config.ts`. | **ALTO** | ❓ Errores de TypeScript invisibles. Preguntar si habilitar type-check en build de CI. |
| 5 | **Prisma apunta a path inexistente.** | **BAJO** | ❓ Preguntar al usuario: ¿quitar Prisma completamente o configurar DB real? |
| 6 | **Easter egg global** — Escucha teclas incluso dentro de inputs. | **BAJO** | ❓ ¿Corregir para que no se active dentro de inputs? |
| 7 | **Sin `loading.tsx` ni `error.tsx`.** | **MEDIO** | ❓ Preguntar si crear error boundaries. |

---

### B.6 — Issues de Configuración

| Archivo | Issue | Investigación |
|---------|-------|---------------|
| `package.json` | `"name": "nextjs_tailwind_shadcn_ts"` (genérico) | ❓ Preguntar si renombrar a `"vantadb-website"`. |
| `package.json` | `"lint": "eslint ."` (lintea todo el proyecto sin exclude) | ❓ Preguntar si cambiar a `"lint": "eslint src/"` para evitar lintear node_modules. |
| `eslint.config.mjs` | 34 reglas deshabilitadas (casi toda la configuración) | ❓ Investigar si fue intencional para desarrollo rápido. Preguntar si re-habilitar gradualmente. |
| `.gitignore` | No ignora: `referencias/`, `*.bak`, `worklog.md`, `RECOVERY-PLAN.md`, `vantadb-home.png`, `dev.err.log` | ❓ Preguntar al usuario qué incluir en `.gitignore`. |
| `next.config.ts` | `reactStrictMode: false` | ❓ Preguntar si cambiar a `true` para detectar side-effects. |

---

### B.7 — Resumen de Investigación Pendiente

| Categoría | Cantidad | Acción Requerida |
|-----------|----------|-----------------|
| Archivos sin uso claro | ~12 + 2 directorios | Investigar uno por uno y preguntar al usuario |
| Componentes shadcn no importados | 36 archivos | Determinar si serán necesarios en reconstrucción |
| Dependencias npm sin import detectado | ~20 | Verificar necesidad post-reconstrucción |
| Hooks candidates a mover | 5 archivos | Preguntar aprobación antes de mover |
| Red flags | 7 | Priorizar por nivel de riesgo, preguntar fixes |
| Config issues | 5 | Preguntar cambios al usuario |

> **Regla:** Ningún cambio se ejecuta sin investigación previa. Si la investigación no da una respuesta concluyente, **preguntar al usuario** antes de continuar.
