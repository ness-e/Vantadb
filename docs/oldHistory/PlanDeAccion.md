### 📊 BLOQUE 1: Matriz de Evaluación Estratégica de Fases

| Fase (ID) | Dimensión Core | Propósito Técnico | Estado Real Actual | Observación Crítica | Viabilidad Técnica | Alineación con Identidad | ROI Estratégico | Veredicto / Prioridad |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **CUARENTENA-01** | Modularización / Clean Architecture | Aislar LISP (`src/eval/`, `src/parser/lisp.rs`) y Gobernanza (`src/governance/`, `src/governor.rs`) en subcrates del workspace. | **Completado** | Limpia la ruta de compilación sin perder código experimental. | Alta | Alta (embedded-first) | Alto | Completada y archivada |
| **SERV-01** | Core Engine / Desacoplamiento | Eliminar dependencias de red/HTTP y Tokio full del core. | **Completado** | Evita arrastrar frameworks de red en compilaciones nativas de Python/FFI. | Alta | Alta (in-process) | Alto | Completada y archivada |
| **PLANNER-02** | Core Engine / Query Engine | AST de consultas, planificador físico Volcano (iteradores perezosos) y CBO ligero. | **Completado** | Mitiga el Fan-Out combinatorio mediante evaluación lazy (`PhysicalOperator`). | Media | Alta | Alto | Completada y archivada |
| **CLI-01** | Consola UX / DevEx | Modernizar `vanta-cli` con `clap` v4, autocompletado multi-shell y salida tabular. | **Completado** | Cierre de la experiencia del desarrollador local. | Baja | Alta | Alto | Completada y archivada |
| **FASE-01-ENV** | Entorno / CI / FFI | Hermetizar entorno de Python y limpiar todos los warnings de Clippy en FFI. | **Pendiente** | Los warnings en bindings FFI y variables stale de Python impiden la automatización robusta. | Alta | Alta | Medio | **Prioridad 1 (Inmediata)** |
| **FASE-02-MMAP** | Rendimiento Mmap / Algoritmo | Resolver el cuello de botella de 127s en SIFT 10K mediante layout antilocatario y alineamiento métrico. | **Pendiente** | Los page faults aleatorios en HNSW Mmap destruyen el rendimiento p99. | Media | Alta | Crítico (p99 < 10ms) | **Prioridad 2 (Inmediata)** |
| **FASE-05-LOCKFREE** | Concurrencia | Implementar indexación concurrente libre de locks usando `sharded-slab`. | Pendiente | Reemplazar RwLock/Mutex para permitir inserción simultánea real multihilo. | Media | Alta | Alto | Prioridad 3 |
| **FASE-06-OPENCORE** | Estructura Workspace | Dividir físicamente el repositorio en `vantadb-core` y `vantadb-pro`. | Pendiente | Preparar la base del dual-licensing inyectando traits para cifrado y compresión. | Alta | Media (Estrategia) | Alto | Prioridad 4 |
| **FASE-07-MCP** | Ecosistema / IA SDK | Crear servidor Model Context Protocol (MCP) para que IAs autónomas consuman el motor. | Pendiente | Permite que VantaDB actúe como copiloto de memoria viva de la IA en tiempo de edición. | Alta | Alta (AI-first) | Alto | Prioridad 5 |
| **FASE-08-SQ8** | Compresión (Pro) | Implementar cuantización escalar SQ8 (compresión f32 a int8). | Pendiente | Permite que índices masivos quepan completamente en la caché de CPU (L3/RAM). | Alta | Alta | Alto (Licencias Pro) | Prioridad 6 |
| **FASE-09-REPL** | Sincronización (Pro) | Implementar replicación Local-First vía WAL Shipping descentralizado P2P. | Pendiente | Sincroniza enjambres de agentes locales de forma directa sin servidores en la nube. | Media | Alta | Alto | Prioridad 7 |
| **FASE-10-GTM** | Estabilización / GTM | Suite de caos de 48h y publicación automatizada en PyPI con Maturin. | Pendiente | Cierre de ciclo para el lanzamiento oficial de la v0.2.0. | Alta | Alta | Alto | Prioridad 8 |

---

### 📋 BLOQUE 2: Backlog Atómico de Ingeniería (Tareas y Subtareas)

| Fase (ID) | ID Tarea | Subtarea | Descripción Técnica Detallada | Propósito Específico | Dependencia Estricta | Criterio de Éxito / DoD |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **FASE-01-ENV** | ENV-01a | Hermetización del entorno Python | Crear target/audit-venv e instruir a las herramientas del workspace para priorizarlo. | Erradicar conflictos causados por instalaciones previas/stale de Python. | Ninguna | Python SDK utiliza exclusivamente target/audit-venv para testing local. |
| **FASE-01-ENV** | ENV-01b | Limpieza de Lints en FFI | Resolver todos los warnings de Clippy en `src/python.rs`, `src/sdk.rs` y el wrapper de Python. | Prevenir fallas y mantener código libre de lints bajo `--all-features`. | Ninguna | `cargo clippy --all-targets --all-features` compila con cero advertencias. |
| **FASE-02-MMAP** | MMAP-02a | Layout Antilocatario de Almacenamiento | Modificar `src/storage.rs` para agrupar nodos HNSW altamente conectados (niveles superiores y hubs) de forma contigua en memoria virtual. | Reducir page faults en Mmap durante la exploración estocástica del HNSW. | Ninguna | Una búsqueda con exploración profunda (`ef_search`) reduce en un 80% los page faults en Mmap. |
| **FASE-02-MMAP** | MMAP-02b | Normalización y Alineación Métrica | Eliminar simulaciones de distancia Euclidiana (L2) si se asume Coseno puro, y adaptar el suite de benchmarks. | Reducir el overhead de CPU provocado por transformaciones de vectores al vuelo. | Ninguna | El benchmark de recall/latencia corre sin fricción matemática de L2 a Coseno. |
| **FASE-05-LOCKFREE** | LFREE-05a | Reemplazo con Sharded-Slab | Reemplazar las estructuras RwLock y Mutex globales por `sharded-slab` en `src/index.rs`. | Habilitar concurrencia nativa sin bloqueos al insertar. | FASE-02-MMAP | Cero contención y bloqueos muertos al mutar el índice desde múltiples hilos de PyO3. |
| **FASE-06-OPENCORE** | OCORE-06a | Bifurcación del Workspace | Mover las APIs base a `vantadb-core` y estructurar interfaces de traits para inyección de features cerradas. | Establecer el esquema open-core. | FASE-05-LOCKFREE | Compilación exitosa en cajas separadas en el workspace. |
| **FASE-07-MCP** | MCP-07a | Servidor MCP Base | Implementar la especificación MCP envolviendo `vantadb-py` para exponer herramientas. | Integración directa y fluida con asistentes de IA locales. | FASE-06-OPENCORE | El servidor MCP responde a llamadas del cliente Claude/Antigravity de forma interactiva. |
| **FASE-08-SQ8** | SQ8-08a | Compresión SQ8 | Desarrollar el kernel SIMD para comprimir f32 a int8 y evaluar distancias sobre bytes. | Reducción de la huella de memoria para datasets empresariales. | FASE-06-OPENCORE | Ingesta y búsqueda HNSW exitosa sobre vectores cuantizados a int8. |
| **FASE-09-REPL** | REPL-09a | WAL shipping P2P | Desarrollar API de extracción incremental del WAL y su propagación asíncrona descentralizada. | Permitir sincronización multi-agente. | FASE-06-OPENCORE | Las bases de datos de dos agentes locales convergen deterministamente tras sincronizar logs de transacciones. |
| **FASE-10-GTM** | GTM-10a | Pruebas de estrés y publicación | Correr la suite de caos de 48h (`chaos_integrity.rs`), compilar bundles de release y subir a PyPI con Maturin. | Lanzamiento oficial del motor v0.2.0 listo para producción. | FASE-09-REPL | Maturin publica exitosamente los wheels compilados y el pipeline de CI/CD completa su flujo en verde. |

---

### ⚠️ Exclusiones, Diferimientos y Justificaciones Técnicas

| Módulo / Feature | Decisión | Justificación Técnica y Estratégica |
| :--- | :--- | :--- |
| **Replica Distribuida Multinodo Cloud** | **Exclusión** | Introducir capas de red directas en el núcleo del motor rompe la ventaja competitiva de latencia sub-milisegundo. La sincronización multi-agente se delega a WAL Shipping P2P liviano y asíncrono (Fase 9). |
| **Persistencia Síncrona redundante** | **Exclusión** | Para optimizar el rendimiento en `SyncMode::Always`, se confía en la durabilidad ACID garantizada exclusivamente por la secuencia determinista del WAL, evitando llamadas duplicadas a `fsync` en el almacenamiento de metadatos o el HNSW. |
