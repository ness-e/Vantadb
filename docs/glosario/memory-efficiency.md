---
title: "Memory Efficiency"
type: glossary-entry
status: stable
tags: [glosario, métricas, memoria, eficiencia, ram]
last_reviewed: 2026-07-03
aliases: [memory usage, RAM efficiency, footprint]
---

# Memory Efficiency

## Definición

La **eficiencia de memoria** mide cuánta RAM consume el sistema por cada elemento indexado. En bases de datos vectoriales, se expresa típicamente como **bytes por vector**.

## Fórmula

$$\text{Memory Efficiency} = \frac{\text{Total RAM Usage}}{\text{Number of Vectors}} \quad \text{(bytes/vector)}$$

## En VantaDB

### Métricas Certificadas

| Dataset | Vectores | RAM Total | Bytes/Vector |
|---------|----------|-----------|--------------|
| Small | 10,000 | ~12 MB | 1,200 |
| Medium | 50,000 | ~58 MB | 1,160 |
| Large | 100,000 | ~117 MB | **1,172** |
| X-Large | 500,000 | ~585 MB | 1,170 |

**Objetivo:** <1,500 bytes/vector para vectores de 128 dimensiones

### Componentes del Uso de Memoria

```
Total RAM: ~117 MB (100K vectores, 128d)
├── HNSW graph structure: ~45 MB (38%)
│   ├── Adjacency lists: ~30 MB
│   └── Node metadata: ~15 MB
├── Vector storage: ~51 MB (44%)
│   └── 100K × 128 × 4 bytes = 51.2 MB
├── BM25 index: ~12 MB (10%)
│   ├── Postings: ~8 MB
│   └── Term dictionary: ~4 MB
└── Overhead: ~9 MB (8%)
    ├── WAL buffer: ~2 MB
    ├── Caches: ~5 MB
    └── Misc: ~2 MB
```

### Comparación con Alternativas

| Sistema | Bytes/Vector (128d) | Notas |
|---------|---------------------|-------|
| **VantaDB** | ~1,172 | HNSW + BM25 + metadata |
| **FAISS** (HNSW) | ~800 | Solo vectores, sin metadata |
| **Pinecone** | ~1,500 | Estimado (cloud) |
| **Qdrant** | ~1,400 | Con metadata |
| **ChromaDB** | ~2,000 | Python overhead |

## Optimización de Memoria

### 1. Memory-Mapped Files ([mmap](mmap.md))

VantaDB utiliza mmap para delegar la gestión de memoria al sistema operativo:

```rust
// src/storage.rs - VantaFile
pub struct VantaFile {
    mmap: MmapMut,  // memmap2
    path: PathBuf,
}

impl VantaFile {
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(Self { mmap, path: path.to_path_buf() })
    }
    
    pub fn get_vector(&self, offset: u64, dim: usize) -> &[f32] {
        let start = offset as usize;
        let end = start + dim * 4;
        unsafe {
            std::slice::from_raw_parts(
                self.mmap.as_ptr().add(start) as *const f32,
                dim
            )
        }
    }
}
```

**Beneficios:**
- RAM usage ≈ 0 (OS maneja page faults)
- Datos persisten en disco automáticamente
- Escalabilidad a datasets >RAM

### 2. Cuantización de Vectores

#### SQ8 (Scalar Quantization 8-bit)

```rust
// Reducir de f32 (4 bytes) a u8 (1 byte)
pub fn quantize_sq8(vector: &[f32], min: f32, max: f32) -> Vec<u8> {
    vector.iter()
        .map(|&x| ((x - min) / (max - min) * 255.0) as u8)
        .collect()
}

// Uso: 75% reducción de memoria
// Original: 128 × 4 = 512 bytes
// SQ8: 128 × 1 = 128 bytes
```

**Impacto:**
- Memoria: 1,172 → ~293 bytes/vector (75% reducción)
- Recall: 0.998 → ~0.985 (1.3% pérdida)

### 3. Compactación de Grafo HNSW

```rust
// Layout BFS (Breadth-First Search) para localidad de caché
pub fn compact_layout_bfs(&mut self) {
    let mut new_order = Vec::with_capacity(self.node_count);
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    
    // BFS desde entry point
    queue.push_back(self.entry_point);
    visited.insert(self.entry_point);
    
    while let Some(node_id) = queue.pop_front() {
        new_order.push(node_id);
        
        for &neighbor in &self.graph[node_id].neighbors {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }
    
    // Reordenar vectores según BFS order
    self.reorder_vectors(&new_order);
}
```

**Beneficio:** Mejor localidad de caché → menos page faults

### 4. Shared Memory para Multi-Proceso

```rust
// Múltiples procesos pueden leer el mismo mmap
// sin duplicar vectores en RAM

// Proceso 1
let db1 = VantaDB::open("./data")?;
db1.search(query)?;

// Proceso 2 (misma base de datos)
let db2 = VantaDB::open("./data")?;
db2.search(query)?;

// RAM total: ~117 MB (no 234 MB)
// Porque ambos procesos comparten el mismo mmap
```

## Monitoreo de Memoria

### Telemetría Integrada

```python
# Perfil de hardware
profile = db.hardware_profile()

print(f"RSS (Resident Set Size): {profile['process_rss_bytes'] / 1024 / 1024:.1f} MB")
print(f"HNSW logical: {profile['hnsw_logical_bytes'] / 1024 / 1024:.1f} MB")
print(f"mmap resident: {profile['mmap_resident_bytes'] / 1024 / 1024:.1f} MB")
```

### Métricas Detalladas

| Métrica | Descripción | Uso |
|---------|-------------|-----|
| `process_rss_bytes` | RAM física usada por el proceso | Monitoreo de OOM |
| `hnsw_logical_bytes` | Tamaño lógico del índice HNSW | Capacidad |
| `mmap_resident_bytes` | Páginas mmap en RAM física | Page fault prediction |
| `volatile_cache_entries` | Entradas en caché volátil | Hit rate |

### Alertas de Memoria

```python
import psutil

# Monitorear uso de RAM
rss = db.hardware_profile()['process_rss_bytes']
available = psutil.virtual_memory().available

if rss > available * 0.8:
    print("⚠️ Memoria al 80%, considerar:")
    print("  1. Habilitar mmap para índices grandes")
    print("  2. Usar cuantización SQ8")
    print("  3. Aumentar RAM del sistema")
```

## Trade-offs: Memoria vs Performance

| Configuración | Memoria | Latencia | Recall |
|---------------|---------|----------|--------|
| **Full precision (f32)** | 1,172 B/vec | 1.2 ms | 0.998 |
| **SQ8 quantization** | ~293 B/vec | 0.8 ms | 0.985 |
| **PQ (Product Quantization)** | ~128 B/vec | 0.5 ms | 0.950 |
| **Binary (1-bit)** | ~16 B/vec | 0.3 ms | 0.850 |

## Casos de Uso por Tamaño de Dataset

| Dataset | Vectores | RAM Estimada | Recomendación |
|---------|----------|--------------|---------------|
| **Small** | <10K | <12 MB | Full precision, in-memory |
| **Medium** | 10K-100K | 12-117 MB | Full precision, mmap opcional |
| **Large** | 100K-1M | 117 MB - 1.2 GB | mmap obligatorio |
| **X-Large** | 1M-10M | 1.2-12 GB | mmap + SQ8 |
| **Massive** | >10M | >12 GB | mmap + PQ, considerar sharding |

## Ejemplo: Estimación de Requisitos

**Escenario:** 1 millón de vectores de 384 dimensiones

**Cálculo:**
```
Vectores: 1,000,000 × 384 × 4 bytes = 1.54 GB (solo vectores)
HNSW graph: ~0.5 GB (estructura de grafo)
BM25 index: ~0.2 GB (asumiendo 1M documentos)
Overhead: ~0.3 GB (WAL, caches, misc)

Total estimado: ~2.5 GB RAM
```

**Recomendación:**
- Sistema con ≥4 GB RAM disponible
- Habilitar mmap para vectores
- Considerar SQ8 si RAM <2 GB

## Véase También

- [mmap](mmap.md) - Memory-mapped I/O
- [Benchmarks](Benchmarks.md) - Suite de medición
- [Latencia](Latencia.md) - Métrica complementaria
- [Recall](Recall.md) - Métrica de calidad
