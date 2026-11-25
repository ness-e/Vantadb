---
title: "FIND-735 — aislar los tests que escribían en un path fijo bajo %TEMP%"
kind: task
status: completed
description: "Los tests de schema/WAL/serialize derivaban su path de %TEMP% con nombre literal o PID, no de un directorio único por ejecución: colisionan entre corridas concurrentes y dejan el residuo de cuarentena de WAL fuera del árbol que el test borra"
tags: [vantadb, tests, isolation, temp]
---

# FIND-735 — aislar los tests que escribían en un path fijo bajo `%TEMP%`

## Contexto

Detectado al cerrar `FIND-732` (`264940d57`). `tempfile::TempDir` limpia en `Drop`, pero:

- un path **literal** (`vanta_schema_test`) es compartido por cualquier corrida concurrente;
- un path con **PID** (`vantadb_ser_test_<pid>`) colisiona cuando el SO recicla el PID de una corrida que crasheó;
- un path con **nonce aleatorio** (`vanta_test_wal_healing_<rand>`) no colisiona, pero el `remove_file(&dir)` del final **no borra los hermanos** que el propio test crea (`<dir>.corrupt` de cuarentena, `<dir>.shard0.corrupt`), y quedan en `%TEMP%` para siempre. El recolector (`dev-tools/temp-collector.ps1`) sólo barre **directorios**, así que estos ficheros sueltos no los ve: mitigarlos por nombre no es arreglarlos.

Medido en `%TEMP%` antes del fix (ficheros sueltos, sin recolector que los toque):

```
vanta_test_wal_healing_1710600387.corrupt        63 B
vanta_test_wal_healing_1735558581.corrupt        63 B
vanta_test_wal_healing_2691421785.corrupt        63 B
vanta_test_sharded_1146638014.shard0.corrupt     54 B   (wal_sharded.rs — fuera de scope)
```

## Alcance

Escritura permitida: `src/schema.rs`, `src/wal.rs`, `src/index/serialize/file.rs` (**sólo los módulos `#[cfg(test)]`**) + este task file.

Los 5 sitios que nombró el ticket, más los 4 hermanos de la misma clase que `rg` encuentra en los mismos tres ficheros (arreglar sólo el del ticket deja cuatro vivos — pre-mortem (c)):

| # | Sitio | Patrón actual | Caso |
|---|---|---|---|
| 1 | `src/schema.rs:260` | `join("vanta_schema_test")` — **literal** | (a) efímero |
| 2 | `src/wal.rs:2396` | `xtest_wal_rt_<rand>` | (a) efímero |
| 3 | `src/wal.rs:2443` | `vanta_test_wal_never_<rand>` | (a) efímero |
| 4 | `src/wal.rs:2444` | `vanta_test_wal_never_ctrl_<rand>` | (a) efímero |
| 5 | `src/wal.rs:2489` | `vanta_test_wal_prepare_<rand>` | (a) efímero |
| 6 | `src/wal.rs:2531` | `vanta_test_wal_batch_fmt_<rand>` | (a) efímero |
| 7 | `src/wal.rs:2607` | `vanta_test_wal_mismatch_<rand>` | (a) efímero |
| 8 | `src/wal.rs:2643` | `vanta_test_wal_healing_<rand>` | (a) efímero — **deja `.corrupt`** |
| 9 | `src/wal.rs:2699` | `vanta_test_wal_quarantine_<rand>` | (a) efímero — **deja `.corrupt`** |
| 10 | `src/index/serialize/file.rs:435` | `vantadb_ser_test_<pid>` | (a) efímero |
| 11 | `src/index/serialize/file.rs:475` | `join("vantadb__nonexistent__index.bin")` — **literal** | (a) efímero |
| 12 | `src/index/serialize/file.rs:482` | `vantadb_ser_test_corrupt_<pid>` | (a) efímero |

## Spec — caso (a) vs caso (b), decidido por test (no por ticket)

La instrucción del ticket distingue dos casos y prohíbe aplicar el mismo patrón a ambos. Verificado test por test leyendo cada cuerpo completo:

- **(a) el test quiere un dir limpio y efímero** → `tempfile::TempDir`. `Drop` borra el **árbol entero**, así que los hermanos que el writer crea (`<wal>.corrupt`) caen dentro y desaparecen con él. Ese es exactamente el residuo que `remove_file` dejaba atrás.
- **(b) el test quiere un nombre único que sobreviva al `Drop`** → sufijo PID+nonce con cleanup explícito.

**Veredicto: los 13 sitios son (a). Cero (b).** Evidencia por test (lo que el ticket marcó como riesgo de falso verde):

| Test | Última lectura del path | ¿post-`Drop`? |
|---|---|---|
| `file_write_read_roundtrip` | `read_from(&path)` + `assert_eq` — dentro del cuerpo | no |
| `test_wal_roundtrip` | `replay_all` + asserts — dentro del cuerpo | no |
| `test_sync_mode_never_skips_auto_sync` | `records_since_sync` — dentro del cuerpo | no |
| `test_wal_v2_prepare_roundtrip_unit` | `match &records[3]` — dentro del cuerpo | no |
| `test_batch_append_byte_format_matches_append` | `std::fs::read` + `replay_all` — dentro del cuerpo | no |
| `test_wal_version_mismatch` | `WalReader::open` devuelve `Err` — dentro del cuerpo | no |
| `test_wal_auto_healing_and_recovery` | `replay_all` + `records[4]` — dentro del cuerpo | no |
| `test_corrupt_wal_tail_is_quarantined` | `assert!(backup.exists())` + `replay_all` — dentro del cuerpo | no |
| `persist_and_load_roundtrip` / `load_nonexistent_file_returns_none` / `load_corrupt_file_returns_none` | `load_from_file` + asserts — dentro del cuerpo | no |

El único que **parece** (b) es `test_corrupt_wal_tail_is_quarantined`, porque inspecciona el backup de cuarentena: pero inspecciona **dentro** del test, antes de que el `TempDir` se dropee. Su semántica (write → corromper cola → reopen → el backup existe) queda idéntica; sólo cambia el prefijo del path. **Ningún test inspecciona nada después del retorno de la función** → no hay falso verde posible.

Regla de orden de `Drop` (docs.rs `TempDir` §Resource Leaking): un handle abierto impide borrar en Windows y el destructor **ignora el error**. Por eso `TempDir` se declara como **primera** sentencia de cada test (los locales caen en orden inverso: el `TempDir` cae último, después de writers y readers).

Fuente: <https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html> — "Once the `TempDir` value is dropped, the directory at the path will be deleted, along with any files and directories it contains" / "It's important to ensure that handles … are dropped before the `TempDir` goes out of scope. The `TempDir` destructor will silently ignore any errors".

Patrón ya establecido en el crate: `wal.rs:2755` (`test_auto_rotate_triggers_at_limit`) y `file.rs:459` ya usan `tempfile::tempdir()`. `tempfile` es dev-dependency (`Cargo.toml:248`), no feature-gated.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/schema.rs` (módulo tests), `src/wal.rs:2388-2790` (todo `mod tests`), `src/index/serialize/file.rs:423-490` (`mod tests`), `dev-tools/temp-collector.ps1`, `.opencode/rules/core-engine.md`, `.opencode/rules/durability.md`.
- **Referencias hacia dentro:** ninguna — los 13 sitios viven en `#[cfg(test)]`, sin llamadores fuera del runner.
- **Referencias hacia fuera:** `WalWriter::open` / `WalReader::open` / `StorageHeader::{write_to,read_from}` / `CPIndex::{persist_to_file,load_from_file}` — se les pasa el path, no cambian de firma. `tempfile` (dev-dep ya presente).
- **Veredicto:** cambio confinado a constructores de path de test. Cero impacto en producción, cero cambio de semántica de los asserts.

## Steps

1. ✅ DISCOVERY: medir los sitios, `%TEMP%`, y verificar que la evidencia del ticket sigue viva en `264940d57`.
2. ✅ Spec (a)/(b) por test (§ arriba).
3. ✅ RED: 16 procesos en paralelo sobre `file_write_read_roundtrip` (binario previo al fix) → **33/64 fallos**.
4. ✅ GREEN: 13 sitios a `TempDir` (3 ficheros).
5. ✅ VERIFY: gate negativo = 0 · nextest filtrado 195/195 · diferencial paralelo 33/64 rojo → 0/64 verde · `tmp.close()` verde.
6. ✅ CIERRE: fmt/clippy/docs gates · `ocr` delegate rule · review P2-01 adversarial · commit local.

## RED medido (reproduction)

Binario de test con el código previo, mismo harness que usará el GREEN:
16 procesos × 4 rondas de `file_write_read_roundtrip`:

```
RED schema (nombre literal, 16 procesos x 4 rondas): runs=64 fails=33
thread 'schema::tests::file_write_read_roundtrip' panicked at src\schema.rs:266:61:
called `Option::unwrap()` on a `None` value
```

`src/schema.rs:266` es `StorageHeader::read_from(&path).unwrap().unwrap()`: un proceso hermano
borró/reescribió el directorio compartido entre el `write_to` y el `read_from` de éste. No es
teoría de colisión, es el fallo observable.

Residuo, determinista (1 ejecución del test previo):

```
RESIDUO .corrupt: antes=3 despues=4 delta=+1     # <TEMP>\vanta_test_wal_healing_<rand>.corrupt
```

## GREEN medido

```
GREEN paridad con RED (16 procesos x 4 rondas): runs=64 fails=0
arranque real: 64/64 procesos reportaron '1 passed'

cargo nextest run --profile audit -p vantadb -E 'test(schema) or test(wal) or test(serialize)' --build-jobs 2
  Summary [9.847s] 195 tests run: 195 passed, 2792 skipped

rg -n 'std::env::temp_dir|"/tmp/' src/schema.rs src/wal.rs src/index/serialize/file.rs   → 0 (exit 1)

RESIDUO .corrupt healing: antes=4 despues=4 (delta=0)   # corroboración débil, ver §Verificación final
```

También 64 procesos con los 4 tests que tocan el filesystem (`file_write_read_roundtrip`,
`test_wal_roundtrip`, `test_wal_auto_healing_and_recovery`, `test_corrupt_wal_tail_is_quarantined`)
× 8 en paralelo × 2 rondas → 0 fallos.

## Gates de cierre

- `cargo fmt --check -- <3 ficheros>` ✅
- `cargo clippy -p vantadb --all-targets` ✅ (0 warnings)
- `node scripts/docs/check-docs.mjs` ✅ exit 0 · `check-links.mjs` ✅ exit 0 · `scripts/validate-docs-coverage.ps1` ✅ 0 gaps
- `gen-index --check` **rojo por diseño de wave**: 3 task files nuevos (`FIND-735`, `FIND-731`, `MCP-47`) — dos son de otras sesiones. No se commitean `docs/index.md`/`llms.txt` desde acá para no arrastrar filas ajenas; el orquestador corre `gen-index --write` una vez en el batch (plan §3.c "Batch, no tarea a tarea").
- `ocr delegate rule` ✅ group 1 (durability: `schema.rs`, `wal.rs`) + group 2 (indexes: `file.rs`) — sin Critical/High/Medium: el diff es sólo `#[cfg(test)]`, sin `unsafe`, sin lock, sin hot path.

## Fuera de scope detectado

`src/wal_sharded.rs:999` usa el mismo patrón nonce y deja `vanta_test_sharded_<rand>.shard0.corrupt`
en `%TEMP%` (54 B por corrida, medido 3 veces). Es `src/**` fuera de los 3 ficheros permitidos →
NOTICED BUT NOT TOUCHING. El recolector no lo ve porque sólo barre **directorios**. Candidato a
fila `FIND-*` nueva: `temp-collector.ps1` podría también recoger los `vanta*`/`vantadb*` **ficheros**
sueltos por nombre (los que hoy se escapan).

## Contrato mecánico

El gate original (`rg 'temp_dir\(\)|%TEMP%'`) era inútil por dos motivos, ambos verificados: `temp_dir\(\)` **nunca** puede casar `tempfile::tempdir()` (el idioma no lleva guion bajo), y `%TEMP%` casa la prosa de los comentarios nuevos (3 falsos positivos). Gate corregido, sólo código:

```
rg -n 'std::env::temp_dir|"/tmp/' src/schema.rs src/wal.rs src/index/serialize/file.rs    # esperado: 0 (exit 1)
cargo nextest run --profile audit -p vantadb -E 'test(schema) or test(wal) or test(serialize)' --build-jobs 2
# prueba de concurrencia: el binario de test, N procesos en paralelo, mismo filtro
```

La prueba de comportamiento es el diferencial, no el `rg`: el mismo harness sobre el binario **previo** da 33/64 fallos (`schema.rs:266`); sobre el binario nuevo, 0/64.

## Invariantes (no romper)

- Ningún test de durabilidad puede volverse efímero **respecto de lo que inspecciona**: el path debe vivir lo que el cuerpo del test necesite.
- `test_corrupt_wal_tail_is_quarantined` debe seguir afirmando `backup.exists()` — con el nombre derivado igual (`format!("{}.corrupt", dir.display())`).
- Cero cambios fuera de los módulos `#[cfg(test)]`.

## SDP

`SDP: campaign-executor, progreso, ponytail (base, auto) + deprecation-and-migration (pinned storage/schema) + doubt-driven-development + source-driven-development`; `incremental-implementation` y `test-driven-development` ya vienen desplegadas en el prompt del agente (mismo contenido, §3b/§3c) — no se re-cargan.

## Review

**Ronda 1 — `vanta-review` (adversarial, contexto fresco): 🔴 REQUEST CHANGES** con 1 blocking, 2 high, 5 medium, 2 low. Nada de lo encontrado era un test vuelto falso verde *por el cambio*; el blocking era una omisión.

### Aplicado en la ronda 1

| # | Severidad | Hallazgo | Disposición |
|---|---|---|---|
| 1 | **BLOCKING** | `schema.rs:272` `read_missing_returns_none` seguía con `Path::new("/tmp/nonexistent_schema_file_for_testing")` — misma clase que FIND-735, en el mismo `mod tests`, 10 líneas más abajo. En Windows `/tmp/x` resuelve a `C:\tmp\x`, que **existe** en esta máquina; el verde dependía de que nadie creara ese fichero. Mi `rg` del contrato no lo cazaba porque sólo buscaba `temp_dir()`/`env::temp_dir`. | **Corregido**: `TempDir` + path garantizado ausente. El `rg` del contrato se amplió a `"/tmp/` para que la clase no vuelva a escaparse. |
| 4 | MEDIUM | Comentario copiado en `test_wal_roundtrip` atribuía el `.corrupt` a ese test, que nunca corrompe la cola. | **Corregido**: comentario reescrito para lo que el test hace. |
| 5 | MEDIUM | `file.rs::persist_and_load_roundtrip` declaraba `index` antes del `TempDir` → el árbol caía con el backend vivo. Hoy inocuo (`InMemory`, `use_mmap=false`), pero el día que se cubra el path mmap se re-crea el leak en silencio. | **Corregido**: `TempDir` primero, `index` después (el único de los sitios con el orden invertido). |
| 7 | MEDIUM | `test_batch_append_byte_format_matches_append:2539` descartaba el `Err` del reset; si fallaba, la aserción de framing fallaba con un mensaje que acusaba a `batch_append`. | **Corregido**: `.expect("reset the WAL before the batch phase")`. |
| 8 | MEDIUM | Nada hacía falsable el claim de "no deja residuo": `Drop` traga los errores de borrado, así que un árbol que no se pudiera borrar dejaba la suite verde. | **Corregido**: `tmp.close().expect(...)` al final de `test_corrupt_wal_tail_is_quarantined` (el test que producía el residuo). |

### Registrado como deuda conocida, NO tocado (fuera del alcance de FIND-735)

- **HIGH #2** `test_wal_version_mismatch`: `assert_eq!(expected_version, WAL_MAX_READABLE_VERSION)` es `X == X` (el error se construye con esa misma constante, `wal.rs:259`), y el fixture es un mismatch de **magic** (`b"NOT_A_VALID_MAGIC_BYTES_123456"`), no de versión — el path de `Error::WALVersionMismatch` (`wal.rs:286-292`) sigue sin cubrir. El comentario del test afirma que el fixture "uses v0", lo cual es falso. **Pre-existente**, y arreglarlo cambia qué afirma un test, que es justo lo que esta tarea no puede tocar. **NO está filed**: `FIND-740` y `FIND-741` ya están tomados (`Backlog.md:799,805`). La fila va con un ID libre (≥742) y el texto exacto está en el RESULTADO de la tarea — el Backlog lo escribe el orquestador (plan §3.c). Si no se aplica, la deuda muere con este task file.
  - Nota del review: existe una opción que **no** toca el test existente — añadir un hermano `test_wal_version_mismatch_v0_header` con un header `VWAL`-magic/v0 hecho a mano que afirme `Error::WALVersionMismatch`. Es lógica de test nueva, no un cambio de semántica; no se hizo acá para no salir del alcance de aislamiento.
- **MEDIUM #6** `load_corrupt_file_returns_none` vs `load_nonexistent_file_returns_none`: el review lo **retiró** en la ronda 2 — `std::fs::write(...).unwrap()` garantiza que el fixture existe en el assert, así que la rama "ausente" es inalcanzable; el colapso de errores a `None` es el contrato público documentado (`file.rs:245-249`). Sin acción.
- **LOW #10** el path de cuarentena se re-deriva a mano (`format!("{}.corrupt", …)`) — coincide con `quarantine_backup_path` (`wal.rs:1742-1752`); acoplamiento anotado, no cambiado.

### Contabilidad corregida (LOW #9)

**13 sitios de path** (no 12) sobre **11 funciones de test**:
`schema` 2 · `file.rs` 3 · `wal.rs` 8 paths repartidos en 7 tests (uno de ellos, `test_sync_mode_never_skips_auto_sync`, crea dos WALs). El 13.º (`schema.rs:272`) lo destapó el review.

## Verificación final (post-review)

```
rg -n 'std::env::temp_dir|"/tmp/' src/schema.rs src/wal.rs src/index/serialize/file.rs   → 0 (exit 1)
cargo nextest run --profile audit -p vantadb -E 'test(schema) or test(wal) or test(serialize)' --build-jobs 2
  Summary [13.577s] 195 tests run: 195 passed, 2792 skipped
GREEN schema 16 procesos x 4 rondas: runs=64 fails=0, '1 passed' en 64/64
4 tests de filesystem x 8 en paralelo x 2 rondas: runs=64 fails=0
cargo fmt --check -- <3 ficheros> ✅ · cargo clippy -p vantadb --all-targets ✅ (scoped, no es el gate de CI)
```

**La evidencia del no-leak es `tmp.close().expect(...)`** al final de `test_corrupt_wal_tail_is_quarantined`: `Drop` traga los errores de borrado, así que es la única señal capaz de fallar si el árbol no se puede borrar — en 1 de los 13 sitios, a propósito (representativo). Los deltas de `%TEMP%` que medí antes/después (0) son corroboración débil y no deben citarse como prueba: los nombres nuevos (`healing.wal`, `index.bin`, `.vanta.schema`) no casan el glob `vanta*`, y el residuo ahora caería **dentro** de un `.tmpXXXXXX/`, que es exactamente lo que `close()` vigila.

`cargo clippy -p vantadb --all-targets` es scoped: el gate de CI es `--workspace --all-targets --all-features --locked -- -D warnings` (`ci-rust.yml:149`) y lo corre el batch. Nada feature-gated cambió.

Único `.tmp*` en `%TEMP%`: 1 directorio **vacío**, delta 0 respecto de mis corridas; coincide con un proceso ajeno matado a mitad de suite (el review corrió `cargo test --lib` en paralelo). El recolector lo recoge por la regla "empty ⇒ pérdida nula" pasados 10 min.
