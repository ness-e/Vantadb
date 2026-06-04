# Plan de Implementación: Fase SCALE-01d — Capa de Paging Vectorial Zero-Copy

Este plan de diseño describe la refactorización de `HnswNode` and `VectorRepresentations` en VantaDB para evitar asignaciones de memoria heap (`Vec<f32>`) de los vectores floats al cargar el índice `vector_index.bin`. En su lugar, se implementará una lectura Zero-Copy directa a través de la memoria virtual del archivo de mmap subyacente.

## User Review Required

> [!IMPORTANT]
> - Para evitar asignaciones en el heap de Rust conservando la compatibilidad de lifetimes y subprocesos con PyO3 (donde lifetimes no estáticas `'a` no son factibles), implementaremos una estructura `SendPtr` que encapsula de forma segura un puntero crudo a los datos mapeados del archivo en memoria virtual.
> - **Gestión de la dirección de memoria virtual:** Dado que las direcciones virtuales de un mmap cambian al invocar `sync_to_mmap()` al final de los flujos de escritura (flushing), implementaremos una recarga Zero-Copy automática e instantánea del índice HNSW desde el nuevo descriptor mmap. Esto garantiza que todos los punteros se actualicen a la nueva dirección física del OS evitando condiciones de carrera o dangling pointers.
> - **Alineación de datos en disco:** El acceso a vectores float32 mediante slices de Rust (`&[f32]`) requiere que las direcciones de memoria estén perfectamente alineadas a 4 bytes. Para garantizar esto de forma robusta e independiente de la plataforma, insertaremos bytes de relleno (padding) dinámico durante la serialización manual del índice. Esto cambiará ligeramente la estructura del archivo `vector_index.bin`. Incrementaremos la versión del formato a `VECTOR_INDEX_VERSION = 4`, forzando a reconstruir de forma transparente los índices viejos al vuelo.

---

## Proposed Changes

### 🐍 Componente: Python & Node Core

#### [MODIFY] [node.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/node.rs)
*   Definir un wrapper seguro `SendPtr` para encapsular `*const f32` de forma que sea seguro pasarlo a través de hilos (`Send + Sync`).
*   Añadir la variante `MmapFull(SendPtr, usize)` a `VectorRepresentations` y omitir su serialización directa con Serde (`#[serde(skip)]`).
*   Adaptar los métodos principales para que interactúen con `MmapFull` de forma transparente.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SendPtr(pub *const f32);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VectorRepresentations {
    Binary(Box<[u64]>),
    Turbo(Box<[u8]>),
    Full(Vec<f32>),
    #[serde(skip)]
    MmapFull(SendPtr, usize),
    None,
}
```

*   Actualizar `memory_size` para que retorne `0` en la variante `MmapFull`, logrando un uso constante de heap RAM.

---

### ⚙️ Componente: HNSW Core (Rust)

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
*   Subir `VECTOR_INDEX_VERSION` a `4`.
*   Actualizar `serialize_to_bytes` para alinear dinámicamente los datos de los vectores float a múltiplos de 4 bytes utilizando padding.
*   Actualizar `deserialize_from_bytes(data: &[u8], force_copy: bool)` para leer con la alineación correspondiente y decidir si se crea una copia heap (`Full`) o si se apunta al mmap (`MmapFull`).
*   Modificar `load_from_file` para recibir el parámetro `use_mmap: bool` y propagar la inicialización física de `MmapMut` en la estructura de backend si corresponde.
*   Actualizar `sync_to_mmap()` para reconstruir dinámicamente el grafo Zero-Copy a partir de la nueva dirección virtual después de sincronizar con el disco.

---

### 🗄️ Componente: Storage Core

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
*   Actualizar las llamadas a `CPIndex::load_from_file` para pasar la variable `use_mmap`.

---

## Verification Plan

### Automated Tests
*   Ejecutar las pruebas unitarias y de integración para validar la compilación y consistencia del índice HNSW:
    ```powershell
    cargo test --test storage -- --nocapture
    ```
*   Ejecutar el benchmark para verificar la integridad del rendimiento de búsqueda:
    ```powershell
    .venv\Scripts\python benchmarks/prefetch_comparison.py --size 10000 --dim 128 --queries 500
    ```
