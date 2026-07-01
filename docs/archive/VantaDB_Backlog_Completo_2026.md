# VantaDB — Backlog Completo y Priorizado
> **Versión:** 2026-06-13 | **Basado en:** Análisis competitivo + auditoría MPTS + estado real del código

---

## Filosofía de Priorización

Cada tarea pasa por tres filtros antes de entrar al roadmap:

1. **¿Lo necesita el ICP real?** (developer de agentes de IA, RAG pipelines, local-first)
2. **¿Refuerza "el SQLite para Agentes de IA"?** (embedded, zero-config, durable, simple)
3. **¿Qué retorno da vs el esfuerzo en un equipo de 1-3 personas?**

**Criterio de lanzamiento en Show HN (septiembre 2026):**
> Un developer puede hacer `pip install vantadb-py`, tener un agente con memoria persistente, búsqueda híbrida y GraphRAG en <10 líneas de código, con latencia <20ms en Python, en menos de 2 minutos.

---

## Leyenda

| Símbolo | Significado |
|---------|-------------|
| 🔴 CRÍTICO | Bloquea el lanzamiento o destruye credibilidad |
| 🟠 ALTO | Necesario para el lanzamiento exitoso |
| 🟡 MEDIO | Mejora significativa, puede ir en patch post-launch |
| 🟢 BAJO | Deseable, no urgente |
| ✅ | Ya existe en backlog, reafirmado |
| 🆕 | Nuevo, no estaba en ningún documento |

---

---

# FASE 3 — COMPLETAR ANTES DEL LANZAMIENTO
## Julio – Agosto 2026

**Objetivo:** Ningún bug crítico, ninguna brecha de credibilidad técnica visible.

---

### 3.A — Bloqueantes Críticos (no se puede lanzar sin estos)

---

**TSK-56** ✅ 🔴 CRÍTICO — Fix Windows CI runner
El pipeline CI usa el runner `windows-2025-vs2026` que no existe. Sin este fix, no se generan wheels para Windows en ningún release. Afecta a ~35% del mercado de developers.
- Cambiar runner a `windows-latest` o `windows-2022`
- Validar build completo en Windows x86_64
- Confirmar que PyPI wheels se generan correctamente

---

**DISC-05** ✅ 🔴 CRÍTICO — Fix telemetría de memoria (~225 GB en máquina de 34 GB)
El sistema reporta uso de RAM imposible. Cualquier developer que active métricas en producción verá esto y perderá confianza inmediatamente. Causa: confusión entre RSS y espacio de direcciones de mmap.
- Separar métricas: `rss_bytes` (RAM real), `mmap_address_space_bytes` (espacio virtual, no RAM)
- Añadir nota en docs explicando la diferencia RSS vs address space
- Test que valide que `rss_bytes` < RAM física del sistema en benchmarks

---

**TSK-52** ✅ 🔴 CRÍTICO — SIGTERM shutdown handler
El servidor no maneja `SIGTERM` correctamente. En cualquier entorno con orchestración (Docker, systemd, Kubernetes) el proceso puede recibir SIGTERM y no cerrar el WAL + Fjall de forma limpia, dejando la base de datos en estado inconsistente.
- Registrar handler para `SIGTERM` y `SIGINT`
- En handler: flush WAL, sync Fjall, cerrar file lock, exit(0)
- Test: enviar SIGTERM durante write, verificar que la DB se recupera correctamente

---

**DISC-04** ✅ 🔴 CRÍTICO — Chaos testing completo con kill -9 durante writes
El claim de "zero pérdida de datos en crashes" no está verificado empíricamente en CI. Es el corazón de la propuesta de valor de durabilidad.
- Suite de chaos tests: kill -9 en posiciones aleatorias del flujo de escritura
- Verificar recovery completo desde WAL en cada escenario
- Añadir al workflow semanal `heavy_certification.yml`
- Publicar resultados en README como evidencia pública

---

### 3.B — Performance Python SDK (la brecha más dañina)

---

**TSK-68** 🆕 🔴 CRÍTICO — Zero-copy FFI: devolver NumPy arrays en lugar de listas Python
El overhead de 62ms vs objetivo de <20ms está causado principalmente (~40ms) por la copia de datos al cruzar la frontera FFI: crear objetos Python, convertir `Vec<f32>` a listas, serializar metadata a dicts. No es el GIL (ya liberado).
- Devolver resultados de búsqueda vectorial como `numpy.ndarray` (via PyO3 memoryview)
- Devolver vectores en `get()` como `memoryview` en lugar de `list[float]`
- Reducir cruces FFI en search_memory: un solo cruce, no uno por campo
- Benchmark antes/después: objetivo <20ms p50 en Python
- **Esto es THE task más importante antes del Show HN**

---

**TSK-73** 🆕 🟠 ALTO — Async Python API (asyncio)
Frameworks modernos como FastAPI, LangChain async, LlamaIndex async, y la mayoría del ecosistema de agentes usan `async/await`. Sin API async, los developers tienen que usar `run_in_executor()` manualmente, lo que es friction innecesaria.
- `async def search_memory_async(...)` wrapeando la versión sync en threadpool
- `async def put_async(...)`
- Compatible con `asyncio`, `trio` y `anyio`
- Ejemplo en quickstart con `async def` para FastAPI + VantaDB

---

**TSK-74** 🆕 🟠 ALTO — Python type stubs (.pyi files)
Sin stubs, mypy y pyright no pueden tipar el módulo, y los IDEs (VSCode, PyCharm) no dan autocomplete. Es una diferencia de DX muy visible para developers de Python profesional.
- Generar `.pyi` para toda la API pública de `vantadb_py`
- Incluir en el wheel de PyPI
- Validar con `mypy --strict` en CI
- Validar con `pyright` en CI

---

**TSK-69** 🆕 🟠 ALTO — put_batch() implementación verificada y expandida
`put_batch()` está documentado en las especificaciones pero necesita verificar que la implementación usa paralelismo Rayon correctamente y que es realmente más rápido que N llamadas individuales.
- Verificar implementación actual de `put_batch()` con paralelismo Rayon
- Si no existe, implementar: Rayon par_iter sobre el batch, luego WAL flush único
- Benchmark: 1000 inserts individuales vs put_batch(1000). Objetivo: >5x speedup
- Añadir a Python SDK con mismo nombre

---

### 3.C — Core Engine

---

**TSK-46** ✅ 🟠 ALTO — MMap-backed HNSW para datasets grandes
Permite que el índice HNSW ocupe más espacio del que cabe en RAM usando mmap. Sin esto, la restricción práctica es ~500K vectores en una máquina de 16GB. Con mmap, el límite es el disco.
- Mapear el grafo HNSW directamente en disco vía mmap
- Paginación lazy: cargar nodos bajo demanda, no todo el grafo en RAM
- Prefetch con `madvise(MADV_SEQUENTIAL)` durante búsqueda
- Test: 1M vectores sin OOM en máquina de 8GB RAM

---

**TSK-47** ✅ 🟡 MEDIO → Re-priorizar a ALTO — Cuantización escalar SQ8 (int8)
Reduce memoria 4x con pérdida de recall <1%. Directamente relevante cuando el ICP tiene >100K vectores. SQ8 es la cuantización correcta para empezar: no rompe el índice HNSW existente.
- Convertir vectores de `f32` a `i8` con factor de escala por dimensión al momento del `put()`
- Implementar distancia coseno con SIMD int8 (`wide::i8x16`)
- Flag opt-in: `db.put(key, vector, quantize="sq8")`
- Benchmark recall@10 antes/después: objetivo pérdida <1%
- **NO implementar todavía:** PQ, IVF-PQ, cuantización 2-bit. Solo SQ8.

---

**TSK-49** ✅ 🟡 MEDIO — Zero-copy deserialization con rkyv
Optimiza la lectura de nodos desde storage. Actualmente deserializar un `UnifiedNode` implica copia de memoria. Con rkyv, el objeto se accede directamente del buffer mmap sin copia.
- Derivar `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize` en `UnifiedNode`
- Reemplazar la deserialización actual en los hot paths de `get()` y `search()`
- Benchmark: latencia de `get()` antes/después

---

**TSK-50** ✅ 🟡 MEDIO — Filtro de admisión (backpressure al 80% RAM)
Cuando el RSS del proceso supera el 80% de la RAM física configurada, las escrituras deben rechazarse con error `OutOfMemory` en lugar de causar OOM kill del proceso. Necesario para producción estable.
- Monitorear RSS real cada N writes (no en cada write, caro)
- Si RSS > umbral: rechazar puts con `VantaError::MemoryPressure`
- Configurable: `memory.backpressure_threshold = 0.8`
- Notificar via log y métricas cuando se active backpressure

---

**TSK-53** ✅ 🟠 ALTO — Validación estricta de metadata en FFI
Valores problemáticos de Python (NaN, Inf, dicts anidados, bytes, objetos arbitrarios) que cruzan la frontera FFI sin validación pueden causar panics en Rust o corrupción silenciosa.
- Rechazar en el boundary PyO3: `NaN`, `Inf`, `-Inf` en floats
- Rechazar metadata con anidamiento profundo (>3 niveles)
- Rechazar metadata con tipos no soportados (bytes, objetos Python, etc.)
- Retornar `VantaError::InvalidMetadata` con mensaje descriptivo

---

**TSK-75** 🆕 🟡 MEDIO — WAL compaction / vacuum
El WAL crece indefinidamente. Sin compactación, en uso intensivo el archivo WAL puede crecer GBs y ralentizar el recovery en startup.
- Comando CLI: `vanta-cli vacuum --db ./data`
- Internamente: una vez aplicados todos los registros WAL al backend Fjall, truncar el WAL
- Trigger automático: cuando WAL > N MB (configurable, default 256 MB)
- Test: verificar que vacuum no pierde datos con chaos test posterior

---

**TSK-76** 🆕 🟠 ALTO — TTL (Time-To-Live) en registros
Los agentes de IA necesitan memoria que expire. Una conversación de contexto temporal no debería vivir para siempre. Sin TTL, los developers tienen que implementar limpieza manual.
Además, el campo `last_accessed` ya existe en `UnifiedNode` — TTL es la extensión natural.
- Campo opcional en `put()`: `ttl_seconds=3600` (expira en 1 hora)
- Background task que borra registros expirados (o lazy evaluation en `search()`)
- `list_memory()` excluye registros expirados por defecto
- Configurable por namespace: `namespace_config.default_ttl_seconds`
- `vanta-cli vacuum` también limpia registros TTL expirados

---

**TSK-76b** 🆕 🟡 MEDIO — Memory eviction por importancia (importance-based)
Los campos `importance: f32` y `hits: u32` ya están en `UnifiedNode`. Falta la política de eviction: cuando la DB supera un límite de registros, eliminar automáticamente los de menor importancia y menos accedidos.
- Configurable: `namespace_config.max_records = 10000`
- Eviction policy: weighted score de `importance * (hits / max_hits)` — el menor sale primero
- Trigger: en cada `put()` cuando `count() > max_records`, evict el de menor score
- Log cuando se activa eviction

---

### 3.D — Testing y Calidad

---

**TSK-36** ✅ 🟠 ALTO — Auditoría estructural del text index (BM25)
El BM25 index es una ventaja competitiva central pero no hay garantía de que sea correcto bajo concurrencia, ni benchmarks de calidad documentados.
- Revisar thread-safety del text index bajo reads/writes concurrentes
- Crear test de exactitud: corpus controlado con queries y resultados esperados
- Benchmark NDCG@10 y MRR@10 del BM25 solo, vs híbrido, vs vectorial solo

---

**TSK-55** ✅ 🟠 ALTO — Datasets de prueba reales en CI (GloVe, NQ)
Las métricas actuales son con dataset SIFT1M (128d). El ICP usa embeddings de 768-3072 dimensiones (sentence-transformers, OpenAI). Sin tests con dimensiones reales, los benchmarks publicados no son representativos.
- Descargar GloVe-100 (1.2M vectores, 100d) y NQ (Natural Questions embeddings, 768d)
- Añadir benchmark de recall@10 con estas dimensiones al workflow semanal
- Publicar resultados en `docs/benchmarks.md`

---

**TSK-54** ✅ 🟡 MEDIO — Job CI de benchmark para detectar regresiones
Sin detección automática de regresiones de performance, un cambio de código puede degradar latencia sin que nadie lo note hasta el siguiente benchmark manual.
- Guardar últimos N resultados de benchmark en un artifact de CI
- Si p50 latencia aumenta >10% vs baseline: marcar CI como failed
- Notificar en PR si la regresión afecta métricas publicadas en README

---

**TSK-78** 🆕 🟡 MEDIO — Expansión de property-based testing con proptest
Los tests actuales de proptest tienen ~30% cobertura. Las combinaciones más peligrosas (vectores de dimensión 0, keys con caracteres Unicode especiales, metadata con valores límite) no están cubiertas.
- Test de invariantes: insert + get siempre retorna el mismo record
- Test de invariantes: insert + delete + get siempre retorna None
- Test de invariantes: WAL recovery produce el mismo estado que antes del crash
- Test boundary: key de exactamente 1 KB (límite), key de 1 KB + 1 byte (debe fallar)

---

**TSK-13** ✅ 🟠 ALTO — Unit tests para vantadb-mcp
El servidor MCP no tiene tests propios. Un bug silencioso en los handlers MCP puede hacer que Cursor o Claude Code fallen de formas inexplicables para el usuario.
- Tests unitarios para `handle_prompts_list`, `handle_prompts_get`
- Tests para los 4 prompts configurados
- Mock del engine para tests rápidos sin I/O real

---

**TSK-14, 15, 16, 17** ✅ 🟠 ALTO — Test suite vantadb-server (auth, rate limiting, TLS, concurrencia)
El servidor HTTP tiene features de seguridad (Bearer token, rate limiting, TLS) sin cobertura de tests. Cualquier bug aquí es un riesgo de seguridad en producción.
- Test auth: request sin token → 401, token válido → 200, token inválido → 401
- Test rate limiting: burst de N requests → algunos 429
- Test TLS: conexión sin TLS → rechazada si configurado como obligatorio
- Test concurrencia: 100 requests simultáneos, verificar no race conditions

---

**TSK-79** 🆕 🟡 MEDIO — Alerts de regresión de benchmark como gate de CI
Complementa TSK-54. Si una métrica crítica regresa más del umbral, el PR no puede mergearse.
- `recall@10` en SIFT1M ≥ 0.95 (actualmente 0.998 — no bajar)
- Python SDK p50 ≤ 20ms (objetivo post-TSK-68)
- Ingesta PUT ≥ 90 ops/sec

---

### 3.E — Observabilidad

---

**TSK-93** 🆕 🟡 MEDIO — Integración Prometheus completa
Las métricas están definidas en la struct `Metrics` pero la integración con Prometheus client no está completa. Sin esto, los enterprise pilots no pueden monitorear VantaDB con su infraestructura existente.
- Exponer endpoint `/metrics` en el servidor HTTP si está activo
- Métricas: `vantadb_search_latency_ms`, `vantadb_put_throughput`, `vantadb_wal_size_bytes`, `vantadb_record_count`, `vantadb_memory_rss_bytes`
- Histogramas con percentiles p50, p95, p99 para latencias
- Ejemplo de configuración Prometheus + Grafana en `docs/observability.md`

---

**TSK-94** 🆕 🟡 MEDIO — Logging estructurado (JSON, log levels)
Actualmente los logs no tienen formato estructurado. En producción, los developers necesitan logs que puedan parsear con herramientas como loki/ELK.
- Usar `tracing` crate con formato JSON configurable
- Levels: ERROR, WARN, INFO, DEBUG
- Eventos importantes: DB open, WAL replay, index rebuild, backpressure activation, eviction
- Configurable: `VANTADB_LOG_LEVEL=info VANTADB_LOG_FORMAT=json`

---

### 3.F — Documentación Esencial Pre-Lanzamiento

---

**TSK-67** 🆕 🟠 ALTO — Documentar y ejemplificar graph traversal con benchmark reproducible
El GraphRAG es la ventaja más única de VantaDB y la peor documentada. El claim "40-60% reducción de tokens" no tiene un ejemplo reproducible público.
- Ejemplo completo: crear grafo de entidades, búsqueda vectorial → expansión de vecinos → inyectar contexto en LLM
- Benchmark reproducible: misma query con RAG tradicional vs GraphRAG, medir tokens en prompt
- Código ejecutable disponible en `examples/graphrag_demo.py`
- Publicar como blog post técnico

---

**TSK-70** 🆕 🟠 ALTO — Documento de garantías de durabilidad
Los developers que consideren VantaDB para producción necesitan entender exactamente qué garantiza la base de datos y en qué escenarios. Este documento reemplaza al "SLA" para una librería embebida.
- Tabla de garantías: "VantaDB garantiza X en escenario Y"
- Ej: "Zero pérdida de datos en kill -9 durante write (validado en 1000 iteraciones de chaos test)"
- Ej: "Recovery automático <100ms tras crash"
- Ej: "CRC32C detecta y rechaza registros WAL corruptos"
- Ej: "File locking previene corrupción por apertura simultánea"
- Publicar en `docs/durability-guarantees.md`

---

**TSK-80** 🆕 🟠 ALTO — Migration guide desde ChromaDB y LanceDB
El path de adopción más común es "venía usando X y migré a VantaDB". Sin guía de migración, el friction de cambio es alto.
- Guía ChromaDB → VantaDB: equivalencia de API, diferencias de modelo de datos, qué mejorar y qué perder
- Guía LanceDB → VantaDB: cuándo tiene sentido migrar, cuándo no (LanceDB sigue siendo mejor para datasets >RAM multimodal)
- Scripts de migración de datos si es posible
- Publicar en `docs/migration/`

---

**TSK-81** 🆕 🟡 MEDIO — README badges y señales de calidad
Los badges en el README comunican estado del proyecto instantáneamente. Un README sin badges en 2026 parece un proyecto abandonado o amateur.
- `[![CI](badge)](actions)` — estado del CI
- `[![PyPI](badge)](pypi)` — versión actual en PyPI
- `[![Downloads](badge)](pypi)` — downloads mensuales (pepy.tech)
- `[![License](badge)](license)` — Apache 2.0
- `[![Rust](badge)](rustc)` — versión mínima de Rust
- `[![Crates.io](badge)](crates)` — versión en crates.io (cuando esté publicado)

---

**TSK-82** 🆕 🟡 MEDIO — CHANGELOG.md formal y estructurado
Developers que evalúan una librería siempre revisan el CHANGELOG para entender la cadencia de releases y qué tan activo está el proyecto. Sin changelog, el proyecto parece dormido.
- Formato: [keepachangelog.com](https://keepachangelog.com)
- Categorías: Added, Changed, Fixed, Removed, Security, Performance
- Una entrada por cada release desde v0.1.0
- Generar automáticamente desde git log con herramienta como `git-cliff`

---

**TSK-83** 🆕 🟡 MEDIO — Issue templates y PR template en GitHub
Sin templates, los bug reports son incompletos y los PRs no tienen contexto. Esto escala mal cuando hay comunidad.
- Bug report template: version, OS, código mínimo reproductor, comportamiento esperado vs actual
- Feature request template: caso de uso, alternativas consideradas
- PR template: qué cambia, cómo se testeó, checklist (tests, docs, CHANGELOG)

---

---

# FASE 4 — LANZAMIENTO COMUNITARIO
## Septiembre – Octubre 2026

**Objetivo:** Lanzamiento Show HN exitoso. Ecosistema funcional. Comunidad en movimiento.

---

### 4.A — TypeScript SDK (nueva plataforma, alto impacto)

---

**TSK-61** 🆕 🔴 CRÍTICO — TypeScript SDK vía WASM
La mitad del ecosistema de agentes de IA corre en Node.js/Bun/Deno (LangChain.js, LlamaIndex.TS, Vercel AI SDK). Sin TS SDK, VantaDB no existe para ese mundo.
- Compilar `vantadb-core` a `wasm32-wasi` (ya está en backlog como ROAD-01, re-priorizar)
- Wrapper TypeScript sobre WASM con API idiomática
- API mínima: `new VantaDB(path)`, `.put()`, `.search()`, `.searchHybrid()`, `.delete()`, `.close()`
- Publicar en npm como `vantadb`
- Compatibilidad: Node.js 18+, Bun 1.0+, Deno 1.40+
- Nota: filesystem en browser usa OPFS (Origin Private File System)

---

**TSK-62** 🆕 🟠 ALTO — TypeScript types y documentación
- Types estrictos con JSDoc para intellisense en VSCode
- README separado para el SDK de TypeScript
- Quickstart: Node.js + TypeScript + embeddings de OpenAI
- Publicar tipos en DefinitelyTyped si es necesario

---

**TSK-84** 🆕 🟠 ALTO — Ejemplos TypeScript con LangChain.js y LlamaIndex.TS
- `examples/ts/langchain-agent-memory.ts` — Agente LangChain con memoria VantaDB
- `examples/ts/llamaindex-rag.ts` — Pipeline RAG con LlamaIndex.TS
- `examples/ts/vercel-ai-sdk.ts` — Integración con Vercel AI SDK

---

### 4.B — Nuevas Operaciones de API

---

**TSK-85** 🆕 🟡 MEDIO — delete_by_filter()
Eliminar registros por condición de metadata, no solo por key. Necesario para mantenimiento masivo de la DB (ej: "elimina todas las memorias del usuario X").
```python
db.delete_by_filter(namespace="chat", filter={"user_id": "user_123"})
# Retorna: {"deleted": 47}
```

---

**TSK-86** 🆕 🟡 MEDIO — similar_to_key() — buscar similares a un registro existente
En vez de proveer un vector query, buscar registros similares a uno que ya existe en la DB. Muy útil para recomendaciones y deduplicación.
```python
# "dame documentos similares a este que ya está guardado"
results = db.similar_to_key(namespace="docs", key="policy_001", top_k=10)
```

---

**TSK-87** 🆕 🟡 MEDIO — count() con filtros
Operación básica que falta: contar registros con condiciones. Necesaria para dashboards, límites de capacidad, debugging.
```python
total = db.count(namespace="chat")
user_memories = db.count(namespace="chat", filter={"user_id": "user_123"})
```

---

**TSK-88** 🆕 🟡 MEDIO — Multi-namespace search
Buscar en varios namespaces simultáneamente y fusionar resultados. Útil para un agente que tiene memoria de conversación + memoria de conocimiento + memoria de preferencias.
```python
results = db.search_multi_namespace(
    namespaces=["conversation", "knowledge", "preferences"],
    vector=query_embedding,
    top_k=10
)
```

---

**TSK-60** 🆕 🟡 MEDIO — Filtros de metadata expandidos (operadores rich)
Los filtros `$gte`, `$in` están documentados pero necesitan cobertura completa y tests. Añadir operadores que faltan.
- `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte` para números y strings
- `$in`, `$nin` para listas
- `$exists` para verificar si un campo existe
- `$and`, `$or`, `$not` para composición lógica
- Tests exhaustivos para cada operador

---

### 4.C — Ecosistema: Integraciones Tier 1 (obligatorias para lanzamiento)

---

**INT-01** ✅ 🔴 CRÍTICO — LangChain adapter (`langchain-vantadb`)
Ya en desarrollo como FEAT-01. Confirmar estas sub-tareas:
- Implementar `VantaDBVectorStore` con la interfaz estándar de LangChain
- `similarity_search()`, `similarity_search_with_score()`, `add_texts()`, `delete()`
- Publicar en PyPI como `langchain-vantadb`
- Submittir PR a `langchain-community` para inclusión oficial
- Ejemplo en `examples/python/langchain_rag.py`

---

**INT-02** ✅ 🔴 CRÍTICO — LlamaIndex adapter (`llama-index-vector-stores-vantadb`)
Ya en desarrollo como FEAT-01. Confirmar estas sub-tareas:
- Implementar `VantaDBVectorStore` con la interfaz de LlamaIndex
- `query()`, `add()`, `delete()`
- Publicar en PyPI como `llama-index-vector-stores-vantadb`
- Submittir PR a `llama-index-integration` para inclusión oficial
- Ejemplo en `examples/python/llamaindex_rag.py`

---

**INT-03** ✅ 🔴 CRÍTICO — MCP server estable
Ya implementado de forma experimental. Estabilizar para lanzamiento:
- Tests unitarios completos (TSK-13 ya en backlog)
- Documentación de configuración para Cursor, Claude Code, Windsurf
- Ejemplo de uso: "agente con memoria de proyecto persistente"
- Publicar como binary standalone descargable

---

### 4.D — Ecosistema: Integraciones Tier 2

---

**TSK-89** 🆕 🟡 MEDIO — CrewAI adapter
CrewAI es uno de los frameworks de multi-agentes más populares. Hay un caso de uso natural: crews con memoria persistente compartida.
- Implementar `VantaDBMemory` para CrewAI
- Ejemplo: crew de investigación que comparte y persiste conocimiento
- Publicar en PyPI como `crewai-vantadb`

---

**TSK-90** 🆕 🟠 ALTO — Mem0 integration
Mem0 es uno de los frameworks de memoria de agentes con más tracción en 2026 (mencionado en benchmarks de agent memory). Posicionar VantaDB como backend alternativo de Mem0.
- Implementar VantaDB como `VectorStoreBackend` en Mem0
- Esto hace que VantaDB sea accesible a toda la base de usuarios de Mem0
- PR al repositorio de Mem0 con la integración
- Blog post: "VantaDB como backend local-first para Mem0"

---

**TSK-91** 🆕 🟡 MEDIO — DSPy integration
DSPy es el framework de programación de LLMs de Stanford. Tiene un concepto de "retrieval modules" que puede usar VantaDB como backend.
- Implementar `VantaDBRM` (Retrieval Module) para DSPy
- Ejemplo: pipeline DSPy con RAG usando VantaDB
- Publicar como ejemplo en `examples/python/dspy_rag.py`

---

**TSK-92** 🆕 🟡 MEDIO — Haystack adapter
Deepset Haystack es popular en enterprise RAG, especialmente en Europa. Una integración amplía el alcance geográfico y de caso de uso.
- Implementar `VantaDBDocumentStore` para Haystack
- Publicar en PyPI como `haystack-integrations-vantadb`

---

**TSK-65** 🆕 🟡 MEDIO — vantadb-openai (paquete de embedding opcional)
```bash
pip install vantadb-openai
```
No bundlea el modelo — solo llama a la API de OpenAI automáticamente antes del `put()`.
```python
from vantadb_openai import OpenAIEmbedder
db = VantaDB("./data", embedder=OpenAIEmbedder(model="text-embedding-3-small"))
db.put(namespace="docs", key="doc1", text="VantaDB es...")
# Genera embedding automáticamente, sin que el usuario lo pida
```

---

**TSK-66** 🆕 🟡 MEDIO — vantadb-ollama (embedding local, completamente offline)
```bash
pip install vantadb-ollama
```
Usa Ollama local para generar embeddings. Caso de uso: privacidad total, sin APIs externas.
```python
from vantadb_ollama import OllamaEmbedder
db = VantaDB("./data", embedder=OllamaEmbedder(model="nomic-embed-text"))
```

---

**TSK-95** 🆕 🟢 BAJO — vantadb-litellm (gateway universal de embeddings)
LiteLLM permite usar cualquier modelo de embeddings (OpenAI, Cohere, Mistral, local) con una sola interfaz. Un adapter aquí cubre cientos de providers.
```bash
pip install vantadb-litellm
```

---

### 4.E — CLI Completeness

---

**TSK-25** ✅ 🟠 ALTO — CLI: comando `search` (búsqueda semántica desde terminal)
```bash
vanta-cli search --db ./data --namespace docs \
  --query "¿Cómo proteger datos sensibles?" --top-k 10
```
Indispensable para debugging y exploración de datos desde la terminal.

---

**TSK-26** ✅ 🟠 ALTO — CLI: comando `delete`
```bash
vanta-cli delete --db ./data --namespace docs --key policy_001
vanta-cli delete --db ./data --namespace docs --filter '{"department": "legal"}'
```

---

**TSK-27** ✅ 🟡 MEDIO — CLI: gestión de namespaces
```bash
vanta-cli namespace list --db ./data
vanta-cli namespace create --db ./data --name new_namespace
vanta-cli namespace delete --db ./data --name old_namespace
vanta-cli namespace stats --db ./data --name docs
```

---

**TSK-63** 🆕 🔴 CRÍTICO — CLI: comando `backup`
Cualquier developer que ponga VantaDB en producción lo primero que preguntará es "¿cómo hago backup?". Sin este comando, la respuesta es "copia el directorio manualmente".
```bash
vanta-cli backup --db ./data --output ./data_backup_2026-09-01.vantadb
# Hace flush del WAL, snapshot atómico, verifica CRC32C del backup
```

---

**TSK-64** 🆕 🟠 ALTO — CLI: comando `restore`
```bash
vanta-cli restore --from ./data_backup.vantadb --to ./data_restored
# Verifica integridad, reconstruye índices
```

---

**TSK-96** 🆕 🟡 MEDIO — CLI: `vanta-cli doctor` (diagnóstico de salud)
Herramienta de diagnóstico que verifica el estado de la DB y detecta problemas antes de que sean críticos.
```bash
vanta-cli doctor --db ./data
# Output:
# ✅ WAL: sin corrupción (1,247 registros)
# ✅ HNSW: 45,230 vectores indexados
# ✅ BM25: 45,230 documentos indexados
# ⚠️ Memory: 87% de límite configurado (OOM en ~2,000 inserts)
# ✅ File lock: no hay otro proceso abierto
# ✅ CRC32C: todos los registros WAL válidos
```

---

**TSK-97** 🆕 🟡 MEDIO — CLI: `vanta-cli inspect --key KEY` (inspector de registros)
```bash
vanta-cli inspect --db ./data --namespace docs --key policy_001
# Output formateado: todos los campos de UnifiedNode para ese key
```

---

**TSK-98** 🆕 🟡 MEDIO — CLI: `vanta-cli stats` (estadísticas de la DB)
```bash
vanta-cli stats --db ./data
# Output:
# Total records: 45,230
# Namespaces: docs (32,100), conversation (8,430), preferences (4,700)
# WAL size: 128 MB | HNSW size: 58 MB | Fjall size: 412 MB | Total: 598 MB
# Memory RSS: 234 MB / 4096 MB (5.7%)
# Avg vector dimensions: 768
```

---

**TSK-99** 🆕 🟢 BAJO — CLI: `vanta-cli repl` (modo interactivo)
REPL para explorar la DB interactivamente. Útil para debugging y demos.
```
vantadb> put docs/key1 "texto del documento"
vantadb> search docs "¿qué dice sobre seguridad?" --top-k 5
vantadb> get docs/key1
vantadb> stats
```

---

**TSK-29** ✅ 🟠 ALTO — Tests unitarios para todos los comandos CLI
Sin tests, el CLI rompe silenciosamente con cada refactor del core.

---

**TSK-30** ✅ 🟡 MEDIO — Unificar binarios CLI + MCP + Server
Actualmente `vanta-cli`, `vantadb-server` y `vantadb-mcp` son binarios separados. Consolidar en un solo binary con subcomandos:
```bash
vantadb serve --http --port 8080
vantadb serve --mcp --port 3000
vantadb backup --db ./data --output ./backup
```

---

### 4.F — Developer Experience (DX)

---

**TSK-104** 🆕 🟠 ALTO — Demo agent app con memoria persistente (showcase)
Un ejemplo completo, funcional y reproducible que demuestre el valor de VantaDB en 5 minutos. Este será el link que acompaña al Show HN post.
- Agente de asistente personal con memoria entre sesiones
- Usa LangChain + VantaDB + Ollama (completamente local, sin APIs externas)
- El agente recuerda preferencias, conversaciones previas, y puede responder "¿qué hablamos ayer?"
- Repo separado: `github.com/ness-e/vantadb-demo-agent`
- Video walkthrough de 3 minutos

---

**TSK-103** 🆕 🟠 ALTO — Public benchmark site (reproducible)
Ningún número en el README es creíble sin código reproducible. Los developers en HN siempre preguntan "¿puedo reproducir esto?".
- Script reproducible: `benchmarks/compare.py --competitors chroma,lancedb,qdrant`
- Medir: recall@10, p50/p95 latencia de search, throughput de ingesta
- Publicar resultados en `benchmarks/results/` con fecha y hardware
- Sitio estático en `vantadb.dev/benchmarks` con tabla actualizada

---

**TSK-35** ✅ 🟡 MEDIO — Suite de ejemplos en Rust
Los developers Rust no tienen ejemplos de uso native. Crítico para adopción en el ecosistema Rust.
- `examples/rust/basic.rs` — CRUD básico
- `examples/rust/hybrid_search.rs` — Búsqueda híbrida
- `examples/rust/graphrag.rs` — GraphRAG completo
- `examples/rust/concurrent.rs` — Uso con múltiples threads

---

**TSK-34** ✅ 🟡 MEDIO — Reorganización física de documentación por audiencia
La documentación actual está organizada por dominio técnico, no por tipo de usuario. Un developer que llega por primera vez no sabe por dónde empezar.
```
docs/
├── getting-started/     ← El developer llega aquí primero
│   ├── quickstart.md
│   ├── installation.md
│   └── first-agent.md
├── guides/              ← Casos de uso concretos
│   ├── rag-pipeline.md
│   ├── graphrag.md
│   └── agent-memory.md
├── api-reference/       ← Referencia técnica
│   ├── python.md
│   └── rust.md
└── architecture/        ← Para contributors
```

---

### 4.G — Distribución y Packaging

---

**TSK-45** ✅ 🔴 CRÍTICO — Publicar en crates.io
Los developers Rust que quieran usar VantaDB como librería native no tienen acceso hasta que esté en crates.io. Esto bloquea la adopción completa en el ecosistema Rust.
- Preparar metadata del crate (description, keywords, categories, license)
- Verificar que el API pública está bien documentada con `///` docs
- Publicar `vantadb` en crates.io
- Publicar documentación en docs.rs

---

**TSK-101** 🆕 🟠 ALTO — ARM64 Linux wheels (fix experimental → estable)
Las Raspberry Pi 5, AWS Graviton, y servidores ARM son cada vez más comunes. El wheel ARM64 Linux está marcado como "experimental".
- Añadir runner ARM64 nativo en CI (via QEMU cross-compilation o runner dedicado)
- Test completo del wheel en plataforma ARM64 real
- Publicar como stable en PyPI

---

**TSK-102** 🆕 🟡 MEDIO — Python 3.13+ support
Python 3.13 tiene cambios en el ABI y el GIL (experimental no-GIL mode). Asegurar compatibilidad.
- Añadir Python 3.13 a la matrix de CI
- Verificar que `py.allow_threads()` sigue siendo correcto con no-GIL experimental
- Publicar wheels para Python 3.13

---

**TSK-100** 🆕 🟡 MEDIO — Homebrew formula para macOS
Muchos developers macOS usan Homebrew para instalar herramientas CLI. Una formula facilita la instalación del server binary sin necesidad de Rust instalado.
```bash
brew install vantadb
vanta-cli --help
```
- Crear formula en `homebrew-vantadb` tap
- Publicar en Homebrew core si alcanza popularidad suficiente

---

**TSK-57** ✅ 🟢 BAJO — Verificación hash del wheel antes de pip install en tests
Añade una capa de seguridad supply chain: verificar SHA256 del wheel descargado antes de instalarlo en el test suite.

---

### 4.H — Comunidad (infraestructura para crecer)

---

**COM-01** 🆕 🔴 CRÍTICO — Discord server con canales estructurados
Sin lugar de encuentro, no hay comunidad. El Show HN post debe linkear al Discord el día del lanzamiento.
- Canales: `#announcements`, `#general`, `#help`, `#showcase`, `#development`, `#roadmap`
- Bots básicos: welcome bot, comando `/docs`, antispam
- Pinned: quickstart, links a PyPI y crates.io, link al GitHub

---

**COM-02** 🆕 🔴 CRÍTICO — CONTRIBUTING.md claro y detallado
Sin guía de contribución, los primeros interesados en contribuir no saben cómo empezar y se van.
- Cómo configurar el entorno de desarrollo local
- Cómo correr los tests
- Convenciones de commits (conventional commits)
- Proceso de review de PRs
- Áreas donde se necesita ayuda

---

**COM-03** 🆕 🔴 CRÍTICO — Code of Conduct
Cualquier comunidad open-source seria necesita CoC. Sin él, el proyecto parece informal.
- Adoptar Contributor Covenant (estándar de la industria)
- Definir proceso de reporte y enforcement

---

**TSK-97** 🆕 🟠 ALTO — Good first issues (mínimo 20)
La métrica de "contributors" en 6 meses depende de que haya issues etiquetados `good first issue` que sean accesibles sin conocer el código en profundidad.
- Identificar 20+ tareas de documentación, tests, o utilidades que no requieran conocer el core
- Cada issue debe tener: descripción clara, contexto, archivos relevantes, criterio de aceptación
- Categorizar: `good-first-issue`, `help-wanted`, `documentation`, `testing`

---

**TSK-106** 🆕 🟡 MEDIO — GitHub Discussions habilitado
Issues son para bugs y features. Discussions es para preguntas, ideas, y compartir proyectos. Separar los dos mejora la señal/ruido.
- Habilitar GitHub Discussions
- Categorías: Q&A, Ideas, Show & Tell, General
- Responder activamente en los primeros 3 meses (tiempo de respuesta <24h)

---

**TSK-107** 🆕 🟡 MEDIO — Community showcase (proyectos de usuarios)
Una galería de proyectos construidos con VantaDB aumenta la credibilidad y el descubrimiento.
- Página `docs/showcase.md` con proyectos de la comunidad
- Template para que usuarios submitan su proyecto
- Premio simbólico para los primeros 10 proyectos (swag, mención en redes)

---

**TSK-108** 🆕 🟢 BAJO — Newsletter (Substack o Beehiiv)
Una newsletter mensual mantiene a la comunidad informada y crea un canal directo con los usuarios más comprometidos.
- Frecuencia: mensual
- Contenido: releases, blog posts, proyectos de la comunidad, roadmap update
- Objetivo: 500 suscriptores en 6 meses

---

### 4.I — Marketing y Lanzamiento

---

**MKT-01** 🆕 🔴 CRÍTICO — Landing page (vantadb.dev)
El primer punto de contacto para alguien que escucha "VantaDB" en HN o Twitter. Si no existe, el proyecto no parece serio.
- Hero: tagline, demo de código en 5 líneas, botón "Get Started"
- Sección: problema (fragmentación, latencia, lock-in) → solución (VantaDB)
- Sección: benchmarks con gráficas
- Sección: comparación con competidores
- Sección: casos de uso (RAG, agent memory, knowledge base)
- Sección: quickstart en tabs (Python, Rust, TypeScript)
- Footer: GitHub, PyPI, Discord, Docs, Blog

---

**MKT-02** 🆕 🔴 CRÍTICO — Blog post de lanzamiento: "Introducing VantaDB"
El post que acompaña al Show HN. Debe ser técnicamente profundo, honesto sobre limitaciones, y con benchmarks reproducibles.
- Historia del proyecto: por qué se construyó
- Arquitectura: WAL, HNSW, BM25, RRF, grafos — explicado para un developer
- Benchmarks: latencia, recall, comparación vs ChromaDB y LanceDB
- Demo de GraphRAG con reducción de tokens medida
- Roadmap honesto: qué está listo y qué no

---

**MKT-03** 🆕 🔴 CRÍTICO — Show HN post (preparación)
El timing y la redacción del post de Hacker News son críticos.
- Timing: martes o miércoles, 10am PST (máxima audiencia)
- Título: "Show HN: VantaDB – Embedded vector+graph database for AI agents (Rust)"
- Primeros comentarios: anticipar preguntas comunes (vs LanceDB, vs ChromaDB, performance Python)
- Tener respuestas preparadas para: "¿por qué no usas X?", "¿los benchmarks son reales?", "¿cuándo habrá TS SDK?"

---

**MKT-04** 🆕 🟠 ALTO — Posts en Reddit (r/rust, r/MachineLearning, r/LocalLLaMA)
- `r/rust`: post técnico sobre la arquitectura WAL + HNSW en Rust
- `r/MachineLearning`: post sobre GraphRAG y reducción de tokens
- `r/LocalLLaMA`: post sobre memoria local para agentes (privacy-first)
- Timing: mismo día del Show HN o día siguiente

---

**MKT-05** 🆕 🟠 ALTO — Blog posts técnicos (mínimo 5 pre-lanzamiento)
El SEO tarda en funcionar. Empezar a publicar 6-8 semanas antes del lanzamiento.
1. "Cómo implementamos HNSW en Rust: lecciones aprendidas"
2. "GraphRAG explicado: reduciendo tokens en LLMs un 40-60%"
3. "Por qué fsync importa: durabilidad en bases de datos embebidas"
4. "VantaDB vs ChromaDB vs LanceDB: comparación honesta (2026)"
5. "Construye un agente con memoria persistente en 20 líneas de Python"

---

**MKT-06** 🆕 🟡 MEDIO — Logo y branding básico
Sin logo, el README, landing page y Discord se ven sin terminar.
- Logo simple, vectorial (SVG), que funcione en modo claro y oscuro
- Paleta de colores consistente
- Favicon para vantadb.dev

---

### 4.J — Seguridad Básica

---

**TSK-106b** 🆕 🟠 ALTO — SECURITY.md y proceso de reporte de vulnerabilidades
Empresas que evalúan VantaDB para uso enterprise preguntan "¿cómo reportamos una vulnerabilidad?". Sin SECURITY.md, la respuesta es "abre un GitHub issue público" — inaceptable.
- Crear `SECURITY.md` con: email de reporte, PGP key si aplica, tiempos de respuesta
- Política: disclosure responsable (90 días)
- Proceso: recibir → confirmar → parchear → publicar CVE

---

**TSK-109** 🆕 🟡 MEDIO — Hardening de inputs: path traversal, vectors malformados, DoS
Un developer malicioso (o simplemente descuidado) puede enviar inputs que crashen el proceso.
- Prevención de path traversal en `db_path`: rechazar paths con `../`, symlinks peligrosos
- Validar dimensiones del vector en el boundary de la API: 0 → error, >32768 → error
- Límite de tamaño de batch en server HTTP: max 10MB por request
- Rate limiting configurable en el servidor HTTP

---

---

# FASE 5 — POST-LANZAMIENTO / PRE-SEED
## Noviembre – Diciembre 2026

**Objetivo:** Enterprise readiness básico. Primeros pilotos. Preparación para pre-seed.

---

### 5.A — Enterprise Readiness

---

**TSK-72** 🆕 🟡 MEDIO — Encriptación at-rest (opcional)
Requerimiento de compliance HIPAA y GDPR para algunos enterprise pilots. Sin esto, VantaDB no puede ser usado en sectores regulados.
- Encriptación AES-256-GCM de los archivos de datos (Fjall + WAL)
- La clave la proporciona el usuario: `VantaDB::open_encrypted("./data", key: &[u8; 32])`
- No almacenar la clave en ningún archivo — responsabilidad del usuario
- Documentar: "VantaDB encripta los datos, no gestiona claves"

---

**TSK-107b** 🆕 🟡 MEDIO — Audit logging para enterprise
Las empresas en sectores regulados necesitan saber quién accedió a qué y cuándo.
- Log estructurado de cada operación: timestamp, operación, namespace, key, resultado
- Opcional (off by default): `config.audit_log = true`
- Formato JSONL en archivo separado: `audit.log`
- Rotación configurable por tamaño o tiempo

---

**TSK-110** 🆕 🟡 MEDIO — SBOM (Software Bill of Materials)
Los enterprise buyers cada vez más piden una lista de todas las dependencias con sus licencias. SLSA Level 1 como mínimo.
- Generar SBOM en formato SPDX o CycloneDX
- Incluir en cada GitHub release como artifact
- Verificar licencias: todas las dependencias deben ser compatibles con Apache 2.0

---

**BIZ-02** ✅ 🟡 MEDIO — WAL Shipping asíncrono (replicación básica)
La solución más simple para alta disponibilidad básica sin implementar Raft. Copiar el WAL a otro servidor cada N segundos — si el primario falla, el secundario puede recuperar desde el WAL.
- Script o daemon que hace rsync/scp del WAL periódicamente
- Documentado como "replication basic" — no es Raft, es WAL shipping
- Candidato a módulo comercial (open core)

---

### 5.B — Escalabilidad Single-Node

---

**TSK-51** ✅ 🟡 MEDIO — Sharded-slab para concurrencia lock-free en HNSW
El índice HNSW actual usa `RwLock` global. Con muchos writers concurrentes esto crea contención. `sharded-slab` particiona el índice para reducir lock contention.
- Evaluar si la contención actual es un problema real en benchmarks
- Implementar solo si el benchmark muestra >20% degradación bajo 8+ concurrent writers

---

**TSK-48** ✅ 🟢 BAJO — Cuantización dinámica por ciclo de vida (curva de olvido)
Reducción progresiva de precisión f32→SQ8→4bit según la frecuencia de acceso (`hits` y `last_accessed` ya en `UnifiedNode`). Los recuerdos "viejos" ocupan menos espacio.
- Implementar solo después de que SQ8 (TSK-47) esté estable
- Política: si `last_accessed` > 30 días y `hits` < 3, promover a SQ8
- Transparente para el usuario: la búsqueda sigue funcionando igual

---

### 5.C — Preparación VantaDB Cloud

---

**CLD-01** ✅ 🟡 MEDIO — VantaDB Cloud Beta
Primera versión del servicio managed: `vantadb-server` desplegado en Fly.io con almacenamiento NVMe persistente.
- Deployment: Fly.io con volumen NVMe
- Auth: Bearer token por tenant
- Pricing: Free tier (100K vectors), Pro ($49/mes)
- Dashboard básico: uso de storage, queries/día

---

**CLD-02** ✅ 🟡 MEDIO — Pitch Deck + One-pager
Necesario para las primeras reuniones con inversores.
- 10 slides: problema, solución, mercado, producto, tracción, equipo, competencia, modelo de negocio, uso del capital, visión
- One-pager ejecutivo de 1 página

---

**CLD-03** ✅ 🟡 MEDIO — Programa de pilotos enterprise (3-5 early adopters)
- Onboarding manual con soporte prioritario
- SLA informal: respuesta <4h en Slack/Discord dedicado
- A cambio: feedback estructurado, testimonio público, caso de estudio

---

**CLD-04** ✅ 🟡 MEDIO — Case Studies (mínimo 2)
Los case studies son el activo de ventas más importante para una ronda pre-seed.
- Formato: problema → implementación → resultados medidos
- Publicar en vantadb.dev/customers

---

### 5.D — Negocio

---

**BIZ-01** ✅ 🟡 MEDIO — Bifurcación del workspace (open-source vs enterprise)
Separar físicamente el código Apache 2.0 del código enterprise (WAL Shipping, encriptación, audit logs).
- `vantadb-core`: Apache 2.0, público
- `vantadb-enterprise`: BSL o SSPL, privado

---

**BIZ-03** ✅ 🟡 MEDIO — Pricing page
Sin pricing page, los enterprise buyers no saben cuánto va a costar escalar. Esto bloquea decisiones de adopción.
- Free: open-source (forever)
- Pro Cloud: $49-99/mes (vectores limitados, support básico)
- Enterprise: custom

---

---

# POST-SEED 2027+ — ESTRATÉGICO
### (No detallar hasta tener equipo y capital)

Estas son capacidades que pueden tener sentido en el futuro pero requieren recursos que hoy no existen. Se listan para no perderlas de vista, no para implementarlas ahora.

| Feature | Por qué esperar |
|---------|----------------|
| **Distributed mode / Raft** | 6-12 meses de engineering. Necesita equipo de 5+ personas. |
| **RBAC / SSO** | Solo relevante para VantaDB Cloud managed. Post-seed. |
| **GPU acceleration** | Contradice zero-config. Solo para cloud. Hardware específico. |
| **SQL completo** | Complejidad enorme. El ICP no lo necesita. pgvector ya lo tiene. |
| **GraphQL API** | El ICP prefiere API programática. Evaluar si hay demanda real. |
| **IVF-PQ disk-based** | LanceDB ya lo tiene mejor. No es el mercado de VantaDB. |
| **Data versioning git-style** | LanceDB ya lo tiene. No es el dolor del ICP de agentes. |
| **Multi-tenancy en core** | Complejidad. Workaround (múltiples instancias) funciona. |
| **Time-series mode** | Diferente producto. Fuera del scope. |
| **Versionado de embeddings** | Interesante para ML workflows, evaluar post-seed. |

---

---

# RESUMEN EJECUTIVO POR FASE

## Fase 3 — Bloqueantes y performance (julio-agosto 2026)

| Prioridad | Tareas clave |
|-----------|-------------|
| 🔴 CRÍTICO | TSK-56 (Windows CI), DISC-05 (telemetría RAM), DISC-04 (chaos testing), TSK-52 (SIGTERM) |
| 🔴 CRÍTICO | TSK-68 (Python SDK zero-copy FFI → <20ms) |
| 🟠 ALTO | TSK-73 (async Python), TSK-74 (type stubs), TSK-46 (mmap HNSW), TSK-53 (input validation) |
| 🟠 ALTO | TSK-67 (GraphRAG docs + benchmark), TSK-70 (durability doc), TSK-80 (migration guide) |
| 🟡 MEDIO | TSK-47 (SQ8), TSK-75 (WAL vacuum), TSK-76 (TTL), TSK-93 (Prometheus), TSK-94 (logging) |

**Criterio de salida:** Python SDK p50 <20ms ✓ | Windows CI verde ✓ | RAM telemetría correcta ✓ | Chaos tests en CI ✓

---

## Fase 4 — Lanzamiento comunitario (septiembre-octubre 2026)

| Prioridad | Tareas clave |
|-----------|-------------|
| 🔴 CRÍTICO | TSK-61 (TypeScript SDK), INT-01 (LangChain), INT-02 (LlamaIndex), INT-03 (MCP estable) |
| 🔴 CRÍTICO | TSK-63 (CLI backup), TSK-104 (demo app), MKT-01 (landing page), MKT-02 (blog lanzamiento) |
| 🔴 CRÍTICO | COM-01 (Discord), COM-02 (CONTRIBUTING.md), COM-03 (Code of Conduct) |
| 🟠 ALTO | TSK-45 (crates.io), TSK-101 (ARM64 Linux), TSK-90 (Mem0), TSK-103 (benchmark site) |
| 🟡 MEDIO | TSK-89 (CrewAI), TSK-91 (DSPy), TSK-65/66 (embedding adapters), TSK-96 (CLI doctor) |

**Criterio de salida:** 1K GitHub stars ✓ | 10K PyPI downloads/mes ✓ | 500 Discord members ✓ | 20 contributors ✓

---

## Fase 5 — Enterprise + pre-seed (noviembre-diciembre 2026)

| Prioridad | Tareas clave |
|-----------|-------------|
| 🟡 MEDIO | TSK-72 (encriptación at-rest), BIZ-02 (WAL shipping), CLD-01 (VantaDB Cloud beta) |
| 🟡 MEDIO | CLD-02 (pitch deck), CLD-03 (pilotos), CLD-04 (case studies), TSK-107b (audit log) |

**Criterio de salida:** 10 enterprise pilots ✓ | $10K MRR ✓ | 3 case studies ✓ | Pitch deck completo ✓

---

## Índice completo de IDs nuevos

| ID | Nombre corto | Fase | Prioridad |
|----|-------------|------|-----------|
| TSK-60 | Filtros metadata expandidos ($eq, $or, etc.) | 4 | 🟡 |
| TSK-61 | TypeScript SDK vía WASM | 4 | 🔴 |
| TSK-62 | TypeScript types + docs | 4 | 🟠 |
| TSK-63 | CLI: backup | 4 | 🔴 |
| TSK-64 | CLI: restore | 4 | 🟠 |
| TSK-65 | vantadb-openai adapter | 4 | 🟡 |
| TSK-66 | vantadb-ollama adapter | 4 | 🟡 |
| TSK-67 | GraphRAG docs + benchmark reproducible | 3 | 🟠 |
| TSK-68 | Zero-copy FFI: NumPy output (latencia Python) | 3 | 🔴 |
| TSK-69 | put_batch() con Rayon | 3 | 🟠 |
| TSK-70 | Documento de garantías de durabilidad | 3 | 🟠 |
| TSK-71 | WASM build (wasm32-wasi) | 4 | ✅ Completado |
| TSK-72 | Encriptación at-rest AES-256 | 5 | 🟡 |
| TSK-73 | Async Python API (asyncio) | 3 | 🟠 |
| TSK-74 | Python type stubs (.pyi) | 3 | 🟠 |
| TSK-75 | WAL compaction / vacuum CLI | 3 | 🟡 |
| TSK-76 | TTL en registros (Time-To-Live) | 3 | 🟠 |
| TSK-76b | Memory eviction por importancia | 3 | 🟡 |
| TSK-77 | put_batch() verificado + benchmark | 3 | 🟠 |
| TSK-78 | Property-based testing expansion | 3 | 🟡 |
| TSK-79 | Benchmark regression alerts en CI | 3 | 🟡 |
| TSK-80 | Migration guide ChromaDB/LanceDB | 3 | 🟠 |
| TSK-81 | README badges | 3 | 🟡 |
| TSK-82 | CHANGELOG.md formal | 3 | 🟡 |
| TSK-83 | Issue y PR templates GitHub | 3 | 🟡 |
| TSK-84 | Ejemplos TypeScript (LangChain.js, LlamaIndex.TS) | 4 | 🟠 |
| TSK-85 | delete_by_filter() | 4 | 🟡 |
| TSK-86 | similar_to_key() | 4 | 🟡 |
| TSK-87 | count() con filtros | 4 | 🟡 |
| TSK-88 | Multi-namespace search | 4 | 🟡 |
| TSK-89 | CrewAI adapter | 4 | 🟡 |
| TSK-90 | Mem0 integration | 4 | 🟠 |
| TSK-91 | DSPy integration | 4 | 🟡 |
| TSK-92 | Haystack adapter | 4 | 🟡 |
| TSK-93 | Prometheus integration completa | 3 | 🟡 |
| TSK-94 | Structured JSON logging | 3 | 🟡 |
| TSK-95 | vantadb-litellm adapter | 4 | 🟢 |
| TSK-96 | CLI: vanta-cli doctor | 4 | 🟡 |
| TSK-97 | CLI: vanta-cli inspect | 4 | 🟡 |
| TSK-98 | CLI: vanta-cli stats | 4 | 🟡 |
| TSK-99 | CLI: vanta-cli repl | 4 | 🟢 |
| TSK-100 | Homebrew formula macOS | 4 | 🟡 |
| TSK-101 | ARM64 Linux wheels estable | 4 | 🟠 |
| TSK-102 | Python 3.13+ support | 4 | 🟡 |
| TSK-103 | Public benchmark site reproducible | 4 | 🟠 |
| TSK-104 | Demo agent app (showcase) | 4 | 🟠 |
| TSK-106b | SECURITY.md + vulnerability process | 4 | 🟠 |
| TSK-107b | Audit logging enterprise | 5 | 🟡 |
| TSK-109 | Input hardening (path traversal, DoS) | 4 | 🟡 |
| TSK-110 | SBOM (Software Bill of Materials) | 5 | 🟡 |
| INT-01 | LangChain adapter completo | 4 | 🔴 |
| INT-02 | LlamaIndex adapter completo | 4 | 🔴 |
| INT-03 | MCP server estable | 4 | 🔴 |
| COM-01 | Discord server + canales | 4 | 🔴 |
| COM-02 | CONTRIBUTING.md | 4 | 🔴 |
| COM-03 | Code of Conduct | 4 | 🔴 |
| MKT-01 | Landing page vantadb.dev | 4 | 🔴 |
| MKT-02 | Blog post de lanzamiento | 4 | 🔴 |
| MKT-03 | Show HN post preparación | 4 | 🔴 |
| MKT-04 | Reddit posts | 4 | 🟠 |
| MKT-05 | Blog posts técnicos (5+) | 4 | 🟠 |
| MKT-06 | Logo y branding | 4 | 🟡 |
| CLD-01 | VantaDB Cloud beta | 5 | 🟡 |
| CLD-02 | Pitch deck + one-pager | 5 | 🟡 |
| CLD-03 | Programa de pilotos (3-5) | 5 | 🟡 |
| CLD-04 | Case studies (2+) | 5 | 🟡 |

---

*Última actualización: 2026-06-13 | Total de tareas nuevas: 62 | Total con backlog existente: ~120*
