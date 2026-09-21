# MKT-05: Completar el 5º blog post pre-launch (benchmarks)

## Metadata
- **Plan file:** docs/plans/2026-08-04-launch-web-campaign.md
- **Creado:** 2026-08-04T07:10
- **last-synced:** 2026-08-04T07:15
- **Estado:** ✅ COMPLETED

## Blast Radius
| Callers | Callees | Implicaciones |
|---------|---------|---------------|
| BLOG_SERIES_PLAN.md (§1.2, §1.3, §4.3) | docs/blog/benchmarks_vs_lancedb_chroma.md | El plan de serie referencia el draft; fila 9 de §4.3 pasa de "Plan" a "Drafted" |
| MKT-15 (benchmarks) | benchmarks/competitive_bench.py | Las cifras del post provienen de un run real del harness (2026-08-04) |
| docs/operations/BENCHMARKS.md | — | Números históricos (jun-2026) NO comparables (metodología pre-Jul-31, doble rebuild); el post lo aclara explícitamente |

## Contrato
"glob docs/blog/*.md → 5 archivos; BLOG_SERIES_PLAN.md actualizado con el 5º post"

## Herramientas
- bash (python bench), read/write/edit, glob

## Steps
### Step 1: Ejecutar/validar datos reales del benchmark
- **Archivos:** `benchmarks/competitive_bench.py`, `benchmarks/README.md`, `docs/operations/BENCHMARKS.md`
- **Acción:** Ejecutar el bench competitivo (deps OK, dataset glove cacheado) y extraer tabla real.
- **Verify:** Salida del harness con tabla de medianas (VantaDB 241.4 QPS / recall 100% / p50 4.124ms; LanceDB 197.5 QPS / recall 22.8%; Chroma 591.1 QPS / recall 95.6%).
- **Estado:** ✅

### Step 2: Escribir 5º post en docs/blog/
- **Archivos:** `docs/blog/benchmarks_vs_lancedb_chroma.md` (nuevo, ~130 líneas)
- **Acción:** Post con frontmatter del formato de la serie (2026-06-06, VantaDB Team, draft: true), metodología transparente, tabla del run, lectura honesta (recall, QPS direccional, límites de lo que mide una tabla), CTA.
- **Verify:** glob docs/blog/*.md → 5 archivos.
- **Estado:** ✅

### Step 3: Actualizar BLOG_SERIES_PLAN.md
- **Archivos:** `docs/strategy/BLOG_SERIES_PLAN.md`
- **Acción:** Status Summary 3/4 → 4/5; §1.2 agrega filas 4 y 5 + nota M1 resuelto; §1.3 nota benchmarks drafted; §4.3 fila 9 → Drafted (MKT-05).
- **Verify:** diff 9 insertions / 5 deletions, solo los 2 archivos en el commit.
- **Estado:** ✅

### Step 4: Commit
- **Acción:** `git add docs/blog/benchmarks_vs_lancedb_chroma.md docs/strategy/BLOG_SERIES_PLAN.md && git commit -m "docs(MKT-05): add 5th pre-launch blog post on benchmarks"`
- **Verify:** commit bf5e6c1e (2 files, +133/-5). Pre-commit hooks pasaron.
- **Estado:** ✅

## Dependencias
- Ninguna (task standalone; alineada con MKT-15 solo a nivel de fuente de datos)

## Notas
- El run real del bench arrojó cifras MUY distintas a las publicadas en docs/operations/BENCHMARKS.md §7 (jun-2026): 241.4 QPS vs 24.3 QPS. Razón: metodología corregida (--batch-size 999 elimina el doble rebuild de HNSW) + estado del engine. El post documenta ambas cosas con transparencia.
- Caveats del run reportados en el post: health check marcó CPU ~85% (números absolutos contaminados, comparación direccional válida) y Chroma completó 1/3 runs en Windows (file lock en cleanup). No se tocó competitive_bench.py (out of scope).
- Se siguieron las reglas: solo docs/blog/ y docs/strategy/BLOG_SERIES_PLAN.md, sin push, sin tocar plan file, sin inventar cifras.

## Context Save Point
- **Fecha:** 2026-08-04T07:15
- **Branch:** develop
- **CI pendiente:** no (cambio solo de docs; pre-commit local pasó)
- **Decisiones:**
  - Ejecutar el bench real en vez de usar números del README porque las deps estaban instaladas y el dataset glove estaba cacheado (485MB) → cifras frescas y honestas del harness.
  - Post transparente con caveats (CPU load, 1/3 runs de Chroma, metodología pre-Jul-31) en lugar de presentar la tabla como absoluta — coherente con la voz honesta de la serie.
  - Fecha del post 2026-06-06 (convención de la serie) aunque el run fue 2026-08-04; la fecha de ejecución se aclara en el cuerpo.
- **Problemas conocidos:**
  - docs/operations/BENCHMARKS.md §7 tiene números históricos (24.3 QPS) que contradicen el post (241.4 QPS); es el registro histórico correcto de su momento, el post lo aclara. Si se quiere, un task futuro debería actualizar ese doc o anotarlo.
  - BLOG_SERIES_PLAN.md aún lista M2–M6 (author/date/title drift, version) pendientes de reconciliación pre-publicación.
- **Próxima tarea:** MKT-06 / continuar serie de lanzamiento (pendiente de la campaña launch-web).
