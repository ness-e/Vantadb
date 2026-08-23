# CAPA 6 — RENDIMIENTO Y OPTIMIZACIÓN

---

## 14. `react-best-practices` — Rendimiento React/Next.js

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Guía de 70 reglas de rendimiento en 8 categorías prioritarias de Vercel Engineering. Prefijos: `async-` (waterfalls), `bundle-` (tamaño), `server-` (SSR/RSC), `client-` (data fetching), `rerender-` (re-renders), `rendering-` (DOM), `js-` (optimización JS), `advanced-` (patrones avanzados). |
| **¿Para qué es?** | Optimizar el rendimiento de componentes React y páginas Next.js eliminando waterfalls, reduciendo bundle size, y minimizando re-renders innecesarios. |
| **¿Para qué se usa?** | Eliminar waterfalls (`Promise.all` para operaciones independientes, `Suspense` para streaming). Optimizar bundle (importar directo, evitar barrel files, `next/dynamic` para heavy components). Optimizar re-renders (`useMemo`, `useCallback`, `startTransition`, `useDeferredValue`). Server-side: `React.cache()` para deduplicación, `after()` para non-blocking. |
| **¿Cómo se usa?** | Consultando las reglas por categoría y prioridad. Cada regla tiene: explicación, código incorrecto, código correcto y contexto adicional. |
| **¿Cómo debería usarse?** | Aplicando las reglas CRITICAL primero (waterfalls y bundle size), luego HIGH (server-side), luego MEDIUM (re-renders y rendering). Las reglas LOW se aplican solo en optimización profunda. |
| **¿Cuándo debería usarse?** | **Fase 4** — Durante code review y optimización de rendimiento. También durante Fase 3 al escribir componentes nuevos. |
| **Dependencias** | Skill de proyecto (`.agent/skills/vercel-react-best-practices/`). No requiere instalación — es guía de 70 reglas de rendimiento React/Next.js de Vercel Engineering. |
| **Requerimientos** | Proyecto React 16.8+ o Next.js. Opcional: React DevTools, Lighthouse, Vercel Analytics. |

## 15. `vercel-optimize` — Auditoría de Costos y Performance Vercel

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Pipeline completo de auditoría de rendimiento y costos en Vercel. Requiere Vercel CLI v53+, proyecto linkeado, y opcionalmente Observability Plus. Soporta Next.js, SvelteKit, Nuxt, y Astro (limitado). Pipeline de 4 fases: collect → gate → investigate → report. |
| **¿Para qué es?** | Reducir la factura de Vercel, identificar rutas lentas o costosas, optimizar caching, Function Invocations, Build Minutes, Fast Data Transfer y Core Web Vitals. |
| **¿Para qué se usa?** | Ejecutar auditorías observability-first: recopilar métricas de producción → filtrar candidatos con gate determinístico → investigar solo rutas con evidencia métrica → generar reporte con recomendaciones verificadas y citadas. |
| **¿Cómo se usa?** | Ejecutando el pipeline de scripts: `collect-signals.mjs` → `scan-codebase.mjs` → `merge-signals.mjs` → `gate-investigations.mjs` → `deep-dive.mjs` → `reconcile-candidates.mjs` → `verify-and-regen.mjs` → `render-report.mjs`. |
| **¿Cómo debería usarse?** | Solo sobre proyectos desplegados en Vercel con tráfico real. Nunca grep repo-wide sin evidencia métrica. Cada recomendación debe trazar a un candidato y a métricas observadas. |
| **¿Cuándo debería usarse?** | **Post-producción** — Cuando hay factura alta, rutas lentas, o se necesita optimización de costos. |
| **Dependencias** | Skill de proyecto (`.agent/skills/vercel-optimize/`). Requiere Vercel CLI v53+ (`npm i -g vercel@latest`). Opcional: Vercel Observability Plus. |
| **Requerimientos** | Node 18+. Proyecto desplegado en Vercel con tráfico real. Acceso a Vercel dashboard. Vercel CLI autenticado (`vercel login`). |

## `roier-seo` — Auditoría Técnica SEO

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Auditor SEO técnico que corre Lighthouse/PageSpeed sobre sitios o servidores dev, analiza puntuaciones SEO/Performance/Accesibilidad e implementa correcciones automáticas. |
| **¿Para qué es?** | Detectar y corregir problemas SEO, Core Web Vitals, meta tags, structured data y accesibilidad antes del deploy. |
| **¿Para qué se usa?** | Ejecutar auditorías automatizadas, implementar fixes para meta tags faltantes, structured data (JSON-LD), lazy loading, contraste, etiquetas ARIA, y jerarquía de headings. |
| **¿Cómo se usa?** | El skill corre Lighthouse/PageSpeed, analiza resultados, y genera parches para los issues encontrados. Para auditoría continua, combinarlo con CI/CD. |
| **¿Cómo debería usarse?** | Como gate de calidad pre-deploy. Si el score baja de 90 en SEO o Performance, no se despliega. También como auditoría periódica en producción. |
| **¿Cuándo debería usarse?** | **Fase 4** — Antes del deploy. También **post-producción** para monitoreo periódico. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add davila7/claude-code-templates@roier-seo -g`). Requiere Lighthouse/PageSpeed (corre desde Node). |
| **Requerimientos** | Node 18+. El skill corre auditorías desde el CLI. Para auditorías en dev, el servidor local debe ser accesible. |

## `ai-seo` — Optimización para Buscadores AI

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill especializado en optimizar contenido para motores de búsqueda AI (ChatGPT Search, Perplexity, Google AI Overviews, Claude, Gemini). Cubre AEO (Answer Engine Optimization), LLMO, y GEO (Generative Engine Optimization). |
| **¿Para qué es?** | Conseguir que el contenido del proyecto aparezca en respuestas generadas por IA, sea citado por LLMs, y optimice para visibilidad en AI Overviews y búsqueda zero-click. |
| **¿Para qué se usa?** | Crear OKF (Open Knowledge Format), `llms.txt`, knowledge bundles, y structured data para agentes. Optimizar contenido para preguntas conversacionales y fragmentos destacados. |
| **¿Cómo se usa?** | Analizando el contenido existente, generando OKF/knowledge bundles, y aplicando técnicas de AI visibility. El skill incluye guías para structured data, entity recognition, y topical authority. |
| **¿Cómo debería usarse?** | Junto con `roier-seo` para cobertura completa: SEO técnico (roier) + AI visibility (ai-seo). El contenido optimizado para AI SEO también beneficia el ranking tradicional. |
| **¿Cuándo debería usarse?** | **Fase 4** — Cuando el proyecto necesita visibilidad en respuestas AI. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add coreyhaines31/marketingskills@ai-seo -g`). No requiere paquetes npm adicionales — es conocimiento y guías de contenido. |
| **Requerimientos** | Ninguno. Solo acceso al contenido del proyecto para analizarlo y reescribirlo. |

## `seo` — Optimización General para Buscadores

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de optimización SEO general: meta tags, sitemap optimization, structured data, canonical URLs, heading hierarchy, alt texts, y buenas prácticas de contenido. |
| **¿Para qué es?** | Mejorar el ranking en buscadores tradicionales (Google, Bing) mediante prácticas on-page y técnicas. |
| **¿Para qué se usa?** | Implementar meta tags (title, description, OG), sitemap.xml, robots.txt, canonical tags, heading structure (`h1`→`h6`), lazy loading de imágenes, y datos estructurados (Schema.org, JSON-LD). |
| **¿Cómo se usa?** | Auditando el sitio contra checklist SEO, implementando correcciones en meta tags, sitemaps, headings, y structured data. Verificando con Lighthouse y Google Search Console. |
| **¿Cómo debería usarse?** | Como base SEO del proyecto. `seo` cubre lo fundamental, `roier-seo` añade auditoría automatizada, y `ai-seo` cubre la visibilidad en búsqueda AI. Los tres son complementarios. |
| **¿Cuándo debería usarse?** | **Fase 4** — En todo proyecto web que requiera visibilidad en buscadores. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add addyosi@web-quality-skills@seo -g`). No requiere paquetes npm — es conocimiento de mejores prácticas SEO. |
| **Requerimientos** | Ninguno. Las verificaciones se hacen con Lighthouse (ya incluido en Chrome/Edge) y Google Search Console. |
