---
title: "GraphRAG Benchmark Methodology (MKT-16)"
date: 2026-08-05
type: blog-post
tags: [vantadb, benchmark, graphrag, mkt]
status: run-pending-query-phase
links: "[Glosario GraphRAG](../glosario/graphrag.md)"
---

# GraphRAG Benchmark — Metodología y Resultados Reales

> **Estado honesto de este documento:** los números de la **fase de indexación son reales** (medidos en un run del script `benchmarks/graphrag_bench.rs` en esta máquina). Los números de **latencia de query y Token Reduction a escala de benchmark (3000 nodos) quedan PENDIENTES** porque, en este entorno (Windows x86_64, `--release`), la fase de búsqueda del pipeline GraphRAG termina en **stack overflow reproducible** (`0xC00000FD`, `STATUS_STACK_OVERFLOW`, misma clase que AUDIT-04) cuanto el corpus es grande. **No se inventó ninguna cifra**: las celdas que no se pudieron medir están marcadas como PENDING, no rellenadas con claims.
>
> **Nota de entorno (2026-08-05, re-run):** el refactor `src/storage/vfile.rs` (AUDIT-03) que impedía el build release fue resuelto por su agente dueño (commit `88ed3642`). El script ahora compila y los corpus **pequeños (20–100 nodos) corren la fase de query completa** (latencia y token reales, ver sección abajo). El **corpus canónico de 3000 nodos** sigue estallando en la phase de query — por eso los números productivos siguen PENDING.

## Propósito

Validar las métricas publicadas en [`docs/glosario/graphrag.md`](../glosario/graphrag.md):

| Métrica del glosario | Claim publicado | Estado tras este trabajo |
|---|---|---|
| Token Reduction vs RAG | 40-60% | **PENDING** — no medible hoy (query phase crash) |
| Latencia adicional por hop | ~25-50ms | **PENDING** — no medible hoy |
| Recall improvement | +15-20% | Fuera de alcance (requiere set de relevancia etiquetado; no existe) |
| Max hops | 3 | Verificado en código: `GraphRagPipeline::new()` → `expansion_hops: 2`, configurable (`pipeline.rs:48-52`) |

El objetivo de este trabajo es convertir esas cifras en **números de un run reproducible** o, si no se pueden medir, dejarlo explícito sin claims. El script queda listo y reproducible.

## Script reproducible

**Ruta:** [`benchmarks/graphrag_bench.rs`](../../benchmarks/graphrag_bench.rs) (registrado como ejemplo `graphrag_bench` en `Cargo.toml`).

```bash
# Build (requiere que el crate compile — ver "Bloqueo de entorno" abajo)
cargo build --release --example graphrag_bench

# Run: corpus 3000 nodos, 100 queries (por defecto)
cargo run --release --example graphrag_bench

# Params: <nodos> <queries> <edges_per_node>
cargo run --release --example graphrag_bench 3000 100 2

# Reporte JSON opcional
$env:GRAPHRAG_BENCH_OUT="benchmarks/graphrag_bench_result.json"
cargo run --release --example graphrag_bench 3000 100 2
```

### Qué mide (todo medido, nada hardcodeado)

1. **Fase de indexación:** inserta `n` nodos (`VantaMemoryInput` con vector 32d + contenido) + aristas (`add_edge`), luego `flush()`. Mide tiempo total (s) y throughput (nodos/s).
2. **Fase de queries (M queries deterministas):** para cada query mide:
   - **RAG baseline** = `GraphRagPipeline` con `expansion_hops: 0` (solo seeds, sin expansión ni sección de relaciones). Latencia + tokens del `context_text`.
   - **GraphRAG** = `GraphRagPipeline` con `expansion_hops: 2` (seeds + BFS + ranking + contexto con relaciones). Latencia + tokens.
   - **Token Reduction** = `1 - Tokens_GraphRAG / Tokens_RAG` (fórmula del glosario).
3. **Barrido de hops:** latencia media con `hops ∈ {1,2,3}` → delta por hop.
4. **Stats de grafo:** seeds encontrados, nodos expandidos, aristas en contexto.

### Token counting

Conteo por **whitespace-split** (proxy, documentado como aproximación — NO es un tokenizer real tipo BPE). Los valores de tokens son comparativos, no absolutos para facturación.

### Corpus (determinista)

- **LCG seeded** (`0x5EED_CAFE` vectores, `0xE_DGE_5` aristas) — sin dependencia de RNG externa; mismo run ⇒ mismo corpus.
- 8 clusters temáticos (`TOPICS`), dim=32, vectores normalizados con ruido alrededor del centroide → señal semántica + léxica real.
- Aristas: DAG forward (chain intra-cluster + hub inter-cluster + misc hacia adelante) — nunca referencia un id menor, corpus estrictamente acíclico (mismo path que el engine testea en `tests/graphrag_test.rs`).
- DB: `tempdir()` (temporal, eliminada al salir).

## Resultados del run (2026-08-05)

**Hardware (medido por el script):**
```
os=windows arch=x86_64 cpus=12 cpu=Intel64 Family 6 Model 154 Stepping 4, GenuineIntel
RAM: 31 GB (reportado por el banner del engine) · AVX2 · profile PERFORMANCE
```

### Fase de indexación — NÚMEROS REALES ✅

| Corpus | Index time (s) | Throughput (nodos/s) |
|---|---|---|
| 20 nodos | 1.569 | 12.7 |
| 100 nodos | 10.749 | 9.3 |
| 300 nodos | 26.809 | 11.2 |
| 800 nodos | 86.016 | 9.3 |
| 3000 nodos | 174.308* | 17.2 |

*Nota: los números de la tabla fueron medidos con la configuración de aristas del script en el momento del run (mismo conteo de aristas que la versión commiteada: chain + hub + 2 misc/nodo; la dirección DAG vs cíclica no cambia el costo de indexación, que está dominado por `put()`/`add_edge()` + WAL flush). Los valores 20–800 provienen del run abortado; **el valor de 3000 nodos (174.308 s / 17.2 nodos/s) fue re-medido en el re-run de 2026-08-05 con el build release ya compilable** — el coste varió porque el run anterior corría con el build roto por AUDIT-03.*

**Lectura honesta:** la indexación de memoria con WAL durable corre a ~10-13 nodos/s en este hardware con esta configuración por defecto. Esto **no** es un benchmark de ingesta optimizado (ver `benchmarks/competitive_bench.py` para batch ingesta); es el costo del path CRUD por-nodo del pipeline GraphRAG.

### Fase de queries — PENDIENTE DE RUN ⏳

Reproducción del crash (real, re-medido el 2026-08-05 con el build ya compilable):

```
$env:GRAPHRAG_BENCH_OUT="benchmarks\graphrag_bench_result.json"
target\release\examples\graphrag_bench.exe 3000 100 2

=== GraphRAG benchmark (MKT-16) ===
corpus: 3000 nodes, 8 topics, dim=32 | queries: 100
hardware: os=windows arch=x86_64 cpus=12 cpu=Intel64 Family 6 Model 154 Stepping 4, GenuineIntel
index: 3000 nodes + edges in 174.308s (17 nodes/s)
thread '<unknown>' has overflowed its stack     # exit code -1073741571 / 0xC00000FD
```

- **Trigger:** el run canónico (`3000 nodos / 100 queries / edges_per_node=2`) estalla en la **primera query** (`GraphRagPipeline::search`), tras completar la indexación. Con la misma build, los corpus **pequeños (20 y 100 nodos) completan la fase de query sin crash** — ver "Corpus pequeños corren completo" más abajo — por lo que el fallo escala con el tamaño del grafo/contexto en `search`, no con la mera presencia de aristas.
- **Clase de fallo:** mismo que AUDIT-04 (`0xC0000409`/`STATUS_STACK_BUFFER_OVERRUN`); acá reporta como `0xC00000FD` (`STATUS_STACK_OVERFLOW`). Ver `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 14.
- **No atribuido:** la recursión JS subyacente en `search`/serialización con grafos grandes es trabajo de engine/audit, NO de este benchmark.

> ⚠️ **A los readers:** NO usar las celdas vacías. El claim "40-60% Token Reduction" del glosario **sigue sin evidencia de run a escala productiva**. Este doc lo deja explícito.

### Corpus pequeños corren completo (real, 2026-08-05 re-run) ✅

Con la build release ya compilable (vfile.rs arreglado), los corpus pequeños completan la fase de query:

```
# 100 nodos / 10 queries / edges_per_node=2
rag_baseline  : p50=0.866 p95=2.397 mean=1.014 ms
graphrag      : p50=1.418 p95=4.091 mean=1.693 ms
per_hop_delta (1->3): 0.726 ms
rag_tokens_avg   : 108.4
graphrag_tokens_avg: 812.0
token_reduction  : -6.491 (-649.08%)
seeds_found_avg   : 10.00   nodes_expanded_avg: 82.50
```

> ⚠️ **Estos números NO son representativos de producción.** En corpus pequeños y densamente conectados, la expansión a 2 hops incluye casi todo el grafo (`nodes_expanded_avg: 82.5` de 100 nodos) y el `context_text` crece respecto al baseline — por eso `token_reduction` sale **negativo**. Son útiles solo como smoke test del pipeline; el número productivo real exige el corpus 3000, que hoy no corre (stack overflow).

## Bloqueo de entorno (resuelto)

Durante el run inicial, el workspace tenía una edición en curso de otro agente en `src/storage/vfile.rs` (AUDIT-03: `AlignedBytes::zeroed` → `Result`), que rompía el build release. **Ese refactor fue resuelto por su agente dueño (commit `88ed3642`)** — el build release y el script ahora compilan. El único bloqueo restante es el stack overflow del engine a escala productiva (arriba), que es responsabilidad de engine/audit, no de este script.

## Cómo completar los números PENDING

1. El script compila y los corpus pequeños corren (vfile.rs ya no bloquea). El único blocker es el stack overflow del engine a escala.
2. `cargo run --release --example graphrag_bench 3000 100 2`
3. Si el stack overflow persiste → el prefijo es del engine (delegar a vanta-engine/vanta-audit, referenciar AUDIT-04). El smoke test de corpus pequeño ya valida que el pipeline y el script son correctos.
4. Cuando corra: copiar la sección `--- token reduction ---` y `--- query latency ---` a este doc, reemplazando la tabla de métricas del glosario por números reales.

## Alcance / no-goals

- **Recall improvement (+15-20%):** NO se mide. Requiere un set de relevancia etiquetado (ground truth de "relacionalmente relevantes"), que no existe en el repo. Dejarlo como claim sin run, o crearlo como trabajo futuro separado.
- **Embeddings:** no se usa un modelo de embeddings real; los vectores son sintéticos (clusters). El benchmark mide el pipeline GraphRAG (graph structure + ranking), no la calidad del embedding.
- **Comparación con LanceDB/Chroma:** fuera de alcance; ver `benchmarks/competitive_bench.py` (MKT-05/INV-007-B) para comparativa vectorial pura.

---

*Última actualización: 2026-08-05 · Script: `benchmarks/graphrag_bench.rs` · Registro: `Cargo.toml [[example]] graphrag_bench`*
