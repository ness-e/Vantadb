# VantaDB — Plan Maestro de Ingeniería y Estrategia Unificado (v0.2.0)

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
**Duración:** Semanas 1–2 | **Estado:** En progreso | **Prioridad:** Crítica (P0)  
**Objetivo de Fase:** Dejar el workspace en estado 100% compilable, sin warnings, sin tests rotos y con la frontera entre módulos estables y experimentales documentada y físicamente verificada.  
**Criterio de Aceptación de Fase:** `cargo test --workspace` pasa al 100% en modo `--release`. `cargo clippy --all-targets -- -D warnings` pasa sin supresiones. La documentación de frontera experimental está publicada en `/docs/operations/EXPERIMENTAL_FEATURES.md`.

#### T0.1 — Estabilización completa del test suite post-cuarentena
* **Objetivo:** Que todos los tests del core compilen y pasen sin depender de features gates experimentales ni de la VM en cuarentena.
* **Subtareas:**
  * **ST0.1.1:** Auditar tests que usan sintaxis LISP y reescribirlos en IQL estándar. Los tests en `tests/api/structured_api_v2.rs` que usaban `(INSERT ...)` deben usar `INSERT` estándar de IQL. Verificar que ningún test en `tests/logic/` importa de `packages/experimental-lisp` ni `packages/experimental-governance`.
    * *Criterio de Aceptación:* `grep -r "experimental_lisp\|experimental_governance" tests/` retorna vacío.
  * **ST0.1.2:** Verificar que `tests/certification/stress_protocol.rs` pasa completo en release mode y que el Recall@10 >= 0.95 se mantiene.
    * *Criterio de Aceptación:* El bloque 1 del stress protocol reporta Recall@10 >= 0.9560 en `cargo test --test stress_protocol --release`.
  * **ST0.1.3:** Confirmar que `#![cfg(debug_assertions)]` está aplicado correctamente en `tests/storage/derived_index_recovery.rs` y `tests/storage/text_index_recovery.rs` para que no fallen en modo release.
    * *Criterio de Aceptación:* `cargo test --workspace --release` pasa sin errores de compilación en esos archivos.
* **Criterio de Aceptación General T0.1:** `cargo nextest run --profile audit` reporta 97/97 tests passing, 0 failing, 0 panics.

#### T0.2 — Limpieza de Clippy y formato
* **Objetivo:** Cero warnings en el pipeline de Integración Continua (CI).
* **Subtareas:**
  * **ST0.2.1:** Ejecutar `cargo fmt --all` y commitear los cambios de formato. Activar `cargo fmt --check` como gate bloqueante en `rust_ci.yml`.
    * *Criterio de Aceptación:* `cargo fmt --check` pasa en CI en cualquier PR.
  * **ST0.2.2:** Ejecutar `cargo clippy --all-targets --all-features -- -D warnings`. Por cada warning en `src/python.rs` relacionado con acceso a objetos Python tras liberación del GIL: aplicar eager conversion (convertir `PyAny`/`PyDict` a tipos Rust nativos antes de llamar `py.allow_threads`). No suprimir con `#[allow]` sin justificación escrita en el código.
    * *Criterio de Aceptación:* `cargo clippy --all-targets --all-features -- -D warnings` pasa sin supresiones en archivos críticos (`python.rs`, `sdk.rs`, `wal.rs`, `storage.rs`).
* **Criterio de Aceptación General T0.2:** El CI de `rust_ci.yml` incluye steps de fmt check y clippy como gates bloqueantes y ambos pasan en verde.

#### T0.3 — Coherencia de versiones en el workspace
* **Objetivo:** Todos los `Cargo.toml` y `pyproject.toml` del workspace reportan la misma versión de desarrollo.
* **Subtareas:**
  * **ST0.3.1:** Auditar `Cargo.toml` raíz, `vantadb-python/Cargo.toml`, `vantadb-server/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `packages/langchain-vantadb/pyproject.toml`, `packages/llamaindex-vantadb/pyproject.toml`. Todos deben reflejar la versión `0.1.4`.
    * *Criterio de Aceptación:* `cargo test --test version_coherence` pasa.
  * **ST0.3.2:** Añadir `version_coherence` al perfil `audit` de nextest para que sea gate en CI rápido normal, no solo en heavy certification.
* **Criterio de Aceptación General T0.3:** `cargo test --test version_coherence` pasa en el perfil de CI rápido y es bloqueante en PR.

#### T0.4 — Documentar frontera experimental en README
* **Objetivo:** El README principal describe detalladamente qué componentes son estables, cuáles experimentales y cuáles diferidos.
* **Subtareas:**
  * **ST0.4.1:** Verificar que la tabla "Product Boundary" del README coincide con lo que está en cuarentena vs lo que está activo en `src/`.
  * **ST0.4.2:** Añadir enlace directo a `/docs/operations/EXPERIMENTAL_FEATURES.md` desde el README principal.
* **Criterio de Aceptación General T0.4:** Un desarrollador externo puede predecir con exactitud qué APIs están activas sin leer el código fuente.

#### T0.5 — Limpieza de datos obsoletos en el repositorio
* **Objetivo:** Eliminar archivos binarios temporales y bases de datos del historial.
* **Subtareas:**
  * **ST0.5.1:** Ejecutar `git rm -r --cached vantadb_data/` para purgar los 64 MB de base de datos trackeados por error.
  * **ST0.5.2:** Agregar `vantadb_data/` y archivos `.log`/`.bin` adicionales al `.gitignore` raíz.
* **Criterios de Aceptación:** El repositorio no contiene archivos de base de datos persistidos en el historial activo de git.

---

### FASE 1: HNSW Scalability & Performance
**Duración:** Semanas 2–8 | **Prioridad:** P0 — Bloqueante de adopción  
**Objetivo de Fase:** Resolver el gap de rendimiento entre el motor Rust nativo (~1.2ms p50) y el SDK Python (~200ms p50) a un rango competitivo de sub-20ms p50. Eliminar el bug de 127 segundos en SIFT 10K high-recall.  
**Criterio de Aceptación de Fase:** Python SDK p50 < 20ms para búsqueda vectorial a 10K vectores 128d. SIFT 10K benchmark con L2 nativo completa en < 15 segundos con Recall@10 >= 0.95.

#### T1.1 — Auditoría y corrección de HNSW multi-layer
* **Objetivo:** Corregir el algoritmo HNSW para que navegue correctamente por todas las capas, resolviendo la complejidad O(N) que provocaba el desvío no lineal.
* **Subtareas:**
  * **ST1.1.1:** Analizar `src/index.rs` y verificar si `search_layer()` es llamado solo para la capa 0 o si desciende desde `max_layer` hasta la capa 0. El algoritmo HNSW correcto requiere: (a) entrada por la capa más alta con un solo candidato, (b) búsqueda greedy en esa capa hasta convergencia, (c) descenso a la siguiente capa con los candidatos seleccionados, repetido hasta capa 0.
    * *Criterio de Aceptación:* Diagrama documentado en código de la travesía del grafo.
  * **ST1.1.2:** Si la búsqueda se realiza solo en la capa 0: implementar la navegación jerárquica multi-capa completa descrita en ST1.1.1.
    * *Criterio de Aceptación:* Test de complejidad sub-lineal: latencia a 100K nodos es menos de 20x la latencia a 10K nodos (no 32x).
  * **ST1.1.3:** Verificar que `insert()` asigne niveles de nodos siguiendo una distribución estadística coherente con `mL = 1/ln(M)`.
    * *Criterio de Aceptación:* En un grafo de 10K nodos con M=32, el número de nodos en capas superiores se aproxima a la distribución esperada (capa 1 ≈ 312, capa 2 ≈ 10).
* **Criterio de Aceptación General T1.1:** `cargo test --test hnsw_validation --release` pasa con Recall@10 >= 0.995 a 50K vectores. El factor de escalado latencia(100K)/latencia(10K) < 10x.

#### T1.2 — Soporte nativo de Distancia Euclidiana (L2)
* **Objetivo:** Habilitar el cálculo nativo de L2 acelerado por hardware para evitar la conversión al vuelo de distancia coseno a L2.
* **Subtareas:**
  * **ST1.2.1:** Añadir `DistanceMetric::Euclidean` al enum en `src/index.rs` e implementar el cálculo SIMD usando `wide::f32x8` para procesar 8 floats en un solo ciclo, con fallback escalar para hardware antiguo.
    * *Criterio de Aceptación:* `cargo test --test hnsw -- euclidean` pasa con delta < 1e-5 vs la implementación de referencia.
  * **ST1.2.2:** Exponer la métrica en la interfaz pública de Python. El constructor de `VantaDB` debe aceptar el parámetro `distance_metric` con opciones `"cosine"` y `"euclidean"`.
    * *Criterio de Aceptación:* `python -c "import vantadb_py; db = vantadb_py.VantaDB('./test', distance_metric='euclidean')"` corre sin error.
  * **ST1.2.3:** Ejecutar el benchmark SIFT 10K con L2 nativo y documentar en `docs/BENCHMARKS.md`.
    * *Criterio de Aceptación:* `benchmarks/vantadb_local_bench.py --metric euclidean --size 10000 --queries 100` completa en < 15s con Recall@10 >= 0.95 en el hardware de referencia.
* **Criterio de Aceptación General T1.2:** SIFT 10K completa en < 15 segundos con Recall@10 >= 0.95.

#### T1.3 — Layout de disco antilocatario para HNSW en MMap
* **Objetivo:** Re-ordenar la disposición física en disco para co-locar nodos topológicamente cercanos, reduciendo fallos de página.
* **Subtareas:**
  * **ST1.3.1:** Implementar una subrutina de re-layout post-construcción: ejecutar un recorrido BFS desde el entry point en la capa 0 y escribir los nodos secuencialmente en base al orden de visita.
    * *Criterio de Aceptación:* La medición de page faults en Linux (`/proc/self/pagemap`) decrece en un $\ge 30\%$ durante 100 consultas.
  * **ST1.3.2:** Integrar el re-layout en la función `sync_to_mmap()` (activado por defecto si la colección es > 10K vectores).
    * *Criterio de Aceptación:* VantaDB con layout optimizado a 50K vectores muestra p50 < 50ms en el SDK.
* **Criterio de Aceptación General T1.3:** Con el re-layout activo, la latencia de búsqueda en 50K vectores 128d está en el rango de 30ms–80ms.

#### T1.4 — Optimización del boundary Python–Rust (Batch Queries)
* **Objetivo:** Amortizar el overhead FFI de PyO3 e inyectar paralelismo real de hilos en Python.
* **Subtareas:**
  * **ST1.4.1:** Implementar el método `search_batch(queries: List[SearchRequest], top_k: int) -> List[SearchResult]` en `vantadb-python/src/lib.rs`. Debe realizar conversión eager de tipos antes de liberar el GIL y ejecutar las búsquedas en paralelo con `Rayon` dentro del bloque `py.allow_threads`.
    * *Criterio de Aceptación:* Un batch de 10 consultas tarda menos de 3x el tiempo de una consulta individual.
  * **ST1.4.2:** Validar que todos los métodos que tocan almacenamiento o índices (`put`, `delete`, `flush`, `rebuild_index`) liberan correctamente el GIL con `allow_threads`.
    * *Criterio de Aceptación:* El test `test_gil.py` reporta eficiencia de CPU Python >= 94.55% bajo concurrencia.
  * **ST1.4.3:** Crear micro-benchmark para medir la latencia de Rust puro frente a la llamada desde Python y documentar breakevens en `docs/BENCHMARKS.md`.
* **Criterio de Aceptación General T1.4:** El SDK individual de Python p50 < 20ms a 10K vectores. Batch de 10 consultas p50 < 60ms.

#### T1.5 — Actualización de Benchmarks y Documentación
* **Objetivo:** Proveer una documentación transparente y reproducible del rendimiento del motor.
* **Subtareas:**
  * **ST1.5.1:** Actualizar `docs/BENCHMARKS.md` con tres secciones diferenciadas: Rust nativo, Python SDK single-query y Python SDK batch.
  * **ST1.5.2:** Actualizar la tabla de rendimiento en el README raíz con las latencias reales del SDK de Python.
* **Criterio de Aceptación General T1.5:** Un desarrollador externo puede predecir el rendimiento que obtendrá en su sistema dentro de un rango de 2x de desviación.

---

### FASE 2: Hardening Arquitectónico
**Duración:** Semanas 5–12 | **Prioridad:** P0 para T2.1 y T2.2, P1 para el resto  
**Objetivo de Fase:** Eliminar bloqueos síncronos en Tokio, resolver la fragmentación de memoria y construir el optimizador del planificador.  
**Criterio de Aceptación de Fase:** `cargo test --test chaos_integrity --features failpoints` pasa 100% en 1,000 iteraciones. Ninguna consulta síncrona de E/S bloquea el reactor asíncrono.

#### T2.1 — Eliminar bloqueos síncronos en el runtime de Tokio
* **Objetivo:** Evitar que operaciones bloqueantes de disco degraden el event loop de Tokio.
* **Subtareas:**
  * **ST2.1.1:** Rastrear estáticamente la base de código buscando `std::fs::`, `std::io::`, `std::sync::Mutex::lock()` y `RwLock::write()` llamados desde funciones asíncronas (`async fn`).
    * *Criterio de Aceptación:* Bitácora completa de los puntos detectados.
  * **ST2.1.2:** Envolver cada punto crítico en un bloque `tokio::task::spawn_blocking(|| { ... }).await`.
    * *Criterio de Aceptación:* Un test de carga con 100 consultas concurrentes usando `wrk` no genera timeouts ni degradación de latencia de cola (p99 < 5x p50).
  * **ST2.1.3:** Implementar el semáforo `max_blocking_threads` en `VantaConfig` para controlar ráfagas concurrentes.
    * *Criterio de Aceptación:* Con `max_blocking_threads=4`, las ráfagas concurrentes de 50 peticiones se encolan de forma ordenada sin pánicos.
* **Criterio de Aceptación General T2.1:** El test de carga con 100 peticiones concurrentes por 60s sobre `vantadb-server` reporta p99 < 2 segundos y cero fallos.

#### T2.2 — Integración de Asignador de Memoria Global (mimalloc / jemalloc)
* **Objetivo:** Mitigar la fragmentación de memoria heap bajo inserciones masivas de vectores de alta dimensión.
* **Subtareas:**
  * **ST2.2.1:** Integrar `mimalloc` o `jemallocator` al Cargo raíz habilitado a través de feature flag `mimalloc-allocator` y activado por defecto en release builds.
    * *Criterio de Aceptación:* El core compila exitosamente con y sin el feature flag.
  * **ST2.2.2:** Medir RSS en un loop de inserción de 100K vectores durante 30 minutos y verificar la estabilidad.
    * *Criterio de Aceptación:* El incremento residual de RSS entre el inicio y el fin del proceso de inserción y consulta mixta es inferior al 10%.
  * **ST2.2.3:** Unificar las métricas en `src/metrics.rs` para reportar por separado: RSS físico del OS, memoria lógica estimada del HNSW y páginas residentes en mmap (vía `mincore` en Linux).
    * *Criterio de Aceptación:* `db.hardware_profile()` retorna las tres métricas por separado en un JSON estructurado.
* **Criterio de Aceptación General T2.2:** La memoria RSS es estable (< 15% drift) en 30 minutos de estrés. El filtro de admisión activa backpressure si el RSS supera el 80%.

#### T2.3 — Planner con Pipeline AST / LogicalPlan / PhysicalPlan
* **Objetivo:** Implementar la optimización y reescritura de consultas en un paso de compilación estática.
* **Subtareas:**
  * **ST2.3.1:** Definir las estructuras del AST en `src/planner/ast.rs` para soportar `Scan`, `Filter`, `VectorSearch`, `TextSearch`, `FuseRRF` y `Limit`.
    * *Criterio de Aceptación:* El AST puede representar las operaciones compuestas sin pérdida de parámetros.
  * **ST2.3.2:** Implementar `LogicalPlanner` para mapear el output del parser de IQL al AST tipado.
    * *Criterio de Aceptación:* Los tests unitarios de `tests/logic/parser.rs` pasan contra el nuevo planificador lógico.
  * **ST2.3.3:** Desarrollar la optimización **Predicate Pushdown**: reordenar operaciones para evaluar filtros relacionales/atributos selectivos antes de recorrer el HNSW.
    * *Criterio de Aceptación:* Una consulta con un filtro que retiene el 10% de los datos se ejecuta en menos del 20% del tiempo de una consulta no optimizada.
  * **ST2.3.4:** Refactorizar `src/executor.rs` para consumir el `PhysicalPlan` optimizado.
* **Criterio de Aceptación General T2.3:** Consultas híbridas altamente selectivas completan en < 50% del tiempo original. El planificador expone en logs las reglas de optimización aplicadas.

#### T2.4 — Versionado del formato de serialización binaria
* **Objetivo:** Prevenir la corrupción de datos históricos por evolución de estructuras internas en disco.
* **Subtareas:**
  * **ST2.4.1:** Incorporar un header estructurado de versión a los archivos del WAL, `neural_index.bin` y snapshots (magic bytes (4B), versión formato (u16), versión schema (u16), timestamp (u64)). Lanzar error explícito en el inicio ante incompatibilidades.
    * *Criterio de Aceptación:* Intentar abrir un archivo de versión vieja produce la excepción `VantaError::IncompatibleFormat`.
  * **ST2.4.2:** Documentar el versionado del formato (`format_v1`) y la compatibilidad en `CHANGELOG.md`.
* **Criterio de Aceptación General T2.4:** La suite de pruebas de compatibilidad `tests/schema_evolution.rs` pasa exitosamente.

---

### FASE 3: Validación de Producción y DX
**Duración:** Semanas 10–16 | **Prioridad:** P1  
**Objetivo de Fase:** Demostrar consistencia de producción bajo caos, certificar el rendimiento en benchmarks de la industria y distribuir binarios firmados.  
**Criterio de Aceptación de Fase:** Comparativa publicada vs LanceDB y Chroma en `docs/BENCHMARKS.md`. 3 clientes piloto activos con SLA p99 < 10ms. Wheels automatizadas.

#### T3.1 — Chaos testing expandido y validación de durabilidad
* **Objetivo:** Garantizar que cortes eléctricos y fallos físicos de disco no corrompan los índices.
* **Subtareas:**
  * **ST3.1.1:** Implementar en `tests/storage/chaos_integrity.rs` escenarios de fallo mediante `failpoints` (kill -9 durante append del WAL, compactación de Fjall, reconstrucción de HNSW, y corrupción de bytes aleatorios).
    * *Criterio de Aceptación:* Todos los escenarios pasan en CI. El motor se recupera en el reinicio al último estado consistente sin pánico.
  * **ST3.1.2:** Crear la utilidad de terminal `dev-tools/chaos_loop.sh` para correr 1,000 iteraciones aleatorias de fallos inyectados bajo carga concurrente.
    * *Criterio de Aceptación:* El script corre 1,000 ciclos en CI sin corrupciones de base de datos.
  * **ST3.1.3:** Publicar y documentar los resultados en `docs/operations/RELIABILITY_GATE.md`.
* **Criterio de Aceptación General T3.1:** El README enlaza la certificación de durabilidad y recuperación tras 1,000 ciclos de fallos.

#### T3.2 — Benchmark competitivo vs LanceDB y Chroma
* **Objetivo:** Proveer comparaciones de rendimiento honestas utilizando frameworks de la industria.
* **Subtareas:**
  * **ST3.2.1:** Desarrollar el conector de VantaDB para el framework `ann-benchmarks` e integrar los datasets estándar `glove-100-angular` y `sift-128-euclidean`.
    * *Criterio de Aceptación:* El conector procesa las consultas y exporta métricas sin errores de tipos.
  * **ST3.2.2:** Medir ingesta, latencias p50/p95/p99, recall, memoria en reposo y bajo carga para VantaDB, LanceDB y Chroma.
  * **ST3.2.3:** Redactar y publicar los resultados en `docs/BENCHMARKS.md`.
* **Criterio de Aceptación General T3.2:** Benchmark transparente publicado en `docs/BENCHMARKS.md` con scripts reproducibles de un solo paso.

#### T3.3 — Pipeline de wheels para distribución (cibuildwheel + Sigstore)
* **Objetivo:** Proveer empaquetado y firmas criptográficas automáticas para el SDK en múltiples plataformas.
* **Subtareas:**
  * **ST3.3.1:** Configurar `cibuildwheel` en el pipeline de GitHub Actions para compilar ruedas en `manylinux2014_x86_64`, macOS Intel/Apple Silicon y Windows x64, incluyendo estáticamente las dependencias de Fjall.
    * *Criterio de Aceptación:* `pip install` funciona en entornos sin compiladores de Rust instalados.
  * **ST3.3.2:** Configurar el Trusted Publishing de PyPI con OIDC para automatizar la publicación tras empujar tags (`v*.*.*`).
  * **ST3.3.3:** Programar el paso de CI `verify_published_wheel` para descargar, validar la firma de Sigstore e importar de forma básica en Python.
* **Criterio de Aceptación General T3.3:** El SDK nativo firmado se publica y valida automáticamente en PyPI al generar un release.

#### T3.4 — Programa de pilotos controlados
* **Objetivo:** Validar el motor en entornos reales y flujos de trabajo de producción de clientes piloto.
* **Subtareas:**
  * **ST3.4.1:** Identificar 3–5 early adopters en foros y comunidades especializadas de agentes de IA locales (ej. Reddit `/r/LocalLLaMA`, Discord de LangChain/LlamaIndex).
  * **ST3.4.2:** Desarrollar un paquete de onboarding de pilotos (Quickstart en <15 min, integración de ejemplo con Ollama y formulario de feedback).
  * **ST3.4.3:** Redactar casos de estudio prácticos en `docs/case_studies/` documentando problemas, integraciones y métricas reales.
* **Criterio de Aceptación General T3.4:** Al menos 3 pilotos completados y documentados en producción con testimonios públicos en discusiones de GitHub.

---

### FASE 4: Community Launch
**Duración:** Semanas 14–20 | **Prioridad:** P1 (Bloqueado por Fase 1 y 2)  
**Objetivo de Fase:** Impulsar la adopción orgánica del proyecto, logrando tracción en foros especializados y atrayendo colaboradores.  
**Criterio de Aceptación de Fase:** 1,000+ stars en GitHub, 20+ forks, 5+ colaboradores externos, Show HN en top 10 y 200+ miembros en Discord.

#### T4.1 — Demo content técnico (asciinema + GIF + Ejemplos)
* **Objetivo:** Mostrar la propuesta de valor en menos de 60 segundos de lectura.
* **Subtareas:**
  * **ST4.1.1:** Grabar y subir a `asciinema.org` una demostración en terminal de 90 segundos que muestre: instalación, inserción, consulta de memoria, kill del proceso y recuperación de la memoria.
  * **ST4.1.2:** Crear un GIF de 30s de alta visibilidad para el encabezado del README y de publicaciones.
  * **ST4.1.3:** Crear el directorio `examples/python/` con códigos de ejemplo comentados para agentes autónomos (`agent_memory.py`), LangChain (`langchain_rag.py`) y LlamaIndex (`llamaindex_search.py`).
* **Criterio de Aceptación General T4.1:** Ejemplos autocontenidos y GIF embebidos en el README raíz.

#### T4.2 — Artículos técnicos de arquitectura
* **Objetivo:** Construir credibilidad técnica y SEO a través de artículos de ingeniería profunda.
* **Subtareas:**
  * **ST4.2.1:** Escribir el Artículo 1: *"Why I Built a Local Memory Engine for AI Agents in Rust"* (Diseño de persistencia embebida, WAL e IQL).
  * **ST4.2.2:** Escribir el Artículo 2: *"How Hybrid Search Works: BM25 + HNSW + RRF in Practice"* (Matemática y optimización de recuperación).
  * **ST4.2.3:** Escribir el Artículo 3: *"SQLite for AI Agents: Benchmarks and Architecture Decisions"* (Análisis comparativo transparente vs LanceDB/Chroma).
* **Criterio de Aceptación General T4.2:** Los 3 artículos publicados en dev.to, Medium y blogs del proyecto.

#### T4.3 — Lanzamiento en HackerNews (Show HN)
* **Objetivo:** Lanzar el proyecto de forma abierta para capturar visibilidad técnica masiva.
* **Subtareas:**
  * **ST4.3.1:** Redactar la descripción técnica corta para el post "Show HN" sin marketing exagerado.
  * **ST4.3.2:** Preparar y documentar respuestas a las 10 críticas técnicas más probables (ej. "¿por qué no usar sqlite-vec?").
  * **ST4.3.3:** Publicar en horario de alto tráfico (martes/miércoles) y responder activamente consultas durante las primeras 6 horas.
* **Criterio de Aceptación General T4.3:** El post alcanza la página principal de HackerNews (> 50 puntos) y se logran más de 200 stars en 24h.

#### T4.4 — Gobernanza de Comunidad y Contribuciones
* **Objetivo:** Canalizar la tracción inicial de desarrolladores a contribuciones efectivas al core.
* **Subtareas:**
  * **ST4.4.1:** Crear el servidor oficial de Discord configurando canales de soporte y debates de roadmap.
  * **ST4.4.2:** Marcar 5–10 issues en GitHub con la etiqueta `good first issue` con descripciones de archivos afectados.
  * **ST4.4.3:** Responder incidencias y Pull Requests en menos de 48 horas.
* **Criterio de Aceptación General T4.4:** Discord activo con 100+ miembros, 5+ colaboradores externos con PRs fusionadas en el core.

---

### FASE 5: Preparación Pre-seed
**Duración:** Semanas 18–24+ | **Prioridad:** P2 (Bloqueado por tracción y métricas)  
**Objetivo de Fase:** Reunir los activos de negocio necesarios para levantar una ronda de inversión institucional de \$250K–\$500K a una valuación de \$2M–\$4M.  
**Criterio de Aceptación de Fase:** Deck de inversión listo, 3 case studies documentados, SDK validado, 5 conversaciones abiertas con fondos de infraestructura de desarrollo.

#### T5.1 — Presentación de Negocio y Due Diligence Técnico
* **Objetivo:** Traducir las capacidades operacionales en una tesis de inversión de infraestructura.
* **Subtareas:**
  * **ST5.1.1:** Desarrollar el Pitch Deck de 10 diapositivas (Problema, Solución, Arquitectura, Moat competitivo, TAM, Tracción de comunidad, Modelo Open Core, Equipo, Proyecciones financieras, Uso del capital).
  * **ST5.1.2:** Consolidar el repositorio privado de due diligence técnico (informes de chaos, benchmarks reproducibles, case studies y ADRs).
* **Criterio de Aceptación General T5.1:** Deck finalizado e informativo listo para envío a inversores.

#### T5.2 — Despliegue de Servidor Cloud Beta
* **Objetivo:** Validar la factibilidad del servicio administrado y recopilar telemetría multi-usuario.
* **Subtareas:**
  * **ST5.2.1:** Desplegar el contenedor `vantadb-server` en Fly.io configurando volúmenes NVMe SSD persistentes de 10GB.
  * **ST5.2.2:** Integrar autenticación básica Bearer Token configurable vía variables de entorno en el servidor expuesto.
  * **ST5.2.3:** Invitar a los usuarios piloto del programa a consumir la versión administrada por 14 días.
* **Criterio de Aceptación General T5.2:** Instancia en la nube operativa bajo HTTPS con autenticación activa y persistencia estable.

---

## 🛤️ 4. Pistas Paralelas (Mejoras No Bloqueantes)

Estas tareas corren en paralelo al desarrollo principal del core y no bloquean el flujo de desarrollo de las fases:

* **MP1 — Seguridad Avanzada del Servidor (Semanas 12):** Implementar cifrado de transporte forzado en el servidor HTTP con `rustls` y rate limiting básico (100 req/min por IP) usando `tower-governor`.
* **MP2 — OpenTelemetry y Logging Estructurado (Semanas 14):** Instrumentar los hot-paths de consultas con `tracing-opentelemetry` y `tracing-subscriber` para exportar trazas structured JSON correlacionadas con `trace_id` a Jaeger/Grafana.
* **MP3 — Tokenizador Avanzado (Semanas 18):** Integrar `tantivy-tokenizer` como dependencia opcional (feature flag `advanced-tokenizer`) para habilitar soporte multilingüe en la indexación BM25.
* **MP4 — Phrase Queries y Snippets (Semanas 20):** Implementar búsquedas de frases exactas e inyección de fragmentos de texto destacados (*highlighting*) en el módulo de búsqueda léxica.
* **MP5 — Go SDK (Semanas 22):** Generar cabeceras de FFI de C utilizando `cbindgen` y construir los bindings oficiales para Go (`cgo`).
* **MP6 — ADRs y Documentación de Arquitectura (Continuo):** Mantener de forma estricta el registro histórico de decisiones de diseño en la carpeta [`docs/adr/`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/adr/).

---

## 📊 5. Ecosistema de Integraciones y Plan de Adopción

La integración fluida con herramientas y frameworks del ecosistema es el principal multiplicador de tracción para VantaDB. Se prioriza el desarrollo de conectores basados en audiencia y dificultad de integración:

### 5.1. Priorización de Integraciones

* **Prioridad 1 (Adopción Inmediata - Completado):**
  * **LangChain (`langchain-vantadb`):** Integración mediante `VantaDBVectorStore`. Requiere añadir un tutorial completo en su repositorio de ejemplos con Ollama local y submission a la lista oficial de almacenamiento.
  * **LlamaIndex (`llamaindex-vantadb`):** Adaptador integrado. Requiere la publicación de un ejemplo de persistencia híbrida local en LlamaIndex Hub.
* **Prioridad 2 (Alta Audiencia / Baja Fricción - Semanas 16–22):**
  * **CrewAI (Dificultad: Baja - 4 días):** Implementar `VantaDBMemory` como el provider de almacenamiento para agentes autónomos que requieran recordar datos entre ejecuciones de tareas.
  * **Mem0 (Dificultad: Baja - 3 días):** Integrar VantaDB como el motor de persistencia relacional-semántico nativo para su almacenamiento.
  * **AutoGen (Dificultad: Media - 5 días):** Crear el adapter de memoria persistente para los agentes conversacionales de Microsoft.
  * **Haystack (Dificultad: Media - 6 días):** Implementar `VantaDBDocumentStore` adaptado a la arquitectura de pipelines de deepset.
* **Prioridad 3 (Alta Audiencia / Alta Fricción - Semanas 22–30):**
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

| Métrica | Baseline (v0.1.4) | Objetivo Fase 1 | Objetivo Lanzamiento | Método de Medición |
| :--- | :---: | :---: | :---: | :--- |
| **Latencia p50 Búsqueda Vectorial** | ~200ms | < 50ms | < 20ms | `vantadb_local_bench.py` |
| **Tiempo de Ingesta SIFT 10K** | 127.88s | < 30s | < 15s | `cargo bench --bench sift_benchmark` |
| **Recall@10 a 50K vectores** | 1.0000 (L0) | >= 0.9980 | >= 0.9980 | `stress_protocol.rs` |
| **Pass Rate en Test de Caos** | N/A | >= 99% | 100% (1k ciclos) | `dev-tools/chaos_loop.sh` |
| **Tiempo de Compilación en CI** | ~12.51s (compile) | < 15s | < 15s | Duración de GitHub Action |
| **Cobertura de Tests (Happy Paths)** | 97/97 tests | 97 + chaos tests | 97 + chaos + edge | `cargo nextest run` |

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
* **`src/eval/` (LISP VM):** Extirpar la máquina virtual experimental de evaluación de expresiones.
* **`src/parser/lisp.rs`:** Purgar el parser dinámico obsoleto.
* **`src/api/mcp.rs`:** Eliminar la lógica del protocolo integrada en el core.
* **`src/governance/` (Consistency Buffer / Conflict Resolver):** Remover abstracciones prematuras de replicación distribuida complejas.
* **`vanta_certification.json`:** Remover del raíz.

### 🛠️ Refactorizar (Reestructuración Estructural)
* **[`src/storage.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs) (Esfuerzo: 4 HH):** Dividir en módulos limpios: `wal.rs` (con soporte CRC32), `vanta_file.rs` (layout binario secuencial) y `backend_manager.rs`.
* **[`src/planner.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner.rs) (Esfuerzo: 5 HH):** Extraer la lógica de optimización a `ast.rs`, `logical.rs`, `physical.rs` y `optimizer.rs`.
* **[`src/sdk.rs`](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs) (Esfuerzo: 3 HH):** Trasladar la lógica de importación y exportación a su propio módulo `src/export.rs`.
* **Directorio de Pruebas `tests/` (Esfuerzo: 2 HH):** Organizar físicamente en `tests/unit/`, `tests/integration/`, `tests/bench/` y `tests/certification/`.

---

## 🚫 11. Sección de Descarte e Inviabilidad Técnica

Se descartan las siguientes propuestas de los planes de origen debido a riesgos graves de seguridad de memoria o penalizaciones inaceptables en latencia:

1. **Intérprete LISP en Runtime para Políticas y Gobernanza:** Causaba allocations masivas en heap, fragmentación de tipos y pánicos del compilador en el *Borrow Checker* (`Rc<RefCell>` al mutar vistas de grafos en `MmapMut`). Se reemplaza por optimización estática del planificador lógico de IQL a nivel de AST en tiempo de compilación interna.
2. **Cuantización de 2 Bits (TurboQuant) y Olvido Temporal de Ebbinghaus:** Comprimir a 2 bits reduce el Recall por debajo del 40%, inhabilitando la recuperación semántica. El borrado o decaimiento de nodos por tiempo destruye la topología del grafo HNSW creando subgrafos huérfanos. Se sustituye por cuantización regular a 8 bits (SQ8) con aceleración SIMD.
3. **Persistencia Síncrona en Cada Mutación de Nodos:** Ejecutar duplicados de `fsync` tanto en los metadatos como en el índice aproximado y el WAL bloquea los hilos de CPU en esperas de E/S de disco. Toda la durabilidad transaccional se relega al flujo secuencial del WAL de alta velocidad, reconstruyendo en caliente el HNSW en memoria.

---

## 🛡️ 12. Matriz de Riesgos Críticos y Fallos de Seguridad (FMEA)

| Identificador | Escenario de Fallo Técnico | Severidad | Probabilidad | Estrategia de Mitigación Implementada |
| :--- | :--- | :---: | :---: | :--- |
| **FMEA-01** | **Corrupción de Índice HNSW por Fallo de Alimentación:** Pérdida de integridad de los enlaces y metadatos en el archivo binario del grafo. | Alta (9) | Media (4) | Integración de CRC32/MurmurHash3 por bloque de registro en el WAL y checkpoints atómicos validados por `fsync` antes de la rotación. |
| **FMEA-02** | **Deadlocks por Bloqueo Concurrente:** Contención severa y bloqueos cruzados al acceder concurrentemente al índice HNSW (`RwLock`). | Alta (8) | Media-Alta (6) | Implementar punteros de intercambio atómicos y técnicas de reclamación de memoria libres de bloqueo (wait-free) en el Core de Rust. |
| **FMEA-03** | **Fuga de Memoria Virtual en Mmap (Disk Thrashing):** Page faults masivos en Windows y saturación de la memoria RAM al explorar grafos muy dispersos. | Alta (8) | Media (5) | Aplicación de layout binario antilocatario para nodos de alta conectividad y directivas `madvise` para precarga secuencial. |
| **FMEA-04** | **Bloqueo del GIL de Python por Ingesta Masiva:** Ingestas masivas bloquean la ejecución del hilo principal del agente de IA consumidor. | Media-Alta (7) | Alta (8) | Inyección forzada de macro `py.allow_threads` en todos los puntos de entrada del SDK nativo para derivar la computación al backend de Rust. |
| **FMEA-05** | **Compromiso de Secretos en Entornos de Integración:** Fuga involuntaria de tokens de acceso a repositorios en los logs públicos de compilación. | Media (5) | Baja (2) | Configuración del pipeline de CI con escaneo pre-commit `gitleaks` y autenticación de credenciales vía OIDC. |
| **FMEA-06** | **Derecho al Olvido (GDPR) Inviable:** Incapacidad de certificar la eliminación física de datos personales en el grafo HNSW de compactación lenta. | Alta (8) | Media-Alta (6) | Implementación de re-layout de vecindad de grafos asíncrono y purga física de páginas mmap conteniendo tombstones. |

---

## 🔬 13. Plan de Verificación y Criterios de Aceptación Cuantitativos

La validación funcional del Plan Maestro se regirá bajo los siguientes criterios estadísticos estrictos e innegociables:

### 1. Validación de Recall en Búsqueda Vectorial
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test hnsw_recall --release -- --nocapture
  ```
* **Criterio de Aceptación:** El Recall@10 en el dataset estándar SIFT10K debe mantenerse en $\ge 0.95$ tras aplicar filtros optimizados por Predicate Pushdown.

### 2. Pruebas de Caos y Recuperación ante Fallos
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test chaos_integrity --release -- --nocapture
  ```
* **Criterio de Aceptación:** El motor debe superar 1,000 iteraciones continuas de caídas simuladas por failpoints a mitad del flujo de escritura del WAL sin reportar pérdida o corrupción de datos consistentes en el reinicio.

### 3. Línea Base de Telemetría de Memoria RSS
* **Comando de Ejecución Sugerido (Ejecución Manual por el Usuario):**
  ```powershell
  cargo test --test memory_telemetry --release -- --nocapture
  ```
* **Criterio de Aceptación:** La medición instrumental del uso de memoria virtual no debe registrar un crecimiento lineal incremental tras 1,000 operaciones de flushing e invalidación de cachés en background (cero fugas de memoria).
