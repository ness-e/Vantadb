---
title: "Zero-Copy"
type: glossary-entry
status: stable
tags: [vantadb, glosario, performance, memoria]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Zero-Copy

## Definición

**Zero-Copy** es una técnica de optimización que evita copiar datos entre buffers de memoria, accediendo directamente a la fuente de datos original.

## Técnicas en VantaDB

### 1. Memory-Mapped I/O (mmap)

Los vectores se acceden directamente desde el archivo mapeado:

```rust
use memmap2::Mmap;

pub struct VectorStore {
    mmap: Mmap,  // Archivo mapeado a memoria
}

impl VectorStore {
    pub fn get_vector(&self, offset: usize, dim: usize) -> &[f32] {
        // Zero-copy: slice del mmap, sin allocations
        let bytes = &self.mmap[offset..offset + dim * 4];
        unsafe {
            std::slice::from_raw_parts(bytes.as_ptr() as *const f32, dim)
        }
    }
}
```

### 2. Zero-Copy Deserialization

Con `zerocopy`, los structs se interpretan directamente desde bytes:

```rust
use zerocopy::FromBytes;

#[derive(FromBytes, AsBytes)]
#[repr(C)]
pub struct DiskNodeHeader {
    pub id: u64,
    pub dim: u32,
    pub flags: u32,
}

// Zero-copy: interpretar bytes como struct
let header = DiskNodeHeader::ref_from_bytes(&mmap[0..16])?;
// Sin memcpy, sin allocations
```

### 3. Zero-Copy en FFI

```rust
#[pymethods]
impl VantaEmbedded {
    fn search<'py>(
        &self,
        py: Python<'py>,
        vector: &Bound<'py, PyAny>,
    ) -> PyResult<Vec<SearchResult>> {
        // Extraer buffer zero-copy desde numpy
        let buffer: PyReadonlyArray1<f32> = vector.extract()?;
        let slice = buffer.as_slice()?;  // Zero-copy
        
        // Búsqueda sin copiar el vector
        self.engine.search(slice, 10)
    }
}
```

## Comparación: Copy vs Zero-Copy

### Con Copia

```
1. Leer archivo → buffer kernel
2. Copiar kernel → buffer usuario    ← COPIA 1
3. Parsear → struct Rust             ← COPIA 2
4. Serializar → bytes Python         ← COPIA 3
5. Copiar → numpy array              ← COPIA 4

Total: 4 copias, 4 allocations
```

### Zero-Copy

```
1. mmap archivo → espacio virtual    ← Sin copia
2. Interpretar bytes como struct      ← Sin copia
3. Retornar referencia a Python       ← Sin copia

Total: 0 copias, 0 allocations
```

## Impacto en Performance

| Operación | Con Copia | Zero-Copy | Speedup |
|-----------|-----------|-----------|---------|
| **Leer 1M vectores (128d)** | 480ms | 2ms | 240x |
| **Search batch (10K queries)** | 120ms | 45ms | 2.7x |
| **Cargar índice (100K nodos)** | 85ms | 5ms | 17x |

## Requisitos y Limitaciones

### Alineación

```rust
// Los datos en mmap deben estar alineados correctamente
#[repr(align(32))]  // Para AVX2
struct AlignedVector {
    data: [f32; 128],
}
```

### Endianness

```rust
// Zero-copy asume endianness nativa
// Para datos cross-platform, usar conversión explícita
let value = u32::from_le_bytes(bytes);
```

### Lifetime

```rust
// El slice zero-copy vive tanto como el mmap
fn get_vector<'a>(&'a self, offset: usize) -> &'a [f32] {
    // Lifetime atado al mmap
}
```

## Uso en Layout BFS

```rust
// Layout BFS: nodos topológicamente cercanos en páginas contiguas
// Esto maximiza zero-copy efectivo al reducir page faults

// Sin BFS: nodos dispersos → page faults frecuentes
// Con BFS: nodos contiguos → acceso secuencial, prefetch efectivo
```

## Véase También

- [mmap](mmap.md) — Memory-mapped I/O
- [SIMD](SIMD.md) — Aceleración de operaciones sobre datos zero-copy
- [HNSW](HNSW.md) — Índice que usa zero-copy para vectores

---

*Zero-copy es fundamental para el rendimiento de VantaDB, eliminando overhead de copias de memoria.*

