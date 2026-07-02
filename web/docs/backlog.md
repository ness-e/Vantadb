# VantaDB — Backlog Completo

> **Versión**: 1.0 — 2026-07-02
> **Propósito**: Único source of truth de todo el trabajo pendiente en la web (`vantadb.github.io/`)
> y el producto (`VantaDB/`). Refleja decisiones del dueño (Grupos S, A–F) y lo ejecutado
> en Fase S + Fase 1.
>
> Documentos fuente:
> - `docs/ANALISIS_COMPLETO_Y_DECISIONES.md` (v4.0) — análisis web + decisiones
> - `VantaDB/docs/REPORTE_INVESTIGACION_Y_DECISIONES.md` (v2.0) — investigaciones profundas

---

## Índice

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Decisiones Vinculantes](#2-decisiones-vinculantes)
3. [✅ Fase S: Corrección de Veracidad (COMPLETADA)](#3-fase-s-corrección-de-veracidad)
4. [✅ Fase 1: Quick Wins (COMPLETADA)](#4-fase-1-quick-wins)
5. [⏳ Fase 2: Contenido Real (PENDIENTE)](#5-fase-2-contenido-real)
6. [⏳ Fase 3: Diseño Visual (PENDIENTE)](#6-fase-3-diseño-visual)
7. [⏳ Fase 4: Calidad y Proceso (PENDIENTE)](#7-fase-4-calidad-y-proceso)
8. [⏳ Fase 5: Polish y SEO (PENDIENTE)](#8-fase-5-polish-y-seo)
9. [⏳ Producto Real — Correcciones (PENDIENTE)](#9-producto-real-correcciones)
10. [⏳ Features Evaluadas y Diferidas](#10-features-evaluadas-y-diferidas)
11. [Apéndice: Estado de Cada Ruta](#11-apéndice-estado-de-cada-ruta)
12. [Apéndice: Deuda Técnica Detallada](#12-apéndice-deuda-técnica-detallada)

---

## 1. Resumen Ejecutivo

### Estado General

| Dimensión | Progreso |
|-----------|----------|
| **Veracidad web vs producto real** | ~25% → **~95%** (34 discrepancias → ~2 remanentes menores) |
| **Dependencias fantasma eliminadas** | 5 dependencias npm |
| **Assets legacy eliminados** | ~55 MB (HDR, PBR, models, images legacy) |
| **CSS legacy** | 0/26 huérfanos (pero hay 8 con naming conflict) |
| **Tests** | 0% — sin configurar |
| **CI/CD** | 0% — sin automatización |
| **TypeScript strict** | 0% — `noUnusedLocals/Parameters: false` |
| **Estilos inline** | 637 ocurrencias — 0 migradas a CSS modules |
| **Fases completadas** | Fase S + Fase 1 = **2/7** |

### Prioridades Restantes (en orden)

```
1. Fase 2 — Contenido Real  (alta — credibilidad del sitio)
2. Fase 3 — Diseño Visual   (alta — primera impresión)
3. Fase 4 — Calidad         (media — sostenibilidad)
4. Producto — Correcciones  (media — alineación)
5. Fase 5 — Polish          (baja — perfeccionamiento)
6. Features diferidas       (baja — no tocar aún)
```

---

## 2. Decisiones Vinculantes

Todas las decisiones del dueño (Julio 2026) — cualquier desviación requiere aprobación.

### Grupo S — Veracidad

| Decisión | Elección | Implicación |
|----------|----------|-------------|
| **S1. Discrepancias** | Corrección total | No mantener claims aspiracionales. La web debe reflejar el producto real. |
| **S2. Benchmarks** | Datos reales diferenciados | Rust Core (1.2ms p50, 0.998 recall) !== Python SDK (39.74ms p50, 24.5% recall). Mostrar ambos con etiquetas claras. |
| **S3. Pricing** | Open Core $0 + Enterprise futuro | Mostrar "Open Source (Apache 2.0) — $0 forever. Enterprise features en desarrollo." Sin tiers ni precios. |
| **S4. SQL Engine** | Eliminar completamente | No existe, no planeado hasta 2027. No mencionar bajo ninguna circunstancia. |
| **S5. Versión** | Reemplazar con datos reales | v0.1.0→v0.2.0 (bump aprobado). Changelog con releases reales. |
| **S6. API snippets** | Reescribir TODOS | API real: `import vantadb_py as vantadb`, `VantaDB()`, `put()`, `search_memory()`. |
| **S7. Página /docs** | Design guide + /docs-api aparte | No mezclar. Design guide en /docs. Docs técnicas en /docs-api. |
| **S8. Licencia** | Apache 2.0 + Enterprise crate | Core Apache 2.0. Enterprise features en crate propietario separado. |
| **S9. Android/iOS** | Eliminar | Sin evidencia, sin planes. No mencionar. |

### Grupo A — Arquitectura

| Decisión | Elección | Implicación |
|----------|----------|-------------|
| **A1. Dominio** | `vantadb.vercel.app` único | Unificar canonical URL y OG images. Decidir si migrar a `vantadb.dev`. |
| **A2. Three.js** | Eliminado | Ya no está en package.json ni en src/. No reinstalar. |
| **A3. Animaciones** | GSAP + motion.dev | animejs solo para TextScramble (reemplazar si hay alternativa). |
| **A4. Testing** | Playwright E2E + Vitest | Configurar ambos frameworks. |
| **A5. CI/CD** | GitHub Actions | lint + typecheck + build en cada push. Deploy automático a Vercel. |
| **A6. CSS Strategy** | CSS Modules | Migrar 637 estilos inline a CSS modules. No usar Tailwind classes para estilos de componentes. |

### Grupo B — Diseño Visual

| Decisión | Elección |
|----------|----------|
| **B1. Hero** | Diseño nuevo animado, grilla, terminal effect |
| **B2. Nav background** | Mantener claro/blando actual (sin cambios) |
| **B3. Estadísticas** | Mover a SwissBenchmarkGrid con datos reales |
| **B4. Subpáginas** | Expandir con datos reales del producto |
| **B5. /docs** | Design guide en /docs + crear /docs-api |
| **B6. Animaciones GSAP** | Implementar todas (ScrollTrigger, count-up, typewriter, stroke-dashoffset) |
| **B7. Tema** | No implementar toggle. Solo warm paper. |

### Grupo C — Contenido

| Decisión | Elección |
|----------|----------|
| **C1. Blog** | Publicar 3 artículos reales + generar más |
| **C2. Páginas faltantes** | Crear `/product/benchmarks`, `/security`, `/roadmap` |
| **C3. Navegación** | Rediseñar con dropdowns/submenús |
| **C4. /product/** | Crear página de producto real |

### Grupo D — Rendimiento

| Decisión | Elección |
|----------|----------|
| **D1. Assets** | Optimizar todo (WebP/AVIF) |
| **D2. Texturas (40MB)** | Eliminadas |
| **D3. Dependencias fantasma** | Eliminadas |

### Grupo E — Calidad

| Decisión | Elección |
|----------|----------|
| **E1. Estilos inline** | Migrar a CSS Modules |
| **E2. TypeScript strict** | Activar `noUnusedLocals` y `noUnusedParameters` |
| **E3. console.error** | Reemplazar con logging service |
| **E4. gsap.registerPlugin** | Centralizado en `src/lib/gsap.ts` |

### Grupo F — Integración con Producto Real

| Decisión | Elección |
|----------|----------|
| **F1. Benchmarks** | Usar benchmarks REALES diferenciados Rust/Python |
| **F2. Docs técnicas** | Crear docs desde cero (no sincronizar automáticamente) |
| **F3. Repo** | Actualizar a `ness-e/Vantadb` |
| **F4. API Docs** | Extraer API real de `VantaDB/docs/api/` |
| **F5. Naming histórico** | Ignorar (legado interno del producto) |

---

## 3. ✅ Fase S: Corrección de Veracidad (COMPLETADA)

> Ejecutada 2026-07-02. 34 discrepancias reducidas a ~2 menores.

| # | Acción | Archivos | Estado |
|---|--------|----------|--------|
| S1 | Licencia MIT → Apache 2.0 | `__root.tsx`, `index.tsx`, `company.tsx`, `community.tsx`, `contact.tsx`, `pricing.tsx`, footer, `public/llms.txt`, `public/og/default.svg` | ✅ |
| S2 | Changelog reescrito con versiones reales | `changelog.tsx` (v0.4.0–0.6.0 → v0.1.1–v0.2.0) | ✅ |
| S3 | SQL references eliminadas | `company.tsx`, `storage.tsx`, `SwissQuickstart.tsx`, `SwissHero.tsx`, `index.tsx`, `llms.txt` | ✅ |
| S4 | API snippets corregidos | `SwissQuickstart.tsx`, `docs.tsx`, `config.tsx`, `ai-agents.tsx`, `local-rag.tsx`, `integrations.tsx` | ✅ |
| S5 | Pricing tiers eliminados | `pricing.tsx` ($0 + Enterprise, sin precios ni límites) | ✅ |
| S6 | Versión CLI corregida | `docs.tsx` (0.6.0 → 0.2.0) | ✅ |
| S7 | Migration guide corregida | `docs.tsx` (v0.5→v0.6 → v0.1.x→v0.2.0) | ✅ |
| S8 | Android/iOS eliminados | `__root.tsx`, `changelog.tsx` | ✅ |
| S9 | Repo URLs corregidas | `community.tsx`, `contact.tsx`, `SwissFooter.tsx`, `Nav.tsx` | ✅ |
| S10 | `pip install` corregido | `SwissQuickstart.tsx`, `SwissHero.tsx`, `SwissMonolith.tsx` | ✅ |
| S11 | LangChain nombre corregido | `integrations.tsx` (`langchain_vantadb` → `vantadb-langchain`) | ✅ |
| S12 | Benchmark claims corregidos | `SwissBenchmarkGrid.tsx` (0.8ms p99 → 1.2ms p50), `engine.tsx` (100% recall → 0.998) | ✅ |
| S13 | "Sub-millisecond" corregido | `company.tsx` (→ "1.2ms p50") | ✅ |
| S14 | Distinción Rust Core vs Python SDK | `latency.tsx` (toggle interactivo, notas, datos reales) | ✅ |
| S15 | `security` type agregado | `changelog.tsx` (TYPE_CONFIG + ALL_TYPES) | ✅ |

### Remanentes Menores (no críticos)

| # | Detalle | Archivo | Nota |
|---|---------|---------|------|
| R1 | `engine.tsx` RRF slider usa fórmulas no basadas en benchmarks reales | `engine.tsx:122-124` | Es un demo interactivo, no claim factual. Aceptable. |
| R2 | `latency.tsx` comparativa 167x vs Pinecone no es benchmark real | `latency.tsx` | Es simulador teórico. Tiene disclaimer con toggle. Aceptable. |

---

## 4. ✅ Fase 1: Quick Wins (COMPLETADA)

| # | Acción | Detalle | Estado |
|---|--------|---------|--------|
| 1.1 | Three.js + dependencias fantasma | `three`, `@react-three/fiber`, `stats.js`, `tweakpane`, `@types/three` eliminados | ✅ |
| 1.2 | Alias @experience | Eliminado de `vite.config.ts`, `tsconfig.json`, `eslint.config.js`, `.prettierignore` | ✅ |
| 1.3 | Texturas/models legacy | `public/textures/` (HDR 34MB, PBR 6MB), `public/models/`, `public/images/`, `public/basis/`, `public/draco/` (~55 MB eliminados) | ✅ |
| 1.4 | GSAP centralizado | `src/lib/gsap.ts` creado. 9 componentes actualizados | ✅ |
| 1.5 | console.error reemplazado | `src/routes/__root.tsx` | ✅ |
| 1.6 | Dependencias npm no usadas | `gray-matter`, `normalize-wheel`, `@playwright/test`, `eslint-config-prettier` eliminadas | ✅ |
| 1.7 | `--border-hover` undefined | Definido en `tokens.css` como `var(--foreground)` | ✅ |
| 1.8 | `--text-label`/`--text-code` undefined | Definidos: `0.72rem`, `0.82rem` | ✅ |
| 1.9 | `fontsize` typo en CSS | `swiss-ecosystem.css:14` corregido | ✅ |
| 1.10 | `--crimson` → `--danger` | Consolidado en `tokens.css` | ✅ |
| 1.11 | `src/SourceDesign/` añadido a `.gitignore` | 34 archivos (3.24 MB) de referencia de diseño | ✅ |
| 1.12 | `motion` re-instalado | Se usaba via `motion/react` en `__root.tsx` (AnimatePresence). No era fantasma. | ✅ |

---

## 5. ⏳ Fase 2: Contenido Real (PENDIENTE)

> **Prioridad**: 🔴 Alta — sin contenido real, el sitio carece de credibilidad técnica.
> **Esfuerzo estimado**: 3-5 días
> **Dependencias**: Ninguna (paralelizable con Fase 3)

### 5.1 Publicar 3 Artículos Técnicos del Producto

Los artículos existen en `VantaDB/docs/articles/`. Solo falta copiarlos a `content/blog/`
con frontmatter YAML completo.

| Artículo | Archivo fuente | Slug | Palabras | Prioridad |
|----------|---------------|------|----------|-----------|
| Why I Built a Local Memory Engine | `VantaDB/docs/articles/why_i_built_local_memory_engine.md` | `why-i-built-a-local-memory-engine` | ~2,300 | 🔴 Alta |
| SQLite for AI Agents | `VantaDB/docs/articles/sqlite_for_ai_agents.md` | `sqlite-for-ai-agents-benchmarks` | ~2,600 | 🔴 Alta |
| How Hybrid Search Works | `VantaDB/docs/articles/how_hybrid_search_works.md` | `how-hybrid-search-works-bm25-hnsw-rrf` | ~3,500 | 🟡 Media |

**Tareas específicas**:
- [ ] Leer cada artículo y agregar frontmatter faltante (`date`, `description`, `tags`, `author`)
- [ ] Copiar a `content/blog/` con slug correcto
- [ ] Verificar que `marked` renderiza correctamente (code blocks, imágenes, tablas)
- [ ] Agregar syntax highlighting (`marked-highlight` + `highlight.js`)
- [ ] Verificar enlaces internos (no apunten a rutas del vault Obsidian)
- [ ] Actualizar `src/routes/blog/index.tsx` para listar posts
- [ ] Agregar meta tags OG por artículo

**Implicaciones**:
- El blog usa `import.meta.glob` (Vite) — agregar archivos a `content/blog/` los incluye automáticamente
- El parser custom de frontmatter en `src/lib/blog.ts` funciona pero podría reemplazarse por `gray-matter`
  (si se reinstala) o mantenerse el regex actual
- Decap CMS (`public/admin/`) está configurado pero desatendido — decidir si mantenerlo

### 5.2 Crear Página /product/benchmarks

**Decisión C2**: Página de alta prioridad con benchmarks reales.

**Fuente de datos**: `VantaDB/docs/operations/BENCHMARKS.md`

**Datos a incluir**:

**Rust Core** (nativo, 10K–100K vectores 128d):
| Métrica | 10K | 50K | 100K |
|---------|-----|-----|------|
| Recall@10 | 0.956 | 0.998 | — |
| p50 Latency | 1.2ms | 6.1ms | — |
| Memory/vector | ~1172 bytes | — | — |

**Python SDK** (via PyO3, datos reales):
| Operación | p50 | p99 | QPS |
|-----------|-----|-----|-----|
| PUT | 10.7ms | — | 95 ops/sec |
| BM25 search | 115.3ms | — | 9 qps |
| HNSW search | 62.0ms | — | 16 qps |
| Hybrid search | 179.8ms | 211ms | 6 qps |
| Index rebuild (10K) | 93.5s | — | — |

**Competitivo** (glove-100-angular, 10K records):
| Engine | p50 | p99 | Recall@10 | QPS | RSS |
|--------|-----|-----|-----------|-----|-----|
| VantaDB | 39.74ms | 58.2ms | 24.50% | 24.3 | 236.5 MB |
| LanceDB | 2.65ms | 6.98ms | 13.90% | 320.5 | 344.2 MB |
| ChromaDB | 0.94ms | 3.35ms | 24.10% | 978.6 | 253.5 MB |

**Tareas**:
- [ ] Crear `src/routes/product/benchmarks.tsx`
- [ ] Agregar ruta en `src/router.tsx`
- [ ] Usar datos reales diferenciados Rust/Python
- [ ] Incluir gráfica o tabla comparativa competitiva
- [ ] Agregar nota: "Benchmarks con v0.1.5. Se ejecutarán nuevos benchmarks con v0.2.0."

### 5.3 Crear Página /security

**Decisión C2**: Importante para credibilidad enterprise.

**Contenido sugerido**:
- Disclosure policy (email `security@vantadb.dev`)
- Responsible disclosure process
- Known security features (Apache 2.0 patent grant, WAL CRC32C, etc.)
- Link to SECURITY.md in repo

**Tareas**:
- [ ] Crear `src/routes/security.tsx`
- [ ] Agregar ruta
- [ ] Enlazar desde footer

### 5.4 Crear Página /about/roadmap

**Decisión C2**: Roadmap real del producto.

**Fuente**: `VantaDB/docs/strategy/ROADMAP.md`

**Contenido**:
- Timeline post-pivote (v0.2.0 → v2.0)
- Features planificadas vs diferidas
- Enlace al repo para feedback

**Tareas**:
- [ ] Crear `src/routes/about/roadmap.tsx`
- [ ] Agregar ruta
- [ ] Extraer hitos de ROADMAP.md

### 5.5 Crear /docs-api con Documentación Técnica Real

**Decisión B5/S7**: Design guide se queda en /docs. Docs técnicas en /docs-api.

**Fuente**: `VantaDB/docs/api/` (EMBEDDED_SDK.md, PYTHON_SDK.md, HTTP_API.md, MCP.md)

**Tareas**:
- [ ] Crear `src/routes/docs-api.tsx` (o ruta anidada)
- [ ] Extraer API docs reales
- [ ] Incluir código ejecutable válido
- [ ] NO incluir design system guide (eso queda en /docs)

### 5.6 Rediseñar Navegación con Dropdowns

**Decisión C3**: Nav actual tiene 4 links sin submenús.

**Estado actual**: `src/components/Nav.tsx` — links planos a Home, Engine, Docs, Changelog.

**Propuesta**:
- Solutions (dropdown): AI Agents, IDE Tooling, Local RAG
- Product (dropdown): Engine, Architecture, Benchmarks, Pricing
- Resources (dropdown): Docs, API, Blog, Changelog
- About (dropdown): Company, Community, Contact, Roadmap

**Tareas**:
- [ ] Rediseñar Nav.tsx con dropdowns
- [ ] Usar CSS modules o mantener estilos inline con refactor posterior
- [ ] Asegurar responsive (hamburger menu en mobile)

### 5.7 Crear Página /product/

**Decisión C4**: Directorio `/product/` vacío actualmente.

**Contenido sugerido**:
- Descripción del producto real (embedded memory engine)
- Key features (HNSW, BM25, RRF, WAL, backends)
- Quickstart embeddable
- SDKs (Rust, Python, WASM, MCP)
- Enlaces a benchmarks, pricing, docs

**Tareas**:
- [ ] Crear `src/routes/product/index.tsx`
- [ ] Agregar ruta y enlaces en Nav

---

## 6. ⏳ Fase 3: Diseño Visual (PENDIENTE)

> **Prioridad**: 🔴 Alta — el Hero y animaciones son la primera impresión del sitio.
> **Esfuerzo estimado**: 2-4 días
> **Dependencias**: Ideal después de Fase 2 (contenido), pero paralelizable.

### 6.1 Rediseñar Hero

**Decisión B1**: Hero actual tiene estadísticas (1.2ms, 0.998 recall), H1 weight 800, center alignment.
El diseño aprobado (DiseñoNuevo.md) especifica:

- **100% tipográfico** — sin estadísticas, sin botón CTA grande
- **Labels técnicas**: `[RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS]`
- **H1 weight 700** (no 800)
- **Left-aligned** (no center)
- **Grilla asimétrica** de fondo como elemento visual
- **Efecto terminal** opcional

**Tareas**:
- [ ] Leer DiseñoNuevo.md para especificación completa
- [ ] Rediseñar `src/routes/index.tsx` Hero section
- [ ] Implementar grilla asimétrica de fondo
- [ ] Agregar labels técnicas
- [ ] Mover estadísticas a SwissBenchmarkGrid
- [ ] Asegurar responsive mobile

### 6.2 Implementar Animaciones GSAP Faltantes

**Decisión B6**: Actualmente solo hay animaciones básicas. Faltan:

| Animación | Técnica | Ubicación | Prioridad |
|-----------|---------|-----------|-----------|
| ScrollTrigger reveals | GSAP + ScrollTrigger | Todas las secciones | 🔴 Alta |
| Count-up numbers | GSAP `gsap.to()` | Estadísticas reales en SwissBenchmarkGrid | 🟡 Media |
| Typewriter effect | GSAP SplitText o custom | Hero subtitle | 🟢 Baja |
| Stroke-dashoffset | GSAP + SVG | VantaDBLogo animación de entrada | 🟢 Baja |
| Route transitions | motion `AnimatePresence` | `__root.tsx` (ya implementado básico) | 🟢 Baja |

**Tareas**:
- [ ] Agregar ScrollTrigger a secciones principales (index, engine, architecture)
- [ ] Implementar count-up para números reales (no estadísticas infladas)
- [ ] Verificar prefers-reduced-motion en todas las animaciones
- [ ] Asegurar que animaciones no bloquean LCP

### 6.3 Revisar Diseño Responsive

**Decisión B4**: Subpáginas actuales parecen "otro sitio" vs landing.

**Problemas conocidos**:
- Subpáginas sin grid consistente
- Algunas tienen padding diferente
- Nav se ve diferente en mobile vs desktop

**Tareas**:
- [ ] Auditar todas las subpáginas para consistencia visual
- [ ] Verificar Swiss grid en todas las rutas
- [ ] Probar en mobile (390×844) y tablet (768×1024)
- [ ] Usar Playwright MCP para screenshots comparativos
- [ ] Corregir padding, gap, y tipografía inconsistente

### 6.4 Corregir Nav Background (Si Aplica)

**Decisión B2**: Nav se mantiene claro/blando actual. Solo si se decide cambiar:
- Nav background actual: `rgba(10,10,10,0.85)` (negro)
- DiseñoNuevo.md especifica: `rgba(249,248,246,0.85)` (warm paper)

---

## 7. ⏳ Fase 4: Calidad y Proceso (PENDIENTE)

> **Prioridad**: 🟡 Media — el sitio funciona, pero no es sostenible sin tests/CI.
> **Esfuerzo estimado**: 3-5 días
> **Dependencias**: Ninguna. Paralelizable con Fase 2 y 3.

### 7.1 Migrar 637 Estilos Inline a CSS Modules

**Decisión E1/A6**: Decisión confirmada: CSS Modules.

**Archivos con más estilos inline** (prioridad de migración):

| Archivo | Count | Prioridad |
|---------|-------|-----------|
| `engine.tsx` | 99 | 🔴 Alta |
| `latency.tsx` | 42 | 🔴 Alta |
| `storage.tsx` | 39 | 🔴 Alta |
| `architecture.tsx` | 37 | 🔴 Alta |
| `maint.tsx` | 30 | 🟡 Media |
| `pricing.tsx` | 26 | 🟡 Media |
| `SwissQuickstart.tsx` | 26 | 🟡 Media |
| `cost.tsx` | 25 | 🟡 Media |
| `config.tsx` | 22 | 🟡 Media |
| `company.tsx` | 20 | 🟡 Media |
| `local-rag.tsx` | 20 | 🟡 Media |
| `ai-ide-tooling.tsx` | 20 | 🟡 Media |
| `ai-agents.tsx` | 20 | 🟡 Media |
| `SwissFooter.tsx` | 18 | 🟢 Baja |
| `changelog.tsx` | 17 | 🟢 Baja |
| `use-cases.tsx` | 16 | 🟢 Baja |
| Otros (16 archivos) | <16 c/u | 🟢 Baja |

**Patrones repetidos a consolidar**:
```css
fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600,
letterSpacing: "0.14em", color: "var(--steel)", textTransform: "uppercase",
color: "var(--amber)", color: "var(--block-dark-muted)", color: "var(--muted)",
fontFamily: "var(--font-display)", fontFamily: "var(--font-sans)"
```

**Tareas**:
- [ ] Investigar estructura de CSS Modules con Vite (soporte nativo: `*.module.css`)
- [ ] Crear archivos `.module.css` por componente/ruta
- [ ] Migrar patrones repetidos a clases CSS reutilizables
- [ ] Migrar engine.tsx primero (99 ocurrencias)
- [ ] Migrar latency.tsx + storage.tsx segundo (42+39)
- [ ] Continuar por orden de prioridad

### 7.2 Activar TypeScript Strict

**Decisión E2**: Activar `noUnusedLocals` y `noUnusedParameters`.

**Estado actual** en `tsconfig.json`:
```json
"noUnusedLocals": false,
"noUnusedParameters": false,
```

**Tareas**:
- [ ] Cambiar ambos a `true`
- [ ] Ejecutar `npx tsc --noEmit`
- [ ] Corregir todos los errores de variables/parámetros no usados
- [ ] Documentar excepciones necesarias con `// @ts-expect-error`

### 7.3 Configurar Vitest

**Decisión A4**: Playwright E2E + Vitest unitario.

**Tareas**:
- [ ] Instalar `vitest`, `@testing-library/react`, `jsdom`
- [ ] Crear `vitest.config.ts`
- [ ] Crear `src/test/setup.ts`
- [ ] Escribir tests unitarios básicos:
  - [ ] Componentes puros (VantaDBLogo, SwissBackToTop, SwissSubpageHero)
  - [ ] Hooks (useTextScramble, useScrollReveal)
  - [ ] Utils (cn, blog parser)
- [ ] Verificar cobertura mínima >20%

### 7.4 Configurar Playwright E2E

**Decisión A4**: Playwright ya instalado, sin configuración de test.

**Tareas**:
- [ ] Crear `playwright.config.ts`
- [ ] Escribir tests E2E básicos:
  - [ ] Navegación funciona (click en cada link del Nav)
  - [ ] Landing page carga sin errores
  - [ ] Cada ruta carga su contenido principal
  - [ ] Changelog filtros funcionan
  - [ ] Latency toggle funciona
- [ ] Agregar script `"test:e2e"` a package.json

### 7.5 Configurar GitHub Actions CI/CD

**Decisión A5**: lint + typecheck + build en cada push. Deploy automático a Vercel.

**Workflow sugerido** (`.github/workflows/ci.yml`):

```yaml
name: CI
on: [push, pull_request]
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci
      - run: npx tsc --noEmit
      - run: npx vite build
```

**Workflow adicional**: Deploy a Vercel via `vercel-action` o integración nativa Vercel GitHub.

**Tareas**:
- [ ] Crear `.github/workflows/ci.yml`
- [ ] Agregar lint + typecheck + build
- [ ] Opcional: configurar deploy automático a Vercel
- [ ] Verificar que `vercel.json` esté correcto:
  ```json
  { "framework": "vite" }  // Actualmente "framework": null
  ```

### 7.6 Consolidar Dual CSS Class Systems

**Problema**: Existen dos sistemas de clases CSS en paralelo:

| Old | New (Swiss) |
|-----|-------------|
| `btn-primary`, `btn-ghost` (buttons.css) | `swiss-button-primary`, `swiss-button-ghost` (swiss-hero.css) |
| `grid-3`, `grid-2` (layout.css) | `swiss-grid`, `col-1`–`col-12` (swiss-grid.css) |
| `vanta-footer` (utilities.css) | `swiss-footer` (footer.css) |

**Tareas**:
- [ ] Auditar qué clases old se usan aún en TSX
- [ ] Migrar usos restantes a Swiss classes
- [ ] Eliminar CSS files legacy (8 archivos identificados)
- [ ] Actualizar imports en `index.css`

---

## 8. ⏳ Fase 5: Polish y SEO (PENDIENTE)

> **Prioridad**: 🟢 Baja — el sitio funciona y es creíble sin esto.
> **Esfuerzo estimado**: 5-8 días

### 8.1 Anti-Slop Audit

**Checklist actual**: 10/14 aprobado. 4 incumplimientos:

- [ ] `text-align: center` en 9 elementos (debe ser left)
- [ ] Animaciones GSAP faltantes (ScrollTrigger, count-up, typewriter)
- [ ] Hero no asimétrico (tipográfico puro)
- [ ] prefers-reduced-motion no implementado

**Tareas**:
- [ ] Corregir `text-align: center` en elementos que no deberían tenerlo
- [ ] Implementar Hero asimétrico (Fase 3)
- [ ] Agregar `@media (prefers-reduced-motion: no-preference)` en animaciones

### 8.2 Performance Budget

**Estado actual**: Bundle JS 769 KB (gzip 233 KB) — supera el límite recomendado de 200 KB.

**Tareas**:
- [ ] Code-split rutas pesadas (engine.tsx 1085 líneas, latency.tsx)
- [ ] Lazy-load componentes no críticos
- [ ] Optimizar font loading (preload, swap, subsetting)
- [ ] Configurar `build.chunkSizeWarningLimit`

### 8.3 prefers-reduced-motion

- [ ] Implementar en todas las animaciones GSAP
- [ ] Usar `gsap.matchMedia()` con `(prefers-reduced-motion: no-preference)`
- [ ] Fallback a transiciones CSS instantáneas

### 8.4 SEO Final Review

**Problemas conocidos**:
- Canonical URL → `vantadb.vercel.app` (debe decidir dominio definitivo)
- OG images → `vantadb.dev` (inconsistente)
- SEO audit desactualizado (`revision/seo-meta-audit.txt`)
- Sin JSON-LD específico para cada página (solo global)

**Tareas**:
- [ ] Decidir dominio definitivo (A1: `vantadb.vercel.app` o migrar a `vantadb.dev`)
- [ ] Unificar canonical URL y OG images
- [ ] Actualizar SEO audit
- [ ] Agregar JSON-LD por página (BreadcrumbList, Article para blog)
- [ ] Verificar `llms.txt` esté actualizado con contenido real
- [ ] Verificar `sitemap.xml` incluya todas las rutas nuevas
- [ ] Verificar `robots.txt`

### 8.5 Unificar Dominio (A1)

**Decisión A1**: `vantadb.vercel.app` como dominio único.

**Implicaciones**:
- `vantadb.dev` redirige a `vantadb.vercel.app` (o viceversa)
- OG images deben apuntar al dominio correcto
- Canonical URLs consistentes
- Decap CMS `site_domain` debe coincidir

**Tareas**:
- [ ] Decidir dominio primario
- [ ] Configurar redirects en Vercel
- [ ] Actualizar OG image URLs
- [ ] Actualizar canonical URLs en cada ruta

---

## 9. ⏳ Producto Real — Correcciones (PENDIENTE)

> **Prioridad**: 🟡 Media. No afecta la web directamente pero es necesario para
> consistencia general del proyecto.
> **Esfuerzo estimado**: 2-4 días

### 9.1 Bump Versión v0.1.5 → v0.2.0

**Decisión S5**: Aprobado. VantaDB merece v0.2.0 por ~340+ commits, nuevas APIs,
WASM, ARM64, LangChain/LlamaIndex, CLI/TUI, SQ8 quantization, Prometheus.

**Tareas**:
- [ ] `Cargo.toml`: `version = "0.2.0"` (crate core)
- [ ] `vantadb-python/Cargo.toml`: `version = "0.2.0"`
- [ ] `vantadb-server/Cargo.toml`: `version = "0.2.0"`
- [ ] `vantadb-mcp/Cargo.toml`: `version = "0.2.0"`
- [ ] `vantadb-wasm/Cargo.toml`: `version = "0.2.0"` (si aplica)
- [ ] `pyproject.toml`: `version = "0.2.0"`
- [ ] `CHANGELOG.md`: Nueva sección `[v0.2.0] - 2026-07-02`
- [ ] Tag git: `git tag v0.2.0 && git push --tags`

### 9.2 Refactorizar API `VantaDB()` → `connect()`

**Decisión de investigación 2.3**: `vantadb.VantaDB()` es redundante.

**Cambio propuesto**:
```python
# Antes
import vantadb_py as vantadb
db = vantadb.VantaDB("./path")

# Después
import vantadb_py as vanta  # o mantenemos como vantadb
db = vanta.connect("./path")  # wrapper que crea VantaDB internamente
```

**Tareas**:
- [ ] Agregar función `connect()` en Python bindings
- [ ] Mantener `VantaDB` class para backward compatibility
- [ ] Actualizar documentación

### 9.3 Ejecutar Benchmarks Fresh para v0.2.0

**Decisión F1**: Benchmarks actuales son de v0.1.5. Se necesitan nuevos para v0.2.0.

**Tareas**:
- [ ] `cargo bench` en v0.2.0
- [ ] Ejecutar Python benchmark suite
- [ ] Publicar resultados
- [ ] Actualizar web con datos de v0.2.0

### 9.4 Agregar CLA para Contribuciones

**Recomendación de investigación 2.1**: Apache 2.0 sin CLA es riesgoso para
futura monetización enterprise.

**Tareas**:
- [ ] Investigar opciones de CLA (Apache ICLA, Google-style, Harmony)
- [ ] Agregar a `.github/CONTRIBUTING.md`
- [ ] Decidir si usar CLA bot (CLA Assistant)

### 9.5 Diseñar Estructura de Crate Enterprise

**Recomendación de investigación 2.1**: Features pagas en crate separado.

**Tareas**:
- [ ] Definir qué features van en `vantadb-enterprise/`
- [ ] Crear estructura inicial del crate
- [ ] Documentar modelo de licencia dual

### 9.6 Limpiar Naming Histórico en Build System

**Problema**: `Dockerfile` aún compila `connectome-server` (no `vanta-server`).

**Tareas**:
- [ ] Buscar `connectome` en todo el repo
- [ ] Reemplazar por `vanta` donde corresponda
- [ ] Verificar que el build no se rompe

---

## 10. Features Evaluadas y Diferidas

> Estas features fueron evaluadas durante la investigación pero NO deben implementarse
> ahora. Se listan solo para contexto y decisión futura.

### 10.1 SQL Engine — NO (hasta 2027)

**Decisión S4**: Eliminado completamente de la web. No implementar en el producto.

| Aspecto | Implicación |
|---------|-------------|
| Costo estimado | 6-12 persona-meses para MVP mínimo |
| Tamaño binario | +5-10 MB |
| Tiempo compilación | +10-20 min |
| Tests necesarios | +10K-50K |
| Alternativa | Composición con SQLite/DuckDB externos |

**Timeline sugerido si se reconsidera**:
- Q3 2026: Launch sin SQL (memoria embebida + hybrid search)
- Q1 2027: Re-evaluar si hay demanda real
- Q2-Q4 2027: Posible MVP SQL

### 10.2 Multi-node Replication — DEFERRED

No implementado. No prometer en la web.

### 10.3 WASM Production-ready — DEFERRED

Estado actual: Experimental. No prometer production-ready.

### 10.4 Android/iOS — RECHAZADO

Decisión S9: Eliminar. Sin evidencia ni planes.

### 10.5 Toggle Tema Claro/Oscuro — RECHAZADO

Decisión B7: No implementar. Solo warm paper.

### 10.6 Integraciones Faltantes (Mem0, CrewAI, DSPy, Haystack) — Planificadas

No implementadas. No listar como existentes.

### 10.7 PyPI Producción — DEFERRED

Solo TestPyPI. No prometer `pip install vantadb-py` en producción hasta que esté en PyPI real.

---

## 11. Apéndice: Estado de Cada Ruta

> Estado después de Fase S + Fase 1 (Julio 2026).

| Ruta | Archivo | Calidad Diseño | Veracidad | Prioridad Acción |
|------|---------|---------------|-----------|------------------|
| `/` | index.tsx | ⭐⭐⭐ 8 secciones | ✅ ~95% | 🟡 Estadísticas a SwissBenchmarkGrid |
| `/engine` | engine.tsx (1085l) | ⭐⭐⭐ Interactivo | ✅ ~98% | 🟢 RRF slider fórmula (cosmético) |
| `/architecture` | architecture.tsx (557l) | ⭐⭐⭐ Profiler | ✅ 100% | 🟢 Sin cambios necesarios |
| `/integrations` | integrations.tsx (392l) | ⭐⭐⭐ Selector | ✅ 100% | 🟢 Sin cambios necesarios |
| `/use-cases` | use-cases.tsx (303l) | ⭐⭐ 8 casos | ⚠️ Algunos ficticios | 🔴 Revisar casos ARM/RISC-V, healthcare |
| `/pricing` | pricing.tsx (483l) | ⭐⭐ | ✅ 100% | 🟢 Sin cambios (Open Core + Enterprise) |
| `/docs` | docs.tsx (281l) | ⚠️ Design guide | ⚠️ No es docs técnica | 🔴 Fase 2: crear /docs-api |
| `/cost` | cost.tsx (353l) | ⭐⭐ Tabla | ⚠️ Comparativa válida | 🟡 Revisar claims |
| `/latency` | latency.tsx | ⭐⭐ Simulador | ✅ ~95% | 🟢 Toggle implementado |
| `/storage` | storage.tsx (486l) | ⭐⭐ 3 capas | ✅ 100% | 🟢 Sin cambios |
| `/config` | config.tsx (321l) | ⭐ 0-config | ✅ 100% | 🟢 Sin cambios |
| `/maint` | maint.tsx (426l) | ⭐⭐ 4h vs 30s | ✅ 100% | 🟢 Sin cambios |
| `/changelog` | changelog.tsx | ⭐⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/solutions/ai-agents` | 333l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/solutions/ai-ide-tooling` | 317l | ⭐⭐ | ⚠️ AST-aware no existe | 🟡 Corregir o eliminar claim |
| `/solutions/local-rag` | 325l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/about/company` | 309l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/about/community` | 379l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/about/contact` | 221l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/about/index` | 245l | ⭐⭐ | ✅ 100% | 🟢 Sin cambios |
| `/blog` | index + $slug | ⭐ Placeholder | ✅ Vacío | 🟡 Publicar artículos (Fase 2) |
| `/product/` | directorio vacío | ❌ | ❌ | 🔴 Crear página (Fase 2) |
| `/product/benchmarks` | no existe | ❌ | ❌ | 🔴 Crear página (Fase 2) |
| `/security` | no existe | ❌ | ❌ | 🟡 Crear página (Fase 2) |
| `/about/roadmap` | no existe | ❌ | ❌ | 🟡 Crear página (Fase 2) |
| `/docs-api` | no existe | ❌ | ❌ | 🟡 Crear página (Fase 2) |

---

## 12. Apéndice: Deuda Técnica Detallada

### 12.1 CSS Issues Conocidos

| # | Issue | Archivo | Severidad |
|---|-------|---------|-----------|
| 1 | `--text-label` y `--text-code` usados en 11 TSX + 2 CSS pero recién definidos | Múltiples | 🟢 Resuelto |
| 2 | Clases `btn-*` coexisten con `swiss-button-*` | buttons.css + swiss-hero.css | 🟡 Medio |
| 3 | Clases `vanta-footer*` coexisten con `swiss-footer*` | utilities.css + footer.css | 🟢 Bajo |
| 4 | `--shadow-glow: none` vestigial en tokens | tokens.css | 🟢 Bajo |
| 5 | `--surface-glass` no es token real | tokens.css | 🟢 Bajo |
| 6 | Múltiples `--amber-*` y `--steel-*` variantes fragmentan paleta | tokens.css | 🟢 Bajo |
| 7 | Clases Swiss BEM definidas pero no usadas en TSX | swiss-benchmark.css, swiss-core.css, etc. | 🟡 Medio |

### 12.2 Dependencias Post-Limpieza

**Después de Fase 1** (24 dependencias activas):

```
dependencies:
  @fontsource-variable/jetbrains-mono, outfit, space-grotesk
  @gsap/react, gsap
  @tailwindcss/vite, tailwindcss, tw-animate-css
  @tanstack/react-query, @tanstack/react-router, @tanstack/router-plugin
  animejs, @types/animejs
  clsx, tailwind-merge
  motion
  marked
  react, react-dom
  vite-tsconfig-paths

devDependencies:
  @eslint/js, eslint
  @types/node, @types/react, @types/react-dom
  @vitejs/plugin-react
  eslint-plugin-prettier, eslint-plugin-react-hooks, eslint-plugin-react-refresh
  globals, prettier
  typescript, typescript-eslint
  vite
```

### 12.3 Build Warnings

| Warning | Detalle | Acción |
|---------|---------|--------|
| JS bundle >500 KB | 769 KB (gzip 233 KB) | Code-split rutas pesadas |

---

*Documento generado: 2026-07-02*
*Fuentes: `docs/ANALISIS_COMPLETO_Y_DECISIONES.md` v4.0 + `VantaDB/docs/REPORTE_INVESTIGACION_Y_DECISIONES.md` v2.0*
*Próxima actualización: al completar Fase 2.*
