# MOD-01 — WAL escrito ANTES de validar → insert/update rechazado resucita datos tras restart

> **Estado:** ⏳ IN PROGRESS · **Appetite:** max 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴
> **Plan:** docs/plans/2026-08-23-backlog-triage.md (Task 1, Wave 1) · NO editar el plan
> **Workflow:** bug-fix · **Fuente:** docs/reviews/modulos/core.md H-1 + cross-modulos F-1
> **Nota:** retry fresco — intento previo murió sin dejar trabajo (task file no existía, worktree limpio verificado)

## Bug

En `InMemoryEngine` (`src/engine.rs`), los mutadores escriben al WAL **antes** de validar:
el registro de una op que el caller recibió como error queda en el WAL, y el replay
(`with_wal`) lo aplica incondicionalmente → corrupción silenciosa tras restart.
Expuesto vía API pública incluido WASM standalone (wasm fuerza `BackendKind::InMemory`,
cross-modulos F-1).

## Root Cause (verificado en fuente hoy, codegraph + lectura completa)

1. `insert()` (engine.rs:221-247): `append_to_wal(Insert)` en :228 **antes** del check
   `contains_key` (:231) → duplicado rechazado deja `WalRecord::Insert(impostor)`.
2. `update()` (engine.rs:260-293): WAL append :266 **antes** del check `!contains_key`
   (:271) → update sobre nodo ausente/eliminado rechazado deja `Update{id,node}`.
3. Replay `with_wal()` (:149-168): aplica `Insert`/`Update` como upsert incondicional
   (`nodes_map.insert`) → resucita payload B sobre A legítimo / nodo eliminado.

## Discovery crítico: TODOS los write paths mapeados

| Path | ¿Viola invariante? | Detalle |
|---|---|---|
| `InMemoryEngine::insert` | 🔴 SÍ | WAL antes del check DuplicateNode |
| `InMemoryEngine::update` | 🔴 SÍ | WAL antes del check NodeNotFound |
| `InMemoryEngine::delete` | 🟡 orden viola invariante pero benigno | Delete de nodo inexistente entra al WAL; replay es idempotente y el orden del log preserva semántica — se corrige igual por invariante único |
| `StorageEngine::insert` (insert.rs:33) + batch/bulk | ✅ NO | Semántica insert-as-upsert por diseño (test `insert_duplicate_overwrites`): no existen rechazos DuplicateNode; guards (memory pressure/read-only/failpoint) corren ANTES del append WAL (:34,:74); si WAL falla no se aplica |
| `StorageEngine` txn path | ✅ NO | Buffer en txn_buffers; WAL solo en commit validado (ERR-013) |

H-1 vive SOLO en el motor legacy `InMemoryEngine` (N-1). No hay paths batch/bulk en él:
sus únicos 3 mutadores son insert/update/delete (leído completo engine.rs:69-481).

## Fix (diseño)

Reordenar a **validate → WAL → apply** bajo UNA sola sección crítica `nodes.write()`.
El double-checked locking sugerido por el reporte queda descartado: entre check-read-lock
y WAL-append otro writer puede colar el id → mismo bug con ventana de carrera, y un WAL
append-only no se puede compensar. La sección crítica única es más simple Y correcta.
Lock discipline intacta: índices ya se mutaban dentro del guard write; nadie toma
wal→nodes en orden inverso (lectores nunca tocan WAL).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/engine.rs` (969L: imports 1-68, struct+replay
  69-193, mutadores 200-307, queries 309-474, tests mod 482-969), `src/wal.rs`
  (WalRecord/WalWriter.append/batch_append), `src/storage/engine/insert.rs` (33-342,
  apply_insert_stats 131-219).
- **Referencias entrantes:** `with_wal` ← tests/core/basic_node.rs; WASM usa InMemory
  backend vía build_config (consumidor del fix, no se toca); 174 callers de
  StorageEngine::insert — NO se toca ese archivo.
- **Referencias salientes:** engine.rs → ShardedWal (append/recover/flush_all),
  EdgeIndex, ScalarIndex — sin cambios de firmas.
- **Tests existentes:** engine.rs tests cubren reject en memoria (:550,:599,:616) pero
  NINGUNO cubre durabilidad post-restart (gap que ocultó H-1).
- **Veredicto:** 1 archivo prod + 1 archivo test (mismo archivo), ~40 líneas netas.
  Sin cambios de API pública ni de formato WAL. Riesgo bajo.

## Contrato (mecánico)

1. Test RED nuevo: insert RECHAZADO (DuplicateNode) → reopen → payload original intacto.
2. Test RED nuevo: update RECHAZADO (NodeNotFound sobre nodo eliminado) → reopen → nodo AUSENTE.
3. Test post-fix obligatorio (pre-mortem): flujo legítimo insert→update→reopen conserva versión actualizada; delete legítimo persiste.
4. `cargo nextest run -p vantadb` completo verde + verify full (fmt/clippy/nextest workspace audit).

## Invariantes (no romper)

1. Upsert legítimo (insert ok / update versionado) sigue funcionando igual.
2. Formato y semántica del WAL sin cambios — solo el ORDEN validate→WAL→apply.
3. Sin unwrap nuevos en código prod; sin unsafe.
4. Durabilidad antes de visibilidad para ops VALIDADAS (WAL primero dentro de la sección crítica, después de validar).

## Steps

### Step 1 ✅ DONE — RED: tests de durabilidad reproducen resurrección
- 4 tests agregados en engine.rs tests mod (tempfile dev-dep ya disponible).
- RED verificado: `rejected_duplicate_insert` FAIL con `left: [9.9, 9.9] right:
  [1.0, 2.0]` (impostor pisó original) · `failed_update_on_deleted_node` FAIL
  ("resurrected via WAL replay") — bug reproducido mecánicamente.
- Flujos legítimos (insert→update→reopen; delete persiste) GREEN pre-fix: 2 passed.

### Step 2 ✅ DONE — GREEN: reordenar validate→WAL→apply en insert/update/delete
- insert(): check DuplicateNode bajo `nodes.write()` ANTES de append_to_wal; WAL
  dentro de la sección crítica (durabilidad antes de visibilidad para ops válidas).
- update(): old_node leído del write guard (sin read-lock previo), check NodeNotFound
  antes del WAL.
- delete(): contains_key → WAL → remove (validación antes de mutar).
- Double-checked descartado documentado en código: ventana TOCTOU no compensable
  en WAL append-only.
- GREEN: test_mod01 4/4 PASS · engine+storage tests 342/342 PASS.

### Step 3 ✅ DONE — Verify full + commit + cierre
- `cargo fmt --check` ✅ · `cargo clippy --workspace --all-targets --all-features -D warnings` ✅
- Contrato: `cargo nextest run -p vantadb` = **2049/2049** PASS (1 skipped preexistente)
- Verify full: `cargo nextest run --profile audit --workspace --build-jobs 2` = **2718/2718** PASS
- `scripts/validate-docs-coverage.ps1` = 0 gaps ✅
- SECURITY: sin deps nuevas (cargo audit N/A) · sin unsafe · errores via Result/VantaError
  (0 unwraps nuevos en prod) · el fix MEJORA la validación en trust boundary de persistencia.
- PERFORMANCE: sin claim (Regla 9). canonical_p99 mide StorageEngine — archivo NO tocado;
  el cambio de orden en el motor legacy es requerido por corrección, efecto throughput
  no afirmado en ninguna dirección.
- Commit: `18fd2c80` `fix(core): MOD-01 valida antes de escribir WAL — insert/update rechazado no resucita datos`
  (pre-commit hooks fmt/clippy/actionlint ok; excluidos del commit: completions/* drift ajeno + plan file).

## Context Save Point

- Tarea COMPLETA — sin trabajo pendiente.
- Deuda relacionada NO introducida (preexistente, candidatos a Backlog): N-1 sugiere
  deprecar InMemoryEngine hacia StorageEngine (~850 líneas, elimina la clase entera);
  replay sigue aplicando Insert/Update como upsert incondicional (defense-in-depth:
  filtrar WALs ya-corrompidos en recovery es cambio de semántica aparte, no hecho).
- Nota: los tests viven en engine::tests (perfil default) — corren en Fast Gate.
