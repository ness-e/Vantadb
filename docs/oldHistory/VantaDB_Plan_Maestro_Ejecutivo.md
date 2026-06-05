# VantaDB — Plan Maestro Ejecutivo
**Versión:** 1.0 | **Fecha:** Junio 2026 | **Estado base:** commit `8ff77ee` (v0.1.4)

> **Cómo leer este documento.** Cada fase tiene un objetivo único y medible. Dentro de cada fase, las tareas son las unidades de trabajo mayores y las subtareas son los pasos concretos y accionables. Cada nivel tiene sus propios criterios de aceptación, que son las condiciones verificables que determinan cuándo algo está terminado, no estimado ni "casi listo". Las fases son secuenciales en lo crítico pero tienen solapamiento intencional donde las dependencias lo permiten. Al final del documento encontrarás las mejoras paralelas que no pertenecen a ninguna fase y el plan de marketing con tiempos explícitos.

---

## MAPA DE FASES Y CRONOGRAMA

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

## FASE 0: Estabilización Post-Cuarentena
**Duración:** Semanas 1–2 | **Estado:** En progreso (commit `8ff77ee`)

### Objetivo de Fase
Dejar el workspace en estado 100% compilable, sin warnings, sin tests rotos, y con la frontera entre módulos estables y experimentales documentada y físicamente verificada. Esta fase es el prerequisito de todo lo demás; ningún trabajo de optimización tiene sentido sobre una base inestable.

**Criterio de aceptación de Fase:** `cargo test --workspace` pasa al 100% en modo `--release`. `cargo clippy --all-targets -- -D warnings` pasa sin supresiones. La documentación de frontera experimental está publicada en `/docs/operations/EXPERIMENTAL_FEATURES.md` y el README principal la refleja correctamente.

---

### T0.1 — Estabilización completa del test suite post-cuarentena

**Objetivo:** Que todos los tests del core compilen y pasen sin depender de features gates experimentales ni módulos en cuarentena.

#### Subtareas

**ST0.1.1** — Auditar tests que usan sintaxis LISP y reescribirlos en IQL estándar. Los tests en `tests/api/structured_api_v2.rs` que usaban `(INSERT ...)` deben usar `INSERT` estándar de IQL. Verificar que ningún test en `tests/logic/` importa de `packages/experimental-lisp` ni `packages/experimental-governance`.

**Criterio de aceptación ST0.1.1:** `grep -r "experimental_lisp\|experimental_governance" tests/` retorna vacío.

**ST0.1.2** — Verificar que `tests/certification/stress_protocol.rs` pasa completo en release mode y que el Recall@10 >= 0.95 se mantiene. Este test es el contrato no negociable de calidad del HNSW.

**Criterio de aceptación ST0.1.2:** El bloque 1 del stress protocol reporta Recall@10 >= 0.9560 en `cargo test --test stress_protocol --release`.

**ST0.1.3** — Confirmar que `#![cfg(debug_assertions)]` está aplicado correctamente en `tests/storage/derived_index_recovery.rs` y `tests/storage/text_index_recovery.rs` para que no fallen en modo release.

**Criterio de aceptación ST0.1.3:** `cargo test --workspace --release` pasa sin errores de compilación en esos archivos.

**Criterio de aceptación T0.1:** `cargo nextest run --profile audit` reporta 97/97 tests passing, 0 failing, 0 panics.

---

### T0.2 — Limpieza de Clippy y formato

**Objetivo:** Cero warnings en CI. Esto no es estético: los warnings de Clippy en `src/python.rs` y `src/sdk.rs` documentados en las auditorías incluyen patrones de uso inseguro del GIL que deben resolverse.

#### Subtareas

**ST0.2.1** — Ejecutar `cargo fmt --all` y commitear los cambios de formato. Activar `cargo fmt --check` como gate bloqueante en `rust_ci.yml`.

**Criterio ST0.2.1:** `cargo fmt --check` pasa en CI en cualquier PR.

**ST0.2.2** — Ejecutar `cargo clippy --all-targets --all-features -- -D warnings`. Por cada warning en `src/python.rs` relacionado con acceso a objetos Python tras liberación del GIL: aplicar eager conversion (convertir `PyAny`/`PyDict` a tipos Rust nativos ANTES de llamar `py.allow_threads`). No suprimir con `#[allow]` sin una justificación escrita en el código.

**Criterio ST0.2.2:** `cargo clippy --all-targets --all-features -- -D warnings` pasa sin ninguna supresión en archivos críticos (`python.rs`, `sdk.rs`, `wal.rs`, `storage.rs`).

**Criterio de aceptación T0.2:** El CI de `rust_ci.yml` incluye steps de fmt check y clippy como gates bloqueantes y ambos pasan en verde.

---

### T0.3 — Coherencia de versiones en el workspace

**Objetivo:** Todos los `Cargo.toml` y `pyproject.toml` del workspace reportan la misma versión. Esto evita que el test `tests/version_coherence.rs` falle silenciosamente al actualizar una subcrate.

#### Subtareas

**ST0.3.1** — Auditar `Cargo.toml` raíz, `vantadb-python/Cargo.toml`, `vantadb-server/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `packages/langchain-vantadb/pyproject.toml`, `packages/llamaindex-vantadb/pyproject.toml`. Todos deben reflejar `0.1.4`.

**Criterio ST0.3.1:** `cargo test --test version_coherence` pasa.

**ST0.3.2** — Añadir `version_coherence` al perfil `audit` de nextest para que sea gate en CI normal, no solo en heavy certification.

**Criterio de aceptación T0.3:** `cargo test --test version_coherence` pasa en el perfil de CI rápido y es bloqueante en PR.

---

### T0.4 — Documentar frontera experimental en README

**Objetivo:** El README principal en inglés describe claramente qué es Production-facing, qué es Experimental, y qué es Deferred. Esto ya existe parcialmente en el README actual pero necesita ser verificado contra el estado real del código post-cuarentena.

#### Subtareas

**ST0.4.1** — Verificar que la tabla "Product Boundary" del README coincide exactamente con lo que está en cuarentena vs lo que está activo en `src/`. Específicamente: IQL/LISP en cuarentena, MCP en crate autónomo, governance en cuarentena.

**ST0.4.2** — Añadir enlace directo a `/docs/operations/EXPERIMENTAL_FEATURES.md` desde el README principal.

**Criterio de aceptación T0.4:** Un desarrollador que lea el README puede predecir correctamente qué APIs están disponibles sin necesidad de leer el código fuente.

---

## FASE 1: HNSW Scalability & Performance
**Duración:** Semanas 2–8 | **Prioridad:** P0 — Bloqueante de adopción

### Objetivo de Fase
Resolver el gap de rendimiento entre el motor Rust nativo (~1.2ms p50) y el SDK Python (~200ms p50) a un rango comercialmente competitivo de sub-20ms p50. Eliminar el bug de 127 segundos en SIFT 10K high-recall. Sin este trabajo, cualquier evaluación real del producto termina en rechazo.

**Criterio de aceptación de Fase:** Python SDK p50 < 20ms para búsqueda vectorial a 10K vectores 128d. SIFT 10K benchmark con L2 nativo (sin conversión al vuelo) completa en < 15 segundos con Recall@10 >= 0.95. Benchmark documentado reproducible en 3 pasos con un solo script.

---

### T1.1 — Auditoría y corrección de HNSW multi-layer

**Objetivo:** Verificar y si es necesario corregir que el algoritmo HNSW navega correctamente por todas las capas (no solo capa 0). Una implementación que solo usa capa 0 tiene complejidad O(n) en búsqueda, lo que explica la degradación no-lineal documentada (10x nodos → 32x latencia).

#### Subtareas

**ST1.1.1** — Leer `src/index.rs` completo. Identificar si `search_layer()` es llamado solo para la capa 0 o si existe un loop que desciende desde la capa más alta hasta la capa 0. El algoritmo HNSW correcto requiere: (a) entrada por la capa más alta con un solo candidato, (b) búsqueda greedy en esa capa hasta convergencia, (c) descenso a la siguiente capa con los candidatos seleccionados, repetido hasta capa 0.

**Criterio ST1.1.1:** Diagrama escrito (comentario en código o ADR) que documenta el flujo de control de búsqueda con el número exacto de capas usadas en un grafo de 50K nodos.

**ST1.1.2** — Si la implementación solo usa capa 0: implementar la navegación multi-layer. El entry point para la búsqueda debe ser el nodo `ep_node` (el entry point global del grafo). Para cada capa desde `max_layer` hasta 1: ejecutar búsqueda greedy con `ef=1` para encontrar el mejor candidato. Solo en capa 0 usar `ef=ef_search` completo.

**Criterio ST1.1.2:** Test que demuestra complejidad sub-lineal: latencia de búsqueda a 100K nodos es menos de 20x la latencia a 10K nodos (no 32x).

**ST1.1.3** — Verificar que `insert()` construye correctamente las capas superiores para nodos con nivel aleatorio > 0. La distribución de niveles debe seguir `floor(-ln(uniform(0,1)) * mL)` donde `mL = 1/ln(M)`.

**Criterio ST1.1.3:** Para un grafo de 10K nodos con M=32, el número esperado de nodos en capa 1 es aproximadamente 10K/32 ≈ 312, en capa 2 ≈ 10. Un test que verifica estas distribuciones estadísticas (con margen ±50%) pasa.

**Criterio de aceptación T1.1:** `cargo test --test hnsw_validation --release` pasa con Recall@10 >= 0.995 a 50K vectores. El factor de escalado latencia(100K)/latencia(10K) < 10x (vs los ~32x actuales).

---

### T1.2 — Soporte nativo de Distancia Euclidiana (L2)

**Objetivo:** El HNSW actual está optimizado para Coseno. Los benchmarks estándar de la industria (SIFT, ann-benchmarks) usan L2. Sin soporte nativo de L2, cualquier comparación competitiva requiere transformaciones al vuelo que destruyen la latencia.

#### Subtareas

**ST1.2.1** — Añadir `DistanceMetric::Euclidean` al enum de métricas en `src/index.rs`. Implementar el cálculo de distancia L2 con aceleración SIMD usando `wide::f32x8` (el mismo patrón que ya usa el cálculo coseno): `dist = sqrt(sum((a_i - b_i)^2))`. Añadir fallback escalar para hardware sin AVX2/NEON.

**Criterio ST1.2.1:** `cargo test --test hnsw -- euclidean` pasa con precision correcta (delta < 1e-5 vs implementación de referencia).

**ST1.2.2** — Exponer `DistanceMetric` en el API de Python SDK. El constructor de `VantaDB` debe aceptar un parámetro `distance_metric` con opciones `"cosine"` (default) y `"euclidean"`.

**Criterio ST1.2.2:** `python -c "import vantadb_py; db = vantadb_py.VantaDB('./test', distance_metric='euclidean')"` funciona sin error.

**ST1.2.3** — Ejecutar el benchmark SIFT 10K con L2 nativo y documentar los resultados en `docs/BENCHMARKS.md`. El objetivo es < 15 segundos para 1,000 queries high-recall.

**Criterio ST1.2.3:** `benchmarks/vantadb_local_bench.py --metric euclidean --size 10000 --queries 100` completa en < 15 segundos con Recall@10 >= 0.95 en el hardware de referencia (12-core CPU, AVX2).

**Criterio de aceptación T1.2:** SIFT 10K benchmark con L2 completa sin timeout, Recall@10 >= 0.95, tiempo total < 15s. Eliminado el bottleneck de conversión al vuelo.

---

### T1.3 — Layout de disco anti-locality para HNSW en MMap

**Objetivo:** El disk thrashing documentado ocurre porque nodos topológicamente adyacentes en el grafo HNSW están físicamente dispersos en el archivo mmap, causando page faults masivos. Re-ordenar el layout en disco para co-locar nodos frecuentemente co-visitados reduce los fallos de página y puede bajar la latencia del Python SDK significativamente.

#### Subtareas

**ST1.3.1** — Implementar una fase de re-layout después de la construcción del índice HNSW. El algoritmo: hacer un BFS desde el entry point en capa 0, y re-numerar internamente los nodos en el orden en que son visitados. Escribir el índice serializado con este nuevo orden. Al deserializar desde mmap, los nodos visitados secuencialmente en la búsqueda greedy estarán en páginas contiguas.

**Criterio ST1.3.1:** Test que mide el número de page faults (`/proc/self/pagemap` en Linux) durante 100 queries antes y después del re-layout. La reducción debe ser >= 30%.

**ST1.3.2** — Integrar el re-layout en `sync_to_mmap()` como paso post-construcción opcional (flag `optimize_layout: bool` en config). Activado por defecto en datasets > 10K vectores.

**Criterio ST1.3.2:** `VantaDB` con `optimize_layout=true` y 50K vectores muestra latencia Python SDK p50 < 50ms (versus los ~200ms actuales, esperamos 2-4x mejora de este paso solo).

**Criterio de aceptación T1.3:** Con el re-layout activo, el Python SDK p50 para búsqueda vectorial en 50K vectores 128d está dentro del rango 30ms–80ms en hardware de referencia.

---

### T1.4 — Optimización del boundary Python–Rust (API de batch queries)

**Objetivo:** El overhead del boundary PyO3 (serialización, GIL release, llamadas FFI por query individual) contribuye significativamente a los ~200ms del Python SDK. Una API de batch queries permite amortizar ese overhead sobre múltiples consultas simultáneas.

#### Subtareas

**ST1.4.1** — Implementar `VantaDB.search_batch(queries: List[SearchRequest], top_k: int) -> List[SearchResult]` en `vantadb-python/src/lib.rs`. Este método debe: (1) convertir todos los queries a tipos Rust eagerly antes de liberar el GIL, (2) ejecutar todas las búsquedas en paralelo con Rayon dentro del bloque `py.allow_threads`, (3) serializar los resultados a Python después.

**Criterio ST1.4.1:** Un batch de 10 queries simultáneos tarda menos de 3x una sola query individual (vs 10x si fueran secuenciales).

**ST1.4.2** — Verificar que `py.allow_threads` está aplicado en TODOS los métodos del SDK que tocan storage o index (no solo en `search_memory`). Revisar: `put`, `delete`, `flush`, `rebuild_index`, `export_namespace`, `import_namespace`.

**Criterio ST1.4.2:** El test `test_gil.py` del repositorio reporta eficiencia de CPU Python >= 94.55% bajo carga concurrente (ya certificado para search, debe mantenerse para otras operaciones).

**ST1.4.3** — Medir el overhead específico del boundary PyO3 con un microbenchmark: llamar al HNSW desde Rust puro (sin Python) vs desde Python con `allow_threads`. Documentar el breakeven en el que vale la pena usar batch vs individual.

**Criterio ST1.4.3:** Documento en `docs/BENCHMARKS.md` que muestra el overhead del boundary y la recomendación de uso de batch (típicamente cuando se hacen >= 3 queries que no dependen entre sí).

**Criterio de aceptación T1.4:** Python SDK p50 individual query < 20ms para búsqueda vectorial a 10K vectores 128d en hardware de referencia (12-core, AVX2). Batch de 10 queries simultáneos p50 < 60ms total.

---

### T1.5 — Actualizar benchmarks y documentación de performance

**Objetivo:** Los números en el README deben reflejar el estado real post-optimización, con metodología reproducible. Los números actuales (1.2ms p50 Rust nativo, 200ms Python SDK) representan dos contextos muy diferentes que confunden a los evaluadores.

#### Subtareas

**ST1.5.1** — Actualizar `docs/BENCHMARKS.md` con tres secciones claramente separadas: (1) Rust nativo in-process (para desarrolladores Rust), (2) Python SDK single-query, (3) Python SDK batch. Para cada sección: p50, p95, p99, throughput, metodología exacta, y hardware de referencia.

**ST1.5.2** — Actualizar la tabla de performance en el README para reflejar los números del Python SDK (que son los relevantes para el 90% de los usuarios), no solo los de Rust puro.

**Criterio de aceptación T1.5:** Un developer que lee el README puede predecir dentro de un factor 2x el rendimiento que obtendrá en su máquina, sin sorpresas negativas. El README incluye una nota explícita sobre la diferencia entre latencia Rust nativa y Python SDK.

---

## FASE 2: Hardening Arquitectónico
**Duración:** Semanas 5–12 | **Prioridad:** P0 para T2.1 y T2.2, P1 para el resto

### Objetivo de Fase
Eliminar los riesgos arquitectónicos que causarían fallos en producción bajo carga real: thread starvation en Tokio, fragmentación de memoria, y la ausencia de predicate pushdown en el planner. Esta fase hace a VantaDB confiable, no solo funcional.

**Criterio de aceptación de Fase:** `cargo test --test chaos_integrity --features failpoints --release` pasa 100% después de 1,000 iteraciones de kill -9 bajo carga. Ningún query introduce bloqueo síncrono en el runtime de Tokio. El planner aplica filtros de metadata ANTES de la búsqueda HNSW cuando la selectividad lo justifica.

---

### T2.1 — Eliminar bloqueos síncronos en el runtime Tokio

**Objetivo:** Todas las operaciones de I/O (disco, fsync, mmap faults) que ocurren en el contexto del servidor Tokio deben pasar por `spawn_blocking`. Los bloqueos síncronos en el event loop de Tokio causan thread starvation bajo carga moderada, lo que hace el servidor inutilizable en producción.

#### Subtareas

**ST2.1.1** — Auditar estáticamente el codebase buscando `std::fs::`, `std::io::`, `std::sync::Mutex::lock()`, y `RwLock::write()` en archivos que son llamados desde contextos async (cualquier función marcada `async fn` que llame a estos). Crear una lista de todos los sitios problemáticos.

**Criterio ST2.1.1:** Lista documentada de todos los sitios de I/O síncrono en contexto async. Estimado: probablemente 5–15 sitios.

**ST2.1.2** — Para cada sitio identificado en ST2.1.1: mover la operación dentro de `tokio::task::spawn_blocking(|| { ... }).await`. Para operaciones que son frecuentes y de corta duración (< 1ms), evaluar si `tokio::fs` async es más apropiado.

**Criterio ST2.1.2:** El servidor bajo test de carga con 100 requests concurrentes (usando `wrk` o similar) no exhibe timeouts ni degradación P99 desproporcionada (P99 < 5x P50).

**ST2.1.3** — Implementar el semáforo de `max_blocking_threads` en `VantaConfig` (ya documentado en ADR-003 pero pendiente de implementación completa). Este semáforo previene que ráfagas de requests paralelos saturen el pool de threads de bloqueo de Tokio.

**Criterio ST2.1.3:** Con `max_blocking_threads=4` configurado y 50 requests simultáneos, los requests adicionales hacen cola ordenadamente sin OOM ni panics.

**Criterio de aceptación T2.1:** Test de carga con 100 requests concurrentes por 60 segundos sobre `vantadb-server` muestra P99 < 2 segundos y cero panics.

---

### T2.2 — Asignador de memoria global mimalloc

**Objetivo:** El asignador por defecto de Rust (glibc ptmalloc en Linux) fragmenta el heap severamente bajo carga de vectores concurrente, causando OOM progresivo. mimalloc reduce la fragmentación y mejora throughput en benchmarks de bases de datos en un 10–20%.

#### Subtareas

**ST2.2.1** — Añadir `mimalloc = "0.1"` (o la versión estable actual) a `Cargo.toml` como dependencia opcional con feature flag `mimalloc-allocator`. Configurar el asignador global:

```rust
#[cfg(feature = "mimalloc-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

**Criterio ST2.2.1:** El crate compila con y sin el feature flag. Por defecto activado en builds de release.

**ST2.2.2** — Ejecutar el stress protocol de 100K vectores con y sin mimalloc y documentar la diferencia en RSS (memoria residente del proceso) a lo largo del tiempo. El objetivo es que RSS sea estable a largo plazo, no que crezca monótonamente.

**Criterio ST2.2.2:** RSS delta entre inicio y fin de un run de 100K inserciones + 10K queries en loop por 30 minutos: < 10% crecimiento con mimalloc.

**ST2.2.3** — Unificar las tres métricas de memoria en `src/metrics.rs`: (1) RSS físico via OS, (2) Memoria lógica del índice HNSW (estimada por el motor), (3) Páginas residentes mmap (via mincore en Linux). Exponer las tres por separado en el telemetry output del SDK.

**Criterio ST2.2.3:** `db.hardware_profile()` retorna un diccionario con las tres métricas diferenciadas, no solo una cifra agregada.

**Criterio de aceptación T2.2:** RSS del proceso es estable (< 15% drift) en un run de 30 minutos con inserciones y queries alternados. El filtro de admisión `AdmissionFilter` activa backpressure cuando RSS supera el 80% del límite configurado.

---

### T2.3 — Planner con pipeline AST / LogicalPlan / PhysicalPlan

**Objetivo:** El planner actual traduce directamente del parser al bytecode sin representación intermedia. Sin AST, es imposible aplicar optimizaciones como predicate pushdown, que pueden reducir el trabajo del HNSW en 60–80% en queries con filtros selectivos.

#### Subtareas

**ST2.3.1** — Definir los tipos del AST en `src/planner/ast.rs`. Los nodos necesarios para el MVP del planner son: `Scan { namespace }`, `Filter { predicate: Vec<Predicate> }`, `VectorSearch { query: Vec<f32>, top_k: usize, metric: DistanceMetric }`, `TextSearch { query: String, top_k: usize }`, `FuseRRF { k: f64 }`, `Limit { n: usize }`. Esto es suficiente para cubrir todos los paths del hybrid retrieval actual.

**Criterio ST2.3.1:** El AST puede representar los tres paths de query actuales (vector-only, text-only, hybrid) sin pérdida de información.

**ST2.3.2** — Implementar el `LogicalPlanner` que convierte el output del parser actual en el AST tipado. Este es el punto de entrada que reemplaza gradualmente la traducción directa.

**Criterio ST2.3.2:** Los tests de `tests/logic/parser.rs` pasan contra el nuevo LogicalPlanner (mismo output que el planner anterior, diferente representación interna).

**ST2.3.3** — Implementar predicate pushdown como una regla de optimización sobre el LogicalPlan: si hay un `Filter` después de un `VectorSearch`, y el filtro usa un campo indexado en metadata, reordenar para aplicar el `Filter` primero (reduce el espacio de búsqueda de HNSW).

**Criterio ST2.3.3:** Query con filtro de metadata que retiene el 10% de los vectores (alta selectividad): latencia con predicate pushdown es < 20% de la latencia sin predicate pushdown.

**ST2.3.4** — Actualizar `src/executor.rs` para consumir un `PhysicalPlan` (derivado del `LogicalPlan` optimizado). Mantener el path antiguo bajo un feature flag durante la transición para no romper tests existentes.

**Criterio ST2.3.4:** Todos los tests de `tests/certification/hybrid_retrieval_quality.rs` pasan con el nuevo executor.

**Criterio de aceptación T2.3:** Un query híbrido con filtro de metadata de alta selectividad (< 20% de registros) completa en < 50% del tiempo del mismo query sin filtro. El planner logea en modo debug qué optimizaciones aplicó.

---

### T2.4 — Versionado de formato de serialización

**Objetivo:** Los datos binarios en disco (WAL, índices, snapshots) están serializados con bincode sin versión de formato. Si se cambia cualquier struct serializado, los datos existentes son incompatibles sin aviso. Para una base de datos, esto es un bloqueante silencioso de producción.

#### Subtareas

**ST2.4.1** — Añadir un header de versión a los archivos del WAL, al archivo del índice HNSW (`neural_index.bin`), y a los snapshots de Fjall. El header debe contener: magic bytes (4B), versión de formato (u16), versión del schema (u16), timestamp de creación (u64). Implementar verificación al abrir: si la versión del archivo es incompatible con la versión del código, emitir error descriptivo con instrucciones de migración.

**Criterio ST2.4.1:** Abrir un archivo de versión anterior con código nuevo produce un error descriptivo como: `VantaError::IncompatibleFormat { file_version: 1, code_version: 2, migration_guide: "run vanta-cli migrate --db PATH" }`.

**ST2.4.2** — Documentar en `CHANGELOG.md` la versión actual del formato (nombrarla `format_v1`) y el compromiso de mantener compatibilidad hacia atrás por dos versiones menores.

**Criterio ST2.4.2:** El CHANGELOG incluye una sección "Format Compatibility" que documenta qué versiones de código pueden leer qué versiones de formato.

**Criterio de aceptación T2.4:** Un test `tests/schema_evolution.rs` crea datos con código v1, los abre con código v2 (simulado con una conversión intencionada), y verifica que o bien la conversión es exitosa o bien el error es descriptivo y contiene instrucciones de acción.

---

## FASE 3: Validación de Producción y DX
**Duración:** Semanas 10–16 | **Prioridad:** P1

### Objetivo de Fase
Demostrar reliability production-grade con datos reproducibles, tener un benchmark competitivo publicable, finalizar el pipeline de distribución de wheels, y conseguir 3–5 usuarios piloto con datos de uso reales.

**Criterio de aceptación de Fase:** Benchmark competitivo publicado en `docs/BENCHMARKS.md` con comparativa vs LanceDB y Chroma usando ann-benchmarks. Al menos 3 pilotos documentados con SLA P99 < 10ms en sus casos de uso reales. Wheels para Linux/macOS/Windows buildables y firmables con `cibuildwheel` en CI.

---

### T3.1 — Chaos testing expandido y validación de durabilidad

**Objetivo:** Demostrar que VantaDB sobrevive kill -9, disk full, y corrupción de datos con recuperación 100% correcta. Esto convierte el WAL hardening de un claim técnico en una garantía verificable públicamente.

#### Subtareas

**ST3.1.1** — Implementar los siguientes escenarios de caos en `tests/storage/chaos_integrity.rs` usando el crate `failpoints` (ya en el proyecto):
- Kill -9 durante WAL append (después de escribir pero antes de fsync)
- Kill -9 durante compactación de Fjall
- Kill -9 durante rebuild del índice HNSW
- Corrupción de bytes aleatorios en mitad del WAL
- Corrupción de bytes en el header del archivo de índice

**Criterio ST3.1.1:** Todos los escenarios son ejecutables en CI (no requieren privilegios especiales). Cada escenario verifica: (1) el motor abre después del fallo, (2) todos los datos con WAL flusheado antes del fallo son recuperables, (3) ningún dato corrupto es visible post-recuperación.

**ST3.1.2** — Crear un script `dev-tools/chaos_loop.sh` que ejecuta 1,000 iteraciones de caos (kill -9 aleatorio durante operaciones mixtas). Este script debe poder correrse en CI como parte del workflow `heavy_certification.yml`.

**Criterio ST3.1.2:** 1,000 iteraciones de `chaos_loop.sh` completan sin ningún fallo de integridad en hardware de referencia.

**ST3.1.3** — Documentar los resultados en `docs/operations/RELIABILITY_GATE.md` con los comandos exactos para reproducirlos, timestamps, y ambiente de ejecución.

**Criterio de aceptación T3.1:** El README puede declarar honestamente "crash-safe WAL with 1,000-iteration chaos test certification" con un enlace al documento de evidencia.

---

### T3.2 — Benchmark competitivo vs LanceDB y Chroma

**Objetivo:** VantaDB no puede posicionarse competitivamente sin datos comparativos reproducibles. Este benchmark es el activo de marketing técnico más importante del proyecto.

#### Subtareas

**ST3.2.1** — Elegir un metodología justa: usar ann-benchmarks (https://ann-benchmarks.com) como framework estándar de la industria. Implementar un conector de VantaDB para ann-benchmarks que permita ejecutar los mismos datasets que LanceDB y Chroma.

**Criterio ST3.2.1:** El conector de VantaDB para ann-benchmarks ejecuta correctamente con el dataset `glove-100-angular` (100d, Coseno) y `sift-128-euclidean` (128d, L2).

**ST3.2.2** — Ejecutar la comparativa en hardware controlado y documentar: (1) VantaDB Python SDK, (2) LanceDB Python SDK, (3) ChromaDB Python SDK. Métricas: throughput de ingesta, P50/P95/P99 de búsqueda, Recall@10, RAM en reposo, RAM bajo carga.

**Criterio ST3.2.2:** Los resultados son reproducibles con un script de un solo comando en cualquier máquina con Python 3.10+ y los paquetes instalados.

**ST3.2.3** — Publicar los resultados en `docs/BENCHMARKS.md` con metodología completa, hardware, y fecha. Ser honesto sobre donde VantaDB pierde (probable en ingesta throughput vs LanceDB) y donde gana (probable en durabilidad y hybrid search).

**Criterio ST3.2.3:** Ningún número en el benchmark es el mejor en su categoría artificialmente (no cherry-pick de métricas). Los resultados incluyen casos donde los competidores son superiores.

**Criterio de aceptación T3.2:** Benchmark public en `docs/BENCHMARKS.md` con datos de VantaDB vs LanceDB vs Chroma, metodología reproducible, y análisis honesto de trade-offs.

---

### T3.3 — Pipeline de wheels para distribución (cibuildwheel + Sigstore)

**Objetivo:** El workflow actual `python_wheels.yml` genera wheels pero aún hay elementos pendientes (señalados en tareas F6-13 y CODE-11). El objetivo es que el pipeline sea completamente automatizado, firmado, y publicado en PyPI producción con tag-gate.

#### Subtareas

**ST3.3.1** — Verificar que `cibuildwheel` está configurado correctamente para las tres plataformas: `manylinux2014_x86_64`, `macosx_10_12_x86_64`, `macosx_11_0_arm64`, `win_amd64`. Los wheels deben incluir todas las dependencias nativas de Fjall sin requerir que el usuario instale herramientas de compilación.

**Criterio ST3.3.1:** `pip install vantadb-py` funciona en una máquina limpia (sin Rust toolchain instalado) en las tres plataformas.

**ST3.3.2** — Configurar OIDC Trusted Publishing en PyPI para `vantadb-py`. Eliminar cualquier API token de larga vida en los secrets del repo. El publishing debe activarse automáticamente en push de tag `v*.*.*`.

**Criterio ST3.3.2:** Un push de tag `v0.1.5` en GitHub publica automáticamente el wheel en PyPI sin intervención manual, usando OIDC.

**ST3.3.3** — Añadir verificación de integridad post-publicación al CI: descargar el wheel recién publicado de PyPI, verificar la firma Sigstore, e importar el módulo en Python con una query de smoke test.

**Criterio ST3.3.3:** El workflow de release incluye un step `verify_published_wheel` que falla si la firma es inválida o el módulo no importa correctamente.

**Criterio de aceptación T3.3:** `pip install vantadb-py` en una máquina limpia en Linux, macOS, y Windows instala el wheel firmado sin errores. El proceso es completamente automatizado sin intervención manual.

---

### T3.4 — Programa de pilotos controlados

**Objetivo:** 3–5 usuarios reales usando VantaDB en producción durante 14 días es la validación de mercado más importante que puede tener el proyecto en este momento. Convierte el "funciona en demos" en "funciona en casos reales".

#### Subtareas

**ST3.4.1** — Identificar a los 3–5 early adopters ideales. Los perfiles más apropiados son: desarrolladores que ya usen LangChain o LlamaIndex con ChromaDB o en memoria, que trabajen en proyectos de AI agents locales, y que estén frustrados con la pérdida de datos al reiniciar. Canales de búsqueda: Reddit /r/LocalLLaMA, Discord de LangChain, Discord de LlamaIndex.

**Criterio ST3.4.1:** Tenemos 3 commits de personas externas o 3 usuarios que reportan usarlo en proyectos reales via GitHub Issues.

**ST3.4.2** — Preparar un paquete de onboarding de pilotos: quickstart de 15 minutos, template de agente con LangChain + VantaDB, canal de soporte directo (Discord o email dedicado), y un formulario de feedback al final de los 14 días.

**Criterio ST3.4.2:** Un developer con Python básico y sin experiencia en Rust puede tener un agente con memoria persistente usando VantaDB funcionando en < 15 minutos.

**ST3.4.3** — Recolectar y documentar los resultados de los pilotos en `docs/case_studies/`. Los case studies son el material de marketing técnico más valioso y son necesarios para fundraising.

**Criterio ST3.4.3:** Al menos 2 case studies documentados con: problema inicial, solución con VantaDB, métricas comparadas (latencia, confiabilidad, complejidad operacional).

**Criterio de aceptación T3.4:** Al menos 3 pilotos completados con feedback positivo documentado. Al menos 1 testimonial público (tweet, post, GitHub discussion) de un usuario real.

---

## FASE 4: Community Launch
**Duración:** Semanas 14–20 | **Prioridad:** P1 (después de Fase 1 y 2 resueltos)

### Objetivo de Fase
De 1 star a 1,000+ stars. De 0 forks a 20+ forks. De 0 contributors externos a 5+. Esto no es vanidad: la tracción de comunidad es el prerequisito para cualquier conversación de fundraising y para que el proyecto sea sostenible a largo plazo.

**Criterio de aceptación de Fase:** 1,000+ GitHub stars, 20+ forks, 5+ contributors externos, 1 artículo en HackerNews en top 10 de "Show HN", 200+ miembros en Discord.

---

### T4.1 — Demo content técnico (asciinema + video + GIF)

**Objetivo:** El contenido de mayor conversión para proyectos técnicos es el demo que muestra el valor en 60 segundos o menos. Sin esto, el README es texto y el proyecto es invisible.

#### Subtareas

**ST4.1.1** — Crear un demo de asciinema (grabación de terminal) que muestre en secuencia: (1) `pip install vantadb-py`, (2) 10 líneas de Python que crean un agente con memoria persistente, (3) kill del proceso y restart, (4) el agente recupera sus memorias. Duración total: 90 segundos o menos.

**Criterio ST4.1.1:** El demo es reproducible por cualquier persona con Python 3.8+ en cualquier plataforma. Subido a `asciinema.org` o similar y embebido en el README.

**ST4.1.2** — Crear un GIF animado (loop de 30 segundos) extraído del asciinema que muestre el momento "aha": el agente recordando información después de un restart. Este GIF va en el README principal y en todos los posts de marketing.

**Criterio ST4.1.2:** El GIF tiene menos de 3MB y es legible en pantalla móvil. El texto en el terminal es visible sin necesidad de zoom.

**ST4.1.3** — Crear un `examples/python/` con ejemplos completos y comentados: `agent_memory.py` (agente básico con memoria), `langchain_rag.py` (RAG con LangChain), `llamaindex_search.py` (búsqueda con LlamaIndex). Cada ejemplo debe funcionar copiando y pegando.

**Criterio ST4.1.3:** Los tres ejemplos funcionan sin modificación en Python 3.8+ con las dependencias declaradas. Tienen comentarios explicativos para cada paso no obvio.

**Criterio de aceptación T4.1:** Un developer puede ver el valor de VantaDB en < 60 segundos de mirar el README. El README tiene: GIF animado, quickstart en < 10 líneas, enlace al demo completo.

---

### T4.2 — Artículos técnicos del blog (serie de 3)

**Objetivo:** Los artículos técnicos de alta calidad son el canal de adquisición más sostenible para herramientas de developer. Generan tráfico orgánico, credibilidad, y backlinks. El timing óptimo es DESPUÉS de resolver la Fase 1 (tener números reales que publicar).

#### Subtareas

**ST4.2.1** — Artículo 1: "Why I Built a Local Memory Engine for AI Agents in Rust" — Historia del proyecto, el problema que resuelve, el diseño técnico del WAL y el hybrid search. Tono: technical blog post para developers, no marketing. Longitud objetivo: 2,000–3,000 palabras. Publicar en dev.to, Medium, y el GitHub Blog del repositorio.

**Criterio ST4.2.1:** El artículo recibe > 100 reacciones en dev.to o > 500 lecturas en la primera semana.

**ST4.2.2** — Artículo 2: "How Hybrid Search Works: BM25 + HNSW + RRF in Practice" — Tutorial técnico sobre el algoritmo de retrieval híbrido implementado en VantaDB, con código de ejemplo y explicación matemática de RRF. Este artículo es SEO-friendly para búsquedas sobre hybrid search y RAG.

**Criterio ST4.2.2:** El artículo aparece en los primeros 10 resultados de Google para "hybrid search bm25 hnsw rust" dentro de 30 días de publicación.

**ST4.2.3** — Artículo 3: "SQLite for AI Agents: Benchmarks and Architecture Decisions" — Comparativa honesta con LanceDB y Chroma usando los datos del T3.2. El artículo incluye los casos donde VantaDB no gana (ingesta masiva vs LanceDB) y explica las trade-offs. La honestidad construye más credibilidad que el cherry-picking.

**Criterio ST4.2.3:** El artículo es referenciado o compartido por al menos 3 personas con > 1,000 seguidores en Twitter/X o LinkedIn.

**Criterio de aceptación T4.2:** Serie de 3 artículos publicados. Al menos uno de los tres genera > 50 clicks de vuelta al repositorio de GitHub (medible con GitHub Traffic > Referring sites).

---

### T4.3 — HackerNews Show HN

**Objetivo:** Un post exitoso en HackerNews "Show HN" es el canal de distribución de mayor ROI para herramientas de developer. Un post que entra en top-10 puede generar 500–2,000 stars en 24 horas. La ventana para tener éxito es cuando el proyecto tiene: demos claros, benchmark honesto, README impecable, y respuestas preparadas para críticas.

#### Subtareas

**ST4.3.1** — Preparar el post de HackerNews con el formato correcto: "Show HN: VantaDB – Embedded hybrid search (BM25+HNSW) for AI agents, written in Rust". El título debe ser descriptivo, no marketero. La descripción inicial del post debe ser técnica (2–3 párrafos máximo), no un pitch.

**Criterio ST4.3.1:** Draft del post revisado por al menos 2 personas técnicas externas al proyecto antes de publicar.

**ST4.3.2** — Preparar respuestas anticipadas a las críticas más probables: (1) "¿Por qué no usar LanceDB?", (2) "¿Qué pasa con Chroma?", (3) "¿El HNSW escala a millones de vectores?", (4) "¿Por qué otro vector DB?". Las respuestas deben ser honestas sobre los límites actuales y claras sobre la diferenciación real (embedded-first, WAL durability, hybrid search nativo).

**Criterio ST4.3.2:** Documento interno con respuestas a las 10 críticas más probables. No publicar hasta tener este documento listo.

**ST4.3.3** — Timing: publicar el martes o miércoles entre 9am–12pm EST (horario de mayor tráfico en HN). Monitorear y responder todos los comentarios en las primeras 6 horas (las respuestas del autor aumentan significativamente el engagement).

**Criterio ST4.3.3:** Presencia activa del autor respondiendo comentarios durante las primeras 6 horas post-publicación.

**Criterio de aceptación T4.3:** El post llega a la página principal de HackerNews (> 50 puntos) y recibe > 10 comentarios técnicos sustanciales. El repositorio recibe > 200 stars en las 24 horas post-publicación.

---

### T4.4 — Comunidad Discord + Good First Issues

**Objetivo:** Convertir el interés generado por HackerNews en contribuidores activos. Un proyecto sin contribuidores externos no es sostenible a largo plazo.

#### Subtareas

**ST4.4.1** — Crear un servidor Discord con los siguientes canales: `#announcements`, `#general`, `#help-and-support`, `#showcase` (usuarios mostrando qué construyeron), `#contributing`, `#roadmap-discussions`. El servidor debe tener reglas claras y bienvenida automática.

**Criterio ST4.4.1:** El servidor de Discord está operativo y tiene al menos 20 miembros en la primera semana post-HackerNews.

**ST4.4.2** — Marcar en GitHub Issues 5–10 issues con el label `good first issue`. Estos issues deben ser genuinamente abordables por alguien con conocimiento básico de Rust: documentación, ejemplos adicionales, tests de casos edge, mejoras de mensajes de error. Cada issue debe tener: contexto claro, criterio de aceptación, y puntero a los archivos relevantes.

**Criterio ST4.4.2:** Al menos 2 de los `good first issues` son reclamados por contributors externos dentro de las 2 semanas post-launch.

**ST4.4.3** — Responder todos los Issues y PRs en < 48 horas. Esta es la métrica de salud más importante de la comunidad. Un issue sin respuesta en > 3 días ahuyenta a contributors potenciales.

**Criterio ST4.4.3:** Tiempo de primera respuesta promedio a Issues y PRs: < 24 horas en las primeras 4 semanas post-launch.

**Criterio de aceptación T4.4:** 100+ miembros en Discord, 5+ contributors externos con al menos un commit merged, el backlog de Issues tiene respuestas en < 48 horas.

---

## FASE 5: Preparación Pre-seed
**Duración:** Semanas 18–24+ | **Prioridad:** P2 (después de tracción demostrada)

### Objetivo de Fase
Tener los prerequisitos para levantar una ronda pre-seed de $250K–$500K a una valuación de $2M–$4M. Los prerequisitos son no negociables: los inversores de infra databases verifican los números técnicos antes de los números financieros.

**Criterio de aceptación de Fase:** 1,000+ GitHub stars, 3+ case studies de producción documentados, Python SDK p50 < 20ms, benchmark competitivo publicado, deck de 10 slides listo, y al menos 5 conversaciones con VCs o angels del espacio de developer infrastructure.

---

### T5.1 — Deck de inversores y one-pager

**Objetivo:** Los inversores de early stage evalúan: (1) equipo, (2) problema real documentado, (3) tamaño de mercado, (4) diferenciación técnica defendible. El deck debe ser honesto sobre el estado actual y claro sobre los hitos financiados.

#### Subtareas

**ST5.1.1** — Crear un deck de 10 slides: (1) problema (fragmentación de stack para AI agents locales), (2) solución (VantaDB como SQLite for AI Agents), (3) cómo funciona (diagrama técnico simple), (4) diferenciación vs LanceDB/Chroma (tabla honesta), (5) mercado (TAM: embedded DB market + AI infra market), (6) tracción (stars, downloads, pilotos, case studies), (7) modelo de negocio (Open Core), (8) equipo, (9) uso del capital, (10) hitos a 12 meses.

**Criterio ST5.1.1:** El deck puede ser presentado en 10 minutos y responde "por qué este equipo, por qué este problema, por qué ahora" sin ambigüedad.

**ST5.1.2** — Preparar due diligence técnico proactivo: documentar en un repositorio privado los resultados del benchmark competitivo, el test suite completo, los ADRs, y los case studies de pilotos. Los inversores técnicos pedirán esto.

**Criterio ST5.1.2:** Carpeta de due diligence lista con: benchmark reproducible, evidencia de caos testing, 2+ case studies, ADRs de decisiones arquitectónicas clave.

**Criterio de aceptación T5.1:** Deck en formato que puede enviarse por email (PDF) y presentarse en Zoom sin ayudas adicionales.

---

### T5.2 — VantaDB Cloud Beta (Fly.io)

**Objetivo:** Una versión hosted de VantaDB (aunque sea beta) abre la posibilidad de revenue inicial y valida el modelo Open Core. Fly.io es la plataforma correcta para esto: bajo costo, deploys rápidos, y la comunidad de developers que usa Fly.io es exactamente el target de VantaDB.

#### Subtareas

**ST5.2.1** — Desplegar `vantadb-server` en Fly.io con persistencia en un volumen persistente. Configurar HTTPS automático. Este es el MVP del cloud: un servidor VantaDB accesible con una URL, sin multi-tenancy ni billing aún.

**Criterio ST5.2.1:** `curl https://vantadb.dev/health` retorna 200 OK. El servidor persiste datos entre reinicios.

**ST5.2.2** — Añadir autenticación Bearer token básica al server (necesaria para el beta). Sin esto, el servidor expuesto es un riesgo de seguridad y los usuarios beta no pueden usarlo con datos reales.

**Criterio ST5.2.2:** Requests sin `Authorization: Bearer <token>` retornan 401. Los tokens se configuran via variable de entorno.

**ST5.2.3** — Invitar a los 3–5 pilotos del T3.4 a usar la versión cloud y recolectar feedback sobre la experiencia cloud vs embedded.

**Criterio ST5.2.3:** Al menos 2 pilotos usan la versión cloud activamente por > 7 días.

**Criterio de aceptación T5.2:** Beta cloud funcional con autenticación básica, accessible publicamente, con al menos 2 usuarios activos.

---

## MEJORAS FUERA DE FASES (Pistas Paralelas)

Estas mejoras no dependen de ninguna fase específica ni bloquean el critical path, pero tienen impacto compuesto significativo si se trabajan en paralelo. Se pueden ir atacando según capacidad disponible.

---

### Mejora P1 — Seguridad del servidor (auth + TLS)

El servidor HTTP actual expone datos sin autenticación. Esto es aceptable para desarrollo local pero es un bloqueante para cualquier uso en red. Esta mejora debe completarse antes de la Fase 4 (launch) para no publicar un proyecto con vulnerabilidades obvias.

**Qué implementar:** Bearer token auth (`Authorization: Bearer <token>` en cada request). TLS via `rustls` con certificados auto-firmados para desarrollo y Let's Encrypt para producción. Rate limiting básico (máximo 100 requests/minuto por IP) para prevenir DoS triviales.

**Criterio de aceptación:** Requests sin token retornan 401. La conexión TLS funciona en curl sin flags de ignorar certificados. Rate limit activo retorna 429 al exceder el threshold.

---

### Mejora P2 — OpenTelemetry + logs estructurados JSON

Actualmente la telemetría es básica. Para que los usuarios de producción puedan monitorear VantaDB, necesitan traces exportables. Esta mejora usa `tracing-opentelemetry` y `tracing-subscriber` con formatter JSON.

**Qué implementar:** Instrumentar con `tracing::span!` las operaciones críticas: WAL append, HNSW search, BM25 query, RRF fusion. Exportar via OTLP a Jaeger/Grafana. Añadir `trace_id` a todos los logs estructurados para correlación.

**Criterio de aceptación:** Un span de `hybrid_search` incluye child spans de `bm25_query` y `hnsw_search` con duraciones individuales. Los logs son JSON parseables con `jq`.

---

### Mejora P3 — Tokenizador avanzado (Unicode folding + stopwords)

El tokenizador actual (`lowercase-ascii-alnum`) no maneja texto en otros idiomas ni variantes léxicas. Para uso real con documentos en español, francés, alemán, etc., esto es un bloqueante de calidad de recall.

**Qué implementar:** Integrar `tantivy-tokenizer` como dependencia opcional (feature flag `advanced-tokenizer`). Implementar: Unicode normalization (NFC), case folding Unicode, stopwords opcionales por idioma, stemming básico (snowball). Mantener el tokenizador actual como default para compatibilidad.

**Criterio de aceptación:** Con `advanced-tokenizer` activado, una búsqueda por "rusés" encuentra documentos que contienen "ruses", "RUSÉS", y "Rusés". Los stopwords en español ("el", "la", "de") no afectan el ranking.

---

### Mejora P4 — Phrase queries y snippets

BM25 texto-only funciona, pero sin phrase queries ("exact phrase search") la búsqueda lexical es competitivamente débil. Phrase queries requieren que el índice almacene posiciones de términos, no solo frecuencias.

**Qué implementar:** Añadir almacenamiento de posiciones de términos al índice invertido en `src/text_index.rs`. Implementar el operador phrase query en el planner. Añadir generación de snippets (extractos con el término resaltado) como campo opcional en los resultados.

**Criterio de aceptación:** `search_memory(namespace, text_query='"exact phrase"', top_k=5)` retorna solo documentos con la frase exacta en ese orden. Los resultados incluyen un campo `snippet` con el contexto del match.

---

### Mejora P5 — Go SDK (alta demanda de la comunidad)

Para el segmento de developers backend que usa Go (microservicios, herramientas CLI, pipelines de datos), un SDK de Go amplía significativamente el mercado. VantaDB expone una capa C-ABI en `src/engine.rs` que puede usarse con `cgo` + `cbindgen`.

**Qué implementar:** Generar bindings C con `cbindgen`. Crear un módulo Go que wrappea el C API: `go get github.com/ness-e/vantadb-go`. Implementar los mismos métodos que el SDK Python: `Put`, `Get`, `Delete`, `SearchMemory`, `Flush`, `Close`.

**Criterio de aceptación:** El ejemplo de agente en Go se ejecuta con `go run main.go` sin instalar Rust ni herramientas adicionales (los binarios nativos se incluyen como assets del módulo Go).

---

### Mejora P6 — ADRs completos y onboarding técnico

Los 3 ADRs existentes (Config, WAL, Sync/Async) son excelentes pero el proyecto tiene más de 15 decisiones arquitectónicas sin documentar que confunden a contributors externos y a inversores técnicos.

**Qué implementar:** ADR-004: StorageBackend trait y estrategia de migración Fjall. ADR-005: Decisión de mantener HNSW propio vs usar librería externa (hnswlib, usearch). ADR-006: Diseño del planner AST/IR. ADR-007: Modelo de concurrencia (sync core, async server). Formato: contexto → decisión → consecuencias (mismo que los ADRs existentes).

**Criterio de aceptación:** Un developer senior de Rust puede leer los ADRs en 30 minutos y entender por qué las decisiones clave son como son, sin necesidad de leer el código fuente.

---

## PLAN DE MARKETING

### Principio Fundamental

El marketing para herramientas de developer técnico funciona diferente al marketing de consumidor. Los developers desconfían del marketing explícito y confían en pares. La estrategia correcta es: **demostrar el valor técnico honestamente → dejar que los developers lo descubran → amplificar lo que ya funciona**. No invertir en publicidad pagada, no escribir posts marketeros, no pedir reviews falsas.

---

### Cuándo Lanzar (Timing Crítico)

Este es el error más costoso que puede cometer el proyecto: lanzar antes de tener los números. Un Show HN que llega sin el Python SDK optimizado, o con 127 segundos en SIFT, generará críticas técnicas que quedan indexadas en Google permanentemente y dañan la reputación del proyecto.

**La fecha de lanzamiento es una función de hitos técnicos, no de un calendario.** Los hitos mínimos para lanzar son:

Primero, Python SDK p50 < 20ms para búsqueda vectorial a 10K vectores (Fase 1 completada). Sin esto, cualquier developer que evalúe el proyecto encontrará la latencia inaceptable en las primeras pruebas.

Segundo, SIFT benchmark sin bug de 127 segundos, con soporte L2 nativo (T1.2 completado). Esto es necesario para que el benchmark competitivo sea publicable.

Tercero, al menos un case study de piloto real documentado (T3.4 completado). Sin esto, el proyecto es teoría, no práctica.

Cuarto, el README tiene: demo GIF animado, benchmark reproducible, tabla de comparación honesta con LanceDB y Chroma.

**Estimación de timing:** Si se empieza a trabajar en la Fase 1 ahora con dedicación full-time, estos hitos son alcanzables en 8–12 semanas. Con tiempo parcial (20 horas/semana), en 14–18 semanas. **El lanzamiento no debería ocurrir antes de la semana 12 desde ahora, y preferiblemente en la semana 14–16.**

---

### Pre-lanzamiento (Semanas 8–14): Seeding Silencioso

El objetivo de esta etapa no es generar tracción masiva sino construir los activos y la audiencia semilla que amplificarán el lanzamiento. Este trabajo se hace en paralelo con las Fases 2 y 3.

**Actividad 1 — Publicar el artículo técnico 1 (T4.2.1).** Este artículo no es un anuncio; es educación técnica. Se publica en dev.to y se comparte en /r/rust y /r/ProgrammingLanguages. El objetivo es que 50–100 developers lean el artículo y empiecen a seguir el repositorio. No se menciona que hay un lanzamiento próximo.

**Actividad 2 — Participar en comunidades existentes.** Antes de pedir atención, contribuir a conversaciones relevantes en /r/LocalLLaMA (sobre RAG local), Discord de LangChain (sobre persistence backends), y Discord de LlamaIndex. Responder preguntas sobre WAL, hybrid search, y AI memory. No hacer spam del proyecto; genuinamente ayudar con problemas. Cuando sea relevante, mencionar: "Esto es exactamente el problema que VantaDB resuelve. Aquí está el enlace si te interesa."

**Actividad 3 — Conseguir los 3–5 pilotos (T3.4).** Contactar directamente a 15–20 developers identificados en las comunidades que mencionaron tener problemas de memory persistence en sus agentes. Un mensaje directo y honesto: "Estoy construyendo VantaDB para resolver exactamente este problema. ¿Estarías dispuesto a probarlo y darme feedback?"

**Actividad 4 — GitHub repo polish.** El repositorio debe estar inmaculado antes del lanzamiento: README con GIF, descripción correcta, topics relevantes (`python rust embedded-database ai hnsw bm25 hybrid-search local-first`), el website del repo apunta a documentación funcional, CONTRIBUTING.md claro y amigable, template de Issues.

---

### Lanzamiento (Semana 14–16): La Semana del HN

El lanzamiento se concentra en una semana específica con una secuencia coordinada.

**Día -7:** Publicar artículo técnico 2 (T4.2.2, sobre hybrid search). El artículo debe circular durante una semana antes del Show HN para que ya haya una audiencia familiarizada con el proyecto.

**Día 0 — Lunes:** Crear el servidor Discord y publicar el README final con el GIF animado. Avisar a los pilotos del programa que el lanzamiento público es inminente y pedirles que estén disponibles para responder preguntas ese día.

**Día 1 — Martes a las 9am EST:** Publicar el Show HN. El titulo exacto del post: "Show HN: VantaDB – Embedded Rust DB for AI Agent Memory (BM25 + HNSW + WAL)". En los primeros 15 minutos: votar el post desde cuentas propias y de amigos cercanos (máximo 3–4 votos tempranos para ayudar con la distribución inicial sin hacerlo obvio). Estar disponible durante las siguientes 6 horas para responder comentarios.

**Día 1 — Tarde:** Publicar en /r/rust (flair "Project"): "I built VantaDB: an embedded Rust database for AI agent memory. Here's what I learned about HNSW + BM25 hybrid search". El post de Reddit es técnico y diferente al Show HN; no es el mismo texto.

**Día 2:** Publicar en /r/MachineLearning (si el Show HN fue bien) y /r/LocalLLaMA. En Twitter/X, un hilo técnico de 5–7 tweets sobre el diseño del WAL y por qué importa para AI agents. En LinkedIn, el artículo técnico de blog reposteado.

**Día 3–7:** Responder todos los comentarios en todos los canales. Agradecer a personas que compartan el proyecto. Integrar el feedback público en el backlog (Issues públicos con label "community-request").

---

### Post-lanzamiento (Semanas 16–24): Amplificación

En esta etapa, el objetivo es convertir la atención inicial en comunidad sostenida y en pilotos adicionales.

**Actividad 1 — Publicar artículo técnico 3** (T4.2.3, benchmark competitivo) dos semanas después del Show HN. Este es el artículo más importante para SEO: la gente busca "vantadb vs lancedb vs chroma" después de descubrir el proyecto.

**Actividad 2 — Conferencias y meetups.** Enviar CFPs a: RustConf (si hay call abierto), PyCon Lightning Talk (demo de 5 minutos), y meetups locales o virtuales de AI/ML. Las conferencias no son solo para hablar; son para encontrar a los primeros 20 contributors y early adopters con nombres y caras.

**Actividad 3 — YouTube tutorial.** Un video de 15–20 minutos en YouTube: "Building an AI Agent with Persistent Memory Using VantaDB". No es producción profesional; pantalla + voz es suficiente. YouTube es el segundo buscador más grande y hay poca competencia para contenido técnico específico sobre herramientas de developer databases.

**Actividad 4 — Newsletter submissions.** Enviar el proyecto a: Rust Weekly, TLDR (tech), Python Weekly, AI Digest. Estos newsletters son gratuitos y llegan a audiencias altamente técnicas y relevantes.

**Actividad 5 — Office hours en Discord.** Cada dos semanas: sesión de office hours en el Discord donde cualquier developer puede preguntar sobre el proyecto, el diseño técnico, o cómo integrarlo. Esto genera credibilidad como maintainer accesible y genera contenido para futuros FAQs.

---

### Métricas de Marketing por Etapa

Estas métricas deben medirse semanalmente. Si después de 4 semanas de launch no se alcanza el mínimo, hay que iterar el mensaje o el target audience.

| Etapa | Métrica | Mínimo | Objetivo |
|---|---|---|---|
| Pre-launch (sem 8–14) | Artículo 1 lecturas | 200 | 500+ |
| Pre-launch | Pilotos confirmados | 3 | 5 |
| Launch week (sem 14–16) | Show HN puntos | 50 | 150+ |
| Launch week | Stars nuevas | 200 | 500 |
| Launch +4 semanas | Stars totales | 500 | 1,000+ |
| Launch +4 semanas | Discord miembros | 100 | 300 |
| Launch +8 semanas | Contributors externos | 3 | 10 |
| Launch +12 semanas | Downloads PyPI/semana | 500 | 2,000 |
| Launch +16 semanas | Case studies | 2 | 5 |

---

## RESUMEN DE DEPENDENCIAS CRÍTICAS

Estas son las dependencias no negociables que no pueden saltarse:

Fase 1 (HNSW scalability) es prerequisito de todo marketing público. Sin Python SDK < 20ms, el lanzamiento genera críticas que se quedan en internet.

Fase 2 (Hardening) es prerequisito de Fase 5 (pre-seed). Ningún inversor serio de infra databases invierte en un proyecto con thread starvation no resuelto en producción.

T3.4 (Pilotos) debe ocurrir antes del Show HN. Los case studies son el argumento más fuerte del post de HackerNews.

T3.2 (Benchmark competitivo) debe publicarse antes del Show HN. La primera pregunta en HN será "¿cómo comparas con LanceDB/Chroma?" y necesitas una respuesta con datos, no con palabras.

El marketing amplifica lo que ya existe. No puede crear valor que no hay. La secuencia correcta es siempre: construir → validar → amplificar. No al revés.

---

---

## MATRIZ DE RIESGOS POR FASE

Cada riesgo tiene probabilidad (1–5), impacto (1–5), y score de riesgo = P × I. Los riesgos con score >= 15 son bloqueantes críticos que requieren plan de mitigación antes de iniciar la fase.

### Riesgos Fase 1 (HNSW Scalability)

| ID | Riesgo | P | I | Score | Mitigación | Trigger de Alerta |
|---|---|:---:|:---:|:---:|---|---|
| R1.1 | La implementación HNSW sí usa multi-layer correctamente y el problema de scalability es otro (mmap exclusivamente) | 3 | 3 | 9 | Auditar código antes de implementar. Si multi-layer ya está correcto, focalizar en layout de disco desde el inicio. | ST1.1.1 revela multi-layer ya implementado |
| R1.2 | El re-layout antilocatario requiere reescribir el formato de serialización del índice, causando incompatibilidad de datos existentes | 4 | 4 | 16 | Implementar T2.4 (versionado de formato) en paralelo con T1.3. El re-layout es la primera "v2" del formato del índice. | ST1.3.1 muestra que el offset de nodos cambia con re-layout |
| R1.3 | El objetivo de Python SDK < 20ms p50 no es alcanzable sin reescribir el binding desde cero | 2 | 5 | 10 | Medir el overhead del boundary PyO3 con microbenchmark (ST1.4.3) antes de comprometerse al objetivo. Si el overhead es > 15ms estructuralmente, ajustar el objetivo a < 50ms con batch queries. | ST1.4.3 muestra overhead fijo > 15ms por llamada |
| R1.4 | La distancia L2 con SIMD produce resultados incorrectos por alineación de memoria | 2 | 4 | 8 | Test de correctness ST1.2.1 contra implementación de referencia escalar antes de activar SIMD. | ST1.2.1 falla con delta > 1e-4 |

### Riesgos Fase 2 (Hardening Arquitectónico)

| ID | Riesgo | P | I | Score | Mitigación | Trigger de Alerta |
|---|---|:---:|:---:|:---:|---|---|
| R2.1 | La migración de I/O síncrono a `spawn_blocking` introduce deadlocks difíciles de reproducir | 3 | 5 | 15 | Migrar un módulo a la vez, no en un PR masivo. Añadir tests de carga (wrk) después de cada módulo migrado. | Latencia P99 sube > 5x P50 en test de carga post-migración |
| R2.2 | mimalloc causa comportamiento diferente en Windows (allocator override no funciona igual) | 3 | 3 | 9 | Testear en CI con Windows runner después de activar mimalloc. Mantener asignador estándar en Windows si hay problemas. | `cargo test --target x86_64-pc-windows-msvc` falla tras activar mimalloc |
| R2.3 | El nuevo AST/LogicalPlan no puede representar todos los queries actuales, causando regresiones | 3 | 4 | 12 | Implementar el AST en modo "shadow": el nuevo planner corre en paralelo con el viejo y se comparan los resultados. El viejo sigue siendo el que retorna resultados hasta que ambos coincidan al 100%. | ST2.3.2 muestra discrepancias entre planner viejo y nuevo en > 5% de tests |
| R2.4 | El versionado de formato (T2.4) rompe compatibilidad con datos de usuarios del beta | 3 | 4 | 12 | Implementar herramienta de migración `vanta-cli migrate --db PATH` antes de publicar la versión que cambia el formato. Dar 60 días de aviso en CHANGELOG. | Primer tag de versión con formato v2 sin herramienta de migración disponible |

### Riesgos Fase 3 (Validación)

| ID | Riesgo | P | I | Score | Mitigación | Trigger de Alerta |
|---|---|:---:|:---:|:---:|---|---|
| R3.1 | Los pilotos no responden o no dan feedback útil | 4 | 3 | 12 | Tener 10 candidatos en pipeline para conseguir 3–5 activos. El feedback de cada piloto se captura en formulario estructurado, no en conversación libre. | Semana 13 y aún no hay 3 pilotos con > 7 días de uso activo |
| R3.2 | El benchmark competitivo muestra VantaDB inferior a Chroma en el caso de uso principal del target audience (RAG con 10K vectores) | 3 | 4 | 12 | No aplazar el benchmark hasta el lanzamiento. Correrlo en semana 10 para tener tiempo de iterar. Si los números son inferiores, iterar técnicamente antes de publicar. | Latencia Python SDK VantaDB > 2x Chroma en ann-benchmarks glove-100 |
| R3.3 | cibuildwheel falla en el runner de macOS ARM (M1/M2) por incompatibilidades de Fjall | 3 | 3 | 9 | Testear el pipeline de wheels en las tres plataformas en semana 10, no en la semana del lanzamiento. | `python_wheels.yml` falla en `macosx_11_0_arm64` target |

### Riesgos Fase 4 (Community Launch)

| ID | Riesgo | P | I | Score | Mitigación | Trigger de Alerta |
|---|---|:---:|:---:|:---:|---|---|
| R4.1 | El Show HN no llega a la página principal (< 50 puntos en primera hora) | 3 | 3 | 9 | Tener 5–8 amigos/conocidos técnicos listos para votar en los primeros 10 minutos. El título del post debe ser descriptivo, no marketero. | Post en < 10 puntos a los 30 minutos |
| R4.2 | Un comentario técnico destructivo en HN sobre el bug de scalability (si aún existe) destruye el lanzamiento | 4 | 5 | 20 | **Este es el riesgo más alto del documento.** No lanzar hasta que T1.1–T1.4 estén 100% completados y el Python SDK p50 < 20ms. Sin esto, no lanzar. | Cualquier evaluación pública del Python SDK antes de completar Fase 1 |
| R4.3 | Los early contributors externos hacen PRs que rompen tests o añaden complejidad innecesaria | 3 | 2 | 6 | CONTRIBUTING.md con requisitos claros de PR (tests requeridos, clippy limpio). Code review dentro de 48 horas de todos los PRs externos. | PR externo mergeado sin revisión que rompe CI |

### Riesgos Fase 5 (Pre-seed)

| ID | Riesgo | P | I | Score | Mitigación | Trigger de Alerta |
|---|---|:---:|:---:|:---:|---|---|
| R5.1 | La ventana de inversión en herramientas de AI infrastructure se cierra antes de que el proyecto tenga tracción | 3 | 4 | 12 | No depender del fundraising para existir. VantaDB debe ser sostenible como OSS antes de buscar capital. | Mercado de VC para AI infra tools cae > 40% según crunchbase |
| R5.2 | Un competidor grande lanza una solución directamente comparable y absorbe la audiencia | 3 | 4 | 12 | La diferenciación de VantaDB (embedded-first + WAL durability + hybrid search nativo) debe estar en el README y artículos antes del lanzamiento. Si un competidor lanza algo similar, ser más rápido en la comunidad, no en el código. | LanceDB o Chroma anuncian WAL durability + hybrid search nativo embebido |
| R5.3 | Los inversores requieren más crecimiento del que el equipo puede demostrar en 12 meses | 2 | 3 | 6 | Apuntar primero a angels y micro-VCs ($50K–$150K) antes de fondos institucionales. Los angels invierten en personas, los VCs en métricas. | Primeras 10 conversaciones con VCs resultan todas en "come back with $5K MRR" |

---

## ESTIMACIÓN DE ESFUERZO POR TAREA

Las estimaciones asumen un desarrollador senior de Rust con conocimiento del codebase existente. Las semanas son de trabajo enfocado (~40 horas/semana). Si el equipo trabaja a tiempo parcial, multiplicar por el factor inverso de dedicación.

| ID | Tarea | Dificultad | Esfuerzo estimado | Prerequisitos |
|---|---|:---:|---|---|
| **FASE 0** | | | | |
| T0.1 | Estabilizar test suite | Baja | 2–3 días | Ninguno |
| T0.2 | Limpieza Clippy | Baja | 1–2 días | T0.1 |
| T0.3 | Coherencia de versiones | Muy baja | 4 horas | Ninguno |
| T0.4 | Documentar frontera experimental | Baja | 1 día | T0.1, T0.2 |
| **FASE 1** | | | | |
| T1.1 | Auditoría y corrección HNSW multi-layer | Alta | 1–2 semanas | T0 completo |
| T1.2 | Soporte distancia Euclidiana (L2) | Media | 3–4 días | T1.1 |
| T1.3 | Layout antilocatario en mmap | Alta | 1–2 semanas | T1.1, T2.4 en paralelo |
| T1.4 | Optimización boundary Python–Rust | Media | 4–6 días | T1.1 |
| T1.5 | Actualizar benchmarks y docs | Baja | 2 días | T1.1–T1.4 |
| **FASE 2** | | | | |
| T2.1 | Eliminar bloqueos síncronos Tokio | Alta | 1–2 semanas | T0 completo |
| T2.2 | Asignador mimalloc | Muy baja | 1 día + 3 días de testing | T0 completo |
| T2.3 | Planner AST/IR + predicate pushdown | Muy alta | 3–4 semanas | T0 completo, T1.1 |
| T2.4 | Versionado de formato serialización | Media | 1 semana | T0 completo |
| **FASE 3** | | | | |
| T3.1 | Chaos testing expandido | Alta | 1–2 semanas | T2.1, T2.4 |
| T3.2 | Benchmark competitivo | Media | 1 semana (implementación) + 3 días (análisis) | T1 completo |
| T3.3 | Pipeline wheels cibuildwheel | Media | 4–5 días | T0.3 |
| T3.4 | Pilotos controlados | Baja (técnico) / Alta (gestión) | Ongoing, 4–6 semanas | T1 completo, T3.3 |
| **FASE 4** | | | | |
| T4.1 | Demo content (asciinema + GIF) | Baja | 2–3 días | T1 completo, T3.3 |
| T4.2 | Artículos técnicos (serie 3) | Media | 2–3 días por artículo | T1 completo, T3.2 |
| T4.3 | HackerNews Show HN | Baja (publicar) / Alta (preparar) | 2–3 días de preparación | T4.1, T4.2.1, T3.2 |
| T4.4 | Discord + Good First Issues | Baja | 2 días setup + ongoing | T4.3 |
| **FASE 5** | | | | |
| T5.1 | Deck inversores + one-pager | Media | 1 semana | T3.4, T4.3 con métricas |
| T5.2 | VantaDB Cloud Beta (Fly.io) | Alta | 2–3 semanas | T2.1, seguridad básica |
| **MEJORAS PARALELAS** | | | | |
| MP1 | Seguridad servidor (auth + TLS) | Media | 1 semana | T0 completo |
| MP2 | OpenTelemetry + JSON logs | Media | 1–2 semanas | T0 completo |
| MP3 | Tokenizador avanzado | Media | 1–2 semanas | T2.3 |
| MP4 | Phrase queries + snippets | Alta | 2–3 semanas | T2.3 |
| MP5 | Go SDK | Alta | 3–4 semanas | T0 completo, cbindgen |
| MP6 | ADRs completos | Baja | 3–4 días | Ongoing |

**Esfuerzo total estimado para llegar al lanzamiento (Fases 0–4):**
- Mínimo (si se trabaja en paralelo con 2 personas): 12–14 semanas
- Máximo (una persona, trabajo secuencial): 20–24 semanas
- Realista (una persona, 70% del tiempo en VantaDB): 16–18 semanas

---

## ECOSISTEMA DE INTEGRACIONES

VantaDB no vive en aislamiento. Su valor se multiplica con cada integración de calidad. El mapa de integraciones define en qué invertir primero basado en el tamaño de la audiencia de cada framework y la fricción de integración.

### Prioridad 1 — Ya implementadas (FEAT-01, completado)

Estas integraciones existen pero necesitan documentación de calidad y ejemplos completos para ser útiles como canal de adquisición.

**LangChain (`langchain-vantadb`):** La integración existe (`VantaDBVectorStore`). Lo que falta es: un tutorial completo publicado en la documentación de LangChain (se puede hacer como contribución a su repositorio de ejemplos), un ejemplo de RAG persistente de fin a fin con Ollama local, y submission a la lista oficial de vector stores de LangChain.

**Criterio:** VantaDB aparece en la documentación oficial de LangChain en la sección "Vector Stores" dentro de 60 días del lanzamiento.

**LlamaIndex (`llamaindex-vantadb`):** Similar situación. El adapter existe. Falta el ejemplo de fin a fin con LlamaIndex y un tutorial en el Hub de LlamaIndex.

**Criterio:** VantaDB aparece en la documentación oficial de LlamaIndex en la sección "Vector Stores" dentro de 60 días del lanzamiento.

### Prioridad 2 — Alta audiencia, implementación sencilla (Semanas 16–22)

**CrewAI:** Framework de multi-agent que tiene > 25,000 stars en GitHub y crecimiento rápido. Los agentes de CrewAI necesitan persistencia de memoria entre sesiones. La integración es: implementar `VantaDBMemory` como provider de memoria para CrewAI. La API de memory de CrewAI es sencilla (store/retrieve por agent_id). Esfuerzo estimado: 3–4 días.

**AutoGen (Microsoft):** Framework de multi-agent con respaldo corporativo. Tiene un sistema de memoria pluggable. Una integración con AutoGen llegaría a la audiencia enterprise que es el target de VantaDB Pro en el futuro.

**Haystack (deepset):** Framework de RAG con arquitectura de componentes. Un `VantaDBDocumentStore` es el componente de integración correcto. La audiencia de Haystack es enterprise y técnica.

**Mem0:** Librería específica de memory for AI agents que tiene > 20,000 stars. La integración de VantaDB como backend de almacenamiento para Mem0 llegaría directamente al público que ya usa herramientas de AI memory.

### Prioridad 3 — Alta audiencia, implementación más compleja (Semanas 22–30)

**LangGraph:** El framework de agent workflows de LangChain. Los workflows de LangGraph persisten estado entre nodos. VantaDB como checkpoint store para LangGraph es un caso de uso natural.

**Semantic Kernel (Microsoft):** Framework de AI para .NET y Python. Una integración con Semantic Kernel abriría el mercado enterprise .NET, que actualmente no tiene acceso a VantaDB.

**DSPy (Stanford):** Framework de AI programming con RAG. Una integración como retriever daría visibilidad académica al proyecto.

### Mapa de Integraciones vs Audiencia

```
                    ALTA
                   audiencia
                      │
         Mem0 ●    ●  CrewAI
                      │
  LlamaIndex ●      ● AutoGen
                      │
   LangChain ● ────── ┼ ──────── ALTA
                      │       fricción de
              Haystack●    DSPy ● integración
                      │
         LangGraph ●  │
                      │
                    BAJA
                   audiencia
```

VantaDB debe apuntar primero a las integraciones que están en el cuadrante superior-izquierdo (alta audiencia, baja fricción): LangChain, LlamaIndex, y Mem0 son los tres targets prioritarios post-lanzamiento.

---

## PLAN DE MONETIZACIÓN DETALLADO

### Principio de Monetización

El modelo Open Core sólo funciona si la separación entre Core (gratuito) y Pro (de pago) es clara, justa, y predecible. Las características Pro deben ser genuinamente enterprise (compliance, seguridad, escala), no restricciones artificiales del producto gratuito.

### Qué pertenece a VantaDB Core (OSS — Apache 2.0)

Permanece gratuito para siempre, sin restricciones de uso o escala:

- Motor HNSW completo con todas las métricas (coseno, L2, producto interno)
- BM25 + Hybrid Search con RRF
- WAL con CRC32C y auto-healing
- Fjall backend y todos los backends de storage
- Python SDK (`vantadb-py`) completo
- MCP server (`vantadb-mcp`)
- LangChain y LlamaIndex adapters
- CLI (`vanta-cli`) completo
- Toda la documentación y ejemplos

### Qué pertenece a VantaDB Pro (Licencia Comercial)

Características que son genuinamente enterprise y que no limitan el uso individual:

**Replicación P2P asíncrona (WAL Shipping):** Sincronización entre instancias sin servidor central. Caso de uso: enjambres de agentes distribuidos que comparten memoria, sincronización offline-first. Esta característica no tiene sentido para usuarios individuales pero sí para empresas con múltiples agentes.

**Cuantización SQ8/PQ (con decompresión acelerada):** La versión Core incluye cuantización básica (RaBitQ, PolarQuant). La versión Pro incluye cuantización escalar de 8 bits con decompresión SIMD optimizada para reducir RAM corporativa en >60% y re-ranking de alta fidelidad. Caso de uso: datasets de > 1M vectores en producción.

**Cifrado AES-256-GCM en reposo:** Cifrado transparente de archivos mmap para compliance (HIPAA, GDPR, SOC2). Los archivos de VantaDB son legibles sin herramientas adicionales en el Core; en Pro están cifrados con KMS configurable. Caso de uso: entornos regulados (salud, finanzas, defensa).

**Soporte prioritario y SLAs:** Respuesta garantizada en < 4 horas en horario laboral, acceso a roadmap privado, reuniones técnicas mensuales.

**Multi-tenancy con aislamiento real:** Aislamiento de datos por tenant con namespace-level encryption y quotas de recursos por tenant. Para SaaS que construyen sobre VantaDB.

### Modelo de Precios (Año 1)

No implementar pricing complejo antes de tener usuarios. El pricing inicial debe ser simple y ajustable.

**Vanta Pro Individual — $49/mes:**
Cuantización SQ8/PQ + cifrado en reposo. Para developers trabajando en proyectos comerciales que requieren compliance básico.

**Vanta Pro Team — $299/mes (hasta 10 desarrolladores):**
Todo lo anterior + WAL Shipping (replicación P2P) + soporte con SLA de 24 horas.

**Vanta Pro Enterprise — Precio personalizado ($ desde 1,000/mes):**
Todo lo anterior + multi-tenancy + SLA 4 horas + acceso a roadmap privado + NDA técnico. Para empresas reguladas con múltiples equipos.

**VantaDB Cloud (Beta — primeros 60 días gratis, luego $29/mes):**
VantaDB hosteado en Fly.io. Sin gestión de infraestructura. El tier gratuito incluye 1 instancia con 100K vectores máximo para evaluación.

### Proyección de Revenue

Estas proyecciones son conservadoras y asumen tracción de lanzamiento moderada (no el escenario optimista del manifiesto interno).

| Periodo | Stars GitHub | PyPI downloads/semana | Pro usuarios | Cloud usuarios | MRR |
|---|---|---|---|---|---|
| Mes 1 (post-launch) | 500 | 200 | 2 | 5 | $148 |
| Mes 3 | 1,000 | 500 | 8 | 15 | $777 |
| Mes 6 | 2,000 | 1,500 | 20 | 40 | $2,498 |
| Mes 9 | 3,500 | 3,000 | 45 | 80 | $5,552 |
| Mes 12 | 5,000 | 5,000 | 80 | 150 | $8,921 |

El punto de inflexión crítico para fundraising es demostrar $5K–$10K MRR, que según esta proyección ocurre entre los meses 9 y 12 post-lanzamiento.

---

## PLAN DE EXPANSIÓN DEL EQUIPO

### Estado Actual del Equipo

El repositorio muestra 2–3 contributors activos. Este es el bus factor más alto del proyecto: si el desarrollador principal no puede trabajar durante 2 semanas, el proyecto se detiene. Esta dependencia de una persona es el riesgo organizacional más crítico después de los problemas técnicos.

### Cuándo Contratar el Primer Colaborador

**No antes de:** Tener $2K MRR sostenido por 3 meses, o levantar una ronda de financiación (lo que ocurra primero). Contratar antes de tener ingresos o financiación consume recursos que no existen.

**El primer rol ideal no es un desarrollador más.** El primer colaborador debe ser alguien que multiplique el impacto del fundador técnico sin duplicar sus capacidades. El perfil correcto es uno de estos dos:

**Opción A — Developer Relations / Technical Writer:** Esta persona escribe tutoriales, responde preguntas en Discord, crea demos, y es el puente entre el proyecto y la comunidad. Impacto inmediato en tracción de comunidad. No requiere conocimiento de Rust. Costo: $2K–$4K/mes (puede ser contractor part-time).

**Opción B — Co-founder técnico con background en databases:** Alguien que haya trabajado en PostgreSQL, SQLite, RocksDB, o sistemas similares. Esta persona resuelve las partes más complejas técnicamente (Fase 2, Fase 3) y reduce el bus factor. Más difícil de encontrar pero mayor impacto en velocidad de producto. Esta sería la contratación post-pre-seed si se levanta financiación.

### Roles para Semilla ($8M–$15M valuation round)

Con capital de seed, el equipo debería expandirse a:

**Cargo 1 — Rust/Systems Engineer (foco en performance):** Resuelve los problemas de scalability (T1.1–T1.4) y el hardening arquitectónico (Fase 2) más rápido. Debe tener experiencia con bases de datos embebidas o sistemas de bajo nivel.

**Cargo 2 — Python SDK / Integration Engineer:** Se hace cargo de todos los adapters (LangChain, LlamaIndex, CrewAI, AutoGen), el Python SDK, y los bindings. La mayoría de los usuarios de VantaDB son Python developers.

**Cargo 3 — DevRel / Community Manager:** Gestiona Discord, Twitter, artículos técnicos, conferencias, y el programa de pilotos. Esta persona convierte la tracción técnica en comunidad sostenida.

---

## TARGETING DE INVERSORES

### Criterios de Selección

Para una ronda pre-seed de herramientas de developer infrastructure en Rust, los fondos correctos son los que han invertido en: OSS dev tools, Rust ecosystem, AI infrastructure, o bases de datos embebidas. Los fondos generalistas son los incorrectos en esta etapa.

### Pre-seed Targets (para ronda de $250K–$500K)

**GitHub Accelerator (no dilutivo, $20K grant):** El camino más fácil y rápido. Requiere repositorio activo con comunidad. Aplicar después de llegar a 200+ stars. No requiere deck; requiere evidencia de comunidad.

**Criterio de aplicación:** 200+ stars, 50+ forks o contributors, case studies publicados.

**Rust Foundation Grants:** Para proyectos de Rust OSS con impacto en el ecosistema. VantaDB califica si el componente WAL hardening y el motor HNSW se documentan como contribuciones al ecosistema Rust de databases.

**Criterio:** Aplicar después del lanzamiento con el artículo técnico sobre el WAL como evidencia de contribución al ecosistema.

**Angels del espacio de developer tools:** Buscar en Twitter/X personas con > 5K seguidores que hayan publicado sobre Rust, bases de datos embebidas, o AI agents locales. Una introducción directa con un demo técnico es más efectiva que un cold email con un deck.

**Targets específicos de micro-VCs:**
- **Tiny VC** (Canada): Invierte en developer tools OSS, checks de $100K–$300K
- **Garage Capital** (Europa): Foco en infraestructura técnica, fundadores técnicos
- **Precursor Ventures** (US): Pre-seed, fundadores técnicos, infraestructura

### Seed Targets (para ronda de $1.5M–$3M)

Estos fondos sólo son relevantes después de tener las métricas del Mes 9 ($5K MRR, 3,000+ stars, 3+ case studies).

- **a16z OSS Fund (ROSS):** Invierte en OSS infrastructure con modelo Open Core. El partner correcto es el que cubre databases y developer tools.
- **Amplify Partners:** Ha invertido en Render y Railway. Perfil de VantaDB encaja.
- **Initialized Capital:** Pre-seed y seed en developer tools. Ex-YC partners.

### Narrativa para Inversores

La narrativa correcta NO es "base de datos vectorial". Esa categoría está saturada y los inversores ya tienen posiciones en Qdrant, Chroma, Weaviate, LanceDB. La narrativa correcta es:

**"VantaDB es la memoria persistente de los AI agents locales. El 80% de las aplicaciones de AI que se están construyendo hoy no necesitan la nube: necesitan un motor local que sea tan confiable como SQLite pero tan inteligente como un sistema de retrieval moderno. VantaDB es eso."**

Los números que los inversores necesitarán ver, ordenados por impacto en la decisión:

1. Python SDK latencia p50 (si es > 50ms, la conversación se complica)
2. Número de pilotos activos y sus testimonials
3. GitHub stars / downloads PyPI (señal de tracción)
4. Benchmark vs LanceDB / Chroma (diferenciación técnica)
5. MRR y growth rate

---

## DASHBOARD DE MÉTRICAS Y KPIs

Este dashboard debe revisarse semanalmente por el equipo. Los semáforos (🟢🟡🔴) indican el estado respecto al objetivo.

### KPIs Técnicos (revisión semanal)

| Métrica | Baseline Actual | Objetivo Fase 1 | Objetivo Lanzamiento | Cómo medir |
|---|---|---|---|---|
| Python SDK p50 búsqueda vectorial | ~200ms | < 50ms | < 20ms | `benchmarks/vantadb_local_bench.py` |
| SIFT 10K High-Recall completion time | 127.88s | < 30s | < 15s | `cargo bench --bench sift_benchmark` |
| Recall@10 a 50K vectores | 1.0000 | >= 0.9980 | >= 0.9980 | `heavy_certification.yml` |
| Chaos test pass rate (1,000 iter) | N/A (no existe aún) | >= 99% | 100% | `dev-tools/chaos_loop.sh` |
| CI build time (fast gate) | ~12.51s compile | < 15s | < 15s | GitHub Actions duration |
| Test coverage (happy paths) | 97/97 | 97/97 + chaos | 97/97 + chaos + edge | `cargo nextest run` |

### KPIs de Producto (revisión semanal post-lanzamiento)

| Métrica | Semana 0 | Semana 4 | Semana 8 | Semana 16 | Cómo medir |
|---|---|---|---|---|---|
| GitHub Stars | 1 (actual) | 300 | 600 | 1,000 | GitHub |
| PyPI downloads/semana | N/A | 100 | 300 | 1,000 | PyPI Stats |
| GitHub Forks | 0 (actual) | 10 | 25 | 50 | GitHub |
| Contributors externos | 0 (actual) | 2 | 5 | 10 | GitHub |
| Discord miembros | 0 | 50 | 150 | 300 | Discord |
| Issues abiertos sin respuesta > 48h | N/A | 0 | 0 | 0 | GitHub Issues |

### KPIs Comerciales (revisión mensual)

| Métrica | Mes 1 | Mes 3 | Mes 6 | Mes 12 |
|---|---|---|---|---|
| Pro usuarios activos | 0 | 5 | 20 | 80 |
| MRR | $0 | $400 | $2,500 | $9,000 |
| Pilotos activos | 3 | 5 | 10 | 20 |
| Conversaciones con inversores | 0 | 3 | 8 | 15 |
| Case studies publicados | 0 | 1 | 3 | 6 |

### Sistema de Semáforos

Revisar semanalmente. Si cualquier KPI técnico está en 🔴 al inicio de una semana, el trabajo de esa semana se enfoca exclusivamente en resolverlo antes de avanzar features.

- 🟢 **Verde:** Dentro del 10% del objetivo
- 🟡 **Amarillo:** 10–30% por debajo del objetivo; investigar causa
- 🔴 **Rojo:** > 30% por debajo del objetivo; parar y resolver antes de continuar

---

## PLAYBOOK COMPETITIVO

### Si LanceDB lanza WAL durability + hybrid search nativo

Esta es la amenaza competitiva más probable. LanceDB tiene financiación significativa y ya tiene hybrid search en su roadmap.

**Respuesta:** VantaDB tiene ventaja en: (1) grafo nativo (LanceDB no tiene grafos), (2) embedded-first sin dependencias de Arrow obligatorio, (3) multi-process safety con flock. Reforzar estos mensajes en el README y artículos. No entrar en guerra de features con LanceDB; diferenciar en el nicho donde LanceDB no irá (agentes locales con grafos, datasets < 500K vectores).

### Si ChromaDB lanza WAL durability

Chroma tiene una base de usuarios enorme pero históricamente ha tenido problemas de durabilidad. Si resuelven esto, el argumento técnico principal de VantaDB se debilita.

**Respuesta:** La ventaja de VantaDB en ese escenario sería performance (HNSW en Rust vs Python nativo de Chroma) y hybrid search nativo con BM25. Publicar el benchmark comparativo inmediatamente y amplificar la diferencia de performance.

### Si SQLite lanza sqlite-vec mejorado con BM25 nativo

SQLite tiene una reputación de durabilidad que VantaDB no puede igualar en el corto plazo. Si sqlite-vec añade BM25 y hybrid search nativo, la propuesta "SQLite for AI Agents" se vuelve literalmente SQLite.

**Respuesta:** Este es el riesgo de positioning más profundo. Si ocurre: pivotar el mensaje hacia la ventaja de grafo (SQLite no tiene grafos), hacia Python SDK de primera clase (sqlite-vec tiene una API de bajo nivel), y hacia el ecosistema de integraciones (LangChain adapter oficial, LlamaIndex adapter oficial).

### Si un maintainer de Rust activo hace fork del proyecto

Un fork de un proyecto OSS con un equipo más grande puede ejecutar más rápido.

**Respuesta:** Agradecer el fork, colaborar activamente con ellos, y focalizar en el nicho donde el proyecto original tiene ventaja diferencial. La comunidad sigue al maintainer original en la mayoría de los casos si la comunicación es activa.

---

## ROADMAP DE SEGURIDAD

La seguridad de VantaDB tiene dos contextos completamente distintos que no deben mezclarse: seguridad del motor embebido (local-first, single-user, sin red) y seguridad del servidor (multi-user, expuesto en red, requiere auth).

### Seguridad del Motor Embebido (Prioridad alta, impacto inmediato)

Estas mejoras de seguridad aplican incluso sin servidor y son importantes para usuarios que confían datos sensibles a VantaDB:

**Cifrado en reposo (Pro tier):** AES-256-GCM aplicado a los archivos mmap del VantaFile. La llave puede provenir de un keyring del OS (Keychain en macOS, DPAPI en Windows, libsecret en Linux) o de un archivo de configuración. Implementación: usar el crate `aes-gcm` de la RustCrypto suite.

**Protección de WAL contra lectura directa:** Los archivos del WAL en texto plano son legibles con cualquier editor hex. Para entornos enterprise donde el WAL puede contener datos sensibles, el cifrado del WAL es parte del tier Pro.

**Auditoría de dependencias continua:** `cargo audit` ya está en CI. Añadir `cargo deny` para políticas de licencias y `cargo sbom` para generar el Software Bill of Materials en cada release.

### Seguridad del Servidor vantadb-server (Prioridad media, bloquea beta cloud)

Necesaria antes de la Fase 5 (VantaDB Cloud Beta):

**Autenticación Bearer token (T básico, Semanas 10–12):** Header `Authorization: Bearer <token>`. Los tokens se configuran en `VantaConfig::server.auth_tokens: Vec<String>`. Si el vector está vacío, el servidor deniega todas las conexiones remotas (sólo localhost). Sin autenticación no hay Cloud Beta.

**TLS con rustls (necesario para Cloud Beta):** `rustls` + `tokio-rustls` para HTTPS. Certificados automáticos via Let's Encrypt usando el crate `acme2`. En desarrollo: certificado auto-firmado con `rcgen`. Sin TLS, cualquier dato en tránsito es legible.

**Rate limiting (Semanas 14–16):** Usar `tower-governor` para limitar a 100 requests/minuto por IP. Protección básica contra DoS. Para usuarios autenticados, el límite puede ser configurable por token.

**Roadmap de Seguridad Avanzada (Post-seed, Semanas 24+):**

RBAC granular: permisos por namespace (lectura/escritura/administración) por token de API. mTLS inter-nodo: para cuando exista replicación P2P. Audit log inmutable: registro de todas las operaciones de escritura con firma HMAC. SOC2 Type I readiness: documentación de controles de seguridad para clientes enterprise.

---

## ARQUITECTURA VANTADB CLOUD (BETA)

El Cloud Beta no es una nueva arquitectura; es vantadb-server desplegado en Fly.io con configuración específica. La arquitectura debe ser la mínima que permita validar el modelo de negocio.

### MVP Cloud (Semanas 18–22)

```
Usuario → HTTPS → Fly.io Load Balancer → vantadb-server
                                               │
                                    Bearer Auth + Rate Limit
                                               │
                                         vantadb-core
                                               │
                                    Fly.io Persistent Volume
                                     (10GB, NVMe SSD)
```

**Limitaciones explícitas del MVP Cloud:**

- Single-instance (sin HA, sin replicación)
- 10GB de almacenamiento por instancia
- Máximo 1M vectores por instancia
- Sin multi-tenancy (una instancia = un cliente)
- Sin backup automático (responsabilidad del usuario en MVP)

Estas limitaciones deben estar documentadas claramente. Es mejor sobre-comunicar las limitaciones del beta que recibir queues de soporte por expectativas no cumplidas.

### Proceso de Provisioning (Automatizado a futuro, Manual en MVP)

En el beta, el provisioning es manual: el equipo de VantaDB crea una instancia de Fly.io por cliente, les da sus credenciales Bearer token, y les conecta al servidor. No hay self-service en el MVP.

El self-service (signup → instancia automática → billing) requiere Stripe, lógica de provisioning automatizada, y un portal web. Esto es trabajo de Semanas 24–30, no del MVP.

---

## TABLA MAESTRA DE TAREAS (RESUMEN EJECUTIVO)

Vista consolidada de todas las tareas para seguimiento rápido. Actualizar el status semanalmente.

| ID | Fase | Tarea | Status | Responsable | Semana Target | Semana Real |
|---|---|---|:---:|---|:---:|:---:|
| T0.1 | 0 | Estabilizar test suite | 🟡 En progreso | Dev principal | 2 | — |
| T0.2 | 0 | Clippy limpio | 🟡 En progreso | Dev principal | 2 | — |
| T0.3 | 0 | Coherencia versiones | ⬜ Pendiente | Dev principal | 1 | — |
| T0.4 | 0 | Documentar frontera experimental | ⬜ Pendiente | Dev principal | 2 | — |
| T1.1 | 1 | HNSW multi-layer audit+fix | ⬜ Pendiente | Dev principal | 4 | — |
| T1.2 | 1 | Distancia Euclidiana L2 | ⬜ Pendiente | Dev principal | 5 | — |
| T1.3 | 1 | Layout antilocatario mmap | ⬜ Pendiente | Dev principal | 7 | — |
| T1.4 | 1 | Python SDK boundary opt. | ⬜ Pendiente | Dev principal | 7 | — |
| T1.5 | 1 | Actualizar benchmarks | ⬜ Pendiente | Dev principal | 8 | — |
| T2.1 | 2 | Spawn_blocking audit | ⬜ Pendiente | Dev principal | 8 | — |
| T2.2 | 2 | mimalloc global allocator | ⬜ Pendiente | Dev principal | 6 | — |
| T2.3 | 2 | Planner AST/IR | ⬜ Pendiente | Dev principal | 12 | — |
| T2.4 | 2 | Versionado de formato | ⬜ Pendiente | Dev principal | 8 | — |
| T3.1 | 3 | Chaos testing expandido | ⬜ Pendiente | Dev principal | 12 | — |
| T3.2 | 3 | Benchmark competitivo | ⬜ Pendiente | Dev principal | 12 | — |
| T3.3 | 3 | cibuildwheel + Sigstore | ⬜ Pendiente | Dev principal | 11 | — |
| T3.4 | 3 | Pilotos controlados | ⬜ Pendiente | Dev principal | 10–16 | — |
| T4.1 | 4 | Demo content (GIF+video) | ⬜ Pendiente | Dev principal | 14 | — |
| T4.2 | 4 | Artículos técnicos (x3) | ⬜ Pendiente | Dev principal | 12–16 | — |
| T4.3 | 4 | HackerNews Show HN | ⬜ Pendiente | Dev principal | 15 | — |
| T4.4 | 4 | Discord + Good First Issues | ⬜ Pendiente | Dev principal | 15 | — |
| T5.1 | 5 | Deck inversores | ⬜ Pendiente | Dev principal | 20 | — |
| T5.2 | 5 | VantaDB Cloud Beta | ⬜ Pendiente | Dev principal | 22 | — |
| MP1 | — | Seguridad servidor (auth+TLS) | ⬜ Pendiente | Dev principal | 12 | — |
| MP2 | — | OpenTelemetry | ⬜ Pendiente | Dev principal | 14 | — |
| MP3 | — | Tokenizador avanzado | ⬜ Pendiente | Dev principal | 18 | — |
| MP4 | — | Phrase queries | ⬜ Pendiente | Dev principal | 20 | — |
| MP5 | — | Go SDK | ⬜ Pendiente | Dev principal | 22 | — |
| MP6 | — | ADRs completos | 🟡 En progreso | Dev principal | Ongoing | — |

**Leyenda de status:**
- 🟢 Completado con criterios de aceptación verificados
- 🟡 En progreso
- 🔴 Bloqueado (documentar el bloqueante en el campo Responsable)
- ⬜ Pendiente (no iniciado)

---

## APÉNDICE A: CHECKLIST DE LANZAMIENTO

Esta checklist debe estar 100% verde antes de publicar el Show HN. No es negociable.

### Técnico
- [ ] Python SDK p50 < 20ms para 10K vectores 128d
- [ ] SIFT 10K con L2 completa en < 15 segundos con Recall >= 0.95
- [ ] `cargo test --workspace --release` pasa al 100%
- [ ] `cargo clippy --all-targets -- -D warnings` pasa sin supresiones
- [ ] Chaos test de 1,000 iteraciones: 100% pass rate
- [ ] Wheels para Linux/macOS/Windows: instalables sin Rust toolchain
- [ ] `pip install vantadb-py` funciona en máquina limpia en las 3 plataformas

### Documentación
- [ ] README en inglés con GIF animado de demo
- [ ] Tabla de performance con números del Python SDK (no solo Rust nativo)
- [ ] Benchmark competitivo vs LanceDB y Chroma publicado
- [ ] `docs/BENCHMARKS.md` con metodología reproducible en 3 pasos
- [ ] `CONTRIBUTING.md` actualizado con proceso de PR claro
- [ ] `SECURITY.md` con proceso de reporte de vulnerabilidades
- [ ] Al menos 2 ejemplos completos en `examples/python/`

### Comunidad
- [ ] Discord creado con canales y bienvenida configurados
- [ ] 5–10 "Good First Issues" marcados en GitHub
- [ ] 3 pilotos activos con > 7 días de uso
- [ ] Al menos 1 case study publicado
- [ ] Respuestas preparadas para las 10 críticas más probables de HN

### Marketing
- [ ] Artículo técnico 1 publicado (7+ días antes del Show HN)
- [ ] Artículo técnico 2 publicado (3+ días antes del Show HN)
- [ ] GIF del demo subido a asciinema.org o similar
- [ ] 5–8 personas técnicas conocidas listas para votar el Show HN en los primeros 10 minutos

---

## APÉNDICE B: GLOSARIO DE TÉRMINOS TÉCNICOS

Para onboarding de nuevos contributors y claridad en la comunicación con inversores y pilotos.

**HNSW (Hierarchical Navigable Small World):** Algoritmo de búsqueda aproximada de vecinos más cercanos. Organiza vectores en un grafo jerárquico de múltiples capas que permite búsqueda en O(log n). El fundamento de la búsqueda vectorial en VantaDB.

**BM25 (Best Match 25):** Algoritmo de ranking lexical estándar de la industria, usado por Elasticsearch, Lucene, y Solr. Calcula la relevancia de documentos a una query de texto basándose en frecuencia de términos y longitud del documento.

**RRF (Reciprocal Rank Fusion):** Algoritmo de fusión de listas de resultados de diferentes motores de búsqueda. No requiere normalizar las puntuaciones de cada motor; usa solo la posición (rank) de cada resultado. Fórmula: `score(d) = Σ 1/(k + rank_i(d))` con k=60.

**WAL (Write-Ahead Log):** Técnica de durabilidad de bases de datos. Cada escritura se registra primero en el log secuencial antes de modificar los datos principales. Si hay un crash, el WAL permite reconstruir el estado completo.

**Fjall:** Motor LSM-tree escrito en Rust puro. Backend de storage por defecto de VantaDB. Alternativa a RocksDB sin dependencias C++.

**LSM-tree (Log-Structured Merge-tree):** Estructura de datos de almacenamiento optimizada para escrituras. Escribe secuencialmente en memoria (MemTable), hace flush a disco en SSTables inmutables, y compacta periódicamente. Usado por RocksDB, LevelDB, Cassandra.

**Embedded-first:** Filosofía de diseño donde el motor corre dentro del mismo proceso que la aplicación (como SQLite), sin servidor externo. Contraste con server-first (como PostgreSQL o Qdrant).

**Hybrid Search:** Búsqueda que combina resultados de múltiples índices (vectorial + lexical) y los fusiona en una sola lista ordenada. VantaDB usa HNSW para el componente vectorial y BM25 para el lexical, fusionados con RRF.

**UnifiedNode:** La unidad de datos fundamental de VantaDB. Un nodo contiene simultáneamente: un vector de embeddings, propiedades relacionales (key-value), y edges del grafo. Permite hacer las tres consultas (vectorial, relacional, grafo) sobre la misma unidad de datos.

**GIL (Global Interpreter Lock):** Mecanismo de Python que previene la ejecución paralela de threads Python. VantaDB libera el GIL durante operaciones de Rust pesadas con `py.allow_threads`, permitiendo paralelismo real.

**mmap (Memory-Mapped File):** Técnica que mapea un archivo de disco directamente al espacio de direcciones virtual del proceso. Los accesos al archivo se ven como accesos a memoria. VantaDB usa mmap para el índice HNSW en modo "Survival" (baja RAM).

**CRC32C (Cyclic Redundancy Check — variante Castagnoli):** Función de checksum usada en VantaDB para detectar corrupción en el WAL. Cada registro del WAL incluye un CRC32C de su payload. Si el checksum no coincide al leer, el registro está corrupto.

---

*Documento generado: Junio 2026. Versión 1.1 — ampliada con matriz de riesgos, esfuerzo estimado, ecosistema de integraciones, monetización, equipo, inversores, dashboard de KPIs, playbook competitivo, roadmap de seguridad, arquitectura cloud, tabla maestra, y apéndices. Basado en análisis de: commit `8ff77ee`, snapshot `2026-05-29`, documentación del proyecto, y benchmarks de certificación. Próxima revisión recomendada: cuando se complete Fase 1 o en 4 semanas, lo que ocurra primero.*

