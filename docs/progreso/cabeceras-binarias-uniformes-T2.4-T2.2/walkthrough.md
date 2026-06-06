# Walkthrough: Header de Serialización Binaria Uniforme (T2.4) e Integración de mimalloc (T2.2)

Se han completado e integrado con éxito las tareas **T2.4** y **T2.2** en el core y la infraestructura del workspace de VantaDB.

## Cambios Realizados

### Componente: Core / Formato Binario Uniforme (T2.4)
- **Módulo Genérico:** Se creó `src/binary_header.rs` que define `VantaHeader` (16 bytes). Valida que los metadatos de persistencia (`magic` (4B), `format_version` (u16), `schema_version` (u16) y `timestamp` (u64)) coincidan con el formato esperado.
- **Errores:** Se añadió el error `IncompatibleFormat` en `src/error.rs`.
- **HNSW Index:** Se modificó `src/index.rs` para utilizar la cabecera `VantaHeader` con la firma `b"VNDX"` y la versión `4`.
- **WAL:** Se modificó `src/wal.rs` para incorporar la cabecera `VantaHeader` con la firma `b"VWAL"` y la versión `1`. La suma CRC32C ahora valida los 16 bytes de la cabecera base.
- **VantaFile (Vector Store):** Se refactorizó `src/storage.rs` de modo que los primeros 16 bytes contengan `VantaHeader` (`b"VFLE"`), de 16..24 contengan `write_cursor`, y de 24..64 sirvan de relleno/alineación para mantener la coherencia física de 64 bytes para los descriptores de nodos.

### Componente: Asignador de Memoria Global (T2.2)
- **CLI Ejecutable:** Se añadió de forma condicional `#[global_allocator]` con `mimalloc` en `src/bin/vanta-cli.rs` bajo la feature-flag `custom-allocator`.
- **Exclusión FFI Windows:** De acuerdo al análisis FMEA, se evitó la definición del allocator global de mimalloc en los bindings de Python para evitar fallos de desasignación cruzada en Windows.

---

## Verificación y Resultados de Pruebas

### Test Unitario de Certificación Principal

```powershell
cargo test --package vantadb mmap_vector_index_certification -- --nocapture
```

**Resultado: ✅ PASS**

```
test mmap_vector_index_certification ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
```

Sub-tests certificados:
- ✅ Serialization: BFS layout preserves search results
- ✅ Serialization: Byte Roundtrip Integrity (VNDX magic + versión 4)
- ✅ Persistence: Cold-Start Performance
- ✅ MMap Governance: Backend Sync & Reload
- ✅ Error Handling: Corrupt/Nonexistent Fallback
- ✅ Abstraction: Memory vs MMap Equivalence

### Suite Completa
- **138 tests pasaron** en `cargo test --workspace --release`
- **18 tests Python SDK** pasaron via `pytest tests/test_sdk.py`

El código se encuentra integrado en `main` sin advertencias de linters de CI.
