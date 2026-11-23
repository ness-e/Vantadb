# FIND-591 — Eliminar casts desalineados en decodificación de vectores + Miri con `encryption`

- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-11-18
- **Commits:** `e806614d3`, `3f34c2325` (Miri CI con `encryption`), `c8b438f59` (rework: `vector_len == 0` fuera del rebuild HNSW)
- **Plan:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md` (Task 14)
- **SDP:** security-and-hardening, systematic-debugging, doubt-driven-development (del plan) + performance-optimization (hot path)

## Problema
- UB latente: `from_raw_parts(.. as *const f32)` sobre `node_bytes` cuya alineación solo está garantizada indirectamente (`AlignedBlob` Vec<u64> en cifrado, mmap en claro). 6 sitios.
- Pérdida silenciosa: `align_to::<u64>` descarta el vector si la base no está alineada. 4 sitios.
- Duplicación: `archive.rs` y `get.rs` inline vs `decode_full_bytes`/`decode_binary_bytes`.
- Miri CI sin `encryption`.

## Diseño
- Primitivas `pub(crate)` sin `unsafe` en `engine/get.rs`: `f32s_from_le_bytes`, `u64s_from_le_bytes` (chunks_exact + from_le_bytes).
- Endianness: LE. Escritor BINARY cifrado usa `to_le_bytes` explícito (`storage/ops.rs:261`); el crate solo compila en LE (`const _` assert en `index_port.rs:103`, `vfile.rs:1334`); `txn.rs` ya decodificaba con `from_le_bytes`.
- Reescribir decoders + sustituir TODOS los sitios inline conservando control de flujo.
- Tests `miri_vector_decode_*` (buffer desalineado, bounds, len 0) + `miri_aead_envelope_roundtrip` + `vector_decode_encrypted_roundtrip_*`.
- CI: `--features "cli,fjall,memmap2,fs2,encryption"` en ambos steps Miri.

## Evidencia (pre-implementación)
- Escritor FULL plaintext: `storage/ops.rs:117` `v.as_bytes()` (byte cast nativo); BINARY plaintext `ops.rs:122-124` cast nativo; BINARY cifrado `ops.rs:261` `to_le_bytes` explícito → formato efectivo LE; plataforma LE-only (`index_port.rs:103`, `vfile.rs:1334` const asserts).
- cpufeatures 0.3.0 soporta Miri nativamente: `cpufeatures-0.3.0/src/lib.rs:22` `#[cfg(miri)] mod miri;` → `miri.rs:8-19` devuelve `false` en toda detección → aes-0.9.3/polyval-0.7.3 (cpufeatures 0.3 dep) caen al backend soft. Sin RUSTFLAGS extra.
- Zero-copy fuera de alcance revisado: `index_port.rs:120` solo recibe `mmap_bytes()` (llamadores `index_port.rs:88`, `vfile.rs:1346` bajo `!is_encrypted()`); `vector_data.rs:176` opera sobre `MmapFull(Arc<Mmap>)` memmap2; `vfile.rs:1362` opera sobre `AlignedBlob` (base Vec<u64>, 8-alineada) + offset 64, SAFETY correcto; tests `storage/ops.rs:557,627` sobre `mmap_bytes()`. Ninguno recibe bytes descifrados de heap.

## Impacto mapeado (Regla 0)
- Leídos: get.rs, archive.rs:340-598, txn.rs:560-778, engine/ops.rs:260-397, ops.rs:60-330 (escritor), vfile.rs:107-1362 (NodeBytes/AlignedBlob/guarda), index_port.rs:90-150, vector_data.rs:150-210, ci-rust.yml:780-870, crypto.rs:640-870, vfile_crypto.rs:140-220, tests/encryption.rs (completo), plan Task 14.
- Referencias entrantes: `decode_vector_by_kind` la usarán también txn.rs/archive.rs/ops.rs; `rebuild_vector_index` (maintenance.rs:463) para test archive.
- Veredicto: cambio interno `pub(crate)`, sin API pública nueva; contrato mecánico del plan.

## Steps
- [x] DISCOVERY + evidencia endianness + cpufeatures
- [x] RED: tests `miri_vector_decode_*` fallan contra código actual
- [x] Primitivas + decoders en `src/storage/engine/get.rs:89-134` (`f32s_from_le_bytes`, `u64s_from_le_bytes`, `decode_full_bytes`, `decode_binary_bytes`)
- [x] Sustituir sitios inline (`get.rs` `get_many`, `archive.rs:376`, `txn.rs:645`, `ops.rs:349`)
- [x] Test cifrado roundtrip FULL/BINARY + rebuild (`src/storage/engine/tests/encryption.rs:509,538,567`; rework `vector_len == 0` fuera del rebuild HNSW en `c8b438f59`)
- [x] Test `miri_aead_envelope_roundtrip` en `src/crypto.rs:1628`
- [x] CI: +`encryption` en ambos steps Miri (`.github/workflows/ci-rust.yml:800-852`, `3f34c2325`)
- [x] Contrato + regresión + clippy + fmt + actionlint
- [x] Review P2-01 (vanta-audit, fresh) + commits locales (`e806614d3`, `3f34c2325`, `c8b438f59`)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-audit` (fresh context) + re-auditoría `audit-w1-storage` (#16) y `audit-w1-crypto` (#18).
- **Enfoque:** Correcto. Eliminados todos los casts desalineados (`as *const f32`, `align_to::<u64>`, `from_raw_parts`) en `archive.rs`, `txn.rs`, `get.rs`, `ops.rs` (`Select-String` = 0 hits) y centralizados en decoders seguros LE (`f32s_from_le_bytes`, `u64s_from_le_bytes`) en `src/storage/engine/get.rs:89-134`.
- **Cómo se probó:** Contrato mecánico nextest + tests `miri_vector_decode_*` con buffers desalineados (`get.rs:666-737`), `miri_aead_envelope_roundtrip` (`src/crypto.rs:1628`), roundtrip cifrado (`tests/encryption.rs:509,538,567`) y Miri CI con `--features "cli,fjall,memmap2,fs2,encryption"` (`.github/workflows/ci-rust.yml:831-852`).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve
