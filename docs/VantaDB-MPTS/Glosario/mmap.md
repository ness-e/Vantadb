---
type: glosario-entry
status: stable
tags: [io, memoria, zero-copy, performance]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Memory-Mapped I/O, Memory Mapping]
description: "Syscall del sistema operativo que mapea un archivo de disco directamente al espacio de direcciones virtuales de un proceso, permitiendo acceso zero-copy al contenido del archivo"
---

# mmap — Memory-Mapped I/O

## Definición

**mmap** (memory mapping) es una syscall del sistema operativo que **mapea un archivo de disco directamente al espacio de direcciones virtuales** de un proceso, permitiendo acceder al contenido del archivo como si fuera memoria RAM, sin copias explícitas.

## Cómo Funciona

### Sin mmap (Lectura Tradicional)

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

Total: 2 copias, 2 context switches
```

### Con mmap (Zero-Copy)

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

Total: 0 copias, 1 context switch
```

## Ventajas de mmap

| Ventaja | Descripción |
|---------|-------------|
| **Zero-Copy** | Sin copias entre kernel y user space |
| **Lazy Loading** | Solo carga páginas cuando se acceden |
| **Cache del OS** | Aprovecha page cache del sistema |
| **Acceso Aleatorio** | Seek instantáneo a cualquier offset |
| **Compartido** | Múltiples procesos pueden mapear el mismo archivo |

## Uso en VantaDB

### Persistencia del Índice [HNSW](HNSW.md)

```rust
use memmap2::Mmap;

pub struct HnswIndex {
    // Archivo en disco mapeado a memoria
    mmap: Mmap,
    
    // Metadata del índice
    num_vectors: usize,
    dimensions: usize,
    entry_point: usize,
}

impl HnswIndex {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Leer header
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

**Razón:** mmap no lee el archivo, solo crea punteros virtuales. Las páginas se cargan bajo demanda (page faults).

## Page Faults

### Qué es un Page Fault

Cuando accedes a una página mapeada que **no está en RAM**:

```
1. CPU intenta acceder a dirección virtual
2. MMU detecta que la página no está en RAM
3. Page fault exception
4. Kernel lee la página del disco
5. Página se carga en RAM
6. Proceso continúa
```

### Tipos de Page Faults

| Tipo | Descripción | Latencia |
|------|-------------|----------|
| **Minor** | Página está en RAM (page cache) | ~1 μs |
| **Major** | Página debe leerse del disco | ~1-10 ms |

### Optimización: Prefetching

```rust
use libc::{madvise, MADV_WILLNEED};

// Decirle al OS que pronto accederemos a estas páginas
unsafe {
    madvise(
        mmap.as_ptr() as *mut libc::c_void,
        len,
        MADV_WILLNEED
    );
}
```

**Efecto:** El OS carga páginas en background, reduciendo major faults.

## Consideraciones de Performance

### Cuándo mmap es Rápido

✅ **Acceso aleatorio** a archivos grandes
✅ **Lectura repetida** del mismo archivo (page cache)
✅ **Archivos > RAM** (solo carga lo necesario)
✅ **Múltiples procesos** leyendo el mismo archivo

### Cuándo mmap es Lento

❌ **Acceso secuencial** a archivos pequeños (overhead de page faults)
❌ **Escrituras frecuentes** (requiere msync)
❌ **Archivos en red** (NFS, SMB) — latencia impredecible
❌ **SSD muy rápido** — a veces read() + buffer es más rápido

## Métricas en VantaDB

### Telemetría de mmap

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

### Ejemplo de Métricas

```
Dataset: 1M vectores (384d)
Tamaño en disco: 1.5 GB

mmap metrics:
- total_pages: 375,000 (4KB cada una)
- resident_pages: 45,000 (12% en RAM)
- minor_faults: 12,345
- major_faults: 45,678

Interpretación:
- Solo 180 MB en RAM (12% de 1.5 GB)
- Mayoría de accesos son a page cache (minor faults)
- Algunos major faults durante warmup
```

## Seguridad de mmap

### Riesgos

1. **Punteros dangling:** Si el archivo se trunca, los punteros son inválidos
2. **SIGBUS:** Acceder a página inválida mata el proceso
3. **Concurrencia:** Múltiples procesos escribiendo = corrupción

### Mitigaciones en VantaDB

```rust
pub struct SafeMmap {
    mmap: Mmap,
    file_lock: FileLock,  // Prevenir truncamiento
    checksum: u32,        // Validar integridad
}

impl SafeMmap {
    pub fn get_vector(&self, idx: usize) -> Result<&[f32]> {
        // 1. Validar índice
        if idx >= self.num_vectors {
            return Err(Error::IndexOutOfBounds);
        }
        
        // 2. Validar checksum (opcional, costoso)
        // self.validate_checksum()?;
        
        // 3. Acceder a memoria
        Ok(unsafe { self.get_vector_unchecked(idx) })
    }
}
```

## Comparación: mmap vs read()

| Dimensión | mmap | read() + buffer |
|-----------|------|-----------------|
| **Copias** | 0 (zero-copy) | 2 (kernel → user) |
| **Latencia inicial** | Baja (solo mmap syscall) | Alta (leer todo el archivo) |
| **Memoria** | Solo páginas accedidas | Todo el archivo (o chunks) |
| **Acceso aleatorio** | Instantáneo (seek virtual) | Requiere lseek() |
| **Escrituras** | Complejo (msync) | Simple (write()) |
| **Archivos > RAM** | ✅ Funciona | ❌ Requiere streaming |
| **Overhead CPU** | Bajo (page faults) | Alto (copias) |

### Cuándo Usar Cada Uno

**Usar mmap:**
- Índices vectoriales grandes (>100K vectores)
- Lectura aleatoria frecuente
- Archivos > RAM disponible

**Usar read():**
- Archivos pequeños (<100 MB)
- Lectura secuencial
- Escrituras frecuentes
- Archivos en red (NFS)

## Problemas Conocidos

### AUD-08: mmap en Windows

**Severidad:** ℹ️ Media

**Descripción:** mmap en Windows tiene comportamiento diferente (file locking más estricto).

**Impacto:** Posibles errores al eliminar archivos mapeados.

**Mitigación:**
```rust
#[cfg(windows)]
fn safe_unmap(mmap: Mmap, path: &Path) -> Result<()> {
    drop(mmap);  // Liberar mmap primero
    std::thread::sleep(Duration::from_millis(10));  // Esperar
    std::fs::remove_file(path)?;  // Ahora sí se puede borrar
    Ok(())
}
```

## Véase También

- [HNSW](HNSW.md) — Índice que usa mmap para persistencia
- [Vectores](Vectores.md) — Datos almacenados vía mmap
- [Zero-Config](Zero-Config.md) — mmap habilita carga instantánea

---

*mmap es la tecnología que permite a VantaDB manejar datasets más grandes que la RAM disponible.*

