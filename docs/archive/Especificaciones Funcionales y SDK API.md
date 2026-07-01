---
type: mpts-section
status: stable
tags: [vantadb, funcional, sdk, api, python, rust]
last_refined: 2026-06-18
links: "[Master Index](Master Index.md)"
description: "API pública Python/Rust, ejemplos de código production-ready, límites operativos, manejo de errores y casos de uso avanzados"
aliases: [SDK, API, Especificaciones, Python SDK, Rust SDK]
---

# Especificaciones Funcionales y SDK API

> **Dominio:** Funcional & Producto (Español)
> **Propósito:** Documentar la API pública, ejemplos de uso y límites operativos
> **Referencia actualizada en inglés:** [`docs/api/EMBEDDED_SDK.md`](../api/EMBEDDED_SDK.md) (Rust SDK)
> **Python SDK:** [`docs/api/PYTHON_SDK.md`](../api/PYTHON_SDK.md)

---

## Filosofía de Diseño de la API

### Principios

1. **Simple por defecto, potente cuando se necesita**
   - API minimalista para casos de uso comunes
   - Configuración avanzada disponible pero no requerida

2. **Type-safe y explícito**
   - Errores tipados (no strings)
   - Validación en el boundary FFI

3. **Idiomático por lenguaje**
   - Python: snake_case, type hints, context managers
   - Rust: Result<T, E>, traits, ownership claro

4. **Zero-config funcional**
   - Funciona out-of-the-box con defaults razonables
   - Configuración opcional para tuning avanzado

---

## Core Engine: Funciones Internas

### StorageEngine

**Responsabilidad:** Orquestar storage, índices y [WAL](Glosario/WAL.md).

```rust
pub struct StorageEngine {
    backend: Box<dyn StorageBackend>,
    wal: WalWriter,
    hnsw: HnswIndex,
    bm25: Bm25Index,
    config: EngineConfig,
}

impl StorageEngine {
    pub fn open(path: &Path, config: EngineConfig) -> Result<Self>;
    pub fn put(&self, node: UnifiedNode) -> Result<()>;
    pub fn get(&self, key: &str) -> Result<Option<UnifiedNode>>;
    pub fn delete(&self, key: &str) -> Result<()>;
    pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
    pub fn flush(&self) -> Result<()>;
    pub fn close(self) -> Result<()>;
}
```

### Límites Operativos

| Parámetro | Límite | Razón |
|-----------|--------|-------|
| **Key size** | 1 KB | Evitar keys excesivas |
| **Vector dimensions** | 32,768 | Límite de [HNSW](Glosario/HNSW.md) |
| **Metadata size** | 64 KB | Evitar documentos gigantes en metadata |
| **Text size** | 10 MB | Documentos muy grandes deben chunkearse |
| **Concurrent writers** | 1 | [RwLock](Glosario/RwLock.md) write es exclusivo |
| **Concurrent readers** | Ilimitado | [RwLock](Glosario/RwLock.md) read es compartido |

### Control de Estado

```rust
pub enum EngineState {
    Initializing,
    Ready,
    Rebuilding,  // Durante rebuild_index
    Flushing,    // Durante flush
    Closing,
    Closed,
}
```

**Transiciones válidas:**
- `Initializing` → `Ready` (open exitoso)
- `Ready` → `Rebuilding` (rebuild_index llamado)
- `Rebuilding` → `Ready` (rebuild completado)
- `Ready` → `Flushing` (flush llamado)
- `Flushing` → `Ready` (flush completado)
- `Ready` → `Closing` (close llamado)
- `Closing` → `Closed` (close completado)

---

## Interfaces / SDKs / APIs

### Python SDK

#### Instalación

```bash
pip install vantadb-py
```

#### Ejemplo de Interacción Mínima

```python
import vantadb_py as vanta

# 1. Crear/abrir base de datos ([Zero-Config](Glosario/Zero-Config.md))
db = vanta.VantaDB("./agent_memory", memory_limit_bytes=256 * 1024 * 1024)

# 2. Insertar nodo con contenido y vector
db.insert(
    id=1,
    content="VantaDB es una base de datos embebida para agentes de IA",
    vector=[0.12, -0.34, 0.56] * 128,  # 384 dimensiones
)

# 3. Búsqueda vectorial (semántica)
results = db.search([0.11, -0.33, 0.55] * 128, top_k=10)
for node_id, distance in results:
    print(f"{node_id}: {distance:.4f}")

# 4. Obtener nodo por ID
node = db.get(1)
print(node["id"])  # 1
print(node["vector"][:5])  # Primeros 5 floats del vector

# 5. Eliminar nodo
db.delete(1)
```

#### API Completa (Python SDK - vantadb-python)

```python
class VantaDB:
    def __new__(
        db_path: str,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False
    ) -> Self:
        """
        Crear o abrir base de datos.
        
        Args:
            db_path: Ruta al directorio de datos
            memory_limit_bytes: Presupuesto de RAM en bytes (opcional)
            read_only: Modo solo lectura (para multi-proceso)
        """
    
    # === Operaciones de Nodos ===
    
    def insert(
        self,
        id: int,
        content: str,
        vector: List[float],
        fields: Optional[Dict] = None
    ) -> None:
        """Insertar nodo con contenido, vector y campos relacionales."""
    
    def get(self, id: int) -> Optional[Dict]:
        """Obtener nodo por ID. Retorna dict o None."""
    
    def delete(self, id: int, reason: str = "manual deletion") -> None:
        """Eliminar nodo por ID con razón auditable."""
    
    def search(
        self,
        vector: List[float],
        top_k: int = 10
    ) -> List[Tuple[int, float]]:
        """Búsqueda K-NN vectorial. Retorna lista de (node_id, distance)."""
    
    def search_batch(
        self,
        vectors: List[List[float]],
        top_k: int = 10
    ) -> List[List[Tuple[int, float]]]:
        """Búsqueda K-NN vectorial en batch (paralelo con Rayon)."""
    
    def query(self, iql_query: str) -> str:
        """Ejecutar query IQL/LISP. Retorna resultado formateado."""
    
    # === Operaciones de Memoria Persistente (Namespace-scoped) ===
    
    def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        metadata: Optional[Dict] = None,
        vector: Optional[List[float]] = None,
        ttl_ms: Optional[int] = None
    ) -> Dict:
        """Insertar/actualizar registro de memoria persistente.
        Si ttl_ms se especifica, el registro expirará tras ese tiempo
        (se añade campo expires_at_ms al registro interno)."""
    
    def get_memory(self, namespace: str, key: str) -> Optional[Dict]:
        """Obtener registro de memoria por namespace/key.
        El dict retornado incluye expires_at_ms si el registro tiene TTL."""
    
    def delete_memory(self, namespace: str, key: str) -> bool:
        """Eliminar registro de memoria por namespace/key."""
    
    def list_memory(
        self,
        namespace: str,
        filters: Optional[Dict] = None,
        limit: int = 100,
        cursor: Optional[int] = None
    ) -> Dict:
        """Listar registros de memoria con filtros y paginación."""
    
    def search_memory(
        self,
        namespace: str,
        query_vector: List[float],
        filters: Optional[Dict] = None,
        text_query: Optional[str] = None,
        top_k: int = 10,
        distance_metric: Optional[str] = None,
        explain: bool = False
    ) -> List[Dict]:
        """Búsqueda híbrida (vector + texto + filtros) en memoria persistente."""
    
    # === Operaciones de Mantenimiento ===
    
    def rebuild_index(self) -> Dict:
        """Reconstruir índices ANN y derivados desde storage canónico."""
    
    def export_namespace(self, path: str, namespace: str) -> Dict:
        """Exportar un namespace como JSONL."""
    
    def export_all(self, path: str) -> Dict:
        """Exportar todos los namespaces como JSONL."""
    
    def import_file(self, path: str) -> Dict:
        """Importar registros desde export JSONL."""
    
    def audit_text_index(
        self,
        namespace: Optional[str] = None,
        deep: bool = False
    ) -> Dict:
        """Auditoría read-only del índice de texto derivado."""
    
    def repair_text_index(self) -> Dict:
        """Reconstruir índice de texto desde storage canónico."""
    
    def compact_wal(self) -> Dict:
        """Compactar/rotar el WAL. Libera espacio eliminando segmentos
        obsolete y reescribe el WAL activo. Trigger automático a 256 MB."""

    def purge_expired(self) -> int:
        """Eliminar todos los registros de memoria cuyo expires_at_ms
        sea anterior al timestamp actual. Retorna el número de registros purgados."""

    def operational_metrics(self) -> Dict:
        """Obtener métricas operativas (startup, replay, rebuild, etc.)."""
```

**Nota:** TODAS las operaciones usan `py.allow_threads()` para liberar el GIL y permitir concurrencia en aplicaciones multi-thread.

#### Validación de Inputs en FFI (TSK-53)

El Python SDK valida estrictamente los datos en la frontera FFI para prevenir panics en el core Rust:

| Input | Validación | Error |
|-------|-----------|-------|
| `float('nan')` en metadata/fields | `float.is_nan()` | `TypeError: Invalid metadata: NaN value is not allowed` |
| `float('inf')` / `float('-inf')` en metadata/fields | `float.is_infinite()` | `TypeError: Invalid metadata: Infinity value is not allowed` |
| `ListFloat` con NaN/Inf | Verificación por elemento | `TypeError` por cada float inválido |
| `dict` anidado en metadata | Aceptado como JSON anidado | N/A |

Esto protege todas las operaciones que reciben metadata/fields (`put`, `insert`, `search_memory`, etc.) contra valores IEEE 754 que causarían comparaciones NaN en `BTreeMap` o errores de serialización JSON.

#### Configuración Avanzada

```python
db = VantaEmbedded(
    "./data",
    config={
        # [HNSW](Glosario/HNSW.md) parameters
        "hnsw": {
            "M": 16,                    # Conexiones por nodo
            "ef_construction": 200,     # Candidatos en construcción
            "ef_search": 100,           # Candidatos en búsqueda
            "metric": "cosine"          # "cosine", "euclidean", "dot"
        },
        
        # [BM25](Glosario/BM25.md) parameters
        "bm25": {
            "k1": 1.2,                  # Saturación de TF
            "b": 0.75                   # Normalización de longitud
        },
        
        # Storage parameters
        "storage": {
            "backend": "fjall",         # "[Fjall](Glosario/Fjall.md)" o "[RocksDB](Glosario/RocksDB.md)"
            "sync_mode": "always"       # "always", "periodic", "never"
        },
        
        # Memory limits
        "memory": {
            "max_ram_mb": 4096,         # Límite de RAM
            "backpressure_threshold": 0.8  # 80% = activar [Backpressure](Glosario/Backpressure.md)
        }
    }
)
```

### Rust SDK

#### Instalación

```toml
[dependencies]
vantadb = "0.1.4"
```

#### Ejemplo de Interacción Mínima

```rust
use vantadb::{VantaEmbedded, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Crear/abrir base de datos
    let db = VantaEmbedded::open("./agent_memory", Config::default())?;
    
    // 2. Insertar documento
    db.put(
        "doc_001",
        Some(&[0.12, -0.34, 0.56]),  // Vector
        Some("VantaDB es una base de datos embebida"),  // Texto
        Some(metadata!{
            "source" => "documentation",
            "version" => "0.1.4"
        })
    )?;
    
    // 3. Búsqueda vectorial
    let results = db.search(
        &[0.11, -0.33, 0.55],  // Query vector
        10,                     // top_k
        SearchMode::Vector
    )?;
    
    for result in results {
        println!("{}: {:.4}", result.key, result.score);
    }
    
    // 4. Obtener documento
    if let Some(doc) = db.get("doc_001")? {
        println!("Vector: {:?}", doc.vector);
        println!("Metadata: {:?}", doc.metadata);
    }
    
    // 5. Eliminar documento
    db.delete("doc_001")?;
    
    // 6. Cerrar (automático al hacer drop)
    Ok(())
}
```

#### API Completa

```rust
pub struct VantaEmbedded {
    engine: Arc<RwLock<StorageEngine>>,
}

impl VantaEmbedded {
    pub fn open<P: AsRef<Path>>(path: P, config: Config) -> Result<Self>;
    
    pub fn put(
        &self,
        key: &str,
        vector: Option<&[f32]>,
        text: Option<&str>,
        metadata: Option<Metadata>
    ) -> Result<()>;
    
    pub fn get(&self, key: &str) -> Result<Option<UnifiedNode>>;
    
    pub fn delete(&self, key: &str) -> Result<()>;
    
    pub fn search(
        &self,
        vector: &[f32],
        top_k: usize,
        mode: SearchMode
    ) -> Result<Vec<SearchResult>>;
    
    pub fn flush(&self) -> Result<()>;
    
    pub fn close(self) -> Result<()>;
    
    pub fn stats(&self) -> Result<Stats>;
}

pub enum SearchMode {
    Vector,
    Text,
    Hybrid,
}

pub struct SearchResult {
    pub key: String,
    pub score: f32,
    pub node: UnifiedNode,
}
```

---

### TypeScript SDK (WASM)

#### Instalación

```bash
npm install vantadb
# o
bun add vantadb
```

#### Quick Start

```typescript
import VantaDB from "vantadb";

const db = await VantaDB.create({ storagePath: "./data" });

// Insertar un registro
const record = await db.put({
  namespace: "docs",
  key: "example",
  payload: "VantaDB es una base de datos vectorial embebida",
  vector: [0.12, -0.34, 0.56],
});

// Buscar por vector
const results = await db.searchVector("docs", [0.11, -0.33, 0.55], { topK: 10 });
// → [{ node_id: "12345", score: 0.98 }, ...]

// Obtener por key
const retrieved = await db.get("docs", "example");

await db.close();
```

#### API Completa (JS/TS)

| Método | Descripción |
|--------|-------------|
| `VantaDB.create(config)` | Crear/abrir base de datos |
| `put(input)` | Insertar/actualizar registro |
| `get(namespace, key)` | Obtener registro por namespace+key |
| `delete(namespace, key)` | Eliminar registro |
| `list(namespace, opts?)` | Listar con paginación y filtros |
| `searchVector(namespace, query, opts?)` | Búsqueda vectorial ANN |
| `search(namespace, query)` | Búsqueda híbrida (IQL) |
| `putBatch(inputs)` | Inserción batch |
| `getNode(id)` | Obtener nodo del grafo HNSW |
| `capabilities()` | Capacidades del runtime |
| `operationalMetrics()` | Métricas operativas |
| `flush()` | Forzar flush a disco |
| `compact()` | Compactar storage |
| `generateSnippet(query, text)` | Generar snippet con contexto |
| `close()` | Cerrar base de datos |

#### Runtime Support

| Runtime | Estado |
|---------|--------|
| Node.js ≥22 (--experimental-wasm-modules) | ✅ Probado (26 tests) |
| Bun | ✅ Esperado |
| Deno | ✅ Esperado |
| Browser (Vite/webpack) | ⏳ Con bundler plugin |

#### Ejemplos de Integración

Todos en `vantadb-ts/examples/`:

| Archivo | Framework | Uso |
|---------|-----------|-----|
| `vercel-ai-memory.mjs` | Vercel AI SDK (`@ai-sdk/openai`) | Tool calling + memoria conversacional |
| `langchain-rag.mjs` | LangChain.js (`@langchain/core`) | Pipeline RAG: split → embed → store → search |
| `llamaindex-rag.mjs` | LlamaIndex.TS (`llamaindex`) | Index docs + vector search |

Patrón común:
```typescript
import { VantaDB } from "vantadb";
const db = await VantaDB.create();
await db.put({ namespace, key, payload, vector });
const results = await db.search({ namespace, query_vector, text_query, top_k: 3 });
```

---

## Manejo de Errores

### Tipos de Error

```rust
pub enum VantaError {
    // I/O errors
    Io(std::io::Error),
    
    // Storage errors
    StorageError(String),
    WalCorruption,
    BackendError(String),
    
    // Validation errors
    InvalidKey(String),
    InvalidVector(String),
    InvalidMetadata(String),
    
    // Concurrency errors
    DatabaseAlreadyOpen,
    LockPoisoned,
    
    // Operation errors
    NotFound,
    AlreadyExists,
    IndexRebuildInProgress,
    
    // Resource errors
    OutOfMemory,
    DiskFull,
}
```

### Ejemplo de Manejo (Python)

```python
from vantadb import VantaEmbedded, VantaError

try:
    db = VantaEmbedded("./data")
    db.put("doc1", vector=[0.1, 0.2, 0.3])
except VantaError.DatabaseAlreadyOpen:
    print("Error: Base de datos ya está abierta por otro proceso")
except VantaError.InvalidVector as e:
    print(f"Error: Vector inválido: {e}")
except VantaError.DiskFull:
    print("Error: Disco lleno")
except VantaError as e:
    print(f"Error inesperado: {e}")
```

### Ejemplo de Manejo (Rust)

```rust
use vantadb::{VantaEmbedded, VantaError};

match VantaEmbedded::open("./data", Config::default()) {
    Ok(db) => {
        // Éxito
    }
    Err(VantaError::DatabaseAlreadyOpen) => {
        eprintln!("Error: Base de datos ya está abierta");
    }
    Err(VantaError::Io(e)) => {
        eprintln!("Error de I/O: {}", e);
    }
    Err(e) => {
        eprintln!("Error inesperado: {}", e);
    }
}
```

---

## Límites y Restricciones

### Límites de Datos

| Tipo | Límite | Razón |
|------|--------|-------|
| **Key** | 1 KB | Evitar keys excesivas en índices |
| **Vector** | 32,768 dimensiones | Límite práctico de HNSW |
| **Text** | 10 MB | Documentos grandes deben chunkearse |
| **Metadata** | 64 KB | Metadata no es para contenido |
| **Edges por nodo** | 10,000 | Evitar nodos super-conectados |

### Límites de Operación

| Operación | Límite | Razón |
|-----------|--------|-------|
| **Batch size** | 10,000 docs | Evitar transacciones gigantes |
| **top_k** | 1,000 | Limitar resultados |
| **Concurrent writers** | 1 | RwLock write es exclusivo |
| **Concurrent readers** | Ilimitado | RwLock read es compartido |

### Límites de Recursos

| Recurso | Límite Default | Configurable |
|---------|----------------|--------------|
| **RAM** | 80% del sistema | ✅ Sí |
| **Disk** | Hasta llenar | ❌ No |
| **File descriptors** | 1024 | ✅ Sí (ulimit) |
| **Threads** | CPU cores | ✅ Sí |

---

## Casos de Uso Avanzados

### [GraphRAG](Glosario/GraphRAG.md): Traversal de [Grafo](Glosario/Grafo.md)

```python
# Crear nodos con relaciones
db.put("alice", text="Alice es ingeniera en Acme",
       edges=[{"target": "acme", "type": "trabaja_en"}])

db.put("bob", text="Bob es amigo de Alice",
       edges=[{"target": "alice", "type": "amigo_de"}])

# Búsqueda con traversal de grafo
results = db.search(
    vector=embed("¿Quién trabaja en Acme?"),
    top_k=10,
    graph_hops=2  # Expandir 2 niveles de relaciones
)

# Resultado incluye:
# - alice (directamente relevante)
# - bob (conectado a alice, quien trabaja en acme)
```

### Búsqueda con Filtros de Metadata

```python
results = db.search(
    vector=embed("política de seguridad"),
    top_k=10,
    filter={
        "department": "legal",
        "version": {"$gte": "2.0"},
        "tags": {"$in": ["security", "compliance"]}
    }
)
```

### Batch Operations

```python
# Inserción masiva
documents = [
    {"key": f"doc_{i}", "vector": embed(text), "text": text}
    for i, text in enumerate(corpus)
]

db.put_batch(documents, batch_size=1000, ttl_ms=86_400_000)  # TTL opcional: 24h

# Búsqueda masiva
queries = [embed(q) for q in query_corpus]
results = db.search_batch(queries, top_k=10)
```

---

## Véase También

- [Master Index](Master Index.md) — Documento padre
- [Arquitectura Técnica y Core Engine](Arquitectura Técnica y Core Engine.md) — Implementación interna
- [Visión y Posicionamiento Estratégico](Visión y Posicionamiento Estratégico.md) — Casos de uso objetivo
- [Operaciones, Calidad y Riesgos](Operaciones, Calidad y Riesgos.md) — Testing y validación

---

*La API de VantaDB está diseñada para ser simple por defecto pero potente cuando se necesita, manteniendo type-safety y zero-config.*
