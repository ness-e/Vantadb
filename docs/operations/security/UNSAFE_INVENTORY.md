# Unsafe/Unwrap Inventory — 2026-07-21

> Baseline de todo código `unsafe`, `.unwrap()`, y `.expect()` en producción del workspace VantaDB.
> Excluye módulos `#[cfg(test)]`, archivos de test, y doc-comments.
>
> **Cobertura:** `src/` (core), `vantadb-python/src/`, `vantadb-wasm/src/`, `vantadb-mcp/src/`,
> `vantadb-server/src/`, más integraciones FFI (openai, crewai, litellm, haystack, mem0, dspy).

---

## Summary

| Categoría | Cantidad |
|-----------|----------|
| Bloques `unsafe` (producción) | 42 |
| Llamadas `.unwrap()` (producción) | 25 |
| Llamadas `.expect()` (producción) | 22 |
| **🔴 Risk** | **3** |
| **🟡 Debt** | **9** |
| **🟢 Acceptable** | **77** |

### Distribución por crate

| Crate | unsafe | .unwrap() | .expect() |
|-------|--------|-----------|-----------|
| `src/` (core) | 42 | 5 | 20 |
| `vantadb-openai/` | 0 | 2 | 0 |
| `vantadb-crewai/` | 0 | 3 | 0 |
| `vantadb-litellm/` | 0 | 2 | 0 |
| `vantadb-haystack/` | 0 | 4 | 0 |
| `vantadb-mem0/` | 0 | 9 | 0 |
| `vantadb-dspy/` | 0 | 2 | 0 |
| `vantadb-wasm/` | 0 | 0 | 1 |
| `vantadb-mcp/` | 0 | 0 | 0 |
| `vantadb-server/` | 0 | 0 | 0 |
| `vantadb-python/` | 0 | 0 | 0 |

---

## 🔴 Risk

| File | Line | Pattern | Module | Impact | Suggestion |
|------|------|---------|--------|--------|------------|
| `src/error.rs` | 525 | `.unwrap()` | `Display` impl | `std::error::Error::source()` retorna `Option`. Si un error sin source se renderiza, paniquea el hilo. En producción esto derriba el request. | Reemplazar con `if let Some(source) = e.source()` |
| `src/error.rs` | 535 | `.unwrap()` | `Display` impl | Idem línea 525 — segundo punto de fallo idéntico en el mismo flujo. | Ídem |
| `vantadb-mem0/src/python.rs` | 144 | `.write().unwrap()` | PyO3 FFI | Llamada bloqueante con `write().unwrap()` dentro de contexto FFI (PyO3). Si otro hilo paniquea teniendo el lock, envenena el RwLock y mata el proceso entero. | Usar `lock_rwlock_mut()` de `sync_ext.rs` (idem poisioning, pero al menos centralizado); o mejor: catch poison con `.unwrap_or_else(PoisonError::into_inner)` |

---

## 🟡 Debt

| File | Line | Pattern | Module | Impact | Suggestion |
|------|------|---------|--------|--------|------------|
| `src/sync_ext.rs` | 10 | `.expect("RwLock poisoned")` | Lock wrappers | Convenience trait usado en toda la codebase. Panic en lugar de recovery. Si un RwLock se envenena, toda operación que use estos traits paniquea. | Cambiar a `.unwrap_or_else(PoisonError::into_inner)` para recuperar el lock (el dato envenenado sigue siendo utilizable) |
| `src/sync_ext.rs` | 13 | `.expect("RwLock poisoned")` | Lock wrappers | Ídem línea 10 | Ídem |
| `src/sync_ext.rs` | 23 | `.expect("Mutex poisoned")` | Lock wrappers | Ídem | Ídem |
| `src/node.rs` | 295 | `unsafe { from_raw_parts }` | `VectorRepresentations::as_f32_slice()` | API pública que expone una referencia a memoria mappeada. Si el mmap se redimensiona concurrentemente mientras un caller retiene el slice, es UAF. `Arc` mantiene vivo el mmap pero no previene resize. | Documentar safety contract: "caller must not hold the returned slice across a resize". O retornar `Cow<[f32]>` con `.to_vec()`. |
| `src/index/graph.rs` | 69 | `pub unsafe fn release_mmap_vector` | Index mmap ops | Función `pub unsafe` con `#[allow(unused_variables)]` en Windows (no-op). El caller debe verificar invariantes de offset+len contra el mmap actual; no hay validación interna. | Agregar `debug_assert!` de bounds, o convertir a `pub(crate)` si no se necesita desde fuera del crate. |
| `src/storage/engine/maintenance.rs` | 262 | `unsafe { release_mmap_vector(...) }` | `consolidate_node` | Llama `release_mmap_vector` con offset+len derivados de un nodo. Si el mmap se redimensionó entre la lectura del offset y la llamada, es page fault. | Elevar la validación de bounds antes de la unsafe call (actualmente solo hay `if offset_usize + vector_size_aligned <= mmap.len()` que es correcto, pero la unsafe call está en un bloque separado) |
| `vantadb-openai/src/python.rs` | 115 | `.read().unwrap()` | PyO3 FFI | Cada llamada paniquea si el lock está envenenado. En un server long-running, un panic en FFI es irrecuperable. | Usar `lock_rwlock()` de `sync_ext.rs` o manejar poison. |
| `vantadb-openai/src/python.rs` | 148 | `.read().unwrap()` | PyO3 FFI | Ídem línea 115 | Ídem |
| *Todas las integraciones FFI* | *var* | `.read().unwrap()` / `.write().unwrap()` | PyO3 wrappers | **11 occurrences** en openai/crewai/litellm/haystack/mem0/dspy — mismo patrón. | Centralizar via `sync_ext.rs` traits y considerar poison recovery. |

---

## 🟢 Acceptable

### `unsafe` — Storage / VFile layer (`src/storage/vfile.rs`)

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/storage/vfile.rs` | 70 | `pub unsafe fn map()` | Mmap shim | API parity con memmap2; implementación segura (solo lee el archivo en un Vec). SAFETY comment completo. |
| `src/storage/vfile.rs` | 72 | `unsafe { MmapOptions::new().map(file) }` | Mmap shim | Wrapper interno, delegado a memmap2. |
| `src/storage/vfile.rs` | 109 | `pub unsafe fn map_mut()` | MmapMut shim | Idem línea 70. |
| `src/storage/vfile.rs` | 111 | `unsafe { MmapOptions::new().map_mut(file) }` | MmapMut shim | Idem línea 72. |
| `src/storage/vfile.rs` | 183 | `unsafe { ... sigaction ... }` | SIGBUS handler | `Once`-protected, solo async-signal-safe functions. |
| `src/storage/vfile.rs` | 205 | `unsafe extern "C" fn sigbus_handler` | SIGBUS handler | Signal handler, solo atomic stores. |
| `src/storage/vfile.rs` | 214 | `unsafe { (*siginfo).si_addr() }` | SIGBUS handler | Non-null verificado antes del acceso. |
| `src/storage/vfile.rs` | 234 | `unsafe { libc::sysconf(...) }` | `get_resident_bytes` | POSIX FFI estándar, async-signal-safe. |
| `src/storage/vfile.rs` | 256 | `unsafe { libc::mincore(...) }` | `get_resident_bytes` | POSIX FFI estándar. |
| `src/storage/vfile.rs` | 268 | `unsafe { libc::mincore(...) }` | `get_resident_bytes` | POSIX FFI estándar. |
| `src/storage/vfile.rs` | 296 | `unsafe { GetCurrentProcess() }` | `get_resident_bytes` (Windows) | Windows FFI estándar, pseudo-handle siempre válido. |
| `src/storage/vfile.rs` | 300 | `unsafe { zeroed() }` | `get_resident_bytes` (Windows) | Zeroed POD struct. |
| `src/storage/vfile.rs` | 315 | `unsafe { QueryWorkingSetEx(...) }` | `get_resident_bytes` (Windows) | Windows FFI estándar con return-code check. |
| `src/storage/vfile.rs` | 320 | `unsafe { entry.VirtualAttributes.Flags }` | `get_resident_bytes` (Windows) | Campo de struct POD. |
| `src/storage/vfile.rs` | 422 | `unsafe impl Send for VantaFile` | Send/Sync | Todos los campos son Send; mmap manejado por memmap2. SAFETY comment completo. |
| `src/storage/vfile.rs` | 425 | `unsafe impl Sync for VantaFile` | Send/Sync | Acceso serializado via `RwLock<VantaFile>` en engine. SAFETY comment completo. |
| `src/storage/vfile.rs` | 488 | `unsafe { MmapOptions::new().map(&file) }` | `open_with_mode` | File handle válido, tamaño verificado. |
| `src/storage/vfile.rs` | 492 | `unsafe { MmapOptions::new().map_mut(&file) }` | `open_with_mode` | Idem línea 488. |
| `src/storage/vfile.rs` | 574 | `unsafe { MmapOptions::new().map_mut(&file) }` | `resize` | File handle reabierto, size actualizado via `set_len()`. |

### `unsafe` — Index layer (`src/index/`)

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/index/graph.rs` | 31 | `unsafe { libc::madvise(...) }` | `prefetch_mmap_vector` | Async-signal-safe, offset+len de mmap owned. |
| `src/index/graph.rs` | 47 | `unsafe { PrefetchVirtualMemory(...) }` | `prefetch_mmap_vector` | Windows FFI, pseudo-handle. |
| `src/index/graph.rs` | 75 | `unsafe { libc::madvise(...) }` | `release_mmap_vector` | Signal-safe; caller garantiza invariantes via contract. |
| `src/index/graph.rs` | 184 | `unsafe { Mmap::map(&file) }` | `mmap_resident_bytes` | Error manejado via `match`; falla silenciosamente. |
| `src/index/distance.rs` | 490 | `unsafe { from_raw_parts(...) }` | Similarity hot path | len acotado por `MAX_VEC_F32_LEN`; mmap vivo via Arc. |
| `src/index/search.rs` | 47 | `unsafe { from_raw_parts(...) }` | Search hot path | Bounds-checked + alignment assertion. |
| `src/index/search.rs` | 171 | `unsafe { from_raw_parts(...) }` | Search hot path | Bounds-checked + alignment assertion. |
| `src/index/serialize.rs` | 128 | `unsafe { from_raw_parts(...) }` | Serialization | Bounds-checked contra `MAX_VEC_F32_LEN`. |
| `src/index/serialize.rs` | 519 | `unsafe { MmapMut::map_mut(&file) }` | Serialization I/O | File handle válido. |
| `src/index/serialize.rs` | 589 | `unsafe { MmapMut::map_mut(&file) }` | Serialization I/O | File handle válido. |
| `src/index/serialize.rs` | 604 | `unsafe { MmapMut::map_mut(&file) }` | Serialization I/O | File handle válido. |

### `unsafe` — Storage engine operations (`src/storage/`)

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/storage/archive.rs` | 74 | `unsafe { MmapMut::map_mut(&tmp_file) }` | Archive compaction | File handle creado con `set_len()`, descartado antes del rename. |
| `src/storage/archive.rs` | 105 | `unsafe { MmapMut::map_mut(&tmp_file) }` | Archive compaction | Remap tras `set_len()`, old mmap dropped. |
| `src/storage/archive.rs` | 217 | `unsafe { from_raw_parts(...) }` | Rebuild HNSW | Bounds-checked + alignment assertion + `.to_vec()`. |
| `src/storage/engine/maintenance.rs` | 141 | `unsafe { MmapMut::map_mut(&file) }` | Index persist | File temporal, `set_len()` antes, `copy_from_slice` después. |
| `src/storage/engine/ops.rs` | 586 | `unsafe { from_raw_parts(...) }` | `get()` | Alineación verificada via `debug_assert_eq!`, `.to_vec()` elimina aliasing. |
| `src/storage/engine/ops.rs` | 711 | `unsafe { from_raw_parts(...) }` | `get_many()` | Ídem línea 586. |
| `src/storage/engine/ops.rs` | 1061 | `unsafe { from_raw_parts(...) }` | `scan_all_nodes()` | Ídem línea 586. |

### `unsafe` — Metrics (`src/metrics/`)

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/metrics/core/mod.rs` | 279 | `unsafe { ... Mach FFI ... }` | macOS memory | POD zeroed, return-code check. |
| `src/metrics/core/mod.rs` | 304 | `unsafe { ... Windows FFI ... }` | Windows memory | POD zeroed, return-code check. |

### `.unwrap()` — Core production

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/cli_server.rs` | 169 | `NonZero::new(1000).unwrap()` | Auth rate limiter | Constante 1000 > 0, panic solo si el universo cambia las matemáticas. |
| `src/binary_header.rs` | 108 | `.unwrap()` | Test | Dentro de `mod tests` (línea 100). |
| `src/binary_header.rs` | 157 | `.expect("system time is after unix epoch")` | Test | Dentro de `mod tests` (línea 152). |

### `.expect()` — Core production

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/cli_server.rs` | 139 | `.expect("GovernorConfig build failed")` | HTTP server startup | Startup-only; config build con parámetros fijos. |
| `src/binary_header.rs` | 67 | `.expect("header bytes slice fits u64")` | `VantaHeader::deserialize` | `try_into()` en `[u8; 8]` es infalible. |
| `src/crypto.rs` | 104 | `.expect("Aes256Gcm::new_from_slice failed...")` | Cipher init | SHA-256 garantiza 32 bytes. Mensaje documenta el invariante. |
| `src/crypto.rs` | 141 | `.expect("AES-256-GCM encryption is infallible...")` | Encryption | RustCrypto garantiza que `encrypt()` con nonce + AAD no falla. |
| `src/index/distance.rs` | 97,100,129,132,204,207,238,241,268,271,297,300,379,420 | `.expect("chunks_exact(X) yields X-element chunks")` | SIMD kernels | 14 calls. `chunks_exact()` garantiza el tamaño del chunk. Hot path, el overhead de `try_into().unwrap()` es despreciable pero evitar `?` es intencional. |
| `src/index/serialize.rs` | 22 | `.expect("Vec::write cannot fail")` | Serialization | `write()` en `Vec<u8>` nunca falla. |
| `src/storage/engine/ops.rs` | 1020 | `.expect("key slice fits [u8; 16]")` | Backend ops | `try_into()` en `[u8; 16]` — infalible por construcción. |

### `.expect()` — CLI helpers (startup-only)

| File | Line | Pattern | Module | Justification |
|------|------|---------|--------|--------------|
| `src/cli_handlers/data.rs` | 63 | `.expect("valid spinner template")` | CLI handler | Template string constante. |
| `src/cli_handlers/fmt.rs` | 17 | `.expect("valid spinner template")` | CLI handler | Template string constante. |
| `src/bin/crash_helper.rs` | 16 | `.expect("Invalid count")` | CLI binary | Herramienta de crash-test; panic es el comportamiento deseado. |
| `src/bin/crash_helper.rs` | 25 | `.expect("Failed to open StorageEngine")` | CLI binary | Startup; panic en error de apertura es aceptable. |

### `.unwrap()` — Test code (todos excluidos del inventario de riesgo)

TODO: Los siguientes archivos contienen `.unwrap()` exclusivamente dentro de `mod tests { }` o `#[cfg(test)]`. No se listan individualmente:

- `src/engine.rs` (~28 unwraps, mod tests en línea 460)
- `src/executor.rs` (~15 unwraps, mod tests en línea 438)
- `src/gc.rs` (~12 unwraps, mod tests en línea 94)
- `src/graph.rs` (~17 unwraps, mod tests en línea 221)
- `src/wal.rs` (~18 unwraps, mod tests en línea 652)
- `src/wal_shipping.rs` (~7 unwraps, mod tests en línea 253)
- `src/migration.rs` (~3 unwraps, mod tests en línea 397)
- `src/governor.rs` (~3 unwraps, mod tests en línea 91)
- `src/governance/consistency.rs` (~7 unwraps, mod tests en línea 261)
- `src/governance/worker.rs` (~1 unwrap, mod tests en línea 209)
- `src/schema.rs` (~4 unwraps, mod tests en línea 198)
- `src/query.rs` (~3 unwraps, mod tests en línea 295)
- `src/index/core.rs` (~8 unwraps/expects, archivo entero `#[cfg(test)]` línea 1)
- `src/backends/fjall_backend.rs` (~20 unwraps, mod tests en línea 264)
- `src/backends/in_memory.rs` (~10 unwraps, mod tests en línea 160)
- `src/backends/rocksdb_backend.rs` (~20 unwraps, mod tests en línea 352)
- `src/storage/engine/tests.rs` (archivo de test completo)
- `src/vector/governor.rs` (~2 unwraps, mod tests en línea 193)
- `src/text_index.rs` (~2 expects, mod tests en línea 698)
- `vantadb-wasm/src/lib.rs` (~25 unwraps, test module after `#[cfg(test)]`)

---

## Recommendations

### Inmediatas (ordenadas por impacto)

1. **Fix 🔴 Risk en `src/error.rs:525,535`** — Reemplazar `e.source().unwrap()` con `if let Some(source) = e.source()`. 2 líneas, elimina 2 puntos de panic en producción. Esto debería ser blocker para el próximo PR que toque error.rs.

2. **Fix 🔴 Risk en `vantadb-mem0/src/python.rs:144`** — El `write().unwrap()` en contexto PyO3 es un panic silencioso desde FFI. Mover a pattern de recovery.

3. **Mitigar 🟡 Debt de `sync_ext.rs`** — Cambiar `.expect("... poisoned")` por `self.read().unwrap_or_else(PoisonError::into_inner)`. Esto mantiene el comportamiento actual (seguir funcionando) pero sin panic. Afecta a **cada** lock en la aplicación (~15 call sites).

### Proactivas

4. **Miri tests para nuevas `unsafe` blocks** — Agregar `cargo miri test` al CI gate para PRs que introduzcan `unsafe` nuevo. Usar `MIRIFLAGS=-Zmiri-tree-borrows`.

5. **Centralizar lock pattern en integraciones FFI** — Los 11 `.read().unwrap()` en los crates de integración (openai, crewai, etc.) copian el mismo patrón propenso a poison. Refactorizar para usar un helper `lock_ns()` que maneje poison centralizadamente.

6. **`clippy::unwrap_used` lint** — Considerar habilitar `#[warn(clippy::unwrap_used)]` en producción para prevenir nuevos `.unwrap()` en código no-test.

7. **Revisar `pub unsafe fn release_mmap_vector`** — Actualmente `pub` con `#[allow(unused_variables)]` en Windows. Reducir visibilidad a `pub(crate)` si es posible.

### Endosos (correcto como está)

- Todo el código de mmap en `vfile.rs` está bien documentado con `// SAFETY:` invariantes completos.
- Los `unsafe` blocks en hot path de search/distance tienen bounds checks + alignment assertions.
- La instalación del SIGBUS handler es correcta (Once, signal-safe).
- Los `expect()` en distance SIMD kernels son correctos (chunks_exact garatiza tamaño).
- Los `expect()` en crypto son correctos (SHA-256 output + RustCrypto guarantees).

---

## Coverage Note

- **Pendiente:** Los crates de integración `vantadb-ollama`, `vantadb-langchain`, `vantadb-letta`, `vantadb-llamaindex` no se escanearon en profundidad (sin hallazgos en el `src/` revisado). Si estos crates contienen `unsafe` o unwraps, deben agregarse en una iteración futura.
- **`lib.rs`:** Los `unwrap()` en doc-comments (líneas 43,50,52,53,54) son ejemplos y no cuentan como producción.
- **`#![deny(unsafe_op_in_unsafe_fn)]`** y **`#![allow(unused_unsafe)]** están activos globalmente en `lib.rs`.
