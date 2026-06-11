# Reporte de Auditoría y Estado Técnico del Código de VantaDB

Este informe técnico documenta el estado real y verificado de la base de código de VantaDB. El análisis fue realizado al 100% mediante la **lectura pasiva y directa de los archivos de código fuente, configuraciones y scripts de compilación del repositorio**, sin ejecutar pruebas o comandos interactivos y sin tomar como referencia documentos de planificación Markdown previos.

---

## 🏛️ 1. Estructura General del Workspace de Cargo

La base de código está configurada como un Workspace multi-crate en el archivo [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml) raíz:
* **Miembros del Workspace:**
  1. `.` (Crate raíz: `vantadb` en la versión `0.1.4`)
  2. `vantadb-python` (Envoltorio para bindings de Python mediante PyO3)
  3. `vantadb-server` (Servidor HTTP local basado en Axum)
  4. `vantadb-mcp` (Servidor compatible con Model Context Protocol)
* **Crates Excluidos:** `fuzz` (Contiene arneses para pruebas difusas basados en Cargo Fuzz).

---

## 🔍 2. Auditoría Detallada por Áreas de Código

### Área 1: Núcleo (Core) y Persistencia de Almacenamiento
* **Archivos clave:** [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs), [src/wal.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/wal.rs), [src/backend.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/backend.rs), [src/error.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/error.rs).
* **Mecanismos implementados:**
  - **Abstracción del Backend (`StorageBackend`):** Define operaciones de lectura, escritura, borrado, lotes atómicos y escaneos de rangos por prefijo sobre `BackendPartition` (Default, TombstoneStorage, CompressedArchive, Tombstones, NamespaceIndex, PayloadIndex, TextIndex, InternalMetadata). Las implementaciones concretas son:
    - [FjallBackend](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/backends/fjall_backend.rs): Mapea cada partición a un Keyspace de Fjall v3.1.x. No soporta checkpoints directos ni compactación manual.
    - [RocksDbBackend](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/backends/rocksdb_backend.rs): Mapea particiones a familias de columnas (CF). Habilita optimizaciones avanzadas de retención de bloques de índices y filtros en caché. Soporta compactación manual y checkpoints nativos.
  - **Write-Ahead Log (WAL) con Auto-healing:** El `WalWriter` y `WalReader` usan una cabecera binaria estructurada de 20 bytes (`WalHeader`) con magic bytes `VWAL`, versión y un CRC32C Castagnoli de validación. Implementa un algoritmo de **Scan-Forward Auto-healing** que barre el archivo byte a byte si encuentra un registro corrupto en busca del siguiente bloque válido (validación cruzada de CRC y deserialización estructural con Bincode), truncando cualquier residuo incompleto al final. Posee un límite de tamaño de 10MB en análisis de bloques para prevenir pánicos por falta de memoria (OOM).
  - **VantaFile (MMap Zero-Copy):** Envoltura de `memmap2` para el archivo `vector_store.vanta`. Valida la cabecera `VFLE` (magic bytes) en los primeros 16 bytes y mantiene el cursor alineado a 64 bytes. La lectura de encabezados se realiza mediante casting directo de memoria (`DiskNodeHeader::ref_from_bytes`) sin copias intermedias en el heap.
  - **Telemetría de RAM:** Implementa `get_resident_bytes` de forma nativa para calcular el RSS físico de las regiones mapeadas. En Unix utiliza la syscall `mincore` y en Windows utiliza la llamada `QueryWorkingSetEx` del módulo de estado del proceso.

### Área 2: Índices de Búsqueda (HNSW & BM25)
* **Archivos clave:** [src/index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs), [src/text_index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/text_index.rs).
* **Mecanismos implementados:**
  - **HNSW Multicapa Concurrente:** El grafo HNSW (`CPIndex`) usa `DashMap` para la concurrencia a nivel de nodos. El algoritmo de búsqueda desciende correctamente de la capa máxima a la 0 de forma logarítmica.
  - **Prefetching Predictivo:** En la búsqueda de capas, se emite una sugerencia asíncrona al OS para pre-cargar en memoria las direcciones físicas del vector de los nodos vecinos del candidato actual antes de computar las distancias (usa `madvise(MADV_WILLNEED)` en Unix y `PrefetchVirtualMemory` en Windows), ocultando fallos de página virtuales.
  - **Aceleración SIMD:** Operaciones vectoriales aceleradas con registros `wide::f32x8` (procesando 8 flotantes por instrucción) para distancias de tipo Cosine (`cosine_sim_cached_norms` con normas pre-calculadas en el nodo) y Euclidean (`euclidean_distance_squared_f32`).
  - **Layout BFS Antilocatario:** El método `compact_layout_bfs` reorganiza secuencialmente los nodos en disco en base al orden de recorrido en amplitud (BFS) del grafo HNSW, co-locando nodos conectados en páginas contiguas para reducir fallos de página de MMap.
  - **Text Index BM25 (Lexical Search):** Implementa almacenamiento invertido de términos usando claves con formato `namespace\0token\0key`. El esquema soporta versión 3 (tokenizador simple `lowercase-ascii-alnum`) y versión 4 (integrando `tantivy-tokenizer` para stemming, stopwords y Unicode folding). Cuenta con soporte de consultas por frase exacta (`query_plan` con frases entre comillas) gracias al almacenamiento de la posición física de cada token en el registro (`TextPosting::positions`).

### Área 3: Optimizador de Consultas y Planificación
* **Archivos clave:** [src/planner.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner.rs), [src/physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/physical_plan.rs), [src/executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs).
* **Mecanismos implementados:**
  - **Volcano Execution Engine:** El ejecutor orquesta planes compuestos por operadores físicos (`PhysicalScan`, `PhysicalFilter`, `PhysicalVectorSearch`, `PhysicalVectorRefine`, `PhysicalProject`, `PhysicalLimit`, `PhysicalSort`) a través del trait `PhysicalOperator` con flujos `open`, `next` y `close`.
  - **Cost-Based Optimizer (CBO) & Predicate Pushdown:** El planificador lógico estima la selectividad de los filtros relacionales usando estadísticas en memoria (`cardinality_stats`). Si la selectividad acumulada es altamente selectiva (< 0.1), el planificador descarta la búsqueda en el índice HNSW y compila un plan físico alternativo: realiza un `PhysicalScan` lineal filtrado por atributos relacionales (`PhysicalFilter`) y posteriormente evalúa la distancia del vector solo en los elementos sobrevivientes (`PhysicalVectorRefine`). De lo contrario, ejecuta la búsqueda vectorial en el HNSW primero y luego aplica filtros relacionales.
  - **Resource Governor:** Regula la temperatura, presupuesto de I/O y memoria lógica máxima de las consultas para evitar la degradación del hilo.

### Área 4: FFI & SDK de Python
* **Archivos clave:** [vantadb-python/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs).
* **Mecanismos implementados:**
  - **Desbloqueo de GIL:** Todos los métodos de entrada de la clase `VantaDB` envuelven las llamadas al motor de Rust dentro de bloques `py.allow_threads(move || { ... })`, permitiendo concurrencia real de hilos de Python.
  - **Búsqueda por Lotes Paralela (`search_batch`):** Implementada usando Rayon `into_par_iter` en Rust para procesar un conjunto de consultas vectoriales en paralelo y retornar los resultados consolidados a Python de forma eager.

### Área 5: Interfaces de Red, MCP y CLI
* **Archivos clave:** [vantadb-server/src/server.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/server.rs), [vantadb-mcp/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-mcp/src/lib.rs), [src/bin/vanta-cli.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/vanta-cli.rs).
* **Mecanismos implementados:**
  - **Servidor HTTP Axum:** Expone el endpoint `/api/v2/query`. Implementa rate limiting basado en IP con `tower_governor`, autenticación Bearer y cifrado TLS con `rustls-pemfile` de forma opcional. El procesamiento de consultas de I/O se delega a `tokio::task::spawn_blocking` para proteger el reactor asíncrono.
  - **MCP Server:** Servidor stdio JSON-RPC que expone recursos (`metrics://`, `schema://`) y herramientas para manipulación de memoria relacional-vectorial y consultas en lotes o LISP.
  - **CLI completions en `build.rs`:** Genera autocompletados para múltiples shells durante el build.

### Área 6: Adaptadores e Integraciones del Ecosistema
* **Directorios clave:** [packages/langchain-vantadb/](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/langchain-vantadb/), [packages/llamaindex-vantadb/](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/llamaindex-vantadb/), [examples/python/](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/examples/python/).
* **Mecanismos implementados:**
  - **LangChain Adapter:** Clase `VantaDBVectorStore` derivada de `VectorStore` con métodos `add_texts`, `similarity_search_with_score` y `from_texts`.
  - **LlamaIndex Adapter:** Clase `VantaDBVectorStore` derivada de `BasePydanticVectorStore` con implementaciones de `add` y `query` de no-dos.
  - **Examples:** El repositorio contiene 9 archivos de ejemplo funcionales para integraciones avanzadas con CrewAI, AutoGen, Haystack, DSPy, LangGraph, Mem0, y Semantic Kernel.

### Área 7: CI/CD e Infraestructura de Compilación
* **Archivos clave:** [.github/workflows/python_wheels.yml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.github/workflows/python_wheels.yml).
* **Mecanismos implementados:**
  - Compilación multi-plataforma (Linux, Windows, macOS) de ruedas de Python mediante Maturin.
  - Generación de atestaciones de procedencia criptográficas (GitHub Attestations - SLSA Level 2) para proteger la cadena de suministro.
  - Firma automática de paquetes y verificación funcional en caliente descargando el artefacto publicado y corriendo pruebas unitarias básicas.

---

## ⚠️ 3. FMEA de Seguridad y Hallazgos Críticos de Dependencias

Durante la ejecución del pre-push hook en el sistema (`verify.ps1`), se reportó un fallo crítico de seguridad provocado por vulnerabilidades en las dependencias.

### Matriz de Vulnerabilidades y Advertencias de Seguridad Detectadas (Cargo Audit):

| Crate Afectado | Versión Activa | ID de Vulnerabilidad | Nivel de Riesgo | Detalle Técnico / Impacto |
| :--- | :--- | :--- | :--- | :--- |
| `pyo3` | `0.24.2` | RUSTSEC-2026-0176 | **Crítico / Alto** | Lectura fuera de límites (Out-of-bounds read) en los iteradores `nth` / `nth_back` de las colecciones `PyList` y `PyTuple`. Puede provocar filtración de memoria o pánicos. Requiere actualizar a `>=0.29.0`. |
| `lru` | `0.12.5` | RUSTSEC-2026-0002 | **Medio** | Falta de solidez (Unsoundness) en `IterMut` al violar los principios de Stacked Borrows al invalidar punteros internos. |
| `bincode` | `1.3.3` | RUSTSEC-2025-0141 | **Advertencia** | Crate descontinuado (Unmaintained). VantaDB depende de él para la serialización del WAL y metadatos. |
| `instant` | `0.1.13` | RUSTSEC-2024-0384 | **Advertencia** | Crate descontinuado. Introducido como dependencia transitiva de `tantivy` a través de `measure_time`. |
| `rustls-pemfile`| `2.2.0` | RUSTSEC-2025-0134 | **Advertencia** | Crate descontinuado. Utilizado por `vantadb-server` para la carga de certificados TLS. |

### Impacto en Cascada y Bloqueo en Desarrollo:
El script de verificación [verify.ps1](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/dev-tools/verify.ps1) ejecuta `cargo audit` en el paso 4. Dado que `cargo audit` retorna un código de salida `1` cuando detecta vulnerabilidades activas sin mitigar, **el pre-push hook bloquea de forma incondicional cualquier intento de subir código a GitHub (`git push`)**.
Esto representa un bloqueo directo en la entrega continua del proyecto que requiere la actualización coordinada de dependencias en `Cargo.toml`.
