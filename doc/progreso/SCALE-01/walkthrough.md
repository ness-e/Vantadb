# Walkthrough: Fase SCALE-01 — MMap con Prefetching Predictivo del Kernel

**Fecha de cierre:** 2026-05-28  
**Estado:** ✅ COMPLETADA Y VERIFICADA

---

## Resumen Ejecutivo

La Fase SCALE-01 introduce prefetching predictivo del kernel en el hot-path de búsqueda HNSW (`search_layer`), optimizando la latencia de acceso a vectores en datasets que exceden la RAM física mediante el mecanismo de memory-mapped I/O ya existente (`VantaFile`).

**Problema resuelto:** En búsquedas sobre datasets >RAM con `VantaFile` como backend, el acceso aleatorio al grafo HNSW provoca page faults síncronos cuando la CPU necesita el vector de un nodo candidato y éste no está en la caché de páginas del kernel. Esto introduce latencia de µs a ms por salto del grafo.

**Solución:** Emitir sugerencias de prefetch al OS para los vecinos *N+1* **antes** de calcular la distancia de los vecinos *N*, permitiendo al kernel iniciar DMA desde SSD → RAM en paralelo con el cómputo SIMD actual.

---

## Cambios Implementados

### `Cargo.toml`
- Añadido `Win32_System_Memory` al feature set de `windows-sys 0.52`:
  ```toml
  [target.'cfg(windows)'.dependencies]
  windows-sys = { version = "0.52", features = [
      "Win32_System_ProcessStatus",
      "Win32_System_Threading",
      "Win32_System_Memory",   # ← SCALE-01: Para PrefetchVirtualMemory
      "Win32_Foundation",
  ] }
  ```

### `src/index.rs`

#### 1. Nueva función `prefetch_mmap_vector` (líneas 13-67)

Función cross-platform, `#[inline(always)]`, best-effort (fallo silencioso):

| Plataforma | API usada | Comportamiento |
|---|---|---|
| Linux / macOS | `libc::madvise(MADV_WILLNEED)` | Async, no bloquea el hilo |
| Windows 8+ | `PrefetchVirtualMemory` | Equivalente Win32, async |
| Otros (WASM, etc.) | No-op (eliminado en release) | Transparente |

**Decisión de diseño:** Se optó por llamadas directas a la API del OS en lugar de `memmap2::Advice::WillNeed` para el bloque Windows, ya que `memmap2` no expone `PrefetchVirtualMemory`. El bloque Unix usa `libc::madvise` directamente para control granular por rango (solo el vector del nodo, no el archivo completo).

**Validación de seguridad del rango** antes de cada llamada:
```rust
if vec_start + vec_len_bytes <= mmap_len && vec_len_bytes > 0 {
    prefetch_mmap_vector(mmap_base, vec_start, vec_len_bytes);
}
```

#### 2. Integración en `search_layer` (hot-path)

Justo antes de iterar los vecinos del candidato actual para calcular distancias, se dispara un bloque de prefetch para todos los vecinos no visitados:

```
[Candidato actual N procesado]
    ↓
[Prefetch emitido para vecinos N+1..N+M no visitados]
    ↓
[Cálculo de distancia para vecinos N+1..N+M]
    (→ las páginas ya están siendo cargadas por el kernel en paralelo)
```

**Alcance del prefetch:** Solo activo cuando `vector_store: Some(vs)` — es decir, en modo `VantaFile`. El modo InMemory (heap puro) no toca esta ruta.

---

## Verificación

### Resultados

```
cargo check --all-targets
    Checking vantadb v0.1.4
    Finished `dev` profile in 6.21s  ✅

cargo test --test storage -- --nocapture
   Compiling windows-sys v0.52.0      ← feature Win32_System_Memory compilada
   Compiling vantadb v0.1.4
    Finished `test` profile in 9.60s

running 3 tests
test storage_engine_certification ... ok
test storage_engine_file_locking_test ... ok
test storage_engine_read_only_barrier_test ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured  ✅
```

### Deuda técnica documentada

| ID | Deuda | Prioridad |
|---|---|---|
| **DT-SCALE-01** | Benchmark de latencia p99 con dataset >RAM real (100K+ vectores) para cuantificar el beneficio del prefetch. `cargo bench --bench hnsw_pure` | Media |
| **DT-SCALE-02** | Evaluar si el prefetch de `M` vecinos por candidato introduce overhead excesivo en datasets InMemory pequeños (hoy ya filtrado por `vector_store: Some(vs)`, pero vale medir) | Baja |
| **DT-SCALE-03** | Capa de Paging Vectorial completa: desacoplar `Vec<f32>` del heap de Rust y delegar completamente al flat binary mmap (requiere refactor de `HnswNode`) | Futura |
