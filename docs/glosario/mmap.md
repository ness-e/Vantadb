---
type: glossary-entry
status: stable
tags: [io, memoria, zero-copy, performance]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Memory-Mapped I/O, Memory Mapping]
description: "Operating system syscall that maps a disk file directly to the virtual address space of a process, allowing zero-copy access to the file's contents"
---
# mmap — Memory-Mapped I/O

##Definition

**mmap** (memory mapping) is an operating system syscall that **maps a disk file directly to the virtual address space** of a process, allowing access to the contents of the file as if it were RAM, without explicit copies.

## How It Works

### Without mmap (Traditional Reading)

```
Disco: archivo.dat (10 GB)
    │
    │ read() syscall
    ▼
Kernel Buffer (copia 1)
    │
    │ copy to user space
    ▼
User Buffer (copia 2)
    │
    │ process data
    ▼
Aplicación

Total: 2 copies, 2 context switches
```

### With mmap (Zero-Copy)

```
Disco: archivo.dat (10 GB)
    │
    │ mmap() syscall
    ▼
Espacio Virtual del Proceso
┌─────────────────────────────┐
│ Punteros a páginas de disco │
│ (no están en RAM aún)       │
└─────────────────────────────┘
    │
    │ page fault (solo al acceder)
    ▼
RAM: Solo las páginas accedidas

Total: 0 copies, 1 context switch
```

## Ventajas de mmap

| Ventaja | Descripción |
|---------|-------------|
| **Zero-Copy** | Sin copias entre kernel y user space |
| **Lazy Loading** | Solo carga páginas cuando se acceden |
| **Cache del OS** | Aprovecha page cache del sistema |
| **Acceso Aleatorio** | Seek instantáneo a cualquier offset |
| **Compartido** | Múltiples procesos pueden mapear el mismo archivo |

## Usage in VantaDB

### Persistencia del Índice [[hnsw]]

```rust
use memmap2::Mmap;

pub struct HnswIndex {
    // File on memory mapped disk
    mmap: Mmap,
    
    // Index metadata
    num_vectors: usize,
    dimensions: usize,
    entry_point: usize,
}

impl HnswIndex {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Read header
        let header: &IndexHeader = unsafe {
            &*(mmap.as_ptr() as *const IndexHeader)
        };
        
        Ok(Self {
            mmap,
            num_vectors: header.num_vectors,
            dimensions: header.dimensions,
            entry_point: header.entry_point,
        })
    }
    
    pub fn get_vector(&self, idx: usize) -> &[f32] {
        let offset = HEADER_SIZE + idx * self.dimensions * 4;
        unsafe {
            std::slice::from_raw_parts(
                self.mmap.as_ptr().add(offset) as *const f32,
                self.dimensions
            )
        }
    }
}
```

### Beneficio: Carga Instantánea

| Método | Tiempo de Carga (1M vectores) |
|--------|------------------------------|
| **Leer archivo + parsear** | ~5-10 segundos |
| **mmap** | **~10 milisegundos** |

**Reason:** mmap does not read the file, it only creates virtual pointers. Pages are loaded on demand (page faults).

## Page Faults

### What is a Page Fault

When you access a mapped page that is **not in RAM**:

```
1. CPU intenta acceder a dirección virtual
2. MMU detecta que la página no está en RAM
3. Page fault exception
4. Kernel lee la página del disco
5. Página se carga en RAM
6. Proceso continúa
```

### Types of Page Faults

| Tipo | Descripción | Latencia |
|------|-------------|----------|
| **Minor** | Página está en RAM (page cache) | ~1 μs |
| **Major** | Página debe leerse del disco | ~1-10 ms |

### Optimization: Prefetching

```rust
use libc::{madvise, MADV_WILLNEED};

// Tell the OS that we will soon access these pages
unsafe {
    madvise(
        mmap.as_ptr() as *mut libc::c_void,
        len,
        MADV_WILLNEED
    );
}
```

**Effect:** The OS loads pages in the background, reducing major faults.

## Performance Considerations

### When mmap is Fast

✅ **Random access** to large files
✅ **Repeated reading** of the same file (page cache)
✅ **Files > RAM** (only load what is necessary)
✅ **Multiple processes** reading the same file

### When mmap is Slow

❌ **Sequential access** to small files (page faults overhead)
❌ **Frequent writes** (requires msync)
❌ **Network files** (NFS, SMB) — unpredictable latency
❌ **Very fast SSD** — sometimes read() + buffer is faster

## Metrics in VantaDB

### mmap Telemetry

```rust
pub struct MmapMetrics {
    // Páginas residentes en RAM
    pub resident_pages: usize,
    
    // Total de páginas mapeadas
    pub total_pages: usize,
    
    // Page faults (minor/major)
    pub minor_faults: u64,
    pub major_faults: u64,
}

impl MmapMetrics {
    pub fn collect(&self) -> Self {
        #[cfg(target_os = "linux")]
        unsafe {
            let mut vec = vec![0u8; self.total_pages];
            libc::mincore(
                self.mmap.as_ptr() as *mut libc::c_void,
                self.mmap.len(),
                vec.as_mut_ptr()
            );
            
            let resident = vec.iter().filter(|&&b| b & 1 != 0).count();
            
            Self {
                resident_pages: resident,
                total_pages: self.total_pages,
                // ...
            }
        }
    }
}
```

### Metrics Example

```
Dataset: 1M vectores (384d)
Tamaño en disco: 1.5 GB

mmap metrics:
- total_pages: 375,000 (4KB cada una)
- resident_pages: 45,000 (12% en RAM)
- minor_faults: 12,345
- major_faults: 45,678

Interpretation:
- Only 180 MB in RAM (12% of 1.5 GB)
- Most accesses are to page cache (minor faults)
- Some major faults during warmup
```

## mmap security

### Risks

1. **Dangling pointers:** If the file is truncated, the pointers are invalid
2. **SIGBUS:** Accessing an invalid page kills the process
3. **Concurrency:** Multiple processes writing = corruption

### Mitigations in VantaDB

```rust
pub struct SafeMmap {
    mmap: Mmap,
    file_lock: FileLock,  // Prevenir truncamiento
    checksum: u32,        // Validar integridad
}

impl SafeMmap {
    pub fn get_vector(&self, idx: usize) -> Result<&[f32]> {
        // 1. Validate index
        if idx >= self.num_vectors {
            return Err(Error::IndexOutOfBounds);
        }
        
        // 2. Validate checksum (optional, expensive)
        // self.validate_checksum()?;
        
        // 3. Access memory
        Ok(unsafe { self.get_vector_unchecked(idx) })
    }
}
```

## Comparison: mmap vs read()

| Dimensión | mmap | read() + buffer |
|-----------|------|-----------------|
| **Copias** | 0 (zero-copy) | 2 (kernel → user) |
| **Latencia inicial** | Baja (solo mmap syscall) | Alta (leer todo el archivo) |
| **Memoria** | Solo páginas accedidas | Todo el archivo (o chunks) |
| **Acceso aleatorio** | Instantáneo (seek virtual) | Requiere lseek() |
| **Escrituras** | Complejo (msync) | Simple (write()) |
| **Archivos > RAM** | ✅ Funciona | ❌ Requiere streaming |
| **Overhead CPU** | Bajo (page faults) | Alto (copias) |

### When to Use Each

**Use mmap:**
- Large vector indices (>100K vectors)
- Frequent random reading
- Files > Available RAM

**Use read():**
- Small files (<100 MB)
- Sequential reading
- Frequent writings
- Network Files (NFS)

## Known Issues

### AUD-08: mmap on Windows

**Severity:** ℹ️ Medium

**Description:** mmap on Windows has different behavior (stricter file locking).

**Impact:** Possible errors when deleting mapped files.

**Mitigation:**
```rust
#[cfg(windows)]
fn safe_unmap(mmap: Mmap, path: &Path) -> Result<()> {
    drop(mmap);  // Release mmap first
    std::thread::sleep(Duration::from_millis(10));  // Wait
    std::fs::remove_file(path)?;  // Now it can be deleted
    Ok(())
}
```

## See Also

- [[hnsw]] — Index used by mmap for persistence
- [[vectors]] — Data stored via mmap
- [[zero-config]] — mmap enables instant loading

### Related Implementation Documentation
- [[../architecture/hnsw_index|HNSW Index Architecture]]
- [[../operations/memory_telemetry|Memory Telemetry]]

---

*mmap is the technology that allows VantaDB to handle datasets larger than the available RAM.*

