---
title: "FIND-750 - cerrar la CLASE de paths de test derivados de %TEMP%, no la cuarta instancia"
kind: task
status: completed
description: "FIND-735 cerro 13 sitios y FIND-744 uno mas, pero el barrido de clase encuentra 24 coincidencias en 15 ficheros. 6 son escrituras (5 sitios + un duplicado muerto de 596 lineas que nadie compila) y 18 son lecturas o literales de struct: esas quedan justificadas por escrito, no borradas. El gate de cierre es el barrido completo, no un grep por fichero."
tags: [vantadb, tests, isolation, temp-dir, class-sweep]
---

# FIND-750 - cerrar la clase de paths de test derivados de `%TEMP%`

Tercera reincidencia de la misma clase (`FIND-735` 13 sitios, `FIND-744` 1 sitio).
Cada arreglo por instancia garantiza una cuarta, asi que el contrato de esta tarea
no es "arreglar 8 sitios" sino **dejar el barrido limpio**: `rg` sobre todo `src/`
y `tests/`, y solo las coincidencias justificadas por escrito abajo.

## Contexto

`FIND-735` midio el sintoma real: un path de `%TEMP%` con nombre literal o por PID
no colisiona solo cuando los corredores se solapan, **tambien deja residuo** (la
cuarentena `.corrupt` hermana, las DBs a medio escribir) que el recolector de
`FIND-732` no alcanzaba y que hoy barre tras `FIND-743`. La decision de aquella
tarea, revisada cuerpo por cuerpo, fue `tempfile::TempDir` para los 13 casos porque
**ninguno lee el path despues del `Drop`**.

Ese criterio se re-valida aca sitio por sitio en vez de aplicarse por patron.

## El barrido (numero real, no el del ticket)

```
rg -n 'std::env::temp_dir|"/tmp/|env::temp_dir' src/ tests/
```
→ **24 coincidencias en 15 ficheros** (el ticket nombraba 8: las lineas se movieron
y el patron encontraba mas hermanos, incluido un duplicado muerto).

| # | Sitio | Escribe? | Disposicion |
|---|---|---|---|
| 1 | `src/server/cli_server_auth_tests.rs:416` | **si** | `TempDir` |
| 2 | `src/cli_server_auth_tests.rs:402` | **si** (si se compilara) | **borrado**: duplicado muerto |
| 3 | `tests/wal_rollback.rs:27` | **si** | `TempDir` (helper devuelve `(TempDir, PathBuf)`) |
| 4 | `src/node/vector_data.rs:447` | **si** | `TempDir` |
| 5 | `src/sdk/version_history.rs:724` | **si** | `TempDir` |
| 6 | `src/sdk/version_history.rs:745` | **si** (solo el cleanup) | absorbido por el `TempDir` |
| 7 | `src/config.rs:3249` | no | **justificado**: lee `temp_dir()` y afirma que un path absoluto se devuelve tal cual (`resolve_local_model_path` corta en `is_absolute()` antes de tocar el FS) |
| 8 | `src/embedding_health.rs:385` | no | **justificado**: `no_such_dir()` es una **etiqueta** que se interpola en el mensaje; `local_health(..., model_present=false)` no sondea el FS, y el nombre debe **no** existir |
| 9 | `src/config.rs:2592` | no | **justificado**: `"/tmp/vanta"` es un valor de `Config` y la asercion lo compara; no hay I/O |
| 10 | `src/config.rs:2593` | no | idem (la asercion) |
| 11 | `src/migration.rs:768` | no | **justificado**: `MigrationEngine::new` solo guarda el `PathBuf` (`migration.rs:185-190`); no crea el dir |
| 12 | `src/migration.rs:769` | no | idem (aserción sobre `.path()`) |
| 13 | `src/migration.rs:774` | no | idem (`test_dry_run_flag`) |
| 14 | `src/sdk/api/namespaces.rs:384` | **si** (doctest) | `TempDir` en linea oculta (`# `) — **corregido tras el review** |
| 15 | `src/sdk/builder.rs:138` | **si** (doctest) | `TempDir` en linea oculta (`# `) — **corregido tras el review** |
| 16 | `src/sdk/types/search.rs:307` | no | **justificado**: literal de campo (`index_path`) en un struct de datos |
| 17 | `src/sdk/types/search.rs:504` | no | idem |
| 18 | `src/sdk/serialization/conversions.rs:164` | no | idem (`PathBuf::from("/tmp/index.hnsw")`) |
| 19 | `src/sdk/types/record.rs:832` | no | idem (`path: "/tmp/export.jsonl"`) |
| 20 | `src/sdk/types/record.rs:1097` | no | idem |
| 21 | `tests/sdk_serialization.rs:596` | no | idem |
| 22 | `tests/sdk_serialization.rs:629` | no | idem |
| 23 | `src/storage/engine/tests/types.rs:116` | no | **justificado**: literal de `IndexRebuildReport.index_path`; struct de datos, cero I/O |
| 24 | `src/storage/engine/tests/types.rs:125` | no | idem (la asercion) |

Ninguna de las 16 "no escribe" se borra: son fixtures de string o aserciones sobre
valores. Borrarlas solo cambiaria el texto de un test sin cambiar lo que mide.
(Los casos 14-15 estaban mal clasificados como "no escribe": los cazo el review
adversarial y se corrigieron — ver §Review.)

## Decision por test (no por patron)

**`src/server/cli_server_auth_tests.rs` (`auth_events_recorded_in_audit_log`).**
El test escribe un `.jsonl`, hace dos requests y **lee el log al final** para
afirmar los 2 eventos y que no se filtren secretos. `TempDir` es correcto porque la
lectura ocurre **antes** del `Drop`. El nombre por PID se va: dos corridas del mismo
filtro en procesos distintos ya no comparten fichero por construccion.

**`tests/wal_rollback.rs`.** El helper devolvia un `PathBuf` con PID + seq + `rand`,
es decir ya era unico por ejecucion — **el riesgo no era la colision, era el
residuo**: `WalWriter::open` crea un **fichero**, `remove_file` del final se salta
en un panic, y tres tests comparten el helper. Ahora devuelve
`(tempfile::TempDir, PathBuf)`: el guard es el primer local y cae ultimo, mismo
patron que `FIND-744`. Se retira la maquinaria `SEQ`/`rand` que el `TempDir` hace
innecesaria.

**`src/node/vector_data.rs` (`test_mmap_full_as_f32_slice_roundtrip`).** Escribe
3 `f32`, mapea el fichero, suelta el `Arc<Mmap>` y borra. No lee el path despues:
`TempDir` y adios al `remove_file` best-effort.

**`src/sdk/version_history.rs` (`versions_backend_partition_roundtrip`).** DB Fjall
en `%TEMP%` con nombre por PID, `remove_dir_all` antes y despues del `close()`. No
se inspecciona el path despues del `Drop` → `TempDir` + `db.close()` + `drop(db)` +
`tmp.close().expect(...)`, para que el no-residuo sea **falsable** (el `Drop` se
traga los errores) sin volverse flaky en Windows por un handle de Fjall vivo.

**`src/sdk/api/namespaces.rs` y `src/sdk/builder.rs` (doctests).** Los dos ejemplos
de API **se ejecutan** (`cargo test --doc`) y abrian una DB Fjall real en `%TEMP%`
con nombre por PID: `vantadb-ns-stats-doctest-<pid>` y `vantadb-open-example-<pid>`.
`namespace_stats` ademas borraba el dir **antes** de soltar la DB, asi que un panic
filtraba para siempre. Se pasan a `TempDir` en una **linea oculta** (`# `), que no
aparece en los docs renderizados: el ejemplo que lee el usuario no cambia, pero el
que compila el CI usa un dir efimero y cierra la DB antes de soltarlo.

**`src/cli_server_auth_tests.rs` — duplicado muerto.** Ver §Incognitas. Se borra en
vez de arreglarse: nadie lo compila.

## Impacto mapeado (Regla 0)

Leidos completos antes de editar: `src/server/cli_server_auth_tests.rs` (708 L),
`src/cli_server_auth_tests.rs` (596 L), `tests/wal_rollback.rs` (238 L),
`src/node/vector_data.rs` (474 L), `src/sdk/version_history.rs` (764 L),
`src/audit.rs:190-249` (quien mantiene el handle del log), `src/wal.rs:676-706`
(`WalWriter::open` crea el fichero), `src/migration.rs:183-200`,
`src/config.rs:620-642`, `src/embedding_health.rs:155-200`.

| Archivo | Referencias hacia dentro | Referencias hacia afuera | Veredicto |
|---|---|---|---|
| `src/server/cli_server_auth_tests.rs` | `src/server/routing.rs:58` (`#[path] mod auth_tests`) — **unico** consumidor | `crate::audit`, `server::{router,state,middleware}`, `sdk::Embedded` | tocar solo el cuerpo del test; compila bajo `cfg(test)` |
| `src/cli_server_auth_tests.rs` | **ninguna** `mod`/`#[path]`/`include!` en el repo (rg sobre `*.rs`/`*.toml`) | idem | **huerfano**; borrarlo no cambia el build |
| `tests/wal_rollback.rs` | target de integracion propio | `vantadb::wal::{WalWriter,WalReader}` | helper local, 3 call sites |
| `src/node/vector_data.rs` | `node/mod.rs` | `memmap2`, `Arc` | solo `mod tests` |
| `src/sdk/version_history.rs` | `sdk/` | `Embedded::open_with_config`, `BackendKind::Fjall` | solo `mod tests` |

Riesgo real declarado: **falso verde**. Un `TempDir` puesto donde el test inspecciona
el path despues del `Drop` haria que `read_to_string` de un path borrado fallara (no
pasaria en silencio), pero un `TempDir` que se dropea **antes** de la lectura si
seria un falso verde por path vacio. En los 4 tests la lectura ocurre con el guard
vivo, y el guard es el primer local.

## Steps

1. ✅ `src/server/cli_server_auth_tests.rs` → `TempDir`.
2. ✅ Borrar `src/cli_server_auth_tests.rs` (duplicado muerto).
3. ✅ `tests/wal_rollback.rs` → helper `(TempDir, PathBuf)`.
4. ✅ `src/node/vector_data.rs` → `TempDir`.
5. ✅ `src/sdk/version_history.rs` → `TempDir` + `close()` falsable.
6. ✅ Verify del contrato (barrido + nextest + collector + 2 corridas concurrentes).

## SDP

`campaign_discover_skills_v2 phase=BUILD archivosClave="src/,tests/"` →
`campaign-executor`, `progreso`, `ponytail`, **pinned** `test-driven-development`,
**pinned** `systematic-debugging`, `source-driven-development`,
`doubt-driven-development`, `incremental-implementation`.

## Incognitas

- **Las dos copias de `cli_server_auth_tests.rs` son duplicado muerto, no dos
  modulos.** `src/server/routing.rs:57-59` declara `#[path = "cli_server_auth_tests.rs"]
  mod auth_tests;` **relativo a `src/server/`**, asi que el fichero vivo es
  `src/server/cli_server_auth_tests.rs`. `src/cli_server_auth_tests.rs` no lo declara
  nadie: es la copia que quedo atras en el split `cli_server.rs -> server/*` (fechada
  2026-10-19 contra 2026-11-02) y es un **subconjunto estricto**: sus 28 funciones
  existen todas en el vivo, que ademas tiene los 6 tests SRV-06 de JWT. Sus tests
  **nunca corren**. Se borra; arreglarle el `%TEMP%` habria sido mantener un fichero
  que no mide nada.
- El barrido no caza `Path::new("/tmp/...")` sin `env::temp_dir` en otras formas
  (ej. `C:\tmp` en Windows), que es exactamente el hueco que el review de `FIND-735`
  encontro en `schema.rs:272`. Se corrio `rg '"/tmp/'` como patron, asi que esa
  forma si entra; lo que no entra es un path relativo (`"target/tmp"`) o un
  `Path::new("tmp")`. No hay ninguno en `src/`+`tests/` hoy.
- **El barrido es un blanco movil mientras hay carriles R en vuelo.** La primera
  pasada (inicio de esta tarea) devolvio **24** coincidencias y no incluia
  `src/hardware/mod.rs`; al cerrar devuelve **18** e incluye dos lineas nuevas
  (`hardware/mod.rs:833,834`, `env_value(Some(OsString::from("/tmp/p")))`) que
  escribio la sesion de `FIND-738` — fichero prohibido para esta tarea. Son
  fixtures de string pasados a una funcion pura (`utils::env_value`), cero I/O, asi
  que caen en la clase "no escribe"; se justifican aca y **no se tocan** para no
  pisar un WIP ajeno. Cualquier re-medicion del contrato debe correrse con el
  arbol quieto.
- **El `-E` del contrato original no medía lo que decia.** `test(wal_rollback)`,
  `test(vector_data)`, `test(version_history)` y `test(embedding_health)` no
  matchean ningun nombre de test: nextest compara contra el nombre, no contra el
  fichero/binario. Los tests de `src/server/cli_server_auth_tests.rs` ademas solo
  existen con `--features server`. Es la misma clase de "gate que no mide" que el
  plan viene cazando; se corrio igual para el registro y ademas el filtro real.

## Verify (medido, no auto-reporte)

**Barrido final** (`rg -n 'std::env::temp_dir|"/tmp/|env::temp_dir' src/ tests/`):
**18 coincidencias en 10 ficheros** — las 16 justificadas de la tabla +
`src/hardware/mod.rs:833,834` (ver §Incognitas: las escribio la sesion de `FIND-738`
mientras esta tarea corria; fixtures de string sin I/O, fichero prohibido).

**Tests** — el `-E` del contrato **no selecciona los cuatro tests que se tocan**
(nextest matchea el *nombre* del test, y ninguno se llama `wal_rollback`,
`version_history`, `vector_data` ni `embedding_health`). Se corrio tal cual, y
ademas el filtro corregido:

| Comando | Resultado |
|---|---|
| `-E 'test(auth) or test(wal_rollback) or ...'` (contrato literal) | 109/109 ✅ |
| `-E 'test(auth) or test(wal_v2) or test(mmap_full) or test(versions_backend) or test(f697_)'` | 16/16 ✅ (incluye `wal_v2_*` ×4, `test_mmap_full_as_f32_slice_roundtrip`, `versions_backend_partition_roundtrip`, `f697_relative_local_model_path_...`) |
| `-E 'test(wal_v2) or test(mmap_full) or test(versions_backend) or test(f697_)'` (post-fix del review) | 8/8 ✅ |
| `--features server -E 'test(auth_tests) + test(events_recorded)'` | 22/22 ✅ — **sin `--features server` los tests de auth no existen**: `src/server/routing.rs:57` es `#[cfg(test)]` dentro de un modulo feature-gated |
| 2 corridas **concurrentes** del filtro corregido (default) | 16/16 y 16/16, residuo 0 |
| 2 corridas **concurrentes** del filtro de auth (`--features server`) | 22/22 y 22/22, residuo 0 |
| `cargo test --doc -p vantadb --features server namespace_stats` | 1/1 ✅ |
| `cargo test --doc -p vantadb --features server 'Embedded::open'` | 3/3 ✅ |

**Residuo** (`%TEMP%`, tras las corridas): `vanta-auth-test-*` 0 ·
`vantadb-versions-test-*` 0 · `vanta_review15_mmap_*` 0 ·
`vantadb_test_wal_rollback_*` 0 · `vantadb-open-example-*` 0 ·
`vantadb-ns-stats-doctest-*` 0 · `.tmp*` 0.

**Recolector:** `pwsh -NoProfile -File dev-tools/temp-collector.ps1 -DryRun` →
`dirs: 0 carpeta(s), 0 B` · `files: 0 fichero(s), 0 B` · exit 0.

**fmt:** `cargo fmt --check` exit 0. **clippy:**
`cargo clippy -p vantadb --all-targets --features server -- -D warnings` exit 0
(compila tambien el WIP ajeno de `FIND-738`, asi que no oculta nada).
**OCR:** `ocr-review.ps1` exit 0; rule group
`project / src/{engine.rs,node/**,config.rs,error.rs,lib.rs}` aplica a
`vector_data.rs` — el diff es solo `mod tests`, no agrega `unsafe` ni toca
produccion.

## Review (P2-01, agente distinto — `vanta-review`, contexto fresco)

**Veredicto inicial: `CHANGES-REQUIRED`** (1 Medium, 3 Low/Nit). El Medium era real y
se corrigio; los Low se cerraron sin cambio de codigo (justificados) salvo uno.

| # | Hallazgo | Sev | Disposicion |
|---|---|---|---|
| 1 | `src/sdk/api/namespaces.rs:384` y `src/sdk/builder.rs:138` no eran "ejemplos de doc-comment": son **doctests que ejecutan** y abren una DB Fjall en `%TEMP%` con nombre por PID (`namespace_stats` ademas borraba el dir antes de soltar la DB → fuga permanente ante panic) | Medium | **corregido**: `TempDir` en linea oculta (`# `) en ambos; `cargo test --doc` 1/1 y 3/3 |
| 2 | `version_history.rs`: `tmp.close()` corria con `db` en scope → flake posible en Windows | Low | **corregido**: `drop(db)` explicito antes del `close()` |
| 3 | `TempDir` se asigna tambien en la iteracion `InMemory` y no se usa | Low | **aceptado**: crear y borrar un dir vacio por corrida; envolverlo en `Option` seria mas codigo que el problema |
| 4 | Mojibake en el task file | Nit | revisado: el fichero es UTF-8 valido; el `?` que vio el reviewer es el render de la consola |

**Respuestas del reviewer (verificadas por el, no auto-reporte):** borrar
`src/cli_server_auth_tests.rs` es seguro (unico `#[path]` resuelve a `src/server/`;
sin `include!`, sin `[[test]]`, sin referencia en CI; lib test target reconstruido
tras el borrado) · ninguno de los 4 `TempDir` es falso verde (la lectura ocurre con
el guard vivo y un `Drop` prematuro **panica**, no pasa en silencio) · el barrido no
deja ningun path de escritura fuera (`env::var("TEMP"/"TMP"/"TMPDIR")`,
`dirs::temp_dir`, literales `"tmp"`/`"target/tmp"`, `C:\tmp`: 0 hits).

**Leccion de metodo:** `cargo nextest` **no corre doctests**, asi que el gate de
tests no veia esas dos escrituras; el unico gate que las ve es el barrido. Por eso
el contrato de esta tarea es el `rg` y no el runner.

**Confirmacion (misma ronda):** `VERDICT: APPROVE`. El revisor re-verifico por su
cuenta: `versions_backend_partition_roundtrip` PASA con el `drop(db)`, los dos
doctests 1/1 y 3/3, barrido 18/10 y retiro el Nit del mojibake (era el codepage de
su consola; el fichero decodifica UTF-8 estricto con 0 `U+FFFD`).

## Cierre de fila

- Fila `FIND-750` retirada de `docs/dev/Backlog.md` junto con su seccion
  (`## Media 2026-11-26 - Ocho sitios mas con %TEMP% en tests`), que quedaba con una
  sola fila (819 → 813 lineas).
- Entrada agregada al final de `docs/dev/avance/activo/core-engine.md`.
- `gen-index.mjs` **no** se corrio: se regenera en el cierre de wave.
- **Desvio registrado:** el `pre-commit` bloqueo el primer intento por
  `cargo fmt --check` sobre `vanta-memory/tests/tmp_repro_747.rs`, archivo de **otro
  carril** (area prohibida para esta tarea). Se uso el escape que el propio hook
  documenta para ese caso (`SKIP=cargo-fmt`), no `--no-verify`; `clippy`,
  `actionlint` y el gate de area staged pasaron sin skip. El fmt de los 7 archivos
  de esta tarea ya estaba en 0 (`cargo fmt --check` exit 0 antes).
