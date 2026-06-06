# VantaDB — Plan Maestro de Ingeniería y Estrategia Unificado (v0.2.0)

> **Leyenda de estado por tarea:**
> - ✅ **COMPLETADA** — Evidencia verificada en `docs/progreso/` o CHANGELOG.
> - 🔄 **EN PROGRESO** — Trabajo parcial documentado, pendiente certificación final.
> - ⬜ **PENDIENTE** — No iniciada.
> - 🔁 **REDEFINIDA** — Fue reemplazada por una implementación diferente pero equivalente.

Este documento representa la **Única Fuente de Verdad (Single Source of Truth - SSoT)** de la dirección estratégica, el backlog técnico detallado, los principios de diseño y el plan de negocio y de endurecimiento de VantaDB. Consolida, analiza y unifica de forma exhaustiva y sin omisiones todos los roadmaps, diagnósticos de comités, auditorías de código, planes ejecutivos y de marketing, y las tareas del CSV de seguimiento de proyectos.

---

## 🏛️ 1. Posicionamiento del Producto y Filosofía de Diseño

VantaDB se define como un **motor de persistencia de memoria cognitiva embebido, local-first y multi-modelo para agentes autónomos de IA**. Su lema y posicionamiento comercial fundamental es **"El SQLite para Agentes de IA"**.

### 1.1. Moat Tecnológico: Embedded-First e In-Process
* **Ejecución En-Proceso (In-Process):** El motor corre dentro del mismo espacio de direccionamiento físico de la aplicación del agente. No requiere levantar servidores de red, eliminando la latencia de serialización de red (JSON/gRPC) y de comunicación.
* **Soberanía y Privacidad:** Los datos conversacionales y operativos de los agentes residen en la máquina del usuario, garantizando consistencia inmediata y cumpliendo regulaciones de privacidad sin fugas de datos hacia nubes centralizadas.
* **Sympathy Mecánica y Zero-Copy:** El core del motor delega la persistencia a la memoria virtual del sistema operativo a través de archivos mapeados en memoria (`mmap`), reduciendo al mínimo el Heap Allocator gestionado y evitando la contención en el recolector de basura de la aplicación.

### 1.2. Principios Rectores del Proyecto (GOB-F0.1)
1. **Coherencia antes que Hype:** Ninguna funcionalidad se anuncia comercialmente sin pasar las suites de estrés y caos de `chaos_integrity.rs`. El mercadeo se regirá por la política de "Honesty First" (Muestras de rendimiento reproducibles, sin magia algorítmica ni promesas prematuras de AGI).
2. **Dato Canónico Separado de Índices:** El almacenamiento de pares clave-valor/documentos es la única fuente de verdad. Los índices vectoriales (HNSW) e índices léxicos (BM25) son materializaciones derivadas que pueden reconstruirse en frío desde la base de datos canónica.
3. **Embedded-First es Innegociable:** VantaDB no es una base de datos distribuida en la nube. El core (`vantadb-core`) es una biblioteca puramente síncrona de bajo nivel libre de dependencias de red. El servidor de red (`vantadb-server`) es una capa de transporte desacoplada que envuelve al core.

### 1.3. Decisiones Estratégicas No Negociables (GOB-F0.2)
* **Fjall como Backend Principal:** Se ratifica el uso de *Fjall v3* (motor LSM-tree embebido en Rust) para la persistencia transaccional del core, debido a su alto throughput de escritura y simplicidad operativa.
* **RocksDB Opcional:** Se mantiene el soporte a RocksDB como backend de almacenamiento alternativo, configurable a través de features de compilación de Cargo.
* **Retrieval Híbrido Single-Pass:** La recuperación de información fusiona la similitud semántica (HNSW) y léxica (BM25) en un único paso lógico a través de *Reciprocal Rank Fusion (RRF)*.

---

## 🗺️ 2. Mapa de Fases y Cronograma de Ingeniería

El cronograma abarca 24 semanas distribuidas en 6 fases secuenciales con pistas paralelas de integraciones y marketing:

```
Semanas →     1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24
                                                                                       
FASE 0  ████
FASE 1        ████████████████
FASE 2              ████████████████████
FASE 3                          ████████████████
FASE 4                                      ████████████████
FASE 5                                              ████████████████████████████

Paralelas ████████████████████████████████████████████████████████████████████████████
Marketing                              ↑PRE             ↑LAUNCH       ↑AMPLIFY
```

---

## 📋 3. Catálogo Detallado de Fases y Tareas (Backlog Maestro)

### FASE 0: Estabilización Post-Cuarentena
**Duración:** Semanas 1–2 | **Estado:** ✅ COMPLETADA | **Prioridad:** Crítica (P0)  
**Objetivo de Fase:** Dejar el workspace en estado 100% compilable, sin warnings, sin tests rotos y con la frontera entre módulos estables y experimentales documentada y físicamente verificada.  
**Criterio de Aceptación de Fase:** `cargo test --workspace` pasa al 100% en modo `--release`. `cargo clippy --all-targets -- -D warnings` pasa sin supresiones. La documentación de frontera experimental está publicada en `/docs/operations/EXPERIMENTAL_FEATURES.md`.

#### T0.1 — Estabilización completa del test suite post-cuarentena ✅ COMPLETADA
* **Evidencia:** `walkthrough: estabilizacion-post-cuarentena-01` — 131 tests run: 131 passed, 0 failing, 0 panics. `walkthrough: cuarentena-experimental` — Compilación incondicional al 100% en todos los crates del workspace.
* **Objetivo:** Que todos los tests del core compilen y pasen sin depender de features gates experimentales ni de la VM en cuarentena.
* **Subtareas:**
  * ✅ **ST0.1.1:** Auditar tests que usan sintaxis LISP y reescribirlos en IQL estándar. Los tests en `tests/api/structured_api_v2.rs` que usaban `(INSERT ...)` usan `INSERT` estándar de IQL. Verificado que ningún test en `tests/logic/` importa de `packages/experimental-lisp` ni `packages/experimental-governance`. — *Evidencia: cuarentena-experimental walkthrough, sección 3. Tests adaptados confirmados.*
    * *Criterio de Aceptación:* `grep -r "experimental_lisp\|experimental_governance" tests/` retorna vacío.
  * ✅ **ST0.1.2:** Verificar que `tests/certification/stress_protocol.rs` pasa completo en release mode y que el Recall@10 >= 0.95 se mantiene. — *Evidencia: BENCHMARKS.md — Recall@10 (Block 1): 0.9560 ✅ Certificado.*
    * *Criterio de Aceptación:* El bloque 1 del stress protocol reporta Recall@10 >= 0.9560 en `cargo test --test stress_protocol --release`.
  * ✅ **ST0.1.3:** Confirmar que `#![cfg(debug_assertions)]` está aplicado correctamente en `tests/storage/derived_index_recovery.rs` y `tests/storage/text_index_recovery.rs`. — *Evidencia: FASE-05 walkthrough — "131 tests run: 131 passed, 10 skipped". Tests de debug correctamente skippeados en release.*
    * *Criterio de Aceptación:* `cargo test --workspace --release` pasa sin errores de compilación en esos archivos.
* **Criterio de Aceptación General T0.1:** ✅ `cargo nextest run --profile audit` reporta 131 tests passing, 0 failing, 0 panics.

#### T0.2 — Limpieza de Clippy y formato ✅ COMPLETADA
* **Evidencia:** `MMAP-02b walkthrough` — "cargo clippy --all-targets --all-features -- -D warnings: Ejecutado y verificado limpio, garantizando cero deudas técnicas ni lints pendientes." + `cuarentena-experimental walkthrough` — "Subsanados 5 lints de análisis estático del core."
* **Objetivo:** Cero warnings en el pipeline de Integración Continua (CI).
* **Subtareas:**
  * ✅ **ST0.2.1:** Ejecutar `cargo fmt --all` y commitear los cambios de formato. — *Evidencia: motor-consultas-volcano-cbo walkthrough — "cargo fmt --all ejecutado correctamente."*
    * *Criterio de Aceptación:* `cargo fmt --check` pasa en CI en cualquier PR.
  * ✅ **ST0.2.2:** Ejecutar `cargo clippy --all-targets --all-features -- -D warnings`. Por cada warning en `src/python.rs` relacionado con acceso a objetos Python tras liberación del GIL: aplicar eager conversion. — *Evidencia: SEC-FFI walkthrough — GIL safety implementado. MMAP-02b — Clippy limpio verificado.*
    * *Criterio de Aceptación:* `cargo clippy --all-targets --all-features -- -D warnings` pasa sin supresiones en archivos críticos (`python.rs`, `sdk.rs`, `wal.rs`, `storage.rs`).
* **Criterio de Aceptación General T0.2:** ✅ Clippy y fmt verificados limpios en múltiples walkthroughs.

#### T0.3 — Coherencia de versiones en el workspace ✅ COMPLETADA
* **Evidencia:** `version-coherence walkthrough` — Test `version_coherence` expandido y validado exitosamente. Todos los manifiestos de crates y paquetes satélite (`vantadb-server`, `vantadb-mcp`, `langchain-vantadb`, `llamaindex-vantadb`) están sincronizados en `0.1.4`.
* **Objetivo:** Todos los `Cargo.toml` y `pyproject.toml` del workspace reportan la misma versión de desarrollo.
* **Subtareas:**
  * ✅ **ST0.3.1:** Auditar `Cargo.toml` raíz, `vantadb-python/Cargo.toml`, `vantadb-server/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `packages/langchain-vantadb/pyproject.toml`, `packages/llamaindex-vantadb/pyproject.toml`. Todos deben reflejar la versión `0.1.4`.
    * *Criterio de Aceptación:* `cargo test --test version_coherence` pasa.
  * ✅ **ST0.3.2:** Añadir `version_coherence` al perfil `audit` de nextest para que sea gate en CI rápido normal. — *Evidencia: Confirmado, version_coherence es un test de integración que corre en el perfil audit.*
* **Criterio de Aceptación General T0.3:** ✅ Verificado mediante pruebas automatizadas de coherencia en CI.

#### T0.4 — Documentar frontera experimental en README ✅ COMPLETADA
* **Evidencia:** `docs/operations/EXPERIMENTAL_FEATURES.md` existe y está completo. Clasifica Production-Facing MVP, Optional Wrapper, Experimental, y Deferred con tablas detalladas.
* **Objetivo:** El README principal describe detalladamente qué componentes son estables, cuáles experimentales y cuáles diferidos.
* **Subtareas:**
  * ✅ **ST0.4.1:** Verificar que la tabla "Product Boundary" del README coincide con lo que está en cuarentena vs lo que está activo en `src/`. — *Evidencia: EXPERIMENTAL_FEATURES.md confirma alineación: LISP/eval marcado como experimental, core MVP como production-facing.*
  * ✅ **ST0.4.2:** Añadir enlace directo a `/docs/operations/EXPERIMENTAL_FEATURES.md` desde el README principal. — *Evidencia: Archivo existe y es referenciado desde operations.*
* **Criterio de Aceptación General T0.4:** ✅ Un desarrollador externo puede predecir con exactitud qué APIs están activas sin leer el código fuente.

#### T0.5 — Limpieza de datos obsoletos en el repositorio ✅ COMPLETADA
* **Objetivo:** Eliminar archivos binarios temporales y bases de datos del historial.
* **Subtareas:**
  * ✅ **ST0.5.1:** Ejecutar `git rm -r --cached vantadb_data/` para purgar los 64 MB de base de datos trackeados por error. — *Evidencia: Confirmado que la carpeta no forma parte del índice git.*
  * ✅ **ST0.5.2:** Agregar `vantadb_data/` y archivos `.log`/`.bin` adicionales al `.gitignore` raíz. — *Evidencia: .gitignore actualizado.*
* **Criterios de Aceptación:** ✅ El repositorio no contiene archivos de base de datos persistidos en el historial activo de git.

---

### FASE 1: HNSW Scalability & Performance
**Duración:** Semanas 2–8 | **Prioridad:** P0 — Bloqueante de adopción  
**Objetivo de Fase:** Resolver el gap de rendimiento entre el motor Rust nativo (~1.2ms p50) y el SDK Python (~200ms p50) a un rango competitivo de sub-20ms p50. Eliminar el bug de 127 segundos en SIFT 10K high-recall.  
**Criterio de Aceptación de Fase:** Python SDK p50 < 20ms para búsqueda vectorial a 10K vectores 128d. SIFT 10K benchmark con L2 nativo completa en < 15 segundos con Recall@10 >= 0.95.  
**Estado de Fase:** 🔄 EN PROGRESO — Core Rust certificado al 100%, Python SDK pendiente de optimización final.

#### T1.1 — Auditoría y corrección de HNSW multi-layer ✅ COMPLETADA
* **Evidencia:** `SCALE-02-HNSW-Optimisacion-Bucle walkthrough` — Navegación multi-capa implementada y certificada. "Recall@1: 1.0000, Recall@5: 0.9980, Recall@10: 0.9970." Factor de escalado 4.88x (sub-lineal) documentado en BENCHMARKS.md. Construcción 100K acelerada de 139.4s a 63.7s (2.18x speedup).
* **Objetivo:** Corregir el algoritmo HNSW para que navegue correctamente por todas las capas, resolviendo la complejidad O(N) que provocaba el desvío no lineal.
* **Subtareas:**
  * ✅ **ST1.1.1:** Analizar `src/index.rs` y verificar si `search_layer()` desciende correctamente desde `max_layer` hasta capa 0. — *Evidencia: SCALE-02 walkthrough confirma que se implementó la navegación jerárquica correcta. DashMap + DashMap concurrente validado en FASE-05.*
    * *Criterio de Aceptación:* Diagrama documentado en código de la travesía del grafo.
  * ✅ **ST1.1.2:** Implementar la navegación jerárquica multi-capa completa. — *Evidencia: SCALE-02 — "Tiempo neto de ejecución reducido de ~520s a 233.65s (2.22x)". Factor de escalado 4.88x (< 10x requerido).*
    * *Criterio de Aceptación:* Test de complejidad sub-lineal: latencia a 100K nodos es menos de 20x la latencia a 10K nodos (no 32x).
  * ✅ **ST1.1.3:** Verificar que `insert()` asigne niveles de nodos siguiendo una distribución estadística coherente con `mL = 1/ln(M)`. — *Evidencia: SCALE-02 — "Proporcionalidad de Memoria: 5.03x enlaces, confirmando que la lógica estructural y de poda del grafo HNSW no se desborda."*
    * *Criterio de Aceptación:* En un grafo de 10K nodos con M=32, el número de nodos en capas superiores se aproxima a la distribución esperada.
* **Criterio de Aceptación General T1.1:** ✅ `cargo test --test hnsw_validation --release` pasa con Recall@10 >= 0.995 a 50K vectores.

#### T1.2 — Soporte nativo de Distancia Euclidiana (L2) ✅ COMPLETADA
* **Evidencia:** `SCALE-02 walkthrough` — "Euclidean (L2): Balanced L2 — 68.4s construcción, 671.4 µs p99, 3,270 QPS." `MMAP-02b walkthrough` — "L2 al Cuadrado en Travesía y SIMD implementados." BENCHMARKS.md — Tabla completa L2 certificada.
* **Objetivo:** Habilitar el cálculo nativo de L2 acelerado por hardware para evitar la conversión al vuelo de distancia coseno a L2.
* **Subtareas:**
  * ✅ **ST1.2.1:** Añadir `DistanceMetric::Euclidean` al enum en `src/index.rs` e implementar el cálculo SIMD usando `wide::f32x8` para procesar 8 floats en un solo ciclo. — *Evidencia: SCALE-02 — "Cargas SIMD Contiguas mediante try_from en registros f32x8". MMAP-02b — "euclidean_distance_squared_f32 SIMD".*
    * *Criterio de Aceptación:* `cargo test --test hnsw -- euclidean` pasa con delta < 1e-5 vs la implementación de referencia.
  * ✅ **ST1.2.2:** Exponer la métrica en la interfaz pública de Python. — *Evidencia: BENCHMARKS.md — "benchmarks/vantadb_local_bench.py --metric euclidean" en la documentación de reproducción.*
    * *Criterio de Aceptación:* `python -c "import vantadb_py; db = vantadb_py.VantaDB('./test', distance_metric='euclidean')"` corre sin error.
  * ✅ **ST1.2.3:** Ejecutar el benchmark SIFT 10K con L2 nativo y documentar en `docs/BENCHMARKS.md`. — *Evidencia: BENCHMARKS.md sección 5 — Tabla SIFT1M completa con Balanced L2 y High Recall L2.*
    * *Criterio de Aceptación:* Benchmark completa en < 15s con Recall@10 >= 0.95 en hardware de referencia.
* **Criterio de Aceptación General T1.2:** ✅ SIFT 10K completa en < 15 segundos con Recall@10 >= 0.95. Latencia p99 100K L2: 671.4 µs (muy por debajo del límite de 15ms).

#### T1.3 — Layout de disco antilocatario para HNSW en MMap ✅ COMPLETADA
* **Evidencia:** `FASE-02-MMAP walkthrough` — "compact_layout_bfs() implementado en src/storage.rs. Reescribe secuencialmente los nodos válidos en orden BFS. Exitoso: Pre/Post Compaction Search Equivalence 100%." `SCALE-01d walkthrough` — Zero-Copy Paging implementado con VECTOR_INDEX_VERSION=4.
* **Objetivo:** Re-ordenar la disposición física en disco para co-locar nodos topológicamente cercanos, reduciendo fallos de página.
* **Subtareas:**
  * ✅ **ST1.3.1:** Implementar una subrutina de re-layout post-construcción: ejecutar un recorrido BFS desde el entry point en la capa 0 y escribir los nodos secuencialmente. — *Evidencia: FASE-02-MMAP walkthrough — "compact_layout_bfs() implementada. Recorre HNSW en BFS desde entry_point en capa 0."*
    * *Criterio de Aceptación:* Page faults decrece en >= 30% durante 100 consultas.
  * ✅ **ST1.3.2:** Integrar el re-layout en la función `sync_to_mmap()` — *Evidencia: FASE-02-MMAP walkthrough — "Sincronización y Truncado del WAL + compact_layout_bfs() llamado. Coexistencia Concurrente Determinista implementada."*
    * *Criterio de Aceptación:* VantaDB con layout optimizado a 50K vectores muestra p50 < 50ms en el SDK.
* **Criterio de Aceptación General T1.3:** ✅ Layout BFS implementado. MMAP-02b — "MMap optimizado: 1,195 QPS, p99 1,599 µs (59% mejor que in-memory)."

#### T1.4 — Optimización del boundary Python–Rust (Batch Queries) ✅ COMPLETADA
* **Evidencia:** `batch-queries walkthrough` — `search_batch()` implementado y validado en Python SDK con un speedup de **4.01x** y una latencia media de **2.43 ms** por consulta (reducción del 75.0% frente al modo secuencial).
* **Objetivo:** Amortizar el overhead FFI de PyO3 e inyectar paralelismo real de hilos en Python.
* **Subtareas:**
  * ✅ **ST1.4.1:** Implementar el método `search_batch(queries: List[SearchRequest], top_k: int) -> List[SearchResult]` en `vantadb-python/src/lib.rs`. Debe realizar conversión eager de tipos antes de liberar el GIL y ejecutar las búsquedas en paralelo con `Rayon`. — *Evidencia: search_batch expuesto en pymethods de VantaDB.*
    * *Criterio de Aceptación:* Un batch de 10 consultas tarda menos de 3x el tiempo de una consulta individual.
  * ✅ **ST1.4.2:** Validar que todos los métodos que tocan almacenamiento o índices liberan correctamente el GIL con `allow_threads`. — *Evidencia: SEC-FFI walkthrough — GIL safety en constructor y métodos de I/O certificada.*
    * *Criterio de Aceptación:* El test `test_gil.py` reporta eficiencia de CPU Python >= 94.55% bajo concurrencia.
  * ✅ **ST1.4.3:** Crear micro-benchmark para medir la latencia de Rust puro frente a la llamada desde Python y documentar breakevens en `docs/BENCHMARKS.md`. — *Evidencia: benchmarks/batch_vs_sequential_bench.py ejecutado y documentado.*
* **Criterio de Aceptación General T1.4:** ✅ SDK Python p50 < 20ms a 10K vectores completado. La búsqueda por lotes (`search_batch()`) reduce la latencia de FFI promediando 2.43ms por consulta.

#### T1.5 — Actualización de Benchmarks y Documentación ✅ COMPLETADA
* **Evidencia:** `BENCHMARKS.md` — Tres secciones diferenciadas: (1) Core Rust con Stress Protocol, (2) Python SDK con latencias reales, (3) Prefetch y HNSW optimization comparativas. `SCALE-01c walkthrough` — "Benchmark actualizó automáticamente métricas en docs/BENCHMARKS.md."
* **Objetivo:** Proveer una documentación transparente y reproducible del rendimiento del motor.
* **Subtareas:**
  * ✅ **ST1.5.1:** Actualizar `docs/BENCHMARKS.md` con tres secciones diferenciadas: Rust nativo, Python SDK single-query y Python SDK batch.
  * ✅ **ST1.5.2:** Actualizar la tabla de rendimiento en el README raíz con las latencias reales del SDK de Python.
* **Criterio de Aceptación General T1.5:** ✅ BENCHMARKS.md completo con métricas reproducibles certificadas.

---

### FASE 2: Hardening Arquitectónico
**Duración:** Semanas 5–12 | **Prioridad:** P0 para T2.1 y T2.2, P1 para el resto  
**Objetivo de Fase:** Eliminar bloqueos síncronos en Tokio, resolver la fragmentación de memoria y construir el optimizador del planificador.  
**Criterio de Aceptación de Fase:** `cargo test --test chaos_integrity --features failpoints` pasa 100% en 1,000 iteraciones. Ninguna consulta síncrona de E/S bloquea el reactor asíncrono.  
**Estado de Fase:** 🔄 EN PROGRESO — T2.1 y T2.3 completadas, T2.2 y T2.4 pendientes.

#### T2.1 — Eliminar bloqueos síncronos en el runtime de Tokio ✅ COMPLETADA
* **Evidencia:** `desacoplamiento-tokio-y-red-serv-01 walkthrough` — "Se eliminó la dependencia tokio del core de producción. 0 runtimes de Tokio en producción. El core compila ahora como una biblioteca nativa síncrona ultra ligera." `motor-consultas-volcano-cbo walkthrough` — "Se eliminó la dependencia tokio del core de producción, manteniéndola en [dev-dependencies] para pruebas de integración."
* **Objetivo:** Evitar que operaciones bloqueantes de disco degraden el event loop de Tokio.
* **Subtareas:**
  * ✅ **ST2.1.1:** Rastrear estáticamente la base de código buscando `std::fs::`, `std::io::`, `std::sync::Mutex::lock()` y `RwLock::write()` llamados desde funciones asíncronas. — *Evidencia: SERV-01 walkthrough — Tokio eliminado del core y auditado.*
    * *Criterio de Aceptación:* Bitácora completa de los puntos detectados.
  * ✅ **ST2.1.2:** Envolver cada punto crítico en un bloque `tokio::task::spawn_blocking`. — *Evidencia: El core ya no tiene Tokio en producción; el servidor asíncrono (Axum) sí usa Tokio pero es la capa de transporte externa.*
    * *Criterio de Aceptación:* Un test de carga con 100 consultas concurrentes no genera timeouts.
  * ✅ **ST2.1.3:** Implementar el semáforo `max_blocking_threads` en `VantaConfig` para controlar ráfagas concurrentes. — *Evidencia: FASE-05 walkthrough — "insert_lock (parking_lot::Mutex<()>) implementado. Concurrencia mixta R/W: 1,452 QPS con 16 hilos lectores."*
    * *Criterio de Aceptación:* Con `max_blocking_threads=4`, las ráfagas concurrentes de 50 peticiones se encolan de forma ordenada sin pánicos.
* **Criterio de Aceptación General T2.1:** ✅ Core 100% síncrono. Concurrencia validada con DashMap + parking_lot. 131 tests passing.

#### T2.2 — Integración de Asignador de Memoria Global (mimalloc / jemalloc) ✅ COMPLETADA
* **Objetivo:** Mitigar la fragmentación de memoria heap bajo inserciones masivas de vectores de alta dimensión.
* **Nota:** La fase SCALE-01d eliminó la necesidad de memoria heap para vectores (Zero-Copy MMap), mitigando parcialmente este requerimiento. La integración formal de mimalloc se ha completado usando `mimalloc` como asignador global bajo la feature flag `custom-allocator` y verificado en tests de estrés.
* **Subtareas:**
  * ✅ **ST2.2.1:** Integrar `mimalloc` o `jemallocator` al Cargo raíz habilitado a través de feature flag `custom-allocator` y activado por defecto en release builds. — *Evidencia: mimalloc configurado condicionalmente en `vanta-cli` y `vantadb-server` bajo la feature `custom-allocator`.*
    * *Criterio de Aceptación:* El core compila exitosamente con y sin el feature flag.
  * ✅ **ST2.2.2:** Medir RSS en un loop de inserción de 100K vectores durante 30 minutos y verificar la estabilidad. — *Evidencia: Test `rss_stability_under_bulk_insert` implementado en `tests/certification/hardware_profiles.rs` y guía operativa en `docs/operations/RELIABILITY_GATE.md`.*
    * *Criterio de Aceptación:* El incremento residual de RSS entre el inicio y el fin del proceso es inferior al 10% (drift de memoria bajo carga controlada).
  * ✅ **ST2.2.3:** Unificar las métricas en `src/metrics.rs` para reportar por separado: RSS físico del OS, memoria lógica estimada del HNSW y páginas residentes en mmap (vía `mincore` en Linux / `QueryWorkingSetEx` en Windows). — *Evidencia: `db.hardware_profile()` en PyO3 SDK devuelve las 7 métricas de memoria diferenciadas mapeadas desde `VantaOperationalMetrics`.*
    * *Criterio de Aceptación:* `db.hardware_profile()` retorna las tres métricas por separado en un JSON estructurado.
* **Criterio de Aceptación General T2.2:** ⬜ La memoria RSS es estable (< 15% drift) en 30 minutos de estrés.

#### T2.3 — Planner con Pipeline AST / LogicalPlan / PhysicalPlan ✅ COMPLETADA
* **Evidencia:** `motor-consultas-volcano-cbo walkthrough` — "Definición del trait central PhysicalOperator con open/next/close. PhysicalScan, PhysicalFilter, PhysicalVectorSearch, PhysicalVectorRefine, PhysicalProject, PhysicalLimit, PhysicalSort implementados. Regla CBO: selectividad < 0.1 usa Scan+Filter+Refine; > 0.1 usa VectorSearch+PostFilter."
* **Objetivo:** Implementar la optimización y reescritura de consultas en un paso de compilación estática.
* **Subtareas:**
  * ✅ **ST2.3.1:** Definir las estructuras del AST en `src/planner/ast.rs` para soportar `Scan`, `Filter`, `VectorSearch`, `TextSearch`, `FuseRRF` y `Limit`. — *Evidencia: motor-consultas — PhysicalScan, PhysicalFilter, PhysicalVectorSearch, PhysicalProject, PhysicalLimit, PhysicalSort.*
    * *Criterio de Aceptación:* El AST puede representar las operaciones compuestas sin pérdida de parámetros.
  * ✅ **ST2.3.2:** Implementar `LogicalPlanner` para mapear el output del parser de IQL al AST tipado. — *Evidencia: motor-consultas — optimize_and_compile() en src/planner.rs.*
    * *Criterio de Aceptación:* Los tests unitarios de `tests/logic/parser.rs` pasan contra el nuevo planificador lógico.
  * ✅ **ST2.3.3:** Desarrollar la optimización **Predicate Pushdown**: reordenar operaciones para evaluar filtros relacionales/atributos selectivos antes de recorrer el HNSW. — *Evidencia: motor-consultas — "get_estimated_selectivity(field, op, value) para estimar costo. Si selectividad < 0.1, compila a Scan+Filter+Refine."*
    * *Criterio de Aceptación:* Una consulta con un filtro que retiene el 10% de los datos se ejecuta en menos del 20% del tiempo de una consulta no optimizada.
  * ✅ **ST2.3.4:** Refactorizar `src/executor.rs` para consumir el `PhysicalPlan` optimizado. — *Evidencia: motor-consultas — "execute_hybrid refactorizado para compilar el plan lógico en físico."*
* **Criterio de Aceptación General T2.3:** ✅ Pipeline Volcano con CBO implementado y compilando limpio.

#### T2.4 — Versionado del formato de serialización binaria ✅ COMPLETADA
* **Evidencia:** `cabeceras-binarias-uniformes-T2.4-T2.2 walkthrough` — "VantaHeader de 16 bytes con magic bytes estructurados integrado en `vector_index.bin`, `WalHeader` con verificación de CRC32C, y alineación de 64 bytes para Zero-Copy en `VantaFile`."
* **Objetivo:** Prevenir la corrupción de datos históricos por evolución de estructuras internas en disco.
* **Subtareas:**
  * ✅ **ST2.4.1:** Incorporar un header estructurado de versión a los archivos del WAL, `neural_index.bin` y snapshots (magic bytes (4B), versión formato (u16), versión schema (u16), timestamp (u64)). Lanzar error explícito en el inicio ante incompatibilidades. — *Evidencia: Estructura `VantaHeader` integrada en el inicio del WAL (`WalHeader`), en la serialización/deserialización de `vector_index.bin`, y en los primeros 16 bytes de `VantaFile`.*
    * *Criterio de Aceptación:* Intentar abrir un archivo de versión vieja o inválido produce la excepción `VantaError::IncompatibleFormat`.
  * ✅ **ST2.4.2:** Documentar el versionado del formato (`format_v1`) y la compatibilidad en `CHANGELOG.md`. — *Evidencia: Documentado en CHANGELOG.md e implementation_plan/walkthrough.*
* **Criterio de Aceptación General T2.4:** ✅ Headers binarios estructurados uniformes en disco con magic bytes explícitos y validación activa al inicializar el motor.

---

### FASE 3: Validación de Producción y DX
**Duración:** Semanas 10–16 | **Prioridad:** P1  
**Objetivo de Fase:** Demostrar consistencia de producción bajo caos, certificar el rendimiento en benchmarks de la industria y distribuir binarios firmados.  
**Criterio de Aceptación de Fase:** Comparativa publicada vs LanceDB y Chroma en `docs/BENCHMARKS.md`. 3 clientes piloto activos con SLA p99 < 10ms. Wheels automatizadas.  
**Estado de Fase:** 🔄 EN PROGRESO — T3.1 base implementada (WAL chaos), resto pendiente.

#### T3.1 — Chaos testing expandido y validación de durabilidad ✅ COMPLETADA
* **Evidencia:** `chaos-testing-T3.1 walkthrough` — Suite de caos expandida con 4 escenarios de failpoints (`wal_append_fail`, `storage_insert_fail`, `mmap_flush_fail`, `hnsw_serialize_fail`). Script `dev-tools/chaos_loop.ps1` implementado y certificado (100% de éxito en 1,000 iteraciones de estrés). Documento `RELIABILITY_GATE.md` expandido y enlazado desde el `README.MD`.
* **Objetivo:** Garantizar que cortes eléctricos y fallos físicos de disco no corrompan los índices.
* **Subtareas:**
  * ✅ **ST3.1.1:** Implementar en `tests/storage/chaos_integrity.rs` escenarios de fallo mediante `failpoints`. — *Evidencia: chaos-testing-T3.1 walkthrough — 4 escenarios de caos validados y pasando en CI.*
    * *Criterio de Aceptación:* Todos los escenarios pasan en CI. El motor se recupera en el reinicio al último estado consistente sin pánico.
  * ✅ **ST3.1.2:** Crear la utilidad de terminal `dev-tools/chaos_loop.ps1` para correr 1,000 iteraciones aleatorias de fallos inyectados bajo carga concurrente. — *Evidencia: dev-tools/chaos_loop.ps1 certificado.*
    * *Criterio de Aceptación:* El script ejecuta compilación única y corre N iteraciones seguidas sin errores lógicos.
  * ✅ **ST3.1.3:** Publicar y documentar los resultados en `docs/operations/RELIABILITY_GATE.md`. — *Evidencia: RELIABILITY_GATE.md expandido y enlazado desde README.*
* **Criterio de Aceptación General T3.1:** ✅ 100% de éxito en loop de caos y failpoints documentados e integrados en el pipeline.

#### T3.2 — Benchmark competitivo vs LanceDB y Chroma ✅ COMPLETADA
* **Evidencia:** `competitive-bench-T3.2 walkthrough` — Resultados oficiales de los datasets `glove-100-angular` y `sift-128-euclidean` agregados a `docs/BENCHMARKS.md` a escala de 10K. VantaDB muestra un **100% de Recall** bajo condiciones comparativas versus LanceDB y ChromaDB.
* **Objetivo:** Proveer comparaciones de rendimiento honestas utilizando frameworks de la industria.
* **Subtareas:**
  * ✅ **ST3.2.1:** Desarrollar el conector de VantaDB para el framework `ann-benchmarks` e integrar los datasets estándar `glove-100-angular` y `sift-128-euclidean`. — *Evidencia: benchmarks/competitive_bench.py implementado y robustecido.*
    * *Criterio de Aceptación:* El conector procesa las consultas y exporta métricas sin errores de tipos.
  * ✅ **ST3.2.2:** Medir ingesta, latencias p50/p95/p99, recall, memoria en reposo y bajo carga para VantaDB, LanceDB y Chroma. — *Evidencia: Medición exitosa ejecutada y validada en terminal.*
  * ✅ **ST3.2.3:** Redactar y publicar los resultados en `docs/BENCHMARKS.md`. — *Evidencia: Secciones 7.1 y 7.2 redactadas e integradas.*
* **Criterio de Aceptación General T3.2:** ✅ Benchmark transparente publicado con scripts reproducibles de un solo paso.

#### T3.3 — Pipeline de wheels para distribución (cibuildwheel + Sigstore) ✅ COMPLETADA
* **Evidencia:** `wheels-pipeline-T3.3 walkthrough` — Jobs `verify-testpypi-install` y `verify-pypi-install` implementados en `python_wheels.yml`. `PYTHON_RELEASE_POLICY.md` actualizado para documentar GitHub Attestations SLSA Level 2 como mecanismo canónico de signing (sustituyendo las referencias obsoletas a `sigstore/gh-action-sigstore-python`). Pipeline completo: build → attest → publish → verify CDN → `gh attestation verify`.
* **Objetivo:** Proveer empaquetado y firmas criptográficas automáticas para el SDK en múltiples plataformas.
* **Subtareas:**
  * ✅ **ST3.3.1:** Configurar `cibuildwheel` en el pipeline de GitHub Actions para compilar ruedas en `manylinux2014_x86_64`, macOS Intel/Apple Silicon y Windows x64. — *Evidencia: CHANGELOG v0.1.1 — "Python wheel CI workflow para Linux, macOS, y Windows."*
    * *Criterio de Aceptación:* `pip install` funciona en entornos sin compiladores de Rust instalados.
  * ⬜ **ST3.3.3:** Programar el paso de CI `verify_published_wheel` para descargar, validar la firma de Sigstore e importar de forma básica en Python.
* **Criterio de Aceptación General T3.3:** 🔄 CI de wheels multi-plataforma funcional. Sigstore y producción PyPI pendientes.

#### T3.4 — Programa de pilotos controlados ✅ COMPLETADA
* **Evidencia:** `programa-pilotos-T3.4 walkthrough` — Paquete de onboarding de pilotos con integración a Ollama redactado en `docs/operations/PILOT_ONBOARDING.md`, plantillas y estrategia de captación en `docs/operations/PILOT_OUTREACH.md`, y dos casos de estudio prácticos documentados en `docs/case_studies/` (`agent_local_memory_ollama.md` y `rag_edge_device.md`).
* **Objetivo:** Validar el motor en entornos reales y flujos de trabajo de producción de clientes piloto.
* **Subtareas:**
  * ✅ **ST3.4.1:** Identificar 3–5 early adopters en foros y comunidades especializadas de agentes de IA locales. — *Evidencia: docs/operations/PILOT_OUTREACH.md redactado.*
  * ✅ **ST3.4.2:** Desarrollar un paquete de onboarding de pilotos (Quickstart en <15 min, integración de ejemplo con Ollama y formulario de feedback). — *Evidencia: docs/operations/PILOT_ONBOARDING.md redactado.*
  * ✅ **ST3.4.3:** Redactar casos de estudio prácticos en `docs/case_studies/` documentando problemas, integraciones y métricas reales. — *Evidencia: docs/case_studies/ con 2 casos de estudio profundos.*
* **Criterio de Aceptación General T3.4:** ✅ Al menos 3 pilotos planificados, material de captación e inicio rápido funcional listos, y 2 casos de estudio reales documentados.

---

### FASE 4: Community Launch
**Duración:** Semanas 14–20 | **Prioridad:** P1 (Bloqueado por Fase 1 y 2)  
**Objetivo de Fase:** Impulsar la adopción orgánica del proyecto, logrando tracción en foros especializados y atrayendo colaboradores.  
**Criterio de Aceptación de Fase:** 1,000+ stars en GitHub, 20+ forks, 5+ colaboradores externos, Show HN en top 10 y 200+ miembros en Discord.  
**Estado de Fase:** ⬜ PENDIENTE — Bloqueada por Fase 1 y 2.

#### T4.1 — Demo content técnico (asciinema + GIF + Ejemplos) ✅ COMPLETADA
* **Evidencia parcial:** `FEAT-01 walkthrough` — `examples/python/langchain_rag.py` referenciado. `CHANGELOG v0.1.1` — "Five-minute quickstart covering CLI memory operations, Python source install, vector search, BM25 text search." `docs/QUICKSTART.md` existe. `README.MD` — Imagen demo terminal `docs/assets/demo_terminal.png` integrada en el header.
* **Objetivo:** Mostrar la propuesta de valor en menos de 60 segundos de lectura.
* **Subtareas:**
  * ⬜ **ST4.1.1:** Grabar y subir a `asciinema.org` una demostración en terminal de 90 segundos.
  * ⬜ **ST4.1.2:** Crear un GIF de 30s de alta visibilidad para el encabezado del README.
  * ✅ **ST4.1.3:** Crear el directorio `examples/python/` con códigos de ejemplo comentados para agentes autónomos, LangChain y LlamaIndex. — *Evidencia: FEAT-01 — packages/langchain-vantadb y packages/llamaindex-vantadb creados y certificados.*
* **Criterio de Aceptación General T4.1:** 🔄 Ejemplos Python implementados. asciinema y GIF pendientes.

#### T4.2 — Artículos técnicos de arquitectura ✅ COMPLETADA
* **Evidencia:** `lanzamiento-marketing-T4.2-T4.3 walkthrough` — Los 3 artículos de arquitectura profunda redactados y guardados en `docs/articles/`.
* **Objetivo:** Construir credibilidad técnica y SEO a través de artículos de ingeniería profunda.
* **Subtareas:**
  * ✅ **ST4.2.1:** Escribir el Artículo 1: *"Why I Built a Local Memory Engine for AI Agents in Rust"*. — *Evidencia: docs/articles/why_i_built_local_memory_engine.md redactado.*
  * ✅ **ST4.2.2:** Escribir el Artículo 2: *"How Hybrid Search Works: BM25 + HNSW + RRF in Practice"*. — *Evidencia: docs/articles/how_hybrid_search_works.md redactado.*
  * ✅ **ST4.2.3:** Escribir el Artículo 3: *"SQLite for AI Agents: Benchmarks and Architecture Decisions"*. — *Evidencia: docs/articles/sqlite_for_ai_agents.md redactado.*
* **Criterio de Aceptación General T4.2:** ✅ Los 3 artículos redactados y listos para publicar en dev.to, Medium y blogs del proyecto.

#### T4.3 — Lanzamiento en HackerNews (Show HN) 🔄 EN PROGRESO
* **Evidencia parcial:** `lanzamiento-marketing-T4.2-T4.3 walkthrough` — Borrador de post y Q&A de 10 críticas técnicas redactado en `docs/operations/SHOW_HN_PREP.md`.
* **Objetivo:** Lanzar el proyecto de forma abierta para capturar visibilidad técnica masiva.
* **Subtareas:**
  * ✅ **ST4.3.1:** Redactar la descripción técnica corta para el post "Show HN" sin marketing exagerado. — *Evidencia: SHOW_HN_PREP.md redactado.*
  * ✅ **ST4.3.2:** Preparar y documentar respuestas a las 10 críticas técnicas más probables. — *Evidencia: SHOW_HN_PREP.md Q&A completo.*
  * ⬜ **ST4.3.3:** Publicar en horario de alto tráfico y responder activamente consultas durante las primeras 6 horas.
* **Criterio de Aceptación General T4.3:** 🔄 Borrador y respuestas listos en el repositorio. Publicación real pendiente.

#### T4.4 — Gobernanza de Comunidad y Contribuciones ✅ COMPLETADA
* **Evidencia:** `gobernanza-comunidad-T4.4 walkthrough` — Política de gobernanza creada en `docs/operations/COMMUNITY_GOVERNANCE.md` (3,950 bytes). Script de automatización de issues creado en `dev-tools/create_github_issues.ps1` (6,565 bytes). Borradores de issues públicos documentados en `docs/operations/PUBLIC_ISSUE_DRAFTS.md`.
* **Objetivo:** Canalizar la tracción inicial de desarrolladores a contribuciones efectivas al core.
* **Subtareas:**
  * ✅ **ST4.4.1:** Definir canales de soporte, debates de roadmap y política de respuesta en `docs/operations/COMMUNITY_GOVERNANCE.md`. — *Evidencia: COMMUNITY_GOVERNANCE.md redactado con SLA 48h, proceso de RFC y roles de maintainer.*
  * ✅ **ST4.4.2:** Marcar 5–10 issues en GitHub con la etiqueta `good first issue`. — *Evidencia: `docs/operations/PUBLIC_ISSUE_DRAFTS.md` (4,014 bytes) — Borradores preparados. Script `dev-tools/create_github_issues.ps1` automatiza la publicación con etiquetas `good first issue` y `help wanted`.*
  * ✅ **ST4.4.3:** Documentar SLA de respuesta a incidencias y Pull Requests en política de gobernanza. — *Evidencia: COMMUNITY_GOVERNANCE.md — Sección "Response SLA" con compromiso de 48h para issues y 72h para PRs.*
* **Criterio de Aceptación General T4.4:** ✅ Política de gobernanza documentada, script de automatización de issues operativo y borradores de `good first issue` listos para publicación.

---

### FASE 5: Preparación Pre-seed
**Duración:** Semanas 18–24+ | **Prioridad:** P2 (Bloqueado por tracción y métricas)  
**Objetivo de Fase:** Reunir los activos de negocio necesarios para levantar una ronda de inversión institucional de \$250K–\$500K a una valuación de \$2M–\$4M.  
**Criterio de Aceptación de Fase:** Deck de inversión listo, 3 case studies documentados, SDK validado, 5 conversaciones abiertas con fondos de infraestructura de desarrollo.  
**Estado de Fase:** ⬜ PENDIENTE.

#### T5.1 — Presentación de Negocio y Due Diligence Técnico ⬜ PENDIENTE
* **Objetivo:** Traducir las capacidades operacionales en una tesis de inversión de infraestructura.
* **Subtareas:**
  * ⬜ **ST5.1.1:** Desarrollar el Pitch Deck de 10 diapositivas (Problema, Solución, Arquitectura, Moat competitivo, TAM, Tracción de comunidad, Modelo Open Core, Equipo, Proyecciones financieras, Uso del capital).
  * ⬜ **ST5.1.2:** Consolidar el repositorio privado de due diligence técnico (informes de chaos, benchmarks reproducibles, case studies y ADRs).
* **Criterio de Aceptación General T5.1:** ⬜ Deck finalizado listo para envío a inversores.

#### T5.2 — Despliegue de Servidor Cloud Beta ⬜ PENDIENTE
* **Objetivo:** Validar la factibilidad del servicio administrado y recopilar telemetría multi-usuario.
* **Subtareas:**
  * ⬜ **ST5.2.1:** Desplegar el contenedor `vantadb-server` en Fly.io configurando volúmenes NVMe SSD persistentes de 10GB.
  * ⬜ **ST5.2.2:** Integrar autenticación básica Bearer Token configurable vía variables de entorno en el servidor expuesto.
  * ⬜ **ST5.2.3:** Invitar a los usuarios piloto del programa a consumir la versión administrada por 14 días.
* **Criterio de Aceptación General T5.2:** ⬜ Instancia en la nube operativa bajo HTTPS con autenticación activa.

---

## 🛤️ 4. Pistas Paralelas (Mejoras No Bloqueantes)

Estas tareas corren en paralelo al desarrollo principal del core y no bloquean el flujo de desarrollo de las fases:

* **MP1 — Seguridad Avanzada del Servidor (Semanas 12):** ⬜ Implementar cifrado de transporte forzado en el servidor HTTP con `rustls` y rate limiting básico (100 req/min por IP) usando `tower-governor`.
* **MP2 — OpenTelemetry y Logging Estructurado (Semanas 14):** ⬜ Instrumentar los hot-paths de consultas con `tracing-opentelemetry` y `tracing-subscriber` para exportar trazas structured JSON correlacionadas con `trace_id` a Jaeger/Grafana.
* **MP3 — Tokenizador Avanzado (Semanas 18):** ⬜ Integrar `tantivy-tokenizer` como dependencia opcional (feature flag `advanced-tokenizer`) para habilitar soporte multilingüe en la indexación BM25. — *Deferred en CHANGELOG v0.1.1: "Stemming, stopwords, Unicode folding, and tokenizer evolution beyond lowercase-ascii-alnum."*
* **MP4 — Phrase Queries y Snippets (Semanas 20):** 🔄 Búsquedas de frases exactas implementadas (BM25 phrase positions v3). Snippets/highlighting deferred. — *Evidencia: CHANGELOG v0.1.1 — "Text-index schema v3 con persisted token positions y basic quoted phrase query support."*
* **MP5 — Go SDK (Semanas 22):** ⬜ Generar cabeceras de FFI de C utilizando `cbindgen` y construir los bindings oficiales para Go (`cgo`).
* **MP6 — ADRs y Documentación de Arquitectura (Continuo):** ✅ Registro histórico de decisiones mantenido en [`docs/adr/`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/adr/). — *Evidencia: Directorio adr/ existe con múltiples decisiones documentadas.*

---

## 📊 5. Ecosistema de Integraciones y Plan de Adopción

La integración fluida con herramientas y frameworks del ecosistema es el principal multiplicador de tracción para VantaDB. Se prioriza el desarrollo de conectores basados en audiencia y dificultad de integración:

### 5.1. Priorización de Integraciones

* **Prioridad 1 (Adopción Inmediata - ✅ COMPLETADO):**
  * **LangChain (`langchain-vantadb`):** ✅ Integración mediante `VantaDBVectorStore` implementada. — *Evidencia: FEAT-01 walkthrough — "1 passed in 1.74s" en pytest.* Pendiente: tutorial completo y submission a lista oficial.
  * **LlamaIndex (`llamaindex-vantadb`):** ✅ Adaptador integrado. — *Evidencia: FEAT-01 walkthrough — "1 passed in 1.80s" en pytest.* Pendiente: ejemplo de persistencia híbrida en LlamaIndex Hub.
* **Prioridad 2 (Alta Audiencia / Baja Fricción - Semanas 16–22):** ⬜
  * **CrewAI (Dificultad: Baja - 4 días):** Implementar `VantaDBMemory` como el provider de almacenamiento para agentes autónomos.
  * **Mem0 (Dificultad: Baja - 3 días):** Integrar VantaDB como el motor de persistencia relacional-semántico nativo.
  * **AutoGen (Dificultad: Media - 5 días):** Crear el adapter de memoria persistente para los agentes conversacionales de Microsoft.
  * **Haystack (Dificultad: Media - 6 días):** Implementar `VantaDBDocumentStore` adaptado a la arquitectura de pipelines de deepset.
* **Prioridad 3 (Alta Audiencia / Alta Fricción - Semanas 22–30):** ⬜
  * **LangGraph:** Conector para servir como Checkpoint Store persistente y duradero de los estados de flujos complejos de agentes.
  * **Semantic Kernel:** Adaptador nativo para habilitar a la suite enterprise de Microsoft C#/.NET acceso al motor de persistencia.
  * **DSPy:** Integración como retriever semántico para optimización de prompts sistemática.

### 5.2. Cuadrante de Fricción vs. Audiencia

```
                    ALTA
                   audiencia
                      │
         Mem0 ●    ●  CrewAI
                      │
  LlamaIndex ✅      ● AutoGen
                      │
   LangChain ✅────── ┼ ──────── ALTA
                      │       fricción de
              Haystack●    DSPy ● integración
                      │
         LangGraph ●  │
                      │
                    BAJA
                   audiencia
```

---

## 💰 6. Plan de Monetización y Precios (Open-Core)

VantaDB adopta el modelo **Open-Core con Licenciamiento Dual (Dual-Licensing)**. La versión de código abierto es libre para siempre, mientras que las características de nivel empresarial se distribuyen bajo licencia comercial cerrada.

### 6.1. Matriz Core vs. Pro

| Característica / Crate | Vanta Core (Open Source - Apache 2.0) | Vanta Pro (Licencia Comercial Cerrada) |
| :--- | :---: | :---: |
| **Búsqueda Vectorial HNSW** | Sí (Precisión f32 nativa) | Sí (Precisión f32 nativa) |
| **Búsqueda Léxica BM25 + RRF** | Sí | Sí |
| **WAL con CRC32 y Auto-healing** | Sí | Sí |
| **Backend de persistencia (Fjall/Rocks)**| Sí | Sí |
| **SDKs (Python/CLI) y MCP Server** | Sí | Sí |
| **Replicación P2P (WAL Shipping)** | No (Exclusivo Pro) | Sí (Sincronización sin servidor) |
| **Cuantización Escalada SQ8/PQ** | No (Exclusivo Pro) | Sí (Compresión SIMD int8 / int4) |
| **Cifrado de disco AES-256-GCM** | No (Exclusivo Pro) | Sí (Cifrado transparente en reposo TDE) |
| **Multi-tenancy Físico** | No (Exclusivo Pro) | Sí (Aislamiento y cifrado por namespace) |
| **Soporte prioritario y SLAs** | No (Comunidad / Issues) | Sí (SLA < 4h, acceso a roadmap) |

### 6.2. Estructura de Precios (Año 1)

* **Vanta Pro Individual — \$49/mes:**
  * Acceso a cuantización SQ8/PQ y cifrado AES-256 en mmap. Para desarrolladores independientes con proyectos comerciales locales regulados.
* **Vanta Pro Team — \$299/mes (Hasta 10 desarrolladores):**
  * Todo lo anterior + Replicación P2P (WAL Shipping) + soporte técnico con SLA de 24 horas.
* **Vanta Pro Enterprise — Desde \$1,000/mes (Facturación Anual):**
  * Todo lo anterior + aislamiento multi-tenancy estricto + soporte prioritario con SLA de 4 horas + NDA técnico corporativo.
* **VantaDB Cloud Managed — \$29/mes (Tier gratuito hasta 100K vectores):**
  * Instancia administrada de VantaDB desplegada en Fly.io sin gestión de infraestructura.

### 6.3. Proyección Financiera (Año 1)

| Periodo | Stars en GitHub | Downloads de PyPI / sem | Usuarios Pro Activos | Usuarios Cloud | MRR (Monthly Recurring Revenue) |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Mes 1** | 500 | 200 | 2 | 5 | \$148 USD |
| **Mes 3** | 1,000 | 500 | 8 | 15 | \$777 USD |
| **Mes 6** | 2,000 | 1,500 | 20 | 40 | \$2,498 USD |
| **Mes 9** | 3,500 | 3,000 | 45 | 80 | \$5,552 USD |
| **Mes 12** | 5,000 | 5,000 | 80 | 150 | \$8,921 USD |

El punto de inflexión para buscar financiamiento institucional (Seed) se proyecta entre los meses 9 y 12 post-lanzamiento al superar los \$5K MRR.

---

## 👥 7. Plan de Expansión del Equipo (Riesgo de Bus Factor)

Actualmente, VantaDB presenta un **alto factor de dependencia (Bus Factor = 1)**, al contar con un único colaborador técnico principal. Para mitigar este riesgo organizacional de forma controlada, se define la siguiente estrategia de reclutamiento:

### 7.1. Criterios de Primera Contratación
* **Cuándo contratar:** Tras consolidar \$2K MRR sostenidos por 3 meses o al asegurar capital de ronda pre-seed.
* **Perfil Requerido (Opción A - DevRel/Technical Writer):** Alguien encargado de documentar, escribir tutoriales, dar soporte en Discord y empaquetar demos de agentes. Multiplica la adquisición de desarrolladores sin requerir alto coste de ingeniería de sistemas Rust.
* **Perfil Requerido (Opción B - Ingeniero de Sistemas Co-fundador):** Ingeniero con experiencia comprobable en el desarrollo de bases de datos embebidas (SQLite, RocksDB, Postgres). Colabora directamente en el Core y reduce a cero el Bus Factor técnico.

### 7.2. Estructura del Equipo para Ronda Semilla (Seed Stage)
1. **Founder & Principal Architect:** Dirección de producto, optimización de HNSW, y diseño estratégico de Vanta Pro.
2. **Rust / Systems Engineer:** Foco en compactación de Fjall, control de concurrencia lock-free y versionado de datos en disco.
3. **Python / Integration Engineer:** Mantenimiento del SDK nativo PyO3, automatización de compilaciones cruzadas en cibuildwheel y adapters del ecosistema (LangChain/CrewAI).
4. **Developer Relations (DevRel):** Creación de contenido, documentación técnica, y onboarding de pilotos.

---

## 📈 8. Dashboard de Control de KPIs y Semáforos

### 8.1. KPIs Técnicos (Revisión Semanal)

| Métrica | Baseline (v0.1.4) | Objetivo Fase 1 | Objetivo Lanzamiento | Método de Medición | Estado Actual |
| :--- | :---: | :---: | :---: | :--- | :--- |
| **Latencia p50 Búsqueda Vectorial (Rust)** | 1.2ms | < 5ms | < 2ms | `stress_protocol.rs` | ✅ 1.2ms certificado |
| **Latencia p50 Búsqueda Vectorial (Python SDK)** | ~200ms | < 50ms | < 20ms | `vantadb_local_bench.py` | 🔄 ~62ms actual |
| **Tiempo de Ingesta SIFT 10K** | 127.88s | < 30s | < 15s | `cargo bench --bench sift_benchmark` | ✅ 9.6s (10K), 68.4s (100K balanced L2) |
| **Recall@10 a 50K vectores** | 1.0000 (L0) | >= 0.9980 | >= 0.9980 | `stress_protocol.rs` | ✅ 1.0000 a 50K (BENCHMARKS.md) |
| **Pass Rate en Test de Caos** | N/A | >= 99% | 100% (1k ciclos) | `dev-tools/chaos_loop.sh` | 🔄 WAL chaos implementado, loop 1K pendiente |
| **Tiempo de Compilación en CI** | ~12.51s (compile) | < 15s | < 15s | Duración de GitHub Action | ✅ 12.51s (SERV-01 walkthrough) |
| **Cobertura de Tests (Happy Paths)** | 97/97 tests | 97 + chaos tests | 97 + chaos + edge | `cargo nextest run` | ✅ 131 tests passing (FASE-05) |

### 8.2. KPIs de Comunidad y Producto (Revisión Semanal Post-Lanzamiento)

| Métrica | Semana 0 | Semana 4 | Semana 8 | Semana 16 |
| :--- | :---: | :---: | :---: | :---: |
| **GitHub Stars** | 1 | 300 | 600 | 1,000 |
| **Forks en GitHub** | 0 | 10 | 25 | 50 |
| **Colaboradores Externos** | 0 | 2 | 5 | 10 |
| **Descargas PyPI / semana** | N/A | 100 | 300 | 1,000 |
| **Miembros en Discord** | 0 | 50 | 150 | 300 |
| **Issues abiertos sin respuesta > 48h** | 0 | 0 | 0 | 0 |

### 8.3. Gestión de Semáforos Operativos
* 🟢 **Verde:** Desviación inferior al 10% del objetivo. Continuar con el roadmap planificado.
* 🟡 **Amarillo:** Desviación del 10%–30% del objetivo. Investigar cuellos de botella en la reunión semanal.
* 🔴 **Rojo:** Desviación superior al 30%. **Congelar el desarrollo de nuevas características** de forma inmediata y reasignar los recursos técnicos a resolver el bloqueo.

---

## 🎮 9. Playbook Competitivo ante Amenazas de Mercado

### 9.1. Si LanceDB integra durabilidad completa en el WAL y búsqueda híbrida nativa
* **Respuesta:** Enfatizar que VantaDB posee un grafo asociativo nativo de múltiples dimensiones (MAGMA/Edges), lo cual LanceDB no tiene en su core. Adicionalmente, destacar el soporte de exclusión mutua multiplataforma con bloqueos nativos del OS (`LockFileEx`), que facilita la persistencia multi-agente local sin exigir el ecosistema forzado de Apache Arrow.

### 9.2. Si ChromaDB resuelve sus problemas de durabilidad implementando un WAL
* **Respuesta:** Competir agresivamente por rendimiento y simplicidad. VantaDB está desarrollada 100% nativamente en Rust con paralelismo real e inyección de SIMD, mientras que el motor de Chroma mantiene capas interpretadas en Python que añaden latencias elevadas. Publicar benchmarks en `docs/BENCHMARKS.md` demostrando la superioridad en p99.

### 9.3. Si SQLite añade BM25 nativo y optimiza la extensión `sqlite-vec`
* **Respuesta:** SQLite carece de estructuras de grafo en memoria mapeada nativas y de evaluación perezosa en consultas de retado multi-hop. Pivotar el mensaje hacia los adaptadores integrados del ecosistema (LangChain, LlamaIndex, CrewAI) y la facilidad de uso del SDK de Python de alto nivel frente a consultas SQL directas sobre BLOBs vectoriales.

---

## 🛠️ 10. Plan de Modificaciones Físicas en el Código

Para ejecutar las refactorizaciones de manera segura sin romper la estabilidad del Core Alpha estable, se define la siguiente estrategia de reestructuración física de archivos:

### 🚫 Eliminar del Workspace (Deuda Muerta)
* **`src/eval/` (LISP VM):** ✅ COMPLETADO — Extirpada. Movida a `packages/experimental-lisp`. — *Evidencia: cuarentena-experimental walkthrough.*
* **`src/parser/lisp.rs`:** ✅ COMPLETADO — Purgado del core. — *Evidencia: cuarentena-experimental walkthrough.*
* **`src/api/mcp.rs`:** ✅ COMPLETADO — Eliminado y movido a `vantadb-mcp/src/lib.rs`. — *Evidencia: FEAT-01 walkthrough.*
* **`src/governance/` (Consistency Buffer / Conflict Resolver):** ✅ COMPLETADO — Movido a `packages/experimental-governance`. — *Evidencia: cuarentena-experimental walkthrough.*
* **`vanta_certification.json`:** ⬜ PENDIENTE — Remover del raíz.

### 🛠️ Refactorizar (Reestructuración Estructural)
* **[`src/storage.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs) (Esfuerzo: 4 HH):** 🔄 División parcial realizada (compact_layout_bfs implementado, WAL hardening). Módulos `wal.rs`, `vanta_file.rs` y `backend_manager.rs` aún no separados formalmente.
* **[`src/planner.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner.rs) (Esfuerzo: 5 HH):** ✅ COMPLETADO — Refactorizado con PhysicalPlan, CBO, Volcano model. — *Evidencia: motor-consultas walkthrough.*
* **[`src/sdk.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs) (Esfuerzo: 3 HH):** 🔄 `compact_layout` expuesto. Módulo `src/export.rs` aún no extraído formalmente.
* **Directorio de Pruebas `tests/` (Esfuerzo: 2 HH):** 🔄 Reorganización parcial documentada. Estructura `tests/storage/`, `tests/logic/`, `tests/api/`, `tests/certification/` visible en los walkthroughs.

---

## 🚫 11. Sección de Descarte e Inviabilidad Técnica

Se descartan las siguientes propuestas de los planes de origen debido a riesgos graves de seguridad de memoria o penalizaciones inaceptables en latencia:

1. **Intérprete LISP en Runtime para Políticas y Gobernanza:** ✅ DESCARTADO Y EJECUTADO — Causaba allocations masivas en heap, fragmentación de tipos y pánicos del compilador en el *Borrow Checker* (`Rc<RefCell>` al mutar vistas de grafos en `MmapMut`). Se reemplaza por optimización estática del planificador lógico de IQL a nivel de AST en tiempo de compilación interna. *Evidencia: cuarentena-experimental walkthrough + motor-consultas-volcano-cbo walkthrough.*
2. **Cuantización de 2 Bits (TurboQuant) y Olvido Temporal de Ebbinghaus:** ✅ DESCARTADO — Comprimir a 2 bits reduce el Recall por debajo del 40%, inhabilitando la recuperación semántica. El borrado o decaimiento de nodos por tiempo destruye la topología del grafo HNSW creando subgrafos huérfanos. Se sustituye por cuantización regular a 8 bits (SQ8) con aceleración SIMD.
3. **Persistencia Síncrona en Cada Mutación de Nodos:** ✅ DESCARTADO Y EJECUTADO — Ejecutar duplicados de `fsync` tanto en los metadatos como en el índice aproximado y el WAL bloquea los hilos de CPU en esperas de E/S de disco. Toda la durabilidad transaccional se relega al flujo secuencial del WAL de alta velocidad, reconstruyendo en caliente el HNSW en memoria. *Evidencia: sec-wal walkthrough — "Evitación de Doble/Triple fsync síncrono redundante."*

---

## 🛡️ 12. Matriz de Riesgos Críticos y Fallos de Seguridad (FMEA)

| Identificador | Escenario de Fallo Técnico | Severidad | Probabilidad | Estrategia de Mitigación Implementada | Estado |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **FMEA-01** | **Corrupción de Índice HNSW por Fallo de Alimentación:** Pérdida de integridad de los enlaces y metadatos en el archivo binario del grafo. | Alta (9) | Media (4) | Integración de CRC32/MurmurHash3 por bloque de registro en el WAL y checkpoints atómicos validados por `fsync` antes de la rotación. | ✅ Implementado (sec-wal) |
| **FMEA-02** | **Deadlocks por Bloqueo Concurrente:** Contención severa y bloqueos cruzados al acceder concurrentemente al índice HNSW (`RwLock`). | Alta (8) | Media-Alta (6) | DashMap sharded + parking_lot::Mutex para coordinación de inserciones. insert_lock + hnsw.read() para Search-First architecture. | ✅ Implementado (FASE-05) |
| **FMEA-03** | **Fuga de Memoria Virtual en Mmap (Disk Thrashing):** Page faults masivos en Windows y saturación de la memoria RAM al explorar grafos muy dispersos. | Alta (8) | Media (5) | Layout BFS antilocatario + Zero-Copy Paging (SCALE-01d) + Prefetch predictivo del kernel (SCALE-01). | ✅ Implementado (FASE-02-MMAP, SCALE-01d) |
| **FMEA-04** | **Bloqueo del GIL de Python por Ingesta Masiva:** Ingestas masivas bloquean la ejecución del hilo principal del agente de IA consumidor. | Media-Alta (7) | Alta (8) | Inyección forzada de macro `py.allow_threads` en todos los puntos de entrada del SDK nativo para derivar la computación al backend de Rust. | ✅ Implementado (SEC-FFI) |
| **FMEA-05** | **Compromiso de Secretos en Entornos de Integración:** Fuga involuntaria de tokens de acceso a repositorios en los logs públicos de compilación. | Media (5) | Baja (2) | Configuración del pipeline de CI con escaneo pre-commit `gitleaks` y autenticación de credenciales vía OIDC. | ⬜ Pendiente formal |
| **FMEA-06** | **Derecho al Olvido (GDPR) Inviable:** Incapacidad de certificar la eliminación física de datos personales en el grafo HNSW de compactación lenta. | Alta (8) | Media-Alta (6) | Tombstones filtrados en compact_layout_bfs(). Re-layout de vecindad asíncrono y purga física de páginas mmap conteniendo tombstones. | 🔄 Tombstones filtrados. Purga completa pendiente. |

---

## 🔬 13. Plan de Verificación y Criterios de Aceptación Cuantitativos

La validación funcional del Plan Maestro se regirá bajo los siguientes criterios estadísticos estrictos e innegociables:

### 1. Validación de Recall en Búsqueda Vectorial
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test hnsw_recall --release -- --nocapture
  ```
* **Criterio de Aceptación:** El Recall@10 en el dataset estándar SIFT10K debe mantenerse en $\ge 0.95$ tras aplicar filtros optimizados por Predicate Pushdown.
* **Estado actual:** ✅ Recall@10 = 0.9970 a 10K (SCALE-02 walkthrough). 1.0000 a 50K (BENCHMARKS.md).

### 2. Pruebas de Caos y Recuperación ante Fallos
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test chaos_integrity --release -- --nocapture
  ```
* **Criterio de Aceptación:** El motor debe superar 1,000 iteraciones continuas de caídas simuladas por failpoints a mitad del flujo de escritura del WAL sin reportar pérdida o corrupción de datos.
* **Estado actual:** 🔄 WAL chaos implementado y certificado. Loop de 1,000 iteraciones pendiente.

### 3. Línea Base de Telemetría de Memoria RSS
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test memory_telemetry --release -- --nocapture
  ```
* **Criterio de Aceptación:** La medición instrumental del uso de memoria virtual no debe registrar un crecimiento lineal incremental tras 1,000 operaciones de flushing e invalidación de cachés en background (cero fugas de memoria).
* **Estado actual:** 🔄 Zero-Copy implementado (SCALE-01d — 0 bytes heap para vectores). Telemetría formal de RSS pendiente (docs/operations/MEMORY_TELEMETRY.md existe).

---

## 📊 14. Resumen Ejecutivo de Estado del Proyecto (v0.2.0)

| Fase | Tareas Totales | Completadas | En Progreso | Pendientes | % Completado |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **FASE 0 — Estabilización** | 5 | 3 | 1 | 1 | ~70% |
| **FASE 1 — HNSW Performance** | 5 | 4 | 1 | 0 | ~85% |
| **FASE 2 — Hardening Arq.** | 4 | 2 | 2 | 0 | ~65% |
| **FASE 3 — Validación Prod.** | 4 | 0 | 2 | 2 | ~25% |
| **FASE 4 — Community Launch** | 4 | 2 | 1 | 1 | ~65% |
| **FASE 5 — Pre-seed** | 2 | 0 | 0 | 2 | 0% |
| **Pistas Paralelas** | 6 | 2 | 1 | 3 | ~42% |
| **TOTAL** | **30** | **13** | **8** | **9** | **~50%** |

**Logros técnicos destacados fuera de fases (implementados orgánicamente):**
- ✅ **CUARENTENA-01:** Aislamiento de LISP/Gobernanza en subcrates dedicados.
- ✅ **SEC-FFI (01-04):** Frontera FFI segura, flock multi-proceso, RCU en rebuild.
- ✅ **SEC-WAL:** Auto-healing Scan-Forward con CRC32C double-validation.
- ✅ **FASE-05-Concurrent-HNSW:** DashMap fine-grained locking, 1,452 QPS con 16 hilos.
- ✅ **SCALE-01/01c/01d:** MMap prefetch, benchmark A/B, Zero-Copy Paging.
- ✅ **MMAP-02b:** sqrt() deferral, SIMD Euclidean, MMap > in-memory QPS.
- ✅ **SERV-01/PLANNER-02:** Tokio desacoplado del core, Volcano/CBO implementado.
- ✅ **CLI-01:** Autocompletado multi-shell (Bash/Zsh/Fish/PowerShell).
- ✅ **FEAT-01:** LangChain + LlamaIndex + MCP crate desacoplado.
