---
type: mpts-section
status: stable
tags: [vantadb, arquitectura, rust, core, persistencia, hnsw, bm25, wal, mmap, fjall]
last_refined: 2026-06-21 (TSK-71 — WASM build + mmap shim; Browser WASM SystemTime fix)
links: "[Master Index](Master Index.md)"
description: "Principios de diseño, componentes del sistema, mecanismos de persistencia (WAL, Fjall, mmap), flujo de datos y modelo de datos unificado"
aliases: [Arquitectura, Core Engine, Diseño Técnico]
---

# Arquitectura Técnica y Core Engine

> **Dominio:** Técnico
> **Propósito:** Documentar la arquitectura interna, componentes y flujos de datos de VantaDB

---

## Principios de Diseño Basilar

### 1. Embedded-First

VantaDB es una **librería [Embebido](Glosario/Embebido.md)**, no un servicio. El core (`vantadb-core`) no depende de capas de red.

```
┌─────────────────────────────────────────┐
│         Aplicación (Python/Rust)         │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │     vantadb-core (linked library)  │ │
│  │  ┌──────┐  ┌──────┐  ┌──────────┐ │ │
│  │  │ WAL  │  │ HNSW │  │ Storage  │ │ │
│  │  └──────┘  └──────┘  └──────────┘ │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**Regla:** El servidor HTTP vive en `vanta-cli server` (in-process, feature `server`). `vantadb-server` es un **wrapper de backward-compat**. Ver [Local-First](Glosario/Local-First.md).

### 2. Canonical Data + Derived Indexes

```
Fuente de Verdad (Canónica):
├── Documentos (texto + metadata)
├── Vectores (embeddings)
└── Grafo (aristas)

Índices Derivados (Reconstruibles):
├── HNSW (búsqueda vectorial ANN)
├── BM25 (búsqueda léxica)
└── Payload indexes (filtros estructurados)
```

**Regla:** Los índices son **materializaciones efímeras**. Si se corrompen, se reconstruyen desde la fuente canónica.

#### Auditoría del Text Index (TSK-36 — Junio 2026)

Se realizó una auditoría estructural completa del índice de texto (BM25) en `src/text_index.rs`:

| Aspecto | Resultado |
|---------|-----------|
| Concurrent writes (RwLock) | ✅ Protegido correctamente |
| Rebuild desde storage canónico | ✅ Funcional, sin data loss |
| CRC32C en WAL para operaciones de texto | ✅ Heredado del sistema WAL general |
| Límite de tamaño de texto (10MB) | ✅ Validado en `put()` |
| Observaciones menores | Sin rate limit en lexical search (bajo carga, puede saturar); TOCTOU benigno en cache de segmentos |

**No se encontraron issues críticos de concurrencia o integridad.** Las observaciones menores son de optimización, no de corrección.

### 3. Zero-Cost Abstractions

Rust permite abstracciones de alto nivel sin overhead de runtime:
- **Traits** para polimorfismo estático
- **[Zero-Copy](Glosario/Zero-Copy.md)** donde sea posible ([mmap](Glosario/mmap.md))
- **[SIMD](Glosario/SIMD.md)** para operaciones vectoriales

### 4. Durabilidad Antes que Performance

**Orden de operaciones:**
1. Append al WAL
2. fsync del WAL ← **DURABILIDAD**
3. Aplicar a storage
4. ACK al cliente

**Nunca:** ACK antes de fsync.

---

## Componentes del Sistema

### Arquitectura de Capas

```
┌─────────────────────────────────────────────────────────────┐
│  Capa 5: SDK / API                                           │
│  ├── Python SDK (PyO3 bindings)                             │
│  ├── Rust SDK (native API)                                  │
│  └── MCP Server (para agentes)                              │
├─────────────────────────────────────────────────────────────┤
│  Capa 4: Query Engine                                        │
│  ├── Query Planner (AST + optimización)                     │
│  ├── Hybrid Search (HNSW + BM25 + RRF)                     │
│  └── Graph Traversal (multi-hop)                            │
├─────────────────────────────────────────────────────────────┤
│  Capa 3: Indexes                                             │
│  ├── HNSW Index (vectorial ANN)                             │
│  ├── BM25 Index (léxico)                                    │
│  └── Payload Indexes (filtros)                              │
├─────────────────────────────────────────────────────────────┤
│  Capa 2: Storage Engine                                      │
│  ├── WAL (Write-Ahead Log)                                  │
│  ├── Fjall Backend (default)                                │
│  └── RocksDB Backend (fallback)                             │
├─────────────────────────────────────────────────────────────┤
│  Capa 1: Persistence                                         │
│  ├── mmap (memory-mapped I/O)                               │
│  ├── fsync (durabilidad)                                    │
│  └── CRC32C (integridad)                                    │
└─────────────────────────────────────────────────────────────┘
```

### Componentes Principales

| Componente | Archivo | Responsabilidad |
|-----------|---------|----------------|
| **VantaEmbedded** | `src/sdk.rs` | API pública, boundary estable |
| **StorageEngine** | `src/storage.rs` | Orquestación de storage |
| **WalWriter** | `src/wal.rs` | Write-ahead log |
| **HnswIndex** | `src/index.rs` | Índice vectorial ANN |
| **Bm25Index** | `src/text_index.rs` | Índice léxico |
| **FjallBackend** | `src/backends/fjall_backend.rs` | Backend LSM-tree |
| **UnifiedNode** | `src/node.rs` | Modelo de datos unificado |

---

## Mecanismos de Persistencia y Memoria

### [WAL](Glosario/WAL.md) (Write-Ahead Log)

**Propósito:** Garantizar durabilidad antes de aplicar mutaciones.

```
┌─────────────────────────────────────┐
│         WAL Record                   │
├─────────────────────────────────────┤
│ Header (8 bytes)                    │
│ ├── Length: u32                     │
│ ├── Type: u8 (Insert/Delete/Update) │
│ └── Flags: u8                       │
├─────────────────────────────────────┤
│ Payload (variable)                  │
│ ├── Key: [u8]                       │
│ ├── Vector: [f32]                   │
│ ├── Text: [u8]                      │
│ └── Metadata: [u8]                  │
├─────────────────────────────────────┤
│ Checksum: u32 (CRC32C)              │
└─────────────────────────────────────┘
```

**Flujo de Escritura:**
```
1. Serializar mutación
2. Calcular CRC32C del payload
3. Append al archivo wal.log
4. fsync() ← DURABILIDAD GARANTIZADA
5. Aplicar a Fjall/RocksDB
6. Actualizar índices (HNSW, BM25)
7. ACK al cliente
```

### Backend de Storage: [Fjall](Glosario/Fjall.md)

**Fjall** es un motor [LSM-Tree](Glosario/LSM-Tree.md) 100% Rust, elegido como backend canónico:

| Característica | Fjall | RocksDB |
|---------------|-------|---------|
| Lenguaje | 100% Rust | C++ (bindings) |
| Compilación | Rápida (~30s) | Lenta (~5-10min) |
| Dependencias | Cero | CMake, Clang, libstdc++ |
| Seguridad | Safe Rust | `unsafe` en bindings |
| MVCC | ✅ Nativo | ✅ Soportado |

**Trait StorageBackend:**
```rust
pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
```

### [HNSW](Glosario/HNSW.md) (Hierarchical Navigable Small World)

**Propósito:** Búsqueda aproximada de vecinos más cercanos ([ANN](Glosario/ANN.md)) en tiempo logarítmico.

```
Capa 2 (más dispersa):
    [A] ────────── [D]

Capa 1 (intermedia):
    [A] ─── [B] ─── [D]
     │       │       │
    [E] ─── [C] ─── [F]

Capa 0 (más densa, todos los vectores):
    [A]─[B]─[C]─[D]─[E]─[F]─[G]─[H]─[I]─[J]
```

**Parámetros:**
- **M:** Máximo de conexiones por nodo (default: 16)
- **ef_construction:** Candidatos durante construcción (default: 200)
- **ef_search:** Candidatos durante búsqueda (default: 100)

**Persistencia:** [mmap](Glosario/mmap.md) del grafo completo → carga instantánea.

### [BM25](Glosario/BM25.md) (Best Matching 25)

**Propósito:** Búsqueda léxica por keywords.

```
Índice Invertido:
"base" → [doc1, doc3, doc7, ...]
"datos" → [doc1, doc2, doc3, ...]
"vectorial" → [doc1, doc8, doc20, ...]
```

**Fórmula de Scoring:**
$$
\text{score}(D, Q) = \sum_{i=1}^{n} \text{IDF}(q_i) \cdot \frac{f(q_i, D) \cdot (k_1 + 1)}{f(q_i, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)}
$$

---

## Flujo de Datos Principal

### Inserción de Documento

```
Cliente: db.put("doc1", vector, text, metadata)
    │
    ▼
┌────────────────────────┐
│ 1. Validar inputs      │
│    - key no vacío      │
│    - vector dim válida │
│    - metadata válida   │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 2. Serializar mutación │
│    Mutation::Insert {  │
│      key, vector,      │
│      text, metadata    │
│    }                   │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 3. Append al WAL       │
│    - Calcular CRC32C   │
│    - Escribir registro │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 4. fsync() del WAL     │ ← DURABILIDAD
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 5. Aplicar a Fjall     │
│    - Insert documento  │
│    - Insert vector     │
│    - Insert metadata   │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│ 6. Actualizar índices  │
│    - HNSW: add vector  │
│    - BM25: index text  │
└──────────┬─────────────┘
           │
           ▼
      ACK al cliente
```

### Búsqueda Híbrida

```
Cliente: db.search(vector, text, top_k=10)
    │
    ├─▶ HNSW Index
    │   └─▶ Lista 1: [doc5, doc12, doc23, ...]
    │
    ├─▶ BM25 Index
    │   └─▶ Lista 2: [doc3, doc7, doc12, doc45, ...]
    │
    └─▶ RRF Fusion
        └─▶ Ranking Unificado: [doc12, doc7, doc45, ...]
            │
            ▼
        Retornar top-K al cliente
```

---

## Modelo de Datos

### UnifiedNode

```rust
pub struct UnifiedNode {
    pub id: u64,                              // Identificador único global
    pub bitset: u128,                         // 128-bit fast filter (country, role, active, etc.)
    pub semantic_cluster: u32,                // Cluster semántico para routing
    pub flags: NodeFlags,                     // Flags de estado (ACTIVE, INDEXED, TOMBSTONE, etc.)
    pub vector: VectorRepresentations,       // Representación vectorial (Full, Turbo, Binary, MmapFull, None)
    pub epoch: u32,                           // Versión de lineage
    pub edges: Vec<Edge>,                     // Relaciones de grafo
    pub relational: RelFields,                // Campos relacionales (BTreeMap<String, FieldValue>)
    pub tier: NodeTier,                       // Storage tier: Hot (RAM) o Cold (disk)
    pub hits: u32,                            // Frecuencia de acceso
    pub last_accessed: u64,                   // Timestamp último acceso (Unix MS)
    pub confidence_score: f32,                // Score de confianza (0.0 - 1.0)
    pub importance: f32,                     // Score de importancia (0.0 - 1.0)
    pub ext_metadata: HashMap<String, Vec<u8>>, // Metadata extensible para forward-compatibility
}
```

### Edge (Relación de Grafo)

```rust
pub struct Edge {
    pub target: u64,             // ID del nodo destino
    pub label: String,           // Etiqueta de la relación
    pub weight: f32,             // Peso de la arista (default: 1.0)
}
```

### FieldValue (Tipos de Metadata)

```rust
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
    ListString(Vec<String>),
    ListInt(Vec<i64>),
    ListFloat(Vec<f64>),
    ListBool(Vec<bool>),
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    Null,
}
```

**Nota:** `VantaValue` es un alias estable para SDKs externos, pero internamente el core usa `FieldValue`.

---

## Concurrencia y Seguridad

### Gestión del [GIL](Glosario/GIL.md) en [PyO3](Glosario/PyO3.md)

El Python SDK (`vantadb-python`) libera consistentemente el GIL en todas las operaciones >10ms usando `py.allow_threads()`:

```rust
#[pymethods]
impl VantaDB {
    fn search(&self, py: Python, vector: Vec<f32>, top_k: usize) -> PyResult<Vec<(u64, f32)>> {
        let engine = self.engine.clone();
        py.allow_threads(move || {
            engine.search_vector(&vector, top_k)
                .map(|hits| hits.into_iter().map(|hit| (hit.node_id, hit.distance)).collect())
                .map_err(|e| PyRuntimeError::new_err(format!("Search error: {:?}", e)))
        })
    }
    
    fn put(&self, py: Python, id: u64, content: &str, vector: Vec<f32>, fields: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let mut input = VantaNodeInput::new(id);
        input.content = Some(content.to_string());
        input.vector = (!vector.is_empty()).then_some(vector);
        // ... procesar fields ...
        
        let engine = self.engine.clone();
        py.allow_threads(move || {
            engine.insert_node(input)
                .map_err(|e| PyRuntimeError::new_err(format!("Insert error: {:?}", e)))
        })?;
        Ok(())
    }
}
```

**Regla:** TODAS las operaciones del Python SDK usan `py.allow_threads()` para permitir concurrencia en aplicaciones multi-thread.

### [RwLock](Glosario/RwLock.md) para Concurrencia Interna

```rust
pub struct VantaEmbedded {
    engine: Arc<RwLock<Engine>>,
}

// Lectura (múltiples threads)
fn get(&self, key: &str) -> Result<Option<Value>> {
    let engine = self.engine.read().unwrap();
    engine.get(key)
}

// Escritura (exclusivo)
fn put(&self, key: &str, value: Value) -> Result<()> {
    let mut engine = self.engine.write().unwrap();
    engine.put(key, value)
}
```

### [File Locking](Glosario/File Locking.md) (Multi-Proceso)

```rust
use fs2::FileExt;

pub fn open(path: &Path) -> Result<Self> {
    let lock_path = path.join(".vantadb.lock");
    let lock_file = File::create(&lock_path)?;
    
    lock_file.try_lock_exclusive()
        .map_err(|_| Error::DatabaseAlreadyOpen)?;
    
    // ... abrir DB ...
    
    Ok(Self { _lock_file: lock_file })
}
```

---

### Soporte WASM (WebAssembly)

A partir de TSK-71, el core de VantaDB compila para `wasm32-wasip1` mediante un enfoque de **compilación condicional** (`cfg`):

| Dependencia | Native | WASM | Estrategia |
|------------|--------|------|------------|
| **sysinfo** | ✅ Real | ❌ Stub | Feature opcional (`sysinfo`) |
| **memmap2** | ✅ mmap real | ✅ Vec-backed shim | Feature opcional (`mmap`), shim en `src/wasm/mmap.rs` |
| **fs2** | ✅ File locking | ❌ Stub devuelve Ok | Feature opcional (`fs2`), stub vacío |
| **prometheus** | ✅ Métricas reales | ❌ Statics cfg-gated | `#[cfg(feature = "prometheus")]` en todos los statics |
| **rayon** | ✅ Thread pool real | ❌ Fallback secuencial | Feature opcional (`rayon`), fallback `iter().map().collect()` |

**Uso del shim mmap para WASM:**
```rust
// src/wasm/mmap.rs
pub struct MmapVec {
    data: Vec<u8>,
    len: usize,
}

impl MmapVec {
    pub fn map(len: usize) -> Self {
        Self { data: vec![0u8; len], len }
    }
    pub fn as_slice(&self) -> &[u8] { &self.data[..self.len] }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { &mut self.data[..self.len] }
}
```

**Build:**
```bash
# Native (default features)
cargo build --release

# WASM
cargo build --release --no-default-features --features wasm --target wasm32-wasip1
```

Esto habilita el pipeline **TypeScript SDK** (4.A) para empaquetar VantaDB via npm con WASM.

#### WASM Browser Build (wasm32-unknown-unknown)

El crate `vantadb-wasm` compila para el target browser (`wasm32-unknown-unknown`) vía `wasm-pack`. Para evitar panics de `std::time::SystemTime::now()` (no disponible en este target), todas las ocurrencias de `std::time::SystemTime`/`UNIX_EPOCH` en el código base fueron reemplazadas por `web_time::SystemTime`/`UNIX_EPOCH` (crate `web-time` v1.1.0), que funciona tanto en native como WASM.

```bash
cd vantadb-wasm && cargo build --target wasm32-unknown-unknown
```

---

## Métricas de Performance

### Core Rust (Stress Protocol)

| Dataset | Recall@10 | Latencia p50 | Memory |
|---------|-----------|--------------|--------|
| 10K vectores (128d) | 0.956 | 1.2 ms | ~12 MB |
| 50K vectores (128d) | 1.000 | 6.1 ms | ~58 MB |
| 100K vectores (128d) | 0.998 | 12.4 ms | ~117 MB |

**Factor de escalado:** 4.83x (LINEAL O(N))

> **Corrección importante:** El consumo de memoria es **LINEAL O(N)** con el número de vectores. Lo que es sub-lineal es el **tiempo de búsqueda** en HNSW: O(log N).

### Python SDK

| Operación | Latencia p50 | Throughput |
|-----------|--------------|------------|
| Ingesta (PUT) | 10.7 ms | 95 ops/sec |
| Search Vectorial | 62.0 ms | 16 qps |
| Search Léxica (BM25) | 115.3 ms | 9 qps |
| Search Híbrida (RRF) | 179.8 ms | 6 qps |
| search_batch | 2.43 ms/query | 4.01x speedup |

---

## Véase También

- [Master Index](Master Index.md) — Documento padre
- [Visión y Posicionamiento Estratégico](Visión y Posicionamiento Estratégico.md) — Por qué se diseñó así
- [Especificaciones Funcionales y SDK API](Especificaciones Funcionales y SDK API.md) — Cómo se usa
- [Operaciones, Calidad y Riesgos](Operaciones, Calidad y Riesgos.md) — Riesgos conocidos

---

*La arquitectura de VantaDB prioriza durabilidad, simplicidad y performance, siguiendo principios de sistemas embebidos modernos.*
