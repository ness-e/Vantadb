# Review de módulo: `benches/`

> **Fecha:** 2026-08-23 · **Revisor:** ox-alpha (performance & observability)
> **Alcance:** 17 benches registrados en `Cargo.toml` L192–258 (`harness = false`) + `common/` + `data/`.
> **Skills:** performance-optimization, code-review-and-quality, ponytail-audit.

## Score general: **7.0 / 10**

| Dimensión | Nota | Comentario |
|---|---|---|
| Correctitud metodológica | 8.5 | `black_box` correcto en casi todos; `iter_custom` para inserts caros; ground truth por fuerza bruta real |
| Reproducibilidad | 7 | Dataset sintético determinista commiteado (seed fija); excepciones: `high_density` y fixtures con `rand::rng()` |
| Cobertura de features vendidas | 5.5 | Insert/search/HNSW-recall/filtros bien cubiertos; **WAL throughput y graph traversal ausentes** |
| Integración con regresión (CI) | 5.5 | 12/17 producen números criterion; 3 son solo-stdout; 1 tiene un brazo criterion **falso** |
| Higiene (ponytail) | 7 | Duplicación de `generate_vectors` ×8; escrituras fuera de tempdir en `high_density` |

---

## Tabla de veredictos

| Bench | Qué mide | Veredicto |
|---|---|---|
| `tokenizer_bench` | Costo del tokenizador avanzado (stemming/stopwords/unicode/multi-lenguaje) vs configs; ERR-044 compara analyzer fresh vs reutilizado | ✅ **Sólido.** Gateado tras feature `advanced-tokenizer`; sin ella el `main()` es no-op informativo (correcto). `black_box` correcto. El arm ERR-044 es el más valioso (evita pagar N builds por N textos). |
| `hybrid_queries` | BM25-only / vector-only / híbrido RRF filtrados vía API pública `VantaEmbedded` | ⚠️ **Correcto pero micro-escala.** Fixture de solo **96 documentos** — mide overhead de dispatch, no rendimiento a escala vendida. Sin recall (no valida que RRF fusiona bien). `black_box` correcto. Ampliar a ≥10k docs. |
| `high_density` | KNN 768d sobre 250k (CI) / 1M nodos (local) + fricción anti-spam | ⚠️ **Con salvedades.** Vectores con `rand::rng()` **sin seed** → no reproducible. Escribe `high_density_bench_db` en el CWD del repo (no tempdir); si crashea deja basura. `assert!(len <= 10)` como materialización es débil pero aceptable. Los comentarios `ponytail:` documentan reducción CI — bien. |
| `stress_test` | Point lookup existente vs rechazo por Bloom filter | ✅ **OK, nombre inflado.** No es stress: son 2 lookups point con perfil fijo. Útil para verificar que Bloom evita I/O; no estresa nada. Renombrar o ampliar. |
| `hnsw_pure` | Insert 10k×1536d y search sobre `CPIndex` puro in-memory | ✅ **Sólido.** Seed determinista, `iter_custom` excluye generación de vectores del timing, config HNSW fija. Es el micro-bench de referencia junto a `canonical_p99`. |
| `bench_concurrent` | QPS/p50/p99 de búsqueda concurrente read-only y mixta (1..16 threads) | ⚠️ **Útil pero frágil.** Accede a internos (`storage.hnsw.load()`, `vector_store[0].read()`) en vez de la API pública — se rompe con refactors de `StorageEngine`. Sin criterion groups → **sin gate de regresión**, resultados solo stdout. Duración fija (3s) con stop-flag: metodología válida. |
| `backend_compare` | Fjall vs RocksDB: insert single, get aleatorio, bulk throughput, distribución de latencia, RSS | ✅ **Bueno.** Mezcla criterio (insert/get) con timings ad-hoc impresos dentro del runner de criterion — funciona pero ensucia la salida del harness. `sample_size(10)` bajo aunque típico para I/O. Incluye memoria lógica + RSS físico. |
| `hnsw_recall_ef` | Trade-off ef_construction (build vs recall@10) y ef_search (recall vs p50/p99) | ✅ **Excelente.** Ground truth brute-force real, `AutoTune::set_ef(1)` para evitar contaminación del auto-tuner (comentario lo explica), barrido 10→400. Diseñado para comparar contra hnswlib. Este es el bench que sostiene el claim de HNSW. |
| `vfile_search` | Search in-memory vs con VantaFile (mmap) vs VantaFile compactado BFS | ✅ **Bueno.** A/B limpio de localidad de layout (DRV-130). Perfil `SearchProfile` vía logs para desglosar I/O vs compute. Detalle menor: el `TempDir` de la variante compactada se dropea al salir de `build_compacted()` (funciona porque el mmap mantiene el handle, pero es sutil). |
| `incremental_bench` | Modos de inserción Rebuild / Auto / Incremental por batch size (10–2000) + paridad de recall@10 | ✅ **Bueno.** Control explícito de las tres rutas; chequeo de calidad de índice post-insert (recall incremental vs rebuild). `skip_wal=true` deliberado para aislar el modo — correcto aquí, pero implica que **el costo WAL nunca se mide**. Bug menor de eficiencia: `recall_at_10` llama `generate_vectors(n)` dentro del loop (O(n²·dim)). |
| `batch_existing_check` | Costo del probe de existencia en `batch_insert` (ERR-037): fresh / overwrite / skip | ✅ **El mejor diseño experimental de la suite.** Brazos de control que aislan el costo del probe del estado de caché y del write-path. Documentación del escenario Hot-tier precisa. |
| `param_sweep` | Barrido M × ef_construction × ef_search sobre **SIFT-128 real** con ground truth | ⚠️ **Contenido excelente, integración nula.** Es un `main()` plano: **cero criterion groups** → los 120 configs producen solo stdout, sin gate de regresión ni artifact JSON. La cabecera promete p99 pero la tabla solo imprime p50. Requiere `scripts/download_ground_truth.py` + datos en `data/benchmark/sift-128/` (no versionados — documento el requisito, falta check amigable). |
| `acorn_filtered_search` | ACORN filtered navigation por selectividad (1%–100%) con recall@10 y p50/p99 | ⚠️ **Igual que param_sweep:** contenido bueno (ground truth filtrado real), pero `main()` plano sin criterion → sin regresión automática. Es el único bench de búsqueda filtrada — feature vendida cubierta solo aquí. |
| `sparse_hot_path` | Atribución del hot-path sparse: sort completo vs top-k parcial, serialización JSON, ListFloat | ✅ **Muy bueno.** Metodología AUDIT-02 impecable: mide candidatos reales materializados, replica exactamente el comparador de `planner::sort_hits`, documenta que inlinea helpers privados (ListFloat) en vez de falsear acceso. Honestidad ejemplar. |
| `ivf_bench` | IVF k-means: build por nlist, trade-off nlist×nprobe (recall, p50/p99, candidatos/query) | ✅ **Sólido.** Métrica extra de `candidates_scanned` (costo de escaneo real, análogo IVF del scan brute-force). El "scan floor" brute-force está etiquetado como estimación no cronometrada — no engañoso. |
| `canonical_p99` | **Contrato canónico Regla 9**: insert 100k×1536d + search, P99 | ✅ **Correcto.** `iter_custom` reconstruye el índice por iteración; histograma p50/p95/p99 calculado una vez fuera de la medición. Es el baseline de no-regresión declarado. Riesgo operativo: es pesado (100k×1536d ≈ 600 MB de RAM por build) — documentarlo. |
| `memory_budget` | Tendencia RSS del proceso vs tamaño de dataset bajo carga sostenida (FND-01) | ⚠️ **Valor real en stdout, criterion falso.** La tabla CSV RSS/lógico/pressure_ratio es valiosa y refleja la señal real del guard. Pero el único grupo criterion (`rss_vs_dataset_trend`) hace `black_box(inserted)` — **cronometra un entero**. Cualquier número criterion de este bench es ruido; debe marcarse o eliminarse. Escala reducible vía env (bien documentado). |

### Infraestructura compartida

- `common/mod.rs`: perfil fijo de medición (warm-up 3s / medición 5s / confianza 0.95 / ruido 0.05) + dataset sintético determinista **commiteado** (`data/synthetic_dataset.bin`, 2000×256 f32) con generador standalone (`gen_dataset.rs`) que reproduce byte-a-byte. ✅ Práctica ejemplar de reproducibilidad.
- `common` se compila en cada bench con `#![allow(dead_code)]` — pragmático, documentado.

---

## Evaluación de cobertura

### ¿Cubre las features vendidas?

| Feature vendida | ¿Cubierta? | Dónde | Brecha |
|---|---|---|---|
| HNSW ANN search | ✅ Fuerte | `hnsw_pure`, `canonical_p99`, `bench_concurrent`, `vfile_search` | — |
| HNSW **recall** (calidad) | ✅ Fuerte | `hnsw_recall_ef`, `param_sweep` (SIFT real), `acorn_filtered_search` | Ningún bench compara contra hnswlib **ejecutándolo** (solo preparado "for papers") |
| Búsqueda filtrada (ACORN) | ✅ | `acorn_filtered_search` | Solo este bench; sin criterion gate |
| **Hybrid RRF** | ⚠️ Débil | `hybrid_queries` | Fixture de 96 docs — toy. La cobertura seria a escala vive solo en el bench Python (ver reporte `benchmarks/`). En Rust no hay recall de RRF |
| **WAL throughput / políticas fsync** | ❌ **Ausente** | — | Ningún bench mide WAL: `skip_wal=true` en `incremental_bench` y en el techo de `batch_existing_check`. `memory_budget` lo incluye implícitamente pero nunca lo aísla. El pipeline WAL Durability (never/write/sync) **no tiene benchmark comparativo** |
| Graph traversal / GraphRAG | ❌ Ausente en `benches/` | `graphrag_bench.rs` vive en `benchmarks/` como *example*, y su fase de query está bloqueada por stack overflow (documentado) | BFS/DFS/traversal no tiene ningún micro-bench |
| Memoria (RSS, presupuesto) | ✅ | `memory_budget`, stats en `backend_compare` | El brazo criterion es falso; la tendencia real está en stdout |

### ¿Hay benchmarks mentirosos o cherry-picks?

No hay manipulación activa — la cultura post-MKT-18g se nota (comentarios de honestidad en `ivf_bench`, `sparse_hot_path`, `high_density`). Hallazgos de integridad:

1. **🔴 `memory_budget`: número criterion fabricado.** `rss_vs_dataset_trend` mide `black_box(inserted)` — un contador. Si alguien cita ese número, está citando ruido.
2. **🟡 Tres benches sin gate:** `param_sweep`, `acorn_filtered_search`, `bench_concurrent` imprimen a stdout. Sus números no pueden ser cherry-picked silenciosamente (no hay artifact), pero tampoco pueden detectar regresiones. Claims basados en ellos dependen 100% de que alguien corra y pegue la salida.
3. **🟡 `hybrid_queries` a 96 docs:** si se cita como evidencia de performance híbrida, es un claim sin sustento de escala.
4. **🟡 `high_density` sin seed:** resultados no reproducibles run-to-run; no usar para comparaciones antes/después.
5. ✅ Lo positivo: `iter_custom` excluye setup donde corresponde, `AutoTune::set_ef(1)` evita contaminación del tuner, ground truths son brute-force reales, y los controles negativos (`skip_fresh`, bloom-miss) existen.

### ¿Falta algo crítico?

1. **WAL throughput bench** (crítico para el rol de durability): comparar políticas never/write/sync y WAL on/off. Hoy es imposible afirmar el costo de durabilidad.
2. **Graph traversal bench**: BFS/DFS sobre grafos de N nodos/aristas. El GraphRAG es feature vendida y su única herramienta de medición está rota (stack overflow) y fuera de `benches/`.
3. **Hybrid RRF a escala + recall de fusión**: ampliar `hybrid_queries` a ≥10k docs y validar orden de fusión.
4. **Latency percentiles**: ✅ bien cubiertos (p50/p95/p99 en 8+ benches). Sin gap.
5. **Memory profiling**: tendencia cubierta; falta un heap-profile puntual (`dhat-rs`) para atribuir RSS interno, pero no es bloqueante.

---

## Ponytail-audit (higiene)

- `shrink:` duplicar `generate_vectors` 8 veces con firmas ligeramente distintas → mover a `common/mod.rs`. [benches/*.rs]
- `delete:` brazo criterion falso de `memory_budget` (`rss_vs_dataset_trend`). Reemplazo: nada (el valor ya está en stdout) o un `b.iter` real sobre `get_memory_stats`. [benches/memory_budget.rs:147]
- `yagni:` `mod common;` en `canonical_p99.rs` solo usa `apply_fixed_profile` — fine, pero `hnsw_pure.rs` declara `common` y no lo usa en absoluto. [benches/hnsw_pure.rs:7]
- `native:` `stress_test` podría usar `BatchSize` estándar en vez de inyección secuencial de 100k–1M nodos dentro del bench function (bloquea el runner). [benches/stress_test.rs:30]
- `shrink:` `recall_at_10` regenera todo el dataset por query → pre-generar una vez. [benches/incremental_bench.rs:65]

Net posible: ~−60 líneas, 0 deps.

---

## Acciones recomendadas (priorizadas)

1. **[Crítico]** Eliminar o arreglar el brazo criterion falso de `memory_budget`.
2. **[Crítico]** Crear `wal_throughput` bench: WAL on/off × política fsync (pipeline de durabilidad lo necesita).
3. **[Alto]** Convertir `param_sweep` y `acorn_filtered_search` en criterion groups (o exportar JSON de resultados para el gate nocturno).
4. **[Alto]** Semillar `high_density` y mover su DB a tempdir.
5. **[Medio]** Ampliar `hybrid_queries` a escala real + métrica de recall RRF.
6. **[Medio]** Graph traversal micro-bench (desbloqueando primero el stack overflow de GraphRAG).

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| `memory_budget`: número criterion fabricado (cronometra `black_box(inserted)`) | **MOD-51** |
| WAL throughput bench ausente (políticas never/write/sync sin medir; `skip_wal=true` en los benches existentes) | **MOD-52** |
| Tres benches solo stdout sin gate de regresión (`param_sweep`, `acorn_filtered_search`, `bench_concurrent`) | **MOD-53** |
| Cobertura débil: hybrid RRF a 96 docs (toy) + graph traversal sin ningún micro-bench | **MOD-54** |
| Nits: seeds faltantes (`high_density`, 2 más), DB fuera de tempdir, duplicación `generate_vectors` ×8 | **MOD-55** |
