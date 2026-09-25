# FND-15 — Crash recovery / WAL en la práctica (Verificación)

> **Categoría:** Durabilidad · **Backlog:** P20c · **Prioridad:** 🔴 · **Esfuerzo:** 🟡
> **Rol:** vanta-chaos (leaf) · **Fecha:** 2026-08-16 · **Tipo:** tarea de VERIFICACIÓN (sin fixes)
> **Contrato:** "kill a mitad de escritura recupera estado consistente; hallazgos documentados (sin fixes si no hay gap)"

## 1. Objetivo

Verificar que un kill a mitad de escritura recupera estado consistente (WAL replay), usando los
suites existentes `chaos_integrity` y `wal_resilience`. Documentar qué cubren, qué NO cubren, y
proponer plan de test de recovery si hay gap. **No se implementan fixes** (leaf; el lead decide).

## 2. Discovery — tests localizados

| Test | Archivo | Tests | Feature requerida |
|---|---|---|---|
| `chaos_integrity_certification` | `tests/storage/chaos_integrity.rs:14` | Ghost node prevention + tombstone resilience | ninguna |
| `chaos_integrity_failpoints_certification` | `tests/storage/chaos_integrity.rs:124` | 6 escenarios failpoint: `wal_append_fail`, `storage_insert_fail`, `mmap_flush_fail`, `hnsw_serialize_fail`, `edge_write_fail`, `snapshot_serialize_fail` + `assert_recovery()` | `failpoints` |
| `test_wal_durability_and_checkpoint_coherence` | `tests/storage/wal_resilience.rs:18` | Replay salta records ≤ checkpoint_seq; recupera nodo post-flush no volcado | ninguna |
| `test_wal_middle_corruption_auto_healing` | `tests/storage/wal_resilience.rs:80` | Corrupción en medio del WAL → scan-forward auto-healing (salta 202, recupera 201/203/204) | ninguna |
| `test_wal_selective_crc_corruption_recovery` | `tests/storage/wal_resilience.rs:205` | CRC32C corrupto en record 2 → skip selectivo + forward recovery | ninguna |
| `test_sharded_wal_truncated_shard_recovery_fails_closed` | `tests/storage/wal_resilience.rs:320` | Tail truncado (torn write) en shard → reopen **falla closed** (no silencioso) | ninguna |
| `test_wal_write_failure_simulated` | `tests/storage/wal_resilience.rs:385` | Insert falla cuando `wal_append_fail` está activo | `failpoints` (cuerpo) |

Harness: `src/testing/chaos.rs` — `ChaosTestHarness::enable/disable/disable_all` envuelven
`crate::cfg_failpoint` / `crate::remove_failpoint` (crate `fail`). `Drop` llama `disable_all()`.

## 3. Ejecución — comandos y resultados reales

Entorno: Windows 11, rama `develop`, `cargo test` (perfil `test`, debug). Build previo de
`--features failpoints` completado en 2m38s (artefactos reutilizados).

| # | Comando | Resultado |
|---|---|---|
| 1 | `cargo nextest run --profile audit --features failpoints --build-jobs 2 -E 'binary(chaos_integrity) or binary(wal_resilience)'` | ❌ **0 tests run** — ambos binarios excluidos por `default-filter` del profile audit (ver hallazgo FND-15-02) |
| 2 | `cargo test --features failpoints --test chaos_integrity --test wal_resilience` (paralelo default) | ❌ chaos 1/2 FAILED: `chaos_integrity_certification` panic en línea 35 con `IoError: "Simulated Storage insert catastrophic I/O failure"` — failpoint de OTRO test |
| 3 | `cargo test --features failpoints --test chaos_integrity chaos_integrity_certification -- --test-threads=1` | ✅ ok |
| 4 | `cargo test --features failpoints --test chaos_integrity -- --test-threads=1` | ✅ 2/2 ok |
| 5 | `cargo test --features failpoints --test wal_resilience` (paralelo default) | ❌ **4/5 FAILED** con `"Simulated WAL append catastrophic I/O failure"` en los inserts de los tests hermanos |
| 6 | `cargo test --features failpoints --test wal_resilience -- --test-threads=1` | ✅ 5/5 ok |
| 7 | `cargo test --features failpoints --test chaos_integrity --test wal_resilience --no-fail-fast` (paralelo default) | ⚠️ chaos 2/2 ok; wal **3/5 FAILED** (subconjunto DISTINTO al run 5: `middle_corruption`, `checkpoint_coherence`, `sharded`) — flaky |

### 3.1 Veredicto de fondo (el invariante que pide FND-15)

**El motor SÍ recupera estado consistente.** Con ejecución serial (que es como corre CI), los 7/7
tests pasan y verifican:

- WAL replay salta records con seq ≤ `checkpoint_seq` (evita duplicación) y recupera el nodo
  post-flush no volcado (`checkpoint_coherence`).
- Corrupción de payload a mitad de archivo → scan-forward: salta el record corrupto y recupera
  los sanos anterior y posterior (`middle_corruption`).
- CRC32C corrupto con payload deserializable → detectado y saltado selectivamente, forward
  recovery intacto (`selective_crc`).
- Torn write (truncación de shard) → **fail closed**: reopen devuelve error, nunca éxito
  silencioso con records perdidos (`sharded_truncated`).
- Failpoint de I/O activo → la escritura es rechazada (no corrupta); al removerlo, el engine
  sigue operando y `assert_recovery()` confirma write+read (`chaos_failpoints`).

El gap encontrado NO está en el producto de recuperación: está en el **aislamiento del harness
de tests** (hallazgo FND-15-01).

## 4. Hallazgos

### [ALTA] FND-15-01 — Fuga de estado global de failpoints entre tests del mismo binario (race)

- **Descripción:** `fail::cfg` (crate `fail`) es estado **global del proceso**. Los binarios
  `chaos_integrity` y `wal_resilience` agrupan tests con y sin failpoints. Con el paralelismo
  default de `cargo test`, el test que activa un failpoint (`chaos_integrity_failpoints_certification`
  con `storage_insert_fail`; `test_wal_write_failure_simulated` con `wal_append_fail`) contamina a
  los tests hermanos: sus inserts caen con el error inyectado y los `.unwrap()` paniquean.
- **Input reproductor:** `cargo test --features failpoints --test wal_resilience` (sin `--test-threads=1`).
  Resultados: run 5 → 4/5 FAILED; run 7 → 3/5 FAILED con subconjunto distinto (flaky, scheduling-dependent).
  En chaos: run 2 → 1/2 FAILED; run 7 → 2/2 ok.
- **Backtrace mínimo:** `thread 'test_wal_middle_corruption_auto_healing' panicked at tests\storage\wal_resilience.rs:98:44: called Result::unwrap() on an Err value: IoError(Custom { kind: Other, error: "Simulated WAL append catastrophic I/O failure" })`.
- **Por qué CI no lo sufre:** `heavy-certification-50.yml:126-131` corre los failpoint tests con
  `--test-threads=1` explícito, y el profile nextest `[profile.chaos]` también fija `test-threads = 1`.
  El riesgo es local (dev) y futuro (cualquier test failpoint nuevo hereda el mismo riesgo si no
  respeta la convención).
- **Fix propuesto (trivial, NO implementado — requiere decisión del lead):** la opción más robusta
  y lazy es **separar los tests failpoint en un binario propio** (`tests/storage/chaos_failpoints.rs`
  o similar) para que ningún test sin failpoints comparta proceso con activaciones de failpoints.
  Alternativa mínima: documentar/forzar `--test-threads=1` como requisito del comando.

### [MEDIA] FND-15-02 — `default-filter` del profile nextest audit excluye los tests de caos

- **Descripción:** `cargo nextest run --profile audit` (verify canónico del repo, ver AGENTS.md)
  excluye `binary(chaos_integrity)` y `binary(wal_resilience)` vía `default-filter`
  (`.config/nextest.toml`). El `-E` del CLI se ANDea con el default-filter → **0 tests corren**
  (run 1). La cobertura de caos queda gateada exclusivamente detrás del workflow semanal
  `heavy-certification-50.yml`.
- **Consecuencia:** un dev que corra el verify estándar nunca ejercita estos invariantes de
  durabilidad. Es by design (Fast Gate <5min, determinista), pero vale registrarlo como deuda de
  visibilidad.

### [MEDIA] FND-15-03 — No existe test de kill REAL a mitad de escritura

- **Descripción:** los tests simulan crash con `drop(storage)` + reopen, truncación de archivo,
  corrupción CRC y failpoints — pero **ninguno mata un proceso hijo a mitad de un batch insert** y
  reabre para verificar replay del WAL. El escenario canónico de P20c ("kill a mitad de escritura")
  no está cubierto directamente. Ver plan de test en §5.

### [INFO] FND-15-04 — `test_wal_write_failure_simulated` no verifica recuperación

- A diferencia de `chaos_integrity_failpoints_certification` (que termina con `assert_recovery()`),
  este test solo asserts que el insert falla con failpoint activo. No hace drop+reopen ni verifica
  consistencia del WAL tras la falla. Cobertura débil del lado recovery.

### [INFO] FND-15-05 — No hay failpoint de fsync

- Failpoints existentes: `wal_append_fail`, `storage_insert_fail`, `mmap_flush_fail`,
  `hnsw_serialize_fail`, `edge_write_fail`, `snapshot_serialize_fail`. El escenario "fsync falso"
  (fsync reporta éxito sin persistir) no está simulado.

## 5. Plan de test de recovery propuesto (cubre FND-15-03)

Propuesta para el lead — NO implementada en esta tarea. Nuevo binario `tests/storage/crash_kill_recovery.rs`:

1. **Kill a mitad de escritura (process kill):** el test spawns el mismo binario como child process
   con un modo helper (env var) que abre la DB, inserta un batch de N nodos y es asesinado
   (`std::process::Command` + kill) a mitad del batch — idealmente con un failpoint `wal_append_fail`
   con acción `sleep` activado para fijar el punto de interrupción.
2. **Reopen y verificación:** el proceso padre reabre la DB y asserts:
   - nodos completos presentes;
   - nodo a medias ausente (o truncado) — nunca corrupto;
   - sin panic; engine operativo (write+read post-recovery, equivalente a `assert_recovery`).
3. **Concurrencia + kill:** 64+ threads escribiendo + kill del proceso → el replay debe reconstruir
   estado coherente (invariante del contrato P20c).
4. **Fsync falso (FND-15-05):** failpoint `fsync_fail`/`fsync_short_write` → verificar que el
   engine no reporta éxito sin persistencia (fail closed o recovery correcto).

Criterio de aceptación: el plan se implementa solo si el lead lo aprueba (leaf no implementa).

## 6. Coverage

| Path / invariante | Estado | Observación |
|---|---|---|
| WAL replay + checkpoint_seq bypass | ✅ | `test_wal_durability_and_checkpoint_coherence` |
| Corrupción de payload en medio → scan-forward | ✅ | `test_wal_middle_corruption_auto_healing` |
| CRC32C corrupto → skip selectivo | ✅ | `test_wal_selective_crc_corruption_recovery` |
| Torn write en shard → fail closed | ✅ | `test_sharded_wal_truncated_shard_recovery_fails_closed` |
| Escritura rechazada bajo failpoint I/O + recovery post-remoción | ✅ | `chaos_integrity_failpoints_certification` (6 escenarios + `assert_recovery`) |
| Kill real de proceso a mitad de escritura | ❌ | No cubierto → plan §5 |
| fsync falso / short write | ❌ | No cubierto → plan §5 |
| Concurrencia 64+ writers + crash | ❌ | No cubierto → plan §5 |
| Aislamiento de failpoints entre tests | ❌ | Race FND-15-01 (test infra, no producto) |

## 7. Recommended fixes

1. **FND-15-01:** separar tests failpoint en binario propio (`tests/storage/chaos_failpoints.rs`).
   Alternativa mínima: exigir `--test-threads=1` en el comando documentado. Archivo sugerido:
   `tests/storage/chaos_integrity.rs`, `tests/storage/wal_resilience.rs` (reorganización).
2. **FND-15-03/04/05:** implementar `tests/storage/crash_kill_recovery.rs` según plan §5 — requiere
   aprobación del lead.
3. **FND-15-02:** registrar como deuda de visibilidad (opcional: agregar job de CI/label en el
   profile chaos para correr localmente con `cargo nextest run --profile chaos`).

## 8. Notas de ejecución

- Se omitió `cargo nextest run --profile chaos` (dedicado a `chaos_integrity_failpoints` con
  `test-threads=1`) porque el requisito era verificar el comportamiento real del suite completo;
  `cargo test` con/sin `--test-threads=1` cubre ambos lados del hallazgo.
- Los tests son rápidos (<4s serial); no se omitió variante por tiempo.
- Sin commits (regla leaf); el lead commitea el reporte al cerrar la wave.