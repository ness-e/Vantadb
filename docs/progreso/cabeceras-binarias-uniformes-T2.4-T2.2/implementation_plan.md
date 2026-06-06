# Plan de Implementación: Header de Serialización Binaria Uniforme (T2.4) e Integración de mimalloc (T2.2)

Este plan describe las refactorizaciones y adiciones necesarias para unificar el formato físico de persistencia en disco de VantaDB mediante una cabecera estándar de 16 bytes, y la integración controlada del asignador de memoria de alto rendimiento `mimalloc` en el CLI y el Core para mitigar la fragmentación.

## User Review Required

> [!IMPORTANT]
> - **Ruptura de Compatibilidad de Formato:** Modificar los metadatos iniciales del WAL y `VantaFile` romperá la compatibilidad con bases de datos antiguas de desarrollo local. El motor detectará la incompatibilidad del índice y el WAL de forma segura y lanzará un error controlado en caliente (`VantaError::IncompatibleFormat` o `WALVersionMismatch`), o descartará y reconstruirá el índice de forma automática desde el WAL según corresponda.
> - **Exclusión de mimalloc en Windows Python SDK:** Debido a conflictos de desasignación cruzada (crashes de segmentación en FFI) entre `mimalloc` interceptando el heap en Windows y el asignador interno de Python (`pymalloc`), **no** registraremos `mimalloc` como allocator global en `vantadb-python` para entornos Windows. Sí se habilitará en el CLI principal y en el servidor.

---

## Proposed Changes

### Componente: Core / Formato Binario Uniforme (T2.4)

#### [NEW] binary_header.rs
- Crear una estructura común `VantaHeader` para representar la cabecera unificada en disco de 16 bytes:
  - `magic`: `[u8; 4]` (Ej: `b"VWAL"`, `b"VNDX"`, `b"VFLE"`).
  - `format_version`: `u16` (Ej: `1`, `4`).
  - `schema_version`: `u16` (Inicialmente `0`).
  - `timestamp`: `u64` (Epoch en milisegundos de creación).
- Implementar serialización/deserialización determinista y validaciones seguras de firmas.

#### [MODIFY] error.rs
- Agregar la variante de error `IncompatibleFormat` para reportar de forma precisa inconsistencias de firma o versión entre archivos binarios y la versión esperada del software.

#### [MODIFY] lib.rs
- Registrar y exportar el nuevo módulo `binary_header`.

#### [MODIFY] index.rs
- Modificar `serialize_to_bytes` para escribir `VantaHeader::new(*b"VNDX", 4, 0).serialize()` al principio.
- Modificar `deserialize_from_bytes` para validar la firma `b"VNDX"` y la versión de la cabecera.

#### [MODIFY] wal.rs
- Modificar `WalHeader` para embeber `VantaHeader` (usando magic `b"VWAL"`, `format_version = 1`).
- Los primeros 16 bytes serán la cabecera unificada, seguidos por `crc` (u32, 4 bytes), total 20 bytes.

#### [MODIFY] storage.rs
- Refactorizar `VantaFile` para reservar los primeros 16 bytes para `VantaHeader` (magic `b"VFLE"`, `version = 1`).
- Los siguientes 8 bytes (16..24) guardarán el `write_cursor` como `u64`.
- Los bytes de 24..64 actuarán como padding para preservar la alineación física a 64 bytes.

---

### Componente: Memoria / Integración de mimalloc (T2.2)

#### [MODIFY] vanta-cli.rs
- Habilitar `#[global_allocator]` condicionalmente tras el feature flag `custom-allocator`.

#### [MODIFY] Cargo.toml
- Asegurar que `custom-allocator` y `mimalloc` estén activos por defecto en perfiles release.

---

## Verification Plan

1. `cargo test --workspace --release`
2. `cargo test --test version_coherence`
3. `cargo test --package vantadb mmap_vector_index_certification -- --nocapture`
