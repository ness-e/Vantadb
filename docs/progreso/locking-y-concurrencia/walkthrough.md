# Walkthrough: Implementación de Bloqueo Shared/Exclusive y Mitigaciones de Concurrencia

## Objetivo

Introducir un mecanismo de bloqueo cooperativo a nivel de sistema de archivos (advisory lock) en VantaDB para prevenir corrupción de datos y crashes de segmentación (`SIGBUS`) en escenarios multi-proceso, con mitigaciones avanzadas para mantener la concurrencia de lectura.

## Cambios Realizados

### 1. Core Error Definition

#### [error.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/error.rs)
- Añadida la variante `DatabaseBusy(String)` al enum `VantaError` para representar errores de bloqueo ocupado con mensajes descriptivos.

---

### 2. File Locking con Backoff Exponencial

#### [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
- Modificado `StorageEngine::open` para implementar locking Shared/Exclusive:
  - **Lectores** (`read_only=true`): adquieren `try_lock_shared()` — múltiples lectores simultáneos permitidos.
  - **Escritores** (`read_only=false`): adquieren `try_lock_exclusive()` — un solo escritor a la vez.
  - **Backoff exponencial** estilo SQLite: intervalos de 5ms → 10ms → 20ms → 50ms → 100ms, timeout total de 1000ms.
  - Si el timeout expira, retorna `VantaError::DatabaseBusy` con mensaje descriptivo.
  - Si `.vanta.lock` no existe en modo lectura, retorna error claro indicando que la DB no está inicializada.

---

### 3. Rename Atómico en Reconstrucción de Índices

#### [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs) — `rebuild_vector_index`
- El índice HNSW reconstruido se escribe en `vector_index.bin.tmp`.
- Al finalizar, se realiza un swap atómico (`std::fs::rename`) de `.tmp` → `.bin`.
- Los lectores en curso siguen leyendo la versión mapeada originalmente sin crashes.

#### [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs) — `sync_to_mmap`
- Implementado double-buffering con rename atómico para las sincronizaciones de índices mmap.

---

### 4. Bug Fix: SDK Ignoraba Backend Seleccionado

#### [sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs) — `open_with_config`
- **Bug encontrado:** La línea `final_config.backend_kind = BackendKind::Fjall` sobreescribía forzosamente el backend a Fjall, ignorando cualquier selección del usuario.
- **Fix:** Eliminada la sobreescritura. El `backend_kind` del config ahora se respeta tal como lo proporciona el caller.
- **Impacto:** El comportamiento default no se alteró porque `VantaConfig::default()` ya usa Fjall como fallback.

#### [lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs) — Bindings Python
- Corregidas rutas de importación: `vantadb::backend::BackendKind` → `vantadb::BackendKind` (el módulo `backend` es `pub(crate)`).
- El constructor Python `VantaDB(path, backend="rocksdb")` ahora funciona correctamente.

---

## Verificación

### Tests de Concurrencia Multi-Proceso (Python)

| Test | Resultado | Detalle |
|------|-----------|---------|
| **Lectores concurrentes** | ✅ PASS | Dos procesos con shared locks simultáneos |
| **Escritor bloqueado por lector** | ✅ PASS | `DatabaseBusy` tras ~1.06s timeout |
| **Escritor post-release** | ✅ PASS | Escritor abre tras liberar locks |
| **Lector bloqueado por escritor** | ✅ PASS | `DatabaseBusy` con mensaje descriptivo |

### Tests Unitarios de Rust
```
cargo test --test basic_node -j 1
test result: ok. 1 passed; 0 failed; 0 ignored
```

## Archivos Modificados

| Archivo | Tipo de Cambio |
|---------|---------------|
| `src/error.rs` | Variante `DatabaseBusy` |
| `src/storage.rs` | File locking + atomic rename |
| `src/index.rs` | Double-buffer mmap sync |
| `src/sdk.rs` | Bug fix backend override |
| `vantadb-python/src/lib.rs` | Import paths + backend param |
