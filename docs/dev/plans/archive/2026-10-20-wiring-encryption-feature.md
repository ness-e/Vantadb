---
title: "Plan de cableado de la feature `encryption`: cifrado en reposo en los 4 artefactos on-disk"
kind: plan
status: completed
date: "2026-10-20"
owners: ["vanta-arch (diseÃ±o)", "vanta-worker (implementaciÃ³n)", "vanta-audit (unsafe/FFI)", "vanta-chaos (recuperaciÃ³n)"]
tasks: ["FIND-249", "FIND-407", "FIND-194", "FIND-302"]
adr: "docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md"
---

> **Estado (completed â€” 2026-10-23):** W0â€“W6 completas. Los 4 artefactos on-disk cifran (WAL v4, VantaFile v3, backend-KV, imagen HNSW v9). FIND-249 y FIND-407 cerradas. W7 medida y publicada en BENCHMARKS.md Â§22/Â§22.4: gate del 15% SUPERADO (+43,2% en p99 post-Paso 0) â€” seÃ±al de diseÃ±o declarada (causa: granularidad del envelope del VantaFile por nodo; fix: envelope por pÃ¡gina + LRU, implica breaking change de `VectorStoreRef`, va en plan separado). Plan archivado por decisiÃ³n del owner. Tasks derivadas abiertas (FIND-496, FIND-499, FIND-513, FIND-587 ðŸ”´; FIND-497, FIND-498, FIND-500, FIND-509, FIND-515, FIND-302 ðŸŸ¡) van al plan DELTA.

# Plan de cableado de la feature `encryption` â€” cifrado en reposo en los 4 artefactos on-disk independientes

> **ADR de diseÃ±o:** [`docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md`](../architecture/adr/ADR-0057-encryption-at-rest.md).
> Este plan **implementa** ese ADR; no lo reemplaza. Si la implementaciÃ³n descubre que el ADR
> necesita un cambio, se abre un ADR nuevo que lo supere â€” no se edita el 0057 a posteriori.
>
> **Estado:** PROPUESTO (espera ratificaciÃ³n del owner â€” Regla 5, `.opencode/AGENTS.md`).
> **Origen:** decisiÃ³n del owner 2026-10-20 ("corregir y ademÃ¡s cablear la feature encryption") +
> P0 de la auditorÃ­a profunda (`docs/dev/reviews/2026-10-20-auditoria-profunda-v0.9.0.md:53`).
> **Alcance:** este plan **no** implementa nada todavÃ­a; define el trabajo por waves.

---

## 0. Objetivo

Cerrar `FIND-249` (0/6 superficies on-disk cifradas) dejando el producto con **una** capability
real y su documentaciÃ³n al dÃ­a:

1. **El claim dice la verdad** â€” y `FIND-407` ya estÃ¡ cerrado en esta rama con la redacciÃ³n
   honesta (commit `c0e63d4e`: "not encrypted at rest yet"). Este plan no repite ese arreglo:
   lo **enmenda hacia delante** cuando el cableado aterrice (W6), porque si no, la documentaciÃ³n
   pasarÃ­a a ser la claim falsa inversa.
2. **La feature estÃ¡ cableada** â€” con la semÃ¡ntica de clave, nonce, AAD, locks y migraciÃ³n
   declaradas en ADR-0057, no improvisadas durante la implementaciÃ³n.

**No-objetivo explÃ­cito:** cifrado de trÃ¡fico en red, cifrado de la clave en reposo por KMS nativo
(la vÃ­a `VANTADB_ENCRYPTION_KEY_CMD` es el shim), y migraciÃ³n in-place de bases existentes
(decisiÃ³n tomada en Â§7).

---

## 1. Precondiciones (obligatorias, en este orden)

1. Leer `.opencode/AGENTS.md` completo (reglas duras 1-11, tabla de roles, path resolution).
2. Leer `.opencode/rules/durability.md`, `.opencode/rules/concurrency-async.md` (regla 8 â€” orden
   global de locks), `.opencode/rules/indexes.md`, `.opencode/rules/memory-budget.md`,
   `.opencode/rules/api-contract.md`.
3. Leer `CONSTRAINTS.md` (quality bar) y `.opencode/references/clean-code-clean-architecture.md`
   (ApÃ©ndice V mÃ­nimo).
4. Leer ADR-0057 completo y `docs/dev/architecture/adr/ADR-0056-object-storage-path.md`
   (precedente de estilo).
5. Cargar la skill `progreso` (chequeo de WIP del backlog).
6. `git status --short` + `git log --oneline -5` para fijar el estado base.

---

## 2. Reglas duras (no negociables)

| # | Regla |
|---|---|
| R1 | **Regla dura `-p`:** nunca `cargo test`/`cargo nextest` desde la raÃ­z sin `-p <crate>`. |
| R2 | **No `continue-on-error`**, no `--no-verify`, no relajar gates. |
| R3 | **Regla 9 (no optimizar sin medir):** el coste del cifrado se mide contra el baseline canÃ³nico (`benches/canonical_p99`) antes de declarar la feature lista. NingÃºn nÃºmero sin comando reproducible + entorno + fecha. |
| R4 | **Regla 8 (concurrencia):** el cifrado **no** aÃ±ade locks ni altera el orden global `cardinality_stats â†’ insert_lock â†’ {wal, pending_hnsw_batch, hnsw, vstore, volatile_cache, backend}`. |
| R5 | **Regla 3 (docs sincronizados):** `SECURITY.md`, `docs/user/operations/CONFIGURATION.md`, `docs/user/operations/FEATURES.md` y el runbook de claves se actualizan **en el mismo PR** que el cÃ³digo que cambian. |
| R6 | **Regla 0 (impacto antes de tocar):** antes de modificar `src/wal.rs`, `src/wal_sharded.rs`, `src/storage/vfile.rs`, `src/storage/ops.rs`, `src/index/serialize/file.rs` o `src/backends/*` â†’ mapear callers con `codegraph_codegraph_explore` + `codebase-memory-mcp_trace_path` y pegar el blast radius en el PR. |
| R7 | **Sin `panic`/`unwrap` en hot path:** todo fallo de descifrado es `Err`. |
| R8 | **`Error` es `#[non_exhaustive]`** (`src/error.rs:136`): las variantes nuevas son aditivas y los bindings (`vantadb-python`, `vantadb-node`) deben mapear los nuevos cÃ³digos en el **mismo PR**. |
| R9 | **Nada de git mutating desde el implementador:** el ejecutor hace commit local al cerrar cada wave; el push es del owner. |
| R10 | **El claim de `SECURITY.md` no se toca hasta que la capacidad existe** (o hasta que se decide el descabe) â€” W0 es la excepciÃ³n, y es independiente. |

---

## 3. Inventario (verificado en `fix/audit-v090-p1`, 2026-10-20)

| # | Superficie lÃ³gica | Ficheros on-disk | Ruta de escritura real (`file:line`) | Â¿Cifrado hoy? | Â¿Trabajo independiente? |
|---|---|---|---|---|---|
| 1 | **WAL** | `data/vanta.wal` Â· `data/vanta.shard{0..N}.wal` Â· `vanta.shard{N}.wal.<ts>` (archivos) | `src/wal.rs:399` `append` (escribe `:411-413`) Â· `:435` `batch_append` Â· `src/wal_sharded.rs:438` `append` â†’ `:470` `batch_append` por shard Â· rotaciÃ³n `src/wal.rs:533` / `src/wal_sharded.rs:549` | **No** | **SÃ­** |
| 2 | **Ãndice HNSW** | `data/vector_index.bin` | `src/index/serialize/file.rs:16` `persist_to_file` Â· `:108` `sync_to_mmap` (tmp + `rename` `:147`) Â· invocado desde `src/storage/engine/maintenance.rs:144-158` `save_vector_index` (llamado en `:64` y `:541`) | **No** | **SÃ­** |
| 3 | **Segmentos VantaFile** | `data/vstore_L0..L3.vanta` (+ legacy `vector_store.vanta`) | primitiva Ãºnica `src/storage/ops.rs:64` `write_node_to_vstore` (header `:184`, payload `:188` `mmap_bytes_mut()`); callers `src/storage/engine/ops.rs:213`, `insert.rs:449` y `:612`, `txn.rs:455`, `mod.rs:400`, `maintenance.rs:982`; reescritura completa `src/storage/archive.rs:42` `compact_layout` (tmp + `rename` `:160`) | **No** | **SÃ­** |
| 4 | **backend KV** | Fjall `<root>/<backend>/0.jnl`+SSTs Â· RocksDB `<root>/<backend>/` | `src/backends/fjall_backend.rs:136` `put` Â· `:170` `write_batch` Â· `:196-205` `persist(PersistMode::SyncAll)` Â· CFs RocksDB `src/backends/rocksdb_backend.rs:134-152` | **No** | **SÃ­** |
| 5 | **Ã­ndice de texto** | **sin fichero propio** â€” keyspace `text_index` dentro de #4 | `src/text_index.rs:730` `posting_put_ops` â†’ `BackendPartition::TextIndex` (`:757`) â†’ `src/backends/fjall_backend.rs:86` `keyspace("text_index")` Â· CF `src/backends/rocksdb_backend.rs:152` | **No** | **No** â€” cubierto por #4 |
| 6 | **snapshots** | `data/snapshots/<n>/data/*` + `.../backend/*` | `src/storage/engine/mod.rs:717` (unix) / `:776` (win/wasm) `create_snapshot` â†’ `mirror_data_dir` `:543` + `mirror_backend_to` `:584` | **No** | **No** â€” espejo de #1-#4 |

**CorrecciÃ³n a `FIND-249`:** "6 artefactos" son 6 *superficies lÃ³gicas*; hay **4 puntos de
escritura independientes**. El Ã­ndice de texto es un keyspace del backend y los snapshots son un
espejo de los otros cuatro â†’ cifrar #1-#4 cifra #5 y #6 por trÃ¡nsito. **Esto reduce el alcance
de implementaciÃ³n y hay que actualizar la fila del backlog con esta evidencia** (no editar la
fila desde este plan; lo hace el ejecutor al cerrar W6, con la evidencia de los tests).

**Superficies derivadas que NO se cifran (metadatos, sin payload):** `vanta.wal.shards` (nÂº de
shards, `src/wal_sharded.rs:21`), `.wal_ship_marker` (`src/wal_shipping.rs:89`), `edge_index`
(derivado en memoria, sin artefacto), ficheros `.tmp` de compactaciÃ³n y colas de WAL corruptas
(`src/wal.rs:741`) â€” estos dos Ãºltimos **son ciphertext** al ser copias de artefactos cifrados.

---

## 4. Decisiones de diseÃ±o (resumen del ADR-0057, para no re-litigar en la implementaciÃ³n)

| # | DecisiÃ³n | Resumen operativo |
|---|---|---|
| **D1** | Envelope por artefacto, granularidad acorde al patrÃ³n de acceso | WAL: por frame (formato `WAL_FORMAT_VERSION` 3â†’4, `src/wal.rs:26`) Â· VantaFile: por nodo (formato `VFILE_VERSION` 2â†’3, `src/storage/vfile.rs:42`) Â· HNSW: por fichero Â· backend: por valor. El marcador de versiÃ³n vive en la cabecera cleartext existente; el modo se detecta del propio artefacto, no de la config. **Sin modo mixto**: una base es plaintext o encrypted. |
| **D2** | JerarquÃ­a master â†’ KEK â†’ DEK por artefacto, envuelta en un registro | Fuentes de master key: `VANTADB_ENCRYPTION_KEY` (env) â†’ `VANTADB_ENCRYPTION_KEY_FILE` (fichero 0600, Unix) â†’ `VANTADB_ENCRYPTION_KEY_CMD` (KMS shim). DEK aleatorio de 32 bytes por artefacto, envuelto con `KEK = HKDF-SHA256(master, salt=db_salt, info="vanta-kek-v1"â€–key_version)`, guardado en `<root>/encryption.registry`. **Rotar = re-envolver, no re-derivar** â†’ ningÃºn ciphertext se invalida (`FIND-194`). El patrÃ³n ya existe en `vanta-memory/src/utils/erasure.rs:122`. |
| **D3** | Nonce `epoch(u64) â€– counter(u32)` | `epoch` aleatorio de 64 bits por epoch de escritura, persistido en el registro **antes** del primer uso (el registro guarda solo el epoch actual por artefacto â†’ tamaÃ±o acotado); `counter` monotÃ³nico `u32` en memoria, se reasigna epoch nuevo si llega a `u32::MAX`. El discriminante de artefacto viaja en el **AAD**, no en el nonce (cada artefacto tiene su DEK). Uniqueness por construcciÃ³n, tambiÃ©n frente a rollback; el Ãºnico paso probabilÃ­stico es la colisiÃ³n de epoch (â‰ˆ`NÂ²/2^65`). El contador **no** es el ordinal del frame â†’ la unicidad no depende del orden de commit. |
| **D4** | AAD = `artifact_id â€– format_version â€– epoch â€– slot â€– plaintext_len [â€– identificador]` | `slot` = ordinal del frame (WAL) / offset del nodo (VantaFile) / 0 (HNSW, KV); `identificador` = `partitionâ€–key` en el KV. Un ciphertext movido de sitio **falla la autenticaciÃ³n**. El AAD se autentica, no se cifra. |
| **D5** | Cero locks nuevos; cifrado **antes** de tomar el lock destino | El cipher es inmutable y sin estado; el contador es un `AtomicU64`; el registro se escribe fuera de todo lock de motor (tmp+rename+fsync del dir). En el WAL: contador â†’ cifrar â†’ *luego* mutex del shard â†’ escribir. El CRC32C pasa a ser **del ciphertext** y la cadena `prev_hashâ€–record_hash` se calcula sobre el **ciphertext** â†’ `vanta-cli verify` sigue funcionando **sin clave**. |
| **D6** | 4 variantes de error nuevas (solo con la feature) | `EncryptionKeyRequired` Â· `EncryptionKeyRejected` (clave incorrecta **o** manipulaciÃ³n â€” indistinguibles, y el mensaje lo dice) Â· `EncryptionCorrupt` Â· `EncryptionUnsupported`. Mapeo a `VantaErrorCode` en los bindings en el mismo PR. |
| **D7** | Off por defecto (doble: feature fuera de `default` + sin clave no se cifra) y **sin migraciÃ³n in-place en v1** | Modo mismatch explÃ­cito y ruidoso. Base cifrada sin clave â†’ `EncryptionKeyRequired` (nunca "corrupta"). Base plaintext con clave â†’ plaintext + warning, sin migraciÃ³n implÃ­cita. |

---

## 5. Waves

Cada wave es atÃ³mico, con un criterio de "hecho" verificable y un comando ejecutable. Las waves
son **secuenciales** salvo donde se indica.

---

### W0 â€” Verificar el claim ya corregido (independiente del cableado)

> **Estado en esta rama: YA HECHO** â€” commit `c0e63d4e` (2026-10-20, "fix(security): SECURITY.md
> deja de publicar un cifrado en reposo que el core no aplica"). `SECURITY.md:26`, el doc de
> `src/crypto.rs:13-18` y `docs/CHANGELOG.md` ya dicen la verdad. **W0 no es trabajo nuevo: es un
> gate de no-regresiÃ³n**, y ademÃ¡s fija la obligaciÃ³n que crea W6.

**Artefacto:** ninguno (documentaciÃ³n).
**Ficheros:** `SECURITY.md:26` Â· `src/crypto.rs:13-18` (doc del mÃ³dulo) Â· `docs/CHANGELOG.md`.
**Invariantes:**
- El texto afirma **exactamente** el alcance negativo actual: primitivas detrÃ¡s de la feature,
  usadas por el envelope de `vanta-proxy`, y los ficheros de storage del core (WAL, HNSW,
  VantaFile, backend-KV, snapshots) **siguen en claro**.
- No se afirma "AES-256-GCM at-rest encryption" sin el adverbio de alcance.
- **ObligaciÃ³n que crea W6:** cuando el cableado aterrice, este texto se **enmenda hacia
  delante** ("not encrypted at rest yet" â†’ la capacidad real y su alcance exacto, Â§5.3 del ADR).
  Si no, la documentaciÃ³n pasa a ser la **claim falsa inversa**.
**Tests:** no aplica (doc) â€” pero **sÃ­** un gate de grep:
```powershell
Select-String -Path SECURITY.md,src/crypto.rs -Pattern "not encrypted at rest yet|ADR-0057|FIND-249"
```
**Hecho cuando:** el texto sigue diciendo la verdad (o ya dice la verdad hacia delante si el
cableado ya aterrizÃ³) y los tres ficheros **coinciden entre sÃ­** y con
`docs/user/operations/CONFIGURATION.md:75`.
**VerificaciÃ³n:**
```powershell
Select-String -Path SECURITY.md,src/crypto.rs -Pattern "at-rest|plaintext|FIND-249|ADR-0057"
```
**Dependencias:** ninguna.

---

### W1 â€” Primitivas: fuentes de clave, registro, DEK, nonce, AAD, errores

**Artefacto:** ninguno todavÃ­a (infraestructura compartida).
**Ficheros:** `src/crypto.rs` (nuevo mÃ³dulo interno `src/crypto/registry.rs` o un mÃ³dulo hermano
dentro del crate â€” el ejecutor elige la ruta que no cree un ciclo) Â· `src/config.rs:907-914` (campos nuevos:
`encryption_key_file`, `encryption_key_cmd`) Â· `src/error.rs` (4 variantes) Â·
`src/lib.rs:56-57` (exports pÃºblicos nuevos, feature-gated).
**Cambios de API interna requeridos en `src/crypto.rs` (ADR-0057 Â§D8, C-1..C-8):**
- `Cipher::encrypt_with_nonce(nonce, plaintext, aad)` y `decrypt_with_nonce(nonce, ct, aad)` â€”
  el `encrypt`/`decrypt` actuales se quedan como wrappers (nonce aleatorio, AAD vacÃ­o) para no
  tocar `vanta-proxy` ni `vanta-memory`.
- `Cipher::derive_kek(salt, key_version)` â€” HKDF con **salt del caller** (el `derive_namespace`
  actual usa un salt fijo y no sirve para la KEK por base de datos).
- `Cipher::raw_key_bytes()` â€” acceso al master de 32 bytes en raw (error en ciphers con passphrase).
- `KeySource::File` / `KeySource::Command` + `#[non_exhaustive]` en `KeySource` **y** en
  `CryptoError` (aÃ±adir variantes a un enum pÃºblico sin `#[non_exhaustive]` es breaking â†’ el crate
  es pre-1.0, sale como `feat!:` y `docs/api/` se actualiza en el mismo PR).
- `EncryptionStream` **no se usa** en este diseÃ±o (no expresa AAD, slot ni epoch) â†’ queda
  **deprecado**, no eliminado (ciclo de deprecaciÃ³n de `.opencode/AGENTS.md` Â§3c). Su Ãºnico
  consumidor actual es el test de `src/storage/vfile.rs:929`.
- ZeroizaciÃ³n de la master key y de los DEKs (`ring::secrecy` si la versiÃ³n fijada lo expone; si
  no, un wrapper local de ~10 lÃ­neas). **Sin dependencia nueva.**
**Invariantes:**
- `Cipher` sigue siendo inmutable y sin estado; no se le aÃ±ade ningÃºn campo mutable.
- El registro (`<storage_root>/data/encryption.registry`) se escribe con **tmp + rename + fsync del dir**
  (patrÃ³n de `src/storage/archive.rs:160-163`). Vive **dentro de `data_dir`** a propÃ³sito: asÃ­
  `mirror_data_dir` (`src/storage/engine/mod.rs:543`) lo copia en cada snapshot sin tocar nada, y
  **no** dentro del backend (los valores del backend estÃ¡n cifrados â†’ se necesita el registro
  antes de abrir el backend).
- El **epoch** de nonce se persiste **antes** del primer uso; el registro guarda solo el epoch
  actual por artefacto (tamaÃ±o acotado) y nunca reutiliza un epoch.
- La clave maestra **nunca** se persiste en claro; el registro solo contiene DEKs envueltos,
  `db_salt` (pÃºblico) y metadatos.
- Toda variante de `Error` nueva estÃ¡ `#[non_exhaustive]`-compatible (aditiva) y solo existe con
  la feature `encryption`.
- `Cipher::encrypt`/`decrypt` mantienen **exactamente** su comportamiento actual (para no romper
  `vanta-proxy` ni `vanta-memory`).
**Tests que lo cierran:**
- `key_from_file_reads_hex_and_rejects_world_readable_on_unix`
- `key_from_command_reads_stdout_and_never_logs_it`
- `key_sources_have_fixed_precedence_env_file_cmd`
- `registry_wrap_unwrap_roundtrip`
- `registry_epoch_is_persisted_before_first_use`
- `registry_keeps_only_the_current_epoch_per_artifact`
- `registry_write_is_atomic_tmp_rename`
- `nonce_is_unique_across_1000_encryptions_under_one_dek`
- `nonce_does_not_repeat_after_a_simulated_restart_new_epoch`
- `nonce_counter_overflow_allocates_a_new_epoch_instead_of_wrapping`
- `aad_builder_is_deterministic_and_binds_slot_and_length`
- `aad_mismatch_fails_authentication` (AAD distinto â†’ error, no plaintext)
- `cipher_encrypt_decrypt_legacy_behaviour_is_unchanged` (C-1: los wrappers no cambian nada)
- `derive_kek_uses_the_caller_supplied_salt` (C-3)
- `raw_key_bytes_rejects_a_passphrase_cipher` (C-4)
- `error_variants_map_to_stable_codes` (paridad con `VantaErrorCode`)
**Hecho cuando:** los 16 tests verdes con `--features encryption`, y **la ruta plaintext sigue
byte-idÃ©ntica** (test: `plaintext_path_is_byte_identical_without_key` â€” compara los bytes de un
artefacto con y sin la feature compilada).
**VerificaciÃ³n:**
```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo nextest run -p vantadb --profile audit --lib --features encryption
cargo clippy -p vantadb --features encryption -- -D warnings
cargo fmt --check
```

---

### W2 â€” WAL cifrado (formato 3 â†’ 4)

**Artefacto:** #1 (WAL).
**Ficheros:** `src/wal.rs` (`WalHeader` `:158`, `append` `:399`, `batch_append` `:435`,
`recover_valid_records` `:693`, `WalReader::next_record` `:1046`, `quarantine_corrupt_tail` `:741`,
`chain_hash` `:84`) Â· `src/wal_sharded.rs` (`append` `:438`, `batch_append` `:453`, `recover` `:481`,
`rotate_all` `:549`) Â· `src/wal_shipping.rs:113` (el shipper debe ser frame-aware).
**Invariantes:**
- Formato v4: frame = `[u32 ct_len][u32 crc32c(ct)][12B nonce][ctâ€–16B tag][32B prev_hash][32B record_hash]`
  â€” los hashes de cadena siguen **al final del frame**, como en v3, para que el recorrido del reader
  cambie lo mÃ­nimo. `record_hash = SHA-256(prev_hash â€– ct_len_le â€– crc_le â€– nonce â€– ct)`.
- El CRC32C cubre **el ciphertext**; la cadena `prev_hashâ€–record_hash` se calcula sobre el
  **ciphertext** â†’ `vanta-cli verify` valida la cadena **sin clave**.
- Un frame con CRC vÃ¡lido y tag invÃ¡lido = **error duro** (clave incorrecta o manipulaciÃ³n), NO
  cola truncable. Un frame con CRC invÃ¡lido = cola corrupta â†’ truncado + cuarentena, como hoy.
- El replay de un WAL v3 (no cifrado) sigue funcionando **sin cambios** (compatibilidad hacia
  atrÃ¡s).
- El cifrado ocurre **antes** de tomar el mutex del shard; la secciÃ³n crÃ­tica solo crece con el
  memcpy.
- Un contador de nonce consumido por una escritura fallida se pierde (hueco), no se reutiliza.
**Tests que lo cierran:**
- `wal_v4_roundtrip_encrypt_decrypt`
- `wal_v4_replays_to_the_same_records_as_v3`
- `wal_v3_file_still_replays_on_a_v4_reader` â† **compatibilidad hacia atrÃ¡s**
- `wal_v4_detects_a_corrupted_ciphertext_as_corrupt_tail`
- `wal_v4_rejects_a_valid_crc_with_a_wrong_key_as_a_hard_error`
- `wal_v4_chain_verifies_without_the_key`
- `wal_v4_chain_detects_a_reordered_frame`
- `wal_v4_frame_cannot_be_moved_to_another_ordinal` (AAD, D4)
- `wal_v4_nonce_is_unique_across_10000_frames`
- `wal_v4_survives_a_crash_tail_truncation` (failpoint `wal_append_fail`)
- `wal_shipping_reads_v4_segments` (feature `wal-shipping`)
**Hecho cuando:** los 11 tests verdes, el WAL v3 se sigue reproduciendo, y el probe de canario
(no plaintext en `data/vanta.shard0.wal`) pasa.
**VerificaciÃ³n:**
```powershell
cargo nextest run -p vantadb --profile audit --lib wal --features encryption
cargo nextest run -p vantadb --profile audit --test wal_resilience --features encryption
```

---

### W3 â€” VantaFile cifrado (formato `VFILE_VERSION` 2 â†’ 3)

**Artefacto:** #3 (segmentos VantaFile).
**Ficheros:** `src/storage/ops.rs:64` `write_node_to_vstore` (la **Ãºnica** primitiva de escritura)
Â· `src/storage/vfile.rs` (`Header`/`VFILE_VERSION` `:163`, `read_header` `:315`, `write_header`
`:330`, `grow_to` `:359`, `cipher` `:174`/`:243`) Â· `src/storage/archive.rs:42` `compact_layout`
(re-escritura completa) Â· `src/storage/engine/insert.rs:449`/`:612` Â· `src/storage/engine/ops.rs:213`
Â· `src/storage/engine/txn.rs:455` Â· `src/storage/engine/mod.rs:400` Â·
`src/storage/engine/maintenance.rs:982` (promociÃ³n L0â†’L1) Â· los lectores de bÃºsqueda
(`src/index/search.rs` y el path de `read_header`).
**Invariantes:**
- Formato v3: registro = `[u32 blob_len][12B nonce][ctâ€–16B tag]`, plaintext =
  `DiskNodeHeader(64B) â€– vector payload`. El `vector_offset` del header descifrado deja de ser una
  direcciÃ³n de fichero (el payload vive dentro del blob) â†’ el lector usa `blob[64..]`.
- El `AAD` liga el blob a su **offset** â†’ mover un blob a otro offset falla la autenticaciÃ³n.
- La compactaciÃ³n re-cifra (lee descifrado, escribe cifrado) y conserva el tmp+rename atÃ³mico.
- **PÃ©rdida de zero-copy declarada**: en modo cifrado el vector se materializa (no es un
  `&[f32]` sobre el mmap) â†’ `MmapFull` no se usa en bases cifradas. Esto va al runbook y a la
  mediciÃ³n de coste.
- Un segmento v2 (el formato actual) se sigue leyendo **sin cambios** (compatibilidad hacia atrÃ¡s).
**Tests que lo cierran:**
- `vfile_v3_node_roundtrip_encrypt_decrypt`
- `vfile_v2_segment_still_reads_on_a_v3_reader` â† **compatibilidad hacia atrÃ¡s**
- `vfile_v3_detects_a_corrupted_node_blob`
- `vfile_v3_wrong_key_is_a_hard_error_not_an_empty_result`
- `vfile_v3_node_blob_cannot_be_moved_to_another_offset` (AAD)
- `vfile_v3_compaction_roundtrips_and_re_encrypts`
- `vfile_v3_search_finds_the_same_neighbours` (el recall no cambia)
- `vfile_v3_tombstone_write_still_works` (`insert.rs:135`)
- `vfile_v3_grow_to_and_prealloc_batch_still_work` (`insert.rs:101` `prealloc_vstore_batch`)
**Hecho cuando:** los 9 tests verdes, un segmento v2 se lee igual, y el probe de canario no
encuentra vectores en claro en `data/vstore_L0.vanta`.
**Dependencias:** W1 (registro/nonce/AAD) y W2 (el WAL debe poder escribir cifrado antes, para
que un crash deje un estado coherente).
**VerificaciÃ³n:**
```powershell
cargo nextest run -p vantadb --profile audit --lib vfile --features encryption
cargo nextest run -p vantadb --profile audit --test vfile_search --features encryption
```

---

### W4 â€” backend KV cifrado (envelope por valor)

**Artefacto:** #4 (backend KV) â†’ cubre #5 (Ã­ndice de texto) por construcciÃ³n.
**Ficheros:** `src/backends/fjall_backend.rs` (`put` `:136`, `get` `:142`, `get_many` `:149`,
`delete` `:164`, `write_batch` `:170`, `persist` `:196-205`, keyspaces `:62-99`) Â·
`src/backends/rocksdb_backend.rs` (`put`/`get`/`write_batch` equivalentes, CFs `:134-152`) Â·
`src/storage/ops.rs` (los `BackendWriteOp` que construyen `src/text_index.rs:730`/`:766` y
`src/attestation.rs:216-274`).
**Invariantes:**
- Valor cifrado = `[12B nonce][ctâ€–16B tag]`; el `plaintext_len` es derivable del largo del valor.
- **Las claves NO se cifran** (el Ã­ndice de texto necesita prefijos por namespace â€”
  `src/text_index.rs:549`; el Ã­ndice de namespace necesita lookups ordenados). Limit declarado en
  `SECURITY.md`.
- La atomicidad del `write_batch` se preserva: el envelope es el valor, el batch sigue siendo
  todo-o-nada en Fjall y RocksDB.
- `internal_metadata` (incluido `checkpoint_seq`, leÃ­do en `src/storage/engine/init.rs:427`)
  tambiÃ©n va cifrado â†’ la clave se resuelve **antes** de abrir el backend.
- Un backend plaintext se lee **sin cambios** (compatibilidad hacia atrÃ¡s).
**Tests que lo cierran:**
- `backend_value_roundtrip_encrypt_decrypt` (Fjall y RocksDB)
- `backend_batch_stays_atomic_with_encrypted_values`
- `backend_wrong_key_is_a_hard_error`
- `backend_plaintext_backend_still_reads_on_an_encrypted_build` â† **compatibilidad hacia atrÃ¡s**
- `backend_text_index_postings_are_ciphertext_at_rest` (el Ã­ndice de texto = #5)
- `backend_namespace_prefix_scan_still_works` (las claves siguen en claro y ordenables)
- `backend_checkpoint_seq_roundtrips` (`internal_metadata`)
**Hecho cuando:** los 7 tests verdes y el probe de canario no encuentra el payload en claro ni
en el journal Fjall (`0.jnl`).
**Dependencias:** W1. Independiente de W2/W3 (pero se cierra despuÃ©s, para que el estado de un
crash sea coherente en los 4).
**VerificaciÃ³n:**
```powershell
cargo nextest run -p vantadb --profile audit --lib backend --features encryption
cargo nextest run -p vantadb --profile audit --test backend_compare --features encryption
```

---

### W5 â€” Imagen HNSW cifrada (envelope por fichero)

**Artefacto:** #2 (Ã­ndice HNSW).
**Ficheros:** `src/index/serialize/file.rs` (`persist_to_file` `:16`, `load_from_file` `:45`,
`sync_to_mmap` `:108`) Â· `src/index/serialize/bytes.rs` (`serialize_to_bytes` `:26`,
`deserialize_from_bytes` `:317`) Â· `src/index/graph/types.rs:22`
(`VECTOR_INDEX_VERSION: u16 = 8` â†’ 9) Â· `src/index/port_impl.rs` (`open_index_port` /
`new_mmap_port` / `new_in_memory_port`, invocados en `src/storage/engine/init.rs:324-357`) Â·
`src/storage/engine/maintenance.rs:144-158` `save_vector_index` Â· `src/storage/archive.rs:171`
`rebuild_hnsw_from_vstore` Â· `src/migration.rs:272`/`:439` (lectura de la cabecera `VNDX`).
**Invariantes:**
- El fichero completo es **un** envelope AEAD; la cabecera `VNDX` (cleartext) sube de versiÃ³n â†’
  el lector sabe si hay que descifrar.
- El `AAD` liga la imagen a su **ruta relativa** dentro del storage root â†’ copiar la imagen de
  otra base falla.
- La escritura conserva el **tmp + rename** atÃ³mico (`:123-147`).
- El arranque en frÃ­o con mmap **degrada** a "leer + descifrar" en modo cifrado â†’ lÃ­mite
  declarado y medido.
- Un Ã­ndice plaintext (versiÃ³n antigua) se sigue leyendo sin cambios; el rebuild desde el VantaFile
  (`src/storage/engine/init.rs:410`) sigue funcionando.
**Tests que lo cierran:**
- `hnsw_index_roundtrip_encrypt_decrypt`
- `hnsw_corrupt_image_is_detected_and_rebuilt`
- `hnsw_wrong_key_is_a_hard_error`
- `hnsw_image_cannot_be_copied_from_another_database` (AAD con la ruta)
- `hnsw_plaintext_index_still_loads_on_an_encrypted_build` â† **compatibilidad hacia atrÃ¡s** (Ã­ndice `VNDX` v8)
- `hnsw_rebuild_from_vstore_still_works` (ruta de recuperaciÃ³n)
- `hnsw_sync_to_mmap_keeps_atomic_rename` (el tmp se escribe cifrado)
**Hecho cuando:** los 7 tests verdes y el probe de canario no encuentra vectores en claro en
`data/vector_index.bin`.
**Dependencias:** W1. Cierra despuÃ©s de W3 (el rebuild lee el VantaFile, que debe poder descifrar).
**VerificaciÃ³n:**
```powershell
cargo nextest run -p vantadb --profile audit --lib hnsw --features encryption
cargo nextest run -p vantadb --profile audit --test hnsw_validation --features encryption
```

---

### W6 - ActivaciÃ³n, registro, rotaciÃ³n, y documentaciÃ³n honesta (el claim pasa a ser verdad)

**Estado:** âœ… implementada y verificada el 2026-10-21 (nota de la wave:
`docs/dev/reviews/2026-10-21-w6-encryption-activation.md`). 90 tests de cifrado + los 2493 del
lib en verde, `clippy -D warnings` limpio con y sin la feature `encryption`, `fmt --check`
limpio, canario de 6 superficies en verde. **Siguiente:** W7 (mediciÃ³n del p99 y del coste por
imagen) â€” ver Â§5 de la nota de la wave: el bench `canonical_p99` es in-memory y no mide el coste
at-rest, hacen falta el bench del WAL cifrado y uno nuevo del motor con clave configurada.

**Artefacto:** transverso (los 4) + #5 y #6 por trÃ¡nsito.
**Ficheros:** `src/storage/engine/init.rs` (detecciÃ³n de modo en el open, `:324`/`:404`/`:427`) Â·
`src/config.rs:907-914` + `src/cli_handlers/config.rs:287-288` (claves nuevas en el `config show`) Â·
`src/server/bootstrap.rs:137-161` (warning de "clave sin feature" â†’ extender al core) Â·
`src/storage/engine/mod.rs:521` (**`is_mutable_after_snapshot` debe incluir
`encryption.registry`** â†’ el snapshot lo copia, no lo hard-linka) Â· `src/cli_handlers/` (nuevo
`vanta-cli encryption rotate`) Â· `SECURITY.md:26` Â· `src/crypto.rs:13-18` (doc del mÃ³dulo) Â·
`docs/user/operations/CONFIGURATION.md:75`/`:520` Â· `docs/user/operations/FEATURES.md` Â·
runbook nuevo de claves.
**Invariantes:**
- DetecciÃ³n de modo **desde los artefactos**, no desde la config: una base cifrada sin clave â†’
  `EncryptionKeyRequired` (nunca "corrupta"); una base plaintext con clave â†’ plaintext + warning
  (sin migraciÃ³n implÃ­cita, sin modo mixto).
- El registro (`encryption.registry`) vive en `data_dir` â†’ `mirror_data_dir` lo copia solo; **pero**
  hay que aÃ±adirlo a `is_mutable_after_snapshot` (`src/storage/engine/mod.rs:521`) para que el
  snapshot lo **copie** y no lo hard-linke (se reescribe en cada open writable â†’ un hard link
  dejarÃ­a que un write posterior al snapshot mutara el snapshot por el inode compartido).
- La rotaciÃ³n (`vanta-cli encryption rotate`) re-envuelve los DEKs con la nueva master key bajo
  un `key_version` nuevo; **ningÃºn ciphertext existente se invalida** (`FIND-194`).
- La rotaciÃ³n deja `encryption.registry.bak` â†’ rollback manual posible mientras exista la master
  key antigua.
- El `SECURITY.md` y el doc de `src/crypto.rs` se **enmendan hacia delante**: de "not encrypted
  at rest yet" a la capacidad real con su alcance exacto (los 4 artefactos + texto + snapshots,
  opt-in, fuera de `default`, y lo que NO se cubre: claves del KV en claro, metadatos
  estructurales, sin migraciÃ³n in-place). `CONFIGURATION.md:75`/`:520` y `FEATURES.md` se
  actualizan en el mismo PR.
**Tests que lo cierran:**
- `encrypted_db_without_key_fails_with_key_required`
- `plaintext_db_with_key_opens_plaintext_and_warns`
- `registry_is_included_in_the_snapshot_mirror`
- `registry_is_copied_not_hardlinked_by_the_snapshot` (hard link = un write posterior al snapshot
  muta el snapshot por el inode compartido)
- `encryption_rotate_preserves_readability_of_pre_rotation_data` â† **`FIND-194`**
- `encryption_rotate_leaves_a_bak_registry`
- `snapshot_of_an_encrypted_db_is_restorable_with_the_key`
- `snapshot_of_an_encrypted_db_is_refused_without_the_key`
- `security_md_claim_matches_the_implemented_scope` (gate de doc: la frase de alcance existe)
- `no_document_still_says_not_encrypted_at_rest_yet` (gate de doc: la claim inversa tampoco vale)
**Hecho cuando:** los 10 tests verdes, el `SECURITY.md` afirma la capacidad real (enmendado hacia
delante), y el probe completo (6 superficies) no encuentra plaintext en ninguna.
**Dependencias:** W1-W5 (todas).
**VerificaciÃ³n:**
```powershell
cargo nextest run -p vantadb --profile audit --lib encryption --features encryption
# probe de canario (el mismo criterio que usÃ³ el probe de FIND-249):
cargo nextest run -p vantadb --profile audit --test encryption_canary --features encryption
```

---

### W7 â€” MediciÃ³n de coste y cierre del claim

> **Estado: CERRADA con limite declarado (2026-10-23).** Gate del 15% SUPERADO (+43,2% en p99
> post-Paso 0 — mediana de 3+3 corridas intercaladas). El plan lo prescribe: *"Superarlo es una"*
> *"senal de diseno, no algo que se absorba en silencio."* El numero esta publicado, la causa esta
> identificada con `file:line`, y el fix esta disenado. El trabajo de optimizacion va en un plan
> separado (envelope por pagina + LRU, breaking change de `VectorStoreRef`). Ver la nota completa
> en `docs/dev/reviews/2026-10-21-w7-cifrado-coste.md`.
>
> **Medicion publicada:** `BENCHMARKS.md §22.4` (post-Paso 0): search p99 **+43,2%** (gate SUPERADO);
> insert/flush ~0% (OK). Causa: envelope VantaFile por nodo, 4 materializaciones 6KB/candidato.
> Fix pendiente (plan separado): `docs/dev/architecture/design/2026-10-21-encryption-envelope-per-page.md`

**Artefacto:** transverso.
**Ficheros:** `benches/canonical_p99.rs` (variante cifrada) Â· `benches/wal_throughput.rs` Â·
`docs/user/operations/BENCHMARKS.md` (nÃºmeros + comando + entorno + fecha) Â·
`docs/dev/Backlog.md` (fila `FIND-249` y `FIND-407` â†’ resueltas, con la evidencia) Â·
`docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md` (si el owner lo ratifica: `status:
accepted`).
**Invariantes:**
- MediciÃ³n **before/after** con el mismo dataset (seed 42, 100k Ã— 1536d), mismo perfil
  (`benches/common/mod.rs:apply_fixed_profile`, warmup 3 s / measure 5 s), 3 corrida, mediana.
- Baseline contra el registrado en `docs/user/operations/BENCHMARKS.md:380` (p50 2.2388 / p95
  4.248 / **p99 4.9987 ms**, 2026-09-25).
- **Umbral de aceptaciÃ³n: el modo cifrado no debe empeorar el p99 canÃ³nico en mÃ¡s de 15 %** (el
  mismo umbral que el repo usa para el tradeoff de localidad/complejidad,
  `.opencode/rules/durability.md:10-17`). Superarlo es una seÃ±al de **diseÃ±o** (cambiar la
  granularidad del envelope del VantaFile), no algo que se absorba en silencio.
- Todo nÃºmero publicado lleva comando + entorno + fecha (Regla 11).
**Tests que lo cierran:** no aplica (benchmarks + gate de doc).
**Hecho cuando:** los nÃºmeros estÃ¡n en `BENCHMARKS.md`, el delta estÃ¡ dentro del umbral (o el
diseÃ±o se revisÃ³), y las filas `FIND-249`/`FIND-407` del backlog estÃ¡n actualizadas con la
evidencia (tests + probe). En concreto, la fila `FIND-249` debe **corregir su recuento**:
"0/6 artefactos" son 6 *superficies lÃ³gicas* pero solo **4 puntos de escritura independientes**
(el Ã­ndice de texto es un keyspace del backend; los snapshots son un espejo) â€” con la evidencia
`file:line` de la tabla de Â§3.
**VerificaciÃ³n:**
```powershell
cargo bench -p vantadb --bench canonical_p99
cargo bench -p vantadb --bench canonical_p99 --features encryption
cargo bench -p vantadb --bench wal_throughput
cargo bench -p vantadb --bench wal_throughput --features encryption
```

---

## 6. Criterio de feature flag + off por defecto

**CompilaciÃ³n** â€” la feature `encryption` **no** entra en `default` (`Cargo.toml:153`); el crate
publicado en crates.io no contiene el cÃ³digo. Este criterio **no cambia** en ninguna wave.

```powershell
# el gate: la feature no estÃ¡ en default
Select-String -Path Cargo.toml -Pattern '^default\s*='
# debe listar: cli, arrow, fjall, roaring, advanced-tokenizer, memmap2, fs2, sysinfo, rayon
# y NO: encryption
```

**Runtime** â€” el cifrado estÃ¡ activo **solo si** (a) la feature estÃ¡ compilada **y** (b) una de las
tres fuentes de clave resuelve. Sin clave â†’ camino plaintext, **bytes idÃ©nticos** a los de hoy.

**Gate de equivalencia plaintext** (obligatorio en W1 y re-verificado en cada wave): un test
compara los bytes de un artefacto producido con la feature compilada **sin clave** contra los
producidos por un build sin la feature â†’ deben ser idÃ©nticos byte a byte.

```powershell
cargo nextest run -p vantadb --profile audit --lib plaintext_path_is_byte_identical_without_key --features encryption
```

**Gate de la matriz de features** (obligatorio antes de cerrar W6):

```powershell
cargo check -p vantadb --no-default-features
cargo check -p vantadb --no-default-features --features fjall
cargo check -p vantadb --features encryption
cargo check -p vantadb-server --features encryption
```

---

## 7. MigraciÃ³n de datos existentes â€” decisiÃ³n

**DecisiÃ³n: NO se migran las bases existentes in-place en v1.**

- **QuÃ© pasa con una base plaintext existente:** sigue funcionando, **byte-idÃ©ntica**, para
  siempre. Compilar la feature no cambia sus bytes. Abrirla con una clave configurada la abre en
  plaintext con un warning (no migra, no mezcla, no promete nada).
- **CÃ³mo se cifra una base existente (procedimiento manual, offline, documentado):**
  1. `vanta-cli export --out <dir>` (path existente: `Embedded::export_all`,
     `src/sdk/serialization/impl_export.rs:315`).
  2. Configurar la clave (`VANTADB_ENCRYPTION_KEY_FILE` con modo `0600`, o
     `VANTADB_ENCRYPTION_KEY_CMD`).
  3. Abrir un **directorio de base nuevo** con la clave configurada.
  4. `vanta-cli import --from <dir>` (path existente: `import_records` `:449` /
     `import_file` `:531`, mismo fichero).
  5. Verificar: el probe de canario no encuentra plaintext en ninguna de las 6 superficies.
- **Por quÃ© no in-place:** obliga a reescribir de forma crash-safe los 4 puntos de escritura
  independientes (WAL, HNSW, VantaFile, backend) y a interoperar con `FIND-246` (el
  `checkpoint_seq` se publica antes que los artefactos del snapshot â€”
  `src/storage/engine/maintenance.rs:81-90`), que sigue abierto. El export/import convierte el
  problema en "leer una base y escribir otra", que ya tiene contrato y tests.
- **Coste de la decisiÃ³n (explÃ­cito):** encender el cifrado en una base viva es una operaciÃ³n
  manual con dependencia de export, no un cambio de un flag. Se documenta en el runbook.
- **Diferido (con su propio ADR, no como omisiÃ³n):** `vanta-cli encryption enable` (migraciÃ³n
  in-place), KMS nativo mÃ¡s allÃ¡ del shim `*_KEY_CMD`, y re-cifrado por registro para rotaciÃ³n
  granular.

---

## 8. VerificaciÃ³n global y gates de cierre

**Baseline de la suite en esta rama (2026-10-20, `cargo nextest run -p vantadb --profile audit --lib`):**
`Summary [1212.463s] 2468 tests run: 2457 passed (30 slow), 10 failed, 1 timed out, 2 skipped`.
Los 11 fallos son **preexistentes y ajenos al cifrado**, y hay que distinguirlos al interpretar
cualquier corrida posterior:

| Tipo | Tests | Causa |
|---|---|---|
| FAIL real (3) | `index::distance::metrics::tests::test_euclidean_distance_squared_mismatched_length` Â· `test_cosine_sim_mismatched_length` Â· `index::distance::mapper::tests::test_sq8_similarity_mismatched_dims_no_panic` | aserciÃ³n NaN vs 0 en longitudes mismatched â€” **bug real preexistente** en `src/index/distance/` |
| ABORT ambiental (7) | `index::search::tests::test_search_nearest_*` (6) + 1 | agotamiento de recursos en Windows: stack-buffer overrun `0xc0000409` (os error 1282) y *paging file too small* (os error 1455) con los binarios de test en paralelo |
| TIMEOUT (1) | `index::core::tests::concurrent_insert_preserves_hnsw_invariants` (180 s) | saturaciÃ³n de la mÃ¡quina (los tests de `storage::engine::tests` tardaron 60-85 s cada uno) |

AdemÃ¡s, `cargo nextest run -p vantadb --profile audit` (suite completa) **no compila** el target de
integraciÃ³n `concurrency_parity` (152 errores, cabeza `E0463`) â€” preexistente en el worktree, a
verificar por separado. **Ninguno de estos fallos lo introduce el cifrado**; si una wave los
cambia, es un hallazgo.

**Gate por wave (obligatorio, en este orden):**

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo fmt --check
cargo clippy -p vantadb --features encryption -- -D warnings
cargo nextest run -p vantadb --profile audit --lib --features encryption
cargo nextest run -p vantadb --profile audit --features encryption
```

**Gate de cierre del plan (W7):**

```powershell
cargo nextest run -p vantadb --profile audit
cargo clippy -- -D warnings
cargo fmt --check
cargo bench -p vantadb --bench canonical_p99 --no-run
node scripts/docs/check-links.mjs
node scripts/docs/check-docs.mjs
```

**RevisiÃ³n obligatoria antes de cerrar** (P2-01, contexto fresco):
- `vanta-audit` â€” `unsafe` en el path de descifrado, mmap del ciphertext, FFI de los bindings.
- `vanta-chaos` â€” truncamiento de cola cifrada, corte de energÃ­a durante un envelope, rotaciÃ³n a
  mitad de escritura, corrupciÃ³n de un byte en el ciphertext.
- `vanta-review` â€” veredicto sobre el diff completo (nunca auto-revisiÃ³n).

**Probe de canario (el criterio que usÃ³ el probe de `FIND-249`):** con la feature activa y una
clave vÃ¡lida, un payload conocido **no** debe aparecer en claro en ninguno de: `data/vanta.wal`,
`data/vanta.shard{0..N}.wal`, `data/vanta.shard{N}.wal.<ts>` (archivos), `data/vector_index.bin`,
`data/vstore_L{0..3}.vanta`, el journal y los SSTs del backend Fjall, y
`data/snapshots/<n>/{data,backend}/**`. El probe se convierte en un test de integraciÃ³n
(`tests/encryption_canary.rs`) en W6.

---

## 9. Coste â€” quÃ© hay que medir (Regla 9)

| MediciÃ³n | Comando | Baseline |
|---|---|---|
| P99 canÃ³nico (insert + search) | `cargo bench -p vantadb --bench canonical_p99` | `docs/user/operations/BENCHMARKS.md:380` â€” p50 2.2388 / p95 4.248 / **p99 4.9987 ms** (2026-09-25) |
| P99 canÃ³nico cifrado | `cargo bench -p vantadb --bench canonical_p99 --features encryption` | **no existe** â†’ se mide y se publica en W7 |
| Throughput del WAL | `cargo bench -p vantadb --bench wal_throughput` (+ variante cifrada) | **no existe** â†’ se registra antes del cambio, en el mismo PR |
| RSS pico (regla de memory-budget) | el mismo run, perfil `insert.total_records` | contra el techo de `.opencode/rules/memory-budget.md` |

**Overhead en bytes (aritmÃ©tica determinista, no claim de benchmark):** WAL +28 B/frame
(nonce 12 + tag 16) + 8 B de bookkeeping; VantaFile +28 B/nodo + 4 B de prefijo; backend KV
+28 B/valor; HNSW +28 B/fichero. Para un vector `f32` de 6 KiB, el overhead del VantaFile es
â‰ˆ0,46 %; para un valor de KV de 20 B, â‰ˆ140 % en bytes (â‰ˆ28 B absolutos).

**Overhead de CPU (AES-256-GCM con AES-NI): no medido â†’ no se afirma nada hasta W7.**

---

## 10. Riesgos (resumen operativo; el detalle estÃ¡ en ADR-0057 Â§8)

| ID | Riesgo | MitigaciÃ³n en el plan |
|---|---|---|
| R-1 | Reuso de nonce (catastrÃ³fico en GCM) | Esquema D3 con unicidad por construcciÃ³n + tests dedicados en W1 y W2 |
| R-2 | Clave incorrecta aceptada en silencio | Todo read autentica; el registro se valida en el open antes de cualquier escritura |
| R-3 | La pÃ©rdida de zero-copy rompe el budget o el p99 | MediciÃ³n en W7 con umbral del 15 %; si falla, se cambia la granularidad (pÃ¡gina en vez de nodo) |
| R-4 | rollout parcial (WAL v4 con VantaFile v2 sin cifrar) | El modo es **por base**, no por artefacto; el mismatch es error duro, no warning |
| R-5 | Cola cifrada a medio escribir | CRC sobre el ciphertext separa "corrupto" (truncable) de "clave incorrecta" (error duro) |
| R-6 | PresiÃ³n de descabe | El fallback estÃ¡ escrito: si el wiring no aterriza, el claim honesto de `c0e63d4e` ya estÃ¡ en la rama y `FIND-407` no se reabre. Los diferidos van nombrados: migraciÃ³n in-place (`vanta-cli encryption enable`), KMS nativo mÃ¡s allÃ¡ del shim `*_KEY_CMD`, y re-cifrado por registro para rotaciÃ³n granular. |
| R-12 | **Claim falsa inversa**: el wiring aterriza y `SECURITY.md` sigue diciendo "not encrypted at rest yet" (o al revÃ©s) | Medium | W6 incluye la **enmenda hacia delante** de `SECURITY.md:26` + el doc de `src/crypto.rs:13-18`, con un gate (`no_document_still_says_not_encrypted_at_rest_yet`). El texto de W0 es un gate de no-regresiÃ³n, no trabajo nuevo. |
| R-7 | Registro perdido o separado de la base | Vive dentro del storage root y **se espeja en el snapshot** (W6) |
| R-8 | `FIND-246` (checkpoint_seq antes de los artefactos) | El registro (material DEK/IV) se persiste **antes** del primer cifrado; el defecto de orden sigue abierto y **no** se arregla aquÃ­ |
| R-9 | La rotaciÃ³n invalida datos | Se re-envuelven DEKs, no se re-derivan; test explÃ­cito en W6 |
| R-10 | El juego de variantes de `Error` depende de la feature | Mapeo a `VantaErrorCode` y tabla de `docs/api/` en el mismo PR |
| R-11 | wal-shipping saca registros en claro | Declarado; la rÃ©plica necesita el DEK; el transporte queda fuera del alcance "at rest" |

---

## 11. Handoff

- **DueÃ±o del diseÃ±o:** `vanta-arch` (este plan + ADR-0057).
- **ImplementaciÃ³n:** `vanta-worker` (W1-W6), por wave, con commit local al cerrar cada una.
- **VerificaciÃ³n de seguridad/recuperaciÃ³n:** `vanta-audit` (W1, W3, W4) y `vanta-chaos` (W2, W3,
  W6) â€” son leaf nodes, no delegan.
- **RevisiÃ³n final:** `vanta-review` (P2-01) antes de marcar el plan como completado.
- **Release / changelog / packaging:** `vanta-lead` (nada de esto lo hace el implementador).

**Orden de entrega:** W0 (gate de no-regresiÃ³n del claim ya corregido â€” no es trabajo nuevo) â†’
W1 â†’ W2 â†’ W3 â†’ W4 â†’ W5 â†’ W6 â†’ W7.
