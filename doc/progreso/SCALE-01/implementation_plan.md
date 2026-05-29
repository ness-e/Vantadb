# Plan de Implementación: MMap con Prefetching Predictivo y Capa de Paging para Escalabilidad >RAM (Fase SCALE-01)

Este plan de implementación define las especificaciones arquitectónicas y de ingeniería para evolucionar la escalabilidad física de VantaDB, permitiendo indexar y buscar conjuntos de datos vectoriales masivos que superan ampliamente la capacidad de la memoria RAM física del sistema sin provocar fallos de Out-Of-Memory (OOM) ni degradación severa por fallos de página (*page faults*).

---

## 1. Goal Description

El objetivo primordial de la fase **SCALE-01** es optimizar la huella de memoria del grafo HNSW de `CPIndex` para conjuntos de datos masivos (>100K-500K vectores). 
Actualmente, `CPIndex` retiene la estructura del grafo y los vectores de coordenadas pesados `Vec<f32>` en un `HashMap` ordinario en la memoria heap de Rust, lo cual limita su escalabilidad.

Proponemos la siguiente solución de ingeniería de sistemas de bajo nivel:
1. **Desacoplamiento de Vectores Pesados de la Memoria Heap (Capa de Paging):** Redefinir la estructura de `HnswNode` para que mantenga su topología de enlaces e IDs ligera en el heap de Rust, pero delegar el almacenamiento y recuperación de las coordenadas de los vectores dimensionales `Vec<f32>` al backend memory-mapped (`memmap2`) de disco de forma perezosa (*lazy loading*).
2. **Prefetching Predictivo con `madvise` / Win32 Equivalentes:** Durante la búsqueda HNSW, el acceso aleatorio al grafo provoca costosos fallos de página físicos al leer vectores desde el archivo mapped. Implementaremos una precarga asíncrona kernel-level mediante `madvise` (`MADV_WILLNEED` en sistemas POSIX) o pre-lectura virtual de memoria en Windows justo cuando un nodo vecino es encolado como candidato, anticipando el cálculo de distancia antes de que la CPU lo requiera.
3. **Mantenimiento y Certificación de Latencia p99:** Asegurar que la penalización por fallos de página no afecte negativamente el rendimiento p99 comparado con la ingesta y consultas puramente en memoria.

---

## 2. User Review Required

> [!IMPORTANT]
> **Estrategia de Paginar Vectores al Mmap de Disco:**
> Para no reescribir todo el algoritmo de HNSW en Rust (lo cual añadiría meses de desarrollo e inestabilidad al motor), mantendremos la topología ligera del grafo (los arrays de IDs de vecinos por capas) en memoria RAM del heap de Rust, pero moveremos la colección de arrays flotantes (`Vec<f32>`) a un layout binario plano mapeado por `memmap2` en disco. 
> Esto disminuye el consumo en heap en más de un **80%**, permitiendo escalar de forma nativa a datasets masivos con el mismo core síncrono.

> [!WARNING]
> **Compatibilidad Cross-Platform de Prefetching de Memoria:**
> La llamada `madvise` (`MADV_WILLNEED`) es exclusiva de sistemas Unix/POSIX. En Windows (`#[cfg(windows)]`), utilizaremos `PrefetchVirtualMemory` de la API de Win32 para pre-cargar páginas físicas al espacio de trabajo del proceso de forma asíncrona a nivel de kernel, previniendo latencia y bloqueos de I/O en hilos de búsqueda. En caso de fallos del OS o no estar soportado, el sistema usará un fallback de pre-lectura síncrona ligera o lectura directa segura.

---

## 3. Open Questions

No hay preguntas de diseño abiertas. La abstracción actual de `IndexBackend` de `CPIndex` ya prevé variantes de MMap que facilitarán la implementación de forma transparente al usuario.

---

## 4. Proposed Changes

### Arquitectura de la Capa de Paging Vectorial (MMap-Backed Vectors)

```mermaid
graph TD
    subgraph RAM (Rust Heap)
        HNSW[HNSW Topology Graph]
        Node1["HnswNode (ID: 42)"]
        Node1 -->|"Layer 0: [12, 85, 99]"| HNSW
        Node1 -->|"Vector Offset: 1680"| MMapFlat
    end

    subgraph Kernel Space (Disk OS Cache)
        MMapFlat["Flat MMap Vector File (memmap2)"]
        Vector42["[0.1, 0.45, ..., 0.89] (Size: 128d)"]
        MMapFlat -.->|Page Fault / Lazy Load| Vector42
        madvise["madvise (MADV_WILLNEED / PrefetchVirtualMemory)"] -.->|Predictive Load| MMapFlat
    end
```

---

### Componente 1: Paging y Desacoplamiento de Vectores

#### [MODIFY] [src/index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
**Especificación:**
1. Modificar `HnswNode` para que el campo de coordenadas vectoriales sea opcional o mapeado:
```rust
pub struct HnswNode {
    pub id: u64,
    pub neighbors: Vec<Vec<u64>>,
    pub storage_offset: u64,
    /// Offset del vector en el archivo binario plano mapeado en memoria.
    /// Si es None, significa que está en memoria (modo InMemory).
    pub vector_mmap_offset: Option<u64>,
    /// Conservado para compatibilidad y modo puramente InMemory.
    pub in_memory_vector: Option<Vec<f32>>,
}
```
2. Implementar recuperación perezosa de vectores a través de `IndexBackend`:
```rust
impl HnswNode {
    pub fn get_vector(&self, backend: &IndexBackend) -> &[f32] {
        if let Some(ref v) = self.in_memory_vector {
            return v;
        }
        if let Some(offset) = self.vector_mmap_offset {
            if let IndexBackend::MMapFile { mmap: Some(m), .. } = backend {
                // Recuperar la rebanada (slice) de floats de forma directa del mmap binario
                let start = offset as usize;
                let len = self.vector_dimension() * 4; // float = 4 bytes
                let byte_slice = &m[start..start + len];
                return unsafe {
                    std::slice::from_raw_parts(
                        byte_slice.as_ptr() as *const f32,
                        self.vector_dimension()
                    )
                };
            }
        }
        panic!("Inconsistent HnswNode: No vector source available.");
    }
}
```

3. Durante `search_nearest`, obtener los vectores usando el método `get_vector(&self.backend)` en lugar de leer el campo directamente del nodo.

---

### Componente 2: Prefetching Predictivo del Kernel (madvise)

#### [MODIFY] [src/index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
**Especificación:**
1. Al recorrer los candidatos en el algoritmo HNSW (bucle de búsqueda), justo cuando encolamos los vecinos no explorados para calcular distancias, disparamos una indicación al sistema operativo mediante prefetching predictivo sobre la dirección física en memoria del mmap del vector del nodo vecino.
```rust
#[cfg(unix)]
fn prefetch_mmap_vector(mmap_ptr: *const u8, offset: u64, len: usize) {
    unsafe {
        let addr = mmap_ptr.add(offset as usize) as *mut std::ffi::c_void;
        libc::madvise(addr, len, libc::MADV_WILLNEED);
    }
}

#[cfg(windows)]
fn prefetch_mmap_vector(mmap_ptr: *const u8, offset: u64, len: usize) {
    use windows_sys::Win32::System::Memory::{PrefetchVirtualMemory, WIN32_MEMORY_RANGE_ENTRY};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let addr = mmap_ptr.add(offset as usize) as *mut std::ffi::c_void;
        let mut entry = WIN32_MEMORY_RANGE_ENTRY {
            VirtualAddress: addr,
            NumberOfBytes: len,
        };
        let process_handle = GetCurrentProcess();
        PrefetchVirtualMemory(process_handle, 1, &mut entry, 0);
    }
}
```

2. Integrar `prefetch_mmap_vector` en el bucle caliente de `search_nearest`:
```rust
// Durante la evaluación de candidatos vecinos en HNSW
for &neighbor in neighbors_to_eval {
    if !visited.contains(&neighbor) {
        if let Some(node) = self.nodes.get(&neighbor) {
            if let Some(offset) = node.vector_mmap_offset {
                if let IndexBackend::MMapFile { mmap: Some(m), .. } = &self.backend {
                    let len = node.vector_dimension() * 4;
                    prefetch_mmap_vector(m.as_ptr(), offset, len);
                }
            }
        }
    }
}
```
Esto permite que el sistema operativo inicie la lectura asíncrona del vector desde el SSD a la caché de RAM física del kernel en paralelo mientras la CPU realiza el cálculo aritmético SIMD actual, mitigando los bloqueos por page fault.

---

## 5. Verification Plan

### Automated Tests
1. **Verificación de Integridad:**
   Asegurar que la carga, guardado e inserción a través de la capa de paging y mmap pase la suite de tests existente.
   ```powershell
   cargo test --test storage -- --nocapture
   cargo test --test mmap_index -- --nocapture
   ```
2. **Pruebas de Estrés con datasets >RAM:**
   Correr el script de benchmark `criterion` o un test de integración específico con 100K+ vectores para medir la estabilidad física de la RAM y certificar que no ocurre OOM.
   ```powershell
   cargo bench --bench hnsw_pure
   ```

### Manual Verification
1. Medir las métricas de memoria antes y después de la implementación de paging mediante el comando de métricas o inspeccionando los logs de telemetría de memoria.
2. Certificar que el contador de `mmap_resident_bytes` se mantiene acotado de forma óptima durante la exploración y búsqueda en comparación con la memoria total retenida por el proceso.
