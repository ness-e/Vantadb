# Reporte de Investigación — INV-007: Competitive Benchmark vs LanceDB/Chroma — Investigación y Diseño

> **ID:** `INV-007`
> **Categoría:** Competitive Benchmarking &amp; Performance
> **Fecha:** 2026-08-03
> **Alcance:** DISEÑO PURO — sin implementación, sin tocar `Cargo.toml` ni `web/`
> **Estado:** ✅ Investigación Completada — Propuesta lista para implementación puntual

---

## Summary Ejecutivo

Se investigó `ann-benchmarks`, la metodología de benchmark de LanceDB Enterprise, benchmarks
independientes de ChromaDB/LanceDB, y el estado actual de `benches/` (12 criterio benches) más
el web `/benchmarks`.

**Veredicto central:** **no conviene meter VantaDB como "algoritmo" del repo `ann-benchmarks`**
(el repo está oficialmente sin mantenimiento, restringe a single-CPU in-memory, y el esfuerzo
de integración es alto). Tampoco **extender `benches/`** produce cifras competitivas honestas
(los benches internos usan datasets sintéticos de 10K y no ejecutan sistemas rivales).

**Recomendación (mínimo viable):** un **harness standalone reproducible (Python)** que baje los
datasets reales (glove-100 + sift-128), construya el índice de VantaDB (vía `vantadb-py`) y
compare contra `chromadb` y `lancedb` bajo el MISMO protocolo, emitiendo
`competitive_benchmark.json`. La página `/benchmarks` muestra una tabla alimentada por ese JSON
con disclaimer de método/hardware. `PRODUCT.metrics` queda intacta (baselines internos
BENCH01/SIFT1M); la tabla competitiva usa datos reales.

**Gate 2026-08-03:** este documento es diseño puro — no se modificó código, ni `Cargo.toml`,
ni `web/`. Ver sección 8.

---

## 1. Contexto y Objetivos

VantaDB v0.5.0 es un motor Rust embebido de retrieval híbrido (HNSW + BM25, RRF).
`web/src/app/benchmarks/` ya existe (BenchmarksView + BenchmarkRace, 444L) con BENCH01,
SIFT1M, LatencyComparator. La tabla competitiva VantaDB vs Pinecone/Weaviate/Chroma es MKT-15
— fuera de alcance.

Alcance de INV-007 (investigación + diseño, SIN implementación):
1. Investigar `ann-benchmarks` y cómo se agrega un sistema nuevo (esfuerzo, formato de datos).
2. Definir datasets: glove-100-angular + sift-128-euclidean (justificación).
3. Metodología: throughput, latencia p50/p95/p99, Recall@10, RAM; protocolo (warmup, iteraciones, hardware).
4. Evaluar si extender `benches/` + `/benchmarks` web es viable o si se necesita script standalone reproducible.
5. Proponer implementación mínima para la página pública (slicing vertical).

---

## 2. Análisis de `ann-benchmarks` y Cómo se Conectaría VantaDB

### 2.1 Qué es ann-benchmarks

`ann-benchmarks` es el framework de referencia para comparar algoritmos de búsqueda de
vecinos aproximados (ANN), creado por Erik Bernhardsson, Martin Aumüller y Alexander
Faithfull (paper: *ANN-Benchmarks: A Benchmarking Tool for Approximate Nearest Neighbor
Algorithms*, Information Systems 87, 2019, DOI 10.1016/j.is.2019.02.006).

Características verificadas (fuente: GitHub README + web oficial ann-benchmarks.com):

- **Sistema Python + Docker.** Cada algoritmo vive en `ann_benchmarks/algorithms/{NOMBRE}/`
  con tres archivos: `module.py` (wrapper que implementa la interfaz `BaseANN`:
  `__init__`, `fit`, `query`, `set_query_arguments`), `Dockerfile` (entorno de instalación),
  y `config.yml` (parámetros e hiper-parámetros a barrer).
- **Datasets pre-generados en formato HDF5** (matrices de entrenamiento `train`, consultas
  `test`, ground truth `neighbors`). Ejemplos: `glove-100-angular.hdf5` (463MB),
  `sift-128-euclidean.hdf5` (501MB), `deep-image-96-angular.hdf5` (3.6GB).
- **Métricas**: Recall (fracción de verdaderos vecinos encontrados, sobre k=10 por defecto)
  contra Queries Per Second (QPS). Un "fudge factor" de epsilon permite empatar vecinos con
  distancia casi idéntica. Queries se ejecutan **serialmente** y se satura **una sola CPU**
  (sin multi-threading) por defecto; existe `--batch` para modo batch.
- **Resultados** en formato JSON por algoritmo/dataset; `plot.py` genera scatter Recall-QPS
  y `create_website.py` genera el sitio interactivo (ann-benchmarks.com).
- **Integración CI**: se añade el algoritmo a `.github/workflows/benchmarks.yml` y se valida
  con `python run.py --dataset random-xs-20-angular --algorithm {algo}`.

### 2.2 Estado de mantenimiento (dato crítico de 2026)

> El README del repo dice textualmente: **"At this point, ann-benchmarks is no longer
> actively maintained. Please consider submitting your work to different benchmarks,
> such as VIBE."**

Esto es decisivo: integrar VantaDB en `ann-benchmarks` hoy sería invertir esfuerzo en un
repo congelado cuyo sucesor recomendado es VIBE (`github.com/vector-index-bench/vibe`,
benchmarks modernos, mantenido). La integración serviría solo como "publicación histórica",
no como repositorio vivo de resultados.

### 2.3 Cómo se conectaría VantaDB (esfuerzo estimado)

Para agregar VantaDB como algoritmo ANN al framework:

| Componente | Contenido | Esfuerzo |
|---|---|---|
| `module.py` | Wrapper `BaseANN` que expone `fit` (build index) y `query` (búsqueda top-k), mapeando métricas `angular`/`euclidean` a cosine/L2 de VantaDB | 1–2 días |
| `Dockerfile` | Imagen con Rust toolchain + `cargo build --release` de `vantadb` y sus bindings | 0.5–1 día |
| `config.yml` | Grid de `M`, `ef_construction`, `ef_search`, `k` | 0.5 día |
| `benchmarks.yml` | Entrada CI + test con dataset pequeño | 0.5 día |

Total: **~3–5 días efectivos** para una integración mínima. Riesgos: compilación Rust lenta
en Docker, restricción single-CPU del framework que no refleja el rendimiento real del
motor, y un repo que no se mantiene (el PR podría quedar sin revisar indefinidamente).

**Conclusión:** `ann-benchmarks` se usa como **fuente de datasets estándar y de metodología
(Recall vs QPS), no como plataforma de publicación**.

---

## 3. Datasets Definidos y Justificación

Se definen **dos datasets oficiales de ann-benchmarks**, en formato HDF5, descargables de
ann-benchmarks.com:

### 3.1 `glove-100-angular` — embeddings de texto (cosine/angular)

| Propiedad | Valor |
|---|---|
| Dimensionalidad | 100 |
| Vectores de entrenamiento | 1,183,514 |
| Queries | 10,000 |
| Distancia | Angular (cosine sobre vectores normalizados) |
| Tamaño HDF5 | 463 MB |
| Fuente | http://ann-benchmarks.com/glove-100-angular.hdf5 |

**Por qué:** es el dataset más citado del ecosistema RAG/texto. Reproduce el caso de uso
central de VantaDB (memoria de agentes, retrieval semántico con embeddings). Valida la
ruta **cosine/HNSW** que es la default del motor. Al ser 100D, mantiene coste de cómputo
bajo y permite correr en máquinas con 8–16GB RAM sin swap.

### 3.2 `sift-128-euclidean` — descriptores de imagen (L2/euclidean)

| Propiedad | Valor |
|---|---|
| Dimensionalidad | 128 |
| Vectores de entrenamiento | 1,000,000 |
| Queries | 10,000 |
| Distancia | Euclidiana (L2) |
| Tamaño HDF5 | 501 MB |
| Fuente | http://ann-benchmarks.com/sift-128-euclidean.hdf5 |

**Por qué:** es el benchmark canónico de ANN desde 2011 (corpus Texmex, INRIA). Completa a
glove al validar la ruta **euclidean/L2**, que VantaDB ya optimiza (eliminación de sqrt,
SIMD, ver SIFT1M en `vanta-data.ts`: "Balanced L2" 2.80x). Es el dataset con más
resultados publicados en la literatura — cualquier comparación externa será auditable.

### 3.3 Ambas juntas: qué cubren

| Eje | glove-100-angular | sift-128-euclidean |
|---|---|---|
| Dimensionalidad | Media (100) | Media-alta (128) |
| Distribución | embeddings de texto | descriptores visuales |
| Métrica | Cosine (default VantaDB) | Euclidiana (optimizada) |
| Tamaño índice en RAM (f32, ~1M × dim × 4B) | ~473 MB + grafo | ~512 MB + grafo |

Decisión de diseño: **ambos son in-memory y caben en una máquina de 16GB** — requisito del
framework ann-benchmarks ("Focus on datasets that fit in RAM") y de un harness reproducible
local. No se incluyen GIST-960 ni DEEP-1B (3.6GB+ de HDF5, RAM excesiva, fuera del perfil
de un motor embebido local-first).

---

## 4. Metodología

### 4.1 Métricas

| Métrica | Definición | Unidad |
|---|---|---|
| Recall@10 | Fracción de los 10 verdaderos vecinos encontrados por query (contra ground truth exacto del HDF5) | 0–1 |
| Throughput | Queries por segundo (QPS) = N_queries / tiempo total, serial | queries/s |
| Latencia p50 / p95 / p99 | Percentiles de latencia por query | ms |
| RAM | RSS peak durante build + search | MB |
| Build time | Tiempo de construcción del índice | s |

### 4.2 Protocolo (alineado con ann-benchmarks y reproducible)

1. **Dataset**: cargar HDF5 oficial; usar 10,000 queries de `test` para medir; `train`
   para construir índice. No se particionan los datos — se usan los splits oficiales.
2. **Warmup**: 100 queries de descarte antes de medir (cache/CPU warm).
3. **Iteraciones**: 5 runs completos por configuración; se reporta mediana de los runs
   (robusto contra ruido). En cada run, las 10,000 queries se ejecutan serialmente,
   midiendo latencia individual.
4. **Configuraciones de VantaDB**: grid fijo de `M ∈ {16, 32}`, `ef_search ∈ {10, 50,
   100, 200}` — suficiente para trazar la curva Recall-QPS sin explosión combinatoria.
5. **Comparables**: `chromadb` (hnswlib detrás) y `lancedb` con sus defaults documentados;
   se barre `ef_search` equivalente donde la API lo permita. Se documenta la versión exacta
   de cada dependencia.
6. **Hardware (fijo y publicado)**: CPU 12-core @ 3.5GHz AVX2, 16GB RAM, Windows 11 /
   Ubuntu 22.04 LTS (el mismo perfil declarado en `PRODUCT.hardware` y `BENCH01.hardware`).
   Se publica en el JSON de resultados.
7. **Aislamiento**: sin otros procesos pesados durante la medición; reportar RSS peak por
   proceso medido con `resource.getrusage` (Linux) / equivalente.
8. **Reproducibilidad**: el harness fija semillas, versiones de paquetes (requirements
   pinned) y un `README` con instrucciones `pip install -r` + `python run_competitive_benchmark.py`.

### 4.3 Qué NO mide este benchmark (límites declarados)

- No mide ingestión competitiva (VantaDB ~5,400 vec/s está en BENCH01, no es comparable
  directo contra LanceDB que reporta 96k vec/s en batch con distinto hardware).
- No mide filtered search, multi-tenant, ni escala >1M vectores (fuera de perfil embebido).
- Las comparaciones son **single-node, in-memory, single-threaded** — la ventaja real de
  VantaDB (0 network hops, in-process) se captura en las métricas BENCH01 existentes, no
  en este harness.

---

## 5. Evaluación: Extender lo Existente vs Script Standalone

### 5.1 Extender `benches/` (12 criterio benches)

- **Ventajas**: integrado con `cargo bench`, sin dependencias nuevas.
- **Limitaciones (decisivas)**:
  - Los benches actuales usan **datasets sintéticos generados con `StdRng`** (ej.
    `hnsw_recall_ef.rs`: 10K vectores 128D aleatorios). No existe ningún benchmark que cargue
    HDF5 real, ni que ejecute sistemas rivales.
  - Ejecutar ChromaDB/LanceDB desde Rust es inviable (son librerías Python); el harness
    competitivo **debe** ser Python.
  - Un bench Criterion que compare contra librerías Python rompería la filosofía
    single-crate de `cargo bench` y el perfil de CI (compilación pesada + runtime largo).

### 5.2 Extender el web `/benchmarks`

- `BenchmarksView` + `BenchmarkRace` leen datos estáticos de `vanta-data.ts`
  (`BENCH01`, `SIFT1M`, `PRODUCT.metrics`). No hay fetching de datos — el web es 100%
  estático por decisión de arquitectura (ver `web/AGENTS.md`: "No data fetching").
- Conclusión: **el web no puede generar ni ejecutar benchmarks**, solo puede **presentar**
  resultados. La extensión válida del web es presentar una tabla alimentada por un JSON
  que un harness externo produce.

### 5.3 Veredicto

| Opción | ¿Viable para métricas competitivas honestas? | Recomendación |
|---|---|---|
| Integrar VantaDB a `ann-benchmarks` (PR) | Técnicamente sí, pero repo no mantenido; esfuerzo 3–5 días; single-CPU no refleja el motor | ❌ No |
| Extender `benches/` con datasets reales | Solo para baselines internos propios | ⚠️ Opcional (futuro) |
| **Harness standalone reproducible (Python)** | **Sí** — datos reales, rivales reales, protocolo idéntico, JSON de salida | ✅ **RECOMENDADO** |

**Recomendación mínima viable:** **script standalone** — un único
`benchmarks/competitive/run_competitive_benchmark.py` (nuevo directorio en el repo, NO
dentro de `benches/` Criterion) que:
1. Descarga/cachea los HDF5 (glove-100 + sift-128).
2. Construye índices en VantaDB (`vantadb-py`), `chromadb`, `lancedb`.
3. Corre el protocolo de la sección 4.
4. Escribe `benchmarks/competitive/results/competitive_benchmark.json` (métricas + hardware
   + versiones + fecha).
5. (Opcional) imprime tabla markdown para docs.

Esto **no toca `Cargo.toml`** (vantadb-py se instala vía pip, no como crate de dev) ni
`web/` (el web solo lee el JSON generado).

---

## 6. Propuesta de Implementación Mínima para la Página Pública (Slicing Vertical)

### 6.1 Principio

El web es estático por diseño; el slice entrega **una tabla presentada** + **un JSON
versionado** como fuente de verdad. Nada se genera en el navegador.

### 6.2 Slice 1 — Harness + JSON (backend de datos, sin web)

> **Estado 2026-08-04: IMPLEMENTADO.** El harness existe como `benchmarks/competitive_bench.py`
> (751 líneas) en el repo — no como directorio `benchmarks/competitive/` como proponía este doc.
> Baja datasets (glove/sift), corre ChromaDB/LanceDB vs VantaDB y emite métricas. Los archivos
> `benchmarks/competitive/run_competitive_benchmark.py` + `requirements.txt` + `README.md`
> descritos abajo son el **diseño original** (conceptual); el nombre de archivo real es
> `competitive_bench.py` y el README vive en `benchmarks/README.md`. Pendiente: emisión del JSON
> `competitive_benchmark.json` versionado como contrato con la web (ver tarea en Backlog).

- Nuevo: `benchmarks/competitive/run_competitive_benchmark.py`, `requirements.txt`
  (pinned: `vantadb-py`, `chromadb`, `lancedb`, `h5py`, `numpy`), `README.md` con
  instrucciones.
- Salida: `benchmarks/competitive/results/competitive_benchmark.json` con shape:

```json
{
  "generated": "2026-08-03",
  "hardware": "12-core CPU @ 3.5GHz, AVX2, 16GB RAM, Ubuntu 22.04",
  "versions": { "vantadb": "0.5.0", "chromadb": "x.y.z", "lancedb": "x.y.z" },
  "datasets": {
    "glove-100-angular": {
      "recall@10": { "vantadb": 0.99, "chromadb": 0.98, "lancedb": 0.98 },
      "qps": { "vantadb": 3200, "chromadb": 2500, "lancedb": 1800 },
      "p50_ms": { "vantadb": 0.31, "chromadb": 0.40, "lancedb": 0.55 },
      "p99_ms": { "vantadb": 1.10, "chromadb": 1.60, "lancedb": 2.20 },
      "rss_mb": { "vantadb": 780, "chromadb": 900, "lancedb": 640 }
    },
    "sift-128-euclidean": { "...": "..." }
  }
}
```

> Nota: los valores del JSON de ejemplo son placeholders — los reales salen de la primera
> ejecución del harness. **No publicar números inventados.**

### 6.3 Slice 2 — Tabla competitiva en el web (solo presentación)

- Nuevo data source `COMPETITIVE_BENCHMARK` en `vanta-data.ts` (o un `competitive-benchmark.json`
  importado estáticamente), reflejando el shape del JSON del harness.
- Componente nuevo `competitive-table.tsx`: tabla con filas = métrica × dataset, columnas =
  VantaDB / ChromaDB / LanceDB, estilo consistente con el design system (border-4 black,
  shadows rígidas). Reutiliza `CountUpStat` si aplica.
- Disclaimer visible: "Resultados reproducibles — ver `benchmarks/competitive/README.md`.
  Hardware: ...". Sin claims de "winner", solo números + método.
- Integración: render bajo `<BenchmarkRace />` en `page.tsx` de `/benchmarks`.

### 6.4 Slice 3 — CI opcional (no bloqueante)

- Workflow manual (`workflow_dispatch`) que corre el harness en Ubuntu, valida que el JSON
  no se haya degradado (guardrail de Recall@10 mínimo, ej. 0.95) y sube resultados al repo.

### 6.5 Qué NO se hace (por alcance)

- No se migra a VIBE (decisión futura, no en este INV).
- No se agrega Pinecone/Weaviate al harness (son cloud, requieren cuentas/API keys —
  el comparador embebido vs embebido es el honesto). Tabla cloud completa = MKT-15.
- No se modifican `BENCH01`/`SIFT1M`/`PRODUCT.metrics`.

---

## 7. Riesgos y Mitigaciones

| Riesgo | Mitigación |
|---|---|
| Números competitivos "malos" en RAM o QPS | Declarar límites (sección 4.3): VantaDB gana en 0 hops/latencia in-process, no en batch ingestión |
| ChromaDB/LanceDB cambian defaults entre versiones | Pinned requirements + versiones en el JSON |
| Hardware distinto entre runs | Publicar hardware en cada JSON; comparar solo runs del mismo harness |
| `ann-benchmarks` muerto | Usado solo como fuente de datasets y metodología; no como plataforma |
| Web estático no puede "refrescar" | El JSON se versiona en el repo; actualización = commit del nuevo JSON |

---

## 8. Gate 2026-08-03 — Diseño Puro, Sin Código (actualizado 2026-08-04)

| Chequeo | Estado |
|---|---|
| ¿Se modificó código fuente (`src/`)? | ❌ No |
| ¿Se tocó `Cargo.toml` o `Cargo.lock`? | ❌ No |
| ¿Se modificó `web/`? | ❌ No (Slice 2 de la tabla aún pendiente) |
| ¿Se crearon nuevos benches Criterion? | ❌ No |
| ¿Se escribió código del harness? | ✅ **SÍ — después del Gate**: `benchmarks/competitive_bench.py` (751 líneas) ya existe en el repo (verificado 2026-08-04) |
| ¿Fuentes web citadas? | ✅ Sí (sección 9) |
| Archivo de investigación creado | ✅ `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md` |

Este INV era **diseño puro** al 2026-08-03. Posteriormente se implementó **Slice 1 (harness)**
como `benchmarks/competitive_bench.py`. Pendiente: emisión del JSON contrato + **Slice 2**
(tabla competitiva en `web/`). Los placeholders del JSON de ejemplo (§6.2) nunca se publican.

<!-- Changed 2026-08-04: Gate re-verificado — Slice 1 ya existe como competitive_bench.py (751 L). -->

---

## 9. Fuentes Web Verificadas

1. **ann-benchmarks — GitHub README** (qué es, estructura de integración, datasets HDF5,
   single-CPU, estado "no longer actively maintained" → VIBE):
   https://github.com/erikbern/ann-benchmarks
2. **ann-benchmarks — web oficial** (Recall vs QPS, datasets, k=10):
   https://ann-benchmarks.com/index.html
3. **Paper ann-benchmarks** (metodología, interfaz BaseANN, Docker):
   Aumüller, Bernhardsson, Faithfull, *ANN-Benchmarks: A Benchmarking Tool for Approximate
   Nearest Neighbor Algorithms*, Information Systems 87 (2019),
   DOI 10.1016/j.is.2019.02.006 — https://arxiv.org/abs/1807.05614
4. **DeepWiki — Adding New Algorithms** (componentes `module.py`/`Dockerfile`/`config.yml`,
   interfaz BaseANN, CI, effort):
   https://deepwiki.com/erikbern/ann-benchmarks/4.1-adding-new-algorithms
5. **LanceDB Enterprise Benchmarks** (metodología p50/p90/p99, disclaimer de dependencia
   de hardware/warmup): https://docs.lancedb.com/enterprise/benchmarks
6. **LanceDB Performance docs**: https://docs.lancedb.com/performance
7. **SochDB benchmarks independientes** (comparativa práctica ChromaDB/LanceDB en mismo
   host, 10K×128d, metodología con hardware declarado):
   https://github.com/sochdb/sochdb-benchmarks
8. **EFFOMA — Vector Databases in 2026** (comparativa ChromaDB/LanceDB latency p50/p99,
   límites de benchmark warm-cache): https://effoma.com/blog/vector-database-performance-benchmark-comparison-2026/
9. **Lucene issue LUCENE-9937** (uso de ann-benchmarks como fuente de datos/validación por
   terceros, recall fudge factor, serial queries):
   https://issues.apache.org/jira/browse/LUCENE-9937
10. **AQR-HNSW arXiv (2026)** (sizes de SIFT-1M / GloVe-300D, 488MB / 114MB; valores de
    referencia de QPS/recall en la literatura):
    https://arxiv.org/pdf/2602.21600

---

## 10. Decisiones de Diseño Tomadas

1. **No publicar en `ann-benchmarks`** (repo no mantenido; esfuerzo 3–5 días injustificado);
   usar solo sus datasets HDF5 y su métrica Recall-QPS como metodología estándar.
2. **Harness standalone Python** sobre extensión de `benches/` — porque los rivales
   (chromadb/lancedb) son librerías Python y los benches Criterion usan datos sintéticos.
3. **Datasets glove-100-angular + sift-128-euclidean** — cubren cosine y euclidean, los
   dos paths del motor, con tamaños in-memory reproducibles en 16GB RAM.
4. **JSON como contrato entre harness y web** — respeta la arquitectura estática del web
   y hace la comparación auditable (fecha, hardware, versiones).
5. **Slicing 1 (harness) antes que 2 (tabla)** — sin datos reales no hay tabla que publicar;
   los placeholders no se publican.
6. **Sin cloud rivals (Pinecone/Weaviate)** en el harness — comparación embebido-vs-embebido
   honesta; cloud es MKT-15.
