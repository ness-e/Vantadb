# Tarea T3.1: Chaos Testing Expandido y Validación de Durabilidad - Checklist

- `[x]` Instrumentar nuevos failpoints en el motor
    - `[x]` Instrumentar `mmap_flush_fail` en `VantaFile::flush` en `src/storage.rs`
    - `[x]` Instrumentar `hnsw_serialize_fail` en `CPIndex::persist_to_file` en `src/index.rs`
- `[x]` Expandir suite de pruebas de caos en `tests/storage/chaos_integrity.rs`
    - `[x]` Crear test de certificación para `storage_insert_fail` (`chaos_integrity_storage_failpoint_certification`)
    - `[x]` Crear test de certificación para `mmap_flush_fail`
    - `[x]` Crear test de certificación para `hnsw_serialize_fail`
- `[x]` Crear script de loop de caos `dev-tools/chaos_loop.ps1`
- `[x]` Añadir el perfil `chaos` en `.config/nextest.toml`
- `[x]` Expandir y formalizar `docs/operations/RELIABILITY_GATE.md`
- `[x]` Enlazar `RELIABILITY_GATE.md` desde `README.MD`
- `[x]` Ejecución manual y certificación (por parte del usuario)
- `[x]` Crear walkthrough y hacer snapshot histórico en `docs/progreso/chaos-testing-T3.1/`
