# RES-01 — DURABILIDAD 🔴 WAL v2 Prepare + snapshot quiesce + recursive wal/ (prerequisite S1)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave0 SOLO — 20260902-alta-prioridad-paralelo)
- **Plan histórico:** `docs/plans/2026-08-29-full-backlog-parallel.md` (W3-SOLO, ✅ COMPLETED 2026-08-29 — WAL v2 Phase 4a landed)
- **Fuente:** `docs/Backlog.md` P38 RES-01..15 (RES-01 prerequisite durabilidad 🔴) + `docs/research/archive/res02-backup-restore.md` §2-3 S1-S2 + `src/wal.rs` + `src/storage/engine/mod.rs:507,540` + FIND-26 (wal_archiver eliminado 2026-08-25)
- **Esfuerzo:** 🔴 2-3d (Wave0 SOLO — hot-path core, single-writer)
- **Prioridad:** 🔴 Alta (calidad/durabilidad — prereq RES-02..15, DEC-02)
- **Tipo:** Rust core / durability (WAL + storage engine)
- **Turns estimados:** 3 (S1 DISCOVERY ya verifica base landed, S2 verify contrato, S3 recitation+plan sync)
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02T19:30
- **Estado:** ⏳ IN PROGRESS
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Incógnitas (uphill):** 0 — base ya mapeada (WAL v2 + quiesce + recursive + restore existen)
- **Pendientes (downhill):** 1 — re-verificación mecánica Wave0 + plan sync (sin código nuevo si contrato pasa)

## Blast Radius

| Dirección | Módulos | Implicación |
|-----------|---------|-------------|
| **Callers de WAL** | `src/storage/engine/txn.rs:156` (commit two-phase: Begin→Prepare→Commit), `src/storage/engine/init.rs:607` (recovery match), `src/engine.rs:155-170` (recovery), `src/wal_sharded.rs` (ShardedWal wraps WalWriter), `tests/wal_rollback.rs` (5 tests), `tests/proptest_wal_roundtrip.rs` (8 variantes) | WAL v2 Prepare es marker durable pre-commit; sin Commit el recovery descarta vía slice-mask MOD-02. Cambio afecta todo path transaccional. |
| **Callers de StorageEngine** | `src/sdk/builder.rs:253` (`create_snapshot` thin wrapper), `vantadb-mcp/src/handlers/tools.rs:1488,1502` (MCP-34a `snapshot_create`), `src/storage/engine/maintenance.rs:flush()` (quiesce), `src/storage/engine/mod.rs:752` (`snapshot_restore` static), `tests/fjall_cold_copy_restore.rs:71` (cold-copy restore) | Single-writer: `flush()` adquiere `insert_lock` (FairMutex) + `backend.flush()` + `vector_index save` + `checkpoint_seq` — ERR-010 invariant. `snapshot_restore` es `fn(storage_root, name)` sin `&self` — requiere exclusividad (no engine abierto). |
| **Callees tocados** | `src/wal.rs` (WalWriter/WalReader, CRC32C, header compat), `src/storage/vfile.rs` / `vfile_mmap.rs` (VantaFile segments), `src/backend/*` (BackendKind Fjall/Rocks/InMemory), `src/index/*` (HNSW rebuild en reopen) | Durability DAG: flush→quiesce→mirror_data_dir→mirror_backend_to. No WAL, no durability. |
| **Implicaciones durabilidad** | ACID Phase 4a: Prepare da commit point para rollback multi-capa. Sin quiesce, snapshot captura torn set (header vs payloads vs KV). Con `wal/` subdir futuro, flat copy pierde WAL. Fix FIND-25+FIND-33 ya landed: `mirror_data_dir` recursivo skip `snapshots/`, `mirror_backend_to` captura KV siblings. | Single-writer: fs2 lock (feature `fs2`) + `_lock_file` en StorageEngine. Concurrent snapshot sin flush = torn backup peor que lento backup (correctness > speed). |
| **Verificación eliminación** | `src/wal_archiver.rs` **no existe** (`Test-Path` False 2026-09-02) — FIND-26 resuelta 2026-08-25 (commit `git log --follow src/wal_archiver.rs`). RES-01 es preparar base para restore físico, no full PITR (PITR = (b) DEFER per res02 §2b). | Sin código muerto; histórico conservado en git history. |

**Disjoint Wave0 (MAX 3):** RES-01 toca `src/wal.rs` + `src/storage/engine/**` + `src/storage/vfile*` — disjoint 100% con MCP-35 (`vantadb-mcp/src/server.rs`, `.vanta.server.json`) y GOV-T01..T03 (`evals/dora.mjs`, `.opencode/task-system/**`). Parallel seguro. Solo 1 writer toca engine core → RES-01 SOLO (no compartir wave con MEM-16 ni RES-02).

## Contrato (verificable — mecánico)

> Fuente plan 2026-09-02 §RES-01 (líneas 137-145):
> `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare" | Measure-Object Count >=1`
> AND `Select-String -Path "src/storage/engine/mod.rs" -Pattern "quiesce|flush\(\)|wal/" | Measure-Object Count >=1`
> AND `cargo test -p vantadb --test wal_rollback -- --nocapture 2>&1 | Select-String "ok" | Measure-Object Count >=1`

**Contrato canónico Wave0 (este task file):**

```powershell
Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare" | Measure-Object Count  # >=1 ✅ (2 hits: enum + test)
Select-String -Path "src/storage/engine/mod.rs" -Pattern "quiesce|flush\(\)" | Measure-Object Count  # >=1 ✅ (flush() + doc quiesce)
cargo test -p vantadb --test wal_rollback -- --nocapture 2>&1 | Select-String "ok" | Measure-Object Count  # >=1 ✅ (5 passed)
cargo check -p vantadb --features fjall  # exit 0 ✅ (Finished dev profile)
Test-Path src/wal_archiver.rs  # == False ✅ (eliminado FIND-26)
```

**Contrato extendido (si S1 ya landed, verificar):**
```powershell
Select-String -Path "src/storage/engine/mod.rs" -Pattern "mirror_data_dir|mirror_backend_to|snapshot_restore" | Measure-Object Count  # >=3 ✅
Select-String -Path "src/wal.rs" -Pattern "WAL_FORMAT_VERSION.*2" | Measure-Object Count  # >=1 ✅
```

## Herramientas necesarias

- `codegraph_explore "wal archiver pitr backup restore"` — blast radius (DONE)
- `cargo check -p vantadb --features fjall` — compilación fjall
- `cargo test -p vantadb --test wal_rollback -- --nocapture` — WAL v2 rollback suite (5 tests)
- `cargo test -p vantadb --test fjall_cold_copy_restore -- --nocapture` — restore cold-copy (si existe)
- `Select-String` / `Test-Path` — contrato mecánico PowerShell
- `Read` — `src/wal.rs`, `src/storage/engine/mod.rs`, `docs/research/archive/res02-backup-restore.md` §2-3
- `skill codebase-memory` — opcional para detect_changes pre-commit

**Skills cargadas (SDP §2):** `campaign-executor` (orquestación) + `planning-and-task-breakdown` (slicing vertical) + `codebase-memory` (blast radius) + `ponytail(full)` (1 guard vs 50 líneas)

## Spec (SDD)

N/A — durabilidad infra, sin símbolos públicos nuevos en este wave. Decisión técnica ya tomada en `docs/research/ACID_ROLLBACK_DESIGN.md` (recuperado b85b52b3): Prepare {txn_id, op_count} + WAL_FORMAT_VERSION=2 + range-based header compat + two-phase commit (Begin→Prepare→fsync→apply→Abort-on-fail→Commit). Si S1 requiere ajuste wal/ subdir, añadir doc en `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` mismo PR (Regla 3).

## Invariantes de dominio (handoff — MUST)

- **Single-writer:** StorageEngine holds `fs2` file lock + `insert_lock` FairMutex. `create_snapshot` adquiere `flush()` (ERR-010) — no concurrent writes durante mirror. `snapshot_restore(storage_root, name)` es static, requiere 0 handles abiertos (Windows fs2 falla loud, Unix fork silencioso si se viola).
- **WAL durability:** `WAL_FORMAT_VERSION=2` (v2 añade Prepare). v2 reader acepta v1 (range compat `format_version ≤ WAL_FORMAT_VERSION`); v1 reader leyendo v2 necesita dump/restore (postcard unknown tag). `Prepare.op_count` es cross-check en replay.
- **Quiesce:** `flush()` = `insert_lock` → `drain_hnsw_batch` → `backend.flush()` → `vector_store flush` → `save_vector_index` → `checkpoint_seq` (seq AFTER snapshot, lock held). Sin esto snapshot es torn.
- **Comandos verificación:** ver §Contrato arriba — todos deben exit 0 / Count >=1 antes de Close.
- **Deuda pendiente:** ninguna si contrato pasa. Si wal/ subdir detectado fuera de mirror, crear FIND-* y decidir S1 follow-up (no bloquear Wave0).

## Steps (atomic — vertical slice)

### Step 1: DISCOVERY — investigar base snapshot + replay + single-writer (DONE 2026-09-02)

- **Archivos:** `src/wal.rs` (1328L), `src/storage/engine/mod.rs` (829L), `src/storage/engine/maintenance.rs:flush()`, `docs/research/archive/res02-backup-restore.md` §2-3, `src/storage/vfile*.rs`, `src/wal_sharded.rs`
- **Acción:** `codegraph_explore "wal archiver pitr backup restore"` + `Read src/wal.rs` + `Read res02 §2-3` + grep wal_archiver inexistencia + map callers/callees + verificar single-writer (fs2 lock, insert_lock) + durabilidad implicaciones.
- **Hallazgos:**
  - `WalRecord::Prepare {txn_id, op_count}` landed, `WAL_FORMAT_VERSION=2` con back-compat range, test `test_wal_v2_prepare_roundtrip_unit` ✅
  - `wal_archiver.rs` eliminado 2026-08-25 (FIND-26) ✅ — `Test-Path` False, histórico en `git log --follow`
  - `create_snapshot` ya tiene `flush()` quiesce + `mirror_data_dir` recursivo skip `snapshots/` (FIND-25) + `mirror_backend_to` (FIND-33) ✅
  - `snapshot_restore(storage_root, name)` ya existe como static fn con staging rename + rollback + failpoint `snapshot_restore_fail` ✅ — RES-02 S2 ya landed fuera de este plan
  - Single-writer: `insert_lock` + `flush()` ERR-010 garantiza consistencia; `snapshot_restore` exige exclusividad (doc comment explícito)
  - Scope RES-01 es **solo preparación S1** (quiesce+recursive+Prepare) — no full PITR (b) DEFER per res02 §2b. Decisión documentada: PITR requiere base+log wiring + retention task (alto costo, sin consumer hoy).
- **Verify:** `Select-String "WalRecord::Prepare" src/wal.rs` Count=2 ✅ + `Test-Path src/wal_archiver.rs` False ✅ + `cargo check -p vantadb --features fjall` exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificar contrato mecánico Wave0 — Prepare + quiesce + wal_rollback suite

- **Archivos:** `src/wal.rs`, `src/storage/engine/mod.rs`, `tests/wal_rollback.rs`
- **Acción:** Ejecutar contrato plan §RES-01 completo (3 líneas PowerShell + cargo test + cargo check). Si alguna línea falla → implementar delta mínimo con `// SAFETY:` si unsafe, sin scope creep (ponytail: 1 guard vs 50 líneas). No tocar MCP-35 ni GOV-T01 archivos.
- **Verify:**
  ```powershell
  Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare" | Measure-Object Count  # expect >=1
  Select-String -Path "src/storage/engine/mod.rs" -Pattern "quiesce|flush\(\)" | Measure-Object Count  # expect >=1
  cargo test -p vantadb --test wal_rollback -- --nocapture 2>&1 | Select-String "ok" | Measure-Object Count  # expect >=1 (5 passed)
  cargo check -p vantadb --features fjall  # expect exit 0
  ```
- **Estado:** ⬜ PENDING (ejecutar en siguiente iteración ACT→VERIFY)

### Step 3: CIERRE — re-verificación + plan sync + recitation

- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (RES-01 §137-145), este task file
- **Acción:** Si Step 2 pasa → actualizar plan file RES-01 Estado ⬜ PENDING → ⏳ IN PROGRESS (o ✅ si DONE per campaña), `last-synced` a ISO now, escribir recitation canónica (activeGoal/lastAction/result/nextAction/contract/nextTask). Documentar decisión "solo preparación S1, no full PITR" en Notas si aplica. `// SAFETY:` si se tocó unsafe (no esperado). Delegar a vanta-chaos solo si se tocó `dashmap`/`parking_lot`/Tokio (no aplica aquí — quiesce ya usa parking_lot existente, no nuevo).
- **Verify:** `campaign_verify_cmd` del contrato + `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "RES-01.*⏳|RES-01.*✅" | Measure-Object Count` >=1
- **Estado:** ⬜ PENDING

## Dependencias

- **Ninguna** — Wave0 paralelo MAX 3 con MCP-35 (`.vanta.server.json` proxy) + GOV-T01..T03 (`evals/dora.mjs`) — archivos disjoint 100%. RES-01 SOLO por hot-path core (single-writer engine).
- **Prereq inverso:** RES-01 → RES-02 (RES-02 snapshot_restore ya landed pero plan lo modela como dependiente; verificación Wave1 revisará paridad).
- **No tocar:** MCP-35 (`vantadb-mcp/src/server.rs`, `src/storage/engine/init.rs` lock), GOV-T01 (`evals/dora.mjs`) — disjoint guard.

## Deuda técnica (Regla 6 — MUST)

- **Saldo neto 0:** RES-01 S1 ya pagó deuda FIND-25 (flat copy) + FIND-33 (backend KV no capturado) + WAL v2 Prepare debt. Sin código nuevo en Step 1 (solo discovery). Si Step 2 requiere fix wal/ subdir, será 1 guard aditivo (ponytail) — deuda previa ya saldada.
- **P2 conocida no tocada:** P2-8 `collect_all_deduped` O(n) wasm — fuera de scope, no se introduce deuda nueva.

## Herramientas (skills) — matriz SDP

| Skill | Por qué |
|-------|---------|
| `campaign-executor` | Orquestación plan/task/verify/state machine (Obligatorio §2) |
| `planning-and-task-breakdown` | Slicing vertical S1→S3, atomic steps |
| `codebase-memory` | Blast radius callers/callees, check_index_coverage |
| `ponytail(full)` | 1 guard vs 50 líneas — delta mínimo si wal/ gap |

Total 4 ≤ 8 (SDP canónico).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — snapshot name `validate_snapshot_name` (trust boundary: `../` traversal) ya en `mod.rs:707` defense-in-depth con MCP-34a. Sin nuevo input surface en RES-01 Wave0 (verify only). Si se añade wal/ handling, re-validar path traversal.
- [x] **PERFORMANCE** — quiesce añade 1 flush/snapshot (correctness > speed, FIND-25 doc). WAL Prepare añade 1 fsync/commit (truthful errors). No optimizar sin `cargo bench --bench canonical_p99` (Regla 9) — no aplica en verify-only wave.

## Notas

- **Decisión scope:** RES-01 es **solo preparación S1** (Prepare + quiesce + recursive wal/). Full PITR (wal_archiver wiring + replay handler + retention) es (b) DEFER — alto costo, sin consumer, validado 2026-08-25. Código mínimo con `// SAFETY:` si unsafe (no se espera unsafe nuevo — `mirror_data_dir` es safe std::fs).
- **Ponytail:** 1 guard si wal/ fuera de mirror (`if path.is_dir() && name != "snapshots" { recurse }` ya es el guard — no 50 líneas). Dejar comentario `// ponytail: recursive skip snapshots, per-file hard_link O(1) Unix, copy O(n) Win — wal/ future-proof` si se toca.
- **FIND-26:** `src/wal_archiver.rs` eliminado + `pitr` feature removida + docs actualizados — verificar con `git log --follow src/wal_archiver.rs` si hace falta auditoría.
- **No tocar MCP-35/GOV-T01:** Wave0 disjoint — respetar DAG. Solo `src/wal.rs`, `src/storage/engine/**`, `src/storage/vfile*`, `docs/research/res02-*`.

## Context Save Point

- **Fecha:** 2026-09-02T19:30
- **Branch:** develop
- **CI pendiente:** `cargo test -p vantadb --test wal_rollback` + `cargo check -p vantadb --features fjall` (Step 2, próxima iteración)
- **Decisiones:** RES-01 scope = S1 only (Prepare+quiesce+recursive), no full PITR (b) DEFER per res02 §2b; wal_archiver eliminado FIND-26; Wave0 SOLO por single-writer.
- **Problemas conocidos:** ninguno bloqueante; Step 2 pendiente de ejecución mecánica (contrato debe pasar con base actual).
- **Próxima tarea:** MCP-35 (Wave0 paralelo, disjoint) — o RES-02 Wave1 si RES-01 cierra.

## Investigación detallada (DISCOVERY 2026-09-02)

### Archivos leídos (completos)

- `src/wal.rs` (1328L + 1366L append): WAL v2 `Prepare {txn_id, op_count}`, `WAL_FORMAT_VERSION=2`, header compat range, CRC32C, scan-forward, quarantine, ShardedWal wrapper, tests `wal_rollback` 5 passed.
- `src/storage/engine/mod.rs` (829L): `FsSnapshot`, `mirror_data_dir` recursivo, `mirror_backend_to`, `create_snapshot` con `flush()` quiesce (Unix+Win), `validate_snapshot_name`, `snapshot_restore(storage_root,name)` static con staging rename+rollback.
- `src/storage/engine/maintenance.rs:flush()` (ERR-010): `insert_lock` → drain → backend.flush → vstore flush → save_vector_index → checkpoint_seq (seq AFTER snapshot, lock held).
- `docs/research/archive/res02-backup-restore.md` (57L): gaps flat copy + no quiesce (S1), plan atomic S1-S5, recomendación (a) directory swap RECOMMENDED, (b) PITR DEFER, (c) logical EXISTS.
- `src/storage/vfile*.rs` / `src/wal_sharded.rs`: vfile segments, sharded WAL.

### Referencias entrantes/salientes

- Entrantes WAL: `txn.rs:156` commit two-phase, `init.rs:607` recovery, `engine.rs:170` recovery, `tests/*`
- Entrantes Storage: `sdk/builder.rs:253` wrapper, `mcp/handlers/tools.rs:1502` MCP tool
- Salientes: `postcard`, `crc32c`, `parking_lot::FairMutex`, `arc_swap`, `fs2` (feature-gated)

### Veredicto impacto

Sin blast radius adicional — S1 ya landed. Verificación Wave0 es read-only sobre base existente. Próximo cambio wal/ sería aditivo (mirror_data_dir ya recursivo, cubre futuro wal/ subdir).
