---
title: "FIND-618 - El descifrado deja plaintext en RAM que no se borra (zeroize de AlignedBlob y open_node)"
kind: task
status: completed
description: "El descifrado en vfile dejaba plaintext en el heap sin wiping al ser expulsado de DecryptCache o liberado de open_node. Se implementa Zeroize y Drop para AlignedBlob, se devuelve Zeroizing<Vec<u8>> en open_node, y se verifica contra SECURITY.md."
tags: [cifrado, seguridad, zeroize, vfile, heap-wiping, p2-01]
---

# FIND-618: Zeroize de buffers de plaintext en el path de descifrado

- **Origen:** `docs/dev/Backlog.md` (`FIND-618`) — detectado en la auditoría post-Paso 0 (`vanta-audit`, 2026-10-21).
- **Rama:** `develop` · **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Estado:** 🟢 **COMPLETED** — Implementado, verificado mecánicamente y revisado.

## Contexto y Causa Raíz

En el path de descifrado de `vfile` (introducido en el Paso 0 de FIND-598):
1. `VFileCrypto::open_node` devolvía un `Vec<u8>` plano sin protección. Al copiarse al `AlignedBlob`, ese `Vec<u8>` se liberaba sin wiping en el heap.
2. `AlignedBlob` contenía `Vec<u64>` con el plaintext descifrado. Al ser expulsado de `DecryptCache` (o al descartarse el último `Arc<AlignedBlob>`), su memoria se desasignaba sin sobreescribir con ceros.
3. Un volcado de memoria (core dump), swap/pagefile o reutilización posterior de esa memoria de heap por el proceso dejaba legible el plaintext de los vectores.

El repositorio ya provee el trait local `Zeroize` y el wrapper `Zeroizing<T>` en `src/crypto.rs` (cumpliendo ADR-0057 C-5).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/vfile.rs` (`decrypt_blob`, `legacy_read`), `src/storage/vfile_crypto.rs` (`open_node`) |
| Callees | `src/crypto.rs` (`Zeroize`, `Zeroizing`) |
| Implicaciones | Sin breaking change en APIs públicas. Sin impacto en el path de datos en claro (las estructuras están bajo `#[cfg(feature = "encryption")]`). |

## Contrato Verificable

1. `AlignedBlob` implementa `Zeroize` y `Drop`, limpiando `words` vía su vista en bytes con barrera `black_box`.
2. `VFileCrypto::open_node` devuelve `Result<Zeroizing<Vec<u8>>>`, asegurando que la salida intermedia del AEAD se borre al terminar su ámbito.
3. `SECURITY.md` documenta formalmente los límites del plaintext en memoria y el comportamiento de la caché.
4. Compilación limpia y sin advertencias en `cargo clippy -p vantadb --lib --features encryption -j 2 -- -D warnings` y en el perfil estándar sin features.

## Verificación

- Tests unitarios nuevos:
  - `vfile::tests::aligned_blob_zeroizes_plaintext` (ok)
  - `vfile_crypto::tests::vfile_v3_open_node_returns_zeroizing_plaintext` (ok)
- Nextest audit:
  - 49/49 tests pasaron en `test(storage::vfile) | test(storage::vfile_crypto)`.
- Comandos ejecutados:
  - `cargo fmt --check` (0 diffs)
  - `cargo clippy -p vantadb --lib --features encryption -j 2 -- -D warnings` (0 warnings)
  - `cargo clippy -p vantadb --lib -j 2 -- -D warnings` (0 warnings)

## Review (P2-01)

- **Reviewer:** Orquestador (sesión 87987f03)
- **Veredicto:** ✅ APROBADO (Contrato 100% cumplido y verificado mecánicamente)
- **Fecha:** 2026-10-23
