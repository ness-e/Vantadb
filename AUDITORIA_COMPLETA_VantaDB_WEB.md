# Auditoría Completa del Sitio Web VantaDB

> **Fecha**: 3 Julio 2026
> **Proyecto**: VantaDB — Embedded Vector Database for AI Agents
> **URL**: https://vantadb.dev
> **Repositorio**: github.com/ness-e/Vantadb
> **Framework**: React 19 + TanStack Router + Vite 8 + Tailwind 4 + GSAP + Three.js

---

## Resumen Ejecutivo

| Dimensión | Score | Estado |
|-----------|-------|--------|
| SEO Técnico | 62/100 | 🟡 Regular |
| Diseño Visual & UI | 72/100 | 🟡 Bueno |
| UX & Navegación | 65/100 | 🟡 Regular |
| Contenido & Copy | 70/100 | 🟡 Bueno |
| Calidad Código Frontend | 55/100 | 🔴 Malo |
| Responsive & Mobile | 65/100 | 🟡 Regular |
| Branding & Consistencia | 82/100 | 🟢 Bueno |
| Docs vs Producto Real | 35/100 | 🔴 Crítico |
| Pricing & Comercial | 45/100 | 🔴 Malo |
| Auditoría General Sitio | 45/100 | 🔴 Malo |
| **PROMEDIO GENERAL** | **59.6/100** | 🟡 **Necesita Mejoras** |

---

## Tabla de Contenidos

1. [SEO Técnico](#1-seo-técnico)
2. [Diseño Visual & UI](#2-diseño-visual--ui)
3. [UX & Navegación](#3-ux--navegación)
4. [Contenido & Copywriting](#4-contenido--copywriting)
5. [Calidad Código Frontend](#5-calidad-código-frontend)
6. [Responsive & Mobile](#6-responsive--mobile)
7. [Branding & Consistencia Visual](#7-branding--consistencia-visual)
8. [Documentación vs Producto Real](#8-documentación-vs-producto-real)
9. [Pricing & Estrategia Comercial](#9-pricing--estrategia-comercial)
10. [Auditoría General del Sitio](#10-auditoría-general-del-sitio)
11. [Plan de Acción Priorizado](#11-plan-de-acción-priorizado)

---

## 1. SEO Técnico

**Score: 62/100** 🟡

### Meta Tags por Página

| Ruta | title | description | OG | canonical |
|------|-------|-------------|----|-----------|
| `/` | ✅ "VantaDB — Embedded Vector Database for AI Agents" | ✅ 157 chars | ❌ Sin og:title/description/url | ✅ |
| `/pricing` | ✅ | ✅ | ❌ | ✅ |
| `/engine` | ✅ | ✅ | ❌ | ✅ |
| `/architecture` | ✅ | ✅ | ❌ | ✅ |
| `/docs` | ✅ | ✅ | ❌ | ✅ |
| `/docs-api` | ✅ | ⚠️ 44 chars, muy corta | ❌ | ✅ |
| `/changelog` | ✅ | ✅ | ❌ | ✅ |
| `/integrations` | ✅ | ✅ | ❌ | ✅ |
| `/use-cases` | ✅ | ✅ | ❌ | ✅ |
| `/security` | ⚠️ Sin formato "—" | ⚠️ 57 chars | ❌ | ✅ |
| `/storage` | ✅ | ✅ | ❌ | ✅ |
| `/latency` | ✅ | ✅ | ❌ | ✅ |
| `/cost` | ✅ | ✅ | ❌ | ✅ |
| `/config` | ✅ | ✅ | ❌ | ✅ |
| `/maint` | ✅ | ✅ | ❌ | ✅ |
| `/about/` | ✅ | ✅ | ❌ | ✅ |
| `/about/company` | ✅ | ⚠️ Casi idéntica a /about/ | ❌ | ✅ |
| `/about/roadmap` | ⚠️ Sin "—" | ⚠️ 50 chars | ❌ | ✅ |
| `/about/contact` | ✅ | ✅ | ❌ | ✅ |
| `/about/community` | ✅ | ✅ | ❌ | ✅ |
| `/blog/` | ⚠️ Sin "—" | ✅ | ❌ | ✅ |
| `/blog/$slug` | ✅ Dinámico | ✅ Dinámico | ❌ | ❌ **Sin canonical** |
| `/solutions/ai-agents` | ✅ | ✅ | ❌ | ✅ |
| `/solutions/local-rag` | ✅ | ✅ | ❌ | ✅ |
| `/solutions/ai-ide-tooling` | ✅ | ✅ | ❌ | ✅ |
| `/product/benchmarks` | ⚠️ Sin "—" | ⚠️ 102 chars | ❌ | ✅ |
| `/about/roadmap` | ⚠️ Sin "—" | ⚠️ | ❌ | ✅ |

### Issues Críticos SEO

1. **🔴 Sin `og:title` ni `og:description` en ninguna página** — El root no define OG tags básicos, ninguna subpage los sobreescribe. En redes sociales solo se verá "VantaDB" sin descripción.
2. **🔴 `og:url` ausente** en todas las páginas.
3. **🔴 `twitter:site` y `twitter:creator` ausentes** — Solo hay `twitter:card` + `twitter:image`.
4. **🔴 `/blog/$slug` sin canonical** — Riesgo de contenido duplicado.
5. **🔴 3 rutas faltan en sitemap.xml**: `/docs-api`, `/security`, `/product/benchmarks`.
6. **🔴 JSON-LD incompleto**: `SoftwareApplication` le falta `url`, `image`, `softwareVersion`, `screenshot`, `featureList`.
7. **🔴 Sin preload de fuentes** — Afecta LCP (Largest Contentful Paint).

### Recomendaciones SEO

- Agregar `og:title`, `og:description`, `og:url` en `__root.tsx` y heredar en cada subpage.
- Agregar `twitter:site="@vantadb"` y `twitter:creator="@vantadb"`.
- Agregar canonical a `/blog/$slug`.
- Completar JSON-LD con `softwareVersion`, `featureList`, `screenshot`.
- Estandarizar titles con formato `"VantaDB — [Page Name]"`.
- Mejorar descriptions cortas (docs-api, security, roadmap).

---

## 2. Diseño Visual & UI

**Score: 72/100** 🟡

### Sistema de Diseño

**Fortalezas:**
- ✅ Uso de OKLCH para colores (perceptualmente uniforme, moderno)
- ✅ Zero-radius y zero-shadows — decisión Swiss intencional
- ✅ Clamp typography con escalas responsivas
- ✅ 3 fuentes bien seleccionadas: Space Grotesk (display), Outfit (body), JetBrains Mono (code)
- ✅ Sistema de easing definido (`--ease-cut`, `--ease-spring`)

**Debilidades:**
- ❌ `--amber` es naranja (#ff5500), no amber. Nombre confuso.
- ❌ `--white: #000000` — nombre extremadamente confuso (comentario lo explica pero es mala práctica)
- ❌ Sin escala de espaciado formal (no hay `--space-xs/sm/md/lg/xl`)
- ❌ Sin z-index scale — valores hardcodeados (100, 101, 98, 99, etc.)
- ❌ Solo light mode — `color-scheme: light` fijo, sin `prefers-color-scheme: dark`

### Inline Styles Problem

**🔴 Hallazgo crítico: ~80% del layout está en inline styles JS**

Los componentes `SwissBenchmarkGrid`, `SwissQuickstart`, `SwissCoreEngine`, `SwissArchSection`, `SwissUseCases`, `SwissEcosystem`, `SwissMonolith`, `SwissFooter` y las páginas `company.tsx`, `contact.tsx`, `community.tsx`, `pricing.lazy.tsx` definen TODO su layout con objetos `style={{}}`.

**Problemas:**
- No hay reutilización de clases CSS existentes
- Bundle más grande (strings de estilo inline se serializan en JS)
- Tailwind no puede optimizar inline styles
- Cambiar un token requiere buscar en 20+ archivos
- Clases CSS como `.swiss-card`, `.bento-grid` existen pero no se usan

### Three.js Hero

- ✅ Wireframe aesthetic coherente con estilo industrial
- ✅ `setPixelRatio(Math.min(devicePixelRatio, 2))` — performance cap
- ✅ Cleanup con `dispose()` y `forceContextLoss()`
- ❌ Sin error boundary — si WebGL falla, no hay fallback visual
- ❌ Mouse tracking global no se desactiva en mobile
- ❌ `group.position.x = 1.5` fijo — en mobile el wireframe se sale parcialmente

### Motion & Animación

- ✅ Sistema de motion definido en `animations.css`
- ✅ `prefers-reduced-motion` en CSS (desactiva animaciones)
- ❌ Solo SwissHero usa `gsap.matchMedia()` para reduced-motion
- ❌ 7 de 8 componentes GSAP no verifican `prefers-reduced-motion`

### Calificaciones por Subcategoría

| Categoría | Puntaje |
|-----------|---------|
| Design Tokens | 7/10 |
| Color System | 6/10 |
| Typography | 9/10 |
| Swiss Style | 9/10 |
| Components | 5/10 |
| Consistency | 4/10 |
| Three.js | 7/10 |
| Motion | 7/10 |

---

## 3. UX & Navegación

**Score: 65/100** 🟡

### Problemas de Navegación

| # | Problema | Severidad |
|---|----------|-----------|
| P1 | **No hay skip-link** — usuarios de teclado/screen reader deben tabular toda la nav | 🔴 Alta |
| P2 | **`isActive()` usa match exacto** — en sub-rutas ningún link se marca activo | 🟡 Media |
| P3 | **Drawer y desktop difieren**: drawer añade "Docs" en links, desktop lo tiene aparte | 🟡 Media |
| P4 | **Sin breadcrumbs** — en páginas anidadas (/solutions/*) no hay contexto de profundidad | 🟡 Media |
| P5 | **13 rutas sin acceso desde navegación principal** | 🔴 Alta |

### Information Architecture

**Problemas:**
- **/docs y /docs-api duplicados** — confunde al usuario sobre cuál visitar
- **24 rutas planas** sin jerarquía clara (producto vs recursos vs compañía)
- **Páginas de soluciones (/solutions/ai-agents, etc.) con contenido excelente pero invisibles**
- **/cost, /config, /latency, /maint, /storage existen como rutas** pero no están enlazadas desde nav ni footer

### Violaciones de Heurísticas Nielsen

| Heurística | Violaciones |
|-----------|------------|
| #1 Visibilidad del estado | Sin active state en sub-rutas |
| #2 Match sistema-realidad | "The database that thinks with you" es abstracto |
| #3 Control y libertad | Sin breadcrumbs, sin "back" contextual |
| #4 Consistencia | Nav drawer ≠ desktop nav; /docs vs /docs-api |
| #6 Reconocimiento | 13 rutas valiosas no accesibles desde nav |
| #8 Diseño minimalista | 24 rutas, varias placeholder |

### Landing Page Flow

**Orden actual:** Hero → BenchmarkGrid → Quickstart → CoreEngine → ArchSection → UseCases → Ecosystem → Monolith

**Problemas:**
- Sin CTA por sección (solo al final)
- BenchmarkGrid antes que Quickstart — prematuro mostrar benchmarks sin contexto
- Ecosystem solo lista nombres sin links ni CTAs
- Sin demo interactiva en landing

---

## 4. Contenido & Copywriting

**Score: 70/100** 🟡

### Hero Copy

| Elemento | Diagnóstico |
|----------|-------------|
| Tagline (h2) | "The database that thinks with you" — memorable pero vago |
| Labels | [RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS] — Excelentes |
| Descripción | "One pip install. Vector search, SQL, and full-text..." — Claro, concreto |
| **Issue** | El `h1` solo contiene "VantaDB". Keywords principales no están en h1 |

### Blog Content (4 posts analizados)

| Post | Score | Notas |
|------|-------|-------|
| Introducing VantaDB | 7/10 | Sólido pero corto, faltan CTAs inline |
| Why I built VantaDB | 9/10 | El mejor post. Problemas concretos, solución técnica |
| SQLite for AI Agents | 8/10 | Benchmarks concretos, diferenciación fuerte |
| How Hybrid Search Works | 8/10 | Código real, solo falta summary ejecutivo |

**Issues blog:**
- ❌ Sin Open Graph tags en `$slug.tsx`
- ❌ Sin schema.org/BlogPosting
- ❌ Sin CTAs de conversión (ningún post pide probar el producto)
- ❌ Sin "related posts" ni suscripción

### Tone of Voice

- ✅ Alto nivel de consistencia — mismas frases recurrentes
- ✅ Buen balance técnico pero accesible
- ✅ Sin buzzwords vacíos
- ✅ Personalidad segura y directa
- ❌ Nav.tsx tiene aria-labels en español ("Cerrar menú", "Abrir menú") — sitio en inglés

### Puntuación por Página

| Página | Score | Notas |
|--------|-------|-------|
| SwissHero | 7/10 | Tagline evocador pero h1 sin keywords |
| Blog posts | 8/10 | Excelente profundidad técnica, sin conversión |
| Pricing | 6/10 | Funcional, sin persuasión ni prueba social |
| Company | 7/10 | Buen propósito, falta humanización |
| Community | 8/10 | Accionable, claro, invitador |
| Roadmap | 4/10 | Demasiado vago, no genera confianza |
| Contact | 7/10 | Claro, profesional |
| Solutions | 8/10 | Problema/solución claro con code snippets |

---

## 5. Calidad Código Frontend

**Score: 55/100** 🔴

### TypeScript Config

- ✅ `strict: true`, path alias `@/*`
- ❌ `noUnusedLocals: false` — desactiva checks importantes
- ❌ `noUnusedParameters: false`
- ❌ Sin `noUncheckedIndexedAccess`

### Componentes React

| Componente | Problemas |
|------------|-----------|
| SwissHero (346 líneas) | Sin React.memo, Three.js cleanup ✅, sin error boundary |
| SwissCoreEngine (187 líneas) | **100% inline styles**, sin accesibilidad keyboard, max-height truncable |
| SwissQuickstart (315 líneas) | **100% inline styles**, CSS inline via `<style>` JSX |
| SwissMonolith | **100% inline styles**, hover via onMouseEnter/Leave (mala práctica grave) |
| SwissFooter | **100% inline styles**, `<style>` JSX inline, colores hardcodeados |
| VsTable | **100% inline styles**, React.memo ✅ |

### CSS Architecture (27 archivos)

**Problemas graves:**
- ❌ Botones duplicados en `buttons.css` y `swiss-hero.css`
- ❌ Footer duplicado en `footer.css` y `utilities.css`
- ❌ Layout sections duplicadas en `layout.css` y `swiss-grid.css`
- ❌ Dependencia de orden de importación frágil
- ❌ 3 sistemas de estilo compitiendo: CSS classes + inline styles + `<style>` JSX

### Performance

- ❌ Solo 4/26 rutas con lazy loading
- ❌ Sin `manualChunks` para code splitting
- ❌ Sin compresión gzip/brotli configurada
- ❌ **GSAP + motion juntos** — 2 librerías de animación redundantes
- ❌ Sin preload de fuentes

### Score General Código

| Dimensión | Puntaje |
|-----------|---------|
| TypeScript | 7/10 |
| Componentes | 4/10 |
| CSS Architecture | 3/10 |
| Performance | 5/10 |
| Routing | 7/10 |
| Consistencia | 3/10 |
| Dependencias | 8/10 |

---

## 6. Responsive & Mobile

**Score: 65/100** 🟡

### Estrategia Responsive

- ✅ Excelente uso de `clamp()` en tipografía
- ❌ **Sin sistema de breakpoints centralizado** — 6+ breakpoints distintos en 27 archivos
- ❌ Usa `max-width` (mobile-last) en vez de `min-width` (mobile-first)
- ❌ Breakpoints duplicados: 768px en 6 archivos, 640px en 5 archivos

### Issues por Sección

| Sección | Score | Problemas |
|---------|-------|-----------|
| Mobile Nav | 8/10 | Nav drawer funcional, touch targets < 44px |
| Responsive Layouts | 6/10 | `.section-split` sin MQ, bento grids frágiles |
| Touch Targets | 6/10 | Hamburger 36px, nav-cta ~32px, close button 36px (mínimo 44px) |
| Typography | 9/10 | Clamp() en toda la base |
| Hero Section | 7/10 | Three.js sin fallback real en mobile |
| Pricing | 7/10 | Tabla scrollable OK, FAQ con !important |
| **About Pages** | **4/10** | **0 media queries propias** — grids de 2-3 cols no colapsan |
| Footer | 8/10 | 3 implementaciones distintas (SwissFooter.tsx, footer.css, utilities.css) |
| Images | 2/10 | Sin srcset/lazy loading (no hay imágenes de contenido) |

### Issues Críticos Responsive

1. **🔴 About pages sin media queries** — company.tsx (grid 2 cols), community.tsx (grid 2 y 3 cols), contact.tsx (grid 3 cols) no colapsan en mobile
2. **🔴 `.section-split` sin media query** — overflow asegurado en mobile
3. **🔴 Touch targets < 44px** — hamburger, close button, nav CTAs
4. **🔴 Triple implementación de footer** — mantenimiento duplicado
5. **🟡 Breakpoints inconsistentes** — 6 breakpoints distintos sin sistema centralizado

---

## 7. Branding & Consistencia Visual

**Score: 82/100** 🟢

### Brand Platform (BRAND_PLATFORM.md)

| Componente | Estado |
|------------|--------|
| Propósito | ✅ "Make vector-native data infrastructure invisible" |
| Visión | ✅ AI apps con zero-ops data infrastructure |
| Misión | ✅ Build fastest converged DB engine in Rust |
| Valores (5) | ✅ Radical Simplicity, Performance, Developer Empathy, Open by Default, AI-Native |
| Territorio | ✅ Industrial precision ↔ Developer warmth |
| Arquetipos | ✅ Primary: Magician / Secondary: Creator |
| Tagline system | ✅ 7 taglines contextuales |

### Verbal Identity (VERBAL_IDENTITY.md)

| Componente | Estado |
|------------|--------|
| Voice dimensions (6) | ✅ Precision/Warmth 60/40, Confidence/Humility 70/30 |
| Tone matrix | ✅ 7 contexts con ajustes específicos |
| Writing principles | ✅ "Show the number", "Lead with the verb", etc. |
| Glosario editorial | ✅ Always Use / Use Carefully / Never Use |

### Inconsistencias Encontradas

| # | Severidad | Issue |
|---|-----------|-------|
| 1 | 🔴 Crítico | **OG image** (`og/default.svg`) usa `#ff6a00` en vez del brand amber `#ff5500`, fondo `#08080c` en vez de `#0a0a0a` |
| 2 | 🔴 Crítico | **Logo** (`VantaDBLogo.tsx`) sin variante dark — se pierde en footer OLED |
| 3 | 🟡 Alto | Pricing hardcodea `#ff3b30` (rojo Apple) en vez de `var(--danger)` |
| 4 | 🟡 Alto | Spec drift: DiseñoNuevo.md especifica Three.js wireframe pero implementation_plan.md lo eliminó sin actualizar DiseñoNuevo.md |
| 5 | 🟡 Alto | SwissHero.tsx hardcodea `"#ff5500"` en vez de `var(--amber)` |
| 6 | 🟡 Alto | Pricing CTA buttons usan inline styles hardcodeados, bypassando el sistema de botones global |
| 7 | 🟢 Medio | Nav tiene texto en español ("Cerrar menú", "Abrir menú") en sitio en inglés |
| 8 | 🟢 Medio | 404 page usa clases Tailwind (`rounded-md`, `bg-primary`) que no existen en tokens.css |

---

## 8. Documentación vs Producto Real

**Score: 35/100** 🔴 **CRÍTICO**

### Claims Verdaderos vs Falsos

| Claim Web | Realidad | Veredicto |
|-----------|----------|-----------|
| "Sub-millisecond hybrid queries" | Rust HNSW: 1.2ms (10K). Python: **179ms**. Competitive: **39.74ms** | ❌ **Falso** |
| "HNSW + BM25 + RRF" | Implementado en core | ✅ **Verdadero** |
| "Apache 2.0" | LICENSE confirma | ✅ **Verdadero** |
| "SQLite for AI Agents" / "SQL" | SQL **no existe**. ARCHITECTURE.md: "SQL, deferred" | ❌ **Falso** |
| "Zero servers, zero ops, one pip install" | `pip install vantadb-py` **no funciona** (solo TestPyPI) | ⚠️ **Exagerado** |
| "Unlimited vectors" | Roadmap: "1M vectors — No OOM" como exit criteria | ⚠️ **Exagerado** |
| "Python SDK + Rust SDK + CLI" | Python ✅, Rust ✅, CLI ✅ | ✅ **Verdadero** |
| "LangChain + LlamaIndex integrations" | **No existen** en el workspace | ❌ **Falso** |
| "SSH/SAML" | No implementado, deferido en roadmap | ❌ **Falso** |
| "Embeddings generated automatically" (Quickstart) | VantaDB **NO** genera embeddings. Debes proveer `Vec<f32>` | ❌ **Falso** |

### API Documentation — Discrepancias Graves

La web muestra APIs que **no existen** en el producto real:

| Web (docs.tsx) | Real (Python SDK) | Problema |
|----------------|-------------------|----------|
| `db.put(key="doc-1", ...)` | `db.put(namespace, key, payload, ...)` | Falta namespace y payload |
| `db.search_memory(query=[...])` | `db.search_memory(namespace, query_vector=[...])` | Parámetro incorrecto |
| `results[0].score` | `results["records"][0]["score"]` | Estructura de retorno incorrecta |
| `db.create_collection("agent_memory", 1536)` | **No existe** | Método inventado |

**Riesgo**: Cualquier desarrollador que intente los ejemplos de la web recibirá errores inmediatos.

### Benchmarks — Discrepancias Mayores

| Métrica | Web dice | Real (BENCHMARKS.md) |
|---------|----------|---------------------|
| VantaDB Search QPS | **1,195** | **24.3** (competitive bench) |
| VantaDB p99 Latency | **2.43ms** | **58.245ms** (competitive) |
| ChromaDB Search QPS | **450** | **978.6** (BENCHMARKS.md) |

**🔴 La realidad es inversa**: Web muestra VantaDB 50x más rápido que ChromaDB, pero BENCHMARKS.md real muestra ChromaDB 40x más rápido que VantaDB.

### Integraciones — Reales vs Ficticias

| Integración | Estado Real |
|-------------|-------------|
| vantadb-python | ✅ Completo |
| vantadb-mcp | ⚠️ Experimental |
| vantadb-server | ✅ Completo |
| vantadb-wasm | ✅ Funcional |
| vantadb-openai/ollama/crewai/etc | 🟡 Scaffold (código básico) |
| vantadb-ts | 🔴 Solo README |
| vantadb-langchain | 🔴 **NO EXISTE** |
| llama-index-vantadb | 🔴 **NO EXISTE** |

### Pricing — Realidad

| Tier | Precio | ¿Existe? |
|------|--------|----------|
| Self-Hosted | $0 | ✅ Código real |
| Cloud Pro ($29/mo) | Contact Sales | ❌ Placeholder — Cloud alpha: Dic 2026 |
| Cloud Business ($149/mo) | Contact Sales | ❌ Placeholder |
| Enterprise (Custom) | Contact Sales | ❌ Placeholder — Pre-seed fundraising Q1 2027 |

### Riesgos de Credibilidad

1. **🔴 Benchmarks falsos**: Si alguien corre los benchmarks reales, descubrirá el engaño.
2. **🔴 API docs rotas**: Ejemplos de código web lanzan errores.
3. **🔴 SQL inventado**: Claim de "SQL" en hero cuando no existe.
4. 🟠 **Pip install no funcional**: El CTA principal de la web (`pip install vantadb-py`) falla.
5. 🟠 **Cloud tiers aspiracionales**: "Deploy Now" para producto que no existe.
6. 🟠 **Embeddings automáticos**: Quickstart dice que se generan solos — falso.

---

## 9. Pricing & Estrategia Comercial

**Score: 45/100** 🔴

### Análisis de Tiers

| Tier | Precio | Problema |
|------|--------|----------|
| Self-Hosted | $0 forever | ✅ Bien posicionado |
| Cloud Pro | $29/mo | ⚠️ **1M vectors** vs Self-Hosted **Unlimited** — castiga al que paga |
| Cloud Business | $149/mo | ⚠️ Salto 5x, justificación débil |
| Enterprise | Custom | ⚠️ SOC 2/HIPAA "ready" sin evidencia |

### Issues Críticos de Pricing

1. **🔴 Cloud plans no existen como SaaS** — No hay infraestructura cloud, signup, ni billing. "Deploy Now" lleva a formulario de contacto.
2. **🔴 SOC 2 / HIPAA mencionados sin implementación** — AES-256-GCM está en roadmap. Riesgo legal.
3. **🔴 Self-Hosted "unlimited vectors" vs Cloud Pro "1M vectors"** — Estás castigando a quien paga.
4. **🔴 Cero social proof** — Sin testimonios, logos, GitHub stars, ni case studies.
5. **🟡 CTA engañoso**: "Deploy Now" → formulario de contacto (bait-and-switch).
6. **🟡 FAQ insuficiente**: Solo 4 preguntas. Faltan: migration, limits, trials, diferencia con competidores.
7. **🟡 POPULAR badge en Cloud Pro es contraproducente** cuando Self-Hosted tiene menos restricciones.

### Conversión

| CTA | Destino | Evaluación |
|-----|---------|------------|
| Self-Hosted → "Get Started" | /docs | ✅ Buen path |
| Cloud Pro → "Deploy Now" | /about/contact | ❌ Fricción máxima |
| Cloud Business → "Deploy Business" | /about/contact | ❌ Engañoso |
| Enterprise → "Contact Sales" | /about/contact | ✅ Correcto |

---

## 10. Auditoría General del Sitio

**Score: 45/100** 🔴

### Inventario de Rutas (26 total)

**Lazy loading**: Solo 4/26 rutas (15%) tienen lazy components

**Rutas huérfanas (no enlazadas desde nav ni footer)**:
- `/about/roadmap`, `/solutions/ai-agents`, `/solutions/ai-ide-tooling`, `/solutions/local-rag`, `/product/benchmarks`, `/docs-api`, `/security`, `/storage`, `/cost`, `/config`, `/latency`, `/maint`, `/changelog`

**13 rutas sin acceso desde navegación principal** — contenido valioso pero invisible.

### Enlaces Rotos

- Discord en community.tsx: `"#"` (placeholder)
- X/Twitter en community.tsx: `"#"` (placeholder)
- Emails en contact.tsx: **texto plano, no `mailto:`** — no clickeables

### Seguridad

- ❌ Sin Content-Security-Policy (CSP)
- ❌ Sin Strict-Transport-Security (HSTS)
- ❌ Sin X-Content-Type-Options
- ❌ Sin HTTP→HTTPS redirect configurado

### Analytics

- ❌ **Cero analytics** — sin visibilidad de tráfico ni conversiones

### Testing

- ❌ 0 tests unitarios
- ❌ 0 tests de componentes
- ⚠️ 2 tests E2E (smoke: homepage loads, nav present)
- ❌ Sin cobertura configurada

---

## 11. Plan de Acción Priorizado

### 🔴 Fase 1 — Crítico (1-2 semanas)

| # | Acción | Categoría | Impacto |
|---|--------|-----------|---------|
| 1 | **Corregir API docs** — Sincronizar ejemplos con API real de PYTHON_SDK.md | Docs/Producto | Evita que usuarios fallen al implementar |
| 2 | **Corregir benchmarks** — Usar números reales de BENCHMARKS.md o quitarlos | Producto | Evita crisis de credibilidad |
| 3 | **Quitar SQL del hero** — Cambiar a "Vector search + BM25 + hybrid search" | Copy/SEO | Evita claim falso |
| 4 | **Agregar OG tags** — og:title, og:description, og:url en __root.tsx + subpages | SEO | Redes sociales, previews |
| 5 | **Marcar Cloud tiers como "Coming Soon"** — Cambiar CTAs a "Join Waitlist" | Pricing | Honestidad, reduce riesgo legal |
| 6 | **Agregar security headers** — CSP, HSTS, XFO en vercel.json | Seguridad | Crítico para producción |
| 7 | **Agregar HTTP→HTTPS redirect** en vercel.json | Seguridad | Estándar mínimo |

### 🟡 Fase 2 — Alto (2-4 semanas)

| # | Acción | Categoría | Impacto |
|---|--------|-----------|---------|
| 8 | **Migrar inline styles a CSS classes** — SwissCoreEngine, SwissFooter, SwissBenchmarkGrid, SwissMonolith | Frontend | Mantenibilidad, consistencia |
| 9 | **Unificar sistema de botones** — Eliminar duplicación buttons.css + swiss-hero.css | CSS | Consistencia visual |
| 10 | **Agregar lazy loading a todas las rutas** — 22 rutas restantes | Performance | Bundle size |
| 11 | **Configurar manualChunks** en vite.config.ts | Performance | Code splitting |
| 12 | **Agregar skip-link** para accesibilidad | UX/A11y | Accesibilidad básica |
| 13 | **Agregar breadcrumbs** — especialmente en /solutions/*, /about/* | UX | Navegación |
| 14 | **Agregar media queries a About pages** — company, community, contact | Responsive | Mobile UX |
| 15 | **Estandarizar titles y descriptions** — formato "VantaDB — [Page]" en TODAS | SEO | Consistencia |
| 16 | **Corregir OG image** — Usar colores de marca (#ff5500, #0a0a0a) | Branding | Consistencia visual |
| 17 | **Agregar dark variant al Logo** — Prop `inverted` para footer OLED | Branding | Visibilidad |

### 🟢 Fase 3 — Medio (1-2 meses)

| # | Acción | Categoría | Impacto |
|---|--------|-----------|---------|
| 18 | **Migrar a mobile-first CSS** (min-width en vez de max-width) | Responsive | Mejora general |
| 19 | **Agregar `prefers-reduced-motion` check** en todos los useGSAP | Motion | Accesibilidad |
| 20 | **Agregar dark mode** con prefers-color-scheme | UI | Experiencia |
| 21 | **Agregar analytics** (Plausible, Fathom, o similar) | Analytics | Visibilidad |
| 22 | **Hacer navegables las páginas Solutions** en Nav | UX | Descubrimiento |
| 23 | **Renombrar tokens confusos**: --amber → --orange, --white → --text-on-dark | Tokens | Claridad |
| 24 | **Agregar canonical a /blog/$slug** | SEO | Evita duplicado |
| 25 | **Agregar spacing scale**: --space-xs/sm/md/lg/xl | Tokens | Consistencia |
| 26 | **Agregar z-index scale** centralizada | Tokens | Mantenibilidad |
| 27 | **Agregar pendienteComponent a rutas lazy** | UX | Loading states |
| 28 | **Escribir al menos 5 tests unitarios** (Nav, Footer, VsTable) | Testing | Calidad |
| 29 | **Expandir FAQ de pricing a 8+ preguntas** | Pricing | Conversión |

### 🔵 Fase 4 — Bajo (3+ meses)

| # | Acción | Categoría | Impacto |
|---|--------|-----------|---------|
| 30 | **Agregar blog posts** (el sistema funciona, falta contenido) | Contenido | SEO, engagement |
| 31 | **Migrar documentación a sistema basado en archivos (MDX)** | Docs | Mantenibilidad |
| 32 | **Agregar calculadora de pricing** | Pricing | Conversión |
| 33 | **Agregar sidebar de navegación en /docs** | UX | Documentación |
| 34 | **Agregar testimonios y social proof** | Branding | Confianza |
| 35 | **Agregar PWA manifest + apple-touch-icon** | Branding | Mobile |
| 36 | **Implementar dark mode** completo | UI | Experiencia |
| 37 | **Renovar roadmap.tsx** con fechas, versiones y progress | Contenido | Confianza |
| 38 | **Construir waitlist con dashboard preview para Cloud** | Producto | Revenue pipeline |
| 39 | **Evaluar unificar GSAP + motion en una sola librería** | Frontend | Bundle size |

---

## Resumen de Scores por Dimensión

```
SEO Técnico        ██████████████░░░░░░░░ 62/100  🟡
Diseño Visual      █████████████████░░░░░ 72/100  🟡
UX/Navegación      █████████████░░░░░░░░░ 65/100  🟡
Contenido/Copy     ██████████████░░░░░░░░ 70/100  🟡
Código Frontend    ███████████░░░░░░░░░░░ 55/100  🔴
Responsive/Mobile  █████████████░░░░░░░░░ 65/100  🟡
Branding           █████████████████░░░░░ 82/100  🟢
Docs vs Producto   ███████░░░░░░░░░░░░░░░ 35/100  🔴
Pricing/Comercial  █████████░░░░░░░░░░░░░ 45/100  🔴
Auditoría General  █████████░░░░░░░░░░░░░ 45/100  🔴
────────────────────────────────────────────────
PROMEDIO           ████████████░░░░░░░░░░ 59.6/100 🟡
```

---

## Metodología

Esta auditoría fue realizada por 10 agentes especializados utilizando las siguientes skills y herramientas:

1. **SEO Técnico**: Análisis manual de meta tags, structured data, sitemap, robots.txt
2. **Diseño Visual**: Revisión de tokens CSS, componentes React, sistema de diseño Swiss
3. **UX/Navegación**: Heurísticas de Nielsen, Krug's laws, análisis de information architecture
4. **Contenido**: Análisis de copy en todas las páginas, blog posts, tone of voice
5. **Código Frontend**: Revisión de TypeScript, React, CSS architecture, performance
6. **Responsive**: Análisis de breakpoints, media queries, touch targets, grids
7. **Branding**: Comparación contra BRAND_PLATFORM.md, VERBAL_IDENTITY.md, tokens
8. **Docs vs Producto**: Comparación de claims web contra documentación real (BENCHMARKS.md, PYTHON_SDK.md, ARCHITECTURE.md)
9. **Pricing**: Análisis de tiers, CTAs, conversión, psicología de precios
10. **Auditoría General**: Inventario completo de rutas, enlaces, seguridad, testing, deployment
