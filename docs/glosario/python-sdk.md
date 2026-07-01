---
type: glossary-entry
status: stable
tags: [glosario, sdk, python, pyo3, ffi]
aliases: [Python SDK, SDK Python, vantadb-py]
---

# SDK Python

## Definición

El **SDK Python** de VantaDB es una interfaz de programación que permite a desarrolladores Python interactuar con el motor de base de datos embebido escrito en Rust, mediante bindings nativos generados con [PyO3](PyO3.md).

## Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                    Aplicación Python                         │
├─────────────────────────────────────────────────────────────┤
│                    vantadb-py (wheel)                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Python API (snake_case, type hints, docstrings)    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    PyO3 Bindings                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  #[pyclass] VantaDB                                 │   │
│  │  #[pymethods] put, get, search, delete              │   │
│  │  py.allow_threads() para liberar [GIL](GIL.md)            │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Rust Core (vantadb-core)                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  StorageEngine, HNSW, BM25, WAL                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Instalación

```bash
# Desde PyPI
pip install vantadb-py

# Verificación
python -c "import vantadb_py; print(vantadb_py.__version__)"
```

### Plataformas Soportadas

| OS | Arquitectura | Wheel Tag |
|----|--------------|-----------|
| Linux | x86_64 | `manylinux2014_x86_64` |
| Linux | ARM64 | `manylinux2014_aarch64` |
| macOS | Intel | `macosx_10_12_x86_64` |
| macOS | Apple Silicon | `macosx_11_0_arm64` |
| Windows | x86_64 | `win_amd64` |

## API Reference

### Clase Principal: VantaDB

```python
import vantadb_py as vanta

# Inicialización
db = vanta.VantaDB(
    db_path: str,                      # Ruta al directorio de datos
    memory_limit_bytes: int = None,    # Presupuesto de memoria (opcional)
    read_only: bool = False            # Modo solo lectura
)
```

### Operaciones CRUD

#### put() - Insertar/Actualizar

```python
record = db.put(
    namespace: str,                    # Espacio de nombres (ej: "agent/main")
    key: str,                          # Clave única
    payload: str,                      # Contenido de texto
    metadata: dict = None,             # Metadata estructurada
    vector: list[float] = None         # Embedding opcional
) -> dict
```

**Ejemplo:**
```python
db.put(
    namespace="knowledge_base",
    key="doc-001",
    payload="VantaDB es un motor de memoria persistente embebido.",
    metadata={"source": "documentation", "version": "0.1.4"},
    vector=[0.12, -0.34, 0.56, ...]   # 384 dimensiones
)
```

#### get_memory() - Recuperar

```python
record = db.get_memory(
    namespace: str,
    key: str
) -> dict | None
```

**Ejemplo:**
```python
doc = db.get_memory("knowledge_base", "doc-001")
if doc:
    print(f"Payload: {doc['payload']}")
    print(f"Vector: {doc['vector'][:5]}...")  # Primeros 5 floats
```

#### delete_memory() - Eliminar

```python
deleted = db.delete_memory(
    namespace: str,
    key: str
) -> bool
```

#### list_memory() - Listar

```python
page = db.list_memory(
    namespace: str,
    filters: dict = None,              # Filtros de igualdad
    limit: int = 100,
    cursor: int = None
) -> dict  # {"records": [...], "next_cursor": int | None}
```

### Búsqueda

#### search_memory()

```python
hits = db.search_memory(
    namespace: str,
    query_vector: list[float],         # Vector de consulta
    filters: dict = None,              # Filtros de metadata
    text_query: str = None,            # Query léxica ([BM25](BM25.md))
    top_k: int = 10,
    distance_metric: str = "cosine",   # "cosine" o "euclidean"
    explain: bool = False              # Incluir explicación de scoring
) -> list[dict]
```

**Ejemplo - busqueda-vectorial:**
```python
results = db.search_memory(
    namespace="knowledge_base",
    query_vector=embed("¿Cómo funciona la persistencia?"),
    top_k=10
)

for hit in results:
    print(f"{hit['record']['key']}: {hit['score']:.4f}")
    print(f"  {hit['record']['payload'][:100]}...")
```

**Ejemplo - [busqueda-hibrida](busqueda-hibrida.md):**
```python
results = db.search_memory(
    namespace="knowledge_base",
    query_vector=embed("persistencia WAL"),
    text_query="persistencia WAL",     # BM25 + HNSW + [RRF](RRF.md)
    top_k=10
)
```

#### search_batch()

```python
# Búsqueda por lotes (paralelizada con Rayon)
results = db.search_batch(
    vectors: list[list[float]],        # Lista de vectores
    top_k: int = 10
) -> list[list[tuple[int, float]]]     # [[(node_id, distance), ...], ...]
```

**Speedup:** 4.01x vs búsqueda secuencial

### Operaciones de Grafo

```python
# Añadir arista dirigida
db.add_edge(
    source_id: int,
    target_id: int,
    label: str,                        # Etiqueta (ej: "belongs_to")
    weight: float = None               # Peso opcional (default: 1.0)
) -> None
```

### Mantenimiento

```python
# Reconstruir índices
report = db.rebuild_index() -> dict
# {
#   "scanned_nodes": int,
#   "indexed_vectors": int,
#   "skipped_tombstones": int,
#   "duration_ms": int,
#   "success": bool
# }

# Auditoría del text index
report = db.audit_text_index(
    namespace: str = None,
    deep: bool = False
) -> dict

# Reparar text index
report = db.repair_text_index() -> dict
```

### Export/Import

```python
# Exportar namespace a JSONL
report = db.export_namespace(path: str, namespace: str) -> dict

# Exportar todos los namespaces
report = db.export_all(path: str) -> dict

# Importar desde JSONL
report = db.import_file(path: str) -> dict
```

### Telemetría

```python
# Perfil de hardware y memoria
profile = db.hardware_profile() -> dict
# {
#   "profile": "PERFORMANCE" | "ENTERPRISE" | "LOW_RESOURCE",
#   "process_rss_bytes": int,
#   "hnsw_logical_bytes": int,
#   "mmap_resident_bytes": int | None
# }

# Métricas operacionales
metrics = db.operational_metrics() -> dict
# {
#   "startup_ms": int,
#   "wal_replay_ms": int,
#   "ann_rebuild_ms": int,
#   "hybrid_query_ms": int
# }
```

### Lifecycle

```python
# Sincronizar buffers a disco
db.flush() -> None

# Cerrar handle del engine
db.close() -> None
```

## Liberación del [GIL](GIL.md)

VantaDB libera el Global Interpreter Lock durante operaciones pesadas para permitir concurrencia real:

```rust
// vantadb-python/src/lib.rs
#[pymethods]
impl VantaDB {
    fn search_memory(&self, py: Python, ...) -> PyResult<Vec<SearchHit>> {
        // Liberar GIL para permitir paralelismo real
        py.allow_threads(|| {
            // Rust ejecuta en paralelo usando Rayon
            self.engine.search(...)
        })
    }
}
```

**Beneficio:** Aplicaciones multi-thread pueden ejecutar búsquedas en paralelo sin bloqueo del GIL.

## Manejo de Errores

```python
from vantadb_py import VantaDB, VantaError

try:
    db = VantaDB("./data")
    db.put("ns", "key", "payload", vector=[0.1, 0.2, 0.3])
except VantaError.DatabaseLocked:
    print("Error: Base de datos bloqueada por otro proceso")
except VantaError.InvalidVector as e:
    print(f"Error: Vector inválido: {e}")
except VantaError.WalCorruption:
    print("Error: WAL corrupto, ejecutando rebuild")
    db.rebuild_index()
```

## Performance

| Operación | Latencia p50 | Throughput |
|-----------|--------------|------------|
| put() | 10.7 ms | 95 ops/sec |
| search_memory() (vectorial) | 62.0 ms | 16 qps |
| search_memory() (BM25) | 115.3 ms | 9 qps |
| search_memory() (híbrida) | 179.8 ms | 6 qps |
| search_batch() | 2.43 ms/query | 4.01x speedup |

## Type Hints

VantaDB incluye type hints completos para mejor DX:

```python
from typing import Optional, Dict, Any, List

class VantaDB:
    def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        metadata: Optional[Dict[str, Any]] = None,
        vector: Optional[List[float]] = None
    ) -> Dict[str, Any]: ...
```

## Ejemplo Completo: Pipeline RAG

```python
#!/usr/bin/env python3
"""Pipeline RAG completo con VantaDB"""
import vantadb_py as vanta
from sentence_transformers import SentenceTransformer
from openai import OpenAI

# 1. Inicializar
model = SentenceTransformer('all-MiniLM-L6-v2')
db = vanta.VantaDB("./rag_data")
llm = OpenAI()

# 2. Indexar documentos
documents = [
    "VantaDB es un motor de memoria persistente embebido.",
    "La busqueda-hibrida combina BM25 y HNSW mediante RRF.",
    "El WAL garantiza durabilidad ante fallos de energía.",
]

for i, text in enumerate(documents):
    db.put(
        namespace="knowledge_base",
        key=f"doc-{i}",
        payload=text,
        vector=model.encode(text).tolist()
    )

# 3. busqueda-hibrida
query = "¿Cómo garantiza VantaDB la durabilidad?"
hits = db.search_memory(
    namespace="knowledge_base",
    query_vector=model.encode(query).tolist(),
    text_query=query,
    top_k=3
)

# 4. Construir contexto para LLM
context = "\n\n".join([
    f"[{hit['record']['key']}] {hit['record']['payload']}"
    for hit in hits
])

# 5. Generar respuesta
response = llm.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "system", "content": "Responde basándote en el contexto."},
        {"role": "user", "content": f"Contexto:\n{context}\n\nPregunta: {query}"}
    ]
)

print(response.choices[0].message.content)
db.close()
```

## Véase También

- [PyO3](PyO3.md) - Framework de bindings Rust-Python
- [GIL](GIL.md) - Global Interpreter Lock
- [FFI](FFI.md) - Foreign Function Interface
- [SDK](SDK Python.md) - Concepto general
