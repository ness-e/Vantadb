# Site Map — VantaDB Web

> Inventario completo de rutas web con estado de implementación Swiss.
> Total: **28 rutas implementadas** | 6 directorios

---

## Root Routes (17)

| Ruta | Archivo | Swiss Status | Subpage Hero | Secciones |
|:---|:---|:---|:---|:---|
| `/` | `index.tsx` | ⚠️ Parcial | N/A | 8 secciones (Hero → Monolith) |
| `/engine` | `engine.tsx` | ❌ Legacy | Pendiente | HNSW, BM25, WAL, PyO3, Zero-Copy, SIMD |
| `/architecture` | `architecture.tsx` | ❌ Legacy | Pendiente | Capas SVG interactivas |
| `/integrations` | `integrations.tsx` | ❌ Legacy | Pendiente | Grid por categoría |
| `/use-cases` | `use-cases.tsx` | ❌ Legacy | Pendiente | Casos expandidos |
| `/cost` | `cost.tsx` | ❌ Legacy | Pendiente | Grid Bento costos |
| `/latency` | `latency.tsx` | ❌ Legacy | Pendiente | Barras SVG benchmarks |
| `/storage` | `storage.tsx` | ❌ Legacy | Pendiente | Diagramas WAL/HNSW |
| `/config` | `config.tsx` | ❌ Legacy | Pendiente | Tabla de opciones |
| `/maint` | `maint.tsx` | ❌ Legacy | Pendiente | Pasos numerados |
| `/changelog` | `changelog.tsx` | ❌ Legacy | Pendiente | Timeline vertical |
| `/docs` | `docs.tsx` | ❌ Legacy | N/A | 2-col sidebar |
| `/docs-api` | `docs-api.tsx` | ❌ Legacy | N/A | API reference |
| `/pricing` | `pricing.tsx` | ❌ Legacy | Pendiente | Grid planes + FAQ |
| `/security` | `security.tsx` | ❌ Legacy | Pendiente | — |
| `/blog` | `blog/index.tsx` | ❌ Legacy | N/A | Grid 2 col posts |
| `/blog/$slug` | `blog/$slug.tsx` | ❌ Legacy | N/A | Editorial layout |

## About Routes (5)

| Ruta | Archivo | Swiss Status | Subpage Hero |
|:---|:---|:---|:---|
| `/about` | `about/index.tsx` | ❌ Legacy | Pendiente |
| `/about/company` | `about/company.tsx` | ❌ Legacy | Pendiente |
| `/about/community` | `about/community.tsx` | ❌ Legacy | Pendiente |
| `/about/contact` | `about/contact.tsx` | ❌ Legacy | Pendiente |
| `/about/roadmap` | `about/roadmap.tsx` | 🗑️ **DELETE** | N/A |

## Solutions Routes (3)

| Ruta | Archivo | Swiss Status | Subpage Hero |
|:---|:---|:---|:---|
| `/solutions/ai-agents` | `solutions/ai-agents.tsx` | ❌ Legacy | Pendiente |
| `/solutions/local-rag` | `solutions/local-rag.tsx` | ❌ Legacy | Pendiente |
| `/solutions/ai-ide-tooling` | `solutions/ai-ide-tooling.tsx` | ❌ Legacy | Pendiente |

## Product Routes (1)

| Ruta | Archivo | Swiss Status |
|:---|:---|:---|
| `/product/benchmarks` | `product/benchmarks.tsx` | ❌ Legacy |

---

## Leyenda

| Estado | Significado |
|:---|:---|
| ✅ **Swiss** | Rediseñado completo con estilo Swiss |
| ⚠️ **Parcial** | En progreso (index con algunos elementos Swiss) |
| ❌ **Legacy** | Diseño anterior — requiere rediseño completo |
| 🗑️ **DELETE** | Pendiente de eliminación |

---

## Prioridad de Rediseño

| Prioridad | Rutas | Fase |
|:---|:---|:---|
| 🔴 **Crítica** | `/` (index) | Fase 2 |
| 🟠 **Alta** | `/engine`, `/architecture`, `/integrations`, `/use-cases` | Fase 3 |
| 🟡 **Media** | `/cost`, `/latency`, `/storage`, `/config`, `/maint`, `/changelog` | Fase 4 |
| 🟢 **Baja** | `/solutions/*`, `/docs`, `/pricing`, `/about/*`, `/blog/*`, `/security`, `/product/*` | Fase 5 |
| ⚫ **Eliminar** | `/about/roadmap` | Fase 6 |

---

## Notas

- Todas las rutas tienen archivo `.tsx` existente (100% coverage)
- 16 de 28 rutas usan lazy loading (`.lazy.tsx`)
- Ruta dinámica: `blog/$slug.tsx`
- Fase 6 incluye purge de rutas legacy (`about/roadmap`)
- Referencia: `design/SUB_PAGE_PATTERNS.md` para el patrón común
