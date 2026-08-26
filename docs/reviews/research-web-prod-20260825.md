# INV-web-prod — Investigación profunda: producto web VantaDB

> **Fecha:** 2026-08-25 · **Comando:** `/research web` · **Plantilla:** `prompts/research-module-product.md`
> **Objeto:** `web/**` (Next.js 16: marketing + docs + playground WASM) · Modo read-only
> **Método:** análisis interno (codegraph, greps file:line, auditoría WDA 2026-08-24 <24h de antigüedad usada como evidencia visual reciente — decisión metodológica: no se duplicó captura playwright porque la auditoría verificó build exit 0 + 31 rutas 200 hace 1 día) + extracción en vivo de los 5 competidores vía Jina Reader (URLs citadas por sección).
> **Nota de evidencia:** las UX-* del Backlog son del módulo desktop, NO de web — no se duplican acá.

## 1. Usuarios objetivo y flujos críticos

Tres audiencias (registro `.opencode/references/research-modules.md` fila `web`):

| Audiencia | Flujo crítico esperado | Estado hoy |
|---|---|---|
| Dev evaluando VantaDB | landing → entender qué es → quickstart con código copiable → instalar | ✅ Cubierto: `/docs` contiene el quickstart; claims numéricos canónicos desde BENCHMARKS.md (WDA-03 eliminó 9 claims falsos). ⚠️ No hay ruta `/quickstart` dedicada (decisión WDA registrada); competidores ponen código instalable arriba del fold |
| Usuario del dashboard embebido del server | conectar al server → explorar namespaces → buscar | ❌ **El dashboard embebido NO existe en `web/src`** (grep `dashboard` = 0 matches en 31 rutas). La UI de exploración (DataExplorer, IngestForm, WorkspaceShell) vive en `desktop/`, no en web. El registro sobre-declara el alcance del módulo |
| Visitante del playground browser-only | abrir `/playground` → correr ejemplo put/search BM25+HNSW+RRF → ver latencias | ✅ Implementado como simulador (`code-playground.tsx`), metadata SEO completa (`playground/layout.tsx`). ⚠️ Ejecuta código vía `new Function` (self-XSS documentado, ver H-04) |

Fricciones detectadas: estados de carga/vacío sin diseñar en playground no documentados; metadata OG en español mezclada con títulos EN (H-02); 4 referencias a assets inexistentes con fallback silencioso (H-03).

## 2. Estándares del ecosistema (verificados contra fuentes oficiales)

- **Next.js 16** (App Router, standalone output, turbopack root — `web/next.config.ts`, verificado en WDA): el proyecto está en la versión mayor actual. Fuente: auditoría interna + `package.json` (`"next": "^16.1.1"`).
- **WCAG 2.2 AA**: contraste ≥4.5:1 texto normal, targets ≥24px (regla propia R-FE-5 va más allá: 44px táctil).
- **Core Web Vitals**: baseline Lighthouse medido en WDA-00: `/` Perf 96 · A11y 95 · BP 100 · SEO 100; `/about/team` 95/96/100/100 (`docs/reviews/web-design-audit-2026-08-24.md` §Lighthouse). **Claim válido pero pre-cambios** (ver H-06).
- **Tailwind v4 CSS-first**: tema en `globals.css` `@theme inline`; correcto según docs Tailwind v4.

## 3. Productos de referencia — matriz

Extracción en vivo 2026-08-25 (Jina Reader sobre homepages):

| Dimensión | **VantaDB web** | qdrant.tech | weaviate.io | lancedb.com | trychroma.com | milvus.io |
|---|---|---|---|---|---|---|
| Hero/messaging | Local-first explícito, audiencia definida (WDA-07) | "High-performance vector search at scale", features técnicas una por una | "Design, build and ship complete AI experiences" — plataforma | "Multimodal Lakehouse" — categoría nueva | "Open-source search infrastructure for AI" | "High-performance vector DB built for scale" + código en hero |
| Social proof | Showcase honesto sin métricas de adopción | SOC2/GDPR badges, casos | 20M+ downloads, logos Booking/Bosch/Cisco, case studies con números (9B vectores, 50K tenants) | Netflix/Uber/NVIDIA/ByteDance, $30M Series A, CrewAI 2B+ ejecuciones | Capital One, 27k stars, 15M downloads/mes | Comunidad meetups, quotes Medium |
| Onboarding | `/docs` con quickstart interno | Docs con Web UI dedicada (qdrant.tech/documentation/web-ui) | Cloud Console "Try now" en nav + quickstart | docs.lancedb.com/quickstart CTA primario | `pip install chromadb` en footer + "$5 free credits" | Código Python EN EL HERO + 3 deployment options |
| Playground/demo interactivo | ✅ `/playground` simulador híbrido (diferenciador real) | Qdrant Web UI self-hosted (collections, REST console, tSNE visualize) | playground.weaviate.io dedicado | Lance Data Viewer (open source) | Agent Search demo interactivo en home | demos.milvus.io (hybrid, multimodal, graph RAG) |
| Estados error/vacío | not-found estilizado propio; resto sin evidencia de problemas | n/a (no evaluado a fondo) | — | — | — | — |
| Pricing | $0 / Custom únicamente | Cloud free tier + pricing | Free tier + usage-based + enterprise | Enterprise + OSS | Free credits + Pro + Enterprise | Zilliz Cloud serverless + BYOC |
| Performance percibida | Lighthouse 96 (medido, WDA-00) | claim sin evidencia en home | claim sin evidencia | benchmarks propios publicados con números (RaBitQ 96% recall, 100K+ QPS — con fuente reproducible en sus benches) | Tabla de latencia p50/p90/p99 warm/cold PUBLICADA en home | "Blazing fast" sin número (claim sin evidencia) |
| Qué copiarían abiertamente | — | Web UI auto-hospedada como producto documentado | Case studies con métricas duras por vertical | Benchmarks publicados como contenido de marketing técnico | Latencias p99 publicadas en home (honestidad técnica) | Código instalable arriba del fold |

## 4. Estado actual de `web/`

- **Inventario:** 31 rutas page.tsx (+ redirect docs-api) ≈ 32 URLs en sitemap (36 tras WDA-02). Componentes en `src/components/vanta/`; design system brutalist propio light-only (R-FE-4); animejs solo en mark components con cleanup + prefers-reduced-motion (WDA-01).
- **Calidad de flujos:** 0 tests E2E en `web/` (desktop tiene 2 specs Playwright). Verificación de calidad = build exit 0 + lint exit 0 + tsc exit 0 + 31 rutas 200 (WDA final).
- **Auditorías previas:** `docs/reviews/web-design-audit-2026-08-24.md` (WDA-00..07 completos: dark-mode huérfano eliminado, −7.615 líneas dead code, 9 claims falsos→0, contrast ×24→0 errores dirigidos, i18n migrado, funnel comercial honesto). Esta investigación NO redescubre lo ya arreglado.
- **Deudas conocidas referenciadas:** assets faltantes (4 refs), densidad efectos home (73 usos), pricing mid-tier, dominio vercel.app, playground `new Function` — todas ya listadas como "Pendiente" en la auditoría WDA; acá se formalizan como hallazgos con ID.
- **i18n:** ~1.2k claves ES/EN simétricas en `dictionaries.ts` (2.939 líneas); helper `createTt`. Gap: metadata de layouts (title EN + description ES mezclados).
- **Seguridad:** sin CSP documentada para el sitio público (no verificado — flag abierto); `new Function` en playground es riesgo latente documentado inline (`ponytail:` comment).

## 5. Framework de evaluación (score 0-10)

| Dimensión | Score | Justificación |
|---|---|---|
| First impression & messaging | 7.5 | Hero honesto con audiencia explícita post-WDA-07; pierde contra heros con código instalable (milvus) o categoría memorable (lancedb "Lakehouse") |
| Flujos core sin fricción | 6.5 | Landing→docs funciona; dashboard embebido prometido por el registro NO existe (H-01); quickstart no tiene ruta/ancla dedicada |
| Accesibilidad | 7 | Errores dirigidos a 0 (WDA-04); sin re-auditoría axe completa posterior ni teclado/SR sistemático documentado |
| Performance | 8 | Baseline medido 96/95 (fuente citada); bundle reducido después; falta re-medición (H-06) |
| i18n | 7.5 | Diccionario simétrico robusto; metadata de layouts inconsistente (H-02) |
| Consistencia design system | 7 | Brutalist propio coherente light-only; densidad decorativa home sin decidir (H-07) |
| Robustez | 7 | not-found propio; assets faltantes con fallbacks silenciosos (H-03) |
| Seguridad | 6.5 | `new Function` sin sandbox (documentado); CSP sin verificar |
| Testabilidad | 4 | Cero E2E en web — único módulo de superficie sin guard de regresión (H-05) |
| **Diferenciación vs qdrant.tech** | 8 | Único con playground híbrido BM25+HNSW+RRF corriendo en-browser sin server + narrativa local-first/embebida que qdrant (server/cloud) no puede reclamar honestamente |

### **SCORE GLOBAL: 7.2 / 10**

## Gap analysis priorizado

**Falta (P0/P1):** dashboard embebido declarado pero ausente (P1 — decidir alcance); E2E web (P1); re-medición Lighthouse (P2 barato).
**Mejorable:** metadata i18n; assets faltantes; sandbox playground; densidad efectos; quickstart más prominente.
**Optimizable:** social proof con métricas propias cuando existan; benchmarks publicados estilo chroma/lancedb (ya tenemos BENCHMARKS.md — convertirlo en página de marketing con tablas).

**Quick wins (<1 día):** H-02, H-03, H-06, H-11.
**Apuestas estratégicas (>1 semana):** H-01, H-05 (parcial), H-07, H-08, H-09, H-10.

## Apéndice de hallazgos H-NN (entrada Fase D)

| ID | Hallazgo | Categoría sugerida | Severidad | Esfuerzo | Evidencia |
|---|---|---|---|---|---|
| H-01 | Dashboard embebido del server declarado en el registro pero inexistente en `web/` (la UI de exploración vive en `desktop/`) — decidir: construir versión web, corregir el registro, o documentar que desktop es la única GUI | ESTRATEGIA | Media | 🔴 | `rg -il dashboard web/src` = 0 matches; registro fila `web` |
| H-02 | Metadata de layouts mezcla título EN con descripción/openGraph en español — un visitante EN comparte cards en ES; migrar a tt() o claves duales | MEJORAR | Baja | 🟢 | `web/src/app/playground/layout.tsx:4-10`, `about/community/layout.tsx:4-10`, `about/company/layout.tsx:4-11` (mismo patrón en about/*) |
| H-03 | 4 referencias a `mascota_gato.png`/`avatar_gato.png` inexistentes en `public/assets/` con fallbacks silenciosos — restaurar assets o eliminar refs muertas | AGREGAR | Baja | 🟢 | `web/src/components/vanta/easter-egg.tsx:78`, `opengraph-image.tsx:15`, `vanta-data.ts:1062,1069` |
| H-04 | Playground ejecuta código con `new Function` (self-XSS documentado inline); sandbox iframe solo si se expone a terceros — decidir política antes de cualquier exposición pública del playground | OPTIMIZAR | Media | 🟡 | `web/src/components/vanta/code-playground.tsx:318-323` |
| H-05 | Cero tests E2E en `web/` (desktop tiene 2 specs) — el flujo crítico landing→docs→playground no tiene guard de regresión | AGREGAR | Media | 🟡 | glob `web/e2e/**` vacío; `docs/reviews/web-design-audit-2026-08-24.md` §Verificación (solo build/lint/rutas) |
| H-06 | Lighthouse no re-medido post-WDA-05 (−7.615 líneas, lazy command-palette): el claim "perf 95-96" del registro queda sin medición fresca — Regla 11 exige re-medir o actualizar el claim | OPTIMIZAR | Baja | 🟢 | `web-design-audit-2026-08-24.md` §Lighthouse ("no re-medido, EPERM ambiental") |
| H-07 | Densidad de efectos decorativos en home: 73 usos, trust-bar ×11 efectos, hero 5 capas — requiere criterio visual del owner (decisión de diseño fino pendiente desde WDA) | MEJORAR | Baja-Media | 🟡 | `web-design-audit-2026-08-24.md` §Pendiente F1; `web/AGENTS.md` conocido-pendiente |
| H-08 | Pricing $0/Custom únicamente — los 5 competidores publican free tier + pricing intermedio; un mid-tier (o pricing transparente "por qué no") reduce fricción de evaluación enterprise | ESTRATEGIA | Media | 🟡 | `web/src/app/pricing/page.tsx`; comparativa §3 |
| H-09 | Dominio canónico `vantadb.vercel.app` (SITE_URL única fuente, correcto técnicamente) — comprar/apuntar dominio propio es señal de seriedad pre-launch (relacionado FIND-17 identidad de marca) | ESTRATEGIA | Media-Baja | 🟢 | `web/src/lib/site-config.ts`; WDA §Pendiente |
| H-10 | Social proof sin métricas de adopción comparables (competidores: 20M downloads, logos Fortune500, case studies con números) — mientras no haya adopción real, alternativa honesta: publicar benchmarks propios con fuente reproducible (estilo chroma: tabla p50/p99 en home) apalancando BENCHMARKS.md existente | ESTRATEGIA | Media | 🟡 | Comparativa §3 (fuentes: weaviate.io, trychroma.com, lancedb.com extraídos 2026-08-25); Regla 11 permite citar BENCHMARKS.md |
| H-11 | Quickstart sin ruta/ancla dedicada — competidores ponen comando instalable arriba del fold; evaluar ancla `#quickstart` en `/docs` o bloque de instalación en home (NO recrear ruta `/quickstart`: decisión WDA cerrada) | MEJORAR | Baja | 🟢 | `web-design-audit-2026-08-24.md` corrección 1; comparativa §3 (milvus hero-code, chroma footer-install) |

**Claims de performance citados con fuente:** Lighthouse 96/95/100/100 (`docs/reviews/web-design-audit-2026-08-24.md`, WDA-00 baseline). Ningún otro número de performance se afirma sin medición (los claims de competidores se marcan como "claim sin evidencia" donde corresponde).

## Recomendaciones

1. Quick wins primero (H-02/H-03/H-06/H-11): un plan wave única, listo para `/pipeline run`.
2. H-01 requiere decisión de producto ANTES de cualquier implementación (¿dashboard web o desktop-only?).
3. H-10 es la palanca de marketing de mayor ROI honesto: ya existe `docs/operations/BENCHMARKS.md` — convertirlo en `/benchmarks` con tablas citadas (la ruta existe: `web/src/app/benchmarks/page.tsx` — verificar que muestre los datos actuales).
