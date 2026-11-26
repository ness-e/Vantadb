---
title: Impedir la escritura de encryption.registry en aperturas read_only (ADR-0001 Read-Only Barrier)
kind: task
description: "Una apertura readonly NUNCA crea ni modifica encryption.registry. Si existe, lo lee"
id: FIND-508
state: completed
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 12)
depends: [FIND-585]
---

## Objetivo

Una apertura `read_only` NUNCA crea ni modifica `encryption.registry`. Si existe, lo lee;
si no existe, no lo crea y devuelve el error correcto (ADR-0057 D2: EncryptionKeyRequired
cuando la base es cifrada; NotFound cuando el dir/base no existe).

## Causa raíz (discovery)

- `init.rs:133` `wal_context` → `WalCrypto::open` → `begin_epoch` → `persist()` en read_only.
- `init.rs:1153-1173` `after_open` → `open_or_create` + `ensure_dek`×4 → `persist()` si falta entry.
- `open_or_create` se usa en read paths (`BackendKvCrypto::open_read`, `VFileCrypto::open_read`,
  `HnswImageCrypto::open_read`, `WalKey::open`): TOCTOU/exists-edge → crea y persiste.

## Solución

- Nuevo `KeyRegistry::open_existing` (never-create/never-persist; `RegistryIo` si ausente).
- `wal_context` devuelve `Ok(None)` en `read_only` (init_wal ya retorna None).
- `after_open` en `read_only`: `open_existing` + NO `ensure_dek` (encrypted = registry.artifacts()).
- `open_read` de backend/vfile/hnsw y `WalKey::open` pasan a `open_existing`.

## Impacto mapeado (Regla 0)

- Leídos completos: src/crypto/registry.rs, src/backends/kv_crypto.rs (crypto ctx),
  src/storage/engine/init.rs (DbEncryption), src/wal.rs (WalKey/WalCrypto open),
  src/storage/vfile_crypto.rs, src/index/serialize/crypto.rs, ADR-0001.
- Entrantes: tests encryption.rs, cli rotation (FIND-587, no tocado).
- Veredicto: blast radius = aperturas cifradas; read_only readers (MCP-35) no deben romperse.

## Casos (pre-mortem)

1. read_only + base cifrada existente → abre registry en lectura, sin escrituras.
2. read_only + dir inexistente → NotFound (init.rs:255/391 ya lo cubre).
3. read_only + cifrada + registry ausente → EncryptionKeyRequired (guard FIND-585).
4. read_only + plaintext + clave configurada → plaintext, sin registry.

## Contrato

cargo nextest run --profile audit -p vantadb --features encryption -E "test(read_only) or test(registry)" --locked --build-jobs 2

SDP: security-and-hardening, test-driven-development, systematic-debugging
