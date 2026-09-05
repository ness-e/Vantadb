# COMP-009 — Binary Bulk Import

> **Estado:** ✅ COMPLETED — 2026-07-27
> **Resultado:** `bulk_import_stream()` + `bulk_import_file()` (src/sdk/api.rs:964/1053), `bulk_commit_interval` en config. Python `bulk_import()`/`bulk_import_bytes()`, WASM bindings. Formato `VDBJSON\n` + serde_json. 3 tests.
> **Sub-agente:** `vanta-worker`
> **Esfuerzo:** 🟢 3-4 días
> **Workflow:** feature-add

## Objetivo

Implementar un protocolo binario de importación masiva (`.vdbdump`) que sea **5-10x más rápido** que `put_batch()` actual. Hoy `put_batch()` itera cada registro con validación individual + `put_one()` por registro — sin aprovechar batch commits ni streaming binario.

### Lo que existe hoy

| Componente | Archivo | Notas |
|---|---|---|
| `put_batch()` core | `src/sdk/api.rs:161` | Valida cada input individual, `chunks().map(put_one)` con rayon opcional |
| `put_batch()` Python | `vantadb-python/src/lib.rs:150` | Keyword API + backward compat con tuplas, pasa a engine.put_batch |
| `put_batch_raw()` Python | `vantadb-python/src/lib.rs:306` | Zero-copy NumPy paths, pero sigue validando y llamando put_one |
| `put_batch()` WASM | `vantadb-wasm/src/lib.rs:528` | JsValue → runtime parse + engine.put_batch |
| rkyv serialization | interno | Ya se usa internamente para serializar nodos/edges |
| `export_namespace` | `src/sdk/api.rs` | Exporta namespace a JSON — reversa del bulk import |

### Lo que hay que construir

#### 1. Formato binario `.vdbdump` (rkyv-based)

```
┌─────────────────────────────┐
│ Magic: "VDBDUMP\n" (8 bytes)│
│ Version: u8 (0x01)          │
│ Flags: u8 (bitmask)         │
│   bit 0: has_checksum       │
│   bit 1: compressed (zstd)  │
│ Record count: u64 (0 = open)│
│ Schema descriptor: varint   │
├─────────────────────────────┤
│ Record batch 1 (rkyv)       │
│ Record batch 2 (rkyv)       │
│ ...                         │
├─────────────────────────────┤
│ Checksum: xxhash3 (8 bytes) │
└─────────────────────────────┘
```

**Decisión de diseño:** Usar rkyv (ya disponible en el tree) como serialización binaria. Cada batch de registros se serializa como `Vec<VantaMemoryInput>` en rkyv. Esto permite:
- Zero-copy deserialization (rkyv es zero-copy)
- Aprovechar el schema existente de `VantaMemoryInput`
- Compatibilidad automática con versiones futuras

**Schema descriptor:** Lista de campos incluidos (namespace, key, payload, metadata, vector, ttl) como bitset — permite omitir campos opcionales.

#### 2. `VantaDB::bulk_import_stream()` — Core Rust

```rust
/// Bulk-import records from a binary stream.
/// ⚠️ Bypasses per-record validation — valida schema una vez al inicio.
/// ⚠️ No trigger WAL compaction ni index rebuild hasta que termine el batch.
pub fn bulk_import_stream<R: Read>(&self, reader: R) -> Result<BulkImportReport> { ... }
```

**Optimizaciones clave:**
- **Bypass validación por registro:** Valida schema una vez, no cada input
- **Batch commit atómico:** Commit WAL cada N registros (configurable via `config.bulk_commit_interval`)
- **Skip index rebuild:** Reconstruye HNSW al final del import, no por registro
- **Progress callback:** Closure opcional `|progress: BulkProgress|` para reportar avance

```rust
pub struct BulkImportReport {
    pub total_records: usize,
    pub batches_committed: usize,
    pub errors: Vec<BulkImportError>,
    pub duration_ms: u64,
}

pub struct BulkProgress {
    pub records_processed: usize,
    pub total_estimates: Option<usize>,
}
```

#### 3. `VantaDB::bulk_import_file()` — Conveniencia desde archivo

```rust
/// Bulk-import desde archivo .vdbdump
pub fn bulk_import_file(&self, path: &str) -> Result<BulkImportReport> {
    let file = std::fs::File::open(path)?;
    self.bulk_import_stream(file)
}
```

#### 4. Export complementario `VantaDB::export_bulk()` 

```rust
/// Exporta namespace/s completos a formato .vdbdump binario
pub fn export_bulk(&self, path: &str, namespaces: Option<&[&str]>) -> Result<BulkExportReport> { ... }
```

#### 5. Python Binding

Agregar en `vantadb-python/src/lib.rs`:

```python
def bulk_import(self, path: str) -> dict:
    """Importa archivo .vdbdump. Returns report dict."""

def bulk_import_bytes(self, data: bytes) -> dict:
    """Importa desde bytes en memoria."""

def export_bulk(self, path: str, namespaces: list[str] | None = None) -> dict:
    """Exporta a .vdbdump binario."""
```

#### 6. WASM Binding

En `vantadb-wasm/src/lib.rs`:

```rust
#[wasm_bindgen]
pub fn bulk_import(&self, data: &[u8]) -> Result<JsValue, JsValue> { ... }

#[wasm_bindgen]
pub fn export_bulk(&self, namespaces: Option<Vec<String>>) -> Result<Vec<u8>, JsValue> { ... }
```

#### 7. Benchmarks

```rust
// benches/bulk_import.rs
// put_batch(1K) vs bulk_import(1K)
// put_batch(10K) vs bulk_import(10K)
// put_batch(100K) vs bulk_import(100K)
// Medir: throughput (records/seg), latency p50/p99, memory peak
```

### Archivos a modificar

| Archivo | Cambio |
|---|---|
| `src/sdk/api.rs` | `bulk_import_stream()`, `bulk_import_file()`, `export_bulk()` |
| `src/config.rs` | Opcional: `bulk_commit_interval` en config |
| `src/lib.rs` | Re-exportar nuevos métodos en `VantaDB` impl |
| `vantadb-python/src/lib.rs` | `bulk_import()`, `bulk_import_bytes()`, `export_bulk()` PyO3 bindings |
| `vantadb-wasm/src/lib.rs` | `bulk_import()`, `export_bulk()` wasm_bindgen |
| `Cargo.toml` | Opcional: `xxhash-rust` si se usa checksum |
| `benches/bulk_import.rs` | Benchmark suite |
| `tests/` | Tests de integración |

### Dependencias externas

- **rkyv**: Ya en el workspace — usar `rkyv::Archive` + `rkyv::Serialize` + `rkyv::Deserialize`
- **xxhash-rust**: (opcional) checksum rápido — `xxhash-rust` es pure Rust, zero deps
- **zstd**: (opcional) compresión — solo si el flag `compressed` está activo

### Criterios de éxito

1. `cargo check -p vantadb` ✅
2. `cargo nextest run --profile audit --workspace` ✅ (tests existentes no rotos)
3. Bulk import 5-10x más rápido que `put_batch()` equivalente
4. Archivos .vdbdump reproducibles (mismo input → mismo hash)
5. Round-trip: export → import produce mismos registros que original
6. Python `bulk_import()` devuelve report con métricas
7. WASM `bulk_import()` acepta Uint8Array y devuelve report

### Estrategia de implementación (Ponytail)

1. **YAGNI primero**: No implementar compresión, checksum, ni progress callbacks en v1. Solo:
   - Formato binario plano (rkyv `Vec<VantaMemoryInput>` serializado directo)
   - Header mínimo con magic + version + record_count
   - `bulk_import_stream()` con batch commit
   - Bypass de validación por registro (validar solo schema)
   - Python + WASM bindings
   - Tests + benchmark
2. **Ponytail:** Skip progress callback (el caller espera sync). Skip export complementario (hacerlo si sobra tiempo). Skip compression.
3. **Si el benchmark no muestra 5x+ mejora:** Revisar bottleneck — probablemente sea WAL fsync por batch. Ajustar `bulk_commit_interval` hasta lograr 5-10x.

### Verificación

```bash
cargo check -p vantadb
cargo nextest run --profile audit --workspace
cargo bench -p vantadb --bench bulk_import  # si se implementó
```

### Referencias

- `src/sdk/api.rs` — `put_batch()` actual (línea 161)
- `vantadb-python/src/lib.rs` — `put_batch()` + `put_batch_raw()` patrones
- `vantadb-wasm/src/lib.rs` — `put_batch()` patrón WASM
- `src/storage/` — StorageBackend trait, put_one internals
- Formato inspirado en `pg_bulkload` / `COPY binary` de PostgreSQL
