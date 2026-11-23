---
title: "FIND-585 — Unificar predicado \"base recién creada\" + fail-closed ante modo mixto o pérdida de `encryption.registry`"
kind: task
description: "Vulnerabilidad P0 de integridad/confidencialidad en cifrado at-rest (ADR-0057, feature v0.10.0)"
---

# FIND-585 — Unificar predicado "base recién creada" + fail-closed ante modo mixto o pérdida de `encryption.registry`

- **Plan:** docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
- **Prioridad:** 🔴 P0 · **Esfuerzo:** 🟡 4h · **Wave:** 1
- **Estado:** ⏳ IN PROGRESS
- **SDP:** security-and-hardening, systematic-debugging, test-driven-development, doubt-driven-development

## Spec

Vulnerabilidad P0 de integridad/confidencialidad en cifrado at-rest (ADR-0057, feature v0.10.0):

1. **Divergencia de predicados.** `init.rs:1054` decide "base nueva" con `is_new_database(root, data_dir)` (artefactos on-disk); `kv_crypto.rs:448-456` decide con `!registry.exists() + backend.is_empty()`. Si el engine decide "encrypt" pero el backend ya tiene datos, kv_crypto responde `warn!` + `Ok(None)` → **modo mixto mitad-cifrado** (WAL/vfile/HNSW cifrados, backend KV en claro).
2. **Fail-open ante pérdida de registry.** `init.rs:1124` llama `KeyRegistry::open_or_create` cuando `mode==Encrypted`; si `encryption.registry` fue borrado, crea un registry FRESCO (nuevo salt/DEKs) sobre ciphertext existente → lecturas corruptas posteriores.

## Decisión de diseño

- **Predicado autoritativo único:** `is_new_database` (artefactos) ∧ `backend.is_empty()` (datos). El engine registra su decisión (`register_process_key`); `kv_crypto` deja de juzgar "nuevo" por su cuenta — su `is_empty` pasa a ser **guardia de consistencia**, no segunda decisión.
- **Fail-closed:** `mode==Encrypted` + registry ausente → `Error::EncryptionCorrupt{artifact:"database"}` (code estable `VANTADB_CORRUPT`, sin nuevo variant → sin cambio de contrato wire). Engine decidió encrypt + backend no vacío + sin registry → `EncryptionCorrupt{artifact:"backend"}`.
- **D7 preservado:** base plaintext legada + clave configurada, sin decisión de engine → `warn!` + plaintext (test existente `plaintext_db_with_key_opens_plaintext_and_warns` no se toca).

## DoD

- [ ] Test (a): registry borrado en base poblada cifrada → `Err(EncryptionCorrupt)` y NO se re-inicializa registry.
- [ ] Test (b): decisión encrypt del engine + backend poblado + sin registry → `Err(EncryptionCorrupt)` (era `warn`+`Ok(None)`).
- [ ] Suite: `cargo nextest run --profile audit -p vantadb --features encryption -E "test(encryption)" --locked --build-jobs 2` ✅
- [ ] Review vanta-audit (superficie de seguridad) + P2-01.

## Impacto mapeado (Regla 0)

- `src/storage/engine/init.rs` (leído completo): `DbEncryption::resolve` (1050), `after_open` (1105), `detect_encryption_mode` (1251), `is_new_database` (1288). Callers: único `open_with_config`.
- `src/backends/kv_crypto.rs` (leído completo): `backend_kv_crypto_from_config` (429) ← `wrap_backend` (496) ← `backends/registry.rs:79` (único caller).
- `src/crypto.rs`: `register_process_key`/`process_decision` (383-404); nuevo `pub(crate) encryption_decision_for(root)`.
- `src/crypto/registry.rs`: `open_or_create` crea registry fresco si falta (409-433) — NO se toca (compartido con FIND-508).
- `src/error.rs`: `EncryptionCorrupt` existe (400), code `VANTADB_CORRUPT` — reusable, sin variant nueva.
- Tests: `src/storage/engine/tests/encryption.rs`, tests inline de `kv_crypto.rs`.
- Invariantes: sin `unsafe`; D7 (plaintext+key → warn+plaintext) intacto; InMemory no toca disco; Fjall y RocksDB comparten `wrap_with_crypto` → mismo comportamiento.

## Steps

- [x] Discovery + task file
- [ ] Tests RED (engine + kv_crypto)
- [ ] Implementación (init.rs guard, kv_crypto guard, crypto.rs helper)
- [ ] GREEN + suite completa
- [ ] Review vanta-audit + commit local
