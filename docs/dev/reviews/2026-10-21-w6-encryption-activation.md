---
title: "W6 — Activacion del cifrado en reposo: un contexto, un registro, rotacion y documentacion honesta"
kind: review
status: active
description: "Nota de la wave W6 del plan de cableado de la feature `encryption` (ADR-0057): que se implemento, que queda abierto y que se verifico."
tags: [vantadb, review, encryption, at-rest, adr-0057, wave-w6]
created: "2026-10-21"
---

# W6 — Activacion del cifrado en reposo (ADR-0057)

> **Wave:** W6 del plan
> [`2026-10-20-wiring-encryption-feature.md`](../plans/2026-10-20-wiring-encryption-feature.md).
> **Design:** [`ADR-0057`](../architecture/adr/ADR-0057-encryption-at-rest.md).
> **Estado:** implementada y verificada (2026-10-21).

W1-W5 dejaron la **infraestructura** (primitivas, WAL v4, VantaFile v3,
backend-KV, imagen HNSW v9) pero **nada estaba encendido**: cada artefacto
resolvia su contexto **del entorno** por su cuenta, cada uno abria **su
propio** registro, y una base creada con clave acababa en **modo mixto**.
W6 es la wave que **activa** la feature.

---

## 1. Que se implemento

### 1.1 Un contexto por proceso (`src/storage/engine/init.rs`)

El motor resuelve **una** master key y **un** decision de modo en el open, y
los reparte a los cuatro artefactos:

| Pieza | Fichero | Que hace |
|---|---|---|
| `DbEncryption` | `src/storage/engine/init.rs` | Resuelve la master key desde el `Config` (una `Config::encryption_key` **programatica** ya es visible), decide el modo **desde los artefactos** y registra la decision para el proceso |
| `detect_encryption_mode` | `src/storage/engine/init.rs` | Lee los **marcadores** del disco: el header del WAL (v4), el de los segmentos (v3), el de la imagen HNSW (v9) y la existencia del registro |
| `is_new_database` | `src/storage/engine/init.rs` | Una base **brand new** (sin ficheros) con clave se cifra desde el primer byte; una base existente **nunca** se migra |
| `init_wal` | `src/storage/engine/init.rs` | El WAL se abre con el contexto del motor (`ShardedWal::new_encrypted`), no con el suyo |
| `recover_state` | `src/storage/engine/init.rs` | El lector del WAL usa la **misma** DEK (`WalReader::open_with_key`) |
| `hnsw_context` | `src/storage/engine/init.rs` | El contexto de **escriba** de la imagen HNSW se resuelve en el open (fuera de todo lock — ADR D5) |
| `process_decision` | `src/crypto.rs` | La decision del motor es **autoritativa** para los artefactos que resuelven su propio contexto (el backend, la imagen HNSW) |

**Base cifrada sin clave** → `Error::EncryptionKeyRequired` **antes** de abrir
el backend (cierra `FIND-516`: el backend ya no puede devolver el ciphertext
como si fuera plano).

### 1.2 Un registro por proceso (`src/crypto/registry.rs`)

`KeyRegistry` paso de ser un valor a ser un **handle** sobre un estado
compartido (`Arc<Mutex<RegistryState>>`), con una **tabla por proceso** de
registros abiertos (path → `Weak`):

* los cuatro artefactos que abren el registro de la misma base obtienen el
  **mismo** estado en memoria → **un** `persist` y **ningun** lost update
  (cierra `FIND-514`: antes un `persist` desde una copia obsoleta borraba las
  entradas de los demas);
* `KeyRegistry::rotate(new_material)` **re-envuelve** las DEKs bajo una KEK
  nueva (subiendo el `key_version`) y deja `encryption.registry.bak` para el
  rollback manual;
* `KeyRegistry::artifacts()` expone el **registro de modo** de la base.

### 1.3 Rotacion (`src/cli_handlers/encryption.rs`, `src/cli.rs`)

```powershell
vanta-cli encryption rotate --db ./mi-db --new-key-file C:\keys\vanta-new.key
```

Re-envuelve las DEKs (no re-deriva: **ningun** ciphertext se invalida,
`FIND-194`), sube el `key_version` y deja el registro anterior en
`encryption.registry.bak`. La base debe estar **cerrada**.

### 1.4 El snapshot copia el registro (`src/storage/engine/mod.rs`)

`encryption.registry` se anade a `is_mutable_after_snapshot`: el snapshot lo
**copia**, no lo hard-linkea (un hard link dejaria que la escritura posterior
del motor —el nuevo epoch de nonce— mutara el snapshot por el inode
compartido).

### 1.5 Documentacion honesta (la "enmenda hacia delante" de ADR-0057 R-12)

| Documento | Que dice ahora |
|---|---|
| `SECURITY.md` | La capacidad **real** (los 4 artefactos + texto + snapshots, opt-in) y los **limites declarados**: las claves del backend y los tokens del indice de texto en claro, el tamano de los valores filtrado, la existencia/numero/orden observables, la master key en memoria, **perder el registro pierde la base**, sin migracion in-place, wal-shipping |
| `docs/user/operations/CONFIGURATION.md` | Las **3 fuentes** de clave (env → fichero → comando) con su precedencia, sus reglas y la feature `encryption` |
| `docs/user/operations/security/KEY_RUNBOOK.md` (nuevo) | Donde se guarda la master key, como se rota y se deshace, que se pierde si se pierde el registro, el procedimiento para cifrar una base existente y el diagnostico |
| `docs/dev/architecture/STORAGE_VERSIONING.md` | El inventario de versiones **actualizado** (WAL v4, VantaFile v3, imagen HNSW v9, registro v1) — estaba obsoleto desde W2 |
| `docs/api/STORAGE_FORMATS.md` (nuevo) | Las constantes publicas (`VECTOR_INDEX_ENCRYPTED_VERSION`, `VECTOR_INDEX_MAX_READABLE_VERSION`, `VFILE_*`, `WAL_*`), los envelopes y la matriz de compatibilidad |

---

## 2. Tests

`src/storage/engine/tests/encryption.rs` (13 tests) y
`tests/encryption_canary.rs` (el probe de canario, el mismo criterio que uso
el probe de `FIND-249`).

| Test | Que fija |
|---|---|
| `engine_resolves_one_context_for_the_four_artifacts` | Un registro, cuatro entradas (el lost update de `FIND-514`) |
| `encrypted_db_without_key_fails_with_key_required` | `EncryptionKeyRequired`, nunca "corrupto" |
| `encrypted_db_with_key_encrypts_the_four_artifacts` | Los 4 artefactos cifrados y el payload en ninguna parte en claro |
| `plaintext_db_with_key_opens_plaintext_and_warns` | Sin migracion implicita, con aviso |
| `registry_is_included_in_the_snapshot_mirror` | El registro viaja en el snapshot |
| `registry_is_copied_not_hardlinked_by_the_snapshot` | Copia, no hard link |
| `encryption_rotate_preserves_readability_of_pre_rotation_data` | `FIND-194`: la data previa sigue legible con la key nueva |
| `encryption_rotate_leaves_a_bak_registry` | El rollback manual existe |
| `registry_keeps_the_four_entries_after_two_rotations` | La rotacion no pierde artefactos |
| `snapshot_of_an_encrypted_db_is_restorable_with_the_key` | El snapshot se abre con la key |
| `snapshot_of_an_encrypted_db_is_refused_without_the_key` | Se refuses sin key |
| `security_md_claim_matches_the_implemented_scope` | Gate de doc: el claim real |
| `no_document_still_says_not_encrypted_at_rest_yet` | Gate de doc: la claim inversa tampoco vale |
| `encrypted_database_has_no_plaintext_canary_on_disk` | El probe de canario (6 superficies) |

---

## 3. Dos fixes fuera del scope declarado (W5)

W6 ha tenido que tocar **dos** ficheros de las waves anteriores (W5). Los dos
son **imprescindibles**: sin ellos la feature no funciona.

### 3.1 `src/index/serialize/file.rs` — el contexto de lectura no se cachea

El camino de lectura de la imagen cacheaba el contexto **de lectura** en el
slot del indice. Con el wiring de W6, ese slot se reutiliza en la
**escritura** → el primer flush de una base cifrada fallaba con *"the hnsw
encryption context is read-only: it cannot seal an image"*.

**Fix (1 linea):** el camino de lectura **no** cachea el contexto de lectura;
el camino de escritura resuelve el suyo (con la misma DEK: el registro la
tiene). Sin este fix, **ninguna** base cifrada podia hacer flush.

### 3.2 `src/index/port_impl.rs` — el contexto de escritura se instala en el open

`open_index_port` recibe ahora el contexto de **escriba** de la imagen y lo
instala en el slot del indice. Asi el epoch de la imagen se asigna (y el
registro se escribe) **fuera de todo lock del motor** (ADR-0057 D5): sin
ello, la imagen resolveria su contexto en su primer write, que ocurre bajo
el `insert_lock`.

---

## 4. Que queda abierto

| # | Hueco | Impacto | Donde |
|---|---|---|---|
| 1 | **La escritura del registro bajo el `insert_lock` (ADR D5) queda cerrada para el caso normal**: el motor resuelve el contexto de **escriba** de la imagen HNSW en el open (fuera de todo lock) y lo instala en el slot del indice. Solo queda el **desbordamiento del contador de nonce** (`u32::MAX` cifrados con la misma DEK), que rola el epoch **en el hot path** (dentro del lock del shard / del segmento). | Un `persist` del registro en el hot path, practicamente inalcanzable (2^32 envelopes) | `src/crypto/registry.rs` (`begin_epoch`) — el epoch lo rola quien agota su contador |
| 2 | **El shipper de WAL no puede leer segmentos v4** (`FIND-496`): el replication de una base cifrada no funciona. | La replicacion de bases cifradas no es soportada | `src/wal_shipping.rs` |
| 3 | **El backup/restore no conoce el registro** (`FIND-507`): restaurar una base cifrada sin su registro la deja ilegible. | El backup debe llevar el registro (documentado en el runbook) | `src/cli_handlers/backup.rs` |
| 4 | **La decision por proceso es ambigua con varias bases**: con dos bases abiertas en el mismo proceso, la decision registrada no se aplica y los artefactos que resuelven su propio contexto usan las fuentes. | Multi-base en un proceso con claves distintas: la imagen HNSW no ve la key programatica | `src/crypto.rs` (`process_decision`) |
| 5 | ~~**El coste no esta medido** (W7)~~ → **cerrado por W7 (2026-10-21)**: el p99 cifrado y el coste por imagen **tienen numero**, con comando, entorno y fecha, en `docs/user/operations/BENCHMARKS.md` §22 (nota de la wave: `docs/dev/reviews/2026-10-21-w7-cifrado-coste.md`). El **gate del 15 %** queda **medido pero no certificado** en esta maquina compartida (el suelo de ruido del p99 es del orden del umbral) — y el **cambio de diseno** que lo resolveria (envelope del VantaFile **por pagina** en vez de por nodo + cache de paginas descifradas) quedo **disenado**, no implementado (ADR-0057 §R-3). | El umbral del 15 % del ADR: **medido, no certificable aqui**; el resto de huecos de W6 siguen abiertos | `benches/encrypted_p99.rs`, `benches/wal_throughput.rs`, `docs/user/operations/BENCHMARKS.md` §22 |

---

## 5. Verificacion

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo nextest run -p vantadb --profile audit --lib --features encryption -E 'test(encryption) or test(registry) or test(rotate)'
cargo nextest run -p vantadb --profile audit --lib
cargo clippy -p vantadb --all-targets -- -D warnings
cargo clippy -p vantadb --features encryption --all-targets -- -D warnings
cargo fmt --check
```

La salida literal de la corrida esta en el informe de la wave (el ejecutor lo
commitea).

### 5.1 Continuacion: W7 (el coste del cifrado, medido)

W6 dejo abierto el **hueco 5** de §4 ("el coste no esta medido"). **W7 lo
cerro con numero** (2026-10-21):

- **Publicacion:** `docs/user/operations/BENCHMARKS.md` **§22** (comandos,
  entorno, fecha y delta de cada cifra) · nota de la wave:
  `docs/dev/reviews/2026-10-21-w7-cifrado-coste.md`.
- **Que midio:** el motor **real** (WAL + VantaFile + backend-KV + imagen HNSW)
  con la feature `encryption` encendida y clave valida, en el regimen **HNSW**
  (25 000 x 1536d) — donde la busqueda **si** descifra; el WAL por frame; el
  `flush` (la imagen HNSW entera) y el `cold start`.
- **Veredicto:** el **p99 de la busqueda se dispara** (el envelope del
  VantaFile es **por nodo**: 1 AEAD + 3 reservas + 3 copias de 6 KB **por
  candidato visitado**, y la cache de descifrado es de **una sola entrada**,
  asi que el recorrido HNSW no acierta nunca) → **senal de diseno**, con el
  cambio propuesto ya disenado en `FIND-598` (envelope **por pagina** + cache
  de paginas descifradas). El **cold start** tambien se ve afectado
  (descifra la imagen y el WAL enteros). El `flush` **no**.
- **Honestidad del numero:** la maquina compartida no permite certificarlo
  (el suelo de ruido del p99 es del orden del propio umbral) → el numero se
  publica **con su delta y sus condiciones**, no como gate de regresion. El
  **gate del 15 %** del plan queda **medido y no certificable aqui**: hace
  falta una maquina dedicada o CI.
- **Benches:** `benches/encrypted_p99.rs` (nuevo, con auto-verificaciones:
  la cabecera del WAL en disco dice v3/v4, y la busqueda tiene que devolver
  hits o la corrida falla) y `benches/wal_throughput.rs` (variante cifrada).
  El bench **vuelca sus numeros a un fichero** (`ENCRYPTED_P99_OUT`) porque
  Rust vacia `stdout` al salir: una corrida muerta se llevaba los numeros
  atras.
