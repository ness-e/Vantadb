# Extracción Documental — INV-020: Evidencia de Arquitectura Milvus

> **ID:** `INV-020`
> **Categoría:** Arquitectura / Evidencia competitiva
> **Fecha de extracción:** 2026-08-04
> **Estado:** `extracted/archived`
> **Fuente:** `VANTADB DOC OLD/milvus.md` (eliminado) — deep-dive único de Milvus del repo, 44 fuentes citadas
> **Disclaimer:** los datos del documento fuente corresponden a **abril 2026**; verificar contra docs oficiales de Milvus antes de decisiones que dependan de versiones.

---

## 1. Propósito de esta extracción

`VANTADB DOC OLD/milvus.md` fue el único deep-dive de arquitectura de Milvus en el
repo. Verdict: **extract insights de ingeniería accionables**, no conservar el doc
completo. Esta nota guarda la evidencia de diseño (con sus fuentes) que alimenta
items del ROADMAP. No se copia código; se documentan decisiones y mecanismos
observables.

**Scope:** solo insights de ingeniería accionables. La narrativa especulativa
LISP/neurobiológica del documento fuente se descarta por no ser evidencia técnica.
La arquitectura distribuida (etcd, Pulsar/Kafka, MinIO, K8s) queda **fuera de
scope** — ver Nota estratégica §6.

---

## 2. Insight 1 — Bitset/roaring pre-filtrado durante el ANN walk

**Hallazgo:** Milvus usa **bitsets** como mecanismo central para filtrar durante la
búsqueda vectorial, no después. El flujo de una query híbrida es:

1. Evaluación de expresión escalar → genera un bitset de filtrado (bits en "1" = cumplen).
2. Se consulta el bitset de eliminación persistente (soft deletes; "1" = borrado).
3. **Knowhere recibe los bitsets y, durante el recorrido del grafo (HNSW) o el
   escaneo de clusters (IVF), omite cualquier ID cuyo bit combinado sea "1".**

Esto es pre-filtrado *in-traversal*: evita recuperar vectores que luego se
descartarían por no cumplir el criterio de metadatos. El mismo mecanismo de bitsets
soporta **Time Travel**: filtrar entidades comparando sus timestamps individuales
contra el timestamp de la consulta.

**Fuentes citadas (del documento OLD):**
- Bitset | Milvus Documentation — [milvus.io/docs/bitset.md](https://milvus.io/docs/bitset.md) (fuente [25])
- Knowhere | Milvus Documentation — [milvus.io/docs/knowhere.md](https://milvus.io/docs/knowhere.md) (fuente [13])
- Filtered Search | Milvus Documentation — [milvus.io/docs/filtered-search.md](https://milvus.io/docs/filtered-search.md) (fuente [11])
- Time Travel (v2.2.x) — [milvus.io/docs/v2.2.x/timetravel_ref.md](https://milvus.io/docs/v2.2.x/timetravel_ref.md) (fuente [26])

**Relevancia para VantaDB (referencias, sin editar ROADMAP):**
- **COMP-003** (In-filter traversal: intersectar `FilterBitset` durante el HNSW walk) — el mecanismo de Knowhere es la validación externa de que el approach funciona en producción a escala.
- **COMP-012** (RoaringBitmaps metadata index) — Milvus usa bitsets para filtrado Y soft deletes; la compactación periódica de masks de borrado es parte del modelo de mantenimiento.
- **COMP-004** (bitset-based filtering + soft deletes con RoaringBitmaps) — mismo patrón de deletion masks que Milvus.

---

## 3. Insight 2 — Modelo growing/sealed segment + indexación incremental + compactación

**Hallazgo:** Milvus divide datos en **segmentos** con dos estados:

| Estado | Rol | Persistencia |
|---|---|---|
| **Growing** | Buffer en memoria para ingesta en tiempo real | WAL (Kafka/Pulsar) → memoria |
| **Sealed** | Bloque inmutable optimizado para búsqueda ANN | Object storage (S3/MinIO) |

Una vez que un segmento alcanza un umbral de tamaño, se **sella** (inmutable) y se
activa la **indexación incremental**. La inmutabilidad permite que los query nodes
carguen desde object storage sin conflictos de escritura → escalabilidad horizontal
casi lineal. La **compactación** (clustering compaction) mergea segmentos pequeños
en archivos grandes, reduciendo llamadas al object storage y la fragmentación del
índice.

**Implicación clave:** este es el mecanismo que le da a Milvus **CRUD sobre HNSW sin
rebuild completo** — los updates/deletes operan vía soft-delete masks y compactación
de segmentos, no reconstruyendo el grafo completo.

**Fuentes citadas (del documento OLD):**
- Milvus Vector Database — Augment Code (fuente [2]) — segmentos growing/sealed
- Clustering Compaction | Milvus Documentation — [milvus.io/docs/clustering-compaction.md](https://milvus.io/docs/clustering-compaction.md) (fuente [6])
- Milvus Architecture Overview — [milvus.io/docs/architecture_overview.md](https://milvus.io/docs/architecture_overview.md) (fuente [7])

**Relevancia para VantaDB (referencias):**
- Gap competitivo documentado en **ROADMAP:80**: `CRUD en HNSW — VantaDB ❌ rebuild completo vs Milvus ✅ (growing/sealed)`. Este insight confirma que el modelo de segmentos inmutables + compactación es la solución de referencia del mercado para el gap.
- **COMP-011** (HNSW CRUD con tombstones, sin rebuild completo) y **COMP-004** (soft deletes) son los items que cierran ese gap en VantaDB.

---

## 4. Insight 3 — Cuantización SQ8/PQ con trade-off memoria/recall

**Hallazgo:** la familia IVF soporta cuantización escalar o de producto
(**SQ8/PQ**): comprime vectores de 32 bits a 8 bits o menos, con ligera pérdida de
precisión. HNSW, por el contrario, penaliza fuertemente en RAM (a menudo ~2× el
tamaño de los vectores originales por la estructura de grafos adyacentes). El trade-off
documentado en Milvus: la cuantización reduce memoria y costos, pero **degrada
recall significativamente** en algunos casos (fuente [20] lo marca como riesgo real).

**Fuentes citadas (del documento OLD):**
- IVF vs HNSW Indexing in Milvus — Medium (fuente [18])
- Understanding IVF Vector Index — [milvus.io/blog/understanding-ivf-vector-index-how-It-works-and-when-to-choose-it-over-hnsw.md](https://milvus.io/blog/understanding-ivf-vector-index-how-It-works-and-when-to-choose-it-over-hnsw.md) (fuente [21])
- How to Cut Vector Database Costs by Up to 80% — [milvus.io/blog/how-to-cut-vector-database-costs-by-up-to-80-a-practical-milvus-optimization-guide.md](https://milvus.io/blog/how-to-cut-vector-database-costs-by-up-to-80-a-practical-milvus-optimization-guide.md) (fuente [20])

**Relevancia para VantaDB (referencias):**
- **R6 / COMP-001** (exponer SQ8 en `distance.rs` hot path) — Milvus demuestra que SQ8 es el estándar de mercado para cortar costos de RAM; VantaDB ya tiene `VectorRepresentations::SQ8` implementado pero sin exponer en el hot path (ROADMAP:95).
- La evidencia del trade-off recall/memoria refuerza la necesidad de **re-ranking** post-cuantización (no solo SQ8 puro) para mitigar la pérdida de recall que Milvus admite.

---

## 5. Insight 4 — Tiered storage + MMap / lazy loading

**Hallazgo:** Milvus maneja datasets que exceden RAM con dos mecanismos:

1. **MMap** (desde 2.3): el SO gestiona carga/descarga de páginas desde disco local (NVMe); reduce uso de RAM entre **60–80%** vs carga completa, con latencias estables.
2. **Tiered Storage** (desde 2.6): **lazy loading** — solo metadatos esenciales se cargan al inicio; vectores e índices se descargan del object storage solo al ser tocados por una query, con política de expulsión **LRU**.

| Métrica | Full Load | MMap (local SSD) | Tiered (S3 + LRU) |
|---|---|---|---|
| Latencia P99 | < 20 ms | 20–40 ms | 100–500 ms (cache miss) |
| Capacidad | Limitada por RAM | Limitada por disco local | Virtualmente ilimitada |
| Costo | Muy alto | Medio | Bajo |

**Fuentes citadas (del documento OLD):**
- How to Cut Vector Database Costs by Up to 80% (fuente [20]) — MMap + métricas
- Milvus Tiered Storage: 80% Less Vector Search Cost — [milvus.io/blog/milvus-tiered-storage-80-less-vector-search-cost-with-on-demand-hot–cold-data-loading.md](https://milvus.io/blog/milvus-tiered-storage-80-less-vector-search-cost-with-on-demand-hot%E2%80%93cold-data-loading.md) (fuente [31])

**Relevancia para VantaDB (referencias):**
- **R7 / COMP-002** (HNSW persistence, eliminar rebuild en cada startup — 30–60s para 1M, ROADMAP:98) — el lazy loading de Milvus es evidencia de que el cold start en escala se resuelve con carga diferida + metadata first, no con carga total en memoria.
- Nota: VantaDB ya documenta MMap HNSW + SQ8 para 1M–10M en `docs/operations/PERFORMANCE_TUNING.md`; este insight valida el approach y añade la alternativa tiered (S3 + LRU) como siguiente nivel.

---

## 6. Nota estratégica — scope competitivo

VantaDB **NO compite con Milvus** en el espacio distribuido
(`docs/vision/VISION.md:162`: *"NOT a distributed database — No native replication,
no auto-sharding; does not compete with Milvus/Qdrant in distributed vector DB
space"*). Por lo tanto, la arquitectura distribuida de Milvus (etcd para metadatos,
Pulsar/Kafka para WAL, MinIO/S3 para storage, K8s/Helm/Operators para deployment)
queda **fuera de scope** de esta extracción y de los items COMP referenciados.

Los insights anteriores se extraen porque aplican a un motor **embebido/local-first**
(Rust), que es el terreno donde VantaDB sí compite: filtrado in-traversal, CRUD sin
rebuild, cuantización y gestión de memoria.

---

## 7. Fuentes del documento original (relevantes a esta extracción)

Números de fuente según el documento OLD (todas con fecha de acceso abril 2026):

| # | Fuente | Usada en |
|---|---|---|
| [2] | Milvus Vector Database — Augment Code | §3 |
| [6] | Clustering Compaction — Milvus Docs | §3 |
| [7] | Milvus Architecture Overview | §3 |
| [11] | Filtered Search — Milvus Docs | §2 |
| [13] | Knowhere — Milvus Docs | §2 |
| [18] | IVF vs HNSW Indexing — Medium | §4 |
| [20] | How to Cut Vector Database Costs by Up to 80% | §4, §5 |
| [21] | Understanding IVF Vector Index | §4 |
| [25] | Bitset — Milvus Docs | §2 |
| [26] | Time Travel (v2.2.x) | §2 |
| [31] | Milvus Tiered Storage — Milvus Blog | §5 |

El resto de las 44 fuentes (distribución, consistencia, Arrow, embedding functions,
JSON shredding, geoespacial, etc.) quedan fuera del alcance de esta extracción; el
doc original fue archivado/eliminado según el veredicto de extracción.
